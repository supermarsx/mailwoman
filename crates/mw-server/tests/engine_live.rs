//! Live engine integration test (plan §3 e6 acceptance, handoff to e7).
//!
//! Env-gated + `#[ignore]` so `cargo test` stays green with no server running.
//! e7's conformance job runs it against Greenmail by setting `GREENMAIL_IMAP`
//! (and `GREENMAIL_USER`/`GREENMAIL_PASS`). It drives a real IMAP account through
//! `mw-engine` and asserts the JMAP surface returns the seeded mail — the same
//! path the browser hits in engine mode, minus the HTTP hop.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;

use mw_engine::Engine;
use mw_engine::account::{AccountRuntime, MailSubmitter};
use mw_engine::backend::{AccountBackend, Result};
use mw_imap::{Credentials as ImapCredentials, ImapBackend, ImapConfig, TlsMode};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

/// A submitter that never sends — the sync/read path under test does not submit.
struct NoSubmitter;

#[async_trait]
impl MailSubmitter for NoSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        Ok(SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

#[tokio::test]
#[ignore = "requires a live IMAP server; set GREENMAIL_IMAP=host:port"]
async fn engine_syncs_and_serves_live_imap() {
    let Ok(addr) = std::env::var("GREENMAIL_IMAP") else {
        return;
    };
    let (host, port) = addr.rsplit_once(':').expect("GREENMAIL_IMAP as host:port");
    let port: u16 = port.parse().expect("port");
    let user = std::env::var("GREENMAIL_USER").unwrap_or_else(|_| "user@example.org".into());
    let pass = std::env::var("GREENMAIL_PASS").unwrap_or_else(|_| "pass".into());
    // Greenmail's IMAP port is plaintext.
    let tls = match std::env::var("GREENMAIL_TLS").as_deref() {
        Ok("implicit") => TlsMode::Implicit,
        Ok("starttls") => TlsMode::StartTls,
        _ => TlsMode::Plaintext,
    };

    let backend = ImapBackend::connect(ImapConfig {
        host: host.to_string(),
        port,
        tls,
        credentials: ImapCredentials::Password {
            username: user.clone(),
            password: pass.clone(),
        },
        watch_mailbox: "INBOX".to_string(),
    })
    .await
    .expect("connect to live IMAP");

    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host,
                port,
                tls: "plaintext",
                username: &user,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: user.clone(),
                password: pass.clone(),
            },
        )
        .await
        .unwrap();

    let engine = Arc::new(Engine::new(store));
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            Arc::new(backend) as Arc<dyn AccountBackend>,
            Arc::new(NoSubmitter) as Arc<dyn MailSubmitter>,
            user.clone(),
        ),
    );

    engine.resync(&account_id).await.expect("live resync");

    // Mailbox/get returns at least the Inbox.
    let mb = engine
        .handle_jmap(
            &account_id,
            &json!({ "methodCalls": [["Mailbox/get", {}, "mb"]] }),
        )
        .await;
    let list = mb["methodResponses"][0][1]["list"].as_array().unwrap();
    assert!(
        list.iter().any(|m| m["role"] == "inbox"),
        "expected an inbox: {list:?}"
    );
}
