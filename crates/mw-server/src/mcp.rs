//! MCP server HTTP surface (SPEC §20.3, plan §2.4, §3 e11 mount). SCAFFOLD
//! (t6-e0): stub handler returning `501`, declared as a `mod` in `lib.rs` but NOT
//! mounted. e11 mounts the Streamable-HTTP MCP endpoint at `/mcp` against
//! `mw-mcp` (tools call the engine/JMAP surface, never raw protocol; per-tool
//! `mw-oauth` scope enforcement; `mail.send` gated → Outbox). The
//! `mailwoman mcp-stdio` subcommand proxies to a configured server (main.rs stub).

use axum::http::StatusCode;

/// `/mcp` — the Streamable-HTTP MCP transport. STUB: `501` until e11 mounts the
/// `mw-mcp` handler.
pub async fn mcp_stub() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
