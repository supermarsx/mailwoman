//! Engine receipt preferences; the account always comes from authentication.
use super::*;
use mw_store::receipts::{SenderDecision, SenderRuleWrite, normalize_sender};

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/account/mdn-senders", get(list).put(write))
        .route("/api/account/mdn-decision/{id}", get(decision))
}

async fn list(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match session_of(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    if state.engine.is_none() {
        return no_mdn_policy();
    }
    match state.store.receipt_sender_rules(&session.account_id).await {
        Ok(snapshot) => {
            let rows: Vec<_> = snapshot.rules.iter().map(|row| json!({"address": row.address, "decision": row.decision.as_str(), "updatedAt": row.updated_at})).collect();
            (
                [
                    ("etag", format!("\"{}\"", snapshot.revision)),
                    ("cache-control", "no-store".into()),
                ],
                Json(json!({"rules": rows})),
            )
                .into_response()
        }
        Err(error) => server_error("receipt sender rules", error),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WriteRule {
    address: String,
    decision: Option<String>,
}

async fn write(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<WriteRule>,
) -> Response {
    let session = match session_of(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    if state.engine.is_none() {
        return no_mdn_policy();
    }
    match held(&state, &session, "mail.readReceipts").await {
        Ok(Some(_)) => return setting_locked("mail.readReceipts"),
        Ok(None) => {}
        Err(response) => return response,
    }
    let Ok(address) = normalize_sender(&body.address) else {
        return bad_request("address must be one mailbox");
    };
    let decision = match body.decision.as_deref() {
        None => None,
        Some(value) => match SenderDecision::parse(value) {
            Some(value) => Some(value),
            None => return bad_request("decision must be always, never, or null to remove"),
        },
    };
    let Some(revision) = headers
        .get("if-match")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix('"'))
        .and_then(|value| value.strip_suffix('"'))
        .filter(|value| !value.is_empty() && value.len() <= 128 && !value.contains('"'))
    else {
        return (
            StatusCode::PRECONDITION_REQUIRED,
            "a current If-Match is required",
        )
            .into_response();
    };
    match state
        .store
        .write_receipt_sender_rule(&session.account_id, &address, decision, revision)
        .await
    {
        Ok(SenderRuleWrite::Applied(revision)) => (
            [
                ("etag", format!("\"{revision}\"")),
                ("cache-control", "no-store".into()),
            ],
            Json(json!({"ok": true})),
        )
            .into_response(),
        Ok(SenderRuleWrite::Conflict) => {
            (StatusCode::PRECONDITION_FAILED, "sender rules changed").into_response()
        }
        Ok(SenderRuleWrite::LimitReached) => {
            (StatusCode::CONFLICT, "sender rule limit reached").into_response()
        }
        Err(error) => server_error("receipt sender rule write", error),
    }
}

async fn decision(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let session = match session_of(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    let Some(engine) = &state.engine else {
        return no_mdn_policy();
    };
    if id.is_empty() || id.len() > 512 {
        return bad_request("invalid message id");
    }
    match engine
        .receipt_response_decision(&session.account_id, &id)
        .await
    {
        Ok(Some(value)) => ([("cache-control", "no-store")], Json(value)).into_response(),
        Ok(None) => not_found("no available receipt request"),
        Err(error) => {
            tracing::warn!("receipt decision unavailable: {error}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "receipt decision unavailable",
            )
                .into_response()
        }
    }
}
