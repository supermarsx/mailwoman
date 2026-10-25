//! Transport for the ManageSieve client: a plaintext TCP stream that can be
//! opened as implicit TLS (rare) or upgraded in place via `STARTTLS` (the RFC
//! 5804 norm on port 4190). Mirrors `mw-imap`'s `ImapStream`.
//!
//! The protocol logic in [`crate::managesieve::Connection`] is generic over any
//! `AsyncRead + AsyncWrite`, so the deterministic transcript tests drive it over
//! an in-memory pipe; production connections finish over [`SieveStream`].
//!
//! The byte stream underneath is a direct TCP connection, a stream a
//! [`Dialer`] opened (an SSH tunnel channel, t32-ext §J), or one the caller
//! already holds ([`SieveStream::from_stream`]). TLS is always terminated here.

use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use mw_net::{BoxStream, Dialer};
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

/// A stream that is either the plaintext stream or a rustls session over it.
pub enum SieveStream {
    Plain(BoxStream),
    Tls(Box<TlsStream<BoxStream>>),
}

impl SieveStream {
    /// Open a TCP connection, applying implicit TLS immediately when requested.
    pub async fn connect(host: &str, port: u16, mode: TlsMode) -> crate::Result<Self> {
        Self::connect_via(None, host, port, mode).await
    }

    /// Like [`Self::connect`], with the stream opened by `dialer` (`None` =
    /// direct TCP).
    pub async fn connect_via(
        dialer: Option<&dyn Dialer>,
        host: &str,
        port: u16,
        mode: TlsMode,
    ) -> crate::Result<Self> {
        let stream = mw_net::dial(dialer, host, port).await?;
        Self::from_stream(stream, host, mode).await
    }

    /// Adopt a stream that is already open to the server (for example a tunnel
    /// from `mw-egress` or a channel from `mw-ssh-tunnel`), applying implicit TLS
    /// immediately when requested. `host` is used for TLS SNI and certificate
    /// validation only; for [`TlsMode::StartTls`] the upgrade happens later, in
    /// the client's handshake.
    pub async fn from_stream(stream: BoxStream, host: &str, mode: TlsMode) -> crate::Result<Self> {
        Self::from_stream_with_tls(stream, host, mode, &mw_net::TlsOptions::default()).await
    }

    pub async fn from_stream_with_tls(
        stream: BoxStream,
        host: &str,
        mode: TlsMode,
        options: &mw_net::TlsOptions,
    ) -> crate::Result<Self> {
        match mode {
            TlsMode::Plaintext | TlsMode::StartTls => Ok(SieveStream::Plain(stream)),
            TlsMode::Implicit => Self::wrap_tls(stream, host, options).await,
        }
    }

    /// Open a TCP connection to a PRE-RESOLVED `addr`, applying implicit TLS
    /// immediately when requested. Unlike [`Self::connect`], the TCP connect never
    /// performs its own DNS lookup — it dials exactly `addr` — so a name cannot
    /// re-resolve to a different, rebound target between a caller's egress check and
    /// this connect (anti-DNS-rebinding). `host` is retained ONLY for TLS SNI and
    /// certificate validation, so the pinned IP still presents the expected name.
    pub async fn connect_pinned(
        host: &str,
        addr: SocketAddr,
        mode: TlsMode,
    ) -> crate::Result<Self> {
        let tcp = TcpStream::connect(addr).await?;
        tcp.set_nodelay(true).ok();
        Self::from_stream(Box::new(tcp), host, mode).await
    }

    /// Upgrade a plaintext stream to TLS in place (the `STARTTLS` handshake).
    pub async fn upgrade(self, host: &str) -> crate::Result<Self> {
        self.upgrade_with_tls(host, &mw_net::TlsOptions::default())
            .await
    }

    pub async fn upgrade_with_tls(
        self,
        host: &str,
        options: &mw_net::TlsOptions,
    ) -> crate::Result<Self> {
        match self {
            SieveStream::Plain(tcp) => Self::wrap_tls(tcp, host, options).await,
            SieveStream::Tls(_) => Err(SieveError::ManageSieve("stream already encrypted".into())),
        }
    }

    async fn wrap_tls(
        tcp: BoxStream,
        host: &str,
        options: &mw_net::TlsOptions,
    ) -> crate::Result<Self> {
        let connector = tls::connector_with_options(options)?;
        let host = options.server_name.as_deref().unwrap_or(host);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The pinned connect must dial the supplied `addr` WITHOUT resolving `host`:
    /// a host that cannot resolve still connects, because only `addr` is dialed.
    /// This is the property that closes the Sieve rebinding TOCTOU (t18 R1) — the
    /// caller resolves+validates once and pins the allowed address here.
    #[tokio::test]
    async fn connect_pinned_dials_addr_not_host() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = listener.accept().await;
        });
        // `host` deliberately does not resolve; if `connect_pinned` re-resolved it
        // the dial would fail. It succeeds because the pinned `addr` is used.
        let stream =
            SieveStream::connect_pinned("does-not-resolve.invalid.", addr, TlsMode::Plaintext)
                .await
                .expect("pinned connect must dial addr, not resolve host");
        assert!(!stream.is_encrypted());
    }
}
