use crate::{model::*, relay::Relay};
use anyhow::Context;
use axum::{
    extract::{DefaultBodyLimit, Path, Request, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::{watch, Mutex};
use tower_http::services::{ServeDir, ServeFile};

pub type Result<T> = std::result::Result<T, ApiError>;
#[derive(Debug)]
pub struct ApiError(pub StatusCode, pub String);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error":self.1}))).into_response()
    }
}
impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        eprintln!("CloudLab internal error: {e:#}");
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The operation could not be saved. Check the coordinator logs.".into(),
        )
    }
}
pub fn bad(message: &str) -> ApiError {
    ApiError(StatusCode::BAD_REQUEST, message.into())
}
pub fn forbidden() -> ApiError {
    ApiError(
        StatusCode::FORBIDDEN,
        "This access key does not allow that operation.".into(),
    )
}
pub fn missing() -> ApiError {
    ApiError(
        StatusCode::NOT_FOUND,
        "The requested resource was not found.".into(),
    )
}
pub fn unauthorized() -> ApiError {
    ApiError(
        StatusCode::UNAUTHORIZED,
        "Enter a valid lab access key to connect.".into(),
    )
}

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<Mutex<Store>>,
    pub admin_hash: String,
    pub relay: Arc<Relay>,
    pub app_url: String,
    pub remote_public_url: Option<String>,
    login_attempts: Arc<Mutex<(u64, u32)>>,
    desktop_bootstrap: Arc<Mutex<Option<(String, u64)>>>,
    shutdown: Option<watch::Sender<bool>>,
}
#[derive(Clone)]
pub struct Actor {
    pub role: String,
    pub lab_id: Option<String>,
    pub session_hash: String,
}
impl Actor {
    pub fn owner(&self) -> Result<()> {
        if self.role == "owner" {
            Ok(())
        } else {
            Err(forbidden())
        }
    }
    pub fn lab(&self, id: &str) -> Result<()> {
        if self.lab_id.as_deref().is_none_or(|l| l == id) {
            Ok(())
        } else {
            Err(forbidden())
        }
    }
    pub fn operate(&self) -> Result<()> {
        if self.role == "viewer" {
            Err(forbidden())
        } else {
            Ok(())
        }
    }
}
pub fn cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get("cookie")?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|c| {
            let (k, v) = c.trim().split_once('=')?;
            (k == name).then(|| v.to_string())
        })
}
pub fn session_actor(db: &Database, h: &str) -> Option<Actor> {
    let s = db
        .sessions
        .iter()
        .find(|s| s.token_hash == h && s.expires_at > now())?;
    if let Some(id) = &s.access_id {
        let access = db
            .access
            .iter()
            .find(|a| &a.id == id && a.expires_at > now())?;
        Some(Actor {
            role: access.role.clone(),
            lab_id: Some(access.lab_id.clone()),
            session_hash: h.into(),
        })
    } else {
        Some(Actor {
            role: "owner".into(),
            lab_id: None,
            session_hash: h.into(),
        })
    }
}
pub fn bearer(headers: &HeaderMap) -> Result<String> {
    Ok(headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .filter(|s| s.len() == 64)
        .ok_or_else(unauthorized)?
        .into())
}
pub async fn agent_node(state: &AppState, headers: &HeaderMap) -> Result<Node> {
    let h = hash(&bearer(headers)?);
    let store = state.store.lock().await;
    store
        .db
        .nodes
        .iter()
        .find(|n| n.credential_hash == h && !n.revoked)
        .cloned()
        .ok_or_else(unauthorized)
}
async fn auth(State(state): State<AppState>, mut req: Request, next: Next) -> Result<Response> {
    let value = cookie(req.headers(), "cloudlab_session").ok_or_else(unauthorized)?;
    let actor =
        session_actor(&state.store.lock().await.db, &hash(&value)).ok_or_else(unauthorized)?;
    req.extensions_mut().insert(actor);
    Ok(next.run(req).await)
}
async fn security(req: Request, next: Next) -> Result<Response> {
    // A custom header on every browser mutation prevents form and cross-origin CSRF.
    // No CORS policy is installed: workspace origins cannot read or mutate this API.
    if !matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS)
        && !req.uri().path().starts_with("/api/agent/")
        && req
            .headers()
            .get("x-cloudlab-client")
            .and_then(|v| v.to_str().ok())
            != Some("web")
    {
        return Err(forbidden());
    }
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("cache-control", HeaderValue::from_static("no-store"));
    headers.insert("content-security-policy",HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'"));
    Ok(response)
}
pub async fn create_state(dir: PathBuf, app_url: String) -> anyhow::Result<AppState> {
    validate_app_url(&app_url)?;
    let store = Store::open(&dir)?;
    let token_path = dir.join("admin-token");
    let admin = if token_path.exists() {
        std::fs::read_to_string(&token_path)?.trim().to_string()
    } else {
        let t = token();
        write_private(&token_path, t.as_bytes())?;
        t
    };
    anyhow::ensure!(admin.len() == 64, "Invalid admin-token file");
    eprintln!(
        "Owner access key: {} (read this file locally; keep it private)",
        token_path.display()
    );
    Ok(AppState {
        store: Arc::new(Mutex::new(store)),
        admin_hash: hash(&admin),
        relay: Arc::new(Relay::default()),
        app_url,
        remote_public_url: None,
        login_attempts: Arc::new(Mutex::new((now(), 0))),
        desktop_bootstrap: Arc::new(Mutex::new(None)),
        shutdown: None,
    })
}
pub fn validate_app_url(value: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        value.matches("{workspace}").count() == 1,
        "App URL must contain one {{workspace}} hostname placeholder"
    );
    let u = url::Url::parse(&value.replace("{workspace}", "w-test"))?;
    let host = u.host_str().unwrap_or("");
    anyhow::ensure!(
        host.starts_with("w-test.")
            && u.username().is_empty()
            && u.password().is_none()
            && u.query().is_none()
            && u.fragment().is_none()
            && u.path() == "/",
        "App URL must be an origin with a workspace subdomain"
    );
    anyhow::ensure!(
        u.scheme() == "https" || (u.scheme() == "http" && host.ends_with(".localhost")),
        "Workspace traffic requires HTTPS outside localhost"
    );
    Ok(())
}
pub fn router(state: AppState, web: PathBuf) -> Router {
    let protected = Router::new()
        .route("/state", get(snapshot))
        .route("/logout", post(logout))
        .route("/labs", post(create_lab))
        .route("/enrollments", post(enrollment))
        .route("/nodes/{id}", delete(revoke_node))
        .route("/workspaces", post(create_workspace))
        .route("/workspaces/{id}/actions", post(workspace_action))
        .route("/workspaces/{id}/open", post(crate::relay::open_workspace))
        .route("/jobs/{id}", get(job))
        .route("/settings", put(settings))
        .route("/access", post(access))
        .route("/access/{id}", delete(revoke_access))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth));
    let api = Router::new()
        .merge(protected)
        .route("/login", post(login))
        .route("/desktop-login", post(desktop_login))
        .route("/desktop-bootstrap/{token}", get(desktop_bootstrap))
        .route(
            "/health",
            get(|State(state): State<AppState>| async move {
                // A public, non-credential fingerprint lets setup verify that
                // DNS reaches this installation, rather than another lab.
                Json(json!({"status":"ok","version":"2.0.0", "instance_id":hash(&format!("cloudlab-public-instance:{}",state.admin_hash))}))
            }),
        )
        .route("/agent/enroll", post(agent_enroll))
        .route("/agent/poll", post(agent_poll))
        .route("/agent/jobs/{id}", post(agent_complete))
        .route("/agent/tunnel", get(crate::relay::agent_tunnel));
    let mut api = api.route("/tls/allow", get(crate::remote::allow_certificate));
    if state.shutdown.is_some() {
        api = api.route("/shutdown", post(shutdown));
    }
    Router::new()
        .nest("/api", api)
        .fallback_service(
            ServeDir::new(&web).not_found_service(ServeFile::new(web.join("index.html"))),
        )
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(middleware::from_fn(security))
        .with_state(state)
}
pub async fn serve(
    bind: SocketAddr,
    app_bind: SocketAddr,
    app_url: String,
    public_url: Option<String>,
    dir: PathBuf,
    web: PathBuf,
) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(bind).await?;
    let gateway = tokio::net::TcpListener::bind(app_bind).await?;
    let mut state = create_state(dir, app_url).await?;
    if let Some(public_url) = public_url {
        crate::remote::configure(&mut state, &public_url).await?;
    }
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    state.shutdown = Some(shutdown_tx);
    eprintln!("CloudLab coordinator: http://{}", listener.local_addr()?);
    eprintln!("Workspace gateway: {}", gateway.local_addr()?);
    let maintenance = tokio::spawn(maintenance(state.clone()));
    let servers = async {
        tokio::try_join!(
            async {
                axum::serve(listener, router(state.clone(), web))
                    .with_graceful_shutdown(wait_for_shutdown(shutdown_rx.clone()))
                    .await
            },
            async {
                axum::serve(gateway, crate::relay::gateway_router(state.clone()))
                    .with_graceful_shutdown(wait_for_shutdown(shutdown_rx.clone()))
                    .await
            }
        )
    };
    tokio::pin!(servers);
    let result = tokio::select! {
        result = &mut servers => result,
        _ = wait_for_shutdown(shutdown_rx.clone()) => {
            eprintln!("Stopping CloudLab coordinator and workspace gateway...");
            // Long polls and active gateway requests must not prevent shutdown.
            match tokio::time::timeout(Duration::from_secs(5), &mut servers).await {
                Ok(result) => result,
                Err(_) => {
                    eprintln!("Shutdown grace period elapsed; closing remaining connections.");
                    Ok(((), ()))
                }
            }
        }
    };
    maintenance.abort();
    result?;
    Ok(())
}

