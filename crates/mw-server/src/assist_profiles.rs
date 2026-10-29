//! AI provider profiles (t39 N8): the admin routes, and the per-request
//! resolution of the profile that applies to a signed-in user.
//!
//! # Which profile applies
//! The settings registry key `assist.profile` holds a profile id. It has a
//! deployment value, per-domain values and per-user values set by an
//! administrator, with the registry's locks (`settings_registry::effective_for`
//! does the precedence: a per-user value over a domain value over the deployment
//! value, unless a lock at a wider scope holds). An empty value means the
//! deployment endpoint of `PUT /admin/assist` (the adapter the gateway was built
//! with). A value naming a profile that does not exist, or whose key does not
//! open, makes Assist unavailable for that user rather than sending their request
//! somewhere else.
//!
//! # What does not change
//! The gateway's pipeline runs unchanged in front of the profile's adapter:
//! the `assist_config` on/off switch and kill switch, the capability grants, the
//! data-class ceiling, redaction, the rate limit, and the content-free audit row
//! (which records the profile's endpoint host). Semantic search keeps the
//! deployment endpoint, because one search index needs one embedding model.
//!
//! # Egress
//! Every profile request is resolved and pinned through the `mw-egress` address
//! gate ([`mw_assist::profiles::ProfileAdapter`]). A loopback or private address
//! is reached only when the endpoint's `host:port` has a grant here
//! (`/admin/assist/egress-grants`, step-up required, audited) — the pattern of the
//! plugin egress grants (`egress_destinations.rs`).
//!
//! # The key
//! Write-only. Sealed by `mw-store` (bound to id, dialect and endpoint), opened
//! only to build the adapter for one request or test, never returned, never in an
//! audit detail or a log line.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::Method,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::{Value, json};

use mw_assist::profiles::{
    self as profiles, Dialect, EgressAccess, PRESETS, ProfileAdapter, ProfileParams, ProfileSpec,
};
use mw_assist::{DataScope, EndpointAdapter};
use mw_store::assist_profiles::{AssistEgressGrantRow, AssistProfileRow, ProfileKey};

use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson},
    admin_router::AdminRegistrar,
    capabilities::Capability,
    settings_routes::UserSubject,
};

/// The settings registry key that names a user's profile.
pub(crate) const PROFILE_KEY: &str = "assist.profile";

/// At most this many profiles and grants.
const MAX_PROFILES: usize = 200;
const MAX_GRANTS: usize = 64;

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/assist/presets",
        Capability::AiAssistantRead,
        presets,
    );
    r.admin_endpoint(
        Method::GET,
        "/assist/profiles",
        Capability::AiAssistantRead,
        list,
    );
    r.admin_endpoint(
        Method::POST,
        "/assist/profiles",
        Capability::AiAssistantWrite,
        create,
    );
    r.admin_endpoint(
        Method::PUT,
        "/assist/profiles/{id}",
        Capability::AiAssistantWrite,
        update,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/assist/profiles/{id}",
        Capability::AiAssistantWrite,
        remove,
    );
    r.admin_endpoint(
        Method::POST,
        "/assist/profiles/{id}/test",
        Capability::AiAssistantWrite,
        test,
    );
    r.admin_endpoint(
        Method::GET,
        "/assist/egress-grants",
        Capability::AiAssistantRead,
        list_grants,
    );
    r.admin_endpoint(
        Method::POST,
        "/assist/egress-grants",
        Capability::AiAssistantWrite,
        create_grant,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/assist/egress-grants/{id}",
        Capability::AiAssistantWrite,
        remove_grant,
    );
}

// ── resolution ───────────────────────────────────────────────────────────────

/// Which adapter a user's Assist request goes to.
pub(crate) enum Resolved {
    /// No profile is assigned: the gateway's own (deployment) adapter.
    Deployment,
    /// The assigned profile's adapter.
    Profile(Arc<dyn EndpointAdapter>),
    /// A profile is assigned but cannot be used (missing, or its key does not
    /// open). Assist is off for this user until an administrator fixes it.
    Unavailable,
}

