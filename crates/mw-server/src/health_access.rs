//! Who may reach `/healthz` and `/readyz`.
//!
//! One rule, applied before every other guard. A probe request is admitted when
//!
//! * the token matches (`MW_HEALTH_TOKEN`, sent as the `X-Health-Token` header
//!   or a `token` query parameter), in any mode; or
//! * `security.health.mode` is `open`; or
//! * the client address (the trusted-proxy-resolved one, see
//!   [`crate::scope_mw::proxy::client_ip`]) is loopback, private, link-local or
//!   unique-local (`internal-only`, the default); or additionally, in mode
//!   `allowlist`, inside one of `security.health.networks`.
//!
//! A request that is not admitted is answered exactly like an unknown path: its
//! path is rewritten to a name no route knows and it continues through the
//! ordinary stack, where the fallback produces the usual 404. Because the
//! rewritten path no longer equals `/healthz`, none of the protection layers'
//! probe exemptions (source bans, admin probe, protocol guard, sign-in geography,
//! request protection, client access) apply to it. Only an admitted request keeps
//! the exact path and with it those exemptions.
//!
//! Caveat: behind a reverse proxy that is not configured through
//! `MW_FORWARDED_MODE` and `MW_TRUSTED_PROXIES`, every request appears to come
//! from the proxy, which is usually a private address. The default rule then
//! admits the whole internet. See `docs/deploy/reverse-proxy.md`.
//!
//! A request without a peer address (an in-process synthetic request; production
//! always installs `ConnectInfo`) is admitted, as every other source guard does.
use crate::{AppState, scope_mw::proxy};
use axum::{
    extract::{Request, State},
    http::{Uri, uri::PathAndQuery},
    middleware::Next,
    response::Response,
};
use mw_store::Store;
use serde_json::Value;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

pub(crate) const PREFIX: &str = "security.health.";
/// The environment variable holding the optional monitor token.
pub(crate) const TOKEN_ENV: &str = "MW_HEALTH_TOKEN";
const TOKEN_HEADER: &str = "x-health-token";
const CACHE_TTL: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mode {
    InternalOnly,
    Allowlist,
    Open,
}

#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub mode: Mode,
    pub networks: Vec<proxy::Cidr>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::InternalOnly,
            networks: Vec::new(),
        }
    }
}
impl Config {
    fn parse(rows: Vec<crate::settings_registry::Effective>) -> Self {
        let mut config = Self::default();
        for row in rows {
            match row.key.strip_prefix(PREFIX) {
                Some("mode") => {
                    config.mode = match row.value.as_str() {
                        Some("allowlist") => Mode::Allowlist,
                        Some("open") => Mode::Open,
                        _ => Mode::InternalOnly,
                    }
                }
                Some("networks") => {
                    if let Value::Array(items) = &row.value {
                        config.networks = items
                            .iter()
                            .filter_map(Value::as_str)
                            .filter_map(proxy::Cidr::parse)
                            .collect();
                    }
                }
                _ => {}
            }
        }
        config
    }
}

/// Loopback, private (RFC 1918), link-local and IPv6 unique-local addresses.
pub(crate) fn is_internal(ip: IpAddr) -> bool {
    match ip.to_canonical() {
        IpAddr::V4(v4) => internal_v4(v4),
        IpAddr::V6(v6) => internal_v6(v6),
    }
}
fn internal_v4(ip: Ipv4Addr) -> bool {
    ip.is_loopback() || ip.is_private() || ip.is_link_local()
}
fn internal_v6(ip: Ipv6Addr) -> bool {
    let first = ip.segments()[0];
    ip.is_loopback() || (first & 0xfe00) == 0xfc00 || (first & 0xffc0) == 0xfe80
}

/// Request extension set on a probe request that was not admitted.
#[derive(Clone, Copy)]
pub(crate) struct Refused;

pub(crate) fn is_probe_path(path: &str) -> bool {
    matches!(path, "/healthz" | "/readyz")
}

/// The decision, separate from the request plumbing.
pub(crate) fn admits(config: &Config, ip: IpAddr, token_ok: bool) -> bool {
    token_ok
        || match config.mode {
            Mode::Open => true,
            Mode::InternalOnly => is_internal(ip),
            Mode::Allowlist => is_internal(ip) || config.networks.iter().any(|n| n.contains(ip)),
        }
}

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn token_ok(request: &Request) -> bool {
    let Some(want) = std::env::var(TOKEN_ENV).ok().filter(|t| !t.is_empty()) else {
        return false;
    };
    if let Some(got) = request
        .headers()
        .get(TOKEN_HEADER)
        .and_then(|v| v.to_str().ok())
        && ct_eq(got.trim().as_bytes(), want.as_bytes())
    {
        return true;
    }
    request.uri().query().is_some_and(|q| {
        q.split('&')
            .filter_map(|pair| pair.strip_prefix("token="))
            .any(|got| ct_eq(got.as_bytes(), want.as_bytes()))
    })
}

