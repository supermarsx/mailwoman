//! Per-request session watch: binding, theft detection and country alerts.
//!
//! A middleware that runs on every cookie-authenticated web request (webmail
//! `/api/*` and `/jmap/*`, and the admin console). It never authenticates; it
//! looks at a session the request already presents and at where it comes from:
//!
//! 1. The first request seen for a session creates its record (client
//!    fingerprint, country, network), a few hundred bytes of hashes and country
//!    codes in the settings table under `sess-sec:v1:<sha256 of the id>`.
//!    Sessions that predate the feature are bound the same way on first sight.
//! 2. Later requests are compared: client mismatch ([`crate::session_binding`]),
//!    impossible travel, concurrent use and country change
//!    ([`crate::session_theft`]).
//! 3. The response follows the administrator's policy: alert only, **step up**
//!    (the session is flagged and every request except sign-out and the
//!    step-up route is answered `403 step_up_required` until the user confirms
//!    their password and second factor, which the existing step-up dialog does),
//!    or **revoke** (the session is deleted and the request answered 401).
//! 4. `/api/session/rotate` calls [`on_rotate`]: the new id inherits the record
//!    and the old id leaves a marker. Presenting a retired id again after the
//!    grace window revokes the whole family and alerts.
//!
//! Exempt: requests with an `Authorization` header or `x-api-key` (native shells
//! and scoped keys), `/dav`, `/oauth`, `/mcp` and every sign-in route. They have
//! no browser fingerprint.
//!
//! State is cached in memory (bounded) and written when something changes or at
//! most every two minutes. Records older than `security.theft.retentionDays`
//! are swept hourly. A restart loses nothing: the records and markers are in
//! the store. With several server processes sharing one store the caches can
//! lag; the next write wins.
use crate::{
    AppState, sec_cfg, security_alerts as alerts,
    session_binding::{self, Action as BindAction, Verdict},
    session_theft::{self as theft, Finding, Reuse, SessionSec, Tombstone},
};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

const STATE_PREFIX: &str = "sess-sec:v1:";
const TOMB_PREFIX: &str = "sess-rot:v1:";
const CACHE_MAX: usize = 20_000;
const SAVE_EVERY_SECS: i64 = 120;
const SWEEP_EVERY_SECS: i64 = 3600;
const KIND_DEBOUNCE_SECS: i64 = 600;

struct Cached {
    sec: SessionSec,
    saved_at: i64,
}
fn cache() -> &'static Mutex<HashMap<String, Cached>> {
    static C: OnceLock<Mutex<HashMap<String, Cached>>> = OnceLock::new();
    C.get_or_init(Default::default)
}
fn cache_lock() -> std::sync::MutexGuard<'static, HashMap<String, Cached>> {
    cache().lock().unwrap_or_else(|p| p.into_inner())
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

struct Target {
    admin: bool,
    token: String,
}

fn classify(request: &Request) -> Option<Target> {
    let headers = request.headers();
    if headers.contains_key(axum::http::header::AUTHORIZATION) || headers.contains_key("x-api-key")
    {
        return None;
    }
    let path = request.uri().path();
    let admin_base = mw_admin::config::admin_path();
    if let Some(suffix) = path
        .strip_prefix(&admin_base)
        .filter(|s| s.starts_with('/'))
    {
        if suffix == "/login" || suffix.starts_with("/login/") || suffix == "/logout" {
            return None;
        }
        let token = crate::hardening::cookie(headers, &mw_admin::config::session_cookie_name())?;
        return Some(Target { admin: true, token });
    }
    let webmail = (path.starts_with("/api/") || path.starts_with("/jmap/"))
        && path != "/api/logout"
        && path != "/api/public-config"
        && !path.starts_with("/api/login")
        && !path.starts_with("/api/sso/")
        && !path.starts_with("/api/signin/");
    if !webmail {
        return None;
    }
    Some(Target {
        admin: false,
        token: crate::cookie_value(headers)?,
    })
}

/// Routes a flagged session may still call: sign out and prove it is the holder.
fn flagged_exempt(path: &str) -> bool {
    path == "/api/logout"
        || path.starts_with("/api/security/step-up")
        || path.contains("/step-up")
        || path.ends_with("/logout")
        || path.starts_with("/api/account/security/session-alerts")
}

pub(crate) async fn guard(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let Some(target) = classify(&request) else {
        return next.run(request).await;
    };
    let inspected = inspect(
        &state,
        &target,
        request.headers(),
        request.extensions(),
        request.uri().path(),
    );
    #[cfg(test)]
    let inspected = test_control::observe(&state, inspected);
    match inspected.await {
        Some(response) => response,
        None => next.run(request).await,
    }
}

pub(crate) struct Config {
    binding: session_binding::Policy,
    rules: theft::Rules,
    webmail_action: u8,
    admin_action: u8,
}
// 0 alert, 1 step-up, 2 revoke
fn theft_action(s: &str) -> u8 {
    match s {
        "revoke" => 2,
        "step-up" => 1,
        _ => 0,
    }
}
async fn config(state: &AppState) -> std::sync::Arc<Config> {
    crate::fixture_profile::measure(crate::fixture_profile::Phase::SessionWatchConfig, config_inner(state)).await
}
async fn config_inner(state: &AppState) -> std::sync::Arc<Config> {
    if let Some(current)=crate::published_guard_context::current().await {
        current.used(crate::published_guard_context::Guard::Watch);return current.inputs.watch();
    }
    let b = sec_cfg::values(&state.store, session_binding::PREFIX).await;
    let t = sec_cfg::values(&state.store, "security.theft.").await;
    let a = sec_cfg::values(&state.store, "security.alerts.").await;
    std::sync::Arc::new(from_values(&b,&t,&a))
}
pub(crate) fn prepare_published_inputs(rows:&[crate::settings_registry::Effective])->std::sync::Arc<Config> {
    let values=sec_cfg::Values::from_pairs(rows.iter().map(|r|(r.key.clone(),r.value.clone())));
    std::sync::Arc::new(from_values(&values,&values,&values))
}
fn from_values(b:&sec_cfg::Values,t:&sec_cfg::Values,a:&sec_cfg::Values)->Config {
    Config {
        binding: session_binding::Policy::from_values(b),
        rules: theft::Rules::from_values(t, a),
        webmail_action: theft_action(&t.text("security.theft.webmailAction")),
        admin_action: theft_action(&t.text("security.theft.adminAction")),
    }
}

