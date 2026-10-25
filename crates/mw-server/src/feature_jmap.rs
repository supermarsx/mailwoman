//! Reject a disabled feature before any member of a JMAP batch executes.
//! Applies equally to native engine requests and upstream proxy requests.
use crate::{AppState, settings_routes::UserSubject};
use axum::response::{IntoResponse, Response};
use serde_json::Value;
use std::collections::BTreeSet;

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
    if required.is_empty() {
        return Ok(());
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
