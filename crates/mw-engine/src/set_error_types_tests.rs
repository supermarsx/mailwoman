//! t55: Email/set per-id SetError types (RFC 8620 §5.3, RFC 8621 §4.6). One
//! test per row of the table in `.orchestration/logs/t55-e1.md` (round 4).
use super::*;
use crate::account::MailSubmitter;
use crate::backend::*;
use async_trait::async_trait;
use mw_store::{
    AccountKind, Credentials, FsUploadBackend, MailboxUpsert, MessageUpsert, NewAccount,
    ServerKey, Store,
};
use std::sync::Arc;

struct Remote;
#[async_trait]
impl AccountBackend for Remote {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps { uidplus: true, r#move: true, ..BackendCaps::default() })
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(vec![])
    }
    async fn sync_mailbox(&self, _: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
        Ok(MailboxDelta { added: vec![], flag_changes: vec![], removed: vec![], next_cursor: c.clone() })
    }
    async fn fetch_raw(&self, _: &[MessageRef]) -> Result<Vec<RawMessage>> {
        Err(EngineError::Unsupported("fixture".into()))
    }
    async fn store_flags(&self, _: &[MessageRef], _: &[Flag], _: &[Flag]) -> Result<()> {
        Ok(())
    }
    async fn move_messages(&self, refs: &[MessageRef], _: &RawMailboxRef) -> Result<MoveOutcome> {
        Ok(MoveOutcome::Uidplus { uidvalidity: 1, uids: (0..refs.len() as u32).map(|i| 500 + i).collect() })
    }
    async fn append(&self, mailbox: &RawMailboxRef, _: &[u8], _: &[Flag]) -> Result<MessageRef> {
        Ok(MessageRef::Imap { mailbox: mailbox.clone(), uidvalidity: 1, uid: 900 })
    }
    async fn expunge_messages(&self, _: &[MessageRef]) -> Result<()> {
        Ok(())
    }
    async fn watch(&self, _: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported("fixture".into()))
    }
}
struct NoSend;
#[async_trait]
impl MailSubmitter for NoSend {
    async fn submit(&self, _: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> {
        panic!("not sending")
    }
}

struct Fixture {
    engine: Engine,
    account: String,
    inbox: String,
    archive: String,
    id: String,
    foreign_id: String,
    foreign_mailbox: String,
    uploads: std::path::PathBuf,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.uploads);
    }
}

async fn account(store: &Store, user: &str) -> (String, String, String, String) {
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "fixture.invalid",
                port: 993,
                tls: "implicit",
                username: user,
                sync_policy_json: "{}",
            },
            &Credentials { username: user.into(), password: "fixture-only".into() },
        )
        .await
        .unwrap();
    let mut boxes = vec![];
    for (name, role) in [("INBOX", "inbox"), ("Archive", "archive"), ("Drafts", "drafts")] {
        boxes.push(
            store
                .upsert_mailbox(&MailboxUpsert {
                    account_id: &account,
                    name,
                    role: Some(role),
                    uidvalidity: 1,
                    uidnext: 2,
                    highestmodseq: 0,
                    total: 0,
                    unread: 0,
                    parent_id: None,
                })
                .await
                .unwrap(),
        );
    }
    let message_id = format!("<row@{user}>");
    let id = store
        .upsert_message(&MessageUpsert {
            account_id: &account,
            mailbox_id: &boxes[0],
            uid: 1,
            uidvalidity: 1,
            message_id: Some(&message_id),
            thread_id: None,
            internaldate: None,
            size: 10,
            flags_json: "[]",
            envelope: Some(b"{}"),
            blob_ref: None,
        })
        .await
        .unwrap();
    (account, boxes[0].clone(), boxes[1].clone(), id)
}

async fn fixture() -> Fixture {
    let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let uploads = std::env::temp_dir().join(format!("mw-t55-set-errors-{unique}"));
    let store = Store::open_in_memory(ServerKey::generate())
        .await
        .unwrap()
        .with_upload_backend(Arc::new(FsUploadBackend::new(uploads.clone())));
    let (account_id, inbox, archive, id) = account(&store, "owner@fixture.invalid").await;
    let (_, foreign_mailbox, _, foreign_id) = account(&store, "other@fixture.invalid").await;
    let engine = Engine::new(store);
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(Arc::new(Remote), Arc::new(NoSend), "owner@fixture.invalid"),
    );
    Fixture { engine, account: account_id, inbox, archive, id, foreign_id, foreign_mailbox, uploads }
}

impl Fixture {
    async fn set(&self, args: Value) -> Value {
        let runtime = self.engine.runtime(&self.account).unwrap();
        self.engine.email_set(&self.account, &runtime, &args, &mut HashMap::new()).await
    }
    async fn update(&self, id: &str, patch: Value) -> Value {
        let out = self.set(json!({ "update": { id: patch } })).await;
        assert!(out["updated"].get(id).is_none(), "{out}");
        out["notUpdated"][id].clone()
    }
    async fn create(&self, spec: Value) -> Value {
        let out = self.set(json!({ "create": { "c": spec } })).await;
        assert!(out["created"].get("c").is_none(), "{out}");
        out["notCreated"]["c"].clone()
    }
}