// ─── t34 bounded guard snapshot: per-request settings reads ─────────────────
//
// With `MW_GUARD_SNAPSHOT=bounded` (SQLite, live mirror) and the request on
// published inputs, the settings rows that [`alerts::user_prefs`] and
// [`crate::geo_lookup::session_alerts_on_failure`] read are reused while
// `EpochMirror::globals_unchanged(G)` holds, `G` stamped strictly before the
// reads. Those rows sit behind the settings revision (0078 triggers on
// setting_policy / setting_value / personal_setting_value / user_group* /
// feature_profile* / domain_alias; 0123 the four guard keys of the key-value
// settings table, and no longer the identity tables), which the 0115 change log
// carries to the mirror; a local write through HTTP expedites the mirror
// (request layer). The 0078 triggers are required definitions of published
// inputs on SQLite, so a dropped one disqualifies published inputs and with it
// this reuse.
//
// The settings subject is the account's sign-in address (`signin_identity`),
// which has no trigger: it is read fresh on every request (one primary-key
// read) and a cached entry is used only for the same account AND the same
// address string, so a sign-in under another alias or letter case costs that
// account one fresh resolution and touches nothing global.
//
// Per-request inputs (client address, user agent, client hints, country) are
// never cached here. Without a live mirror or published inputs (mode off,
// PostgreSQL, invalid configuration, unpublished request) the original readers
// run unchanged.

/// The alert-preference rows one account's request reads, as read for `address`.
struct PrefInputs {
    address: String,
    country_change: Option<crate::settings_registry::Effective>,
    channels: Option<crate::settings_registry::Effective>,
    sensitivity: Option<crate::settings_registry::Effective>,
}
type Certified<T> = (T, mw_store::authority_publication::mirror::MirrorGen);
const PREFS_MAX: usize = 20_000;
type PrefsCache = Mutex<HashMap<(uuid::Uuid, String), Certified<std::sync::Arc<PrefInputs>>>>;
fn prefs_cache() -> &'static PrefsCache {
    static C: OnceLock<PrefsCache> = OnceLock::new();
    C.get_or_init(Default::default)
}
fn geo_cache() -> &'static Mutex<HashMap<uuid::Uuid, Certified<bool>>> {
    static C: OnceLock<Mutex<HashMap<uuid::Uuid, Certified<bool>>>> = OnceLock::new();
    C.get_or_init(Default::default)
}

/// The mirror, when this request may reuse settings rows: bounded mode on
/// SQLite AND the request runs on published inputs (whose cold validation
/// requires the settings-revision triggers).
async fn reuse_mirror(state: &AppState) -> Option<std::sync::Arc<mw_store::authority_publication::mirror::EpochMirror>> {
    let mirror = crate::request_inputs::snapshot::mirror(state)?;
    crate::published_guard_context::current().await?;
    Some(mirror)
}

/// [`alerts::user_prefs`] for the watch hot path.
async fn alert_prefs(state: &AppState, account: &str) -> alerts::UserPrefs {
    let Some(mirror) = reuse_mirror(state).await else {
        #[cfg(test)]
        test_control::record(|c| c.prefs_fresh += 1);
        return alerts::user_prefs(state, account).await;
    };
    // The administrator switches: the same process-wide reader user_prefs uses.
    let globals = sec_cfg::values(&state.store, "security.alerts.").await;
    // The settings subject, fresh: no trigger covers it.
    let Ok(address) = state.store.security_account_address(account).await else {
        #[cfg(test)]
        test_control::record(|c| c.prefs_fresh += 1);
        return alerts::user_prefs(state, account).await;
    };
    let address = address.unwrap_or_default();
    let key = (state.store.cache_scope(), account.to_owned());
    let cached = prefs_cache()
        .lock()
        .ok()
        .and_then(|cache| cache.get(&key).cloned());
    if let Some((inputs, built)) = cached
        && inputs.address == address
        && mirror.globals_unchanged(built)
    {
        #[cfg(test)]
        test_control::record(|c| c.prefs_reused += 1);
        return fold_prefs(&globals, &inputs);
    }
    #[cfg(test)]
    test_control::record(|c| c.prefs_fresh += 1);
    // Strictly BEFORE the settings reads (EpochMirror correctness note).
    let built = mirror.stamp_gen();
    let (inputs, complete) = pref_rows(state, account, address).await;
    let inputs = std::sync::Arc::new(inputs);
    if let (Some(built), true, Ok(mut cache)) = (built, complete, prefs_cache().lock()) {
        if cache.len() >= PREFS_MAX && !cache.contains_key(&key) {
            cache.clear();
        }
        cache.insert(key, (inputs.clone(), built));
    }
    fold_prefs(&globals, &inputs)
}

/// The per-account reads of [`alerts::user_prefs`], in its order. `complete` is
/// false when any read failed: such a result is used once, never cached.
#[cfg(test)]
async fn pref_inputs(state: &AppState, account: &str) -> (PrefInputs, bool) {
    match state.store.security_account_address(account).await {
        Ok(address) => pref_rows(state, account, address.unwrap_or_default()).await,
        Err(_) => (pref_rows(state, account, String::new()).await.0, false),
    }
}

