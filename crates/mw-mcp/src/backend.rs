//! The engine/JMAP seam the tools call — an abstract [`McpBackend`] so `mw-mcp`
//! carries **no `mw-engine` dependency** (avoiding the crate cycle). e11 supplies
//! the real engine-backed impl at mount; [`crate::mock`] is the in-memory
//! reference used by this crate's tests.
//!
//! Send is split into two backend calls, [`McpBackend::enqueue_outbox`] and
//! [`McpBackend::send_now`], so the safety-critical gate ([`crate::gate_send`])
//! lives in `mw-mcp` and a test can assert `send_now` is never reached on the
//! Outbox path.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A backend failure (engine/JMAP error). Wrapped as [`crate::McpError::Engine`].
#[derive(Debug, thiserror::Error)]
#[error("backend error: {0}")]
pub struct BackendError(pub String);

impl BackendError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

/// A `mail.search` hit (message-derived → wrapped untrusted at the tool layer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub message_id: String,
    pub folder: String,
    pub from: String,
    pub subject: String,
    pub snippet: String,
    pub date: String,
}

/// A `mail.read` message body (message-derived → wrapped untrusted).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailBody {
    pub message_id: String,
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub date: String,
    pub body_text: String,
    pub body_html: Option<String>,
}

/// A mail folder (server metadata — trusted).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub role: Option<String>,
    pub unread: u32,
}

/// Draft/outbound composition input parsed from tool arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftInput {
    pub to: Vec<String>,
    pub subject: String,
    pub body_text: String,
}

/// Reference to a created draft.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftRef {
    pub draft_id: String,
}

/// The abstract surface the MCP tools drive. Every method targets an account by
/// id; the tool layer has already authorized the caller's [`mw_oauth::Scope`]
/// before any of these are called.
#[async_trait]
pub trait McpBackend: Send + Sync {
    async fn mail_search(
        &self,
        account: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchHit>, BackendError>;

    async fn mail_read(&self, account: &str, message_id: &str) -> Result<MailBody, BackendError>;

    async fn folders_list(&self, account: &str) -> Result<Vec<Folder>, BackendError>;

    async fn drafts_create(
        &self,
        account: &str,
        draft: DraftInput,
    ) -> Result<DraftRef, BackendError>;

    /// Queue an outbound message into the V2 Outbox (human confirms in-app).
    /// Returns the outbox id. This is the DEFAULT send path.
    async fn enqueue_outbox(
        &self,
        account: &str,
        draft: DraftInput,
    ) -> Result<String, BackendError>;

    /// Transmit immediately. ONLY reachable for a key with `unattended_send` AND an
    /// admin countersignature ([`crate::gate_send`]); never on the default path.
    /// Returns the sent message id.
    async fn send_now(&self, account: &str, draft: DraftInput) -> Result<String, BackendError>;

    async fn calendar_read(
        &self,
        account: &str,
        range: &str,
    ) -> Result<Vec<serde_json::Value>, BackendError>;

    async fn calendar_propose(
        &self,
        account: &str,
        proposal: serde_json::Value,
    ) -> Result<String, BackendError>;

    async fn tasks_read(&self, account: &str) -> Result<Vec<serde_json::Value>, BackendError>;

    async fn tasks_write(
        &self,
        account: &str,
        task: serde_json::Value,
    ) -> Result<String, BackendError>;

    async fn contacts_read(&self, account: &str) -> Result<Vec<serde_json::Value>, BackendError>;
}
