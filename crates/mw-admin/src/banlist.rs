//! Login monitor + ban list (plan §2.5, §19 observability). Emits
//! authentication-failure lines in a **fail2ban-compatible** format so an
//! operator can point a fail2ban jail at Mailwoman's log and ban brute-force
//! sources, while the in-process monitor tracks failures per source IP and
//! recommends a ban once a threshold is crossed within a rolling window.

use std::collections::HashMap;
use std::net::IpAddr;
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
///
/// `ip` is an [`IpAddr`], not a string: the `rhost=` token is what a jail bans,
/// so it must be an address the caller *resolved*, never a label it invented.
/// `user` is the login name as supplied by the caller — i.e. attacker-chosen —
/// and is passed through [`sanitize_logname`] before it reaches the line.
pub fn fail2ban_line(ts: DateTime<Utc>, user: &str, ip: IpAddr) -> String {
    format!(
        "{} mailwoman[auth]: authentication failure; logname={} rhost={}",
        ts.to_rfc3339(),
        sanitize_logname(user),
        ip,
    )
}

/// Reduce an attacker-supplied login name to a token that cannot restructure the
/// log line it is embedded in.
///
/// Two concrete attacks, both reachable because the username arrives in a JSON
/// login body and JSON strings carry any character:
///
/// * A `\r` or `\n` in the name ends the record and starts another. The forged
///   continuation can be a complete, well-formed auth-failure line naming any
///   `rhost=` the attacker likes — so a jail reading our log would ban an address
///   the attacker chose. That turns the ban list into a remote denial-of-service
///   primitive aimed at third parties.
/// * An `=` lets the name carry its own `rhost=` token. Our own `rhost=` is
///   emitted last and [`FAIL2BAN_FAILREGEX`]'s `.*` is greedy, so the shipped
///   filter still reads the real address — but an operator's hand-written
///   non-greedy variant would not, and that is not a distinction to bet on.
///
/// The rule is an allowlist, not a denylist: ASCII alphanumerics plus `@ . _ - +`
/// (everything a mail login needs) survive; every other byte becomes `_`. The
/// token is capped so one request cannot write an unbounded log record.
fn sanitize_logname(user: &str) -> String {
    const MAX: usize = 128;
    let mut out = String::with_capacity(user.len().min(MAX));
    for ch in user.chars().take(MAX) {
        if ch.is_ascii_alphanumeric() || matches!(ch, '@' | '.' | '_' | '-' | '+') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push('-');
    }
    out
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
///
/// The bucket key is an [`IpAddr`] rather than a string. Until 26.19 it was a
/// `String`, and the sole caller — the admin-panel login handler — passed the
/// literal `"admin-panel"` for every attempt from every source. Each bucket held
/// the whole internet, so the threshold said "ban" the moment any five failures
/// occurred anywhere, and per-source tracking did not exist. Typing the key means
/// a caller with no address cannot invent one: it has to say so (see
/// [`crate::Admin::record_login_failure`], which takes `Option<IpAddr>`).
pub struct LoginMonitor {
    max_failures: u32,
    window: Duration,
    state: Mutex<HashMap<IpAddr, Vec<DateTime<Utc>>>>,
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
    /// `ip` is canonicalised first, so a dual-stack listener's `::ffff:a.b.c.d`
    /// and the same peer seen as `a.b.c.d` share one bucket instead of getting a
    /// free second allowance each.
    pub fn record_failure(&self, ip: IpAddr, now: DateTime<Utc>) -> LoginVerdict {
        let window = chrono::Duration::from_std(self.window).unwrap_or(chrono::Duration::zero());
        let cutoff = now - window;
        let mut state = self.state.lock().expect("login monitor poisoned");
        let hits = state.entry(ip.to_canonical()).or_default();
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
    pub fn record_success(&self, ip: IpAddr) {
        self.state
            .lock()
            .expect("login monitor poisoned")
            .remove(&ip.to_canonical());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().expect("test address")
    }

    #[test]
    fn emitted_line_is_fail2ban_parseable() {
        let ts = Utc::now();
        let line = fail2ban_line(ts, "alice", ip("203.0.113.7"));
        // The exported failregex (with <HOST> expanded) matches our line and the
        // captured host is the source IP — proving fail2ban compatibility.
        assert_eq!(parse_host(&line).as_deref(), Some("203.0.113.7"));
        assert!(FAIL2BAN_FAILREGEX.contains("<HOST>"));
        assert!(line.contains("authentication failure;"));
    }

    #[test]
    fn ipv6_host_is_captured() {
        let line = fail2ban_line(Utc::now(), "bob", ip("2001:db8::1"));
        assert_eq!(parse_host(&line).as_deref(), Some("2001:db8::1"));
    }

    #[test]
    fn monitor_bans_after_threshold() {
        let mon = LoginMonitor::new(3, Duration::from_secs(600));
        let now = Utc::now();
        assert_eq!(
            mon.record_failure(ip("1.2.3.4"), now),
            LoginVerdict::Watched { failures: 1 }
        );
        assert_eq!(
            mon.record_failure(ip("1.2.3.4"), now),
            LoginVerdict::Watched { failures: 2 }
        );
        assert_eq!(
            mon.record_failure(ip("1.2.3.4"), now),
            LoginVerdict::Ban { failures: 3 }
        );
    }

    #[test]
    fn old_failures_fall_out_of_window() {
        let mon = LoginMonitor::new(3, Duration::from_secs(600));
        let start = Utc::now();
        mon.record_failure(ip("5.6.7.8"), start);
        mon.record_failure(ip("5.6.7.8"), start);
        // 20 minutes later, the first two are outside the 10-minute window.
        let later = start + chrono::Duration::minutes(20);
        assert_eq!(
            mon.record_failure(ip("5.6.7.8"), later),
            LoginVerdict::Watched { failures: 1 }
        );
    }

    #[test]
    fn success_clears_counter() {
        let mon = LoginMonitor::new(2, Duration::from_secs(600));
        let now = Utc::now();
        mon.record_failure(ip("9.9.9.9"), now);
        mon.record_success(ip("9.9.9.9"));
        assert_eq!(
            mon.record_failure(ip("9.9.9.9"), now),
            LoginVerdict::Watched { failures: 1 }
        );
    }

    /// The B3 regression, stated as a property: one source crossing the threshold
    /// must not carry any other source with it. Under the old shared `"admin-panel"`
    /// key this failed — every attempt landed in one bucket, so the fifth failure
    /// from anywhere banned everyone at once.
    #[test]
    fn buckets_are_per_source_not_shared() {
        let mon = LoginMonitor::new(3, Duration::from_secs(600));
        let now = Utc::now();

        // One noisy source runs itself up to the threshold.
        for _ in 0..2 {
            mon.record_failure(ip("198.51.100.9"), now);
        }
        assert_eq!(
            mon.record_failure(ip("198.51.100.9"), now),
            LoginVerdict::Ban { failures: 3 }
        );

        // An unrelated source is untouched by that: it starts from one.
        assert_eq!(
            mon.record_failure(ip("203.0.113.4"), now),
            LoginVerdict::Watched { failures: 1 }
        );
        // ...and so does a v6 source.
        assert_eq!(
            mon.record_failure(ip("2001:db8::5"), now),
            LoginVerdict::Watched { failures: 1 }
        );
        // Clearing one source does not clear another.
        mon.record_success(ip("198.51.100.9"));
        assert_eq!(
            mon.record_failure(ip("203.0.113.4"), now),
            LoginVerdict::Watched { failures: 2 }
        );
    }

    /// A dual-stack listener reports a v4 peer as `::ffff:a.b.c.d`. If that were a
    /// distinct key, the same host would get a second full allowance by arriving
    /// over the other socket family.
    #[test]
    fn mapped_v4_shares_a_bucket_with_plain_v4() {
        let mon = LoginMonitor::new(2, Duration::from_secs(600));
        let now = Utc::now();
        mon.record_failure(ip("::ffff:203.0.113.4"), now);
        assert_eq!(
            mon.record_failure(ip("203.0.113.4"), now),
            LoginVerdict::Ban { failures: 2 }
        );
        // Success on either spelling clears the one shared bucket.
        mon.record_success(ip("::ffff:203.0.113.4"));
        assert_eq!(
            mon.record_failure(ip("203.0.113.4"), now),
            LoginVerdict::Watched { failures: 1 }
        );
    }

    /// A username arrives in a JSON login body, so it can hold any character. It
    /// must not be able to end the log record and write a second one — a forged
    /// continuation naming a victim's `rhost=` would make a jail ban that victim.
    #[test]
    fn logname_cannot_forge_a_second_log_record() {
        let line = fail2ban_line(
            Utc::now(),
            "eve\n2026-01-01T00:00:00+00:00 mailwoman[auth]: authentication failure; logname=x rhost=203.0.113.99",
            ip("198.51.100.9"),
        );
        assert_eq!(line.lines().count(), 1, "must stay one log record");
        assert!(!line.contains('\n') && !line.contains('\r'));
        // The forged text survives as inert characters — substitution, not
        // deletion — but it can no longer be a *record* or an *rhost token*, which
        // is what a jail acts on. There is exactly one of each, and it is ours.
        assert_eq!(line.matches("rhost=").count(), 1, "{line}");
        assert_eq!(line.matches("mailwoman[auth]:").count(), 1, "{line}");
        assert_eq!(parse_host(&line).as_deref(), Some("198.51.100.9"));
    }

    #[test]
    fn logname_cannot_smuggle_its_own_rhost_token() {
        let line = fail2ban_line(Utc::now(), "rhost=203.0.113.99", ip("198.51.100.9"));
        assert_eq!(parse_host(&line).as_deref(), Some("198.51.100.9"));
        assert_eq!(line.matches("rhost=").count(), 1, "one rhost= token only");
    }

    #[test]
    fn logname_keeps_ordinary_mail_logins_readable() {
        // The allowlist is not so tight that it mangles a normal login name.
        assert_eq!(
            sanitize_logname("alice.smith+tag@example.com"),
            "alice.smith+tag@example.com"
        );
        assert_eq!(sanitize_logname("Bob_99-x"), "Bob_99-x");
        // Substitution, not removal, so length is still evidence.
        assert_eq!(sanitize_logname("a b"), "a_b");
        // An empty or fully-substituted name still produces a token.
        assert_eq!(sanitize_logname(""), "-");
        // One request cannot write an unbounded record.
        assert_eq!(sanitize_logname(&"x".repeat(500)).chars().count(), 128);
    }
}
