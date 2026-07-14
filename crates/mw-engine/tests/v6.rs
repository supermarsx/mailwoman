//! V6 engine-wiring integration tests (plan §3 e10 acceptance): cache-aside on
//! the header-window / message-body read paths, the structural zero-access
//! plaintext exclusion at the engine boundary, and the audit / webhook feed off
//! rule executions.
//!
//! These drive the real engine over a deterministic in-process backend — no live
//! server. The regression that the *non*-zero-access + no-cache path is
//! byte-unchanged is covered by the pre-existing `engine.rs`/`v2.rs` suites,
//! which run with no cache/feed attached (the inert default).

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_cache::{AccountPosture, Cache, CacheClass, CacheLayer, ClassPolicy, ScopeMatrix};
use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, Flag, MailboxDelta, MailboxRole, MessageRef,
    MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor, WatchHandle,
};
use mw_engine::v6::{AccountPostureSource, AuditEvent, AuditFeed, V6Hooks};
use mw_engine::{Engine, MailSubmitter};
use mw_sieve::{Action, Condition, MatchOp, Rule, StringTest};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

const UIDVALIDITY: u32 = 100;

// ── A minimal single-INBOX backend: delivers a fixed message set once ────────

struct SingleInbox {
    messages: Vec<(u32, Vec<u8>)>,
    delivered: AtomicBool,
}

impl SingleInbox {
    fn new(messages: Vec<(u32, Vec<u8>)>) -> Self {
        Self {
            messages,
            delivered: AtomicBool::new(false),
        }
    }

    fn inbox_ref(&self) -> RawMailboxRef {
        RawMailboxRef {
            name: "INBOX".into(),
            uidvalidity: UIDVALIDITY,
        }
    }
}

#[async_trait]
impl AccountBackend for SingleInbox {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps::default())
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(vec![RawMailbox {
            mailbox_ref: self.inbox_ref(),
            role: MailboxRole::Inbox,
            parent: None,
            uidnext: self.messages.len() as u32 + 1,
            highestmodseq: 0,
            total: self.messages.len() as u32,
            unread: self.messages.len() as u32,
        }])
    }

    async fn sync_mailbox(
        &self,
        _mbox: &RawMailboxRef,
        _cursor: &SyncCursor,
    ) -> Result<MailboxDelta> {
        // Deliver everything on the first sync, nothing thereafter.
        let added = if self.delivered.swap(true, Ordering::SeqCst) {
            Vec::new()
        } else {
            self.messages
                .iter()
                .map(|(uid, _)| MessageRef::Imap {
                    mailbox: self.inbox_ref(),
                    uidvalidity: UIDVALIDITY,
                    uid: *uid,
                })
                .collect()
        };
        Ok(MailboxDelta {
            added,
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: UIDVALIDITY,
                uidnext: self.messages.len() as u32 + 1,
            },
        })
    }

    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        let mut out = Vec::new();
        for r in refs {
            if let MessageRef::Imap { uid, .. } = r
                && let Some((_, raw)) = self.messages.iter().find(|(u, _)| u == uid)
            {
                out.push(RawMessage {
                    message_ref: r.clone(),
                    raw: raw.clone(),
                    flags: Vec::new(),
                    internaldate: Some("2026-07-01T09:00:00Z".into()),
                });
            }
        }
        Ok(out)
    }

    async fn store_flags(&self, _refs: &[MessageRef], _add: &[Flag], _rm: &[Flag]) -> Result<()> {
        Ok(())
    }

    async fn move_messages(
        &self,
        _refs: &[MessageRef],
        _to: &RawMailboxRef,
    ) -> Result<MoveOutcome> {
        Ok(MoveOutcome::RederiveByMessageId)
    }

    async fn append(
        &self,
        _mbox: &RawMailboxRef,
        _raw: &[u8],
        _flags: &[Flag],
    ) -> Result<MessageRef> {
        Err(mw_engine::backend::EngineError::Unsupported(
            "append".into(),
        ))
    }

    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

// ── A submitter that is never dialed by these tests ──────────────────────────

