//! Integration tests for the MCP server (plan §2.4 acceptance).
//!
//! Covers: the MCP handshake + `tools/list` + a `mail.search` round-trip against
//! the mock backend; per-tool scope grant/deny (both a mock authorizer and the
//! real `mw-oauth` `OAuthAuthorizer` over minted API keys); the three send-gating
//! paths (Outbox / 403 / transmit) with a hard assertion that the Outbox and
//! denied paths NEVER transmit; untrusted provenance on mail content; and a stdio
//! bridge round-trip.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_mcp::mock::{MockAuthorizer, MockBackend};
use mw_mcp::{Credential, McpServer, RpcForwarder, mcp_router, run_stdio};
use mw_oauth::{
    AuthServer, AuthServerConfig, AuthorizeRequest, CollectingAudit, InMemoryOAuthStore,
    OAuthClient, OAuthStore, Scope, ScopeSelector, TokenRequest, challenge_s256, mint_api_key,
};

// ── helpers ──────────────────────────────────────────────────────────────

fn full_scope() -> Scope {
    Scope {
        read: true,
        send: true,
        delete: true,
        accounts: ScopeSelector::All,
        folders: ScopeSelector::All,
        mail: true,
        pim: true,
        ip_allowlist: Vec::new(),
        expires_at: None,
        rate_limit: None,
        mcp_tools: mw_mcp::ALL_TOOLS
            .iter()
            .map(|t| t.wire_name().to_string())
            .collect(),
        unattended_send: false,
    }
}

fn send_scope(unattended: bool) -> Scope {
    Scope {
        read: false,
        send: true,
        delete: false,
        accounts: ScopeSelector::All,
        folders: ScopeSelector::All,
        mail: true,
        pim: false,
        ip_allowlist: Vec::new(),
        expires_at: None,
        rate_limit: None,
        mcp_tools: vec!["mail.send".to_string()],
        unattended_send: unattended,
    }
}

fn server_with(
    backend: Arc<MockBackend>,
    authz: MockAuthorizer,
) -> McpServer<MockBackend, MockAuthorizer> {
    McpServer::new(backend, Arc::new(authz))
}

fn cred() -> Credential<'static> {
    Credential {
        token: "mwk_test.secret",
        source_ip: None,
        resource: None,
    }
}

fn call(tool: &str, args: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": tool, "arguments": args } })
}

// ── handshake + tools/list + mail.search round-trip ────────────────────────

#[tokio::test]
async fn handshake_lists_tools_and_searches() {
    let backend = Arc::new(MockBackend::new());
    let server = server_with(backend, MockAuthorizer::new("acct1", full_scope()));

    // initialize
    let init = server
        .handle_rpc(
            &cred(),
            json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize",
                    "params": { "protocolVersion": "2025-06-18" } }),
        )
        .await
        .expect("response");
    assert_eq!(init["result"]["serverInfo"]["name"], "mailwoman-mcp");
    assert!(init["result"]["protocolVersion"].is_string());
    assert!(init["result"]["capabilities"]["tools"].is_object());

    // notifications/initialized → no response
    assert!(
        server
            .handle_rpc(
                &cred(),
                json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
            )
            .await
            .is_none()
    );

    // tools/list
    let list = server
        .handle_rpc(
            &cred(),
            json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
        )
        .await
        .expect("response");
    let tools = list["result"]["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), 10);
    let search = tools
        .iter()
        .find(|t| t["name"] == "mail.search")
        .expect("mail.search present");
    assert!(
        search["description"]
            .as_str()
            .unwrap()
            .contains("UNTRUSTED"),
        "mail tool description must declare mail input untrusted"
    );

    // mail.search round-trip
    let resp = server
        .handle_rpc(
            &cred(),
            call("mail.search", json!({ "account": "acct1", "query": "" })),
        )
        .await
        .expect("response");
    let results = resp["result"]["structuredContent"]["results"]
        .as_array()
        .expect("results");
    assert!(!results.is_empty());
    assert_eq!(results[0]["provenance"]["trust"], "untrusted");
    assert_eq!(results[0]["provenance"]["source"], "mail-body");
    assert!(results[0]["content"]["subject"].is_string());
    assert_eq!(resp["result"]["isError"], false);
}

