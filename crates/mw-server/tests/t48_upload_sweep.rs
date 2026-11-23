//! t48: the upload sweep (`gc_uploads`, `mailwoman maintenance gc-uploads`) must
//! not take the attachment of a send that is still waiting.
//!
//! A message with an uploaded attachment is scheduled three days out. The sweep
//! then runs with a cutoff past every upload. The submission fires when the
//! dispatcher is stepped four days ahead, and the message on the wire carries
//! the uploaded bytes. A second upload that nothing refers to is removed by the
//! same sweep, so the sweep did run.
//!
//! The attachment survives because `Email/set` copies the upload into the
//! composed message at create time and the submission names that message, not
//! the upload. This test pins that: if a later change makes submissions refer
//! to uploads directly, the sweep has to learn about it or this test fails.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

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
use mw_store::{AccountKind, Credentials, FsUploadBackend, NewAccount, ServerKey, Store};

mod common;
use common::test_db;

const PAYLOAD: &[u8] = b"scheduled-send-attachment-0123456789";

struct EmptyBackend;

#[async_trait]
impl AccountBackend for EmptyBackend {
    async fn capabilities(&self) -> BackendResult<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> BackendResult<Vec<RawMailbox>> {
        Ok(vec![RawMailbox {
            mailbox_ref: RawMailboxRef {
                name: "INBOX".into(),
                uidvalidity: 100,
            },
            role: MailboxRole::Inbox,
            parent: None,
            uidnext: 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
        }])
    }
    async fn sync_mailbox(
        &self,
        _mbox: &RawMailboxRef,
        _cursor: &SyncCursor,
    ) -> BackendResult<MailboxDelta> {
        Ok(MailboxDelta {
            added: Vec::new(),
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: 100,
                uidnext: 1,
            },
        })
    }
    async fn fetch_raw(&self, _refs: &[MessageRef]) -> BackendResult<Vec<RawMessage>> {
        Ok(Vec::new())
    }
    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _rm: &[Flag]) -> BackendResult<()> {
        Ok(())
    }
    async fn move_messages(
        &self,
        _r: &[MessageRef],
        _to: &RawMailboxRef,
    ) -> BackendResult<MoveOutcome> {
        Err(EngineError::Unsupported("empty backend".into()))
    }
    async fn append(
        &self,
        _mbox: &RawMailboxRef,
        _raw: &[u8],
        _flags: &[Flag],
    ) -> BackendResult<MessageRef> {
        Err(EngineError::Unsupported("empty backend".into()))
    }
    async fn watch(&self, _sink: ChangeSink) -> BackendResult<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

#[derive(Default)]
struct Capture {
    last: Mutex<Option<Outgoing>>,
    calls: AtomicUsize,
}

#[async_trait]
impl MailSubmitter for Capture {
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

fn result<'a>(resp: &'a Value, call_id: &str) -> &'a Value {
    resp["methodResponses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r[2] == call_id)
        .map(|r| &r[1])
        .unwrap_or(&Value::Null)
}

async fn drive(store: Store, root: &std::path::Path, dialect: &str) {
    let uname = format!("me-{}@example.org", test_db::unique_tag());
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
    let engine = Arc::new(Engine::new(store.clone()));
    let capture = Arc::new(Capture::default());
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            Arc::new(EmptyBackend) as Arc<dyn AccountBackend>,
            capture.clone() as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    engine.resync(&account_id).await.expect("resync");

    let attached = engine
        .store_upload(&account_id, "application/octet-stream", PAYLOAD)
        .await
        .unwrap();
    let unused = engine
        .store_upload(&account_id, "text/plain", b"never attached")
        .await
        .unwrap();

    let send_at = (chrono::Utc::now() + chrono::Duration::days(3))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    let resp = engine
        .handle_jmap(
            &account_id,
            &json!({ "methodCalls": [
                ["Email/set", { "create": { "draft": {
                    "to": [{ "email": "boss@example.org" }],
                    "subject": "Scheduled with attachment",
                    "from": [{ "email": "me@example.org" }],
                    "bodyValues": { "1": { "value": "see attached" } },
                    "textBody": [{ "partId": "1", "type": "text/plain" }],
                    "attachments": [{ "blobId": attached, "type": "application/octet-stream", "name": "report.bin" }]
                } } }, "c1"],
                ["EmailSubmission/set", { "create": { "sub": {
                    "emailId": "#draft",
                    "sendAt": send_at
                } } }, "c2"]
            ] }),
        )
        .await;
    assert!(
        result(&resp, "c2")["created"]["sub"]["id"].is_string(),
        "[{dialect}] scheduled submission accepted: {resp}"
    );
    assert_eq!(capture.calls.load(Ordering::SeqCst), 0, "[{dialect}] not sent yet");

    // The sweep runs with its cutoff past both uploads.
    let past_all = chrono::Duration::seconds(-5);
    let removed = store.sweep_uploads(past_all).await.unwrap();
    assert!(
        store.get_upload(&account_id, &unused).await.unwrap().is_none(),
        "[{dialect}] the unattached upload is removed ({removed} removed)"
    );

    // Three days later the submission fires with the attachment intact.
    engine
        .dispatch_tick_at(chrono::Utc::now() + chrono::Duration::days(4))
        .await
        .unwrap();
    assert_eq!(
        capture.calls.load(Ordering::SeqCst),
        1,
        "[{dialect}] the scheduled message was sent"
    );
    let sent = capture.last.lock().unwrap().clone().unwrap();
    let raw = String::from_utf8_lossy(&sent.raw);
    let flat: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    use base64::Engine as _;
    let b64 = base64::engine::general_purpose::STANDARD.encode(PAYLOAD);
    assert!(raw.contains("report.bin"), "[{dialect}] attachment name on the wire: {raw}");
    assert!(
        flat.contains(&b64),
        "[{dialect}] attachment bytes on the wire after the sweep: {raw}"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn scheduled_send_keeps_its_attachment_after_upload_sweep_sqlite() {
    let root = test_db::unique_dir("mw-t48-upload-sweep");
    let store = Store::open_in_memory(ServerKey::generate())
        .await
        .unwrap()
        .with_upload_backend(Arc::new(FsUploadBackend::new(root.clone())));
    drive(store, &root, "sqlite").await;
}

#[tokio::test]
async fn scheduled_send_keeps_its_attachment_after_upload_sweep_postgres() {
    let Some(dsn) = common::gate::pg_dsn() else {
        common::gate::skip(
            "[t48 upload sweep] MW_E14_PG_DSN and DATABASE_URL_PG unset — live Postgres not driven.",
        );
        return;
    };
    let root = test_db::unique_dir("mw-t48-upload-sweep-pg");
    let store = Store::open(&dsn, ServerKey::generate())
        .await
        .expect("open live Postgres store")
        .with_upload_backend(Arc::new(FsUploadBackend::new(root.clone())));
    drive(store, &root, "postgres").await;
}