struct NoopSubmitter;

#[async_trait]
impl MailSubmitter for NoopSubmitter {
    async fn submit(&self, _outgoing: Outgoing) -> Result<SubmissionResult> {
        Err(mw_engine::backend::EngineError::Unsupported(
            "submit".into(),
        ))
    }
}

// ── Test posture source + audit feed ─────────────────────────────────────────

struct ZeroAccessSet(HashSet<String>);

impl AccountPostureSource for ZeroAccessSet {
    fn posture(&self, account_id: &str) -> AccountPosture {
        if self.0.contains(account_id) {
            AccountPosture::ZeroAccess
        } else {
            AccountPosture::Standard
        }
    }
}

#[derive(Default)]
struct CollectFeed(Mutex<Vec<AuditEvent>>);

impl AuditFeed for CollectFeed {
    fn emit(&self, event: AuditEvent) {
        self.0.lock().unwrap().push(event);
    }
}

// ── Harness ──────────────────────────────────────────────────────────────────

fn msg(uid: u32, subject: &str) -> (u32, Vec<u8>) {
    let raw = format!(
        "Message-ID: <m{uid}@example.org>\r\n\
         From: sender@example.org\r\n\
         To: me@example.org\r\n\
         Subject: {subject}\r\n\
         Date: Wed, 01 Jul 2026 09:00:00 +0000\r\n\
         \r\n\
         body of {subject}\r\n"
    )
    .into_bytes();
    (uid, raw)
}

async fn engine_with(messages: Vec<(u32, Vec<u8>)>) -> (Arc<Engine>, String) {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: "me@example.org",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "me@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    let engine = Arc::new(Engine::new(store));
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            Arc::new(SingleInbox::new(messages)) as Arc<dyn AccountBackend>,
            Arc::new(NoopSubmitter) as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    (engine, account_id)
}

/// The stable id of the first message in the inbox after a resync.
async fn first_email_id(engine: &Engine, account_id: &str) -> String {
    let inbox = engine
        .store()
        .list_mailboxes(account_id)
        .await
        .unwrap()
        .into_iter()
        .find(|m| m.role.as_deref() == Some("inbox"))
        .expect("inbox");
    let req = json!({
        "methodCalls": [[
            "Email/query",
            { "filter": { "inMailbox": inbox.id }, "sort": [] },
            "c0"
        ]]
    });
    let resp = engine.handle_jmap(account_id, &req).await;
    resp["methodResponses"][0][1]["ids"][0]
        .as_str()
        .expect("at least one email id")
        .to_string()
}

/// Drive `Email/get` so `build_email` walks the cache-aside envelope path.
async fn email_get(engine: &Engine, account_id: &str, id: &str) -> Value {
    let req = json!({
        "methodCalls": [["Email/get", { "ids": [id] }, "c0"]]
    });
    engine.handle_jmap(account_id, &req).await
}

