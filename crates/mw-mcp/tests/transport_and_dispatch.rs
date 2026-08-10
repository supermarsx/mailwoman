//! The parts of `mw-mcp` the acceptance suite in `tests/mcp.rs` does not reach: the
//! **real** Streamable-HTTP transport (driven over a loopback socket rather than
//! asserted to be constructible), the `HttpForwarder` that backs `mailwoman
//! mcp-stdio`, and the dispatch core's PIM tool arms and error mapping.
//!
//! What the HTTP half is here to pin, specifically:
//!
//! * the endpoint's configured RFC 8707 resource reaches the [`Authorizer`] as
//!   `Credential::resource`. If that thread breaks, audience enforcement turns off
//!   silently and every existing audience test still passes, because they all call
//!   `handle_rpc` directly and pass the resource by hand;
//! * `Authorization: Bearer …` is parsed off the wire, and a request with no
//!   credential arrives as an empty token (⇒ denied) rather than as "no check";
//! * a notification is answered `202` with no body, and a malformed body is a
//!   JSON-RPC parse-error envelope.
//!
//! The dispatch half pins the prompt-injection posture (which tools wrap their
//! output untrusted) and the JSON-RPC error codes each failure maps to.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_mcp::mock::{MockAuthorizer, MockBackend};
use mw_mcp::{
    ALL_TOOLS, AuthorizedCall, Authorizer, BackendError, Credential, DraftInput, DraftRef, Folder,
    HttpForwarder, MailBody, McpBackend, McpServer, McpTool, Provenance, RpcForwarder, SearchHit,
    SendOutcome, mcp_router, run_stdio,
};
use mw_oauth::{Scope, ScopeSelector};

const RESOURCE: &str = "https://mail.example/mcp";

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
        mcp_tools: ALL_TOOLS
            .iter()
            .map(|t| t.wire_name().to_string())
            .collect(),
        unattended_send: false,
    }
}

fn call(tool: &str, args: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": tool, "arguments": args } })
}

// ── an Authorizer that actually inspects the credential ──────────────────────

/// Authorizes only the token `"good"`, and records the [`Credential::resource`] it
/// was handed. `MockAuthorizer` ignores the credential entirely, so it cannot tell
/// whether the transport threaded anything through.
#[derive(Default)]
struct TokenAuthorizer {
    seen_tokens: Mutex<Vec<String>>,
    seen_resources: Mutex<Vec<Option<String>>>,
}

#[async_trait]
impl Authorizer for TokenAuthorizer {
    async fn authorize(
        &self,
        cred: &Credential<'_>,
        _required: &Scope,
    ) -> Result<AuthorizedCall, mw_mcp::McpError> {
        self.seen_tokens
            .lock()
            .unwrap()
            .push(cred.token.to_string());
        self.seen_resources
            .lock()
            .unwrap()
            .push(cred.resource.map(str::to_string));
        if cred.token == "good" {
            Ok(AuthorizedCall {
                account_id: "acct1".into(),
                scope: full_scope(),
                admin_countersigned: false,
            })
        } else {
            Err(mw_mcp::McpError::ScopeDenied)
        }
    }
}

/// Spawn `mcp_router` on a loopback socket and return its base URL.
async fn serve(
    server: Arc<McpServer<MockBackend, TokenAuthorizer>>,
    resource: Option<String>,
) -> String {
    let router = mcp_router(server, resource);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind loopback");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    format!("http://{addr}/")
}

async fn serving() -> (String, Arc<TokenAuthorizer>, Arc<MockBackend>) {
    let authz = Arc::new(TokenAuthorizer::default());
    let backend = Arc::new(MockBackend::new());
    let server = Arc::new(McpServer::new(backend.clone(), authz.clone()));
    let url = serve(server, Some(RESOURCE.to_string())).await;
    (url, authz, backend)
}

// ── the real HTTP transport ──────────────────────────────────────────────────

