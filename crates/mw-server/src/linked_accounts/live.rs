//! Live updates for linked child mailboxes (t46).
//!
//! A linked child is an ordinary engine account, so the engine already runs its
//! push source: the IMAP backend's IDLE connection, the POP3 backend's timed
//! poll, a bridge's poll loop, each with the watch loop's bounded reconnect
//! backoff (`mw_engine::Engine::start_watch`). One upstream connection serves
//! every user who linked that account. What this module adds is delivery: the
//! parent's own realtime stream (`/jmap/ws`, `/jmap/eventsource`) also carries
//! its verified links' changes, tagged with the link id and the child's opaque
//! storage authority, never the child's raw account id.
//!
//! The set of links a stream carries ([`live_links`]) is resolved from the
//! server's own link rows, the same checks `GET /api/accounts` uses for a
//! `ready` link: the sealed owner binding, the child session's clocks, both
//! accounts' linking policy and the child's account flags. It is capped at the
//! deployment's `accounts.maxLinked`. Every change is verified again
//! ([`verify_link`]) before it is written to the socket, so an unlink, a revoked
//! or expired child session, a parent sign-out, a disabled account or a
//! changed authority stops delivery at the next change, not at the next
//! periodic re-check.
use super::*;
use std::sync::OnceLock;
use tokio::sync::watch;

/// One verified link a parent's realtime stream carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LiveLink {
    /// The opaque link id: the selector the client already holds.
    pub(crate) link: String,
    /// The child's engine account id. Server-side only; never sent.
    pub(crate) account_id: String,
    /// The child's opaque storage authority (`msa1_…`).
    pub(crate) storage_authority: String,
}

/// Bumped whenever a link is created or removed, so open streams re-resolve
/// their link set at once instead of at their next periodic re-check.
fn epoch() -> &'static watch::Sender<u64> {
    static EPOCH: OnceLock<watch::Sender<u64>> = OnceLock::new();
    EPOCH.get_or_init(|| watch::channel(0).0)
}

/// Tell open realtime streams that some user's links changed.
pub(crate) fn links_changed() {
    epoch().send_modify(|n| *n = n.wrapping_add(1));
}

/// A receiver that wakes when [`links_changed`] is called.
pub(crate) fn subscribe_links() -> watch::Receiver<u64> {
    epoch().subscribe()
}

/// The links `parent`'s realtime stream may carry, in link order, at most the
/// deployment's `accounts.maxLinked`. Any read that fails yields no links (fail
/// closed); the parent's own changes are unaffected.
pub(crate) async fn live_links(state: &AppState, parent: &mw_store::Session) -> Vec<LiveLink> {
    match resolve_links(state, parent).await {
        Ok(links) => links,
        Err(why) => {
            tracing::info!("linked live updates unavailable for this stream: {why}");
            Vec::new()
        }
    }
}

async fn resolve_links(
    state: &AppState,
    parent: &mw_store::Session,
) -> Result<Vec<LiveLink>, &'static str> {
    if !enabled(state, parent)
        .await
        .map_err(|_| "policy unreadable")?
    {
        return Ok(Vec::new());
    }
    let limit = maximum(state).await.map_err(|_| "limit unreadable")?;
    let rows = state
        .store
        .linked_accounts(&parent.id)
        .await
        .map_err(|_| "links unreadable")?;
    let mut out = Vec::new();
    for row in rows {
        if out.len() >= limit {
            break;
        }
        if let Some(link) = verify_link(state, parent, &row.id).await {
            out.push(link);
        }
    }
    Ok(out)
}

/// Re-verify one link for `parent` from the store, exactly as for a `ready`
/// entry in `GET /api/accounts`. `None` when the link is gone, belongs to
/// someone else, or may not be used right now.
pub(crate) async fn verify_link(
    state: &AppState,
    parent: &mw_store::Session,
    link: &str,
) -> Option<LiveLink> {
    let (_, source) = state
        .store
        .linked_source(&parent.id, link)
        .await
        .ok()
        .flatten()?;
    if source.account_id.is_empty() || source.account_id == parent.account_id {
        return None;
    }
    crate::security_center::enforce_session_clock(state, &source)
        .await
        .ok()?;
    let gate = crate::account_gate::for_session(&state.store, &source)
        .await
        .ok()?;
    if gate.disabled || gate.password_change_required {
        return None;
    }
    if !enabled(state, parent).await.ok()? || !enabled(state, &source).await.ok()? {
        return None;
    }
    let storage_authority = state
        .store
        .session_storage_authority(&source.id)
        .await
        .ok()
        .flatten()?;
    Some(LiveLink {
        link: link.to_string(),
        account_id: source.account_id,
        storage_authority,
    })
}

/// The account's Inbox unread count as the store holds it (what `Mailbox/get`
/// reports), for the mailbox picker's badge. `None` when unknown.
pub(crate) async fn inbox_unread(state: &AppState, account_id: &str) -> Option<u32> {
    state
        .store
        .list_mailboxes(account_id)
        .await
        .ok()?
        .into_iter()
        .find(|m| m.role.as_deref() == Some("inbox"))
        .map(|m| m.unread)
}
