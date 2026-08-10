//! PROXY protocol v1 + v2 on the HTTP listener (t20, plan §D3).
//!
//! mailwoman ships built-in ACME (`tls.rs`), so terminating TLS in the app while an
//! L4 balancer passes the TCP stream through untouched is a supported deployment:
//! HAProxy `send-proxy-v2` in `mode tcp`, nginx `stream` with `proxy_protocol on`,
//! an AWS NLB with proxy protocol v2 enabled, Envoy's TCP proxy. In that shape the
//! socket peer is the balancer for every request, so without parsing the PROXY
//! header the scoped-key IP allowlist, the per-key rate limit and the audit log all
//! see one address for the whole internet.
//!
//! The header arrives **before** anything else on the connection — before the TLS
//! ClientHello, before the HTTP request line — so it is read here, at accept time,
//! and the address it carries replaces the peer address handed to
//! `ConnectInfo<SocketAddr>`. Everything downstream ([`crate::scope_mw::proxy`],
//! the header-auth peer gate, `mw_oauth::enforce`) then works unchanged: it asks for
//! the peer address and gets the client's, because that is what the connection
//! really came from.
//!
//! ## Trust
//!
//! A PROXY header is a claim about who is calling, made by whoever is calling. It is
//! honoured only from a peer inside `MW_TRUSTED_PROXIES` — the same list
//! [`crate::scope_mw::proxy`] uses for `X-Forwarded-For` — and a peer outside that
//! list that sends one has its **connection dropped**, not merely ignored. Ignoring
//! it would be the more forgiving choice and the wrong one: the bytes are already in
//! the stream, and a deployment that receives a forged PROXY header is one where the
//! app port is reachable by something that should not reach it.
//!
//! `MW_PROXY_PROTOCOL` selects the posture:
//!
//! * `off` (default) — the header is never read. Byte-for-byte today's behaviour.
//! * `accept` — a trusted peer may send one; a peer that sends none is served as a
//!   direct client. For a deployment migrating onto a balancer.
//! * `require` — every connection must arrive from a trusted peer carrying a valid
//!   header. Anything else is dropped. This is the correct setting once the app port
//!   is behind an L4 balancer, because a connection without a header there did not
//!   come through the balancer.
//!
//! ## What is parsed
//!
//! v1 (the ASCII line, ≤107 bytes) and v2 (the 12-byte signature, a 4-byte header
//! and a length-delimited address block) in full, for `TCP4`/`AF_INET` and
//! `TCP6`/`AF_INET6`. `UNKNOWN` (v1), `LOCAL` (v2 — what a balancer's own health
//! check sends), `AF_UNSPEC` and `AF_UNIX` are recognised, consumed, and resolve to
//! the socket peer address, which is the right answer for all four.
//!
//! v2 TLVs are consumed as part of the declared length and **not interpreted**:
//! `PP2_TYPE_SSL` (the client certificate the balancer saw), `PP2_TYPE_AUTHORITY`
//! (the SNI name), `PP2_TYPE_NETNS`, and the optional `PP2_TYPE_CRC32C` checksum,
//! which is therefore not verified. None of them feed anything this server decides;
//! adding one later is a parse of a byte range already in hand.
//!
//! ## Head-of-line blocking
//!
//! Reading a header is network I/O inside `accept`, so a peer that connects and
//! sends nothing would stall every later connection if it were done inline. Each
//! connection is negotiated in its own task and the results are fed through a
//! channel, so a slow or silent peer costs one task and a [`HEADER_TIMEOUT`],
//! not the listener. In `off` mode nothing is spawned at all.

use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::scope_mw::proxy::CidrSet;

/// Which posture to run in: `off` (default), `accept`, `require`.
pub const PROXY_PROTOCOL_ENV: &str = "MW_PROXY_PROTOCOL";

/// Which peers may send a PROXY header. Shared with the `X-Forwarded-For` model —
/// a proxy trusted to report an address over one mechanism is trusted over the other.
pub const TRUSTED_PROXIES_ENV: &str = crate::scope_mw::proxy::TRUSTED_PROXIES_ENV;

/// How long a connection has to deliver its header once it is expected. Generous
/// for a balancer (which writes the header with the first segment) and short enough
/// that a silent peer releases its task quickly.
const HEADER_TIMEOUT: Duration = Duration::from_secs(5);

/// v2's fixed 12-byte signature. Chosen by the spec so it cannot be confused with
/// the start of an HTTP request, a TLS ClientHello, or a v1 header.
const V2_SIGNATURE: [u8; 12] = [
    0x0d, 0x0a, 0x0d, 0x0a, 0x00, 0x0d, 0x0a, 0x51, 0x55, 0x49, 0x54, 0x0a,
];

/// v1's opening token, including its trailing space.
const V1_TAG: &[u8] = b"PROXY ";

/// The longest legal v1 line, CRLF included (spec §2.1).
const V1_MAX_LEN: usize = 107;

/// Connections negotiated but not yet handed to the server. Bounded so a burst
/// applies backpressure to the accept loop rather than growing without limit.
const ACCEPT_BACKLOG: usize = 128;

