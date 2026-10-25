//! Feature catalogue and profile/group administration use settings model C.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson},
    admin_router::AdminRegistrar,
    capabilities::Capability,
    settings_registry as registry,
    settings_routes::UserSubject,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, Method},
    response::{IntoResponse, Response},
    routing::get,
};
use mw_store::feature_profiles::{FeatureProfile, ProfileAssignment, ProfileValue};
use mw_store::groups::UserGroup;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Preserve arbitrary existing plugin IDs without normalizing two IDs together.
pub(crate) fn ui_plugin_policy_key(id: &str) -> String {
    let digest: String = Sha256::digest(id.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("cap.ui_plugin.{digest}")
}

pub(crate) fn plugin_policy_key(id: &str) -> String {
    ui_plugin_policy_key(id).replacen("cap.ui_plugin.", "cap.integration.", 1)
}

pub(crate) async fn plugin_policy_admit(
    store: &mw_store::Store,
    plugin: &str,
    account: Option<&str>,
) -> Result<(), String> {
    let account = account.unwrap_or("");
    let username = if account.is_empty() {
        String::new()
    } else {
        match store.get_account(account).await {
            Ok(row) => row.username,
            Err(mw_store::StoreError::NotFound) => {
                let rows = store
                    .session_names_by_account(account)
                    .await
                    .map_err(|_| "plugin identity unavailable")?;
                let mut names = std::collections::BTreeSet::new();
                for row in rows {
                    names.insert(
                        store
                            .canonical_policy_address(&row.username)
                            .await
                            .map_err(|_| "plugin identity invalid")?,
                    );
                }
                if names.len() != 1 {
                    return Err("plugin identity missing or ambiguous".into());
                }
                names.into_iter().next().unwrap()
            }
            Err(_) => return Err("plugin identity unavailable".into()),
        }
    };
    let subject = mw_store::settings_policy::SettingSubject {
        account_id: account,
        domain: username.rsplit_once('@').map(|(_, d)| d).unwrap_or(""),
        address: &username,
    };
    let mut keys = vec!["cap.plugins".to_string(), plugin_policy_key(plugin)];
    match plugin {
        "bridge-graph" => keys.push("cap.integration.microsoft".into()),
        "bridge-gmail" => keys.push("cap.integration.google".into()),
        "nextcloud" | "nextcloud-plugin" => keys.push("cap.integration.nextcloud".into()),
        _ => {}
    }
    for key in keys {
        let value = registry::effective_for(store, &key, &subject)
            .await
            .map_err(|_| "plugin policy unavailable")?;
        if !value.is_some_and(|v| v.value.as_bool() == Some(true)) {
            return Err(format!("feature_disabled: {key}"));
        }
    }
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FeatureDef {
    pub id: &'static str,
    pub capability_keys: &'static [&'static str],
    pub limit_keys: &'static [&'static str],
    pub setting_keys: &'static [&'static str],
    pub rest_entry_points: &'static [&'static str],
    pub jmap_families: &'static [&'static str],
}
macro_rules! feature {
    ($id:literal,$cap:literal,$rest:expr,$jmap:expr) => {
        FeatureDef {
            id: $id,
            capability_keys: &[$cap],
            limit_keys: &[],
            setting_keys: &[],
            rest_entry_points: $rest,
            jmap_families: $jmap,
        }
    };
}
pub(crate) static CATALOGUE: &[FeatureDef] = &[
    feature!(
        "integrations.nextcloud",
        "cap.integration.nextcloud",
        &["/api/nextcloud"],
        &[]
    ),
    feature!("integrations.google", "cap.integration.google", &[], &[]),
    feature!(
        "integrations.microsoft",
        "cap.integration.microsoft",
        &[],
        &[]
    ),
    feature!("ai_assistant", "cap.ai_assistant", &["/api/assist"], &[]),
    feature!("plugins.ui", "cap.plugins", &["/api/ui-plugins"], &[]),
    feature!(
        "calendar",
        "cap.pim.calendar",
        &[],
        &["Calendar", "CalendarEvent"]
    ),
    feature!(
        "contacts",
        "cap.pim.contacts",
        &[],
        &["AddressBook", "ContactCard", "ContactGroup"]
    ),
    feature!("tasks", "cap.pim.tasks", &[], &["TaskList", "Task"]),
    feature!("notes", "cap.pim.notes", &[], &["Notebook", "Note"]),
    feature!("export", "cap.export", &["/api/export"], &[]),
    feature!("import", "cap.import", &["/api/import"], &[]),
    feature!("api_keys", "cap.api_keys", &["/api/keys"], &[]),
    feature!("oauth_apps", "cap.oauth_apps", &["/api/oauth"], &[]),
    feature!("rules", "cap.rules", &[], &[]),
    feature!("rules.forward", "cap.rules.forward", &[], &[]),
    feature!(
        "privacy.remote_images",
        "cap.remote_images",
        &["/api/rc"],
        &[]
    ),
    feature!("sieve.raw", "cap.sieve.raw_edit", &[], &["SieveScript"]),
    feature!(
        "forwarding.external",
        "cap.sieve.forward_external",
        &[],
        &[]
    ),
    feature!("customisation.css", "cap.user_css", &[], &[]),
    feature!("customisation.background", "cap.user_background", &[], &[]),
];