/// The three settings lookups of [`alerts::user_prefs`] for `address`.
async fn pref_rows(state: &AppState, account: &str, address: String) -> (PrefInputs, bool) {
    let mut complete = true;
    let subject = mw_store::settings_policy::SettingSubject {
        personal_owner: None,
        domain: address.rsplit_once('@').map_or("", |(_, d)| d),
        account_id: account,
        address: &address,
    };
    let mut values = [None, None, None];
    for (slot, key) in values.iter_mut().zip([
        "security.alerts.countryChange",
        "security.alerts.channels",
        "security.alerts.sensitivity",
    ]) {
        match crate::settings_registry::effective_for(&state.store, key, &subject).await {
            Ok(value) => *slot = value,
            Err(_) => complete = false,
        }
    }
    let [country_change, channels, sensitivity] = values;
    let inputs = PrefInputs {
        address,
        country_change,
        channels,
        sensitivity,
    };
    (inputs, complete)
}

/// Must stay identical to the folding in [`alerts::user_prefs`]; the test
/// `cached_preferences_fold_exactly_like_user_prefs` compares the two.
fn fold_prefs(globals: &sec_cfg::Values, inputs: &PrefInputs) -> alerts::UserPrefs {
    use mw_store::settings_policy::SettingSource;
    let mut prefs = alerts::UserPrefs {
        email_allowed: globals.flag("security.alerts.emailEnabled"),
        ..alerts::UserPrefs::default()
    };
    if let Some(e) = &inputs.country_change {
        let value = e.value.as_bool().unwrap_or(true);
        prefs.country_alert = value
            || !(matches!(e.source, SettingSource::User)
                && !globals.flag("security.alerts.countryChangeUserMute"));
    }
    if let Some(e) = &inputs.channels {
        prefs.channels = match e.value.as_str() {
            Some("app") => alerts::Channels::App,
            Some("email") => alerts::Channels::Email,
            _ => alerts::Channels::Both,
        };
    }
    if let Some(e) = &inputs.sensitivity {
        prefs.sensitivity = theft::Sensitivity::parse(e.value.as_str().unwrap_or("normal"));
    }
    prefs
}

/// [`crate::geo_lookup::session_alerts_on_failure`] for the watch hot path
/// (read on every request whose country cannot be told, e.g. no GeoIP database).
async fn geo_failure_alerts(state: &AppState) -> bool {
    let Some(mirror) = reuse_mirror(state).await else {
        return crate::geo_lookup::session_alerts_on_failure(state).await;
    };
    let key = state.store.cache_scope();
    let cached = geo_cache().lock().ok().and_then(|cache| cache.get(&key).copied());
    if let Some((value, built)) = cached
        && mirror.globals_unchanged(built)
    {
        #[cfg(test)]
        test_control::record(|c| c.geo_reused += 1);
        return value;
    }
    let built = mirror.stamp_gen();
    // Same reader, same key; an error is `false` there and is not cached here.
    let read = crate::settings_registry::effective_for(
        &state.store,
        crate::geo_lookup::SESSION_KEY,
        &Default::default(),
    )
    .await;
    let value = read
        .as_ref()
        .ok()
        .and_then(Option::as_ref)
        .and_then(|e| e.value.as_str().map(|v| v == "alert"))
        .unwrap_or(false);
    if let (Some(built), true, Ok(mut cache)) = (built, read.is_ok(), geo_cache().lock()) {
        cache.insert(key, (value, built));
    }
    value
}

// ─── realtime streams: cheap "still valid as of the current epochs" ─────────
//
// For the WebSocket push loop (`push.rs` `recheck`, every
// `MW_REALTIME_RECHECK_SECS`). Protocol, per open stream (three lines):
//   1. Full path: `let (ok, cert) = realtime_certify_with(&state, &session, bearer, || recheck(..)).await;`
//   2. Each later period: `realtime_check(&state, &cert?, period)`: `Current` keeps the stream (0 SQL).
//   3. `Revalidate`, no certificate or a failed recheck: back to step 1 (or close the stream, as today).
//
// Soundness: the generation is stamped before the recheck's first read and the
// certificate's fence is read after its last one, both inside
// `realtime_certify_with`; the fence names every scope those reads depend on
// (session, account, principal, the published entry's domains and subjects, the
// global kinds and the settings revision). `still_valid(fence, g)` fails once
// any of them changed after `g` (within the bound across processes, at once
// after a local revocation through `expedite`).
//
// Why a late stamp cannot be expressed: there is no public stamp. The only way
// to obtain a `RealtimeCertificate` is `realtime_certify_with`, which stamps,
// then calls the caller's `recheck` closure (the recheck future does not exist
// before the stamp, and a future does no work until awaited), then reads the
// fence. The certificate's fields are private, it is neither `Clone` nor
// constructible elsewhere, and the generation inside it never leaves it. The
// one misuse left is a closure that only returns a verdict computed earlier;
// the closure must BE the recheck (perform its reads when awaited).

