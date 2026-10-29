//! The unread badge colour is one value for the whole account (t39).
//!
//! The rest of the appearance object belongs to the mailbox it was saved in:
//! a linked child keeps its own theme and accent. The badge colour does not.
//! It is stored once, under the SIGN-IN (parent) storage authority, and laid
//! over the appearance that `GET /api/account/appearance` returns for the
//! parent and for every linked mailbox, so the folder list, the inbox tabs and
//! the mailbox picker show the same colour whichever mailbox is open.
//!
//! Earlier values were saved per mailbox, inside each appearance object. They
//! are read through once, not copied: until the account-wide value is first
//! written, the parent's own saved `badgeColor` is the account's value, and a
//! child's own saved value is ignored ("the parent's value wins").
use super::*;

/// The appearance field that is account wide.
pub(super) const FIELD: &str = "badgeColor";
/// A badge colour id is short (`default`, `accent`, an accent id); anything
/// longer is not stored. The client re-validates the value on read.
const MAX_LEN: usize = 64;

fn key(parent_authority: &str) -> String {
    personal_storage::key("appearance-badge", parent_authority)
}

/// The stored account-wide value. `badge: None` is a reset: the parent's
/// legacy value is NOT read through after one.
#[derive(Debug, Clone, Deserialize)]
struct Envelope {
    updated_at: i64,
    badge: Option<String>,
}

/// The sign-in storage authority behind this request: `owner` itself for the
/// sign-in mailbox, the parent session's authority for a linked mailbox.
pub(super) async fn parent_authority(
    state: &AppState,
    session: &mw_store::Session,
    headers: &HeaderMap,
    owner: &str,
) -> Result<String, Response> {
    if !headers.contains_key(crate::linked_accounts::SELECTOR) {
        return Ok(owner.to_string());
    }
    let unavailable = || StatusCode::SERVICE_UNAVAILABLE.into_response();
    let parent = match state.store.linked_context_authority(&session.id).await {
        Ok(Some(authority)) => authority.parent,
        _ => return Err(unavailable()),
    };
    match state.store.session_storage_authority(&parent.id).await {
        Ok(Some(authority)) => Ok(authority),
        _ => Err(unavailable()),
    }
}

/// The account-wide badge colour and when it was set, or `None` when the
/// account has none (the deployment default then applies).
pub(super) async fn read(
    store: &Store,
    parent_authority: &str,
) -> Result<Option<(String, i64)>, StoreError> {
    if let Some(raw) = store.get_setting(&key(parent_authority)).await?
        && let Ok(envelope) = serde_json::from_str::<Envelope>(&raw)
    {
        return Ok(envelope.badge.map(|badge| (badge, envelope.updated_at)));
    }
    // Not written yet: the parent's own per-mailbox value, from before t39.
    let legacy = do_get_appearance(store, parent_authority).await?;
    Ok(legacy.and_then(|envelope| {
        let badge = envelope.prefs.as_ref()?.get(FIELD)?.as_str()?.to_string();
        Some((badge, envelope.updated_at))
    }))
}

/// Store the account-wide value (`None` = reset).
pub(super) async fn write(
    store: &Store,
    parent_authority: &str,
    badge: Option<&str>,
    updated_at: i64,
) -> Result<(), StoreError> {
    let envelope = json!({ "v": 1, "updated_at": updated_at, "badge": badge });
    store
        .set_setting(&key(parent_authority), &envelope.to_string())
        .await
}

/// The badge colour a PUT body carries, if it is one this module stores.
pub(super) fn from_body(prefs: &serde_json::Value) -> Option<&str> {
    prefs
        .get(FIELD)
        .and_then(serde_json::Value::as_str)
        .filter(|badge| !badge.is_empty() && badge.len() <= MAX_LEN)
}

/// Lay the account-wide value over one mailbox's stored appearance. Returns
/// the appearance to send and its `updatedAt` (the later of the two writes, so
/// a device that synced this mailbox before the badge changed sees the change).
pub(super) fn overlay(
    appearance: Option<serde_json::Value>,
    updated_at: Option<i64>,
    shared: Option<(String, i64)>,
) -> (Option<serde_json::Value>, Option<i64>) {
    match shared {
        Some((badge, at)) => {
            let mut object = match appearance {
                Some(serde_json::Value::Object(map)) => map,
                _ => serde_json::Map::new(),
            };
            object.insert(FIELD.to_string(), serde_json::Value::String(badge));
            (
                Some(serde_json::Value::Object(object)),
                Some(updated_at.map_or(at, |own| own.max(at))),
            )
        }
        None => {
            let appearance = appearance.map(|mut value| {
                if let Some(map) = value.as_object_mut() {
                    map.remove(FIELD);
                }
                value
            });
            (appearance, updated_at)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_replaces_a_mailbox_value_and_keeps_the_later_stamp() {
        let (out, at) = overlay(
            Some(json!({ "theme": "ocean-dark", "badgeColor": "plum" })),
            Some(10),
            Some(("grove".into(), 20)),
        );
        assert_eq!(
            out,
            Some(json!({ "theme": "ocean-dark", "badgeColor": "grove" }))
        );
        assert_eq!(at, Some(20));
    }

    #[test]
    fn overlay_gives_a_mailbox_with_nothing_stored_the_account_value() {
        let (out, at) = overlay(None, None, Some(("accent".into(), 5)));
        assert_eq!(out, Some(json!({ "badgeColor": "accent" })));
        assert_eq!(at, Some(5));
    }

    #[test]
    fn overlay_drops_a_child_value_when_the_account_has_none() {
        let (out, at) = overlay(
            Some(json!({ "badgeColor": "plum", "density": "compact" })),
            Some(3),
            None,
        );
        assert_eq!(out, Some(json!({ "density": "compact" })));
        assert_eq!(at, Some(3));
    }

    #[test]
    fn from_body_takes_only_a_short_string() {
        assert_eq!(from_body(&json!({ "badgeColor": "plum" })), Some("plum"));
        assert_eq!(from_body(&json!({ "badgeColor": 7 })), None);
        assert_eq!(from_body(&json!({ "badgeColor": "" })), None);
        assert_eq!(from_body(&json!({ "badgeColor": "x".repeat(65) })), None);
        assert_eq!(from_body(&json!({})), None);
    }
}