#[tokio::test]
async fn http_post_threads_the_bearer_and_the_endpoint_resource_to_the_authorizer() {
    let (url, authz, _backend) = serving().await;

    let resp = reqwest::Client::new()
        .post(&url)
        .bearer_auth("good")
        .json(&call(
            "mail.search",
            json!({ "account": "acct1", "query": "quarterly" }),
        ))
        .send()
        .await
        .expect("POST /mcp");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body: Value = resp.json().await.expect("json body");
    assert!(
        body["result"]["structuredContent"]["results"].is_array(),
        "expected a tool result, got {body}"
    );

    // The token was parsed off the header (not the raw "Bearer good"), …
    assert_eq!(authz.seen_tokens.lock().unwrap().as_slice(), ["good"]);
    // …and the resource the mount configured reached the audience check. Without
    // this, a deployment that set MW_MCP_RESOURCE would enforce nothing.
    assert_eq!(
        authz.seen_resources.lock().unwrap().as_slice(),
        [Some(RESOURCE.to_string())]
    );
}

#[tokio::test]
async fn http_post_without_an_authorization_header_is_denied() {
    // An absent credential must reach the authorizer as an empty token — i.e. it is
    // *checked and refused*, never skipped.
    let (url, authz, _backend) = serving().await;

    let body: Value = reqwest::Client::new()
        .post(&url)
        .json(&call("mail.search", json!({ "account": "acct1" })))
        .send()
        .await
        .expect("POST")
        .json()
        .await
        .expect("json");
    assert_eq!(body["error"]["code"], json!(-32001), "{body}");
    assert_eq!(authz.seen_tokens.lock().unwrap().as_slice(), [""]);
}

#[tokio::test]
async fn http_post_ignores_a_non_bearer_authorization_scheme() {
    // `Basic …` is not a bearer credential; treating its payload as a token would be
    // a credential-confusion bug.
    let (url, authz, _backend) = serving().await;
    let body: Value = reqwest::Client::new()
        .post(&url)
        .header("Authorization", "Basic Z29vZDo=")
        .json(&call("mail.search", json!({ "account": "acct1" })))
        .send()
        .await
        .expect("POST")
        .json()
        .await
        .expect("json");
    assert_eq!(body["error"]["code"], json!(-32001), "{body}");
    assert_eq!(authz.seen_tokens.lock().unwrap().as_slice(), [""]);
}

#[tokio::test]
async fn http_notification_is_accepted_with_no_body() {
    // A JSON-RPC notification (no `id`) gets 202 and an empty body per the transport
    // spec — returning a response envelope would break a conforming client.
    let (url, _authz, _backend) = serving().await;
    let resp = reqwest::Client::new()
        .post(&url)
        .bearer_auth("good")
        .json(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }))
        .send()
        .await
        .expect("POST");
    assert_eq!(resp.status(), reqwest::StatusCode::ACCEPTED);
    assert!(resp.text().await.expect("body").is_empty());
}

#[tokio::test]
async fn http_malformed_body_is_a_json_rpc_parse_error() {
    // Not an HTTP 400: JSON-RPC carries its own error channel, and a client that only
    // reads the status would otherwise see a hang-shaped failure.
    let (url, _authz, _backend) = serving().await;
    let resp = reqwest::Client::new()
        .post(&url)
        .bearer_auth("good")
        .header("Content-Type", "application/json")
        .body("{ this is not json")
        .send()
        .await
        .expect("POST");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body: Value = resp.json().await.expect("json");
    assert_eq!(body["error"]["code"], json!(-32700), "{body}");
    assert_eq!(body["id"], Value::Null);
}

