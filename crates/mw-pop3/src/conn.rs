//! POP3 transport: line framing, the command set, and TLS (implicit/STLS).
//!
//! [`Pop3Conn`] wraps a boxed async stream so the same code drives a plaintext
//! test socket, an implicit-TLS `:995` connection, and an `STLS`-upgraded
//! `:110` connection. All untrusted bytes flow through [`crate::proto`], which
//! is total, so this layer only concerns itself with framing and command
//! sequencing.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use mw_engine::backend::EngineError;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use crate::backend::{Pop3Auth, Pop3Config, TlsMode};
use crate::proto::{
    CapaInfo, Status, dot_unstuff, parse_capa, parse_list_body, parse_stat, parse_status,
    parse_uidl_body, trim_eol,
};
use crate::sasl;

/// Result alias local to the crate, matching the engine seam.
type Result<T> = mw_engine::backend::Result<T>;

/// Any bidirectional async byte stream the connection can run over.
///
/// Blanket-implemented for `TcpStream` and `tokio_rustls` streams, so a boxed
/// trait object erases the TLS/plaintext distinction after the handshake.
pub trait AsyncStream: AsyncRead + AsyncWrite + Send + Unpin {}
impl<T: AsyncRead + AsyncWrite + Send + Unpin> AsyncStream for T {}

fn transport(e: impl std::fmt::Display) -> EngineError {
    EngineError::Transport(e.to_string())
}

/// One live POP3 session (greeting consumed, ready for AUTHORIZATION/TRANSACTION).
pub struct Pop3Conn {
    stream: BufReader<Box<dyn AsyncStream>>,
    /// The channel binding captured at the TLS handshake, before the typed
    /// stream was boxed, as a `(cb-name, bytes)` pair: `tls-exporter` (RFC 9266)
    /// on TLS 1.3, `tls-server-end-point` (RFC 5929) on TLS 1.2. `None` on a
    /// plaintext transport. Enables the `SCRAM-SHA-256-PLUS` mechanism when the
    /// server also advertises it, binding the exchange to the negotiated type.
    channel_binding: Option<(&'static str, Vec<u8>)>,
}

/// Outcome of reading one line during a SASL `AUTH` exchange.
enum AuthStep {
    Ok,
    Err(String),
    Continue(String),
}

impl Pop3Conn {
    fn new(stream: Box<dyn AsyncStream>) -> Self {
        Self {
            stream: BufReader::new(stream),
            channel_binding: None,
        }
    }

