//! Acceptance tests (plan §3 e4). Cover: capability grant/deny; E2EE-decrypted
//! content never in an outbound payload by default (asserted); attachments excluded
//! by default; rate-limit trips; content-free audit; no send/accept/delete
//! capability exists; adapters parse OpenAI/Anthropic response shapes from fixtures;
//! streaming SSE decode; LocalProcess spawns + reads stdio; the assistant tool call
//! inherits scope + send→Outbox gating.

use std::sync::Arc;

use async_trait::async_trait;
use futures_util::StreamExt;
use serde_json::json;

use super::*;

// --- a network-free stub adapter for pipeline tests ------------------------

struct NullAdapter;

#[async_trait]
impl EndpointAdapter for NullAdapter {
    async fn chat(&self, _payload: &ChatPayload) -> Result<ChatStream> {
        Ok(futures_util::stream::empty::<Result<StreamChunk>>().boxed())
    }
    async fn embed(&self, _input: &str) -> Result<Vec<f32>> {
        Ok(vec![0.0])
    }
    async fn transcribe(&self, _audio: &[u8], _mime: &str) -> Result<String> {
        Ok(String::new())
    }
    fn host(&self) -> String {
        "null-host".to_string()
    }
}

fn gateway_with(grants: Vec<AssistCapability>) -> AssistGateway {
    let config = AssistConfig {
        enabled: true,
        capability_grants: grants,
        data_ceiling: DataScope {
            accounts: vec!["acct".into()],
            folders: vec![],
            include_e2ee: true, // ceiling allows; per-call scope still defaults off
            include_attachments: true,
        },
        adapter: None,
        rate_limit_per_min: None,
    };
    AssistGateway::new(config).with_adapter(Arc::new(NullAdapter))
}

fn scope() -> DataScope {
    DataScope {
        accounts: vec!["acct".into()],
        ..DataScope::default()
    }
}

// --- safe defaults ---------------------------------------------------------

#[test]
fn data_scope_defaults_are_safe() {
    let s = DataScope::default();
    assert!(!s.include_e2ee, "E2EE content excluded by default (R4)");
    assert!(!s.include_attachments, "attachments excluded by default");
}

#[tokio::test]
async fn unconfigured_gateway_is_disabled() {
    let gw = AssistGateway::new(AssistConfig::default());
    assert!(!gw.is_enabled());
    assert!(matches!(
        gw.invoke(
            AssistCapability::Summarize,
            DataScope::default(),
            &AssistInput::prompt("hi")
        )
        .await,
        Err(AssistError::Disabled)
    ));
}

// --- capability grant / deny ----------------------------------------------

#[tokio::test]
async fn capability_grant_and_deny() {
    let gw = gateway_with(vec![AssistCapability::Summarize]);
    assert!(
        gw.invoke(
            AssistCapability::Summarize,
            scope(),
            &AssistInput::prompt("x")
        )
        .await
        .is_ok()
    );
    assert!(matches!(
        gw.invoke(AssistCapability::Draft, scope(), &AssistInput::prompt("x"))
            .await,
        Err(AssistError::CapabilityDenied(AssistCapability::Draft))
    ));
}

// --- redaction: E2EE + attachments excluded by default --------------------

fn mixed_input() -> AssistInput {
    AssistInput {
        prompt: "please summarize".into(),
        context: vec![
            ContextItem {
                account: "acct".into(),
                folder: String::new(),
                text: "PLAIN-BODY-VISIBLE".into(),
                kind: ContentKind::Plain,
            },
            ContextItem {
                account: "acct".into(),
                folder: String::new(),
                text: "E2EE-SECRET-PLAINTEXT".into(),
                kind: ContentKind::E2eeDecrypted,
            },
            ContextItem {
                account: "acct".into(),
                folder: String::new(),
                text: "ATTACHMENT-SECRET-BYTES".into(),
                kind: ContentKind::Attachment,
            },
        ],
    }
}

#[test]
fn e2ee_and_attachments_excluded_by_default() {
    let eff = scope(); // include_e2ee=false, include_attachments=false
    let (payload, report) =
        redact::redact_chat_reported(&mixed_input(), &eff, AssistCapability::Summarize);

    assert!(payload.prompt.contains("PLAIN-BODY-VISIBLE"));
    assert!(
        !payload.prompt.contains("E2EE-SECRET-PLAINTEXT"),
        "E2EE-decrypted content must NEVER be in an outbound payload by default (R4)"
    );
    assert!(
        !payload.prompt.contains("ATTACHMENT-SECRET-BYTES"),
        "attachments must be excluded by default"
    );
    assert_eq!(report.dropped_e2ee, 1);
    assert_eq!(report.dropped_attachment, 1);
    assert_eq!(report.kept, 1);
}

