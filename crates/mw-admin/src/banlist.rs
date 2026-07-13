//! Login monitor + ban list (plan §2.5, §19 observability). Emits
//! authentication-failure lines in a **fail2ban-compatible** format so an
//! operator can point a fail2ban jail at Mailwoman's log and ban brute-force
//! sources, while the in-process monitor tracks failures per source IP and
//! recommends a ban once a threshold is crossed within a rolling window.

use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::Duration;

use chrono::{DateTime, Utc};
use regex::Regex;

/// The fail2ban `failregex` an operator adds to a jail filter to match
/// Mailwoman's auth-failure lines. `<HOST>` is fail2ban's IP/host token.
///
/// ```text
/// [Definition]
/// failregex = mailwoman\[auth\]: authentication failure; .*rhost=<HOST>
/// ```
pub const FAIL2BAN_FAILREGEX: &str = r"mailwoman\[auth\]: authentication failure; .*rhost=<HOST>";

/// The Rust equivalent of [`FAIL2BAN_FAILREGEX`] with `<HOST>` expanded to a
/// capturing group — used to prove our emitted lines are parseable.
static LINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"mailwoman\[auth\]: authentication failure; .*rhost=(?P<host>\S+)")
        .expect("valid fail2ban line regex")
});

/// Format one authentication-failure log line in a fail2ban-parseable shape.
/// The leading timestamp is RFC 3339 (matched by fail2ban's ISO-8601 date
/// detector). `user` is included for operators but is not required by the
/// filter; the IP is the `rhost=` token fail2ban keys on.
pub fn fail2ban_line(ts: DateTime<Utc>, user: &str, ip: &str) -> String {
    format!(
        "{} mailwoman[auth]: authentication failure; logname={} rhost={}",
        ts.to_rfc3339(),
        user,
        ip,
    )
}

/// Extract the host/IP from a line produced by [`fail2ban_line`] (mirrors what a
/// fail2ban jail does). Returns `None` if the line does not match the filter.
pub fn parse_host(line: &str) -> Option<String> {
    LINE_RE
        .captures(line)
        .and_then(|c| c.name("host"))
        .map(|m| m.as_str().to_string())
}

/// The verdict of recording a login failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginVerdict {
    /// Below threshold; the source is being watched.
    Watched { failures: u32 },
    /// Threshold crossed within the window — the source should be banned.
    Ban { failures: u32 },
}

/// In-process failure tracker (plan §2.5 login monitor). A source is recommended
/// for ban once `max_failures` failures occur within `window`. Successful logins
/// clear the counter. Persisted bans live in the [`crate::store::AdminBackend`];
/// this type only decides *when* to ban.
pub struct LoginMonitor {
    max_failures: u32,
    window: Duration,
    state: Mutex<HashMap<String, Vec<DateTime<Utc>>>>,
}

impl LoginMonitor {
    /// `max_failures` within `window` triggers a ban recommendation.
    pub fn new(max_failures: u32, window: Duration) -> Self {
        Self {
            max_failures,
            window,
            state: Mutex::new(HashMap::new()),
        }
    }

    /// Sensible default: 5 failures within 15 minutes.
    pub fn with_defaults() -> Self {
        Self::new(5, Duration::from_secs(15 * 60))
    }

    /// Record a failure for `ip` at `now`; prune the window and decide a verdict.
    pub fn record_failure(&self, ip: &str, now: DateTime<Utc>) -> LoginVerdict {
        let window = chrono::Duration::from_std(self.window).unwrap_or(chrono::Duration::zero());
        let cutoff = now - window;
        let mut state = self.state.lock().expect("login monitor poisoned");
        let hits = state.entry(ip.to_string()).or_default();
        hits.retain(|t| *t >= cutoff);
        hits.push(now);
        let failures = hits.len() as u32;
        if failures >= self.max_failures {
            LoginVerdict::Ban { failures }
        } else {
            LoginVerdict::Watched { failures }
        }
    }

    /// Clear the failure counter for `ip` (call on a successful login).
    pub fn record_success(&self, ip: &str) {
        self.state
            .lock()
            .expect("login monitor poisoned")
            .remove(ip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emitted_line_is_fail2ban_parseable() {
        let ts = Utc::now();
        let line = fail2ban_line(ts, "alice", "203.0.113.7");
        // The exported failregex (with <HOST> expanded) matches our line and the
        // captured host is the source IP — proving fail2ban compatibility.
        assert_eq!(parse_host(&line).as_deref(), Some("203.0.113.7"));
        assert!(FAIL2BAN_FAILREGEX.contains("<HOST>"));
        assert!(line.contains("authentication failure;"));
    }

    #[test]
    fn ipv6_host_is_captured() {
        let line = fail2ban_line(Utc::now(), "bob", "2001:db8::1");
        assert_eq!(parse_host(&line).as_deref(), Some("2001:db8::1"));
    }

    #[test]
    fn monitor_bans_after_threshold() {
        let mon = LoginMonitor::new(3, Duration::from_secs(600));
        let now = Utc::now();
        assert_eq!(
            mon.record_failure("1.2.3.4", now),
            LoginVerdict::Watched { failures: 1 }
        );
        assert_eq!(
            mon.record_failure("1.2.3.4", now),
            LoginVerdict::Watched { failures: 2 }
        );
        assert_eq!(
            mon.record_failure("1.2.3.4", now),
            LoginVerdict::Ban { failures: 3 }
        );
    }

    #[test]
    fn old_failures_fall_out_of_window() {
        let mon = LoginMonitor::new(3, Duration::from_secs(600));
        let start = Utc::now();
        mon.record_failure("5.6.7.8", start);
        mon.record_failure("5.6.7.8", start);
        // 20 minutes later, the first two are outside the 10-minute window.
        let later = start + chrono::Duration::minutes(20);
        assert_eq!(
            mon.record_failure("5.6.7.8", later),
            LoginVerdict::Watched { failures: 1 }
        );
    }

    #[test]
    fn success_clears_counter() {
        let mon = LoginMonitor::new(2, Duration::from_secs(600));
        let now = Utc::now();
        mon.record_failure("9.9.9.9", now);
        mon.record_success("9.9.9.9");
        assert_eq!(
            mon.record_failure("9.9.9.9", now),
            LoginVerdict::Watched { failures: 1 }
        );
    }
}