/// The profile id assigned to `session`'s user, or `""` for none.
async fn assigned_id(state: &AppState, session: &mw_store::Session) -> Result<String, ()> {
    let subject = UserSubject::resolved(&state.store, session)
        .await
        .map_err(|_| ())?;
    let effective =
        crate::settings_registry::effective_for(&state.store, PROFILE_KEY, &subject.as_subject())
            .await
            .map_err(|_| ())?;
    Ok(effective
        .and_then(|e| e.value.as_str().map(str::to_string))
        .unwrap_or_default())
}

/// The normalised grant destinations.
async fn grant_destinations(state: &AppState) -> Result<Vec<String>, mw_store::StoreError> {
    Ok(state
        .store
        .list_assist_egress_grants()
        .await?
        .into_iter()
        .map(|g| g.destination)
        .collect())
}

/// Build the adapter for a stored profile, its key opened and its egress access
/// decided from the grants stored now.
async fn adapter_for(
    state: &AppState,
    row: &AssistProfileRow,
) -> Result<ProfileAdapter, mw_store::StoreError> {
    let api_key = state
        .store
        .assist_profile_key(&row.id)
        .await?
        .unwrap_or_default();
    let params: ProfileParams = serde_json::from_str(&row.params_json).unwrap_or_default();
    let dialect = Dialect::parse(&row.dialect).unwrap_or(Dialect::Openai);
    let grants = grant_destinations(state).await?;
    let access = EgressAccess::for_endpoint(&row.endpoint, &grants);
    Ok(ProfileAdapter::new(
        ProfileSpec {
            id: row.id.clone(),
            dialect,
            endpoint: row.endpoint.clone(),
            model: row.model.clone(),
            api_key,
            params,
        },
        access,
    ))
}

/// The adapter `session`'s Assist requests go to (see the module docs).
pub(crate) async fn resolve(state: &AppState, session: &mw_store::Session) -> Resolved {
    let Ok(id) = assigned_id(state, session).await else {
        tracing::warn!("assist profile: the user's assignment could not be read");
        return Resolved::Unavailable;
    };
    if id.is_empty() {
        return Resolved::Deployment;
    }
    let row = match state.store.get_assist_profile(&id).await {
        Ok(Some(row)) => row,
        Ok(None) => {
            tracing::warn!("assist profile {id} is assigned but does not exist; Assist is off");
            return Resolved::Unavailable;
        }
        Err(e) => {
            tracing::warn!("assist profile {id} could not be read: {e}");
            return Resolved::Unavailable;
        }
    };
    match adapter_for(state, &row).await {
        Ok(adapter) => Resolved::Profile(adapter.shared()),
        Err(e) => {
            tracing::warn!("assist profile {id} could not be opened: {e}");
            Resolved::Unavailable
        }
    }
}

/// `scope` without the accounts that are zero-access (or whose state cannot be
/// read). Context from those accounts is then dropped by redaction as outside
/// the ceiling, whatever the client sent and whatever the deployment allows: the
/// server must not forward plaintext of an account whose mail it is not meant to
/// see.
pub(crate) async fn without_zero_access(state: &AppState, mut scope: DataScope) -> DataScope {
    let mut kept = Vec::with_capacity(scope.accounts.len());
    for account in std::mem::take(&mut scope.accounts) {
        match state.store.get_zeroaccess(&account).await {
            Ok(Some(row)) if row.enabled => {}
            Ok(_) => kept.push(account),
            Err(e) => tracing::warn!(
                "zero-access state of an Assist scope account could not be read ({e}); \
                 its content is withheld"
            ),
        }
    }
    scope.accounts = kept;
    scope
}

// ── wire ─────────────────────────────────────────────────────────────────────

