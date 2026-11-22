//! One HTTP request's verified, versioned INPUT frame.
//!
//! Only sealed source/policy DATA survives in the publisher. The decoded
//! Session below lives in this request's scope, never in the worker, and is not
//! an authentication result. Original callers still perform their checks.
use crate::{AppState, feature_gate::native_metadata_admission::publication::Published};
use axum::{extract::{Request, State}, http::{Method, StatusCode, header},
    middleware::Next, response::{IntoResponse, Response}, Json};
use mw_store::{Session, Store, authority_publication::NativeAuthorityFence};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use tokio::sync::OnceCell;

/// Environment DATA has its own lifetime; SQL epochs cannot certify it.
/// One vars_os snapshot supplies every resolver and parser in a cold build.
/// The standard library's transient snapshot is not a hard allocator bound.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Environment {
    values: std::collections::BTreeMap<&'static str, (u8, Option<std::ffi::OsString>)>,
}
impl Environment {
    pub(crate) fn capture() -> Option<Self> {
        let names = crate::settings_registry::REGISTRY.iter().filter_map(|d| d.env.map(|v| v.0))
            .chain(["MW_PUBLIC_URL"]).collect::<std::collections::BTreeSet<_>>();
        if names.len() > 128 { return None; }
        let mut values = names.into_iter().enumerate().map(|(index, n)| (n, (index as u8, None)))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut bytes = 0usize;
        for (name, value) in std::env::vars_os() {
            let Some(name) = name.to_str() else { continue; };
            let key = values.get_key_value(name).map(|(key, _)| *key).or_else(|| {
                if cfg!(windows) { values.keys().find(|key| key.eq_ignore_ascii_case(name)).copied() }
                else { None }
            });
            let Some((_, slot)) = key.and_then(|key| values.get_mut(key)) else { continue; };
            bytes = bytes.checked_add(value.len())?;
            if slot.is_some() || value.len() > 8192 || bytes > 65536 { return None; }
            *slot = Some(value);
        }
        Some(Self { values })
    }
    pub(crate) fn lookup(&self, name: &str) -> Option<String> {
        self.values.get(name)?.1.as_ref()?.to_str().map(str::to_owned)
    }
    pub(crate) fn matches(&self) -> bool {
        // No reconstructed map or rehash on a freshness boundary. The single
        // stdlib snapshot is still real work, not claimed to be free/bounded
        // by the selected retained values. Exact-name lookups are logarithmic;
        // only Windows case-variant names need the small bounded fallback.
        let mut seen = 0u128;
        for (name, actual) in std::env::vars_os() {
            let Some(name) = name.to_str() else { continue; };
            let found = self.values.get(name).or_else(|| {
                if cfg!(windows) { self.values.iter().find(|(key, _)| key.eq_ignore_ascii_case(name)).map(|(_, value)| value) }
                else { None }
            });
            let Some((index, expected)) = found else { continue; };
            let bit = 1u128 << *index;
            if seen & bit != 0 || expected.as_ref() != Some(&actual) { return false; }
            seen |= bit;
        }
        self.values.values().all(|(index, value)| value.is_none() || seen & (1u128 << *index) != 0)
    }
    pub(crate) fn fingerprint(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        for (name, (_, value)) in &self.values {
            hash.update((name.len() as u64).to_le_bytes()); hash.update(name.as_bytes());
            match value {
                None => hash.update([0]),
                Some(value) => {
                    hash.update([1]);
                    let bytes = value.as_encoded_bytes();
                    hash.update((bytes.len() as u64).to_le_bytes()); hash.update(bytes);
                }
            }
        }
        hash.finalize().iter().map(|byte| format!("{byte:02x}")).collect()
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        self.values.iter().map(|(key, (_, value))| key.len() + value.as_ref().map_or(0, |v| v.len()) + 96).sum()
    }
}

