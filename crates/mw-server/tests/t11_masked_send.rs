//! t11-e3 live-E2E — masked-email on-send From-rewrite through the REAL submission path.
//!
//! Proves the 26.11 slice `67c79d5` (masked on-send From-rewrite via the `MaskedSubmitter`
//! decorator) is not just unit-green but WIRED and COMPOSES: the real `MaskedSubmitter`
//! (constructed exactly as `engine_mode::register` constructs it —
//! `MaskedSubmitter::new(engine.store().clone(), account_id, inner)`) sits in a registered
//! standards `AccountRuntime.submitter` slot, and a real JMAP `Email/set` (draft) +
//! `EmailSubmission/set` (inline, `mailwomanHoldSeconds:0`) drives
//! `submit_email → rt.submitter.submit(Outgoing{..})` straight through the decorator's
//! decision path.
//!
//! A capture inner submitter observes what actually reaches the wire (per the t11-e3
//! brief: "a captured/mock inner submitter is fine to observe the rewrite — the point is
//! the decorator's real decision path"). The store, the engine's whole submission path,
//! and the decorator are the real ones; only the SMTP transport is captured.
//!
//! Scenarios (SPEC §28.4, 26.10 follow-up a):
//!   1. owned + ENABLED alias  → envelope From rewritten to the canonical alias
//!      (case-normalized) + `lastUsedAt` bumped + submission `final`.
//!   2. another account's alias → FAIL-CLOSED: submission `notCreated`, inner NEVER called.
//!   3. owned but DISABLED alias → FAIL-CLOSED (never sends).
//!   4. owned but DELETED alias  → FAIL-CLOSED (never sends).
//!   5. ordinary (non-alias) From → byte-unchanged, delivered.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::Engine;
use mw_engine::account::{AccountRuntime, MailSubmitter};
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MessageRef,
    MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result as BackendResult, SyncCursor,
    WatchHandle,
};
use mw_server::masked::MaskedSubmitter;
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{Credentials, MaskedEmailRow, NewAccount, ServerKey, Store};

const MASKED_DOMAIN: &str = "masked.vogue-homes.com";

// ── Capture inner submitter: the "wire". Records the last envelope + call count so a
//    fail-closed rejection can assert NOTHING reached the wire. ─────────────────────
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

// ── A minimal standards AccountBackend. The compose+submit path only ever touches
//    `append` (best-effort: `tolerant(..)` swallows `Unsupported`), so every method is a
//    benign no-op / Unsupported. This is NOT a bridge (register_backend, not
//    register_plugin_backend), so the account is treated as a standards IMAP account and
//    its submitter is honored on send. ─────────────────────────────────────────────
struct NoopBackend;

#[async_trait]
impl AccountBackend for NoopBackend {
    async fn capabilities(&self) -> BackendResult<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> BackendResult<Vec<RawMailbox>> {
        Ok(Vec::new())
    }
    async fn sync_mailbox(
        &self,
        _mbox: &RawMailboxRef,
        _cursor: &SyncCursor,
    ) -> BackendResult<MailboxDelta> {
        Err(EngineError::Unsupported("noop backend".into()))
    }
    async fn fetch_raw(&self, _refs: &[MessageRef]) -> BackendResult<Vec<RawMessage>> {
        Ok(Vec::new())
    }
    async fn store_flags(
        &self,
        _refs: &[MessageRef],
        _add: &[Flag],
        _remove: &[Flag],
    ) -> BackendResult<()> {
        Ok(())
    }
    async fn move_messages(
        &self,
        _refs: &[MessageRef],
        _to: &RawMailboxRef,
    ) -> BackendResult<MoveOutcome> {
        Err(EngineError::Unsupported("noop backend".into()))
    }
    async fn append(
        &self,
        _mbox: &RawMailboxRef,
        _raw: &[u8],
        _flags: &[Flag],
    ) -> BackendResult<MessageRef> {
        // `Unsupported` ⇒ `tolerant(..)` treats the upstream append as a no-op; the
        // engine still files the local Sent/Drafts copy.
        Err(EngineError::Unsupported("noop backend".into()))
    }
    async fn watch(&self, _sink: ChangeSink) -> BackendResult<WatchHandle> {
        Err(EngineError::Unsupported("noop backend".into()))
    }
}

