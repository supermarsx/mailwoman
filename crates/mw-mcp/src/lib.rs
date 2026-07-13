#![forbid(unsafe_code)]
// SCAFFOLD (t6-e0): stub crate — the frozen §2.4 tool contract names exist so e11
// (mount) compiles against them; e4 owns the real `rmcp`-backed implementation.
#![allow(dead_code, clippy::unused_async)]
//! MCP server + tool registry for Mailwoman V6 (SPEC §20.3, plan §2.4).
//!
//! **Frozen contract (§2.4):** the [`McpTool`] set; each tool maps to a required
//! `mw-oauth::Scope` fragment; each result envelope carries [`Provenance`]
//! (`trust:"untrusted", source:"mail-body"`) on any mail-content field.
//! `mail.send` returns `{queued:true, outboxId}` (Outbox, human-in-the-loop)
//! UNLESS the key carries `unattended_send` AND its admin-countersign flag —
//! else 403. Transport: [`mcp_streamable_http`] (axum handler, mounted by e11) +
//! [`mcp_stdio`] (the `mailwoman mcp-stdio` subcommand proxy).
//!
//! **Prompt-injection posture (§7.1):** tools call the engine/JMAP surface, never
//! raw protocol; mail bodies are labelled untrusted; tool descriptions declare
//! mail as untrusted input. Zero-access accounts expose only what a client
//! session could decrypt = nothing server-side (documented).
//!
//! e4 fills the bodies (currently `unimplemented!()`) and links `rmcp` + `mw-oauth`.

use serde::{Deserialize, Serialize};

/// The frozen §2.4 MCP tool set. Each maps to a required scope fragment; each
/// is individually grantable per API key (`mcp_tools`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpTool {
    /// `mail.search`
    MailSearch,
    /// `mail.read`
    MailRead,
    /// `folders.list`
    FoldersList,
    /// `drafts.create`
    DraftsCreate,
    /// `mail.send` — gated → Outbox unless `unattended_send` + admin countersign.
    MailSend,
    /// `calendar.read`
    CalendarRead,
    /// `calendar.propose`
    CalendarPropose,
    /// `tasks.read`
    TasksRead,
    /// `tasks.write`
    TasksWrite,
    /// `contacts.read`
    ContactsRead,
}

impl McpTool {
    /// The canonical wire name (`mail.search`, `folders.list`, …).
    pub fn wire_name(self) -> &'static str {
        match self {
            McpTool::MailSearch => "mail.search",
            McpTool::MailRead => "mail.read",
            McpTool::FoldersList => "folders.list",
            McpTool::DraftsCreate => "drafts.create",
            McpTool::MailSend => "mail.send",
            McpTool::CalendarRead => "calendar.read",
            McpTool::CalendarPropose => "calendar.propose",
            McpTool::TasksRead => "tasks.read",
            McpTool::TasksWrite => "tasks.write",
            McpTool::ContactsRead => "contacts.read",
        }
    }
}

/// Provenance label attached to any mail-content field in a tool result (§2.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    /// `"untrusted"` for mail-derived content, `"trusted"` for server metadata.
    pub trust: String,
    /// e.g. `"mail-body"`.
    pub source: String,
}

impl Provenance {
    /// The standard label for untrusted mail-body content.
    pub fn untrusted_mail_body() -> Self {
        Self {
            trust: "untrusted".into(),
            source: "mail-body".into(),
        }
    }
}

/// The outcome of a gated `mail.send` (§2.4): queued to the Outbox pending
/// in-app confirmation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendOutcome {
    pub queued: bool,
    pub outbox_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("tool not granted by the caller's scope")]
    ScopeDenied,
    #[error("unattended send requires an admin-countersigned key")]
    CountersignRequired,
    #[error("engine error: {0}")]
    Engine(String),
    #[error("protocol error: {0}")]
    Protocol(String),
}

/// Build the Streamable-HTTP MCP handler mounted at `/mcp` by e11.
/// STUB: e4 returns the real `rmcp` axum service; the return type is finalized
/// there (kept `()` here to avoid pulling `rmcp`/`axum` into the scaffold).
pub fn mcp_streamable_http() {
    unimplemented!("mw-mcp::mcp_streamable_http — filled by t6-e4")
}

/// Run the `mailwoman mcp-stdio` proxy (stdin/stdout JSON-RPC ↔ a configured
/// server). STUB: e4 fills the loop.
pub async fn mcp_stdio() -> Result<(), McpError> {
    unimplemented!("mw-mcp::mcp_stdio — filled by t6-e4")
}