    /// Like [`new`](Self::new) but carrying the `(cb-name, bytes)` channel
    /// binding captured at the TLS handshake (`None` for plaintext).
    fn with_binding(
        stream: Box<dyn AsyncStream>,
        channel_binding: Option<(&'static str, Vec<u8>)>,
    ) -> Self {
        Self {
            stream: BufReader::new(stream),
            channel_binding,
        }
    }

    /// Connect, run any TLS handshake, consume the greeting, and authenticate.
    pub async fn open(cfg: &Pop3Config) -> Result<Self> {
        let addr = (cfg.host.as_str(), cfg.port);
        let tcp = TcpStream::connect(addr).await.map_err(transport)?;
        tcp.set_nodelay(true).ok();

        let mut conn = match cfg.tls {
            TlsMode::Plain => {
                let mut c = Pop3Conn::new(Box::new(tcp));
                c.read_greeting().await?;
                c
            }
            TlsMode::Implicit => {
                let (tls, binding) = tls_connect(Box::new(tcp), &cfg.host).await?;
                let mut c = Pop3Conn::with_binding(tls, binding);
                c.read_greeting().await?;
                c
            }
            TlsMode::StartTls => {
                let mut c = Pop3Conn::new(Box::new(tcp));
                c.read_greeting().await?;
                c.stls().await?;
                let inner = c.into_inner()?;
                let (tls, binding) = tls_connect(inner, &cfg.host).await?;
                Pop3Conn::with_binding(tls, binding)
            }
        };
        conn.authenticate(cfg).await?;
        Ok(conn)
    }

    /// Recover the underlying stream for a TLS upgrade.
    ///
    /// Refuses if the buffer holds server bytes queued before the handshake —
    /// that would be a plaintext-injection (STARTTLS-stripping) attempt.
    fn into_inner(self) -> Result<Box<dyn AsyncStream>> {
        if !self.stream.buffer().is_empty() {
            return Err(EngineError::Protocol(
                "server sent data before STLS handshake".into(),
            ));
        }
        Ok(self.stream.into_inner())
    }

    // ---- line framing -----------------------------------------------------

    async fn read_line_raw(&mut self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        let n = self
            .stream
            .read_until(b'\n', &mut buf)
            .await
            .map_err(transport)?;
        if n == 0 {
            return Err(EngineError::Transport("connection closed by server".into()));
        }
        Ok(buf)
    }

    async fn read_status(&mut self) -> Result<Status> {
        let line = self.read_line_raw().await?;
        parse_status(&line)
    }

    /// Read a byte-stuffed multi-line body up to the `.` terminator, unstuffed.
    async fn read_multiline(&mut self) -> Result<Vec<u8>> {
        let mut raw = Vec::new();
        loop {
            let line = self.read_line_raw().await?;
            if trim_eol(&line) == b"." {
                break;
            }
            raw.extend_from_slice(&line);
        }
        Ok(dot_unstuff(&raw))
    }

    async fn send(&mut self, cmd: &str) -> Result<()> {
        let w = self.stream.get_mut();
        w.write_all(cmd.as_bytes()).await.map_err(transport)?;
        w.write_all(b"\r\n").await.map_err(transport)?;
        w.flush().await.map_err(transport)?;
        Ok(())
    }

    // ---- greeting / TLS / auth -------------------------------------------

    async fn read_greeting(&mut self) -> Result<()> {
        match self.read_status().await? {
            Status::Ok(_) => Ok(()),
            Status::Err(m) => Err(EngineError::Transport(format!(
                "server refused connection: {m}"
            ))),
        }
    }

    async fn stls(&mut self) -> Result<()> {
        self.send("STLS").await?;
        require_ok(self.read_status().await?)?;
        Ok(())
    }

    async fn authenticate(&mut self, cfg: &Pop3Config) -> Result<()> {
        match &cfg.auth {
            Pop3Auth::UserPass => {
                self.send(&format!("USER {}", cfg.username)).await?;
                self.require_ok_auth().await?;
                self.send(&format!("PASS {}", cfg.secret)).await?;
                self.require_ok_auth().await?;
            }
            Pop3Auth::SaslPlain => {
                let ir = sasl::plain(&cfg.username, &cfg.secret);
                self.send(&format!("AUTH PLAIN {ir}")).await?;
                self.require_ok_auth().await?;
            }
            Pop3Auth::SaslLogin => {
                let (u, p) = sasl::login(&cfg.username, &cfg.secret);
                self.send("AUTH LOGIN").await?;
                self.expect_continue().await?;
                self.send(&u).await?;
                self.expect_continue().await?;
                self.send(&p).await?;
                self.require_ok_auth().await?;
            }
            Pop3Auth::XOAuth2 => {
                let ir = sasl::xoauth2(&cfg.username, &cfg.secret);
                self.send(&format!("AUTH XOAUTH2 {ir}")).await?;
                match self.read_auth_step().await? {
                    AuthStep::Ok => {}
                    AuthStep::Err(m) => return Err(EngineError::Auth(m)),
                    AuthStep::Continue(_) => {
                        // Server returned a base64 error challenge; ack with an
                        // empty line and surface the follow-up failure.
                        self.send("").await?;
                        let msg = match self.read_status().await? {
                            Status::Err(m) => m,
                            Status::Ok(m) => m,
                        };
                        return Err(EngineError::Auth(msg));
                    }
                }
            }
            Pop3Auth::SaslScram => {
                // Prefer `-PLUS` only when we captured a channel binding at the
                // handshake AND the server advertises it (probe `CAPA` — a
                // no-op on the plaintext path since the binding is `None`). The
                // policy itself lives with the `Pop3Auth` enum in `backend.rs`.
                let use_plus = if self.channel_binding.is_some() {
                    let capa = self.capa().await?;
                    let binding = self.channel_binding.as_ref().map(|(_, b)| b.as_slice());
                    Pop3Auth::scram_prefers_plus(binding, &capa.sasl)
                } else {
                    false
                };
                self.authenticate_scram_sha256(&cfg.username, &cfg.secret, use_plus)
                    .await?;
            }
            Pop3Auth::OAuthBearer => {
                self.authenticate_oauthbearer(&cfg.username, &cfg.secret)
                    .await?;
            }
        }
        Ok(())
    }

    /// SASL `SCRAM-SHA-256`(`-PLUS`) (RFC 5802 / RFC 7677) over POP3 `AUTH`
    /// (RFC 5034), challenge/response form: `AUTH SCRAM-SHA-256[-PLUS]` → `+` →
    /// client-first → `+` server-first → client-final → server-final/`+OK`. The
    /// proof math is in [`crate::sasl::ScramSha256`] (pinned by the RFC 7677
    /// vector test).
    ///
    /// When `use_plus` is set, sends `AUTH SCRAM-SHA-256-PLUS` and binds the
    /// exchange to the `(cb-name, bytes)` binding captured at the handshake
    /// (`self.channel_binding`) — `tls-exporter` on TLS 1.3, `tls-server-end-point`
    /// on TLS 1.2; otherwise the plain `n,,` mechanism. The caller
    /// ([`authenticate`](Self::authenticate)) only sets `use_plus` when a
    /// binding is present and the server advertised the `-PLUS` mechanism.
    ///
    /// Selected via [`Pop3Auth::SaslScram`](crate::backend::Pop3Auth) on the
    /// config; [`authenticate`](Self::authenticate) dispatches here.
    pub async fn authenticate_scram_sha256(
        &mut self,
        username: &str,
        password: &str,
        use_plus: bool,
    ) -> Result<()> {
        let (mech, binding) = if use_plus {
            ("AUTH SCRAM-SHA-256-PLUS", self.channel_binding.clone())
        } else {
            ("AUTH SCRAM-SHA-256", None)
        };
        let nonce = sasl::client_nonce();
        let (mut scram, client_first) = sasl::ScramSha256::new(username, password, &nonce, binding);

        self.send(mech).await?;
        // First continuation asks for the client-first-message (payload empty).
        self.expect_continue().await?;
        self.send(&B64.encode(&client_first)).await?;

        // Second continuation carries the base64 server-first-message.
        let server_first = self.decode_continuation().await?;
        let client_final = scram
            .client_final(&server_first)
            .map_err(EngineError::Auth)?;
        self.send(&B64.encode(&client_final)).await?;

        match self.read_auth_step().await? {
            AuthStep::Ok => Ok(()),
            AuthStep::Err(m) => Err(EngineError::Auth(m)),
            AuthStep::Continue(c) => {
                // Server-final (v=) delivered as a final continuation: verify it,
                // acknowledge with an empty line, then read the +OK/-ERR.
                let server_final = decode_b64_utf8(&c)?;
                scram.verify(&server_final).map_err(EngineError::Auth)?;
                self.send("").await?;
                self.require_ok_auth().await
            }
        }
    }

    /// SASL `OAUTHBEARER` (RFC 7628) over POP3 `AUTH` with an inline initial
    /// response (mirrors the `XOAUTH2` path). On failure the server sends a
    /// continuation error challenge; the client acks with the `%x01` kvsep.
    ///
    /// Selected via [`Pop3Auth::OAuthBearer`](crate::backend::Pop3Auth) on the
    /// config; [`authenticate`](Self::authenticate) dispatches here.
    pub async fn authenticate_oauthbearer(&mut self, username: &str, token: &str) -> Result<()> {
        let ir = sasl::oauthbearer(username, token);
        self.send(&format!("AUTH OAUTHBEARER {ir}")).await?;
        match self.read_auth_step().await? {
            AuthStep::Ok => Ok(()),
            AuthStep::Err(m) => Err(EngineError::Auth(m)),
            AuthStep::Continue(_) => {
                self.send(&B64.encode("\x01")).await?;
                let msg = match self.read_status().await? {
                    Status::Err(m) | Status::Ok(m) => m,
                };
                Err(EngineError::Auth(msg))
            }
        }
    }

    /// Read a SASL continuation and decode its base64 payload to UTF-8.
    async fn decode_continuation(&mut self) -> Result<String> {
        let c = self.expect_continue().await?;
        decode_b64_utf8(&c)
    }

    async fn read_auth_step(&mut self) -> Result<AuthStep> {
        let line = self.read_line_raw().await?;
        let trimmed = trim_eol(&line);
        if trimmed.starts_with(b"+OK") {
            Ok(AuthStep::Ok)
        } else if trimmed.starts_with(b"-ERR") {
            Ok(AuthStep::Err(
                String::from_utf8_lossy(&trimmed[4..]).trim().to_string(),
            ))
        } else if let Some(rest) = trimmed.strip_prefix(b"+") {
            let rest = rest.strip_prefix(b" ").unwrap_or(rest);
            Ok(AuthStep::Continue(
                String::from_utf8_lossy(rest).into_owned(),
            ))
        } else {
            Err(EngineError::Protocol(format!(
                "unexpected AUTH response {:?}",
                String::from_utf8_lossy(trimmed)
            )))
        }
    }

    async fn expect_continue(&mut self) -> Result<String> {
        match self.read_auth_step().await? {
            AuthStep::Continue(c) => Ok(c),
            AuthStep::Err(m) => Err(EngineError::Auth(m)),
            AuthStep::Ok => Err(EngineError::Protocol("server ended AUTH early".into())),
        }
    }

    async fn require_ok_auth(&mut self) -> Result<()> {
        match self.read_status().await? {
            Status::Ok(_) => Ok(()),
            Status::Err(m) => Err(EngineError::Auth(m)),
        }
    }

    // ---- commands ---------------------------------------------------------

    /// `CAPA` (RFC 2449). A `-ERR`/absent CAPA yields empty capabilities.
    pub async fn capa(&mut self) -> Result<CapaInfo> {
        self.send("CAPA").await?;
        match self.read_status().await? {
            Status::Ok(_) => {
                let body = self.read_multiline().await?;
                Ok(parse_capa(&body))
            }
            Status::Err(_) => Ok(CapaInfo::default()),
        }
    }

    /// `STAT` → `(message-count, octet-total)`.
    pub async fn stat(&mut self) -> Result<(u64, u64)> {
        self.send("STAT").await?;
        let tail = require_ok(self.read_status().await?)?;
        parse_stat(&tail)
    }

    /// `UIDL` (no arg) → all `(msg-number, uidl)` pairs.
    pub async fn uidl_all(&mut self) -> Result<Vec<(u32, String)>> {
        self.send("UIDL").await?;
        require_ok(self.read_status().await?)?;
        let body = self.read_multiline().await?;
        Ok(parse_uidl_body(&body))
    }

    /// `LIST` (no arg) → all `(msg-number, octet-size)` pairs.
    pub async fn list_all(&mut self) -> Result<Vec<(u32, String)>> {
        self.send("LIST").await?;
        require_ok(self.read_status().await?)?;
        let body = self.read_multiline().await?;
        Ok(parse_list_body(&body))
    }

    /// `RETR n` → the full RFC822 message bytes (dot-unstuffed).
    pub async fn retr(&mut self, num: u32) -> Result<Vec<u8>> {
        self.send(&format!("RETR {num}")).await?;
        require_ok(self.read_status().await?)?;
        self.read_multiline().await
    }

    /// `TOP n lines` → headers plus `lines` body lines (dot-unstuffed).
    pub async fn top(&mut self, num: u32, lines: u32) -> Result<Vec<u8>> {
        self.send(&format!("TOP {num} {lines}")).await?;
        require_ok(self.read_status().await?)?;
        self.read_multiline().await
    }

    /// `DELE n` — mark for deletion (committed at `QUIT`).
    pub async fn dele(&mut self, num: u32) -> Result<()> {
        self.send(&format!("DELE {num}")).await?;
        require_ok(self.read_status().await?)?;
        Ok(())
    }

    /// `RSET` — unmark all deletions.
    pub async fn rset(&mut self) -> Result<()> {
        self.send("RSET").await?;
        require_ok(self.read_status().await?)?;
        Ok(())
    }

    /// `QUIT` — enter UPDATE state, committing any `DELE`s, then close.
    pub async fn quit(&mut self) -> Result<()> {
        self.send("QUIT").await?;
        require_ok(self.read_status().await?)?;
        Ok(())
    }
}

fn require_ok(status: Status) -> Result<String> {
    match status {
        Status::Ok(m) => Ok(m),
        Status::Err(m) => Err(EngineError::Protocol(format!("server said -ERR {m}"))),
    }
}

/// Decode a base64 SASL blob to its UTF-8 payload (SCRAM messages are text).
fn decode_b64_utf8(s: &str) -> Result<String> {
    let bytes = B64
        .decode(s.trim())
        .map_err(|e| EngineError::Protocol(format!("bad base64 SASL blob: {e}")))?;
    String::from_utf8(bytes).map_err(|e| EngineError::Protocol(format!("non-UTF-8 SASL blob: {e}")))
}

// ---------------------------------------------------------------------------
// Minimum TLS version
//
// One process-wide floor. rustls 0.23 speaks TLS 1.2 and 1.3 only, so it is a
// two-value choice. It defaults to `MinTls::V12`, which builds exactly the
// configuration this crate built before the floor existed. The floor is read each
// time a connection is wrapped, so a change applies to the next connection; a
// session already open keeps the version it negotiated.
// ---------------------------------------------------------------------------

/// The lowest TLS version an outbound POP3 connection may negotiate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinTls {
    /// TLS 1.2 or 1.3 (the default; rustls offers nothing older).
    V12,
    /// TLS 1.3 only. A server limited to TLS 1.2 is refused at the handshake.
    V13,
}

static MIN_TLS: AtomicU8 = AtomicU8::new(0);

/// Set the floor for every POP3 TLS connection opened from now on (implicit TLS
/// and STLS alike). Connections already established are not touched.
pub fn set_min_tls(v: MinTls) {
    MIN_TLS.store(matches!(v, MinTls::V13) as u8, Ordering::SeqCst);
}

/// The floor currently in force.
pub fn min_tls() -> MinTls {
    if MIN_TLS.load(Ordering::SeqCst) == 0 {
        MinTls::V12
    } else {
        MinTls::V13
    }
}

/// The start of the error text produced when a handshake fails because the peer
/// did not offer TLS 1.3 while the floor is [`MinTls::V13`]. It reaches the caller
/// as
/// `EngineError::Transport`.
pub const MIN_TLS_REFUSED: &str = "the minimum TLS version is set to 1.3";

/// Rewrite a handshake error that the floor explains. Under a 1.3 floor the
/// client offers TLS 1.3 only; a peer without it answers with a
/// `protocol_version` alert, or with a ServerHello rustls rejects as
/// incompatible. Every other error, and every error under the 1.2 floor, is
/// returned unchanged.
fn explain(e: std::io::Error, floor: MinTls) -> std::io::Error {
    if floor != MinTls::V13 {
        return e;
    }
    let caused_by_floor = matches!(
        e.get_ref()
            .and_then(|inner| inner.downcast_ref::<rustls::Error>()),
        Some(
            rustls::Error::AlertReceived(rustls::AlertDescription::ProtocolVersion)
                | rustls::Error::PeerIncompatible(_)
        )
    );
    if !caused_by_floor {
        return e;
    }
    std::io::Error::new(
        e.kind(),
        format!("{MIN_TLS_REFUSED} and the server did not offer it ({e})"),
    )
}

/// Run the TLS handshake and, **while the stream is still typed**, compute the
/// SCRAM channel binding — this is the one moment the negotiated
/// [`rustls::ClientConnection`] is reachable, before the stream is erased into
/// `Box<dyn AsyncStream>`. Returns the boxed stream alongside the
/// `(cb-name, bytes)` binding (see [`channel_binding`]). The server is verified
/// against the Mozilla webpki root set, at the floor currently in force
/// ([`min_tls`]).
async fn tls_connect(
    io: Box<dyn AsyncStream>,
    host: &str,
) -> Result<(Box<dyn AsyncStream>, Option<(&'static str, Vec<u8>)>)> {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    tls_connect_with_roots(io, host, roots).await
}

/// [`tls_connect`] with the trust anchors supplied. Private: the only caller
/// outside this module's tests is [`tls_connect`], which passes the webpki roots.
async fn tls_connect_with_roots(
    io: Box<dyn AsyncStream>,
    host: &str,
    roots: rustls::RootCertStore,
) -> Result<(Box<dyn AsyncStream>, Option<(&'static str, Vec<u8>)>)> {
    let floor = min_tls();
    let config = tls_client_config(roots, floor)?;
    let connector = TlsConnector::from(config);
    let server_name = rustls_pki_types::ServerName::try_from(host.to_string())
        .map_err(|_| EngineError::Transport(format!("invalid TLS server name: {host}")))?;
    let tls = connector
        .connect(server_name, io)
        .await
        .map_err(|e| transport(explain(e, floor)))?;
    let binding = {
        let (_, conn) = tls.get_ref();
        channel_binding(conn)
    };
    Ok((Box::new(tls), binding))
}

/// Compute the SCRAM `-PLUS` channel binding from the negotiated connection,
/// preferring `tls-exporter` (RFC 9266) on **TLS 1.3** and falling back to
/// `tls-server-end-point` (RFC 5929) on **TLS 1.2** — matching the reality of
/// current servers (Dovecot 2.4.x implements only `tls-unique`/`tls-exporter`)
/// and RFC 9266's TLS-1.3 scope. The type is not configured separately: it
/// follows the negotiated protocol version, so under a [`MinTls::V13`] floor it
/// is always `tls-exporter`.
///
/// On TLS 1.3 the binding is a 32-byte exporter keyed on the RFC 9266 label
/// `EXPORTER-Channel-Binding` with an empty context. On TLS 1.2 it is the
/// leaf certificate's `tls-server-end-point` digest. Returns `None` if neither
/// is derivable (no negotiated version / no peer certificate), which is not
/// expected for a completed TLS handshake.
fn channel_binding(conn: &rustls::ClientConnection) -> Option<(&'static str, Vec<u8>)> {
    match conn.protocol_version() {
        Some(rustls::ProtocolVersion::TLSv1_3) => {
            let out = [0u8; 32];
            let material = conn
                .export_keying_material(out, b"EXPORTER-Channel-Binding", Some(&[]))
                .ok()?;
            Some(("tls-exporter", material.to_vec()))
        }
        _ => conn
            .peer_certificates()
            .and_then(|certs| certs.first())
            .map(|leaf| {
                (
                    "tls-server-end-point",
                    sasl::tls_server_end_point(leaf.as_ref()),
                )
            }),
    }
}

/// The client configuration for `roots` at `floor`: explicit `ring` provider, no
/// client certificate. `MinTls::V12` is the rustls safe default (1.2 and 1.3).
fn tls_client_config(
    roots: rustls::RootCertStore,
    floor: MinTls,
) -> Result<Arc<rustls::ClientConfig>> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = rustls::ClientConfig::builder_with_provider(provider);
    let builder = match floor {
        MinTls::V12 => builder.with_safe_default_protocol_versions(),
        MinTls::V13 => builder.with_protocol_versions(&[&rustls::version::TLS13]),
    }
    .map_err(|e| EngineError::Transport(e.to_string()))?;
    let config = builder.with_root_certificates(roots).with_no_client_auth();
    Ok(Arc::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;

    /// STLS framing (RFC 2595): the client must send `STLS`, accept `+OK`, and
    /// leave no buffered bytes so the subsequent TLS handshake is injection-safe.
    /// The handshake itself is exercised only against live TLS servers.
    #[tokio::test]
    async fn stls_framing_and_injection_guard() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (sock, _) = listener.accept().await.unwrap();
            let (rd, mut wr) = sock.into_split();
            let mut reader = BufReader::new(rd);
            wr.write_all(b"+OK mailwoman ready\r\n").await.unwrap();
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            wr.write_all(b"+OK begin TLS negotiation\r\n")
                .await
                .unwrap();
            line.trim_end_matches(['\r', '\n']).to_string()
        });

        let tcp = TcpStream::connect(addr).await.unwrap();
        let mut conn = Pop3Conn::new(Box::new(tcp));
        conn.read_greeting().await.unwrap();
        conn.stls().await.unwrap();

        let sent = server.await.unwrap();
        assert_eq!(sent, "STLS");
        // No queued plaintext before the handshake -> upgrade would be safe.
        assert!(conn.into_inner().is_ok());
    }

