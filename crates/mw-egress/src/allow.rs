//! The audited internal-destination allowlist (t32 e31b, plan M11).
//!
//! [`crate::ip_allowed`] refuses every private address, which is right for mail
//! content and wrong for one narrow case: an on-prem SIEM collector or webhook
//! receiver on RFC1918 that an admin deliberately points Mailwoman at. t32-h5 made
//! webhooks refuse private targets; this list is how an admin re-admits a specific
//! one without turning the refusal off.
//!
//! # What an entry can never do
//! * **Widen another class.** Only [`TrafficClass::Webhooks`],
//!   [`TrafficClass::Siem`] and [`TrafficClass::Analytics`] are eligible ([`TrafficClass::internal_allow_eligible`]);
//!   [`AllowEntry::new`] refuses the rest, and [`InternalAllowList::permits`] checks
//!   again, so a list built some other way cannot widen remote content either.
//! * **Reach loopback, link-local or the cloud-metadata address.** A match is
//!   additionally held to [`crate::on_prem_allowed`], which refuses `127/8`, `::1`,
//!   `169.254/16` (including `169.254.169.254`), `fe80::/10`, multicast,
//!   unspecified and every NAT64/6to4/Teredo/ISATAP embedding of those. An entry
//!   naming such a range is refused when it is parsed, and a broad entry that
//!   happens to contain one still cannot match it.
//! * **Allow everything.** CIDR entries need at least `/8` (IPv4) or `/16` (IPv6).
//!
//! Entries come from `egress_internal_allow` rows written by an admin with the
//! egress capability, and every write is audited by the admin API. Nothing in a
//! request can add one.

use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::TrafficClass;

/// Shortest IPv4 prefix an entry may name.
pub const MIN_V4_PREFIX: u8 = 8;
/// Shortest IPv6 prefix an entry may name.
pub const MIN_V6_PREFIX: u8 = 16;

/// One allowed destination: an address range, or one host name, optionally with a
/// port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllowDestination {
    /// An IPv4/IPv6 network. A single address is a full-length prefix. `port` is
    /// only accepted on a single address (`10.1.2.3:6514`, `[fd00::7]:6514`).
    Net {
        /// The network address, host bits clear.
        addr: IpAddr,
        /// Prefix length.
        prefix: u8,
        /// The only port the entry admits, when set.
        port: Option<u16>,
    },
    /// One host name, compared exactly (lowercase, no trailing dot) with the URL's
    /// host. Its addresses are still held to [`crate::on_prem_allowed`].
    Host {
        /// The name.
        name: String,
        /// The only port the entry admits, when set.
        port: Option<u16>,
    },
}

