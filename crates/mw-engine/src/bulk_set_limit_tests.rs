//! t55: an Email/set within the advertised `maxObjectsInSet` is applied in
//! full, although one index publication admits at most `max_batch_documents`
//! documents. Updates and destroys are published in chunks; a chunk the index
//! refuses fails only its own ids, as per-id SetErrors.
use super::*;
use crate::account::MailSubmitter;
use crate::backend::*;
use async_trait::async_trait;
use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
};
use std::sync::{
    Arc,
    atomic::{AtomicU32, AtomicUsize, Ordering},
};

#[derive(Default)]
struct Remote {
    next_uid: AtomicU32,
    expunged: AtomicUsize,
}
#[async_trait]
impl AccountBackend for Remote {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps {
            uidplus: true,
            r#move: true,
            ..BackendCaps::default()
        })
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(vec![])
    }
    async fn sync_mailbox(&self, _: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
        Ok(MailboxDelta {
            added: vec![],
            flag_changes: vec![],
            removed: vec![],
            next_cursor: c.clone(),
        })
    }
    async fn fetch_raw(&self, _: &[MessageRef]) -> Result<Vec<RawMessage>> {
        Err(EngineError::Unsupported("fixture".into()))
    }
    async fn store_flags(&self, _: &[MessageRef], _: &[Flag], _: &[Flag]) -> Result<()> {
        Ok(())
    }
    async fn move_messages(&self, refs: &[MessageRef], _: &RawMailboxRef) -> Result<MoveOutcome> {
        let first = self.next_uid.fetch_add(refs.len() as u32, Ordering::SeqCst) + 1_000_000;
        Ok(MoveOutcome::Uidplus {
            uidvalidity: 1,
            uids: (0..refs.len() as u32).map(|i| first + i).collect(),
        })
    }
    async fn append(&self, _: &RawMailboxRef, _: &[u8], _: &[Flag]) -> Result<MessageRef> {
        Err(EngineError::Unsupported("fixture".into()))
    }
    async fn expunge_messages(&self, refs: &[MessageRef]) -> Result<()> {
        self.expunged.fetch_add(refs.len(), Ordering::SeqCst);
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
    remote: Arc<Remote>,
    account: String,
    archive: String,
    ids: Vec<String>,
}

async fn fixture(messages: usize, options: Option<crate::PublicationOptions>) -> Fixture {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "fixture.invalid",
                port: 993,
                tls: "implicit",
                username: "owner@fixture.invalid",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "owner@fixture.invalid".into(),
                password: "fixture-only".into(),
            },
        )
        .await
        .unwrap();
    let mut boxes = vec![];
    for (name, role) in [("INBOX", "inbox"), ("Archive", "archive")] {
        boxes.push(
            store
                .upsert_mailbox(&MailboxUpsert {
                    account_id: &account,
                    name,
                    role: Some(role),
                    uidvalidity: 1,
                    uidnext: messages as u32 + 1,
                    highestmodseq: 0,
                    total: 0,
                    unread: 0,
                    parent_id: None,
                })
                .await
                .unwrap(),
        );
    }
    let mut ids = Vec::with_capacity(messages);
    for uid in 1..=messages as u32 {
        let message_id = format!("<bulk-{uid}@fixture.invalid>");
        ids.push(
            store
                .upsert_message(&MessageUpsert {
                    account_id: &account,
                    mailbox_id: &boxes[0],
                    uid,
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
                .unwrap(),
        );
    }
    let engine = match options {
        Some(options) => Engine::new_with_publication_options(store, options).unwrap(),
        None => Engine::new(store),
    };
    // The index holds the messages, as after a sync.
    let docs: Vec<mw_search::IndexDoc> = ids
        .iter()
        .map(|id| mw_search::IndexDoc {
            stable_id: id.clone(),
            account_id: account.clone(),
            mailbox_id: boxes[0].clone(),
            ..Default::default()
        })
        .collect();
    engine.search().upsert_batch(&docs).unwrap();
    let remote = Arc::new(Remote::default());
    engine.register_backend(
        account.clone(),
        AccountRuntime::new(remote.clone(), Arc::new(NoSend), "owner@fixture.invalid"),
    );
    Fixture {
        engine,
        remote,
        account,
        archive: boxes.remove(1),
        ids,
    }
}

impl Fixture {
    async fn set(&self, args: Value) -> Value {
        let runtime = self.engine.runtime(&self.account).unwrap();
        self.engine
            .email_set(&self.account, &runtime, &args, &mut HashMap::new())
            .await
    }
    fn update_all(&self, patch: Value) -> Value {
        let update: Map<String, Value> = self.ids.iter().map(|id| (id.clone(), patch.clone())).collect();
        json!({ "update": update })
    }
    /// `$flagged` messages as the index-backed Email/query counts them.
    async fn flagged_in_index(&self) -> u64 {
        let query = self
            .engine
            .handle_jmap(
                &self.account,
                &json!({ "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"], "methodCalls": [
                    ["Email/query", { "accountId": self.account, "filter": { "hasKeyword": "$flagged" }, "calculateTotal": true }, "q"]
                ]}),
            )
            .await;
        query["methodResponses"][0][1]["total"].as_u64().unwrap_or_else(|| panic!("{query}"))
    }
    async fn flagged(&self) -> usize {
        let mut n = 0;
        for id in &self.ids {
            if let Ok(m) = self.engine.store().get_message(id).await
                && m.flags_json.contains("Flagged")
            {
                n += 1;
            }
        }
        n
    }
}

