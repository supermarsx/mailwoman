//! The remote-DNS route kind (`socks5h`, t32 e31b, plan R2).
//!
//! # What it gives up, and who decides
//! Every other route resolves the origin name **here**, checks each address with
//! [`crate::ip_allowed`], and hands the proxy a literal. A remote-DNS route hands
//! the proxy the **name**, so the address it connects to is never seen by
//! Mailwoman and cannot be checked. That is the bypass the rest of this crate is
//! built to prevent, so it is only available when:
//!   * an admin with the egress capability has ticked "I accept that hosts
//!     reachable from the proxy's network can be fetched", which is stored with who
//!     and when and audited, and which an edit that changes the route clears; and
//!   * `mw-server`'s one `ProxyRoute` construction site sees that acceptance —
//!     without it the route is refused, never carried as a different kind.
//!
//! # What still holds
//! * A **literal** origin address is never sent as a name: it is checked with
//!   [`crate::ip_allowed`] here and sent as `ATYP` IPv4/IPv6, exactly as on a
//!   `socks5` route ([`RemoteDest::Addr`]).
//! * A **name** must pass [`RemoteName::new`]: it must contain a dot (a resolver
//!   completes a bare name with its search domains), and it must not match the
//!   denylist in `data/remote_dns_denylist.txt` (`localhost`, `*.local`,
//!   `*.internal`, `metadata.google.internal`, `*.svc`, …).
//! * Scheme, credentials-in-URL, the plaintext rule, TLS to the origin by name,
//!   the size and time caps and per-redirect re-checking are unchanged.
//!
//! The residual — a public name that resolves, at the proxy, to an address inside
//! the proxy's network — is what the admin accepted. `docs/security/egress.md`
//! states it.

use std::net::{IpAddr, SocketAddr};
use std::sync::OnceLock;

use tokio::io::{AsyncRead, AsyncWrite};

use super::{ProxyRefusal, socks5};
use crate::{Refusal, ip_allowed};

/// The denylist, as shipped.
const DENYLIST: &str = include_str!("../../data/remote_dns_denylist.txt");

/// `ATYP` for a domain name (RFC 1928 §5). Used only by [`encode_connect_by_name`],
/// whose input is a [`RemoteName`].
const ATYP_DOMAINNAME: u8 = 0x03;

enum Pattern {
    Exact(String),
    Suffix(String),
}

fn denylist() -> &'static [Pattern] {
    static LIST: OnceLock<Vec<Pattern>> = OnceLock::new();
    LIST.get_or_init(|| {
        DENYLIST
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| {
                let l = l.to_ascii_lowercase();
                match l.strip_prefix("*.") {
                    Some(suffix) => Pattern::Suffix(format!(".{suffix}")),
                    None => Pattern::Exact(l),
                }
            })
            .collect()
    })
}

/// Whether `name` (lowercase, no trailing dot) is on the denylist.
fn denied(name: &str) -> bool {
    denylist().iter().any(|p| match p {
        Pattern::Exact(n) => name == n,
        Pattern::Suffix(s) => name.ends_with(s.as_str()),
    })
}

/// A host name a remote-DNS proxy may be asked to resolve: not an IP literal, at
/// most 255 bytes, contains a dot, and not on the denylist. The only input
/// [`encode_connect_by_name`] accepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteName(String);

impl RemoteName {
    /// Check `host` (as `reqwest::Url::host_str` gives it). Every refusal is
    /// [`Refusal::Blocked`], the same coarse answer the address gate gives.
    pub fn new(host: &str) -> Result<RemoteName, Refusal> {
        let name = host.trim_end_matches('.').to_ascii_lowercase();
        if name.is_empty()
            || name.len() > 255
            || name.starts_with('[')
            || name.parse::<IpAddr>().is_ok()
            || !name.contains('.')
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.' || b == b'_')
            || denied(&name)
        {
            return Err(Refusal::Blocked);
        }
        Ok(RemoteName(name))
    }

    /// The checked name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Where a remote-DNS route asks its proxy to connect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteDest {
    /// A literal address that passed [`crate::ip_allowed`] — sent as `ATYP`
    /// IPv4/IPv6.
    Addr(SocketAddr),
    /// A checked name and a port — sent as `ATYP` domain name.
    Name(RemoteName, u16),
}

/// The URL checks of `crate::validate_and_resolve`, without resolving: scheme,
/// credentials, host, port; then a literal host goes through [`crate::ip_allowed`]
/// and a name through [`RemoteName::new`]. Returns the destination and the host
/// name to verify origin TLS against.
pub fn remote_destination(url: &reqwest::Url) -> Result<(RemoteDest, String), Refusal> {
    match url.scheme() {
        "http" | "https" => {}
        _ => return Err(Refusal::BadRequest("only http/https URLs are proxied")),
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(Refusal::BadRequest("credentials in URL are not allowed"));
    }
    let host = url
        .host_str()
        .filter(|h| !h.is_empty())
        .ok_or(Refusal::BadRequest("URL has no host"))?
        .to_string();
    let port = url
        .port_or_known_default()
        .ok_or(Refusal::BadRequest("URL has no port"))?;
    let literal = match host.strip_prefix('[').and_then(|h| h.strip_suffix(']')) {
        Some(v6) => v6.parse::<IpAddr>().ok(),
        None => host.parse::<IpAddr>().ok(),
    };
    let dest = match literal {
        Some(ip) if ip_allowed(&ip) => RemoteDest::Addr(SocketAddr::new(ip, port)),
        Some(_) => return Err(Refusal::Blocked),
        None => RemoteDest::Name(RemoteName::new(&host)?, port),
    };
    Ok((dest, host))
}

