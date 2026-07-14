//! Plain, target-independent types the mapping modules produce. The `wasm32` guest
//! maps these 1:1 onto the frozen `mailwoman:plugin` WIT records; the host-side unit
//! tests assert against them directly. Keeping them separate from the WIT bindings
//! (which are `cfg(target_arch = "wasm32")`) is what lets the whole Graph mapping be
//! unit-tested on the host without a wasm toolchain.

/// A backend mailbox coordinate. `name` is the Graph folder addressing key — a
/// well-known name (`inbox`, `archive`, …) when the folder has one, else the opaque
/// folder id — so `sync_mailbox` can re-address the folder from just this ref.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailboxRef {
    pub name: String,
    pub uidvalidity: u32,
}

/// A mailbox/folder with its JMAP special-use role (lowercased).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mailbox {
    pub mailbox_ref: MailboxRef,
    pub role: String,
    pub parent: Option<String>,
    pub total: u32,
    pub unread: u32,
}

/// Server-authoritative flags/keywords (mirrors `mw_engine::Flag`). Focused-Inbox
/// state is carried as the reserved keywords `$Focused` / `$Other` (plan §2.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Flag {
    Seen,
    Answered,
    Flagged,
    Deleted,
    Draft,
    Recent,
    Keyword(String),
}

/// Reserved keyword carrying Graph `inferenceClassification == focused`.
pub const KEYWORD_FOCUSED: &str = "$Focused";
/// Reserved keyword carrying Graph `inferenceClassification == other`.
pub const KEYWORD_OTHER: &str = "$Other";

/// A backend message reference. `raw` is the Graph message id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRef {
    pub raw: String,
    pub mailbox: MailboxRef,
}

/// A fetched raw (RFC 5322 MIME) message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawMessage {
    pub message_ref: MessageRef,
    pub raw: Vec<u8>,
    pub msg_flags: Vec<Flag>,
    pub internaldate: Option<String>,
}

/// An opaque sync cursor the engine persists verbatim (here: the Graph
/// `@odata.deltaLink`/`nextLink` bytes ⇒ `SyncCursor::Plugin { opaque }`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyncCursor {
    pub opaque: Vec<u8>,
}

/// A delta of a mailbox since a cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailboxDelta {
    pub added: Vec<MessageRef>,
    pub removed: Vec<MessageRef>,
    pub flag_changes: Vec<(MessageRef, Vec<Flag>)>,
    pub next_cursor: SyncCursor,
}

/// A change event the watch loop emits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeEvent {
    MailboxChanged(MailboxRef),
    Disconnected,
}

/// Feature flags the bridge advertises (incl. the optional Outlook-parity caps the
/// engine prefers when present, plan §2.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendCaps {
    pub idle: bool,
    pub move_cap: bool,
    pub reactions: bool,
    pub voting: bool,
    pub recall: bool,
    pub focused_sync: bool,
}