fn assert_all_updated(response: &Value, ids: &[String]) {
    assert!(response.get("type").is_none(), "method-level error: {}", response["type"]);
    assert!(response.get("notUpdated").is_none(), "notUpdated: {}", response["notUpdated"].as_object().map_or(0, Map::len));
    assert_eq!(response["updated"].as_object().unwrap().len(), ids.len());
}

/// The `maxObjectsInSet` the Session advertises. Every bulk test below sends
/// exactly this many ids, so raising the advertised limit without making each
/// Email/set path accept it fails these tests.
fn advertised_max_objects_in_set() -> usize {
    let session = session_json("account", "owner@fixture.invalid");
    let limit = session["capabilities"]["urn:ietf:params:jmap:core"]["maxObjectsInSet"]
        .as_u64()
        .expect("maxObjectsInSet is advertised") as usize;
    // The request-size check refuses above the same number.
    assert_eq!(limit, core::MAX_OBJECTS);
    limit
}

#[test]
fn the_advertised_limit_exceeds_one_index_publication_batch() {
    // The case this file exists for: one call may name more ids than one
    // publication admits, so Email/set must split.
    assert!(advertised_max_objects_in_set() > crate::PublicationOptions::default().max_batch_documents);
}

#[tokio::test]
async fn a_keywords_update_of_max_objects_ids_is_applied_in_full() {
    let f = fixture(advertised_max_objects_in_set(), None).await;
    let response = f.set(f.update_all(json!({ "keywords/$flagged": true }))).await;
    assert_all_updated(&response, &f.ids);
    assert_eq!(f.flagged().await, f.ids.len());
    assert_eq!(f.flagged_in_index().await, f.ids.len() as u64);
}

#[tokio::test]
async fn a_move_of_max_objects_ids_is_applied_in_full() {
    let f = fixture(advertised_max_objects_in_set(), None).await;
    let response = f.set(f.update_all(json!({ "mailboxIds": { f.archive.clone(): true } }))).await;
    assert_all_updated(&response, &f.ids);
    assert_eq!(f.engine.store().list_message_ids(&f.archive, 10_000, 0).await.unwrap().len(), f.ids.len());
}

#[tokio::test]
async fn a_destroy_of_max_objects_ids_is_applied_in_full() {
    let f = fixture(advertised_max_objects_in_set(), None).await;
    let response = f.set(json!({ "destroy": f.ids })).await;
    assert!(response.get("notDestroyed").is_none(), "notDestroyed: {}", response["notDestroyed"].as_object().map_or(0, Map::len));
    assert_eq!(response["destroyed"].as_array().unwrap().len(), f.ids.len());
    assert_eq!(f.remote.expunged.load(Ordering::SeqCst), f.ids.len());
}

/// Chunk boundaries with a small batch: 23 ids in batches of 5.
#[tokio::test]
async fn updates_and_destroys_split_at_the_configured_batch_size() {
    let options = crate::PublicationOptions { max_batch_documents: 5, ..Default::default() };
    let f = fixture(23, Some(options)).await;
    let mut update: Map<String, Value> = Map::new();
    for id in &f.ids[..17] {
        update.insert(id.clone(), json!({ "keywords/$flagged": true }));
    }
    update.insert("missing-id".into(), json!({ "keywords/$flagged": true }));
    let response = f.set(json!({ "update": update, "destroy": f.ids[17..] })).await;
    assert!(response.get("type").is_none(), "{response}");
    assert_eq!(response["updated"].as_object().unwrap().len(), 17, "{response}");
    // (An unknown id is answered as before this change; not asserted here.)
    assert!(response["notUpdated"].get("missing-id").is_some(), "{response}");
    assert_eq!(response["notUpdated"].as_object().unwrap().len(), 1, "{response}");
    assert_eq!(response["destroyed"].as_array().unwrap().len(), 6, "{response}");
    assert_eq!(f.flagged().await, 17);
    // Every chunk reached the index, not only the first.
    assert_eq!(f.flagged_in_index().await, 17);
}

/// An index that refuses admission fails each id with a SetError; nothing is
/// written for a refused id, and the method itself succeeds.
#[tokio::test]
async fn an_index_admission_refusal_is_a_per_id_set_error_and_writes_nothing() {
    let options = crate::PublicationOptions { max_batch_documents: 4, ..Default::default() };
    let f = fixture(10, Some(options)).await;
    assert!(f.engine.drain_index_publication().await);
    let response = f.set(f.update_all(json!({ "keywords/$flagged": true }))).await;
    assert!(response.get("type").is_none(), "method-level error: {response}");
    let not_updated = response["notUpdated"].as_object().unwrap();
    assert_eq!(not_updated.len(), f.ids.len(), "{response}");
    assert!(
        not_updated.values().all(|e| e["type"] == "serverFail"
            && e["description"].as_str().is_some_and(|d| d.starts_with("not applied: index publication"))),
        "{response}"
    );
    assert_eq!(response["updated"].as_object().unwrap().len(), 0);
    assert_eq!(f.flagged().await, 0, "a refused id was written");
}
