//! RFC 1928 SOCKS5 client — hand-rolled, so that **"the encoder cannot express a
//! domain name" is a property of this code rather than a promise in a review.**
//!
//! # Why hand-rolled (t22-e11, plan §7)
//! The whole upstream-proxy design rests on one property: the proxy is never asked
//! to resolve a name. For SOCKS5 that means the `CONNECT` request must carry
//! `ATYP=0x01` (IPv4) or `ATYP=0x04` (IPv6) and **never** `ATYP=0x03`
//! (`DOMAINNAME`). Every general-purpose SOCKS crate accepts a hostname, because
//! that is what ordinary callers want; delegating to one turns our security
//! property into an assertion about someone else's control flow that has to be
//! re-checked on every upgrade.
//!
//! Here it is structural instead:
//!
//!   * [`encode_connect_request`] takes a [`SocketAddr`]. That type is a closed sum
//!     of exactly `V4` and `V6` — **there is no inhabitant of the input type that
//!     carries a name**, so no caller can ask for a domain request even by mistake.
//!   * The body is emitted by an exhaustive `match` on [`IpAddr`]'s two variants.
//!     There is no third arm, and the two `ATYP` bytes come from the constants
//!     below, both statically asserted not to be `0x03`.
//!   * `DOMAINNAME` has no constant, no encoder, and no call site.
//!
//! # What is deliberately NOT hardened here
//! The *reply's* `BND.ADDR` may legally carry `ATYP=0x03`; [`read_connect_reply`]
//! parses it only to consume the right number of bytes and **discards it**. It is
//! never used as a connect target — the socket is already established at that point
//! — so a lying `BND.ADDR` reaches nothing.
//!
//! Authentication is no-auth (`0x00`) and username/password (`0x02`, RFC 1929)
//! only; GSSAPI is not implemented (plan OQ-7).

use std::net::{IpAddr, SocketAddr};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::ProxyRefusal;

/// SOCKS protocol version 5.
pub const VER: u8 = 0x05;
/// `CONNECT` command (RFC 1928 §4).
pub const CMD_CONNECT: u8 = 0x01;
/// Address type: IPv4, four bytes.
pub const ATYP_IPV4: u8 = 0x01;
/// Address type: IPv6, sixteen bytes.
pub const ATYP_IPV6: u8 = 0x04;
// There is deliberately NO `ATYP_DOMAINNAME` (0x03) constant. Adding one would be
// the first step of the bypass this module exists to prevent; the two assertions
// below fail the build if either literal above is ever edited into it.
const _: () = assert!(ATYP_IPV4 != 0x03, "IPv4 ATYP must never be DOMAINNAME");
const _: () = assert!(ATYP_IPV6 != 0x03, "IPv6 ATYP must never be DOMAINNAME");

/// Auth method: no authentication required.
pub const AUTH_NONE: u8 = 0x00;
/// Auth method: username/password (RFC 1929).
pub const AUTH_USERPASS: u8 = 0x02;
/// Auth method: the server accepted none of the ones we offered.
pub const AUTH_UNACCEPTABLE: u8 = 0xff;

/// Encode a SOCKS5 `CONNECT` request for a **literal socket address**.
///
/// The parameter type is the security control: [`SocketAddr`] cannot hold a
/// hostname, so this function has no name to forward and no branch that could emit
/// `ATYP=0x03`. Layout (RFC 1928 §4):
///
/// ```text
/// +-----+-----+-------+------+----------+----------+
/// | VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
/// +-----+-----+-------+------+----------+----------+
/// |  1  |  1  | X'00' |  1   | 4 or 16  |    2     |
/// +-----+-----+-------+------+----------+----------+
/// ```
pub fn encode_connect_request(dst: SocketAddr) -> Vec<u8> {
    let mut out = Vec::with_capacity(22);
    out.push(VER);
    out.push(CMD_CONNECT);
    out.push(0x00); // RSV
    // Exhaustive over `IpAddr`'s two variants. No further arm exists, and neither
    // constant is 0x03 (statically asserted above).
    match dst.ip() {
        IpAddr::V4(v4) => {
            out.push(ATYP_IPV4);
            out.extend_from_slice(&v4.octets());
        }
        IpAddr::V6(v6) => {
            out.push(ATYP_IPV6);
            out.extend_from_slice(&v6.octets());
        }
    }
    out.extend_from_slice(&dst.port().to_be_bytes());
    out
}