/// See the section comment. Opaque; holds no credential.
pub(crate) struct RealtimeCertificate {
    session_id: String,
    fence: std::sync::Arc<mw_store::authority_publication::NativeAuthorityFence>,
    built: mw_store::authority_publication::mirror::MirrorGen,
    idle_secs: u64,
    absolute_secs: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RealtimeCheck {
    /// Nothing the full recheck depends on changed: keep the stream.
    Current,
    /// Run the full recheck.
    Revalidate,
}

/// Runs the caller's full session recheck between a mirror stamp (0 SQL, taken
/// first) and the certificate's fence read (one statement plus the clock-policy
/// resolution: revision read and memo). The recheck's verdict is returned as
/// is; a failed recheck yields no certificate and no fence read. The
/// certificate is `None` (re-validate fully every period, today's path) for:
/// mode off, PostgreSQL, a mirror that is not fresh, bearer streams, sessions
/// outside the engine primary fence (linked, proxy), no published request-input
/// entry for the session, an entry or account that no longer matches the
/// fence, or any read error.
#[allow(dead_code)] // wired into push.rs by a later t34 task
pub(crate) async fn realtime_certify_with<T, E, F>(
    state: &AppState,
    session: &mw_store::Session,
    bearer: bool,
    recheck: impl FnOnce() -> F,
) -> Result<(T, Option<RealtimeCertificate>), E>
where
    F: std::future::Future<Output = Result<T, E>>,
{
    // native_sessions (bearer) has no authority trigger: never certified.
    // Strictly BEFORE the recheck's first read (EpochMirror correctness note).
    let stamped = (!bearer)
        .then(|| crate::request_inputs::snapshot::mirror(state).and_then(|mirror| mirror.stamp_gen()))
        .flatten();
    let verdict = recheck().await?;
    let certificate = match stamped {
        Some(stamped) => certify_after_recheck(state, session, stamped).await,
        None => None,
    };
    Ok((verdict, certificate))
}

/// The fence read after a successful recheck that started after `stamped`.
async fn certify_after_recheck(
    state: &AppState,
    session: &mw_store::Session,
    stamped: mw_store::authority_publication::mirror::MirrorGen,
) -> Option<RealtimeCertificate> {
    crate::request_inputs::snapshot::mirror(state)?;
    // The same entry an ordinary request of this session would use.
    let process = crate::request_inputs::process_environment()?;
    let entry = state
        .native_publication
        .as_ref()?
        .request_inputs_by_fingerprint(&session.id, &process.fingerprint)?;
    let fence = state
        .store
        .native_publication_fence(&session.id, &entry.domains, &entry.subjects)
        .await
        .ok()??;
    // The entry's domain/subject lists are current only if it was built at the
    // versions this fence reads; otherwise its scope list may miss a domain.
    if fence.account_id() != session.account_id || !entry.matches(&fence) {
        return None;
    }
    let subject = mw_store::settings_policy::SettingSubject {
        personal_owner: None,
        domain: session.username.rsplit_once('@').map_or("", |(_, d)| d),
        account_id: &session.account_id,
        address: &session.username,
    };
    let revision = state.store.settings_read_revision().await.ok()?;
    let values = crate::settings_registry::effective_all_at_revision(
        &state.store,
        &subject,
        |def| matches!(def.key, "session.idleSecs" | "session.absoluteSecs"),
        &revision,
    )
    .await
    .ok()?;
    let policy = crate::security_center::clock_policy_from_values(
        &values,
        state.hardening.idle_timeout.as_secs(),
        state.hardening.absolute_timeout.as_secs(),
    );
    Some(RealtimeCertificate {
        session_id: session.id.clone(),
        fence: std::sync::Arc::new(fence),
        built: stamped,
        idle_secs: policy.idle_secs,
        absolute_secs: policy.absolute_secs,
    })
}

/// Zero statements. `Current` only when the mirror is live and fresh, no
/// expedite is pending, no scope of the certificate's fence, global kind or the
/// settings revision changed since the stamp, the process session map does not
/// expire it, and the session deadline computed from the certificate's clocks
/// (which can only be older than the row) does not pass before the next check
/// plus the snapshot clock margin. Anything else: `Revalidate`.
#[allow(dead_code)] // wired into push.rs by a later t34 task
pub(crate) fn realtime_check(
    state: &AppState,
    cert: &RealtimeCertificate,
    period: std::time::Duration,
) -> RealtimeCheck {
    let Some(mirror) = crate::request_inputs::snapshot::mirror(state) else {
        return RealtimeCheck::Revalidate;
    };
    if state
        .sessions
        .expired(&cert.session_id, &state.hardening)
        .is_some()
        || !mirror.still_valid(&cert.fence, cert.built)
    {
        return RealtimeCheck::Revalidate;
    }
    let ahead = chrono::TimeDelta::from_std(period)
        .ok()
        .and_then(|period| period.checked_add(&crate::request_inputs::snapshot::CLOCK_MARGIN));
    let (created, activity) = cert.fence.clocks();
    let clear = ahead
        .and_then(|ahead| chrono::Utc::now().checked_add_signed(ahead))
        .is_some_and(|later| {
            !crate::security_center::session_deadline_passed(
                created,
                activity,
                later,
                cert.idle_secs,
                cert.absolute_secs,
            )
        });
    if clear {
        RealtimeCheck::Current
    } else {
        RealtimeCheck::Revalidate
    }
}

fn state_key(hash: &str) -> String {
    format!("{STATE_PREFIX}{hash}")
}

async fn load(state: &AppState, hash: &str) -> Option<SessionSec> {
    if let Some(c) = cache_lock().get(hash) {
        return Some(c.sec.clone());
    }
    let raw = state
        .store
        .get_setting(&state_key(hash))
        .await
        .ok()
        .flatten()?;
    let sec: SessionSec = serde_json::from_str(&raw).ok()?;
    remember(hash, &sec, now());
    Some(sec)
}
fn remember(hash: &str, sec: &SessionSec, saved_at: i64) {
    let mut c = cache_lock();
    if c.len() >= CACHE_MAX && !c.contains_key(hash) {
        let mut ages: Vec<_> = c.iter().map(|(k, v)| (v.sec.last_ts, k.clone())).collect();
        ages.sort_unstable();
        for (_, key) in ages.into_iter().take(CACHE_MAX / 4) {
            c.remove(&key);
        }
    }
    c.insert(
        hash.to_owned(),
        Cached {
            sec: sec.clone(),
            saved_at,
        },
    );
}
async fn save(state: &AppState, hash: &str, sec: &SessionSec) {
    if let Ok(raw) = serde_json::to_string(sec)
        && state
            .store
            .set_setting(&state_key(hash), &raw)
            .await
            .is_ok()
    {
        remember(hash, sec, now());
    }
}
async fn forget(state: &AppState, hash: &str) {
    cache_lock().remove(hash);
    let _ = state.store.delete_sec_state(&state_key(hash)).await;
}

fn unauthorized_revoked() -> Response {
    let mut r = (
        StatusCode::UNAUTHORIZED,
        axum::Json(json!({"error":"session revoked","code":"sessionRevoked"})),
    )
        .into_response();
    r.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        "no-store".parse().unwrap(),
    );
    r
}

