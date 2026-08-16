//! HTTP `CONNECT` tunnelling (RFC 9110 §9.3.6) with an **IP-literal authority**.
//!
//! # The property
//! The request line this module emits is `CONNECT 203.0.113.7:443 HTTP/1.1` — never
//! `CONNECT origin.example:443`. The authority is rendered from a [`SocketAddr`],
//! which cannot hold a name, so there is nothing for the proxy to resolve. The
//! address in it has already been through this crate's [`ip_allowed`] gate; the
//! proxy's only remaining job is to open a socket to it.
//!
//! [`ip_allowed`]: crate::ip_allowed
//!
//! **Documented limitation:** a proxy configured to reject IP-literal authorities
//! (some deployments require a name for policy logging) is unusable with Mailwoman.
//! That is accepted — handing the name over is the bypass this design exists to
//! close, so there is no fallback to a named authority.
//!
//! # Credentials
//! `Proxy-Authorization` belongs to the tunnel setup and is consumed by the proxy.
//! It is written here and **nowhere else**: the request that later travels inside
//! the tunnel is constructed separately (see [`super::http`]), so a proxy
//! credential is structurally incapable of reaching an origin, including across a
//! redirect (each redirect tears the tunnel down and rebuilds it from the top).

use std::net::SocketAddr;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::ProxyRefusal;

/// Cap on the `CONNECT` response head we will buffer before giving up. A proxy that
/// streams an unbounded header block is refused rather than tolerated.
const MAX_RESPONSE_HEAD: usize = 8 * 1024;

/// Render the request-line authority for `dst`.
///
/// [`SocketAddr`]'s `Display` already produces the RFC 3986 forms — `1.2.3.4:443`
/// and `[2001:db8::1]:443` — so IPv6 arrives bracketed without a second code path.
/// The parameter type is the control: no name is representable.
pub fn authority(dst: SocketAddr) -> String {
    dst.to_string()
}

/// Build the `CONNECT` request head sent to the proxy.
///
/// `credentials` becomes a `Proxy-Authorization: Basic` header. The caller owns
/// keeping the value out of logs; this function never emits one.
pub fn encode_request(dst: SocketAddr, credentials: Option<(&str, &str)>) -> String {
    let auth = authority(dst);
    let mut head = format!("CONNECT {auth} HTTP/1.1\r\nHost: {auth}\r\n");
    if let Some((user, pass)) = credentials {
        head.push_str(&format!(
            "Proxy-Authorization: Basic {}\r\n",
            basic_credentials(user, pass)
        ));
    }
    // Ask the proxy to keep the tunnel open for the exchange that follows.
    head.push_str("Proxy-Connection: keep-alive\r\n\r\n");
    head
}

/// Base64 (RFC 4648 standard alphabet, padded) of `user:pass` for HTTP Basic.
///
/// Hand-rolled rather than pulling `base64` into this crate's dependency set: this
/// is the only encode site, the alphabet is fixed, and it keeps the crate's net-zero
/// dependency position intact.
fn basic_credentials(user: &str, pass: &str) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let raw = format!("{user}:{pass}");
    let bytes = raw.as_bytes();
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[(n >> 18) as usize & 0x3f] as char);
        out.push(ALPHABET[(n >> 12) as usize & 0x3f] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 0x3f] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 0x3f] as char
        } else {
            '='
        });
    }
    out
}

/// Send a `CONNECT` for `dst` and consume the proxy's response head, leaving the
/// stream as a byte pipe on success.
pub async fn handshake<S>(
    stream: &mut S,
    dst: SocketAddr,
    credentials: Option<(&str, &str)>,
) -> Result<(), ProxyRefusal>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    stream
        .write_all(encode_request(dst, credentials).as_bytes())
        .await
        .map_err(|_| ProxyRefusal::ProxyUnreachable("CONNECT request could not be sent"))?;
    stream
        .flush()
        .await
        .map_err(|_| ProxyRefusal::ProxyUnreachable("CONNECT request could not be sent"))?;

    // Read exactly up to the end of the head. Anything after CRLFCRLF belongs to
    // the tunnel, so read one byte at a time rather than over-buffering into it.
    let mut head = Vec::with_capacity(256);
    let mut byte = [0u8; 1];
    loop {
        let n = stream
            .read(&mut byte)
            .await
            .map_err(|_| ProxyRefusal::ProxyUnreachable("CONNECT response could not be read"))?;
        if n == 0 {
            return Err(ProxyRefusal::ProxyUnreachable(
                "proxy closed the connection before answering CONNECT",
            ));
        }
        head.push(byte[0]);
        if head.ends_with(b"\r\n\r\n") {
            break;
        }
        if head.len() > MAX_RESPONSE_HEAD {
            return Err(ProxyRefusal::ProxyRejected(
                "CONNECT response head exceeded the cap".into(),
            ));
        }
    }
    check_status(&head)
}

