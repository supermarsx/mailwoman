//! Feature-aware user route helpers. Admission is evaluated against live policy.
use crate::{AppState, settings_registry, settings_routes::UserSubject};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

pub(crate) async fn feature_session(
    state: &AppState,
    headers: &axum::http::HeaderMap,
    id: &str,
) -> Result<mw_store::Session, Response> {
    let session = crate::authed(state, headers).await?;
    let subject = UserSubject::resolved(&state.store, &session)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE.into_response())?;
    require_feature(state, &subject, id).await?;
    Ok(session)
}

pub(crate) async fn require_key(
    state: &AppState,
    session: &mw_store::Session,
    key: &str,
    feature: &str,
) -> Result<(), Response> {
    let subject = UserSubject::resolved(&state.store, session)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE.into_response())?;
    match settings_registry::effective_for(&state.store, key, &subject.as_subject()).await {
        Ok(Some(value)) if value.value.as_bool() == Some(true) => Ok(()),
        Ok(_) => Err((
            StatusCode::FORBIDDEN,
            axum::Json(json!({"code":"feature_disabled","feature":feature})),
        )
            .into_response()),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE.into_response()),
    }
}

/// Call after authentication and before any mutation or backend request.
pub(crate) async fn require_feature(
    state: &AppState,
    subject: &UserSubject,
    id: &str,
) -> Result<(), Response> {
    let Some(feature) = crate::features::CATALOGUE.iter().find(|f| f.id == id) else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({"code":"unknown_feature"})),
        )
            .into_response());
    };
    for key in feature.capability_keys {
        match settings_registry::effective_for(&state.store, key, &subject.as_subject()).await {
            Ok(Some(value)) if value.value.as_bool() == Some(true) => {}
            Ok(_) => {
                return Err((
                    StatusCode::FORBIDDEN,
                    axum::Json(json!({"code":"feature_disabled","feature":id})),
                )
                    .into_response());
            }
            Err(_) => return Err(StatusCode::SERVICE_UNAVAILABLE.into_response()),
        }
    }
    Ok(())
}