    /// Drive a full `AUTH SCRAM-SHA-256` challenge/response exchange against a
    /// mock server that echoes the client nonce and accepts the proof. The proof
    /// math is pinned by the RFC 7677 vector test in `sasl`; this pins the POP3
    /// framing/dispatch (`AUTH` → `+` → client-first → `+` server-first →
    /// client-final → `+OK`).
    #[tokio::test]
    async fn scram_sha256_dispatch_framing() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (sock, _) = listener.accept().await.unwrap();
            let (rd, mut wr) = sock.into_split();
            let mut reader = BufReader::new(rd);
            wr.write_all(b"+OK mailwoman ready\r\n").await.unwrap();

            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            assert_eq!(line.trim_end_matches(['\r', '\n']), "AUTH SCRAM-SHA-256");
            wr.write_all(b"+ \r\n").await.unwrap();

            line.clear();
            reader.read_line(&mut line).await.unwrap();
            let client_first =
                String::from_utf8(B64.decode(line.trim_end_matches(['\r', '\n'])).unwrap())
                    .unwrap();
            assert!(client_first.starts_with("n,,n=user,r="), "{client_first}");
            let nonce = client_first
                .rsplit(',')
                .next()
                .and_then(|f| f.strip_prefix("r="))
                .unwrap()
                .to_string();
            let server_first = format!("r={nonce}SRV,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096");
            wr.write_all(format!("+ {}\r\n", B64.encode(&server_first)).as_bytes())
                .await
                .unwrap();

