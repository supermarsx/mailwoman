//! The **frozen** account-backend seam (plan §2.1, SPEC §6.5).
//!
//! Authored by the scaffolder gate (e0) so every parallel crate — `mw-imap`,
//! `mw-pop3`, `mw-smtp`, `mw-store`, `mw-autoconfig` — compiles against these
//! types from minute one. **Changing anything in this file requires the
//! coordinator to re-broadcast**: the trait and its supporting types are the
//! interchangeability contract between backends and `mw-engine`.
//!
//! Design invariants (plan §1.5/§1.6):
//! - Backends speak only in raw server coordinates ([`MessageRef`],
//!   [`RawMailboxRef`]). They never see the engine's opaque stable ids.
//! - The engine never leaks raw UIDs upward to the JMAP surface.
//! - The backend picks the strongest [`SyncCursor`] its [`BackendCaps`]
//!   support; the engine persists whatever the backend returns.

use std::collections::BTreeSet;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, watch};

/// Crate-wide result alias for backend + engine operations.
pub type Result<T> = std::result::Result<T, EngineError>;

/// Errors surfaced across the backend seam.
///
/// Kept deliberately coarse at the frozen boundary — concrete backends map
/// their protocol-specific failures onto these variants so `mw-engine` can
/// apply uniform retry/degrade policy (plan §6.1).
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// Malformed or unexpected protocol response from the server.
    #[error("backend protocol error: {0}")]
    Protocol(String),
    /// Authentication (LOGIN / SASL) was rejected.
    #[error("authentication failed: {0}")]
    Auth(String),
    /// Transport-level failure (connect/TLS/socket).
    #[error("transport error: {0}")]
    Transport(String),
    /// The requested capability is not advertised by this server.
    #[error("capability not supported: {0}")]
    Unsupported(String),
    /// A referenced mailbox does not exist upstream.
    #[error("mailbox not found: {0}")]
    MailboxNotFound(String),
    /// Persistence failure while reading/writing the local cache.
    #[error("store error: {0}")]
    Store(#[from] mw_store::StoreError),
}

/// The interchangeability seam every account backend implements (plan §2.1).
///
/// `mw-imap` and `mw-pop3` implement this today; future JMAP-passthrough and
/// protocol bridges (V7) implement the same trait. `mw-engine` consumes only
/// the trait, so backends are developed fully in parallel against it.
#[async_trait]
pub trait AccountBackend: Send + Sync {
    /// Probe capabilities & special-use folders; called once per connect.
    async fn capabilities(&self) -> Result<BackendCaps>;

    /// Enumerate mailboxes/folders with role (SPECIAL-USE) + status counts.
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>>;

    /// Incremental sync of one mailbox from a persisted cursor; returns changes.
    async fn sync_mailbox(&self, mbox: &RawMailboxRef, cursor: &SyncCursor)
    -> Result<MailboxDelta>;

    /// Fetch full raw RFC822 bytes for a set of message refs
    /// (handed to `mw-mime` for parsing inside the render jail).
    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>>;

    /// Flag/keyword changes (server-authoritative per SPEC §15.2).
    async fn store_flags(&self, refs: &[MessageRef], add: &[Flag], remove: &[Flag]) -> Result<()>;

    /// Move messages between mailboxes
    /// (idempotent by stable id; MOVE, or COPY+EXPUNGE fallback).
    async fn move_messages(&self, refs: &[MessageRef], to: &RawMailboxRef) -> Result<MoveOutcome>;

    /// Append a message (e.g. save-to-Sent after SMTP submission).
    async fn append(&self, mbox: &RawMailboxRef, raw: &[u8], flags: &[Flag]) -> Result<MessageRef>;

    /// Optional: begin an idle/poll loop feeding a change channel
    /// (IMAP IDLE / POP3 poll). Returns a handle whose drop stops the loop.
    async fn watch(&self, sink: ChangeSink) -> Result<WatchHandle>;