#[test]
fn e2ee_forwarded_only_when_explicitly_granted() {
    let eff = DataScope {
        accounts: vec!["acct".into()],
        include_e2ee: true,
        ..DataScope::default()
    };
    let (payload, _) =
        redact::redact_chat_reported(&mixed_input(), &eff, AssistCapability::Summarize);
    assert!(
        payload.prompt.contains("E2EE-SECRET-PLAINTEXT"),
        "explicit include_e2ee=true forwards decrypted content"
    );
    // Attachment still excluded — a separate grant.
    assert!(!payload.prompt.contains("ATTACHMENT-SECRET-BYTES"));
}

#[test]
fn out_of_ceiling_account_is_dropped() {
    let eff = scope(); // accounts = ["acct"]
    let input = AssistInput {
        prompt: "q".into(),
        context: vec![ContextItem {
            account: "OTHER-ACCOUNT".into(),
            folder: String::new(),
            text: "FOREIGN-BODY".into(),
            kind: ContentKind::Plain,
        }],
    };
    let (payload, report) = redact::redact_chat_reported(&input, &eff, AssistCapability::Summarize);
    assert!(!payload.prompt.contains("FOREIGN-BODY"));
    assert_eq!(report.dropped_scope, 1);
    assert_eq!(report.kept, 0);
}

#[test]
fn clamp_ands_the_booleans() {
    // Call asks for e2ee, ceiling forbids ⇒ effective off.
    let call = DataScope {
        include_e2ee: true,
        ..DataScope::default()
    };
    let ceiling = DataScope {
        include_e2ee: false,
        ..DataScope::default()
    };
    assert!(!call.clamp(&ceiling).include_e2ee);
}

// --- rate-limit ------------------------------------------------------------

#[tokio::test]
async fn rate_limit_trips() {
    let config = AssistConfig {
        enabled: true,
        capability_grants: vec![AssistCapability::Summarize],
        data_ceiling: scope(),
        adapter: None,
        rate_limit_per_min: Some(2),
    };
    let gw = AssistGateway::new(config).with_adapter(Arc::new(NullAdapter));
    let inp = AssistInput::prompt("x");
    assert!(
        gw.invoke(AssistCapability::Summarize, scope(), &inp)
            .await
            .is_ok()
    );
    assert!(
        gw.invoke(AssistCapability::Summarize, scope(), &inp)
            .await
            .is_ok()
    );
    assert!(matches!(
        gw.invoke(AssistCapability::Summarize, scope(), &inp).await,
        Err(AssistError::RateLimited)
    ));
}

// --- content-free audit ----------------------------------------------------

#[tokio::test]
async fn audit_carries_host_cap_scope_but_no_content() {
    let audit = Arc::new(InMemoryAudit::default());
    let gw = gateway_with(vec![AssistCapability::Summarize]).with_audit(audit.clone());

    let _ = gw
        .invoke(AssistCapability::Summarize, scope(), &mixed_input())
        .await;

    let rows = audit.rows();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.capability, AssistCapability::Summarize);
    assert_eq!(row.endpoint_host, "null-host");
    assert!(row.scope_summary.contains("accounts="));

    // Structural + serialized guarantee: NO mail content anywhere in the row.
    let serialized = serde_json::to_string(row).expect("serialize audit");
    for secret in [
        "E2EE-SECRET-PLAINTEXT",
        "ATTACHMENT-SECRET-BYTES",
        "PLAIN-BODY-VISIBLE",
    ] {
        assert!(
            !serialized.contains(secret),
            "audit row must never contain mail content (R4)"
        );
    }
}

// --- no send/accept/delete capability exists -------------------------------

#[test]
fn no_send_accept_delete_capability_exists() {
    assert_eq!(AssistCapability::ALL.len(), 8);
    for cap in AssistCapability::ALL {
        let name = serde_json::to_string(&cap).expect("serialize");
        for forbidden in ["send", "delete", "accept", "transmit"] {
            assert!(
                !name.contains(forbidden),
                "no Assist capability may name a transmit/delete/accept path: {name}"
            );
        }
    }
}

// --- adapters parse OpenAI / Anthropic fixtures ----------------------------

#[test]
fn parse_openai_fixtures() {
    let chat = r#"{"choices":[{"message":{"role":"assistant","content":"Hello there."}}]}"#;
    assert_eq!(parse_openai_chat(chat).unwrap(), "Hello there.");

    let emb = r#"{"data":[{"embedding":[0.1,0.2,0.3]}]}"#;
    assert_eq!(
        parse_openai_embeddings(emb).unwrap(),
        vec![0.1_f32, 0.2, 0.3]
    );

    let stt = r#"{"text":"transcribed words"}"#;
    assert_eq!(
        parse_openai_transcription(stt).unwrap(),
        "transcribed words"
    );
}

