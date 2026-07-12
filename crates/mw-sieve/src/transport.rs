//! Transport for the ManageSieve client: a plaintext TCP stream that can be
//! opened as implicit TLS (rare) or upgraded in place via `STARTTLS` (the RFC
//! 5804 norm on port 4190). Mirrors `mw-imap`'s `ImapStream`.
//!
//! The protocol logic in [`crate::managesieve::Connection`] is generic over any
//! `AsyncRead + AsyncWrite`, so the deterministic transcript tests drive it over
//! an in-memory pipe; production connections finish over [`SieveStream`].

use std::pin::Pin;
use std::task::{Context, Poll};

use rustls_pki_types::ServerName;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

use crate::{SieveError, tls};

/// How the transport reaches the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsMode {
    /// Implicit TLS from the first byte.
    Implicit,
    /// Plaintext connect, then upgrade via the `STARTTLS` command.
    StartTls,
    /// No TLS — cleartext dev / the in-crate mock transcript tests.
    Plaintext,
}

/// A stream that is either plaintext TCP or a rustls-wrapped TCP stream.
pub enum SieveStream {
    Plain(TcpStream),
    Tls(Box<TlsStream<TcpStream>>),
}

impl SieveStream {
    /// Open a TCP connection, applying implicit TLS immediately when requested.
    pub async fn connect(host: &str, port: u16, mode: TlsMode) -> crate::Result<Self> {
        let tcp = TcpStream::connect((host, port)).await?;
        tcp.set_nodelay(true).ok();
        match mode {
            TlsMode::Plaintext | TlsMode::StartTls => Ok(SieveStream::Plain(tcp)),
            TlsMode::Implicit => Self::wrap_tls(tcp, host).await,
        }
    }

    /// Upgrade a plaintext stream to TLS in place (the `STARTTLS` handshake).
    pub async fn upgrade(self, host: &str) -> crate::Result<Self> {
        match self {
            SieveStream::Plain(tcp) => Self::wrap_tls(tcp, host).await,
            SieveStream::Tls(_) => Err(SieveError::ManageSieve("stream already encrypted".into())),
        }
    }

    async fn wrap_tls(tcp: TcpStream, host: &str) -> crate::Result<Self> {
        let connector = tls::connector()?;
        let server_name = ServerName::try_from(host.to_string())
            .map_err(|e| SieveError::ManageSieve(format!("invalid server name {host:?}: {e}")))?;
        let tls = connector
            .connect(server_name, tcp)
            .await
            .map_err(|e| SieveError::ManageSieve(format!("tls handshake: {e}")))?;
        Ok(SieveStream::Tls(Box::new(tls)))
    }

    /// Whether the transport is currently encrypted.
    pub fn is_encrypted(&self) -> bool {
        matches!(self, SieveStream::Tls(_))
    }
}

impl AsyncRead for SieveStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            SieveStream::Plain(s) => Pin::new(s).poll_read(cx, buf),
            SieveStream::Tls(s) => Pin::new(s.as_mut()).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for SieveStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            SieveStream::Plain(s) => Pin::new(s).poll_write(cx, buf),
            SieveStream::Tls(s) => Pin::new(s.as_mut()).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            SieveStream::Plain(s) => Pin::new(s).poll_flush(cx),
            SieveStream::Tls(s) => Pin::new(s.as_mut()).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            SieveStream::Plain(s) => Pin::new(s).poll_shutdown(cx),
            SieveStream::Tls(s) => Pin::new(s.as_mut()).poll_shutdown(cx),
        }
    }
}
