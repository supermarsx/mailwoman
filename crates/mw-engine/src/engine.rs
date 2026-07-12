//! The orchestrator: a registry of connected accounts plus the sync engine that
//! keeps the local `mw-store` cache fresh (plan §0, §1.6–§1.8).
//!
//! The engine never talks a wire protocol itself — it drives whatever
//! [`AccountBackend`] it was handed. Backends speak in raw server coordinates
//! ([`MessageRef`]); the engine maps those to/from the store's opaque stable ids
//! and never leaks a UID up to the JMAP surface.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use mw_store::{MailboxUpsert, MessageUpsert, Store};
use tokio::sync::{broadcast, mpsc};

use crate::account::AccountRuntime;
use crate::backend::{
    ChangeEvent, ChangeSink, EngineError, MailboxDelta, MessageRef, RawMailbox, RawMessage, Result,
};
use crate::change::StateChange;
use crate::mapping::{
    cursor_from_json, cursor_to_json, flags_to_json, initial_cursor, role_to_store,
};
use crate::thread::thread_root;

/// The `sessionState` string the JMAP surface advertises. V1 does not implement
/// state-based change tracking (the browser re-polls), so a constant is correct.
pub(crate) const SESSION_STATE: &str = "engine-0";

/// The engine: one local store plus a registry of live account backends.
///
/// Cloneable-by-`Arc` at the call sites that need `'static` tasks (change
/// ingestion); the struct itself is shared behind an `Arc<Engine>` by the server.
pub struct Engine {
    store: Store,
    accounts: Mutex<HashMap<String, AccountRuntime>>,
    /// Realtime change fan-out (plan §1.2, §2.2). `start_watch` will
    /// `broadcast.send(StateChange{…})` after each resync (e9); `mw-server`
    /// (e10) subscribes per session to feed `/jmap/ws` + `/jmap/eventsource`.
    changes: broadcast::Sender<StateChange>,
}

impl Engine {
    /// Build an engine over an open store with no accounts yet.
    pub fn new(store: Store) -> Self {
        let (changes, _rx) = broadcast::channel(256);
        Self {
            store,
            accounts: Mutex::new(HashMap::new()),
            changes,
        }
    }

    /// The underlying store (so the server can persist accounts/sessions).
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Subscribe to the realtime `StateChange` stream (plan §2.2). Each
    /// authenticated WS/SSE session in `mw-server` (e10) holds one receiver.
    pub fn subscribe(&self) -> broadcast::Receiver<StateChange> {
        self.changes.subscribe()
    }

    /// Register (or replace) a connected account's runtime.
    pub fn register_backend(&self, account_id: impl Into<String>, runtime: AccountRuntime) {
        self.accounts
            .lock()
            .expect("accounts lock")
            .insert(account_id.into(), runtime);
    }

    /// Whether an account is currently connected in this engine.
    pub fn is_registered(&self, account_id: &str) -> bool {
        self.accounts
            .lock()
            .expect("accounts lock")
            .contains_key(account_id)
    }

    /// Drop an account's runtime (stopping any watch loop on the last `Arc`).
    pub fn unregister(&self, account_id: &str) -> Option<AccountRuntime> {
        self.accounts
            .lock()
            .expect("accounts lock")
            .remove(account_id)
    }

    /// Snapshot an account's runtime out of the registry (cheap `Arc` clone) so
    /// callers hold no lock across an `await`.
    pub(crate) fn runtime(&self, account_id: &str) -> Option<AccountRuntime> {
        self.accounts
            .lock()
            .expect("accounts lock")
            .get(account_id)
            .cloned()
    }

    fn require_runtime(&self, account_id: &str) -> Result<AccountRuntime> {
        self.runtime(account_id).ok_or_else(|| {
            EngineError::Unsupported(format!("account {account_id} is not connected"))
        })
    }

    // ---- sync engine ----------------------------------------------------

