//! Explicit private-destination grants, separate from route and plugin hostname grants.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Json,
    extract::{Path, State},
    http::Method,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

pub(super) fn register(registrar: &mut impl AdminRegistrar) {
    registrar.admin_endpoint(
        Method::GET,
        "/egress/destinations",
        Capability::EgressRead,
        list,
    );
    registrar.admin_endpoint(
        Method::POST,
        "/egress/destinations",
        Capability::EgressWrite,
        create,
    );
    registrar.admin_endpoint(
        Method::DELETE,
        "/egress/destinations/{id}",
        Capability::EgressWrite,
        remove,
    );
}

async fn list(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::EgressRead)?;
    let rows = state
        .store
        .list_egress_internal_allow()
        .await
        .map_err(AdminApiError::internal)?;
    Ok(Json(json!({"destinations": rows.into_iter().map(|row| json!({
        "id":row.id,"trafficClass":row.traffic_class,"destination":row.destination,
        "note":row.note,"createdBy":row.created_by,"createdAt":row.created_at,
    })).collect::<Vec<_>>(), "eligibleClasses": mw_egress::TrafficClass::ALL.into_iter().filter(|class| class.internal_allow_eligible()).map(|class|class.as_str()).collect::<Vec<_>>()})).into_response())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Create {
    traffic_class: String,
    destination: String,
    #[serde(default)]
    note: String,
}

fn validate(
    body: &Create,
) -> Result<(mw_egress::TrafficClass, mw_egress::AllowDestination), AdminApiError> {
    let class = mw_egress::TrafficClass::parse(&body.traffic_class)
        .filter(|class| class.internal_allow_eligible())
        .ok_or_else(|| {
            AdminApiError::invalid(
                "invalidTrafficClass",
                "This traffic class cannot reach private destinations.",
                "/trafficClass",
            )
        })?;
    if body.destination.len() > 512
        || body.note.len() > 256
        || body.note.chars().any(char::is_control)
    {
        return Err(AdminApiError::invalid(
            "invalidDestination",
            "Destination or note is too long or contains control characters.",
            "/destination",
        ));
    }
    let destination = mw_egress::AllowDestination::parse(&body.destination)
        .map_err(|message| AdminApiError::invalid("invalidDestination", message, "/destination"))?;
    Ok((class, destination))
}

async fn create(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(body): AdminJson<Create>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::EgressWrite)?;
    crate::step_up::require_admin(
        &state,
        &ctx,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    let (class, destination) = validate(&body)?;
    let rows = state
        .store
        .list_egress_internal_allow()
        .await
        .map_err(AdminApiError::internal)?;
    if rows.len() >= 256 {
        return Err(AdminApiError::invalid(
            "limitExceeded",
            "At most 256 internal destinations are allowed.",
            "/destination",
        ));
    }
    let destination = destination.to_string();
    if rows
        .iter()
        .any(|row| row.traffic_class == class.as_str() && row.destination == destination)
    {
        return Err(AdminApiError::invalid(
            "alreadyExists",
            "This class already has that destination grant.",
            "/destination",
        ));
    }
    // Stable per-class IDs make concurrent identical grants hit the existing PK.
    // Class is part of the key, so an analytics grant cannot become a plugin grant.
    let id: String = Sha256::digest(format!("{}\0{}", class.as_str(), destination).as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    state
        .store
        .add_egress_internal_allow(&mw_store::EgressAllowRow {
            id: id.clone(),
            traffic_class: class.as_str().into(),
            destination: destination.clone(),
            note: body.note,
            created_by: ctx.username.clone(),
            created_at: chrono::Utc::now().timestamp(),
        })
        .await
        .map_err(AdminApiError::internal)?;
    crate::image_proxy::refresh_plan(&state.store)
        .await
        .map_err(|_| AdminApiError::internal("egress reload"))?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "egress-destination-created",
        &format!("egress-destination:{id}"),
        None,
        Some(json!({"trafficClass":class.as_str(),"destination":destination})),
    )
    .await;
    Ok(Json(json!({"id":id})).into_response())
}

async fn remove(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::EgressWrite)?;
    crate::step_up::require_admin(
        &state,
        &ctx,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    let row = state
        .store
        .list_egress_internal_allow()
        .await
        .map_err(AdminApiError::internal)?
        .into_iter()
        .find(|row| row.id == id)
        .ok_or_else(|| AdminApiError::not_found("Internal destination not found."))?;
    let removed = state
        .store
        .delete_egress_internal_allow(&id)
        .await
        .map_err(AdminApiError::internal)?;
    crate::image_proxy::refresh_plan(&state.store)
        .await
        .map_err(|_| AdminApiError::internal("egress reload"))?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "egress-destination-removed",
        &format!("egress-destination:{id}"),
        Some(json!({"trafficClass":row.traffic_class,"destination":row.destination})),
        Some(json!({"removed":removed})),
    )
    .await;
    Ok(Json(json!({"removed":removed})).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_grants_are_class_specific_and_never_admit_metadata() {
        let body = |class: &str, destination: &str| Create {
            traffic_class: class.into(),
            destination: destination.into(),
            note: String::new(),
        };
        assert_eq!(
            validate(&body("plugins", "ews.corp.test:443")).unwrap().0,
            mw_egress::TrafficClass::Plugins
        );
        for class in ["remote_content", "oauth", "push", "all"] {
            assert!(validate(&body(class, "10.2.0.0/16")).is_err());
        }
        for destination in [
            "169.254.169.254",
            "127.0.0.1",
            "0.0.0.0/0",
            "http://ews.corp.test/",
            "ews.corp.test:0",
        ] {
            assert!(
                validate(&body("plugins", destination)).is_err(),
                "{destination}"
            );
        }
    }
}