            line.clear();
            reader.read_line(&mut line).await.unwrap();
            let client_final =
                String::from_utf8(B64.decode(line.trim_end_matches(['\r', '\n'])).unwrap())
                    .unwrap();
            assert!(client_final.starts_with("c=biws,r=") && client_final.contains(",p="));
            wr.write_all(b"+OK maildrop locked and ready\r\n")
                .await
                .unwrap();
        });

        let tcp = TcpStream::connect(addr).await.unwrap();
        let mut conn = Pop3Conn::new(Box::new(tcp));
        conn.read_greeting().await.unwrap();
        conn.authenticate_scram_sha256("user", "pencil", false)
            .await
            .expect("SCRAM authentication succeeds");
        server.await.unwrap();
    }

    /// Drive `AUTH SCRAM-SHA-256-PLUS` against a mock server that asserts the
    /// `-PLUS` mechanism is sent, the client-first gs2-header advertises the
    /// given `cb_name`, and the client-final `c=` echoes the base64 of
    /// `p=<cb_name>,,` || the captured channel binding. The plain path is pinned
    /// by `scram_sha256_dispatch_framing`.
    async fn drive_scram_plus_dispatch(cb_name: &'static str, binding: Vec<u8>) {
        let expected_cbind = {
            let mut v = format!("p={cb_name},,").into_bytes();
            v.extend_from_slice(&binding);
            B64.encode(&v)
        };
        let gs2_prefix = format!("p={cb_name},,n=user,r=");
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let expected = expected_cbind.clone();
        let server = tokio::spawn(async move {
            let (sock, _) = listener.accept().await.unwrap();
            let (rd, mut wr) = sock.into_split();
            let mut reader = BufReader::new(rd);
            wr.write_all(b"+OK mailwoman ready\r\n").await.unwrap();

            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            assert_eq!(
                line.trim_end_matches(['\r', '\n']),
                "AUTH SCRAM-SHA-256-PLUS"
            );
            wr.write_all(b"+ \r\n").await.unwrap();

            line.clear();
            reader.read_line(&mut line).await.unwrap();
            let client_first =
                String::from_utf8(B64.decode(line.trim_end_matches(['\r', '\n'])).unwrap())
                    .unwrap();
            assert!(client_first.starts_with(&gs2_prefix), "{client_first}");
            let nonce = client_first
                .rsplit(',')
                .next()
                .and_then(|f| f.strip_prefix("r="))
                .unwrap()
                .to_string();
            let server_first = format!("r={nonce}SRV,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096");
            wr.write_all(format!("+ {}\r\n", B64.encode(&server_first)).as_bytes())
                .await
                .unwrap();

            line.clear();
            reader.read_line(&mut line).await.unwrap();
            let client_final =
                String::from_utf8(B64.decode(line.trim_end_matches(['\r', '\n'])).unwrap())
                    .unwrap();
            assert!(
                client_final.starts_with(&format!("c={expected},r=")),
                "c= must echo the channel binding: {client_final}"
            );
            assert!(client_final.contains(",p="));
            wr.write_all(b"+OK maildrop locked and ready\r\n")
                .await
                .unwrap();
        });

        let tcp = TcpStream::connect(addr).await.unwrap();
        let mut conn = Pop3Conn::with_binding(Box::new(tcp), Some((cb_name, binding)));
        conn.read_greeting().await.unwrap();
        conn.authenticate_scram_sha256("user", "pencil", true)
            .await
            .expect("SCRAM-SHA-256-PLUS authentication succeeds");
        server.await.unwrap();
    }

    /// TLS 1.2 fallback: `-PLUS` binds `tls-server-end-point` (RFC 5929).
    #[tokio::test]
    async fn scram_sha256_plus_dispatch_framing() {
        drive_scram_plus_dispatch("tls-server-end-point", vec![0x5Au8; 32]).await;
    }

    /// TLS 1.3 headline: `-PLUS` binds `tls-exporter` (RFC 9266). The gs2-header
    /// and the `c=` echo both carry `p=tls-exporter,,`, proving the cb-name
    /// threads through the full POP3 `AUTH` dispatch.
    #[tokio::test]
    async fn scram_sha256_plus_tls_exporter_dispatch_framing() {
        drive_scram_plus_dispatch("tls-exporter", vec![0x77u8; 32]).await;
    }

    /// `AUTH OAUTHBEARER` inline-IR dispatch: assert the RFC 7628 client
    /// response and that a `+OK` completes authentication.
    #[tokio::test]
    async fn oauthbearer_dispatch_framing() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (sock, _) = listener.accept().await.unwrap();
            let (rd, mut wr) = sock.into_split();
            let mut reader = BufReader::new(rd);
            wr.write_all(b"+OK mailwoman ready\r\n").await.unwrap();

            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let got = line.trim_end_matches(['\r', '\n']).to_string();
            let ir = got
                .strip_prefix("AUTH OAUTHBEARER ")
                .expect("AUTH OAUTHBEARER line");
            let decoded = B64.decode(ir).unwrap();
            assert_eq!(
                decoded,
                b"n,a=user@example.com,\x01auth=Bearer tok123\x01\x01"
            );
            wr.write_all(b"+OK maildrop locked and ready\r\n")
                .await
                .unwrap();
        });

        let tcp = TcpStream::connect(addr).await.unwrap();
        let mut conn = Pop3Conn::new(Box::new(tcp));
        conn.read_greeting().await.unwrap();
        conn.authenticate_oauthbearer("user@example.com", "tok123")
            .await
            .expect("OAUTHBEARER authentication succeeds");
        server.await.unwrap();
    }

    /// A self-signed certificate for `localhost` (P-256, valid to 2126), used only
    /// as the test peer's identity and as the single trust anchor of the test
    /// client. Not a secret.
    const CERT_PEM: &str = "-----BEGIN CERTIFICATE-----