#[test]
fn parse_anthropic_fixture() {
    let msg = r#"{"content":[{"type":"text","text":"Claude reply."}],"role":"assistant"}"#;
    assert_eq!(parse_anthropic_message(msg).unwrap(), "Claude reply.");
}

#[test]
fn sse_decode_openai_stream() {
    let mut dec = SseDecoder::new(Provider::OpenAi);
    // Split a delta across two transport chunks to exercise buffering.
    let mut chunks = dec.push(b"data: {\"choices\":[{\"delta\":{\"content\":\"Hel");
    chunks.extend(dec.push(b"lo\"}}]}\n"));
    chunks.extend(dec.push(b"data: [DONE]\n"));
    let deltas: Vec<StreamChunk> = chunks.into_iter().map(|r| r.unwrap()).collect();
    assert_eq!(deltas[0].delta, "Hello");
    assert!(deltas.last().unwrap().done);
}

#[test]
fn sse_decode_anthropic_stream() {
    let mut dec = SseDecoder::new(Provider::Anthropic);
    let mut chunks =
        dec.push(b"data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"Hi\"}}\n");
    chunks.extend(dec.push(b"data: {\"type\":\"message_stop\"}\n"));
    let deltas: Vec<StreamChunk> = chunks.into_iter().map(|r| r.unwrap()).collect();
    assert_eq!(deltas[0].delta, "Hi");
    assert!(deltas.last().unwrap().done);
}

// --- LocalProcess spawns + reads stdio -------------------------------------

#[tokio::test]
async fn local_process_spawns_and_reads_stdio() {
    // Write a canned JSON response to a temp file, then point the LocalProcess
    // adapter at a shell that echoes it — proving spawn + stdio round-trip
    // cross-platform (the request is written to the child's stdin).
    let mut path = std::env::temp_dir();
    path.push(format!("mw-assist-local-{}.json", std::process::id()));
    std::fs::write(&path, br#"{"content":"hello from stdio"}"#).expect("write fixture");
    let p = path.to_string_lossy().to_string();

    let adapter = if cfg!(windows) {
        AdapterConfig::LocalProcess {
            program: "cmd".into(),
            args: vec!["/C".into(), "type".into(), p.clone()],
        }
    } else {
        AdapterConfig::LocalProcess {
            program: "cat".into(),
            args: vec![p.clone()],
        }
    }
    .build()
    .expect("build local adapter");

    let stream = adapter
        .chat(&ChatPayload {
            system: None,
            prompt: "ignored".into(),
        })
        .await
        .expect("chat");
    let out: String = stream
        .filter_map(|r| async move { r.ok().map(|c| c.delta) })
        .collect::<Vec<_>>()
        .await
        .join("");
    let _ = std::fs::remove_file(&path);
    assert!(out.contains("hello from stdio"), "got {out:?}");
}

// --- assistant inherits scope + send→Outbox gating (via mw-mcp) -------------

use mw_mcp::mock::{MockAuthorizer, MockBackend};
use mw_mcp::{Credential, McpServer};
use mw_oauth::Scope;

fn cred() -> Credential<'static> {
    Credential {
        token: "",
        source_ip: None,
        resource: None,
    }
}

#[tokio::test]
async fn assistant_send_denied_without_send_scope() {
    let backend = Arc::new(MockBackend::new());
    // Read-only scope — no `send` verb, no mail.send tool grant.
    let auth = Arc::new(MockAuthorizer::new("acct", Scope::read_only("acct")));
    let server = Arc::new(McpServer::new(backend.clone(), auth));
    let tools = AssistantTools::new(server);

    let resp = tools
        .call_tool(
            &cred(),
            "mail.send",
            json!({"account":"acct","to":["x@y.example"]}),
        )
        .await;

    assert!(
        resp.get("error").is_some(),
        "send must be scope-denied: {resp}"
    );
    assert_eq!(backend.transmitted(), 0, "no transmit on a denied send");
}

#[tokio::test]
async fn assistant_send_routes_to_outbox_when_scoped() {
    let backend = Arc::new(MockBackend::new());
    let mut granted = Scope::read_only("acct");
    granted.send = true;
    granted.mail = true;
    granted.mcp_tools = vec!["mail.send".into()];
    // Not admin-countersigned ⇒ Outbox, never a direct transmit.
    let auth = Arc::new(MockAuthorizer::new("acct", granted));
    let server = Arc::new(McpServer::new(backend.clone(), auth));
    let tools = AssistantTools::new(server);

    let resp = tools
        .call_tool(
            &cred(),
            "mail.send",
            json!({"account":"acct","to":["x@y.example"]}),
        )
        .await;

    let queued = resp
        .pointer("/result/structuredContent/queued")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    assert!(queued, "scoped send is gated to the Outbox: {resp}");
    assert_eq!(backend.enqueued(), 1);
    assert_eq!(backend.transmitted(), 0, "Assist never transmits directly");
}
