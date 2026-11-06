//! Admission and forced headers for composed messages (Options > Message
//! options). A request that creates an Email with `header:<Name>:asText`
//! properties is checked against the administrator's `compose.headers.*`
//! settings before the engine sees it, and the domain's forced headers are
//! written into every message the same request submits.
//!
//! The engine keeps the floor (`mw_engine::jmap::optional_header_allowed`: fixed
//! names plus plain `X-` tokens, one control-free line each, at most 20); this is
//! the policy on top of it, which needs the session's settings.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use crate::AppState;

/// Headers whose use an administrator gates, and the setting that gates each.
fn gate_for(name: &str) -> Option<&'static str> {
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "auto-submitted" => Some("compose.headers.allowAutoSubmitted"),
        "return-receipt-to" => Some("compose.headers.allowReturnReceiptTo"),
        _ if lower.starts_with("x-")
            && !matches!(lower.as_str(), "x-priority" | "x-msmail-priority" | "x-mailer") =>
        {
            Some("compose.headers.allowCustom")
        }
        _ => None,
    }
}

/// The header name of a `header:<Name>:asText` property.
fn header_name(key: &str) -> Option<&str> {
    let rest = key.strip_prefix("header:")?;
    Some(rest.split_once(':').map_or(rest, |(name, _)| name))
}

/// Parse `compose.headers.forced` entries (`Name=value`), dropping any that name
/// a protected header or carry a line break.
pub(crate) fn parse_forced(list: &[String]) -> Vec<(String, String)> {
    list.iter()
        .filter_map(|line| {
            let (name, value) = line.split_once('=')?;
            let (name, value) = (name.trim(), value.trim());
            (!name.is_empty()
                && !value.is_empty()
                && value.chars().count() <= 998
                && !value.chars().any(char::is_control)
                && mw_engine::jmap::optional_header_allowed(name))
            .then(|| (name.to_string(), value.to_string()))
        })
        .collect()
}

fn refuse(code: &str, header: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        axum::Json(json!({
            "code": code,
            "header": header,
            "error": "this header is not allowed by administrator policy",
        })),
    )
        .into_response()
}

async fn flag(state: &AppState, session: &mw_store::Session, key: &str) -> bool {
    crate::settings_routes::effective_for_session(state, session, key)
        .await
        .ok()
        .flatten()
        .is_some_and(|e| e.value.as_bool() == Some(true))
}

/// Check the message headers of `request` and add the forced ones. `Ok` when the
/// request carries none (the common case costs one scan of the method calls).
pub(crate) async fn apply(
    state: &AppState,
    session: &mw_store::Session,
    request: &mut Value,
) -> Result<(), Response> {
    let Some(calls) = request.get_mut("methodCalls").and_then(Value::as_array_mut) else {
        return Ok(());
    };
    // Ids (creation keys) of the Emails that a submission in this request sends.
    let mut submitted: Vec<String> = Vec::new();
    for call in calls.iter() {
        if call.get(0).and_then(Value::as_str) == Some("EmailSubmission/set")
            && let Some(creates) = call.get(1).and_then(|a| a.get("create")).and_then(Value::as_object)
        {
            for create in creates.values() {
                if let Some(id) = create.get("emailId").and_then(Value::as_str).and_then(|s| s.strip_prefix('#')) {
                    submitted.push(id.to_string());
                }
            }
        }
    }
    let mut forced: Option<Vec<(String, String)>> = None;
    for call in calls.iter_mut() {
        if call.get(0).and_then(Value::as_str) != Some("Email/set") {
            continue;
        }
        let Some(creates) = call
            .get_mut(1)
            .and_then(|a| a.get_mut("create"))
            .and_then(Value::as_object_mut)
        else {
            continue;
        };
        for (id, spec) in creates.iter_mut() {
            let Some(object) = spec.as_object_mut() else {
                continue;
            };
            let names: Vec<String> = object
                .keys()
                .filter_map(|k| header_name(k).map(str::to_string))
                .collect();
            for name in &names {
                if let Some(gate) = gate_for(name)
                    && !flag(state, session, gate).await
                {
                    return Err(refuse("headerNotAllowed", name));
                }
            }
            if submitted.iter().any(|s| s == id) {
                if forced.is_none() {
                    let list = crate::settings_routes::effective_for_session(state, session, "compose.headers.forced")
                        .await
                        .ok()
                        .flatten()
                        .and_then(|e| e.value.as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect::<Vec<_>>()))
                        .unwrap_or_default();
                    forced = Some(parse_forced(&list));
                }
                for (name, value) in forced.as_deref().unwrap_or_default() {
                    // A forced header replaces whatever the user set under any spelling.
                    object.retain(|k, _| !header_name(k).is_some_and(|n| n.eq_ignore_ascii_case(name)));
                    object.insert(format!("header:{name}:asText"), Value::String(value.clone()));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forced_entries_drop_protected_names_and_line_breaks() {
        let list: Vec<String> = [
            "Sensitivity=company-confidential",
            "From=evil@example.org",
            "X-Policy=ok",
            "X-Bad=a\r\nBcc: z@example.org",
            "no equals sign",
            "X-Empty=",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(
            parse_forced(&list),
            vec![
                ("Sensitivity".to_string(), "company-confidential".to_string()),
                ("X-Policy".to_string(), "ok".to_string()),
            ]
        );
    }

    #[test]
    fn gates_cover_the_risky_headers_only() {
        assert_eq!(gate_for("Auto-Submitted"), Some("compose.headers.allowAutoSubmitted"));
        assert_eq!(gate_for("return-receipt-to"), Some("compose.headers.allowReturnReceiptTo"));
        assert_eq!(gate_for("X-Ticket"), Some("compose.headers.allowCustom"));
        assert_eq!(gate_for("X-MSMail-Priority"), None);
        assert_eq!(gate_for("Sensitivity"), None);
        assert_eq!(header_name("header:Sensitivity:asText"), Some("Sensitivity"));
        assert_eq!(header_name("subject"), None);
    }
}