    /// Full sync: enumerate mailboxes, upsert them, then incrementally sync each
    /// from its persisted cursor. Idempotent — safe to call on every change tick.
    pub async fn resync(&self, account_id: &str) -> Result<()> {
        let rt = self.require_runtime(account_id)?;
        let mailboxes = rt.backend.list_mailboxes().await?;
        for rm in &mailboxes {
            let mailbox_id = self.upsert_mailbox(account_id, rm).await?;
            self.sync_one(account_id, &rt, rm, &mailbox_id).await?;
        }
        Ok(())
    }

    /// Upsert a backend-enumerated mailbox into the store, returning its id.
    async fn upsert_mailbox(&self, account_id: &str, rm: &RawMailbox) -> Result<String> {
        let role = role_to_store(rm.role);
        Ok(self
            .store
            .upsert_mailbox(&MailboxUpsert {
                account_id,
                name: &rm.mailbox_ref.name,
                role,
                uidvalidity: rm.mailbox_ref.uidvalidity,
                uidnext: rm.uidnext,
                highestmodseq: rm.highestmodseq,
                total: rm.total,
                unread: rm.unread,
                parent_id: None,
            })
            .await?)
    }

    /// Incrementally sync one mailbox: load its cursor, apply the [`MailboxDelta`]
    /// (fetch+parse+store new messages, apply flag changes, drop removals), then
    /// persist the next cursor.
    async fn sync_one(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        rm: &RawMailbox,
        mailbox_id: &str,
    ) -> Result<()> {
        let cursor = match self.store.load_cursor(account_id, mailbox_id).await? {
            Some(json) => cursor_from_json(&json).unwrap_or_else(initial_cursor),
            None => initial_cursor(),
        };
        let delta: MailboxDelta = rt.backend.sync_mailbox(&rm.mailbox_ref, &cursor).await?;

        // New messages: fetch raw bytes, parse in one pass, ingest.
        if !delta.added.is_empty() {
            let raws = rt.backend.fetch_raw(&delta.added).await?;
            for raw in &raws {
                self.ingest(account_id, mailbox_id, raw).await?;
            }
        }

        // Flag changes (server-authoritative). A UID we have never seen arriving
        // as a "change" (QRESYNC surfaces the modified set) is actually new: fetch
        // and ingest it so nothing is lost.
        for (mref, flags) in &delta.flag_changes {
            match self.resolve_ref(account_id, mailbox_id, mref).await? {
                Some(sid) => self.store.set_flags(&sid, &flags_to_json(flags)).await?,
                None => {
                    let raws = rt.backend.fetch_raw(std::slice::from_ref(mref)).await?;
                    for raw in &raws {
                        self.ingest(account_id, mailbox_id, raw).await?;
                    }
                }
            }
        }

        // Removals (EXPUNGE / VANISHED / dropped UIDL).
        for mref in &delta.removed {
            if let Some(sid) = self.resolve_ref(account_id, mailbox_id, mref).await? {
                self.store.delete_message(&sid).await?;
            }
        }

        self.store
            .save_cursor(account_id, mailbox_id, &cursor_to_json(&delta.next_cursor))
            .await?;
        Ok(())
    }