/// Stable typed reader inputs. No Session, password, gate result or deadline
/// result is retained here. The original evaluators consume these per request.
pub(crate) struct Prepared {
    pub(crate) account: mw_store::Account,
    pub(crate) identity: Arc<crate::domain_config::IdentitySnapshot>,
    pub(crate) upstream: crate::upstream_resolve::Upstream,
    pub(crate) configuration: Arc<crate::domain_config::UpstreamConfig>,
    _route: Arc<RouteData>,
    listed: bool,
    idle: u64,
    absolute: u64,
    subject: crate::settings_routes::UserSubject,
    feature_values: Vec<(&'static str, serde_json::Value)>,
    rate_domain: String,
    knobs: crate::rate_leeway::Knobs,
}
/// Parsed domain DATA shared only under exact source/version identity. No
/// expanded user address, credential, signin verdict or connector lives here.
pub(crate) struct RouteData {
    pub(crate) source: String,
    pub(crate) upstream: crate::upstream_resolve::Upstream,
    pub(crate) configuration: Arc<crate::domain_config::UpstreamConfig>,
}
impl RouteData {
    pub(crate) fn build(identity: &crate::domain_config::IdentitySnapshot, listed: bool) -> Option<Self> {
        let row = identity.row.as_ref()?;
        if !row.enabled { return None; }
        let (upstream, configuration) = match crate::upstream_resolve::resolve_configured(identity, true, listed).ok()? {
            crate::upstream_resolve::Configured::Resolved(upstream @ crate::upstream_resolve::Upstream::Mail(_), Some(config)) => (upstream, config),
            _ => return None,
        };
        Some(Self { source: row.upstream_json.clone(), upstream, configuration: Arc::new(configuration) })
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        // Same conservative parser-expansion charge as before, now once per
        // shared component, including its exact raw-input comparison material.
        self.source.len().saturating_mul(64).saturating_add(2048)
    }
}
// Exact union of feature_gate::jmap_features and feature_jmap's family gates.
// Future selectors outside this declared DATA coverage fail closed after use.
const JMAP_FEATURES: &[&str] = &["keys", "vacation", "rules.forward", "compose.scheduled_send",
    "compose.rich_text", "compose.attachments", "compose.read_receipts", "labels", "snooze",
    "follow_up", "views.events", "search.operators", "calendar", "contacts", "tasks", "notes",
    "rules", "import", "export"];
pub(crate) fn policy_keys() -> Option<&'static [&'static str]> {
    static KEYS: std::sync::OnceLock<Option<Vec<&'static str>>> = std::sync::OnceLock::new();
    KEYS.get_or_init(|| {
        let mut keys = vec!["access.protocols", "mail.allowSend", "limit.recipients_per_message"];
        for id in JMAP_FEATURES {
            keys.extend_from_slice(crate::features::feature(id)?.capability_keys);
        }
        keys.sort_unstable(); keys.dedup();
        (keys.len() <= 64).then_some(keys)
    }).as_deref()
}
pub(crate) fn needed(def: &crate::settings_registry::SettingDef,
    subject: &mw_store::settings_policy::CanonicalSettingSubject) -> bool
{
    if !subject.account_id.is_empty() {
        matches!(def.key, "session.idleSecs" | "session.absoluteSecs")
            || policy_keys().is_some_and(|keys| keys.contains(&def.key))
    } else if !subject.address.is_empty() || subject.domain.is_empty() {
        def.key == "signin.domainMode"
    } else { def.key.starts_with("ratelimit.") }
}
impl Prepared {
    pub(crate) fn build(entry: &Published, session: &Session, store: &Store,
        route: impl FnOnce(&crate::domain_config::IdentitySnapshot, bool) -> Option<Arc<RouteData>>,
    ) -> Option<Self> {
        use mw_store::settings_policy::SettingSubject;
        let account = entry.snapshot.account().ok()??;
        let namespaces = entry.snapshot.native_subjects(store).ok()??;
        if session.jmap_url != "engine" || session.api_url != "engine"
            || account.id != session.account_id || account.kind == mw_store::AccountKind::Ews
            || session.credentials.username.trim() != account.username.trim()
            || entry.snapshot.linked_policy_required().ok()?
        { return None; }
        let input = crate::domain_config::QualifiedIdentityInput::parse(&session.credentials.username).ok()??;
        let row = entry.snapshot.domains().remove(&input.domain)?.ok()?;
        if row.second_alias { return None; }
        let identity = Arc::new(crate::domain_config::IdentitySnapshot::from_qualified(
            input, row.canonical, row.configuration, false));
        if !identity.matches_existing_identity(&account.username) { return None; }
        let listed = entry.value(&SettingSubject::default(), "signin.domainMode")?.value == "listed";
        let domain_subject = SettingSubject { personal_owner: None,
            domain: &identity.identity.domain, account_id: "", address: &identity.identity.address };
        let domain_listed = entry.value(&domain_subject, "signin.domainMode")?.value == "listed";
        let route = route(&identity, domain_listed)?;
        let upstream = route.upstream.clone();
        let configuration = route.configuration.clone();
        let clock_subject = SettingSubject { personal_owner: None,
            domain: session.username.rsplit_once('@').map_or("", |(_, domain)| domain),
            account_id: &session.account_id, address: &session.username };
        let idle = entry.value(&clock_subject, "session.idleSecs")?.value.as_u64()
            .unwrap_or(crate::hardening::MAILBOX_IDLE_SECS);
        let absolute = entry.value(&clock_subject, "session.absoluteSecs")?.value.as_u64()
            .unwrap_or(crate::hardening::MAILBOX_ABSOLUTE_CAP_SECS);
        let raw = session.username.trim().to_ascii_lowercase();
        let subject = SettingSubject { personal_owner: None,
            domain: raw.rsplit_once('@')?.1, account_id: &session.account_id, address: &raw };
        let canonical = entry.canonical(&subject)?;
        let feature_values = policy_keys()?.iter().map(|key|
            entry.value(&subject, key).map(|row| (*key, row.value.clone())))
            .collect::<Option<Vec<_>>>()?;
        let rate_domain = crate::rate_leeway::domain_of(&session.username).to_owned();
        let rate_rows = entry.values(&SettingSubject { domain: &rate_domain, ..Default::default() },
            |def| def.key.starts_with("ratelimit."))?;
        let knobs = crate::rate_leeway::Knobs::from_rows(&rate_rows);
        let subject = crate::settings_routes::UserSubject {
            proxy_policy_owner: None,
            personal_owner: Some(namespaces.storage_authority),
            record_owner: Some(namespaces.policy.personal_subject),
            domain: canonical.domain.clone(), account_id: canonical.account_id.clone(),
            address: canonical.address.clone(),
        };
        Some(Self { account, identity, upstream, configuration, _route: route, listed, idle, absolute, subject, feature_values, rate_domain, knobs })
    }
    pub(crate) fn value(&self, key: &str) -> Option<&serde_json::Value> {
        self.feature_values.iter().find(|(name, _)| *name == key).map(|(_, value)| value)
    }
    pub(crate) fn rate_knobs(&self, domain: &str) -> Option<crate::rate_leeway::Knobs> {
        (domain == self.rate_domain).then_some(self.knobs)
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        // Conservative logical retention charge, not an allocator/RSS claim.
        let account = [&self.account.id, &self.account.host, &self.account.tls,
            &self.account.username, &self.account.sync_policy_json].iter().map(|s| s.len()).sum::<usize>();
        let routing = self.identity.row.as_ref().map_or(0, |row| row.upstream_json.len());
        let features = self.feature_values.iter().fold(0usize, |bytes, (key, value)|
            bytes.saturating_add(key.len()).saturating_add(serde_json::to_vec(value)
                .map_or(usize::MAX, |v| v.len())).saturating_add(96));
        // Expanded per-user identity still owns raw domain metadata; the parsed
        // configuration's conservative charge belongs to the shared route.
        account.saturating_mul(2).saturating_add(routing.saturating_mul(2))
            .saturating_add(features).saturating_add(2048)
    }
}

