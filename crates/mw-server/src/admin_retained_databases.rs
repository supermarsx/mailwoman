//! Explicit, global-administrator retirement of verified SQLite predecessors.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Json,
    extract::State,
    http::{Method, StatusCode, header},
    response::{IntoResponse, Response},
};

pub(crate) fn register(registrar: &mut impl AdminRegistrar) {
    registrar.admin_endpoint(
        Method::GET,
        "/maintenance/database-retained",
        Capability::MaintenanceRead,
        list,
    );
    registrar.admin_endpoint(
        Method::DELETE,
        "/maintenance/database-retained",
        Capability::MaintenanceRun,
        retire,
    );
}
async fn list(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::MaintenanceRead)?;
    let report = state.store.retained_databases().await.map_err(|detail| {
        AdminApiError::new(StatusCode::CONFLICT, "retainedDatabaseUnavailable", detail)
    })?;
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(report)).into_response())
}
async fn retire(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Json(request): Json<mw_store::RetireDatabaseRequest>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::MaintenanceRun)?;
    // Keep the deletion and audit lifecycle alive even if the client disconnects.
    let result=tokio::spawn(async move {
        let report=state.store.retire_retained_database(request).await?;
        let entry=mw_admin::AuditLogEntry {
            id:uuid::Uuid::new_v4().to_string(),ts:chrono::Utc::now().to_rfc3339(),actor:ctx.username,
            actor_kind:mw_admin::ActorKind::Admin,action:"retained-database-deleted".into(),target:Some(report.file_name.clone()),
            detail_json:mw_admin::redact_detail(&serde_json::json!({"generationId":report.generation_id,"removedBytes":report.removed_bytes})),ip:None,
        };
        if let Err(error)=state.v6.admin.audit(entry).await {tracing::warn!(%error,"retained database retirement audit unavailable");}
        Ok::<_,String>(report)
    }).await.map_err(AdminApiError::internal)?
      .map_err(|detail|AdminApiError::new(StatusCode::CONFLICT,"retainedDatabaseUnavailable",detail))?;
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(result)).into_response())
}