impl AllowDestination {
    /// Parse an admin-entered destination: `10.20.0.0/16`, `10.20.3.4`,
    /// `10.20.3.4:6514`, `fd00:1::/48`, `[fd00:1::7]:6514`, `siem.corp.example`,
    /// `siem.corp.example:6514`.
    ///
    /// The error is a fixed sentence the admin API can show as the field's message.
    pub fn parse(input: &str) -> Result<AllowDestination, &'static str> {
        let s = input.trim();
        if s.is_empty() {
            return Err("enter an address, a network or a host name");
        }
        if let Some((addr, prefix)) = s.split_once('/') {
            let addr: IpAddr = addr
                .trim_matches(['[', ']'])
                .parse()
                .map_err(|_| "the part before / must be an IP address")?;
            let prefix: u8 = prefix
                .parse()
                .map_err(|_| "the part after / must be a prefix length")?;
            let (min, max) = match addr {
                IpAddr::V4(_) => (MIN_V4_PREFIX, 32),
                IpAddr::V6(_) => (MIN_V6_PREFIX, 128),
            };
            if prefix > max {
                return Err("the prefix length is too long for this address");
            }
            if prefix < min {
                return Err("the network is too broad (use at least /8 for IPv4, /16 for IPv6)");
            }
            if mask(&addr, prefix) != addr {
                return Err("the address has bits set beyond the prefix length");
            }
            refuse_never_allowed(&addr)?;
            return Ok(AllowDestination::Net {
                addr,
                prefix,
                port: None,
            });
        }
        if let Some((ip, port)) = split_ip_port(s)? {
            refuse_never_allowed(&ip)?;
            let prefix = if ip.is_ipv4() { 32 } else { 128 };
            return Ok(AllowDestination::Net {
                addr: ip,
                prefix,
                port,
            });
        }
        let (name, port) = match s.rsplit_once(':') {
            Some((n, p)) => (n, Some(parse_port(p)?)),
            None => (s, None),
        };
        let name = name.trim_end_matches('.').to_ascii_lowercase();
        validate_host_name(&name)?;
        Ok(AllowDestination::Host { name, port })
    }

    /// Whether a request to `host:port` that resolved to `ip` falls under this
    /// entry. `host` is the URL host as `reqwest::Url::host_str` gives it.
    fn matches(&self, host: &str, port: u16, ip: &IpAddr) -> bool {
        match self {
            AllowDestination::Net {
                addr,
                prefix,
                port: want,
            } => {
                if want.is_some_and(|p| p != port) {
                    return false;
                }
                let ip = ip.to_canonical();
                match (addr, ip) {
                    (IpAddr::V4(_), IpAddr::V4(_)) | (IpAddr::V6(_), IpAddr::V6(_)) => {
                        mask(&ip, *prefix) == *addr
                    }
                    _ => false,
                }
            }
            AllowDestination::Host { name, port: want } => {
                if want.is_some_and(|p| p != port) {
                    return false;
                }
                host.trim_end_matches('.').eq_ignore_ascii_case(name)
            }
        }
    }
}

impl fmt::Display for AllowDestination {
    /// The canonical spelling, which [`AllowDestination::parse`] reads back to the
    /// same value. Stored and shown.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllowDestination::Net {
                addr,
                prefix,
                port: None,
            } if *prefix == full_prefix(addr) => write!(f, "{addr}"),
            AllowDestination::Net {
                addr: IpAddr::V4(a),
                port: Some(p),
                ..
            } => write!(f, "{a}:{p}"),
            AllowDestination::Net {
                addr: IpAddr::V6(a),
                port: Some(p),
                ..
            } => write!(f, "[{a}]:{p}"),
            AllowDestination::Net { addr, prefix, .. } => write!(f, "{addr}/{prefix}"),
            AllowDestination::Host {
                name,
                port: Some(p),
            } => write!(f, "{name}:{p}"),
            AllowDestination::Host { name, port: None } => write!(f, "{name}"),
        }
    }
}

/// One allowlist entry: a class and a destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowEntry {
    class: TrafficClass,
    destination: AllowDestination,
}

impl AllowEntry {
    /// An entry for an eligible class. Any other class is refused.
    pub fn new(
        class: TrafficClass,
        destination: AllowDestination,
    ) -> Result<AllowEntry, &'static str> {
        if !class.internal_allow_eligible() {
            return Err(
                "only the webhooks, siem and analytics classes may reach internal destinations",
            );
        }
        Ok(AllowEntry { class, destination })
    }

    /// The class this entry widens.
    pub fn class(&self) -> TrafficClass {
        self.class
    }

    /// The destination it admits.
    pub fn destination(&self) -> &AllowDestination {
        &self.destination
    }
}

/// The admin's internal-destination allowlist, as loaded from
/// `egress_internal_allow`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InternalAllowList {
    entries: Vec<AllowEntry>,
}

impl InternalAllowList {
    /// An empty list: no class reaches any private destination.
    pub fn empty() -> InternalAllowList {
        InternalAllowList::default()
    }

    /// A list of already validated entries.
    pub fn new(entries: Vec<AllowEntry>) -> InternalAllowList {
        InternalAllowList { entries }
    }

    /// The entries.
    pub fn entries(&self) -> &[AllowEntry] {
        &self.entries
    }