/// Encode the method-selection greeting. Offers no-auth, plus username/password
/// when the route carries credentials. Never offers GSSAPI (plan OQ-7).
pub fn encode_greeting(offer_userpass: bool) -> Vec<u8> {
    if offer_userpass {
        vec![VER, 2, AUTH_NONE, AUTH_USERPASS]
    } else {
        vec![VER, 1, AUTH_NONE]
    }
}

/// Encode the RFC 1929 username/password sub-negotiation request. Both fields are
/// length-prefixed with a single byte, so either exceeding 255 bytes is a
/// configuration error rather than something to truncate.
pub fn encode_userpass(username: &str, password: &str) -> Result<Vec<u8>, ProxyRefusal> {
    let u = username.as_bytes();
    let p = password.as_bytes();
    if u.len() > 255 || p.len() > 255 {
        // The message names the field, never the value.
        return Err(ProxyRefusal::RouteInvalid(
            "SOCKS5 username/password must each be at most 255 bytes",
        ));
    }
    let mut out = Vec::with_capacity(3 + u.len() + p.len());
    out.push(0x01); // sub-negotiation version
    out.push(u.len() as u8);
    out.extend_from_slice(u);
    out.push(p.len() as u8);
    out.extend_from_slice(p);
    Ok(out)
}

/// Map a SOCKS5 reply code (RFC 1928 §6) to a short, value-free reason.
fn reply_reason(rep: u8) -> &'static str {
    match rep {
        0x01 => "general SOCKS server failure",
        0x02 => "connection not allowed by ruleset",
        0x03 => "network unreachable",
        0x04 => "host unreachable",
        0x05 => "connection refused",
        0x06 => "TTL expired",
        0x07 => "command not supported",
        0x08 => "address type not supported",
        _ => "unknown SOCKS5 reply code",
    }
}

/// Read and discard the `BND.ADDR`/`BND.PORT` tail of a reply, given its `ATYP`.
///
/// `ATYP=0x03` is accepted **here** — a server is free to name its own bound
/// address and we must consume the right number of bytes to stay framed — but the
/// value is dropped on the floor. Nothing in this crate ever connects to it.
async fn discard_bound_address<S>(stream: &mut S, atyp: u8) -> Result<(), ProxyRefusal>
where
    S: AsyncRead + Unpin,
{
    let len = match atyp {
        ATYP_IPV4 => 4,
        ATYP_IPV6 => 16,
        0x03 => {
            let mut n = [0u8; 1];
            stream
                .read_exact(&mut n)
                .await
                .map_err(|_| ProxyRefusal::ProxyRejected("truncated SOCKS5 reply".into()))?;
            n[0] as usize
        }
        _ => {
            return Err(ProxyRefusal::ProxyRejected(
                "SOCKS5 reply carried an unknown address type".into(),
            ));
        }
    };
    let mut sink = vec![0u8; len + 2]; // address + 2-byte port
    stream
        .read_exact(&mut sink)
        .await
        .map_err(|_| ProxyRefusal::ProxyRejected("truncated SOCKS5 reply".into()))?;
    Ok(())
}

/// Read a `CONNECT` reply header and succeed only on `REP == 0x00`.
async fn read_connect_reply<S>(stream: &mut S) -> Result<(), ProxyRefusal>
where
    S: AsyncRead + Unpin,
{
    let mut head = [0u8; 4]; // VER, REP, RSV, ATYP
    stream
        .read_exact(&mut head)
        .await
        .map_err(|_| ProxyRefusal::ProxyRejected("no SOCKS5 CONNECT reply".into()))?;
    if head[0] != VER {
        return Err(ProxyRefusal::ProxyRejected(
            "SOCKS5 reply had the wrong version".into(),
        ));
    }
    // Frame the reply before judging it, so a rejection still leaves the stream in
    // a known state (and so a malformed tail is reported as malformed).
    discard_bound_address(stream, head[3]).await?;
    if head[1] != 0x00 {
        return Err(ProxyRefusal::ProxyRejected(format!(
            "SOCKS5 CONNECT refused: {}",
            reply_reason(head[1])
        )));
    }
    Ok(())
}

