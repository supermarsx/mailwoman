//! Admin roles: what an account may do, and the roles API (t32-ext §M, e24c).
//!
//! * [`grants_for`] turns an admin's account rows into an [`AdminCtx`]: the union of
//!   the capabilities of their roles (a built-in role's from
//!   [`crate::capabilities::BUILTIN_ROLES`], a custom role's from
//!   `admin_role_capability`), each for the domains the assignment names (`NULL` =
//!   all), plus `allow` overrides (all domains), minus `deny` overrides. Deny wins.
//! * `GET/POST /admin/roles`, `PUT/DELETE /admin/roles/{id}`. Built-in roles are
//!   read-only (`409 builtinRole`); a role may only contain capabilities the writer
//!   holds for all domains (`403 capabilityNotHeld`, t32-cfg §8 A19).
//! * [`last_super_admin_guard`]: the last enabled account holding Super admin for
//!   all domains cannot be disabled, deleted or lose the role (`409
//!   lastSuperAdmin`).

use std::collections::BTreeMap;

use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};

use mw_store::{AdminAccountRow, AdminRoleGrant, AdminRoleRow};

use crate::AppState;
use crate::admin_ctx::{
    AdminApiError, AdminCtx, AdminJson, DomainScope, check_if_match, etag_of, with_etag,
};
use crate::admin_router::AdminRegistrar;
use crate::capabilities::{BUILTIN_ROLES, Capability, SUPER_ADMIN, builtin_role};

/// The roles routes.
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/roles", Capability::AdminsRead, list_roles);
    r.admin_endpoint(Method::POST, "/roles", Capability::RolesManage, create_role);
    r.admin_endpoint(
        Method::PUT,
        "/roles/{id}",
        Capability::RolesManage,
        update_role,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/roles/{id}",
        Capability::RolesManage,
        delete_role,
    );
}

/// Make sure the built-in roles have their rows (assignments refer to them).
pub(crate) async fn ensure_builtin_roles(state: &AppState) -> Result<(), AdminApiError> {
    let rows: Vec<(&str, &str)> = BUILTIN_ROLES.iter().map(|r| (r.id, r.name)).collect();
    state.store.ensure_builtin_admin_roles(&rows).await?;
    Ok(())
}

/// The capabilities `role` grants.
pub(crate) fn role_capabilities(role: &AdminRoleRow) -> Vec<Capability> {
    if role.builtin
        && let Some(b) = builtin_role(&role.id)
    {
        return (b.capabilities)();
    }
    // An id this build does not know (a role written by a newer release) grants
    // nothing here.
    role.capabilities
        .iter()
        .filter_map(|c| Capability::parse(c))
        .collect()
}

/// The effective grants of an account, and the names of its roles.
pub(crate) fn effective(
    assignments: &[AdminRoleGrant],
    roles: &[AdminRoleRow],
    overrides: &[(String, String)],
) -> (BTreeMap<Capability, DomainScope>, Vec<String>) {
    let mut grants: BTreeMap<Capability, DomainScope> = BTreeMap::new();
    let mut names = Vec::new();
    for a in assignments {
        let Some(role) = roles.iter().find(|r| r.id == a.role_id) else {
            continue;
        };
        names.push(role.name.clone());
        let scope = match &a.domains {
            None => DomainScope::All,
            Some(list) => {
                DomainScope::Domains(list.iter().map(|d| d.to_ascii_lowercase()).collect())
            }
        };
        for cap in role_capabilities(role) {
            grants
                .entry(cap)
                .and_modify(|s| s.union(&scope))
                .or_insert_with(|| scope.clone());
        }
    }
    for (cap, effect) in overrides {
        let Some(cap) = Capability::parse(cap) else {
            continue;
        };
        if effect == "allow" {
            grants.insert(cap, DomainScope::All);
        }
    }
    for (cap, effect) in overrides {
        if effect == "deny"
            && let Some(cap) = Capability::parse(cap)
        {
            grants.remove(&cap);
        }
    }
    (grants, names)
}

/// The signed-in admin `username` with their grants. `401` for an account that is
/// disabled or no longer exists. Before the first sign-in has created any account,
/// the environment admin holds every capability.
pub(crate) async fn grants_for(
    state: &AppState,
    username: &str,
) -> Result<AdminCtx, AdminApiError> {
    let Some(acc) = state.store.get_admin_account_by_username(username).await? else {
        if crate::admin_accounts::is_env_admin(state, username)
            && state.store.count_admin_accounts().await? == 0
        {
            let mut ctx = AdminCtx::all_capabilities(username);
            ctx.roles = vec!["Super admin".to_string()];
            return Ok(ctx);
        }
        return Err(AdminApiError::unauthenticated());
    };
    if acc.disabled {
        return Err(AdminApiError::unauthenticated());
    }
    let assignments = state.store.admin_account_roles(&acc.id).await?;
    let roles = state.store.list_admin_roles().await?;
    let overrides = state.store.admin_overrides(&acc.id).await?;
    let (grants, names) = effective(&assignments, &roles, &overrides);
    let mut ctx = AdminCtx::with_grants(username, grants);
    ctx.account_id = Some(acc.id);
    ctx.display_name = Some(acc.display_name);
    ctx.roles = names;
    Ok(ctx)
}

