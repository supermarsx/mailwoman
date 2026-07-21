//! High-level ManageSieve client: dial, optional `STARTTLS` upgrade, and SASL
//! authenticate, then delegate script operations to [`Connection`].
//!
//! The TLS paths reuse the `mw-imap`/`mw-smtp` `tokio` + `tokio-rustls` (ring)
//! setup and are not exercised by the plaintext transcript tests; the connect →
//! authenticate → command flow over plaintext is (a mock `TcpListener`).

use std::net::SocketAddr;

use crate::managesieve::{Connection, Credentials, ScriptInfo};
use crate::transport::{SieveStream, TlsMode};
use crate::{Result, SieveError};

/// A connected, authenticated ManageSieve session.
pub struct ManageSieveClient {
    conn: Connection<SieveStream>,
}

impl ManageSieveClient {
    /// Connect to `host:port`, upgrade via `STARTTLS` when requested, and
    /// authenticate. Per RFC 5804 the server re-issues its capabilities after a
    /// successful TLS handshake, so the post-upgrade greeting is re-read.
    pub async fn connect(host: &str, port: u16, mode: TlsMode, creds: Credentials) -> Result<Self> {
        let stream = SieveStream::connect(host, port, mode).await?;
        Self::handshake(host, stream, mode, creds).await
    }

    /// Like [`Self::connect`], but the TCP connection is pinned to a caller-supplied,
    /// PRE-RESOLVED [`SocketAddr`] instead of re-resolving `host`. The caller
    /// resolves + egress-validates the target once and passes the allowed address
    /// here, so a name cannot rebind to a different (metadata/loopback) target
    /// between the check and the connect (anti-DNS-rebinding; t18 R1). `host` is
    /// still used for TLS SNI and the `STARTTLS` upgrade's certificate validation.
    pub async fn connect_pinned(
        host: &str,
        addr: SocketAddr,
        mode: TlsMode,
        creds: Credentials,
    ) -> Result<Self> {
        let stream = SieveStream::connect_pinned(host, addr, mode).await?;
        Self::handshake(host, stream, mode, creds).await
    }

    /// Shared post-connect flow: open the connection, perform the optional
    /// `STARTTLS` upgrade (re-reading capabilities per RFC 5804), then authenticate.
    /// `host` is used for TLS SNI during any upgrade.
    async fn handshake(
        host: &str,
        stream: SieveStream,
        mode: TlsMode,
        creds: Credentials,
    ) -> Result<Self> {
        let mut conn = Connection::open(stream).await?;

        if mode == TlsMode::StartTls {
            if !conn.capabilities().starttls {
                return Err(SieveError::ManageSieve(
                    "server does not advertise STARTTLS".into(),
                ));
            }
            conn.starttls().await?;
            let stream = conn.into_inner()?.upgrade(host).await?;
            conn = Connection::open(stream).await?;
        }

        conn.authenticate(&creds).await?;
        Ok(Self { conn })
    }

    /// The negotiated server capabilities.
    pub fn capabilities(&self) -> &crate::Capabilities {
        self.conn.capabilities()
    }

    /// Upload (or replace) a script.
    pub async fn put_script(&mut self, name: &str, body: &str) -> Result<()> {
        self.conn.put_script(name, body).await
    }

    /// List stored scripts and their active state.
    pub async fn list_scripts(&mut self) -> Result<Vec<ScriptInfo>> {
        self.conn.list_scripts().await
    }

    /// Activate a script (empty name deactivates all).
    pub async fn set_active(&mut self, name: &str) -> Result<()> {
        self.conn.set_active(name).await
    }

    /// Fetch a stored script's source.
    pub async fn get_script(&mut self, name: &str) -> Result<String> {
        self.conn.get_script(name).await
    }

    /// Delete a stored script.
    pub async fn delete_script(&mut self, name: &str) -> Result<()> {
        self.conn.delete_script(name).await
    }

    /// End the session cleanly.
    pub async fn logout(mut self) -> Result<()> {
        self.conn.logout().await
    }
}