MIIBojCCAUigAwIBAgIUXZfggy7a6Ac70pTuOuVMCuNzsNEwCgYIKoZIzj0EAwIw
HDEaMBgGA1UEAwwRbXctdGxzLWZsb29yLXRlc3QwIBcNMjYxMDA1MDAzMDA1WhgP
MjEyNjA5MTEwMDMwMDVaMBwxGjAYBgNVBAMMEW13LXRscy1mbG9vci10ZXN0MFkw
EwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAECjpBfVlabTNR7L8L59lw7vc/NaVVGmKn
1tAX8qOht2J5VmI1K0rbmOUevAO5cYmc52ATirpamxCpmne9BQExZaNmMGQwHQYD
VR0OBBYEFBFUy5FE3DLSfLMO3C8ZIQKQS736MB8GA1UdIwQYMBaAFBFUy5FE3DLS
fLMO3C8ZIQKQS736MBQGA1UdEQQNMAuCCWxvY2FsaG9zdDAMBgNVHRMBAf8EAjAA
MAoGCCqGSM49BAMCA0gAMEUCICMiQGvni0u2s5i7LpPBCkPD5bdvJuRtdareNrwa
QNGgAiEAoA+Jfh/SX0fNzYyti/9S7+BJ+t9hS1MrxabN7QUlVaA=
-----END CERTIFICATE-----
";
    const KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQg9JIIL1eAr3QLu3pR
