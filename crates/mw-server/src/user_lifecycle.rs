//! Admin controls for local Mailwoman access; no upstream mailbox destruction.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, check_if_match, etag_of, with_etag},
    admin_router::AdminRegistrar,
    capabilities::Capability,
    step_up::{StepUpAction, require_admin},
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use mw_store::user_lifecycle::{LifecycleChange, UserLifecycle};
use serde::Deserialize;
use serde_json::json;
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/users/{id}/legacy-block",
        Capability::UsersRead,
        legacy_get,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/users/{id}/legacy-block",
        Capability::UsersFlags,
        legacy_clear,
    );
    r.admin_endpoint(
        Method::GET,
        "/users/{id}/lifecycle",
        Capability::UsersRead,
        get,
    );
    r.admin_endpoint(
        Method::PUT,
        "/users/{id}/lifecycle",
        Capability::UsersFlags,
        put,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/users/{id}/lifecycle",
        Capability::UsersDelete,
        delete,
    );
}
async fn resolve(
    state: &AppState,
    ctx: &AdminCtx,
    id: &str,
) -> Result<UserLifecycle, AdminApiError> {
    let identity = state
        .store
        .audit_identity("user", id)
        .await?
        .ok_or_else(|| {
            AdminApiError::new(StatusCode::NOT_FOUND, "userNotFound", "User not found.")
        })?;
    let address = identity.address.ok_or_else(|| {
        AdminApiError::new(
            StatusCode::CONFLICT,
            "identityUnknown",
            "The directory identity could not be verified.",
        )
    })?;
    let policy = state.store.canonical_policy_address(&address).await?;
    ctx.require_domain(&crate::admin_ctx::domain_of(&policy))?;
    state
        .store
        .user_lifecycle(&address)
        .await?
        .ok_or_else(|| AdminApiError::new(StatusCode::NOT_FOUND, "userNotFound", "User not found."))
}
fn result(value: &UserLifecycle) -> Response {
    let body = json!({
        "status": value.status,
        "directoryAddress": value.directory_address,
        "deletedAt": value.deleted_at,
        "canRestore": false,
    });
    let tag = etag_of(&json!(value));
    with_etag(
        ([(header::CACHE_CONTROL, "private, no-store")], Json(body)),
        &tag,
    )
    .into_response()
}

