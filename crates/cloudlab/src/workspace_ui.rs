//! Small, self-contained workspace chrome served on the container's own origin.
//! It has no coordinator privileges and never needs the lab access key.
use crate::{
    model::{now, Node, Workspace},
    server::ApiError,
};
use axum::{
    http::{HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde_json::json;

fn headers(mut response: Response) -> Response {
    let h = response.headers_mut();
    h.insert("cache-control", HeaderValue::from_static("no-store"));
    h.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    h.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    h.insert("x-frame-options", HeaderValue::from_static("SAMEORIGIN"));
    h.insert("content-security-policy", HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; frame-src 'self'; frame-ancestors 'self'; base-uri 'none'; form-action 'none'"));
    response
}
pub fn asset(path: &str) -> Option<Response> {
    let (kind, body) = match path {
        "/_cloudlab/workspace.css" => (
            "text/css; charset=utf-8",
            include_str!("../assets/workspace.css"),
        ),
        "/_cloudlab/workspace.js" => (
            "text/javascript; charset=utf-8",
            include_str!("../assets/workspace.js"),
        ),
        "/_cloudlab/favicon.svg" => ("image/svg+xml", include_str!("../../../public/favicon.svg")),
        _ => return None,
    };
    Some(headers(([("content-type", kind)], body).into_response()))
}
pub fn shell() -> Response {
    headers(Html(include_str!("../assets/workspace.html")).into_response())
}
pub fn metadata(w: &Workspace, node: &Node, lab: &str) -> Response {
    headers(
        Json(json!({
            "name": w.name, "template": w.template, "cpus": w.cpus,
            "memory_mb": w.memory_mb, "network": w.network,
            "node": node.name, "lab": lab,
            "status": w.status, "node_online": node.last_seen + 45 > now(),
            "metrics": w.metrics, "history": w.history, "gpu_ids": w.gpu_ids,
            "gpu_history": node.history.iter().map(|m| json!({"at": m.at, "gpus": m.gpus.iter().filter(|g| w.gpu_ids.contains(&g.id)).collect::<Vec<_>>()})).collect::<Vec<_>>(),
            "node_metrics_at": node.metrics.as_ref().map(|m| m.at),
            "gpus": node.gpus.iter().filter(|g| w.gpu_ids.contains(&g.id)).collect::<Vec<_>>(), "path": if w.template == "jupyter" { "/lab" } else { "/" }
        }))
        .into_response(),
    )
}
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
pub fn error(ApiError(status, message): ApiError) -> Response {
    let (title, detail) = if status == StatusCode::UNAUTHORIZED {
        ("Open this workspace from CloudLab", "This workspace link has expired or its session has ended. Return to your CloudLab dashboard and choose Open workspace to reconnect.")
    } else {
        ("Your workspace is unavailable", message.as_str())
    };
    headers((status, Html(format!(r#"<!doctype html>
<html lang="en" data-cloudlab-error><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Workspace connection · CloudLab</title><link rel="icon" href="/_cloudlab/favicon.svg"><link rel="stylesheet" href="/_cloudlab/workspace.css"></head>
<body class="error-page"><main class="connection-card"><img src="/_cloudlab/favicon.svg" alt="" width="48" height="48"><p class="eyebrow">CLOUDLAB · WORKSPACE</p><h1>{}</h1><p>{}</p><p class="help">Your files stay on your compute node. Reconnecting does not create a new container.</p></main></body></html>"#, escape(title), escape(detail)))).into_response())
}
