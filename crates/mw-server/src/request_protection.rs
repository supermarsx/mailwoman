//! Explainable aggregate admission and typed adaptive observations.
//! Never infer abuse from a generic HTTP status code.
mod api;
pub(crate) mod config;
/// The Adaptive protection page's policy editor and simulator (t97).
mod editor;
/// Per-minute counts behind the Aggregate and Adaptive charts.
mod history;
mod probe_path;
mod tier_sim;
pub(crate) mod user_agent;
/// Optional in-memory accounting with background durable flush (t34).
pub(crate) mod write_behind;
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

/// What a claimed user-agent class action comes to for a request: a signed-in
/// session is never blocked by class (it is limited like any other request).
/// Shared with the Adaptive protection simulator.
pub(crate) fn class_outcome(
    action: crate::trust_tier::UaAction,
    signed_in: bool,
) -> crate::trust_tier::UaAction {
    match action {
        crate::trust_tier::UaAction::Block if signed_in => crate::trust_tier::UaAction::Limit,
        other => other,
    }
}

#[derive(Default)]
pub(crate) struct Runtime {
    cache: Cache,
    /// Keyed by the live configured path; a changed console prefix never uses
    /// stale normalization, and ordinary requests do not decode it repeatedly.
    normalized_admin: std::sync::RwLock<Option<(String, Arc<str>)>>,
    /// The protocol guard's policy cache (`security.protocol.*`), invalidated
    /// together with the request-protection policy.
    pub(crate) protocol: crate::protocol_guard::Cache,
    /// Who may reach `/healthz` (`security.health.*`).
    pub(crate) health: crate::health_access::Cache,
    /// `Some` only with `MW_REQUEST_PROTECTION_ACCOUNTING=write-behind` on SQLite.
    accounting: Option<Arc<write_behind::WriteBehind>>,
}
impl Runtime {
    /// Reads the accounting switches; an invalid value stops startup.
    pub fn start(store: mw_store::Store) -> Arc<Self> {
        Self::try_start(store).unwrap_or_else(|error| panic!("{error}"))
    }
    /// `start`, returning an invalid accounting switch as an error.
    pub fn try_start(store: mw_store::Store) -> Result<Arc<Self>, String> {
        Ok(Self::start_with(store, write_behind::AccountingConfig::from_env()?))
    }
    pub(crate) fn start_with(
        store: mw_store::Store,
        config: write_behind::AccountingConfig,
    ) -> Arc<Self> {
        let mode = config.mode.effective(store.uses_sqlite());
        if mode != config.mode {
            tracing::warn!(target: "request_protection", "{}=write-behind is SQLite-only; using synchronous accounting", write_behind::MODE_VAR);
        }
        let accounting = (mode == write_behind::AccountingMode::WriteBehind)
            .then(|| write_behind::WriteBehind::start(store.clone(), config));
        let runtime = Arc::new(Self { accounting, ..Self::default() });
        let weak = Arc::downgrade(&runtime);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                let Some(alive) = weak.upgrade() else {
                    break;
                };
                crate::trust_tier::prune();
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
    pub(crate) fn write_behind(&self) -> Option<&write_behind::WriteBehind> {
        self.accounting.as_deref()
    }
    /// Graceful shutdown hook: commit queued protection observations before
    /// `deadline`. Always `Ok` in synchronous mode.
    pub async fn flush_all(
        &self,
        deadline: tokio::time::Instant,
    ) -> Result<(), write_behind::FlushIncomplete> {
        match &self.accounting {
            Some(accounting) => accounting.flush_all(deadline).await,
            None => Ok(()),
        }
    }
    /// Commit queued observations before an administrative read; errors are left
    /// to the read itself.
    pub(crate) async fn flush_for_read(&self) {
        if let Some(accounting) = &self.accounting
            && accounting.flush_pending().await.is_err()
        {
            warning();
        }
    }
    /// Write-behind counters; `None` in synchronous mode.
    pub(crate) fn accounting_status(&self) -> Option<write_behind::WriteBehindStatus> {
        self.accounting.as_ref().map(|accounting| accounting.status())
    }
    /// Administrative clear by expected revision. Write-behind commits pending
    /// observations first and resets the cached subject.
    pub(crate) async fn clear_subject(
        &self,
        store: &mw_store::Store,
        key: &str,
        expected: &str,
        now: i64,
    ) -> Result<bool, mw_store::StoreError> {
        match &self.accounting {
            Some(accounting) => accounting.clear(key, expected, now).await,
            None => store.clear_request_protection(key, expected, now).await,
        }
    }
    pub fn invalidate(&self) {
        if let Some(accounting) = &self.accounting {
            accounting.invalidate();
        }
        crate::trust_tier::invalidate();
        crate::rate_leeway::forget_knobs();
        self.cache.invalidate();
        self.protocol.invalidate();
        self.health.invalidate();
    }
}
/// Charge one typed observation to the transport source's client subject, the
/// way a probe-path hit is charged. Returns the refusal when the source is now
/// under an enforced adaptive action. No-op while adaptive protection is off.
pub(crate) async fn observe_source(
    state: &AppState,
    source: std::net::IpAddr,
    signal: Signal,
) -> Option<Response> {
    let config = state
        .request_protection
        .cache
        .get(&state.store)
        .await
        .ok()?;
    let source = source.to_canonical();
    let source_ip = source.to_string();
    let policy = config.subject("client", &source_ip)?;
    if policy.adaptive_mode == mw_store::request_protection_model::Mode::Off {
        return None;
    }
    let country = match mw_engine::security::country_lookup(Some(source)) {
        mw_engine::security::CountryLookup::Known(country) => Some(country),
        _ => None,
    };
    if let Some(accounting) = state.request_protection.write_behind() {
        let answer = accounting
            .record(
                "client",
                &source_ip,
                &policy,
                chrono::Utc::now().timestamp(),
                Some(signal),
                mw_store::request_protection::ProtectionSource {
                    country: country.as_deref(),
                    ip: Some(&source_ip),
                },
            )
            .await;
        if answer.storage_failed {
            warning();
        }
        return answer
            .decision
            .enforced
            .then(|| response(limited(answer.decision)));
    }
    match state
        .store
        .request_protection_with_source(
            "client",
            &source_ip,
            &policy,
            chrono::Utc::now().timestamp(),
            Some(signal),
            mw_store::request_protection::ProtectionSource {
                country: country.as_deref(),
                ip: Some(&source_ip),
            },
        )
        .await
    {
        Ok(outcome) if outcome.decision.enforced => Some(response(limited(outcome.decision))),
        Ok(_) => None,
        Err(_) => {
            warning();
            None
        }
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
    legacy_route: bool,
    source_country: Option<String>,
    source_ip: Option<String>,
    display_subresource: bool,
    /// Trust tier of this request (0 unknown, 1 known, 2 authenticated).
    tier: AtomicU8,
    trust: Arc<crate::trust_tier::Policy>,
    /// `ratelimit.trustedMultiplier` when the request comes from a source with a
    /// successful sign-in on record (else 100): scales a signed-in account's allowance.
    trusted_percent: u32,
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
/// Write-behind storage failure at admission: `storageFailure: closed` with an
/// enforcing mode answers 503, as the synchronous path does. Otherwise the local
/// decision stands, so a subject already over its limit in this process is refused.
fn storage_failure_answer(
    fail_closed: bool,
    policy: &Policy,
    local: Decision,
) -> Result<(), AdminApiError> {
    use mw_store::request_protection_model::Mode;
    if fail_closed && (policy.rate_mode == Mode::Enforce || policy.adaptive_mode == Mode::Enforce) {
        Err(unavailable())
    } else if local.enforced {
        Err(limited(local))
    } else {
        Ok(())
    }
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
    fn tier(&self) -> crate::trust_tier::Tier {
        match self.tier.load(Ordering::Relaxed) {
            2 => crate::trust_tier::Tier::Authenticated,
            1 => crate::trust_tier::Tier::Known,
            _ => crate::trust_tier::Tier::Unknown,
        }
    }
    fn admission_policy(&self, kind: &'static str, value: &str) -> Option<Policy> {
        if !self.legacy_route && !kind.starts_with("ua-") {
            return None;
        }
        self.config.subject(kind, value)
    }
    async fn admit(&self, kind: &'static str, value: &str) -> Result<(), AdminApiError> {
        let Some(mut policy) = self.admission_policy(kind, value) else {
            return Ok(());
        };
        // A verified image/inline-part capability belongs to the already
        // admitted signing batch. Active adaptive actions and independent UA
        // limits still apply; its individual download spends no second budget.
        if self.display_subresource && !kind.starts_with("ua-") {
            policy.rate_mode = mw_store::request_protection_model::Mode::Off;
        }
        // Trust tier: only the request allowance of an anonymous source scales.
        // Adaptive signal thresholds and active blocks are never relaxed.
        if kind == "client" || kind.starts_with("ua-") {
            policy.rate_limit = self.trust.scale_limit(policy.rate_limit, self.tier());
        }
        // A signed-in mailbox used from a source that has signed in before gets
        // more room (never less, and never for the anonymous or admin budgets).
        if kind == "account" && self.trusted_percent > 100 && policy.rate_limit > 0 {
            policy.rate_limit = u32::try_from(
                (u64::from(policy.rate_limit) * u64::from(self.trusted_percent)).div_ceil(100),
            )
            .unwrap_or(u32::MAX);
        }
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
        if let Some(accounting) = self.state.request_protection.write_behind() {
            let answer = accounting
                .record(
                    kind,
                    value,
                    &policy,
                    chrono::Utc::now().timestamp(),
                    kind.strip_prefix("ua-")
                        .filter(|category| self.config.user_agent.blocks_bot(category))
                        .map(|_| Signal::BotUserAgent),
                    mw_store::request_protection::ProtectionSource {
                        country: self.source_country.as_deref(),
                        ip: self.source_ip.as_deref(),
                    },
                )
                .await;
            if !answer.storage_failed {
                history::note_decision(kind, &answer.decision);
                if answer.decision.enforced {
                    return Err(limited(answer.decision));
                }
                charged.push(Charged {
                    kind,
                    value: value.to_owned(),
                    policy,
                });
                return Ok(());
            }
            warning();
            return storage_failure_answer(self.config.fail_closed, &policy, answer.decision);
        }
        let answer = self
            .state
            .store
            .request_protection_with_source(
                kind,
                value,
                &policy,
                chrono::Utc::now().timestamp(),
                kind.strip_prefix("ua-")
                    .filter(|category| self.config.user_agent.blocks_bot(category))
                    .map(|_| Signal::BotUserAgent),
                mw_store::request_protection::ProtectionSource {
                    country: self.source_country.as_deref(),
                    ip: self.source_ip.as_deref(),
                },
            )
            .await;
        if let Ok(outcome) = &answer {
            history::note_decision(kind, &outcome.decision);
        }
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
        if self.display_subresource && signal == Signal::Normal {
            return;
        }
        for subject in self.charged.lock().await.iter() {
            if let Some(accounting) = self.state.request_protection.write_behind() {
                // Ordinary completions are queued; only a transition is committed
                // before the response returns.
                if accounting
                    .record(
                        subject.kind,
                        &subject.value,
                        &subject.policy,
                        chrono::Utc::now().timestamp(),
                        Some(signal),
                        mw_store::request_protection::ProtectionSource {
                            country: self.source_country.as_deref(),
                            ip: self.source_ip.as_deref(),
                        },
                    )
                    .await
                    .storage_failed
                {
                    warning()
                }
                continue;
            }
            if self
                .state
                .store
                .request_protection_with_source(
                    subject.kind,
                    &subject.value,
                    &subject.policy,
                    chrono::Utc::now().timestamp(),
                    Some(signal),
                    mw_store::request_protection::ProtectionSource {
                        country: self.source_country.as_deref(),
                        ip: self.source_ip.as_deref(),
                    },
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
/// Request-owned budget evidence only, never an authentication decision. A
/// contended/uncharged context uses the existing complete admission path.
pub(crate) fn native_account_already_admitted(account: &str) -> bool {
    let Some(context) = REQUEST.try_with(Arc::clone).ok() else { return true; };
    if context.state.engine.is_none() { return false; }
    if context.admission_policy("account", account).is_none() { return true; }
    context.charged.try_lock().is_ok_and(|charged| {
        charged.iter().any(|subject| subject.kind == "account" && subject.value == account)
    })
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
            "/bans/protection"
                | "/bans/protection/subjects"
                | "/bans/protection/events"
                | "/bans/protection/policy"
        ) {
        Capability::BansRead
    } else if method == axum::http::Method::POST && path == "/bans/protection/simulate" {
        Capability::BansRead
    } else if method == axum::http::Method::PUT && path == "/bans/protection/policy" {
        Capability::SettingsWrite
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
fn ua_protected(path: &str) -> bool {
    let admin = mw_admin::config::admin_path();
    if path.starts_with("/assets/")
        || path.starts_with("/static/")
        || path
            .strip_prefix(&admin)
            .is_some_and(|p| p.starts_with("/assets/") || p.starts_with("/static/"))
    {
        return false;
    }
    if !protected(path)
        && path.rsplit('.').next().is_some_and(|ext| {
            matches!(
                ext,
                "js" | "css"
                    | "map"
                    | "wasm"
                    | "ico"
                    | "png"
                    | "jpg"
                    | "jpeg"
                    | "svg"
                    | "webp"
                    | "woff"
                    | "woff2"
                    | "ttf"
            )
        })
    {
        return false;
    }
    if protected(path) {
        return true;
    }
    // Public documents are crawler targets too. Protocol recovery, static assets,
    // and liveness checks must remain available without this extra dimension.
    if path.starts_with("/api/")
        || path.starts_with("/oidc/")
        || matches!(path, "/health" | "/healthz" | "/ready" | "/readyz" | "/metrics")
        || path.starts_with("/assets/")
        || path.starts_with("/static/")
    {
        return false;
    }
    if path
        .strip_prefix(&admin)
        .is_some_and(|p| matches!(p, "/logout" | "/login/2fa" | "/login/recovery"))
    {
        return false;
    }
    !path.rsplit('/').next().unwrap_or("").contains('.')
}
/// Keep operational recovery and ordinary assets available. Explicit hostile
/// paths are checked independently, including probes hidden below asset roots.
fn probe_block_scope(path: &str) -> bool {
    let admin = mw_admin::config::admin_path();
    if matches!(
        path,
        "/health"
            | "/healthz"
            | "/ready"
            | "/readyz"
            | "/api/logout"
            | "/api/sso/logout"
            | "/api/public-config"
            | "/api/rc/frame"
            | "/api/login/2fa"
            | "/oidc/logout"
            | "/oidc/logout/context"
    ) || path.strip_prefix(&admin).is_some_and(|suffix| {
        matches!(suffix, "/logout" | "/login/2fa" | "/login/recovery")
            || suffix.starts_with("/assets/")
            || suffix.starts_with("/static/")
    }) || path.starts_with("/assets/")
        || path.starts_with("/static/")
    {
        return false;
    }
    if protected(path) {
        return true;
    }
    !path.rsplit('.').next().is_some_and(|extension| {
        matches!(
            extension,
            "js" | "css"
                | "map"
                | "wasm"
                | "ico"
                | "png"
                | "jpg"
                | "jpeg"
                | "svg"
                | "webp"
                | "woff"
                | "woff2"
                | "ttf"
        )
    })
}
fn normalized_admin(runtime: &Runtime, admin: &str) -> Arc<str> {
    if let Some((_, normalized)) = runtime
        .normalized_admin
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .filter(|(configured, _)| configured == admin)
    {
        return normalized.clone();
    }
    let normalized: Arc<str> = probe_path::normalized_path(admin)
        .unwrap_or_else(|| admin.to_owned())
        .into();
    *runtime
        .normalized_admin
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) =
        Some((admin.to_owned(), normalized.clone()));
    normalized
}
fn hostile_normalized_probe(normalized: &str, admin: &str) -> bool {
    // A configured console can have a scanner-like name. Only foreign probes
    // below that console are classified, with whole-segment prefix matching.
    if normalized == admin {
        return false;
    }
    let path = normalized
        .strip_prefix(admin)
        .filter(|suffix| suffix.starts_with('/'))
        .unwrap_or(normalized);
    probe_path::hostile_normalized_probe_path(path)
}

/// `Sec-Fetch-Site` present and neither `none` (typed URL, bookmark) nor
/// `same-origin`: a browser fetched this for another site's page.
fn browser_cross_site(headers: &axum::http::HeaderMap) -> bool {
    headers.get("sec-fetch-site").is_some_and(|value| {
        !value
            .to_str()
            .is_ok_and(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "none" | "same-origin"))
    })
}

/// The fail2ban `failregex` for [`fail2ban_probe_block_line`].
#[cfg_attr(not(test), allow(dead_code))]
pub const FAIL2BAN_PROBE_BLOCK_FAILREGEX: &str =
    r"mailwoman\[request-protection\]: source blocked; .*rhost=<HOST>";

/// One line when a probe-path hit starts an enforced block, in the format of
/// the admin-probe fail2ban line. `rhost` is the canonical client address.
pub fn fail2ban_probe_block_line(
    ts: chrono::DateTime<chrono::Utc>,
    block_secs: u32,
    ip: &str,
) -> String {
    let ip = ip.parse::<std::net::IpAddr>().map_or_else(|_| ip.to_owned(), |ip| ip.to_canonical().to_string());
    format!(
        "{} mailwoman[request-protection]: source blocked; reason=probe-path blockSecs={block_secs} rhost={ip}",
        ts.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    )
}

/// The recorded hit started (or restarted) an enforced block: the full block
/// duration remains. Later hits inside the block report less, so this logs at
/// most about once per source per second while a block is being set.
fn probe_block_started(decision: &Decision, policy: &Policy) -> bool {
    decision.enforced
        && decision.action == mw_store::request_protection_model::Action::Block
        && decision.reason == Some(mw_store::request_protection_model::Reason::ProbePath)
        && decision.retry_after_secs == policy.block_secs
}

/// The source's probe-path gate. `Ok(Some(evidence))` when the session lookup
/// ran (its result is reused by the rest of the guard), `Err` with the refusal.
async fn probe_gate(
    state: &AppState,
    config: &Config,
    headers: &axum::http::HeaderMap,
    path: &str,
    probe: bool,
    source: &str,
    source_country: Option<&str>,
) -> Result<Option<Option<SessionEvidence>>, Response> {
    let Some(policy) = config.probe_policy(source) else { return Ok(None) };
    if policy.adaptive_mode == mw_store::request_protection_model::Mode::Off {
        return Ok(None);
    }
    let probe_subject = format!("probe-path:{source}");
    let now = chrono::Utc::now().timestamp();
    // A request a browser made for another site (an image, script, link or
    // navigation from a third-party page) is not evidence about this source:
    // it is not recorded. Scanners send no Sec-Fetch-Site; a typed URL is `none`.
    let mut record = probe && !browser_cross_site(headers);
    // A verified mailbox or administrator session is neither recorded as a
    // probe nor refused by an active probe block. The lookup runs only for a
    // recordable probe hit or an enforced block, never on ordinary requests.
    let mut session = None;
    if record {
        let evidence = session_evidence_for(state, headers, path).await?;
        record = evidence.is_none();
        session = Some(evidence);
    }
    if matches!(session, Some(Some(_))) {
        return Ok(session);
    }
    let source_meta = mw_store::request_protection::ProtectionSource {
        country: source_country,
        ip: Some(source),
    };
    let (decision, storage_failed) = if let Some(accounting) = state.request_protection.write_behind() {
        let answer = if record {
            accounting
                .record("client", &probe_subject, &policy, now, Some(Signal::ProbePath), source_meta)
                .await
        } else {
            accounting.probe_decision("client", &probe_subject, &policy, now).await
        };
        (answer.decision, answer.storage_failed)
    } else {
        // Strong scanner evidence belongs to the transport source. Subsequent
        // checks never charge an anonymous budget or revise a subject simply
        // because a page was read.
        let decision = if record {
            state
                .store
                .request_protection_with_source(
                    "client",
                    &probe_subject,
                    &policy,
                    now,
                    Some(Signal::ProbePath),
                    source_meta,
                )
                .await
                .map(|outcome| outcome.decision)
        } else {
            state
                .store
                .request_protection_probe_decision("client", &probe_subject, &policy, now)
                .await
        };
        match decision {
            Ok(decision) => (decision, false),
            Err(_) => (Decision::default(), true),
        }
    };
    if storage_failed {
        warning();
        if config.fail_closed
            && policy.adaptive_mode == mw_store::request_protection_model::Mode::Enforce
        {
            return Err(response(unavailable()));
        }
    } else {
        history::note_decision("client", &decision);
    }
    if record && probe_block_started(&decision, &policy) {
        tracing::warn!(
            target: "request_protection",
            "{}",
            fail2ban_probe_block_line(chrono::Utc::now(), policy.block_secs, source)
        );
    }
    if decision.enforced {
        let evidence = match session {
            Some(evidence) => evidence,
            None => session_evidence_for(state, headers, path).await?,
        };
        if evidence.is_none() {
            return Err(response(limited(decision)));
        }
        return Ok(Some(evidence));
    }
    Ok(session)
}

fn bot_protected(path: &str) -> bool {
    let admin = mw_admin::config::admin_path();
    !matches!(
        path,
        "/health"
            | "/healthz"
            | "/ready"
            | "/readyz"
            // Prometheus scrapers send curl/Go/Python user agents. Only the
            // user-agent rules skip it; its bearer token and the rest stay.
            | "/metrics"
            | "/api/logout"
            | "/api/sso/logout"
            | "/api/public-config"
            | "/api/rc/frame"
            | "/api/login/2fa"
            | "/oidc/logout"
            | "/oidc/logout/context"
    ) && !path
        .strip_prefix(&admin)
        .is_some_and(|suffix| matches!(suffix, "/logout" | "/login/2fa" | "/login/recovery"))
}
pub(crate) async fn guard(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let legacy_route = protected(request.uri().path());
    let ua_route = ua_protected(request.uri().path());
    let bot_route = bot_protected(request.uri().path());
    let admin = mw_admin::config::admin_path();
    let normalized_admin = normalized_admin(&state.request_protection, &admin);
    let probe = probe_path::normalized_path(request.uri().path())
        .is_some_and(|path| hostile_normalized_probe(&path, &normalized_admin));
    let source_scope = probe || probe_block_scope(request.uri().path());
    if (request.method() == axum::http::Method::OPTIONS && !probe)
        || (!legacy_route && !ua_route && !source_scope && !bot_route)
    {
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
    if let Err(response) = crate::request_inputs::effect_boundary(&state.store, crate::published_guard_context::Guard::Protection).await { return response; }
    let prepared=crate::published_guard_context::current().await.and_then(|current| {
        let value=current.inputs.protection()?;
        current.used(crate::published_guard_context::Guard::Protection);Some(value)
    });
    // No synthetic legacy revision: published DATA is independently scoped and
    // fenced. Unconsumed readers keep their exact request-revision/cache path.
    let revision=if prepared.is_some(){None}else {Some(match crate::settings_request_revision::get(&state.store,&mut request).await{Ok(value)=>value,Err(_)=>return response(unavailable())})};
    let config = if let Some(config)=prepared {config}else {match state.request_protection.cache.get_at_revision(&state.store,revision.as_ref().expect("original protection revision")).await {
        Ok(config) => config,
        Err(()) => return response(unavailable()),
    }};
    let source = crate::scope_mw::proxy::client_ip(request.headers(), request.extensions())
        .map(|ip| ip.to_canonical());
    let source_country = match mw_engine::security::country_lookup(source) {
        mw_engine::security::CountryLookup::Known(country) => Some(country),
        _ => None,
    };
    let source_ip = source.map(|ip| ip.to_string());
    // Successful evidence belongs only to this request. Handler authentication
    // remains independent; no revocation/expiry decisions survive a request.
    let mut session_evidence = None;
    let mut session_checked = false;
    if source_scope && let Some(source) = source_ip.as_deref() {
        // Boxed: the probe gate is a separate frame, keeping the guard's own
        // future and stack use unchanged on ordinary requests.
        match Box::pin(probe_gate(
            &state,
            &config,
            request.headers(),
            request.uri().path(),
            probe,
            source,
            source_country.as_deref(),
        ))
        .await
        {
            Ok(Some(evidence)) => {
                session_evidence = evidence;
                session_checked = true;
            }
            Ok(None) => {}
            Err(refused) => return refused,
        }
    }
    if let Err(response) = crate::request_inputs::effect_boundary(&state.store, crate::published_guard_context::Guard::Trust).await { return response; }
    let trust = match revision.as_ref() {Some(revision)=>crate::trust_tier::policy_at_revision(&state.store,revision).await,
        None=>crate::trust_tier::policy(&state.store).await};
    let mut ua_allowed = false;
    let ua_claim = (ua_route || bot_route)
        .then(|| {
            config.user_agent.claimed_class(
                request
                    .headers()
                    .get(header::USER_AGENT)
                    .and_then(|value| value.to_str().ok()),
            )
        })
        .flatten();
    if ua_route || bot_route {
        let mut claimed = ua_claim.map(|class| (class, trust.ua_action(class)));
        if let Some((_, action)) = claimed.as_mut()
            && *action == crate::trust_tier::UaAction::Block
        {
            if !session_checked {
                session_evidence = match session_evidence_for(&state, request.headers(), request.uri().path()).await {
                    Ok(evidence) => evidence,
                    Err(response) => return response,
                };
            }
            if session_evidence.is_some() {
                *action = class_outcome(crate::trust_tier::UaAction::Block, true);
            }
        }
        match claimed {
            Some((class, crate::trust_tier::UaAction::Block)) => {
                tracing::info!(target: "request_protection", class, "anonymous request refused by user-agent class action");
                history::note_class_refused(class);
                let mut refused = response(AdminApiError::new(
                    StatusCode::FORBIDDEN,
                    "automatedClientBlocked",
                    "Automated access is not available here",
                ));
                refused.headers_mut().insert(
                    "x-robots-tag",
                    "noindex, nofollow, noai, noimageai".parse().unwrap(),
                );
                return refused;
            }
            Some((_, crate::trust_tier::UaAction::Allow)) => ua_allowed = true,
            _ => {}
        }
    }
    if !config.enabled() {
        return next.run(request).await;
    }
    let ua_category = config
        .user_agent
        .monitored_class(ua_claim)
        .filter(|category| ua_route || config.user_agent.blocks_bot(category))
        .filter(|_| !ua_allowed);
    if (!legacy_route || !config.legacy_enabled()) && ua_category.is_none() {
        return next.run(request).await;
    }
    let display_proof = crate::image_proxy::remote_content::display_proof(
        &state,
        request.method(),
        request.uri().path(),
    )
    .await;
    if let Some(proof) = &display_proof {
        request.extensions_mut().insert(proof.clone());
    }
    let trusted_percent = if source.is_some_and(crate::trust_tier::is_known) {
        if let Err(response) = crate::request_inputs::effect_boundary(&state.store, crate::published_guard_context::Guard::GlobalKnobs).await { return response; }
        match revision.as_ref() {
            Some(revision)=>crate::rate_leeway::knobs_at_revision(&state.store,"",revision).await,
            None=>crate::rate_leeway::knobs(&state.store,"").await,
        }.trusted_percent
    } else {
        100
    };
    let context = Arc::new(Context {
        state,
        config,
        legacy_route,
        source_country,
        source_ip,
        display_subresource: display_proof.is_some(),
        tier: AtomicU8::new(0),
        trust,
        trusted_percent,
        charged: Default::default(),
        signal: AtomicU8::new(0),
    });
    let anonymous_only = sign_in_route(request.uri().path());
    REQUEST
        .scope(context.clone(), async move {
            // t97: a request with a valid mailbox or admin session is charged to
            // that identity (per account or administrator, a larger allowance),
            // not to its source address; anonymous requests and every sign-in or
            // identity-confirmation route stay on the strict per-address budget.
            let identified = if let Some(proof) = &display_proof {
                if let Err(limited) = session_account(proof.session()).await {
                    return limited;
                }
                true
            } else if anonymous_only {
                false
            } else {
                match identify(
                    &context.state,
                    request.headers(),
                    request.uri().path(),
                    session_evidence,
                )
                .await
                {
                    Ok(found) => found,
                    Err(limited) => return limited,
                }
            };
            context.tier.store(
                match crate::trust_tier::tier_for(source, identified) {
                    crate::trust_tier::Tier::Authenticated => 2,
                    crate::trust_tier::Tier::Known => 1,
                    crate::trust_tier::Tier::Unknown => 0,
                },
                Ordering::Relaxed,
            );
            history::note_tier(context.tier.load(Ordering::Relaxed) as usize);
            if legacy_route
                && !identified
                && let Some(source) = context.source_ip.as_deref()
                && let Err(error) = context.admit("client", source).await
            {
                return response(error);
            }
            if (!identified || !context.config.user_agent.anonymous_only)
                && let Some(kind) = ua_category.and_then(user_agent::kind)
                && let Some(source) = context.source_ip.as_deref()
                && let Err(error) = context.admit(kind, source).await
            {
                return response(error);
            }
            let result = next.run(request).await;
            context.observe(result.status()).await;
            result
        })
        .await
}

/// Evidence from a lookup performed earlier in this request, never a cache.
enum SessionEvidence {
    Administrator(String),
    Native(String),
    Mailbox(mw_store::Session),
}
async fn session_evidence_for(
    state: &AppState,
    headers: &axum::http::HeaderMap,
    path: &str,
) -> Result<Option<SessionEvidence>, Response> {
    let admin = mw_admin::config::admin_path();
    if path
        .strip_prefix(&admin)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    {
        return Ok(crate::admin_ctx::AdminCtx::resolve(state, headers)
            .await.ok().map(|ctx| SessionEvidence::Administrator(
                ctx.account_id.unwrap_or(ctx.username),
            )));
    }
    if let Some(token) = crate::push_relay::bearer_token(headers) {
        let hash = crate::push_relay::hash_token(&token);
        return Ok(matches!(state.store.get_native_session(&hash).await, Ok(Some(_)))
            .then_some(SessionEvidence::Native(token)));
    }
    let Some(id) = crate::cookie_value(headers) else { return Ok(None); };
    if let Some(session) = crate::request_inputs::stored_session(
        &state.store, &id, crate::published_guard_context::Guard::Protection,
    ).await? {
        return Ok(Some(SessionEvidence::Mailbox(session)));
    }
    #[cfg(test)] crate::request_inputs::test_control::record(|counts| counts.legacy_guard_session_reads += 1);
    Ok(state
        .store
        .get_primary_session(&id)
        .await
        .ok()
        .map(SessionEvidence::Mailbox))
}

/// Sign-in, second-factor, recovery, password and identity-confirmation routes:
/// always on the per-address budget, whatever cookie the request carries.
fn sign_in_route(path: &str) -> bool {
    let admin = mw_admin::config::admin_path();
    if let Some(suffix) = path.strip_prefix(&admin) {
        return suffix == "/login"
            || suffix.starts_with("/login/")
            || suffix.starts_with("/step-up")
            || suffix == "/me/password"
            || suffix.starts_with("/passkeys/login");
    }
    path == "/api/login"
        || path.starts_with("/api/login/")
        || path.starts_with("/api/sso/")
        || path.starts_with("/api/security/step-up")
        || path.starts_with("/api/password")
        || path.starts_with("/api/passkeys/login")
        || path == "/oauth/token"
        || path.starts_with("/oidc/")
}

/// Whether the request carries a valid session, charging its identity when it
/// does. `Err` is the refusal when that identity is over its own budget.
async fn identify(
    state: &AppState,
    headers: &axum::http::HeaderMap,
    path: &str,
    evidence: Option<SessionEvidence>,
) -> Result<bool, Response> {
    let native_token = match evidence {
        Some(SessionEvidence::Administrator(subject)) => {
            administrator(&subject).await.map_err(response)?;
            return Ok(true);
        }
        Some(SessionEvidence::Mailbox(row)) => {
            session_account(&row).await?;
            return Ok(true);
        }
        Some(SessionEvidence::Native(token)) => Some(token),
        None => None,
    };
    let admin = mw_admin::config::admin_path();
    if path
        .strip_prefix(&admin)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    {
        return match crate::admin_ctx::AdminCtx::resolve(state, headers).await {
            Ok(_) => Ok(true),
            Err(error) if error.status == StatusCode::TOO_MANY_REQUESTS => Err(response(error)),
            Err(_) => Ok(false),
        };
    }
    let session = if let Some(token) = native_token {
        token
    } else if let Some(token) = crate::push_relay::bearer_token(headers) {
        let hash = crate::push_relay::hash_token(&token);
        if !matches!(state.store.get_native_session(&hash).await, Ok(Some(_))) {
            return Ok(false);
        }
        token
    } else if let Some(id) = crate::cookie_value(headers) {
        id
    } else {
        return Ok(false);
    };
    // A stored session only: the in-memory idle clock is left to the handler,
    // so a forged cookie never seeds it and an expired one is still deleted there.
    let row = match crate::request_inputs::stored_session(
        &state.store, &session, crate::published_guard_context::Guard::Protection,
    ).await? {
        Some(row) => row,
        None => {
            #[cfg(test)] crate::request_inputs::test_control::record(|counts| counts.legacy_guard_session_reads += 1);
            let Ok(row) = state.store.get_primary_session(&session).await else { return Ok(false); };
            row
        }
    };
    session_account(&row).await?;
    Ok(true)
}

#[cfg(test)]
mod identity_tests {
    #[test]
    fn cached_console_normalization_tracks_configuration_changes() {
        let runtime = Runtime::default();
        let first = normalized_admin(&runtime, "/Private-Control");
        assert_eq!(&*first, "/private-control");
        assert!(Arc::ptr_eq(
            &first,
            &normalized_admin(&runtime, "/Private-Control")
        ));
        let changed = normalized_admin(&runtime, "/wp-admin");
        assert_eq!(&*changed, "/wp-admin");
        assert!(!Arc::ptr_eq(&first, &changed));
        // Operator recovery cannot inherit an exemption for the former path.
        assert!(!hostile_normalized_probe("/wp-admin", &changed));
        assert!(hostile_normalized_probe(
            "/private-control/wp-admin",
            &changed
        ));
    }
    #[test]
    fn normalized_probe_keeps_console_boundaries_and_resource_namespaces() {
        for admin in ["/private-control", "/wp-admin", "/console"] {
            for input in [
                "/",
                "/healthz",
                "/api/download/.env",
                "/jmap/files/wp-admin",
                "/pdfjs/web/viewer.mjs.map",
                "/assets/app.js",
                "/robots.txt",
                "/bad%GG",
                "/.git/config",
                "/api/cmd_sco",
                "/nested/%2577p-admin",
            ] {
                let reference = probe_path::normalized_path(input).is_some_and(|path| {
                    if path == admin {
                        return false;
                    }
                    match path
                        .strip_prefix(admin)
                        .filter(|suffix| suffix.starts_with('/'))
                    {
                        Some(suffix) => probe_path::hostile_probe_path(suffix),
                        None => probe_path::hostile_probe_path(input),
                    }
                });
                assert_eq!(
                    probe_path::normalized_path(input)
                        .is_some_and(|path| hostile_normalized_probe(&path, admin)),
                    reference,
                    "{admin}: {input}"
                );
            }
            assert!(!hostile_normalized_probe(admin, admin));
            assert!(!hostile_normalized_probe(
                &format!("{admin}/api/files/.env"),
                admin
            ));
            assert!(hostile_normalized_probe(
                &format!("{admin}/nested/wp-admin"),
                admin
            ));
            assert!(hostile_normalized_probe(
                &format!("{admin}-other/wp-admin"),
                admin
            ));
        }
        for path in [
            "/healthz",
            "/readyz",
            "/api/logout",
            "/api/public-config",
            "/api/rc/frame",
            "/assets/app.js",
        ] {
            assert!(!probe_block_scope(path), "{path}");
        }
    }
    #[test]
    fn ua_coverage_includes_documents_but_does_not_confuse_api_suffixes_with_assets() {
        assert!(super::ua_protected("/"));
        assert!(super::ua_protected("/public-document"));
        assert!(super::ua_protected("/api/blob/example.png"));
        assert!(!super::ua_protected("/assets/app.js"));
        assert!(!super::ua_protected("/api/logout"));
        assert!(!super::ua_protected("/health"));
    }
    use super::*;
    #[test]
    fn only_browser_requests_for_another_site_skip_probe_recording() {
        let with = |value: Option<&[u8]>| {
            let mut headers = axum::http::HeaderMap::new();
            if let Some(value) = value {
                headers.insert("sec-fetch-site", axum::http::HeaderValue::from_bytes(value).unwrap());
            }
            browser_cross_site(&headers)
        };
        assert!(!with(None), "scanners send no Sec-Fetch-Site");
        assert!(!with(Some(b"none")) && !with(Some(b"same-origin")) && !with(Some(b"Same-Origin")));
        assert!(with(Some(b"cross-site")) && with(Some(b"same-site")) && with(Some(b"\xff")));
    }
    #[test]
    fn fail2ban_block_line_matches_its_filter_and_logs_only_a_new_block() {
        use mw_store::request_protection_model::{Action, Mode, Reason, WindowPolicy};
        let line = fail2ban_probe_block_line(
            chrono::DateTime::from_timestamp(1_790_000_000, 0).unwrap(),
            300,
            "::ffff:198.51.100.4",
        );
        assert!(line.ends_with("reason=probe-path blockSecs=300 rhost=198.51.100.4"), "{line}");
        assert_eq!(line.matches("rhost=").count(), 1);
        let literal = FAIL2BAN_PROBE_BLOCK_FAILREGEX
            .split(" .*rhost=<HOST>")
            .next()
            .unwrap()
            .replace('\\', "");
        assert!(line.contains(&literal), "{line} / {literal}");
        let window = WindowPolicy { seconds: 60, minimum_samples: 1, absolute_anomalies: 1, ratio_percent: 0 };
        let policy = Policy {
            rate_mode: Mode::Off, adaptive_mode: Mode::Enforce, rate_limit: 0, rate_window_secs: 60,
            short: window, long: WindowPolicy { seconds: 3600, ..window }, normal_cap_per_bucket: 100,
            signals: 0, throttle_secs: 3, block_secs: 300, decay_secs: 3600, max_stage: 2,
        };
        let new = Decision { action: Action::Block, reason: Some(Reason::ProbePath), enforced: true, retry_after_secs: 300 };
        assert!(probe_block_started(&new, &policy));
        assert!(!probe_block_started(&Decision { retry_after_secs: 299, ..new }, &policy), "inside an existing block");
        assert!(!probe_block_started(&Decision { enforced: false, ..new }, &policy), "observe mode");
        assert!(!probe_block_started(&Decision { action: Action::Throttle, ..new }, &policy));
    }
    #[test]
    fn the_shipped_probe_block_filter_carries_the_failregex_in_code() {
        let filter = include_str!("../../../deploy/fail2ban/filter.d/mailwoman-request-protection.conf");
        let lines: Vec<&str> = filter
            .lines()
            .filter_map(|l| l.strip_prefix("failregex"))
            .map(|rest| rest.trim_start().trim_start_matches('=').trim())
            .collect();
        assert_eq!(lines, [FAIL2BAN_PROBE_BLOCK_FAILREGEX]);
        let jail = include_str!("../../../deploy/fail2ban/jail.d/mailwoman.conf");
        assert!(jail.contains("[mailwoman-request-protection]"));
        assert!(jail.contains("filter   = mailwoman-request-protection"));
    }
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
            max_stage: 2,
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
