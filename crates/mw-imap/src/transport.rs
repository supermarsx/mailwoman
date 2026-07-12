//! Transport abstraction: a plaintext TCP stream that can be upgraded to TLS
//! in place (STARTTLS) or opened as implicit TLS from the first byte (993).
//!
//! Plaintext mode exists for STARTTLS bootstrap and for the deterministic mock
//! / Greenmail transports used by the test suite; production accounts always
//! finish in [`ImapStream::Tls`].

use std::pin::Pin;
use std::task::{Context, Poll};

use rustls_pki_types::ServerName;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

use crate::error::{ImapError, ImapResult};
use crate::tls;

/// How the transport reaches the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsMode {
    /// Implicit TLS from the first byte (IMAPS, port 993).
    Implicit,
    /// Plaintext connect, then upgrade via the `STARTTLS` command.
    StartTls,
    /// No TLS at all — for the in-crate mock socket and cleartext dev/Greenmail.
    Plaintext,
}

/// A stream that is either plaintext TCP or a rustls-wrapped TCP stream.
///
/// Both inner types are `Unpin`, so the `AsyncRead`/`AsyncWrite` delegation
/// uses `Pin::new` on the matched inner stream.
pub enum ImapStream {
    Plain(TcpStream),
    Tls(Box<TlsStream<TcpStream>>),
}

impl ImapStream {
    /// Open a TCP connection, applying implicit TLS immediately when requested.
    pub async fn connect(host: &str, port: u16, mode: TlsMode) -> ImapResult<Self> {
        let tcp = TcpStream::connect((host, port)).await?;
        tcp.set_nodelay(true).ok();
        match mode {
            TlsMode::Plaintext | TlsMode::StartTls => Ok(ImapStream::Plain(tcp)),
            TlsMode::Implicit => Self::wrap_tls(tcp, host).await,
        }
    }

    /// Upgrade a plaintext stream to TLS in place (STARTTLS handshake body).
    ///
    /// A no-op (error) if the stream is already encrypted.
    pub async fn upgrade(self, host: &str) -> ImapResult<Self> {
        match self {
            ImapStream::Plain(tcp) => Self::wrap_tls(tcp, host).await,
            ImapStream::Tls(_) => Err(ImapError::Tls("stream already encrypted".into())),
        }
    }

    async fn wrap_tls(tcp: TcpStream, host: &str) -> ImapResult<Self> {
        let connector = tls::connector()?;
        let server_name = ServerName::try_from(host.to_string())
            .map_err(|e| ImapError::Tls(format!("invalid server name {host:?}: {e}")))?;
        let tls = connector
            .connect(server_name, tcp)
            .await
            .map_err(|e| ImapError::Tls(format!("tls handshake: {e}")))?;
        Ok(ImapStream::Tls(Box::new(tls)))
    }

    /// Whether the transport is currently encrypted.
    pub fn is_encrypted(&self) -> bool {
        matches!(self, ImapStream::Tls(_))
    }
}

impl AsyncRead for ImapStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            ImapStream::Plain(s) => Pin::new(s).poll_read(cx, buf),
            ImapStream::Tls(s) => Pin::new(s.as_mut()).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for ImapStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            ImapStream::Plain(s) => Pin::new(s).poll_write(cx, buf),
            ImapStream::Tls(s) => Pin::new(s.as_mut()).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            ImapStream::Plain(s) => Pin::new(s).poll_flush(cx),
            ImapStream::Tls(s) => Pin::new(s.as_mut()).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            ImapStream::Plain(s) => Pin::new(s).poll_shutdown(cx),
            ImapStream::Tls(s) => Pin::new(s.as_mut()).poll_shutdown(cx),
        }
    }
}