#[tokio::test]
async fn http_forwarder_drives_the_stdio_bridge_against_a_real_endpoint() {
    // `tests/mcp.rs` proves `run_stdio` against a hand-written forwarder. This proves
    // the pair actually used by `mailwoman mcp-stdio`: HttpForwarder over the wire,
    // including the 202-with-no-body case which must NOT be emitted as a response
    // line (a bare `null` on stdout desynchronises the client).
    let (url, _authz, _backend) = serving().await;

    let input = concat!(
        r#"{"jsonrpc":"2.0","id":7,"method":"tools/list"}"#,
        "\n",
        "\n", // a blank line is skipped, not a protocol error
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":8,"method":"ping"}"#,
        "\n",
    );
    let mut out: Vec<u8> = Vec::new();
    run_stdio(
        input.as_bytes(),
        &mut out,
        HttpForwarder::new(&url, Some("good".to_string())),
    )
    .await
    .expect("stdio bridge");

    let lines: Vec<&str> = std::str::from_utf8(&out)
        .expect("utf-8")
        .lines()
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(
        lines.len(),
        2,
        "one line per request, none for the notification: {lines:?}"
    );
    let first: Value = serde_json::from_str(lines[0]).expect("json");
    assert_eq!(first["id"], json!(7));
    assert_eq!(
        first["result"]["tools"].as_array().map(Vec::len),
        Some(ALL_TOOLS.len())
    );
    let second: Value = serde_json::from_str(lines[1]).expect("json");
    assert_eq!(second["id"], json!(8));
    assert_eq!(second["result"], json!({}), "ping answers an empty result");
}

#[tokio::test]
async fn the_stdio_bridge_rejects_a_line_that_is_not_json() {
    let (url, _authz, _backend) = serving().await;
    let mut out: Vec<u8> = Vec::new();
    let err = run_stdio(
        "not json at all\n".as_bytes(),
        &mut out,
        HttpForwarder::new(&url, None),
    )
    .await
    .expect_err("a malformed stdin line is a protocol error");
    assert!(matches!(err, mw_mcp::McpError::Protocol(_)), "got {err:?}");
    assert!(out.is_empty());
}

#[tokio::test]
async fn http_forwarder_surfaces_an_unreachable_endpoint_as_a_protocol_error() {
    // Port 1 on loopback has no listener: the bridge must report, not hang or panic.
    let fwd = HttpForwarder::new("http://127.0.0.1:1/", None);
    let err = fwd
        .forward(json!({ "jsonrpc": "2.0", "id": 1, "method": "ping" }))
        .await
        .expect_err("connection refused must surface");
    assert!(matches!(err, mw_mcp::McpError::Protocol(_)), "got {err:?}");
}

// ── dispatch core: provenance posture ────────────────────────────────────────

fn mock_server() -> McpServer<MockBackend, MockAuthorizer> {
    McpServer::new(
        Arc::new(MockBackend::new()),
        Arc::new(MockAuthorizer::new("acct1", full_scope())),
    )
}

fn cred() -> Credential<'static> {
    Credential {
        token: "mwk_test.secret",
        source_ip: None,
        resource: None,
    }
}

#[tokio::test]
async fn every_content_bearing_tool_labels_its_output_untrusted() {
    // The whole prompt-injection posture is this label. A PIM tool that returned bare
    // content would hand an agent mail-authored text with no marker on it.
    let server = mock_server();
    let cases = [
        ("calendar.read", "events", "calendar"),
        ("tasks.read", "tasks", "task"),
        ("contacts.read", "contacts", "contact"),
    ];
    for (tool, key, source) in cases {
        let resp = server
            .handle_rpc(&cred(), call(tool, json!({ "account": "acct1" })))
            .await
            .expect("response");
        let items = resp["result"]["structuredContent"][key]
            .as_array()
            .unwrap_or_else(|| panic!("{tool} should return a {key} array: {resp}"));
        assert!(!items.is_empty(), "{tool} returned nothing to label");
        for item in items {
            assert_eq!(item["provenance"]["trust"], json!("untrusted"), "{tool}");
            assert_eq!(item["provenance"]["source"], json!(source), "{tool}");
            assert!(item.get("content").is_some(), "{tool} content is wrapped");
        }
    }
}