51s9cIWRVry46AolZDGxmGbZs3OhRANCAAQKOkF9WVptM1Hsvwvn2XDu9z81pVUa
YqfW0Bfyo6G3YnlWYjUrStuY5R68A7lxiZznYBOKulqbEKmad70FATFl
-----END PRIVATE KEY-----
";

    fn test_roots() -> rustls::RootCertStore {
        use rustls_pki_types::pem::PemObject;
        let mut roots = rustls::RootCertStore::empty();
        roots
            .add(rustls_pki_types::CertificateDer::from_pem_slice(CERT_PEM.as_bytes()).unwrap())
            .unwrap();
        roots
    }

    /// The protocol versions of a peer that has not been upgraded past TLS 1.2.
    static ONLY_TLS12: &[&rustls::SupportedProtocolVersion] = &[&rustls::version::TLS12];

    /// A loopback TLS server that accepts only the given protocol versions. After
    /// each handshake it sends its own RFC 9266 exporter value (32 bytes; zeros
    /// when the session is not TLS 1.3) so the client's binding can be compared
    /// with what the server derived.
    async fn tls_peer(
        versions: &'static [&'static rustls::SupportedProtocolVersion],
    ) -> std::net::SocketAddr {
        use rustls_pki_types::pem::PemObject;
        use tokio::io::AsyncReadExt;
        let cert = rustls_pki_types::CertificateDer::from_pem_slice(CERT_PEM.as_bytes()).unwrap();
        let key = rustls_pki_types::PrivateKeyDer::from_pem_slice(KEY_PEM.as_bytes()).unwrap();
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = rustls::ServerConfig::builder_with_provider(provider)
            .with_protocol_versions(versions)
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .unwrap();
        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            while let Ok((tcp, _)) = listener.accept().await {
                let acceptor = acceptor.clone();
                tokio::spawn(async move {
                    if let Ok(mut tls) = acceptor.accept(tcp).await {
                        let mut exported = [0u8; 32];
                        let conn = tls.get_ref().1;
                        if conn.protocol_version() == Some(rustls::ProtocolVersion::TLSv1_3) {
                            exported = conn
                                .export_keying_material(
                                    exported,
                                    b"EXPORTER-Channel-Binding",
                                    Some(&[]),
                                )
                                .unwrap();
                        }
                        let _ = tls.write_all(&exported).await;
                        let mut sink = Vec::new();
                        let _ = tls.read_to_end(&mut sink).await;
                    }
                });
            }
        });
        addr
    }

    /// The channel binding of a session, and the exporter value its server sent.
    type Bound = (Option<(&'static str, Vec<u8>)>, [u8; 32]);

    /// One handshake through this module's `tls_connect` path at the floor in
    /// force.
    async fn upgrade(addr: std::net::SocketAddr) -> Result<Bound> {
        use tokio::io::AsyncReadExt;
        let tcp = TcpStream::connect(addr).await.unwrap();
        let (mut tls, binding) =
            tls_connect_with_roots(Box::new(tcp), "localhost", test_roots()).await?;
        let mut server_exported = [0u8; 32];
        tls.read_exact(&mut server_exported).await.unwrap();
        Ok((binding, server_exported))
    }

    /// The floor is process-wide, so every assertion that moves it lives in this
    /// one test.
    #[tokio::test]
    async fn a_tls13_floor_refuses_a_tls12_only_server_and_scram_plus_still_binds() {
        use rustls_pki_types::pem::PemObject;
        let only12 = tls_peer(ONLY_TLS12).await;
        let both = tls_peer(rustls::ALL_VERSIONS).await;
        let leaf = rustls_pki_types::CertificateDer::from_pem_slice(CERT_PEM.as_bytes()).unwrap();
        let end_point = sasl::tls_server_end_point(leaf.as_ref());

        // Precondition: unset, the floor is 1.2 and the 1.2-only peer is reachable
        // through this harness, binding to the certificate hash.
        assert_eq!(min_tls(), MinTls::V12);
        let (binding, _) = upgrade(only12).await.unwrap();
        assert_eq!(binding, Some(("tls-server-end-point", end_point.clone())));

        set_min_tls(MinTls::V13);
        assert_eq!(min_tls(), MinTls::V13);
        let Err(err) = upgrade(only12).await else {
            panic!("a 1.2-only server must be refused under a 1.3 floor");
        };
        let EngineError::Transport(text) = &err else {
            panic!("expected a transport error, got {err:?}");
        };
        assert!(
            text.starts_with(MIN_TLS_REFUSED),
            "the error names the floor: {text}"
        );

        // The floor refuses a version, not every server; and under it the
        // SCRAM-SHA-256-PLUS binding is the exporter value the server derived too.
        let (binding, server_exported) = upgrade(both).await.unwrap();
        assert_ne!(server_exported, [0u8; 32], "the peer negotiated TLS 1.3");
        assert_eq!(binding, Some(("tls-exporter", server_exported.to_vec())));

        // Lowering it again restores the connection.
        set_min_tls(MinTls::V12);
        let (binding, _) = upgrade(only12).await.unwrap();
        assert_eq!(binding, Some(("tls-server-end-point", end_point)));
    }

    #[test]
    fn an_unrelated_handshake_error_is_not_blamed_on_the_floor() {
        let cert_error = || {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                rustls::Error::InvalidCertificate(rustls::CertificateError::UnknownIssuer),
            )
        };
        let before = cert_error().to_string();
        assert_eq!(explain(cert_error(), MinTls::V13).to_string(), before);
        // Under the 1.2 floor nothing is rewritten, whatever the cause.
        let alert = std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            rustls::Error::AlertReceived(rustls::AlertDescription::ProtocolVersion),
        );
        assert!(
            !explain(alert, MinTls::V12)
                .to_string()
                .starts_with(MIN_TLS_REFUSED)
        );
    }
}