/// How long to wait before re-peeking when a peer has sent some of a header but not
/// all of it. `peek` returns immediately while any byte is buffered, so a partial
/// prefix has to be re-read on a timer instead of awaited.
const PARTIAL_HEADER_BACKOFF: Duration = Duration::from_millis(2);

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// The deployment's PROXY-protocol posture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Never read a header. The socket peer is the client.
    #[default]
    Off,
    /// A trusted peer may send one; a peer that sends none is a direct client.
    Accept,
    /// Every connection must come from a trusted peer with a valid header.
    Require,
}

impl Mode {
    /// Parse the env value. Anything unrecognised is [`Mode::Off`] with a warning:
    /// a typo must not silently change how connections are admitted, in either
    /// direction.
    fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "accept" => Self::Accept,
            "require" => Self::Require,
            "" | "off" => Self::Off,
            other => {
                tracing::warn!(
                    "{PROXY_PROTOCOL_ENV}={other:?} is not one of off|accept|require; \
                     PROXY protocol stays off"
                );
                Self::Off
            }
        }
    }
}

/// Mode plus the peers allowed to assert a header.
#[derive(Debug, Clone, Default)]
pub struct Config {
    mode: Mode,
    trusted: CidrSet,
}

impl Config {
    /// Read [`PROXY_PROTOCOL_ENV`] and [`TRUSTED_PROXIES_ENV`].
    pub fn from_env() -> Self {
        let mode = std::env::var(PROXY_PROTOCOL_ENV)
            .map(|v| Mode::parse(&v))
            .unwrap_or_default();
        let trusted = std::env::var(TRUSTED_PROXIES_ENV).unwrap_or_default();
        Self::new(mode, &trusted)
    }

    /// Build from an already-read mode and trusted list — the env-free seam the unit
    /// tests drive.
    pub(crate) fn new(mode: Mode, trusted: &str) -> Self {
        let trusted = CidrSet::parse(trusted);
        if mode != Mode::Off && trusted.is_empty() {
            // Not a warning that can be ignored: in `require` this drops every
            // connection, and in `accept` it means no header will ever be honoured.
            tracing::warn!(
                "{PROXY_PROTOCOL_ENV} is enabled but {TRUSTED_PROXIES_ENV} lists no usable \
                 network; no peer can send a PROXY header"
            );
        }
        Self { mode, trusted }
    }

    /// Is the header never read? The listener skips per-connection negotiation
    /// entirely when so.
    pub fn is_off(&self) -> bool {
        self.mode == Mode::Off
    }

    /// May `peer` assert a client address?
    fn trusts(&self, peer: SocketAddr) -> bool {
        self.trusted.contains(peer.ip())
    }
}

// ---------------------------------------------------------------------------
// Header detection
// ---------------------------------------------------------------------------

/// What the first bytes of a connection look like.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    V1,
    V2,
    /// Not a PROXY header: an HTTP request, a TLS ClientHello, anything else.
    Absent,
    /// Consistent with a header so far, but too few bytes to be sure.
    Partial,
}

/// Classify the bytes seen so far. Deliberately decides `Absent` as soon as the
/// prefix rules both versions out, so an ordinary request is never delayed.
fn classify(seen: &[u8]) -> Kind {
    if seen.len() >= V1_TAG.len() {
        if seen.starts_with(V1_TAG) {
            return Kind::V1;
        }
    } else if V1_TAG.starts_with(seen) {
        return Kind::Partial;
    }
    if seen.len() >= V2_SIGNATURE.len() {
        if seen.starts_with(&V2_SIGNATURE) {
            return Kind::V2;
        }
    } else if V2_SIGNATURE.starts_with(seen) {
        return Kind::Partial;
    }
    Kind::Absent
}