struct Scope {
    state: AppState,
    cookie: String,
    first: OnceCell<Option<Arc<Current>>>,
    body: Mutex<Option<[u8; 32]>>,
    held_allowed: bool,
}
pub(crate) struct Current {
    pub(crate) entry: Arc<Published>,
    first: Arc<NativeAuthorityFence>,
    current: Mutex<Arc<NativeAuthorityFence>>,
    // A single request-local decode, shared by the original reader boundaries.
    session: Session,
    used: AtomicBool,
    pub(crate) guard: std::sync::OnceLock<Arc<crate::published_guard_context::Current>>,
}
tokio::task_local! { static SCOPE: Arc<Scope>; static ORIGINAL_READER: bool; }

pub(crate) async fn layer(State(state): State<AppState>, request: Request, next: Next) -> Response {
    #[cfg(feature="load-test-fixtures")]
    if !crate::request_inputs_fixture::enabled() { return next.run(request).await; }
    #[cfg(test)]
    if !test_control::enabled() { return next.run(request).await; }
    let eligible_route = (request.method() == Method::GET && request.uri().path() == "/api/me")
        || (request.method() == Method::POST && request.uri().path() == "/jmap/api");
    let headers = request.headers();
    // Selector expansion and other credential kinds retain their complete
    // original path. No body is consumed or source work queued here.
    if !eligible_route || request.uri().query().is_some() || state.engine.is_none()
        || !state.store.uses_sqlite()
        || crate::session_persistence::flush_seconds() == 0
        || headers.contains_key(header::AUTHORIZATION) || headers.contains_key("x-api-key")
        || headers.contains_key(crate::linked_accounts::SELECTOR)
        || headers.contains_key("x-mailwoman-read-shape")
        || crate::websocket_actions::has_frame_actor()
    { return next.run(request).await; }
    let Some(cookie) = crate::cookie_value(headers).filter(|s| !s.is_empty() && s.len() <= 1024)
    else { return next.run(request).await; };
    let held_allowed = request.uri().path() == "/api/me";
    #[cfg(feature="load-test-fixtures")]
    crate::request_inputs_fixture::record(crate::request_inputs_fixture::Event::Context);
    let scope = Arc::new(Scope { state, cookie, first: OnceCell::new(), body: Mutex::new(None), held_allowed });
    SCOPE.scope(scope, next.run(request)).await
}

