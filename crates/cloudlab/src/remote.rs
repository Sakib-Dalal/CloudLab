//! Explicitly configured HTTPS origins and a bounded Caddy certificate allowlist.
use crate::{model::valid_id, server::AppState};
use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;

pub async fn configure(state: &mut AppState, public_url: &str) -> anyhow::Result<()> {
    let url = crate::agent::validate_coordinator(public_url)?;
    anyhow::ensure!(
        url.scheme() == "https",
        "Managed cloud access requires HTTPS"
    );
    let app = url::Url::parse(&state.app_url.replace("{workspace}", "w-test"))?;
    anyhow::ensure!(
        app.scheme() == "https",
        "Managed cloud workspaces require HTTPS"
    );
    let public_url = url.as_str().trim_end_matches('/').to_string();
    let mut store = state.store.lock().await;
    store.db.settings.public_url = public_url.clone();
    store.save()?;
    state.remote_public_url = Some(public_url);
    Ok(())
}

#[derive(Deserialize)]
pub struct CertificateRequest {
    domain: String,
}

/// Called only by the reverse proxy. No caller-supplied URL is fetched. This is
/// disabled on unmanaged installations and never authorizes arbitrary suffixes.
pub async fn allow_certificate(
    State(state): State<AppState>,
    Query(request): Query<CertificateRequest>,
) -> StatusCode {
    let Some(public_url) = &state.remote_public_url else {
        return StatusCode::FORBIDDEN;
    };
    if request.domain.len() > 253 || request.domain.is_empty() {
        return StatusCode::FORBIDDEN;
    }
    let domain = request.domain.to_ascii_lowercase();
    let matches = |origin: &str| {
        url::Url::parse(origin)
            .ok()
            .is_some_and(|u| u.host_str() == Some(&domain))
    };
    if matches(public_url) {
        return StatusCode::OK;
    }
    let Some(id) = domain
        .split('.')
        .next()
        .and_then(|label| label.strip_prefix("w-"))
        .filter(|id| valid_id(id))
    else {
        return StatusCode::FORBIDDEN;
    };
    if !matches(&state.app_url.replace("{workspace}", &format!("w-{id}"))) {
        return StatusCode::FORBIDDEN;
    }
    let store = state.store.lock().await;
    // Stopped workspaces stay eligible for renewal; deleted workspaces do not.
    if store
        .db
        .workspaces
        .iter()
        .any(|w| w.id == id && matches!(w.template.as_str(), "jupyter" | "code"))
    {
        StatusCode::OK
    } else {
        StatusCode::FORBIDDEN
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::*,
        server::{create_state, router},
    };
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn certificates_are_limited_to_managed_registered_origins() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = create_state(
            dir.path().into(),
            "https://{workspace}.8-8-8-8.sslip.io".into(),
        )
        .await
        .unwrap();
        let allowed = |state: AppState, domain: String| async move {
            router(state, dir_path())
                .oneshot(
                    Request::builder()
                        .uri(format!("/api/tls/allow?domain={domain}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap()
                .status()
        };
        assert_eq!(
            allowed(state.clone(), "lab.8-8-8-8.sslip.io".into()).await,
            StatusCode::FORBIDDEN
        );
        assert!(configure(&mut state, "http://127.0.0.1").await.is_err());
        configure(&mut state, "https://lab.8-8-8-8.sslip.io/")
            .await
            .unwrap();
        assert_eq!(
            state.store.lock().await.db.settings.public_url,
            "https://lab.8-8-8-8.sslip.io"
        );
        let id = id();
        state.store.lock().await.db.workspaces.push(Workspace {
            id: id.clone(),
            lab_id: crate::model::id(),
            node_id: crate::model::id(),
            name: "Notebook".into(),
            template: "jupyter".into(),
            cpus: 1,
            memory_mb: 1024,
            status: "stopped".into(),
            created_at: now(),
            last_used: now(),
            error: String::new(),
            network: false,
            gpu_ids: vec![],
            metrics: None,
            history: vec![],
        });
        for domain in [
            "lab.8-8-8-8.sslip.io".into(),
            format!("w-{id}.8-8-8-8.sslip.io"),
        ] {
            assert_eq!(allowed(state.clone(), domain).await, StatusCode::OK);
        }
        for domain in [
            "attacker.example".into(),
            "unknown.8-8-8-8.sslip.io".into(),
            format!("w-{}.8-8-8-8.sslip.io", crate::model::id()),
            format!("w-{id}.8-8-8-8.sslip.io.attacker.example"),
            "lab.8-8-8-8.sslip.io:443".into(),
            "https://lab.8-8-8-8.sslip.io".into(),
        ] {
            assert_eq!(allowed(state.clone(), domain).await, StatusCode::FORBIDDEN);
        }
        state.store.lock().await.db.settings.public_url = "https://attacker.example".into();
        assert_eq!(
            allowed(state.clone(), "attacker.example".into()).await,
            StatusCode::FORBIDDEN
        );
        state.store.lock().await.db.workspaces.clear();
        assert_eq!(
            allowed(state, format!("w-{id}.8-8-8-8.sslip.io")).await,
            StatusCode::FORBIDDEN
        );
    }

    fn dir_path() -> std::path::PathBuf {
        std::path::PathBuf::from("dist")
    }
}