#[tokio::test]
async fn mail_read_wraps_untrusted_provenance() {
    let server = server_with(
        Arc::new(MockBackend::new()),
        MockAuthorizer::new("acct1", full_scope()),
    );
    let resp = server
        .handle_rpc(
            &cred(),
            call(
                "mail.read",
                json!({ "account": "acct1", "message_id": "m1" }),
            ),
        )
        .await
        .expect("response");
    let sc = &resp["result"]["structuredContent"];
    assert_eq!(sc["provenance"]["trust"], "untrusted");
    assert_eq!(sc["provenance"]["source"], "mail-body");
    assert!(sc["content"]["body_text"].is_string());
}

// ── per-tool scope grant / deny ────────────────────────────────────────────

#[tokio::test]
async fn scope_denies_ungranted_tool() {
    // A read-only mail scope with ONLY mail.search granted.
    let mut scope = Scope::read_only("acct1");
    scope.mail = true;
    scope.mcp_tools = vec!["mail.search".to_string()];
    let server = server_with(
        Arc::new(MockBackend::new()),
        MockAuthorizer::new("acct1", scope),
    );

    // Granted tool works.
    let ok = server
        .handle_rpc(&cred(), call("mail.search", json!({ "account": "acct1" })))
        .await
        .expect("response");
    assert!(ok["result"].is_object());

    // Ungranted tool (folders.list) is denied.
    let denied = server
        .handle_rpc(&cred(), call("folders.list", json!({ "account": "acct1" })))
        .await
        .expect("response");
    assert_eq!(denied["error"]["code"], -32001, "scope-denied code");

    // A write tool (drafts.create) is denied for a read-only key.
    let denied2 = server
        .handle_rpc(
            &cred(),
            call(
                "drafts.create",
                json!({ "account": "acct1", "to": ["x@y.z"] }),
            ),
        )
        .await
        .expect("response");
    assert_eq!(denied2["error"]["code"], -32001);
}

#[tokio::test]
async fn real_oauth_enforcement_over_minted_key() {
    // Prove enforcement runs through real mw-oauth: mint a scoped API key, store it,
    // and drive tool calls through OAuthAuthorizer.
    let mut scope = Scope::read_only("acct1");
    scope.mail = true;
    scope.mcp_tools = vec!["mail.search".to_string()];
    let minted = mint_api_key("acct1", scope);
    let token = minted.display_token.clone();

    let auth_server = Arc::new(AuthServer::new(InMemoryOAuthStore::new()));
    auth_server
        .store()
        .put_api_key(minted.record)
        .await
        .unwrap();
    let audit = Arc::new(CollectingAudit::new());
    let authz = Arc::new(mw_mcp::OAuthAuthorizer::without_countersign(
        auth_server.clone(),
        audit.clone(),
    ));
    let server = McpServer::new(Arc::new(MockBackend::new()), authz);

    let c = Credential {
        token: &token,
        source_ip: None,
        resource: None,
    };

    // Granted: mail.search.
    let ok = server
        .handle_rpc(&c, call("mail.search", json!({ "account": "acct1" })))
        .await
        .expect("response");
    assert!(ok["result"].is_object(), "mail.search should succeed: {ok}");

    // Denied: mail.read (not in mcp_tools).
    let denied = server
        .handle_rpc(
            &c,
            call(
                "mail.read",
                json!({ "account": "acct1", "message_id": "m1" }),
            ),
        )
        .await
        .expect("response");
    assert_eq!(denied["error"]["code"], -32001);

    // Denied: a bogus credential.
    let bad = Credential {
        token: "mwk_test.notreal",
        source_ip: None,
        resource: None,
    };
    let denied_bad = server
        .handle_rpc(&bad, call("mail.search", json!({ "account": "acct1" })))
        .await
        .expect("response");
    assert_eq!(denied_bad["error"]["code"], -32001);

    // Audit emitted for each attempt (grant + 2 denials).
    assert!(audit.events().len() >= 3);
    assert!(audit.events().iter().any(|e| e.allowed));
    assert!(audit.events().iter().any(|e| !e.allowed));
}

// ── send-gating: the three paths (safety-critical) ─────────────────────────

