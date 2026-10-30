//! t94: the user's Focused inbox overrides ("Always move to Focused / Other").
//!
//! The owner is the session's personal record owner (`personal_record_owner`,
//! the same fence as signatures): never a value from the request, so a user
//! reads and changes only their own rows. Writes need the storage-authority
//! header the web app sends with personal settings.
//!
//! * `GET  /api/account/focused-senders` → `{senders:[{email,tab,updatedAt}]}`
//! * `PUT  /api/account/focused-senders` `{email, tab:"focused"|"other"|null}`
//!   (`null` removes the override)
//! * `POST /api/account/focused-senders/import` `{senders:{email:tab}}`: the
//!   one-time move of a browser's old local overrides; rows already on the
//!   server win. Answers `{added, senders}`.
use super::*;
use mw_store::focused_senders::{
    FocusedSenderWrite, MAX_FOCUSED_SENDERS, normalize_focused_sender, valid_focus_tab,
};

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/account/focused-senders", get(list).put(write))
        .route(
            "/api/account/focused-senders/import",
            axum::routing::post(import),
        )
}

fn no_store(body: serde_json::Value) -> Response {
    ([("cache-control", "no-store")], Json(body)).into_response()
}

async fn senders_json(store: &Store, owner: &str) -> Result<serde_json::Value, StoreError> {
    let rows = store.list_focused_senders(owner).await?;
    Ok(json!(
        rows.iter()
            .map(|r| json!({"email": r.sender, "tab": r.tab, "updatedAt": r.updated_at}))
            .collect::<Vec<_>>()
    ))
}

async fn list(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let owner = match personal_record_owner(&state, &headers, false).await {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    match senders_json(&state.store, &owner).await {
        Ok(senders) => no_store(json!({ "senders": senders })),
        Err(e) => server_error("list focused senders", e),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WriteBody {
    email: String,
    tab: Option<String>,
}

async fn write(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<WriteBody>,
) -> Response {
    let owner = match personal_record_owner(&state, &headers, true).await {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    if normalize_focused_sender(&body.email).is_err() {
        return bad_request("email must be one mailbox address");
    }
    if body.tab.as_deref().is_some_and(|tab| !valid_focus_tab(tab)) {
        return bad_request("tab must be focused, other, or null to remove");
    }
    match state
        .store
        .set_focused_sender(&owner, &body.email, body.tab.as_deref())
        .await
    {
        Ok(FocusedSenderWrite::Applied) => ok(),
        Ok(FocusedSenderWrite::LimitReached) => (
            StatusCode::CONFLICT,
            Json(json!({"code":"limitReached","error":"Too many Focused inbox senders."})),
        )
            .into_response(),
        Err(e) => server_error("write focused sender", e),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ImportBody {
    senders: serde_json::Map<String, serde_json::Value>,
}

async fn import(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ImportBody>,
) -> Response {
    let owner = match personal_record_owner(&state, &headers, true).await {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    if body.senders.len() > MAX_FOCUSED_SENDERS {
        return payload_too_large("too many senders");
    }
    let entries: Vec<(String, String)> = body
        .senders
        .iter()
        .filter_map(|(email, tab)| Some((email.clone(), tab.as_str()?.to_string())))
        .collect();
    let added = match state.store.import_focused_senders(&owner, &entries).await {
        Ok(added) => added,
        Err(e) => return server_error("import focused senders", e),
    };
    match senders_json(&state.store, &owner).await {
        Ok(senders) => no_store(json!({ "added": added, "senders": senders })),
        Err(e) => server_error("list focused senders", e),
    }
}
