//! Conservative admission for receipt-controlled upstream submissions.
//! Existing upstream drafts cannot be inspected atomically with their send.
//! Only explicit, ordered composition in this request is admitted; opaque or
//! mutable references fail closed. This is not an upstream queue recall API.
use serde_json::Value;
use std::collections::BTreeMap;

pub(super) fn has_submission(request: &Value) -> bool {
    request["methodCalls"]
        .as_array()
        .is_some_and(|calls| calls.iter().any(|call| call[0] == "EmailSubmission/set"))
}

pub(super) fn has_email_write(request: &Value) -> bool {
    request["methodCalls"]
        .as_array()
        .is_some_and(|calls| calls.iter().any(|call| call[0] == "Email/set"))
}

/// SMTP DSN is requested by envelope parameters, not by message body headers.
pub(super) fn allows_without_dsn(request: &Value, account: &str) -> bool {
    let Some(calls) = request["methodCalls"].as_array() else {
        return false;
    };
    if calls.len() > 64 {
        return false;
    }
    for call in calls.iter().filter(|call| call[0] == "EmailSubmission/set") {
        let Some(args) = call[1].as_object() else {
            return false;
        };
        if args.keys().any(|key| key.starts_with('#'))
            || args
                .get("accountId")
                .is_some_and(|id| id.as_str() != Some(account))
        {
            return false;
        }
        if let Some(updates) = args.get("update").filter(|value| !value.is_null()) {
            let Some(updates) = updates.as_object() else {
                return false;
            };
            if updates.values().any(|patch| {
                patch.as_object().is_none_or(|p| {
                    p.len() != 1 || p.get("undoStatus").and_then(Value::as_str) != Some("canceled")
                })
            }) {
                return false;
            }
        }
        if let Some(creates) = args.get("create").filter(|value| !value.is_null()) {
            let Some(creates) = creates.as_object() else {
                return false;
            };
            if creates.len() > 32 {
                return false;
            }
            for submission in creates.values() {
                let Some(submission) = submission.as_object() else {
                    return false;
                };
                if submission.keys().any(|key| key.starts_with('#')) {
                    return false;
                }
                let Some(envelope) = submission.get("envelope").filter(|value| !value.is_null())
                else {
                    continue;
                };
                let Some(envelope) = envelope.as_object() else {
                    return false;
                };
                if envelope.keys().any(|key| key.starts_with('#')) {
                    return false;
                }
                let mut addresses = Vec::new();
                if let Some(sender) = envelope.get("mailFrom") {
                    addresses.push(sender);
                }
                if let Some(recipients) = envelope.get("rcptTo") {
                    let Some(recipients) = recipients.as_array() else {
                        return false;
                    };
                    if recipients.len() > 1000 {
                        return false;
                    }
                    addresses.extend(recipients);
                }
                for address in addresses {
                    let Some(address) = address.as_object() else {
                        return false;
                    };
                    if address.keys().any(|key| key.starts_with('#')) {
                        return false;
                    }
                    if let Some(parameters) =
                        address.get("parameters").filter(|value| !value.is_null())
                    {
                        let Some(parameters) = parameters.as_object() else {
                            return false;
                        };
                        if parameters.keys().any(|key| {
                            matches!(
                                key.to_ascii_uppercase().as_str(),
                                "RET" | "ENVID" | "NOTIFY" | "ORCPT"
                            )
                        }) {
                            return false;
                        }
                    }
                }
            }
        }
    }
    true
}

/// Prevent another request through this gateway from replacing a checked
/// composition with a report while the upstream processes the send batch.
pub(super) fn allows_email_writes(request: &Value, account: &str) -> bool {
    let Some(calls) = request["methodCalls"].as_array() else {
        return false;
    };
    if calls.len() > 64 {
        return false;
    }
    for call in calls.iter().filter(|call| call[0] == "Email/set") {
        let Some(args) = call[1].as_object() else {
            return false;
        };
        if args
            .get("accountId")
            .is_some_and(|id| id.as_str() != Some(account))
            || args.keys().any(|key| key.starts_with('#'))
        {
            return false;
        }
        for field in ["create", "update"] {
            if let Some(values) = args.get(field).filter(|value| !value.is_null()) {
                let Some(values) = values.as_object() else {
                    return false;
                };
                if values.len() > 1000 {
                    return false;
                }
                for message in values.values() {
                    let Some(object) = message.as_object() else {
                        return false;
                    };
                    let mut mime = serde_json::Map::new();
                    for (key, value) in object {
                        if field == "update"
                            && (key.starts_with("keywords/")
                                || key.starts_with("mailboxIds/")
                                || key.starts_with("bodyValues/"))
                        {
                            continue;
                        }
                        mime.insert(key.clone(), value.clone());
                    }
                    if !ordinary_composition(&Value::Object(mime)) {
                        return false;
                    }
                }
            }
        }
    }
    true
}

