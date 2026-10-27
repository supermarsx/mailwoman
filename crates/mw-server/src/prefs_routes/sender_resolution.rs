//! A one-message sender lookup; never creates a persistent identity or grants authority.
use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Request {
    email: String,
}

pub(super) async fn resolve(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Request>,
) -> Response {
    let mut response = resolve_inner(&state, &headers, body).await;
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("private, no-store"),
    );
    response
}

async fn resolve_inner(state: &AppState, headers: &HeaderMap, body: Request) -> Response {
    let session = match session_of(state, headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    if headers
        .get("x-mailwoman-account")
        .is_some_and(|value| value.to_str().ok() != Some(session.account_id.as_str()))
    {
        return (StatusCode::CONFLICT, Json(json!({"code":"accountChanged"}))).into_response();
    }
    if let Err(response) = personal_storage::record_owner(state, &session, headers, true).await {
        return response;
    }
    if !valid_address(&body.email) {
        return bad_request("Enter one email address without a display name.");
    }
    match resolve_sender_identity(state, &session, &body.email).await {
        Ok(Some(identity)) => Json(json!({"email":body.email,"identityId":identity})).into_response(),
        Ok(None) => (StatusCode::FORBIDDEN, Json(json!({"code":"forbiddenFrom","error":"This sending address is not authorized for this account."}))).into_response(),
        Err(()) => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"code":"senderAuthorityUnavailable","error":"Sending authority could not be verified."}))).into_response(),
    }
}

// Deliberately bounded addr-spec input. Display names are a separate composer field;
// authorization below still requires an exact current authoritative address match.
fn valid_address(value: &str) -> bool {
    if value.len() > 254
        || value
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || "<>,;:\"()[]\\".contains(c))
    {
        return false;
    }
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && local.len() <= 64
        && !domain.is_empty()
        && !domain.contains('@')
        && !local.starts_with('.')
        && !local.ends_with('.')
        && !local.contains("..")
        && domain.split('.').all(|label| {
            !label.is_empty()
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
}

#[cfg(test)]
mod tests {
    use super::valid_address;
    #[test]
    fn accepts_one_bounded_address_never_header_syntax() {
        for address in ["alias@example.test", "first+tag@example.test"] {
            assert!(valid_address(address));
        }
        for address in [
            "",
            "Name <alias@example.test>",
            "a@example.test,b@example.test",
            "a@example.test\r\nBcc:evil@example.test",
            "a@@example.test",
            ".a@example.test",
            "a@-example.test",
        ] {
            assert!(!valid_address(address));
        }
    }
}
