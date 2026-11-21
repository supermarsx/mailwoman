//! Bounded, server-only delivery of a cookie rotation performed by an action.
//! A receipt completes transport; it never authorizes or replays the action.
use super::contract;
use crate::{AppState, admin_ctx::AdminCtx};
use axum::{
    Json,
    body::{Body, to_bytes},
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

#[derive(Clone, PartialEq, Eq)]
pub(super) struct AdminActor {
    pub username: String,
    pub account: Option<String>,
    pub session_hash: String,
}
impl AdminActor {
    pub(super) fn bounded(ctx: &AdminCtx) -> bool {
        ctx.username.len() <= 1024
            && ctx.account_id.as_ref().is_none_or(|a| a.len() <= 1024)
            && ctx.session_hash.len() == 64
            && ctx.socket_grants_fingerprint().is_some()
    }
    pub(super) fn new(ctx: &AdminCtx) -> Self {
        Self {
            username: ctx.username.clone(),
            account: ctx.account_id.clone(),
            session_hash: ctx.session_hash.clone(),
        }
    }
    pub(super) fn matches(&self, ctx: &AdminCtx) -> bool {
        self.username == ctx.username
            && self.account == ctx.account_id
            && self.session_hash == ctx.session_hash
    }
}
#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
struct Grant {
    token: String,
    absolute: i64,
}
struct Receipt {
    actor: AdminActor,
    grants: [u8; 32],
    origin: String,
    retired: Arc<AtomicBool>,
    ready: Option<Ready>,
}
struct Ready {
    ciphertext: Vec<u8>,
    new_hash: String,
    deadline: Instant,
    delivered: bool,
}
pub(crate) struct Service {
    receipts: Mutex<HashMap<String, Receipt>>,
    closing: AtomicBool,
    slots: Arc<tokio::sync::Semaphore>,
}
impl Service {
    pub(crate) fn new() -> Arc<Self> {
        let service = Arc::new(Self {
            receipts: Mutex::new(HashMap::new()),
            closing: AtomicBool::new(false),
            slots: Arc::new(tokio::sync::Semaphore::new(
                contract::get().cookie_handoff.capacity,
            )),
        });
        if let Ok(mut registry) = registry().lock() {
            registry.retain(|s| s.strong_count() > 0);
            if registry.len() < 64 {
                registry.push(Arc::downgrade(&service));
            } else {
                service.closing.store(true, Ordering::Release);
            }
        } else {
            service.closing.store(true, Ordering::Release);
        }
        service
    }
    pub(super) fn dispatch_slot(&self) -> Option<tokio::sync::OwnedSemaphorePermit> {
        if self.closing.load(Ordering::Acquire) {
            return None;
        }
        let permit = self.slots.clone().try_acquire_owned().ok()?;
        // If shutdown raced the initial check, either it observes this already
        // held permit or this second check refuses before any action executes.
        if self.closing.load(Ordering::Acquire) {
            return None;
        }
        Some(permit)
    }
    pub(super) fn reserve(
        self: &Arc<Self>,
        ctx: &AdminCtx,
        origin: &str,
        retired: Arc<AtomicBool>,
    ) -> Option<Ticket> {
        if self.closing.load(Ordering::Acquire) || origin.len() > 2048 || !AdminActor::bounded(ctx)
        {
            return None;
        }
        let grants = ctx.socket_grants_fingerprint()?;
        let token = mw_store::ServerKey::generate().to_hex();
        let digest = crate::push_relay::hash_token(&token);
        let mut map = self.receipts.lock().ok()?;
        map.retain(|_, r| r.ready.as_ref().is_none_or(|r| r.deadline > Instant::now()));
        if map.len() >= contract::get().cookie_handoff.capacity
            || map
                .values()
                .any(|r| r.actor.session_hash == ctx.session_hash)
        {
            return None;
        }
        map.insert(
            digest.clone(),
            Receipt {
                actor: AdminActor::new(ctx),
                grants,
                origin: origin.into(),
                retired,
                ready: None,
            },
        );
        Some(Ticket {
            service: self.clone(),
            digest,
            token,
            keep: false,
        })
    }
    /// Bounded shutdown does not claim that an outstanding mutation was rolled
    /// back. Ready receipts can be dropped only when their process exits.
    pub(crate) async fn shutdown(&self) -> bool {
        self.closing.store(true, Ordering::Release);
        let until = tokio::time::Instant::now() + Duration::from_secs(2);
        loop {
            let active = self
                .receipts
                .lock()
                .map(|m| m.values().any(|r| r.ready.is_none()))
                .unwrap_or(true);
            if !active && self.slots.available_permits() == contract::get().cookie_handoff.capacity
            {
                return true;
            }
            if tokio::time::Instant::now() >= until {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}
/// Only admin_rotation receives a Capture. It cannot adopt arbitrary response
/// headers or widen cookie name/path/domain, and contains no cookie plaintext.
#[derive(Clone)]
pub(crate) struct Capture {
    service: Arc<Service>,
    digest: String,
}
pub(super) struct Ticket {
    service: Arc<Service>,
    digest: String,
    token: String,
    keep: bool,
}
impl Drop for Ticket {
    fn drop(&mut self) {
        if !self.keep {
            if let Ok(mut m) = self.service.receipts.lock() {
                m.remove(&self.digest);
            }
        }
    }
}
impl Ticket {
    pub(super) fn capture(&self) -> Capture {
        Capture {
            service: self.service.clone(),
            digest: self.digest.clone(),
        }
    }
    pub(super) fn finish(mut self) -> Option<serde_json::Value> {
        let remaining = self.service.receipts.lock().ok().and_then(|m| {
            m.get(&self.digest)
                .and_then(|r| r.ready.as_ref())
                .and_then(|r| r.deadline.checked_duration_since(Instant::now()))
        });
        if let Some(remaining) = remaining.filter(|d| d.as_millis() > 0) {
            self.keep = true;
            Some(
                serde_json::json!({"token":self.token,"expiresInMs":remaining.as_millis().min(u64::MAX as u128) as u64}),
            )
        } else {
            None
        }
    }
}
fn origin(headers: &HeaderMap) -> Option<&str> {
    headers.get(header::ORIGIN)?.to_str().ok()
}
fn delivery_path() -> String {
    format!(
        "{}{}",
        mw_admin::config::admin_path(),
        contract::get().admin.cookie_delivery_suffix
    )
}
fn refusal() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({"code":"cookieDeliveryRefused","reauthenticate":true})),
    )
        .into_response()
}
fn with_cookie(headers: &HeaderMap, token: &str, csrf: &str) -> Option<HeaderMap> {
    let name = mw_admin::config::session_cookie_name();
    let mut parts = Vec::new();
    for raw in headers.get_all(header::COOKIE).iter() {
        let raw = raw.to_str().ok()?;
        for item in raw.split(';') {
            let (key, _) = item.trim().split_once('=')?;
            if key != name && key != crate::hardening::CSRF_COOKIE {
                parts.push(item.trim().to_owned());
            }
        }
    }
    parts.push(format!("{name}={token}"));
    parts.push(format!("{}={csrf}", crate::hardening::CSRF_COOKIE));
    let mut out = headers.clone();
    out.remove(header::COOKIE);
    out.insert(header::COOKIE, parts.join("; ").parse().ok()?);
    out.insert(crate::hardening::CSRF_HEADER, csrf.parse().ok()?);
    out.insert(crate::admin_ctx::PEEK_HEADER, "1".parse().ok()?);
    Some(out)
}
async fn fresh(
    state: &AppState,
    headers: &HeaderMap,
    grant: &Grant,
    actor: &AdminActor,
    fingerprint: [u8; 32],
) -> Option<HeaderMap> {
    if grant.absolute <= chrono::Utc::now().timestamp() {
        return None;
    }
    let headers = with_cookie(
        headers,
        &grant.token,
        &state.store.session_csrf_token(&grant.token),
    )?;
    let ctx = AdminCtx::resolve(state, &headers).await.ok()?;
    if ctx.username != actor.username
        || ctx.account_id != actor.account
        || ctx.session_hash != crate::push_relay::hash_token(&grant.token)
        || ctx.socket_grants_fingerprint() != Some(fingerprint)
    {
        return None;
    }
    Some(headers)
}
impl Capture {
    /// Called exclusively by the canonical issuer after its SQL rotation. A
    /// failure leaves the action completed but requires re-login, never replay.
    pub(crate) async fn issued(
        &self,
        state: &AppState,
        headers: &HeaderMap,
        token: &str,
        absolute: i64,
    ) -> Option<(IssuedCookies, Vec<axum::http::HeaderValue>)> {
        let Some((actor, fingerprint, expected_origin)) =
            self.service.receipts.lock().ok().and_then(|m| {
                m.get(&self.digest)
                    .map(|r| (r.actor.clone(), r.grants, r.origin.clone()))
            })
        else {
            return None;
        };
        if origin(headers) != Some(expected_origin.as_str()) {
            return None;
        }
        if let Ok(map) = self.service.receipts.lock() {
            if let Some(r) = map.get(&self.digest) {
                r.retired.store(true, Ordering::Release);
            }
        }
        let grant = Grant {
            token: token.into(),
            absolute,
        };
        if fresh(state, headers, &grant, &actor, fingerprint)
            .await
            .is_none()
        {
            return None;
        }
        // Two fresh resolutions straddle the issuer's async policy/session work;
        // neither the old identity nor cached grants authorize a new receipt.
        if fresh(state, headers, &grant, &actor, fingerprint)
            .await
            .is_none()
        {
            return None;
        }
        let Ok(bytes) = serde_json::to_vec(&grant).map(Zeroizing::new) else {
            return None;
        };
        let Ok(ciphertext) = state
            .store
            .seal_websocket_cookie_handoff(&self.digest, &bytes)
        else {
            return None;
        };
        if ciphertext.len() > contract::get().cookie_handoff.max_sealed_bytes {
            return None;
        }
        let Ok(mut map) = self.service.receipts.lock() else {
            return None;
        };
        let Some(receipt) = map.get_mut(&self.digest) else {
            return None;
        };
        receipt.retired.store(true, Ordering::Release);
        receipt.ready = Some(Ready {
            ciphertext,
            new_hash: crate::push_relay::hash_token(token),
            deadline: Instant::now()
                + Duration::from_millis(contract::get().cookie_handoff.receipt_ttl_ms),
            delivered: false,
        });
        let values = crate::admin_rotation::cookies(state, token, absolute);
        let marker = IssuedCookies::new(&values)?;
        Some((marker, values))
    }
}
#[derive(Clone)]
struct Prepared {
    digest: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Delivery {
    token: String,
}
fn read(state: &AppState, digest: &str) -> Option<(AdminActor, [u8; 32], String, String, Grant)> {
    let (actor, fp, origin, new_hash, ciphertext) =
        state.websocket_actions.receipts.lock().ok().and_then(|m| {
            let r = m.get(digest)?;
            let ready = r.ready.as_ref()?;
            (ready.deadline > Instant::now()).then(|| {
                (
                    r.actor.clone(),
                    r.grants,
                    r.origin.clone(),
                    ready.new_hash.clone(),
                    ready.ciphertext.clone(),
                )
            })
        })?;
    let bytes = Zeroizing::new(
        state
            .store
            .open_websocket_cookie_handoff(digest, &ciphertext)
            .ok()?,
    );
    let grant: Grant = serde_json::from_slice(&bytes).ok()?;
    Some((actor, fp, origin, new_hash, grant))
}
/// Runs before identity-consuming guards, on this ONE exact control path only.
/// Actual old/new cookie + Origin + double-submit proof is checked before any
/// server-side identity translation; normal guards then see the issued identity.
pub(crate) async fn prepare(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if request.uri().path() != delivery_path() {
        return next.run(request).await;
    }
    if request.method() != axum::http::Method::POST || request.uri().query().is_some() {
        return refusal();
    }
    if !crate::hardening::origin_ok(request.headers(), None)
        || !crate::hardening::csrf_request_ok(request.headers(), &state.store, true)
    {
        return refusal();
    }
    let (mut parts, body) = request.into_parts();
    let Ok(bytes) = to_bytes(body, 512).await else {
        return refusal();
    };
    let Ok(delivery) = serde_json::from_slice::<Delivery>(&bytes) else {
        return refusal();
    };
    if delivery.token.len() != contract::get().cookie_handoff.token_characters
        || !delivery
            .token
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return refusal();
    }
    let digest = crate::push_relay::hash_token(&delivery.token);
    let Some((actor, fp, expected_origin, new_hash, grant)) = read(&state, &digest) else {
        return refusal();
    };
    let Some(cookie) =
        crate::hardening::cookie(&parts.headers, &mw_admin::config::session_cookie_name())
    else {
        return refusal();
    };
    let hash = crate::push_relay::hash_token(&cookie);
    if origin(&parts.headers) != Some(expected_origin.as_str())
        || (hash != actor.session_hash && hash != new_hash)
    {
        return refusal();
    }
    let Some(headers) = fresh(&state, &parts.headers, &grant, &actor, fp).await else {
        return refusal();
    };
    // Keep original bytes for the ordinary router's body/framing/limit guards.
    parts.headers = headers;
    parts.extensions.insert(Prepared { digest });
    next.run(Request::from_parts(parts, Body::from(bytes)))
        .await
}
pub(crate) async fn deliver(State(state): State<AppState>, request: Request) -> Response {
    let Some(prepared) = request.extensions().get::<Prepared>().cloned() else {
        return refusal();
    };
    let Some((actor, fp, expected_origin, _, grant)) = read(&state, &prepared.digest) else {
        return refusal();
    };
    if origin(request.headers()) != Some(expected_origin.as_str())
        || !crate::hardening::csrf_request_ok(request.headers(), &state.store, true)
    {
        return refusal();
    }
    if fresh(&state, request.headers(), &grant, &actor, fp)
        .await
        .is_none()
    {
        return refusal();
    }
    if fresh(&state, request.headers(), &grant, &actor, fp)
        .await
        .is_none()
    {
        return refusal();
    }
    // Last synchronous receipt check after all awaits. Re-delivery does not
    // extend either monotonic receipt TTL or the original absolute session TTL.
    let Ok(mut map) = state.websocket_actions.receipts.lock() else {
        return refusal();
    };
    let Some(receipt) = map.get_mut(&prepared.digest) else {
        return refusal();
    };
    let Some(ready) = receipt.ready.as_mut() else {
        return refusal();
    };
    if ready.deadline <= Instant::now() || grant.absolute <= chrono::Utc::now().timestamp() {
        return refusal();
    }
    ready.delivered = true;
    let mut response = Json(serde_json::json!({"delivered":true})).into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    for cookie in crate::admin_rotation::cookies(&state, &grant.token, grant.absolute) {
        response.headers_mut().append(header::SET_COOKIE, cookie);
    }
    response
}

/// Only the canonical rotation producer can attach this marker. Never infer it
/// by parsing arbitrary Set-Cookie values from a handler response.
#[derive(Clone)]
pub(crate) struct IssuedCookies(Vec<[u8; 32]>);
fn cookie_hash(value: &axum::http::HeaderValue) -> Option<[u8; 32]> {
    // Outer security headers may append Secure; no other attribute/value is
    // accepted as an issuer match. Delivery recreates canonical cookies itself.
    let value = value.to_str().ok()?;
    let normalized = value
        .split(';')
        .map(str::trim)
        .filter(|s| !s.eq_ignore_ascii_case("secure"))
        .collect::<Vec<_>>()
        .join(";");
    Some(Sha256::digest(normalized.as_bytes()).into())
}
impl IssuedCookies {
    fn new(values: &[axum::http::HeaderValue]) -> Option<Self> {
        Some(Self(
            values.iter().map(cookie_hash).collect::<Option<Vec<_>>>()?,
        ))
    }
}
pub(super) fn issued_cookies_match(response: &Response) -> bool {
    let Some(expected) = response.extensions().get::<IssuedCookies>() else {
        return false;
    };
    let actual = response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .map(cookie_hash)
        .collect::<Option<Vec<_>>>();
    actual.as_ref() == Some(&expected.0)
}

fn registry() -> &'static Mutex<Vec<Weak<Service>>> {
    static REGISTRY: OnceLock<Mutex<Vec<Weak<Service>>>> = OnceLock::new();
    REGISTRY.get_or_init(Default::default)
}
pub(crate) async fn shutdown_all() -> bool {
    let services = match registry().lock() {
        Ok(mut list) => {
            list.retain(|s| s.strong_count() > 0);
            list.iter().filter_map(Weak::upgrade).collect::<Vec<_>>()
        }
        Err(_) => return false,
    };
    for s in &services {
        s.closing.store(true, Ordering::Release);
    }
    tokio::time::timeout(
        Duration::from_secs(2),
        futures_util::future::join_all(services.iter().map(|s| s.shutdown())),
    )
    .await
    .map(|results| results.into_iter().all(|ok| ok))
    .unwrap_or(false)
}

#[cfg(test)]
impl Service {
    pub(super) fn expire_for_test(&self, token: &str) {
        let mut map = self.receipts.lock().unwrap();
        let ready = map
            .get_mut(&crate::push_relay::hash_token(token))
            .unwrap()
            .ready
            .as_mut()
            .unwrap();
        ready.deadline = Instant::now() - Duration::from_millis(1);
    }
}