/// Peek — never consume — until the connection's opening bytes can be classified.
/// Returns [`Kind::Absent`] on EOF, error, or timeout; the caller's mode decides
/// whether that is fatal.
async fn detect(stream: &TcpStream) -> Kind {
    let mut buf = [0u8; V2_SIGNATURE.len()];
    let deadline = tokio::time::Instant::now() + HEADER_TIMEOUT;
    loop {
        let n = match tokio::time::timeout_at(deadline, stream.peek(&mut buf)).await {
            Ok(Ok(0)) | Err(_) => return Kind::Absent,
            Ok(Ok(n)) => n,
            Ok(Err(e)) => {
                tracing::debug!("proxy-protocol peek failed: {e}");
                return Kind::Absent;
            }
        };
        match classify(&buf[..n]) {
            Kind::Partial => {
                // More bytes are needed but `peek` will return the same short read
                // immediately, so wait a moment rather than spinning.
                if tokio::time::timeout_at(deadline, tokio::time::sleep(PARTIAL_HEADER_BACKOFF))
                    .await
                    .is_err()
                {
                    return Kind::Absent;
                }
            }
            decided => return decided,
        }
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

fn malformed(what: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("PROXY header: {what}"))
}

/// Consume a v1 header. `Ok(None)` means the header was valid but names no usable
/// client address (`UNKNOWN`), in which case the socket peer stands.
async fn read_v1(stream: &mut TcpStream) -> io::Result<Option<SocketAddr>> {
    // Read to the CRLF a byte at a time: the line has no length prefix, and
    // over-reading would eat the TLS ClientHello behind it.
    let mut line = Vec::with_capacity(V1_MAX_LEN);
    loop {
        if line.len() >= V1_MAX_LEN {
            return Err(malformed("v1 line exceeds 107 bytes with no CRLF"));
        }
        let mut byte = [0u8; 1];
        stream.read_exact(&mut byte).await?;
        line.push(byte[0]);
        if line.ends_with(b"\r\n") {
            break;
        }
    }
    parse_v1(&line[..line.len() - 2])
}

/// Parse a v1 line with its CRLF already stripped. Fields are single-space
/// separated by the spec, so the split is exact rather than whitespace-tolerant.
fn parse_v1(line: &[u8]) -> io::Result<Option<SocketAddr>> {
    let text = std::str::from_utf8(line).map_err(|_| malformed("v1 line is not ASCII"))?;
    let mut fields = text.split(' ');
    if fields.next() != Some("PROXY") {
        return Err(malformed("v1 line does not start with PROXY"));
    }
    let v6 = match fields.next() {
        Some("TCP4") => false,
        Some("TCP6") => true,
        // The proxy is telling us it does not know the origin (health check, or a
        // protocol it does not understand). Valid; the peer address stands.
        Some("UNKNOWN") => return Ok(None),
        _ => return Err(malformed("v1 protocol field is not TCP4/TCP6/UNKNOWN")),
    };
    let (Some(src), Some(dst), Some(sport), Some(dport), None) = (
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
    ) else {
        return Err(malformed("v1 line does not have exactly six fields"));
    };
    let src = parse_v1_addr(src, v6)?;
    // The destination is validated but unused: a header whose halves disagree on
    // family is malformed, and accepting it would mean accepting a parser we do not
    // actually agree with the sender about.
    let _dst = parse_v1_addr(dst, v6)?;
    let sport: u16 = sport
        .parse()
        .map_err(|_| malformed("v1 source port is not a number"))?;
    let _dport: u16 = dport
        .parse()
        .map_err(|_| malformed("v1 destination port is not a number"))?;
    Ok(Some(SocketAddr::new(src, sport)))
}

/// Parse one v1 address, rejecting a family that contradicts the `TCP4`/`TCP6` tag.
fn parse_v1_addr(raw: &str, v6: bool) -> io::Result<IpAddr> {
    let addr: IpAddr = raw
        .parse()
        .map_err(|_| malformed("v1 address does not parse"))?;
    if addr.is_ipv6() != v6 {
        return Err(malformed(
            "v1 address family contradicts the protocol field",
        ));
    }
    Ok(addr)
}

/// Consume a v2 header. `Ok(None)` means the header was valid but names no usable
/// client address (`LOCAL`, `AF_UNSPEC`, `AF_UNIX`).
async fn read_v2(stream: &mut TcpStream) -> io::Result<Option<SocketAddr>> {
    let mut head = [0u8; 16];
    stream.read_exact(&mut head).await?;
    // The length is trusted only to the extent that u16 bounds it; the block is
    // consumed whatever it holds, so the stream stays framed for TLS behind it.
    let len = usize::from(u16::from_be_bytes([head[14], head[15]]));
    let mut body = vec![0u8; len];
    stream.read_exact(&mut body).await?;
    parse_v2(&head, &body)
}

/// Parse an already-read v2 header and address block.
fn parse_v2(head: &[u8; 16], body: &[u8]) -> io::Result<Option<SocketAddr>> {
    if head[..12] != V2_SIGNATURE {
        return Err(malformed("v2 signature mismatch"));
    }
    if head[12] >> 4 != 0x2 {
        return Err(malformed("v2 version field is not 2"));
    }
    match head[12] & 0x0f {
        // LOCAL: the balancer's own connection (a health check). By the spec the
        // address block must be ignored, so the peer address is the answer.
        0x0 => return Ok(None),
        0x1 => {}
        _ => return Err(malformed("v2 command is not LOCAL or PROXY")),
    }
    // The low nibble of byte 13 is the transport (STREAM/DGRAM). It is read but not
    // enforced: nothing here depends on it, and rejecting an unexpected value would
    // refuse senders for a fact we do not use.
    match head[13] >> 4 {
        // AF_INET: 4 + 4 address bytes then two ports.
        0x1 => {
            let b: &[u8; 12] = body
                .get(..12)
                .and_then(|s| s.try_into().ok())
                .ok_or_else(|| malformed("v2 AF_INET block is shorter than 12 bytes"))?;
            let src = Ipv4Addr::new(b[0], b[1], b[2], b[3]);
            let sport = u16::from_be_bytes([b[8], b[9]]);
            Ok(Some(SocketAddr::new(IpAddr::V4(src), sport)))
        }
        // AF_INET6: 16 + 16 address bytes then two ports.
        0x2 => {
            let b: &[u8; 36] = body
                .get(..36)
                .and_then(|s| s.try_into().ok())
                .ok_or_else(|| malformed("v2 AF_INET6 block is shorter than 36 bytes"))?;
            let mut src = [0u8; 16];
            src.copy_from_slice(&b[..16]);
            let sport = u16::from_be_bytes([b[32], b[33]]);
            Ok(Some(SocketAddr::new(
                IpAddr::V6(Ipv6Addr::from(src)),
                sport,
            )))
        }
        // AF_UNSPEC and AF_UNIX carry nothing this server can use as a client IP.
        // Both are valid headers, so the connection is served from its peer address
        // rather than dropped.
        0x0 | 0x3 => Ok(None),
        _ => Err(malformed("v2 address family is not INET/INET6/UNSPEC/UNIX")),
    }
}

// ---------------------------------------------------------------------------
// Negotiation
// ---------------------------------------------------------------------------

/// Why a connection was dropped before it reached the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Refusal {
    /// A PROXY header from a peer that is not a configured proxy — a forged client
    /// address, or an app port reachable by something that should not reach it.
    UntrustedSender,
    /// `require` and no header arrived: this connection did not come through the
    /// balancer.
    MissingHeader,
    /// A header that did not parse, or ended early.
    Malformed,
}