fn enforcement(id: &str) -> &'static str {
    match id {
        "ai_assistant"
        | "calendar"
        | "contacts"
        | "tasks"
        | "notes"
        | "rules"
        | "export"
        | "import"
        | "integrations.nextcloud"
        | "plugins.ui"
        | "sieve.raw"
        | "forwarding.external" => "enforced",
        "integrations.google" | "integrations.microsoft" => "enforced",
        _ => "pending",
    }
}

pub(crate) fn user_router() -> Router<AppState> {
    Router::new().route("/api/features", get(user_features))
}
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/features/groups/{id}/members",
        Capability::SettingsRead,
        get_members,
    );
    r.admin_endpoint(
        Method::GET,
        "/features",
        Capability::SettingsRead,
        admin_features,
    );
    r.admin_endpoint(
        Method::PUT,
        "/features/profiles/{id}",
        Capability::SettingsWrite,
        put_profile,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/features/profiles/{id}",
        Capability::SettingsWrite,
        delete_profile,
    );
    r.admin_endpoint(
        Method::PUT,
        "/features/groups/{id}",
        Capability::SettingsWrite,
        put_group,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/features/groups/{id}",
        Capability::SettingsWrite,
        delete_group,
    );
    r.admin_endpoint(
        Method::PUT,
        "/features/groups/{id}/members",
        Capability::SettingsUserValues,
        put_members,
    );
    r.admin_endpoint(
        Method::PUT,
        "/features/assignments",
        Capability::SettingsUserValues,
        put_assignment,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/features/assignments",
        Capability::SettingsUserValues,
        delete_assignment,
    );
}