#[tokio::test]
async fn send_without_unattended_goes_to_outbox_never_transmits() {
    let backend = Arc::new(MockBackend::new());
    let server = server_with(
        backend.clone(),
        MockAuthorizer::new("acct1", send_scope(false)),
    );

    let resp = server
        .handle_rpc(
            &cred(),
            call(
                "mail.send",
                json!({ "account": "acct1", "to": ["boss@example.com"], "subject": "hi", "body_text": "yo" }),
            ),
        )
        .await
        .expect("response");
    let sc = &resp["result"]["structuredContent"];
    assert_eq!(sc["queued"], true, "must queue to Outbox: {resp}");
    assert_eq!(sc["outboxId"], "outbox-1");
    // HARD safety assertion: the Outbox path must never transmit.
    assert_eq!(backend.transmitted(), 0, "Outbox path must NOT transmit");
    assert_eq!(backend.enqueued(), 1);
}

#[tokio::test]
async fn unattended_without_countersign_is_403_and_never_transmits() {
    let backend = Arc::new(MockBackend::new());
    let authz = MockAuthorizer::new("acct1", send_scope(true)).countersigned(false);
    let server = server_with(backend.clone(), authz);

    let resp = server
        .handle_rpc(
            &cred(),
            call(
                "mail.send",
                json!({ "account": "acct1", "to": ["x@example.com"] }),
            ),
        )
        .await
        .expect("response");
    assert_eq!(
        resp["error"]["code"], -32002,
        "unattended without countersign must be refused (403-equivalent): {resp}"
    );
    // Neither transmitted nor queued.
    assert_eq!(backend.transmitted(), 0);
    assert_eq!(backend.enqueued(), 0);
}

#[tokio::test]
async fn unattended_with_countersign_transmits() {
    let backend = Arc::new(MockBackend::new());
    let authz = MockAuthorizer::new("acct1", send_scope(true)).countersigned(true);
    let server = server_with(backend.clone(), authz);

    let resp = server
        .handle_rpc(
            &cred(),
            call(
                "mail.send",
                json!({ "account": "acct1", "to": ["x@example.com"] }),
            ),
        )
        .await
        .expect("response");
    let sc = &resp["result"]["structuredContent"];
    assert_eq!(
        sc["sent"], true,
        "countersigned unattended send transmits: {resp}"
    );
    assert_eq!(sc["messageId"], "sent-1");
    assert_eq!(backend.transmitted(), 1);
    assert_eq!(backend.enqueued(), 0);
}

#[tokio::test]
async fn send_without_send_scope_is_denied() {
    let backend = Arc::new(MockBackend::new());
    // read-only key cannot send at all.
    let mut scope = Scope::read_only("acct1");
    scope.mail = true;
    scope.mcp_tools = vec!["mail.send".to_string()]; // tool granted but no `send` verb
    let server = server_with(backend.clone(), MockAuthorizer::new("acct1", scope));
    let resp = server
        .handle_rpc(
            &cred(),
            call(
                "mail.send",
                json!({ "account": "acct1", "to": ["x@example.com"] }),
            ),
        )
        .await
        .expect("response");
    assert_eq!(resp["error"]["code"], -32001);
    assert_eq!(backend.transmitted(), 0);
    assert_eq!(backend.enqueued(), 0);
}

// ── stdio bridge round-trip ────────────────────────────────────────────────

/// A forwarder that dispatches straight into an in-process server (stands in for
/// the HTTP hop so the bridge framing round-trips deterministically).
struct LocalForwarder {
    server: Arc<McpServer<MockBackend, MockAuthorizer>>,
}

#[async_trait]
impl RpcForwarder for LocalForwarder {
    async fn forward(&self, request: Value) -> Result<Value, mw_mcp::McpError> {
        let c = cred();
        Ok(self
            .server
            .handle_rpc(&c, request)
            .await
            .unwrap_or(Value::Null))
    }
}

#[tokio::test]
async fn stdio_bridge_round_trips_a_call() {
    let server = Arc::new(server_with(
        Arc::new(MockBackend::new()),
        MockAuthorizer::new("acct1", full_scope()),
    ));
    let forwarder = LocalForwarder {
        server: server.clone(),
    };

    // One newline-delimited tools/call on stdin.
    let line = serde_json::to_string(&call(
        "mail.search",
        json!({ "account": "acct1", "query": "quarterly" }),
    ))
    .unwrap();
    let input = format!("{line}\n");

    let mut out: Vec<u8> = Vec::new();
    run_stdio(input.as_bytes(), &mut out, forwarder)
        .await
        .expect("bridge runs");

    let resp: Value = serde_json::from_slice(&out).expect("one JSON response line");
    let results = resp["result"]["structuredContent"]["results"]
        .as_array()
        .expect("results");
    assert_eq!(
        results.len(),
        1,
        "query 'quarterly' matches one seeded message"
    );
    assert_eq!(results[0]["provenance"]["trust"], "untrusted");
}

