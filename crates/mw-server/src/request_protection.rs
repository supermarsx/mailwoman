//! Explainable aggregate admission and typed adaptive observations.
//! Never infer abuse from a generic HTTP status code.
mod api;
mod config;
/// The Adaptive protection page's policy editor and simulator (t97).
mod editor;
use crate::{AppState, admin_ctx::AdminApiError};
pub(crate) fn register(r: &mut impl crate::admin_router::AdminRegistrar) {
    api::register(r);
    editor::register(r);
}
use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use config::{Cache, Config};
use mw_store::request_protection_model::{Decision, Policy, Signal};
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

#[derive(Default)]
pub(crate) struct Runtime {
    cache: Cache,
}
impl Runtime {
    pub fn start(store: mw_store::Store) -> Arc<Self> {
        let runtime = Arc::new(Self::default());
        let weak = Arc::downgrade(&runtime);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                let Some(alive) = weak.upgrade() else {
                    break;
                };
                if store
                    .prune_request_protection(chrono::Utc::now().timestamp())
                    .await
                    .is_err()
                {
                    warning();
                }
                drop(alive);
            }
        });
        runtime
    }
    pub fn invalidate(&self) {
        self.cache.invalidate();
    }
}
struct Charged {
    kind: &'static str,
    value: String,
    policy: Policy,
}
struct Context {
    state: AppState,
    config: Arc<Config>,
    charged: tokio::sync::Mutex<Vec<Charged>>,
    signal: AtomicU8,
}
tokio::task_local! { static REQUEST: Arc<Context>; }