/// Encode a SOCKS5 `CONNECT` for a checked name (RFC 1928 §4, `ATYP=0x03`).
pub fn encode_connect_by_name(name: &RemoteName, port: u16) -> Vec<u8> {
    let bytes = name.as_str().as_bytes();
    let mut out = Vec::with_capacity(7 + bytes.len());
    out.push(socks5::VER);
    out.push(socks5::CMD_CONNECT);
    out.push(0x00);
    out.push(ATYP_DOMAINNAME);
    // `RemoteName::new` caps the length at 255, so this cannot truncate.
    out.push(bytes.len() as u8);
    out.extend_from_slice(bytes);
    out.extend_from_slice(&port.to_be_bytes());
    out
}

/// The SOCKS5 handshake to `dest`: a literal through [`socks5::handshake`], a name
/// through [`encode_connect_by_name`].
pub(crate) async fn handshake<S>(
    stream: &mut S,
    dest: &RemoteDest,
    credentials: Option<(&str, &str)>,
) -> Result<(), ProxyRefusal>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    match dest {
        RemoteDest::Addr(a) => socks5::handshake(stream, *a, credentials).await,
        RemoteDest::Name(n, port) => {
            socks5::handshake_with(stream, &encode_connect_by_name(n, *port), credentials).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> reqwest::Url {
        reqwest::Url::parse(s).unwrap()
    }

    #[test]
    fn ordinary_public_names_are_accepted_and_normalised() {
        for (host, want) in [
            ("cdn.example.com", "cdn.example.com"),
            ("CDN.Example.COM.", "cdn.example.com"),
            ("a-b.c_d.example", "a-b.c_d.example"),
        ] {
            assert_eq!(RemoteName::new(host).unwrap().as_str(), want);
        }
    }

    #[test]
    fn denylisted_bare_and_literal_names_are_refused() {
        for host in [
            "localhost",
            "LOCALHOST.",
            "api.localhost",
            "printer.local",
            "db.internal",
            "x.localdomain",
            "metadata.google.internal",
            "metadata.goog",
            "instance-data",
            "kubernetes.default.svc",
            "web.ns.svc.cluster.local",
            "nas.home.arpa",
            "1.0.0.127.in-addr.arpa",
            "files.corp",
            "intranet",
            "router",
            "127.0.0.1",
            "10.0.0.5",
            "[::1]",
            "fd00::1",
            "",
            "bad name.example",
            "evil.example/x",
        ] {
            assert_eq!(RemoteName::new(host), Err(Refusal::Blocked), "{host:?}");
        }
        assert_eq!(
            RemoteName::new(&format!("{}.example", "a".repeat(260))),
            Err(Refusal::Blocked)
        );
    }

    #[test]
    fn a_suffix_entry_does_not_match_a_longer_label() {
        // `*.local` must not refuse `notlocal.example` or `local.example.com`.
        for host in [
            "notlocal.example",
            "local.example.com",
            "internal.example.org",
        ] {
            assert!(RemoteName::new(host).is_ok(), "{host}");
        }
    }

    #[test]
    fn a_literal_origin_is_checked_here_and_never_sent_as_a_name() {
        let (dest, host) = remote_destination(&url("http://1.2.3.4:8080/x")).unwrap();
        assert_eq!(dest, RemoteDest::Addr("1.2.3.4:8080".parse().unwrap()));
        assert_eq!(host, "1.2.3.4");
        let (dest, _) = remote_destination(&url("https://[2606:2800:220:1::1]/x")).unwrap();
        assert!(matches!(dest, RemoteDest::Addr(_)), "{dest:?}");
        for u in [
            "http://127.0.0.1/",
            "http://10.0.0.5/",
            "http://169.254.169.254/latest/meta-data/",
            "http://[::1]/",
            "http://[64:ff9b::7f00:1]/",
            "http://0x7f.1/",
            "http://2130706433/",
        ] {
            assert_eq!(
                remote_destination(&url(u)).unwrap_err(),
                Refusal::Blocked,
                "{u}"
            );
        }
    }

    #[test]
    fn a_name_origin_is_sent_as_a_checked_name() {
        let (dest, host) = remote_destination(&url("https://cdn.example.com/p.png")).unwrap();
        assert_eq!(
            dest,
            RemoteDest::Name(RemoteName::new("cdn.example.com").unwrap(), 443)
        );
        assert_eq!(host, "cdn.example.com");
        assert_eq!(
            remote_destination(&url("http://db.internal/")).unwrap_err(),
            Refusal::Blocked
        );
    }

    #[test]
    fn the_url_checks_are_unchanged() {
        for u in [
            "file:///etc/passwd",
            "ftp://cdn.example.com/",
            "http://u:p@cdn.example.com/",
        ] {
            assert!(
                matches!(remote_destination(&url(u)), Err(Refusal::BadRequest(_))),
                "{u}"
            );
        }
    }

    #[test]
    fn the_name_request_is_rfc_1928_atyp_3() {
        let req = encode_connect_by_name(&RemoteName::new("cdn.example.com").unwrap(), 443);
        assert_eq!(&req[..5], &[0x05, 0x01, 0x00, 0x03, 15]);
        assert_eq!(&req[5..20], b"cdn.example.com");
        assert_eq!(&req[20..], &443u16.to_be_bytes());
    }
}
