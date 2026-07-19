//! t16-e-e2e — conversation threading over REAL JWZ thread data (W2 backend proof).
//!
//! The web builds the grouped conversation list (W2) over the `threadId` the engine
//! attaches to every message. "unit-green != wired": the JWZ algorithm is unit-tested
//! in `mw-engine`, but THIS leg drives the whole ingest→thread→`Thread/get` seam
//! against a real store (SQLite AND live Postgres) through the JMAP surface the client
//! actually calls — three messages linked by `References`/`In-Reply-To` are ingested
//! through the genuine sync/fetch pipeline, converge onto ONE JWZ thread, and
//! `Thread/get` returns their ids oldest-first (the grouped-list data W2 renders).

use std::sync::Arc;

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

const UIDVALIDITY: u32 = 77;

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

/// One RFC822 message with optional threading headers.
fn raw_msg(mid: &str, subject: &str, in_reply_to: Option<&str>, refs: &[&str]) -> Vec<u8> {
    let mut s = String::new();
    s.push_str(&format!("Message-ID: {mid}\r\n"));
    s.push_str("From: sender@example.com\r\n");
    s.push_str("To: rcpt@example.com\r\n");
    s.push_str(&format!("Subject: {subject}\r\n"));
    if let Some(irt) = in_reply_to {
        s.push_str(&format!("In-Reply-To: {irt}\r\n"));
    }
    if !refs.is_empty() {
        s.push_str(&format!("References: {}\r\n", refs.join(" ")));
    }
    s.push_str("Date: Mon, 01 Jan 2024 00:00:00 +0000\r\n\r\nbody\r\n");
    s.into_bytes()
}

/// A backend whose INBOX contains a three-message JWZ thread (root + two replies).
struct ThreadBackend {
    msgs: Vec<(u32, Vec<u8>, String)>, // (uid, raw, internaldate)
}

impl ThreadBackend {
    fn thread_of_three() -> Self {
        ThreadBackend {
            msgs: vec![
                (
                    10,
                    raw_msg("<root@x>", "Quarterly plan", None, &[]),
                    "2024-01-01T00:00:00Z".into(),
                ),
                (
                    11,
                    raw_msg(
                        "<r1@x>",
                        "Re: Quarterly plan",
                        Some("<root@x>"),
                        &["<root@x>"],
                    ),
                    "2024-01-02T00:00:00Z".into(),
                ),
                (
                    12,
                    raw_msg(
                        "<r2@x>",
                        "Re: Quarterly plan",
                        Some("<r1@x>"),
                        &["<root@x>", "<r1@x>"],
                    ),
                    "2024-01-03T00:00:00Z".into(),
                ),
            ],
        }
    }

    fn refs(&self) -> Vec<MessageRef> {
        self.msgs
            .iter()
            .map(|(uid, _, _)| MessageRef::Imap {
                mailbox: RawMailboxRef {
                    name: "INBOX".into(),
                    uidvalidity: UIDVALIDITY,
                },
                uidvalidity: UIDVALIDITY,
                uid: *uid,
            })
            .collect()
    }
}

#[async_trait]
impl AccountBackend for ThreadBackend {
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
            uidnext: 13,
            highestmodseq: 0,
            total: self.msgs.len() as u32,
            unread: self.msgs.len() as u32,
        }])
    }
    async fn sync_mailbox(
        &self,
        _mbox: &RawMailboxRef,
        _cursor: &SyncCursor,
    ) -> BackendResult<MailboxDelta> {
        Ok(MailboxDelta {
            added: self.refs(),
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: UIDVALIDITY,
                uidnext: 13,
            },
        })
    }
    async fn fetch_raw(&self, refs: &[MessageRef]) -> BackendResult<Vec<RawMessage>> {
        let mut out = Vec::new();
        for r in refs {
            if let MessageRef::Imap { uid, .. } = r
                && let Some((_, raw, when)) = self.msgs.iter().find(|(u, _, _)| u == uid)
            {
                out.push(RawMessage {
                    message_ref: r.clone(),
                    raw: raw.clone(),
                    flags: Vec::new(),
                    internaldate: Some(when.clone()),
                });
            }
        }
        Ok(out)
    }
    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _rm: &[Flag]) -> BackendResult<()> {
        Ok(())
    }
    async fn move_messages(
        &self,
        _r: &[MessageRef],
        _to: &RawMailboxRef,
    ) -> BackendResult<MoveOutcome> {
        Err(EngineError::Unsupported("thread backend".into()))
    }
    async fn append(
        &self,
        _mbox: &RawMailboxRef,
        _raw: &[u8],
        _flags: &[Flag],
    ) -> BackendResult<MessageRef> {
        Err(EngineError::Unsupported("thread backend".into()))
    }
    async fn watch(&self, _sink: ChangeSink) -> BackendResult<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

