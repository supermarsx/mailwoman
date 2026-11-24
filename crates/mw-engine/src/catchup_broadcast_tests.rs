//! t55: the periodic catch-up pass publishes a `StateChange` only when it
//! changed stored mail state. Every published change reaches state-change
//! webhooks and push wakes (mw-server), so a no-change pass must publish nothing.
use super::*;
use crate::account::MailSubmitter;
use crate::backend::*;
use async_trait::async_trait;
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};
use std::sync::atomic::{AtomicUsize, Ordering};

const INTERVAL: Duration = Duration::from_millis(40);

fn inbox_ref() -> RawMailboxRef {
    RawMailboxRef {
        name: "INBOX".into(),
        uidvalidity: 1,
    }
}

/// An upstream with one INBOX. Each `sync_mailbox` call is counted (one per
/// catch-up pass); refs queued in `pending` appear as new mail on the next pass.
#[derive(Default)]
struct Upstream {
    syncs: AtomicUsize,
    pending: std::sync::Mutex<Vec<MessageRef>>,
}

#[async_trait]
impl AccountBackend for Upstream {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(vec![RawMailbox {
            mailbox_ref: inbox_ref(),
            role: MailboxRole::Inbox,
            parent: None,
            uidnext: 10,
            highestmodseq: 0,
            total: 0,
            unread: 0,
        }])
    }
    async fn sync_mailbox(&self, _: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
        self.syncs.fetch_add(1, Ordering::SeqCst);
        Ok(MailboxDelta {
            added: std::mem::take(&mut *self.pending.lock().unwrap()),
            flag_changes: vec![],
            removed: vec![],
            next_cursor: c.clone(),
        })
    }
    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        Ok(refs
            .iter()
            .map(|r| RawMessage {
                message_ref: r.clone(),
                raw: b"From: sender@example.invalid\r\nTo: owner@fixture.invalid\r\nMessage-ID: <catchup@fixture.invalid>\r\nSubject: New\r\nContent-Type: text/plain\r\n\r\nHello\r\n".to_vec(),
                flags: vec![],
                internaldate: Some("2026-11-24T09:00:00Z".into()),
            })
            .collect())
    }
    async fn store_flags(&self, _: &[MessageRef], _: &[Flag], _: &[Flag]) -> Result<()> {
        Ok(())
    }
    async fn move_messages(&self, _: &[MessageRef], _: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("fixture".into()))
    }
    async fn append(&self, _: &RawMailboxRef, _: &[u8], _: &[Flag]) -> Result<MessageRef> {
        Err(EngineError::Unsupported("fixture".into()))
    }
    async fn watch(&self, _: ChangeSink) -> Result<WatchHandle> {
        // An IDLE that never reports anything: only the catch-up pass runs.
        Ok(WatchHandle::new(watch::channel(false).0))
    }
}

struct NoSend;
#[async_trait]
impl MailSubmitter for NoSend {
    async fn submit(&self, _: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> {
        panic!("not sending")
    }
}

async fn fixture() -> (Arc<Engine>, Arc<Upstream>, String) {
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
    let engine = Arc::new(Engine::new(store));
    engine.set_catchup_interval(INTERVAL);
    let upstream = Arc::new(Upstream::default());
    engine.register_backend(
        account.clone(),
        AccountRuntime::new(upstream.clone(), Arc::new(NoSend), "owner@fixture.invalid"),
    );
    // The first sync caches the mailbox; from here on upstream is quiet.
    engine.resync(&account).await.unwrap();
    (engine, upstream, account)
}

/// Wait until at least `passes` more catch-up passes have completed.
async fn wait_passes(upstream: &Upstream, passes: usize) {
    let target = upstream.syncs.load(Ordering::SeqCst) + passes;
    tokio::time::timeout(Duration::from_secs(20), async {
        while upstream.syncs.load(Ordering::SeqCst) < target {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("catch-up passes did not run");
    // Let the last pass finish its post-sync publication step.
    tokio::time::sleep(INTERVAL / 2).await;
}

fn drain(rx: &mut broadcast::Receiver<StateChange>) -> Vec<StateChange> {
    let mut out = vec![];
    while let Ok(change) = rx.try_recv() {
        out.push(change);
    }
    out
}

#[tokio::test]
async fn a_catch_up_pass_that_changes_nothing_publishes_nothing() {
    let (engine, upstream, account) = fixture().await;
    let mut rx = engine.subscribe();
    engine.start_watch(&account).await.unwrap();
    wait_passes(&upstream, 3).await;
    let published = drain(&mut rx);
    assert!(
        published.is_empty(),
        "{} StateChange(s) published by {} no-change catch-up passes",
        published.len(),
        upstream.syncs.load(Ordering::SeqCst) - 1
    );
    engine.unregister(&account);
}

#[tokio::test]
async fn a_catch_up_pass_that_stores_new_mail_publishes_the_new_state() {
    let (engine, upstream, account) = fixture().await;
    let before = engine.store().current_core_mail_states(&account).await.unwrap();
    let mut rx = engine.subscribe();
    engine.start_watch(&account).await.unwrap();
    wait_passes(&upstream, 1).await;
    assert!(drain(&mut rx).is_empty(), "quiet pass published");

    upstream.pending.lock().unwrap().push(MessageRef::Imap {
        mailbox: inbox_ref(),
        uidvalidity: 1,
        uid: 1,
    });
    let published = tokio::time::timeout(Duration::from_secs(20), rx.recv())
        .await
        .expect("new mail was not published")
        .unwrap();
    let after = engine.store().current_core_mail_states(&account).await.unwrap();
    assert_ne!(before, after, "the pass stored the new message");
    assert_eq!(published.account_id, account);
    assert_eq!(published.email, after.get("Email").copied().unwrap_or(0).to_string());

    // Let that pass finish, then quiet again: nothing further is published.
    wait_passes(&upstream, 1).await;
    for repeat in drain(&mut rx) {
        assert_eq!(repeat, published, "only the new state is published");
    }
    wait_passes(&upstream, 2).await;
    assert!(drain(&mut rx).is_empty(), "quiet passes after new mail published");
    engine.unregister(&account);
}

