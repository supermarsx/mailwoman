//! Reverse-proxy posture: one start-up check of the proxy settings and a runtime
//! warning when the server looks like it sits behind a proxy it does not trust
//! (t56).
//!
//! How the client address is decided is unchanged: `scope_mw::proxy::client_ip`
//! still resolves through `mw_protection::proxy::resolve`. This module only
//! validates and warns.
//!
//! Start-up ([`validate_startup`], called from `main.rs` next to the t34 mode
//! check): `MW_FORWARDED_MODE`, `MW_PROXY_PROTOCOL`, `MW_TRUSTED_PROXIES` and
//! `MW_HEADER_AUTH_TRUSTED_IPS` must parse. The runtime parsers drop what they
//! cannot read (an unknown mode is `off`, a malformed network is skipped), which
//! is the safe reading of a typo but leaves the operator with a setting that
//! silently does nothing; here the same values stop `serve` with an error naming
//! the variable. Combinations that parse but have no effect, or that widen trust,
//! are logged as warnings:
//! * a forwarded mode with no trusted proxy (headers stay ignored);
//! * trusted proxies with both the forwarded mode and PROXY protocol off (the list
//!   is used for `X-Forwarded-Proto`/`X-Forwarded-Host` only; the client address
//!   stays the proxy's);
//! * a range rather than a single address in `MW_TRUSTED_PROXIES` while PROXY
//!   protocol is on (every host in the range can declare any client address,
//!   including one in `MW_HEADER_AUTH_TRUSTED_IPS`; see
//!   `header_auth_peer_allowed` in `lib.rs`);
//! * a range in `MW_HEADER_AUTH_TRUSTED_IPS` while `MW_HEADER_AUTH` is on (every
//!   host in the range can assert an identity).
//!
//! Runtime ([`observe`], called from `client_ip`): a request that carries
//! `X-Forwarded-For` or `Forwarded` from a private, loopback, link-local or
//! shared-address peer whose header is NOT used — the forwarded mode is off, or the
//! peer is not in `MW_TRUSTED_PROXIES` — means every client behind that proxy is
//! seen as the proxy's address. One warning is logged, then at most one per hour,
//! and [`forwarded_headers_ignored`] turns true for the admin diagnostics.
//! Nothing in the header is believed: the observation only reports it exists.
use axum::http::HeaderMap;
use mw_protection::proxy::{
    Cidr, FORWARDED_MODE_ENV, ForwardedMode, ProxyConfig, TRUSTED_PROXIES_ENV,
};
use std::net::IpAddr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const PROXY_PROTOCOL_ENV: &str = "MW_PROXY_PROTOCOL";
const HEADER_AUTH_ENV: &str = "MW_HEADER_AUTH";
const HEADER_AUTH_TRUSTED_ENV: &str = "MW_HEADER_AUTH_TRUSTED_IPS";

/// Minimum spacing between two runtime warnings after the first.
pub(crate) const WARN_INTERVAL: Duration = Duration::from_secs(60 * 60);

// ---- start-up -------------------------------------------------------------

fn parse_forwarded_mode(raw: &str) -> Result<ForwardedMode, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "off" => Ok(ForwardedMode::Off),
        "xff" | "x-forwarded-for" => Ok(ForwardedMode::Xff),
        "forwarded" => Ok(ForwardedMode::Forwarded),
        _ => Err(format!(
            "{FORWARDED_MODE_ENV}={raw:?} is not one of off|xff|forwarded"
        )),
    }
}

/// `true` when PROXY protocol is accepted or required.
fn parse_proxy_protocol(raw: &str) -> Result<bool, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "off" => Ok(false),
        "accept" | "require" => Ok(true),
        _ => Err(format!(
            "{PROXY_PROTOCOL_ENV}={raw:?} is not one of off|accept|require"
        )),
    }
}