/// Return the exact proposed message objects, before any upstream mutation.
/// Bound work even when a caller bypasses the normal compose UI.
pub(crate) fn proposed_messages<'a>(
    request: &'a Value,
    account: &str,
) -> Result<Vec<&'a Value>, ()> {
    let calls = request["methodCalls"].as_array().ok_or(())?;
    if calls.len() > 64 || request.get("createdIds").is_some_and(|v| !v.is_null()) {
        return Err(());
    }
    let mut drafts = BTreeMap::new();
    let mut messages = Vec::new();
    for call in calls {
        let method = call[0].as_str().ok_or(())?;
        let args = call[1].as_object().ok_or(())?;
        if !matches!(method, "Email/set" | "EmailSubmission/set") {
            // Other mutation families share JMAP's creation-id namespace.
            // Do not allow an import/copy to replace the validated reference.
            if method.ends_with("/get")
                || method.ends_with("/query")
                || method.ends_with("/queryChanges")
                || method.ends_with("/changes")
            {
                continue;
            }
            return Err(());
        }
        if args
            .get("accountId")
            .is_some_and(|id| id.as_str() != Some(account))
        {
            return Err(());
        }
        if args.keys().any(|key| key.starts_with('#')) {
            return Err(());
        }
        if method == "Email/set" {
            // An update may target a creation reference; do not guess patch order
            // or MIME implications. Ordinary draft creation has no updates.
            if args.get("update").is_some_and(|v| !v.is_null()) {
                return Err(());
            }
            if let Some(created) = args.get("create").filter(|v| !v.is_null()) {
                for (id, message) in created.as_object().ok_or(())? {
                    if drafts.len() >= 32 || drafts.insert(id.as_str(), message).is_some() {
                        return Err(());
                    }
                }
            }
            if args.get("destroy").is_some_and(|v| !v.is_null()) {
                return Err(());
            }
        } else {
            if let Some(updates) = args.get("onSuccessUpdateEmail").filter(|v| !v.is_null()) {
                for patch in updates.as_object().ok_or(())?.values() {
                    if patch.as_object().ok_or(())?.keys().any(|key| {
                        !matches!(key.as_str(), "keywords" | "mailboxIds")
                            && !key.starts_with("keywords/")
                            && !key.starts_with("mailboxIds/")
                    }) {
                        return Err(());
                    }
                }
            }
            if let Some(updates) = args.get("update").filter(|v| !v.is_null()) {
                for patch in updates.as_object().ok_or(())?.values() {
                    let patch = patch.as_object().ok_or(())?;
                    if patch.len() != 1
                        || patch.get("undoStatus").and_then(Value::as_str) != Some("canceled")
                    {
                        return Err(());
                    }
                }
            }
            if let Some(created) = args.get("create").filter(|v| !v.is_null()) {
                for submission in created.as_object().ok_or(())?.values() {
                    let submission = submission.as_object().ok_or(())?;
                    if submission.keys().any(|key| key.starts_with('#')) {
                        return Err(());
                    }
                    // Deferred upstream work cannot be rechecked at its later
                    // transmission boundary, unlike the native engine queue.
                    if submission.get("sendAt").is_some_and(|v| !v.is_null())
                        || submission
                            .get("mailwomanHoldSeconds")
                            .is_some_and(|v| v.as_u64() != Some(0))
                    {
                        return Err(());
                    }
                    let reference = submission
                        .get("emailId")
                        .and_then(Value::as_str)
                        .ok_or(())?;
                    let id = reference.strip_prefix('#').ok_or(())?;
                    let message = drafts.get(id).copied().ok_or(())?;
                    if messages.len() >= 32 {
                        return Err(());
                    }
                    messages.push(message);
                }
            }
        }
    }
    Ok(messages)
}

pub(super) fn allows(request: &Value, account: &str) -> bool {
    proposed_messages(request, account)
        .is_ok_and(|messages| messages.into_iter().all(ordinary_composition))
}

