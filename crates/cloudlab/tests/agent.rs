use cloudlab::model::{id, token, write_private};
use std::{path::Path, process::Stdio, time::Duration};
use tokio::{
    process::{Child, Command},
    time::timeout,
};

fn command(dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cloudlab"));
    command
        .env_remove("CLOUDLAB_COORDINATOR")
        .env_remove("CLOUDLAB_ENROLLMENT")
        .env("CLOUDLAB_AGENT_DIR", dir)
        // Exercise self-thread detection even on single-core Linux runners.
        .env("TOKIO_WORKER_THREADS", "2")
        .kill_on_drop(true);
    command
}

fn pairing(dir: &Path) -> String {
    std::fs::create_dir_all(dir).unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let key = token();
    let credentials = serde_json::json!({
        "id":id(), "token":key,
        "coordinator":format!("http://{}", listener.local_addr().unwrap())
    });
    write_private(
        &dir.join("credentials.json"),
        &serde_json::to_vec(&credentials).unwrap(),
    )
    .unwrap();
    key
}

async fn start(dir: &Path) -> Child {
    let mut child = command(dir)
        .args(["agent", "start"])
        .stdout(Stdio::null())
        .stderr(std::fs::File::create(dir.join("agent-test.log")).unwrap())
        .spawn()
        .unwrap();
    timeout(Duration::from_secs(10), async {
        while !dir.join("agent-run").exists() {
            assert!(
                child.try_wait().unwrap().is_none(),
                "agent exited before startup: {}",
                std::fs::read_to_string(dir.join("agent-test.log")).unwrap()
            );
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
    })
    .await
    .unwrap();
    child
}

async fn success(dir: &Path, args: &[&str]) -> String {
    let result = timeout(Duration::from_secs(15), command(dir).args(args).output())
        .await
        .unwrap()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).unwrap()
}

#[tokio::test]
async fn stop_is_scoped_keeps_pairing_and_allows_restart() {
    let root = tempfile::tempdir().unwrap();
    let a = root.path().join("node a");
    let b = root.path().join("node b");
    let secret = pairing(&a);
    pairing(&b);
    let original = std::fs::read(a.join("credentials.json")).unwrap();
    let mut first = start(&a).await;
    let mut second = start(&b).await;
    let status = success(&a, &["agent", "status"]).await;
    assert!(status.contains("running") && !status.contains(&secret));
    let json = success(&a, &["agent", "status", "--json"]).await;
    assert!(!json.contains(&secret));
    let status: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(status["process"], "running");
    assert_eq!(status["enrollment"], "saved");
    assert_eq!(status["scope"], "local");
    let duplicate = command(&a).args(["agent", "start"]).output().await.unwrap();
    assert!(!duplicate.status.success());
    assert!(String::from_utf8_lossy(&duplicate.stderr).contains("already using"));
    success(&a, &["agent", "stop"]).await;
    assert!(timeout(Duration::from_secs(5), first.wait())
        .await
        .unwrap()
        .unwrap()
        .success());
    assert!(second.try_wait().unwrap().is_none());
    assert_eq!(std::fs::read(a.join("credentials.json")).unwrap(), original);
    assert!(success(&a, &["agent", "status"]).await.contains("stopped"));
    // A stale request from an earlier process must not stop a new generation.
    write_private(&a.join("agent-stop"), token().as_bytes()).unwrap();
    let mut restarted = start(&a).await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(restarted.try_wait().unwrap().is_none());
    success(&a, &["agent", "stop", "--data-dir", a.to_str().unwrap()]).await;
    assert!(timeout(Duration::from_secs(5), restarted.wait())
        .await
        .unwrap()
        .unwrap()
        .success());
    success(&b, &["agent", "stop"]).await;
    assert!(timeout(Duration::from_secs(5), second.wait())
        .await
        .unwrap()
        .unwrap()
        .success());
}