/// Every entry must parse; returns the entries that are ranges (prefix shorter
/// than a host route). Separators match `CidrSet::parse`.
fn parse_list(name: &str, raw: &str) -> Result<(usize, Vec<String>), String> {
    let mut count = 0;
    let mut ranges = Vec::new();
    for entry in raw
        .split([',', ' ', '\t', '\n', '\r'])
        .filter(|e| !e.trim().is_empty())
    {
        if Cidr::parse(entry).is_none() {
            return Err(format!(
                "{name} entry {entry:?} is not an IP address or CIDR network"
            ));
        }
        count += 1;
        if is_range(entry) {
            ranges.push(entry.trim().to_owned());
        }
    }
    Ok((count, ranges))
}

/// A parsed entry with a prefix shorter than the address width.
fn is_range(entry: &str) -> bool {
    let Some((addr, len)) = entry.trim().split_once('/') else {
        return false;
    };
    let addr = addr.trim();
    let addr = addr
        .strip_prefix('[')
        .and_then(|r| r.strip_suffix(']'))
        .unwrap_or(addr);
    let max = match addr.parse::<IpAddr>() {
        Ok(IpAddr::V4(_)) => 32,
        Ok(IpAddr::V6(_)) => 128,
        Err(_) => return false,
    };
    len.trim().parse::<u8>().is_ok_and(|bits| bits < max)
}

fn flag(raw: Option<String>) -> bool {
    raw.is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Pure validation over a variable lookup. `Err` names the variable; `Ok` holds
/// the warnings for accepted combinations.
pub(crate) fn validate_from(var: impl Fn(&str) -> Option<String>) -> Result<Vec<String>, String> {
    let get = |name: &str| var(name).unwrap_or_default();
    let mode = parse_forwarded_mode(&get(FORWARDED_MODE_ENV))?;
    let proxy_protocol = parse_proxy_protocol(&get(PROXY_PROTOCOL_ENV))?;
    let (trusted, trusted_ranges) = parse_list(TRUSTED_PROXIES_ENV, &get(TRUSTED_PROXIES_ENV))?;
    let (_, header_ranges) = parse_list(HEADER_AUTH_TRUSTED_ENV, &get(HEADER_AUTH_TRUSTED_ENV))?;
    let header_auth = flag(var(HEADER_AUTH_ENV));

    let mut warnings = Vec::new();
    if mode != ForwardedMode::Off && trusted == 0 {
        warnings.push(format!(
            "{FORWARDED_MODE_ENV} is set but {TRUSTED_PROXIES_ENV} is empty: forwarded \
             headers stay ignored and every client is seen as the connecting peer"
        ));
    }
    if trusted > 0 && mode == ForwardedMode::Off && !proxy_protocol {
        warnings.push(format!(
            "{TRUSTED_PROXIES_ENV} is set but {FORWARDED_MODE_ENV} and {PROXY_PROTOCOL_ENV} \
             are off: the list is used for X-Forwarded-Proto and X-Forwarded-Host only, and \
             every client behind the proxy is seen as the proxy's address; set \
             {FORWARDED_MODE_ENV}=xff or forwarded"
        ));
    }
    if proxy_protocol && !trusted_ranges.is_empty() {
        warnings.push(format!(
            "{TRUSTED_PROXIES_ENV} lists a range ({}) while {PROXY_PROTOCOL_ENV} is on: every \
             host in it can declare any client address{}; list the proxy's own addresses",
            trusted_ranges.join(", "),
            if header_auth {
                format!(", including one in {HEADER_AUTH_TRUSTED_ENV}, and so assert an identity")
            } else {
                String::new()
            }
        ));
    }
    if header_auth && !header_ranges.is_empty() {
        warnings.push(format!(
            "{HEADER_AUTH_TRUSTED_ENV} lists a range ({}) while {HEADER_AUTH_ENV} is on: every \
             host in it can assert a signed-in identity; list the proxy's own addresses",
            header_ranges.join(", ")
        ));
    }
    Ok(warnings)
}

/// Run [`validate_from`] on the process environment and log its warnings.
pub fn validate_startup() -> Result<(), String> {
    for warning in validate_from(|name| std::env::var(name).ok())? {
        tracing::warn!("{warning}");
    }
    Ok(())
}

// ---- runtime --------------------------------------------------------------

/// Why a forwarded header from a private peer was not used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ignored {
    /// `MW_FORWARDED_MODE` is off.
    ModeOff,
    /// A mode is set but the peer is not in `MW_TRUSTED_PROXIES`.
    PeerNotTrusted,
}