/// `403 capabilityNotHeld` unless `ctx` holds every one of `caps` for all domains.
pub(crate) fn require_can_grant(ctx: &AdminCtx, caps: &[Capability]) -> Result<(), AdminApiError> {
    for cap in caps {
        if !ctx.has_unscoped(*cap) {
            return Err(AdminApiError::capability_not_held(*cap));
        }
    }
    Ok(())
}

/// Does `account` hold Super admin for all domains, enabled?
async fn is_full_super(state: &AppState, account: &AdminAccountRow) -> Result<bool, AdminApiError> {
    if account.disabled
        || state
            .store
            .admin_overrides(&account.id)
            .await?
            .iter()
            .any(|(_, effect)| effect == "deny")
    {
        return Ok(false);
    }
    Ok(state
        .store
        .admin_account_roles(&account.id)
        .await?
        .iter()
        .any(|g| g.role_id == SUPER_ADMIN && g.domains.is_none()))
}

/// `409 lastSuperAdmin` when `account` is the last enabled full Super admin and
/// the change would take that away.
pub(crate) async fn last_super_admin_guard(
    state: &AppState,
    account: &AdminAccountRow,
) -> Result<(), AdminApiError> {
    if !is_full_super(state, account).await? {
        return Ok(());
    }
    for other in state.store.list_admin_accounts().await? {
        if other.id != account.id && is_full_super(state, &other).await? {
            return Ok(());
        }
    }
    Err(AdminApiError::conflict(
        "lastSuperAdmin",
        "This is the last enabled Super admin. Give another account Super admin first.",
    ))
}

// ─── roles API ──────────────────────────────────────────────────────────────────

fn role_json(role: &AdminRoleRow, holders: usize) -> Value {
    let caps: Vec<&str> = role_capabilities(role).iter().map(|c| c.as_str()).collect();
    json!({
        "id": role.id,
        "name": role.name,
        "builtin": role.builtin,
        "capabilities": caps,
        "holders": holders,
    })
}

async fn role_view(state: &AppState, role: &AdminRoleRow) -> Result<Value, AdminApiError> {
    let holders = state.store.admin_accounts_with_role(&role.id).await?.len();
    Ok(role_json(role, holders))
}

/// `GET /admin/roles`.
async fn list_roles(
    State(state): State<AppState>,
    _ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    ensure_builtin_roles(&state).await?;
    let mut out = Vec::new();
    for role in state.store.list_admin_roles().await? {
        let mut v = role_view(&state, &role).await?;
        v["etag"] = Value::String(etag_of(&v));
        out.push(v);
    }
    Ok(axum::Json(json!({ "roles": out })).into_response())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RoleBody {
    name: String,
    capabilities: Vec<String>,
    #[serde(default)]
    dry_run: bool,
}

/// Check a role body: a name of 1–64 characters not used by another role, and
/// known capability ids the writer may grant.
async fn validate_role(
    state: &AppState,
    ctx: &AdminCtx,
    body: &RoleBody,
    id: Option<&str>,
) -> Result<(String, Vec<Capability>), AdminApiError> {
    let name = body.name.trim().to_string();
    if name.is_empty() || name.chars().count() > 64 {
        return Err(AdminApiError::invalid(
            "invalidName",
            "A role name has 1 to 64 characters.",
            "/name",
        ));
    }
    let roles = state.store.list_admin_roles().await?;
    if roles
        .iter()
        .any(|r| r.name.eq_ignore_ascii_case(&name) && Some(r.id.as_str()) != id)
    {
        return Err(AdminApiError::invalid(
            "nameTaken",
            format!("A role named {name} already exists."),
            "/name",
        ));
    }
    let mut caps = Vec::new();
    for (i, id) in body.capabilities.iter().enumerate() {
        let Some(cap) = Capability::parse(id) else {
            return Err(AdminApiError::invalid(
                "unknownCapability",
                format!("{id} is not a capability."),
                format!("/capabilities/{i}"),
            ));
        };
        if !caps.contains(&cap) {
            caps.push(cap);
        }
    }
    require_can_grant(ctx, &caps)?;
    Ok((name, caps))
}

/// `POST /admin/roles {name, capabilities}` → `201` with the role.
async fn create_role(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(body): AdminJson<RoleBody>,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::AdminRoleChange)
        .await?;
    ensure_builtin_roles(&state).await?;
    let (name, caps) = validate_role(&state, &ctx, &body, None).await?;
    if body.dry_run {
        return Ok(axum::Json(json!({ "impact": { "accounts": 0 } })).into_response());
    }
    let id = uuid::Uuid::new_v4().to_string();
    let ids: Vec<String> = caps.iter().map(|c| c.as_str().to_string()).collect();
    state.store.put_admin_role(&id, &name, &ids).await?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "admin-role-created",
        &format!("role:{id}"),
        None,
        Some(json!({ "name": name, "capabilities": ids })),
    )
    .await;
    let role = state
        .store
        .get_admin_role(&id)
        .await?
        .ok_or_else(|| AdminApiError::internal("role vanished after insert"))?;
    let v = role_view(&state, &role).await?;
    let tag = etag_of(&v);
    Ok(with_etag((StatusCode::CREATED, axum::Json(v)), &tag))
}

