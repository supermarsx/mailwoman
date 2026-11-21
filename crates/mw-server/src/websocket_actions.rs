//! JSON actions use the unchanged HTTP router and a fresh request context.
//! Upgrade authority is an immutable actor anchor, never a cached grant/session.
mod contract;
pub(crate) mod cookie_delivery;
use crate::{AppState, admin_ctx::AdminCtx};
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::{
        ConnectInfo, FromRequestParts, State,
        ws::{Message, WebSocketUpgrade},
    },
    http::{HeaderMap, Request, StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tower::ServiceExt;
#[derive(Clone)]
pub(crate) struct Dispatcher(pub Router);
#[derive(Clone)]
pub(crate) struct Context {
    headers: HeaderMap,
    transport: Option<ConnectInfo<std::net::SocketAddr>>,
    dispatcher: Option<Dispatcher>,
}
impl<S: Send + Sync> FromRequestParts<S> for Context {
    type Rejection = Infallible;
    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self {
            headers: parts.headers.clone(),
            transport: parts
                .extensions
                .get::<ConnectInfo<std::net::SocketAddr>>()
                .cloned(),
            dispatcher: parts.extensions.get::<Dispatcher>().cloned(),
        })
    }
}
#[derive(Clone)]
enum Actor {
    Webmail(mw_store::WebsocketPrimaryActor),
    Admin(cookie_delivery::AdminActor),
}
tokio::task_local! {static FRAME:Actor;}
/// The registrar calls this with its own freshly resolved AdminCtx, not a
/// handshake extension. A webmail socket can never acquire an admin route.
pub(crate) fn admin_actor_ok(ctx: &AdminCtx) -> bool {
    FRAME
        .try_with(|actor| matches!(actor,Actor::Admin(expected) if expected.matches(ctx)))
        .unwrap_or(true)
}
pub(crate) async fn primary_actor_ok(state: &AppState, session: &mw_store::Session) -> bool {
    let actor = FRAME.try_with(Clone::clone).ok();
    match actor {
        None => true,
        Some(Actor::Admin(_)) => false,
        Some(Actor::Webmail(expected)) => {
            // A linked selected context never changes the socket's primary
            // cookie actor. Called before linked-account selection in authed.
            if expected.session_id() != session.id {
                return false;
            }
            state
                .store
                .websocket_primary_actor(&session.id)
                .await
                .ok()
                .flatten()
                .is_some_and(|current| current == expected)
        }
    }
}
pub(crate) fn has_frame_actor() -> bool {
    FRAME.try_with(|_| ()).is_ok()
}
pub(crate) fn native_actor_ok(
    store: &mw_store::Store,
    session: &mw_store::Session,
    principal: &str,
    binding: &str,
) -> bool {
    FRAME.try_with(|actor|matches!(actor,Actor::Webmail(expected) if expected.matches_native(store,session,principal,binding))).unwrap_or(true)
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Action {
    id: String,
    method: String,
    path: String,
    #[serde(default, deserialize_with = "unique_headers")]
    headers: HashMap<String, String>,
    #[serde(default)]
    body: Option<String>,
}
fn unique_headers<'de, D: serde::Deserializer<'de>>(
    decoder: D,
) -> Result<HashMap<String, String>, D::Error> {
    struct Headers;
    impl<'de> serde::de::Visitor<'de> for Headers {
        type Value = HashMap<String, String>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("bounded unique action headers")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut output = HashMap::new();
            let mut bytes = 0usize;
            while let Some((key, value)) = map.next_entry::<String, String>()? {
                bytes = bytes.saturating_add(key.len()).saturating_add(value.len());
                let key = key.to_ascii_lowercase();
                if output.len() >= contract::get().limits.headers
                    || bytes > contract::get().limits.header_bytes
                    || output.contains_key(&key)
                {
                    return Err(serde::de::Error::custom(
                        "duplicate or oversized action headers",
                    ));
                }
                output.insert(key, value);
            }
            Ok(output)
        }
    }
    decoder.deserialize_map(Headers)
}
fn refused(id: &str, status: StatusCode, http: bool) -> Value {
    let mut out = json!({"id":id,"actionOutcome":"notDispatched","notDispatched":true,"status":status.as_u16(),"headers":{"content-type":"application/json","cache-control":"no-store"},"body":json!({"code":"websocketActionRefused"}).to_string()});
    if http {
        out["httpRequired"] = json!(true);
    }
    out
}
fn completed_failure(id: &str, reauthenticate: bool) -> Value {
    json!({"id":id,"actionOutcome":"completed","status":502,"headers":{"content-type":"application/json","cache-control":"no-store"},"body":json!({"code":"websocketActionCompletedTransportFailure","reauthenticate":reauthenticate}).to_string()})
}
async fn enabled(state: &AppState, headers: &HeaderMap, admin: bool) -> bool {
    if admin {
        if AdminCtx::resolve(state, headers).await.is_err() {
            return false;
        }
        crate::notification_policy::allowed(
            &state.store,
            &Default::default(),
            "realtime.actions.enabled",
        )
        .await
        .unwrap_or(false)
    } else {
        let Ok(session) = crate::authed(state, headers).await else {
            return false;
        };
        let Ok(subject) =
            crate::settings_routes::UserSubject::resolved(&state.store, &session).await
        else {
            return false;
        };
        crate::notification_policy::allowed(
            &state.store,
            &subject.as_subject(),
            "realtime.actions.enabled",
        )
        .await
        .unwrap_or(false)
    }
}
pub(crate) async fn upgrade(
    State(state): State<AppState>,
    context: Context,
    ws: WebSocketUpgrade,
) -> Response {
    open(state, context, ws, false).await
}
pub(crate) async fn upgrade_admin(
    State(state): State<AppState>,
    context: Context,
    ws: WebSocketUpgrade,
) -> Response {
    open(state, context, ws, true).await
}
async fn open(state: AppState, context: Context, ws: WebSocketUpgrade, admin: bool) -> Response {
    if !context.headers.contains_key(header::ORIGIN)
        || !crate::hardening::origin_ok(&context.headers, None)
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    if !enabled(&state, &context.headers, admin).await {
        return StatusCode::FORBIDDEN.into_response();
    }
    let actor = if admin {
        let Ok(ctx) = AdminCtx::resolve(&state, &context.headers).await else {
            return StatusCode::UNAUTHORIZED.into_response();
        };
        if !cookie_delivery::AdminActor::bounded(&ctx) {
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
        Actor::Admin(cookie_delivery::AdminActor::new(&ctx))
    } else {
        let Ok(session) = crate::authed(&state, &context.headers).await else {
            return StatusCode::UNAUTHORIZED.into_response();
        };
        let Ok(Some(actor)) = state.store.websocket_primary_actor(&session.id).await else {
            return StatusCode::UNAUTHORIZED.into_response();
        };
        Actor::Webmail(actor)
    };
    let Some(dispatcher) = context.dispatcher.clone() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let retired = Arc::new(AtomicBool::new(false));
    let maximum = if admin {
        contract::get().admin.max_in_flight
    } else {
        contract::get().webmail.max_in_flight
    };
    ws.max_message_size(contract::get().limits.frame_bytes).on_upgrade(move|mut socket|async move {
        if socket.send(Message::Text(json!({"type":"ready","protocol":contract::get().protocol}).to_string().into())).await.is_err(){return;}
        let mut tasks=tokio::task::JoinSet::new();
        let mut ids=HashSet::new();
        loop {tokio::select! {
            frame=socket.recv(),if tasks.len()<maximum=>{
                let Some(Ok(frame))=frame else{return;};
                let Message::Text(text)=frame else{if matches!(frame,Message::Close(_)){return;}continue;};
                let action:Action=match serde_json::from_str(&text){Ok(v)=>v,Err(_)=>{let _=socket.send(Message::Close(None)).await;return;}};
                if action.id.is_empty()||action.id.len()>contract::get().limits.id_bytes||ids.contains(&action.id){let _=socket.send(Message::Close(None)).await;return;}
                let state=state.clone();let context=context.clone();let dispatcher=dispatcher.clone();let actor=actor.clone();let retired=retired.clone();
                // Bound even the preflight awaits BEFORE detaching ownership.
                let slot=match state.websocket_actions.dispatch_slot(){Some(slot)=>Some(slot),None=>{let reply=refused(&action.id,StatusCode::SERVICE_UNAVAILABLE,false);if socket.send(Message::Text(reply.to_string().into())).await.is_err(){return;}continue;}};
                let request_id=action.id.clone();ids.insert(request_id.clone());
                tasks.spawn(async move {
                    let payload=if admin {
                        // The owned dispatch is detached if the receiver closes:
                        // it keeps the pre-reserved bounded ticket through any
                        // rotation. No action is replayed to recover an ACK.
                        let id=action.id.clone();
                        owned_admin(actor,id,execute(state,context,dispatcher,action,retired,slot)).await
                    }else{FRAME.scope(actor,execute(state,context,dispatcher,action,retired,slot)).await};
                    (request_id,payload)
                });
            },
            reply=tasks.join_next(),if !tasks.is_empty()=>{
                let Some(Ok((id,payload)))=reply else{return;};ids.remove(&id);
                if socket.send(Message::Text(payload.to_string().into())).await.is_err(){return;}
            }
        }}
    }).into_response()
}

async fn owned_admin<F>(actor: Actor, id: String, future: F) -> Value
where
    F: std::future::Future<Output = Value> + Send + 'static,
{
    tokio::spawn(FRAME.scope(actor, future))
        .await
        .unwrap_or_else(
            |_| json!({"id":id,"actionOutcome":"unknown","status":502,"headers":{},"body":""}),
        )
}
fn pattern_matches(pattern: &str, path: &str) -> bool {
    let mut actual = path.split('/');
    for part in pattern.split('/') {
        if part.starts_with("{*") && part.ends_with('}') {
            return actual.next().is_some();
        }
        let Some(value) = actual.next() else {
            return false;
        };
        if part.starts_with('{') && part.ends_with('}') {
            if value.is_empty() {
                return false;
            }
        } else if part != value {
            return false;
        }
    }
    actual.next().is_none()
}
fn route_allowed(path: &str, method: &str, admin: bool) -> bool {
    let base = mw_admin::config::admin_path();
    if admin {
        let Some(suffix) = path.strip_prefix(&base).filter(|s| s.starts_with('/')) else {
            return false;
        };
        !contract::requires_http(suffix, method, true)
            && crate::admin_router::registry().iter().any(|e| {
                e.access != crate::admin_router::Access::Public
                    && e.method.as_str() == method
                    && pattern_matches(e.suffix, suffix)
            })
    } else {
        contract::webmail_path(path, &base) && !contract::requires_http(path, method, false)
    }
}
fn headers(context: &Context, action: &Action) -> Option<HeaderMap> {
    let limits = &contract::get().limits;
    if action.headers.len() > limits.headers
        || action
            .headers
            .iter()
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>()
            > limits.header_bytes
    {
        return None;
    }
    let mut headers = HeaderMap::new();
    // Only transport/source cookies survive upgrade. No auth/policy request
    // extensions, bearer credentials or stale account selection are copied.
    for (name, value) in &context.headers {
        if matches!(
            name.as_str(),
            "cookie" | "origin" | "host" | "user-agent" | "forwarded" | "x-real-ip"
        ) || name.as_str().starts_with("x-forwarded-")
        {
            headers.append(name.clone(), value.clone());
        }
    }
    let mut seen = HashSet::new();
    for (name, value) in &action.headers {
        let name = name.to_ascii_lowercase();
        if !seen.insert(name.clone()) {
            return None;
        }
        if !contract::get().request_headers.contains(&name) {
            return None;
        }
        headers.insert(
            axum::http::HeaderName::from_bytes(name.as_bytes()).ok()?,
            value.parse().ok()?,
        );
    }
    Some(headers)
}
async fn execute(
    state: AppState,
    context: Context,
    dispatcher: Dispatcher,
    action: Action,
    retired: Arc<AtomicBool>,
    _slot: Option<tokio::sync::OwnedSemaphorePermit>,
) -> Value {
    let id = &action.id;
    let limits = &contract::get().limits;
    if id.is_empty()
        || id.len() > limits.id_bytes
        || !contract::get().methods.contains(&action.method)
    {
        return refused(id, StatusCode::BAD_REQUEST, false);
    }
    let Some(target) = contract::target(&action.path) else {
        return refused(id, StatusCode::BAD_REQUEST, false);
    };
    let Some(mut headers) = headers(&context, &action) else {
        return refused(id, StatusCode::BAD_REQUEST, false);
    };
    let admin = FRAME.with(|actor| matches!(actor, Actor::Admin(_)));
    if !route_allowed(&target.path, &action.method, admin) {
        return refused(id, StatusCode::UNSUPPORTED_MEDIA_TYPE, true);
    }
    if retired.load(Ordering::Acquire) {
        return refused(id, StatusCode::UNAUTHORIZED, false);
    }
    let body = action.body.as_deref().unwrap_or("");
    if body.len() > limits.body_bytes {
        return refused(id, StatusCode::PAYLOAD_TOO_LARGE, false);
    }
    if !body.is_empty()
        && headers
            .get(header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .is_none_or(|h| h.split(';').next().map(str::trim) != Some("application/json"))
    {
        return refused(id, StatusCode::UNSUPPORTED_MEDIA_TYPE, true);
    }
    if !enabled(&state, &headers, admin).await {
        return refused(id, StatusCode::FORBIDDEN, false);
    }
    let ticket = if admin {
        let Ok(ctx) = AdminCtx::resolve(&state, &headers).await else {
            return refused(id, StatusCode::UNAUTHORIZED, false);
        };
        if !admin_actor_ok(&ctx) {
            return refused(id, StatusCode::UNAUTHORIZED, false);
        }
        let Some(origin) = headers.get(header::ORIGIN).and_then(|h| h.to_str().ok()) else {
            return refused(id, StatusCode::FORBIDDEN, false);
        };
        let Some(ticket) = state
            .websocket_actions
            .reserve(&ctx, origin, retired.clone())
        else {
            return refused(id, StatusCode::SERVICE_UNAVAILABLE, false);
        };
        Some(ticket)
    } else {
        None
    };
    headers.insert(
        header::CONTENT_LENGTH,
        body.len().to_string().parse().unwrap(),
    );
    let Ok(mut request) = Request::builder()
        .method(action.method.as_str())
        .uri(target.uri)
        .body(Body::from(body.to_owned()))
    else {
        return refused(id, StatusCode::BAD_REQUEST, false);
    };
    *request.headers_mut() = headers;
    if let Some(info) = context.transport {
        request.extensions_mut().insert(info);
    }
    if let Some(ticket) = &ticket {
        request.extensions_mut().insert(ticket.capture());
    }
    let response = dispatcher
        .0
        .oneshot(request)
        .await
        .unwrap_or_else(|never| match never {});
    #[cfg(test)]
    test_control::after_dispatch(&state, id, response.status()).await;
    let known_cookies = cookie_delivery::issued_cookies_match(&response);
    let had_cookies = response.headers().contains_key(header::SET_COOKIE);
    let delivery = ticket.and_then(|ticket| ticket.finish());
    let status = response.status().as_u16();
    let mut response_headers = HashMap::new();
    for name in [
        "content-type",
        "cache-control",
        "etag",
        "retry-after",
        "last-modified",
        "location",
    ] {
        if let Some(value) = response.headers().get(name).and_then(|v| v.to_str().ok()) {
            response_headers.insert(name.to_owned(), value.to_owned());
        }
    }
    // Redirects require an explicit transport contract; never let a captured
    // response redirect cookie delivery or trigger action replay.
    response_headers.remove("location");
    let json_response = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.starts_with("application/json") || v.starts_with("application/problem+json")
        });
    let bytes = to_bytes(response.into_body(), limits.response_bytes).await;
    let mut reply = match bytes.ok().and_then(|v| String::from_utf8(v.to_vec()).ok()) {
        Some(body)
            if (!had_cookies || (known_cookies && delivery.is_some()))
                && (json_response || body.is_empty()) =>
        {
            json!({"id":id,"actionOutcome":"completed","status":status,"headers":response_headers,"body":body})
        }
        _ => completed_failure(id, had_cookies),
    };
    if had_cookies {
        retired.store(true, Ordering::Release);
    }
    if let Some(delivery) = delivery {
        reply["cookieDelivery"] = delivery;
    }
    // JSON encoding can expand control characters sixfold. Count exact encoded
    // bytes without allocating a second oversized response; no header/ID leak.
    struct Count(usize);
    impl std::io::Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0 = self.0.saturating_add(b.len());
            if self.0 > contract::get().limits.reply_frame_bytes {
                return Err(std::io::Error::other("reply bound"));
            }
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut count = Count(0);
    if serde_json::to_writer(&mut count, &reply).is_err() {
        let delivery = reply.get("cookieDelivery").cloned();
        reply = completed_failure(id, false);
        if let Some(v) = delivery {
            reply["cookieDelivery"] = v;
        }
    }
    reply
}