struct NoSubmit;
#[async_trait]
impl MailSubmitter for NoSubmit {
    async fn submit(&self, msg: Outgoing) -> BackendResult<SubmissionResult> {
        Ok(SubmissionResult {
            accepted: msg.rcpt_to.clone(),
            rejected: Vec::new(),
        })
    }
}

async fn make_account(store: &Store) -> String {
    let uname = format!("me-{}@example.org", unique());
    store
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
        .unwrap()
}

async fn jmap(engine: &Engine, account: &str, calls: Value) -> Value {
    engine
        .handle_jmap(account, &json!({ "methodCalls": calls }))
        .await
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

/// Ingest the 3-message thread through the real sync/fetch pipeline, then assert the
/// grouped-list + `Thread/get` behaviour over `dialect`'s real store.
async fn drive(store: Store, dialect: &str) {
    let account = make_account(&store).await;
    let engine = Arc::new(Engine::new(store));
    engine.register_backend(
        account.clone(),
        AccountRuntime::new(
            Arc::new(ThreadBackend::thread_of_three()) as Arc<dyn AccountBackend>,
            Arc::new(NoSubmit) as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    engine
        .resync(&account)
        .await
        .expect("resync ingests the thread");
    let store = engine.store();

    // The three messages landed in INBOX (the flat rows the client lists).
    let inbox = store
        .list_mailboxes(&account)
        .await
        .unwrap()
        .into_iter()
        .find(|m| m.name == "INBOX")
        .expect("INBOX exists after resync");
    let ids = store.list_message_ids(&inbox.id, 100, 0).await.unwrap();
    assert_eq!(ids.len(), 3, "[{dialect}] all three messages ingested");

    // They converge onto ONE JWZ thread id (the grouping key W2 renders over).
    let mut thread_ids = std::collections::HashSet::new();
    for id in &ids {
        let m = store.get_message(id).await.unwrap();
        thread_ids.insert(m.thread_id.expect("every message has a threadId"));
    }
    assert_eq!(
        thread_ids.len(),
        1,
        "[{dialect}] the three messages converge onto ONE JWZ thread"
    );
    let thread_id = thread_ids.into_iter().next().unwrap();

    // Thread/get (RFC 8621 §3) over the real store — the grouped conversation the W2
    // list view renders, ids oldest-first.
    let t = jmap(
        &engine,
        &account,
        json!([["Thread/get", { "accountId": account, "ids": [thread_id] }, "t"]]),
    )
    .await;
    let tlist = result(&t, "t")["list"].as_array().unwrap();
    assert_eq!(tlist.len(), 1, "[{dialect}] one thread returned: {t}");
    let email_ids: Vec<&str> = tlist[0]["emailIds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(
        email_ids.len(),
        3,
        "[{dialect}] the thread groups all three messages: {t}"
    );
    // Oldest-first ordering (RFC 8621): the root (earliest internaldate) precedes its
    // replies. Identify the root by its Message-ID header.
    let root_mid = store.get_message(email_ids[0]).await.unwrap().message_id;
    // The store persists Message-IDs with angle brackets stripped.
    assert_eq!(
        root_mid.as_deref(),
        Some("root@x"),
        "[{dialect}] the thread root sorts first (oldest-first): {t}"
    );
    assert!(
        result(&t, "t")["notFound"].as_array().unwrap().is_empty(),
        "[{dialect}] no spurious notFound"
    );
}

#[tokio::test]
async fn threading_groups_real_jwz_data_sqlite() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    drive(store, "sqlite").await;
}

#[tokio::test]
async fn threading_groups_real_jwz_data_postgres() {
    let Ok(dsn) = std::env::var("MW_E14_PG_DSN") else {
        eprintln!(
            "\n[t16 threading SKIP] MW_E14_PG_DSN unset — JWZ threading not driven on live Postgres.\n"
        );
        return;
    };
    let store = Store::open(&dsn, ServerKey::generate())
        .await
        .expect("open live Postgres store");
    drive(store, "postgres").await;
}