fn network_of(ip: Option<std::net::IpAddr>) -> String {
    ip.map_or_else(|| "?".into(), session_binding::coarse_network)
}

async fn revoke_session(state: &AppState, admin: bool, token: &str) {
    let hash = crate::push_relay::hash_token(token);
    if admin {
        let _ = state.store.delete_admin_session(&hash).await;
    } else {
        let _ = state.store.delete_session(token).await;
        let _ = state.store.delete_native_session(&hash).await;
        state.sessions.forget(token);
        state.step_up.forget_user(token);
    }
    forget(state, &hash).await;
    // t34: a theft-watch revocation can happen on any request, GET and JMAP
    // included, which the request layer does not expedite. No cached authority
    // fence may validate again until a poll that started after this delete.
    crate::request_inputs::snapshot::expedite(state);
}

/// Revoke every live session of `account` whose record belongs to `family`.
async fn revoke_family(state: &AppState, account: &str, family: &str) -> usize {
    let Ok(sessions) = state.store.sessions_by_account(account).await else {
        return 0;
    };
    let mut revoked = 0;
    for s in sessions {
        let hash = crate::push_relay::hash_token(&s.id);
        if load(state, &hash).await.is_some_and(|r| r.family == family) {
            revoke_session(state, false, &s.id).await;
            revoked += 1;
        }
    }
    revoked
}

/// Revoke every live admin session of `username` whose record belongs to `family`.
/// A rotated admin id and its successors share one family, so a replayed retired
/// id ends the whole chain.
pub(crate) async fn revoke_admin_family(state: &AppState, username: &str, family: &str) -> usize {
    let Ok(sessions) = state.store.list_admin_sessions(0).await else {
        return 0;
    };
    let mut revoked = 0;
    for s in sessions.into_iter().filter(|s| s.admin_id == username) {
        if load(state, &s.token_hash)
            .await
            .is_some_and(|r| r.family == family)
        {
            let _ = state.store.delete_admin_session(&s.token_hash).await;
            forget(state, &s.token_hash).await;
            revoked += 1;
        }
    }
    if revoked > 0 {
        // Admin sessions are never served from the guard snapshot; expedite anyway
        // so every revocation site in this module behaves the same.
        crate::request_inputs::snapshot::expedite(state);
    }
    revoked
}

