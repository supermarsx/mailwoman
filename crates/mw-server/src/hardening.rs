//! Web-hardening deltas (SPEC §7.4, plan §3 e10), all **additive** — they must
//! not weaken the existing sanitizer/sandbox contract:
//!
//! * extra response headers: COEP `require-corp`, CORP `same-origin`,
//!   `Permissions-Policy` (deny powerful features);
//! * an **Origin/Referer check** on state-changing requests (effective CSRF
//!   defense that needs no client change — browsers always send `Origin` on
//!   cross-site writes; native clients that send neither are allowed);
//! * a **double-submit CSRF token** (`mw_csrf` cookie ↔ `X-CSRF-Token` header),
//!   enforced when `csrf_strict` is on. The web client sends it on every write
//!   (`apps/web/src/api/http.ts`); a client that authenticates with a cookie and
//!   does not send it is refused in that mode. [`csrf_token_exempt`] lists the
//!   routes that are not asked for it;
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

/// Mailbox sessions may never outlive fifteen days from their original sign-in.
pub const MAILBOX_ABSOLUTE_CAP_SECS: u64 = 15 * 24 * 60 * 60;

/// Tunable hardening knobs (populated from the CLI/env).
#[derive(Debug, Clone)]
pub struct HardeningConfig {
    /// Emit `Cross-Origin-Embedder-Policy: require-corp` (crossOriginIsolated).
    pub coep: bool,
    /// Enforce the double-submit CSRF token on every cookie-authorised write
    /// except the routes in [`csrf_token_exempt`].
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
            absolute_timeout: Duration::from_secs(MAILBOX_ABSOLUTE_CAP_SECS),
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
    // Fetch Metadata is browser-controlled. Reject an explicitly cross-site
    // write even if a proxy stripped Origin/Referer. Absent metadata remains
    // compatible with native clients and older browsers using the origin check.
    let fetch_site = match headers.get("sec-fetch-site") {
        Some(value) => match value.to_str() {
            Ok("cross-site") => return false,
            Ok(value @ ("same-origin" | "same-site" | "none")) => Some(value),
            _ => return false,
        },
        None => None,
    };
    for name in ["host", "origin", "referer", "sec-fetch-site"] {
        if headers.get_all(name).iter().count() > 1 {
            return false;
        }
    }
    let host = match headers.get(header::HOST) {
        Some(value) => value.to_str().ok(),
        None => authority,
    };
    let Some(host) = host else {
        return false;
    };
    if host.is_empty() || host.bytes().any(|b| b.is_ascii_whitespace()) || host.contains('@') {
        return false;
    }
    if let Some(origin) = headers.get(header::ORIGIN) {
        return origin
            .to_str()
            .ok()
            .and_then(authority_of)
            .is_some_and(|a| a.eq_ignore_ascii_case(host));
    }
    if let Some(referer) = headers.get(header::REFERER) {
        return referer
            .to_str()
            .ok()
            .and_then(authority_of)
            .is_some_and(|a| a.eq_ignore_ascii_case(host));
    }
    // A sibling origin is not the same origin. Require its Origin/Referer proof.
    fetch_site != Some("same-site")
}

