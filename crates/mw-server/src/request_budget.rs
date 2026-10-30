//! The diagnostics budget of a signed-in admin session (t104).
//!
//! # What it replaced
//! `diagnostics::limit` allowed two runs per minute per admin session, shared by
//! `POST {console}/diag/run` and `POST {console}/diag/jobs`, so running the
//! server suite and then the backend checks twice was refused with `429`. A
//! signed-in session is attributable (it can be ended, its owner is audited),
//! so it now has a token bucket of its own, keyed by the session and never by
//! its address (admins behind one office address do not share it), with a burst
//! for several checks in a row. A repeat of a job the session already has
//! running is answered with that job and is not charged (`diagnostics_jobs`).
//!
//! # What it does not widen
//! Sign-in, step-up, password changes and linked-mailbox renewals keep their own
//! strict limiters (`signin_cooldown`, `signin_challenge`, `step_up`,
//! `linked_accounts`); this budget is never consulted for them. Anonymous
//! callers never reach these routes (`AdminCtx` answers `401` first).
//!
//! # Configuration
//! `rateLimit.diagnosticsBurst` and `rateLimit.diagnosticsPerMinute` (defaults
//! in [`Limits::default`]), written only through `PUT {console}/rate-limits`
//! (security policy permission, step-up, audited); the generic settings routes
//! refuse them. Every refusal is `429` with `Retry-After` and `retryAfterSecs`.
//!
//! State is process-local, per replica, and resets on restart, like the other
//! in-process limiters ([`crate::scope_mw::rate_limit`]).
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson},
    admin_router::AdminRegistrar,
    capabilities::Capability,
    scope_mw::rate_limit::KeyedRateLimiter,
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, Method, StatusCode, header},
    response::Response,
};
use mw_store::{Store, settings_policy::SettingPolicyRow};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

/// The console route that owns every `rateLimit.*` key.
pub const ROUTE: &str = "/rate-limits";

/// Burst and sustained rate of one bucket.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Limits {
    /// Runs one session may start at once (a never-seen session starts full).
    pub burst: u32,
    /// Runs per minute the bucket regains.
    pub per_minute: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            burst: 10,
            per_minute: 10,
        }
    }
}

/// `(key, field)` of each setting.
const KEYS: [(&str, &str); 2] = [
    ("rateLimit.diagnosticsBurst", "diagnosticsBurst"),
    ("rateLimit.diagnosticsPerMinute", "diagnosticsPerMinute"),
];

impl Limits {
    fn get(&self, field: &str) -> u32 {
        if field == "diagnosticsBurst" {
            self.burst
        } else {
            self.per_minute
        }
    }
    fn set(&mut self, field: &str, value: u32) {
        if field == "diagnosticsBurst" {
            self.burst = value;
        } else {
            self.per_minute = value;
        }
    }

    /// The values as the console reads and writes them.
    pub(crate) fn to_json(self) -> Value {
        let mut out = serde_json::Map::new();
        for (_, field) in KEYS {
            out.insert(field.into(), json!(self.get(field)));
        }
        Value::Object(out)
    }

    /// Each value inside its registry range. `Err(field)` names the field.
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        for (key, field) in KEYS {
            let def = crate::settings_registry::lookup(key).expect("registered rateLimit key");
            crate::settings_registry::validate(def, &json!(self.get(field)), "")
                .map_err(|_| field)?;
        }
        Ok(())
    }

    /// The stored values (defaults where unset). A stored value that no longer
    /// validates keeps its default rather than meaning "unlimited".
    pub(crate) async fn load(store: &Store) -> Result<Self, mw_store::StoreError> {
        let policies = store.list_setting_policies().await?;
        let mut limits = Self::default();
        for (key, field) in KEYS {
            let value = policies
                .iter()
                .find(|p| p.key == key)
                .and_then(|p| p.global_value.as_deref())
                .and_then(|v| v.parse::<u32>().ok());
            let def = crate::settings_registry::lookup(key).expect("registered rateLimit key");
            if let Some(value) = value
                && crate::settings_registry::validate(def, &json!(value), "").is_ok()
            {
                limits.set(field, value);
            }
        }
        Ok(limits)
    }
}