/// Run the full RFC 1928 client handshake over an already-connected stream and
/// leave it as a byte pipe to `dst`.
///
/// `dst` is a [`SocketAddr`]: by construction the proxy is handed a literal
/// address and performs no name resolution.
pub async fn handshake<S>(
    stream: &mut S,
    dst: SocketAddr,
    credentials: Option<(&str, &str)>,
) -> Result<(), ProxyRefusal>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    stream
        .write_all(&encode_greeting(credentials.is_some()))
        .await
        .map_err(|_| ProxyRefusal::ProxyUnreachable("SOCKS5 greeting could not be sent"))?;
    stream
        .flush()
        .await
        .map_err(|_| ProxyRefusal::ProxyUnreachable("SOCKS5 greeting could not be sent"))?;

    let mut sel = [0u8; 2];
    stream
        .read_exact(&mut sel)
        .await
        .map_err(|_| ProxyRefusal::ProxyUnreachable("no SOCKS5 method selection"))?;
    if sel[0] != VER {
        return Err(ProxyRefusal::ProxyRejected(
            "SOCKS5 method selection had the wrong version".into(),
        ));
    }
    match sel[1] {
        AUTH_NONE => {}
        AUTH_USERPASS => {
            // An authentication problem the operator can act on: the route needs
            // credentials it does not have.
            let (u, p) = credentials.ok_or(ProxyRefusal::ProxyAuthRejected(
                "the proxy requires a username and password but the route has none",
            ))?;
            stream
                .write_all(&encode_userpass(u, p)?)
                .await
                .map_err(|_| ProxyRefusal::ProxyRejected("SOCKS5 auth could not be sent".into()))?;
            stream
                .flush()
                .await
                .map_err(|_| ProxyRefusal::ProxyRejected("SOCKS5 auth could not be sent".into()))?;
            let mut ack = [0u8; 2];
            stream
                .read_exact(&mut ack)
                .await
                .map_err(|_| ProxyRefusal::ProxyRejected("no SOCKS5 auth reply".into()))?;
            if ack[1] != 0x00 {
                // The failure is reported; the credential never is.
                return Err(ProxyRefusal::ProxyAuthRejected(
                    "the proxy rejected the route's username and password",
                ));
            }
        }
        AUTH_UNACCEPTABLE => {
            // The server accepted none of the methods we offered — an authentication
            // negotiation failure, and actionable: it means the route's credentials
            // (or lack of them) do not match what the proxy demands.
            return Err(ProxyRefusal::ProxyAuthRejected(
                "the proxy accepted none of the authentication methods offered",
            ));
        }
        _ => {
            return Err(ProxyRefusal::ProxyRejected(
                "SOCKS5 proxy selected an auth method we do not implement".into(),
            ));
        }
    }

    stream
        .write_all(&encode_connect_request(dst))
        .await
        .map_err(|_| ProxyRefusal::ProxyRejected("SOCKS5 CONNECT could not be sent".into()))?;
    stream
        .flush()
        .await
        .map_err(|_| ProxyRefusal::ProxyRejected("SOCKS5 CONNECT could not be sent".into()))?;

    read_connect_reply(stream).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    /// The `ATYP` byte a request carries, whatever the address.
    fn atyp_of(dst: SocketAddr) -> u8 {
        encode_connect_request(dst)[3]
    }

    #[test]
    fn atyp_is_never_domainname_across_the_whole_input_domain() {
        // `SocketAddr` is a closed sum of V4 | V6 and `encode_connect_request`
        // matches both exhaustively, so covering one address of each kind covers
        // every reachable branch of the encoder — this is case exhaustion over the
        // input type, not "we happened not to emit 0x03".
        //
        // The vectors below are chosen to also stress the byte layout at the edges
        // of each address family.
        let v4s = [
            Ipv4Addr::UNSPECIFIED,
            Ipv4Addr::new(255, 255, 255, 255),
            Ipv4Addr::new(203, 0, 113, 7),
            Ipv4Addr::LOCALHOST,
        ];
        let v6s = [
            Ipv6Addr::UNSPECIFIED,
            Ipv6Addr::LOCALHOST,
            Ipv6Addr::new(0x2606, 0x2800, 0x220, 1, 0, 0, 0, 1),
            Ipv6Addr::new(
                0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff,
            ),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for ip in v4s {
            for port in [0u16, 80, 443, 65535] {
                let bytes = encode_connect_request(SocketAddr::from((ip, port)));
                seen.insert(bytes[3]);
                assert_eq!(bytes[3], ATYP_IPV4);
                assert_eq!(bytes.len(), 4 + 4 + 2, "v4 request is 10 bytes");
                assert_eq!(&bytes[4..8], &ip.octets());
                assert_eq!(&bytes[8..10], &port.to_be_bytes());
            }
        }
        for ip in v6s {
            for port in [0u16, 80, 443, 65535] {
                let bytes = encode_connect_request(SocketAddr::from((ip, port)));
                seen.insert(bytes[3]);
                assert_eq!(bytes[3], ATYP_IPV6);
                assert_eq!(bytes.len(), 4 + 16 + 2, "v6 request is 22 bytes");
                assert_eq!(&bytes[4..20], &ip.octets());
                assert_eq!(&bytes[20..22], &port.to_be_bytes());
            }
        }
        // The SET of ATYP bytes the encoder can emit, over every branch it has.
        assert_eq!(
            seen,
            std::collections::BTreeSet::from([ATYP_IPV4, ATYP_IPV6]),
            "the encoder must emit exactly {{0x01, 0x04}} and nothing else"
        );
        assert!(!seen.contains(&0x03), "DOMAINNAME must be unreachable");
    }

    #[test]
    fn atyp_is_a_function_of_the_address_family_alone() {
        // If some future edit made the ATYP depend on anything but the family
        // (a flag, a port, a length) this equality breaks.
        for port in [0u16, 1080, 8080, 65535] {
            assert_eq!(atyp_of(SocketAddr::from((Ipv4Addr::LOCALHOST, port))), 0x01);
            assert_eq!(atyp_of(SocketAddr::from((Ipv6Addr::LOCALHOST, port))), 0x04);
        }
    }

    #[test]
    fn header_prefix_is_the_rfc_1928_connect_shape() {
        let bytes = encode_connect_request(SocketAddr::from((Ipv4Addr::new(203, 0, 113, 7), 443)));
        assert_eq!(&bytes[0..3], &[VER, CMD_CONNECT, 0x00]);
    }

    #[test]
    fn greeting_offers_userpass_only_when_credentials_exist() {
        assert_eq!(encode_greeting(false), vec![0x05, 1, 0x00]);
        assert_eq!(encode_greeting(true), vec![0x05, 2, 0x00, 0x02]);
        // GSSAPI (0x01) is never offered.
        assert!(!encode_greeting(true).contains(&0x01));
    }

    #[test]
    fn userpass_is_length_prefixed_and_bounded() {
        let msg = encode_userpass("op", "sekrit").unwrap();
        assert_eq!(
            msg,
            vec![0x01, 2, b'o', b'p', 6, b's', b'e', b'k', b'r', b'i', b't']
        );
        let long = "x".repeat(256);
        assert!(encode_userpass(&long, "p").is_err());
        assert!(encode_userpass("u", &long).is_err());
    }

    #[tokio::test]
    async fn reply_with_domain_bound_address_is_framed_and_discarded() {
        // A server naming its own bound address must not desync our reader.
        let reply: Vec<u8> = [
            vec![VER, 0x00, 0x00, 0x03, 3],
            b"abc".to_vec(),
            vec![0x01, 0xbb], // port 443
            b"TRAILER".to_vec(),
        ]
        .concat();
        let mut cursor = std::io::Cursor::new(reply);
        read_connect_reply(&mut cursor).await.unwrap();
        let mut rest = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut cursor, &mut rest)
            .await
            .unwrap();
        assert_eq!(rest, b"TRAILER", "the reply tail must be consumed exactly");
    }

    #[tokio::test]
    async fn socks5_auth_failures_are_their_own_discriminant() {
        use tokio::io::duplex;

        // (a) the proxy selects username/password but the route carries none.
        let (mut ours, mut theirs) = duplex(1024);
        tokio::spawn(async move {
            let mut greeting = [0u8; 3];
            let _ = theirs.read_exact(&mut greeting).await;
            let _ = theirs.write_all(&[VER, AUTH_USERPASS]).await;
        });
        let err = handshake(&mut ours, "203.0.113.7:443".parse().unwrap(), None)
            .await
            .unwrap_err();
        assert!(
            matches!(err, ProxyRefusal::ProxyAuthRejected(_)),
            "route without credentials → {err:?}"
        );

        // (b) the sub-negotiation itself fails.
        let (mut ours, mut theirs) = duplex(1024);
        tokio::spawn(async move {
            let mut greeting = [0u8; 4];
            let _ = theirs.read_exact(&mut greeting).await;
            let _ = theirs.write_all(&[VER, AUTH_USERPASS]).await;
            let mut auth = vec![0u8; 3 + 2 + 6];
            let _ = theirs.read_exact(&mut auth).await;
            let _ = theirs.write_all(&[0x01, 0x01]).await; // non-zero = failure
        });
        let err = handshake(
            &mut ours,
            "203.0.113.7:443".parse().unwrap(),
            Some(("op", "sekrit")),
        )
        .await
        .unwrap_err();
        match err {
            ProxyRefusal::ProxyAuthRejected(m) => {
                assert!(
                    !m.contains("sekrit"),
                    "the credential must never appear: {m}"
                )
            }
            other => panic!("sub-negotiation failure → {other:?}"),
        }

        // (c) the proxy accepts none of the methods we offered.
        let (mut ours, mut theirs) = duplex(1024);
        tokio::spawn(async move {
            let mut greeting = [0u8; 3];
            let _ = theirs.read_exact(&mut greeting).await;
            let _ = theirs.write_all(&[VER, AUTH_UNACCEPTABLE]).await;
        });
        let err = handshake(&mut ours, "203.0.113.7:443".parse().unwrap(), None)
            .await
            .unwrap_err();
        assert!(
            matches!(err, ProxyRefusal::ProxyAuthRejected(_)),
            "no acceptable method → {err:?}"
        );
    }

    #[tokio::test]
    async fn a_destination_refusal_is_not_an_auth_failure() {
        // NEGATIVE CONTROL for the split. `REP=0x02` is the proxy refusing the
        // TARGET under its own ruleset — authorization of the destination, not
        // authentication of us. Reporting it as an auth failure would send an
        // operator to check a password that is correct.
        let reply = vec![VER, 0x02, 0x00, ATYP_IPV4, 0, 0, 0, 0, 0, 0];
        let mut cursor = std::io::Cursor::new(reply);
        let err = read_connect_reply(&mut cursor).await.unwrap_err();
        assert!(
            matches!(err, ProxyRefusal::ProxyRejected(_)),
            "REP=0x02 must stay a destination refusal, got {err:?}"
        );
    }

    #[tokio::test]
    async fn nonzero_reply_code_is_a_rejection_with_a_value_free_reason() {
        let reply = vec![VER, 0x02, 0x00, ATYP_IPV4, 0, 0, 0, 0, 0, 0];
        let mut cursor = std::io::Cursor::new(reply);
        let err = read_connect_reply(&mut cursor).await.unwrap_err();
        match err {
            ProxyRefusal::ProxyRejected(m) => {
                assert!(m.contains("not allowed by ruleset"), "{m}")
            }
            other => panic!("expected ProxyRejected, got {other:?}"),
        }
    }
}
