//! Personal documents require a verified realm, not a raw upstream account ID.
use super::*;

pub(crate) const AUTHORITY_HEADER: &str = "x-mailwoman-storage-authority";

pub(crate) fn key(kind: &str, owner: &str) -> String {
    format!("personal-v2:{kind}:{owner}")
}

pub(crate) async fn owner(
    state: &AppState,
    session: &mw_store::Session,
    headers: &HeaderMap,
    mutation: bool,
) -> Result<String, Response> {
    // t50hp-e9: on published inputs the authority is the frame's (as /api/me).
    let framed = crate::request_inputs::current(&state.store).await
        .filter(|inputs| inputs.matches_session(session))
        .map(|inputs| Ok(Some(inputs.storage_authority())));
    let read = match framed { Some(framed) => framed, None => state.store.session_storage_authority(&session.id).await };
    let authority = match read {
        Ok(Some(authority)) => authority,
        _ => return Err((StatusCode::SERVICE_UNAVAILABLE, Json(json!({"code":"storageAuthorityUnavailable","error":"Sign in again before loading personal settings."}))).into_response()),
    };
    match headers.get(AUTHORITY_HEADER) {
        Some(value) if value.to_str().ok() != Some(authority.as_str()) =>
            Err((StatusCode::CONFLICT, Json(json!({"code":"accountChanged","error":"The signed-in account changed. Reload these settings."}))).into_response()),
        None if mutation => Err((StatusCode::PRECONDITION_REQUIRED, Json(json!({"code":"storageAuthorityRequired","error":"Reload these settings before saving."}))).into_response()),
        _ => Ok(authority),
    }
}

/// Durable record owner with the same browser/session authority fence as personal KV.
pub(crate) async fn record_owner(
    state: &AppState,
    session: &mw_store::Session,
    headers: &HeaderMap,
    mutation: bool,
) -> Result<String, Response> {
    owner(state, session, headers, mutation).await?;
    match state.store.session_personal_subject(&session.id).await {
        Ok(Some(owner)) => Ok(owner),
        _ => Err(StatusCode::SERVICE_UNAVAILABLE.into_response()),
    }
}