/// One limiter, rebuilt when the configured rate changes.
fn bucket() -> &'static Mutex<Option<(Limits, KeyedRateLimiter)>> {
    static B: OnceLock<Mutex<Option<(Limits, KeyedRateLimiter)>>> = OnceLock::new();
    B.get_or_init(Default::default)
}

/// Retained sessions. Only signed-in admins reach this, so far fewer in practice.
const KEY_CAPACITY: usize = 4_096;

/// Charge one run to `session`. `Err(seconds)` is how long until one is
/// admitted again.
pub(crate) fn charge_at(limits: Limits, session: &str, now: Instant) -> Result<(), u64> {
    let mut slot = bucket().lock().unwrap_or_else(|e| e.into_inner());
    if slot.as_ref().is_none_or(|(current, _)| *current != limits) {
        *slot = Some((
            limits,
            KeyedRateLimiter::new(
                limits.burst as f64,
                limits.per_minute as f64 / 60.0,
                KEY_CAPACITY,
            ),
        ));
    }
    let limiter = &mut slot.as_mut().expect("just set").1;
    match limiter.retry_after_at(session, now) {
        None => Ok(()),
        Some(wait) => Err(retry_secs(wait)),
    }
}

/// Whole seconds to wait, at least 1, at most an hour.
fn retry_secs(wait: Duration) -> u64 {
    let secs = wait.as_secs() + u64::from(wait.subsec_nanos() > 0);
    secs.clamp(1, 3600)
}

/// The `429` a refusal answers: a code, the wait, and `Retry-After` (set by
/// [`AdminApiError`]'s response from `retryAfterSecs`).
pub(crate) fn refused(retry_after_secs: u64) -> AdminApiError {
    AdminApiError::new(
        StatusCode::TOO_MANY_REQUESTS,
        "rate_limited",
        format!("Too many requests. Try again in {retry_after_secs} s."),
    )
    .with("retryAfterSecs", retry_after_secs)
}

/// Charge an admin session's diagnostic run or job.
pub(crate) async fn diagnostics(state: &AppState, ctx: &AdminCtx) -> Result<(), AdminApiError> {
    let limits = Limits::load(&state.store).await?;
    charge_at(limits, &ctx.session_hash, Instant::now()).map_err(refused)
}

// ─── admin API ───────────────────────────────────────────────────────────────────

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, ROUTE, Capability::SecurityRead, read);
    r.admin_endpoint(Method::PUT, ROUTE, Capability::SecurityPolicyWrite, save);
}

fn body(limits: Limits) -> Value {
    json!({
        "limits": limits.to_json(),
        "defaults": Limits::default().to_json(),
        "keys": KEYS.iter().map(|(key, field)| json!({"key": key, "field": field})).collect::<Vec<_>>(),
    })
}

fn private(value: Value) -> Response {
    let tag = crate::admin_ctx::etag_of(&value);
    crate::admin_ctx::with_etag(
        ([(header::CACHE_CONTROL, "private, no-store")], Json(value)),
        &tag,
    )
}

async fn read(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SecurityRead)?;
    Ok(private(body(Limits::load(&state.store).await?)))
}

/// `PUT {console}/rate-limits`: either field or both; an absent field keeps its
/// stored value and `null` returns it to the default.
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Change {
    #[serde(default, deserialize_with = "present")]
    diagnostics_burst: Option<Option<u32>>,
    #[serde(default, deserialize_with = "present")]
    diagnostics_per_minute: Option<Option<u32>>,
}

/// A field that is present: `null` is `Some(None)` (back to the default), absent
/// is `None` (unchanged).
fn present<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<Option<u32>>, D::Error> {
    Option::<u32>::deserialize(d).map(Some)
}

impl Change {
    fn field(&self, field: &str) -> Option<Option<u32>> {
        if field == "diagnosticsBurst" {
            self.diagnostics_burst
        } else {
            self.diagnostics_per_minute
        }
    }
}

