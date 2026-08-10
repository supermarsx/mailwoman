//! Web-hardening deltas (SPEC §7.4, plan §3 e10), all **additive** — they must
//! not weaken the existing sanitizer/sandbox contract:
//!
//! * extra response headers: COEP `require-corp`, CORP `same-origin`,
//!   `Permissions-Policy` (deny powerful features);
//! * an **Origin/Referer check** on state-changing requests (effective CSRF
//!   defense that needs no client change — browsers always send `Origin` on
//!   cross-site writes; native clients that send neither are allowed);
//! * an optional **double-submit CSRF token** (`mw_csrf` cookie ↔ `X-CSRF-Token`
//!   header), enforced only when `csrf_strict` is on so V1 clients keep working;
//! * in-process **idle / absolute session timeouts** and **rotation** helpers
//!   (the store does not expose session timestamps and is owned by e9, so timing
//!   is tracked here for the process lifetime).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::http::{HeaderMap, HeaderValue, Method, header};

/// Cookie carrying the readable CSRF token (double-submit half).
pub const CSRF_COOKIE: &str = "mw_csrf";
/// Header the SPA echoes the token in.
pub const CSRF_HEADER: &str = "x-csrf-token";

/// Permissions-Policy: deny the powerful features the webmail never uses.
const PERMISSIONS_POLICY: &str = "accelerometer=(), autoplay=(), camera=(), \
    display-capture=(), encrypted-media=(), fullscreen=(self), geolocation=(), \
    gyroscope=(), magnetometer=(), microphone=(), midi=(), payment=(), \
    picture-in-picture=(), usb=(), interest-cohort=()";

/// Tunable hardening knobs (populated from the CLI/env).
#[derive(Debug, Clone)]
pub struct HardeningConfig {
    /// Emit `Cross-Origin-Embedder-Policy: require-corp` (crossOriginIsolated).
    pub coep: bool,
    /// Enforce the double-submit CSRF token (needs the SPA to send the header).
    pub csrf_strict: bool,
    /// No activity for this long invalidates the session.
    pub idle_timeout: Duration,
    /// A session is force-expired this long after creation regardless of use.
    pub absolute_timeout: Duration,
}

impl Default for HardeningConfig {
    fn default() -> Self {
        Self {
            coep: true,
            csrf_strict: false,
            idle_timeout: Duration::from_secs(30 * 60),
            absolute_timeout: Duration::from_secs(12 * 60 * 60),
        }
    }
}

/// Append the additive security headers to a response header map. The base CSP /
/// XFO / nosniff / COOP set is applied by the caller; this adds COEP/CORP/PP.
pub fn apply_extra_headers(h: &mut HeaderMap, coep: bool) {
    h.insert(
        "cross-origin-resource-policy",
        HeaderValue::from_static("same-origin"),
    );
    h.insert(
        "permissions-policy",
        HeaderValue::from_static(PERMISSIONS_POLICY),
    );
    if coep {
        h.insert(
            "cross-origin-embedder-policy",
            HeaderValue::from_static("require-corp"),
        );
    }
}

/// Whether a method mutates state and therefore needs CSRF/Origin protection.
pub fn is_state_changing(method: &Method) -> bool {
    matches!(
        *method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    )
}

/// Origin/Referer same-site check. Returns `true` when the request is safe to
/// process: either it carries no `Origin`/`Referer` (non-browser client) or the
/// header's authority matches the request's own target authority.
///
/// `authority` is the URI authority (`req.uri().authority()`), which is where
/// HTTP/2 carries `:authority` — hyper does not synthesise a `Host` header for
/// h2, so a proxy that speaks h2 to the backend (Caddy and Traefik both can)
/// produces requests with no `Host` header at all. The `Host` header wins when
/// present; the URI authority is the h2 fallback.
///
/// **Fails closed with no authority at all** (t20 B8). The previous behaviour
/// returned `true` here, which let a request that simply omitted `Host` skip the
/// same-site check entirely — the check has nothing to compare an `Origin`
/// against, so it cannot attest anything, and allowing it turned a malformed
/// request into a CSRF bypass. HTTP/1.1 makes `Host` mandatory and h2 supplies
/// `:authority`, so a well-formed request from any real client still passes.
pub fn origin_ok(headers: &HeaderMap, authority: Option<&str>) -> bool {
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .or(authority);
    let Some(host) = host else {
        return false;
    };
    if let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
        return authority_of(origin).is_some_and(|a| a.eq_ignore_ascii_case(host));
    }
    if let Some(referer) = headers.get(header::REFERER).and_then(|v| v.to_str().ok()) {
        return authority_of(referer).is_some_and(|a| a.eq_ignore_ascii_case(host));
    }
    true
}

