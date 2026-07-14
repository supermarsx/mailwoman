#![forbid(unsafe_code)]
//! `mw-engine` — the orchestrator that drives account backends
//! (`mw-imap`, `mw-pop3`, …) and presents the JMAP surface `apps/web`
//! already consumes (plan §0, SPEC §6.5).
//!
//! In V1 this crate owns the sync engine, the UID↔stable-id map, and
//! engine-side JWZ threading. The scaffolder (e0) authors only the frozen
//! [`backend`] seam; the orchestration itself is filled in by e6.

pub mod account;
pub mod backend;
pub mod change;
pub mod dispatcher;
pub mod engine;
pub mod identity;
pub mod jmap;
pub mod mapping;
pub mod meta;
pub mod pim;
pub mod query;
pub mod rules;
pub mod search_index;
pub mod security;
pub mod state;
pub mod submission;
pub mod thread;
pub mod v6;

pub use account::{AccountPolicy, AccountRuntime, MailSubmitter};
pub use backend::{
    AccountBackend, BackendCaps, ChangeEvent, ChangeSink, EngineError, Flag, MailboxDelta,
    MailboxRole, MessageRef, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result,
    SyncCursor, WatchHandle,
};
pub use engine::{BlobData, Engine};
pub use jmap::session_json;

// ── V2 frozen types (§2.1/§2.2) authored by e0; logic filled by e9/e10. ──
pub use change::{ChangeOp, ChangeRecord, ChangeType, Changes, StateChange, StateToken};
pub use identity::Identity;
pub use meta::{EmailMeta, Tag};
pub use query::{Comparator, EmailFilter, SavedSearch, SortProperty};
pub use submission::{EmailSubmission, UndoStatus};

// ── V3 frozen PIM types (§2.1) authored by e0; logic filled by e8. ──
pub use pim::types::{
    AddressBook, Alert, Anniversary, Calendar, CalendarEvent, CalendarShare, ContactCard,
    ContactEmail, ContactGroup, ContactName, ContactValue, EventLocation, Note, NoteLink,
    Participant, Task,
};

// ── V4 frozen crypto/security types (§2.1) authored by e0; logic filled by e6.
// Re-exported from `mw-crypto` (single source of truth, plan §1.5) so the engine,
// the mock, and the WASM boundary emit byte-identical shapes. ──
pub use security::types::{
    AttachmentRisk, AuthVerdict, CryptoKey, DlpRule, DlpVerdict, MailRule, ReceivedHop,
    SecurityVerdict, SignatureVerdict,
};

// ── V6 additive wiring (§3 e10): cache-aside + zero-access opacity + audit feed.
// Inert until e11 attaches a Cache / posture source / audit feed. The mw-cache
// types are re-exported so the mount site + engine share one contract. ──
pub use v6::{
    AccountPosture, AccountPostureSource, AuditEvent, AuditFeed, Cache, CacheClass,
    PlaintextDerived, StandardPosture, V6Hooks,
};
