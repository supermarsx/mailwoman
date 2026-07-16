//! POP3 transport: line framing, the command set, and TLS (implicit/STLS).
//!
//! [`Pop3Conn`] wraps a boxed async stream so the same code drives a plaintext
//! test socket, an implicit-TLS `:995` connection, and an `STLS`-upgraded
//! `:110` connection. All untrusted bytes flow through [`crate::proto`], which
//! is total, so this layer only concerns itself with framing and command
//! sequencing.

use std::sync::Arc;

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
                let tls = tls_connect(Box::new(tcp), &cfg.host).await?;
                let mut c = Pop3Conn::new(Box::new(tls));
                c.read_greeting().await?;
                c
            }
            TlsMode::StartTls => {
                let mut c = Pop3Conn::new(Box::new(tcp));
                c.read_greeting().await?;
                c.stls().await?;
                let inner = c.into_inner()?;
                let tls = tls_connect(inner, &cfg.host).await?;
                Pop3Conn::new(Box::new(tls))
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
                self.authenticate_scram_sha256(&cfg.username, &cfg.secret)
                    .await?;
            }
            Pop3Auth::OAuthBearer => {
                self.authenticate_oauthbearer(&cfg.username, &cfg.secret)
                    .await?;
            }
        }
        Ok(())
    }

    /// SASL `SCRAM-SHA-256` (RFC 5802 / RFC 7677) over POP3 `AUTH` (RFC 5034),
    /// challenge/response form: `AUTH SCRAM-SHA-256` → `+` → client-first → `+`
    /// server-first → client-final → server-final/`+OK`. The proof math is in
    /// [`crate::sasl::ScramSha256`] (pinned by the RFC 7677 vector test).
    ///
    /// Selected via [`Pop3Auth::SaslScram`](crate::backend::Pop3Auth) on the
    /// config; [`authenticate`](Self::authenticate) dispatches here.
    pub async fn authenticate_scram_sha256(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<()> {
        let nonce = sasl::client_nonce();
        let (mut scram, client_first) = sasl::ScramSha256::new(username, password, &nonce);

        self.send("AUTH SCRAM-SHA-256").await?;
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

async fn tls_connect(
    io: Box<dyn AsyncStream>,
    host: &str,
) -> Result<tokio_rustls::client::TlsStream<Box<dyn AsyncStream>>> {
    let config = tls_client_config()?;
    let connector = TlsConnector::from(config);
    let server_name = rustls_pki_types::ServerName::try_from(host.to_string())
        .map_err(|_| EngineError::Transport(format!("invalid TLS server name: {host}")))?;
    connector.connect(server_name, io).await.map_err(transport)
}

fn tls_client_config() -> Result<Arc<rustls::ClientConfig>> {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| EngineError::Transport(e.to_string()))?
        .with_root_certificates(roots)
        .with_no_client_auth();
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
        conn.authenticate_scram_sha256("user", "pencil")
            .await
            .expect("SCRAM authentication succeeds");
        server.await.unwrap();
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
}