fn profile_wire(row: &AssistProfileRow, grants: &[String]) -> Value {
    let params: Value = serde_json::from_str(&row.params_json).unwrap_or_else(|_| json!({}));
    let access = EgressAccess::for_endpoint(&row.endpoint, grants);
    json!({
        "id": row.id,
        "name": row.name,
        "preset": row.preset,
        "dialect": row.dialect,
        "endpoint": row.endpoint,
        "model": row.model,
        "params": params,
        "hasApiKey": row.has_key,
        "destination": profiles::destination_of(&row.endpoint),
        "egress": match access {
            EgressAccess::Granted => "granted",
            EgressAccess::Public => "public",
        },
        "createdBy": row.created_by,
        "createdAt": row.created_at,
        "updatedBy": row.updated_by,
        "updatedAt": row.updated_at,
    })
}

/// The audit detail of a profile: no key, only whether one is stored.
fn profile_audit(row: &AssistProfileRow) -> Value {
    json!({
        "name": row.name,
        "preset": row.preset,
        "dialect": row.dialect,
        "endpointHost": profiles::destination_of(&row.endpoint),
        "model": row.model,
        "hasApiKey": row.has_key,
    })
}

/// A profile as `POST` and `PUT` take it. `apiKey` absent or `null` keeps the
/// stored key (create: none), a string sets it, `""` removes it.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProfileBody {
    name: String,
    preset: String,
    /// Defaults to the preset's dialect.
    #[serde(default)]
    dialect: Option<String>,
    endpoint: String,
    #[serde(default)]
    model: String,
    #[serde(default)]
    params: ProfileParams,
    #[serde(default)]
    api_key: Option<String>,
    /// Read-only (GET carries it); accepted and ignored so a GET body can be sent back.
    #[serde(default, rename = "hasApiKey")]
    _has_api_key: Option<bool>,
}

fn invalid(pointer: &str, message: impl Into<String>) -> AdminApiError {
    AdminApiError::invalid("invalidProfile", message.into(), pointer)
}

fn plain_text(s: &str, max: usize) -> bool {
    s.chars().count() <= max && !s.chars().any(char::is_control)
}

/// Check a body and return the normalised fields plus the key change.
fn validate(
    body: &ProfileBody,
) -> Result<(String, Dialect, String, String, ProfileKey), AdminApiError> {
    let name = body.name.trim().to_string();
    if name.is_empty() || !plain_text(&name, 80) {
        return Err(invalid("/name", "name must be 1 to 80 characters"));
    }
    let preset =
        profiles::preset(&body.preset).ok_or_else(|| invalid("/preset", "unknown preset"))?;
    let dialect = match &body.dialect {
        None => preset.dialect,
        Some(d) => Dialect::parse(d)
            .ok_or_else(|| invalid("/dialect", "dialect is openai, azure-openai or anthropic"))?,
    };
    let endpoint = body.endpoint.trim().trim_end_matches('/').to_string();
    profiles::validate_endpoint(&endpoint).map_err(|m| invalid("/endpoint", m))?;
    let model = body.model.trim().to_string();
    if !plain_text(&model, 200) {
        return Err(invalid("/model", "model must be at most 200 characters"));
    }
    if model.is_empty() && dialect != Dialect::AzureOpenai {
        return Err(invalid("/model", "model is required"));
    }
    body.params.validate().map_err(|m| invalid("/params", m))?;
    let key = match &body.api_key {
        None => ProfileKey::Keep,
        Some(k) if k.is_empty() => ProfileKey::Clear,
        Some(k) if k.len() > 4096 || k.chars().any(char::is_control) => {
            return Err(invalid(
                "/apiKey",
                "the API key is too long or has control characters",
            ));
        }
        Some(k) => ProfileKey::Set(k.clone()),
    };
    Ok((name, dialect, endpoint, model, key))
}

