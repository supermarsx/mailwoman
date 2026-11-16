//! Optional bridge-native Outlook-parity capabilities the engine prefers when the
//! backend advertises them (plan §2.5/§2.6): reactions, voting, message-recall, and
//! categories. Focused-Inbox sync is handled inline in [`crate::mail`] (the
//! `$Focused`/`$Other` keywords), and advertised here too.
//!
//! Recall uses the documented beta Sent Items action and reports only acceptance,
//! never guaranteed removal. Sender isRead does not establish recipient read state.
//! Outlook mailbox reactions have no documented Graph endpoint: capability probing
//! leaves them on the engine's standards fallback. These methods cross the additive
//! plugin-pim exports implemented in guest.rs.
//!
use crate::graph::{GraphClient, Result, Transport};
use crate::types::BackendCaps;

/// The capability set the Graph bridge advertises. Focused-Inbox sync + categories
/// are genuine Graph features; recall is beta/best-effort and voting uses a reply.
/// Mailbox reactions are unavailable through Graph.
pub fn backend_caps() -> BackendCaps {
    BackendCaps {
        idle: false,
        move_cap: true,
        reactions: false,
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

/// Request recall through Microsoft's documented beta Sent Items action. The
/// service owns recipient state and tenant policy; only acceptance is reported.
pub fn recall<T: Transport>(
    client: &GraphClient<'_, T>,
    message_id: &str,
) -> Result<RecallOutcome> {
    // The sender's Sent Items isRead says nothing about recipient delivery/read
    // state. Ask the documented beta action; only acceptance can be reported.
    client.post_empty(&format!(
        "https://graph.microsoft.com/beta/me/mailFolders/sentitems/messages/{}/recall",
        crate::graph::encode_segment(message_id)
    ))?;

    Ok(RecallOutcome {
        requested: true,
        guaranteed: false,
        note: "recall requested through Microsoft's beta API; best-effort and subject \
               to the tenant's recall policy. Delivery and recipient read state are \
               not verified by this request; check the provider's recall report"
            .to_string(),
    })
}

/// Refuse unsupported native mailbox reactions before making a network request.
pub fn react<T: Transport>(
    _client: &GraphClient<'_, T>,
    _message_id: &str,
    _reaction: &str,
) -> Result<bool> {
    Err(crate::graph::BridgeError::Unsupported("Microsoft Graph has no documented mailbox reaction API; use the standards reaction fallback".into()))
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
