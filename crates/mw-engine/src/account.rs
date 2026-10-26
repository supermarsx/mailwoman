//! Per-account runtime wiring: the backend, the submitter, and the account's own
//! identity, plus the serde policy the store persists opaquely.
//!
//! Backend *construction* (dialling IMAP/POP3) lives in `mw-server`, which can
//! depend on `mw-imap`/`mw-pop3` — this crate cannot, because those crates
//! depend on it for the frozen trait. So the engine receives an already-built
//! [`AccountBackend`] and a [`MailSubmitter`] and orchestrates over them.

use std::sync::Arc;

use async_trait::async_trait;
use mw_smtp::{Outgoing, SubmissionResult, SubmitOptions, Submitter};
use serde::{Deserialize, Serialize};

use crate::backend::{AccountBackend, EngineError, Result, WatchHandle};

/// The send seam, mirroring `mw-smtp::Submitter::submit` but as a trait so tests
/// can inject a fake and the engine stays decoupled from a live SMTP socket.
#[async_trait]
pub trait MailSubmitter: Send + Sync {
    /// Submit one already-serialized message, reporting per-recipient outcome.
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult>;

    /// A fresh observation from this submitter's immutable transport authority.
    fn cached_dsn_support(&self) -> bool { false }

    /// Probe configured transport capabilities without submitting a message.
    async fn probe_dsn(&self) -> Result<bool> { Ok(false) }

    /// Submit with ESMTP options (t32-e46: DSN). A submitter that cannot honour
    /// a DSN request answers [`EngineError::Unsupported`] with
    /// [`DSN_NOT_OFFERED`] before anything is sent. Requested options are never
    /// silently discarded. Without a DSN request this is [`Self::submit`].
    async fn submit_with(&self, msg: Outgoing, opts: SubmitOptions) -> Result<SubmissionResult> {
        if opts.dsn.is_some() {
            return Err(EngineError::Unsupported(DSN_NOT_OFFERED.into()));
        }
        self.submit(msg).await
    }
}

/// The refusal text for a DSN request the submission server cannot honour. It
/// is the text `mw-smtp` gives when the server's EHLO has no `DSN`, which it
/// checks before `MAIL FROM`, so nothing has been sent when it is returned.
pub const DSN_NOT_OFFERED: &str = "server does not advertise DSN";

/// Text for one header line: `s` with every control character removed (CR, LF,
/// NUL, TAB, DEL, C1 …), so the value cannot end the header it is written into.
///
/// This is for values the engine interpolates into a header itself. It does no
/// RFC 2047 encoding: non-ASCII text passes through as UTF-8.
pub(crate) fn header_text(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).collect()
}

/// Adapt the real `mw-smtp` submitter onto the engine's [`MailSubmitter`] seam.
#[async_trait]
impl MailSubmitter for Submitter {
    fn cached_dsn_support(&self) -> bool { Submitter::cached_dsn_support(self) }
    async fn probe_dsn(&self) -> Result<bool> {
        Submitter::probe_dsn(self).await.map_err(|_| EngineError::Transport("SMTP capability probe unavailable".into()))
    }

    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        MailSubmitter::submit_with(self, msg, SubmitOptions::default()).await
    }

    async fn submit_with(&self, msg: Outgoing, opts: SubmitOptions) -> Result<SubmissionResult> {
        Submitter::submit_with(self, msg, opts)
            .await
            .map_err(|e| match e {
                mw_smtp::SmtpError::Auth(m) => EngineError::Auth(m),
                mw_smtp::SmtpError::Protocol(m) if m == DSN_NOT_OFFERED => {
                    EngineError::Unsupported(m)
                }
                mw_smtp::SmtpError::DeliveryUncertain(m) => EngineError::SubmissionUncertain(m),
                mw_smtp::SmtpError::Transport(m) => EngineError::Transport(m),
                mw_smtp::SmtpError::Protocol(m) => EngineError::Protocol(m),
                e @ mw_smtp::SmtpError::InvalidAddress(_) => EngineError::Protocol(e.to_string()),
                // The server lacks an extension the message cannot do without.
                // `Unsupported` is what `transmit_draft` does not retry.
                e @ mw_smtp::SmtpError::SmtpUtf8Required(_) => {
                    EngineError::Unsupported(e.to_string())
                }
            })
    }
}

