use super::*;

struct Fixture {
    directory: tempfile::TempDir,
    state: AppState,
    actor: Actor,
    workspace: Workspace,
    headers: HeaderMap,
}

impl Fixture {
    async fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let state = create_state(
            directory.path().into(),
            "http://{workspace}.localhost:8089".into(),
        )
        .await
        .unwrap();
        let mut store = state.store.lock().await;
        let lab = store.db.labs[0].id.clone();
        let secret = token();
        let node = Node {
            id: id(),
            lab_id: lab.clone(),
            name: "Test node".into(),
            platform: "Darwin".into(),
            arch: "aarch64".into(),
            cpus: 4,
            memory_mb: 4096,
            cpu_usage: 0.,
            memory_used_mb: 0,
            last_seen: now() - 60,
            docker: true,
            revoked: false,
            credential_hash: hash(&secret),
            gpus: vec![],
            metrics: None,
            history: vec![],
        };
        let workspace = Workspace {
            id: id(),
            lab_id: lab.clone(),
            node_id: node.id.clone(),
            name: "Test lab".into(),
            template: "jupyter".into(),
            cpus: 1,
            memory_mb: 512,
            status: "stopping".into(),
            created_at: now(),
            last_used: now(),
            error: String::new(),
            network: false,
            gpu_ids: vec![],
            metrics: None,
            history: vec![],
        };
        store.db.nodes.push(node);
        store.db.workspaces.push(workspace.clone());
        drop(store);
        let mut headers = HeaderMap::new();
        headers.insert("authorization", format!("Bearer {secret}").parse().unwrap());
        Self {
            directory,
            state,
            actor: Actor {
                role: "operator".into(),
                lab_id: Some(lab),
                session_hash: String::new(),
            },
            workspace,
            headers,
        }
    }

    async fn action(&self, action: &str) -> Result<Json<Value>> {
        workspace_action(
            State(self.state.clone()),
            Extension(self.actor.clone()),
            Path(self.workspace.id.clone()),
            Json(Action {
                action: action.into(),
                command: String::new(),
            }),
        )
        .await
    }

    async fn poll(&self, ready: bool, docker: bool) -> Value {
        agent_poll(
            State(self.state.clone()),
            self.headers.clone(),
            Json(Heartbeat {
                cpu_usage: 0.,
                memory_used_mb: 0,
                docker,
                ready,
                containers: [(self.workspace.id.clone(), "running".into())].into(),
                metrics: None,
                workspace_metrics: Default::default(),
            }),
        )
        .await
        .unwrap()
        .0["job"]
            .clone()
    }

    async fn complete(&self, job: &str, ok: bool) {
        let response = agent_complete(
            State(self.state.clone()),
            self.headers.clone(),
            Path(job.into()),
            Json(Completion {
                ok,
                output: String::new(),
                error: if ok {
                    String::new()
                } else {
                    "Docker operation failed".into()
                },
            }),
        )
        .await
        .unwrap();
        assert_eq!(response.0["ok"], true);
    }
}

#[tokio::test]
async fn offline_removal_supersedes_queued_work_and_survives_restart() {
    for action in ["create", "start", "stop", "exec"] {
        let mut f = Fixture::new().await;
        let previous = f
            .state
            .store
            .lock()
            .await
            .db
            .enqueue(f.workspace.clone(), action, "");
        let removal = f.action("delete").await.unwrap().0["id"]
            .as_str()
            .unwrap()
            .to_string();
        assert_eq!(f.action("delete").await.unwrap().0["id"], removal);
        {
            let mut store = f.state.store.lock().await;
            assert_eq!(store.db.jobs.len(), 2);
            assert_eq!(store.db.jobs[0].status, "failed");
            assert!(store.db.jobs[0].error.contains("Cancelled"));
            assert_eq!(store.db.workspaces[0].status, "deleting");
            store.db.jobs[1].created_at = now() - 86401;
            store.db.prune();
            store.save().unwrap();
        }
        f.state = create_state(f.directory.path().into(), f.state.app_url.clone())
            .await
            .unwrap();
        assert_eq!(
            f.state.store.lock().await.db.workspaces[0].status,
            "deleting"
        );
        // Docker must be reachable before a missing-container delete is trusted.
        assert!(f.poll(true, false).await.is_null());
        assert_eq!(f.poll(true, true).await["id"], removal);
        f.complete(&previous.id, true).await;
        assert_eq!(
            f.state.store.lock().await.db.workspaces[0].status,
            "deleting"
        );
        f.complete(&removal, true).await;
        assert!(f.state.store.lock().await.db.workspaces.is_empty());
        // A delayed receipt cannot recreate the removed workspace.
        f.complete(&previous.id, true).await;
        assert!(f.state.store.lock().await.db.workspaces.is_empty());
    }
}

#[tokio::test]
async fn removal_waits_for_leased_work_and_keeps_its_status_after_completion() {
    for action in ["create", "start", "stop", "exec"] {
        for ok in [false, true] {
            let f = Fixture::new().await;
            let previous = {
                let mut store = f.state.store.lock().await;
                let j = store.db.enqueue(f.workspace.clone(), action, "");
                store.db.jobs[0].status = "leased".into();
                store.db.jobs[0].lease_until = now() + 900;
                j
            };
            let removal = f.action("delete").await.unwrap().0["id"]
                .as_str()
                .unwrap()
                .to_string();
            assert!(f.poll(false, true).await.is_null());
            // A newly restarted agent must not overlap a still-valid lease.
            assert!(f.poll(true, true).await.is_null());
            f.complete(&previous.id, ok).await;
            {
                let store = f.state.store.lock().await;
                assert_eq!(store.db.workspaces[0].status, "deleting");
                assert!(store.db.workspaces[0].error.is_empty());
            }
            assert_eq!(f.poll(true, true).await["id"], removal);
            f.complete(&removal, true).await;
            assert!(f.state.store.lock().await.db.workspaces.is_empty());
        }
    }
}

