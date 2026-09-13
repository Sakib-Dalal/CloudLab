//! Authenticated outbound node relay. Workspace apps use one origin per
//! container, separate from the coordinator and every other workspace.
use crate::{agent::Credentials, model::*, sandbox, server::*};
use axum::{
    body::{to_bytes, Body},
    extract::{
        ws::{Message, WebSocket},
        Path, Request, State, WebSocketUpgrade,
    },
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::any,
    Extension, Json, Router,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use futures_util::{SinkExt, StreamExt};
use http_body_util::{BodyExt, Full};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{mpsc, oneshot, Mutex, Semaphore};
use tokio_tungstenite::tungstenite::{self, client::IntoClientRequest};

const MAX_BODY: usize = 16 * 1024 * 1024;
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Envelope {
    Http {
        id: String,
        workspace: String,
        method: String,
        path: String,
        headers: Vec<(String, String)>,
        body: String,
    },
    HttpResult {
        id: String,
        status: u16,
        headers: Vec<(String, String)>,
        body: String,
    },
    WsOpen {
        id: String,
        workspace: String,
        path: String,
        headers: Vec<(String, String)>,
    },
    WsReady {
        id: String,
        protocol: Option<String>,
    },
    Data {
        id: String,
        data: String,
        binary: bool,
    },
    Close {
        id: String,
    },
}
impl Envelope {
    fn id(&self) -> &str {
        match self {
            Self::Http { id, .. }
            | Self::HttpResult { id, .. }
            | Self::WsOpen { id, .. }
            | Self::WsReady { id, .. }
            | Self::Data { id, .. }
            | Self::Close { id } => id,
        }
    }
}
type Sender = mpsc::Sender<Envelope>;
#[derive(Default)]
pub struct Relay {
    peers: Mutex<HashMap<String, (String, Sender)>>,
    pending: Mutex<HashMap<String, (String, oneshot::Sender<Envelope>)>>,
    streams: Mutex<HashMap<String, (String, Sender)>>,
}
impl Relay {
    pub async fn disconnect(&self, node: &str) {
        self.peers.lock().await.remove(node);
        self.pending.lock().await.retain(|_, (n, _)| n != node);
        self.streams.lock().await.retain(|_, (n, _)| n != node);
    }
    async fn sender(&self, node: &str) -> Result<Sender> {
        self.peers
            .lock()
            .await
            .get(node)
            .map(|(_, s)| s.clone())
            .ok_or_else(|| {
                ApiError(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "The node's workspace relay is offline. Try again after it reconnects.".into(),
                )
            })
    }
    async fn request(&self, node: &str, message: Envelope) -> Result<Envelope> {
        let key = message.id().to_string();
        let sender = self.sender(node).await?;
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            if pending.len() >= 128 {
                return Err(ApiError(
                    StatusCode::TOO_MANY_REQUESTS,
                    "The workspace gateway is busy.".into(),
                ));
            }
            pending.insert(key.clone(), (node.into(), tx));
        }
        if sender.send(message).await.is_err() {
            self.pending.lock().await.remove(&key);
            return Err(bad("The node disconnected."));
        }
        let result = tokio::time::timeout(Duration::from_secs(60), rx).await;
        self.pending.lock().await.remove(&key);
        result
            .map_err(|_| {
                ApiError(
                    StatusCode::GATEWAY_TIMEOUT,
                    "The workspace did not respond in time.".into(),
                )
            })?
            .map_err(|_| bad("The node disconnected."))
    }
    async fn incoming(&self, node: &str, message: Envelope) {
        let key = message.id().to_string();
        let mut pending = self.pending.lock().await;
        if pending.get(&key).is_some_and(|(n, _)| n == node) {
            if let Some((_, tx)) = pending.remove(&key) {
                let _ = tx.send(message);
                return;
            }
        }
        drop(pending);
        let sender = self
            .streams
            .lock()
            .await
            .get(&key)
            .filter(|(n, _)| n == node)
            .map(|(_, s)| s.clone());
        if let Some(sender) = sender {
            let _ = sender.send(message).await;
        }
    }
}
pub async fn agent_tunnel(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<Response> {
    let node = agent_node(&state, &headers).await?;
    Ok(ws
        .max_message_size(MAX_BODY * 2)
        .max_frame_size(MAX_BODY * 2)
        .on_upgrade(move |socket| coordinator_connection(state, node.id, socket))
        .into_response())
}
async fn coordinator_connection(state: AppState, node: String, socket: WebSocket) {
    let generation = id();
    let (tx, mut rx) = mpsc::channel(64);
    state
        .relay
        .peers
        .lock()
        .await
        .insert(node.clone(), (generation.clone(), tx));
    let (mut sink, mut source) = socket.split();
    let mut tick = tokio::time::interval(Duration::from_secs(10));
    loop {
        tokio::select! {
                    event=rx.recv()=>match event{Some(message)=>{if sink.send(Message::Text(serde_json::to_string(&message).unwrap().into())).await.is_err(){break;}},None=>break},
                    frame=source.next()=>match frame{Some(Ok(Message::Text(text)))=>if let Ok(message)=serde_json::from_str(&text){state.relay.incoming(&node,message).await;},Some(Ok(Message::Pong(_)))=>{let mut s=state.store.lock().await;if let Some(n)=s.db.nodes.iter_mut().find(|n|n.id==node&&!n.revoked){n.last_seen=now();}else{break;}},Some(Ok(Message::Ping(v)))=>{if sink.send(Message::Pong(v)).await.is_err(){break;}},Some(Ok(Message::Close(_)))|None|Some(Err(_))=>break,_=>{}},
                    _=tick.tick()=>{if !state.store.lock().await.db.nodes.iter().any(|n|n.id==node&&!n.revoked){break;}
        if sink.send(Message::Ping(vec![].into())).await.is_err(){break;}}
                }
    }
    let mut peers = state.relay.peers.lock().await;
    if peers.get(&node).is_some_and(|(g, _)| g == &generation) {
        peers.remove(&node);
        drop(peers);
        state
            .relay
            .pending
            .lock()
            .await
            .retain(|_, (n, _)| n != &node);
        state
            .relay
            .streams
            .lock()
            .await
            .retain(|_, (n, _)| n != &node);
    }
}

pub async fn open_workspace(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    actor.operate()?;
    let mut s = state.store.lock().await;
    let w =
        s.db.workspaces
            .iter_mut()
            .find(|w| w.id == id)
            .ok_or_else(missing)?;
    actor.lab(&w.lab_id)?;
    if w.status != "running" || w.template == "terminal" {
        return Err(bad(
            "This workspace does not have a running browser application.",
        ));
    }
    w.last_used = now();
    let ticket = token();
    s.db.app_sessions.push(AppSession {
        token_hash: hash(&ticket),
        workspace_id: id.clone(),
        session_hash: actor.session_hash,
        expires_at: now() + 60,
        ticket: true,
    });
    s.save()?;
    Ok(Json(
        json!({"url":format!("{}/?ticket={ticket}",app_origin(&state,&id))}),
    ))
}
fn app_origin(state: &AppState, id: &str) -> String {
    state
        .app_url
        .trim_end_matches('/')
        .replace("{workspace}", &format!("w-{id}"))
}
pub fn gateway_router(state: AppState) -> Router {
    Router::new()
        .route("/", any(gateway_response))
        .route("/{*path}", any(gateway_response))
        .with_state(state)
}
fn gateway_workspace(state: &AppState, headers: &HeaderMap) -> Result<String> {
    let authority = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(missing)?;
    let label = authority.split('.').next().ok_or_else(missing)?;
    let id = label
        .strip_prefix("w-")
        .filter(|id| valid_id(id))
        .ok_or_else(missing)?;
    let expected = url::Url::parse(&app_origin(state, id)).map_err(|_| missing())?;
    let host = if let Some(port) = expected.port() {
        format!("{}:{port}", expected.host_str().unwrap_or(""))
    } else {
        expected.host_str().unwrap_or("").into()
    };
    if !authority.eq_ignore_ascii_case(&host) {
        return Err(missing());
    }
    Ok(id.into())
}
fn app_cookie_name(state: &AppState) -> &'static str {
    if state.app_url.starts_with("https://") {
        "__Host-cloudlab_app"
    } else {
        "cloudlab_app"
    }
}
fn app_authorized(db: &Database, workspace: &str, session: &str) -> bool {
    db.app_sessions.iter().any(|s| {
        !s.ticket
            && s.token_hash == session
            && s.workspace_id == workspace
            && s.expires_at > now()
            && session_actor(db, &s.session_hash).is_some_and(|a| {
                a.operate().is_ok()
                    && a.lab(
                        &db.workspaces
                            .iter()
                            .find(|w| w.id == workspace)
                            .map(|w| w.lab_id.clone())
                            .unwrap_or_default(),
                    )
                    .is_ok()
            })
    })
}
async fn gateway(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: std::result::Result<
        WebSocketUpgrade,
        axum::extract::ws::rejection::WebSocketUpgradeRejection,
    >,
    req: Request,
) -> Result<Response> {
    let ws = ws.ok();
    let workspace = gateway_workspace(&state, &headers)?;
    let origin = app_origin(&state, &workspace);
    if matches!(
        *req.method(),
        axum::http::Method::GET | axum::http::Method::HEAD
    ) {
        if let Some(response) = crate::workspace_ui::asset(req.uri().path()) {
            return Ok(response);
        }
    }
    // Only the launch endpoint consumes CloudLab tickets. App routes may use
    // their own query parameters with the same name.
    let ticket = req
        .uri()
        .query()
        .filter(|_| req.uri().path() == "/")
        .and_then(|query| {
            url::form_urlencoded::parse(query.as_bytes())
                .find(|(k, _)| k == "ticket")
                .map(|(_, v)| v.into_owned())
        });
    if let Some(ticket) = ticket {
        if req.method() != axum::http::Method::GET {
            return Err(forbidden());
        }
        let mut s = state.store.lock().await;
        let index =
            s.db.app_sessions
                .iter()
                .position(|a| {
                    a.ticket
                        && a.token_hash == hash(&ticket)
                        && a.workspace_id == workspace
                        && a.expires_at > now()
                })
                .ok_or_else(unauthorized)?;
        let session = s.db.app_sessions[index].clone();
        let actor = session_actor(&s.db, &session.session_hash).ok_or_else(unauthorized)?;
        actor.operate()?;
        let new = token();
        s.db.app_sessions.remove(index);
        let expiry =
            s.db.sessions
                .iter()
                .find(|s| s.token_hash == actor.session_hash)
                .unwrap()
                .expires_at;
        s.db.app_sessions.push(AppSession {
            token_hash: hash(&new),
            workspace_id: workspace,
            session_hash: actor.session_hash,
            expires_at: expiry,
            ticket: false,
        });
        s.save()?;
        let mut r = Redirect::to("/_cloudlab/").into_response();
        r.headers_mut().insert(
            "set-cookie",
            HeaderValue::from_str(&format!(
                // A launch starts on the coordinator's different site. Lax
                // allows the cookie on this top-level GET redirect. Mutations
                // and WebSockets still require the exact workspace Origin.
                "{}={new}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}",
                app_cookie_name(&state),
                expiry.saturating_sub(now()),
                if origin.starts_with("https:") {
                    "; Secure"
                } else {
                    ""
                }
            ))
            .unwrap(),
        );
        r.headers_mut()
            .insert("referrer-policy", HeaderValue::from_static("no-referrer"));
        r.headers_mut()
            .insert("cache-control", HeaderValue::from_static("no-store"));
        return Ok(r);
    }
    let session = hash(&cookie(&headers, app_cookie_name(&state)).ok_or_else(unauthorized)?);
    let mut s = state.store.lock().await;
    if !app_authorized(&s.db, &workspace, &session) {
        return Err(unauthorized());
    }
    let w =
        s.db.workspaces
            .iter_mut()
            .find(|w| w.id == workspace && w.status == "running")
            .ok_or_else(missing)?;
    w.last_used = now();
    let w = w.clone();
    let node = w.node_id.clone();
    let n =
        s.db.nodes
            .iter()
            .find(|n| n.id == node && !n.revoked)
            .ok_or_else(forbidden)?;
    if req.uri().path().starts_with("/_cloudlab/") {
        if !matches!(
            *req.method(),
            axum::http::Method::GET | axum::http::Method::HEAD
        ) {
            return Err(forbidden());
        }
        let response = match req.uri().path() {
            "/_cloudlab/" => crate::workspace_ui::shell(),
            "/_cloudlab/workspace.json" => {
                let lab = s.db.labs.iter().find(|l| l.id == w.lab_id);
                crate::workspace_ui::metadata(
                    &w,
                    n,
                    lab.map(|l| l.name.as_str()).unwrap_or("CloudLab"),
                )
            }
            _ => return Err(missing()),
        };
        drop(s);
        // Metadata doubles as a connection check for an already-open shell.
        state.relay.sender(&node).await?;
        return Ok(response);
    }
    drop(s);
    let request_origin = headers.get("origin").and_then(|h| h.to_str().ok());
    if (ws.is_some()
        || !matches!(
            *req.method(),
            axum::http::Method::GET | axum::http::Method::HEAD
        ))
        && request_origin != Some(&origin)
    {
        return Err(forbidden());
    }
    let path = req
        .uri()
        .path_and_query()
        .map(|p| p.as_str())
        .unwrap_or("/")
        .to_string();
    let mut forwarded = forward_headers(&headers, ws.is_some());
    // Workspace auth cookies never reach the user's container.
    forwarded.retain(|(k, _)| k != "cookie");
    if let Some(c) = headers.get("cookie").and_then(|v| v.to_str().ok()) {
        let safe = c
            .split(';')
            .filter(|part| {
                !part.trim().starts_with("cloudlab_")
                    && !part.trim().starts_with("__Host-cloudlab_")
            })
            .collect::<Vec<_>>()
            .join(";");
        if !safe.is_empty() {
            forwarded.push(("cookie".into(), safe));
        }
    }
    if let Some(ws) = ws {
        let key = id();
        let (tx, rx) = mpsc::channel(64);
        if state.relay.streams.lock().await.len() >= 256 {
            return Err(ApiError(
                StatusCode::TOO_MANY_REQUESTS,
                "Too many workspace connections.".into(),
            ));
        }
        state
            .relay
            .streams
            .lock()
            .await
            .insert(key.clone(), (node.clone(), tx));
        let result = state
            .relay
            .request(
                &node,
                Envelope::WsOpen {
                    id: key.clone(),
                    workspace: workspace.clone(),
                    path,
                    headers: forwarded,
                },
            )
            .await;
        match result {
            Ok(Envelope::WsReady { protocol, .. }) => {
                let ws = if let Some(protocol) = protocol {
                    ws.protocols([protocol])
                } else {
                    ws
                };
                Ok(ws
                    .max_message_size(MAX_BODY)
                    .on_upgrade(move |socket| {
                        browser_connection(state, node, workspace, session, key, rx, socket)
                    })
                    .into_response())
            }
            _ => {
                state.relay.streams.lock().await.remove(&key);
                Err(ApiError(
                    StatusCode::BAD_GATEWAY,
                    "The workspace WebSocket could not be opened.".into(),
                ))
            }
        }
    } else {
        let method = req.method().to_string();
        let body = to_bytes(req.into_body(), MAX_BODY)
            .await
            .map_err(|_| bad("Workspace requests are limited to 16 MiB."))?;
        let response = state
            .relay
            .request(
                &node,
                Envelope::Http {
                    id: id(),
                    workspace,
                    method,
                    path,
                    headers: forwarded,
                    body: B64.encode(body),
                },
            )
            .await?;
        if let Envelope::HttpResult {
            status,
            headers,
            body,
            ..
        } = response
        {
            let bytes = B64.decode(body).map_err(|_| bad("Invalid node response"))?;
            if bytes.len() > MAX_BODY {
                return Err(bad("Workspace responses are limited to 16 MiB."));
            }
            let mut response = Response::new(Body::from(bytes));
            *response.status_mut() =
                StatusCode::from_u16(status).map_err(|_| bad("Invalid status"))?;
            for (k, v) in headers {
                if safe_response_header(&k, &v) {
                    if let (Ok(k), Ok(v)) = (
                        k.parse::<axum::http::HeaderName>(),
                        v.parse::<HeaderValue>(),
                    ) {
                        response.headers_mut().append(k, v);
                    }
                }
            }
            response
                .headers_mut()
                .insert("referrer-policy", HeaderValue::from_static("no-referrer"));
            response.headers_mut().insert(
                "x-content-type-options",
                HeaderValue::from_static("nosniff"),
            );
            response.headers_mut().insert(
                "content-security-policy",
                HeaderValue::from_static("frame-ancestors 'self'"),
            );
            response
                .headers_mut()
                .insert("x-frame-options", HeaderValue::from_static("SAMEORIGIN"));
            response
                .headers_mut()
                .insert("cache-control", HeaderValue::from_static("no-store"));
            Ok(response)
        } else {
            Err(bad("Invalid node response"))
        }
    }
}
async fn gateway_response(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: std::result::Result<
        WebSocketUpgrade,
        axum::extract::ws::rejection::WebSocketUpgradeRejection,
    >,
    req: Request,
) -> Response {
    let html = headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("text/html"));
    match gateway(State(state), headers, ws, req).await {
        Ok(response) => response,
        Err(error) if html => crate::workspace_ui::error(error),
        Err(error) => error.into_response(),
    }
}
fn forward_headers(headers: &HeaderMap, websocket: bool) -> Vec<(String, String)> {
    headers
        .iter()
        .filter_map(|(k, v)| {
            let key = k.as_str();
            if matches!(
                key,
                "authorization"
                    | "connection"
                    | "upgrade"
                    | "content-length"
                    | "transfer-encoding"
                    | "accept-encoding"
                    | "sec-websocket-key"
                    | "sec-websocket-version"
                    | "sec-websocket-extensions"
            ) || key.starts_with("x-forwarded-")
                || (!websocket && key == "sec-websocket-protocol")
            {
                return None;
            }
            v.to_str().ok().map(|v| (key.into(), v.into()))
        })
        .collect()
}
fn safe_response_header(key: &str, value: &str) -> bool {
    if matches!(
        key,
        "connection"
            | "transfer-encoding"
            | "content-length"
            | "content-security-policy"
            | "x-frame-options"
            | "clear-site-data"
            | "access-control-allow-origin"
            | "access-control-allow-credentials"
    ) {
        return false;
    }
    if key == "set-cookie" {
        let lower = value.to_lowercase();
        return !lower.starts_with("cloudlab_")
            && !lower.starts_with("__host-cloudlab_")
            && !lower.contains("domain=");
    }
    true
}
async fn browser_connection(
    state: AppState,
    node: String,
    workspace: String,
    session: String,
    key: String,
    mut incoming: mpsc::Receiver<Envelope>,
    socket: WebSocket,
) {
    let Ok(sender) = state.relay.sender(&node).await else {
        return;
    };
    let (mut sink, mut source) = socket.split();
    let mut tick = tokio::time::interval(Duration::from_secs(5));
    loop {
        tokio::select! {
                    message=incoming.recv()=>match message{Some(Envelope::Data{data,binary,..})=>{let Ok(bytes)=B64.decode(data)else{break;};let frame=if binary{Message::Binary(bytes.into())}else{let Ok(text)=String::from_utf8(bytes)else{break;};Message::Text(text.into())};if sink.send(frame).await.is_err(){break;}},Some(Envelope::Close{..})|None=>break,_=>{}},
                    frame=source.next()=>{let envelope=match frame{Some(Ok(Message::Binary(data)))=>Envelope::Data{id:key.clone(),data:B64.encode(data),binary:true},Some(Ok(Message::Text(text)))=>Envelope::Data{id:key.clone(),data:B64.encode(text.as_bytes()),binary:false},Some(Ok(Message::Ping(data)))=>{if sink.send(Message::Pong(data)).await.is_err(){break;}continue;},Some(Ok(Message::Pong(_)))=>continue,_=>break};if sender.send(envelope).await.is_err(){break;}},
                    _=tick.tick()=>{let mut s=state.store.lock().await;if !app_authorized(&s.db,&workspace,&session)||!s.db.nodes.iter().any(|n|n.id==node&&!n.revoked){break;}
        if let Some(w)=s.db.workspaces.iter_mut().find(|w|w.id==workspace){w.last_used=now();}}
                }
    }
    state.relay.streams.lock().await.remove(&key);
    let _ = sender.send(Envelope::Close { id: key }).await;
    let _ = sink.send(Message::Close(None)).await;
}