#[tokio::test]
async fn server_metadata_and_write_results_are_not_wrapped_untrusted() {
    // The label has to mean something: folder metadata and the ids returned by write
    // tools are server-generated, and labelling them untrusted too would train an
    // agent to ignore the marker.
    let server = mock_server();
    let folders = server
        .handle_rpc(&cred(), call("folders.list", json!({ "account": "acct1" })))
        .await
        .expect("response");
    let list = folders["result"]["structuredContent"]["folders"]
        .as_array()
        .expect("folders array");
    assert_eq!(list.len(), 2);
    assert!(list[0].get("provenance").is_none(), "{folders}");
    assert_eq!(list[0]["role"], json!("inbox"));

    for (tool, args, field, want) in [
        (
            "calendar.propose",
            json!({ "account": "acct1", "proposal": { "summary": "Sync" } }),
            "proposalId",
            "proposal-1",
        ),
        (
            "tasks.write",
            json!({ "account": "acct1", "task": { "title": "Ship" } }),
            "taskId",
            "task-1",
        ),
        (
            "drafts.create",
            json!({ "account": "acct1", "to": ["a@example.com"] }),
            "draftId",
            "draft-1",
        ),
    ] {
        let resp = server
            .handle_rpc(&cred(), call(tool, args))
            .await
            .expect("response");
        assert_eq!(
            resp["result"]["structuredContent"][field],
            json!(want),
            "{tool}: {resp}"
        );
        assert!(
            resp["result"]["structuredContent"]
                .get("provenance")
                .is_none(),
            "{tool} result is server metadata"
        );
    }
}

#[tokio::test]
async fn tools_list_declares_untrusted_output_for_exactly_the_content_tools() {
    // `_meta.untrustedOutput` is what a client uses to decide how to present a tool's
    // result; it must agree with which tools actually wrap their content.
    let server = mock_server();
    let resp = server
        .handle_rpc(
            &cred(),
            json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }),
        )
        .await
        .expect("response");
    let tools = resp["result"]["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), ALL_TOOLS.len());

    let untrusted: Vec<&str> = tools
        .iter()
        .filter(|t| t["_meta"]["untrustedOutput"] == json!(true))
        .map(|t| t["name"].as_str().expect("name"))
        .collect();
    assert_eq!(
        untrusted,
        [
            "mail.search",
            "mail.read",
            "calendar.read",
            "tasks.read",
            "contacts.read"
        ]
    );
    // Each of those also says so in the description an agent reads.
    for t in tools
        .iter()
        .filter(|t| t["_meta"]["untrustedOutput"] == json!(true))
    {
        let desc = t["description"].as_str().expect("description");
        assert!(
            desc.contains("UNTRUSTED"),
            "{} should declare untrusted input: {desc}",
            t["name"]
        );
    }
    // Every tool advertises a schema whose required list includes the account.
    for t in tools {
        let required = t["inputSchema"]["required"].as_array().expect("required");
        assert!(
            required.contains(&json!("account")),
            "{} must require an account: {}",
            t["name"],
            t["inputSchema"]
        );
        assert_eq!(t["inputSchema"]["type"], json!("object"));
    }
}

// ── dispatch core: error mapping ─────────────────────────────────────────────

#[tokio::test]
async fn protocol_level_failures_map_to_their_json_rpc_codes() {
    let server = mock_server();
    let cases: [(Value, i64, &str); 5] = [
        (
            json!({ "jsonrpc": "2.0", "id": 1, "method": "resources/list" }),
            -32601,
            "an unimplemented method is method-not-found",
        ),
        (
            call("mail.teleport", json!({ "account": "acct1" })),
            -32601,
            "an unknown tool is method-not-found",
        ),
        (
            json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {} }),
            -32602,
            "a call with no tool name is invalid-params",
        ),
        (
            call("mail.search", json!({})),
            -32602,
            "a call with no account is invalid-params",
        ),
        (
            call(
                "mail.read",
                json!({ "account": "acct1", "message_id": "missing" }),
            ),
            -32000,
            "a backend failure is an engine error",
        ),
    ];
    for (req, code, why) in cases {
        let resp = server.handle_rpc(&cred(), req).await.expect("response");
        assert_eq!(resp["error"]["code"], json!(code), "{why}: {resp}");
        assert!(
            resp["error"]["message"]
                .as_str()
                .is_some_and(|m| !m.is_empty()),
            "{why}: an error needs a message"
        );
        assert!(resp.get("result").is_none(), "{why}");
    }
}