impl Refusal {
    fn reason(self) -> &'static str {
        match self {
            Self::UntrustedSender => {
                "PROXY header from a peer outside MW_TRUSTED_PROXIES; connection dropped"
            }
            Self::MissingHeader => {
                "MW_PROXY_PROTOCOL=require but no PROXY header; connection dropped"
            }
            Self::Malformed => "unparseable PROXY header; connection dropped",
        }
    }
}

/// Decide a connection's client address, consuming a PROXY header if one is there
/// and may be believed. `Err` means the connection must be closed.
async fn negotiate(
    cfg: &Config,
    stream: &mut TcpStream,
    peer: SocketAddr,
) -> Result<SocketAddr, Refusal> {
    if cfg.mode == Mode::Off {
        return Ok(peer);
    }
    let kind = detect(stream).await;
    if kind == Kind::Absent {
        // Includes a peer that opened a connection and said nothing: in `require`
        // that is exactly the case worth dropping.
        return match cfg.mode {
            Mode::Require => Err(Refusal::MissingHeader),
            _ => Ok(peer),
        };
    }
    // A header is present. Whether it may be believed is decided by who is sending
    // it, before a single byte of it is consumed.
    if !cfg.trusts(peer) {
        return Err(Refusal::UntrustedSender);
    }
    let parsed = match kind {
        Kind::V1 => read_v1(stream).await,
        _ => read_v2(stream).await,
    };
    match parsed {
        // A header that names no client (UNKNOWN/LOCAL/UNSPEC/UNIX) leaves the peer
        // address in place, which for all four is the true origin of the connection.
        Ok(None) => Ok(peer),
        Ok(Some(client)) => Ok(client),
        Err(e) => {
            tracing::debug!("proxy-protocol parse from {peer}: {e}");
            Err(Refusal::Malformed)
        }
    }
}

// ---------------------------------------------------------------------------
// The listener
// ---------------------------------------------------------------------------

/// A TCP listener that resolves each connection's client address before handing it
/// on, and closes connections that fail the posture in [`Config`].
///
/// Implements [`axum::serve::Listener`] so the plaintext path serves through it
/// directly; [`crate::tls::TlsListener`] holds one so the header is read before the
/// TLS handshake it fronts.
pub struct ProxyAcceptor {
    conns: mpsc::Receiver<(TcpStream, SocketAddr)>,
    local: SocketAddr,
    task: JoinHandle<()>,
}

impl ProxyAcceptor {
    /// Take over `tcp` and start negotiating connections. Must be called from within
    /// a tokio runtime.
    pub fn new(tcp: TcpListener, cfg: Config) -> io::Result<Self> {
        let local = tcp.local_addr()?;
        let (tx, conns) = mpsc::channel(ACCEPT_BACKLOG);
        let task = tokio::spawn(accept_loop(tcp, cfg, tx));
        Ok(Self { conns, local, task })
    }

    /// The next connection, with its client address resolved. Refused connections
    /// never appear here — they are closed inside the accept loop.
    pub async fn next_conn(&mut self) -> (TcpStream, SocketAddr) {
        match self.conns.recv().await {
            Some(pair) => pair,
            // The accept loop only ends when this acceptor is dropped, so this is
            // unreachable in practice. Stay pending rather than inventing a
            // connection: `accept` has no way to report failure.
            None => std::future::pending().await,
        }
    }

    /// The bound address, captured at construction (the loop owns the listener).
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        Ok(self.local)
    }
}

impl Drop for ProxyAcceptor {
    fn drop(&mut self) {
        // Stop accepting when the server stops serving; without this the loop would
        // outlive a graceful shutdown and keep the port open.
        self.task.abort();
    }
}

impl axum::serve::Listener for ProxyAcceptor {
    type Io = TcpStream;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        self.next_conn().await
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        ProxyAcceptor::local_addr(self)
    }
}

