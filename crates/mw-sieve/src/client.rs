//! High-level ManageSieve client: dial, optional `STARTTLS` upgrade, and SASL
//! authenticate, then delegate script operations to [`Connection`].
//!
//! The TLS paths reuse the `mw-imap`/`mw-smtp` `tokio` + `tokio-rustls` (ring)
//! setup and are not exercised by the plaintext transcript tests; the connect →
//! authenticate → command flow over plaintext is (a mock `TcpListener`).

use std::net::SocketAddr;

use crate::managesieve::{Checked, Connection, Credentials, ScriptInfo};
use crate::transport::{SieveStream, TlsMode};
use crate::{Result, SieveError};

/// A connected, authenticated ManageSieve session.
pub struct ManageSieveClient {
    conn: Connection<SieveStream>,
}

/// A checked save distinguishes local policy, upstream syntax, and transport
/// errors so callers can preserve structured line/column diagnostics.
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("script violates account policy")]
    Policy(Vec<crate::policy::Violation>),
    #[error("script failed upstream validation")]
    Invalid(Checked),
    #[error(transparent)]
    Protocol(#[from] SieveError),
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

    /// Like [`Self::connect`], with the stream opened by `dialer` (`None` =
    /// direct TCP).
    pub async fn connect_via(
        dialer: Option<&dyn mw_net::Dialer>,
        host: &str,
        port: u16,
        mode: TlsMode,
        creds: Credentials,
    ) -> Result<Self> {
        let stream = SieveStream::connect_via(dialer, host, port, mode).await?;
        Self::handshake(host, stream, mode, creds).await
    }

    /// Run the session over a stream that is already open to the server (a
    /// tunnel from `mw-egress`, a channel from `mw-ssh-tunnel`): implicit TLS or
    /// `STARTTLS` as `mode` says, validated against `host`, then authenticate.
    pub async fn from_stream(
        stream: mw_net::BoxStream,
        host: &str,
        mode: TlsMode,
        creds: Credentials,
    ) -> Result<Self> {
        let stream = SieveStream::from_stream(stream, host, mode).await?;
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

    /// Upload (or replace) a script; returns the server's warnings, if any.
    pub async fn put_script(&mut self, name: &str, body: &str) -> Result<Option<String>> {
        self.conn.put_script(name, body).await
    }

    /// Validate policy and the actual upstream extension inventory before
    /// uploading. A policy refusal performs no script-related wire operation.
    pub async fn put_script_checked(
        &mut self,
        name: &str,
        body: &str,
        policy: &crate::policy::Policy,
        edit: crate::policy::EditKind,
    ) -> std::result::Result<Option<String>, SaveError> {
        let mut effective = policy.clone();
        let advertised: std::collections::BTreeSet<_> = self
            .capabilities()
            .sieve
            .iter()
            .map(|s| s.to_ascii_lowercase())
            .collect();
        effective.advertised = Some(match effective.advertised.take() {
            Some(configured) => configured.intersection(&advertised).cloned().collect(),
            None => advertised,
        });
        crate::policy::enforce(body, &effective, edit).map_err(SaveError::Policy)?;
        let scripts = self.list_scripts().await?;
        crate::policy::check_script_count(
            scripts.len(),
            scripts.iter().any(|script| script.name == name),
            &effective,
        )
        .map_err(|error| SaveError::Policy(vec![error]))?;
        self.have_space(name, body.len() as u64).await?;
        if let invalid @ Checked::Invalid { .. } = self.check_script(body).await? {
            return Err(SaveError::Invalid(invalid));
        }
        Ok(self.put_script(name, body).await?)
    }

    /// Ask the server whether a script compiles, without storing it.
    pub async fn check_script(&mut self, body: &str) -> Result<Checked> {
        self.conn.check_script(body).await
    }

    /// Whether a script of `size` bytes under `name` fits the quota.
    pub async fn have_space(&mut self, name: &str, size: u64) -> Result<()> {
        self.conn.have_space(name, size).await
    }

    /// Rename a stored script (keeps its active flag).
    pub async fn rename_script(&mut self, old: &str, new: &str) -> Result<()> {
        self.conn.rename_script(old, new).await
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