/// Seed an enabled/disabled/deleted masked alias owned by `account_id`.
async fn seed_alias(store: &Store, id: &str, account_id: &str, alias: &str, state: &str) {
    store
        .put_masked_email(&MaskedEmailRow {
            id: id.into(),
            account_id: account_id.into(),
            alias_addr: alias.into(),
            target_desc: json!({ "target": "real@vogue-homes.com" }).to_string(),
            state: state.into(),
            created_at: "2026-07-14T00:00:00Z".into(),
            last_used_at: None,
        })
        .await
        .unwrap();
}

/// Build an engine + a standards account whose `submitter` is the REAL `MaskedSubmitter`
/// (wrapping `inner`) — the exact wrapping `engine_mode::register` performs. Returns
/// `(engine, account_id)`.
async fn engine_with_masked_account(inner: Arc<CaptureInner>) -> (Arc<Engine>, String) {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = store
        .create_account(
            &NewAccount {
                kind: mw_store::AccountKind::Imap,
                host: "mail.vogue-homes.com",
                port: 993,
                tls: "implicit",
                username: "alice@vogue-homes.com",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "alice@vogue-homes.com".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();

    let engine = Arc::new(Engine::new(store));
    // EXACT register() wrapping: MaskedSubmitter over the (capture) inner submitter.
    let masked = MaskedSubmitter::new(
        engine.store().clone(),
        account_id.clone(),
        inner as Arc<dyn MailSubmitter>,
    );
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            Arc::new(NoopBackend),
            Arc::new(masked) as Arc<dyn MailSubmitter>,
            "alice@vogue-homes.com",
        ),
    );
    (engine, account_id)
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

/// Compose a draft with the given envelope `from` and submit it inline (hold=0) in one
/// JMAP request — the real `Email/set` + `EmailSubmission/set` path.
fn compose_and_submit(from: &str) -> Value {
    json!([
        ["Email/set", { "create": { "draft": {
            "from": [{ "email": from }],
            "to": [{ "email": "dest@partner.example" }],
            "subject": "hello",
            "bodyValues": { "1": { "value": "hi there" } },
            "textBody": [{ "partId": "1", "type": "text/plain" }]
        } } }, "c1"],
        ["EmailSubmission/set", { "create": { "sub1": {
            "emailId": "#draft",
            "mailwomanHoldSeconds": 0
        } } }, "c2"]
    ])
}

// ── 1. owned + enabled alias → rewrite to the canonical alias + lastUsedAt bump ──────
#[tokio::test]
async fn send_from_owned_enabled_alias_rewrites_envelope_and_bumps_last_used() {
    let inner = Arc::new(CaptureInner::default());
    let (engine, account_id) = engine_with_masked_account(inner.clone()).await;
    let alias = format!("shield@{MASKED_DOMAIN}");
    seed_alias(engine.store(), "m-1", &account_id, &alias, "enabled").await;

    // Compose with the alias in MIXED CASE — the canonical stored form must win.
    let resp = jmap(
        &engine,
        &account_id,
        compose_and_submit(&alias.to_ascii_uppercase()),
    )
    .await;

    assert!(
        method_result(&resp, "c1")["created"]["draft"]["id"].is_string(),
        "draft created: {resp}"
    );
    let sub = method_result(&resp, "c2");
    assert!(
        sub["created"]["sub1"]["id"].is_string(),
        "submission accepted + sent: {resp}"
    );
    assert!(
        sub["notCreated"].get("sub1").is_none(),
        "not refused: {resp}"
    );

    // The wire saw the CANONICAL alias (case-normalized), not the uppercased input.
    assert_eq!(
        inner.calls.load(Ordering::SeqCst),
        1,
        "reached the wire once"
    );
    assert_eq!(
        inner.last.lock().unwrap().as_ref().unwrap().mail_from,
        alias,
        "envelope MAIL FROM rewritten to the canonical alias"
    );

    // lastUsedAt was bumped on the owning row.
    assert!(
        engine
            .store()
            .get_masked_email("m-1")
            .await
            .unwrap()
            .unwrap()
            .last_used_at
            .is_some(),
        "lastUsedAt bumped after send"
    );
}

// ── 2. another account's alias → FAIL-CLOSED (nothing sends) ─────────────────────────
#[tokio::test]
async fn send_from_another_accounts_alias_fails_closed() {
    let inner = Arc::new(CaptureInner::default());
    let (engine, account_id) = engine_with_masked_account(inner.clone()).await;
    // The alias is owned by a DIFFERENT account; our account must not send as it.
    let alias = format!("stranger@{MASKED_DOMAIN}");
    seed_alias(engine.store(), "m-b", "other-account", &alias, "enabled").await;

    let resp = jmap(&engine, &account_id, compose_and_submit(&alias)).await;

    let sub = method_result(&resp, "c2");
    assert!(
        sub["created"].get("sub1").is_none(),
        "a cross-account alias must NOT be reported as sent: {resp}"
    );
    assert!(
        sub["notCreated"]["sub1"].is_object(),
        "cross-account send surfaces a structured error (fail-closed): {resp}"
    );
    assert_eq!(
        inner.calls.load(Ordering::SeqCst),
        0,
        "cross-account alias must NEVER reach the wire"
    );
}

// ── 3 & 4. owned-but-disabled and owned-but-deleted → FAIL-CLOSED ────────────────────
#[tokio::test]
async fn send_from_disabled_or_deleted_owned_alias_fails_closed() {
    for (label, state) in [("disabled", "disabled"), ("deleted", "deleted")] {
        let inner = Arc::new(CaptureInner::default());
        let (engine, account_id) = engine_with_masked_account(inner.clone()).await;
        let alias = format!("{label}@{MASKED_DOMAIN}");
        seed_alias(engine.store(), "m-x", &account_id, &alias, state).await;

        let resp = jmap(&engine, &account_id, compose_and_submit(&alias)).await;

        let sub = method_result(&resp, "c2");
        assert!(
            sub["created"].get("sub1").is_none(),
            "{label} alias must NOT be reported as sent: {resp}"
        );
        assert!(
            sub["notCreated"]["sub1"].is_object(),
            "{label} alias send is fail-closed (structured error): {resp}"
        );
        assert_eq!(
            inner.calls.load(Ordering::SeqCst),
            0,
            "a {label} alias must NEVER reach the wire"
        );
    }
}

// ── 5. ordinary (non-alias) From → byte-unchanged, delivered ─────────────────────────
#[tokio::test]
async fn send_from_ordinary_address_is_byte_unchanged() {
    let inner = Arc::new(CaptureInner::default());
    let (engine, account_id) = engine_with_masked_account(inner.clone()).await;
    // An alias exists for the account, but this send is from the REAL address.
    seed_alias(
        engine.store(),
        "m-1",
        &account_id,
        &format!("unused@{MASKED_DOMAIN}"),
        "enabled",
    )
    .await;

    let from = "alice@vogue-homes.com";
    let resp = jmap(&engine, &account_id, compose_and_submit(from)).await;

    assert!(
        method_result(&resp, "c2")["created"]["sub1"]["id"].is_string(),
        "ordinary send accepted: {resp}"
    );
    assert_eq!(inner.calls.load(Ordering::SeqCst), 1, "delivered once");
    assert_eq!(
        inner.last.lock().unwrap().as_ref().unwrap().mail_from,
        from,
        "a non-alias From is passed through byte-unchanged"
    );
}