fn params_json(params: &ProfileParams) -> String {
    serde_json::to_string(params).unwrap_or_else(|_| "{}".into())
}

// ── handlers ─────────────────────────────────────────────────────────────────

async fn presets(_ctx: AdminCtx) -> Response {
    Json(json!({ "presets": PRESETS })).into_response()
}

async fn list(State(state): State<AppState>, _ctx: AdminCtx) -> Result<Response, AdminApiError> {
    let rows = state
        .store
        .list_assist_profiles()
        .await
        .map_err(AdminApiError::internal)?;
    let grants = grant_destinations(&state)
        .await
        .map_err(AdminApiError::internal)?;
    Ok(Json(json!({
        "profiles": rows.iter().map(|r| profile_wire(r, &grants)).collect::<Vec<_>>(),
        "assignmentKey": PROFILE_KEY,
    }))
    .into_response())
}

async fn create(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(body): AdminJson<ProfileBody>,
) -> Result<Response, AdminApiError> {
    let (name, dialect, endpoint, model, key) = validate(&body)?;
    let existing = state
        .store
        .list_assist_profiles()
        .await
        .map_err(AdminApiError::internal)?;
    if existing.len() >= MAX_PROFILES {
        return Err(invalid("/name", format!("at most {MAX_PROFILES} profiles")));
    }
    let now = chrono::Utc::now().timestamp();
    let id = format!("p-{}", &uuid::Uuid::new_v4().simple().to_string()[..12]);
    let row = AssistProfileRow {
        id: id.clone(),
        name,
        preset: body.preset.clone(),
        dialect: dialect.as_str().into(),
        endpoint,
        model,
        params_json: params_json(&body.params),
        has_key: matches!(&key, ProfileKey::Set(k) if !k.is_empty()),
        created_by: ctx.username.clone(),
        created_at: now,
        updated_by: ctx.username.clone(),
        updated_at: now,
    };
    let key = if key == ProfileKey::Keep {
        ProfileKey::Clear
    } else {
        key
    };
    state
        .store
        .put_assist_profile(&row, &key)
        .await
        .map_err(AdminApiError::internal)?
        .map_err(|m| invalid("/apiKey", m))?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "assist-profile-created",
        &format!("assist-profile:{id}"),
        None,
        Some(profile_audit(&row)),
    )
    .await;
    let grants = grant_destinations(&state)
        .await
        .map_err(AdminApiError::internal)?;
    Ok(Json(profile_wire(&row, &grants)).into_response())
}

async fn update(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    AdminJson(body): AdminJson<ProfileBody>,
) -> Result<Response, AdminApiError> {
    let before = state
        .store
        .get_assist_profile(&id)
        .await
        .map_err(AdminApiError::internal)?
        .ok_or_else(|| AdminApiError::not_found("Profile not found."))?;
    let (name, dialect, endpoint, model, key) = validate(&body)?;
    let has_key = match &key {
        ProfileKey::Keep => before.has_key,
        ProfileKey::Set(k) => !k.is_empty(),
        ProfileKey::Clear => false,
    };
    let row = AssistProfileRow {
        id: id.clone(),
        name,
        preset: body.preset.clone(),
        dialect: dialect.as_str().into(),
        endpoint,
        model,
        params_json: params_json(&body.params),
        has_key,
        created_by: before.created_by.clone(),
        created_at: before.created_at,
        updated_by: ctx.username.clone(),
        updated_at: chrono::Utc::now().timestamp(),
    };
    state
        .store
        .put_assist_profile(&row, &key)
        .await
        .map_err(AdminApiError::internal)?
        .map_err(|m| invalid("/apiKey", m))?;
    let mut after = profile_audit(&row);
    after["keyChanged"] = json!(!matches!(key, ProfileKey::Keep));
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "assist-profile-updated",
        &format!("assist-profile:{id}"),
        Some(profile_audit(&before)),
        Some(after),
    )
    .await;
    let grants = grant_destinations(&state)
        .await
        .map_err(AdminApiError::internal)?;
    Ok(Json(profile_wire(&row, &grants)).into_response())
}

