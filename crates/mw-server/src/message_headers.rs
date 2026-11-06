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
        "in-reply-to" | "references" | "message-id" => Some("compose.headers.allowThreadingEdit"),
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

/// Lower-cased addresses found in a JMAP `replyTo` value (an address array) or
/// in a stored identity `reply_to` string.
pub(crate) fn addresses_in(value: &Value) -> Vec<String> {
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|a| a.get("email").and_then(Value::as_str))
            .map(|e| e.trim().to_ascii_lowercase())
            .collect(),
        Value::String(text) => text
            .split(|c: char| c == ',' || c == ';' || c == '<' || c == '>' || c.is_whitespace())
            .filter(|p| p.contains('@'))
            .map(|p| p.trim_matches('"').to_ascii_lowercase())
            .collect(),
        _ => Vec::new(),
    }
}

/// Whether a Reply-To the user wrote is acceptable when the administrator turned
/// the override off: every address must be one the sending identity already
/// carries. An empty Reply-To is always fine.
pub(crate) fn reply_to_acceptable(requested: &[String], identity_reply_to: &[String]) -> bool {
    requested.iter().all(|r| identity_reply_to.contains(r))
}

/// Enforce `compose.sentFolder.skipCopy`: every submission in the call files no
/// copy (the message is destroyed once sent) and cannot move it elsewhere.
pub(crate) fn force_skip_copy(args: &mut Value) {
    let Some(object) = args.as_object_mut() else {
        return;
    };
    let keys: Vec<String> = object
        .get("create")
        .and_then(Value::as_object)
        .map(|c| c.keys().map(|k| format!("#{k}")).collect())
        .unwrap_or_default();
    if keys.is_empty() {
        return;
    }
    object.remove("onSuccessUpdateEmail");
    object.insert("onSuccessDestroyEmail".into(), json!(keys));
}

/// Apply `mail.priority.sendHeaders` to one message: the priority the user chose
/// (`Importance`, with `X-Priority` beside it) is written under the names the
/// administrator lists, and `X-MSMail-Priority` is added when listed. The
/// engine writes `Importance` and `X-Priority` as a pair (it refuses one
/// alone), so they stay together when either is listed and go together when
/// neither is.
pub(crate) fn apply_priority_headers(object: &mut serde_json::Map<String, Value>, send: &[String]) {
    let find = |object: &serde_json::Map<String, Value>, name: &str| {
        object
            .keys()
            .find(|k| header_name(k).is_some_and(|n| n.eq_ignore_ascii_case(name)))
            .cloned()
    };
    let Some(importance_key) = find(object, "Importance") else {
        return;
    };
    let level = object
        .get(&importance_key)
        .and_then(Value::as_str)
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    let listed = |name: &str| send.iter().any(|s| s.eq_ignore_ascii_case(name));
    if !listed("Importance") && !listed("X-Priority") {
        object.remove(&importance_key);
        if let Some(k) = find(object, "X-Priority") {
            object.remove(&k);
        }
    }
    if let Some(k) = find(object, "X-MSMail-Priority") {
        object.remove(&k);
    }
    if listed("X-MSMail-Priority") {
        let word = match level.as_str() {
            "high" => "High",
            "low" => "Low",
            _ => "Normal",
        };
        object.insert("header:X-MSMail-Priority:asText".into(), Value::String(word.into()));
    }
}