    /// Whether a `class` request to `host:port` that resolved to `ip` may proceed
    /// even though [`crate::ip_allowed`] refuses `ip`.
    ///
    /// All three must hold: the class is eligible, `ip` passes
    /// [`crate::on_prem_allowed`] (so loopback, link-local, metadata and their
    /// embeddings never pass), and an entry for this class matches.
    pub fn permits(&self, class: TrafficClass, host: &str, port: u16, ip: &IpAddr) -> bool {
        class.internal_allow_eligible()
            && crate::on_prem_allowed(ip)
            && self
                .entries
                .iter()
                .any(|e| e.class == class && e.destination.matches(host, port, ip))
    }
}

fn full_prefix(addr: &IpAddr) -> u8 {
    if addr.is_ipv4() { 32 } else { 128 }
}

/// `addr` with every bit past `prefix` cleared.
fn mask(addr: &IpAddr, prefix: u8) -> IpAddr {
    match addr {
        IpAddr::V4(a) => {
            let bits = u32::from(*a);
            let m = if prefix == 0 {
                0
            } else {
                u32::MAX << (32 - u32::from(prefix))
            };
            IpAddr::V4(Ipv4Addr::from(bits & m))
        }
        IpAddr::V6(a) => {
            let bits = u128::from(*a);
            let m = if prefix == 0 {
                0
            } else {
                u128::MAX << (128 - u32::from(prefix))
            };
            IpAddr::V6(Ipv6Addr::from(bits & m))
        }
    }
}

/// Refuse an entry whose address is one [`crate::on_prem_allowed`] never admits,
/// so the admin is told at once rather than finding the entry silently inert.
fn refuse_never_allowed(addr: &IpAddr) -> Result<(), &'static str> {
    if crate::on_prem_allowed(addr) {
        Ok(())
    } else {
        Err("loopback, link-local, multicast and metadata addresses can never be allowed")
    }
}

/// An IP literal with an optional port: `10.1.2.3`, `10.1.2.3:6514`, `fd00::7`,
/// `[fd00::7]`, `[fd00::7]:6514`. `Ok(None)` when `s` is not an IP literal.
fn split_ip_port(s: &str) -> Result<Option<(IpAddr, Option<u16>)>, &'static str> {
    if let Some(rest) = s.strip_prefix('[') {
        let (ip, after) = rest
            .split_once(']')
            .ok_or("a bracketed IPv6 address needs its closing ]")?;
        let ip: Ipv6Addr = ip
            .parse()
            .map_err(|_| "the bracketed part must be an IPv6 address")?;
        let port = match after {
            "" => None,
            p => Some(parse_port(
                p.strip_prefix(':')
                    .ok_or("a port follows the ] after a colon")?,
            )?),
        };
        return Ok(Some((IpAddr::V6(ip), port)));
    }
    if let Ok(ip) = s.parse::<IpAddr>() {
        return Ok(Some((ip, None)));
    }
    if let Some((ip, port)) = s.rsplit_once(':')
        && let Ok(ip) = ip.parse::<Ipv4Addr>()
    {
        return Ok(Some((IpAddr::V4(ip), Some(parse_port(port)?))));
    }
    Ok(None)
}

fn parse_port(p: &str) -> Result<u16, &'static str> {
    match p.parse::<u16>() {
        Ok(0) | Err(_) => Err("the port must be a number from 1 to 65535"),
        Ok(n) => Ok(n),
    }
}

