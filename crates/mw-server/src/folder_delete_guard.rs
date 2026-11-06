//! A folder delete (`Mailbox/set` with `destroy`) destroys the messages inside it,
//! so it is refused for the two account states that already stop a message being
//! destroyed: a legal hold (`dsr.legalHold.subjects`) and an active processing
//! restriction. The answer is a 403 with a stable `code` and `scope: "folder"` so
//! the client can name the folders it could not delete.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use crate::AppState;

/// Why a folder delete is refused, as `(code, message)`.
pub(crate) fn decide(held: bool, restricted: bool) -> Option<(&'static str, &'static str)> {
    if held {
        return Some(("legalHold", "this account is under a legal hold; folders cannot be deleted"));
    }
    if restricted {
        return Some((
            "processingRestricted",
            "processing of this account is restricted; folders cannot be deleted",
        ));
    }
    None
}

/// Whether any `Mailbox/set` call in the request destroys folders.
pub(crate) fn destroys_folders(request: &Value) -> bool {
    request
        .get("methodCalls")
        .and_then(Value::as_array)
        .is_some_and(|calls| {
            calls.iter().any(|call| {
                call.get(0).and_then(Value::as_str) == Some("Mailbox/set")
                    && call
                        .get(1)
                        .and_then(|a| a.get("destroy"))
                        .and_then(Value::as_array)
                        .is_some_and(|d| !d.is_empty())
            })
        })
}

pub(crate) async fn admit(state: &AppState, session: &mw_store::Session, request: &Value) -> Result<(), Response> {
    if !destroys_folders(request) {
        return Ok(());
    }
    let held = crate::user_retention::is_held(&state.store, &session.account_id).await;
    let restricted = matches!(
        state.store.restriction_active(&session.account_id, chrono::Utc::now().timestamp()).await,
        Ok(Some(_))
    );
    match decide(held, restricted) {
        None => Ok(()),
        Some((code, error)) => Err((
            StatusCode::FORBIDDEN,
            axum::Json(json!({"code": code, "error": error, "scope": "folder"})),
        )
            .into_response()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hold_or_restriction_refuses_a_folder_delete() {
        assert_eq!(decide(true, false).map(|d| d.0), Some("legalHold"));
        assert_eq!(decide(false, true).map(|d| d.0), Some("processingRestricted"));
        assert_eq!(decide(true, true).map(|d| d.0), Some("legalHold"));
        assert!(decide(false, false).is_none());
        assert!(decide(true, false).unwrap().1.contains("folders"));
    }

    #[test]
    fn folder_destroys_are_found_and_other_calls_are_not() {
        assert!(destroys_folders(&json!({"methodCalls": [["Mailbox/set", {"accountId": "a", "destroy": ["m1"]}, "s"]]})));
        assert!(!destroys_folders(&json!({"methodCalls": [["Mailbox/set", {"accountId": "a", "create": {}, "destroy": []}, "s"]]})));
        assert!(!destroys_folders(&json!({"methodCalls": [["Email/set", {"destroy": ["e1"]}, "s"]]})));
        assert!(!destroys_folders(&json!({})));
    }
}