#[tokio::test]
async fn expired_leases_and_failed_removal_can_recover() {
    for action in ["create", "stop", "exec"] {
        let f = Fixture::new().await;
        let previous = {
            let mut store = f.state.store.lock().await;
            let j = store.db.enqueue(f.workspace.clone(), action, "");
            store.db.jobs[0].status = "leased".into();
            store.db.jobs[0].lease_until = now() - 1;
            j
        };
        let removal = f.action("delete").await.unwrap().0["id"]
            .as_str()
            .unwrap()
            .to_string();
        let leased = f.poll(true, true).await;
        if action == "exec" {
            assert_eq!(leased["id"], removal);
            assert_eq!(f.state.store.lock().await.db.jobs[0].status, "failed");
        } else {
            assert_eq!(leased["id"], previous.id);
            f.complete(&previous.id, true).await;
            assert_eq!(f.poll(true, true).await["id"], removal);
        }
        f.complete(&removal, false).await;
        assert_eq!(f.state.store.lock().await.db.workspaces[0].status, "error");
        let retry = f.action("delete").await.unwrap().0["id"]
            .as_str()
            .unwrap()
            .to_string();
        assert_ne!(retry, removal);
        assert!(f.state.store.lock().await.db.workspaces[0].error.is_empty());
        assert_eq!(f.poll(true, true).await["id"], retry);
        f.complete(&retry, true).await;
        assert!(f.state.store.lock().await.db.workspaces.is_empty());
    }
}

#[tokio::test]
async fn offline_removal_preserves_permissions_and_other_action_guards() {
    let mut f = Fixture::new().await;
    for action in ["start", "stop", "exec"] {
        assert!(f.action(action).await.unwrap_err().1.contains("offline"));
    }
    f.actor.role = "viewer".into();
    assert_eq!(
        f.action("delete").await.unwrap_err().0,
        StatusCode::FORBIDDEN
    );
    f.actor.role = "operator".into();
    f.actor.lab_id = Some(id());
    assert_eq!(
        f.action("delete").await.unwrap_err().0,
        StatusCode::FORBIDDEN
    );
    f.actor.lab_id = Some(f.workspace.lab_id.clone());
    {
        let mut store = f.state.store.lock().await;
        store.db.nodes[0].last_seen = now();
        store.db.enqueue(f.workspace.clone(), "stop", "");
    }
    assert!(f
        .action("start")
        .await
        .unwrap_err()
        .1
        .contains("in progress"));
    f.state.store.lock().await.db.nodes[0].revoked = true;
    assert!(f.action("delete").await.unwrap_err().1.contains("revoked"));
    assert_eq!(f.state.store.lock().await.db.jobs.len(), 1);
}

#[tokio::test]
async fn forgetting_a_revoked_node_workspace_only_removes_its_records() {
    for missing_node in [false, true] {
        let mut f = Fixture::new().await;
        assert!(f
            .action("forget")
            .await
            .unwrap_err()
            .1
            .contains("still enrolled"));
        let other_workspace = Workspace {
            id: id(),
            ..f.workspace.clone()
        };
        {
            let mut store = f.state.store.lock().await;
            store.db.nodes[0].revoked = true;
            if missing_node {
                store.db.nodes.clear();
            }
            store.db.enqueue(f.workspace.clone(), "stop", "");
            store.db.enqueue(f.workspace.clone(), "delete", "");
            store.db.jobs[0].status = "leased".into();
            store.db.workspaces.push(other_workspace.clone());
            store.db.enqueue(other_workspace.clone(), "stop", "");
            for w in [&f.workspace, &other_workspace] {
                store.db.app_sessions.push(AppSession {
                    token_hash: hash(&token()),
                    workspace_id: w.id.clone(),
                    session_hash: String::new(),
                    expires_at: now() + 60,
                    ticket: false,
                });
            }
        }
        f.actor.role = "viewer".into();
        assert_eq!(
            f.action("forget").await.unwrap_err().0,
            StatusCode::FORBIDDEN
        );
        f.actor.role = "operator".into();
        f.actor.lab_id = Some(id());
        assert_eq!(
            f.action("forget").await.unwrap_err().0,
            StatusCode::FORBIDDEN
        );
        f.actor.lab_id = Some(f.workspace.lab_id.clone());
        assert_eq!(f.action("forget").await.unwrap().0["removed"], true);
        let restored = Store::open(f.directory.path()).unwrap();
        assert_eq!(restored.db.workspaces.len(), 1);
        assert_eq!(restored.db.workspaces[0].id, other_workspace.id);
        assert_eq!(restored.db.jobs.len(), 3);
        assert_eq!(restored.db.jobs[0].status, "failed");
        assert_eq!(restored.db.jobs[1].status, "failed");
        assert_eq!(restored.db.jobs[2].status, "queued");
        assert_eq!(restored.db.app_sessions.len(), 1);
        assert_eq!(restored.db.app_sessions[0].workspace_id, other_workspace.id);
        assert!(restored
            .db
            .events
            .last()
            .unwrap()
            .message
            .contains("any container and volume remain"));
        assert!(restored.db.nodes.iter().all(|n| n.revoked));
    }
}
