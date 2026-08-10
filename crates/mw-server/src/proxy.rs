//! Trusted reverse-proxy model: how a request's client IP is decided (t20 B1).
//!
//! Before this module, `X-Forwarded-For` was honoured from **any** peer and its
//! **leftmost** hop was taken, so anyone able to reach the app port could name their
//! own source IP. That defeated the scoped-key IP allowlist
//! (`mw_oauth::enforce::ip_allowed`), let a caller evade the per-key rate limit by
//! rotating the header, and wrote attacker-chosen addresses into the audit log.
//! Worse, the peer address was never even available: the serve path did not install
//! `ConnectInfo`, so the fallback branch was dead code.
//!
//! The model this module implements:
//!
//! * The **peer address** (`ConnectInfo<SocketAddr>`, installed by
//!   `into_make_service_with_connect_info` in `main.rs`) is the floor. Without it
//!   there is no client IP at all — a header on its own never produces one.
//! * A forwarded header is read only when `MW_FORWARDED_MODE` selects one
//!   (`xff` or `forwarded`; default `off`) **and** the peer address is inside
//!   `MW_TRUSTED_PROXIES`. Both are required: configuring proxies without choosing
//!   a mode changes nothing, and choosing a mode without listing proxies changes
//!   nothing.
//! * The hop list is walked **right to left**, skipping hops that are themselves
//!   trusted proxies, and stops at the first hop that is not. That hop is the
//!   client; everything to its left is whatever the client chose to claim and is
//!   never used. A hop that cannot be parsed (`unknown`, an RFC 7239 obfuscated
//!   identifier) stops the walk at the last verified address rather than being
//!   skipped over.
//!
//! Config is read from the environment per call rather than held in `AppState`,
//! matching the rest of the crate's env-driven settings (`MW_HEADER_AUTH_*`,
//! `MW_MCP_RESOURCE`, `MW_RENDER_JAIL`). The work is two `getenv`s and a parse of a
//! short list, against a guard that already does a database lookup.
//!
//! Reachable crate-wide as `crate::scope_mw::proxy` (declared as a `#[path]` child of
//! `scope_mw` so this wave's owner of `lib.rs` is not disturbed); a later `lib.rs`
//! change can promote it to a top-level `pub mod proxy;` unchanged.

use std::net::{IpAddr, SocketAddr};

use axum::extract::ConnectInfo;
use axum::http::{Extensions, HeaderMap};

/// Comma-separated CIDRs (or bare addresses) whose peers may assert a forwarded
/// header, and whose appearances inside one are skipped as intermediate hops.
pub(crate) const TRUSTED_PROXIES_ENV: &str = "MW_TRUSTED_PROXIES";

/// Which forwarded header to read, if any: `off` (default), `xff`, `forwarded`.
pub(crate) const FORWARDED_MODE_ENV: &str = "MW_FORWARDED_MODE";

/// Which forwarded header the deployment's proxy emits. Exactly one is read — never
/// both — so a client cannot pick whichever header the other one does not set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ForwardedMode {
    /// Ignore forwarded headers entirely. The peer address is the client IP.
    #[default]
    Off,
    /// `X-Forwarded-For: client, proxy1, proxy2`.
    Xff,
    /// RFC 7239 `Forwarded: for=client, for=proxy1`.
    Forwarded,
}

impl ForwardedMode {
    /// Parse the env value. Anything unrecognised (including empty) is `Off` — an
    /// operator typo must not silently start trusting headers.
    fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "xff" | "x-forwarded-for" => Self::Xff,
            "forwarded" => Self::Forwarded,
            _ => Self::Off,
        }
    }
}

/// An IP network: a base address plus a prefix length, hand-rolled so no dependency
/// is added for ~60 lines of prefix comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Cidr {
    net: IpAddr,
    bits: u8,
}

