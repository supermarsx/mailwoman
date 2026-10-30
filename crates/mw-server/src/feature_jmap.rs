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

/// The first method call of a batch that names an account other than the
/// session's (t87 D10): `(call id, method, argument, the account it named)`.
/// The engine serves exactly one account per session, so every first-party
/// client sends `session.primaryAccounts[...]`; a different `accountId` (or
/// `fromAccountId`) is a stale cache or a forged request. Before t87 the engine
/// ignored the argument and quietly applied the call to the session account.
/// `Core/echo` carries arbitrary data and is exempt.
fn foreign_account<'a>(
    request: &'a Value,
    account_id: &str,
) -> Option<(&'a str, &'a str, &'static str, String)> {
    for call in request
        .get("methodCalls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let name = call.get(0).and_then(Value::as_str).unwrap_or_default();
        if name == "Core/echo" {
            continue;
        }
        let id = call.get(2).and_then(Value::as_str).unwrap_or_default();
        let Some(args) = call.get(1).and_then(Value::as_object) else {
            continue;
        };
        for key in ["accountId", "fromAccountId"] {
            match args.get(key) {
                None | Some(Value::Null) => {}
                Some(Value::String(named)) if named == account_id => {}
                Some(Value::String(named)) => return Some((id, name, key, named.clone())),
                Some(other) => return Some((id, name, key, other.to_string())),
            }
        }
    }
    None
}

/// Refuse the whole batch, before any member runs, when a call names another
/// account (t87 D10, every session). Native engine sessions only: an upstream
/// JMAP server behind the proxy has its own account ids and enforces them.
fn refuse_foreign_account(
    state: &AppState,
    session: &mw_store::Session,
    request: &Value,
) -> Result<(), Response> {
    if state.engine.is_none() {
        return Ok(());
    }
    let Some((call, method, argument, named)) = foreign_account(request, &session.account_id)
    else {
        return Ok(());
    };
    Err((
        axum::http::StatusCode::BAD_REQUEST,
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        axum::Json(serde_json::json!({
            "type": "urn:mailwoman:jmap:error:accountNotFound",
            "status": 400,
            "code": "accountNotFound",
            "detail": format!(
                "Method call {call:?} ({method}) names {argument} {named:?}, but this session acts for account {:?}. No call in this request was run.",
                session.account_id
            ),
            "callId": call,
            "method": method,
            "argument": argument,
            "accountId": named,
            "sessionAccountId": session.account_id,
        })),
    )
        .into_response())
}

pub(crate) async fn admit(
    state: &AppState,
    session: &mw_store::Session,
    request: &Value,
) -> Result<(), Response> {
    refuse_foreign_account(state, session, request)?;
    if state.engine.is_none() && requests_exact_sender_domain(request) {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({
                "code": "searchOperatorUnsupported",
                "error": "This mailbox does not support exact sender-domain search."
            })),
        )
            .into_response());
    }
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

fn requests_exact_sender_domain(request: &Value) -> bool {
    request
        .get("methodCalls")
        .and_then(Value::as_array)
        .is_some_and(|calls| {
            calls.iter().any(|call| {
                call.get(0).and_then(Value::as_str) == Some("Email/query")
                    && call
                        .get(1)
                        .and_then(|args| args.get("filter"))
                        .is_some_and(mw_engine::query::has_exact_sender_domain_filter)
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_domain_proxy_guard_detects_operators_without_matching_literal_text() {
        let query =
            |filter| serde_json::json!({"methodCalls":[["Email/query",{"filter":filter},"q"]]});
        assert!(requests_exact_sender_domain(&query(
            serde_json::json!({"text":"from-domain:example.org"})
        )));
        assert!(requests_exact_sender_domain(&query(
            serde_json::json!({"operator":"OR","conditions":[{"text":"-from-domain:example.org"}]})
        )));
        assert!(!requests_exact_sender_domain(&query(
            serde_json::json!({"text":"subject:\"from-domain:example.org\""})
        )));
    }
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
    #[test]
    fn t87_a_call_naming_another_account_is_found_and_echo_is_exempt() {
        let own = json!({"methodCalls":[
            ["Mailbox/get",{"accountId":"a1"},"0"],
            ["Core/echo",{"accountId":"other"},"1"],
            ["Email/query",{},"2"],
            ["Email/copy",{"accountId":"a1","fromAccountId":"a1"},"3"],
            ["Calendar/get",{"accountId":null},"4"]
        ]});
        assert!(foreign_account(&own, "a1").is_none());
        let stale = json!({"methodCalls":[
            ["Mailbox/get",{"accountId":"a1"},"0"],
            ["CalendarEvent/set",{"accountId":"parent","create":{}},"c7"]
        ]});
        assert_eq!(
            foreign_account(&stale, "a1"),
            Some(("c7", "CalendarEvent/set", "accountId", "parent".to_owned()))
        );
        let copy =
            json!({"methodCalls":[["Email/copy",{"accountId":"a1","fromAccountId":"b2"},"x"]]});
        assert_eq!(
            foreign_account(&copy, "a1"),
            Some(("x", "Email/copy", "fromAccountId", "b2".to_owned()))
        );
        let typed = json!({"methodCalls":[["Email/get",{"accountId":7},"n"]]});
        assert_eq!(
            foreign_account(&typed, "a1"),
            Some(("n", "Email/get", "accountId", "7".to_owned()))
        );
    }
}
