//! Authenticated, effective-mailbox checkpoint; no client-selected account IDs.
use super::*;

pub(super) async fn read(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match session_of(&state, &headers).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    let owner = match personal_storage::owner(&state, &session, &headers, false).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    match state.store.mail_check_checkpoint(&owner).await {
        Ok(previous) => {
            Json(json!({"checkedAt":previous,"now":chrono::Utc::now().timestamp()})).into_response()
        }
        Err(error) => server_error("mail check", error),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Advance {
    expected: Option<i64>,
    checked_at: i64,
}

pub(super) async fn advance(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Advance>,
) -> Response {
    let session = match session_of(&state, &headers).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    let owner = match personal_storage::owner(&state, &session, &headers, true).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    // Reject extreme integers before arithmetic in the store's expiry token.
    let now = chrono::Utc::now().timestamp();
    if body.checked_at > now
        || body.checked_at < now - 600
        || body.expected.is_some_and(|v| v < 0 || v > now)
    {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error":"Invalid mail check time"})),
        )
            .into_response();
    }
    match state
        .store
        .advance_mail_check(&owner, body.expected, body.checked_at)
        .await
    {
        Ok(accepted) => Json(json!({"accepted":accepted})).into_response(),
        Err(error) => server_error("mail check", error),
    }
}
