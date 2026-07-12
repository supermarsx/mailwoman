//! Engine-local per-message metadata + the tag registry (plan §1.5, §2.7).
//!
//! IMAP can hold labels (JMAP keywords) but not pins/snooze/follow-up or tag
//! colors. Those live engine-local in `message_meta` + `tags` (§2.7), keyed by
//! `stable_id`, and are surfaced as extra `Email` properties (§2.1).
//!
//! ## Scaffolder note (e0)
//! e0 freezes these shapes; e9 fills `message_meta`/`tags` CRUD + the snooze
//! resurface scheduler.

use serde::{Deserialize, Serialize};

/// Engine-local extras surfaced on `Email` (§2.1): `pinned`, `snoozedUntil`,
/// `followUpAt`. Keyed by `stable_id` in `message_meta` (§2.7).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailMeta {
    pub pinned: bool,
    /// RFC3339 resurface time, or `None` when not snoozed.
    pub snoozed_until: Option<String>,
    /// RFC3339 follow-up reminder time, or `None`.
    pub follow_up_at: Option<String>,
}

/// A per-user tag color/icon registry entry (plan §1.5, §2.7 `tags`). The label
/// itself round-trips to IMAP as a JMAP keyword; only the presentation metadata
/// lives here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub name: String,
    /// CSS color token or hex (validated by the web token layer, e4/e7).
    pub color: String,
    pub icon: Option<String>,
}
