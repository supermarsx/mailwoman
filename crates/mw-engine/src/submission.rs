//! The real, persisted `EmailSubmission` (plan §1.3, §2.1) — the undo-send /
//! send-later queue that replaces the V1 synchronous `submit_email`.
//!
//! ## Scaffolder note (e0)
//! e0 freezes the row shape surfaced by `EmailSubmission/get`/`/query` (the
//! Outbox) and `/set` (create=enqueue, update=cancel). e9 owns the persisted
//! `submissions` table + the delayed dispatcher task.

use serde::{Deserialize, Serialize};

/// Lifecycle of a submission (plan §1.3). `pending` while the undo/send-at
/// window is open; `final` once SMTP fired; `canceled` if undone in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UndoStatus {
    Pending,
    Final,
    Canceled,
}

/// A persisted submission surfaced by `EmailSubmission/get`/`/query` (§2.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailSubmission {
    pub id: String,
    pub email_id: String,
    pub identity_id: Option<String>,
    /// Scheduled send time (RFC3339) for send-later; `None` = fire as soon as
    /// the hold window elapses.
    pub send_at: Option<String>,
    pub undo_status: UndoStatus,
    /// Engine-held delay before SMTP dispatch (the undo-send window), in seconds.
    pub mailwoman_hold_seconds: u32,
}