pub async fn agent_connection(credentials: Credentials) -> anyhow::Result<()> {
    let url = format!("{}/api/agent/tunnel", credentials.coordinator).replacen("http", "ws", 1);
    let mut request = url.into_client_request()?;
    request.headers_mut().insert(
        "authorization",
        format!("Bearer {}", credentials.token).parse()?,
    );
    let config = tungstenite::protocol::WebSocketConfig::default()
        .max_message_size(Some(MAX_BODY * 2))
        .max_frame_size(Some(MAX_BODY * 2));
    let (socket, _) =
        tokio_tungstenite::connect_async_with_config(request, Some(config), false).await?;
    let (mut sink, mut source) = socket.split();
    let (tx, mut rx) = mpsc::channel::<Envelope>(64);
    let streams: Arc<Mutex<HashMap<String, Sender>>> = Arc::new(Mutex::new(HashMap::new()));
    let permits = Arc::new(Semaphore::new(32));
    loop {
        tokio::select! {
            message=rx.recv()=>{let Some(message)=message else{break;};sink.send(tungstenite::Message::Text(serde_json::to_string(&message)?.into())).await?;},
            frame=source.next()=>match frame {
                Some(Ok(tungstenite::Message::Text(text)))=>{let message:Envelope=serde_json::from_str(&text)?;match message {
                    Envelope::Http{..}|Envelope::WsOpen{..}=>{let tx=tx.clone();let node=credentials.id.clone();let streams=streams.clone();let permit=permits.clone().try_acquire_owned();let key=message.id().to_string();match permit{Ok(permit)=>{tokio::spawn(async move{let _permit=permit;let is_ws=matches!(message,Envelope::WsOpen{..});if let Err(e)=agent_request(message,&node,tx.clone(),streams).await {if is_ws{let _=tx.send(Envelope::Close{id:key}).await;}else{let _=tx.send(Envelope::HttpResult{id:key,status:502,headers:vec![("content-type".into(),"text/plain".into())],body:B64.encode(format!("The container application is not ready: {e}"))}).await;}}});},Err(_)=>{tx.send(Envelope::Close{id:key}).await?;}}},
                    Envelope::Data{..}|Envelope::Close{..}=>{let sender=streams.lock().await.get(message.id()).cloned();if let Some(sender)=sender{let _=sender.send(message).await;}},_=>{}
                }},
                Some(Ok(tungstenite::Message::Ping(v)))=>sink.send(tungstenite::Message::Pong(v)).await?,
                Some(Ok(tungstenite::Message::Close(_)))|None=>break,
                Some(Err(e))=>return Err(e.into()),_=>{}
            }
        }
    }
    streams.lock().await.clear();
    Ok(())
}
async fn agent_request(
    message: Envelope,
    node: &str,
    tx: Sender,
    streams: Arc<Mutex<HashMap<String, Sender>>>,
) -> anyhow::Result<()> {
    match message {
        Envelope::Http {
            id,
            workspace,
            method,
            path,
            headers,
            body,
        } => {
            anyhow::ensure!(
                path.starts_with('/') && !path.contains(['\r', '\n']),
                "Invalid application path"
            );
            let stream = sandbox::app_stream(&workspace, node).await?;
            let (mut sender, connection) =
                hyper::client::conn::http1::handshake(TokioIo::new(stream)).await?;
            tokio::spawn(async move {
                let _ = tokio::time::timeout(Duration::from_secs(60), connection).await;
            });
            let mut request = hyper::Request::builder().method(method.as_str()).uri(path);
            for (k, v) in headers {
                request = request.header(k, v);
            }
            let body = B64.decode(body)?;
            anyhow::ensure!(body.len() <= MAX_BODY, "Request too large");
            let mut response = tokio::time::timeout(
                Duration::from_secs(55),
                sender.send_request(request.body(Full::new(axum::body::Bytes::from(body)))?),
            )
            .await??;
            let status = response.status().as_u16();
            let headers = response
                .headers()
                .iter()
                .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.into())))
                .collect();
            let mut bytes = Vec::new();
            while let Some(frame) =
                tokio::time::timeout(Duration::from_secs(55), response.body_mut().frame()).await?
            {
                if let Ok(chunk) = frame?.into_data() {
                    anyhow::ensure!(
                        bytes.len() + chunk.len() <= MAX_BODY,
                        "Response exceeds 16 MiB"
                    );
                    bytes.extend_from_slice(&chunk);
                }
            }
            tx.send(Envelope::HttpResult {
                id,
                status,
                headers,
                body: B64.encode(bytes),
            })
            .await?;
        }
        Envelope::WsOpen {
            id,
            workspace,
            path,
            headers,
        } => {
            anyhow::ensure!(
                path.starts_with('/') && !path.contains(['\r', '\n']),
                "Invalid application path"
            );
            let stream = sandbox::app_stream(&workspace, node).await?;
            let mut request = format!("ws://127.0.0.1:8080{path}").into_client_request()?;
            for (k, v) in headers {
                if !matches!(
                    k.as_str(),
                    "sec-websocket-key" | "sec-websocket-version" | "connection" | "upgrade"
                ) {
                    request
                        .headers_mut()
                        .insert(k.parse::<tungstenite::http::HeaderName>()?, v.parse()?);
                }
            }
            let (socket, response) = tokio::time::timeout(
                Duration::from_secs(15),
                tokio_tungstenite::client_async(request, stream),
            )
            .await??;
            let protocol = response
                .headers()
                .get("sec-websocket-protocol")
                .and_then(|s| s.to_str().ok())
                .map(str::to_string);
            let (mut sink, mut source) = socket.split();
            let (sender, mut receiver) = mpsc::channel(64);
            streams.lock().await.insert(id.clone(), sender);
            tx.send(Envelope::WsReady {
                id: id.clone(),
                protocol,
            })
            .await?;
            loop {
                tokio::select! {
                                    frame=source.next()=>match frame{Some(Ok(tungstenite::Message::Text(text)))=>{if text.len()>MAX_BODY{break;}
                if tx.send(Envelope::Data{id:id.clone(),data:B64.encode(text.as_bytes()),binary:false}).await.is_err(){break;}},Some(Ok(tungstenite::Message::Binary(data)))=>{if data.len()>MAX_BODY{break;}
                if tx.send(Envelope::Data{id:id.clone(),data:B64.encode(data),binary:true}).await.is_err(){break;}},Some(Ok(tungstenite::Message::Ping(v)))=>{if sink.send(tungstenite::Message::Pong(v)).await.is_err(){break;}},Some(Ok(tungstenite::Message::Pong(_)))=>{},_=>break},
                                    message=receiver.recv()=>match message{Some(Envelope::Data{data,binary,..})=>{let bytes=B64.decode(data)?;anyhow::ensure!(bytes.len()<=MAX_BODY,"Frame too large");let message=if binary{tungstenite::Message::Binary(bytes.into())}else{tungstenite::Message::Text(String::from_utf8(bytes)?.into())};if sink.send(message).await.is_err(){break;}},_=>break}
                                }
            }
            streams.lock().await.remove(&id);
            let _ = tx.send(Envelope::Close { id }).await;
            let _ = sink.close().await;
        }
        _ => anyhow::bail!("Unexpected relay request"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;

    async fn fixture() -> (
        tempfile::TempDir,
        AppState,
        String,
        Actor,
        mpsc::Receiver<Envelope>,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let state = create_state(
            dir.path().into(),
            "http://{workspace}.localhost:8089".into(),
        )
        .await
        .unwrap();
        let workspace = id();
        let node = id();
        let actor = Actor {
            role: "operator".into(),
            lab_id: Some("lab".into()),
            session_hash: hash("parent"),
        };
        {
            let mut store = state.store.lock().await;
            store.db.access.push(Access {
                id: "access".into(),
                lab_id: "lab".into(),
                name: "Operator".into(),
                role: "operator".into(),
                expires_at: now() + 3600,
                token_hash: hash("key"),
            });
            store.db.sessions.push(Session {
                token_hash: actor.session_hash.clone(),
                access_id: Some("access".into()),
                expires_at: now() + 3600,
            });
            store.db.workspaces.push(Workspace {
                id: workspace.clone(),
                lab_id: "lab".into(),
                node_id: node.clone(),
                name: "Notebook <test>".into(),
                template: "jupyter".into(),
                cpus: 1,
                memory_mb: 1024,
                status: "running".into(),
                created_at: now(),
                last_used: now(),
                error: String::new(),
                network: false,
                gpu_ids: Vec::new(),
                metrics: None,
                history: Vec::new(),
            });
            store.db.nodes.push(Node {
                id: node.clone(),
                lab_id: "lab".into(),
                name: "Test node".into(),
                platform: "Linux".into(),
                arch: "arm64".into(),
                cpus: 4,
                memory_mb: 8192,
                cpu_usage: 0.,
                memory_used_mb: 0,
                last_seen: now(),
                docker: true,
                revoked: false,
                gpus: Vec::new(),
                metrics: None,
                history: Vec::new(),
                credential_hash: hash("node-secret"),
            });
        }
        let (tx, rx) = mpsc::channel(8);
        state.relay.peers.lock().await.insert(node, (id(), tx));
        (dir, state, workspace, actor, rx)
    }
    async fn launch(state: &AppState, workspace: &str, actor: Actor) -> String {
        let Json(result) = open_workspace(
            State(state.clone()),
            Extension(actor),
            Path(workspace.into()),
        )
        .await
        .unwrap();
        let url = url::Url::parse(result["url"].as_str().unwrap()).unwrap();
        format!("/?{}", url.query().unwrap())
    }
    async fn request(
        state: &AppState,
        workspace: &str,
        path: &str,
        cookie: &str,
        method: &str,
        origin: Option<&str>,
        html: bool,
    ) -> Response {
        let mut req = Request::builder()
            .uri(path)
            .method(method)
            .header("host", format!("w-{workspace}.localhost:8089"))
            .header("cookie", cookie);
        if let Some(origin) = origin {
            req = req.header("origin", origin);
        }
        if html {
            req = req.header("accept", "text/html");
        }
        gateway_router(state.clone())
            .oneshot(req.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }
    #[tokio::test]
    async fn launch_cookie_survives_cross_site_navigation_and_is_scoped() {
        let (_dir, state, workspace, actor, _rx) = fixture().await;
        let path = launch(&state, &workspace, actor).await;
        let response = request(&state, &workspace, &path, "", "GET", None, false).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], "/_cloudlab/");
        let cookie = response.headers()["set-cookie"].to_str().unwrap();
        assert!(cookie.contains("Path=/; HttpOnly; SameSite=Lax"));
        assert!(!cookie.contains("Domain="));
        let cookie = cookie.split(';').next().unwrap();
        assert_eq!(
            request(&state, &workspace, &path, "", "GET", None, false)
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
        let shell = request(&state, &workspace, "/_cloudlab/", cookie, "GET", None, true).await;
        assert_eq!(shell.status(), StatusCode::OK);
        assert_eq!(shell.headers()["x-frame-options"], "SAMEORIGIN");
        let body = to_bytes(shell.into_body(), MAX_BODY).await.unwrap();
        assert!(String::from_utf8_lossy(&body).contains("CloudLab"));
        let metadata = request(
            &state,
            &workspace,
            "/_cloudlab/workspace.json",
            cookie,
            "GET",
            None,
            false,
        )
        .await;
        let body = to_bytes(metadata.into_body(), MAX_BODY).await.unwrap();
        let info: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(info["name"], "Notebook <test>");
        assert_eq!(info["node"], "Test node");
        assert!(!String::from_utf8_lossy(&body).contains("secret"));
        assert_eq!(
            request(&state, &id(), "/_cloudlab/", cookie, "GET", None, false)
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
        for origin in [None, Some("http://evil.example")] {
            assert_eq!(
                request(
                    &state,
                    &workspace,
                    "/api/contents",
                    cookie,
                    "POST",
                    origin,
                    false
                )
                .await
                .status(),
                StatusCode::FORBIDDEN
            );
        }
        state.store.lock().await.db.access.clear();
        assert_eq!(
            request(
                &state,
                &workspace,
                "/_cloudlab/workspace.json",
                cookie,
                "GET",
                None,
                false
            )
            .await
            .status(),
            StatusCode::UNAUTHORIZED
        );
    }
    #[tokio::test]
    async fn launch_rejects_expired_tickets_and_revoked_parent_sessions() {
        let (_dir, state, workspace, actor, _rx) = fixture().await;
        let path = launch(&state, &workspace, actor.clone()).await;
        state.store.lock().await.db.app_sessions[0].expires_at = now() - 1;
        assert_eq!(
            request(&state, &workspace, &path, "", "GET", None, false)
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
        let path = launch(&state, &workspace, actor).await;
        state.store.lock().await.db.sessions.clear();
        assert_eq!(
            request(&state, &workspace, &path, "", "GET", None, false)
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }
    #[tokio::test]
    async fn https_launch_keeps_host_only_secure_cookie() {
        let (_dir, mut state, workspace, actor, _rx) = fixture().await;
        state.app_url = "https://{workspace}.apps.example.com".into();
        let path = launch(&state, &workspace, actor).await;
        let request = Request::builder()
            .uri(path)
            .header("host", format!("w-{workspace}.apps.example.com"))
            .body(Body::empty())
            .unwrap();
        let response = gateway_router(state).oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let cookie = response.headers()["set-cookie"].to_str().unwrap();
        assert!(cookie.starts_with("__Host-cloudlab_app="));
        assert!(cookie.contains("HttpOnly; SameSite=Lax"));
        assert!(cookie.ends_with("; Secure"));
        assert!(!cookie.contains("Domain="));
    }
    #[tokio::test]
    async fn stopped_workspaces_and_offline_nodes_do_not_report_connected() {
        let (_dir, state, workspace, actor, _rx) = fixture().await;
        let path = launch(&state, &workspace, actor).await;
        let response = request(&state, &workspace, &path, "", "GET", None, false).await;
        let cookie = response.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap();
        state.relay.peers.lock().await.clear();
        assert_eq!(
            request(
                &state,
                &workspace,
                "/_cloudlab/workspace.json",
                cookie,
                "GET",
                None,
                false
            )
            .await
            .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        state.store.lock().await.db.workspaces[0].status = "stopped".into();
        assert_eq!(
            request(
                &state,
                &workspace,
                "/_cloudlab/workspace.json",
                cookie,
                "GET",
                None,
                false
            )
            .await
            .status(),
            StatusCode::NOT_FOUND
        );
    }
    #[tokio::test]
    async fn gateway_errors_are_branded_for_documents_and_json_for_apis() {
        let (_dir, state, workspace, _actor, _rx) = fixture().await;
        let response = request(&state, &workspace, "/", "", "GET", None, true).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(response.headers()["content-type"]
            .to_str()
            .unwrap()
            .contains("text/html"));
        let body = to_bytes(response.into_body(), MAX_BODY).await.unwrap();
        assert!(String::from_utf8_lossy(&body).contains("Open this workspace from CloudLab"));
        assert!(!String::from_utf8_lossy(&body).contains("Enter a valid lab access key"));
        let response = request(&state, &workspace, "/api/status", "", "GET", None, false).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response.headers()["content-type"], "application/json");
    }
    #[test]
    fn container_cannot_set_auth_or_parent_cookies() {
        assert!(!safe_response_header(
            "set-cookie",
            "cloudlab_app=x; Path=/"
        ));
        assert!(!safe_response_header(
            "set-cookie",
            "__Host-cloudlab_app=x; Secure; Path=/"
        ));
        assert!(!safe_response_header(
            "set-cookie",
            "x=y; Domain=example.com"
        ));
        assert!(safe_response_header("set-cookie", "_xsrf=123; Path=/"));
    }
    #[test]
    fn authorization_is_not_forwarded() {
        let mut h = HeaderMap::new();
        h.insert("authorization", HeaderValue::from_static("Bearer secret"));
        h.insert("x-forwarded-host", HeaderValue::from_static("evil"));
        h.insert("content-type", HeaderValue::from_static("application/json"));
        let result = forward_headers(&h, false);
        assert_eq!(
            result,
            vec![("content-type".into(), "application/json".into())]
        );
    }
}