async fn user_features(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    let subject = match UserSubject::resolved(&state.store, &session).await {
        Ok(v) => v,
        Err(_) => return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let values = match registry::effective_all(&state.store, &subject.as_subject(), |d| {
        d.key.starts_with("cap.") || d.key.starts_with("limit.")
    })
    .await
    {
        Ok(v) => v,
        Err(_) => return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let mut features = serde_json::Map::new();
    for f in CATALOGUE {
        let capabilities: serde_json::Map<String, Value> = values
            .iter()
            .filter(|v| f.capability_keys.contains(&v.key.as_str()))
            .map(|v| (v.key.clone(), v.to_user_json()))
            .collect();
        let enabled = f.capability_keys.iter().all(|key| {
            values
                .iter()
                .any(|v| v.key == *key && v.value.as_bool() == Some(true))
        });
        let limits: serde_json::Map<String, Value> = values
            .iter()
            .filter(|v| f.limit_keys.contains(&v.key.as_str()))
            .map(|v| (v.key.clone(), v.to_user_json()))
            .collect();
        features.insert(
            f.id.into(),
            json!({"enabled":enabled,"capabilities":capabilities,"limits":limits,"enforcement":enforcement(f.id)}),
        );
    }
    (
        [(axum::http::header::CACHE_CONTROL, "no-store")],
        Json(json!({"features":features})),
    )
        .into_response()
}

async fn admin_features(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsRead)?;
    let mut plugin_keys: Vec<Value> = state
        .store
        .list_ui_plugins()
        .await?
        .into_iter()
        .map(|p| json!({"id":p.id,"key":ui_plugin_policy_key(&p.id),"kind":"ui-plugin"}))
        .collect();
    plugin_keys.extend(state.store.list_plugins().await?.into_iter().map(|p|json!({"id":p.id,"key":plugin_policy_key(&p.id),"kind":"wasm-plugin","enforcement":"enforced"})));
    let catalogue: Vec<Value> = CATALOGUE
        .iter()
        .map(|f| {
            let mut value = serde_json::to_value(f).expect("static feature definition");
            value["enforcement"] = enforcement(f.id).into();
            value
        })
        .collect();
    Ok(Json(json!({"catalogue":catalogue,"integrationPolicyKeys":plugin_keys,"profiles":state.store.feature_profiles().await?,"groups":state.store.user_groups().await?,"assignments":state.store.feature_profile_assignments().await?})).into_response())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileInput {
    name: String,
    values: BTreeMap<String, ProfileValue>,
}
async fn put_profile(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    AdminJson(mut input): AdminJson<ProfileInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    if input.name.trim().is_empty() || input.name.len() > 128 || input.values.len() > 256 {
        return Err(AdminApiError::invalid(
            "invalidProfile",
            "invalid profile size or name",
            "/name",
        ));
    }
    for (key, value) in &mut input.values {
        let def = registry::lookup(key).ok_or_else(|| {
            AdminApiError::invalid(
                "unknownSetting",
                "unknown setting",
                format!("/values/{key}"),
            )
        })?;
        registry::refuse_if_dedicated(def).map_err(refusal)?;
        registry::refuse_if_pinned(def, &registry::process_env).map_err(refusal)?;
        if !def
            .scopes
            .allows(mw_store::settings_policy::SettingScope::Profile)
        {
            return Err(AdminApiError::invalid(
                "invalidScope",
                "setting cannot be assigned to a profile",
                format!("/values/{key}"),
            ));
        }
        value.value = registry::validate(def, &value.value, &format!("/values/{key}/value"))
            .map_err(refusal)?;
    }
    state
        .store
        .put_feature_profile(
            &FeatureProfile {
                id: id.clone(),
                name: input.name,
                values: input.values,
                builtin: false,
                updated_at: chrono::Utc::now().timestamp(),
            },
            &ctx.username,
        )
        .await?;
    audit(&state, &ctx, "profile-changed", &id).await;
    Ok(Json(json!({"ok":true})).into_response())
}
async fn delete_profile(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    state.store.delete_feature_profile(&id).await?;
    audit(&state, &ctx, "profile-deleted", &id).await;
    Ok(Json(json!({"ok":true})).into_response())
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GroupInput {
    name: String,
    #[serde(default)]
    domain: String,
}
async fn put_group(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    AdminJson(input): AdminJson<GroupInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    if input.name.trim().is_empty() || input.name.len() > 128 {
        return Err(AdminApiError::invalid(
            "invalidGroup",
            "invalid group name",
            "/name",
        ));
    }
    if state
        .store
        .user_groups()
        .await?
        .iter()
        .any(|g| g.id == id && g.source != "manual")
    {
        return Err(AdminApiError::conflict(
            "directoryManaged",
            "directory group cannot be edited manually",
        ));
    }
    state
        .store
        .put_user_group(&UserGroup {
            id: id.clone(),
            name: input.name,
            domain: input.domain,
            source: "manual".into(),
            ldap_dn: None,
            created_at: chrono::Utc::now().timestamp(),
        })
        .await?;
    audit(&state, &ctx, "group-changed", &id).await;
    Ok(Json(json!({"ok":true})).into_response())
}
async fn delete_group(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    state.store.delete_user_group(&id).await?;
    audit(&state, &ctx, "group-deleted", &id).await;
    Ok(Json(json!({"ok":true})).into_response())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MembersInput {
    members: Vec<String>,
}
async fn get_members(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsRead)?;
    if !state.store.user_groups().await?.iter().any(|g| g.id == id) {
        return Err(AdminApiError::not_found("unknown group"));
    }
    Ok(Json(json!({"members":state.store.user_group_members(&id).await?})).into_response())
}
async fn put_members(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    AdminJson(input): AdminJson<MembersInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsUserValues)?;
    let group = state
        .store
        .user_groups()
        .await?
        .into_iter()
        .find(|g| g.id == id)
        .ok_or_else(|| AdminApiError::not_found("unknown group"))?;
    if group.source != "manual" {
        return Err(AdminApiError::conflict(
            "directoryManaged",
            "directory group membership cannot be edited manually",
        ));
    }
    if input.members.len() > 10000
        || input
            .members
            .iter()
            .any(|id| id.is_empty() || id.len() > 256)
    {
        return Err(AdminApiError::invalid(
            "invalidMembers",
            "invalid group members",
            "/members",
        ));
    }
    state
        .store
        .replace_user_group_members(&id, &input.members)
        .await?;
    audit(&state, &ctx, "group-members-changed", &id).await;
    Ok(Json(json!({"ok":true})).into_response())
}
async fn put_assignment(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(input): AdminJson<ProfileAssignment>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsUserValues)?;
    validate_assignment(&state, &input).await?;
    state.store.put_feature_profile_assignment(&input).await?;
    audit(&state, &ctx, "profile-assigned", &input.profile_id).await;
    Ok(Json(json!({"ok":true})).into_response())
}
async fn delete_assignment(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(input): AdminJson<ProfileAssignment>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsUserValues)?;
    validate_assignment(&state, &input).await?;
    state
        .store
        .delete_feature_profile_assignment(&input)
        .await?;
    audit(&state, &ctx, "profile-unassigned", &input.profile_id).await;
    Ok(Json(json!({"ok":true})).into_response())
}

