//! Admin-probe protection (t100): slow down and then block a source that keeps
//! looking for the admin console, with room for honest mistakes.
//!
//! Four signals, per client source (the trusted-proxy client address; IPv6 per
//! /64), each worth one point:
//!
//! * `probe-path` — a path on the probe list (`/wp-admin`, `/.env`, `/admin` once
//!   the console has moved, …), anywhere outside the console;
//! * `slug-guess` — a path at the console's depth that answered 404 (static-file
//!   names such as `favicon.ico` and the server's own prefixes do not count);
//! * `admin-signin-failed` — a refused admin sign-in at the real console path;
//! * `admin-unauthenticated` — an admin API call answered 401 (one point per
//!   distinct session cookie, so an expired console tab costs one point).
//!
//! The score drains continuously at `graceAllowance / windowSecs` points a second,
//! so the first `graceAllowance` misses in a window do nothing. Above that, each
//! further miss is answered after `tarpitDelayMs`. At `graceAllowance +
//! blockAfter` the source is blocked for `blockSecs`, doubled for each block within
//! `repeatWindowSecs` of the previous one, up to `maxBlockSecs`. While blocked,
//! every request from the source gets the same answer whatever its path, the
//! console included, so a block says nothing about where the console is.
//!
//! Never counted: allowlisted networks, a configured trusted proxy itself (a
//! request without a forwarded client address), and requests without a peer
//! address. A successful admin sign-in clears its source.
//!
//! State is per process: a bounded table ([`CAPACITY`] sources), plus active blocks
//! saved to the store and read back at start. Several instances each keep their
//! own scores; a block is only seen by the instance that made it (and by every
//! instance after a restart).
mod api;
mod config;
mod tracker;

pub(crate) use api::register;

use crate::AppState;
use crate::audit::{SecurityEvent, security_event};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use config::{AuditIp, Policy};
use serde_json::json;
use std::collections::VecDeque;
use std::net::IpAddr;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
use tracker::{Outcome, Signal, Tracker};

/// The admin route that owns the `security.adminProbe.*` keys.
pub(crate) const ROUTE: &str = "/security/admin-probe";
/// Sources kept in memory.
pub(crate) const CAPACITY: usize = 10_000;
/// The `settings` row holding active blocks across a restart.
const BLOCKS_KEY: &str = "adminProbe.blocks";
/// Active blocks saved at most.
const PERSIST_MAX: usize = 1_000;

/// The fail2ban `failregex` for the line written when a source is blocked
/// (`deploy/fail2ban/filter.d/mailwoman-admin-probe.conf` carries it verbatim).
// Read by the filter-file test; the filter itself lives in deploy/.
#[cfg_attr(not(test), allow(dead_code))]
pub const FAIL2BAN_PROBE_FAILREGEX: &str =
    r"mailwoman\[admin-probe\]: source blocked; .*rhost=<HOST>";

/// One fail2ban line. `rhost` is the full client address: a jail bans addresses.
pub fn fail2ban_probe_line(
    ts: chrono::DateTime<chrono::Utc>,
    signal: &str,
    strike: u32,
    block_secs: u64,
    ip: IpAddr,
) -> String {
    format!(
        "{} mailwoman[admin-probe]: source blocked; signal={signal} strike={strike} blockSecs={block_secs} rhost={}",
        ts.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        ip.to_canonical()
    )
}

#[derive(Default)]
struct Stats {
    blocks: AtomicU64,
    refused: AtomicU64,
    tarpitted: AtomicU64,
    cleared: AtomicU64,
    signals: [AtomicU64; 4],
    /// Block start times (ms), the last 24 h at most [`CAPACITY`] long.
    recent: Mutex<VecDeque<u64>>,
}

type Clock = Arc<dyn Fn() -> u64 + Send + Sync>;

pub(crate) struct Runtime {
    tracker: Mutex<Tracker>,
    config: config::Cache,
    stats: Stats,
    clock: Clock,
    /// Hashes a source into the opaque id the API uses.
    salt: [u8; 16],
    started_ms: u64,
    persist: tokio::sync::Mutex<()>,
}