pub(crate) async fn current(store: &Store) -> Option<Arc<Current>> {
    if ORIGINAL_READER.try_with(|bypass| *bypass).unwrap_or(false) { return None; }
    let scope = SCOPE.try_with(Arc::clone).ok()?;
    if scope.state.store.cache_scope() != store.cache_scope() { return None; }
    scope.first.get_or_init(|| async {
        let environment = Environment::capture()?;
        let entry = scope.state.native_publication.as_ref()?.request_inputs(&scope.cookie, &environment)?;
        let prepared = entry.request.as_ref()?;
        if entry.environment.as_deref()? != &environment { return None; }
        let guard = entry.guard.as_ref()?;
        if !guard.ordinary_supported() { return None; }
        #[cfg(feature="load-test-fixtures")]
        crate::request_inputs_fixture::record(crate::request_inputs_fixture::Event::FirstFence);
        let fence = store.native_publication_fence(&scope.cookie, &entry.domains, &entry.subjects)
            .await.ok()??;
        #[cfg(test)] test_control::record(|counts| counts.first_fences += 1);
        if !entry.environment.as_ref()?.matches()
            || !entry.matches(&fence) || !guard.matches_configuration(store, &fence).ok()?
            || fence.primary_policy_revision(store).ok()??.number().is_none()
        {
            scope.state.native_publication.as_ref()?.discard_request_inputs(&scope.cookie, &environment, &entry);
            return None;
        }
        #[cfg(test)] test_control::record(|counts| counts.session_decodes += 1);
        let source = entry.snapshot.session(store).ok()??;
        let session = source.session;
        let account = &prepared.account;
        // Classify unsupported frame shapes before publishing ANY input into
        // original readers. In particular a linked-login policy is not absent.
        if session.id != fence.session_id() || session.account_id != fence.account_id()
            || account.id != session.account_id || session.jmap_url != "engine"
            || session.api_url != "engine" || account.kind == mw_store::AccountKind::Ews
            || session.credentials.username.trim() != account.username.trim()
        { return None; }
        let (created, activity) = fence.clocks();
        if chrono::DateTime::parse_from_rfc3339(created).is_err()
            || chrono::DateTime::parse_from_rfc3339(activity).is_err()
            || store.session_touch_may_be_due_at(fence.last_seen(),
                crate::session_persistence::flush_seconds(), chrono::Utc::now())
        { return None; }
        let first = Arc::new(fence);
        #[cfg(feature="load-test-fixtures")]
        crate::request_inputs_fixture::record(crate::request_inputs_fixture::Event::Ready);
        Some(Arc::new(Current { entry, current: Mutex::new(first.clone()), first,
            session, used: AtomicBool::new(false), guard: std::sync::OnceLock::new() }))
    }).await.clone()
}
pub(crate) async fn scope_current() -> Option<Arc<Current>> {
    let scope = SCOPE.try_with(Arc::clone).ok()?;
    current(&scope.state.store).await
}