fn ordinary_composition(message: &Value) -> bool {
    let Some(object) = message.as_object() else {
        return false;
    };
    // No opaque root blob or custom header can override generated MIME.
    if object.keys().any(|key| {
        !matches!(
            key.as_str(),
            "mailboxIds"
                | "keywords"
                | "from"
                | "sender"
                | "to"
                | "cc"
                | "bcc"
                | "replyTo"
                | "subject"
                | "sentAt"
                | "messageId"
                | "inReplyTo"
                | "references"
                | "textBody"
                | "htmlBody"
                | "bodyValues"
                | "bodyStructure"
                | "attachments"
                | "mailwomanRequestReadReceipt"
        )
    }) {
        return false;
    }
    let mut remaining = 2048;
    if let Some(root) = object.get("bodyStructure") {
        if ["textBody", "htmlBody", "attachments"]
            .iter()
            .any(|key| object.contains_key(*key))
        {
            return false;
        }
        return ordinary_part(root, 0, &mut remaining);
    }
    let mut ordinary_body = false;
    for (key, expected) in [("textBody", "text/plain"), ("htmlBody", "text/html")] {
        if let Some(parts) = object.get(key) {
            let Some(parts) = parts.as_array() else {
                return false;
            };
            if parts.len() != 1 {
                return false;
            }
            if !parts.iter().all(|part| {
                part["type"]
                    .as_str()
                    .is_some_and(|kind| kind.eq_ignore_ascii_case(expected))
                    && ordinary_part(part, 0, &mut remaining)
            }) {
                return false;
            }
            ordinary_body = true;
        }
    }
    // RFC8621 does not guarantee the root type of an attachment-only create.
    // Require an explicit ordinary body rather than assuming multipart/mixed.
    object.get("attachments").is_none_or(|parts| {
        parts
            .as_array()
            .is_some_and(|v| v.len() <= 256 && (v.is_empty() || ordinary_body))
    })
}