fn wall_ms() -> u64 {
    u64::try_from(chrono::Utc::now().timestamp_millis()).unwrap_or(0)
}

impl Runtime {
    fn with_clock(clock: Clock) -> Self {
        let mut salt = [0u8; 16];
        salt.copy_from_slice(&uuid::Uuid::new_v4().as_bytes()[..]);
        let started_ms = clock();
        Self {
            tracker: Mutex::new(Tracker::new(CAPACITY)),
            config: config::Cache::default(),
            stats: Stats::default(),
            clock,
            salt,
            started_ms,
            persist: tokio::sync::Mutex::new(()),
        }
    }

    /// Start with the wall clock and read back the blocks saved by the last run.
    pub fn start(store: mw_store::Store) -> Arc<Self> {
        let runtime = Arc::new(Self::with_clock(Arc::new(wall_ms)));
        let weak = Arc::downgrade(&runtime);
        tokio::spawn(async move {
            let Some(runtime) = weak.upgrade() else {
                return;
            };
            runtime.restore(&store).await;
        });
        runtime
    }

    fn now(&self) -> u64 {
        (self.clock)()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Tracker> {
        self.tracker.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub fn invalidate(&self) {
        self.config.invalidate();
    }

    /// The opaque id of a source key (stable for this process).
    fn id_of(&self, key: IpAddr) -> String {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(self.salt);
        h.update(key.to_string().as_bytes());
        h.finalize()[..16]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    async fn restore(&self, store: &mw_store::Store) {
        let Ok(policy) = self.config.get(store).await else {
            return;
        };
        let Ok(Some(raw)) = store.get_setting(BLOCKS_KEY).await else {
            return;
        };
        let Ok(rows) = serde_json::from_str::<Vec<Persisted>>(&raw) else {
            return;
        };
        let now = self.now();
        let mut t = self.lock();
        for row in rows.into_iter().take(PERSIST_MAX) {
            if let Ok(ip) = row.source.parse::<IpAddr>() {
                t.restore(
                    tracker::source_key(ip),
                    row.until_ms,
                    row.strikes,
                    now,
                    &policy.limits,
                );
            }
        }
    }

    async fn save(&self, store: &mw_store::Store, limits: &tracker::Limits) {
        let _one = self.persist.lock().await;
        let now = self.now();
        let rows: Vec<Persisted> = {
            let mut t = self.lock();
            let mut rows: Vec<Persisted> = t
                .snapshot(now, limits)
                .into_iter()
                .filter_map(|(key, entry, _)| {
                    entry.blocked_until_ms.map(|until| Persisted {
                        source: key.to_string(),
                        until_ms: until,
                        strikes: entry.strikes,
                    })
                })
                .collect();
            rows.sort_by_key(|r| std::cmp::Reverse(r.until_ms));
            rows.truncate(PERSIST_MAX);
            rows
        };
        let body = serde_json::to_string(&rows).unwrap_or_else(|_| "[]".into());
        if let Err(e) = store.set_setting(BLOCKS_KEY, &body).await {
            tracing::warn!(target: "admin_probe", "active admin-probe blocks not saved: {e}");
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Persisted {
    source: String,
    until_ms: u64,
    strikes: u32,
}

/// What a source looks like in the audit log and the API.
pub(crate) fn display_source(key: IpAddr, mode: AuditIp) -> String {
    match (mode, key) {
        (AuditIp::Full, IpAddr::V4(v4)) => v4.to_string(),
        (AuditIp::Full, IpAddr::V6(v6)) => format!("{v6}/64"),
        (AuditIp::Masked, IpAddr::V4(v4)) => {
            let o = v4.octets();
            format!("{}.{}.{}.0/24", o[0], o[1], o[2])
        }
        (AuditIp::Masked, IpAddr::V6(v6)) => {
            let s = v6.segments();
            format!("{:x}:{:x}:{:x}::/48", s[0], s[1], s[2])
        }
    }
}

/// Whether `path` contains an entry of the **default** probe list
/// (`security.adminProbe.probePaths` as shipped) as whole segments, any case.
/// For callers without the live policy, such as request-history labelling
/// (t102's `not_found::probe_shaped`); enforcement uses the configured list.
// The shared matcher offered to t102 (request-history labelling).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn default_probe_path(path: &str) -> bool {
    static LIST: std::sync::LazyLock<Vec<Vec<String>>> = std::sync::LazyLock::new(|| {
        crate::settings_registry::lookup("security.adminProbe.probePaths")
            .and_then(|d| d.default_value().as_array().cloned())
            .unwrap_or_default()
            .iter()
            .filter_map(|v| v.as_str())
            .map(|p| config::segments(&p.to_ascii_lowercase()))
            .filter(|s| !s.is_empty())
            .collect()
    });
    let segs = config::segments(&path.to_ascii_lowercase());
    LIST.iter().any(|probe| {
        probe.len() <= segs.len() && segs.windows(probe.len()).any(|w| w == probe.as_slice())
    })
}

// ─── classification ─────────────────────────────────────────────────────────────

/// Final-segment extensions a browser or crawler asks for on its own.
const INNOCENT_EXTENSIONS: &[&str] = &[
    "ico",
    "png",
    "jpg",
    "jpeg",
    "gif",
    "svg",
    "webp",
    "avif",
    "txt",
    "xml",
    "json",
    "webmanifest",
    "css",
    "js",
    "mjs",
    "map",
    "woff",
    "woff2",
    "ttf",
    "otf",
];

#[derive(Debug, PartialEq, Eq)]
enum Shape {
    /// `POST {console}/login…`.
    Signin {
        cookie: String,
    },
    /// Under the console, not a sign-in step.
    ConsoleApi {
        cookie: Option<u64>,
    },
    /// On the probe list.
    Probe,
    /// One 404 away from a slug guess.
    SlugCandidate,
    Other,
}

#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    Signal(Signal, Option<u64>),
    Success,
}

fn cookie_value<'h>(headers: &'h HeaderMap, name: &str) -> Option<&'h str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(k, v)| *k == name && !v.is_empty())
        .map(|(_, v)| v)
}

fn fingerprint(value: &str) -> u64 {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(value.as_bytes());
    u64::from_le_bytes(digest[..8].try_into().expect("8 bytes"))
}

fn shape(method: &Method, path: &str, headers: &HeaderMap, policy: &Policy) -> Shape {
    if method == Method::OPTIONS {
        return Shape::Other;
    }
    let console = mw_admin::config::admin_path();
    let suffix = if path == console {
        Some("")
    } else {
        path.strip_prefix(console.as_str())
            .filter(|rest| rest.starts_with('/'))
    };
    if let Some(suffix) = suffix {
        let cookie_name = mw_admin::config::session_cookie_name();
        if method == Method::POST && (suffix == "/login" || suffix.starts_with("/login/")) {
            return Shape::Signin {
                cookie: cookie_name,
            };
        }
        // The page itself, sign-out and the "am I signed in?" check are what an
        // admin's own browser sends while signed out.
        if suffix.is_empty() || suffix == "/logout" || suffix == "/session" {
            return Shape::Other;
        }
        return Shape::ConsoleApi {
            cookie: cookie_value(headers, &cookie_name).map(fingerprint),
        };
    }
    let lower = path.to_ascii_lowercase();
    let segs = config::segments(&lower);
    let Some(first) = segs.first() else {
        return Shape::Other;
    };
    if mw_admin::config::RESERVED_ADMIN_SEGMENTS
        .iter()
        .any(|r| r.eq_ignore_ascii_case(first))
    {
        return Shape::Other;
    }
    if policy.is_probe(&segs) {
        return Shape::Probe;
    }
    let depth = config::segments(&console).len();
    let innocent = segs.last().is_some_and(|last| {
        last.rsplit_once('.')
            .is_some_and(|(_, ext)| INNOCENT_EXTENSIONS.contains(&ext))
    });
    if segs.len() == depth
        && !innocent
        && matches!(*method, Method::GET | Method::HEAD | Method::POST)
    {
        return Shape::SlugCandidate;
    }
    Shape::Other
}

fn sets_session(response: &Response, name: &str) -> bool {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .any(|v| {
            let mut parts = v.split(';');
            let first = parts.next().unwrap_or_default().trim();
            let clears = parts.any(|p| p.trim().eq_ignore_ascii_case("max-age=0"));
            first
                .split_once('=')
                .is_some_and(|(k, val)| k == name && !val.is_empty())
                && !clears
        })
}

fn verdict(shape: &Shape, response: &Response) -> Option<Verdict> {
    let status = response.status();
    match shape {
        Shape::Signin { cookie } => {
            if status == StatusCode::UNAUTHORIZED || status == StatusCode::TOO_MANY_REQUESTS {
                Some(Verdict::Signal(Signal::SigninFailed, None))
            } else if status.is_success() && sets_session(response, cookie) {
                Some(Verdict::Success)
            } else {
                None
            }
        }
        Shape::ConsoleApi { cookie } => (status == StatusCode::UNAUTHORIZED)
            .then_some(Verdict::Signal(Signal::Unauthenticated, *cookie)),
        Shape::Probe => Some(Verdict::Signal(Signal::ProbePath, None)),
        Shape::SlugCandidate => {
            (status == StatusCode::NOT_FOUND).then_some(Verdict::Signal(Signal::SlugGuess, None))
        }
        Shape::Other => None,
    }
}

// ─── responses ──────────────────────────────────────────────────────────────────

/// The answer to a blocked source: the same for every path and method. The body
/// is the one a deployment IP ban gives, so it does not even say why.
fn blocked(policy: &Policy, until_ms: u64, now_ms: u64) -> Response {
    let status = StatusCode::from_u16(policy.block_status).unwrap_or(StatusCode::TOO_MANY_REQUESTS);
    let retry = until_ms.saturating_sub(now_ms).div_ceil(1000).max(1);
    let mut response = (
        status,
        axum::Json(json!({"code":"sourceUnavailable","error":"Request unavailable."})),
    )
        .into_response();
    let h = response.headers_mut();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert(header::RETRY_AFTER, HeaderValue::from(retry));
    response
}

/// The router layer. Sits outside the other source guards and inside the
/// security headers, so a blocked answer carries the same headers everywhere.
pub(crate) async fn guard(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let runtime = state.admin_probe.clone();
    let Some(ip) = crate::scope_mw::proxy::client_ip(request.headers(), request.extensions())
    else {
        return next.run(request).await;
    };
    let policy = match runtime.config.get(&state.store).await {
        Ok(policy) => policy,
        // Storage trouble never takes the server down; the last policy (if any) holds.
        Err(()) => match runtime.config.last() {
            Some(policy) => policy,
            None => return next.run(request).await,
        },
    };
    if !policy.enabled
        || policy.allowlisted(ip)
        || crate::scope_mw::proxy::ProxyConfig::from_env().is_trusted(ip.to_canonical())
    {
        return next.run(request).await;
    }
    let key = tracker::source_key(ip);
    let now = runtime.now();
    if let Some(until) = runtime.lock().blocked_until(key, now, &policy.limits) {
        runtime.stats.refused.fetch_add(1, Ordering::Relaxed);
        return blocked(&policy, until, now);
    }
    let shape = shape(
        request.method(),
        request.uri().path(),
        request.headers(),
        &policy,
    );
    if shape == Shape::Other {
        return next.run(request).await;
    }
    let response = next.run(request).await;
    let Some(verdict) = verdict(&shape, &response) else {
        return response;
    };
    let (signal, cookie) = match verdict {
        Verdict::Success => {
            if runtime.lock().clear(key) {
                runtime.stats.cleared.fetch_add(1, Ordering::Relaxed);
            }
            return response;
        }
        Verdict::Signal(signal, cookie) => (signal, cookie),
    };
    let now = runtime.now();
    let outcome = runtime
        .lock()
        .record(key, signal, cookie, now, &policy.limits);
    if outcome.is_some() {
        runtime.stats.signals[signal as usize].fetch_add(1, Ordering::Relaxed);
    }
    match outcome {
        None | Some(Outcome::Grace { .. }) => response,
        Some(Outcome::Tarpit { .. }) => {
            runtime.stats.tarpitted.fetch_add(1, Ordering::Relaxed);
            if policy.tarpit_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(u64::from(
                    policy.tarpit_ms,
                )))
                .await;
            }
            response
        }
        Some(Outcome::AlreadyBlocked { until_ms }) => {
            drop(response);
            blocked(&policy, until_ms, now)
        }
        Some(Outcome::Blocked { until_ms, strike }) => {
            drop(response);
            on_block(&state, &policy, ip, key, signal, strike, until_ms, now).await;
            blocked(&policy, until_ms, now)
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn on_block(
    state: &AppState,
    policy: &Policy,
    ip: IpAddr,
    key: IpAddr,
    signal: Signal,
    strike: u32,
    until_ms: u64,
    now_ms: u64,
) {
    let runtime = &state.admin_probe;
    runtime.stats.blocks.fetch_add(1, Ordering::Relaxed);
    {
        let mut recent = runtime
            .stats
            .recent
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        while recent
            .front()
            .is_some_and(|t| now_ms.saturating_sub(*t) > 86_400_000)
            || recent.len() >= CAPACITY
        {
            recent.pop_front();
        }
        recent.push_back(now_ms);
    }
    let block_secs = until_ms.saturating_sub(now_ms).div_ceil(1000);
    if policy.fail2ban {
        tracing::warn!(
            "{}",
            fail2ban_probe_line(chrono::Utc::now(), signal.as_str(), strike, block_secs, ip)
        );
    }
    let id = runtime.id_of(key);
    security_event(
        state,
        SecurityEvent::system("admin-probe-blocked")
            .target(format!("source:{id}"))
            .ip((policy.audit_ip == AuditIp::Full).then_some(ip.to_canonical()))
            .detail(json!({
                "source": display_source(key, policy.audit_ip),
                "sourceId": id,
                "signal": signal.as_str(),
                "strike": strike,
                "blockSecs": block_secs,
                "until": iso(until_ms),
            })),
    )
    .await;
    runtime.save(&state.store, &policy.limits).await;
}

fn iso(ms: u64) -> Option<String> {
    chrono::DateTime::from_timestamp_millis(i64::try_from(ms).ok()?).map(|t| t.to_rfc3339())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn policy() -> Policy {
        let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        config::Cache::default()
            .get(&store)
            .await
            .unwrap()
            .as_ref()
            .clone()
    }

    #[tokio::test]
    async fn shapes_follow_the_console_path_and_ignore_innocent_files() {
        let p = policy().await;
        let h = HeaderMap::new();
        let console = mw_admin::config::admin_path();
        let s = |m: Method, path: &str| shape(&m, path, &h, &p);
        assert!(matches!(
            s(Method::POST, &format!("{console}/login")),
            Shape::Signin { .. }
        ));
        assert!(matches!(
            s(Method::POST, &format!("{console}/login/totp")),
            Shape::Signin { .. }
        ));
        assert_eq!(s(Method::GET, &format!("{console}/session")), Shape::Other);
        assert_eq!(s(Method::GET, &console), Shape::Other);
        assert_eq!(
            s(Method::GET, &format!("{console}/users")),
            Shape::ConsoleApi { cookie: None }
        );
        assert_eq!(s(Method::GET, "/wp-admin/setup.php"), Shape::Probe);
        assert_eq!(s(Method::GET, "/.env"), Shape::Probe);
        assert_eq!(s(Method::GET, "/favicon.ico"), Shape::Other);
        assert_eq!(s(Method::GET, "/apple-touch-icon-120.png"), Shape::Other);
        assert_eq!(s(Method::GET, "/api/nothing"), Shape::Other);
        assert_eq!(s(Method::OPTIONS, "/wp-admin"), Shape::Other);
        let depth = config::segments(&console).len();
        let guess = format!("/{}", vec!["guess"; depth].join("/"));
        assert_eq!(s(Method::GET, &guess), Shape::SlugCandidate);
        assert_eq!(s(Method::GET, &format!("{guess}/deeper")), Shape::Other);
    }

    #[test]
    fn the_default_list_is_usable_without_a_policy() {
        assert!(default_probe_path("/wp-admin/install.php"));
        assert!(default_probe_path("/blog/WP-LOGIN.PHP"));
        assert!(!default_probe_path("/inbox"));
        assert!(!default_probe_path("/wp-administration"));
    }

    #[test]
    fn masked_sources_hide_the_host_part() {
        let v4 = tracker::source_key("203.0.113.77".parse().unwrap());
        let v6 = tracker::source_key("2001:db8:aa:bb:1::7".parse().unwrap());
        assert_eq!(display_source(v4, AuditIp::Masked), "203.0.113.0/24");
        assert_eq!(display_source(v6, AuditIp::Masked), "2001:db8:aa::/48");
        assert_eq!(display_source(v4, AuditIp::Full), "203.0.113.77");
        assert_eq!(display_source(v6, AuditIp::Full), "2001:db8:aa:bb::/64");
    }

    #[test]
    fn the_fail2ban_line_has_one_rhost_last() {
        let line = fail2ban_probe_line(
            chrono::Utc::now(),
            "probe-path",
            2,
            1800,
            "::ffff:198.51.100.4".parse().unwrap(),
        );
        assert!(line.ends_with("rhost=198.51.100.4"), "{line}");
        assert_eq!(line.matches("rhost=").count(), 1);
        // The filter's literal part, unescaped, then `.*rhost=<HOST>`.
        let literal = FAIL2BAN_PROBE_FAILREGEX
            .split(" .*rhost=<HOST>")
            .next()
            .unwrap()
            .replace('\\', "");
        assert!(line.contains(&literal), "{line} / {literal}");
        assert_eq!(line.rsplit("rhost=").next(), Some("198.51.100.4"));
    }

    #[test]
    fn the_shipped_filter_carries_the_failregex_in_code() {
        let filter = include_str!("../../../deploy/fail2ban/filter.d/mailwoman-admin-probe.conf");
        let lines: Vec<&str> = filter
            .lines()
            .filter_map(|l| l.strip_prefix("failregex"))
            .map(|rest| rest.trim_start().trim_start_matches('=').trim())
            .collect();
        assert_eq!(lines, [FAIL2BAN_PROBE_FAILREGEX]);
        let jail = include_str!("../../../deploy/fail2ban/jail.d/mailwoman.conf");
        assert!(jail.contains("[mailwoman-admin-probe]"));
        assert!(jail.contains("filter   = mailwoman-admin-probe"));
    }

    #[test]
    fn a_session_cookie_counts_only_when_it_is_set_not_cleared() {
        let with = |v: &str| {
            let mut r = StatusCode::OK.into_response();
            r.headers_mut()
                .append(header::SET_COOKIE, HeaderValue::from_str(v).unwrap());
            r
        };
        assert!(sets_session(
            &with("mw_admin_session=abc; Path=/x; HttpOnly"),
            "mw_admin_session"
        ));
        assert!(!sets_session(
            &with("mw_admin_session=; Max-Age=0"),
            "mw_admin_session"
        ));
        assert!(!sets_session(
            &with("mw_admin_session=abc; Max-Age=0"),
            "mw_admin_session"
        ));
        assert!(!sets_session(&with("other=abc"), "mw_admin_session"));
    }
}
