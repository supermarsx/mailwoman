//! t14-E-e2e — LEG 4: blob-attachment forward — the SENT message carries the part.
//!
//! WS4a (E6) honors `Email/set` create `attachments` whose `blobId` resolves to an
//! EXISTING stored message part via `Engine::fetch_blob` (forward / attach-from-mail).
//! E6's engine test proves the composed DRAFT is multipart carrying the part; this
//! leg proves the part survives the real SEND path — a chained `Email/set` create +
//! inline `EmailSubmission/set` (hold=0) — so the `Outgoing` that reaches the wire is
//! a multipart message carrying the forwarded PDF part. Driven over a REAL store
//! (SQLite always; live Postgres via `MW_E14_PG_DSN`), ingesting the source message
//! through a real `resync` so the engine computes the part's blobId itself.
//!
//! ## Running
//!   cargo test -p mw-server --test t14_blob_attach                      # SQLite always
//!   docker compose -f docker-compose.ci.yml up -d --wait postgres
//!   MW_E14_PG_DSN=postgres://mailwoman:mailwoman@localhost:5432/mailwoman \
//!     cargo test -p mw-server --test t14_blob_attach -- --nocapture --test-threads=1

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::Engine;
use mw_engine::account::{AccountRuntime, MailSubmitter};
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MailboxRole,
    MessageRef, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result as BackendResult,
    SyncCursor, WatchHandle,
};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

const UIDVALIDITY: u32 = 100;

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

/// A multipart/mixed invoice: a text part + a base64 `application/pdf` attachment
/// whose decoded bytes are `%PDF-1.4\n` (base64 `JVBERi0xLjQK`).
fn invoice_msg() -> Vec<u8> {
    concat!(
        "Message-ID: <invoice@x>\r\n",
        "From: alice@example.org\r\n",
        "To: me@example.org\r\n",
        "Subject: Invoice 2026\r\n",
        "Date: Wed, 01 Jul 2026 09:00:00 +0000\r\n",
        "MIME-Version: 1.0\r\n",
        "Content-Type: multipart/mixed; boundary=\"b1\"\r\n",
        "\r\n",
        "--b1\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "Please pay the attached invoice\r\n",
        "--b1\r\n",
        "Content-Type: application/pdf; name=\"invoice.pdf\"\r\n",
        "Content-Disposition: attachment; filename=\"invoice.pdf\"\r\n",
        "Content-Transfer-Encoding: base64\r\n",
        "\r\n",
        "JVBERi0xLjQK\r\n",
        "--b1--\r\n",
    )
    .as_bytes()
    .to_vec()
}

/// Serves exactly one invoice message on INBOX so `resync` ingests it and the engine
/// computes the attachment part's blobId. `append` is a no-op (best-effort Sent).
struct InvoiceBackend;

#[async_trait]
impl AccountBackend for InvoiceBackend {
    async fn capabilities(&self) -> BackendResult<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> BackendResult<Vec<RawMailbox>> {
        Ok(vec![RawMailbox {
            mailbox_ref: RawMailboxRef {
                name: "INBOX".into(),
                uidvalidity: UIDVALIDITY,
            },
            role: MailboxRole::Inbox,
            parent: None,
            uidnext: 2,
            highestmodseq: 0,
            total: 1,
            unread: 1,
        }])
    }
    async fn sync_mailbox(
        &self,
        mbox: &RawMailboxRef,
        cursor: &SyncCursor,
    ) -> BackendResult<MailboxDelta> {
        let from = match cursor {
            SyncCursor::UidWindow { uidnext, .. } => *uidnext,
            _ => 1,
        };
        let added = if from <= 1 {
            vec![MessageRef::Imap {
                mailbox: mbox.clone(),
                uidvalidity: UIDVALIDITY,
                uid: 1,
            }]
        } else {
            Vec::new()
        };
        Ok(MailboxDelta {
            added,
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: UIDVALIDITY,
                uidnext: 2,
            },
        })
    }
    async fn fetch_raw(&self, refs: &[MessageRef]) -> BackendResult<Vec<RawMessage>> {
        Ok(refs
            .iter()
            .map(|r| RawMessage {
                message_ref: r.clone(),
                raw: invoice_msg(),
                flags: Vec::new(),
                internaldate: Some("2026-07-01T09:00:00Z".into()),
            })
            .collect())
    }
    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _rm: &[Flag]) -> BackendResult<()> {
        Ok(())
    }
    async fn move_messages(
        &self,
        _r: &[MessageRef],
        _to: &RawMailboxRef,
    ) -> BackendResult<MoveOutcome> {
        Err(EngineError::Unsupported("invoice backend".into()))
    }
    async fn append(
        &self,
        _mbox: &RawMailboxRef,
        _raw: &[u8],
        _flags: &[Flag],
    ) -> BackendResult<MessageRef> {
        Err(EngineError::Unsupported("invoice backend".into()))
    }
    async fn watch(&self, _sink: ChangeSink) -> BackendResult<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

/// Captures the `Outgoing` that reaches the wire.
#[derive(Default)]
struct CaptureInner {
    last: Mutex<Option<Outgoing>>,
    calls: AtomicUsize,
}

#[async_trait]
impl MailSubmitter for CaptureInner {
    async fn submit(&self, msg: Outgoing) -> BackendResult<SubmissionResult> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let accepted = msg.rcpt_to.clone();
        *self.last.lock().unwrap() = Some(msg);
        Ok(SubmissionResult {
            accepted,
            rejected: Vec::new(),
        })
    }
}

