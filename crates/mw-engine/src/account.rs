//! Per-account runtime wiring: the backend, the submitter, and the account's own
//! identity, plus the serde policy the store persists opaquely.
//!
//! Backend *construction* (dialling IMAP/POP3) lives in `mw-server`, which can
//! depend on `mw-imap`/`mw-pop3` — this crate cannot, because those crates
//! depend on it for the frozen trait. So the engine receives an already-built
//! [`AccountBackend`] and a [`MailSubmitter`] and orchestrates over them.

use std::sync::Arc;

use async_trait::async_trait;
use mw_smtp::{Outgoing, SubmissionResult, Submitter};
use serde::{Deserialize, Serialize};

use crate::backend::{AccountBackend, EngineError, Result, WatchHandle};

/// The send seam, mirroring `mw-smtp::Submitter::submit` but as a trait so tests
/// can inject a fake and the engine stays decoupled from a live SMTP socket.
#[async_trait]
pub trait MailSubmitter: Send + Sync {
    /// Submit one already-serialized message, reporting per-recipient outcome.
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult>;
}

/// Adapt the real `mw-smtp` submitter onto the engine's [`MailSubmitter`] seam.
#[async_trait]
impl MailSubmitter for Submitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        Submitter::submit(self, msg).await.map_err(|e| match e {
            mw_smtp::SmtpError::Auth(m) => EngineError::Auth(m),
            mw_smtp::SmtpError::Transport(m) => EngineError::Transport(m),
            mw_smtp::SmtpError::Protocol(m) => EngineError::Protocol(m),
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
        }
    }
}

/// The opaque per-account policy the engine serializes into
/// `accounts.sync_policy_json`. It carries what the *store* row does not: the
/// sibling SMTP endpoint and the POP3 retention knobs. `mw-server` reads it back
/// on reconnect to rebuild the backend + submitter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AccountPolicy {
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