/// Called only at the owning CONFIG guard's explicit pre-Next return. Never
/// replaces a downstream response after CORS/security/status decoration.
pub(crate) async fn terminal(response: Response) -> Response {
    let Ok(scope) = SCOPE.try_with(Arc::clone) else { return response; };
    let Some(Some(current)) = scope.first.get() else { return response; };
    if !current.used.load(Ordering::Relaxed) { return response; }
    if reobserve(&scope, current).await.is_ok() { response } else { refused() }
}

/// A CONFIG effect must not borrow an older first guard's version. Call this
/// at its ORIGINAL load point, even when the earlier configuration was off.
/// Changing consumed input is a terminal refusal, never an Option miss that
/// falls into a different loader or lets an old disabled branch skip effects.
pub(crate) async fn effect_boundary(store: &Store, guard: crate::published_guard_context::Guard) -> Result<(), Response> {
    let Some(current) = current(store).await else { return Ok(()); };
    let scope = SCOPE.try_with(Arc::clone).map_err(|_| refused())?;
    #[cfg(test)] test_control::before_effect(guard).await;
    #[cfg(not(test))] let _ = guard;
    reobserve(&scope, &current).await.map_err(|_| refused())
}

/// Borrow stored DATA for the same actor at the original guard's load point.
/// Live guard decisions and effects remain with that guard. A changed frame is
/// a refusal; it must never become anonymous evidence or a legacy-loader miss.
pub(crate) async fn stored_session(
    store: &Store,
    id: &str,
    guard: crate::published_guard_context::Guard,
) -> Result<Option<Session>, Response> {
    let Some(inputs) = current(store).await else { return Ok(None); };
    if inputs.session.id != id { return Ok(None); }
    #[cfg(test)] test_control::before_stored(guard).await;
    effect_boundary(store, guard).await?;
    inputs.consume();
    #[cfg(test)] test_control::record(|counts| counts.stored_reader_hits += 1);
    Ok(Some(inputs.session.clone()))
}

pub(crate) fn active() -> bool {
    !ORIGINAL_READER.try_with(|bypass| *bypass).unwrap_or(false)
        && SCOPE.try_with(|scope| matches!(scope.first.get(), Some(Some(_)))).unwrap_or(false)
}