async fn wait_for_shutdown(mut shutdown: watch::Receiver<bool>) {
    let _ = shutdown.wait_for(|stopping| *stopping).await;
}

async fn shutdown(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Value>> {
    // Only the owner key read from disk can stop a standalone server. Browser
    // sessions, scoped access keys, and agent credentials cannot shut it down.
    if hash(&bearer(&headers)?) != state.admin_hash {
        return Err(unauthorized());
    }
    state
        .shutdown
        .as_ref()
        .ok_or_else(missing)?
        .send_replace(true);
    Ok(Json(json!({"status":"stopping"})))
}

pub async fn stop(mut bind: SocketAddr, dir: PathBuf) -> anyhow::Result<()> {
    let token_path = dir.join("admin-token");
    let owner_key = std::fs::read_to_string(&token_path).with_context(|| {
        format!(
            "Cannot read owner key at {}. Use the same --data-dir as cloudlab serve",
            token_path.display()
        )
    })?;
    let owner_key = owner_key.trim();
    anyhow::ensure!(
        owner_key.len() == 64 && owner_key.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "Invalid admin-token file at {}",
        token_path.display()
    );
    // Wildcard listen addresses are not destinations; contact them locally.
    if bind.ip().is_unspecified() {
        bind.set_ip(if bind.is_ipv4() {
            std::net::Ipv4Addr::LOCALHOST.into()
        } else {
            std::net::Ipv6Addr::LOCALHOST.into()
        });
    }
    let response = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(10))
        .build()?
        .post(format!("http://{bind}/api/shutdown"))
        .header("x-cloudlab-client", "web")
        .bearer_auth(owner_key)
        .send()
        .await
        .with_context(|| format!("Cannot reach CloudLab at {bind}. Check that it is running and use the same --bind as cloudlab serve"))?;
    anyhow::ensure!(
        response.status() != StatusCode::UNAUTHORIZED,
        "The owner key does not match the running coordinator. Use the same --data-dir as cloudlab serve"
    );
    let rejection = if matches!(
        response.status(),
        StatusCode::NOT_FOUND | StatusCode::METHOD_NOT_ALLOWED
    ) {
        "The running server does not support cloudlab stop yet. Stop its existing serve process once (Ctrl+C in its terminal), then restart serve with the updated binary. Rebuilding alone does not update a running server. If the desktop app owns the server, close the app instead."
    } else {
        "The server rejected the CloudLab shutdown request"
    };
    let body: Value = response
        .error_for_status()
        .context(rejection)?
        .json()
        .await
        .context("The server did not return a CloudLab shutdown response")?;
    anyhow::ensure!(
        body["status"] == "stopping",
        "The server did not acknowledge shutdown"
    );
    println!("CloudLab shutdown requested. The coordinator and workspace gateway will stop within 5 seconds; compute containers remain running.");
    Ok(())
}
pub async fn start_desktop(dir: PathBuf, mut web: PathBuf) -> anyhow::Result<String> {
    if cfg!(debug_assertions) {
        web = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dist");
    }
    // Stable ports keep enrolled agents connected across desktop restarts.
    // If a standalone coordinator already owns the port, act as its client.
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:8088").await {
        Ok(listener) => listener,
        Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(3))
                .build()?;
            let health: Value = client
                .get("http://127.0.0.1:8088/api/health")
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            anyhow::ensure!(
                health["status"] == "ok" && health["version"] == "2.0.0",
                "Port 8088 is occupied by another application"
            );
            return Ok("http://127.0.0.1:8088/".into());
        }
        Err(error) => return Err(error.into()),
    };
    let address = format!("http://{}", listener.local_addr()?);
    let gateway = tokio::net::TcpListener::bind("127.0.0.1:8089").await?;
    let state = create_state(
        dir,
        format!(
            "http://{{workspace}}.localhost:{}",
            gateway.local_addr()?.port()
        ),
    )
    .await?;
    let bootstrap = token();
    *state.desktop_bootstrap.lock().await = Some((hash(&bootstrap), now() + 60));
    tokio::spawn(maintenance(state.clone()));
    let other = state.clone();
    tokio::spawn(async move {
        let _ = axum::serve(gateway, crate::relay::gateway_router(other)).await;
    });
    tokio::spawn(async move {
        let _ = axum::serve(listener, router(state, web)).await;
    });
    Ok(format!("{address}/api/desktop-bootstrap/{bootstrap}"))
}
async fn maintenance(state: AppState) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        let mut store = state.store.lock().await;
        store.db.prune();
        let idle = store.db.settings.idle_minutes;
        let to_stop: Vec<_> = store
            .db
            .workspaces
            .iter()
            .filter(|w| idle > 0 && w.status == "running" && w.last_used + idle * 60 < now())
            .cloned()
            .collect();
        for w in to_stop {
            if let Some(row) = store.db.workspaces.iter_mut().find(|r| r.id == w.id) {
                row.status = "stopping".into();
            }
            store.db.enqueue(w.clone(), "stop", "");
            store.db.event(
                &w.lab_id,
                format!("{} reached the idle limit", w.name),
                "workspace",
            );
        }
        if let Err(e) = store.save() {
            eprintln!("State save failed: {e}");
        }
    }
}