pub(crate) fn register(r: &mut impl crate::admin_router::AdminRegistrar) {
    r.admin_endpoint(
        axum::http::Method::GET,
        "/actions",
        crate::admin_router::Access::SignedIn,
        upgrade_admin,
    );
    #[cfg(test)]
    r.admin_endpoint(
        axum::http::Method::POST,
        "/actions-test/pause",
        crate::capabilities::Capability::SettingsWrite,
        test_control::paused,
    );

    // Public here means capability-delivery proof, NOT public authorization:
    // prepare() mints the private marker, and deliver() fresh-validates it.
    r.admin_endpoint(
        axum::http::Method::POST,
        "/actions/cookies",
        crate::admin_router::Access::Public,
        cookie_delivery::deliver,
    );
}

#[cfg(test)]
pub(crate) mod test_control {
    use super::*;
    use std::cell::RefCell;
    pub struct Pause {
        pub entered: tokio::sync::Notify,
        pub release: tokio::sync::Notify,
        pub disable: bool,
        pub calls: std::sync::atomic::AtomicUsize,
    }
    tokio::task_local! {pub static STATE:RefCell<Option<AppState>>;pub static PAUSE:Arc<Pause>;}
    pub(crate) fn record_state(state: &AppState) {
        let _ = STATE.try_with(|slot| *slot.borrow_mut() = Some(state.clone()));
    }
    /// A real-wire fixture crosses Axum/upgrade tasks, so its ownership gate
    /// cannot be a task-local. No request extension or environment supplies it.
    pub(super) struct WirePause {
        pub entered: tokio::sync::Notify,
        pub release: tokio::sync::Semaphore,
        pub calls: std::sync::atomic::AtomicUsize,
    }
    type WireKey = (uuid::Uuid, String);
    fn wire_pauses() -> &'static std::sync::Mutex<HashMap<WireKey, std::sync::Weak<WirePause>>> {
        static PAUSES: std::sync::OnceLock<
            std::sync::Mutex<HashMap<WireKey, std::sync::Weak<WirePause>>>,
        > = std::sync::OnceLock::new();
        PAUSES.get_or_init(Default::default)
    }
    pub(super) struct WireLease {
        key: WireKey,
        pub pause: Arc<WirePause>,
    }
    impl Drop for WireLease {
        fn drop(&mut self) {
            if let Ok(mut entries) = wire_pauses().lock() {
                entries.remove(&self.key);
            }
            // Also unblocks a still-live owner on assertion panic/timeout.
            self.pause.release.close();
        }
    }
    pub(super) fn pause_wire(state: &AppState, id: &str) -> WireLease {
        assert!(!id.is_empty() && id.len() <= contract::get().limits.id_bytes);
        let key = (state.store.cache_scope(), id.to_owned());
        let pause = Arc::new(WirePause {
            entered: tokio::sync::Notify::new(),
            release: tokio::sync::Semaphore::new(0),
            calls: std::sync::atomic::AtomicUsize::new(0),
        });
        let mut entries = wire_pauses().lock().unwrap();
        entries.retain(|_, weak| weak.strong_count() > 0);
        assert!(entries.len() < 4 && !entries.contains_key(&key));
        entries.insert(key.clone(), Arc::downgrade(&pause));
        WireLease { key, pause }
    }
    pub(super) async fn after_dispatch(state: &AppState, id: &str, status: StatusCode) {
        let pause = wire_pauses().lock().ok().and_then(|entries| {
            entries
                .get(&(state.store.cache_scope(), id.to_owned()))
                .and_then(std::sync::Weak::upgrade)
        });
        let Some(pause) = pause else { return };
        assert!(status.is_success(), "real fixture mutation must succeed");
        pause.calls.fetch_add(1, Ordering::SeqCst);
        pause.entered.notify_one();
        let _ = pause.release.acquire().await;
    }
    pub(super) async fn paused(State(state): State<AppState>) -> Response {
        let pause = PAUSE.with(Clone::clone);
        pause.calls.fetch_add(1, Ordering::SeqCst);
        pause.entered.notify_one();
        pause.release.notified().await;
        if pause.disable {
            state
                .store
                .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
                    key: "realtime.actions.enabled".into(),
                    global_value: Some("false".into()),
                    allow_user_override: false,
                    global_locked: true,
                    updated_by: "ws-test".into(),
                    updated_at: chrono::Utc::now().timestamp(),
                })
                .await
                .unwrap();
        }
        axum::Json(json!({"actionApplied":true})).into_response()
    }
}
#[cfg(test)]
#[path = "websocket_actions/tests.rs"]
mod tests;
