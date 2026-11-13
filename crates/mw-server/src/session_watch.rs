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
    match inspect(
        &state,
        &target,
        request.headers(),
        request.extensions(),
        request.uri().path(),
    )
    .await
    {
        Some(response) => response,
        None => next.run(request).await,
    }
}

struct Config {
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
async fn config(state: &AppState) -> Config {
    let b = sec_cfg::values(&state.store, session_binding::PREFIX).await;
    let t = sec_cfg::values(&state.store, "security.theft.").await;
    let a = sec_cfg::values(&state.store, "security.alerts.").await;
    Config {
        binding: session_binding::Policy::from_values(&b),
        rules: theft::Rules::from_values(&t, &a),
        webmail_action: theft_action(&t.text("security.theft.webmailAction")),
        admin_action: theft_action(&t.text("security.theft.adminAction")),
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
    revoked
}

async fn inspect(
    state: &AppState,
    target: &Target,
    headers: &HeaderMap,
    ext: &axum::http::Extensions,
    path: &str,
) -> Option<Response> {
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
        alerts::user_prefs(state, &sec.account).await
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
        && crate::geo_lookup::session_alerts_on_failure(state).await
        && sec.debounce("countryUnavailable", t, 86_400)
    {
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
        save(state, &hash, &sec).await;
    } else {
        // Keep the in-memory history current without a write.
        let saved_at = cache_lock().get(&hash).map_or(t, |c| c.saved_at);
        remember(&hash, &sec, saved_at);
    }
    sweep_if_due(state, cfg.rules.retention_days);
    gate
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
    let session = match state.store.get_primary_session(&target.token).await {
        Ok(s) => s,
        Err(_) => return None,
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
        state
            .store
            .get_primary_session(&target.token)
            .await
            .ok()
            .map(|s| s.account_id)
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

fn sweep_if_due(state: &AppState, retention_days: i64) {
    use std::sync::atomic::{AtomicI64, Ordering};
    static LAST: AtomicI64 = AtomicI64::new(0);
    let t = now();
    let last = LAST.load(Ordering::Relaxed);
    if t - last < SWEEP_EVERY_SECS
        || LAST
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