fn draft(extra: Value) -> Value {
    let mut spec = json!({
        "from": [{ "email": "owner@fixture.invalid" }],
        "to": [{ "email": "friend@example.org" }],
        "subject": "Row",
        "textBody": [{ "partId": "b", "type": "text/plain" }],
        "bodyValues": { "b": { "value": "Body" } }
    });
    for (k, v) in extra.as_object().unwrap() {
        spec[k] = v.clone();
    }
    spec
}

// ---- update ----------------------------------------------------------------

#[tokio::test]
async fn u1_an_unknown_id_is_not_found() {
    let f = fixture().await;
    assert_eq!(f.update("no-such-id", json!({ "keywords/$seen": true })).await["type"], "notFound");
}

#[tokio::test]
async fn u2_another_accounts_id_is_answered_exactly_like_an_unknown_one() {
    let f = fixture().await;
    let foreign = f.update(&f.foreign_id, json!({ "keywords/$seen": true })).await;
    let unknown = f.update("no-such-id", json!({ "keywords/$seen": true })).await;
    assert_eq!(foreign["type"], "notFound");
    assert_eq!(foreign, unknown, "no oracle for another owner's id");
}

#[tokio::test]
async fn u3_an_empty_patch_is_invalid_patch() {
    let f = fixture().await;
    assert_eq!(f.update(&f.id, json!({})).await["type"], "invalidPatch");
}

#[tokio::test]
async fn u4_a_property_update_cannot_apply_is_invalid_properties() {
    let f = fixture().await;
    let e = f.update(&f.id, json!({ "subject": "rewritten" })).await;
    assert_eq!(e["type"], "invalidProperties");
    assert_eq!(e["properties"], json!(["subject"]));
}

#[tokio::test]
async fn u5_a_move_to_no_mailbox_of_this_account_is_invalid_properties_and_writes_nothing() {
    let f = fixture().await;
    for target in ["no-such-mailbox".to_string(), f.foreign_mailbox.clone()] {
        let e = f.update(&f.id, json!({ "mailboxIds": { target: true } })).await;
        assert_eq!(e["type"], "invalidProperties", "{e}");
        assert_eq!(e["properties"], json!(["mailboxIds"]));
    }
    assert_eq!(f.engine.store().get_message(&f.id).await.unwrap().mailbox_id, f.inbox);
}

#[tokio::test]
async fn u6_two_mailboxes_is_invalid_properties() {
    let f = fixture().await;
    let e = f.update(&f.id, json!({ "mailboxIds": { f.inbox.clone(): true, f.archive.clone(): true } })).await;
    assert_eq!(e["type"], "invalidProperties", "{e}");
}

#[tokio::test]
async fn u7_an_id_the_same_call_destroys_is_will_destroy() {
    let f = fixture().await;
    let out = f.set(json!({ "update": { f.id.clone(): { "keywords/$seen": true } }, "destroy": [f.id] })).await;
    assert_eq!(out["notUpdated"][&f.id]["type"], "willDestroy", "{out}");
}

#[tokio::test]
async fn u8_a_state_mismatch_stays_method_level() {
    let f = fixture().await;
    let out = f.set(json!({ "ifInState": "no-such-state", "update": { f.id.clone(): { "keywords/$seen": true } } })).await;
    assert_eq!(out["type"], "stateMismatch", "{out}");
}

#[tokio::test]
async fn u9_a_valid_move_still_applies() {
    let f = fixture().await;
    let out = f.set(json!({ "update": { f.id.clone(): { "mailboxIds": { f.archive.clone(): true } } } })).await;
    assert!(out["updated"].as_object().unwrap().contains_key(&f.id), "{out}");
}

// ---- destroy ---------------------------------------------------------------

#[tokio::test]
async fn d1_destroying_an_unknown_or_foreign_id_is_not_found() {
    let f = fixture().await;
    let out = f.set(json!({ "destroy": ["no-such-id", f.foreign_id] })).await;
    assert_eq!(out["notDestroyed"]["no-such-id"]["type"], "notFound", "{out}");
    assert_eq!(out["notDestroyed"][&f.foreign_id]["type"], "notFound", "{out}");
}

// ---- create ----------------------------------------------------------------

#[tokio::test]
async fn c1_an_attachment_blob_that_does_not_resolve_is_blob_not_found() {
    let f = fixture().await;
    let e = f.create(draft(json!({ "attachments": [{ "blobId": "0000deadbeef.9" }] }))).await;
    assert_eq!(e["type"], "blobNotFound", "{e}");
    assert_eq!(e["notFound"], json!(["0000deadbeef.9"]));
}

#[tokio::test]
async fn c2_attachments_over_the_composition_limit_are_too_large() {
    let f = fixture().await;
    let blob = f
        .engine
        .store_upload(&f.account, "application/octet-stream", &vec![0u8; 33 * 1024 * 1024])
        .await
        .unwrap();
    let e = f.create(draft(json!({ "attachments": [{ "blobId": blob, "type": "application/octet-stream", "name": "big.bin" }] }))).await;
    assert_eq!(e["type"], "tooLarge", "{e}");
}

#[tokio::test]
async fn c3_an_address_that_cannot_be_written_is_invalid_properties() {
    let f = fixture().await;
    let e = f.create(draft(json!({ "to": [{ "email": "friend@example.org>\r\nBcc: x@y" }] }))).await;
    assert_eq!(e["type"], "invalidProperties", "{e}");
}