async fn inspect(
    state: &AppState,
    target: &Target,
    headers: &HeaderMap,
    ext: &axum::http::Extensions,
    path: &str,
) -> Option<Response> {
    // t50hp-e9: the configuration feeds decisions first; the source is
    // observed once, right before this guard's first effect (record write,
    // alert, revocation, step-up flag, sweep): see `watch_effect`.
    if let Err(response) = crate::request_inputs::decision_boundary(&state.store, crate::published_guard_context::Guard::Watch).await { return Some(response); }
    let mut observed = false;
    let cfg = config(state).await;
    let want_country_alert = cfg.rules.country_alert;
    if !cfg.binding.enabled && !cfg.rules.theft && !want_country_alert {
        return None;
    }
    let t = now();
    let hash = crate::push_relay::hash_token(&target.token);
    let ip = crate::scope_mw::proxy::client_ip(headers, ext);
    let country = ip.and_then(mw_engine::security::country_for_ip);
    let net = network_of(ip);
    let client = session_binding::parse_client(headers);
    let secret = state.store.signin_challenge_secret("session-binding");
    let fp = session_binding::derive(&secret, &client, country.as_deref(), ip, cfg.binding.attrs);

    let Some(mut sec) = load(state, &hash).await else {
        if let Err(response) = watch_effect(state, &mut observed).await { return Some(response); }
        return create_or_reuse(
            state, target, &cfg, &hash, fp, country, &net, t, headers, ext,
        )
        .await;
    };

    // Compare against the stored fingerprint.
    let verdict = if cfg.binding.enabled {
        session_binding::compare(&sec.fp, &fp, &client, cfg.binding.strict)
    } else {
        Verdict::Match
    };
    if verdict == Verdict::Match {
        session_binding::merge_on_match(&mut sec.fp, &fp);
    }
    let admin = target.admin;
    let bind_action = cfg.binding.action(admin);
    let theft_level = if admin {
        cfg.admin_action
    } else {
        cfg.webmail_action
    };

    // Preferences are read only when something could be reported.
    let needs_prefs =
        !matches!(verdict, Verdict::Match) || sec.location_event(country.as_deref(), &net);
    let prefs = if !needs_prefs {
        alerts::UserPrefs::default()
    } else if admin {
        alerts::UserPrefs {
            channels: alerts::Channels::App,
            ..Default::default()
        }
    } else {
        alert_prefs(state, &sec.account).await
    };
    let findings = sec.observe(
        t,
        country.as_deref(),
        &net,
        &cfg.rules,
        prefs.sensitivity,
        prefs.country_alert,
    );

    // level: 0 alert-only, 1 step-up, 2 revoke
    let mut level = 0u8;
    let mut raised: Vec<(Finding, u8)> = Vec::new();
    let mut mismatch: Option<Vec<&'static str>> = None;
    for f in findings {
        let l = match f {
            Finding::CountryChange { .. } => 0,
            _ => theft_level,
        };
        raised.push((f, l));
    }
    match &verdict {
        Verdict::Mismatch(why) => {
            let l = match bind_action {
                BindAction::Off | BindAction::Detect => 0,
                BindAction::StepUp => 1,
                BindAction::Revoke => 2,
            };
            if bind_action != BindAction::Off {
                mismatch = Some(why.clone());
                level = level.max(l);
            }
        }
        Verdict::Missing => {
            // A request with no browser identity is recorded, never acted on.
            if cfg.binding.enabled
                && bind_action != BindAction::Off
                && sec.debounce("clientMissing", t, KIND_DEBOUNCE_SECS)
            {
                tracing::info!(
                    "session {}: request without a User-Agent",
                    mw_store::session_handle(&target.token)
                );
            }
        }
        Verdict::Match => {}
    }
    for (_, l) in &raised {
        level = level.max(*l);
    }
    // A low-sensitivity user is only ever warned about location findings.
    if !admin && prefs.sensitivity == theft::Sensitivity::Low {
        level = 0;
    }
    let action_name = match level {
        2 => "revoked",
        1 => "step-up",
        _ => "alerted",
    };

    let short = mw_store::session_handle(&target.token);
    let mut events: Vec<alerts::Alert> = Vec::new();
    for (f, _) in &raised {
        let (kind, reason, from, to) = match f {
            Finding::CountryChange { from, to } => (
                "countryChange",
                "country-changed".to_owned(),
                from.clone(),
                Some(to.clone()),
            ),
            Finding::ImpossibleTravel { from, to, secs } => (
                "impossibleTravel",
                format!("country-changed-in-{secs}s"),
                Some(from.clone()),
                Some(to.clone()),
            ),
            Finding::Concurrent {
                from,
                to,
                cross_country,
            } => (
                "concurrentUse",
                if *cross_country {
                    "alternating-countries"
                } else {
                    "alternating-networks"
                }
                .to_owned(),
                Some(from.clone()),
                Some(to.clone()),
            ),
        };
        if kind != "countryChange" && !sec.debounce(kind, t, KIND_DEBOUNCE_SECS) {
            continue;
        }
        events.push(alerts::Alert {
            kind,
            reason,
            from,
            to,
            action: if kind == "countryChange" {
                "alerted"
            } else {
                action_name
            },
            session: short.clone(),
            ts: t,
            admin,
            account: sec.account.clone(),
            context: Some(alerts::evidence::request(state, &target.token, headers, ext).await),
        });
    }
    if let Some(why) = mismatch
        && sec.debounce("fingerprintMismatch", t, KIND_DEBOUNCE_SECS)
    {
        events.push(alerts::Alert {
            kind: "fingerprintMismatch",
            reason: why.join(","),
            from: sec.last_country.clone(),
            to: country.clone(),
            action: if bind_action == BindAction::Detect {
                "alerted"
            } else {
                action_name
            },
            session: short.clone(),
            ts: t,
            admin,
            account: sec.account.clone(),
            context: Some(alerts::evidence::request(state, &target.token, headers, ext).await),
        });
    }
    // A request whose country cannot be told (and is not a local network): with
    // `security.geo.sessionLookupFailure = alert`, tell the holder once a day per
    // session which cause it was. Never a block or a revoke.
    if country.is_none()
        && !admin
        && cfg.rules.country_alert
        && matches!(
            crate::geo_lookup::classify(ip),
            crate::geo_lookup::Outcome::NoDatabase | crate::geo_lookup::Outcome::NotFound
        )
        && geo_failure_alerts(state).await
        && sec.debounce("countryUnavailable", t, 86_400)
    {
        if let Err(response) = watch_effect(state, &mut observed).await { return Some(response); }
        let user = alerts::user_prefs(state, &sec.account).await;
        alerts::raise(
            state,
            alerts::Alert {
                kind: "countryUnavailable",
                reason: crate::geo_lookup::classify(ip).cause().to_owned(),
                from: sec.last_country.clone(),
                to: None,
                action: "alerted",
                session: short.clone(),
                ts: t,
                admin,
                account: sec.account.clone(),
                context: Some(alerts::evidence::request(state, &target.token, headers, ext).await),
            },
            &user,
        )
        .await;
    }
    if (!events.is_empty() || level > 0)
        && let Err(response) = watch_effect(state, &mut observed).await
    { return Some(response); }
    for e in events {
        alerts::raise(state, e, &prefs).await;
    }

    if level == 2 {
        revoke_session(state, admin, &target.token).await;
        return Some(unauthorized_revoked());
    }
    if level == 1 && sec.flagged.is_none() {
        sec.flagged = Some(theft::Flag {
            reason: "session-security".into(),
            ts: t,
        });
        // The step-up flag is enforced below from this module's own state, which
        // the guard snapshot never caches. Expedite anyway: a step-up is a
        // security transition and costs one poll.
        crate::request_inputs::snapshot::expedite(state);
    }

    // A flagged session needs a live step-up proof for anything but sign-out.
    let mut gate: Option<Response> = None;
    if sec.flagged.is_some() && !flagged_exempt(path) {
        gate = step_up_gate(state, target, headers).await;
        if gate.is_none() {
            sec.flagged = None;
        }
    }

    let changed = cache_lock().get(&hash).is_none_or(|c| {
        t - c.saved_at >= SAVE_EVERY_SECS
            || c.sec.flagged != sec.flagged
            || c.sec.alerted != sec.alerted
            || c.sec.last_country != sec.last_country
            || c.sec.noted != sec.noted
            || c.sec.fp != sec.fp
    });
    if changed {
        if let Err(response) = watch_effect(state, &mut observed).await { return Some(response); }
        save(state, &hash, &sec).await;
    } else {
        // Keep the in-memory history current without a write.
        let saved_at = cache_lock().get(&hash).map_or(t, |c| c.saved_at);
        remember(&hash, &sec, saved_at);
    }
    if sweep_due() {
        if let Err(response) = watch_effect(state, &mut observed).await { return Some(response); }
        sweep_if_due(state, cfg.rules.retention_days);
    }
    gate
}

