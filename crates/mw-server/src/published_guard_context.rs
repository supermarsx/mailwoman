//! Request-only reuse of versioned INPUTS for a declared metadata read.
//! The declaration is not authority. A consumed context cannot fall through to
//! an ordinary application dispatch or an inner-only authentication restart.
use crate::{AppState, feature_gate, published_guard_inputs::PreparedGuardInputs};
use axum::body::HttpBody;
use axum::{
    Json,
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use mw_store::authority_publication::NativeAuthorityFence;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicU32, Ordering},
};
use tokio::sync::OnceCell;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Contract {
    schema: u8,
    header: String,
    value: String,
    method: String,
    path: String,
    max_body_bytes: usize,
    max_ids: usize,
    max_id_bytes: usize,
    max_call_id_bytes: usize,
    properties: Vec<String>,
    arguments: Vec<String>,
}
fn contract() -> &'static Contract {
    static VALUE: OnceLock<Contract> = OnceLock::new();
    VALUE.get_or_init(|| {
        let value: Contract = serde_json::from_str(include_str!(
            "../../../apps/web/src/api/native-read-intent-contract.json"
        ))
        .expect("static native read contract");
        assert!(
            value.schema == 1
                && value.header == "x-mailwoman-read-shape"
                && value.value == "native-metadata-v1"
                && value.method == "POST"
                && value.path == "/jmap/api"
                && value.max_body_bytes <= 2 * 1024 * 1024
                && value.max_ids == 50
                && value.max_id_bytes <= 1024
                && value.max_call_id_bytes <= 1024
                && value.properties.len() == 36
        );
        let unique = value
            .properties
            .iter()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique.len(), 36);
        value
    })
}
#[derive(Clone, Copy)]
#[repr(u32)]
pub(crate) enum Guard {
    Protocol = 1,
    Probe = 2,
    Protection = 4,
    Inbound = 8,
    Client = 16,
    Maintenance = 32,
    Watch = 64,
    Geography = 128,
    Trust = 256,
    GlobalKnobs = 512,
}
pub(crate) struct Current {
    pub inputs: Arc<PreparedGuardInputs>,
    source: Arc<NativeAuthorityFence>,
    used: Arc<AtomicU32>,
}
impl Current {
    pub(crate) fn used(&self, guard: Guard) {
        self.used.fetch_or(guard as u32, Ordering::Relaxed);
        #[cfg(test)]
        test_control::record(guard);
    }
}
struct Scope {
    state: AppState,
    headers: HeaderMap,
    eligible: bool,
    first: OnceCell<Option<Arc<Current>>>,
    used: Arc<AtomicU32>,
    body: Mutex<Option<[u8; 32]>>,
}
tokio::task_local! {static SCOPE:Arc<Scope>;}
pub(crate) async fn layer(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let c = contract();
    let declared = request.method().as_str() == c.method
        && request.uri().path() == c.path
        && request
            .headers()
            .get(c.header.as_str())
            .and_then(|v| v.to_str().ok())
            == Some(c.value.as_str());
    if !declared {
        return next.run(request).await;
    }
    let h = request.headers();
    // A WS original-router request has an exact in-memory Body size without a
    // wire Content-Length. This reads metadata only, never polls the body.
    let known_size = h
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .or_else(|| request.body().size_hint().exact());
    let mut eligible = request.uri().query().is_none()
        && crate::session_persistence::flush_seconds() > 0
        && state.store.uses_sqlite()
        && !h.contains_key(header::AUTHORIZATION)
        && !h.contains_key("x-api-key")
        && !h.contains_key("x-mailwoman-link")
        && h.get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|s| s.split(';').next().map(str::trim) == Some("application/json"))
        && known_size.is_some_and(|n| n > 0 && n <= c.max_body_bytes as u64);
    let mut hints = HeaderMap::new();
    let cookie = h
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .into_iter()
        .flat_map(|v| v.split(';'))
        .find_map(|v| v.trim().strip_prefix("mw_session="));
    match cookie.filter(|s| !s.is_empty() && s.len() <= 1024) {
        Some(cookie) => {
            if let Ok(v) = format!("mw_session={cookie}").parse() {
                hints.insert(header::COOKIE, v);
            } else {
                eligible = false;
            }
        }
        None => eligible = false,
    }
    for name in ["x-mailwoman-account", "x-mailwoman-storage-authority"] {
        if let Some(value) = h.get(name) {
            if value.as_bytes().len() <= 1024 {
                hints.insert(name, value.clone());
            } else {
                eligible = false;
            }
        }
    }
    let scope = Arc::new(Scope {
        state,
        headers: hints,
        eligible,
        first: OnceCell::new(),
        used: Arc::new(AtomicU32::new(0)),
        body: Mutex::new(None),
    });
    SCOPE.scope(scope, next.run(request)).await
}
pub(crate) async fn current() -> Option<Arc<Current>> {
    let scope = SCOPE.try_with(Clone::clone).ok()?;
    scope
        .first
        .get_or_init(|| async {
            let h = &scope.headers;
            // Header-only eligibility never reads the body or queues build work.
            if !scope.eligible {
                return None;
            }
            let (entry, source) =
                feature_gate::current_declared_guard_source(&scope.state, h).await?;
            let inputs = entry.guard.as_ref()?.clone();
            Some(Arc::new(Current {
                inputs,
                source,
                used: scope.used.clone(),
            }))
        })
        .await
        .clone()
}
pub(crate) fn consumed() -> bool {
    SCOPE
        .try_with(|s| s.used.load(Ordering::Relaxed) != 0)
        .unwrap_or(false)
}
fn refused() -> Response {
    let mut r=(StatusCode::SERVICE_UNAVAILABLE,Json(serde_json::json!({"code":"publishedGuardFenceRefusal","notDispatched":true,"detail":"Read source changed before dispatch"}))).into_response();
    r.headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    r.headers_mut()
        .insert(header::RETRY_AFTER, "1".parse().unwrap());
    r
}
pub(crate) async fn fallback_refusal(state: &AppState, headers: &HeaderMap) -> Option<Response> {
    if !consumed() {
        return None;
    }
    // Preserve terminal expiry/revocation cleanup on an unsupported initial
    // native preparation too. Successful authentication still cannot bypass
    // consumed outer guards or authorize ANY application dispatch.
    Some(match crate::authed(state, headers).await {
        Err(response) => response,
        Ok(_) => refused(),
    })
}
pub(crate) fn validate_body(body: &[u8]) -> Result<(), Response> {
    validate_using(body, || {
        serde_json::from_slice::<Value>(body)
            .ok()
            .is_some_and(|request| exact_shape(&request, body.len()))
    })
}
/// Only the existing feature body reader calls this, immediately after parsing
/// these SAME bytes. The parsed value is never authority or request-extension
/// state; actual bytes still bind the declaration and the handler rechecks it.
pub(crate) fn validate_parsed_body(body: &[u8], parsed: Option<&Value>) -> Result<(), Response> {
    validate_using(body, || {
        parsed.is_some_and(|request| exact_shape(request, body.len()))
    })
}
fn validate_using(body: &[u8], valid_shape: impl FnOnce() -> bool) -> Result<(), Response> {
    let Ok(scope) = SCOPE.try_with(Clone::clone) else {
        return Ok(());
    };
    if body.len() > contract().max_body_bytes {
        return Err(bad_shape());
    }
    let digest: [u8; 32] = Sha256::digest(body).into();
    if scope
        .body
        .lock()
        .ok()
        .is_some_and(|v| v.as_ref() == Some(&digest))
    {
        return Ok(());
    }
    if !valid_shape() {
        return Err(bad_shape());
    }
    *scope.body.lock().map_err(|_| refused())? = Some(digest);
    Ok(())
}
fn bad_shape() -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"code":"invalidDeclaredReadShape","notDispatched":true})),
    )
        .into_response()
}
fn exact_shape(request: &Value, bytes: usize) -> bool {
    let c = contract();
    if mw_engine::jmap::core::validate(request, bytes, true).is_err()
        || request.as_object().is_none_or(|o| {
            o.keys()
                .any(|k| !matches!(k.as_str(), "using" | "methodCalls"))
        })
    {
        return false;
    }
    let Some(calls) = request
        .get("methodCalls")
        .and_then(Value::as_array)
        .filter(|a| a.len() == 1)
    else {
        return false;
    };
    let Some(call) = calls[0].as_array().filter(|a| {
        a.len() == 3
            && a[0].as_str() == Some("Email/get")
            && a[2]
                .as_str()
                .is_some_and(|s| !s.is_empty() && s.len() <= c.max_call_id_bytes)
    }) else {
        return false;
    };
    let args = &call[1];
    if args
        .as_object()
        .is_none_or(|o| o.keys().any(|k| !c.arguments.contains(k)))
        || args
            .get("accountId")
            .and_then(Value::as_str)
            .is_none_or(|s| s.is_empty() || s.len() > c.max_id_bytes)
    {
        return false;
    }
    for key in [
        "mailwomanBodyUse",
        "fetchHTMLBodyValues",
        "fetchTextBodyValues",
        "fetchAllBodyValues",
    ] {
        if args.get(key).is_some_and(|v| v.as_bool() != Some(false)) {
            return false;
        }
    }
    if args.get("ids").and_then(Value::as_array).is_none_or(|ids| {
        ids.is_empty()
            || ids.len() > c.max_ids
            || ids.iter().any(|v| {
                v.as_str()
                    .is_none_or(|s| s.is_empty() || s.len() > c.max_id_bytes)
            })
    }) {
        return false;
    }
    let Some(props) = args
        .get("properties")
        .and_then(Value::as_array)
        .filter(|a| a.len() == 36)
    else {
        return false;
    };
    let mut mask = 0u64;
    for p in props {
        let Some(i) = p
            .as_str()
            .and_then(|p| c.properties.iter().position(|known| known == p))
        else {
            return false;
        };
        let bit = 1u64 << i;
        if mask & bit != 0 {
            return false;
        }
        mask |= bit;
    }
    mask == (1u64 << 36) - 1
}
pub(crate) fn sample_request() -> Value {
    serde_json::json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/get",{"accountId":"shape-only","ids":["shape-only"],"properties":contract().properties},"shape"]]})
}
pub(crate) fn verify_source(source: &NativeAuthorityFence) -> bool {
    let Ok(scope) = SCOPE.try_with(Clone::clone) else {
        return true;
    };
    if scope.used.load(Ordering::Relaxed) == 0 {
        return true;
    }
    let Some(Some(current)) = scope.first.get() else {
        return false;
    };
    current
        .source
        .same_published_source(source)
        .unwrap_or(false)
        && current
            .inputs
            .matches_configuration(&scope.state.store, source)
            .unwrap_or(false)
}
/// The same final observation used by native admission; no new await or query.
pub(crate) fn verify_final(source: &NativeAuthorityFence) -> bool {
    let Ok(scope) = SCOPE.try_with(Clone::clone) else {
        return true;
    };
    if scope.used.load(Ordering::Relaxed) == 0 {
        return true;
    }
    let Some(Some(current)) = scope.first.get() else {
        return false;
    };
    scope.body.lock().ok().is_some_and(|b| b.is_some())
        && current
            .source
            .same_published_source(source)
            .unwrap_or(false)
        && current
            .inputs
            .matches_configuration(&scope.state.store, source)
            .unwrap_or(false)
}
/// Generic config helpers may be called with another Store; never borrow this
/// request's inputs across installations, even when the keys are identical.
pub(crate) async fn current_for(store: &mw_store::Store) -> Option<Arc<Current>> {
    if !SCOPE
        .try_with(|s| s.state.store.cache_scope() == store.cache_scope())
        .ok()?
    {
        return None;
    }
    current().await
}

#[cfg(test)]
pub(crate) mod test_control {
    use super::*;
    tokio::task_local! {static USED:Arc<AtomicU32>;}
    pub(super) fn record(guard: Guard) {
        let _ = USED.try_with(|used| used.fetch_or(guard as u32, Ordering::Relaxed));
    }
    pub(crate) async fn count<F: std::future::Future>(future: F) -> (F::Output, u32) {
        let used = Arc::new(AtomicU32::new(0));
        let result = USED.scope(used.clone(), future).await;
        (result, used.load(Ordering::Relaxed))
    }
}
