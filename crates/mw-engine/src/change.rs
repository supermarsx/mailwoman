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
    // ── V4 crypto/security datatypes (§2.2). `CryptoKey`/`MailRule` participate
    // in `*/changes` + the push `StateChange.changed` map (new keys
    // `CryptoKey`/`MailRule`), sourced from the `crypto_changes` log (e6/e7).
    // `SecurityVerdict` is a lazy read with no change token. ──
    CryptoKey,
    MailRule,
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
            ChangeType::CryptoKey => "CryptoKey",
            ChangeType::MailRule => "MailRule",
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
    // ── V4 crypto/security state tokens (plan §2.2). Emitted in the pushed
    // `changed` map under the `CryptoKey`/`MailRule` keys so a key/rule change
    // in one session reaches the others without a refresh (sourced from the
    // `crypto_changes` log via `broadcast_state`). ──
    pub crypto_key: String,
    pub mail_rule: String,
}

impl StateChange {
    /// The exact RFC 8887 wire object (frozen §2.2):
    /// `{"@type":"StateChange","changed":{"<acct>":{"Email":..,"Mailbox":..,
    /// "EmailSubmission":..,"Thread":..,"CryptoKey":..,"MailRule":..}}}`. Encoded
    /// once here so the WS server + the web client (`contracts/push.ts`) cannot
    /// drift.
    pub fn to_wire(&self) -> serde_json::Value {
        let mut inner = serde_json::Map::new();
        inner.insert("Email".into(), self.email.clone().into());
        inner.insert("Mailbox".into(), self.mailbox.clone().into());
        inner.insert("EmailSubmission".into(), self.submission.clone().into());
        inner.insert("Thread".into(), self.thread.clone().into());
        inner.insert("CryptoKey".into(), self.crypto_key.clone().into());
        inner.insert("MailRule".into(), self.mail_rule.clone().into());
        let mut changed = serde_json::Map::new();
        changed.insert(self.account_id.clone(), serde_json::Value::Object(inner));
        serde_json::json!({ "@type": "StateChange", "changed": changed })
    }
}

/// The account's current [`StateChange`], read from the store (t35).
///
/// The same fields `Engine::broadcast_state` publishes, from the same two
/// statements (`current_core_mail_states` + `current_crypto_states`, read
/// concurrently; `Thread` carries the `Email` token, as there). The difference
/// is the failure mode: `broadcast_state` degrades a failed read to zeros, this
/// returns the error, because its callers (a realtime stream catching up after
/// a gap, a relay consumer's sweep) must not send an account state backwards.
///
/// State tokens are absolute, so sending the value read here after a gap
/// covers every change the gap skipped.
pub async fn state_change_from_store(
    store: &mw_store::Store,
    account_id: &str,
) -> Result<StateChange, mw_store::StoreError> {
    let (mail, crypto) = tokio::join!(
        store.current_core_mail_states(account_id),
        store.current_crypto_states(account_id),
    );
    let (mail, crypto) = (mail?, crypto?);
    let read = |counters: &std::collections::HashMap<String, u64>, kind: ChangeType| {
        counters
            .get(kind.as_str())
            .copied()
            .unwrap_or(0)
            .to_string()
    };
    let email = read(&mail, ChangeType::Email);
    Ok(StateChange {
        account_id: account_id.to_string(),
        thread: email.clone(),
        email,
        mailbox: read(&mail, ChangeType::Mailbox),
        submission: read(&mail, ChangeType::EmailSubmission),
        crypto_key: read(&crypto, ChangeType::CryptoKey),
        mail_rule: read(&crypto, ChangeType::MailRule),
    })
}

// ── Typed events (26.20 t28-e18) ─────────────────────────────────────────────

/// What happened, for an outbound webhook that asked for it. A [`StateChange`]
/// says only that a state token moved; these say which kind of change moved it,
/// so a webhook subscribed to new mail is not called for a flag change.
///
/// Emitted by the engine at the mutation that causes it (see [`EventSink`]); the
/// wire names are [`EventKind::as_str`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    /// A message the account did not have before arrived from the mail server.
    MessageReceived,
    /// A message's keywords (read, flagged, …) changed through the API.
    MessageFlags,
    /// A message moved to another folder.
    MessageMoved,
    /// A message was deleted, here or on the mail server.
    MessageDeleted,
    /// The mail server accepted a submission.
    SubmissionSent,
    /// A submission was given up on: it will not be sent.
    SubmissionFailed,
    /// A calendar, event, task, note, address book, contact or group changed.
    PimChanged,
}

impl EventKind {
    /// Every kind, in a stable order (the admin screen lists them this way).
    pub const ALL: [EventKind; 7] = [
        EventKind::MessageReceived,
        EventKind::MessageFlags,
        EventKind::MessageMoved,
        EventKind::MessageDeleted,
        EventKind::SubmissionSent,
        EventKind::SubmissionFailed,
        EventKind::PimChanged,
    ];