#[cfg(test)]
pub(crate) mod test_control {
    #[derive(Clone, Copy, Default, Debug)]
    pub(crate) struct Counts {
        pub first_fences: u64, pub refresh_fences: u64, pub session_decodes: u64,
        pub legacy_session_reads: u64, pub failure_classifications: u64,
        pub stored_reader_hits: u64, pub legacy_guard_session_reads: u64,
    }
    tokio::task_local! { static ENABLED: bool; static COUNTS: std::cell::RefCell<Counts>;
        static CONNECTOR: std::sync::Arc<tokio::sync::Notify>;
        static EFFECT: std::sync::Arc<EffectPause>;
        static STORED: std::sync::Arc<EffectPause>; }
    pub(crate) struct EffectPause {
        guard: u32,
        pub entered: tokio::sync::Notify,
        pub release: tokio::sync::Notify,
    }
    impl EffectPause {
        pub(crate) fn new(guard: crate::published_guard_context::Guard) -> std::sync::Arc<Self> {
            std::sync::Arc::new(Self { guard: guard as u32,
                entered: tokio::sync::Notify::new(), release: tokio::sync::Notify::new() })
        }
    }
    pub(super) fn enabled() -> bool { ENABLED.try_with(|enabled| *enabled).unwrap_or(false) }
    pub(crate) fn record(update: impl FnOnce(&mut Counts)) {
        let _ = COUNTS.try_with(|counts| update(&mut counts.borrow_mut()));
    }
    pub(crate) async fn scope<F: std::future::Future>(enabled: bool, future: F) -> F::Output {
        ENABLED.scope(enabled, future).await
    }
    pub(crate) async fn measure<F: std::future::Future>(enabled: bool, future: F) -> (F::Output, Counts) {
        COUNTS.scope(std::cell::RefCell::new(Counts::default()), async {
            let output = scope(enabled, future).await;
            let counts = COUNTS.with(|counts| *counts.borrow());
            (output, counts)
        }).await
    }
    pub(crate) fn connector_waiting() {
        let _ = CONNECTOR.try_with(|entered| entered.notify_one());
    }
    pub(crate) async fn connector_probe<F: std::future::Future>(entered: std::sync::Arc<tokio::sync::Notify>, future: F) -> F::Output {
        CONNECTOR.scope(entered, future).await
    }
    pub(super) async fn before_effect(guard: crate::published_guard_context::Guard) {
        let Ok(pause) = EFFECT.try_with(std::sync::Arc::clone) else { return; };
        if pause.guard != guard as u32 { return; }
        pause.entered.notify_one();
        tokio::time::timeout(std::time::Duration::from_secs(10), pause.release.notified())
            .await.expect("bounded actual guard-load pause");
    }
    pub(super) async fn before_stored(guard: crate::published_guard_context::Guard) {
        let Ok(pause) = STORED.try_with(std::sync::Arc::clone) else { return; };
        if pause.guard != guard as u32 { return; }
        pause.entered.notify_one();
        tokio::time::timeout(std::time::Duration::from_secs(10), pause.release.notified())
            .await.expect("bounded actual stored-session reader pause");
    }
    pub(crate) async fn stored_probe<F: std::future::Future>(pause: std::sync::Arc<EffectPause>, future: F) -> F::Output {
        STORED.scope(pause, future).await
    }
    pub(crate) async fn effect_probe<F: std::future::Future>(pause: std::sync::Arc<EffectPause>, future: F) -> F::Output {
        EFFECT.scope(pause, future).await
    }
}

impl Current {
    pub(crate) fn matches_session(&self, session: &Session) -> bool {
        session.id == self.session.id && session.account_id == self.session.account_id
            && session.username == self.session.username
    }
    pub(crate) fn matches_subject(&self, subject: &crate::settings_routes::UserSubject) -> bool {
        self.entry.request.as_ref().is_some_and(|inputs| {
            let expected = &inputs.subject;
            expected.account_id == subject.account_id && expected.domain == subject.domain
                && expected.address == subject.address && expected.proxy_policy_owner == subject.proxy_policy_owner
                && expected.personal_owner == subject.personal_owner && expected.record_owner == subject.record_owner
        })
    }
    pub(crate) fn session(&self) -> &Session {
        self.consume();
        &self.session
    }
    pub(crate) fn source(&self) -> Option<Arc<NativeAuthorityFence>> {
        self.current.lock().ok().map(|s| s.clone())
    }
    pub(crate) fn consume(&self) {
        #[cfg(feature="load-test-fixtures")]
        if !self.used.swap(true,Ordering::Relaxed) {
            crate::request_inputs_fixture::record(crate::request_inputs_fixture::Event::Consumed);
        }
        #[cfg(not(feature="load-test-fixtures"))]
        self.used.store(true,Ordering::Relaxed);
    }

