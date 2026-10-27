//! Reject a disabled feature before any member of a JMAP batch executes.
//! Applies equally to native engine requests and upstream proxy requests.
use crate::{AppState, settings_routes::UserSubject};
use axum::response::{IntoResponse, Response};
use serde_json::Value;
use std::collections::BTreeSet;

#[path = "receipt_proxy.rs"]
pub(crate) mod receipt_proxy;

fn required_features(request: &Value) -> BTreeSet<&'static str> {
    let mut required = BTreeSet::new();
    for call in request
        .get("methodCalls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some((family, operation)) = call
            .get(0)
            .and_then(Value::as_str)
            .and_then(|s| s.split_once('/'))
        else {
            continue;
        };
        // Email has no whole-family feature gate: ordinary read/copy calls
        // remain available, but import must obey the same policy as REST.
        if family == "Email" && operation == "import" {
            required.insert("import");
        }
        let feature = match family {
            "Calendar" | "CalendarEvent" => Some("calendar"),
            "AddressBook" | "ContactCard" | "ContactGroup" | "Contact" => Some("contacts"),
            "TaskList" | "Task" => Some("tasks"),
            "Notebook" | "Note" => Some("notes"),
            "MailRule" | "VacationResponse" => Some("rules"),
            _ => None,
        };
        if let Some(feature) = feature {
            required.insert(feature);
            if operation == "export" {
                required.insert("export");
            }
            if operation == "import" {
                required.insert("import");
            }
        }
    }
    required
}

pub(crate) async fn admit(
    state: &AppState,
    session: &mw_store::Session,
    request: &Value,
) -> Result<(), Response> {
    let required = required_features(request);
    let receipt_send = request
        .get("methodCalls")
        .and_then(Value::as_array)
        .is_some_and(|calls| {
            calls
                .iter()
                .any(|call| call.get(0).and_then(Value::as_str) == Some("MDN/send"))
        });
    let proxy_submission = state.engine.is_none() && receipt_proxy::has_submission(request);
    let proxy_email_write = state.engine.is_none() && receipt_proxy::has_email_write(request);
    if required.is_empty() && !receipt_send && !proxy_submission && !proxy_email_write {
        return Ok(());
    }
    if proxy_submission {
        crate::dlp::admit_proxy(state, request).await?;
        let dsn =
            crate::settings_routes::effective_for_session(state, session, "receipts.dsnAllowed")
                .await
                .map_err(|_| axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response())?
                .ok_or_else(|| axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response())?;
        if dsn.value.as_bool() != Some(true)
            && !receipt_proxy::allows_without_dsn(request, &session.account_id)
        {
            return Err((axum::http::StatusCode::FORBIDDEN,axum::Json(serde_json::json!({"error":"delivery status requests are disabled by administrator policy"}))).into_response());
        }
    }
    if receipt_send || proxy_submission || proxy_email_write {
        let policy =
            crate::settings_routes::effective_for_session(state, session, "mail.readReceipts")
                .await
                .map_err(|_| axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response())?
                .ok_or_else(|| axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response())?;
        if policy.locked()
            && !policy.value.as_str().is_some_and(|mode| {
                matches!(
                    mode,
                    "ask" | "always" | "contactsOnly" | "orgOnly" | "orgAlwaysAskOthers"
                )
            })
            && (receipt_send
                || !receipt_proxy::allows_email_writes(request, &session.account_id)
                || (proxy_submission && !receipt_proxy::allows(request, &session.account_id)))
        {
            return Err((axum::http::StatusCode::FORBIDDEN,axum::Json(serde_json::json!({"error":"read receipt policy requires a verifiable immediate composition"}))).into_response());
        }
    }
    let subject = UserSubject::resolved(&state.store, session)
        .await
        .map_err(|_| axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response())?;
    for feature in required {
        crate::user_router::require_feature(state, &subject, feature).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn email_import_requires_import_before_any_batch_member_runs() {
        let request = json!({"methodCalls":[["Email/set",{},"1"],["Email/import",{},"2"]]});
        assert_eq!(required_features(&request), BTreeSet::from(["import"]));
        let ordinary = json!({"methodCalls":[["Email/parse",{},"1"],["Email/copy",{},"2"],["Email/imported",{},"3"]]});
        assert!(required_features(&ordinary).is_empty());
    }
    #[test]
    fn all_pim_families_and_import_export_require_their_own_capabilities() {
        let request = json!({"methodCalls":[
            ["CalendarEvent/import",{},"1"],["ContactGroup/get",{},"2"],
            ["Task/export",{},"3"],["Note/set",{},"4"],["MailRule/set",{},"5"]
        ]});
        assert_eq!(
            required_features(&request),
            BTreeSet::from([
                "calendar", "contacts", "tasks", "notes", "rules", "import", "export"
            ])
        );
    }
    #[test]
    fn method_names_are_exact_and_raw_sieve_keeps_existing_authority() {
        let request = json!({"methodCalls":[["Email/get",{},"1"],["SieveScript/get",{},"2"],["Calendarish/set",{},"3"]]});
        assert!(required_features(&request).is_empty());
    }
}