async fn remove(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    let before = state
        .store
        .get_assist_profile(&id)
        .await
        .map_err(AdminApiError::internal)?
        .ok_or_else(|| AdminApiError::not_found("Profile not found."))?;
    let removed = state
        .store
        .delete_assist_profile(&id)
        .await
        .map_err(AdminApiError::internal)?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "assist-profile-deleted",
        &format!("assist-profile:{id}"),
        Some(profile_audit(&before)),
        None,
    )
    .await;
    Ok(Json(json!({ "removed": removed })).into_response())
}

/// `POST /admin/assist/profiles/{id}/test` — send the fixed test prompt (no mail
/// content) and answer `{ok, latencyMs, reply, error}`. Runs whether or not Assist
/// is switched on, so a profile can be checked before anyone uses it; it is
/// audited with its outcome.
async fn test(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    let row = state
        .store
        .get_assist_profile(&id)
        .await
        .map_err(AdminApiError::internal)?
        .ok_or_else(|| AdminApiError::not_found("Profile not found."))?;
    let adapter = match adapter_for(&state, &row).await {
        Ok(a) => a,
        Err(mw_store::StoreError::Seal(_)) => {
            return Err(AdminApiError::conflict(
                "keyUnreadable",
                "The stored API key does not open with this server's key. Enter the key again.",
            ));
        }
        Err(e) => return Err(AdminApiError::internal(e)),
    };
    let outcome = profiles::test_profile(&adapter).await;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "assist-profile-tested",
        &format!("assist-profile:{id}"),
        None,
        Some(json!({
            "ok": outcome.ok,
            "latencyMs": outcome.latency_ms,
            "error": outcome.error.as_ref().map(|e| e.kind.code()),
            "status": outcome.error.as_ref().and_then(|e| e.status),
        })),
    )
    .await;
    Ok(Json(outcome).into_response())
}

// ── egress grants ────────────────────────────────────────────────────────────

fn grant_wire(g: &AssistEgressGrantRow) -> Value {
    json!({
        "id": g.id,
        "destination": g.destination,
        "note": g.note,
        "createdBy": g.created_by,
        "createdAt": g.created_at,
    })
}

