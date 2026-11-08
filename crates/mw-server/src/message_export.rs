//! Policy and audit for `GET /api/export/{stableId}` (the message export behind
//! the reader's Export dialog).
//!
//! The route itself lives in `lib.rs`. Here: `export.message.download.enabled`
//! and the administrator's `export.formats` list are enforced on the server
//! (the dialog only mirrors them), and every export, refused or served, writes
//! a `user-message-export` audit row naming the message and format.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

use crate::{AppState, settings_registry};

/// Whether `format` is on the administrator's list. An empty list allows nothing.
pub(crate) fn format_allowed(list: &[String], format: &str) -> bool {
    list.iter().any(|f| f == format)
}

fn problem(status: StatusCode, code: &str) -> Response {
    (status, Json(json!({ "error": code }))).into_response()
}

/// Refuse the export when it is switched off or `format` is not offered.
/// Every refusal is audited; the caller audits the served case itself.
pub(crate) async fn gate(
    state: &AppState,
    session: &mw_store::Session,
    stable_id: &str,
    format: &str,
) -> Result<(), Response> {
    let unavailable = || problem(StatusCode::SERVICE_UNAVAILABLE, "exportPolicyUnavailable");
    let identity = crate::settings_routes::UserSubject::resolved(&state.store, session)
        .await
        .map_err(|_| unavailable())?;
    let subject = identity.as_subject();
    let get = |key: &'static str| {
        let subject = &subject;
        async move {
            settings_registry::effective_for(&state.store, key, subject)
                .await
                .ok()
                .flatten()
                .map(|e| e.value)
        }
    };
    let enabled = get("export.message.download.enabled").await.and_then(|v| v.as_bool());
    let formats: Option<Vec<String>> = get("export.formats").await.and_then(|v| {
        v.as_array()
            .map(|a| a.iter().filter_map(|s| s.as_str().map(str::to_string)).collect())
    });
    let (Some(enabled), Some(formats)) = (enabled, formats) else {
        return Err(unavailable());
    };
    if !enabled {
        audit(state, session, stable_id, format, "refused-disabled").await;
        return Err(problem(StatusCode::FORBIDDEN, "exportDisabled"));
    }
    if !format_allowed(&formats, format) {
        audit(state, session, stable_id, format, "refused-format").await;
        return Err(problem(StatusCode::BAD_REQUEST, "exportFormatNotAllowed"));
    }
    Ok(())
}

pub(crate) async fn audit(
    state: &AppState,
    session: &mw_store::Session,
    stable_id: &str,
    format: &str,
    outcome: &str,
) {
    let detail: Value = json!({ "format": format, "outcome": outcome });
    let row = mw_store::AuditRow {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor: session.username.clone(),
        actor_kind: "user".to_string(),
        action: "user-message-export".to_string(),
        target: Some(stable_id.to_string()),
        detail_json: detail.to_string(),
        ip: None,
    };
    if let Err(e) = state.store.append_audit(&row).await {
        tracing::warn!("message export audit append failed: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::format_allowed;

    #[test]
    fn listed_formats_pass_and_the_rest_do_not() {
        let list = vec!["eml".to_string(), "mbox".to_string()];
        assert!(format_allowed(&list, "eml"));
        assert!(!format_allowed(&list, "docx"));
        assert!(!format_allowed(&[], "eml"));
    }
}