/// t50hp-e9: one source observation before the first effect of [`inspect`]
/// (the decisions before it used the request's last observation; see
/// `request_inputs::decision_boundary`). Later effects of the same call
/// follow it without another wait on the source.
async fn watch_effect(state: &AppState, observed: &mut bool) -> Result<(), Response> {
    if !*observed {
        crate::request_inputs::effect_boundary(&state.store, crate::published_guard_context::Guard::Watch).await?;
        *observed = true;
    }
    Ok(())
}

/// `None` when the session holds a live step-up proof, else the 403 to send.
async fn step_up_gate(state: &AppState, target: &Target, headers: &HeaderMap) -> Option<Response> {
    use crate::step_up::{StepUpAction, require_admin, require_user};
    if target.admin {
        let ctx = match crate::admin_ctx::AdminCtx::resolve(state, headers).await {
            Ok(ctx) => ctx,
            Err(_) => return None, // not a live admin session: the route will say 401
        };
        return require_admin(state, &ctx, StepUpAction::SessionVerification)
            .await
            .err()
            .map(IntoResponse::into_response);
    }
    let session = match crate::request_inputs::stored_session(
        &state.store, &target.token, crate::published_guard_context::Guard::Watch,
    ).await {
        Err(response) => return Some(response),
        Ok(Some(session)) => session,
        Ok(None) => {
            #[cfg(test)] crate::request_inputs::test_control::record(|counts| counts.legacy_guard_session_reads += 1);
            match state.store.get_primary_session(&target.token).await {
                Ok(session) => session,
                Err(_) => return None,
            }
        }
    };
    require_user(state, &session, StepUpAction::SessionVerification)
        .await
        .err()
}

#[allow(clippy::too_many_arguments)]
async fn create_or_reuse(
    state: &AppState,
    target: &Target,
    cfg: &Config,
    hash: &str,
    fp: session_binding::Fingerprint,
    country: Option<String>,
    net: &str,
    t: i64,
    headers: &HeaderMap,
    ext: &axum::http::Extensions,
) -> Option<Response> {
    let account = if target.admin {
        state.store.get_admin_session(hash).await.ok().flatten()
    } else {
        match crate::request_inputs::stored_session(
            &state.store, &target.token, crate::published_guard_context::Guard::Watch,
        ).await {
            Err(response) => return Some(response),
            Ok(Some(session)) => Some(session.account_id),
            Ok(None) => {
                #[cfg(test)] crate::request_inputs::test_control::record(|counts| counts.legacy_guard_session_reads += 1);
                state.store.get_primary_session(&target.token).await.ok().map(|session| session.account_id)
            }
        }
    };
    let Some(account) = account else {
        return reuse_check(state, cfg, hash, &target.token, t, headers, ext).await;
    };
    let sec = SessionSec::new(
        if target.admin { "admin" } else { "webmail" },
        &account,
        uuid::Uuid::new_v4().to_string(),
        fp,
        country,
        net,
        t,
    );
    save(state, hash, &sec).await;
    None
}

/// A cookie that names no live session: is it a retired (rotated) id?
#[allow(clippy::too_many_arguments)]
async fn reuse_check(
    state: &AppState,
    cfg: &Config,
    hash: &str,
    token: &str,
    t: i64,
    headers: &HeaderMap,
    ext: &axum::http::Extensions,
) -> Option<Response> {
    if !cfg.rules.theft {
        return None;
    }
    let key = format!("{TOMB_PREFIX}{hash}");
    let mut tomb: Tombstone =
        serde_json::from_str(&state.store.get_setting(&key).await.ok().flatten()?).ok()?;
    match theft::classify_reuse(&tomb, t, cfg.rules.grace_secs) {
        Reuse::InFlight | Reuse::Reported => None,
        Reuse::Theft => {
            tomb.reported = true;
            if let Ok(raw) = serde_json::to_string(&tomb) {
                let _ = state.store.set_setting(&key, &raw).await;
            }
            let admin = tomb.surface == "admin";
            let mut context = alerts::evidence::request(state, token, headers, ext).await;
            context.subject = crate::notification_center::account_owner(state, &tomb.account).await;
            let revoked = if admin {
                revoke_admin_family(state, &tomb.account, &tomb.family).await
            } else {
                revoke_family(state, &tomb.account, &tomb.family).await
            };
            let prefs = if admin {
                alerts::UserPrefs {
                    channels: alerts::Channels::App,
                    ..Default::default()
                }
            } else {
                alerts::user_prefs(state, &tomb.account).await
            };
            alerts::raise(
                state,
                alerts::Alert {
                    kind: "rotationReuse",
                    reason: format!("retired-session-id-presented;sessions-ended={revoked}"),
                    from: None,
                    to: None,
                    action: "revoked",
                    session: mw_store::session_handle(token),
                    ts: t,
                    admin,
                    account: tomb.account,
                    context: Some(context),
                },
                &prefs,
            )
            .await;
            Some(unauthorized_revoked())
        }
    }
}

/// `/api/session/rotate` succeeded: carry the record to the new id and leave a
/// marker for the old one.
pub(crate) async fn on_rotate(state: &AppState, old_id: &str, new_id: &str) {
    let old_hash = crate::push_relay::hash_token(old_id);
    let new_hash = crate::push_relay::hash_token(new_id);
    let t = now();
    let Some(mut sec) = load(state, &old_hash).await else {
        return;
    };
    let tomb = Tombstone {
        family: sec.family.clone(),
        account: sec.account.clone(),
        surface: sec.surface.clone(),
        rotated_at: t,
        reported: false,
    };
    if let Ok(raw) = serde_json::to_string(&tomb) {
        let _ = state
            .store
            .set_setting(&format!("{TOMB_PREFIX}{old_hash}"), &raw)
            .await;
    }
    sec.last_ts = t;
    save(state, &new_hash, &sec).await;
    forget(state, &old_hash).await;
}