async fn list_grants(
    State(state): State<AppState>,
    _ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    let rows = state
        .store
        .list_assist_egress_grants()
        .await
        .map_err(AdminApiError::internal)?;
    Ok(Json(json!({ "grants": rows.iter().map(grant_wire).collect::<Vec<_>>() })).into_response())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GrantBody {
    destination: String,
    #[serde(default)]
    note: String,
}

async fn create_grant(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(body): AdminJson<GrantBody>,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(
        &state,
        &ctx,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    let destination = profiles::parse_grant(&body.destination)
        .map_err(|m| AdminApiError::invalid("invalidDestination", m, "/destination"))?;
    if !plain_text(&body.note, 256) {
        return Err(AdminApiError::invalid(
            "invalidDestination",
            "The note is longer than 256 characters or has control characters.",
            "/note",
        ));
    }
    let rows = state
        .store
        .list_assist_egress_grants()
        .await
        .map_err(AdminApiError::internal)?;
    if rows.len() >= MAX_GRANTS {
        return Err(AdminApiError::invalid(
            "limitExceeded",
            format!("At most {MAX_GRANTS} grants are allowed."),
            "/destination",
        ));
    }
    let row = AssistEgressGrantRow {
        id: format!("g-{}", &uuid::Uuid::new_v4().simple().to_string()[..12]),
        destination: destination.clone(),
        note: body.note.trim().to_string(),
        created_by: ctx.username.clone(),
        created_at: chrono::Utc::now().timestamp(),
    };
    let added = state
        .store
        .add_assist_egress_grant(&row)
        .await
        .map_err(AdminApiError::internal)?;
    if !added {
        return Err(AdminApiError::invalid(
            "alreadyExists",
            "This destination is already granted.",
            "/destination",
        ));
    }
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "assist-egress-grant-created",
        &format!("assist-egress-grant:{}", row.id),
        None,
        Some(json!({ "destination": destination })),
    )
    .await;
    Ok(Json(grant_wire(&row)).into_response())
}

async fn remove_grant(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(
        &state,
        &ctx,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    let removed = state
        .store
        .delete_assist_egress_grant(&id)
        .await
        .map_err(AdminApiError::internal)?
        .ok_or_else(|| AdminApiError::not_found("Grant not found."))?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "assist-egress-grant-removed",
        &format!("assist-egress-grant:{id}"),
        Some(json!({ "destination": removed.destination })),
        None,
    )
    .await;
    Ok(Json(json!({ "removed": true })).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(v: Value) -> ProfileBody {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn bodies_are_checked_and_the_key_change_is_explicit() {
        let ok = body(json!({
            "name": "Mock", "preset": "openai-compatible",
            "endpoint": "http://127.0.0.1:9/v1/", "model": "m"
        }));
        let (_, dialect, endpoint, _, key) = validate(&ok).ok().unwrap();
        assert_eq!(dialect, Dialect::Openai);
        assert_eq!(endpoint, "http://127.0.0.1:9/v1", "trailing slash dropped");
        assert!(key == ProfileKey::Keep);
        let clear = body(json!({
            "name": "x", "preset": "anthropic", "endpoint": "https://api.anthropic.com",
            "model": "claude-haiku-5-5", "apiKey": ""
        }));
        assert!(validate(&clear).ok().unwrap().4 == ProfileKey::Clear);
        for bad in [
            json!({"name": "", "preset": "openai", "endpoint": "https://x/v1", "model": "m"}),
            json!({"name": "x", "preset": "nope", "endpoint": "https://x/v1", "model": "m"}),
            json!({"name": "x", "preset": "openai", "endpoint": "ftp://x", "model": "m"}),
            json!({"name": "x", "preset": "openai", "endpoint": "https://x/v1", "model": ""}),
            json!({"name": "x", "preset": "openai", "endpoint": "https://x/v1", "model": "m",
                   "params": {"temperature": 9}}),
            json!({"name": "x", "preset": "openai", "endpoint": "https://x/v1", "model": "m",
                   "dialect": "cohere"}),
        ] {
            assert!(validate(&body(bad.clone())).is_err(), "{bad}");
        }
        let unknown: Result<ProfileBody, _> = serde_json::from_value(json!({
            "name": "x", "preset": "openai", "endpoint": "https://x/v1", "model": "m", "extra": 1
        }));
        assert!(unknown.is_err(), "unknown fields are refused");
    }

    #[test]
    fn the_wire_and_audit_shapes_carry_no_key() {
        let row = AssistProfileRow {
            id: "p-1".into(),
            name: "n".into(),
            preset: "ollama".into(),
            dialect: "openai".into(),
            endpoint: "http://127.0.0.1:11434/v1".into(),
            model: "llama3.2".into(),
            params_json: r#"{"timeoutMs":5000}"#.into(),
            has_key: true,
            created_by: "root".into(),
            created_at: 1,
            updated_by: "root".into(),
            updated_at: 2,
        };
        let wire = profile_wire(&row, &["127.0.0.1:11434".into()]);
        assert_eq!(wire["egress"], "granted");
        assert_eq!(wire["hasApiKey"], true);
        assert_eq!(wire["params"]["timeoutMs"], 5000);
        assert!(wire.get("apiKey").is_none());
        assert_eq!(profile_wire(&row, &[])["egress"], "public");
        assert!(profile_audit(&row).get("apiKey").is_none());
    }
}
