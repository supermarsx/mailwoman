//! Real stored MIME -> rebuilt index -> paginated JMAP query and projection.
use super::*;
use crate::backend::*;
use async_trait::async_trait;
use mw_store::{
    AccountKind, Credentials, IdentityRow, MailboxUpsert, MaskedEmailRow, MessageUpsert,
    NewAccount, ServerKey, Store,
};
use std::sync::Arc;
struct Offline;
#[async_trait]
impl AccountBackend for Offline {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(vec![])
    }
    async fn sync_mailbox(&self, _m: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
        Ok(MailboxDelta {
            added: vec![],
            flag_changes: vec![],
            removed: vec![],
            next_cursor: c.clone(),
        })
    }
    async fn fetch_raw(&self, _r: &[MessageRef]) -> Result<Vec<RawMessage>> {
        panic!("test must read stored MIME")
    }
    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _d: &[Flag]) -> Result<()> {
        Ok(())
    }
    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("offline".into()))
    }
    async fn append(&self, _m: &RawMailboxRef, _raw: &[u8], _f: &[Flag]) -> Result<MessageRef> {
        Err(EngineError::Unsupported("offline".into()))
    }
    async fn watch(&self, _s: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported("offline".into()))
    }
}
#[async_trait]
impl crate::account::MailSubmitter for Offline {
    async fn submit(&self, _msg: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> {
        panic!("test never sends mail")
    }
}
#[tokio::test]
async fn mentions_me_queries_all_pages_and_authorized_aliases_without_recipient_false_positives() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "h",
                port: 993,
                tls: "implicit",
                username: "me@example.org",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "me@example.org".into(),
                password: "p".into(),
            },
        )
        .await
        .unwrap();
    store
        .upsert_native_personal_identity(&IdentityRow {
            id: "alias".into(),
            account_id: account.clone(),
            name: "Alias".into(),
            email: "alias@example.org".into(),
            source: "configured".into(),
            ..IdentityRow::default()
        })
        .await
        .unwrap();
    store
        .put_masked_email(&MaskedEmailRow {
            id: "alias".into(),
            account_id: account.clone(),
            alias_addr: "alias@example.org".into(),
            target_desc: "me@example.org".into(),
            state: "enabled".into(),
            created_at: "2026-11-09T00:00:00Z".into(),
            last_used_at: None,
        })
        .await
        .unwrap();
    let mut boxes = vec![];
    for name in ["INBOX", "Other"] {
        boxes.push(
            store
                .upsert_mailbox(&MailboxUpsert {
                    account_id: &account,
                    name,
                    role: None,
                    uidvalidity: 1,
                    uidnext: 1,
                    highestmodseq: 0,
                    total: 0,
                    unread: 0,
                    parent_id: None,
                })
                .await
                .unwrap(),
        );
    }
    let bodies = [
        "<p>Hi <a href='mailto:me%40example.org'>@Me</a></p>",
        "<p>Hi <a href='mailto:alias%40example.org'><strong>@Alias</strong></a></p>",
        "<p>Ordinary mail to me, without a mention.</p>",
        "<blockquote>@me@example.org</blockquote>",
        "<p>@me@example.org.evil</p>",
    ];
    let mut ids = vec![];
    for (index, body) in bodies.iter().enumerate() {
        let raw = format!(
            "From: sender@example.org\r\nTo: me@example.org\r\nSubject: mention-{index}\r\nDate: Mon, 09 Nov 2026 09:00:0{index} +0000\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{body}"
        );
        let blob = store.put_body(&account, raw.as_bytes()).await.unwrap();
        ids.push(
            store
                .upsert_message(&MessageUpsert {
                    account_id: &account,
                    mailbox_id: &boxes[index % 2],
                    uid: index as u32 + 1,
                    uidvalidity: 1,
                    message_id: None,
                    thread_id: None,
                    internaldate: None,
                    size: raw.len() as u64,
                    flags_json: "[]",
                    envelope: None,
                    blob_ref: Some(&blob),
                })
                .await
                .unwrap(),
        );
    }
    let engine = Engine::new(store);
    engine.register_backend(
        &account,
        AccountRuntime::new(Arc::new(Offline), Arc::new(Offline), "me@example.org"),
    );
    assert!(engine.reconcile_search_index().await.unwrap().is_some());
    let query = |position| json!({"filter":{"text":"mentions:me"},"calculateTotal":true,"position":position,"limit":1});
    let first = engine.email_query(&account, &query(0)).await;
    let second = engine.email_query(&account, &query(1)).await;
    assert_eq!(first["total"], json!(2), "{first}");
    assert_eq!(second["total"], json!(2));
    let mut actual = vec![
        first["ids"][0].as_str().unwrap().to_owned(),
        second["ids"][0].as_str().unwrap().to_owned(),
    ];
    actual.sort();
    let mut expected = ids[..2].to_vec();
    expected.sort();
    assert_eq!(actual, expected);
    let got = engine
        .email_get(
            &account,
            &json!({"ids":ids,"properties":["id","keywords","mailwomanMentions"]}),
        )
        .await;
    let rows = got["list"].as_array().unwrap();
    assert_eq!(rows[0]["mailwomanMentions"], json!(["me@example.org"]));
    assert_eq!(rows[1]["mailwomanMentions"], json!(["alias@example.org"]));
    assert_eq!(rows[2]["mailwomanMentions"], json!([]));
    for row in rows {
        assert!(
            !row["keywords"]
                .as_object()
                .unwrap()
                .keys()
                .any(|key| key.starts_with(crate::mentions::PREFIX))
        );
    }
}