fn ordinary_part(part: &Value, depth: usize, remaining: &mut usize) -> bool {
    if depth > 32 || *remaining == 0 {
        return false;
    }
    *remaining -= 1;
    let Some(object) = part.as_object() else {
        return false;
    };
    if object
        .keys()
        .any(|key| key.to_ascii_lowercase().starts_with("header") || key.starts_with('#'))
    {
        return false;
    }
    let Some(kind) = object.get("type").and_then(Value::as_str) else {
        return false;
    };
    match kind.to_ascii_lowercase().as_str() {
        "text/plain" | "text/html" => !object.contains_key("subParts"),
        "multipart/mixed" | "multipart/alternative" | "multipart/related" => object
            .get("subParts")
            .and_then(Value::as_array)
            .is_some_and(|parts| {
                !parts.is_empty()
                    && parts
                        .iter()
                        .all(|part| ordinary_part(part, depth + 1, remaining))
            }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn compose(message: Value) -> Value {
        json!({"methodCalls":[["Email/set",{"accountId":"a","create":{"draft":message}},"e"],["EmailSubmission/set",{"accountId":"a","create":{"send":{"emailId":"#draft"}}},"s"]]})
    }
    #[test]
    fn ordinary_compose_allowed_but_raw_reports_and_header_overrides_refused() {
        assert!(allows(
            &compose(
                json!({"from":[{"email":"me@example.org"}],"htmlBody":[{"partId":"body","type":"text/html"}],"bodyValues":{"body":{"value":"hello"}}})
            ),
            "a"
        ));
        for message in [
            json!({"blobId":"raw"}),
            json!({"bodyStructure":{"type":"multipart/report","subParts":[]}}),
            json!({"header:Content-Type:asRaw":"message/disposition-notification"}),
            json!({"textBody":[{"type":"message/disposition-notification"}]}),
        ] {
            assert!(!allows(&compose(message), "a"));
        }
    }
    #[test]
    fn mutable_opaque_cross_account_and_deferred_submissions_fail_closed() {
        let base = compose(json!({"textBody":[{"type":"text/plain","partId":"p"}]}));
        for change in [
            "existing",
            "foreign",
            "deferred",
            "patch",
            "forward-reference",
        ] {
            let mut request = base.clone();
            match change {
                "existing" => {
                    request["methodCalls"][1][1]["create"]["send"]["emailId"] = json!("old")
                }
                "foreign" => request["methodCalls"][0][1]["accountId"] = json!("other"),
                "deferred" => {
                    request["methodCalls"][1][1]["create"]["send"]["sendAt"] =
                        json!("2030-01-01T00:00:00Z")
                }
                "patch" => {
                    request["methodCalls"][0][1]["update"] = json!({"#draft":{"bodyStructure":{"type":"message/disposition-notification"}}})
                }
                _ => request["methodCalls"].as_array_mut().unwrap().swap(0, 1),
            }
            assert!(!allows(&request, "a"), "{change}");
        }
        assert!(allows(
            &json!({"methodCalls":[["EmailSubmission/set",{"update":{"old":{"undoStatus":"canceled"}}},"s"]]}),
            "a"
        ));
    }

    #[test]
    fn creation_id_aliases_and_other_mutations_cannot_replace_inspected_draft() {
        let mut request = compose(json!({"textBody":[{"partId":"p","type":"text/plain"}]}));
        request["createdIds"] = json!({"draft":"existing-report"});
        assert!(!allows(&request, "a"));
        request.as_object_mut().unwrap().remove("createdIds");
        request["methodCalls"].as_array_mut().unwrap().insert(1,json!(["Email/import",{"accountId":"a","emails":{"draft":{"blobId":"raw-report"}}},"i"]));
        assert!(!allows(&request, "a"));
    }

    #[test]
    fn parallel_gateway_edit_cannot_replace_checked_mime() {
        for patch in [
            json!({"bodyStructure":{"type":"multipart/report"}}),
            json!({"bodyStructure/type":"message/disposition-notification"}),
            json!({"header:Content-Type:asRaw":"multipart/report"}),
            json!({"blobId":"report"}),
        ] {
            assert!(!allows_email_writes(
                &json!({"methodCalls":[["Email/set",{"accountId":"a","update":{"id":patch}},"e"]]}),
                "a"
            ));
        }
        assert!(allows_email_writes(
            &json!({"methodCalls":[["Email/set",{"accountId":"a","update":{"id":{"keywords/$seen":true,"subject":"safe","htmlBody":[{"type":"text/html","partId":"p"}],"bodyValues":{"p":{"value":"ordinary edit"}}}}},"e"]]}),
            "a"
        ));
    }

    #[test]
    fn submission_success_patch_cannot_change_mime_before_second_send() {
        let mut request = compose(json!({"htmlBody":[{"type":"text/html","partId":"p"}]}));
        request["methodCalls"][1][1]["onSuccessUpdateEmail"] =
            json!({"#send":{"bodyStructure":{"type":"message/disposition-notification"}}});
        assert!(!allows(&request, "a"));
        request["methodCalls"][1][1]["onSuccessUpdateEmail"] =
            json!({"#send":{"mailboxIds":{"sent":true},"keywords/$draft":null}});
        assert!(allows(&request, "a"));
    }

    #[test]
    fn attachment_only_and_conflicting_mime_cannot_choose_a_report_root() {
        let attachments = json!([{"blobId":"report","type":"multipart/report"}]);
        assert!(!allows(&compose(json!({"attachments":attachments})), "a"));
        assert!(!allows(
            &compose(json!({"textBody":[],"attachments":attachments})),
            "a"
        ));
        assert!(!allows(
            &compose(
                json!({"bodyStructure":{"type":"text/plain","partId":"p"},"attachments":attachments})
            ),
            "a"
        ));
        assert!(allows(
            &compose(
                json!({"textBody":[{"type":"text/plain","partId":"p"}],"bodyValues":{"p":{"value":"forwarded report"}},"attachments":attachments})
            ),
            "a"
        ));
    }

    #[test]
    fn dsn_authority_checks_envelope_parameters_and_opaque_release() {
        let base = compose(json!({"textBody":[{"type":"text/plain","partId":"p"}]}));
        assert!(allows_without_dsn(&base, "a"));
        for parameter in ["RET", "envid", "Notify", "ORCPT"] {
            let mut request = base.clone();
            request["methodCalls"][1][1]["create"]["send"]["envelope"] = json!({"mailFrom":{"email":"a@example.org","parameters":{parameter:"x"}},"rcptTo":[]});
            assert!(!allows_without_dsn(&request, "a"), "{parameter}");
        }
        assert!(!allows_without_dsn(
            &json!({"methodCalls":[["EmailSubmission/set",{"update":{"old":{"sendAt":null}}},"s"]]}),
            "a"
        ));
        assert!(allows_without_dsn(
            &json!({"methodCalls":[["EmailSubmission/set",{"update":{"old":{"undoStatus":"canceled"}}},"s"]]}),
            "a"
        ));
        let mut request = base;
        request["methodCalls"][1][1]["create"]["send"]["#envelope"] =
            json!({"resultOf":"other","name":"Email/get","path":"/list/0"});
        assert!(!allows_without_dsn(&request, "a"));
    }
}
