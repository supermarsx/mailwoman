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
pub mod engine;
pub mod jmap;
pub mod mapping;
pub mod thread;

pub use account::{AccountPolicy, AccountRuntime, MailSubmitter};
pub use backend::{
    AccountBackend, BackendCaps, ChangeEvent, ChangeSink, EngineError, Flag, MailboxDelta,
    MailboxRole, MessageRef, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result,
    SyncCursor, WatchHandle,
};
pub use engine::Engine;
pub use jmap::session_json;
