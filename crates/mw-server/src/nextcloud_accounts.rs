//! Administrator-approved instances and independently sealed mailbox links.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson, check_if_match, etag_of, with_etag},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, put},
};
use mw_store::nextcloud_links::{NextcloudInstance, NextcloudLink};
use serde::Deserialize;
use serde_json::{Value, json};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/account/nextcloud", get(user_view))
        .route(
            "/api/account/nextcloud/links/{id}",
            put(link).delete(unlink),
        )
        .route("/api/nextcloud/files", get(files))
}
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/nextcloud/instances",
        Capability::IntegrationsRead,
        admin_list,
    );
    r.admin_endpoint(
        Method::PUT,
        "/nextcloud/instances/{id}",
        Capability::IntegrationsWrite,
        admin_save,
    );
}
fn metadata(row: &NextcloudInstance) -> Value {
    json!({"id":row.id,"name":row.name,"baseUrl":row.base_url,"enabled":row.enabled,"caPem":row.ca_pem,"etag":etag_of(&json!(row.revision))})
}
fn link_metadata(row: &NextcloudLink) -> Value {
    json!({"instanceId":row.instance_id,"username":row.username,"instanceRevision":row.instance_revision,"etag":etag_of(&json!(row.revision))})
}
fn failure(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({"error":message}))).into_response()
}
async fn admin_list(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsRead)?;
    let rows = state
        .store
        .nextcloud_instances()
        .await
        .map_err(AdminApiError::internal)?;
    Ok(Json(json!({"instances":rows.iter().map(metadata).collect::<Vec<_>>(),"maxInstances":64,"maxFileBytes":mw_egress::MAX_IMAGE_BYTES})).into_response())
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InstanceWrite {
    name: String,
    base_url: String,
    #[serde(default)]
    ca_pem: String,
    enabled: bool,
}
async fn admin_save(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    headers: HeaderMap,
    AdminJson(body): AdminJson<InstanceWrite>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsWrite)?;
    crate::step_up::require_admin(
        &state,
        &ctx,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    if id.is_empty()
        || id.len() > 64
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        || body.name.trim().is_empty()
        || body.name.len() > 128
        || body.name.chars().any(char::is_control)
    {
        return Err(AdminApiError::invalid(
            "invalidInstance",
            "Use a short instance identifier and name.",
            "/name",
        ));
    }
    let base = crate::nextcloud_config::base_url(&body.base_url)
        .map_err(|message| AdminApiError::invalid("invalidInstance", message, "/baseUrl"))?;
    crate::nextcloud_config::ca_roots(&body.ca_pem)
        .map_err(|message| AdminApiError::invalid("invalidCa", message, "/caPem"))?;
    let old = state
        .store
        .nextcloud_instance(&id)
        .await
        .map_err(AdminApiError::internal)?;
    match &old {
        Some(old) => {
            if !headers.contains_key("if-match") {
                return Err(AdminApiError::new(
                    StatusCode::PRECONDITION_REQUIRED,
                    "ifMatchRequired",
                    "Reload before editing.",
                ));
            }
            check_if_match(&headers, &etag_of(&json!(old.revision)), None, None)?;
        }
        None => {
            if headers.get("if-none-match").and_then(|v| v.to_str().ok()) != Some("*") {
                return Err(AdminApiError::new(
                    StatusCode::PRECONDITION_REQUIRED,
                    "ifNoneMatchRequired",
                    "Create with If-None-Match: *.",
                ));
            }
        }
    }
    let desired = NextcloudInstance {
        id,
        name: body.name.trim().into(),
        base_url: base.to_string(),
        ca_pem: body.ca_pem,
        enabled: body.enabled,
        revision: String::new(),
    };
    let saved = state
        .store
        .put_nextcloud_instance(&desired, old.as_ref().map(|row| row.revision.as_str()))
        .await
        .map_err(AdminApiError::internal)?
        .ok_or_else(|| AdminApiError::stale(None, None))?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "nextcloud-instance-configured",
        &format!("nextcloud:{}", saved.id),
        None,
        Some(json!({"enabled":saved.enabled})),
    )
    .await;
    Ok(with_etag(
        Json(metadata(&saved)),
        &etag_of(&json!(saved.revision)),
    ))
}
async fn user_view(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    let subject = match crate::settings_routes::UserSubject::resolved(&state.store, &session).await
    {
        Ok(subject) => subject,
        Err(_) => return failure(StatusCode::SERVICE_UNAVAILABLE, "policy unavailable"),
    };
    let allowed = crate::user_router::require_feature(&state, &subject, "integrations.nextcloud")
        .await
        .is_ok();
    let instances = match state.store.nextcloud_instances().await {
        Ok(rows) => rows,
        Err(_) => {
            return failure(
                StatusCode::INTERNAL_SERVER_ERROR,
                "configuration unavailable",
            );
        }
    };
    let links = match state.store.nextcloud_links(&session.account_id).await {
        Ok(rows) => rows,
        Err(_) => return failure(StatusCode::INTERNAL_SERVER_ERROR, "links unavailable"),
    };
    Json(json!({"accountId":session.account_id,"allowed":allowed,"instances":instances.iter().filter(|row|(allowed&&row.enabled)||links.iter().any(|link|link.instance_id==row.id)).map(metadata).collect::<Vec<_>>(),"links":links.iter().map(|row|{let mut value=link_metadata(row);value["ready"]=json!(allowed&&instances.iter().any(|instance|instance.id==row.instance_id&&instance.enabled&&instance.revision==row.instance_revision));value}).collect::<Vec<_>>(),"maxFileBytes":mw_egress::MAX_IMAGE_BYTES})).into_response()
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LinkWrite {
    account_id: String,
    username: String,
    app_password: String,
}
async fn link(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<LinkWrite>,
) -> Response {
    let session =
        match crate::user_router::feature_session(&state, &headers, "integrations.nextcloud").await
        {
            Ok(session) => session,
            Err(response) => return response,
        };
    if body.account_id != session.account_id {
        return failure(
            StatusCode::CONFLICT,
            "Account changed; reload before linking.",
        );
    }
    if crate::nextcloud_config::username(&body.username).is_err()
        || body.app_password.is_empty()
        || body.app_password.len() > 4096
    {
        return failure(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Enter the app-password username and password from Nextcloud security settings.",
        );
    }
    let instance = match state.store.nextcloud_instance(&id).await {
        Ok(Some(row)) if row.enabled => row,
        _ => return failure(StatusCode::NOT_FOUND, "instance unavailable"),
    };
    let old = match state.store.nextcloud_links(&session.account_id).await {
        Ok(rows) => rows.into_iter().find(|row| row.instance_id == id),
        Err(_) => return failure(StatusCode::INTERNAL_SERVER_ERROR, "links unavailable"),
    };
    if let Some(old) = &old {
        if !headers.contains_key("if-match") {
            return failure(
                StatusCode::PRECONDITION_REQUIRED,
                "Reload the link before replacing it.",
            );
        }
        if check_if_match(&headers, &etag_of(&json!(old.revision)), None, None).is_err() {
            return failure(StatusCode::PRECONDITION_FAILED, "Link changed; reload.");
        }
    }
    if let Err(reason) = crate::nextcloud_transport::verify(
        &state,
        &session,
        &instance,
        &body.username,
        &body.app_password,
    )
    .await
    {
        return failure(StatusCode::BAD_GATEWAY, reason);
    }
    let desired = NextcloudLink {
        account_id: session.account_id,
        instance_id: id,
        instance_revision: instance.revision,
        username: body.username,
        revision: String::new(),
    };
    match state
        .store
        .put_verified_nextcloud_link(
            &desired,
            &body.app_password,
            old.as_ref().map(|row| row.revision.as_str()),
        )
        .await
    {
        Ok(Some(saved)) => with_etag(
            Json(link_metadata(&saved)),
            &etag_of(&json!(saved.revision)),
        ),
        Ok(None) => failure(
            StatusCode::PRECONDITION_FAILED,
            "Instance or link changed; verify again.",
        ),
        Err(_) => failure(StatusCode::INTERNAL_SERVER_ERROR, "Could not save link."),
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AccountFence {
    account_id: String,
}
async fn unlink(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<AccountFence>,
) -> Response {
    // Unlink remains available even if the integration is disabled.
    let session = match crate::authed(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    if body.account_id != session.account_id {
        return failure(
            StatusCode::CONFLICT,
            "Account changed; reload before unlinking.",
        );
    }
    let row = match state.store.nextcloud_links(&session.account_id).await {
        Ok(rows) => rows.into_iter().find(|row| row.instance_id == id),
        Err(_) => return failure(StatusCode::INTERNAL_SERVER_ERROR, "links unavailable"),
    };
    let Some(row) = row else {
        return StatusCode::NO_CONTENT.into_response();
    };
    if !headers.contains_key("if-match") {
        return failure(
            StatusCode::PRECONDITION_REQUIRED,
            "Reload before unlinking.",
        );
    }
    if check_if_match(&headers, &etag_of(&json!(row.revision)), None, None).is_err() {
        return failure(StatusCode::PRECONDITION_FAILED, "Link changed.");
    }
    match state
        .store
        .delete_nextcloud_link(&session.account_id, &id, &row.revision)
        .await
    {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => failure(StatusCode::PRECONDITION_FAILED, "Link changed."),
        Err(_) => failure(StatusCode::INTERNAL_SERVER_ERROR, "Could not remove link."),
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FilesQuery {
    account_id: Option<String>,
    instance_id: String,
    #[serde(default)]
    path: String,
}
async fn files(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<FilesQuery>,
) -> Response {
    let session =
        match crate::user_router::feature_session(&state, &headers, "integrations.nextcloud").await
        {
            Ok(session) => session,
            Err(response) => return response,
        };
    if query
        .account_id
        .as_deref()
        .is_some_and(|id| id != session.account_id.as_str())
    {
        return failure(StatusCode::CONFLICT, "active account changed");
    }
    let (instance, link, password) = match state
        .store
        .nextcloud_authority(&session.account_id, &query.instance_id)
        .await
    {
        Ok(Some(authority)) => authority,
        _ => {
            return failure(
                StatusCode::NOT_FOUND,
                "No current link for this account; link again.",
            );
        }
    };
    let root = match crate::nextcloud_config::file_url(&instance.base_url, &link.username, "/") {
        Ok(url) => url,
        Err(reason) => return failure(StatusCode::BAD_GATEWAY, reason),
    };
    let url =
        match crate::nextcloud_config::file_url(&instance.base_url, &link.username, &query.path) {
            Ok(url) => url,
            Err(reason) => return failure(StatusCode::BAD_REQUEST, reason),
        };
    match crate::nextcloud_transport::call(
        &state,
        &session,
        &instance,
        &link.username,
        &password,
        crate::nextcloud_transport::Operation {
            url: url.clone(),
            method: mw_egress::DavMethod::Propfind,
            body: Some(("application/xml", crate::nextcloud_transport::PROPERTIES)),
            depth: Some(1),
        },
    )
    .await
    {
        Ok(bytes) => match crate::nextcloud_xml::entries(&bytes, &root, &url) {
            Ok(entries) => Json(json!({"entries":entries})).into_response(),
            Err(reason) => failure(StatusCode::BAD_GATEWAY, reason),
        },
        Err(reason) => {
            if reason == "credentials rejected; link again" {
                let _ = state
                    .store
                    .delete_nextcloud_link(&session.account_id, &query.instance_id, &link.revision)
                    .await;
            }
            failure(StatusCode::BAD_GATEWAY, reason)
        }
    }
}