/// Accept only a 2xx status line. The proxy's reason phrase is not echoed back —
/// it is third-party text on a path that ends at a user-visible error.
fn check_status(head: &[u8]) -> Result<(), ProxyRefusal> {
    let line = head
        .split(|b| *b == b'\n')
        .next()
        .ok_or_else(|| ProxyRefusal::ProxyRejected("empty CONNECT response".into()))?;
    let line = String::from_utf8_lossy(line);
    let mut parts = line.split_whitespace();
    let version = parts.next().unwrap_or_default();
    if !version.starts_with("HTTP/1.") {
        return Err(ProxyRefusal::ProxyRejected(
            "CONNECT response was not HTTP/1.x".into(),
        ));
    }
    let status: u16 = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ProxyRefusal::ProxyRejected("CONNECT response had no status".into()))?;
    match status {
        200..=299 => Ok(()),
        // 407 is the one refusal an operator can fix themselves, so it gets its own
        // discriminant rather than being recoverable only by parsing this string.
        // 403 deliberately does NOT come here: that is the proxy refusing the
        // DESTINATION, and telling an operator to check their password when the real
        // answer is "your proxy will not reach that host" sends them to the wrong
        // place.
        407 => Err(ProxyRefusal::ProxyAuthRejected(
            "the proxy requires authentication (HTTP 407)",
        )),
        _ => Err(ProxyRefusal::ProxyRejected(format!(
            "proxy refused CONNECT with status {status}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn authority_is_always_an_ip_literal() {
        assert_eq!(
            authority(SocketAddr::from((Ipv4Addr::new(203, 0, 113, 7), 443))),
            "203.0.113.7:443"
        );
        assert_eq!(
            authority(SocketAddr::from((
                Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1),
                443
            ))),
            "[2001:db8::1]:443",
            "IPv6 authorities must be bracketed"
        );
    }

    #[test]
    fn request_line_and_host_carry_the_same_literal_and_no_name() {
        let req = encode_request(SocketAddr::from((Ipv4Addr::new(203, 0, 113, 7), 443)), None);
        assert!(
            req.starts_with("CONNECT 203.0.113.7:443 HTTP/1.1\r\n"),
            "{req}"
        );
        assert!(req.contains("\r\nHost: 203.0.113.7:443\r\n"), "{req}");
        assert!(req.ends_with("\r\n\r\n"));
        assert!(!req.contains("Proxy-Authorization"));
    }

    #[test]
    fn credentials_go_in_proxy_authorization_only() {
        let req = encode_request(
            SocketAddr::from((Ipv4Addr::new(203, 0, 113, 7), 443)),
            Some(("aladdin", "opensesame")),
        );
        // RFC 7617's own worked example.
        assert!(
            req.contains("Proxy-Authorization: Basic YWxhZGRpbjpvcGVuc2VzYW1l\r\n"),
            "{req}"
        );
        // Never the plain `Authorization` header, which would travel to an origin.
        assert!(!req.contains("\r\nAuthorization:"), "{req}");
        assert!(!req.contains("opensesame"), "the plaintext must not appear");
    }

    #[test]
    fn basic_credentials_pads_every_residue() {
        assert_eq!(basic_credentials("a", ""), "YTo=");
        assert_eq!(basic_credentials("ab", ""), "YWI6");
        assert_eq!(basic_credentials("abc", ""), "YWJjOg==");
        assert_eq!(basic_credentials("user", "pass"), "dXNlcjpwYXNz");
    }

    #[test]
    fn only_2xx_opens_the_tunnel() {
        assert!(check_status(b"HTTP/1.1 200 Connection established\r\n\r\n").is_ok());
        assert!(check_status(b"HTTP/1.0 200 OK\r\n\r\n").is_ok());
        for bad in [
            &b"HTTP/1.1 403 Forbidden\r\n\r\n"[..],
            &b"HTTP/1.1 407 Proxy Authentication Required\r\n\r\n"[..],
            &b"HTTP/1.1 502 Bad Gateway\r\n\r\n"[..],
            &b"ICY 200 OK\r\n\r\n"[..],
            &b"HTTP/1.1 banana\r\n\r\n"[..],
        ] {
            assert!(
                check_status(bad).is_err(),
                "{}",
                String::from_utf8_lossy(bad)
            );
        }
    }

    #[test]
    fn only_407_is_an_authentication_failure() {
        // t22-e12: an operator's "test this route" button must be able to say "your
        // proxy password is wrong" without parsing a message string.
        assert_eq!(
            check_status(b"HTTP/1.1 407 Proxy Authentication Required\r\n\r\n").unwrap_err(),
            ProxyRefusal::ProxyAuthRejected("the proxy requires authentication (HTTP 407)")
        );
        // NEGATIVE CONTROL, and the distinction that makes the split worth having:
        // 403 is the proxy refusing the DESTINATION, not our credentials. If this
        // ever became `ProxyAuthRejected` the operator would be told to check a
        // password that is perfectly correct.
        for destination_refusal in [
            &b"HTTP/1.1 403 Forbidden\r\n\r\n"[..],
            &b"HTTP/1.1 502 Bad Gateway\r\n\r\n"[..],
            &b"HTTP/1.1 503 Service Unavailable\r\n\r\n"[..],
        ] {
            assert!(
                matches!(
                    check_status(destination_refusal).unwrap_err(),
                    ProxyRefusal::ProxyRejected(_)
                ),
                "{} must not read as an auth failure",
                String::from_utf8_lossy(destination_refusal)
            );
        }
    }

    #[test]
    fn proxy_reason_phrase_is_not_echoed_into_the_error() {
        let err = check_status(b"HTTP/1.1 403 <script>alert(1)</script>\r\n\r\n").unwrap_err();
        let ProxyRefusal::ProxyRejected(msg) = err else {
            panic!("expected ProxyRejected");
        };
        assert!(msg.contains("403"), "{msg}");
        assert!(
            !msg.contains("script"),
            "third-party text must not be echoed"
        );
    }
}