/// Peers a reverse proxy in front of this server usually has: loopback,
/// RFC 1918, link-local, RFC 6598 shared space, IPv6 unique-local and link-local.
fn proxy_like(ip: IpAddr) -> bool {
    match ip.to_canonical() {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || (o[0] == 100 && (o[1] & 0xc0) == 64)
        }
        IpAddr::V6(v6) => {
            let s = v6.segments()[0];
            v6.is_loopback() || (s & 0xfe00) == 0xfc00 || (s & 0xffc0) == 0xfe80
        }
    }
}

/// The classification alone, without state.
pub(crate) fn classify(cfg: &ProxyConfig, peer: IpAddr, headers: &HeaderMap) -> Option<Ignored> {
    if !headers.contains_key("x-forwarded-for") && !headers.contains_key("forwarded") {
        return None;
    }
    if !proxy_like(peer) {
        return None;
    }
    if cfg.mode == ForwardedMode::Off {
        Some(Ignored::ModeOff)
    } else if !cfg.is_trusted(peer.to_canonical()) {
        Some(Ignored::PeerNotTrusted)
    } else {
        None
    }
}

/// Process-wide record of the observation and of the last warning.
#[derive(Default)]
pub(crate) struct Monitor {
    seen: AtomicBool,
    last_warning: Mutex<Option<Instant>>,
}

impl Monitor {
    /// Record one request. Returns the finding when a warning is due now.
    pub(crate) fn note(
        &self,
        cfg: &ProxyConfig,
        peer: IpAddr,
        headers: &HeaderMap,
        now: Instant,
    ) -> Option<Ignored> {
        let finding = classify(cfg, peer, headers)?;
        self.seen.store(true, Ordering::Relaxed);
        let mut last = self.last_warning.lock().unwrap_or_else(|p| p.into_inner());
        match *last {
            Some(at) if now.saturating_duration_since(at) < WARN_INTERVAL => None,
            _ => {
                *last = Some(now);
                Some(finding)
            }
        }
    }

    pub(crate) fn seen(&self) -> bool {
        self.seen.load(Ordering::Relaxed)
    }
}

static MONITOR: std::sync::LazyLock<Monitor> = std::sync::LazyLock::new(Monitor::default);

fn warning_text(finding: Ignored) -> String {
    match finding {
        Ignored::ModeOff => format!(
            "requests from a private-network peer carry X-Forwarded-For or Forwarded, but \
             {FORWARDED_MODE_ENV} is off: if this server is behind a reverse proxy, every \
             client is seen as the proxy's address, so rate limits, bans, sign-in cooldowns, \
             trust tiers and sign-in geography treat all users as one, and /healthz answers \
             everyone; set {FORWARDED_MODE_ENV}=xff (or forwarded) and {TRUSTED_PROXIES_ENV} to the \
             proxy's address"
        ),
        Ignored::PeerNotTrusted => format!(
            "requests from a private-network peer carry a forwarded header, but the peer is \
             not in {TRUSTED_PROXIES_ENV}, so {FORWARDED_MODE_ENV} does not apply to it: if it \
             is your reverse proxy, every client behind it is seen as the proxy's address, \
             so rate limits, bans, sign-in cooldowns, trust tiers and sign-in geography treat \
             all users as one, and /healthz answers everyone; add the proxy's address to {TRUSTED_PROXIES_ENV}"
        ),
    }
}

/// Called once per resolved client address (`client_ip`). Never changes it.
pub(crate) fn observe(cfg: &ProxyConfig, peer: IpAddr, headers: &HeaderMap) {
    if let Some(finding) = MONITOR.note(cfg, peer, headers, Instant::now()) {
        tracing::warn!("{} (logged at most once an hour)", warning_text(finding));
    }
}