async fn jmap(engine: &Engine, account_id: &str, calls: Value) -> Value {
    engine
        .handle_jmap(account_id, &json!({ "methodCalls": calls }))
        .await
}

fn method_result<'a>(resp: &'a Value, call_id: &str) -> &'a Value {
    resp["methodResponses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r[2] == call_id)
        .map(|r| &r[1])
        .unwrap_or(&Value::Null)
}

/// The `<stableId>.<partId>` blobId of the ingested invoice's PDF attachment.
async fn invoice_attachment_blob(engine: &Engine, account_id: &str) -> String {
    let mb = jmap(engine, account_id, json!([["Mailbox/get", {}, "mb"]])).await;
    let inbox = method_result(&mb, "mb")["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "inbox")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let q = jmap(
        engine,
        account_id,
        json!([[
            "Email/query",
            { "filter": { "inMailbox": inbox, "hasAttachment": true } },
            "q"
        ]]),
    )
    .await;
    let id = method_result(&q, "q")["ids"][0]
        .as_str()
        .expect("ingested invoice has an attachment")
        .to_string();
    let g = jmap(
        engine,
        account_id,
        json!([[
            "Email/get",
            { "ids": [id], "properties": ["attachments"] },
            "g"
        ]]),
    )
    .await;
    method_result(&g, "g")["list"][0]["attachments"][0]["blobId"]
        .as_str()
        .expect("attachment blobId")
        .to_string()
}

/// Compose a forward referencing `att_blob` and submit it inline in one request.
fn forward_and_submit(att_blob: &str) -> Value {
    json!([
        ["Email/set", { "create": { "draft": {
            "to": [{ "email": "boss@example.org" }],
            "subject": "Fwd: Invoice 2026",
            "from": [{ "email": "me@example.org" }],
            "bodyValues": { "1": { "value": "forwarding the invoice" } },
            "textBody": [{ "partId": "1", "type": "text/plain" }],
            "attachments": [{ "blobId": att_blob, "type": "application/pdf", "name": "invoice.pdf" }]
        } } }, "c1"],
        ["EmailSubmission/set", { "create": { "sub1": {
            "emailId": "#draft",
            "mailwomanHoldSeconds": 0
        } } }, "c2"]
    ])
}

async fn drive(store: Store, dialect: &str) {
    let uname = format!("me-{}@example.org", unique());
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: &uname,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: uname.clone(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    let engine = std::sync::Arc::new(Engine::new(store));
    let inner = std::sync::Arc::new(CaptureInner::default());
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            std::sync::Arc::new(InvoiceBackend) as std::sync::Arc<dyn AccountBackend>,
            inner.clone() as std::sync::Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    engine
        .resync(&account_id)
        .await
        .expect("resync ingests invoice");

    let att_blob = invoice_attachment_blob(&engine, &account_id).await;
    // The blobId must address an existing stored part: `<stableId>.<partId>`.
    assert!(
        att_blob.contains('.'),
        "[{dialect}] attachment blobId names a stored part: {att_blob}"
    );

    let resp = jmap(&engine, &account_id, forward_and_submit(&att_blob)).await;
    assert!(
        method_result(&resp, "c1")["created"]["draft"]["id"].is_string(),
        "[{dialect}] forward draft created (no notCreated): {resp}"
    );
    assert!(
        method_result(&resp, "c1").get("notCreated").is_none(),
        "[{dialect}] no notCreated (blob resolved): {resp}"
    );
    assert!(
        method_result(&resp, "c2")["created"]["sub1"]["id"].is_string(),
        "[{dialect}] submission accepted + sent inline: {resp}"
    );
    assert_eq!(
        inner.calls.load(Ordering::SeqCst),
        1,
        "[{dialect}] the forward reached the wire exactly once"
    );

    // The SENT bytes are a multipart message carrying the forwarded PDF part.
    let sent = inner
        .last
        .lock()
        .unwrap()
        .clone()
        .expect("captured Outgoing");
    let raw = String::from_utf8_lossy(&sent.raw);
    assert!(
        raw.contains("multipart/"),
        "[{dialect}] sent message is multipart: {raw}"
    );
    assert!(
        raw.contains("application/pdf"),
        "[{dialect}] sent message carries the application/pdf part type: {raw}"
    );
    assert!(
        raw.contains("invoice.pdf"),
        "[{dialect}] sent message carries the invoice.pdf filename: {raw}"
    );
    // The forwarded PDF bytes (`%PDF-1.4\n`) ride the wire, base64 `JVBERi0xLjQK`.
    assert!(
        raw.contains("JVBERi0xLjQK"),
        "[{dialect}] sent message carries the forwarded PDF part bytes: {raw}"
    );
}

#[tokio::test]
async fn forwarded_blob_attachment_rides_the_sent_message_sqlite() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    drive(store, "sqlite").await;
}

#[tokio::test]
async fn forwarded_blob_attachment_rides_the_sent_message_postgres() {
    let Ok(dsn) = std::env::var("MW_E14_PG_DSN") else {
        eprintln!("\n[t14 blob SKIP] MW_E14_PG_DSN unset — live Postgres send path not driven.\n");
        return;
    };
    let store = Store::open(&dsn, ServerKey::generate())
        .await
        .expect("open live Postgres store");
    drive(store, "postgres").await;
}
