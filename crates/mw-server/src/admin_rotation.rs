//! Admin console session id rotation.
//!
//! A new admin session id is already minted at every sign-in (`start_session`).
//! This module adds the rest of the mailbox behaviour (`/api/session/rotate`):
//!
//! * every `admin.session.rotateSecs` (default 900, 0 = off) of use, and
//!   immediately after a successful step-up (`POST {admin}/step-up`, the
//!   privilege elevation), the id is replaced with a fresh one;
//! * the new row keeps the old row's creation time, idle deadline and absolute
//!   deadline, so rotation never extends a session;
//! * the old id is deleted, a tombstone is left (`session_watch::on_rotate`)
//!   and the step-up proof, factor fingerprint and pending authenticator
//!   enrolment follow the session;
//! * presenting the retired id after `security.theft.rotationGraceSecs` revokes
//!   the whole admin session family and raises an audit event and alert
//!   (`session_watch::reuse_check`).
//!
//! The new cookie has the same flags as the sign-in cookie (`HttpOnly`,
//! `SameSite=Strict`, the console `Path`, `Secure` by the global rule) with a
//! `Max-Age` of the time left to the absolute deadline, and the CSRF cookie is
//! reissued because it is derived from the session id.
use crate::AppState;
use axum::{
    extract::{Request, State},
    http::{HeaderValue, Method, header},
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

const TRACK_MAX: usize = 4096;
const MIN_INTERVAL: i64 = 60;

fn issued() -> std::sync::MutexGuard<'static, HashMap<String, i64>> {
    static M: OnceLock<Mutex<HashMap<String, i64>>> = OnceLock::new();
    M.get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

/// Has this session id been in use for `interval` seconds, as far as this
/// process knows? A session first seen (after a restart) counts from now.
fn age_exceeds(hash: &str, now: i64, interval: i64) -> bool {
    if interval <= 0 {
        return false;
    }
    let mut map = issued();
    if map.len() >= TRACK_MAX && !map.contains_key(hash) {
        let floor = now - 2 * 86_400;
        map.retain(|_, t| *t > floor);
        if map.len() >= TRACK_MAX {
            map.clear();
        }
    }
    let since = *map.entry(hash.to_owned()).or_insert(now);
    now - since >= interval.max(MIN_INTERVAL)
}

async fn interval(state: &AppState) -> i64 {
    crate::settings_registry::effective_for(
        &state.store,
        "admin.session.rotateSecs",
        &Default::default(),
    )
    .await
    .ok()
    .flatten()
    .and_then(|v| v.value.as_i64())
    .unwrap_or(900)
    .clamp(0, 86_400)
}

/// Replace `old_token` with a new session id. `None` when the old id is not a
/// live admin session. Returns the new token and the absolute deadline.
pub(crate) async fn rotate(state: &AppState, old_token: &str) -> Option<(String, i64)> {
    let new_token = crate::push_relay::hash_token(&format!(
        "{}:{}",
        mw_store::ServerKey::generate().to_hex(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    let old_hash = crate::push_relay::hash_token(old_token);
    let new_hash = crate::push_relay::hash_token(&new_token);
    let now = chrono::Utc::now().timestamp();
    let absolute = state
        .store
        .rotate_admin_session(&old_hash, &new_hash, now)
        .await
        .ok()
        .flatten()?;
    state.step_up.move_admin(&old_hash, &new_hash);
    crate::admin_accounts::move_enrollments(&old_hash, &new_hash);
    crate::session_watch::on_rotate(state, old_token, &new_token).await;
    let mut map = issued();
    map.remove(&old_hash);
    map.insert(new_hash, now);
    Some((new_token, absolute))
}

/// The `Set-Cookie` values for a rotated session: admin cookie first, then CSRF.
pub(crate) fn cookies(state: &AppState, token: &str, absolute: i64) -> Vec<HeaderValue> {
    let left = (absolute - chrono::Utc::now().timestamp()).max(1);
    let mut out = Vec::new();
    if let Ok(v) = crate::admin_accounts::set_admin_cookie(
        token,
        state.cookie_secure,
        left,
        &state.base_path,
    )
    .parse()
    {
        out.push(v);
    }
    out.push(crate::csrf_cookie(
        &state.store.session_csrf_token(token),
        state.cookie_secure,
    ));
    out
}

fn sets_admin_cookie(resp: &Response) -> bool {
    let name = format!("{}=", mw_admin::config::session_cookie_name());
    resp.headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .any(|v| v.trim_start().starts_with(&name))
}

/// What a request makes of rotation: `Some((token, forced))` when it carries an
/// admin session cookie on a route that may rotate.
fn target(method: &Method, path: &str, cookie: Option<String>) -> Option<(String, bool)> {
    let suffix = path
        .strip_prefix(&mw_admin::config::admin_path())
        .filter(|s| s.starts_with('/'))?;
    if suffix == "/login" || suffix.starts_with("/login/") || suffix == "/logout" {
        return None;
    }
    let forced = *method == Method::POST && suffix == "/step-up";
    Some((cookie?, forced))
}

pub(crate) async fn guard(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let cookie =
        crate::hardening::cookie(request.headers(), &mw_admin::config::session_cookie_name());
    let target = target(request.method(), request.uri().path(), cookie);
    let mut response = next.run(request).await;
    let Some((token, forced)) = target else {
        return response;
    };
    if !response.status().is_success() || sets_admin_cookie(&response) {
        return response;
    }
    let hash = crate::push_relay::hash_token(&token);
    let now = chrono::Utc::now().timestamp();
    if !forced && !age_exceeds(&hash, now, interval(&state).await) {
        return response;
    }
    if let Some((new_token, absolute)) = rotate(&state, &token).await {
        for v in cookies(&state, &new_token, absolute) {
            response.headers_mut().append(header::SET_COOKIE, v);
        }
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_is_due_after_the_interval_and_never_when_off() {
        let h = "unit-test-hash-a";
        assert!(!age_exceeds(h, 1_000, 900), "first sight starts the clock");
        assert!(!age_exceeds(h, 1_899, 900));
        assert!(age_exceeds(h, 1_900, 900));
        assert!(!age_exceeds(h, 9_999_999, 0), "0 turns interval rotation off");
        assert!(
            !age_exceeds("unit-test-hash-b", 5, 10),
            "first sight never rotates"
        );
    }

    #[test]
    fn login_logout_and_non_console_routes_do_not_rotate() {
        let base = mw_admin::config::admin_path();
        let c = || Some("tok".to_string());
        assert!(target(&Method::GET, &format!("{base}/login"), c()).is_none());
        assert!(target(&Method::POST, &format!("{base}/login/totp"), c()).is_none());
        assert!(target(&Method::POST, &format!("{base}/logout"), c()).is_none());
        assert!(target(&Method::GET, "/api/me", c()).is_none());
        assert!(target(&Method::GET, &format!("{base}/settings"), None).is_none());
        assert_eq!(
            target(&Method::GET, &format!("{base}/settings"), c()),
            Some(("tok".into(), false))
        );
        assert_eq!(
            target(&Method::POST, &format!("{base}/step-up"), c()),
            Some(("tok".into(), true))
        );
    }
}
