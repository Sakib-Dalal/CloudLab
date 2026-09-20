//! Package names become arguments to uv inside a verified container, never host commands.
use crate::server::*;
use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackageRequest {
    pub action: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
}

pub fn valid_package_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name.as_bytes()[0].is_ascii_alphanumeric()
        && name.as_bytes()[name.len() - 1].is_ascii_alphanumeric()
        && name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}
impl PackageRequest {
    pub fn validate(&self) -> Result<()> {
        if !matches!(self.action.as_str(), "list" | "install" | "remove")
            || (self.action != "list" && !valid_package_name(&self.name))
            || self.version.len() > 80
            || (!self.version.is_empty()
                && (!self.version.as_bytes()[0].is_ascii_alphanumeric()
                    || !self
                        .version
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b".!+_-".contains(&c))))
        {
            return Err(bad(
                "Choose a package name and a specific version, without URLs or command options.",
            ));
        }
        Ok(())
    }
}

pub async fn operation(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<String>,
    Json(body): Json<PackageRequest>,
) -> Result<Json<Value>> {
    actor.operate()?;
    body.validate()?;
    let mut store = state.store.lock().await;
    let w = store
        .db
        .workspaces
        .iter()
        .find(|w| w.id == id)
        .cloned()
        .ok_or_else(missing)?;
    actor.lab(&w.lab_id)?;
    if w.status != "running" {
        return Err(bad("Resume this workspace before managing packages."));
    }
    let node = store
        .db
        .nodes
        .iter()
        .find(|n| n.id == w.node_id && !n.revoked)
        .ok_or_else(missing)?;
    if node.last_seen + 45 <= crate::model::now() || !node.docker {
        return Err(bad(
            "The compute node must be online with Docker available.",
        ));
    }
    if body.action == "install" && !w.network {
        return Err(bad(
            "Enable outbound internet access in workspace settings before installing packages.",
        ));
    }
    if store
        .db
        .jobs
        .iter()
        .any(|j| j.workspace.id == id && matches!(j.status.as_str(), "queued" | "leased"))
    {
        return Err(bad("A workspace operation is already in progress. Wait for it to finish, then refresh packages."));
    }
    let command = serde_json::to_string(&body).map_err(anyhow::Error::from)?;
    let job = store.db.enqueue(w.clone(), "packages", &command);
    if body.action != "list" {
        store.db.event(
            &w.lab_id,
            format!(
                "{}: package {} requested for {}",
                w.name, body.action, body.name
            ),
            "workspace",
        );
    }
    store
        .db
        .workspaces
        .iter_mut()
        .find(|w| w.id == id)
        .unwrap()
        .last_used = crate::model::now();
    store.save()?;
    Ok(Json(json!({"id": job.id})))
}

#[derive(Deserialize)]
pub struct Search {
    q: String,
}

pub async fn search(
    Extension(actor): Extension<Actor>,
    Query(query): Query<Search>,
) -> Result<Json<Value>> {
    actor.operate()?;
    let q = query.q.trim().to_ascii_lowercase().replace('_', "-");
    if !valid_package_name(&q) {
        return Err(bad(
            "Search by a PyPI package name, such as numpy or requests.",
        ));
    }
    // PyPI has no public full-text search API. Exact-name lookup works for any
    // project; popular-name suggestions also help with partial queries.
    let popular = [
        "numpy",
        "pandas",
        "scipy",
        "matplotlib",
        "seaborn",
        "requests",
        "httpx",
        "polars",
        "scikit-learn",
        "torch",
        "torchvision",
        "torchaudio",
        "transformers",
        "datasets",
        "pillow",
        "fastapi",
        "rich",
        "pytest",
        "ruff",
        "sympy",
        "plotly",
        "opencv-python",
    ];
    let mut names = vec![q.clone()];
    names.extend(
        popular
            .iter()
            .filter(|n| **n != q && n.contains(&q))
            .take(5)
            .map(|n| (*n).into()),
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("CloudLab/2.0 package search")
        .build()
        .map_err(anyhow::Error::from)?;
    let mut results = Vec::new();
    for name in names {
        let response = client
            .get(format!("https://pypi.org/pypi/{name}/json"))
            .send()
            .await
            .map_err(|_| {
                bad("Could not reach PyPI. Check the coordinator's internet connection.")
            })?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            continue;
        }
        if !response.status().is_success() {
            return Err(bad("PyPI is temporarily unavailable. Try again shortly."));
        }
        let mut stream = response.bytes_stream();
        let mut bytes = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| bad("PyPI returned an incomplete response."))?;
            if bytes.len() + chunk.len() > 8 * 1024 * 1024 {
                return Err(bad("This package's metadata is too large to display."));
            }
            bytes.extend_from_slice(&chunk);
        }
        let value: Value = serde_json::from_slice(&bytes)
            .map_err(|_| bad("PyPI returned unreadable package metadata."))?;
        let info = &value["info"];
        results.push(json!({"name": info["name"], "version": info["version"], "summary": info["summary"], "requires_python": info["requires_python"], "url": format!("https://pypi.org/project/{name}/")}));
    }
    Ok(Json(json!({"packages": results})))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn package_arguments_do_not_accept_commands_paths_or_options() {
        for name in [
            "numpy",
            "scikit-learn",
            "zope.interface",
            "typing_extensions",
        ] {
            assert!(valid_package_name(name));
        }
        for name in [
            "",
            "-r",
            "../requirements.txt",
            "https://example.org/x.whl",
            "numpy;id",
            "numpy\n",
            "a b",
            "x[extra]",
            "numpy==1.0",
        ] {
            assert!(!valid_package_name(name));
        }
        for version in ["-r", "1;id", "../wheel", "1.*", "1 2"] {
            assert!(PackageRequest {
                action: "install".into(),
                name: "numpy".into(),
                version: version.into()
            }
            .validate()
            .is_err());
        }
    }
}