    /// Fetch → parse (MIME) → thread → seal → upsert one message, returning its
    /// stable id. Cross-UIDVALIDITY re-keying is handled inside the store's
    /// `upsert_message` identity match; POP3 messages also record their UIDL.
    pub(crate) async fn ingest(
        &self,
        account_id: &str,
        mailbox_id: &str,
        raw: &RawMessage,
    ) -> Result<String> {
        let parsed = match mw_mime::parse(&raw.raw) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("skipping unparseable message: {e}");
                return Err(EngineError::Protocol(format!("mime parse: {e}")));
            }
        };
        let mut email = parsed.email;
        // INTERNALDATE is the JMAP receivedAt; fall back to the Date header.
        let received_at = raw
            .internaldate
            .clone()
            .or_else(|| email.received_at.clone())
            .or_else(|| email.sent_at.clone());
        email.received_at = received_at.clone();

        // Engine-side JWZ thread assignment.
        let thread_id = match thread_root(&parsed.envelope) {
            Some(root) => Some(self.store.assign_thread(account_id, &root).await?),
            None => None,
        };
        email.thread_id = thread_id.clone();

        let env_bytes = serde_json::to_vec(&email).unwrap_or_default();
        let blob_ref = self.store.put_body(account_id, &raw.raw).await?;
        let flags_json = flags_to_json(&raw.flags);
        let (uidvalidity, uid, uidl) = coords(&raw.message_ref);

        let sid = self
            .store
            .upsert_message(&MessageUpsert {
                account_id,
                mailbox_id,
                uid,
                uidvalidity,
                message_id: parsed.envelope.message_id.as_deref(),
                thread_id: thread_id.as_deref(),
                internaldate: received_at.as_deref(),
                size: raw.raw.len() as u64,
                flags_json: &flags_json,
                envelope: Some(&env_bytes),
                blob_ref: Some(&blob_ref),
            })
            .await?;

        if let Some(uidl) = uidl {
            self.store.record_uidl(account_id, &uidl, &sid).await?;
        }
        Ok(sid)
    }

    /// Resolve a backend message ref to its stable id, if the store knows it.
    pub(crate) async fn resolve_ref(
        &self,
        account_id: &str,
        mailbox_id: &str,
        mref: &MessageRef,
    ) -> Result<Option<String>> {
        Ok(match mref {
            MessageRef::Imap {
                uidvalidity, uid, ..
            } => {
                self.store
                    .stable_id_for(account_id, mailbox_id, *uidvalidity, *uid)
                    .await?
            }
            MessageRef::Pop3 { uidl } => self.store.stable_id_for_uidl(account_id, uidl).await?,
        })
    }

    // ---- change ingestion ----------------------------------------------

    /// Start the backend's watch loop; on each `MailboxChanged` re-sync so the
    /// next browser poll sees fresh mail. V1 does not push to the browser.
    pub async fn start_watch(self: &Arc<Self>, account_id: &str) -> Result<()> {
        let rt = self.require_runtime(account_id)?;
        let (tx, mut rx) = mpsc::unbounded_channel();
        let handle = rt.backend.watch(ChangeSink::new(tx)).await?;

        let engine = Arc::clone(self);
        let acct = account_id.to_string();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    ChangeEvent::MailboxChanged { .. } => {
                        if let Err(e) = engine.resync(&acct).await {
                            tracing::warn!("resync after change failed: {e}");
                        }
                    }
                    ChangeEvent::Disconnected => break,
                }
            }
        });

        if let Some(rt) = self
            .accounts
            .lock()
            .expect("accounts lock")
            .get_mut(account_id)
        {
            rt.watch = Some(Arc::new(handle));
        }
        Ok(())
    }
}

/// Decompose a message ref into `(uidvalidity, uid, uidl)`. POP3 has no UID, so a
/// deterministic pseudo-UID keeps the store's `(mailbox, uidvalidity, uid)`
/// uniqueness index happy while the UIDL table remains the identity of record.
fn coords(mref: &MessageRef) -> (u32, u32, Option<String>) {
    match mref {
        MessageRef::Imap {
            uidvalidity, uid, ..
        } => (*uidvalidity, *uid, None),
        MessageRef::Pop3 { uidl } => (0, pop3_pseudo_uid(uidl), Some(uidl.clone())),
    }
}

/// A stable, non-zero pseudo-UID derived from a POP3 UIDL.
fn pop3_pseudo_uid(uidl: &str) -> u32 {
    let mut h = DefaultHasher::new();
    uidl.hash(&mut h);
    (h.finish() as u32) | 1
}
