//! Real per-account state tokens, `*/changes`, and the realtime broadcast
//! (frozen §2.1/§2.2). Replaces the V1 `SESSION_STATE = "engine-0"` constant
//! (plan §1.2).
//!
//! ## Scaffolder note (e0)
//! e0 freezes these shapes; e9 sources the tokens from the store `changes`
//! counter and feeds [`StateChange`] from the `start_watch` loop. e10 serializes
//! [`StateChange::to_wire`] onto `/jmap/ws` + `/jmap/eventsource`. Only the wire
//! encoder is implemented here (it is the contract, not engine logic).

use serde::{Deserialize, Serialize};

/// A per-account monotonic state token (opaque string), advanced on any account
/// change so `Email/changes`/`Mailbox/changes`/`Email/queryChanges` can answer
/// "what moved since state X" (plan §1.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateToken(pub String);

/// The datatype a change touched. Serializes to the JMAP PascalCase type name
/// used as a `StateChange.changed` key (RFC 8887).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Email,
    Mailbox,
    EmailSubmission,
    Thread,
    // ── V3 PIM datatypes (§1.8/§2.2). Each participates in `*/changes` + the
    // push `StateChange.changed` map, sourced from the `pim_changes` log (e8). ──
    Calendar,
    CalendarEvent,
    Task,
    Note,
    AddressBook,
    ContactCard,
    ContactGroup,
}

impl ChangeType {
    /// The JMAP PascalCase type name used as a `changes` row key + a
    /// `StateChange.changed` key (RFC 8887).
    pub fn as_str(self) -> &'static str {
        match self {
            ChangeType::Email => "Email",
            ChangeType::Mailbox => "Mailbox",
            ChangeType::EmailSubmission => "EmailSubmission",
            ChangeType::Thread => "Thread",
            ChangeType::Calendar => "Calendar",
            ChangeType::CalendarEvent => "CalendarEvent",
            ChangeType::Task => "Task",
            ChangeType::Note => "Note",
            ChangeType::AddressBook => "AddressBook",
            ChangeType::ContactCard => "ContactCard",
            ChangeType::ContactGroup => "ContactGroup",
        }
    }
}

/// The operation a [`ChangeRecord`] records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeOp {
    Created,
    Updated,
    Destroyed,
}

impl ChangeOp {
    /// The lowercase op token persisted in the `changes` log.
    pub fn as_str(self) -> &'static str {
        match self {
            ChangeOp::Created => "created",
            ChangeOp::Updated => "updated",
            ChangeOp::Destroyed => "destroyed",
        }
    }
}

/// One row of the store `changes` log (plan §2.7): the raw material for state
/// diffs. e9 appends one per mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeRecord {
    pub account_id: String,
    pub kind: ChangeType,
    pub state: u64,
    pub stable_id: String,
    pub op: ChangeOp,
}

/// The `{oldState,newState,created,updated,destroyed}` response shape for
/// `Email/changes` / `Mailbox/changes` / `EmailSubmission/changes` (§2.1).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Changes {
    pub old_state: String,
    pub new_state: String,
    pub created: Vec<String>,
    pub updated: Vec<String>,
    pub destroyed: Vec<String>,
    pub has_more_changes: bool,
}

/// The RFC 8887 `StateChange` pushed over `/jmap/ws` + `/jmap/eventsource`
/// (§2.2). The engine `broadcast`s one after each resync (plan §1.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateChange {
    pub account_id: String,
    pub email: String,
    pub mailbox: String,
    pub submission: String,
    pub thread: String,
}

impl StateChange {
    /// The exact RFC 8887 wire object (frozen §2.2):
    /// `{"@type":"StateChange","changed":{"<acct>":{"Email":..,"Mailbox":..,
    /// "EmailSubmission":..,"Thread":..}}}`. Encoded once here so the WS server
    /// (e10) and the web client (e6, `contracts/push.ts`) cannot drift.
    pub fn to_wire(&self) -> serde_json::Value {
        let mut inner = serde_json::Map::new();
        inner.insert("Email".into(), self.email.clone().into());
        inner.insert("Mailbox".into(), self.mailbox.clone().into());
        inner.insert("EmailSubmission".into(), self.submission.clone().into());
        inner.insert("Thread".into(), self.thread.clone().into());
        let mut changed = serde_json::Map::new();
        changed.insert(self.account_id.clone(), serde_json::Value::Object(inner));
        serde_json::json!({ "@type": "StateChange", "changed": changed })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_change_wire_shape_is_rfc8887() {
        let sc = StateChange {
            account_id: "acct1".into(),
            email: "5".into(),
            mailbox: "3".into(),
            submission: "2".into(),
            thread: "5".into(),
        };
        let wire = sc.to_wire();
        assert_eq!(wire["@type"], "StateChange");
        assert_eq!(wire["changed"]["acct1"]["Email"], "5");
        assert_eq!(wire["changed"]["acct1"]["EmailSubmission"], "2");
    }
}