    // ── ACL (RFC 4314) + METADATA (RFC 5464), added 26.13 (t13 plan §6) ──────
    //
    // Additive, model-agnostic transport surface. Backends that do not speak
    // ACL/METADATA inherit the default `EngineError::Unsupported` below and
    // compile unchanged (POP3, plugin/bridge, and any future backend). Only
    // `mw-imap` (t13 E6) overrides them, delegating to the upstream server's
    // ACL/METADATA commands; the engine seam (E7) exposes them on the JMAP
    // method surface. The upstream IMAP server is the real enforcement point —
    // these methods carry the `identifier`+`rights` shape verbatim and do not
    // impose a policy of their own.

    /// GETACL (RFC 4314): list every `identifier`→`rights` grant on `mbox`.
    async fn get_acl(&self, mbox: &RawMailboxRef) -> Result<Vec<AclEntry>> {
        let _ = mbox;
        Err(EngineError::Unsupported("get_acl".into()))
    }

    /// SETACL (RFC 4314): set (replace) `identifier`'s `rights` on `mbox`.
    ///
    /// `rights` is an RFC 4314 rights string (e.g. `"lrswipkxtea"`); the
    /// upstream server is the authority on which bits the caller may grant.
    async fn set_acl(&self, mbox: &RawMailboxRef, identifier: &str, rights: &str) -> Result<()> {
        let _ = (mbox, identifier, rights);
        Err(EngineError::Unsupported("set_acl".into()))
    }

    /// DELETEACL (RFC 4314): remove `identifier`'s entire ACL entry on `mbox`.
    async fn delete_acl(&self, mbox: &RawMailboxRef, identifier: &str) -> Result<()> {
        let _ = (mbox, identifier);
        Err(EngineError::Unsupported("delete_acl".into()))
    }

    /// LISTRIGHTS (RFC 4314): the set of rights `identifier` may be granted on
    /// `mbox` (required + optional rights tokens, server-defined).
    async fn list_rights(&self, mbox: &RawMailboxRef, identifier: &str) -> Result<Vec<String>> {
        let _ = (mbox, identifier);
        Err(EngineError::Unsupported("list_rights".into()))
    }

    /// MYRIGHTS (RFC 4314): the rights string the authenticated user holds on
    /// `mbox`. Drives the web UI's read/write affordance gating (E8).
    async fn my_rights(&self, mbox: &RawMailboxRef) -> Result<String> {
        let _ = mbox;
        Err(EngineError::Unsupported("my_rights".into()))
    }

    /// GETMETADATA (RFC 5464): fetch annotation `entries`. `mbox == None`
    /// targets server-level metadata (the empty mailbox `""`); `Some(_)` targets
    /// a specific mailbox's annotations.
    async fn get_metadata(
        &self,
        mbox: Option<&RawMailboxRef>,
        entries: &[String],
    ) -> Result<Vec<MetadataEntry>> {
        let _ = (mbox, entries);
        Err(EngineError::Unsupported("get_metadata".into()))
    }

    /// SETMETADATA (RFC 5464): set one annotation `entry` to `value`, or remove
    /// it when `value == None` (RFC 5464 NIL). `mbox` scope as in `get_metadata`.
    async fn set_metadata(
        &self,
        mbox: Option<&RawMailboxRef>,
        entry: &str,
        value: Option<&str>,
    ) -> Result<()> {
        let _ = (mbox, entry, value);
        Err(EngineError::Unsupported("set_metadata".into()))
    }
}

/// One ACL grant (RFC 4314): an `identifier` (user/group/`anyone`) paired with
/// its rights string. Frozen transport shape shared by the `mw-imap` impl
/// (t13 E6) and the engine seam (E7); serializes directly onto the JMAP surface
/// (no separate JMAP type is required — t13 plan §6 E7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AclEntry {
    /// RFC 4314 identifier: a username, a group (`-`-prefixed negative rights
    /// included), or a special identifier such as `anyone`.
    pub identifier: String,
    /// RFC 4314 rights string — the subset of `lrswipkxtea` (+ server
    /// extensions) granted to `identifier`.
    pub rights: String,
}