#[derive(Deserialize)]
struct Login {
    token: String,
}
async fn desktop_bootstrap(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Response> {
    let mut response = desktop_login(State(state), Json(Login { token })).await?;
    *response.status_mut() = StatusCode::SEE_OTHER;
    response
        .headers_mut()
        .insert("location", HeaderValue::from_static("/"));
    Ok(response)
}
async fn desktop_login(State(state): State<AppState>, Json(body): Json<Login>) -> Result<Response> {
    let mut bootstrap = state.desktop_bootstrap.lock().await;
    if !bootstrap
        .as_ref()
        .is_some_and(|(h, expiry)| *h == hash(&body.token) && *expiry > now())
    {
        return Err(unauthorized());
    }
    let mut store = state.store.lock().await;
    let value = token();
    let seconds = store.db.settings.session_hours * 3600;
    store.db.sessions.push(Session {
        token_hash: hash(&value),
        access_id: None,
        expires_at: now() + seconds,
    });
    store.save()?;
    *bootstrap = None;
    Ok((
        [(
            "set-cookie",
            format!(
                "cloudlab_session={value}; HttpOnly; SameSite=Strict; Path=/; Max-Age={seconds}"
            ),
        )],
        Json(json!({"ok":true})),
    )
        .into_response())
}
async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Login>,
) -> Result<Response> {
    let mut attempts = state.login_attempts.lock().await;
    if attempts.0 + 60 < now() {
        *attempts = (now(), 0);
    }
    if attempts.1 >= 30 {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many sign-in attempts. Try again in a minute.".into(),
        ));
    }
    attempts.1 += 1;
    drop(attempts);
    if body.token.len() != 64 {
        return Err(unauthorized());
    }
    let h = hash(&body.token);
    let mut store = state.store.lock().await;
    let access_id = if h == state.admin_hash {
        None
    } else {
        Some(
            store
                .db
                .access
                .iter()
                .find(|a| a.token_hash == h && a.expires_at > now())
                .ok_or_else(unauthorized)?
                .id
                .clone(),
        )
    };
    let value = token();
    let expiry = now() + store.db.settings.session_hours * 3600;
    store.db.sessions.push(Session {
        token_hash: hash(&value),
        access_id,
        expires_at: expiry,
    });
    store.save()?;
    let secure = headers
        .get("origin")
        .and_then(|h| h.to_str().ok())
        .is_some_and(|s| s.starts_with("https://"));
    let cookie = format!(
        "cloudlab_session={value}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}{}",
        expiry - now(),
        if secure { "; Secure" } else { "" }
    );
    Ok(([("set-cookie", cookie)], Json(json!({"ok":true}))).into_response())
}
async fn logout(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
) -> Result<Response> {
    let mut s = state.store.lock().await;
    s.db.sessions.retain(|v| v.token_hash != actor.session_hash);
    s.save()?;
    Ok((
        [(
            "set-cookie",
            "cloudlab_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0",
        )],
        Json(json!({"ok":true})),
    )
        .into_response())
}
async fn snapshot(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
) -> Result<Json<Value>> {
    let store = state.store.lock().await;
    let db = &store.db;
    let labs: Vec<_> = db
        .labs
        .iter()
        .filter(|l| actor.lab(&l.id).is_ok())
        .collect();
    let nodes: Vec<Value> = db
        .nodes
        .iter()
        .filter(|n| actor.lab(&n.lab_id).is_ok())
        .map(|n| {
            let mut v = serde_json::to_value(n).unwrap();
            v.as_object_mut().unwrap().remove("credential_hash");
            v["history"] = json!(n.history);
            v
        })
        .collect();
    let workspaces: Vec<Value> = db
        .workspaces
        .iter()
        .filter(|w| actor.lab(&w.lab_id).is_ok())
        .map(|w| {
            let mut value = serde_json::to_value(w).unwrap();
            value["history"] = json!(w.history);
            value
        })
        .collect();
    let access: Vec<Value> = if actor.role == "owner" {
        db.access
            .iter()
            .filter(|a| actor.lab(&a.lab_id).is_ok() && a.expires_at > now())
            .map(|a| {
                let mut v = serde_json::to_value(a).unwrap();
                v.as_object_mut().unwrap().remove("token_hash");
                v
            })
            .collect()
    } else {
        vec![]
    };
    Ok(Json(
        json!({"labs":labs,"nodes":nodes,"workspaces":workspaces,"events":db.events.iter().filter(|e|actor.lab(&e.lab_id).is_ok()).collect::<Vec<_>>(),"settings":db.settings,"access":access,"role":actor.role,
            "remote_access":{"managed":state.remote_public_url.is_some(),"public_url":db.settings.public_url,"workspace_url":state.app_url}}),
    ))
}
#[derive(Deserialize)]
struct NewLab {
    name: String,
    #[serde(default)]
    description: String,
}
async fn create_lab(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(body): Json<NewLab>,
) -> Result<Json<Lab>> {
    actor.owner()?;
    if !valid_name(&body.name) || body.description.len() > 160 {
        return Err(bad(
            "Use a name of 2–60 characters and a description under 160 characters.",
        ));
    }
    let mut s = state.store.lock().await;
    if s.db.labs.len() >= 50 {
        return Err(bad("This coordinator has reached its 50-lab limit."));
    }
    let lab = Lab {
        id: id(),
        name: body.name.trim().into(),
        description: body.description,
    };
    s.db.labs.push(lab.clone());
    s.db.event(&lab.id, format!("{} created", lab.name), "lab");
    s.save()?;
    Ok(Json(lab))
}
#[derive(Deserialize)]
struct ScopedName {
    lab_id: String,
    name: String,
    #[serde(default)]
    role: String,
}
async fn enrollment(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(body): Json<ScopedName>,
) -> Result<Json<Value>> {
    actor.owner()?;
    if !valid_name(&body.name) {
        return Err(bad("Use a device name of 2–60 characters."));
    }
    let mut s = state.store.lock().await;
    if !s.db.labs.iter().any(|l| l.id == body.lab_id) {
        return Err(missing());
    }
    let t = token();
    s.db.enrollments.push(Enrollment {
        token_hash: hash(&t),
        lab_id: body.lab_id,
        name: body.name,
        expires_at: now() + 600,
    });
    s.save()?;
    Ok(Json(json!({"token":t,"expires_in":600})))
}
async fn access(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(body): Json<ScopedName>,
) -> Result<Json<Value>> {
    actor.owner()?;
    if !valid_name(&body.name) || !matches!(body.role.as_str(), "viewer" | "operator") {
        return Err(bad("Choose a name and a valid permission."));
    }
    let mut s = state.store.lock().await;
    if !s.db.labs.iter().any(|l| l.id == body.lab_id) {
        return Err(missing());
    }
    let t = token();
    s.db.access.push(Access {
        id: id(),
        lab_id: body.lab_id.clone(),
        name: body.name.clone(),
        role: body.role,
        expires_at: now() + 7 * 86400,
        token_hash: hash(&t),
    });
    s.db.event(
        &body.lab_id,
        format!("Access key created for {}", body.name),
        "access",
    );
    s.save()?;
    Ok(Json(json!({"token":t})))
}
async fn revoke_access(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    actor.owner()?;
    let mut s = state.store.lock().await;
    let a =
        s.db.access
            .iter()
            .find(|a| a.id == id)
            .cloned()
            .ok_or_else(missing)?;
    s.db.access.retain(|a| a.id != id);
    s.db.sessions
        .retain(|a| a.access_id.as_deref() != Some(&id));
    s.db.event(
        &a.lab_id,
        format!("Access revoked for {}", a.name),
        "access",
    );
    s.save()?;
    Ok(Json(json!({"ok":true})))
}
async fn revoke_node(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    actor.owner()?;
    let mut s = state.store.lock().await;
    let n =
        s.db.nodes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or_else(missing)?;
    n.revoked = true;
    let lab = n.lab_id.clone();
    let name = n.name.clone();
    s.db.event(&lab, format!("{} access revoked", name), "node");
    s.save()?;
    drop(s);
    state.relay.disconnect(&id).await;
    Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]