    pub(crate) fn gate(&self, account: bool) -> Result<crate::account_gate::AccountGate, mw_store::StoreError> {
        let inputs = self.prepared()?;
        self.consume();
        Ok(self.entry.gates[usize::from(account)].evaluate(&self.entry.directory, inputs.listed))
    }
    pub(crate) fn prepared(&self) -> Result<&Prepared, mw_store::StoreError> {
        self.entry.request.as_deref().ok_or(mw_store::StoreError::Invalid("request input typed coverage"))
    }
    pub(crate) fn subject(&self) -> Result<crate::settings_routes::UserSubject, mw_store::StoreError> {
        let source = &self.prepared()?.subject;
        self.consume();
        Ok(crate::settings_routes::UserSubject {
            proxy_policy_owner: source.proxy_policy_owner.clone(),
            personal_owner: source.personal_owner.clone(), record_owner: source.record_owner.clone(),
            domain: source.domain.clone(), account_id: source.account_id.clone(), address: source.address.clone(),
        })
    }
    pub(crate) fn clock_policy(&self, state: &AppState) -> Result<crate::security_center::ClockPolicy, mw_store::StoreError> {
        let inputs = self.prepared()?;
        self.consume();
        Ok(crate::security_center::ClockPolicy {
            idle_secs: inputs.idle.min(state.hardening.idle_timeout.as_secs()),
            absolute_secs: crate::hardening::mailbox_absolute_secs(inputs.absolute,
                state.hardening.absolute_timeout.as_secs()),
        })
    }
}

/// Queue only after the ORIGINAL authentication chain succeeds. Invalid
/// cookies and header-only traffic cannot allocate publication work.
pub(crate) fn authenticated(session: &Session) {
    #[cfg(feature="load-test-fixtures")]
    if !crate::request_inputs_fixture::enabled() { return; }
    if ORIGINAL_READER.try_with(|bypass| *bypass).unwrap_or(false) { return; }
    if active() { return; }
    let _ = SCOPE.try_with(|scope| {
        if scope.cookie == session.id && session.jmap_url == "engine"
            && let Some(worker) = &scope.state.native_publication
        { worker.hint_request_inputs(&scope.cookie, &session.account_id); }
    });
}

fn refused() -> Response {
    let mut response = (StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"code":"requestInputSourceChanged","notDispatched":true})))
        .into_response();
    response.headers_mut().insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    response.headers_mut().insert(header::RETRY_AFTER, "1".parse().unwrap());
    response
}

/// Body waits and dispatch waits each require a NEW database observation.
/// Changed consumed input can never fall back into an inner-only auth retry.
pub(crate) async fn refresh() -> Result<(), Response> {
    let Ok(scope) = SCOPE.try_with(Arc::clone) else { return Ok(()); };
    let Some(Some(current)) = scope.first.get() else { return Ok(()); };
    if !current.used.load(Ordering::Relaxed) { return Ok(()); }
    if reobserve(&scope, current).await.is_err() { return Err(classify_failure(&scope).await); }
    Ok(())
}

async fn reobserve(scope: &Scope, current: &Current) -> Result<(), ()> {
    #[cfg(feature="load-test-fixtures")]
    crate::request_inputs_fixture::record(crate::request_inputs_fixture::Event::RefreshFence);
    #[cfg(test)] test_control::record(|counts| counts.refresh_fences += 1);
    let observed = scope.state.store.native_publication_fence(&scope.cookie,
        &current.entry.domains, &current.entry.subjects).await.map_err(|_| ())?.ok_or(())?;
    if !current.first.same_published_source(&observed).unwrap_or(false)
        || !current.entry.matches(&observed)
        || !current.entry.environment.as_ref().is_some_and(|environment| environment.matches())
        || !current.entry.guard.as_ref().is_some_and(|guard|
            guard.matches_configuration(&scope.state.store, &observed).unwrap_or(false))
    { return Err(()); }
    *current.current.lock().map_err(|_| ())? = Arc::new(observed);
    Ok(())
}

/// The Engine adapter's existing public error type is String. Classify no
/// authentication here: the owning HTTP handler seals/classifies its refusal
/// once before returning an upstream error, preserving terminal cleanup/body.
pub(crate) async fn connector_boundary() -> Result<(), String> {
    let Ok(scope) = SCOPE.try_with(Arc::clone) else { return Ok(()); };
    let Some(Some(current)) = scope.first.get() else { return Ok(()); };
    reobserve(&scope, current).await.map_err(|_| "request input source changed".to_owned())
}