/// One METADATA annotation (RFC 5464): an entry name paired with its value.
/// `value == None` encodes the RFC 5464 NIL (entry absent / removed). Frozen
/// transport shape shared by the `mw-imap` impl (E6) and the engine seam (E7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataEntry {
    /// RFC 5464 entry name, e.g. `/shared/comment` or `/private/comment`.
    pub entry: String,
    /// Entry value; `None` is the RFC 5464 NIL (no value / removed).
    pub value: Option<String>,
}

/// Feature flags detected from the server so `mw-engine` can degrade (plan §6.1).
///
/// The backend fills these from `CAPABILITY` (IMAP) / `CAPA` (POP3); the engine
/// reads them to choose the strongest [`SyncCursor`] and command variants.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BackendCaps {
    /// IMAP4rev2 (RFC 9051) — else fall back to RFC 3501 behaviour.
    pub imap4rev2: bool,
    /// QRESYNC (RFC 7162) — strongest sync ladder rung.
    pub qresync: bool,
    /// CONDSTORE (RFC 7162) — MODSEQ-based delta.
    pub condstore: bool,
    /// UIDPLUS (RFC 4315) — COPYUID/APPENDUID on move/append.
    pub uidplus: bool,
    /// MOVE (RFC 6851) — else COPY + STORE \Deleted + EXPUNGE.
    #[serde(rename = "move")]
    pub r#move: bool,
    /// SPECIAL-USE (RFC 6154) — role attributes on LIST.
    pub special_use: bool,
    /// LIST-STATUS (RFC 5819) — counts folded into LIST.
    pub list_status: bool,
    /// IDLE (RFC 2177) — server-push change notification.
    pub idle: bool,
    /// OBJECTID (RFC 8474) — stable EMAILID/THREADID hints.
    pub objectid: bool,
    /// ESEARCH (RFC 4731) — extended SEARCH return.
    pub esearch: bool,
    /// ENABLE (RFC 5161).
    pub enable: bool,
    /// ID (RFC 2971).
    pub id: bool,
    /// COMPRESS=DEFLATE (RFC 4978) — optional in V1.
    pub compress: bool,
    /// SORT (RFC 5256) — server-side result ordering.
    pub sort: bool,
    /// THREAD=REFERENCES (RFC 5256) — References-based server-side threading.
    pub thread_references: bool,
    /// THREAD=ORDEREDSUBJECT (RFC 5256) — subject-based server-side threading.
    pub thread_orderedsubject: bool,
    /// ACL (RFC 4314) advertised. Detection only — mailbox-rights read/edit UI
    /// is deferred (t12 plan §6); the flag lets callers feature-probe.
    pub acl: bool,
    /// METADATA / METADATA-SERVER (RFC 5464) advertised. Detection only —
    /// annotation read/edit UI is deferred (t12 plan §6).
    pub metadata: bool,
    /// SASL AUTHENTICATE PLAIN.
    pub sasl_plain: bool,
    /// SASL AUTHENTICATE LOGIN.
    pub sasl_login: bool,
    /// SASL AUTHENTICATE XOAUTH2 (Gmail/Outlook).
    pub sasl_xoauth2: bool,
    /// SASL AUTHENTICATE OAUTHBEARER (RFC 7628).
    pub sasl_oauthbearer: bool,
    /// SASL AUTHENTICATE SCRAM-SHA-256 (RFC 7677).
    pub sasl_scram_sha256: bool,
    /// SASL AUTHENTICATE SCRAM-SHA-256-PLUS (RFC 7677, channel-bound).
    pub sasl_scram_sha256_plus: bool,
}

/// Special-use role of a mailbox (RFC 6154), mapped to `mw_jmap::Mailbox.role`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MailboxRole {
    Inbox,
    Archive,
    Drafts,
    Sent,
    Trash,
    Junk,
    Flagged,
    All,
    /// No special-use role (an ordinary user folder).
    None,
}

