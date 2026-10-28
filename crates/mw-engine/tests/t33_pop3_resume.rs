//! t33-e11: a POP3 sync that stops before its cursor is saved resumes where
//! ingest got to; it does not download the messages it already stored.
//!
//! The engine saves a mailbox's sync cursor once, after the whole sync. Each
//! POP3 message's UIDL is recorded as the message is stored. A server that is
//! restarted while the first sync after a login is still running (that sync runs
//! in the background since dcd88d33) left the cursor behind the store, and the
//! next sync fetched every stored message from the mail server again. The
//! `t28_search_restart` server test showed it as a doubled `RETR` count.
//!
//! The backend here is a UIDL diff, the same rule as `mw-pop3`'s backend, and
//! counts what it is asked to fetch. "The process stopped before the cursor was
//! saved" is reproduced by writing back the cursor the sync started from.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MailboxRole,
    MessageRef, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor,
    WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

/// A maildrop of `(uidl, raw)` messages behind a UIDL-diff sync.
struct Maildrop {
    messages: Mutex<Vec<(String, Vec<u8>)>>,
    fetched: AtomicUsize,
}

impl Maildrop {
    fn deliver(&self, uidl: &str) {
        let raw = format!(
            "Message-ID: <{uidl}@example.org>\r\n\
             From: sender@example.org\r\n\
             To: me@example.org\r\n\
             Subject: message {uidl}\r\n\
             Date: Wed, 01 Jul 2026 09:00:00 +0000\r\n\
             \r\n\
             body of {uidl}\r\n"
        );
        self.messages
            .lock()
            .unwrap()
            .push((uidl.to_string(), raw.into_bytes()));
    }

    fn remove(&self, uidl: &str) {
        self.messages.lock().unwrap().retain(|(u, _)| u != uidl);
    }

    fn fetched(&self) -> usize {
        self.fetched.load(Ordering::SeqCst)
    }

    fn inbox() -> RawMailboxRef {
        RawMailboxRef {
            name: "INBOX".into(),
            uidvalidity: 0,
        }
    }
}

#[async_trait]
impl AccountBackend for Maildrop {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps::default())
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        let total = self.messages.lock().unwrap().len() as u32;
        Ok(vec![RawMailbox {
            mailbox_ref: Self::inbox(),
            role: MailboxRole::Inbox,
            parent: None,
            uidnext: 0,
            highestmodseq: 0,
            total,
            unread: total,
        }])
    }

    async fn sync_mailbox(
        &self,
        _mbox: &RawMailboxRef,
        cursor: &SyncCursor,
    ) -> Result<MailboxDelta> {
        let seen = match cursor {
            SyncCursor::Pop3Uidl { seen } => seen.clone(),
            _ => BTreeSet::new(),
        };
        let present: BTreeSet<String> = self
            .messages
            .lock()
            .unwrap()
            .iter()
            .map(|(u, _)| u.clone())
            .collect();
        Ok(MailboxDelta {
            added: present
                .difference(&seen)
                .map(|uidl| MessageRef::Pop3 { uidl: uidl.clone() })
                .collect(),
            flag_changes: Vec::new(),
            removed: seen
                .difference(&present)
                .map(|uidl| MessageRef::Pop3 { uidl: uidl.clone() })
                .collect(),
            next_cursor: SyncCursor::Pop3Uidl { seen: present },
        })
    }

    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        let messages = self.messages.lock().unwrap();
        let mut out = Vec::new();
        for r in refs {
            let MessageRef::Pop3 { uidl } = r else {
                continue;
            };
            if let Some((_, raw)) = messages.iter().find(|(u, _)| u == uidl) {
                self.fetched.fetch_add(1, Ordering::SeqCst);
                out.push(RawMessage {
                    message_ref: r.clone(),
                    raw: raw.clone(),
                    flags: Vec::new(),
                    internaldate: None,
                });
            }
        }
        Ok(out)
    }

    async fn store_flags(
        &self,
        _refs: &[MessageRef],
        _add: &[Flag],
        _remove: &[Flag],
    ) -> Result<()> {
        Ok(())
    }

    async fn move_messages(
        &self,
        _refs: &[MessageRef],
        _to: &RawMailboxRef,
    ) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("POP3 has one mailbox".into()))
    }

    async fn append(
        &self,
        _mailbox: &RawMailboxRef,
        _raw: &[u8],
        _flags: &[Flag],
    ) -> Result<MessageRef> {
        Err(EngineError::Unsupported("POP3 cannot append".into()))
    }

    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported("POP3 has no push".into()))
    }
}