async fn classify_failure(scope: &Scope) -> Response {
    ORIGINAL_READER.scope(true, failure_only(scope)).await
}

async fn failure_only(scope: &Scope) -> Response {
    #[cfg(test)] test_control::record(|counts| counts.failure_classifications += 1);
    // This path can classify a terminal failure and perform its cleanup, but
    // cannot perform successful auth effects: no check/reseed, session touch,
    // adaptive account admission, target adoption, queue hint or connector.
    let state = &scope.state;
    let Ok((mut session, _)) = state.store.get_primary_session_with_activity(&scope.cookie).await
        else { return crate::unauthorized(); };
    if state.sessions.expired(&scope.cookie, &state.hardening).is_some() {
        let _ = state.store.delete_session(&scope.cookie).await;
        state.sessions.forget(&scope.cookie);
        return crate::session_expired();
    }
    if let Err(response) = crate::security_center::enforce_session_clock(state, &session).await {
        return response;
    }
    match crate::linked_login::blocked(state, &session).await {
        Ok(true) => return crate::unauthorized(),
        Err(error) => return crate::account_gate::unavailable("linked login policy", error),
        Ok(false) => {}
    }
    // This context only supports Engine sessions. The original Engine domain
    // evaluator has no successful repoint/write path, unlike proxy discovery.
    if let Err(response) = crate::domain_upgrade::refresh_session(state, &mut session).await {
        let _ = state.store.delete_session(&session.id).await;
        state.sessions.forget(&session.id);
        crate::account_gate::stop_engine_work(state, &session.account_id);
        return response;
    }
    match crate::account_gate::for_session(&state.store, &session).await {
        Err(error) => crate::account_gate::unavailable("session", error),
        Ok(gate) if gate.disabled => {
            let _ = state.store.delete_session(&session.id).await;
            state.sessions.forget(&session.id);
            crate::account_gate::stop_engine_work(state, &session.account_id);
            crate::unauthorized()
        }
        Ok(gate) if gate.password_change_required && !scope.held_allowed => crate::account_gate::password_change_required(),
        Ok(_) => refused(),
    }
}

pub(crate) async fn seal() -> Result<(), Response> {
    refresh().await?;
    let Ok(scope) = SCOPE.try_with(Arc::clone) else { return Ok(()); };
    let Some(Some(current)) = scope.first.get() else { return Ok(()); };
    if !current.used.load(Ordering::Relaxed) { return Ok(()); }
    // Pure clock/process decisions are evaluated using NOW and the LAST source
    // observation. The original timeout helper owns any terminal cleanup.
    if scope.state.sessions.expired(&current.session.id, &scope.state.hardening).is_some() {
        return Err(classify_failure(&scope).await);
    }
    crate::security_center::enforce_session_clock(&scope.state, &current.session).await
}

pub(crate) async fn body_boundary(body: &[u8]) -> Result<(), Response> {
    use sha2::{Digest, Sha256};
    let Ok(scope) = SCOPE.try_with(Arc::clone) else { return Ok(()); };
    let digest: [u8; 32] = Sha256::digest(body).into();
    let previous = *scope.body.lock().map_err(|_| refused())?;
    if let Some(previous) = previous {
        return if previous == digest { Ok(()) } else { Err(refused()) };
    }
    // An initial lookup begun only now is already after the body wait. An
    // earlier consumed frame must be freshly re-observed, not silently reused.
    current(&scope.state.store).await;
    refresh().await?;
    *scope.body.lock().map_err(|_| refused())? = Some(digest);
    Ok(())
}

pub(crate) async fn revision(store: &Store, after_body: bool)
    -> Option<mw_store::settings_policy::SettingsReadRevision>
{
    if after_body && !SCOPE.try_with(|scope| scope.body.lock().ok().is_some_and(|body| body.is_some())).ok()? {
        return None;
    }
    let inputs = current(store).await?;
    let source = inputs.source()?;
    let revision = source.primary_policy_revision(store).ok()??.clone();
    inputs.consume();
    Some(revision)
}
