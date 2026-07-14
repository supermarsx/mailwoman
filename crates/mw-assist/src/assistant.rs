//! The `Assistant` capability's tool surface — an **unprivileged CLIENT** of the
//! frozen V6 `mw-mcp` tool registry (plan §2.4 / SPEC §14.3).
//!
//! The assistant chat routes every tool call through [`mw_mcp::McpServer`] with the
//! caller's `mw_oauth::Scope`, so it **inherits** the exact per-tool scope
//! enforcement and the `mail.send`→Outbox gating that the MCP surface already
//! provides. There is **no privileged path**: Assist adds no new send/delete/accept
//! capability, and this wrapper cannot mint one — `mail.send` is reachable only if
//! the caller's own scope grants it, and even then it is gated to the Outbox unless
//! the key carries an admin countersignature (resolved by the `mw-mcp`
//! `Authorizer`; the real countersign resolver is wired by e14 at mount — this
//! crate only consumes the seam).

use std::sync::Arc;

use mw_mcp::{Authorizer, Credential, McpBackend, McpServer};
use serde_json::{Value, json};

/// A thin client over an [`McpServer`] for the `Assistant` capability. Every call
/// is dispatched as a JSON-RPC `tools/call` through the same handler the MCP
/// transport uses, so scope + provenance + send-gating are identical.
pub struct AssistantTools<B: McpBackend, A: Authorizer> {
    server: Arc<McpServer<B, A>>,
}

impl<B: McpBackend, A: Authorizer> AssistantTools<B, A> {
    /// Wrap an MCP server (built at mount with the real engine backend +
    /// `OAuthAuthorizer`).
    #[must_use]
    pub fn new(server: Arc<McpServer<B, A>>) -> Self {
        Self { server }
    }

    /// Enumerate the tools available to the assistant (`tools/list`).
    pub async fn list_tools(&self, cred: &Credential<'_>) -> Value {
        let req = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" });
        self.server
            .handle_rpc(cred, req)
            .await
            .unwrap_or(Value::Null)
    }

    /// Invoke a tool by wire name (e.g. `mail.search`, `mail.send`). The caller's
    /// [`Credential`] carries the `mw-oauth` scope; the MCP server authorizes it
    /// per call and applies send-gating — the assistant never bypasses either.
    pub async fn call_tool(&self, cred: &Credential<'_>, name: &str, arguments: Value) -> Value {
        let req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": name, "arguments": arguments },
        });
        self.server
            .handle_rpc(cred, req)
            .await
            .unwrap_or(Value::Null)
    }
}
