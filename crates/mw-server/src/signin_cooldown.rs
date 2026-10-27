//! Initial sign-in pacing. Independent of failed-credential bans and MFA limits.
//!
//! # Only attempts that do not end in a session count (t33 D1)
//! Each attempt is charged one token per client network (IPv4 address, IPv6 /64)
//! **up front**, exactly as before, so a burst of parallel guesses from one source
//! still gets one attempt per `signin.retryCooldownSecs` and the rest a `429` while
//! the first is in flight. When the attempt ends in an issued session, its token is
//! given back ([`Admission::settle`] / [`Admission::succeeded`]), so a successful
//! sign-in does not make the next person behind the same NAT or office address wait.
//!
//! Everything else keeps the charge: a wrong password, an unknown user, a disabled
//! account, an upstream error, and a correct password that only reaches a
//! second-factor challenge (no session yet — refunding there would hand someone who
//! holds a password a fresh 5-guess challenge with no pacing). So failed attempts
//! are paced precisely as before, and a success can only return the one token it was
//! itself charged; it never resets a wait an earlier failure started, because that
//! wait has to elapse before the successful attempt is admitted at all.
use crate::{AppState, scope_mw::rate_limit::KeyedRateLimiter, settings_registry};
use axum::{
    Json,
    http::{Extensions, HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use std::{
    net::IpAddr,
    sync::Mutex,
    time::{Duration, Instant},
};

#[derive(Default)]
pub(crate) struct Cooldown(Mutex<Option<(u64, KeyedRateLimiter)>>);
impl Cooldown {
    fn admit(&self, ip: IpAddr, seconds: u64, now: Instant) -> Option<Duration> {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if seconds == 0 {
            *state = None;
            return None;
        }
        if state.as_ref().is_none_or(|(old, _)| *old != seconds) {
            *state = Some((
                seconds,
                KeyedRateLimiter::new(1.0, 1.0 / seconds as f64, 10_000),
            ));
        }
        state.as_mut().unwrap().1.retry_after_at(&key(ip), now)
    }
    /// Give back the token an admitted attempt was charged. A no-op when the policy
    /// changed in between (the limiter was rebuilt, so the charge is gone anyway).
    fn refund(&self, ip: IpAddr, seconds: u64, now: Instant) {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((current, limiter)) = state.as_mut()
            && *current == seconds
        {
            limiter.refund_at(&key(ip), now);
        }
    }
}

/// Response marker: this response issued a signed-in session. Set by the one
/// webmail session builder (`authenticated_response`), read by [`Admission::settle`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct SessionIssued;

/// The charge an admitted attempt holds. Dropping it keeps the charge (the attempt
/// counted); [`settle`](Self::settle) or [`succeeded`](Self::succeeded) gives it back
/// once a session was issued.
#[must_use = "settle the admission once the attempt's outcome is known"]
pub(crate) struct Admission(Option<(IpAddr, u64)>);
impl Admission {
    /// Give the token back if `response` issued a session ([`SessionIssued`]).
    pub(crate) fn settle(self, state: &AppState, response: &Response) {
        if response.extensions().get::<SessionIssued>().is_some() {
            self.succeeded(state);
        }
    }
    /// The attempt ended in an issued session: give its token back.
    pub(crate) fn succeeded(self, state: &AppState) {
        if let Some((ip, seconds)) = self.0 {
            state.signin_cooldown.refund(ip, seconds, Instant::now());
        }
    }
}
fn key(ip: IpAddr) -> String {
    match ip {
        IpAddr::V4(ip) => ip.to_string(),
        IpAddr::V6(ip) => match ip.to_ipv4_mapped() {
            Some(ip) => ip.to_string(),
            None => {
                let s = ip.segments();
                format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
            }
        },
    }
}
/// Admit one sign-in attempt, charging its source; `Err` is the `429` to answer.
pub(crate) async fn check(
    state: &AppState,
    headers: &HeaderMap,
    ext: &Extensions,
) -> Result<Admission, Response> {
    // Production serving always installs ConnectInfo; synthetic in-process requests may not.
    let Some(source) = crate::scope_mw::proxy::client_ip(headers, ext) else {
        return Ok(Admission(None));
    };
    let seconds = match settings_registry::effective_for(
        &state.store,
        "signin.retryCooldownSecs",
        &Default::default(),
    )
    .await
    {
        Ok(Some(value)) => value.value.as_u64().unwrap_or(3).min(300),
        Ok(None) => 3,
        Err(_) => return Err(StatusCode::SERVICE_UNAVAILABLE.into_response()),
    };
    let Some(remaining) = state.signin_cooldown.admit(source, seconds, Instant::now()) else {
        return Ok(Admission((seconds > 0).then_some((source, seconds))));
    };
    let seconds = remaining.as_secs() + u64::from(remaining.subsec_nanos() != 0);
    let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(serde_json::json!({"code":"signinCooldown","retryAfterSecs":seconds,"error":"Please wait before trying again."}))).into_response();
    response
        .headers_mut()
        .insert(header::RETRY_AFTER, seconds.to_string().parse().unwrap());
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    Err(response)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_deadline_rejections_do_not_extend_cooldown_and_policy_is_hot() {
        let c = Cooldown::default();
        let ip = "192.0.2.1".parse().unwrap();
        let now = Instant::now();
        assert_eq!(c.admit(ip, 3, now), None);
        assert_eq!(
            c.admit(ip, 3, now + Duration::from_secs(1)),
            Some(Duration::from_secs(2))
        );
        assert!(c.admit(ip, 3, now + Duration::from_secs(3)).is_none());
        assert!(c.admit(ip, 0, now).is_none());
        assert!(c.admit(ip, 5, now).is_none());
        assert_eq!(c.admit(ip, 5, now), Some(Duration::from_secs(5)));
    }
    #[test]
    fn a_refunded_attempt_leaves_no_wait_and_an_unrefunded_one_keeps_it() {
        let c = Cooldown::default();
        let ip: IpAddr = "192.0.2.1".parse().unwrap();
        let now = Instant::now();
        // Success: charged up front, refunded, the next attempt is admitted at once.
        assert_eq!(c.admit(ip, 3, now), None);
        c.refund(ip, 3, now);
        assert_eq!(
            c.admit(ip, 3, now),
            None,
            "a refunded success leaves no wait"
        );
        // That second attempt is a failure (not refunded): the full window applies.
        assert_eq!(c.admit(ip, 3, now), Some(Duration::from_secs(3)));
        assert_eq!(
            c.admit(ip, 3, now + Duration::from_secs(2)),
            Some(Duration::from_secs(1)),
            "a rejected retry neither extends nor shortens the window"
        );
        // A refund cannot bank credit beyond one attempt.
        let other: IpAddr = "192.0.2.2".parse().unwrap();
        assert_eq!(c.admit(other, 3, now), None);
        c.refund(other, 3, now);
        c.refund(other, 3, now);
        assert_eq!(c.admit(other, 3, now), None);
        assert!(c.admit(other, 3, now).is_some(), "burst stays at one");
        // A refund for a different policy value is a no-op.
        assert_eq!(c.admit(ip, 3, now + Duration::from_secs(3)), None);
        c.refund(ip, 5, now + Duration::from_secs(3));
        assert!(c.admit(ip, 3, now + Duration::from_secs(3)).is_some());
    }
    #[test]
    fn cooldown_is_a_real_global_setting_with_bounded_values() {
        let def = settings_registry::lookup("signin.retryCooldownSecs").unwrap();
        assert_eq!(def.default_value(), serde_json::json!(3));
        assert!(settings_registry::refuse_if_dedicated(def).is_ok());
        for value in [0, 3, 300] {
            assert!(settings_registry::validate(def, &serde_json::json!(value), "/value").is_ok());
        }
        for value in [-1, 301] {
            assert!(settings_registry::validate(def, &serde_json::json!(value), "/value").is_err());
        }
    }

    #[test]
    fn network_keys_cannot_rotate_within_ipv6_subnet_or_mapped_ipv4() {
        assert_eq!(
            key("::ffff:192.0.2.1".parse().unwrap()),
            key("192.0.2.1".parse().unwrap())
        );
        assert_eq!(
            key("2001:db8:1:2::1".parse().unwrap()),
            key("2001:db8:1:2::abcd".parse().unwrap())
        );
        assert_ne!(
            key("2001:db8:1:2::1".parse().unwrap()),
            key("2001:db8:1:3::1".parse().unwrap())
        );
    }
}