// ── RFC 8707 resource-indicator (audience) enforcement (A3) ────────────────

/// Issue a real OAuth 2.1 access token bound to `resource`, granting mail.search.
async fn oauth_token_for_resource(
    resource: &str,
) -> (
    Arc<AuthServer<InMemoryOAuthStore>>,
    Arc<CollectingAudit>,
    String,
) {
    let server = Arc::new(AuthServer::with_config(
        InMemoryOAuthStore::new(),
        AuthServerConfig::default(),
    ));
    server
        .store()
        .put_client(OAuthClient {
            client_id: "client-1".into(),
            name: "Test App".into(),
            redirect_uris: vec!["https://app.example/cb".into()],
            approved_by: "admin".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
        })
        .await
        .unwrap();

    // A scope that grants exactly mail.search on acct1.
    let mut scope = Scope::read_only("acct1");
    scope.mail = true;
    scope.accounts = ScopeSelector::All;
    scope.mcp_tools = vec!["mail.search".to_string()];

    let verifier = "verifier-abc-123-verifier-abc-123-verifier";
    let challenge = challenge_s256(verifier);
    let auth = server
        .authorize(
            &AuthorizeRequest {
                response_type: "code".into(),
                client_id: "client-1".into(),
                redirect_uri: "https://app.example/cb".into(),
                scope,
                state: None,
                code_challenge: challenge,
                code_challenge_method: "S256".into(),
                resource: resource.into(),
            },
            "acct1",
        )
        .await
        .unwrap();
    let tokens = server
        .token(&TokenRequest::AuthorizationCode {
            code: auth.code,
            redirect_uri: "https://app.example/cb".into(),
            client_id: "client-1".into(),
            code_verifier: verifier.into(),
            resource: resource.into(),
        })
        .await
        .unwrap();

    let audit = Arc::new(CollectingAudit::new());
    (server, audit, tokens.access_token)
}

#[tokio::test]
async fn wrong_audience_token_is_rejected_at_mcp() {
    let resource = "https://mcp.example/mcp";
    let (auth_server, audit, token) = oauth_token_for_resource(resource).await;
    let authz = Arc::new(mw_mcp::OAuthAuthorizer::without_countersign(
        auth_server,
        audit,
    ));
    let server = McpServer::new(Arc::new(MockBackend::new()), authz);

    // Right audience → the resource-bound token is accepted and the tool runs.
    let right = Credential {
        token: &token,
        source_ip: None,
        resource: Some(resource),
    };
    let ok = server
        .handle_rpc(&right, call("mail.search", json!({ "account": "acct1" })))
        .await
        .expect("response");
    assert!(
        ok["result"].is_object(),
        "matching audience should pass: {ok}"
    );

    // Wrong audience → the token was issued for another resource server; REJECT it
    // before any tool runs (RFC 8707).
    let wrong = Credential {
        token: &token,
        source_ip: None,
        resource: Some("https://other.example/mcp"),
    };
    let denied = server
        .handle_rpc(&wrong, call("mail.search", json!({ "account": "acct1" })))
        .await
        .expect("response");
    assert_eq!(
        denied["error"]["code"], -32001,
        "wrong-audience token must be scope-denied: {denied}"
    );

    // No endpoint resource configured → audience enforcement is off (accepted).
    let unset = Credential {
        token: &token,
        source_ip: None,
        resource: None,
    };
    let ok2 = server
        .handle_rpc(&unset, call("mail.search", json!({ "account": "acct1" })))
        .await
        .expect("response");
    assert!(
        ok2["result"].is_object(),
        "no configured audience → not enforced: {ok2}"
    );
}

// ── the axum router is constructible (mounts by e11) ───────────────────────

#[tokio::test]
async fn http_router_builds() {
    let server = Arc::new(server_with(
        Arc::new(MockBackend::new()),
        MockAuthorizer::new("acct1", full_scope()),
    ));
    let _router: axum::Router = mcp_router(server, Some("https://mcp.example/mcp".into()));
}