/// Accept forever, negotiating each connection off the accept path.
async fn accept_loop(tcp: TcpListener, cfg: Config, tx: mpsc::Sender<(TcpStream, SocketAddr)>) {
    loop {
        let (stream, peer) = match tcp.accept().await {
            Ok(pair) => pair,
            Err(e) => {
                // Transient (fd exhaustion, a connection reset between the SYN and
                // the accept). Back off briefly rather than spinning on the error.
                tracing::debug!("tcp accept error: {e}");
                tokio::time::sleep(Duration::from_millis(50)).await;
                continue;
            }
        };
        if cfg.is_off() {
            if tx.send((stream, peer)).await.is_err() {
                return;
            }
            continue;
        }
        let cfg = cfg.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut stream = stream;
            match negotiate(&cfg, &mut stream, peer).await {
                Ok(client) => {
                    let _ = tx.send((stream, client)).await;
                }
                Err(refusal) => {
                    // Dropping the stream closes the connection. The peer address is
                    // logged because it is the only address here that is not a claim.
                    tracing::warn!("{}: peer {peer}", refusal.reason());
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::io::AsyncWriteExt;

    fn sa(s: &str) -> SocketAddr {
        s.parse().unwrap()
    }

    /// A v2 header: signature, ver_cmd, fam_proto, length, body.
    fn v2(ver_cmd: u8, fam_proto: u8, body: &[u8]) -> Vec<u8> {
        let mut out = V2_SIGNATURE.to_vec();
        out.push(ver_cmd);
        out.push(fam_proto);
        out.extend_from_slice(&(body.len() as u16).to_be_bytes());
        out.extend_from_slice(body);
        out
    }

    /// AF_INET address block: src, dst, sport, dport.
    fn v4_block(src: [u8; 4], dst: [u8; 4], sport: u16, dport: u16) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&src);
        b.extend_from_slice(&dst);
        b.extend_from_slice(&sport.to_be_bytes());
        b.extend_from_slice(&dport.to_be_bytes());
        b
    }

    // -- classification ----------------------------------------------------

    #[test]
    fn classifies_openings() {
        assert_eq!(classify(b"PROXY TCP4 1.2.3.4"), Kind::V1);
        assert_eq!(classify(&V2_SIGNATURE), Kind::V2);

        // Ordinary traffic is ruled out as early as the bytes allow, so a normal
        // request never waits for the timeout.
        assert_eq!(classify(b"GET / HTTP/1.1"), Kind::Absent);
        assert_eq!(classify(b"GET"), Kind::Absent);
        // A TLS ClientHello starts 0x16 0x03 — neither signature.
        assert_eq!(classify(&[0x16, 0x03, 0x01, 0x02, 0x00]), Kind::Absent);
        // `PROXY` without its trailing space is not v1 and cannot become it.
        assert_eq!(classify(b"PROXYX"), Kind::Absent);

        // Too few bytes to decide: keep waiting rather than guessing either way.
        assert_eq!(classify(b""), Kind::Partial);
        assert_eq!(classify(b"PROX"), Kind::Partial);
        assert_eq!(classify(&V2_SIGNATURE[..5]), Kind::Partial);
    }

    // -- v1 ----------------------------------------------------------------

    #[test]
    fn parses_v1_addresses() {
        assert_eq!(
            parse_v1(b"PROXY TCP4 198.51.100.7 10.0.0.1 51234 443").unwrap(),
            Some(sa("198.51.100.7:51234"))
        );
        assert_eq!(
            parse_v1(b"PROXY TCP6 2001:db8::7 2001:db8::1 51234 443").unwrap(),
            Some(sa("[2001:db8::7]:51234"))
        );
        // UNKNOWN is valid and names nobody: the peer address stands.
        assert_eq!(parse_v1(b"PROXY UNKNOWN").unwrap(), None);
        assert_eq!(
            parse_v1(b"PROXY UNKNOWN 198.51.100.7 10.0.0.1 51234 443").unwrap(),
            None
        );
    }

    #[test]
    fn rejects_malformed_v1() {
        for bad in [
            &b"PROXY TCP4 198.51.100.7 10.0.0.1 51234"[..],
            b"PROXY TCP4 198.51.100.7 10.0.0.1 51234 443 extra",
            b"PROXY TCP5 198.51.100.7 10.0.0.1 51234 443",
            b"HELLO TCP4 198.51.100.7 10.0.0.1 51234 443",
            b"PROXY TCP4 not-an-ip 10.0.0.1 51234 443",
            b"PROXY TCP4 198.51.100.7 10.0.0.1 nope 443",
            b"PROXY TCP4 198.51.100.7 10.0.0.1 99999 443",
            // A family that contradicts the protocol field: the sender and this
            // parser disagree about the header, so it is not believed.
            b"PROXY TCP4 2001:db8::7 2001:db8::1 51234 443",
            b"PROXY TCP6 198.51.100.7 10.0.0.1 51234 443",
            // Two spaces: the spec's separator is exactly one.
            b"PROXY  TCP4 198.51.100.7 10.0.0.1 51234 443",
        ] {
            assert!(
                parse_v1(bad).is_err(),
                "should reject {:?}",
                String::from_utf8_lossy(bad)
            );
        }
    }

    #[tokio::test]
    async fn reads_a_v1_header_without_eating_what_follows() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let send = tokio::spawn(async move {
            let mut s = TcpStream::connect(addr).await.unwrap();
            s.write_all(b"PROXY TCP4 198.51.100.7 10.0.0.1 51234 443\r\nGET / HTTP/1.1\r\n")
                .await
                .unwrap();
            s.flush().await.unwrap();
            // Hold the connection open until the reader is done.
            tokio::time::sleep(Duration::from_millis(200)).await;
        });
        let (mut sock, _) = listener.accept().await.unwrap();
        let client_addr = read_v1(&mut sock).await.unwrap();
        assert_eq!(client_addr, Some(sa("198.51.100.7:51234")));

        // Exactly the header was consumed: the request line is intact behind it.
        let mut rest = [0u8; 16];
        sock.read_exact(&mut rest).await.unwrap();
        assert_eq!(&rest, b"GET / HTTP/1.1\r\n");
        send.await.unwrap();
    }

    // -- v2 ----------------------------------------------------------------

    #[test]
    fn parses_v2_addresses() {
        let h = v2(
            0x21,
            0x11,
            &v4_block([198, 51, 100, 7], [10, 0, 0, 1], 51234, 443),
        );
        let head: &[u8; 16] = h[..16].try_into().unwrap();
        assert_eq!(
            parse_v2(head, &h[16..]).unwrap(),
            Some(sa("198.51.100.7:51234"))
        );

        let mut body = Vec::new();
        body.extend_from_slice(&"2001:db8::7".parse::<Ipv6Addr>().unwrap().octets());
        body.extend_from_slice(&"2001:db8::1".parse::<Ipv6Addr>().unwrap().octets());
        body.extend_from_slice(&51234u16.to_be_bytes());
        body.extend_from_slice(&443u16.to_be_bytes());
        let h = v2(0x21, 0x21, &body);
        let head: &[u8; 16] = h[..16].try_into().unwrap();
        assert_eq!(
            parse_v2(head, &h[16..]).unwrap(),
            Some(sa("[2001:db8::7]:51234"))
        );
    }

    #[test]
    fn v2_tlvs_are_consumed_but_not_interpreted() {
        let mut body = v4_block([198, 51, 100, 7], [10, 0, 0, 1], 51234, 443);
        // PP2_TYPE_AUTHORITY: type byte, 2-byte length, value.
        body.extend_from_slice(&[0x02, 0x00, 0x04]);
        body.extend_from_slice(b"mail");
        let h = v2(0x21, 0x11, &body);
        let head: &[u8; 16] = h[..16].try_into().unwrap();
        assert_eq!(
            parse_v2(head, &h[16..]).unwrap(),
            Some(sa("198.51.100.7:51234")),
            "a TLV after the address block must not disturb the address"
        );
    }

    #[test]
    fn v2_local_and_unspec_fall_back_to_the_peer() {
        // LOCAL (cmd 0x0): a balancer health-checking us on its own behalf. The
        // address block is present and must be ignored, not believed.
        let h = v2(0x20, 0x11, &v4_block([1, 2, 3, 4], [10, 0, 0, 1], 1, 2));
        let head: &[u8; 16] = h[..16].try_into().unwrap();
        assert_eq!(parse_v2(head, &h[16..]).unwrap(), None);

        // AF_UNSPEC and AF_UNIX name nothing usable, and are not errors.
        let h = v2(0x21, 0x00, &[]);
        let head: &[u8; 16] = h[..16].try_into().unwrap();
        assert_eq!(parse_v2(head, &h[16..]).unwrap(), None);
        let h = v2(0x21, 0x31, &[0u8; 216]);
        let head: &[u8; 16] = h[..16].try_into().unwrap();
        assert_eq!(parse_v2(head, &h[16..]).unwrap(), None);
    }

    #[test]
    fn rejects_malformed_v2() {
        // Version nibble other than 2.
        let h = v2(0x11, 0x11, &v4_block([1, 2, 3, 4], [10, 0, 0, 1], 1, 2));
        let head: &[u8; 16] = h[..16].try_into().unwrap();
        assert!(parse_v2(head, &h[16..]).is_err());

        // Command other than LOCAL/PROXY.
        let h = v2(0x27, 0x11, &v4_block([1, 2, 3, 4], [10, 0, 0, 1], 1, 2));
        let head: &[u8; 16] = h[..16].try_into().unwrap();
        assert!(parse_v2(head, &h[16..]).is_err());

        // Truncated address block: a short read must not be read as a short address.
        let h = v2(0x21, 0x11, &[1, 2, 3, 4]);
        let head: &[u8; 16] = h[..16].try_into().unwrap();
        assert!(parse_v2(head, &h[16..]).is_err());
        let h = v2(0x21, 0x21, &[0u8; 20]);
        let head: &[u8; 16] = h[..16].try_into().unwrap();
        assert!(parse_v2(head, &h[16..]).is_err());

        // Unknown address family.
        let h = v2(0x21, 0x51, &[0u8; 12]);
        let head: &[u8; 16] = h[..16].try_into().unwrap();
        assert!(parse_v2(head, &h[16..]).is_err());

        // Signature mismatch (this is what `classify` screens for, asserted here so
        // the parser does not depend on having been screened).
        let mut h = v2(0x21, 0x11, &v4_block([1, 2, 3, 4], [10, 0, 0, 1], 1, 2));
        h[3] = 0x00;
        let head: &[u8; 16] = h[..16].try_into().unwrap();
        assert!(parse_v2(head, &h[16..]).is_err());
    }

    // -- mode / config -----------------------------------------------------

    #[test]
    fn mode_parses_conservatively() {
        assert_eq!(Mode::parse("accept"), Mode::Accept);
        assert_eq!(Mode::parse(" REQUIRE "), Mode::Require);
        assert_eq!(Mode::parse("off"), Mode::Off);
        assert_eq!(Mode::parse(""), Mode::Off);
        assert_eq!(Mode::parse("1"), Mode::Off);
        assert_eq!(Mode::parse("true"), Mode::Off);
        assert_eq!(Mode::default(), Mode::Off);
        assert!(Config::default().is_off());
    }

    #[test]
    fn trust_uses_the_shared_cidr_set() {
        let cfg = Config::new(Mode::Require, "10.0.0.0/8, 2001:db8::/32");
        assert!(cfg.trusts(sa("10.1.2.3:5")));
        assert!(cfg.trusts(sa("[2001:db8::9]:5")));
        assert!(!cfg.trusts(sa("198.51.100.7:5")));
        // A dual-stack listener reports a v4 peer mapped; it still matches.
        assert!(cfg.trusts(sa("[::ffff:10.1.2.3]:5")));
        // An unusable list trusts nobody rather than everybody.
        assert!(!Config::new(Mode::Require, "").trusts(sa("10.1.2.3:5")));
        assert!(!Config::new(Mode::Require, "nonsense").trusts(sa("10.1.2.3:5")));
    }

    // -- negotiation (the security boundary) -------------------------------

    /// Run `negotiate` against a real socket pair with `first` already written by
    /// the client, presenting `peer` as the socket peer.
    async fn negotiated(
        cfg: &Config,
        first: &[u8],
        peer: SocketAddr,
    ) -> Result<SocketAddr, Refusal> {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let payload = first.to_vec();
        let send = tokio::spawn(async move {
            let mut s = TcpStream::connect(addr).await.unwrap();
            s.write_all(&payload).await.unwrap();
            s.flush().await.unwrap();
            tokio::time::sleep(Duration::from_millis(300)).await;
        });
        let (mut sock, _) = listener.accept().await.unwrap();
        let out = negotiate(cfg, &mut sock, peer).await;
        send.abort();
        out
    }

    const V1_LINE: &[u8] = b"PROXY TCP4 198.51.100.7 10.0.0.1 51234 443\r\n";

    #[tokio::test]
    async fn off_never_reads_the_header() {
        let cfg = Config::new(Mode::Off, "10.0.0.0/8");
        // Even from a trusted peer, and even though the header is right there.
        assert_eq!(
            negotiated(&cfg, V1_LINE, sa("10.0.0.1:4000"))
                .await
                .unwrap(),
            sa("10.0.0.1:4000")
        );
    }

    #[tokio::test]
    async fn an_untrusted_sender_is_dropped_not_believed() {
        for mode in [Mode::Accept, Mode::Require] {
            let cfg = Config::new(mode, "10.0.0.0/8");
            // THE ATTACK: a direct client forges a header naming someone else.
            assert_eq!(
                negotiated(&cfg, V1_LINE, sa("198.51.100.9:4000")).await,
                Err(Refusal::UntrustedSender),
                "{mode:?}: a forged v1 header must drop the connection"
            );
            // Same in v2, and the address it names is never reached.
            let h = v2(
                0x21,
                0x11,
                &v4_block([198, 51, 100, 7], [10, 0, 0, 1], 1, 2),
            );
            assert_eq!(
                negotiated(&cfg, &h, sa("198.51.100.9:4000")).await,
                Err(Refusal::UntrustedSender),
                "{mode:?}: a forged v2 header must drop the connection"
            );
            // Not even a header naming an address inside the trusted range.
            let inside = b"PROXY TCP4 10.9.9.9 10.0.0.1 51234 443\r\n";
            assert_eq!(
                negotiated(&cfg, inside, sa("198.51.100.9:4000")).await,
                Err(Refusal::UntrustedSender)
            );
        }
    }

    #[tokio::test]
    async fn a_trusted_sender_is_believed() {
        let cfg = Config::new(Mode::Require, "10.0.0.0/8");
        assert_eq!(
            negotiated(&cfg, V1_LINE, sa("10.0.0.1:4000"))
                .await
                .unwrap(),
            sa("198.51.100.7:51234")
        );
        let h = v2(
            0x21,
            0x11,
            &v4_block([198, 51, 100, 7], [10, 0, 0, 1], 51234, 443),
        );
        assert_eq!(
            negotiated(&cfg, &h, sa("10.0.0.1:4000")).await.unwrap(),
            sa("198.51.100.7:51234")
        );
    }

    #[tokio::test]
    async fn accept_serves_a_direct_client_and_require_drops_it() {
        let request = b"GET / HTTP/1.1\r\nHost: x\r\n\r\n";

        let cfg = Config::new(Mode::Accept, "10.0.0.0/8");
        // No header from an untrusted peer: an ordinary direct client.
        assert_eq!(
            negotiated(&cfg, request, sa("198.51.100.9:4000"))
                .await
                .unwrap(),
            sa("198.51.100.9:4000")
        );
        // No header from a trusted peer either — `accept` means optional.
        assert_eq!(
            negotiated(&cfg, request, sa("10.0.0.1:4000"))
                .await
                .unwrap(),
            sa("10.0.0.1:4000")
        );

        let cfg = Config::new(Mode::Require, "10.0.0.0/8");
        assert_eq!(
            negotiated(&cfg, request, sa("198.51.100.9:4000")).await,
            Err(Refusal::MissingHeader)
        );
        // Behind an L4 balancer, a headerless connection from the balancer's own
        // address did not come through the balancer's proxy path.
        assert_eq!(
            negotiated(&cfg, request, sa("10.0.0.1:4000")).await,
            Err(Refusal::MissingHeader)
        );
    }

    #[tokio::test]
    async fn a_malformed_header_from_a_trusted_sender_is_dropped() {
        let cfg = Config::new(Mode::Require, "10.0.0.0/8");
        assert_eq!(
            negotiated(&cfg, b"PROXY TCP4 bogus\r\n", sa("10.0.0.1:4000")).await,
            Err(Refusal::Malformed)
        );
        // Truncated v2: the declared length never arrives, so the read ends short.
        let mut h = v2(
            0x21,
            0x11,
            &v4_block([198, 51, 100, 7], [10, 0, 0, 1], 1, 2),
        );
        h.truncate(20);
        assert_eq!(
            negotiated(&cfg, &h, sa("10.0.0.1:4000")).await,
            Err(Refusal::Malformed)
        );
    }

    #[tokio::test]
    async fn an_unconfigured_trust_list_admits_nobody() {
        // `require` with no trusted network is a misconfiguration that fails closed:
        // a header cannot be believed, so no connection qualifies.
        let cfg = Config::new(Mode::Require, "");
        assert_eq!(
            negotiated(&cfg, V1_LINE, sa("10.0.0.1:4000")).await,
            Err(Refusal::UntrustedSender)
        );
    }

    // -- end to end through axum ------------------------------------------

    /// Serve a one-route app through a [`ProxyAcceptor`] in `require` mode, echoing
    /// whatever address `ConnectInfo` ended up holding.
    async fn spawn_echo_peer(trusted: &str) -> SocketAddr {
        use axum::extract::ConnectInfo;
        use axum::serve::ListenerExt;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = axum::Router::new().route(
            "/",
            axum::routing::get(|ConnectInfo(who): ConnectInfo<SocketAddr>| async move {
                who.to_string()
            }),
        );
        let acceptor = ProxyAcceptor::new(listener, Config::new(Mode::Require, trusted)).unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(
                acceptor.tap_io(|_| {}),
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await;
        });
        addr
    }

    fn v2_then_get() -> Vec<u8> {
        let mut req = v2(
            0x21,
            0x11,
            &v4_block([198, 51, 100, 7], [10, 0, 0, 1], 51234, 443),
        );
        req.extend_from_slice(b"GET / HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n");
        req
    }

    #[tokio::test]
    async fn a_believed_header_reaches_connect_info() {
        // The whole point of the lane: the address downstream code reads is the one
        // the balancer reported, not the socket peer. Loopback is the trusted proxy
        // here, so this connection is what `send-proxy-v2` looks like on the wire.
        let addr = spawn_echo_peer("127.0.0.0/8").await;
        let mut sock = TcpStream::connect(addr).await.unwrap();
        sock.write_all(&v2_then_get()).await.unwrap();
        let mut response = String::new();
        sock.read_to_string(&mut response).await.unwrap();
        assert!(
            response.contains("198.51.100.7:51234"),
            "ConnectInfo should hold the header's client address, got: {response}"
        );
    }

    #[tokio::test]
    async fn a_forged_header_gets_no_response_at_all() {
        // Same bytes, but loopback is now outside the trusted list — which is what a
        // direct client sending a hand-written PROXY header looks like. The
        // connection is closed before the request is parsed, so there is no reply to
        // read and nothing downstream ever sees the claimed address.
        let addr = spawn_echo_peer("10.0.0.0/8").await;
        let mut sock = TcpStream::connect(addr).await.unwrap();
        // The write may itself fail once the server has closed the socket.
        let _ = sock.write_all(&v2_then_get()).await;
        let mut response = String::new();
        let _ = sock.read_to_string(&mut response).await;
        assert!(
            response.is_empty(),
            "a forged header must be dropped, not served: {response}"
        );
    }

    #[tokio::test]
    async fn a_silent_peer_does_not_stall_the_listener() {
        // Two connections, the second silent: it must not delay the first, because
        // negotiation happens off the accept path.
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let mut acceptor =
            ProxyAcceptor::new(listener, Config::new(Mode::Accept, "127.0.0.0/8")).unwrap();

        let silent = TcpStream::connect(addr).await.unwrap();
        let mut talker = TcpStream::connect(addr).await.unwrap();
        talker.write_all(V1_LINE).await.unwrap();
        talker.flush().await.unwrap();

        // HEADER_TIMEOUT is 5 s; anything under it proves the silent peer was not
        // in the way.
        let (_sock, client) = tokio::time::timeout(Duration::from_secs(2), acceptor.next_conn())
            .await
            .expect("the talking peer must arrive without waiting for the silent one");
        assert_eq!(client, sa("198.51.100.7:51234"));
        drop(silent);
    }
}