/// A backend's own identity for a mailbox — never exposed to the UI.
///
/// The engine maps this to/from an opaque `mw_jmap::Mailbox.id`. For POP3 there
/// is exactly one, `INBOX`, with `uidvalidity == 0`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RawMailboxRef {
    /// Server-side mailbox name/path (IMAP hierarchy) or `"INBOX"` for POP3.
    pub name: String,
    /// UIDVALIDITY at resolution time (`0` for POP3, which has none).
    pub uidvalidity: u32,
}

/// A mailbox as enumerated by the backend, with role and status counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawMailbox {
    pub mailbox_ref: RawMailboxRef,
    pub role: MailboxRole,
    /// Parent mailbox name for hierarchy, if any.
    pub parent: Option<String>,
    /// UIDNEXT reported by STATUS/SELECT.
    pub uidnext: u32,
    /// HIGHESTMODSEQ (0 when CONDSTORE is unavailable).
    pub highestmodseq: u64,
    /// Total message count.
    pub total: u32,
    /// Unseen/unread count.
    pub unread: u32,
}

/// A backend-level reference to a single message — never a stable id.
///
/// The engine translates it to/from the opaque `mw_jmap::Email.id`; backends
/// never see stable ids (plan §1.6).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageRef {
    /// IMAP coordinates: mailbox + UIDVALIDITY + UID.
    Imap {
        mailbox: RawMailboxRef,
        uidvalidity: u32,
        uid: u32,
    },
    /// POP3 coordinate: the UIDL string (RFC 1939 / 2449).
    Pop3 { uidl: String },
    /// Opaque bridge/plugin message ref (V7, R1-residual — parallel to
    /// [`SyncCursor::Plugin`]). A WASM/native bridge backend carries its **native**
    /// message id here losslessly — Microsoft Graph message id, Gmail message id, EWS
    /// `ItemId`(+`ChangeKey`) — instead of smuggling it through one of the IMAP/POP3
    /// coordinates above (e12 previously packed a Gmail id into `Pop3 { uidl }`). The
    /// engine treats `raw` as opaque and round-trips it verbatim through the
    /// `mw-plugin` adapter (WIT `message-ref.raw` passthrough); only the originating
    /// backend interprets it. Additive: `mw-imap`/`mw-pop3` never emit or consume it.
    ///
    /// A `String` (not `Vec<u8>`) is natural for provider ids, and — unlike
    /// `SyncCursor` — `MessageRef` is **externally**-tagged, so a struct variant is not
    /// forced; `{ raw }` mirrors the `SyncCursor::Plugin { opaque }` shape.
    Plugin { raw: String },
}

/// An IMAP system flag or an arbitrary server/JMAP keyword.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Flag {
    Seen,
    Answered,
    Flagged,
    Deleted,
    Draft,
    Recent,
    /// Any other keyword (`$Forwarded`, `$Junk`, user labels, …).
    Keyword(String),
}

/// Full raw bytes for one message, fetched for MIME parsing in the jail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawMessage {
    pub message_ref: MessageRef,
    /// Full RFC822 bytes; parsed by `mw-mime` inside the `mw-render` jail.
    pub raw: Vec<u8>,
    /// Current flags at fetch time.
    pub flags: Vec<Flag>,
    /// INTERNALDATE as RFC3339, when the server supplies it.
    pub internaldate: Option<String>,
}

/// The set of changes an incremental `sync_mailbox` produced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailboxDelta {
    /// Newly appeared messages (refs only; bodies fetched via `fetch_raw`).
    pub added: Vec<MessageRef>,
    /// Messages whose flags changed, paired with their new full flag set.
    pub flag_changes: Vec<(MessageRef, Vec<Flag>)>,
    /// Messages that vanished (EXPUNGE / VANISHED / dropped UIDL).
    pub removed: Vec<MessageRef>,
    /// The cursor to persist for the next incremental sync.
    pub next_cursor: SyncCursor,
}