struct NewWorkspace {
    lab_id: String,
    node_id: String,
    name: String,
    template: String,
    cpus: u32,
    memory_mb: u64,
    #[serde(default)]
    network: bool,
    #[serde(default)]
    gpu_ids: Vec<String>,
}
async fn create_workspace(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(body): Json<NewWorkspace>,
) -> Result<Json<Workspace>> {
    actor.operate()?;
    actor.lab(&body.lab_id)?;
    if !valid_name(&body.name)
        || !matches!(body.template.as_str(), "terminal" | "jupyter" | "code")
        || (body.cpus == 0 || body.cpus > 256)
        || !(256..=1048576).contains(&body.memory_mb)
    {
        return Err(bad(
            "Choose a valid name, template, CPU budget, and memory budget.",
        ));
    }
    let mut s = state.store.lock().await;
    let node =
        s.db.nodes
            .iter()
            .find(|n| n.id == body.node_id && n.lab_id == body.lab_id && !n.revoked)
            .ok_or_else(missing)?;
    if node.last_seen + 45 < now() || !node.docker {
        return Err(bad("Choose an online node with Docker available."));
    }
    let existing: Vec<_> =
        s.db.workspaces
            .iter()
            .filter(|w| w.node_id == node.id && w.status != "deleting")
            .collect();
    if existing.iter().map(|w| w.cpus).sum::<u32>() + body.cpus > node.cpus
        || existing.iter().map(|w| w.memory_mb).sum::<u64>() + body.memory_mb > node.memory_mb
    {
        return Err(bad("This node does not have enough unallocated CPU or memory. Remove a workspace or choose another node."));
    }
    if body.gpu_ids.len() > 8
        || body
            .gpu_ids
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
            != body.gpu_ids.len()
    {
        return Err(bad("Choose up to eight distinct GPUs."));
    }
    for gpu in &body.gpu_ids {
        if !node
            .gpus
            .iter()
            .any(|g| &g.id == gpu && matches!(g.access.as_str(), "nvidia" | "dri"))
        {
            return Err(bad("This GPU is unavailable for workspace access. Refresh the node and check its drivers."));
        }
        if s.db
            .workspaces
            .iter()
            .any(|w| w.node_id == node.id && w.gpu_ids.contains(gpu))
        {
            return Err(bad("This GPU is already assigned to another workspace. Remove that workspace to release it."));
        }
    }
    if !body.gpu_ids.is_empty() && node.metrics.as_ref().is_none_or(|m| m.at + 30 < now()) {
        return Err(bad(
            "GPU detection is out of date. Wait for a fresh node sample.",
        ));
    }
    if body.network && !s.db.settings.allow_network {
        return Err(bad("Outbound networking is disabled in lab settings."));
    }
    if s.db
        .workspaces
        .iter()
        .filter(|w| w.lab_id == body.lab_id)
        .count()
        >= s.db.settings.max_workspaces
    {
        return Err(bad("This lab has reached its workspace limit."));
    }
    let w = Workspace {
        id: id(),
        lab_id: body.lab_id,
        node_id: body.node_id,
        name: body.name.trim().into(),
        template: body.template,
        cpus: body.cpus,
        memory_mb: body.memory_mb,
        status: "creating".into(),
        created_at: now(),
        last_used: now(),
        error: String::new(),
        network: body.network,
        gpu_ids: body.gpu_ids,
        metrics: None,
        history: Vec::new(),
    };
    s.db.workspaces.push(w.clone());
    s.db.enqueue(w.clone(), "create", "");
    s.db.event(
        &w.lab_id,
        format!("{} creation requested", w.name),
        "workspace",
    );
    s.save()?;
    Ok(Json(w))
}
#[derive(Deserialize)]
struct Action {
    action: String,
    #[serde(default)]
    command: String,
}
async fn workspace_action(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<String>,
    Json(body): Json<Action>,
) -> Result<Json<Value>> {
    actor.operate()?;
    if !matches!(body.action.as_str(), "start" | "stop" | "delete" | "exec") {
        return Err(bad("Unsupported workspace action."));
    }
    if body.command.len() > 4096 {
        return Err(bad("Commands are limited to 4096 bytes."));
    }
    let mut s = state.store.lock().await;
    let w =
        s.db.workspaces
            .iter()
            .find(|w| w.id == id)
            .cloned()
            .ok_or_else(missing)?;
    actor.lab(&w.lab_id)?;
    if !s
        .db
        .nodes
        .iter()
        .any(|n| n.id == w.node_id && !n.revoked && n.last_seen + 45 > now())
    {
        return Err(bad("The compute node is offline or revoked."));
    }
    if s.db
        .jobs
        .iter()
        .any(|j| j.workspace.id == id && matches!(j.status.as_str(), "queued" | "leased"))
    {
        return Err(bad("This workspace already has an operation in progress."));
    }
    if (body.action == "exec" || body.action == "stop") && w.status != "running" {
        return Err(bad("The workspace must be running."));
    }
    if body.action == "start" && w.status != "stopped" {
        return Err(bad("Only stopped workspaces can be resumed."));
    }
    let row = s.db.workspaces.iter_mut().find(|w| w.id == id).unwrap();
    row.last_used = now();
    row.status = match body.action.as_str() {
        "start" => "starting",
        "stop" => "stopping",
        "delete" => "deleting",
        _ => &w.status,
    }
    .into();
    let j = s.db.enqueue(w.clone(), &body.action, &body.command);
    s.db.event(
        &w.lab_id,
        format!("{}: {} requested", w.name, body.action),
        "workspace",
    );
    s.save()?;
    Ok(Json(json!({"id":j.id})))
}
async fn job(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    actor.operate()?;
    let s = state.store.lock().await;
    let j = s.db.jobs.iter().find(|j| j.id == id).ok_or_else(missing)?;
    actor.lab(&j.workspace.lab_id)?;
    Ok(Json(
        json!({"id":j.id,"status":j.status,"output":j.output,"error":j.error}),
    ))
}
async fn settings(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(body): Json<Settings>,
) -> Result<Json<Value>> {
    actor.owner()?;
    if !valid_name(&body.name)
        || !(1..=72).contains(&body.session_hours)
        || body.idle_minutes > 10080
        || !(1..=100).contains(&body.max_workspaces)
        || body.default_cpus == 0
        || body.default_cpus > 256
        || !(256..=1048576).contains(&body.default_memory_mb)
    {
        return Err(bad("One or more settings are outside the allowed range."));
    }
    if !body.public_url.is_empty() {
        crate::agent::validate_coordinator(&body.public_url)
            .map_err(|_| bad("Use an HTTPS URL, or HTTP on localhost."))?;
    }
    if state
        .remote_public_url
        .as_ref()
        .is_some_and(|url| url != &body.public_url)
    {
        return Err(bad("This address is managed by the cloud setup. Run the setup again on your server to change it."));
    }
    let mut s = state.store.lock().await;
    s.db.settings = body;
    s.save()?;
    Ok(Json(json!({"ok":true})))
}