async fn custom_role(state: &AppState, id: &str) -> Result<AdminRoleRow, AdminApiError> {
    let role = state
        .store
        .get_admin_role(id)
        .await?
        .ok_or_else(|| AdminApiError::not_found("No such role."))?;
    if role.builtin {
        return Err(AdminApiError::conflict(
            "builtinRole",
            "Built-in roles cannot be changed. Clone it into a new role instead.",
        ));
    }
    Ok(role)
}

/// `PUT /admin/roles/{id} {name, capabilities}` (`If-Match` honoured).
async fn update_role(
    State(state): State<AppState>,
    ctx: AdminCtx,
    UrlPath(id): UrlPath<String>,
    headers: HeaderMap,
    AdminJson(body): AdminJson<RoleBody>,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::AdminRoleChange)
        .await?;
    let role = custom_role(&state, &id).await?;
    let before = role_view(&state, &role).await?;
    check_if_match(&headers, &etag_of(&before), None, None)?;
    // Changing a role changes what its holders may do: the writer must hold what
    // the role grants now, too, or a role could be used to strip a capability the
    // writer cannot give back.
    require_can_grant(&ctx, &role_capabilities(&role))?;
    let (name, caps) = validate_role(&state, &ctx, &body, Some(&id)).await?;
    if body.dry_run {
        let holders = state.store.admin_accounts_with_role(&id).await?;
        return Ok(axum::Json(json!({ "impact": { "accounts": holders.len() } })).into_response());
    }
    let ids: Vec<String> = caps.iter().map(|c| c.as_str().to_string()).collect();
    state.store.put_admin_role(&id, &name, &ids).await?;
    let role = custom_role(&state, &id).await?;
    let after = role_view(&state, &role).await?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "admin-role-updated",
        &format!("role:{id}"),
        Some(before),
        Some(after.clone()),
    )
    .await;
    let tag = etag_of(&after);
    Ok(with_etag(axum::Json(after), &tag))
}

/// `DELETE /admin/roles/{id}` — the role and its assignments.
async fn delete_role(
    State(state): State<AppState>,
    ctx: AdminCtx,
    UrlPath(id): UrlPath<String>,
    headers: HeaderMap,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::AdminRoleChange)
        .await?;
    let role = custom_role(&state, &id).await?;
    require_can_grant(&ctx, &role_capabilities(&role))?;
    let before = role_view(&state, &role).await?;
    check_if_match(&headers, &etag_of(&before), None, None)?;
    state.store.delete_admin_role(&id).await?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "admin-role-deleted",
        &format!("role:{id}"),
        Some(before),
        None,
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn role(id: &str, builtin: bool, caps: &[&str]) -> AdminRoleRow {
        AdminRoleRow {
            id: id.into(),
            name: id.into(),
            builtin,
            capabilities: caps.iter().map(|c| c.to_string()).collect(),
        }
    }

    #[test]
    fn union_scope_and_deny_wins() {
        let roles = vec![
            role("helpdesk", true, &[]),
            role("custom", false, &["logs.read", "not.a.capability"]),
        ];
        let assignments = vec![
            AdminRoleGrant {
                role_id: "helpdesk".into(),
                domains: Some(vec!["A.example".into()]),
            },
            AdminRoleGrant {
                role_id: "custom".into(),
                domains: None,
            },
        ];
        let overrides = vec![
            ("users.read".to_string(), "deny".to_string()),
            ("audit.export".to_string(), "allow".to_string()),
        ];
        let (g, names) = effective(&assignments, &roles, &overrides);
        assert_eq!(names, vec!["helpdesk", "custom"]);
        assert!(!g.contains_key(&Capability::UsersRead), "deny wins");
        assert_eq!(g[&Capability::AuditExport], DomainScope::All);
        assert_eq!(g[&Capability::LogsRead], DomainScope::All);
        assert!(g[&Capability::UsersDisable].allows("a.example"));
        assert!(!g[&Capability::UsersDisable].allows("b.example"));
    }
}