async fn validate_assignment(
    state: &AppState,
    input: &ProfileAssignment,
) -> Result<(), AdminApiError> {
    if !matches!(input.scope_kind.as_str(), "domain" | "group" | "user")
        || input.scope_id.is_empty()
        || input.scope_id.len() > 256
    {
        return Err(AdminApiError::invalid(
            "invalidAssignment",
            "invalid assignment scope",
            "/scopeKind",
        ));
    }
    if !state
        .store
        .feature_profiles()
        .await?
        .iter()
        .any(|p| p.id == input.profile_id)
    {
        return Err(AdminApiError::not_found("unknown profile"));
    }
    if input.scope_kind == "group"
        && !state
            .store
            .user_groups()
            .await?
            .iter()
            .any(|g| g.id == input.scope_id)
    {
        return Err(AdminApiError::not_found("unknown group"));
    }
    if input.scope_kind == "domain" {
        let domain = state.store.canonical_policy_domain(&input.scope_id).await?;
        if state.store.get_domain_config(&domain).await?.is_none() {
            return Err(AdminApiError::not_found("unknown domain"));
        }
    }
    Ok(())
}
async fn audit(state: &AppState, ctx: &AdminCtx, action: &str, id: &str) {
    let entry = mw_admin::AuditLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor: ctx.username.clone(),
        actor_kind: mw_admin::ActorKind::Admin,
        action: action.into(),
        target: Some(format!("feature:{id}")),
        detail_json: "{}".into(),
        ip: None,
    };
    if let Err(e) = state.v6.admin.audit(entry).await {
        tracing::warn!("feature audit append failed: {e}");
    }
}

fn refusal(r: registry::Refusal) -> AdminApiError {
    let mut error = AdminApiError::new(
        axum::http::StatusCode::from_u16(r.status)
            .unwrap_or(axum::http::StatusCode::UNPROCESSABLE_ENTITY),
        r.code,
        r.detail,
    );
    if let Some(field) = r.field {
        error = error.at(field);
    }
    if let Some((key, value)) = r.extra {
        error = error.with(key, value);
    }
    error
}

#[cfg(test)]
mod integration_policy_tests {
    use super::*;
    #[tokio::test]
    async fn account_policy_uses_persisted_alias_identity_and_never_guesses_account_ids() {
        let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        store
            .put_domain_config(&mw_store::DomainConfigRow {
                name: "canonical.test".into(),
                upstream_json: "{}".into(),
                allowlist_json: "[]".into(),
                blocklist_json: "[]".into(),
                enabled: true,
                preferred: false,
                settings_json: "{}".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
        store
            .put_domain_alias("alias.test", "canonical.test")
            .await
            .unwrap();
        let account = store
            .create_account(
                &mw_store::NewAccount {
                    kind: mw_store::AccountKind::Imap,
                    host: "mail.test",
                    port: 993,
                    tls: "implicit",
                    username: "User@Alias.Test",
                    sync_policy_json: "{}",
                },
                &mw_store::Credentials {
                    username: "User@Alias.Test".into(),
                    password: "fixture".into(),
                },
            )
            .await
            .unwrap();
        let key = plugin_policy_key("bridge-ews");
        store
            .put_setting_value(&mw_store::settings_policy::SettingValueRow {
                scope: mw_store::settings_policy::SettingScope::Domain,
                scope_id: "canonical.test".into(),
                key: key.clone(),
                value: "false".into(),
                locked: true,
                updated_by: "admin".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
        assert!(
            plugin_policy_admit(&store, "bridge-ews", Some(&account))
                .await
                .unwrap_err()
                .contains("feature_disabled")
        );
        assert!(
            plugin_policy_admit(&store, "bridge-ews", None)
                .await
                .is_ok()
        );
        assert!(
            plugin_policy_admit(&store, "bridge-ews", Some("user@canonical.test"))
                .await
                .is_err()
        );
        assert_eq!(
            store.get_account(&account).await.unwrap().username,
            "User@Alias.Test"
        );
    }
}