/// Extract the `host[:port]` authority from a URL-ish string (`scheme://auth/..`).
fn authority_of(url: &str) -> Option<&str> {
    let after = url.split("://").nth(1)?;
    Some(after.split(['/', '?', '#']).next().unwrap_or(after))
}

/// Double-submit CSRF check: the `X-CSRF-Token` header must equal the `mw_csrf`
/// cookie (both present and non-empty).
pub fn csrf_double_submit_ok(headers: &HeaderMap) -> bool {
    let header_tok = headers.get(CSRF_HEADER).and_then(|v| v.to_str().ok());
    let cookie_tok = cookie(headers, CSRF_COOKIE);
    match (header_tok, cookie_tok) {
        (Some(h), Some(c)) => !h.is_empty() && h == c,
        _ => false,
    }
}

/// Pull one cookie value out of the `Cookie` header.
pub fn cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    let needle = format!("{name}=");
    for part in raw.split(';') {
        if let Some(v) = part.trim().strip_prefix(&needle)
            && !v.is_empty()
        {
            return Some(v.to_string());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Session timing (idle + absolute timeouts, rotation)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct Timing {
    created: Instant,
    last_seen: Instant,
}

/// Reason a session was rejected (surfaced for logs/tests).
#[derive(Debug, PartialEq, Eq)]
pub enum Expiry {
    Idle,
    Absolute,
}

/// In-process record of when each session was created and last used, so the
/// server can enforce idle/absolute timeouts without persisting timestamps.
#[derive(Default)]
pub struct SessionGuard {
    inner: Mutex<HashMap<String, Timing>>,
}

impl SessionGuard {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start tracking a freshly created session (login), anchoring both the idle
    /// and absolute clocks at now.
    pub fn begin(&self, id: &str) {
        let now = Instant::now();
        self.inner.lock().expect("session guard lock").insert(
            id.to_string(),
            Timing {
                created: now,
                last_seen: now,
            },
        );
    }

    /// Validate a session id against the timeouts and record activity. First
    /// sighting (e.g. after a restart) seeds the timers leniently. Returns
    /// `Err(reason)` when the session has expired (caller should delete it).
    pub fn check(&self, id: &str, cfg: &HardeningConfig) -> Result<(), Expiry> {
        self.check_at(id, cfg, Instant::now())
    }

    fn check_at(&self, id: &str, cfg: &HardeningConfig, now: Instant) -> Result<(), Expiry> {
        let mut map = self.inner.lock().expect("session guard lock");
        match map.get(id).copied() {
            Some(t) => {
                if now.duration_since(t.created) > cfg.absolute_timeout {
                    map.remove(id);
                    return Err(Expiry::Absolute);
                }
                if now.duration_since(t.last_seen) > cfg.idle_timeout {
                    map.remove(id);
                    return Err(Expiry::Idle);
                }
                map.insert(
                    id.to_string(),
                    Timing {
                        created: t.created,
                        last_seen: now,
                    },
                );
                Ok(())
            }
            None => {
                map.insert(
                    id.to_string(),
                    Timing {
                        created: now,
                        last_seen: now,
                    },
                );
                Ok(())
            }
        }
    }

    /// Drop a session's timing (logout / expiry).
    pub fn forget(&self, id: &str) {
        self.inner.lock().expect("session guard lock").remove(id);
    }

    /// Move timing from an old id to a new one, resetting the idle clock but
    /// preserving the absolute-creation instant (rotation must not extend the
    /// absolute lifetime).
    pub fn rotate(&self, old: &str, new: &str) {
        let mut map = self.inner.lock().expect("session guard lock");
        let created = map
            .remove(old)
            .map(|t| t.created)
            .unwrap_or_else(Instant::now);
        let now = Instant::now();
        map.insert(
            new.to_string(),
            Timing {
                created,
                last_seen: now,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut m = HeaderMap::new();
        for (k, v) in pairs {
            m.insert(
                header::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                HeaderValue::from_str(v).unwrap(),
            );
        }
        m
    }

    #[test]
    fn origin_absent_is_allowed() {
        assert!(origin_ok(&h(&[("host", "mail.example.org")]), None));
    }

    #[test]
    fn matching_origin_is_allowed_mismatch_is_blocked() {
        assert!(origin_ok(
            &h(&[
                ("host", "mail.example.org"),
                ("origin", "https://mail.example.org"),
            ]),
            None
        ));
        assert!(!origin_ok(
            &h(&[
                ("host", "mail.example.org"),
                ("origin", "https://evil.example"),
            ]),
            None
        ));
    }

    #[test]
    fn referer_is_the_fallback() {
        assert!(origin_ok(
            &h(&[
                ("host", "mail.example.org"),
                ("referer", "https://mail.example.org/inbox"),
            ]),
            None
        ));
        assert!(!origin_ok(
            &h(&[
                ("host", "mail.example.org"),
                ("referer", "https://evil.example/x"),
            ]),
            None
        ));
    }

    // t20 B8: with nothing to compare an Origin against, the same-site check can
    // attest nothing — so it must refuse rather than wave the request through.
    #[test]
    fn missing_host_fails_closed() {
        assert!(!origin_ok(&h(&[]), None));
        assert!(!origin_ok(&h(&[("origin", "https://evil.example")]), None));
        // Even a request whose Origin would have matched is refused: without a
        // Host there is no target authority to have matched *against*.
        assert!(!origin_ok(
            &h(&[("origin", "https://mail.example.org")]),
            None
        ));
    }

    // HTTP/2 carries the target authority in `:authority`, not a `Host` header;
    // hyper leaves it on the URI. Those requests must still be checkable.
    #[test]
    fn uri_authority_substitutes_for_host_on_h2() {
        assert!(origin_ok(
            &h(&[("origin", "https://mail.example.org")]),
            Some("mail.example.org")
        ));
        assert!(!origin_ok(
            &h(&[("origin", "https://evil.example")]),
            Some("mail.example.org")
        ));
        // A present Host still wins over the URI authority.
        assert!(origin_ok(
            &h(&[
                ("host", "mail.example.org"),
                ("origin", "https://mail.example.org"),
            ]),
            Some("internal.svc")
        ));
    }

    #[test]
    fn csrf_double_submit_requires_matching_pair() {
        assert!(csrf_double_submit_ok(&h(&[
            ("x-csrf-token", "abc"),
            ("cookie", "mw_session=s; mw_csrf=abc"),
        ])));
        assert!(!csrf_double_submit_ok(&h(&[
            ("x-csrf-token", "abc"),
            ("cookie", "mw_csrf=zzz"),
        ])));
        assert!(!csrf_double_submit_ok(&h(&[("cookie", "mw_csrf=abc")])));
    }

    #[test]
    fn idle_timeout_expires_the_session() {
        let g = SessionGuard::new();
        let cfg = HardeningConfig {
            idle_timeout: Duration::from_secs(100),
            absolute_timeout: Duration::from_secs(10_000),
            ..HardeningConfig::default()
        };
        let t0 = Instant::now();
        assert!(g.check_at("s", &cfg, t0).is_ok());
        // Within idle window: OK.
        assert!(g.check_at("s", &cfg, t0 + Duration::from_secs(50)).is_ok());
        // Idle beyond the window (measured from last activity at +50s): expired.
        assert_eq!(
            g.check_at("s", &cfg, t0 + Duration::from_secs(200)),
            Err(Expiry::Idle)
        );
    }

    #[test]
    fn absolute_timeout_expires_even_when_active() {
        let g = SessionGuard::new();
        let cfg = HardeningConfig {
            idle_timeout: Duration::from_secs(10_000),
            absolute_timeout: Duration::from_secs(100),
            ..HardeningConfig::default()
        };
        let t0 = Instant::now();
        assert!(g.check_at("s", &cfg, t0).is_ok());
        assert!(g.check_at("s", &cfg, t0 + Duration::from_secs(50)).is_ok());
        assert_eq!(
            g.check_at("s", &cfg, t0 + Duration::from_secs(150)),
            Err(Expiry::Absolute)
        );
    }

    #[test]
    fn rotate_preserves_creation_and_forgets_old() {
        let g = SessionGuard::new();
        let cfg = HardeningConfig::default();
        assert!(g.check("old", &cfg).is_ok());
        g.rotate("old", "new");
        // Old id is gone (treated as first-sight again), new id is tracked.
        assert!(g.check("new", &cfg).is_ok());
    }
}