// A matrix where both header windows and message bodies are memory-tiered, so
// the exclusion + hit assertions can inspect the memory tier directly.
fn memory_tiered_matrix() -> ScopeMatrix {
    let mut m = ScopeMatrix::spec_defaults();
    m.apply_override(ClassPolicy {
        class: CacheClass::MessageBodies,
        layers: vec![CacheLayer::Memory],
        ttl_secs: 300,
    });
    m
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn standard_account_populates_the_header_window_cache() {
    let (engine, account_id) = engine_with(vec![msg(1, "Hello")]).await;
    let cache = Cache::in_memory(memory_tiered_matrix());
    engine.attach_v6(V6Hooks::new().with_cache(cache.clone()));

    engine.resync(&account_id).await.unwrap();
    let id = first_email_id(&engine, &account_id).await;

    // Not cached until the first read.
    assert!(!cache.memory_contains(CacheClass::HeaderWindows, &id));
    let resp = email_get(&engine, &account_id, &id).await;
    assert_eq!(resp["methodResponses"][0][1]["list"][0]["id"], id);

    // A standard account's envelope is now cached (a hit on the next read).
    assert!(
        cache.memory_contains(CacheClass::HeaderWindows, &id),
        "standard account envelope should populate the header-window cache"
    );
}

#[tokio::test]
async fn zero_access_account_never_materializes_plaintext_in_the_engine_cache() {
    let (engine, account_id) = engine_with(vec![msg(1, "Secret")]).await;
    let cache = Cache::in_memory(memory_tiered_matrix());
    let posture = Arc::new(ZeroAccessSet(HashSet::from([account_id.clone()])));
    engine.attach_v6(
        V6Hooks::new()
            .with_cache(cache.clone())
            .with_posture_source(posture),
    );
    assert_eq!(
        engine.account_posture(&account_id),
        AccountPosture::ZeroAccess
    );

    engine.resync(&account_id).await.unwrap();
    let id = first_email_id(&engine, &account_id).await;

    // Read through both the header-window (Email/get) and body (fetch_blob) paths.
    let _ = email_get(&engine, &account_id, &id).await;
    let _ = engine.fetch_blob(&account_id, &id).await.unwrap();
    cache.run_pending_memory_tasks().await;

    // Structural exclusion: no plaintext-derived value ever reached a cache tier.
    assert!(
        !cache.memory_contains(CacheClass::HeaderWindows, &id),
        "zero-access header window must not be cached"
    );
    assert!(
        !cache.memory_contains(CacheClass::MessageBodies, &id),
        "zero-access message body must not be cached"
    );
}

#[tokio::test]
async fn message_body_read_path_is_cache_aside() {
    let (engine, account_id) = engine_with(vec![msg(1, "Bodycache")]).await;
    let cache = Cache::in_memory(memory_tiered_matrix());
    engine.attach_v6(V6Hooks::new().with_cache(cache.clone()));

    engine.resync(&account_id).await.unwrap();
    let id = first_email_id(&engine, &account_id).await;

    // Whole-message blob download walks `message_raw` → `cached_body`.
    let blob = engine.fetch_blob(&account_id, &id).await.unwrap().unwrap();
    assert!(!blob.bytes.is_empty());
    assert!(
        cache.memory_contains(CacheClass::MessageBodies, &id),
        "standard account body should populate the message-body cache"
    );
}

#[tokio::test]
async fn rule_execution_emits_an_audit_and_webhook_feed_event() {
    let (engine, account_id) = engine_with(vec![msg(1, "Invoice 42"), msg(2, "Hello")]).await;
    let feed = Arc::new(CollectFeed::default());
    engine.attach_v6(V6Hooks::new().with_feed(feed.clone()));

    // Tag any inbox message whose subject contains "Invoice".
    engine
        .set_rules(
            &account_id,
            &[Rule {
                id: "r1".into(),
                name: "flag invoices".into(),
                match_all: true,
                conditions: vec![Condition::Subject(StringTest {
                    op: MatchOp::Contains,
                    value: "Invoice".into(),
                })],
                actions: vec![Action::Tag {
                    keyword: "important".into(),
                }],
                enabled: true,
            }],
        )
        .await
        .unwrap();

    engine.resync(&account_id).await.unwrap();

    let events = feed.0.lock().unwrap();
    assert_eq!(events.len(), 1, "exactly one rule should have fired");
    let ev = &events[0];
    assert_eq!(ev.account_id, account_id);
    assert_eq!(ev.action, "rule.executed");
    assert_eq!(ev.detail["action"], "tag");
    assert_eq!(ev.detail["keyword"], "important");
}

#[tokio::test]
async fn no_cache_and_no_feed_attached_is_a_silent_no_op() {
    // The inert default: reads still work, nothing is cached, nothing emitted.
    let (engine, account_id) = engine_with(vec![msg(1, "Plain")]).await;
    assert!(!engine.cache_attached());
    assert_eq!(
        engine.account_posture(&account_id),
        AccountPosture::Standard
    );

    engine.resync(&account_id).await.unwrap();
    let id = first_email_id(&engine, &account_id).await;
    let resp = email_get(&engine, &account_id, &id).await;
    assert_eq!(resp["methodResponses"][0][1]["list"][0]["id"], id);
}