impl Cidr {
    /// Parse `10.0.0.0/8`, `2001:db8::/32`, `[2001:db8::1]`, or a bare address (which
    /// becomes a host route). Returns `None` for anything malformed — a bad entry is
    /// dropped rather than widening the list.
    pub(crate) fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }
        let (addr, len) = match raw.split_once('/') {
            Some((a, b)) => (a.trim(), Some(b.trim())),
            None => (raw, None),
        };
        // Tolerate a bracketed IPv6 literal, as operators copy them from URLs.
        let addr = addr
            .strip_prefix('[')
            .and_then(|r| r.strip_suffix(']'))
            .unwrap_or(addr);
        let net: IpAddr = addr.parse().ok()?;
        let max = match net {
            IpAddr::V4(_) => 32,
            IpAddr::V6(_) => 128,
        };
        let bits = match len {
            None => max,
            Some(l) => {
                let b: u8 = l.parse().ok()?;
                if b > max {
                    return None;
                }
                b
            }
        };
        Some(Self { net, bits })
    }

    /// Is `ip` inside this network? An IPv4-mapped IPv6 address (`::ffff:10.0.0.1`,
    /// what a dual-stack listener reports for a v4 peer) is canonicalised first, so
    /// `10.0.0.0/8` matches it. Families that still differ never match.
    pub(crate) fn contains(&self, ip: IpAddr) -> bool {
        match (self.net, ip.to_canonical()) {
            (IpAddr::V4(net), IpAddr::V4(ip)) => prefix_eq(&net.octets(), &ip.octets(), self.bits),
            (IpAddr::V6(net), IpAddr::V6(ip)) => prefix_eq(&net.octets(), &ip.octets(), self.bits),
            _ => false,
        }
    }
}

/// Do `a` and `b` agree on their first `bits` bits? Host bits in either operand are
/// masked off, so `127.0.0.1/8` behaves like `127.0.0.0/8`.
fn prefix_eq(a: &[u8], b: &[u8], bits: u8) -> bool {
    let bits = usize::from(bits);
    let whole = bits / 8;
    if a[..whole] != b[..whole] {
        return false;
    }
    let rest = bits % 8;
    if rest == 0 {
        return true;
    }
    let mask = 0xffu8 << (8 - rest);
    a[whole] & mask == b[whole] & mask
}

/// A parsed allowlist of networks. Shared by every "is this address permitted"
/// question in the crate so there is exactly one CIDR implementation: the
/// trusted-proxy list here, and `MW_HEADER_AUTH_TRUSTED_IPS` in `lib.rs`.
#[derive(Debug, Clone, Default)]
pub(crate) struct CidrSet(Vec<Cidr>);

impl CidrSet {
    /// Parse a comma- (or whitespace-) separated list. Empty and malformed entries
    /// are skipped: a typo drops one network, it never widens the set.
    pub(crate) fn parse(list: &str) -> Self {
        Self(
            list.split([',', ' ', '\t', '\n', '\r'])
                .filter_map(Cidr::parse)
                .collect(),
        )
    }

    /// No usable entry. Callers gating on an allowlist should treat this as
    /// "unconfigured" and decide fail-open or fail-closed explicitly.
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Is `ip` inside any network in the set? IPv4-mapped IPv6 is canonicalised, so a
    /// v4 allowlist still matches a peer a dual-stack listener reports as
    /// `::ffff:a.b.c.d`.
    pub(crate) fn contains(&self, ip: IpAddr) -> bool {
        self.0.iter().any(|c| c.contains(ip))
    }
}

/// The deployment's forwarded-header posture.
#[derive(Debug, Clone, Default)]
pub(crate) struct ProxyConfig {
    pub(crate) mode: ForwardedMode,
    pub(crate) trusted: CidrSet,
}

impl ProxyConfig {
    /// Read [`FORWARDED_MODE_ENV`] and [`TRUSTED_PROXIES_ENV`]. Absent or unparseable
    /// values give the default posture: headers ignored, peer address only.
    pub(crate) fn from_env() -> Self {
        let mode = std::env::var(FORWARDED_MODE_ENV)
            .map(|v| ForwardedMode::parse(&v))
            .unwrap_or_default();
        let trusted = std::env::var(TRUSTED_PROXIES_ENV).unwrap_or_default();
        Self::new(mode, &trusted)
    }

    /// Build from an already-read mode and trusted list (the env-free seam the unit
    /// tests drive).
    pub(crate) fn new(mode: ForwardedMode, trusted: &str) -> Self {
        let trusted = CidrSet::parse(trusted);
        if mode != ForwardedMode::Off && trusted.is_empty() {
            warn_once(concat!(
                "MW_FORWARDED_MODE is set but MW_TRUSTED_PROXIES lists no usable network; ",
                "forwarded headers stay ignored"
            ));
        }
        Self { mode, trusted }
    }

    /// Is `ip` one of the configured reverse proxies?
    pub(crate) fn is_trusted(&self, ip: IpAddr) -> bool {
        self.trusted.contains(ip)
    }
}