/// One connected account the engine can serve JMAP calls for.
///
/// Cheap to clone (everything is an `Arc` or a `String`) so callers snapshot it
/// out of the registry lock and then operate without holding the lock across an
/// `await`.
#[derive(Clone)]
pub struct AccountRuntime {
    /// The live IMAP/POP3 backend behind the frozen seam.
    pub backend: Arc<dyn AccountBackend>,
    /// The submission client for `EmailSubmission/set`.
    pub submitter: Arc<dyn MailSubmitter>,
    /// The account's own address, used as `From`/`MAIL FROM` when composing.
    pub identity: String,
    /// A running change-ingestion loop, kept alive so its `Drop`/`stop` fires
    /// when the account is unregistered.
    pub watch: Option<Arc<WatchHandle>>,
    /// Opaque randomized sealed credentials, used only for rejection CAS.
    pub credential_revision: Option<Vec<u8>>,
    /// The account's CalDAV/CardDAV connection config (plan §2.3, V3). `None`
    /// for a mail-only account; when set, the PIM surface (e8) drives
    /// `mw-dav`/`mw-carddav` to sync CalDAV-backed calendars + CardDAV-backed
    /// address books over it. The per-collection href is stored on each
    /// calendar/address-book row; this carries the shared base URL + credentials.
    pub dav: Option<mw_dav::DavConfig>,
    /// Per-operation account authority; never falls back to direct networking.
    pub dav_transport: Option<Arc<dyn mw_dav::DavTransport>>,
    pub(crate) pim_sync: Arc<crate::pim::schedule::SyncControl>,
}

impl AccountRuntime {
    /// Build a runtime from its parts (no watch loop attached yet).
    pub fn new(
        backend: Arc<dyn AccountBackend>,
        submitter: Arc<dyn MailSubmitter>,
        identity: impl Into<String>,
    ) -> Self {
        Self {
            backend,
            submitter,
            identity: identity.into(),
            watch: None,
            credential_revision: None,
            dav: None,
            dav_transport: None,
            pim_sync: Arc::default(),
        }
    }

    /// Attach a guarded PIM transport. Its implementation rechecks live authority.
    pub fn with_dav_transport(mut self, transport: Arc<dyn mw_dav::DavTransport>) -> Self {
        self.dav_transport = Some(transport);
        self
    }

    pub(crate) fn dav_client(&self) -> mw_dav::Result<mw_dav::DavClient> {
        let config = self
            .dav
            .clone()
            .ok_or_else(|| mw_dav::DavError::Transport("DAV is not configured".into()))?;
        match &self.dav_transport {
            Some(transport) => Ok(mw_dav::DavClient::with_transport(config, transport.clone())),
            None => mw_dav::DavClient::new(config),
        }
    }

    pub(crate) fn carddav_client(&self) -> mw_dav::Result<mw_carddav::CardDavClient> {
        let config = self
            .dav
            .clone()
            .ok_or_else(|| mw_dav::DavError::Transport("DAV is not configured".into()))?;
        match &self.dav_transport {
            Some(transport) => Ok(mw_carddav::CardDavClient::with_transport(
                config,
                transport.clone(),
            )),
            None => mw_carddav::CardDavClient::new(config),
        }
    }

    /// Attach a CalDAV/CardDAV connection config so the PIM surface can sync
    /// this account's DAV-backed collections (builder form; leaves the rest).
    pub fn with_dav(mut self, dav: mw_dav::DavConfig) -> Self {
        self.dav = Some(dav);
        self
    }
}

/// The opaque per-account policy the engine serializes into
/// `accounts.sync_policy_json`. It carries what the *store* row does not: the
/// sibling SMTP endpoint and the POP3 retention knobs. `mw-server` reads it back
/// on reconnect to rebuild the backend + submitter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AccountPolicy {
    /// Opaque binding to operator-selected domain authorities and auth policy.
    pub domain_binding: Option<String>,
    /// Submission host (empty ⇒ no send configured).
    pub smtp_host: String,
    /// Submission port.
    pub smtp_port: u16,
    /// `implicit` | `starttls` | `plaintext`.
    pub smtp_security: String,
    /// POP3 retention: `keep` | `delete_on_retrieval` | `delete_after_days`.
    pub leave_policy: String,
    /// Days to retain for `delete_after_days`.
    pub leave_days: u32,
    /// `watch` poll interval (seconds) for POP3 / non-IDLE IMAP.
    pub poll_secs: u64,
}

impl Default for AccountPolicy {
    fn default() -> Self {
        Self {
            domain_binding: None,
            smtp_host: String::new(),
            smtp_port: 587,
            smtp_security: "starttls".to_string(),
            leave_policy: "keep".to_string(),
            leave_days: 30,
            poll_secs: 300,
        }
    }
}

impl AccountPolicy {
    /// Serialize to the opaque JSON the store persists.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Parse from the store's opaque JSON, falling back to defaults.
    pub fn from_json(json: &str) -> Self {
        serde_json::from_str(json).unwrap_or_default()
    }
}