/// Has this process seen a forwarded header from a private peer that it did not
/// use? Shown by the admin diagnostics.
pub(crate) fn forwarded_headers_ignored() -> bool {
    MONITOR.seen()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |name| map.get(name).cloned()
    }

    #[test]
    fn unset_is_valid_and_quiet() {
        assert_eq!(validate_from(env(&[])), Ok(vec![]));
    }

    #[test]
    fn correct_configuration_is_quiet() {
        let w = validate_from(env(&[
            ("MW_FORWARDED_MODE", "xff"),
            ("MW_TRUSTED_PROXIES", "10.0.0.5, [2001:db8::1]"),
        ]));
        assert_eq!(w, Ok(vec![]));
        let w = validate_from(env(&[
            ("MW_PROXY_PROTOCOL", "require"),
            ("MW_TRUSTED_PROXIES", "10.0.0.5/32"),
            ("MW_HEADER_AUTH", "1"),
            ("MW_HEADER_AUTH_TRUSTED_IPS", "10.0.0.5"),
        ]));
        assert_eq!(w, Ok(vec![]), "a /32 is a single address");
    }

    #[test]
    fn unparseable_values_refuse_start() {
        for (name, value) in [
            ("MW_FORWARDED_MODE", "x-forwarded"),
            ("MW_PROXY_PROTOCOL", "yes"),
            ("MW_TRUSTED_PROXIES", "10.0.0.0/8, 10.0.0.300"),
            ("MW_TRUSTED_PROXIES", "10.0.0.0/33"),
            ("MW_HEADER_AUTH_TRUSTED_IPS", "proxy.internal"),
        ] {
            let err = validate_from(env(&[(name, value)])).unwrap_err();
            assert!(err.contains(name), "{err} names {name}");
        }
    }

    #[test]
    fn mode_without_trusted_proxies_warns() {
        let w = validate_from(env(&[("MW_FORWARDED_MODE", "forwarded")])).unwrap();
        assert_eq!(w.len(), 1);
        assert!(w[0].contains("MW_TRUSTED_PROXIES is empty"), "{}", w[0]);
    }

    #[test]
    fn trusted_proxies_with_everything_off_warns() {
        let w = validate_from(env(&[("MW_TRUSTED_PROXIES", "10.0.0.5")])).unwrap();
        assert_eq!(w.len(), 1);
        assert!(
            w[0].contains("X-Forwarded-Proto and X-Forwarded-Host only"),
            "{}",
            w[0]
        );
        // PROXY protocol uses the list on its own: no warning.
        let w = validate_from(env(&[
            ("MW_TRUSTED_PROXIES", "10.0.0.5"),
            ("MW_PROXY_PROTOCOL", "accept"),
        ]));
        assert_eq!(w, Ok(vec![]));
    }

    #[test]
    fn a_range_with_proxy_protocol_warns() {
        let w = validate_from(env(&[
            ("MW_FORWARDED_MODE", "xff"),
            ("MW_TRUSTED_PROXIES", "10.0.0.0/8"),
            ("MW_PROXY_PROTOCOL", "accept"),
        ]))
        .unwrap();
        assert_eq!(w.len(), 1);
        assert!(
            w[0].contains("10.0.0.0/8") && !w[0].contains("identity"),
            "{}",
            w[0]
        );
        let w = validate_from(env(&[
            ("MW_FORWARDED_MODE", "xff"),
            ("MW_TRUSTED_PROXIES", "10.0.0.0/8"),
            ("MW_PROXY_PROTOCOL", "require"),
            ("MW_HEADER_AUTH", "true"),
            ("MW_HEADER_AUTH_TRUSTED_IPS", "10.0.0.5"),
        ]))
        .unwrap();
        assert_eq!(w.len(), 1);
        assert!(w[0].contains("assert an identity"), "{}", w[0]);
        // Without PROXY protocol a range in the proxy list does not reach header auth.
        let w = validate_from(env(&[
            ("MW_FORWARDED_MODE", "xff"),
            ("MW_TRUSTED_PROXIES", "10.0.0.0/8"),
            ("MW_HEADER_AUTH", "1"),
            ("MW_HEADER_AUTH_TRUSTED_IPS", "10.0.0.5"),
        ]));
        assert_eq!(w, Ok(vec![]));
    }

    #[test]
    fn a_range_in_the_header_auth_list_warns() {
        let w = validate_from(env(&[
            ("MW_HEADER_AUTH", "1"),
            ("MW_HEADER_AUTH_TRUSTED_IPS", "172.16.0.0/12"),
        ]))
        .unwrap();
        assert_eq!(w.len(), 1);
        assert!(w[0].contains("172.16.0.0/12"), "{}", w[0]);
        // Header auth off: the list is unused, no warning.
        let w = validate_from(env(&[("MW_HEADER_AUTH_TRUSTED_IPS", "172.16.0.0/12")]));
        assert_eq!(w, Ok(vec![]));
    }

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }
    fn xff() -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-for", "203.0.113.9".parse().unwrap());
        h
    }

    #[test]
    fn classification() {
        let off = ProxyConfig::new(ForwardedMode::Off, "");
        assert_eq!(
            classify(&off, ip("172.17.0.1"), &xff()),
            Some(Ignored::ModeOff)
        );
        assert_eq!(
            classify(&off, ip("127.0.0.1"), &xff()),
            Some(Ignored::ModeOff)
        );
        assert_eq!(
            classify(&off, ip("100.64.1.2"), &xff()),
            Some(Ignored::ModeOff)
        );
        assert_eq!(
            classify(&off, ip("fd00::1"), &xff()),
            Some(Ignored::ModeOff)
        );
        assert_eq!(
            classify(&off, ip("::ffff:10.1.2.3"), &xff()),
            Some(Ignored::ModeOff)
        );
        let mut fwd = HeaderMap::new();
        fwd.insert("forwarded", "for=203.0.113.9".parse().unwrap());
        assert_eq!(classify(&off, ip("10.1.2.3"), &fwd), Some(Ignored::ModeOff));
        // A public peer sending the header is a client making a claim, not a proxy.
        assert_eq!(classify(&off, ip("198.51.100.7"), &xff()), None);
        // No forwarded header: nothing to report.
        assert_eq!(classify(&off, ip("10.1.2.3"), &HeaderMap::new()), None);
        // Correctly configured: the header is used.
        let on = ProxyConfig::new(ForwardedMode::Xff, "10.1.2.3");
        assert_eq!(classify(&on, ip("10.1.2.3"), &xff()), None);
        // Mode on, but this private peer is not listed.
        assert_eq!(
            classify(&on, ip("10.9.9.9"), &xff()),
            Some(Ignored::PeerNotTrusted)
        );
    }

    #[test]
    fn the_warning_fires_once_then_at_most_hourly() {
        let m = Monitor::default();
        let off = ProxyConfig::new(ForwardedMode::Off, "");
        let t0 = Instant::now();
        assert!(!m.seen());
        assert_eq!(
            m.note(&off, ip("10.0.0.2"), &xff(), t0),
            Some(Ignored::ModeOff)
        );
        assert!(m.seen());
        for i in 1..50 {
            let t = t0 + Duration::from_secs(i * 60);
            assert_eq!(m.note(&off, ip("10.0.0.2"), &xff(), t), None, "minute {i}");
        }
        assert_eq!(
            m.note(&off, ip("10.0.0.2"), &xff(), t0 + WARN_INTERVAL),
            Some(Ignored::ModeOff)
        );
    }

    #[test]
    fn no_warning_when_configured_or_without_headers() {
        let m = Monitor::default();
        let t0 = Instant::now();
        let off = ProxyConfig::new(ForwardedMode::Off, "");
        assert_eq!(m.note(&off, ip("10.0.0.2"), &HeaderMap::new(), t0), None);
        let on = ProxyConfig::new(ForwardedMode::Xff, "10.0.0.2");
        assert_eq!(m.note(&on, ip("10.0.0.2"), &xff(), t0), None);
        assert!(!m.seen());
    }
}
