//! Server-side gate for destroying messages. The client hides the Delete
//! button; this refuses the request itself.
//!
//! * An account under a legal hold (`dsr.legalHold.subjects`) or with an active
//!   processing restriction cannot destroy any message through JMAP.
//! * `Mailbox/stats` (the folder Properties sizes) is refused when
//!   `properties.folder.sizeCompute` is `off`.
//! * `Email/set` with `mailwomanPermanent: true` (what the message Properties
//!   dialog sends) is refused when `properties.message.permanentDelete` is `deny`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use crate::AppState;

/// The argument the Properties dialog adds to a permanent delete.
pub(crate) const PERMANENT_MARKER: &str = "mailwomanPermanent";

/// Why a destroy is refused, as `(code, message)`.
pub(crate) fn decide(held: bool, restricted: bool, permanent: bool, policy: &str) -> Option<(&'static str, &'static str)> {
    if held {
        return Some(("legalHold", "this account is under a legal hold; messages cannot be permanently deleted"));
    }
    if restricted {
        return Some(("processingRestricted", "processing of this account is restricted; messages cannot be permanently deleted"));
    }
    if permanent && policy == "deny" {
        return Some(("permanentDeleteDenied", "permanent delete is turned off by administrator policy"));
    }
    None
}

/// Whether any `Email/set` call in the request destroys messages, and whether one is marked permanent.
fn scan(request: &mut Value) -> (bool, bool) {
    let (mut destroys, mut permanent) = (false, false);
    if let Some(calls) = request.get_mut("methodCalls").and_then(Value::as_array_mut) {
        for call in calls.iter_mut() {
            if call.get(0).and_then(Value::as_str) != Some("Email/set") {
                continue;
            }
            let Some(args) = call.get_mut(1).and_then(Value::as_object_mut) else {
                continue;
            };
            if args.get("destroy").and_then(Value::as_array).is_some_and(|d| !d.is_empty()) {
                destroys = true;
                if args.get(PERMANENT_MARKER).and_then(Value::as_bool) == Some(true) {
                    permanent = true;
                }
            }
            args.remove(PERMANENT_MARKER);
        }
    }
    (destroys, permanent)
}

/// Whether any call in the request is `Mailbox/stats`.
fn asks_for_stats(request: &Value) -> bool {
    request
        .get("methodCalls")
        .and_then(Value::as_array)
        .is_some_and(|calls| calls.iter().any(|c| c.get(0).and_then(Value::as_str) == Some("Mailbox/stats")))
}

pub(crate) async fn admit(state: &AppState, session: &mw_store::Session, request: &mut Value) -> Result<(), Response> {
    if asks_for_stats(request) {
        let mode = crate::settings_routes::effective_for_session(state, session, "properties.folder.sizeCompute")
            .await
            .ok()
            .flatten()
            .and_then(|e| e.value.as_str().map(str::to_string))
            .unwrap_or_default();
        if mode == "off" {
            return Err((
                StatusCode::FORBIDDEN,
                axum::Json(json!({"code": "sizeComputeOff", "error": "folder sizes are turned off by administrator policy"})),
            )
                .into_response());
        }
    }
    let (destroys, permanent) = scan(request);
    if !destroys {
        return Ok(());
    }
    let held = crate::user_retention::is_held(&state.store, &session.account_id).await;
    let restricted = matches!(
        state.store.restriction_active(&session.account_id, chrono::Utc::now().timestamp()).await,
        Ok(Some(_))
    );
    let policy = if permanent {
        crate::settings_routes::effective_for_session(state, session, "properties.message.permanentDelete")
            .await
            .ok()
            .flatten()
            .and_then(|e| e.value.as_str().map(str::to_string))
            .unwrap_or_default()
    } else {
        String::new()
    };
    match decide(held, restricted, permanent, &policy) {
        None => Ok(()),
        Some((code, error)) => Err((StatusCode::FORBIDDEN, axum::Json(json!({"code": code, "error": error}))).into_response()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hold_or_restriction_refuses_every_destroy() {
        assert_eq!(decide(true, false, false, "allow").map(|d| d.0), Some("legalHold"));
        assert_eq!(decide(false, true, false, "allow").map(|d| d.0), Some("processingRestricted"));
        assert_eq!(decide(true, true, true, "deny").map(|d| d.0), Some("legalHold"));
    }

    #[test]
    fn deny_policy_only_refuses_the_marked_permanent_delete() {
        assert_eq!(decide(false, false, true, "deny").map(|d| d.0), Some("permanentDeleteDenied"));
        assert!(decide(false, false, false, "deny").is_none(), "emptying the trash is not the dialog's delete");
        assert!(decide(false, false, true, "confirm-twice").is_none());
        assert!(decide(false, false, true, "allow").is_none());
    }

    #[test]
    fn stats_calls_are_found() {
        assert!(asks_for_stats(&json!({"methodCalls": [["Mailbox/get", {}, "a"], ["Mailbox/stats", {}, "b"]]})));
        assert!(!asks_for_stats(&json!({"methodCalls": [["Mailbox/get", {}, "a"]]})));
        assert!(!asks_for_stats(&json!({})));
    }

    #[test]
    fn the_marker_is_stripped_and_detected() {
        let mut request = json!({"methodCalls": [["Email/set", {"accountId": "a", "destroy": ["e1"], "mailwomanPermanent": true}, "s"]]});
        assert_eq!(scan(&mut request), (true, true));
        assert!(request["methodCalls"][0][1].get(PERMANENT_MARKER).is_none());
        let mut plain = json!({"methodCalls": [["Email/set", {"accountId": "a", "update": {}}, "s"]]});
        assert_eq!(scan(&mut plain), (false, false));
    }
}
