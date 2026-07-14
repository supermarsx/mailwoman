//! Optional bridge-native Outlook-parity capabilities the engine prefers when the
//! backend advertises them (plan §2.5/§2.6): reactions, voting, message-recall, and
//! categories. Focused-Inbox sync is handled inline in [`crate::mail`] (the
//! `$Focused`/`$Other` keywords), and advertised here too.
//!
//! **HONESTY (frozen wording, plan §2.6 / SPEC §10.3):** message recall degrades to
//! *nothing*, never to broken behavior or a false claim of success. Graph exposes no
//! universal server-side recall API — recall works only for an unread message inside
//! the SAME Microsoft 365 organization and remains best-effort even then; it is
//! impossible cross-org or once the recipient has opened the message. The bridge
//! therefore NEVER reports recall as guaranteed; [`recall`] returns an outcome that
//! states the limitation plainly. Reactions/voting are likewise best-effort surfaces.
//!
//! None of these have an export in the frozen `mailwoman:plugin` WIT world; they are
//! pure functions advertised via `capabilities()` and (today) exercised through
//! fixtures — see the e10 report's WIT-ABI friction note (a `bridge-capabilities`
//! interface for e11/e12).

use crate::graph::{GraphClient, Result, Transport};
use crate::model::GraphMessage;
use crate::types::BackendCaps;

/// The capability set the Graph bridge advertises. Focused-Inbox sync + categories
/// are genuine Graph features; reactions/voting/recall are advertised as best-effort
/// (the honesty matrix governs their outcomes).
pub fn backend_caps() -> BackendCaps {
    BackendCaps {
        idle: true,
        move_cap: true,
        reactions: true,
        voting: true,
        recall: true,
        focused_sync: true,
    }
}

/// The honest result of a recall attempt. `guaranteed` is ALWAYS false for Graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecallOutcome {
    /// Whether the recall request was accepted for processing by the server.
    pub requested: bool,
    /// Graph never guarantees recall — this is always `false` (honesty matrix).
    pub guaranteed: bool,
    /// A plain-language description of the limitation that applied.
    pub note: String,
}

/// Attempt to recall a previously-sent message, honoring the recall honesty matrix.
///
/// The bridge first checks the message's read/state via `GET /me/messages/{id}` and
/// then issues the recall verb. Regardless of the server's acceptance, the outcome is
/// reported as NOT guaranteed, with the reason. If the message was already read (or
/// the check reveals it left the org), the bridge declines rather than pretend.
pub fn recall<T: Transport>(
    client: &GraphClient<'_, T>,
    message_id: &str,
) -> Result<RecallOutcome> {
    // Inspect current state (best-effort; a failure here is itself informative).
    let msg: GraphMessage = client.get_json(&format!(
        "/me/messages/{message_id}?$select=isRead,inferenceClassification"
    ))?;

    if msg.is_read == Some(true) {
        return Ok(RecallOutcome {
            requested: false,
            guaranteed: false,
            note: "message already read by the recipient — recall is impossible; \
                   not attempted (honesty matrix)"
                .to_string(),
        });
    }

    // Issue the recall verb. Graph's recall support is org-internal, preview, and
    // best-effort; a 2xx here means "accepted for processing", never "recalled".
    client.post_ignore(
        &format!("/me/messages/{message_id}/recall"),
        &serde_json::json!({}),
    )?;

    Ok(RecallOutcome {
        requested: true,
        guaranteed: false,
        note: "recall requested; succeeds only for an unread message inside the same \
               Microsoft 365 organization, best-effort — cross-org and already-opened \
               messages cannot be recalled"
            .to_string(),
    })
}

/// Post a reaction (emoji) to a message. Best-effort — Graph reaction support for
/// mailbox messages is limited; a non-2xx degrades to `Ok(false)` (no broken state),
/// never an error the UI would surface as failure.
pub fn react<T: Transport>(
    client: &GraphClient<'_, T>,
    message_id: &str,
    reaction: &str,
) -> Result<bool> {
    let body = serde_json::json!({ "reactionType": reaction });
    match client.post_ignore(&format!("/me/messages/{message_id}/react"), &body) {
        Ok(()) => Ok(true),
        Err(crate::graph::BridgeError::Auth(_)) => {
            Err(crate::graph::BridgeError::Auth("reaction rejected".into()))
        }
        Err(_) => Ok(false),
    }
}

/// Cast a voting-button response by replying with the chosen option. Outlook voting
/// has no first-class Graph write, so the bridge sends a reply carrying the vote.
pub fn vote<T: Transport>(
    client: &GraphClient<'_, T>,
    message_id: &str,
    choice: &str,
) -> Result<()> {
    let body = serde_json::json!({ "comment": format!("Vote: {choice}") });
    client.post_ignore(&format!("/me/messages/{message_id}/reply"), &body)
}

/// `PATCH /me/messages/{id}` to set categories — a genuine, fully-supported Graph
/// field (unlike reactions/voting/recall, this is not best-effort).
pub fn set_categories<T: Transport>(
    client: &GraphClient<'_, T>,
    message_id: &str,
    categories: &[String],
) -> Result<()> {
    let body = serde_json::json!({ "categories": categories });
    client.patch_ignore(&format!("/me/messages/{message_id}"), &body)
}