async fn get(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    Ok(result(&resolve(&state, &ctx, &id).await?))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Change {
    disabled: bool,
}
async fn put(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Change>,
) -> Result<Response, AdminApiError> {
    change(
        &state,
        &ctx,
        &id,
        &headers,
        if body.disabled { "disabled" } else { "active" },
    )
    .await
}
async fn delete(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AdminApiError> {
    change(&state, &ctx, &id, &headers, "deleted").await
}
async fn change(
    state: &AppState,
    ctx: &AdminCtx,
    id: &str,
    headers: &HeaderMap,
    status: &str,
) -> Result<Response, AdminApiError> {
    let before = resolve(state, ctx, id).await?;
    require_admin(state, ctx, StepUpAction::UserManage).await?;
    let tag = headers
        .get(header::IF_MATCH)
        .and_then(|value| value.to_str().ok())
        .filter(|value| *value != "*")
        .ok_or_else(|| {
            AdminApiError::new(
                StatusCode::PRECONDITION_REQUIRED,
                "preconditionRequired",
                "Reload the user before changing access.",
            )
        })?;
    let expected = etag_of(&json!(before));
    if tag != expected {
        return Err(AdminApiError::stale(None, None));
    }
    check_if_match(headers, &expected, None, None)?;
    let bindings = state
        .store
        .lifecycle_account_bindings(&before.directory_address)
        .await?;
    let mut native = Vec::new();
    for binding in bindings {
        if state
            .store
            .verified_native_directory_login(&binding, &before.directory_address)
            .await?
            .is_some()
        {
            native.push(binding);
        }
    }
    match state
        .store
        .change_user_lifecycle(
            &before,
            status,
            &ctx.username,
            chrono::Utc::now().timestamp(),
        )
        .await?
    {
        LifecycleChange::Stale => Err(AdminApiError::stale(None, None)),
        LifecycleChange::Deleted => Err(AdminApiError::new(
            StatusCode::CONFLICT,
            "userDeleted",
            "This local access record is deleted. It cannot be reactivated here.",
        )),
        LifecycleChange::NotFound => Err(AdminApiError::new(
            StatusCode::NOT_FOUND,
            "userNotFound",
            "User not found.",
        )),
        LifecycleChange::Applied {
            state: after,
            sessions,
        } => {
            for session in &sessions {
                state.sessions.forget(session);
                state.step_up.forget_user(session);
            }
            if status != "active"
                && let Some(engine) = &state.engine
            {
                for account in native {
                    let _ = engine.unregister(&account);
                }
            }
            let action = match status {
                "deleted" => "user-local-access-deleted",
                "disabled" => "user-login-blocked",
                _ => "user-directory-block-cleared",
            };
            let target = format!("user:{}", before.directory_address);
            let previous = Some(json!({"status": before.status}));
            let changed = Some(json!({
                "status": status,
                "revokedSessions": sessions.len(),
                "upstreamDeleted": false,
            }));
            crate::admin_accounts::audit(state, ctx, action, &target, previous, changed).await;
            Ok(result(&after))
        }
    }
}

async fn legacy_snapshot(
    state: &AppState,
    id: &str,
) -> Result<(String, Option<String>, mw_admin::UserFeatureFlags), AdminApiError> {
    let mut key = format!("v6:admin:flags:{}", crate::account_gate::flag_subject(id));
    let mut raw = state.store.get_setting(&key).await?;
    if raw.is_none() && id != crate::account_gate::flag_subject(id) {
        key = format!("v6:admin:flags:{id}");
        raw = state.store.get_setting(&key).await?;
    }
    let flags = raw
        .as_deref()
        .map(serde_json::from_str)
        .transpose()
        .map_err(|_| {
            AdminApiError::new(
                StatusCode::CONFLICT,
                "flagsInvalid",
                "The legacy flags require administrator review.",
            )
        })?
        .unwrap_or_default();
    Ok((key, raw, flags))
}
async fn legacy_get(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    resolve(&state, &ctx, &id).await?;
    let (key, raw, flags) = legacy_snapshot(&state, &id).await?;
    Ok(with_etag(
        (
            [(header::CACHE_CONTROL, "private, no-store")],
            Json(json!({"disabled":flags.disabled})),
        ),
        &etag_of(&json!([key, raw])),
    )
    .into_response())
}
async fn legacy_clear(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AdminApiError> {
    let user = resolve(&state, &ctx, &id).await?;
    require_admin(&state, &ctx, StepUpAction::UserManage).await?;
    if matches!(user.status.as_str(), "deleted" | "deleting") {
        return Err(AdminApiError::new(
            StatusCode::CONFLICT,
            "userDeleted",
            "Deleted access cannot be restored.",
        ));
    }
    let (key, raw, mut flags) = legacy_snapshot(&state, &id).await?;
    let expected = etag_of(&json!([key, raw]));
    let tag = headers
        .get(header::IF_MATCH)
        .and_then(|v| v.to_str().ok())
        .filter(|v| *v != "*")
        .ok_or_else(|| {
            AdminApiError::new(
                StatusCode::PRECONDITION_REQUIRED,
                "preconditionRequired",
                "Reload the legacy block before changing it.",
            )
        })?;
    if tag != expected {
        return Err(AdminApiError::stale(None, None));
    }
    flags.disabled = false;
    let replacement = serde_json::to_string(&flags).map_err(|_| {
        AdminApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "flagsInvalid",
            "Could not encode flags.",
        )
    })?;
    if !state
        .store
        .compare_and_set_setting(&key, raw.as_deref(), &replacement)
        .await?
    {
        return Err(AdminApiError::stale(None, None));
    }
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "user-legacy-login-block-cleared",
        &format!("user:{}", user.directory_address),
        None,
        Some(json!({"directoryStatus":user.status,"otherFlagsPreserved":true})),
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}
