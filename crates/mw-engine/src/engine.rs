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
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use mw_store::{MailboxUpsert, MessageUpsert, Store};
use tokio::sync::{broadcast, mpsc};

use crate::account::AccountRuntime;
use crate::backend::{
    ChangeEvent, ChangeSink, EngineError, MailboxDelta, MessageRef, RawMailbox, RawMessage, Result,
};
use crate::change::{ChangeOp, ChangeType, StateChange};
use crate::mapping::{
    cursor_from_json, cursor_to_json, flags_to_json, flags_to_keywords, initial_cursor,
    role_to_store,
};
use crate::search_index;
use crate::thread::thread_root;

/// The engine: one local store plus a registry of live account backends.
///
/// Cloneable-by-`Arc` at the call sites that need `'static` tasks (change
/// ingestion); the struct itself is shared behind an `Arc<Engine>` by the server.
pub struct Engine {
    store: Store,
    accounts: Mutex<HashMap<String, AccountRuntime>>,
    /// Realtime change fan-out (plan §1.2, §2.2). `start_watch` broadcasts a
    /// `StateChange` after each resync (e9); `mw-server` (e10) subscribes per
    /// session to feed `/jmap/ws` + `/jmap/eventsource`.
    changes: broadcast::Sender<StateChange>,
    /// The engine-side full-text index (plan §1.1). Written at [`Engine::ingest`],
    /// re-keyed at move, deleted at expunge. Shared behind an `Arc` so the
    /// dispatcher/query paths can hold it cheaply.
    search: Arc<mw_search::Index>,
    /// Guards the single delayed dispatcher task (undo-send + snooze, §1.3/§1.5).
    dispatcher_started: AtomicBool,
}

impl Engine {
    /// Build an engine over an open store with an in-RAM search index. The
    /// index rebuilds from the store on restart; production uses
    /// [`Engine::open_with_search`] to persist it under the data dir.
    pub fn new(store: Store) -> Self {
        let search = Arc::new(
            mw_search::Index::open_in_ram().expect("in-RAM search index construction cannot fail"),
        );
        Self::with_search(store, search)
    }

    /// Build an engine over a persistent index rooted at `dir` (server path).
    pub fn open_with_search(store: Store, dir: &Path) -> Result<Self> {
        let search = mw_search::Index::open(dir)
            .map_err(|e| EngineError::Protocol(format!("open search index: {e}")))?;
        Ok(Self::with_search(store, Arc::new(search)))
    }

    /// Build an engine over an explicit search index (tests / custom wiring).
    pub fn with_search(store: Store, search: Arc<mw_search::Index>) -> Self {
        let (changes, _rx) = broadcast::channel(256);
        Self {
            store,
            accounts: Mutex::new(HashMap::new()),
            changes,
            search,
            dispatcher_started: AtomicBool::new(false),
        }
    }

    /// The underlying store (so the server can persist accounts/sessions).
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// The engine-side full-text index (plan §1.1).
    pub fn search(&self) -> &mw_search::Index {
        &self.search
    }

    /// The broadcast sender (used by [`crate::state`] to fan out `StateChange`).
    pub(crate) fn changes_tx(&self) -> &broadcast::Sender<StateChange> {
        &self.changes
    }

    /// The dispatcher-started guard (used by [`crate::dispatcher`]).
    pub(crate) fn dispatcher_started(&self) -> &AtomicBool {
        &self.dispatcher_started
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
        // Upsert every mailbox first so ingest-time rules (plan §0.6) can target
        // any folder even when it sorts after the inbox in the backend's list.
        let mut pairs = Vec::with_capacity(mailboxes.len());
        for rm in &mailboxes {
            let mailbox_id = self.upsert_mailbox(account_id, rm).await?;
            pairs.push((rm, mailbox_id));
        }
        for (rm, mailbox_id) in &pairs {
            self.sync_one(account_id, &rt, rm, mailbox_id).await?;
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
                let _ = self.search.delete(&sid);
                self.record_change(account_id, ChangeType::Email, &sid, ChangeOp::Destroyed)
                    .await?;
                self.record_change(
                    account_id,
                    ChangeType::Mailbox,
                    mailbox_id,
                    ChangeOp::Updated,
                )
                .await?;
            }
        }