/// Log a configuration complaint once per process — `ProxyConfig` is built per
/// request, so an unguarded warning would be per-request noise.
fn warn_once(msg: &'static str) {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| tracing::warn!("{msg}"));
}

/// Resolve the client IP for a request: the peer address from `ConnectInfo`, refined
/// by a forwarded header only when the configuration permits it.
///
/// `None` means the serve path did not install `ConnectInfo` — there is then no
/// address any check may rely on, and callers treat that as "no source IP" rather
/// than falling back to a header.
pub(crate) fn client_ip(headers: &HeaderMap, ext: &Extensions) -> Option<IpAddr> {
    let peer = peer_ip(ext)?;
    Some(resolve(&ProxyConfig::from_env(), peer, headers))
}

/// The address actually on the other end of the socket, canonicalised — no header is
/// consulted, and no proxy configuration can change it.
///
/// Use this, not [`client_ip`], for any check about **who is connecting** rather than
/// who the connection is on behalf of. Header authentication is the case in point: a
/// proxy trusted to report a client address is not thereby trusted to assert an
/// identity, so `MW_HEADER_AUTH_TRUSTED_IPS` must gate on the peer.
///
/// `None` means the serve path installed no `ConnectInfo`; a gate that requires an
/// address must refuse rather than proceed.
pub(crate) fn peer_ip(ext: &Extensions) -> Option<IpAddr> {
    Some(ext.get::<ConnectInfo<SocketAddr>>()?.0.ip().to_canonical())
}

/// The right-to-left walk. `peer` is the floor and is returned unchanged whenever the
/// header must not be believed.
pub(crate) fn resolve(cfg: &ProxyConfig, peer: IpAddr, headers: &HeaderMap) -> IpAddr {
    let peer = peer.to_canonical();
    let hops = match cfg.mode {
        // Headers off: the connection is the only source of truth.
        ForwardedMode::Off => return peer,
        // The peer is not a configured proxy, so whatever it appended is its own
        // claim about itself. Ignore the header entirely.
        _ if !cfg.is_trusted(peer) => return peer,
        ForwardedMode::Xff => xff_hops(headers),
        ForwardedMode::Forwarded => forwarded_hops(headers),
    };

    // Walk from the nearest hop outwards. `verified` tracks the last address we have
    // confirmed is a trusted proxy, so an unusable entry falls back to it instead of
    // to something the client wrote.
    let mut verified = peer;
    for hop in hops.iter().rev() {
        match hop {
            Some(ip) if cfg.is_trusted(*ip) => verified = *ip,
            // First untrusted hop: this is the client. Anything further left was
            // supplied by it and is not evidence of anything.
            Some(ip) => return *ip,
            // `unknown` / obfuscated / malformed: we cannot show it is a proxy, and
            // we cannot use it as an address either. Stop here.
            None => return verified,
        }
    }
    // Every hop was a trusted proxy (or there were none): the leftmost entry is the
    // originating address, and the peer address if the header was absent.
    hops.first().copied().flatten().unwrap_or(verified)
}

/// All `X-Forwarded-For` hops, left to right, across repeated header lines.
fn xff_hops(headers: &HeaderMap) -> Vec<Option<IpAddr>> {
    headers
        .get_all("x-forwarded-for")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .map(parse_hop)
        .collect()
}

/// All RFC 7239 `Forwarded` `for=` hops, left to right. Elements are comma-separated;
/// parameters within an element are semicolon-separated. An element without a `for=`
/// contributes nothing.
fn forwarded_hops(headers: &HeaderMap) -> Vec<Option<IpAddr>> {
    let mut out = Vec::new();
    for value in headers.get_all("forwarded").iter() {
        let Ok(value) = value.to_str() else { continue };
        for element in value.split(',') {
            for param in element.split(';') {
                let param = param.trim();
                let Some((name, v)) = param.split_once('=') else {
                    continue;
                };
                if name.trim().eq_ignore_ascii_case("for") {
                    out.push(parse_hop(v));
                    break;
                }
            }
        }
    }
    out
}