async fn save(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(change): AdminJson<Change>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SecurityPolicyWrite)?;
    let before = Limits::load(&state.store).await?;
    crate::admin_ctx::check_if_match(
        &headers,
        &crate::admin_ctx::etag_of(&body(before)),
        None,
        None,
    )?;
    let mut next = before;
    for (_, field) in KEYS {
        match change.field(field) {
            None => {}
            Some(None) => next.set(field, Limits::default().get(field)),
            Some(Some(value)) => next.set(field, value),
        }
    }
    next.validate().map_err(|field| {
        AdminApiError::invalid(
            "invalidValue",
            format!("{field} is outside its allowed range"),
            format!("/{field}"),
        )
    })?;
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::BanManage).await?;
    let now = chrono::Utc::now().timestamp();
    for (key, field) in KEYS {
        let Some(requested) = change.field(field) else {
            continue;
        };
        state
            .store
            .put_setting_policy(&SettingPolicyRow {
                key: key.into(),
                global_value: requested.map(|v| v.to_string()),
                allow_user_override: false,
                global_locked: false,
                updated_by: ctx.username.clone(),
                updated_at: now,
            })
            .await?;
    }
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "rate-limits-changed",
        "admin:rate-limits",
        Some(before.to_json()),
        Some(next.to_json()),
    )
    .await;
    Ok(private(body(next)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    fn session() -> String {
        format!("t104-{}", uuid::Uuid::new_v4())
    }

    #[test]
    fn a_session_has_a_burst_and_its_own_bucket() {
        let limits = Limits::default();
        let now = Instant::now();
        let one = session();
        let admitted = (0..100)
            .take_while(|_| charge_at(limits, &one, now).is_ok())
            .count();
        assert_eq!(admitted, limits.burst as usize);
        assert!(admitted > 2, "the old limit was two a minute");
        // Keyed by session: another admin (same office address or not) is unspent.
        assert!(charge_at(limits, &session(), now).is_ok());
        // Refill: one more run after 60 / per_minute seconds.
        let later = now + Duration::from_secs(60 / limits.per_minute as u64);
        assert!(charge_at(limits, &one, later).is_ok());
    }

    #[test]
    fn a_refusal_names_the_wait_and_sends_retry_after() {
        let limits = Limits {
            burst: 1,
            per_minute: 1,
        };
        let one = session();
        let now = Instant::now();
        assert!(charge_at(limits, &one, now).is_ok());
        let wait = charge_at(limits, &one, now).unwrap_err();
        assert_eq!(wait, 60);
        let response = refused(wait).into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers()[header::RETRY_AFTER], "60");
        assert_eq!(retry_secs(Duration::from_millis(1)), 1);
        assert_eq!(retry_secs(Duration::from_secs(99_999)), 3600);
    }

    #[test]
    fn values_are_validated_against_the_registry_and_only_the_route_writes_them() {
        assert!(Limits::default().validate().is_ok());
        assert_eq!(
            Limits {
                burst: 0,
                ..Limits::default()
            }
            .validate(),
            Err("diagnosticsBurst")
        );
        assert_eq!(
            Limits {
                per_minute: 1_000_000,
                ..Limits::default()
            }
            .validate(),
            Err("diagnosticsPerMinute")
        );
        for (key, _) in KEYS {
            let def = crate::settings_registry::lookup(key).unwrap();
            assert_eq!(def.dedicated_route, Some(ROUTE), "{key}");
            assert!(crate::settings_registry::refuse_if_dedicated(def).is_err());
            assert!(!def.user_override, "{key}");
        }
    }

    #[tokio::test]
    async fn stored_values_load_and_invalid_ones_keep_the_default() {
        let store = Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        assert_eq!(Limits::load(&store).await.unwrap(), Limits::default());
        for (key, value) in [
            ("rateLimit.diagnosticsBurst", "3"),
            ("rateLimit.diagnosticsPerMinute", "0"),
        ] {
            store
                .put_setting_policy(&SettingPolicyRow {
                    key: key.into(),
                    global_value: Some(value.into()),
                    allow_user_override: false,
                    global_locked: false,
                    updated_by: "test".into(),
                    updated_at: 1,
                })
                .await
                .unwrap();
        }
        let limits = Limits::load(&store).await.unwrap();
        assert_eq!(limits.burst, 3);
        assert_eq!(
            limits.per_minute,
            Limits::default().per_minute,
            "0 is out of range and keeps the default"
        );
    }
}
