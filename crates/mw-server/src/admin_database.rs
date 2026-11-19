//! Global, read-only database observability. No arbitrary SQL or credentials.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    capabilities::Capability as C,
};
use axum::{
    Json,
    extract::{Query, State},
    http::{Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::sync::{Arc, LazyLock};

pub(crate) fn register(registrar: &mut impl AdminRegistrar) {
    registrar.admin_endpoint(
        Method::GET,
        "/observability/database",
        C::ObservabilityRead,
        report,
    );
}
#[derive(Deserialize, Default)]
struct Options {
    #[serde(default)]
    detailed: bool,
}
static INSPECTION: LazyLock<Arc<tokio::sync::Semaphore>> =
    LazyLock::new(|| Arc::new(tokio::sync::Semaphore::new(1)));

async fn report(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(options): Query<Options>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::ObservabilityRead)?;
    let permit = if options.detailed {
        match INSPECTION.clone().try_acquire_owned() {
            Ok(permit) => Some(permit),
            Err(_) => {
                return Ok((
                    StatusCode::TOO_MANY_REQUESTS,
                    [(header::RETRY_AFTER, "30")],
                    Json(serde_json::json!({"code":"databaseInspectionBusy"})),
                )
                    .into_response());
            }
        }
    } else {
        None
    };
    let value = tokio::spawn(async move {
        let _permit = permit;
        let mut value=state.store.database_observability(options.detailed).await?;
        value["backgroundMaintenance"]=crate::background_maintenance::status(&state.store);
        Ok::<_,mw_store::StoreError>(value)
    })
    .await
    .map_err(AdminApiError::internal)??;
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(value)).into_response())
}

/// Sampling continues with no console open. History is bounded and in memory;
/// only a deduplicated sustained-growth event reaches the audit log.
pub(crate) fn start(state: AppState) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut last_alert = 0i64;
        loop {
            interval.tick().await;
            match state.store.database_observability(false).await {
                Ok(report) => {
                    let rapid = report["growth"]["status"] == "rapid-growth";
                    let now = chrono::Utc::now().timestamp();
                    let enabled = state
                        .store
                        .get_setting_policy("observability.database.growthAlerts")
                        .await
                        .ok()
                        .flatten()
                        .and_then(|p| p.global_value)
                        .as_deref()
                        != Some("false");
                    if rapid && enabled && now - last_alert >= 1800 {
                        last_alert = now;
                        let growth = &report["growth"];
                        let detail = serde_json::json!({"status":growth["status"],"bytesPerMinute":growth["bytesPerMinute"],"thresholdBytesPerMinute":growth["thresholdBytesPerMinute"],
                            "writeOrigins":growth["writeOrigins"].as_array().map(|rows|rows.iter().take(10).cloned().collect::<Vec<_>>())});
                        tracing::warn!(growth=%detail,"Sustained database growth: inspect Observability > Database");
                        crate::audit::security_event(
                            &state,
                            crate::audit::SecurityEvent::system("database-rapid-growth")
                                .target("database")
                                .detail(detail)
                                .category("operational"),
                        )
                        .await;
                    }
                }
                Err(error) => tracing::warn!(%error,"Database growth sampling unavailable"),
            }
        }
    });
}