fn refuse_reply_to() -> Response {
    (
        StatusCode::FORBIDDEN,
        axum::Json(json!({
            "code": "replyToNotAllowed",
            "error": "changing the Reply-To address is not allowed by administrator policy",
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
    let mut skip_copy: Option<bool> = None;
    let mut priority_headers: Option<Vec<String>> = None;
    let mut override_ok: Option<bool> = None;
    let mut identity_reply_to: Option<Vec<(String, Vec<String>)>> = None;
    for call in calls.iter_mut() {
        if call.get(0).and_then(Value::as_str) == Some("EmailSubmission/set")
            && call.get(1).and_then(|a| a.get("create")).is_some()
        {
            if skip_copy.is_none() {
                skip_copy = Some(flag(state, session, "compose.sentFolder.skipCopy").await);
            }
            if skip_copy == Some(true) {
                force_skip_copy(&mut call[1]);
            }
        }
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
            if let Some(requested) = object.get("replyTo").map(addresses_in).filter(|r| !r.is_empty()) {
                if override_ok.is_none() {
                    override_ok = Some(flag(state, session, "compose.replyTo.allowOverride").await);
                }
                if override_ok == Some(false) {
                    if identity_reply_to.is_none() {
                        let rows = state
                            .store
                            .list_native_personal_identities(&session.account_id)
                            .await
                            .unwrap_or_default();
                        identity_reply_to = Some(
                            rows.into_iter()
                                .map(|r| {
                                    let rt = r.reply_to.map(Value::String).map(|v| addresses_in(&v)).unwrap_or_default();
                                    (r.email.to_ascii_lowercase(), rt)
                                })
                                .collect(),
                        );
                    }
                    let from = object
                        .get("from")
                        .map(addresses_in)
                        .and_then(|f| f.into_iter().next())
                        .unwrap_or_default();
                    let allowed: Vec<String> = identity_reply_to
                        .as_deref()
                        .unwrap_or_default()
                        .iter()
                        .filter(|(email, _)| *email == from)
                        .flat_map(|(_, rt)| rt.clone())
                        .collect();
                    if !reply_to_acceptable(&requested, &allowed) {
                        return Err(refuse_reply_to());
                    }
                }
            }
            if object.keys().any(|k| header_name(k).is_some_and(|n| n.eq_ignore_ascii_case("Importance"))) {
                if priority_headers.is_none() {
                    priority_headers = Some(
                        crate::settings_routes::effective_for_session(state, session, "mail.priority.sendHeaders")
                            .await
                            .ok()
                            .flatten()
                            .and_then(|e| e.value.as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect::<Vec<_>>()))
                            .unwrap_or_else(|| vec!["Importance".into(), "X-Priority".into(), "X-MSMail-Priority".into()]),
                    );
                }
                apply_priority_headers(object, priority_headers.as_deref().unwrap_or_default());
            }
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
    fn reply_to_override_needs_the_identity_value() {
        let identity = addresses_in(&json!("Support <help@example.org>, other@example.org"));
        assert_eq!(identity, vec!["help@example.org", "other@example.org"]);
        let same = addresses_in(&json!([{"email": "Help@Example.org"}]));
        assert!(reply_to_acceptable(&same, &identity));
        let other = addresses_in(&json!([{"email": "evil@example.net"}]));
        assert!(!reply_to_acceptable(&other, &identity));
        assert!(reply_to_acceptable(&[], &[]));
        assert!(!reply_to_acceptable(&other, &[]));
    }

    #[test]
    fn skip_copy_replaces_the_sent_copy_move() {
        let mut args = json!({
            "accountId": "a",
            "create": {"send": {"emailId": "#draft"}},
            "onSuccessUpdateEmail": {"#send": {"mailboxIds": {"m1": true}}}
        });
        force_skip_copy(&mut args);
        assert!(args.get("onSuccessUpdateEmail").is_none());
        assert_eq!(args["onSuccessDestroyEmail"], json!(["#send"]));
        // A call that creates no submission is left alone.
        let mut none = json!({"accountId": "a"});
        force_skip_copy(&mut none);
        assert!(none.get("onSuccessDestroyEmail").is_none());
    }

    #[test]
    fn priority_headers_follow_the_administrator_list() {
        let spec = || {
            json!({"header:Importance:asText": "high", "header:X-Priority:asText": "1", "subject": "s"})
                .as_object()
                .cloned()
                .unwrap()
        };
        let all: Vec<String> = ["Importance", "X-Priority", "X-MSMail-Priority"].map(String::from).to_vec();
        let mut full = spec();
        apply_priority_headers(&mut full, &all);
        assert_eq!(full["header:X-MSMail-Priority:asText"], "High");
        assert_eq!(full["header:Importance:asText"], "high");
        let mut ms_only = spec();
        apply_priority_headers(&mut ms_only, &["X-MSMail-Priority".to_string()]);
        assert!(ms_only.get("header:Importance:asText").is_none());
        assert!(ms_only.get("header:X-Priority:asText").is_none());
        assert_eq!(ms_only["header:X-MSMail-Priority:asText"], "High");
        let mut imp = spec();
        apply_priority_headers(&mut imp, &["Importance".to_string()]);
        assert!(imp.get("header:X-MSMail-Priority:asText").is_none());
        assert_eq!(imp["header:Importance:asText"], "high");
        let mut none = spec();
        apply_priority_headers(&mut none, &[]);
        assert!(none.keys().all(|k| !k.starts_with("header:")));
        // A message without a priority is left alone.
        let mut plain = json!({"subject": "s"}).as_object().cloned().unwrap();
        apply_priority_headers(&mut plain, &all);
        assert_eq!(plain.len(), 1);
    }

    #[test]
    fn threading_headers_are_gated() {
        assert_eq!(gate_for("In-Reply-To"), Some("compose.headers.allowThreadingEdit"));
        assert_eq!(gate_for("references"), Some("compose.headers.allowThreadingEdit"));
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