// ─── retention ──────────────────────────────────────────────────────────────

static SWEEP_LAST: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
/// Whether [`sweep_if_due`] would start a sweep now (no claim is taken).
fn sweep_due() -> bool {
    now() - SWEEP_LAST.load(std::sync::atomic::Ordering::Relaxed) >= SWEEP_EVERY_SECS
}
fn sweep_if_due(state: &AppState, retention_days: i64) {
    use std::sync::atomic::Ordering;
    let t = now();
    let last = SWEEP_LAST.load(Ordering::Relaxed);
    if t - last < SWEEP_EVERY_SECS
        || SWEEP_LAST
            .compare_exchange(last, t, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
    {
        return;
    }
    let state = state.clone();
    tokio::spawn(async move {
        sweep(&state, t, retention_days).await;
    });
}

/// Delete session records and rotation markers not touched for `retention_days`.
pub(crate) async fn sweep(state: &AppState, t: i64, retention_days: i64) -> usize {
    let floor = t - retention_days * 86_400;
    let mut removed = 0;
    if let Ok(rows) = state.store.settings_with_prefix(STATE_PREFIX).await {
        for (key, value) in rows {
            let last = serde_json::from_str::<SessionSec>(&value).map_or(0, |s| s.last_ts);
            if last < floor && state.store.delete_sec_state(&key).await.unwrap_or(false) {
                cache_lock().remove(key.trim_start_matches(STATE_PREFIX));
                removed += 1;
            }
        }
    }
    if let Ok(rows) = state.store.settings_with_prefix(TOMB_PREFIX).await {
        for (key, value) in rows {
            let at = serde_json::from_str::<Tombstone>(&value).map_or(0, |s| s.rotated_at);
            if at < floor && state.store.delete_sec_state(&key).await.unwrap_or(false) {
                removed += 1;
            }
        }
    }
    removed
}

/// Test-only measurement of the watch middleware: statements issued while it
/// runs, how its settings reads were served, and the AppState it ran with.
/// Inactive (no nested measurement) unless a test opened [`measure`].
#[cfg(test)]
pub(crate) mod test_control {
    use crate::AppState;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
    pub(crate) struct Counts {
        pub inspections: u64,
        pub reads: u64,
        pub writes: u64,
        pub other: u64,
        pub prefs_fresh: u64,
        pub prefs_reused: u64,
        pub geo_reused: u64,
    }
    tokio::task_local! {
        static COUNTS: std::cell::RefCell<Counts>;
        static STATE: std::cell::RefCell<Option<AppState>>;
    }
    pub(crate) fn record(update: impl FnOnce(&mut Counts)) {
        let _ = COUNTS.try_with(|counts| update(&mut counts.borrow_mut()));
    }
    pub(super) async fn observe<F>(state: &AppState, inspect: F) -> F::Output
    where
        F: std::future::Future,
    {
        if COUNTS.try_with(|_| ()).is_err() {
            return inspect.await;
        }
        let _ = STATE.try_with(|slot| *slot.borrow_mut() = Some(state.clone()));
        let (output, sql) = mw_store::pool_telemetry::measure_queries(Box::pin(inspect)).await;
        record(|c| {
            c.inspections += 1;
            c.reads += sql.read.calls;
            c.writes += sql.write.calls;
            c.other += sql.other.calls;
        });
        output
    }
    pub(crate) async fn measure<F: std::future::Future>(future: F) -> (F::Output, Counts, Option<AppState>) {
        COUNTS
            .scope(std::cell::RefCell::new(Counts::default()), async {
                STATE
                    .scope(std::cell::RefCell::new(None), async {
                        let output = future.await;
                        let counts = COUNTS.with(|c| *c.borrow());
                        (output, counts, STATE.with(|s| s.borrow().clone()))
                    })
                    .await
            })
            .await
    }
}

#[cfg(test)]
#[path = "session_watch_tests.rs"]
mod snapshot_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flagged_sessions_may_still_sign_out_and_confirm() {
        for p in [
            "/api/logout",
            "/api/security/step-up",
            "/api/security/step-up/passkey",
            "/admin/step-up",
            "/admin/logout",
            "/api/account/security/session-alerts",
        ] {
            assert!(flagged_exempt(p), "{p}");
        }
        for p in ["/api/me", "/jmap/api", "/admin/settings"] {
            assert!(!flagged_exempt(p), "{p}");
        }
    }

    fn req(path: &str, headers: &[(&str, &str)]) -> Request {
        let mut b = axum::http::Request::builder().uri(path);
        for (k, v) in headers {
            b = b.header(*k, *v);
        }
        b.body(axum::body::Body::empty()).unwrap()
    }

    #[test]
    fn bearer_api_key_dav_and_signin_routes_are_exempt() {
        let cookie = ("cookie", "mw_session=abc");
        assert!(classify(&req("/api/me", &[cookie])).is_some());
        assert!(classify(&req("/jmap/api", &[cookie])).is_some());
        assert!(classify(&req("/api/me", &[cookie, ("authorization", "Bearer x")])).is_none());
        assert!(classify(&req("/api/me", &[cookie, ("x-api-key", "k")])).is_none());
        for p in [
            "/dav/calendars",
            "/oauth/token",
            "/mcp",
            "/api/login",
            "/api/login/2fa",
            "/api/logout",
            "/api/public-config",
            "/healthz",
        ] {
            assert!(classify(&req(p, &[cookie])).is_none(), "{p}");
        }
        assert!(
            classify(&req("/api/me", &[])).is_none(),
            "no cookie, nothing to watch"
        );
    }
}