        self.store
            .save_cursor(account_id, mailbox_id, &cursor_to_json(&delta.next_cursor))
            .await?;
        Ok(())
    }

    /// Fetch → parse (MIME) → thread → seal → upsert one message, returning its
    /// stable id. Also indexes the message for search (plan §1.1), runs inbox
    /// rules on genuinely new arrivals (plan §0.6), and records the `Email`
    /// change that advances state (plan §1.2). Cross-UIDVALIDITY re-keying is
    /// handled inside the store's `upsert_message` identity match; POP3 messages
    /// also record their UIDL.
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

        let (uidvalidity, uid, uidl) = coords(&raw.message_ref);
        // Was this coordinate already cached? Distinguishes a genuinely new
        // arrival (→ run rules, record `created`) from a re-ingest (→ `updated`).
        let existed = match &uidl {
            Some(u) => self
                .store
                .stable_id_for_uidl(account_id, u)
                .await?
                .is_some(),
            None => self
                .store
                .stable_id_for(account_id, mailbox_id, uidvalidity, uid)
                .await?
                .is_some(),
        };

        let env_bytes = serde_json::to_vec(&email).unwrap_or_default();
        let blob_ref = self.store.put_body(account_id, &raw.raw).await?;
        let flags_json = flags_to_json(&raw.flags);

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

        if let Some(uidl) = &uidl {
            self.store.record_uidl(account_id, uidl, &sid).await?;
        }

        // Index for search. Keywords + attachment filenames back `tag:`/`is:` and
        // `filename:`; pin defaults false on a fresh document.
        let keywords: Vec<String> = flags_to_keywords(&raw.flags).into_keys().collect();
        let filenames = search_index::attachment_filenames(&raw.raw);
        let doc = search_index::build_index_doc(
            &sid, account_id, mailbox_id, &email, keywords, filenames, false,
        );
        if let Err(e) = self.search.upsert(&doc) {
            tracing::warn!("search index upsert failed for {sid}: {e}");
        }

        // Rules run once, on genuinely new inbox arrivals (never on our own
        // drafts/sent, never on historical re-sync).
        if !existed {
            self.apply_rules_at_ingest(account_id, mailbox_id, &sid, &email, &raw.flags)
                .await?;
        }

        // Record the change that advances Email state (+ the mailbox counter).
        let op = if existed {
            ChangeOp::Updated
        } else {
            ChangeOp::Created
        };
        self.record_change(account_id, ChangeType::Email, &sid, op)
            .await?;
        self.record_change(
            account_id,
            ChangeType::Mailbox,
            mailbox_id,
            ChangeOp::Updated,
        )
        .await?;
        Ok(sid)
    }

    /// Rebuild a message's search-index document from the store (after a flag,
    /// meta, or move change). Best-effort — a failure only degrades search, not
    /// correctness. Loads the sealed body to recover attachment filenames.
    pub(crate) async fn reindex_message(&self, stable_id: &str) {
        if let Err(e) = self.try_reindex_message(stable_id).await {
            tracing::warn!("re-index of {stable_id} failed: {e}");
        }
    }

    async fn try_reindex_message(&self, stable_id: &str) -> Result<()> {
        let msg = self
            .store
            .get_message(stable_id)
            .await
            .map_err(EngineError::Store)?;
        let email: mw_jmap::Email = match self.store.get_envelope(stable_id).await? {
            Some(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            None => match &msg.blob_ref {
                Some(blob) => match self.store.get_body(blob).await? {
                    Some(raw) => mw_mime::parse(&raw).map(|p| p.email).unwrap_or_default(),
                    None => mw_jmap::Email::default(),
                },
                None => mw_jmap::Email::default(),
            },
        };
        let filenames = match &msg.blob_ref {
            Some(blob) => match self.store.get_body(blob).await? {
                Some(raw) => search_index::attachment_filenames(&raw),
                None => Vec::new(),
            },
            None => Vec::new(),
        };
        let keywords: Vec<String> =
            flags_to_keywords(&crate::mapping::flags_from_json(&msg.flags_json))
                .into_keys()
                .collect();
        let pinned = self
            .store
            .get_message_meta(stable_id)
            .await?
            .map(|m| m.pinned)
            .unwrap_or(false);
        let doc = search_index::build_index_doc(
            stable_id,
            &msg.account_id,
            &msg.mailbox_id,
            &email,
            keywords,
            filenames,
            pinned,
        );
        self.search
            .upsert(&doc)
            .map_err(|e| EngineError::Protocol(format!("index upsert: {e}")))?;
        Ok(())
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

    // ---- blob download (plan §2.4 / e14) -------------------------------

    /// Resolve a download `blobId` to its bytes plus the HTTP framing the server
    /// needs. The blobId scheme is engine-owned:
    /// - `<stableId>` — the whole message, returned verbatim as
    ///   `message/rfc822` (a re-parse of the output equals the original, so EML
    ///   export round-trips).
    /// - `<stableId>.<partId>` — one decoded MIME part, with its own
    ///   content-type + filename (the attachment viewers' download path).
    ///
    /// Bytes come from the sealed body cache when present, else a backend
    /// `fetch_raw`. Returns `Ok(None)` when the id names no message/part owned by
    /// `account_id` (→ 404); cross-account ids never resolve.
    pub async fn fetch_blob(&self, account_id: &str, blob_id: &str) -> Result<Option<BlobData>> {
        let (stable_id, part_id) = match blob_id.split_once('.') {
            Some((sid, pid)) => (sid, Some(pid)),
            None => (blob_id, None),
        };
        let Some(raw) = self.message_raw(account_id, stable_id).await? else {
            return Ok(None);
        };
        match part_id {
            None => Ok(Some(BlobData {
                content_type: "message/rfc822".to_string(),
                filename: format!("{stable_id}.eml"),
                bytes: raw,
            })),
            Some(pid) => {
                let Ok(idx) = pid.parse::<u32>() else {
                    return Ok(None);
                };
                let Some(part) = mw_mime::part_blob(&raw, idx) else {
                    return Ok(None);
                };
                let filename = part
                    .filename
                    .unwrap_or_else(|| format!("{stable_id}.{pid}.bin"));
                Ok(Some(BlobData {
                    content_type: part.content_type,
                    filename,
                    bytes: part.bytes,
                }))
            }
        }
    }

    /// Raw RFC822 bytes for a stable id, scoped to `account_id`: served from the
    /// sealed body cache, else pulled from the backend by the message's server
    /// coordinates. `Ok(None)` when the id is unknown or owned by another account.
    async fn message_raw(&self, account_id: &str, stable_id: &str) -> Result<Option<Vec<u8>>> {
        let msg = match self.store.get_message(stable_id).await {
            Ok(m) => m,
            Err(mw_store::StoreError::NotFound) => return Ok(None),
            Err(e) => return Err(EngineError::Store(e)),
        };
        if msg.account_id != account_id {
            return Ok(None);
        }
        if let Some(blob) = &msg.blob_ref
            && let Some(raw) = self.store.get_body(blob).await?
        {
            return Ok(Some(raw));
        }
        // Body not cached (e.g. evicted): re-fetch from the backend if the
        // message still has upstream coordinates.
        if let Some(mref) = self.imap_ref_for(stable_id).await? {
            let rt = self.require_runtime(account_id)?;
            let raws = rt.backend.fetch_raw(std::slice::from_ref(&mref)).await?;
            if let Some(first) = raws.into_iter().next() {
                return Ok(Some(first.raw));
            }
        }
        Ok(None)
    }

    // ---- change ingestion ----------------------------------------------

    /// Start the backend's watch loop; on each `MailboxChanged` re-sync then
    /// broadcast a `StateChange` so subscribed WS/SSE sessions push it to the
    /// browser (plan §1.2). Also ensures the delayed dispatcher is running.
    pub async fn start_watch(self: &Arc<Self>, account_id: &str) -> Result<()> {
        self.start_dispatcher();
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
                        } else {
                            engine.broadcast_state(&acct).await;
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

/// A resolved download blob (plan §2.4 / e14): the bytes plus the content-type
/// and filename the server puts on the `Content-Type` / `Content-Disposition`
/// headers. Produced by [`Engine::fetch_blob`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobData {
    /// MIME type: `message/rfc822` for a whole message, the part's type for a part.
    pub content_type: String,
    /// Suggested download filename.
    pub filename: String,
    /// The blob bytes.
    pub bytes: Vec<u8>,
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
