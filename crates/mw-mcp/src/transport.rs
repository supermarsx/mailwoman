//! Transports for the MCP dispatch core.
//!
//! - [`mcp_router`] — the **Streamable-HTTP** endpoint (`POST /mcp`, mounted by
//!   e11). A single JSON-RPC request in → a single JSON response out (the
//!   non-streaming subset of the MCP Streamable-HTTP transport, sufficient for the
//!   request/response tool calls; SSE fan-out is not needed by these tools).
//! - [`run_stdio`] — the `mailwoman mcp-stdio` bridge: reads newline-delimited
//!   JSON-RPC from a reader and writes responses to a writer, forwarding each
//!   message through an [`RpcForwarder`]. [`HttpForwarder`] proxies to a configured
//!   remote `/mcp`; [`run_stdio_http`] wires stdin/stdout to it for the subcommand.

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

use crate::auth::{Authorizer, Credential};
use crate::backend::McpBackend;
use crate::{McpError, McpServer};

// ── Streamable-HTTP transport ───────────────────────────────────────────────

/// Build the axum router for the MCP Streamable-HTTP endpoint. e11 nests this at
/// `/mcp`: `router.nest("/mcp", mcp_router(server))`.
pub fn mcp_router<B, A>(server: Arc<McpServer<B, A>>) -> Router
where
    B: McpBackend + 'static,
    A: Authorizer + 'static,
{
    Router::new()
        .route("/", post(handle::<B, A>))
        .with_state(server)
}

async fn handle<B, A>(
    State(server): State<Arc<McpServer<B, A>>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response
where
    B: McpBackend + 'static,
    A: Authorizer + 'static,
{
    let req: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return Json(json!({
                "jsonrpc": "2.0",
                "id": Value::Null,
                "error": { "code": -32700, "message": format!("parse error: {e}") },
            }))
            .into_response();
        }
    };
    let token = bearer(&headers).unwrap_or_default();
    // Source IP is supplied by e11's mount (ConnectInfo / trusted proxy header);
    // absent here, IP-allowlisted keys are denied by mw-oauth until wired.
    let cred = Credential {
        token,
        source_ip: None,
        resource: None,
    };
    match server.handle_rpc(&cred, req).await {
        // A single JSON response satisfies the Streamable-HTTP request/response case.
        Some(resp) => Json(resp).into_response(),
        // Notification accepted, no body (per the transport spec).
        None => StatusCode::ACCEPTED.into_response(),
    }
}

/// Extract a `Bearer` token from the `Authorization` header.
fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::trim)
}

// ── stdio bridge ────────────────────────────────────────────────────────────

/// Forwards a JSON-RPC request to an MCP server and returns its response value
/// (`Value::Null` for a notification / empty response).
#[async_trait]
pub trait RpcForwarder: Send + Sync {
    async fn forward(&self, request: Value) -> Result<Value, McpError>;
}

/// The `mailwoman mcp-stdio` bridge core: pump newline-delimited JSON-RPC from
/// `reader` to `forwarder`, writing each response to `writer`. Notifications (no
/// `id`) produce no output. Transport-agnostic so it round-trips in tests without a
/// socket.
pub async fn run_stdio<R, W, F>(reader: R, mut writer: W, forwarder: F) -> Result<(), McpError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
    F: RpcForwarder,
{
    let mut lines = BufReader::new(reader).lines();
    while let Some(line) = lines
        .next_line()
        .await
        .map_err(|e| McpError::Protocol(e.to_string()))?
    {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let request: Value =
            serde_json::from_str(line).map_err(|e| McpError::Protocol(e.to_string()))?;
        let is_notification = request.get("id").is_none();
        let response = forwarder.forward(request).await?;
        if is_notification || response.is_null() {
            continue;
        }
        let mut out =
            serde_json::to_string(&response).map_err(|e| McpError::Protocol(e.to_string()))?;
        out.push('\n');
        writer
            .write_all(out.as_bytes())
            .await
            .map_err(|e| McpError::Protocol(e.to_string()))?;
        writer
            .flush()
            .await
            .map_err(|e| McpError::Protocol(e.to_string()))?;
    }
    Ok(())
}

/// An [`RpcForwarder`] that POSTs to a remote MCP `/mcp` endpoint over HTTPS
/// (rustls, via the in-tree `reqwest`). Used by the real `mailwoman mcp-stdio`.
pub struct HttpForwarder {
    client: reqwest::Client,
    url: String,
    token: Option<String>,
}

impl HttpForwarder {
    pub fn new(url: impl Into<String>, token: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: url.into(),
            token,
        }
    }
}

#[async_trait]
impl RpcForwarder for HttpForwarder {
    async fn forward(&self, request: Value) -> Result<Value, McpError> {
        let mut rb = self
            .client
            .post(&self.url)
            .header(header::ACCEPT, "application/json")
            .json(&request);
        if let Some(t) = &self.token {
            rb = rb.bearer_auth(t);
        }
        let resp = rb
            .send()
            .await
            .map_err(|e| McpError::Protocol(e.to_string()))?;
        if resp.status() == reqwest::StatusCode::ACCEPTED {
            return Ok(Value::Null); // notification accepted, no body
        }
        let text = resp
            .text()
            .await
            .map_err(|e| McpError::Protocol(e.to_string()))?;
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_str(&text).map_err(|e| McpError::Protocol(e.to_string()))
    }
}

/// Run the `mailwoman mcp-stdio` bridge against a configured remote server: wire
/// stdin/stdout to an [`HttpForwarder`]. e11's clap subcommand calls this.
pub async fn run_stdio_http(url: impl Into<String>, token: Option<String>) -> Result<(), McpError> {
    let forwarder = HttpForwarder::new(url, token);
    run_stdio(tokio::io::stdin(), tokio::io::stdout(), forwarder).await
}
