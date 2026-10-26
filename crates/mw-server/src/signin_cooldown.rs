//! Initial sign-in pacing. Independent of failed-credential bans and MFA limits.
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
pub(crate) async fn check(
    state: &AppState,
    headers: &HeaderMap,
    ext: &Extensions,
) -> Option<Response> {
    // Production serving always installs ConnectInfo; synthetic in-process requests may not.
    let source = crate::scope_mw::proxy::client_ip(headers, ext)?;
    let seconds = match settings_registry::effective_for(
        &state.store,
        "signin.retryCooldownSecs",
        &Default::default(),
    )
    .await
    {
        Ok(Some(value)) => value.value.as_u64().unwrap_or(3).min(300),
        Ok(None) => 3,
        Err(_) => return Some(StatusCode::SERVICE_UNAVAILABLE.into_response()),
    };
    let remaining = state
        .signin_cooldown
        .admit(source, seconds, Instant::now())?;
    let seconds = remaining.as_secs() + u64::from(remaining.subsec_nanos() != 0);
    let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(serde_json::json!({"code":"signinCooldown","retryAfterSecs":seconds,"error":"Please wait before trying again."}))).into_response();
    response
        .headers_mut()
        .insert(header::RETRY_AFTER, seconds.to_string().parse().unwrap());
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    Some(response)
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