#[derive(Deserialize)]
pub struct EnrollRequest {
    pub token: String,
    pub platform: String,
    pub arch: String,
    pub cpus: u32,
    pub memory_mb: u64,
}
async fn agent_enroll(
    State(state): State<AppState>,
    Json(body): Json<EnrollRequest>,
) -> Result<Json<Value>> {
    let mut s = state.store.lock().await;
    let i =
        s.db.enrollments
            .iter()
            .position(|e| e.token_hash == hash(&body.token) && e.expires_at > now())
            .ok_or_else(unauthorized)?;
    if body.platform.len() > 40
        || body.arch.len() > 40
        || body.cpus == 0
        || body.cpus > 4096
        || body.memory_mb == 0
    {
        return Err(bad("Invalid device information."));
    }
    let e = s.db.enrollments.remove(i);
    let t = token();
    let node = Node {
        id: id(),
        lab_id: e.lab_id.clone(),
        name: e.name.clone(),
        platform: body.platform,
        arch: body.arch,
        cpus: body.cpus,
        memory_mb: body.memory_mb,
        cpu_usage: 0.,
        memory_used_mb: 0,
        last_seen: 0,
        docker: false,
        revoked: false,
        credential_hash: hash(&t),
        gpus: Vec::new(),
        metrics: None,
        history: Vec::new(),
    };
    s.db.nodes.push(node.clone());
    s.db.event(&e.lab_id, format!("{} joined the lab", e.name), "node");
    s.save()?;
    Ok(Json(json!({"id":node.id,"token":t})))
}
#[derive(Deserialize)]
pub struct Heartbeat {
    pub cpu_usage: f32,
    pub memory_used_mb: u64,
    pub docker: bool,
    #[serde(default = "ready_by_default")]
    pub ready: bool,
    #[serde(default)]
    pub containers: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub metrics: Option<Metrics>,
    #[serde(default)]
    pub workspace_metrics: std::collections::HashMap<String, Metrics>,
}
fn ready_by_default() -> bool {
    true
}
async fn agent_poll(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Heartbeat>,
) -> Result<Json<Value>> {
    let node = agent_node(&state, &headers).await?;
    let mut s = state.store.lock().await;
    let n =
        s.db.nodes
            .iter_mut()
            .find(|n| n.id == node.id && !n.revoked)
            .ok_or_else(unauthorized)?;
    n.last_seen = now();
    n.docker = body.docker;
    n.cpu_usage = body.cpu_usage.clamp(0., 100.);
    n.memory_used_mb = body.memory_used_mb.min(n.memory_mb);
    let received = now();
    if let Some(mut m) = body
        .metrics
        .filter(|m| m.at <= received + 5 && m.at + 30 >= received)
    {
        m.at = m.at.min(received);
        m.cpu_usage = m.cpu_usage.clamp(0., 100.);
        m.memory_total_mb = n.memory_mb;
        m.memory_used_mb = m.memory_used_mb.min(n.memory_mb);
        m.gpus.truncate(32);
        n.gpus = m.gpus.clone();
        record_metrics(&mut n.history, &m);
        n.metrics = Some(m);
    }
    for w in s.db.workspaces.iter_mut().filter(|w| w.node_id == node.id) {
        if let Some(mut m) = body
            .workspace_metrics
            .get(&w.id)
            .cloned()
            .filter(|m| m.at <= received + 5 && m.at + 30 >= received)
        {
            m.at = m.at.min(received);
            m.cpu_usage = (m.cpu_usage / w.cpus.max(1) as f32).clamp(0., 100.);
            m.memory_total_mb = w.memory_mb;
            m.memory_used_mb = m.memory_used_mb.min(w.memory_mb);
            m.gpus.clear(); // Device-wide GPU readings are never presented as container measurements.
            record_metrics(&mut w.history, &m);
            w.metrics = Some(m);
        }
    }
    for w in s.db.workspaces.iter_mut().filter(|w| {
        body.ready && w.node_id == node.id && matches!(w.status.as_str(), "running" | "stopped")
    }) {
        if let Some(status) = body.containers.get(&w.id) {
            if matches!(status.as_str(), "running" | "stopped") {
                w.status = status.clone();
            }
        } else if body.docker {
            w.status = "error".into();
            w.error =
                "Container is missing on this node. Remove the workspace and create it again."
                    .into();
        }
    }
    // Console commands are deliberately at-most-once: an expired exec lease is never replayed.
    for j in s.db.jobs.iter_mut().filter(|j| {
        j.node_id == node.id && j.status == "leased" && j.lease_until < now() && j.action == "exec"
    }) {
        j.status = "failed".into();
        j.error =
            "Command outcome is unknown after the node disconnected. It was not replayed.".into();
    }
    let job =
        s.db.jobs
            .iter_mut()
            .find(|j| {
                body.ready
                    && j.node_id == node.id
                    && (j.status == "queued"
                        || (j.status == "leased" && j.lease_until < now() && j.action != "exec"))
            })
            .map(|j| {
                j.status = "leased".into();
                j.lease_until = now() + 900;
                j.clone()
            });
    s.save()?;
    Ok(Json(json!({"job":job})))
}
#[derive(Deserialize)]
pub struct Completion {
    pub ok: bool,
    #[serde(default)]
    pub output: String,
    #[serde(default)]
    pub error: String,
}
async fn agent_complete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<Completion>,
) -> Result<Json<Value>> {
    let node = agent_node(&state, &headers).await?;
    if body.output.len() > 65536 || body.error.len() > 4096 {
        return Err(bad("Job output exceeds the limit."));
    }
    let mut s = state.store.lock().await;
    let j =
        s.db.jobs
            .iter_mut()
            .find(|j| j.id == id && j.node_id == node.id)
            .ok_or_else(missing)?;
    if j.status == "done" || j.status == "failed" {
        return Ok(Json(json!({"ok":true})));
    }
    j.status = if body.ok { "done" } else { "failed" }.into();
    j.output = body.output;
    j.error = body.error.clone();
    let j = j.clone();
    if j.action != "exec" {
        if body.ok && j.action == "delete" {
            s.db.workspaces.retain(|w| w.id != j.workspace.id);
        } else if let Some(w) = s.db.workspaces.iter_mut().find(|w| w.id == j.workspace.id) {
            w.status = if !body.ok {
                "error"
            } else if j.action == "stop" {
                "stopped"
            } else {
                "running"
            }
            .into();
            w.error = body.error;
            w.last_used = now();
        }
    }
    s.db.event(
        &j.workspace.lab_id,
        format!(
            "{}: {} {}",
            j.workspace.name,
            j.action,
            if body.ok { "completed" } else { "failed" }
        ),
        "workspace",
    );
    s.save()?;
    Ok(Json(json!({"ok":true})))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn gpu_allocation_and_telemetry_are_scoped_to_the_node() {
        let directory = tempfile::tempdir().unwrap();
        let state = create_state(
            directory.path().to_path_buf(),
            "http://{workspace}.localhost:8089".into(),
        )
        .await
        .unwrap();
        let lab = state.store.lock().await.db.labs[0].id.clone();
        let node_id = id();
        let secret = token();
        let gpu = Gpu {
            id: "GPU-test".into(),
            name: "Test GPU".into(),
            access: "nvidia".into(),
            ..Default::default()
        };
        state.store.lock().await.db.nodes.push(Node {
            id: node_id.clone(),
            lab_id: lab.clone(),
            name: "GPU node".into(),
            platform: "Linux".into(),
            arch: "x86_64".into(),
            cpus: 8,
            memory_mb: 8192,
            cpu_usage: 0.,
            memory_used_mb: 0,
            last_seen: now(),
            docker: true,
            revoked: false,
            credential_hash: hash(&secret),
            gpus: vec![gpu.clone()],
            metrics: Some(Metrics {
                at: now(),
                ..Default::default()
            }),
            history: vec![],
        });
        let actor = Actor {
            role: "operator".into(),
            lab_id: Some(lab.clone()),
            session_hash: String::new(),
        };
        let body = |gpu_ids| NewWorkspace {
            lab_id: lab.clone(),
            node_id: node_id.clone(),
            name: "GPU workspace".into(),
            template: "terminal".into(),
            cpus: 2,
            memory_mb: 1024,
            network: false,
            gpu_ids,
        };
        assert!(create_workspace(
            State(state.clone()),
            Extension(actor.clone()),
            Json(body(vec!["/dev/mem".into()]))
        )
        .await
        .is_err());
        let w = create_workspace(
            State(state.clone()),
            Extension(actor.clone()),
            Json(body(vec![gpu.id.clone()])),
        )
        .await
        .unwrap()
        .0;
        assert!(create_workspace(
            State(state.clone()),
            Extension(actor.clone()),
            Json(body(vec![gpu.id.clone()]))
        )
        .await
        .is_err());
        let mut headers = HeaderMap::new();
        headers.insert("authorization", format!("Bearer {secret}").parse().unwrap());
        let sample = Metrics {
            at: now(),
            cpu_usage: 160.,
            memory_used_mb: 256,
            memory_total_mb: 9000,
            ..Default::default()
        };
        let heartbeat = Heartbeat {
            cpu_usage: 25.,
            memory_used_mb: 300,
            docker: true,
            ready: false,
            containers: Default::default(),
            metrics: Some(Metrics {
                gpus: vec![gpu],
                ..sample.clone()
            }),
            workspace_metrics: [(w.id.clone(), sample.clone()), (id(), sample)]
                .into_iter()
                .collect(),
        };
        let _ = agent_poll(State(state.clone()), headers, Json(heartbeat))
            .await
            .unwrap();
        let snapshot = snapshot(State(state.clone()), Extension(actor))
            .await
            .unwrap()
            .0;
        assert_eq!(snapshot["workspaces"][0]["metrics"]["cpu_usage"], 80.);
        assert_eq!(
            snapshot["workspaces"][0]["metrics"]["memory_total_mb"],
            1024
        );
        assert_eq!(snapshot["workspaces"][0]["metrics"]["gpus"], json!([]));
        assert_eq!(
            snapshot["workspaces"][0]["history"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(snapshot["nodes"][0].get("credential_hash").is_none());
        assert_eq!(snapshot["nodes"][0]["history"].as_array().unwrap().len(), 1);
        let outsider = Actor {
            role: "viewer".into(),
            lab_id: Some(id()),
            session_hash: String::new(),
        };
        let snapshot = super::snapshot(State(state), Extension(outsider))
            .await
            .unwrap()
            .0;
        assert!(snapshot["nodes"].as_array().unwrap().is_empty());
        assert!(snapshot["workspaces"].as_array().unwrap().is_empty());
    }
    #[tokio::test]
    async fn desktop_bootstrap_sets_cookie_once_and_rejects_expiry() {
        let directory = tempfile::tempdir().unwrap();
        let state = create_state(
            directory.path().to_path_buf(),
            "http://{workspace}.localhost:8089".into(),
        )
        .await
        .unwrap();
        let key = token();
        *state.desktop_bootstrap.lock().await = Some((hash(&key), now() + 60));
        let response = desktop_bootstrap(State(state.clone()), Path(key.clone()))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], "/");
        assert!(response.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .contains("HttpOnly; SameSite=Strict"));
        assert_eq!(state.store.lock().await.db.sessions.len(), 1);
        assert!(desktop_bootstrap(State(state.clone()), Path(key.clone()))
            .await
            .is_err());
        *state.desktop_bootstrap.lock().await = Some((hash(&key), now() - 1));
        assert!(desktop_bootstrap(State(state), Path(key)).await.is_err());
    }
    #[test]
    fn app_origin_requires_workspace_subdomain() {
        assert!(validate_app_url("https://{workspace}.apps.example.com").is_ok());
        assert!(validate_app_url("http://{workspace}.localhost:8089").is_ok());
        assert!(validate_app_url("https://lab.example.com/{workspace}").is_err());
        assert!(validate_app_url("http://{workspace}.example.com").is_err());
    }
    #[test]
    fn expired_and_revoked_sessions_do_not_authorize() {
        let mut db = Database::default();
        db.sessions.push(Session {
            token_hash: "h".into(),
            access_id: Some("a".into()),
            expires_at: now() + 30,
        });
        assert!(session_actor(&db, "h").is_none());
        db.access.push(Access {
            id: "a".into(),
            lab_id: "l".into(),
            name: "A".into(),
            role: "viewer".into(),
            expires_at: now() + 30,
            token_hash: "x".into(),
        });
        let actor = session_actor(&db, "h").unwrap();
        assert!(actor.operate().is_err());
        assert!(actor.lab("elsewhere").is_err());
        assert!(actor.owner().is_err());
        db.access.clear();
        assert!(session_actor(&db, "h").is_none());
    }
}