/// The `/healthz` route. A refused request gets the fallback's answer, the
/// same as any unknown path.
pub(crate) async fn healthz(
    State(state): State<AppState>,
    method: axum::http::Method,
    original: axum::extract::OriginalUri,
    uri: Uri,
    headers: axum::http::HeaderMap,
    refused: Option<axum::Extension<Refused>>,
) -> Response {
    use axum::response::IntoResponse;
    if refused.is_some() {
        return crate::static_handler(State(state), method, original, uri, headers).await;
    }
    "ok".into_response()
}

/// Outermost source guard: see the module docs.
pub(crate) async fn layer(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    if !is_probe_path(request.uri().path()) {
        return next.run(request).await;
    }
    let Some(ip) = proxy::client_ip(request.headers(), request.extensions()) else {
        return next.run(request).await;
    };
    let config = state.request_protection.health.get(&state.store).await;
    if admits(&config, ip, token_ok(&request)) {
        return next.run(request).await;
    }
    // Not admitted. Routing already happened (a router layer runs after the
    // route is chosen), so mark the request for the route handler, which
    // answers it with the fallback; and become an unknown path for every inner
    // layer (`/healthz/` is in no exemption list), query dropped so the token
    // is not carried further.
    request.extensions_mut().insert(Refused);
    let path = format!("{}/", request.uri().path());
    let mut parts = request.uri().clone().into_parts();
    if let Ok(pq) = path.parse::<PathAndQuery>() {
        parts.path_and_query = Some(pq);
        if let Ok(uri) = Uri::from_parts(parts) {
            *request.uri_mut() = uri;
        }
    }
    next.run(request).await
}

struct Cached {
    at: Instant,
    epoch: u64,
    value: Arc<Config>,
}
#[derive(Default)]
pub(crate) struct Cache {
    value: RwLock<Option<Cached>>,
    epoch: std::sync::atomic::AtomicU64,
}
impl Cache {
    pub fn invalidate(&self) {
        self.epoch
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    }
    /// A store failure yields the default (internal only), never "open".
    pub async fn get(&self, store: &Store) -> Arc<Config> {
        let epoch = self.epoch.load(std::sync::atomic::Ordering::Acquire);
        if let Some(cached) = self
            .value
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
            .filter(|c| c.epoch == epoch && c.at.elapsed() < CACHE_TTL)
        {
            return cached.value.clone();
        }
        let value = match crate::settings_registry::effective_all(store, &Default::default(), |def| {
            def.key.starts_with(PREFIX)
        })
        .await
        {
            Ok(rows) => Arc::new(Config::parse(rows)),
            Err(_) => Arc::new(Config::default()),
        };
        *self.value.write().unwrap_or_else(|p| p.into_inner()) = Some(Cached {
            at: Instant::now(),
            epoch,
            value: value.clone(),
        });
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn internal_ranges() {
        for a in [
            "127.0.0.1",
            "::1",
            "10.1.2.3",
            "172.16.0.1",
            "192.168.1.1",
            "169.254.1.1",
            "fd00::1",
            "fe80::1",
            "::ffff:10.0.0.1",
        ] {
            assert!(is_internal(ip(a)), "{a}");
        }
        for a in [
            "8.8.8.8",
            "172.32.0.1",
            "100.64.0.1",
            "2001:db8::1",
            "::ffff:8.8.8.8",
        ] {
            assert!(!is_internal(ip(a)), "{a}");
        }
    }

    #[test]
    fn modes() {
        let mut c = Config::default();
        assert!(!admits(&c, ip("8.8.8.8"), false));
        assert!(admits(&c, ip("8.8.8.8"), true));
        c.networks = vec![proxy::Cidr::parse("8.8.8.0/24").unwrap()];
        assert!(
            !admits(&c, ip("8.8.8.8"), false),
            "networks ignored in internal-only"
        );
        c.mode = Mode::Allowlist;
        assert!(admits(&c, ip("8.8.8.8"), false));
        assert!(admits(&c, ip("10.0.0.1"), false));
        assert!(!admits(&c, ip("9.9.9.9"), false));
        c.mode = Mode::Open;
        assert!(admits(&c, ip("9.9.9.9"), false));
    }
}