/// Persisted sync position; the backend returns the strongest form it supports
/// and the engine stores it verbatim (plan §1.8).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SyncCursor {
    /// QRESYNC (RFC 7162): resync via UIDVALIDITY + HIGHESTMODSEQ.
    Qresync {
        uidvalidity: u32,
        highestmodseq: u64,
    },
    /// CONDSTORE (RFC 7162): MODSEQ-based delta.
    Condstore { uidvalidity: u32, modseq: u64 },
    /// Plain UID-window poll: everything at/after `uidnext` is new.
    UidWindow { uidvalidity: u32, uidnext: u32 },
    /// POP3 UIDL diff: the set of UIDLs already ingested.
    Pop3Uidl { seen: BTreeSet<String> },
    /// Opaque bridge/plugin cursor (plan §3 e8, R1-residual). A WASM/native bridge
    /// backend carries its **native** sync token here losslessly — Graph
    /// `deltaLink`, Gmail `historyId`, EWS `SyncState` — instead of smuggling it
    /// through one of the IMAP/POP3-shaped variants above. The engine persists it
    /// verbatim and never inspects it; only the originating backend interprets
    /// `opaque`. Additive: `mw-imap`/`mw-pop3` never emit or consume it.
    ///
    /// This is a **struct** variant, not a newtype `Plugin(Vec<u8>)`, because the
    /// enum is `#[serde(tag = "kind")]` internally-tagged — serde cannot encode a
    /// newtype variant wrapping a sequence (`Vec<u8>`) into an internally-tagged
    /// form, but a struct field round-trips cleanly through `cursor_to_json`.
    Plugin { opaque: Vec<u8> },
}

/// Result of a `move_messages` call (plan §2.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveOutcome {
    /// UIDPLUS present: server reported COPYUID — destination UIDVALIDITY + UIDs
    /// (parallel to the source refs the engine passed in).
    Uidplus { uidvalidity: u32, uids: Vec<u32> },
    /// No UIDPLUS (e.g. some Gmail ops): the engine must re-derive the moved
    /// messages' destination refs by Message-ID.
    RederiveByMessageId,
}

/// A change event emitted by a backend `watch` loop into the engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeEvent {
    /// A mailbox reported activity (IMAP IDLE / POP3 poll); engine should sync.
    MailboxChanged { mailbox: RawMailboxRef },
    /// The upstream connection dropped; engine should reconnect.
    Disconnected,
}

/// The sender half handed to a backend's `watch` loop.
///
/// Wraps an unbounded channel so a backend can push [`ChangeEvent`]s without
/// blocking on the engine's consumption rate; the engine holds the receiver.
#[derive(Debug, Clone)]
pub struct ChangeSink {
    tx: mpsc::UnboundedSender<ChangeEvent>,
}

impl ChangeSink {
    /// Wrap a channel sender (constructed by the engine) as a sink.
    pub fn new(tx: mpsc::UnboundedSender<ChangeEvent>) -> Self {
        Self { tx }
    }

    /// Emit a change event; errors only if the engine stopped listening.
    pub fn emit(&self, event: ChangeEvent) -> Result<()> {
        self.tx
            .send(event)
            .map_err(|_| EngineError::Transport("change sink closed".into()))
    }
}

/// Handle to a running backend `watch` loop.
///
/// Signalling it (or dropping it) tells the loop to terminate — the engine
/// holds this so it can cancel IDLE/poll on logout or reconnect.
#[derive(Debug)]
pub struct WatchHandle {
    stop: watch::Sender<bool>,
}

impl WatchHandle {
    /// Wrap a stop-signal sender (the backend watches its receiver).
    pub fn new(stop: watch::Sender<bool>) -> Self {
        Self { stop }
    }

    /// Signal the backend's watch loop to terminate.
    pub fn stop(&self) {
        let _ = self.stop.send(true);
    }
}