#[tokio::test]
async fn local_delete_stops_agent_and_only_removes_pairing_files() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("node");
    pairing(&dir);
    let receipt = dir.join(format!("receipt-{}.json", id()));
    std::fs::write(&receipt, "{}").unwrap();
    std::fs::write(dir.join("notes.txt"), "keep me").unwrap();
    std::fs::write(dir.join("receipt-not-an-id.json"), "keep me too").unwrap();
    let mut child = start(&dir).await;
    success(&dir, &["agent", "delete", "--local-only"]).await;
    assert!(timeout(Duration::from_secs(5), child.wait())
        .await
        .unwrap()
        .unwrap()
        .success());
    assert!(!dir.join("credentials.json").exists());
    assert!(!receipt.exists());
    assert_eq!(
        std::fs::read_to_string(dir.join("notes.txt")).unwrap(),
        "keep me"
    );
    assert!(dir.join("receipt-not-an-id.json").exists());
    assert!(success(&dir, &["agent", "status"])
        .await
        .contains("not paired"));
    success(&dir, &["agent", "delete", "--local-only"]).await;
}

#[cfg(unix)]
#[tokio::test]
async fn live_agent_without_lock_still_blocks_start_and_delete() {
    let root = tempfile::tempdir().unwrap();
    pairing(root.path());
    let original = std::fs::read(root.path().join("credentials.json")).unwrap();
    let mut child = start(root.path()).await;
    // Simulate a legacy agent that runs without holding the current lock file.
    // This is confined to the test's temporary folder and child process.
    std::fs::remove_file(root.path().join("agent.lock")).unwrap();
    for args in [
        &["agent", "start"][..],
        &["agent", "delete", "--local-only"][..],
    ] {
        let result = timeout(
            Duration::from_secs(15),
            command(root.path()).args(args).output(),
        )
        .await
        .unwrap()
        .unwrap();
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(!result.status.success(), "{args:?} unexpectedly succeeded");
        assert!(
            error.contains("An older agent may still be using"),
            "{error}"
        );
        assert!(child.try_wait().unwrap().is_none());
        assert_eq!(
            std::fs::read(root.path().join("credentials.json")).unwrap(),
            original
        );
    }
    child.kill().await.unwrap();
    child.wait().await.unwrap();
    success(root.path(), &["agent", "delete", "--local-only"]).await;
}

#[tokio::test]
async fn remote_failure_keeps_credentials_and_unpaired_help_is_actionable() {
    let root = tempfile::tempdir().unwrap();
    pairing(root.path());
    let original = std::fs::read(root.path().join("credentials.json")).unwrap();
    let result = command(root.path())
        .args(["agent", "delete"])
        .output()
        .await
        .unwrap();
    assert!(!result.status.success());
    assert_eq!(
        std::fs::read(root.path().join("credentials.json")).unwrap(),
        original
    );
    assert!(String::from_utf8_lossy(&result.stderr).contains("--local-only"));
    let result = command(root.path())
        .args(["agent", "--enrollment", "fresh-key"])
        .output()
        .await
        .unwrap();
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("cloudlab agent delete"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let untouched = root.path().join("not-created");
    success(&untouched, &["agent", "stop"]).await;
    success(&untouched, &["agent", "status"]).await;
    assert!(!untouched.exists());
}

#[tokio::test]
async fn command_help_separates_startup_flags_and_bare_cli_has_no_side_effects() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("not-created");
    assert!(success(&dir, &[]).await.contains("GET STARTED"));
    for subcommand in ["stop", "delete", "status"] {
        let help = success(&dir, &["agent", subcommand, "--help"]).await;
        assert!(help.contains("--data-dir"));
        assert!(!help.contains("--enrollment") && !help.contains("--allow-network"));
    }
    let help = command(&dir)
        .args(["agent", "start", "--help"])
        .env("CLOUDLAB_ENROLLMENT", "never-show-this-secret")
        .output()
        .await
        .unwrap();
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    assert!(help.contains("--enrollment") && !help.contains("never-show-this-secret"));
    let status: serde_json::Value =
        serde_json::from_str(&success(&dir, &["agent", "status", "--json"]).await).unwrap();
    assert_eq!(status["enrollment"], "not_paired");
    assert_eq!(status["node_id"], serde_json::Value::Null);
    assert!(!dir.exists());
}