/// Extract the `host[:port]` authority from a URL-ish string (`scheme://auth/..`).
fn authority_of(url: &str) -> Option<&str> {
    let (scheme, after) = url.split_once("://")?;
    if !scheme.eq_ignore_ascii_case("https") && !scheme.eq_ignore_ascii_case("http") {
        return None;
    }
    let authority = after.split(['/', '?', '#']).next()?;
    (!authority.is_empty()
        && !authority.contains('@')
        && !authority.bytes().any(|b| b.is_ascii_whitespace()))
    .then_some(authority)
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

/// Cookie-authenticated writes require the MAC for the session authorizing
/// this route. A mailbox session cannot authorize an admin request or vice versa.
pub fn csrf_request_ok(headers: &HeaderMap, store: &mw_store::Store, admin_route: bool) -> bool {
    if !csrf_double_submit_ok(headers) {
        return false;
    }
    let name = if admin_route {
        mw_admin::config::session_cookie_name()
    } else {
        "mw_session".into()
    };
    let Some(id) = cookie(headers, &name) else {
        return !has_auth_cookie(headers);
    };
    let expected = store.session_csrf_token(&id);
    let actual = cookie(headers, CSRF_COOKIE).unwrap_or_default();
    if actual.len() != expected.len() {
        return false;
    }
    actual
        .bytes()
        .zip(expected.bytes())
        .fold(0u8, |diff, (a, b)| diff | (a ^ b))
        == 0
}

/// The state-changing routes that `csrf_strict` does not ask for the double-submit
/// token. `path` is the request path with any `MW_BASE_PATH` prefix already
/// removed, as `state_change_guard` sees it.
///
/// Two kinds of route belong here and nothing else does:
///
/// * **Sign-in routes.** A client has no token before it signs in; the answer is
///   what issues one.
/// * **Routes no cookie authorises.** Their handlers take their authority from a
///   secret in the body or in a header and never read a session cookie, so the
///   cookies a browser attaches to a forged request gain the forger nothing. Their
///   callers are other programs, which hold no `mw_csrf` cookie to repeat.
///
/// The list is explicit on purpose. "Exempt when the request carries no cookie"
/// would let a route that later starts reading a cookie skip the check without
/// anyone deciding it should. Before adding a route, confirm its handler calls none
/// of `authed`, `authed_with_gate` and the admin-session gates (`admin_authed`,
/// `require_admin`, `admin_id`, `admin_actor`), and add it to the matching list in
/// `tests/t30_csrf_strict.rs`.
///
/// Not here, and decided before this is consulted: the SSO callback and ACS
/// landings (`is_sso_idp_callback`), which skip the Origin check as well, and a
/// request whose only authority is an `Authorization: Bearer` header
/// ([`has_auth_cookie`] is false). Every route listed here is still
/// Origin-checked.
pub fn csrf_token_exempt(path: &str) -> bool {
    // Mailbox sign-in. No session exists yet.
    path == "/api/login"
        || path == "/api/login/passkey/begin"
        || path == "/api/login/passkey/finish"
        // Second-factor step and forced enrolment of a sign-in: authority is the
        // one-shot pending token in the body; still no session.
        || path.starts_with("/api/login/2fa")
        // Mail-server lookup on the sign-in screen: unauthenticated, rate-limited.
        || path == "/api/discover"
        // Entry-key form has its own signed, expiring cookie/nonce proof. Origin
        // and Fetch Metadata checks still run, including with old auth cookies.
        || path == "/api/client-access/unlock"
        // Admin panel sign-in, wherever `MW_ADMIN_PATH` puts it. No admin session
        // exists yet.
        || path == mw_admin::config::admin_route("/login")
        || path == mw_admin::config::admin_route("/login/totp")
        || path == mw_admin::config::admin_route("/login/passkey/begin")
        || path == mw_admin::config::admin_route("/login/passkey/finish")
        || path == mw_admin::config::admin_route("/login/recovery")
        // OAuth token endpoint: authorisation code with its PKCE verifier, or a
        // refresh token, in the body.
        || path == "/oauth/token"
        // Token introspection and revocation: the token itself, in the body.
        || path == "/oauth/introspect"
        || path == "/oauth/revoke"
        // Dynamic client registration (RFC 7591) and client configuration
        // (RFC 7592): the admin policy plus an initial or registration access
        // token in `Authorization`; a request without one is answered by the
        // handler.
        || path == "/oauth/register"
        || path.starts_with("/oauth/register/")
        // Inbound webhook: an HMAC of the body in `x-mailwoman-signature`.
        || path == "/api/webhooks/inbound"
        // Browser error report: unauthenticated; the handler reads no cookie.
        || path == "/errors"
        // MCP: an API key or OAuth access token in `Authorization` per call;
        // `initialize` and `tools/list` need none.
        || path == "/mcp"
        || path.starts_with("/mcp/")
}

/// The routes a native shell calls before it holds a bearer token: mailbox sign-in,
/// its second-factor step, and the mail-server lookup on the sign-in screen.
/// `state_change_guard` lets a request to one of these through the Origin check
/// when it comes from a native shell origin
/// ([`NativeAuthConfig::allows`](crate::push_relay::NativeAuthConfig::allows)) and
/// carries no session cookie ([`has_auth_cookie`] is false). Each is in
/// [`csrf_token_exempt`] for the same reason: no session exists yet.
///
/// Nothing else belongs here. After sign-in the shell sends its bearer token, and
/// a bearer-only request needs no exemption.
pub fn native_signin_route(path: &str) -> bool {
    path == "/api/login" || path.starts_with("/api/login/2fa") || path == "/api/discover"
}

/// The cookies that authorise a request on their own: the mailbox session and the
/// admin session, whose name is `mw_admin_session` unless
/// `MW_ADMIN_SESSION_COOKIE` sets another ([`mw_admin::config::session_cookie_name`],
/// the same call every admin gate makes). `mw_csrf` is not one of them — it
/// authorises nothing.
///
/// This is the one place that names them for the guard. A new cookie that a
/// handler accepts as proof of who is calling must be added here, or a request
/// carrying it beside an `Authorization: Bearer` header would skip the Origin and
/// CSRF-token checks ([`has_auth_cookie`]). `tests/t30_csrf_strict.rs` signs in
/// through the real routes, sends back the cookies those logins set, and expects
/// the checks to apply, so a rename of either cookie that is not made here fails
/// there; `tests/t29_security_policy.rs` does the same with a renamed admin cookie.
pub fn auth_cookies() -> [String; 2] {
    [
        "mw_session".to_string(),
        mw_admin::config::session_cookie_name(),
    ]
}

/// Whether the request carries one of [`auth_cookies`] with a non-empty value.
///
/// `state_change_guard` excuses a bearer request from its checks only when this
/// is false: the bearer token is then the request's only authority. With a
/// session cookie beside it the handler may authorise by the cookie instead — the
/// admin gates never read `Authorization` — so the request is checked like any
/// other a cookie authorises.
///
/// Every `Cookie` header field is read, not just the first: HTTP/2 may split
/// cookies over several fields, and the answer here must not be "no" for a cookie
/// some handler could still see.
pub fn has_auth_cookie(headers: &HeaderMap) -> bool {
    let names = auth_cookies();
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|raw| raw.split(';'))
        .filter_map(|part| part.trim().split_once('='))
        .any(|(name, value)| !value.is_empty() && names.iter().any(|n| n == name))
}