#[tokio::test]
async fn an_empty_account_string_is_rejected_like_a_missing_one() {
    // `""` would otherwise authorize against a scope fragment naming no account.
    let server = mock_server();
    let resp = server
        .handle_rpc(
            &cred(),
            call("mail.read", json!({ "account": "", "message_id": "m1" })),
        )
        .await
        .expect("response");
    assert_eq!(resp["error"]["code"], json!(-32602), "{resp}");
}

#[tokio::test]
async fn a_notification_gets_no_response_from_the_dispatch_core() {
    let server = mock_server();
    assert!(
        server
            .handle_rpc(&cred(), json!({ "jsonrpc": "2.0", "method": "ping" }))
            .await
            .is_none()
    );
}

#[tokio::test]
async fn initialize_reports_the_protocol_version_and_server_identity() {
    let server = mock_server();
    let resp = server
        .handle_rpc(
            &cred(),
            json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize" }),
        )
        .await
        .expect("response");
    assert_eq!(resp["result"]["protocolVersion"], json!("2025-06-18"));
    assert_eq!(resp["result"]["serverInfo"]["name"], json!("mailwoman-mcp"));
    assert_eq!(
        resp["result"]["capabilities"]["tools"]["listChanged"],
        json!(false)
    );
}

// ── dispatch core: draft parsing + the search limit clamp ────────────────────

#[tokio::test]
async fn recipients_accept_a_bare_string_but_never_an_empty_list() {
    let server = mock_server();
    // A single recipient sent as a string (rather than a one-element array) is a
    // common client shape and is accepted.
    let resp = server
        .handle_rpc(
            &cred(),
            call(
                "drafts.create",
                json!({ "account": "acct1", "to": "solo@example.com" }),
            ),
        )
        .await
        .expect("response");
    assert_eq!(
        resp["result"]["structuredContent"]["draftId"],
        json!("draft-1")
    );

    for bad in [json!([]), json!(null), json!(42)] {
        let resp = server
            .handle_rpc(
                &cred(),
                call("drafts.create", json!({ "account": "acct1", "to": bad })),
            )
            .await
            .expect("response");
        assert_eq!(
            resp["error"]["code"],
            json!(-32602),
            "a draft with no usable recipients must be refused: {resp}"
        );
    }
}

/// Records the `limit` `mail.search` was invoked with; everything else delegates to
/// [`MockBackend`].
struct LimitSpy {
    inner: MockBackend,
    last_limit: AtomicUsize,
}