    /// The wire name (`X-Mailwoman-Event`, the payload `type`, the stored filter).
    pub fn as_str(self) -> &'static str {
        match self {
            EventKind::MessageReceived => "message.received",
            EventKind::MessageFlags => "message.flags",
            EventKind::MessageMoved => "message.moved",
            EventKind::MessageDeleted => "message.deleted",
            EventKind::SubmissionSent => "submission.sent",
            EventKind::SubmissionFailed => "submission.failed",
            EventKind::PimChanged => "pim.changed",
        }
    }

    /// The kind with this wire name.
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == name)
    }
}

/// One typed event: which account, what happened, and the opaque ids it touched
/// (Email, EmailSubmission or PIM object ids). Never a subject, address or body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineEvent {
    pub account_id: String,
    pub kind: EventKind,
    pub ids: Vec<String>,
}

/// Where the engine sends [`EngineEvent`]s. `mw-server` attaches one that feeds
/// the webhook dispatcher ([`crate::Engine::attach_event_sink`]); with none
/// attached nothing is emitted. `emit` must not block: it is called on the
/// mutation path.
pub trait EventSink: Send + Sync {
    fn emit(&self, event: EngineEvent);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_kinds_round_trip_their_wire_names() {
        for k in EventKind::ALL {
            assert_eq!(EventKind::parse(k.as_str()), Some(k));
        }
        assert_eq!(EventKind::parse("StateChange"), None);
        let names: std::collections::HashSet<&str> =
            EventKind::ALL.iter().map(|k| k.as_str()).collect();
        assert_eq!(names.len(), EventKind::ALL.len(), "wire names are distinct");
    }

    /// `state_change_from_store` sends what `broadcast_state` sends for the same
    /// store state: an untouched account, every counter populated at distinct
    /// values, and crypto-only counters.
    #[tokio::test]
    async fn state_change_from_store_matches_broadcast_state() {
        use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let engine = crate::Engine::new(store.clone());
        let account = |name: &'static str| {
            let store = store.clone();
            async move {
                store
                    .create_account(
                        &NewAccount {
                            kind: AccountKind::Imap,
                            host: "h",
                            port: 993,
                            tls: "implicit",
                            username: name,
                            sync_policy_json: "{}",
                        },
                        &Credentials {
                            username: name.into(),
                            password: "p".into(),
                        },
                    )
                    .await
                    .unwrap()
            }
        };
        let full = account("parity-full").await;
        let crypto_only = account("parity-crypto").await;
        let mail = [
            ChangeType::Email,
            ChangeType::Mailbox,
            ChangeType::EmailSubmission,
        ];
        for (n, kind) in mail.iter().enumerate() {
            for i in 0..=n {
                engine
                    .record_change(&full, *kind, &format!("m{n}-{i}"), ChangeOp::Created)
                    .await
                    .unwrap();
            }
        }
        for (n, kind) in [ChangeType::CryptoKey, ChangeType::MailRule]
            .iter()
            .enumerate()
        {
            for i in 0..=n + 3 {
                for id in [&full, &crypto_only] {
                    engine
                        .record_crypto_change(id, *kind, &format!("c{n}-{i}"), ChangeOp::Created)
                        .await
                        .unwrap();
                }
            }
        }
        for id in [full.as_str(), crypto_only.as_str(), "parity-never-seen"] {
            let mut rx = engine.subscribe();
            engine.broadcast_state(id).await;
            let sent = rx.try_recv().expect("broadcast_state publishes");
            let read = state_change_from_store(&store, id).await.unwrap();
            assert_eq!(read, sent, "{id}");
        }
        let full_read = state_change_from_store(&store, &full).await.unwrap();
        assert_ne!(full_read.email, "0", "the premise: populated counters");
        assert_ne!(full_read.mail_rule, full_read.crypto_key, "distinct values");
    }

    #[test]
    fn state_change_wire_shape_is_rfc8887() {
        let sc = StateChange {
            account_id: "acct1".into(),
            email: "5".into(),
            mailbox: "3".into(),
            submission: "2".into(),
            thread: "5".into(),
            crypto_key: "4".into(),
            mail_rule: "1".into(),
        };
        let wire = sc.to_wire();
        assert_eq!(wire["@type"], "StateChange");
        assert_eq!(wire["changed"]["acct1"]["Email"], "5");
        assert_eq!(wire["changed"]["acct1"]["EmailSubmission"], "2");
        assert_eq!(wire["changed"]["acct1"]["CryptoKey"], "4");
        assert_eq!(wire["changed"]["acct1"]["MailRule"], "1");
    }
}