/// Parse one hop: a bare address, a quoted one, `1.2.3.4:5678`, or
/// `"[2001:db8::1]:443"`. `None` for anything that is not an address.
fn parse_hop(raw: &str) -> Option<IpAddr> {
    let v = raw.trim().trim_matches('"').trim();
    if v.is_empty() {
        return None;
    }
    let candidate = if let Some(rest) = v.strip_prefix('[') {
        // Bracketed IPv6, with or without a port.
        rest.split(']').next().unwrap_or(rest)
    } else if v.matches(':').count() == 1 {
        // Exactly one colon means `v4:port`; a bare IPv6 always has more.
        v.rsplit_once(':').map(|(h, _)| h).unwrap_or(v)
    } else {
        v
    };
    candidate.parse::<IpAddr>().ok().map(|ip| ip.to_canonical())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    fn hdrs(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.append(
                axum::http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                v.parse().unwrap(),
            );
        }
        h
    }

    fn xff(cfg_trusted: &str, header: &str, peer: &str) -> IpAddr {
        let cfg = ProxyConfig::new(ForwardedMode::Xff, cfg_trusted);
        resolve(&cfg, ip(peer), &hdrs(&[("x-forwarded-for", header)]))
    }

    #[test]
    fn cidr_matches_v4_prefixes() {
        let c = Cidr::parse("10.0.0.0/8").unwrap();
        assert!(c.contains(ip("10.0.0.1")));
        assert!(c.contains(ip("10.255.255.255")));
        assert!(!c.contains(ip("11.0.0.1")));
        assert!(!c.contains(ip("9.255.255.255")));

        // Non-byte-aligned prefix.
        let c = Cidr::parse("192.168.4.0/22").unwrap();
        assert!(c.contains(ip("192.168.4.1")));
        assert!(c.contains(ip("192.168.7.255")));
        assert!(!c.contains(ip("192.168.8.0")));

        // Host bits in the operand are masked off.
        let c = Cidr::parse("127.0.0.1/8").unwrap();
        assert!(c.contains(ip("127.9.9.9")));

        // A bare address is a host route.
        let c = Cidr::parse("203.0.113.7").unwrap();
        assert!(c.contains(ip("203.0.113.7")));
        assert!(!c.contains(ip("203.0.113.8")));

        // /0 matches everything, /32 only itself.
        assert!(Cidr::parse("0.0.0.0/0").unwrap().contains(ip("8.8.8.8")));
        let c = Cidr::parse("8.8.8.8/32").unwrap();
        assert!(c.contains(ip("8.8.8.8")) && !c.contains(ip("8.8.8.9")));
    }

    #[test]
    fn cidr_matches_v6_and_mapped_v4() {
        let c = Cidr::parse("2001:db8::/32").unwrap();
        assert!(c.contains(ip("2001:db8::1")));
        assert!(c.contains(ip("2001:db8:ffff::1")));
        assert!(!c.contains(ip("2001:db9::1")));
        assert!(!c.contains(ip("10.0.0.1")), "families must not cross");

        // A dual-stack listener reports a v4 peer as ::ffff:a.b.c.d.
        assert!(
            Cidr::parse("10.0.0.0/8")
                .unwrap()
                .contains(ip("::ffff:10.1.2.3"))
        );
        assert!(Cidr::parse("::1/128").unwrap().contains(ip("::1")));
        assert!(
            Cidr::parse("[2001:db8::1]")
                .unwrap()
                .contains(ip("2001:db8::1"))
        );
    }

    #[test]
    fn cidr_rejects_malformed_entries() {
        assert!(Cidr::parse("").is_none());
        assert!(Cidr::parse("not-an-ip").is_none());
        assert!(Cidr::parse("10.0.0.0/33").is_none());
        assert!(Cidr::parse("2001:db8::/129").is_none());
        assert!(Cidr::parse("10.0.0.0/x").is_none());
        // A malformed entry is dropped, the rest of the list survives.
        let cfg = ProxyConfig::new(ForwardedMode::Xff, "nonsense, 10.0.0.0/8");
        assert!(cfg.is_trusted(ip("10.1.1.1")));
        assert!(!cfg.is_trusted(ip("8.8.8.8")));
        // An all-malformed list leaves nothing trusted — it never becomes match-all.
        let cfg = ProxyConfig::new(ForwardedMode::Xff, "nonsense, 10.0.0.0/33");
        assert!(!cfg.is_trusted(ip("10.1.1.1")));
    }

    #[test]
    fn cidr_set_parses_lists_and_matches() {
        let set = CidrSet::parse("10.0.0.0/8, 192.168.1.7 , 2001:db8::/32");
        assert!(!set.is_empty());
        assert!(set.contains(ip("10.9.9.9")));
        assert!(set.contains(ip("192.168.1.7")));
        assert!(!set.contains(ip("192.168.1.8")), "bare IP is a host route");
        assert!(set.contains(ip("2001:db8::5")));
        assert!(!set.contains(ip("8.8.8.8")));
        // A dual-stack listener's mapped v4 peer still matches a v4 network.
        assert!(set.contains(ip("::ffff:10.9.9.9")));

        // Empty and all-malformed lists are empty sets, never match-alls.
        assert!(CidrSet::parse("").is_empty());
        assert!(CidrSet::default().is_empty());
        let junk = CidrSet::parse("nonsense, 10.0.0.0/33");
        assert!(junk.is_empty());
        assert!(!junk.contains(ip("10.0.0.1")));
    }

    #[test]
    fn peer_ip_reads_only_connect_info() {
        assert!(peer_ip(&Extensions::new()).is_none());

        let mut ext = Extensions::new();
        ext.insert(ConnectInfo(
            "198.51.100.7:52000".parse::<SocketAddr>().unwrap(),
        ));
        assert_eq!(peer_ip(&ext).unwrap(), ip("198.51.100.7"));

        // A mapped v4 peer is canonicalised, so a v4 allowlist matches it.
        let mut ext = Extensions::new();
        ext.insert(ConnectInfo(
            "[::ffff:10.1.2.3]:52000".parse::<SocketAddr>().unwrap(),
        ));
        assert_eq!(peer_ip(&ext).unwrap(), ip("10.1.2.3"));
    }

    #[test]
    fn mode_parses_conservatively() {
        assert_eq!(ForwardedMode::parse("xff"), ForwardedMode::Xff);
        assert_eq!(ForwardedMode::parse(" XFF "), ForwardedMode::Xff);
        assert_eq!(ForwardedMode::parse("forwarded"), ForwardedMode::Forwarded);
        assert_eq!(ForwardedMode::parse("off"), ForwardedMode::Off);
        assert_eq!(ForwardedMode::parse(""), ForwardedMode::Off);
        assert_eq!(ForwardedMode::parse("true"), ForwardedMode::Off);
        assert_eq!(ForwardedMode::default(), ForwardedMode::Off);
    }

    #[test]
    fn default_posture_ignores_the_header() {
        let cfg = ProxyConfig::default();
        let h = hdrs(&[("x-forwarded-for", "8.8.8.8")]);
        assert_eq!(resolve(&cfg, ip("203.0.113.9"), &h), ip("203.0.113.9"));

        // Even with proxies listed, no mode means no header.
        let cfg = ProxyConfig::new(ForwardedMode::Off, "203.0.113.9/32");
        assert_eq!(resolve(&cfg, ip("203.0.113.9"), &h), ip("203.0.113.9"));
    }

    #[test]
    fn untrusted_peer_cannot_assert_a_header() {
        // The peer is not in the trusted list, so its header is its own claim.
        assert_eq!(
            xff("10.0.0.0/8", "8.8.8.8", "203.0.113.9"),
            ip("203.0.113.9")
        );
        // Not even to claim an address inside the trusted range.
        assert_eq!(
            xff("10.0.0.0/8", "10.1.2.3", "203.0.113.9"),
            ip("203.0.113.9")
        );
        // No header at all: the peer, unchanged.
        let cfg = ProxyConfig::new(ForwardedMode::Xff, "10.0.0.0/8");
        assert_eq!(
            resolve(&cfg, ip("10.0.0.1"), &HeaderMap::new()),
            ip("10.0.0.1")
        );
    }

    #[test]
    fn trusted_peer_yields_the_rightmost_untrusted_hop() {
        // Single proxy: the one hop it appended is the client.
        assert_eq!(
            xff("10.0.0.0/8", "203.0.113.9", "10.0.0.1"),
            ip("203.0.113.9")
        );

        // Two trusted proxies in the chain: skip both, take the hop before them.
        assert_eq!(
            xff("10.0.0.0/8", "203.0.113.9, 10.0.0.2, 10.0.0.3", "10.0.0.1"),
            ip("203.0.113.9")
        );

        // THE ATTACK: the real client prepends a forged hop. The walk stops at the
        // rightmost untrusted entry — the address the trusted proxy actually saw —
        // so the forged one on the left is never reached.
        assert_eq!(
            xff("10.0.0.0/8", "10.9.9.9, 203.0.113.9", "10.0.0.1"),
            ip("203.0.113.9"),
        );
        assert_eq!(
            xff("10.0.0.0/8", "8.8.8.8, 1.1.1.1, 203.0.113.9", "10.0.0.1"),
            ip("203.0.113.9"),
        );

        // Every hop is itself a trusted proxy: the leftmost is the origin.
        assert_eq!(
            xff("10.0.0.0/8", "10.5.5.5, 10.0.0.2", "10.0.0.1"),
            ip("10.5.5.5")
        );
    }

    #[test]
    fn unusable_hops_stop_the_walk_at_the_last_verified_address() {
        // `unknown` cannot be shown to be a proxy and is not an address: stop, and
        // report the last confirmed proxy rather than anything further left.
        assert_eq!(
            xff("10.0.0.0/8", "203.0.113.9, unknown", "10.0.0.1"),
            ip("10.0.0.1")
        );
        assert_eq!(
            xff("10.0.0.0/8", "203.0.113.9, unknown, 10.0.0.2", "10.0.0.1"),
            ip("10.0.0.2")
        );
        // An empty header value contributes an unusable hop, not a bypass.
        assert_eq!(xff("10.0.0.0/8", "", "10.0.0.1"), ip("10.0.0.1"));
    }

    #[test]
    fn xff_spans_repeated_header_lines_and_ports() {
        let cfg = ProxyConfig::new(ForwardedMode::Xff, "10.0.0.0/8");
        let h = hdrs(&[
            ("x-forwarded-for", "8.8.8.8"),
            ("x-forwarded-for", "203.0.113.9, 10.0.0.2"),
        ]);
        assert_eq!(resolve(&cfg, ip("10.0.0.1"), &h), ip("203.0.113.9"));

        // Some proxies append a port; both families parse.
        assert_eq!(
            xff("10.0.0.0/8", "203.0.113.9:41234", "10.0.0.1"),
            ip("203.0.113.9")
        );
        assert_eq!(
            xff("10.0.0.0/8", "[2001:db8::9]:41234", "10.0.0.1"),
            ip("2001:db8::9")
        );
        assert_eq!(
            xff("10.0.0.0/8", "2001:db8::9", "10.0.0.1"),
            ip("2001:db8::9")
        );
    }

    #[test]
    fn forwarded_mode_reads_only_the_forwarded_header() {
        let cfg = ProxyConfig::new(ForwardedMode::Forwarded, "10.0.0.0/8");

        let h = hdrs(&[("forwarded", "for=203.0.113.9;proto=https")]);
        assert_eq!(resolve(&cfg, ip("10.0.0.1"), &h), ip("203.0.113.9"));

        let h = hdrs(&[("forwarded", "for=\"[2001:db8::1]:443\";proto=https")]);
        assert_eq!(resolve(&cfg, ip("10.0.0.1"), &h), ip("2001:db8::1"));

        // Same right-to-left rule across elements, forged left hop ignored.
        let h = hdrs(&[("forwarded", "for=10.9.9.9, for=203.0.113.9, for=10.0.0.2")]);
        assert_eq!(resolve(&cfg, ip("10.0.0.1"), &h), ip("203.0.113.9"));

        // In `forwarded` mode an X-Forwarded-For is not read, and vice versa: a
        // client cannot choose whichever header the proxy leaves alone.
        let h = hdrs(&[("x-forwarded-for", "8.8.8.8")]);
        assert_eq!(resolve(&cfg, ip("10.0.0.1"), &h), ip("10.0.0.1"));
        let cfg = ProxyConfig::new(ForwardedMode::Xff, "10.0.0.0/8");
        let h = hdrs(&[("forwarded", "for=8.8.8.8")]);
        assert_eq!(resolve(&cfg, ip("10.0.0.1"), &h), ip("10.0.0.1"));
    }

    #[test]
    fn client_ip_requires_connect_info() {
        let h = hdrs(&[("x-forwarded-for", "8.8.8.8")]);
        // No ConnectInfo → no client IP, whatever the header says.
        assert!(client_ip(&h, &Extensions::new()).is_none());

        let mut ext = Extensions::new();
        ext.insert(ConnectInfo(
            "198.51.100.7:52000".parse::<SocketAddr>().unwrap(),
        ));
        // Default posture (no env set in this process): the peer address.
        assert_eq!(client_ip(&h, &ext).unwrap(), ip("198.51.100.7"));
    }
}
