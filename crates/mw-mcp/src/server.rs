//! The transport-agnostic MCP dispatch core.
//!
//! [`McpServer::handle_rpc`] takes a decoded JSON-RPC 2.0 request plus the caller's
//! [`Credential`] and returns the response value (or `None` for notifications). Both
//! the axum Streamable-HTTP handler and the stdio bridge drive this same method, so
//! the handshake, `tools/list`, and per-tool scope + gating + provenance logic have
//! exactly one implementation.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::auth::{AuthorizedCall, Authorizer, Credential};
use crate::backend::{DraftInput, McpBackend};
use crate::gating::{SendDecision, gate_send};
use crate::{ALL_TOOLS, McpError, McpTool, SendOutcome, untrusted_envelope};

/// Protocol version this server speaks (MCP revision).
const PROTOCOL_VERSION: &str = "2025-06-18";

/// The MCP server: an [`McpBackend`] (the engine seam) + an [`Authorizer`]
/// (per-call `mw-oauth` scope enforcement).
pub struct McpServer<B: McpBackend, A: Authorizer> {
    backend: Arc<B>,
    authorizer: Arc<A>,
}

impl<B: McpBackend, A: Authorizer> McpServer<B, A> {
    pub fn new(backend: Arc<B>, authorizer: Arc<A>) -> Self {
        Self {
            backend,
            authorizer,
        }
    }

    /// Dispatch one JSON-RPC request. Returns `Some(response_value)` for requests
    /// (those with an `id`) and `None` for notifications.
    pub async fn handle_rpc(&self, cred: &Credential<'_>, req: Value) -> Option<Value> {
        let method = req
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let params = req.get("params").cloned().unwrap_or(Value::Null);

        // Notifications (no `id`) never get a response.
        let id = req.get("id").cloned()?;

        let response = match method {
            "initialize" => ok(id, self.initialize_result()),
            "ping" => ok(id, json!({})),
            "tools/list" => ok(id, tools_list()),
            "tools/call" => self.tools_call(cred, id, &params).await,
            other => err_val(id, -32601, &format!("method not found: {other}")),
        };
        Some(response)
    }

