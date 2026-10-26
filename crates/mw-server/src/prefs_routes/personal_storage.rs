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
    let authority = match state.store.session_storage_authority(&session.id).await {
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