fn unavailable() -> AdminApiError {
    AdminApiError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "requestProtectionUnavailable",
        "Request protection is temporarily unavailable",
    )
}
fn limited(decision: Decision) -> AdminApiError {
    AdminApiError::new(
        StatusCode::TOO_MANY_REQUESTS,
        "requestProtectionLimited",
        "Wait before trying this request again",
    )
    .with("retryAfterSecs", decision.retry_after_secs)
}
fn response(error: AdminApiError) -> Response {
    let mut response = error.into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "private, no-store".parse().unwrap());
    response
}
fn warning() {
    use std::sync::atomic::AtomicI64;
    static LAST: AtomicI64 = AtomicI64::new(0);
    let now = chrono::Utc::now().timestamp();
    let previous = LAST.load(Ordering::Relaxed);
    if now.saturating_sub(previous) >= 60
        && LAST
            .compare_exchange(previous, now, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
    {
        tracing::warn!(target:"request_protection", "Request protection storage is unavailable; configured failure policy applied");
    }
}
impl Context {
    async fn admit(&self, kind: &'static str, value: &str) -> Result<(), AdminApiError> {
        let Some(policy) = self.config.subject(kind, value) else {
            return Ok(());
        };
        let mut charged = self.charged.lock().await;
        if charged
            .iter()
            .any(|subject| subject.kind == kind && subject.value == value)
        {
            return Ok(());
        }
        // Auth helpers may resolve the same identity more than once. The request
        // owns one charge, and never accepts caller-selected subject identifiers.
        if charged.len() >= 4 {
            return Err(unavailable());
        }
        let answer = self
            .state
            .store
            .request_protection(kind, value, &policy, chrono::Utc::now().timestamp(), None)
            .await;
        match answer {
            Ok(outcome) if outcome.decision.enforced => Err(limited(outcome.decision)),
            Ok(_) => {
                charged.push(Charged {
                    kind,
                    value: value.to_owned(),
                    policy,
                });
                Ok(())
            }
            Err(_) => {
                warning();
                if self.config.fail_closed
                    && (policy.rate_mode == mw_store::request_protection_model::Mode::Enforce
                        || policy.adaptive_mode
                            == mw_store::request_protection_model::Mode::Enforce)
                {
                    Err(unavailable())
                } else {
                    Ok(())
                }
            }
        }
    }
    async fn observe(&self, status: StatusCode) {
        let signal = match self.signal.load(Ordering::Relaxed) {
            1 => Signal::Malformed,
            2 => Signal::Oversized,
            3 => Signal::CredentialRejected,
            4 => Signal::PermissionDenied,
            5 => Signal::BodyDeadline,
            6 => Signal::AdminRouteMiss,
            _ if status.is_success() => Signal::Normal,
            _ => Signal::Ignored,
        };
        if signal == Signal::Ignored {
            return;
        }
        for subject in self.charged.lock().await.iter() {
            if self
                .state
                .store
                .request_protection(
                    subject.kind,
                    &subject.value,
                    &subject.policy,
                    chrono::Utc::now().timestamp(),
                    Some(signal),
                )
                .await
                .is_err()
            {
                warning()
            }
        }
    }
}
/// Proxy account IDs are local to the authenticated service, not deployment-wide.
fn proxy_account_key(endpoint: &str, account: &str) -> Result<String, mw_store::StoreError> {
    use sha2::{Digest, Sha256};
    let identity = crate::primary_identity::VerifiedLoginIdentity::proxy(endpoint, account)?;
    let input = serde_json::to_vec(&(identity.authority, identity.subject))
        .expect("identity tuple serializes");
    let digest: String = Sha256::digest(input)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Ok(format!("proxy:{digest}"))
}
/// Called only with a session that has passed authentication and account checks.
pub(crate) async fn session_account(session: &mw_store::Session) -> Result<(), Response> {
    let Some(context) = REQUEST.try_with(Arc::clone).ok() else {
        return Ok(());
    };
    let key = if context.state.engine.is_some() {
        session.account_id.clone()
    } else {
        proxy_account_key(&session.jmap_url, &session.account_id)
            .map_err(|_| response(unavailable()))?
    };
    context.admit("account", &key).await.map_err(response)
}
/// A legacy key carries no verified proxy service realm. Never choose an arbitrary
/// session sharing its upstream-local ID to manufacture that missing authority.
pub(crate) async fn account(value: &str) -> Result<(), Response> {
    let Some(context) = REQUEST.try_with(Arc::clone).ok() else {
        return Ok(());
    };
    if context.state.engine.is_none() {
        if context.config.account_dimension_enforced() {
            return Err(response(unavailable()));
        }
        return Ok(());
    }
    context.admit("account", value).await.map_err(response)
}
pub(crate) async fn administrator(value: &str) -> Result<(), AdminApiError> {
    let context = REQUEST.try_with(Arc::clone).ok();
    match context {
        Some(context) => context.admit("admin", value).await,
        None => Ok(()),
    }
}
/// One authoritative anomaly per request; no status-code or user-agent guessing.
pub(crate) fn observe(signal: Signal) {
    let code = match signal {
        Signal::Malformed => 1,
        Signal::Oversized => 2,
        Signal::CredentialRejected => 3,
        Signal::PermissionDenied => 4,
        Signal::BodyDeadline => 5,
        Signal::AdminRouteMiss => 6,
        _ => return,
    };
    let _ = REQUEST.try_with(|context| {
        let _ = context
            .signal
            .compare_exchange(0, code, Ordering::Relaxed, Ordering::Relaxed);
    });
}
/// Only called from the router's unmatched-path fallback, never from an
/// application's entity-not-found response. Do not retain the requested path.
pub(crate) fn admin_route_miss(path: &str) {
    let admin = mw_admin::config::admin_path();
    let Some(suffix) = path.strip_prefix(&format!("{admin}/")) else {
        return;
    };
    if suffix.is_empty()
        || suffix == "login"
        || suffix.starts_with("login/")
        || suffix.starts_with("assets/")
        || suffix.starts_with("static/")
        || suffix.rsplit('.').next().is_some_and(|extension| {
            matches!(
                extension,
                "js" | "css"
                    | "map"
                    | "woff"
                    | "woff2"
                    | "ttf"
                    | "ico"
                    | "png"
                    | "jpg"
                    | "jpeg"
                    | "svg"
                    | "webp"
            )
        })
    {
        return;
    }
    observe(Signal::AdminRouteMiss);
}
pub(crate) fn body_error(error: &axum::Error) {
    let mut cause: Option<&(dyn std::error::Error + 'static)> = Some(error);
    while let Some(error) = cause {
        if error.is::<http_body_util::LengthLimitError>() {
            observe(Signal::Oversized);
            return;
        }
        cause = error.source();
    }
}
async fn operator_control(
    state: &AppState,
    method: &axum::http::Method,
    request_path: &str,
    headers: &axum::http::HeaderMap,
) -> bool {
    use crate::capabilities::Capability;
    let admin = mw_admin::config::admin_path();
    let Some(path) = request_path.strip_prefix(&admin) else {
        return false;
    };
    let capability = if method == axum::http::Method::GET
        && matches!(
            path,
            "/bans/protection" | "/bans/protection/subjects" | "/bans/protection/events"
        ) {
        Capability::BansRead
    } else if (method == axum::http::Method::DELETE
        && path
            .strip_prefix("/bans/protection/subjects/")
            .is_some_and(|key| key.len() == 64 && key.bytes().all(|b| b.is_ascii_hexdigit())))
        || (method == axum::http::Method::POST && matches!(path, "/step-up" | "/step-up/passkey"))
        || (method == axum::http::Method::GET && path == "/session")
    {
        Capability::BansWrite
    } else {
        return false;
    };
    // Reading the current client's own row must not change its revision after
    // rendering. Only a verified, unscoped operator gets this narrow recovery
    // path; the clear handler still demands fresh proof and the exact revision.
    crate::admin_ctx::AdminCtx::resolve(state, headers)
        .await
        .is_ok_and(|ctx| ctx.require_unscoped(capability).is_ok())
}
fn protected(path: &str) -> bool {
    let admin = mw_admin::config::admin_path();
    let suffix = path
        .strip_prefix(&admin)
        .filter(|suffix| suffix.starts_with('/'));
    if matches!(
        path,
        "/api/logout"
            | "/api/sso/logout"
            | "/api/public-config"
            | "/api/rc/frame"
            | "/api/login/2fa"
            | "/oidc/logout"
            | "/oidc/logout/context"
    ) || suffix
        .is_some_and(|suffix| matches!(suffix, "/logout" | "/login/2fa" | "/login/recovery"))
    {
        return false;
    }
    suffix.is_some()
        || path == "/mcp"
        || ["/api/", "/jmap/", "/dav/", "/mcp/", "/oauth/", "/oidc/"]
            .iter()
            .any(|prefix| path.starts_with(prefix))
}
pub(crate) async fn guard(State(state): State<AppState>, request: Request, next: Next) -> Response {
    if request.method() == axum::http::Method::OPTIONS || !protected(request.uri().path()) {
        return next.run(request).await;
    }
    if operator_control(
        &state,
        request.method(),
        request.uri().path(),
        request.headers(),
    )
    .await
    {
        return next.run(request).await;
    }
    let config = match state.request_protection.cache.get(&state.store).await {
        Ok(config) => config,
        Err(()) => return response(unavailable()),
    };
    if !config.enabled() {
        return next.run(request).await;
    }
    let source = crate::scope_mw::proxy::client_ip(request.headers(), request.extensions());
    let context = Arc::new(Context {
        state,
        config,
        charged: Default::default(),
        signal: AtomicU8::new(0),
    });
    if let Some(source) = source
        && let Err(error) = context
            .admit("client", &source.to_canonical().to_string())
            .await
    {
        return response(error);
    }
    REQUEST
        .scope(context.clone(), async move {
            let result = next.run(request).await;
            context.observe(result.status()).await;
            result
        })
        .await
}

#[cfg(test)]
mod identity_tests {
    use super::*;
    #[test]
    fn upstream_local_accounts_require_the_verified_service_realm() {
        let first = proxy_account_key("https://mail.example/a/session", "1").unwrap();
        assert_eq!(
            first,
            proxy_account_key("https://MAIL.example:443/a/session", "1").unwrap()
        );
        assert_ne!(
            first,
            proxy_account_key("https://mail.example/b/session", "1").unwrap()
        );
        assert_ne!(
            first,
            proxy_account_key("https://mail.example/a/session?tenant=other", "1").unwrap()
        );
        assert_ne!(
            first,
            proxy_account_key("https://mail.example/a/session", "2").unwrap()
        );
        assert!(proxy_account_key("file:///tmp/session", "1").is_err());
    }
    #[tokio::test]
    async fn independent_services_do_not_consume_each_others_account_budget() {
        use mw_store::{ServerKey, Store, request_protection_model::Mode};
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let policy = Policy {
            rate_mode: Mode::Enforce,
            rate_limit: 1,
            adaptive_mode: Mode::Off,
            rate_window_secs: 60,
            short: mw_store::request_protection_model::WindowPolicy {
                seconds: 60,
                minimum_samples: 1,
                absolute_anomalies: 1,
                ratio_percent: 0,
            },
            long: mw_store::request_protection_model::WindowPolicy {
                seconds: 3600,
                minimum_samples: 1,
                absolute_anomalies: 1,
                ratio_percent: 0,
            },
            normal_cap_per_bucket: 100,
            signals: 1,
            throttle_secs: 3,
            block_secs: 60,
            decay_secs: 3600,
        };
        let first = proxy_account_key("https://mail.example/a/session", "1").unwrap();
        let second = proxy_account_key("https://mail.example/b/session", "1").unwrap();
        assert!(
            !store
                .request_protection("account", &first, &policy, 100, None)
                .await
                .unwrap()
                .decision
                .enforced
        );
        assert!(
            !store
                .request_protection("account", &second, &policy, 100, None)
                .await
                .unwrap()
                .decision
                .enforced
        );
        assert!(
            store
                .request_protection("account", &first, &policy, 100, None)
                .await
                .unwrap()
                .decision
                .enforced
        );
    }
}