fn validate_host_name(name: &str) -> Result<(), &'static str> {
    if name.is_empty() || name.len() > 253 {
        return Err("enter a host name");
    }
    if name == "localhost" || name.ends_with(".localhost") {
        return Err("loopback, link-local, multicast and metadata addresses can never be allowed");
    }
    for label in name.split('.') {
        if label.is_empty()
            || label.len() > 63
            || label.starts_with('-')
            || label.ends_with('-')
            || !label
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err("a host name uses letters, digits, hyphens and dots only");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    fn list(entries: &[(TrafficClass, &str)]) -> InternalAllowList {
        InternalAllowList::new(
            entries
                .iter()
                .map(|(c, d)| AllowEntry::new(*c, AllowDestination::parse(d).unwrap()).unwrap())
                .collect(),
        )
    }

    #[test]
    fn parses_and_prints_every_accepted_form() {
        for (input, canonical) in [
            ("10.20.0.0/16", "10.20.0.0/16"),
            ("10.20.3.4", "10.20.3.4"),
            ("10.20.3.4/32", "10.20.3.4"),
            ("10.20.3.4:6514", "10.20.3.4:6514"),
            ("fd00:1::/48", "fd00:1::/48"),
            ("fd00:1::7", "fd00:1::7"),
            ("[fd00:1::7]", "fd00:1::7"),
            ("[fd00:1::7]:6514", "[fd00:1::7]:6514"),
            ("SIEM.Corp.Example.", "siem.corp.example"),
            ("siem.corp.example:6514", "siem.corp.example:6514"),
            ("  collector  ", "collector"),
        ] {
            let d = AllowDestination::parse(input).unwrap_or_else(|e| panic!("{input}: {e}"));
            assert_eq!(d.to_string(), canonical, "{input}");
            assert_eq!(
                AllowDestination::parse(canonical).unwrap(),
                d,
                "{canonical}"
            );
        }
    }

    #[test]
    fn refuses_entries_that_could_never_be_honoured_or_are_too_broad() {
        for input in [
            "",
            "127.0.0.1",
            "127.0.0.0/8",
            "169.254.169.254",
            "169.254.0.0/16",
            "::1",
            "fe80::/10",
            "ff02::1",
            "0.0.0.0",
            "localhost",
            "api.localhost",
            "0.0.0.0/0",
            "10.0.0.0/7",
            "::/0",
            "fc00::/7",
            "10.20.3.4/16",
            "10.0.0.0/33",
            "10.20.3.4:0",
            "10.20.3.4:70000",
            "host:port",
            "bad_name.example",
            "-lead.example",
            "a..b",
            "[fd00::1",
            "[fd00::1]6514",
            "64:ff9b::7f00:1",
            "::ffff:169.254.169.254",
        ] {
            assert!(
                AllowDestination::parse(input).is_err(),
                "{input:?} must be refused"
            );
        }
    }

    #[test]
    fn a_class_that_is_not_eligible_cannot_be_given_an_entry() {
        let d = AllowDestination::parse("10.0.0.0/8").unwrap();
        for c in TrafficClass::ALL {
            assert_eq!(
                AllowEntry::new(c, d.clone()).is_ok(),
                c.internal_allow_eligible(),
                "{c}"
            );
        }
    }

    #[test]
    fn a_network_entry_admits_its_range_for_its_class_only() {
        let l = list(&[(TrafficClass::Siem, "10.20.0.0/16")]);
        assert!(l.permits(TrafficClass::Siem, "siem", 6514, &ip("10.20.9.9")));
        assert!(l.permits(TrafficClass::Siem, "siem", 443, &ip("::ffff:10.20.9.9")));
        assert!(!l.permits(TrafficClass::Siem, "siem", 6514, &ip("10.21.0.1")));
        assert!(!l.permits(TrafficClass::Webhooks, "siem", 6514, &ip("10.20.9.9")));
        for c in TrafficClass::ALL {
            if c != TrafficClass::Siem {
                assert!(!l.permits(c, "siem", 6514, &ip("10.20.9.9")), "{c}");
            }
        }
    }

    #[test]
    fn analytics_permission_is_scoped_and_cannot_admit_metadata_or_loopback() {
        let l = list(&[(TrafficClass::Analytics, "collector.corp.example:443")]);
        assert!(l.permits(
            TrafficClass::Analytics,
            "collector.corp.example",
            443,
            &ip("10.20.9.9")
        ));
        for class in TrafficClass::ALL {
            if class != TrafficClass::Analytics {
                assert!(!l.permits(class, "collector.corp.example", 443, &ip("10.20.9.9")));
            }
        }
        for address in [
            "127.0.0.1",
            "::1",
            "169.254.169.254",
            "::ffff:169.254.169.254",
        ] {
            assert!(!l.permits(
                TrafficClass::Analytics,
                "collector.corp.example",
                443,
                &ip(address)
            ));
        }
        assert!(!l.permits(
            TrafficClass::Analytics,
            "other.corp.example",
            443,
            &ip("10.20.9.9")
        ));
        assert!(!l.permits(
            TrafficClass::Analytics,
            "collector.corp.example",
            80,
            &ip("10.20.9.9")
        ));
    }

    #[test]
    fn a_port_on_an_entry_is_enforced() {
        let l = list(&[
            (TrafficClass::Webhooks, "10.1.2.3:8443"),
            (TrafficClass::Webhooks, "hooks.corp.example:9000"),
        ]);
        assert!(l.permits(TrafficClass::Webhooks, "10.1.2.3", 8443, &ip("10.1.2.3")));
        assert!(!l.permits(TrafficClass::Webhooks, "10.1.2.3", 443, &ip("10.1.2.3")));
        assert!(l.permits(
            TrafficClass::Webhooks,
            "hooks.corp.example",
            9000,
            &ip("10.9.9.9")
        ));
        assert!(!l.permits(
            TrafficClass::Webhooks,
            "hooks.corp.example",
            443,
            &ip("10.9.9.9")
        ));
    }

    #[test]
    fn a_host_entry_matches_the_name_not_the_address() {
        let l = list(&[(TrafficClass::Webhooks, "hooks.corp.example")]);
        assert!(l.permits(
            TrafficClass::Webhooks,
            "HOOKS.corp.example.",
            443,
            &ip("10.9.9.9")
        ));
        assert!(!l.permits(TrafficClass::Webhooks, "evil.example", 443, &ip("10.9.9.9")));
        assert!(!l.permits(
            TrafficClass::Webhooks,
            "hooks.corp.example.evil",
            443,
            &ip("10.9.9.9")
        ));
    }

    #[test]
    fn no_entry_can_reach_loopback_link_local_or_metadata() {
        // Built directly so the parse-time refusal is out of the way: what is
        // measured here is the match-time `on_prem_allowed` check.
        let broad = InternalAllowList::new(vec![
            AllowEntry::new(
                TrafficClass::Siem,
                AllowDestination::Net {
                    addr: ip("0.0.0.0"),
                    prefix: 0,
                    port: None,
                },
            )
            .unwrap(),
            AllowEntry::new(
                TrafficClass::Siem,
                AllowDestination::Net {
                    addr: ip("::"),
                    prefix: 0,
                    port: None,
                },
            )
            .unwrap(),
            AllowEntry::new(
                TrafficClass::Siem,
                AllowDestination::Host {
                    name: "metadata.internal".into(),
                    port: None,
                },
            )
            .unwrap(),
        ]);
        for a in [
            "127.0.0.1",
            "169.254.169.254",
            "169.254.1.1",
            "0.0.0.0",
            "224.0.0.1",
            "::1",
            "fe80::1",
            "::ffff:127.0.0.1",
            "::ffff:169.254.169.254",
            "64:ff9b::a9fe:a9fe",
            "2002:7f00:1::",
            "2001:0:4136:e378:8000:ffff:5601:5601",
            "2001:470::5efe:a9fe:a9fe",
        ] {
            assert!(
                !broad.permits(TrafficClass::Siem, "metadata.internal", 80, &ip(a)),
                "{a} must never be admitted"
            );
        }
        // Negative control: the same list does admit a private address, so the
        // refusals above are the loopback/link-local rule and not a dead list.
        assert!(broad.permits(TrafficClass::Siem, "metadata.internal", 80, &ip("10.0.0.1")));
    }

    #[test]
    fn the_empty_list_admits_nothing() {
        let l = InternalAllowList::empty();
        for c in TrafficClass::ALL {
            assert!(!l.permits(c, "x", 443, &ip("10.0.0.1")), "{c}");
        }
    }
}