struct NoSubmit;

#[async_trait]
impl MailSubmitter for NoSubmit {
    async fn submit(&self, _outgoing: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> {
        Err(EngineError::Unsupported("unused".into()))
    }
}

async fn setup() -> (Arc<Engine>, String, Arc<Maildrop>) {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Pop3,
                host: "pop.example.org",
                port: 995,
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
    let drop = Arc::new(Maildrop {
        messages: Mutex::default(),
        fetched: AtomicUsize::new(0),
    });
    engine.register_backend(
        account.clone(),
        AccountRuntime::new(
            drop.clone() as Arc<dyn AccountBackend>,
            Arc::new(NoSubmit) as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    (engine, account, drop)
}

async fn inbox_id(engine: &Engine, account: &str) -> String {
    engine.store().list_mailboxes(account).await.unwrap()[0]
        .id
        .clone()
}

async fn stored(engine: &Engine, account: &str) -> usize {
    let inbox = inbox_id(engine, account).await;
    engine
        .store()
        .list_message_ids(&inbox, 1000, 0)
        .await
        .unwrap()
        .len()
}

#[tokio::test]
async fn a_first_sync_that_stopped_before_its_cursor_does_not_fetch_again() {
    let (engine, account, drop) = setup().await;
    for uidl in ["m1", "m2", "m3"] {
        drop.deliver(uidl);
    }
    engine.resync(&account).await.unwrap();
    assert_eq!(drop.fetched(), 3, "precondition: three messages fetched");
    assert_eq!(stored(&engine, &account).await, 3);

    // The process stopped after ingest and before the cursor was written: the
    // mailbox has no usable cursor, as before the first sync.
    let inbox = inbox_id(&engine, &account).await;
    engine
        .store()
        .save_cursor(&account, &inbox, "null")
        .await
        .unwrap();

    engine.resync(&account).await.unwrap();
    assert_eq!(
        drop.fetched(),
        3,
        "the stored messages were not fetched again"
    );
    assert_eq!(stored(&engine, &account).await, 3);
}

#[tokio::test]
async fn a_later_sync_that_stopped_before_its_cursor_fetches_only_what_is_new() {
    let (engine, account, drop) = setup().await;
    drop.deliver("m1");
    engine.resync(&account).await.unwrap();
    let inbox = inbox_id(&engine, &account).await;
    let before = engine
        .store()
        .load_cursor(&account, &inbox)
        .await
        .unwrap()
        .unwrap();

    drop.deliver("m2");
    drop.deliver("m3");
    engine.resync(&account).await.unwrap();
    assert_eq!(drop.fetched(), 3);
    // Stopped before the cursor of that second sync was written.
    engine
        .store()
        .save_cursor(&account, &inbox, &before)
        .await
        .unwrap();

    drop.deliver("m4");
    engine.resync(&account).await.unwrap();
    assert_eq!(drop.fetched(), 4, "only the new message was fetched");
    assert_eq!(stored(&engine, &account).await, 4);
}

#[tokio::test]
async fn a_message_gone_from_the_maildrop_is_removed_once() {
    let (engine, account, drop) = setup().await;
    drop.deliver("m1");
    drop.deliver("m2");
    engine.resync(&account).await.unwrap();

    drop.remove("m1");
    engine.resync(&account).await.unwrap();
    assert_eq!(stored(&engine, &account).await, 1);
    let seen = engine.store().seen_uidls(&account).await.unwrap();
    assert_eq!(
        seen.into_iter().collect::<Vec<_>>(),
        vec!["m2"],
        "the removed UIDL is no longer recorded as ingested"
    );

    // A further sync has nothing to remove and records no change.
    let state = engine
        .store()
        .current_state(&account, "Email")
        .await
        .unwrap();
    engine.resync(&account).await.unwrap();
    assert_eq!(
        engine
            .store()
            .current_state(&account, "Email")
            .await
            .unwrap(),
        state,
        "the removal was not reported again"
    );
    assert_eq!(drop.fetched(), 2);
}