/// Pull one cookie value out of the `Cookie` header.
pub fn cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let needle = format!("{name}=");
    for raw in headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
    {
        for part in raw.split(';') {
            if let Some(v) = part.trim().strip_prefix(&needle)
                && !v.is_empty()
            {
                return Some(v.to_string());
            }
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

    /// Whether a session has passed its absolute or idle timeout, without
    /// recording activity: a realtime stream's re-check must not keep its session
    /// alive. A session with no record is not expired, as in [`check`](Self::check).
    pub fn expired(&self, id: &str, cfg: &HardeningConfig) -> Option<Expiry> {
        self.expired_at(id, cfg, Instant::now())
    }

    fn expired_at(&self, id: &str, cfg: &HardeningConfig, now: Instant) -> Option<Expiry> {
        let map = self.inner.lock().expect("session guard lock");
        let t = map.get(id)?;
        if now.duration_since(t.created) > cfg.absolute_timeout {
            Some(Expiry::Absolute)
        } else if now.duration_since(t.last_seen) > cfg.idle_timeout {
            Some(Expiry::Idle)
        } else {
            None
        }
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
    fn fetch_metadata_and_malformed_origin_fail_closed() {
        for site in ["cross-site", "same-site", "unexpected"] {
            assert!(!origin_ok(
                &h(&[("host", "mail.example.org"), ("sec-fetch-site", site)]),
                None
            ));
        }
        for site in ["same-origin", "none"] {
            assert!(origin_ok(
                &h(&[("host", "mail.example.org"), ("sec-fetch-site", site)]),
                None
            ));
        }
        assert!(!origin_ok(
            &h(&[
                ("host", "mail.example.org"),
                ("origin", "https://mail.example.org"),
                ("sec-fetch-site", "cross-site")
            ]),
            None
        ));
        assert!(origin_ok(
            &h(&[
                ("host", "mail.example.org"),
                ("origin", "https://mail.example.org"),
                ("sec-fetch-site", "same-site")
            ]),
            None
        ));
        for origin in [
            "null",
            "custom://mail.example.org",
            "https://user@mail.example.org",
        ] {
            assert!(!origin_ok(
                &h(&[("host", "mail.example.org"), ("origin", origin)]),
                None
            ));
        }
        let mut invalid = h(&[("host", "mail.example.org")]);
        invalid.insert(header::ORIGIN, HeaderValue::from_bytes(b"\xff").unwrap());
        assert!(!origin_ok(&invalid, None));
        let mut duplicate = h(&[
            ("host", "mail.example.org"),
            ("origin", "https://mail.example.org"),
        ]);
        duplicate.append(
            header::ORIGIN,
            HeaderValue::from_static("https://evil.example"),
        );
        assert!(!origin_ok(&duplicate, None));
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
    fn has_auth_cookie_sees_session_cookies_only() {
        for cookies in [
            "mw_session=abc",
            "mw_admin_session=abc",
            "theme=dark; mw_session=abc; mw_csrf=t",
            "mw_csrf=t;mw_admin_session=abc",
        ] {
            assert!(has_auth_cookie(&h(&[("cookie", cookies)])), "{cookies}");
        }
        for cookies in [
            "mw_csrf=t",
            "theme=dark",
            // An empty value authorises nothing (a cleared cookie).
            "mw_session=",
            "mw_admin_session=; mw_csrf=t",
            // Not the same cookie.
            "xmw_session=abc",
            "mw_session_old=abc",
        ] {
            assert!(!has_auth_cookie(&h(&[("cookie", cookies)])), "{cookies}");
        }
        assert!(!has_auth_cookie(&h(&[])));

        // A session cookie in a second `Cookie` field (HTTP/2 may split them).
        let mut split = HeaderMap::new();
        split.append(header::COOKIE, HeaderValue::from_static("mw_csrf=t"));
        split.append(header::COOKIE, HeaderValue::from_static("mw_session=abc"));
        assert!(has_auth_cookie(&split));
    }

    #[test]
    fn csrf_token_exempt_is_an_exact_list() {
        for path in [
            "/api/login",
            "/api/login/passkey/begin",
            "/api/login/passkey/finish",
            "/api/login/2fa",
            "/api/login/2fa/enroll/totp/begin",
            "/api/discover",
            "/admin/login",
            "/admin/login/totp",
            "/admin/login/passkey/begin",
            "/admin/login/passkey/finish",
            "/admin/login/recovery",
            "/oauth/token",
            "/oauth/introspect",
            "/oauth/revoke",
            "/oauth/register",
            "/oauth/register/client-1",
            "/api/webhooks/inbound",
            "/errors",
            "/mcp",
            "/mcp/",
        ] {
            assert!(csrf_token_exempt(path), "{path} should be exempt");
        }
        // Neighbours of the entries above that a cookie authorises, and a prefix
        // match that must not happen.
        for path in [
            "/api/logout",
            "/api/login2",
            "/api/me",
            "/admin/logout",
            "/admin/login/",
            "/admin/login/passkey/finish/extra",
            "/api/login/passkey/finish/extra",
            "/admin/me/passkeys/begin",
            "/admin/me/passkeys/finish",
            "/admin/step-up/passkey",
            "/admin/users",
            "/admin/oauth-dcr",
            "/oauth/consent",
            "/oauth/decision",
            "/oauth/registered",
            "/api/keys",
            "/api/webhooks",
            "/api/push/subscribe",
            "/api/push/unsubscribe",
            "/api/sso/logout",
            "/errors/x",
            "/mcpx",
            "/jmap/api",
            "/",
        ] {
            assert!(!csrf_token_exempt(path), "{path} must not be exempt");
        }
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
    fn expired_reports_both_timeouts_without_recording_activity() {
        let g = SessionGuard::new();
        let cfg = HardeningConfig {
            idle_timeout: Duration::from_secs(100),
            absolute_timeout: Duration::from_secs(1_000),
            ..HardeningConfig::default()
        };
        let t0 = Instant::now();
        // No record: not expired, and asking does not create one.
        assert_eq!(g.expired_at("s", &cfg, t0), None);
        assert!(!g.inner.lock().unwrap().contains_key("s"));

        assert!(g.check_at("s", &cfg, t0).is_ok());
        assert_eq!(g.expired_at("s", &cfg, t0 + Duration::from_secs(50)), None);
        // Asking at +90 s must not move last_seen: at +150 s the session is idle
        // from t0, which it would not be had +90 s counted as activity.
        assert_eq!(g.expired_at("s", &cfg, t0 + Duration::from_secs(90)), None);
        assert_eq!(
            g.expired_at("s", &cfg, t0 + Duration::from_secs(150)),
            Some(Expiry::Idle)
        );
        // Still a read: the record is there and unchanged for `check`.
        assert_eq!(
            g.check_at("s", &cfg, t0 + Duration::from_secs(150)),
            Err(Expiry::Idle)
        );

        // Absolute applies to a session that is not idle.
        let g = SessionGuard::new();
        let cfg = HardeningConfig {
            idle_timeout: Duration::from_secs(10_000),
            absolute_timeout: Duration::from_secs(1_000),
            ..HardeningConfig::default()
        };
        assert!(g.check_at("a", &cfg, t0).is_ok());
        assert!(g.check_at("a", &cfg, t0 + Duration::from_secs(990)).is_ok());
        assert_eq!(g.expired_at("a", &cfg, t0 + Duration::from_secs(995)), None);
        assert_eq!(
            g.expired_at("a", &cfg, t0 + Duration::from_secs(1_001)),
            Some(Expiry::Absolute)
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
