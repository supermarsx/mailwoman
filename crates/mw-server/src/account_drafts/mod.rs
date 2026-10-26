//! Account drafts use confirmed upstream persistence and durable save claims.
mod model;
use crate::AppState;
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use model::EditorState;
use mw_store::account_drafts::{
    AccountDraft, DraftClaim, DraftDiscard, DraftSave, MAX_DRAFT_STATE,
};
use serde::Deserialize;
use serde_json::{Value, json};

fn failure(status: StatusCode, code: &str) -> Response {
    (status, Json(json!({"code":code,"error":code}))).into_response()
}
fn document(row: &AccountDraft, authority: &str, with_state: bool) -> Result<Value, Response> {
    let summary: Value = if row.summary.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&row.summary)
            .map_err(|_| failure(StatusCode::SERVICE_UNAVAILABLE, "draftStateUnavailable"))?
    };
    let mut value = json!({"accountId":row.account_id,"storageAuthority":authority,"id":row.id,"revision":row.revision,"emailId":row.email_id,"savedAt":row.updated_at,"status":row.status,"cleanupPending":row.cleanup_email_id.is_some(),"summary":summary});
    if with_state && !row.state.is_empty() {
        value["state"] = serde_json::from_slice(&row.state)
            .map_err(|_| failure(StatusCode::SERVICE_UNAVAILABLE, "draftStateUnavailable"))?;
    }
    Ok(value)
}
async fn context(
    state: &AppState,
    headers: &HeaderMap,
    mutation: bool,
) -> Result<(mw_store::Session, String, String), Response> {
    let session = crate::authed(state, headers).await?;
    let authority =
        crate::prefs_routes::personal_storage::owner(state, &session, headers, mutation).await?;
    let owner = state
        .store
        .session_personal_subject(&session.id)
        .await
        .ok()
        .flatten()
        .ok_or_else(|| {
            failure(
                StatusCode::SERVICE_UNAVAILABLE,
                "storageAuthorityUnavailable",
            )
        })?;
    Ok((session, authority, owner))
}
pub(crate) fn router() -> Router<AppState> {
    let capacity = std::sync::Arc::new(tokio::sync::Semaphore::new(2));
    Router::new()
        .route("/api/account/drafts", get(list))
        .route("/api/account/drafts/resolve", get(resolve))
        .route(
            "/api/account/drafts/{id}",
            get(load).put(save).delete(discard),
        )
        .route("/api/account/drafts/{id}/cleanup", post(cleanup))
        .layer(DefaultBodyLimit::max(MAX_DRAFT_STATE + 8192))
        .layer(axum::middleware::from_fn(
            move |request, next: axum::middleware::Next| {
                let capacity = capacity.clone();
                async move {
                    let Ok(_permit) = capacity.try_acquire() else {
                        return failure(StatusCode::TOO_MANY_REQUESTS, "draftCapacityBusy");
                    };
                    let mut response = match tokio::time::timeout(
                        std::time::Duration::from_secs(45),
                        next.run(request),
                    )
                    .await
                    {
                        Ok(response) => response,
                        Err(_) => failure(StatusCode::GATEWAY_TIMEOUT, "draftSaveUncertain"),
                    };
                    response.headers_mut().insert(
                        "cache-control",
                        axum::http::HeaderValue::from_static("no-store"),
                    );
                    response
                }
            },
        ))
}
async fn list(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let (session, authority, owner) = match context(&state, &headers, false).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let supported = match &state.engine {
        Some(engine) => {
            crate::engine_mode::ensure_account(engine, &session.account_id)
                .await
                .is_ok()
                && engine
                    .account_draft_support(&session.account_id)
                    .await
                    .is_ok()
        }
        None => false,
    };
    let rows = match state.store.account_drafts(&owner).await {
        Ok(v) => v,
        Err(_) => return failure(StatusCode::SERVICE_UNAVAILABLE, "draftStateUnavailable"),
    };
    let items: Result<Vec<_>, _> = rows
        .iter()
        .map(|r| document(r, &authority, false))
        .collect();
    match items{Ok(items)=>Json(json!({"accountId":session.account_id,"storageAuthority":authority,"supported":supported,"reason":if supported{Value::Null}else{json!("upstreamDraftsUnavailable")},"items":items,"maximumDrafts":100})).into_response(),Err(e)=>e}
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Resolve {
    email_id: String,
}
async fn resolve(
    State(state): State<AppState>,
    Query(query): Query<Resolve>,
    headers: HeaderMap,
) -> Response {
    let (session, authority, owner) = match context(&state, &headers, false).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    if query.email_id.len() > 512 {
        return failure(StatusCode::BAD_REQUEST, "invalidDraft");
    }
    match state
        .store
        .account_draft_for_email(&owner, &query.email_id)
        .await
    {
        Ok(row) => {
            let draft = match row
                .as_ref()
                .map(|r| document(r, &authority, true))
                .transpose()
            {
                Ok(v) => v,
                Err(e) => return e,
            };
            Json(json!({"accountId":session.account_id,"storageAuthority":authority,"draft":draft}))
                .into_response()
        }
        Err(_) => failure(StatusCode::SERVICE_UNAVAILABLE, "draftStateUnavailable"),
    }
}
async fn load(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let (_, authority, owner) = match context(&state, &headers, false).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    match state.store.account_draft(&owner, &id).await {
        Ok(Some(row)) => match document(&row, &authority, true) {
            Ok(v) => Json(v).into_response(),
            Err(e) => e,
        },
        Ok(None) => failure(StatusCode::NOT_FOUND, "draftNotFound"),
        Err(_) => failure(StatusCode::SERVICE_UNAVAILABLE, "draftStateUnavailable"),
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Save {
    operation_id: String,
    expected_revision: Option<String>,
    state: EditorState,
}
async fn save(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    let (session, authority, owner) = match context(&state, &headers, true).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let mut request: Save = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => return failure(StatusCode::BAD_REQUEST, "invalidDraft"),
    };
    if let Err(code) = request.state.validate() {
        return failure(StatusCode::BAD_REQUEST, code);
    }
    let Some(engine) = &state.engine else {
        return failure(StatusCode::NOT_IMPLEMENTED, "upstreamDraftsUnavailable");
    };
    if crate::engine_mode::ensure_account(engine, &session.account_id)
        .await
        .is_err()
    {
        return failure(StatusCode::SERVICE_UNAVAILABLE, "upstreamDraftsUnavailable");
    }
    match state.store.get_zeroaccess(&session.account_id).await {
        Ok(Some(row)) if row.enabled => {
            return failure(StatusCode::FORBIDDEN, "protectedDraftUnsupported");
        }
        Ok(_) => {}
        Err(_) => return failure(StatusCode::SERVICE_UNAVAILABLE, "draftPolicyUnavailable"),
    }
    let state_bytes = match serde_json::to_vec(&request.state) {
        Ok(v) => v,
        Err(_) => return failure(StatusCode::BAD_REQUEST, "invalidDraft"),
    };
    let summary = match serde_json::to_vec(&request.state.summary()) {
        Ok(v) => v,
        Err(_) => return failure(StatusCode::BAD_REQUEST, "invalidDraft"),
    };
    let input = DraftSave {
        owner: &owner,
        account_id: &session.account_id,
        id: &id,
        expected_revision: request.expected_revision.as_deref(),
        operation_id: &request.operation_id,
        state: &state_bytes,
        summary: &summary,
        now: chrono::Utc::now().timestamp(),
    };
    match state.store.claim_account_draft(&input).await {
        Ok(DraftClaim::Claimed) => {}
        Ok(DraftClaim::Saved(row)) => {
            return match document(&row, &authority, true) {
                Ok(v) => Json(v).into_response(),
                Err(e) => e,
            };
        }
        Ok(other) => {
            return failure(
                StatusCode::CONFLICT,
                match other {
                    DraftClaim::Uncertain => "draftSaveUncertain",
                    DraftClaim::CleanupPending => "draftCleanupPending",
                    DraftClaim::Limit => "draftLimit",
                    DraftClaim::Invalid => "invalidDraft",
                    DraftClaim::Refused => "draftSaveRefused",
                    _ => "draftConflict",
                },
            );
        }
        Err(_) => return failure(StatusCode::SERVICE_UNAVAILABLE, "draftStateUnavailable"),
    }
    let prepared = match engine
        .prepare_account_draft(
            &session.account_id,
            &request.state.compose_spec(),
            Some(&request.state.identity_id),
        )
        .await
    {
        Ok(v) => v,
        Err(_) => {
            let _ = state
                .store
                .refuse_account_draft_before_append(&owner, &id, &request.operation_id)
                .await;
            return failure(StatusCode::UNPROCESSABLE_ENTITY, "draftPreparationRefused");
        }
    };
    if !matches!(context(&state, &headers, true).await, Ok((_, a, o)) if a == authority && o == owner)
    {
        let _ = state
            .store
            .refuse_account_draft_before_append(&owner, &id, &request.operation_id)
            .await;
        return failure(StatusCode::CONFLICT, "accountChanged");
    }
    let email = match engine.persist_account_draft(prepared).await {
        Ok(v) => v,
        Err(_) => {
            let _ = state
                .store
                .uncertain_account_draft(&owner, &id, &request.operation_id)
                .await;
            return failure(StatusCode::CONFLICT, "draftSaveUncertain");
        }
    };
    // From this point even local failures retain the claim: APPEND may be accepted.
    let parts = match engine
        .account_draft_parts(&session.account_id, &email)
        .await
    {
        Ok(v) => v,
        Err(_) => return failure(StatusCode::CONFLICT, "draftSaveUncertain"),
    };
    if rebind(&mut request.state, &parts).is_err() {
        return failure(StatusCode::CONFLICT, "draftSaveUncertain");
    }
    let rebound = match serde_json::to_vec(&request.state) {
        Ok(v) => v,
        Err(_) => return failure(StatusCode::CONFLICT, "draftSaveUncertain"),
    };
    if !matches!(
        state
            .store
            .rebind_account_draft_state(&owner, &id, &request.operation_id, &rebound)
            .await,
        Ok(true)
    ) {
        return failure(StatusCode::CONFLICT, "draftSaveUncertain");
    }
    let mut row = match state
        .store
        .complete_account_draft(
            &owner,
            &id,
            &request.operation_id,
            &email,
            chrono::Utc::now().timestamp(),
        )
        .await
    {
        Ok(Some(v)) => v,
        _ => return failure(StatusCode::CONFLICT, "draftSaveUncertain"),
    };
    if let Some(old) = row.cleanup_email_id.clone()
        && engine
            .cleanup_account_draft(&session.account_id, &old)
            .await
            .is_ok()
        && matches!(
            state
                .store
                .account_draft_cleanup_done(&owner, &id, &old)
                .await,
            Ok(true)
        )
    {
        row.cleanup_email_id = None;
    }
    match document(&row, &authority, true) {
        Ok(v) => Json(v).into_response(),
        Err(e) => e,
    }
}
fn rebind(state: &mut EditorState, parts: &Value) -> Result<(), ()> {
    let files = parts
        .get("attachments")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    if files.len() != state.attachments.len() {
        return Err(());
    }
    for (file, part) in state.attachments.iter_mut().zip(files) {
        file.blob_id = part["blobId"].as_str().ok_or(())?.into();
        file.size = part["size"].as_u64().ok_or(())?;
    }
    let inline = parts
        .get("mailwomanInlineParts")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    for file in &mut state.inline {
        let mut matching = inline
            .iter()
            .filter(|part| part["cid"].as_str() == Some(file.cid.as_str()));
        let part = matching.next().ok_or(())?;
        if matching.next().is_some() {
            return Err(());
        }
        file.blob_id = part["blobId"].as_str().ok_or(())?.into();
        file.size = part["size"].as_u64().ok_or(())?;
    }
    Ok(())
}
async fn cleanup(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let (session, authority, owner) = match context(&state, &headers, true).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let mut row = match state.store.account_draft(&owner, &id).await {
        Ok(Some(v)) => v,
        Ok(None) => return failure(StatusCode::NOT_FOUND, "draftNotFound"),
        Err(_) => return failure(StatusCode::SERVICE_UNAVAILABLE, "draftStateUnavailable"),
    };
    if row.cleanup_email_id.is_some() && row.cleanup_email_id == row.email_id {
        return failure(StatusCode::CONFLICT, "draftDiscardPending");
    }
    if let Some(old) = row.cleanup_email_id.clone() {
        let Some(engine) = &state.engine else {
            return failure(StatusCode::NOT_IMPLEMENTED, "upstreamDraftsUnavailable");
        };
        if crate::engine_mode::ensure_account(engine, &session.account_id)
            .await
            .is_err()
            || engine
                .cleanup_account_draft(&session.account_id, &old)
                .await
                .is_err()
        {
            return failure(StatusCode::CONFLICT, "draftCleanupPending");
        }
        if !matches!(
            state
                .store
                .account_draft_cleanup_done(&owner, &id, &old)
                .await,
            Ok(true)
        ) {
            return failure(StatusCode::CONFLICT, "draftCleanupPending");
        }
        row.cleanup_email_id = None;
    }
    match document(&row, &authority, true) {
        Ok(v) => Json(v).into_response(),
        Err(e) => e,
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Discard {
    expected_revision: Option<String>,
}
async fn discard(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<Discard>,
) -> Response {
    let (session, authority, owner) = match context(&state, &headers, true).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let row=match state.store.claim_account_draft_discard(&owner,&id,request.expected_revision.as_deref()).await{
  Ok(DraftDiscard::Gone)=>return Json(json!({"accountId":session.account_id,"storageAuthority":authority,"id":id,"deleted":true})).into_response(),
  Ok(DraftDiscard::Ready(row))=>row,
  Ok(other)=>return failure(StatusCode::CONFLICT,match other{DraftDiscard::Uncertain=>"draftSaveUncertain",DraftDiscard::CleanupPending=>"draftCleanupPending",_=>"draftConflict"}),
  Err(_)=>return failure(StatusCode::SERVICE_UNAVAILABLE,"draftStateUnavailable")
 };
    let Some(engine) = &state.engine else {
        return failure(StatusCode::NOT_IMPLEMENTED, "upstreamDraftsUnavailable");
    };
    let Some(email) = row.email_id.as_deref() else {
        return failure(StatusCode::CONFLICT, "draftDiscardPending");
    };
    if crate::engine_mode::ensure_account(engine, &session.account_id)
        .await
        .is_err()
        || engine
            .cleanup_account_draft(&session.account_id, email)
            .await
            .is_err()
    {
        return failure(StatusCode::CONFLICT, "draftDiscardPending");
    }
    if !matches!(
        state
            .store
            .complete_account_draft_discard(
                &owner,
                &id,
                row.revision.as_deref().unwrap_or(""),
                email
            )
            .await,
        Ok(true)
    ) {
        return failure(StatusCode::CONFLICT, "draftDiscardPending");
    }
    Json(
        json!({"accountId":session.account_id,"storageAuthority":authority,"id":id,"deleted":true}),
    )
    .into_response()
}