    fn initialize_result(&self) -> Value {
        json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": "mailwoman-mcp", "version": env!("CARGO_PKG_VERSION") },
        })
    }

    async fn tools_call(&self, cred: &Credential<'_>, id: Value, params: &Value) -> Value {
        let name = match params.get("name").and_then(Value::as_str) {
            Some(n) => n,
            None => return err_from(id, &McpError::BadArguments("missing tool name".into())),
        };
        let tool = match McpTool::from_wire(name) {
            Some(t) => t,
            None => return err_from(id, &McpError::UnknownTool(name.to_string())),
        };
        let args = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let account = match args.get("account").and_then(Value::as_str) {
            Some(a) if !a.is_empty() => a.to_string(),
            _ => return err_from(id, &McpError::BadArguments("missing 'account'".into())),
        };

        // Per-call scope enforcement via mw-oauth.
        let required = tool.required_scope(&account);
        let authed = match self.authorizer.authorize(cred, &required).await {
            Ok(a) => a,
            Err(e) => return err_from(id, &e),
        };

        match self.invoke(tool, &authed, &args).await {
            Ok(structured) => ok(id, tool_result(structured)),
            Err(e) => err_from(id, &e),
        }
    }

    /// Execute a tool against the backend and wrap message-derived output in
    /// untrusted provenance.
    async fn invoke(
        &self,
        tool: McpTool,
        authed: &AuthorizedCall,
        args: &Value,
    ) -> Result<Value, McpError> {
        let account = &authed.account_id;
        match tool {
            McpTool::MailSearch => {
                let query = args
                    .get("query")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let limit = args
                    .get("limit")
                    .and_then(Value::as_u64)
                    .unwrap_or(20)
                    .min(500) as usize;
                let hits = self
                    .backend
                    .mail_search(account, query, limit)
                    .await
                    .map_err(engine)?;
                let results: Vec<Value> = hits
                    .into_iter()
                    .map(|h| {
                        untrusted_envelope(
                            "mail-body",
                            serde_json::to_value(h).unwrap_or(Value::Null),
                        )
                    })
                    .collect();
                Ok(json!({ "results": results }))
            }
            McpTool::MailRead => {
                let mid = require_str(args, "message_id")?;
                let body = self
                    .backend
                    .mail_read(account, &mid)
                    .await
                    .map_err(engine)?;
                Ok(untrusted_envelope(
                    "mail-body",
                    serde_json::to_value(body).unwrap_or(Value::Null),
                ))
            }
            McpTool::FoldersList => {
                let folders = self.backend.folders_list(account).await.map_err(engine)?;
                Ok(json!({ "folders": folders }))
            }
            McpTool::DraftsCreate => {
                let draft = parse_draft(args)?;
                let r = self
                    .backend
                    .drafts_create(account, draft)
                    .await
                    .map_err(engine)?;
                Ok(json!({ "draftId": r.draft_id }))
            }
            McpTool::MailSend => self.handle_send(authed, args).await,
            McpTool::CalendarRead => {
                let range = args
                    .get("range")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let events = self
                    .backend
                    .calendar_read(account, range)
                    .await
                    .map_err(engine)?;
                Ok(json!({ "events": wrap_all("calendar", events) }))
            }
            McpTool::CalendarPropose => {
                let proposal = args.get("proposal").cloned().unwrap_or(Value::Null);
                let id = self
                    .backend
                    .calendar_propose(account, proposal)
                    .await
                    .map_err(engine)?;
                Ok(json!({ "proposalId": id }))
            }
            McpTool::TasksRead => {
                let tasks = self.backend.tasks_read(account).await.map_err(engine)?;
                Ok(json!({ "tasks": wrap_all("task", tasks) }))
            }
            McpTool::TasksWrite => {
                let task = args.get("task").cloned().unwrap_or(Value::Null);
                let id = self
                    .backend
                    .tasks_write(account, task)
                    .await
                    .map_err(engine)?;
                Ok(json!({ "taskId": id }))
            }
            McpTool::ContactsRead => {
                let contacts = self.backend.contacts_read(account).await.map_err(engine)?;
                Ok(json!({ "contacts": wrap_all("contact", contacts) }))
            }
        }
    }

    /// The safety-critical `mail.send` path — routes through [`gate_send`].
    async fn handle_send(&self, authed: &AuthorizedCall, args: &Value) -> Result<Value, McpError> {
        let draft = parse_draft(args)?;
        match gate_send(&authed.scope, authed.admin_countersigned) {
            SendDecision::Queue => {
                let outbox_id = self
                    .backend
                    .enqueue_outbox(&authed.account_id, draft)
                    .await
                    .map_err(engine)?;
                let outcome = SendOutcome {
                    queued: true,
                    outbox_id,
                };
                Ok(serde_json::to_value(outcome).unwrap_or(Value::Null))
            }
            SendDecision::SendNow => {
                let message_id = self
                    .backend
                    .send_now(&authed.account_id, draft)
                    .await
                    .map_err(engine)?;
                Ok(json!({ "queued": false, "sent": true, "messageId": message_id }))
            }
            SendDecision::Deny => Err(McpError::CountersignRequired),
        }
    }
}

/// Build the `tools/list` result from the frozen tool set.
fn tools_list() -> Value {
    let tools: Vec<Value> = ALL_TOOLS
        .into_iter()
        .map(|t| {
            json!({
                "name": t.wire_name(),
                "description": t.description(),
                "inputSchema": t.input_schema(),
                "_meta": { "untrustedOutput": t.content_source().is_some() },
            })
        })
        .collect();
    json!({ "tools": tools })
}

/// Wrap each element of a list in untrusted provenance.
fn wrap_all(source: &str, items: Vec<Value>) -> Vec<Value> {
    items
        .into_iter()
        .map(|v| untrusted_envelope(source, v))
        .collect()
}

/// Assemble an MCP `CallToolResult` around a structured payload.
fn tool_result(structured: Value) -> Value {
    let text = serde_json::to_string(&structured).unwrap_or_else(|_| "{}".to_string());
    json!({
        "content": [ { "type": "text", "text": text } ],
        "structuredContent": structured,
        "isError": false,
    })
}

fn parse_draft(args: &Value) -> Result<DraftInput, McpError> {
    let to = match args.get("to") {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect::<Vec<_>>(),
        Some(Value::String(s)) => vec![s.clone()],
        _ => return Err(McpError::BadArguments("missing 'to' recipients".into())),
    };
    if to.is_empty() {
        return Err(McpError::BadArguments("empty 'to' recipients".into()));
    }
    Ok(DraftInput {
        to,
        subject: args
            .get("subject")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        body_text: args
            .get("body_text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    })
}

fn require_str(args: &Value, key: &str) -> Result<String, McpError> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .ok_or_else(|| McpError::BadArguments(format!("missing '{key}'")))
}

fn engine(e: crate::backend::BackendError) -> McpError {
    McpError::Engine(e.0)
}

fn ok(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn err_val(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn err_from(id: Value, e: &McpError) -> Value {
    err_val(id, e.rpc_code(), &e.to_string())
}