impl LimitSpy {
    fn new() -> Self {
        Self {
            inner: MockBackend::new(),
            last_limit: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl McpBackend for LimitSpy {
    async fn mail_search(
        &self,
        account: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchHit>, BackendError> {
        self.last_limit.store(limit, Ordering::SeqCst);
        self.inner.mail_search(account, query, limit).await
    }
    async fn mail_read(&self, a: &str, m: &str) -> Result<MailBody, BackendError> {
        self.inner.mail_read(a, m).await
    }
    async fn folders_list(&self, a: &str) -> Result<Vec<Folder>, BackendError> {
        self.inner.folders_list(a).await
    }
    async fn drafts_create(&self, a: &str, d: DraftInput) -> Result<DraftRef, BackendError> {
        self.inner.drafts_create(a, d).await
    }
    async fn enqueue_outbox(&self, a: &str, d: DraftInput) -> Result<String, BackendError> {
        self.inner.enqueue_outbox(a, d).await
    }
    async fn send_now(&self, a: &str, d: DraftInput) -> Result<String, BackendError> {
        self.inner.send_now(a, d).await
    }
    async fn calendar_read(&self, a: &str, r: &str) -> Result<Vec<Value>, BackendError> {
        self.inner.calendar_read(a, r).await
    }
    async fn calendar_propose(&self, a: &str, p: Value) -> Result<String, BackendError> {
        self.inner.calendar_propose(a, p).await
    }
    async fn tasks_read(&self, a: &str) -> Result<Vec<Value>, BackendError> {
        self.inner.tasks_read(a).await
    }
    async fn tasks_write(&self, a: &str, t: Value) -> Result<String, BackendError> {
        self.inner.tasks_write(a, t).await
    }
    async fn contacts_read(&self, a: &str) -> Result<Vec<Value>, BackendError> {
        self.inner.contacts_read(a).await
    }
}

#[tokio::test]
async fn the_search_limit_is_defaulted_and_clamped_before_it_reaches_the_backend() {
    // An unbounded caller-supplied limit would let one authorized tool call ask the
    // engine for an arbitrarily large result set.
    let spy = Arc::new(LimitSpy::new());
    let server = McpServer::new(
        spy.clone(),
        Arc::new(MockAuthorizer::new("acct1", full_scope())),
    );

    for (arg, want) in [
        (json!({ "account": "acct1" }), 20usize),       // default
        (json!({ "account": "acct1", "limit": 5 }), 5), // honoured
        (json!({ "account": "acct1", "limit": 1_000_000 }), 500), // clamped
        (json!({ "account": "acct1", "limit": -3 }), 20), // not a u64 ⇒ default
    ] {
        server
            .handle_rpc(&cred(), call("mail.search", arg))
            .await
            .expect("response");
        assert_eq!(spy.last_limit.load(Ordering::SeqCst), want);
    }
}

// ── small public surfaces the acceptance suite does not touch ────────────────

#[test]
fn wire_names_round_trip_and_unknown_names_do_not_resolve() {
    for t in ALL_TOOLS {
        assert_eq!(McpTool::from_wire(t.wire_name()), Some(t));
    }
    for bogus in ["", "mail", "mail.search ", "MAIL.SEARCH", "mail.delete"] {
        assert_eq!(McpTool::from_wire(bogus), None, "{bogus} must not resolve");
    }
}

#[test]
fn required_scopes_split_read_from_send_and_mail_from_pim() {
    // A tool's required fragment is what `Scope::allows` is checked against, so an
    // over-broad fragment here would silently widen what a key can do.
    let s = McpTool::MailSearch.required_scope("acct1");
    assert!(s.read && s.mail && !s.send && !s.pim);
    assert_eq!(s.mcp_tools, ["mail.search"]);
    assert!(
        !s.unattended_send,
        "mail.send's unattended bypass is decided by gate_send on the GRANTED scope, \
         never demanded as a precondition"
    );

    let s = McpTool::MailSend.required_scope("acct1");
    assert!(s.send && s.mail && !s.read && !s.unattended_send);

    let s = McpTool::TasksWrite.required_scope("acct1");
    assert!(s.send && s.pim && !s.mail);

    let s = McpTool::ContactsRead.required_scope("acct1");
    assert!(s.read && s.pim && !s.mail);

    // Every fragment is scoped to the named account only.
    for t in ALL_TOOLS {
        match t.required_scope("acct1").accounts {
            ScopeSelector::Subset(ref v) => assert_eq!(v, &["acct1".to_string()]),
            other => panic!("{} must not request all accounts: {other:?}", t.wire_name()),
        }
    }
}

#[test]
fn provenance_and_outcome_types_serialize_as_the_wire_expects() {
    let p = Provenance::untrusted_mail_body();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        json!({ "trust": "untrusted", "source": "mail-body" })
    );
    // camelCase on the wire: a client reading `outbox_id` would find nothing.
    let outcome = SendOutcome {
        queued: true,
        outbox_id: "outbox-1".into(),
    };
    assert_eq!(
        serde_json::to_value(&outcome).unwrap(),
        json!({ "queued": true, "outboxId": "outbox-1" })
    );
}

#[test]
fn a_backend_error_keeps_its_message() {
    let e = BackendError::new("engine unavailable");
    assert_eq!(e.to_string(), "backend error: engine unavailable");
}
