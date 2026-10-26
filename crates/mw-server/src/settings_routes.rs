//! Settings model C routes (26.20 t32-e10b, worker f01; t32-admin §2.4, R-CFG 1-6).
//!
//! **Admin** (registered in `admin_router.rs` with capabilities):
//! * `GET /admin/settings` — every registered key with its definition, global
//!   value, flags, environment state and how many narrower values it has;
//!   `GET /admin/settings/{key}` — one of them, with an `ETag`.
//! * `PUT /admin/settings/{key}` `{globalValue?, allowUserOverride?, locked?,
//!   dryRun?}` — omitted members keep their stored value; `globalValue: null`
//!   reverts the value to the default. `DELETE` removes the key's stored policy.
//! * `GET /admin/settings/{key}/values?scope=domain|profile|group|admin_user|user
//!   &cursor=&limit=` — the narrower values, one page.
//! * `PUT|DELETE /admin/settings/{key}/domains/{domain}` `{value, locked?, dryRun?}`
//!   and `PUT|DELETE /admin/settings/{key}/users/{id}` `{value}` — a domain value
//!   and an admin's value for one user (`id` is the account id, or the canonical
//!   address for `limit.*`/`quota.*`).
//!
//! Refusals are `{code, detail, field}` ([`AdminApiError`]): `422 invalidValue`
//! with a JSON pointer, `403 useDedicatedRoute` for `signin.*`, `409 pinnedByEnv`
//! for a pinned key, `403 setting_locked` for a domain-scoped admin on a locked
//! row, `422 aboveGlobalLimit` from the limit ceiling, `412 stale` on an `If-Match`
//! mismatch. Locking a key (or turning user overrides off) that has narrower
//! values accepts `dryRun: true` and answers `{impact}` without writing. Every
//! write is audited as `setting-changed` with target `setting:<key>` and the
//! before and after values.
//!
//! **User** (merged into the account-preferences router):
//! * `GET /api/settings` → `{settings: {key: {value, source, locked, lockedBy?,
//!   userValue?}}}` for the keys a user sees;
//! * `PUT /api/settings/{key}` `{value}` → the new entry, or `403
//!   {code:"setting_locked"}` when an admin holds the key;
//! * `DELETE /api/settings/{key}` → the entry after removing the user's value.
//!
//! Five keys keep the user's own value where it lived before the settings model
//! ([`USER_LAYER_ELSEWHERE`]): `appearance.theme|accent|density` in the
//! appearance object, `mail.readReceipts` in `mdn_policy:<account>` (the engine
//! reads it) and `notifications.enabled` in the notification rules. `GET` reads
//! them from there; `PUT`/`DELETE` answer `409 {code:"useDedicatedRoute", route}`
//! naming the route that writes them, which applies the administrator's locks.

use axum::Json;
use axum::Router;
use axum::extract::{Path as UrlPath, Query, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use serde::Deserialize;
use serde_json::{Map, Value, json};

use mw_store::settings_policy::{SettingPolicyRow, SettingScope, SettingSubject, SettingValueRow};

use crate::AppState;
use crate::admin_ctx::{
    AdminApiError, AdminCtx, AdminJson, check_if_match, domain_of, etag_of, with_etag,
};
use crate::admin_router::AdminRegistrar;
use crate::capabilities::Capability;
use crate::settings_registry::{
    self as registry, Actor, CeilingError, CeilingTarget, Effective, Refusal, SettingDef,
};

// ─── registration ─────────────────────────────────────────────────────────────

/// The admin settings routes, registered through `admin_router::endpoints`.
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/settings", Capability::SettingsRead, list);
    r.admin_endpoint(
        Method::GET,
        "/settings/{key}",
        Capability::SettingsRead,
        get_one,
    );
    r.admin_endpoint(
        Method::PUT,
        "/settings/{key}",
        Capability::SettingsWrite,
        put_global,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/settings/{key}",
        Capability::SettingsWrite,
        delete_global,
    );
    r.admin_endpoint(
        Method::GET,
        "/settings/{key}/values",
        Capability::SettingsRead,
        list_values,
    );
    r.admin_endpoint(
        Method::PUT,
        "/settings/{key}/domains/{domain}",
        Capability::SettingsWrite,
        put_domain,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/settings/{key}/domains/{domain}",
        Capability::SettingsWrite,
        delete_domain,
    );
    r.admin_endpoint(
        Method::PUT,
        "/settings/{key}/users/{id}",
        Capability::SettingsUserValues,
        put_user,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/settings/{key}/users/{id}",
        Capability::SettingsUserValues,
        delete_user,
    );
}

/// The user routes (`/api/settings`), merged by `prefs_routes::prefs_router`.
pub(crate) fn user_router() -> Router<AppState> {
    Router::new()
        .route("/api/settings", get(user_list))
        .route("/api/settings/{key}", put(user_put).delete(user_delete))
}

// ─── shared helpers ───────────────────────────────────────────────────────────

fn refusal(r: Refusal) -> AdminApiError {
    let status = StatusCode::from_u16(r.status).unwrap_or(StatusCode::UNPROCESSABLE_ENTITY);
    let mut e = AdminApiError::new(status, r.code, r.detail);
    if let Some(field) = r.field {
        e = e.at(field);
    }
    if let Some((k, v)) = r.extra {
        e = e.with(k, v);
    }
    e
}

fn ceiling_err(e: CeilingError) -> AdminApiError {
    match e {
        CeilingError::Refused(r) => refusal(r),
        CeilingError::Store(e) => AdminApiError::internal(e),
    }
}

fn def_of(key: &str) -> Result<&'static SettingDef, AdminApiError> {
    registry::lookup(key).ok_or_else(|| AdminApiError::not_found(format!("no setting named {key}")))
}

/// The ceiling's view of the caller: unscoped when `settings.write` is held for
/// all domains.
fn actor_of(ctx: &AdminCtx, cap: Capability) -> Actor {
    match ctx.domain_filter(cap) {
        None => Actor::Unscoped,
        Some(set) => Actor::Domains(set.iter().cloned().collect()),
    }
}

fn parse_json(s: &str) -> Value {
    serde_json::from_str(s).unwrap_or(Value::Null)
}

fn policy_json(p: Option<&SettingPolicyRow>) -> Value {
    match p {
        None => Value::Null,
        Some(p) => json!({
            "globalValue": p.global_value.as_deref().map(parse_json),
            "allowUserOverride": p.allow_user_override,
            "globalLocked": p.global_locked,
            "updatedBy": p.updated_by,
            "updatedAt": p.updated_at,
        }),
    }
}

fn value_json(v: Option<&SettingValueRow>) -> Value {
    match v {
        None => Value::Null,
        Some(v) => json!({
            "scope": v.scope.as_str(),
            "scopeId": v.scope_id,
            "value": parse_json(&v.value),
            "locked": v.locked,
            "updatedBy": v.updated_by,
            "updatedAt": v.updated_at,
        }),
    }
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Append the `setting-changed` audit row (R-CFG 3). Best effort, like every
/// other admin audit append: a failed append is logged and the write stands.
async fn audit(state: &AppState, ctx: &AdminCtx, key: &str, detail: Value) {
    let entry = mw_admin::AuditLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor: ctx.username.clone(),
        actor_kind: mw_admin::ActorKind::Admin,
        action: "setting-changed".to_string(),
        target: Some(format!("setting:{key}")),
        detail_json: mw_admin::redact_detail(&detail),
        ip: None,
    };
    if let Err(e) = state.v6.admin.audit(entry).await {
        tracing::warn!("settings audit append failed: {e}");
    }
}

/// The overview entry of one key.
fn describe_key(
    def: &SettingDef,
    key: &str,
    policy: Option<&SettingPolicyRow>,
    counts: &Map<String, Value>,
) -> Value {
    let effective = registry::resolve_loaded(
        def,
        key,
        &SettingSubject::default(),
        policy,
        &[],
        &registry::process_env,
    );
    let env = registry::env_state(def, &registry::process_env).map(|e| {
        json!({
            "var": e.var,
            "precedence": e.precedence.as_str(),
            "set": e.value.is_some(),
            "value": e.value,
        })
    });
    let stored = policy_json(policy);
    let mut v = def.describe();
    v["key"] = Value::String(key.to_string());
    v["stored"] = stored.clone();
    v["globalValue"] = policy
        .and_then(|p| p.global_value.as_deref())
        .map(parse_json)
        .unwrap_or(Value::Null);
    v["allowUserOverride"] = Value::Bool(match def.scopes {
        registry::Scopes::GlobalDomainUser => policy.is_none_or(|p| p.allow_user_override),
        _ => false,
    });
    v["globalLocked"] = Value::Bool(policy.is_some_and(|p| p.global_locked));
    v["updatedBy"] = policy
        .map(|p| Value::String(p.updated_by.clone()))
        .unwrap_or(Value::Null);
    v["updatedAt"] = policy.map(|p| json!(p.updated_at)).unwrap_or(Value::Null);
    v["effective"] = json!({ "value": effective.value, "source": effective.source.as_str() });
    v["envState"] = env.unwrap_or(Value::Null);
    v["overrides"] = Value::Object(counts.clone());
    v["etag"] = Value::String(etag_of(&stored));
    // Flat aliases of the definition for the console's `RegistryEntry` reader
    // (`screens/Admin/shell/adminApi.ts`, `sourceFromRegistry`).
    let kind = def.kind.describe();
    v["type"] = kind["type"].clone();
    for (alias, member) in [
        ("min", "min"),
        ("max", "max"),
        ("options", "values"),
        ("maxLength", "maxLength"),
    ] {
        if let Some(x) = kind.get(member) {
            v[alias] = x.clone();
        }
    }
    v["locked"] = v["globalLocked"].clone();
    if let Some((var, p)) = def.env {
        v["envVar"] = Value::String(var.to_string());
        v["precedence"] = Value::String(p.as_str().to_string());
    }
    v
}

async fn counts_for(
    state: &AppState,
) -> Result<std::collections::HashMap<String, Map<String, Value>>, AdminApiError> {
    let mut out: std::collections::HashMap<String, Map<String, Value>> = Default::default();
    for c in state.store.count_setting_values().await? {
        out.entry(c.key)
            .or_default()
            .insert(c.scope.as_str().to_string(), json!(c.count));
    }
    Ok(out)
}

// ─── admin: global ────────────────────────────────────────────────────────────

async fn list(State(state): State<AppState>, _ctx: AdminCtx) -> Result<Response, AdminApiError> {
    let policies = state.store.list_setting_policies().await?;
    let counts = counts_for(&state).await?;
    let empty = Map::new();
    let mut items = Vec::new();
    for def in registry::REGISTRY.iter().filter(|d| !d.is_family()) {
        let policy = policies.iter().find(|p| p.key == def.key);
        items.push(describe_key(
            def,
            def.key,
            policy,
            counts.get(def.key).unwrap_or(&empty),
        ));
    }
    // Family members that have a stored value of any kind.
    let mut family: Vec<&str> = policies
        .iter()
        .map(|p| p.key.as_str())
        .chain(counts.keys().map(String::as_str))
        .filter(|k| registry::lookup(k).is_some_and(SettingDef::is_family))
        .collect();
    family.sort_unstable();
    family.dedup();
    for key in family {
        if let Some(def) = registry::lookup(key) {
            let policy = policies.iter().find(|p| p.key == key);
            items.push(describe_key(
                def,
                key,
                policy,
                counts.get(key).unwrap_or(&empty),
            ));
        }
    }
    let families: Vec<Value> = registry::REGISTRY
        .iter()
        .filter(|d| d.is_family())
        .map(SettingDef::describe)
        .collect();
    let body = json!({ "settings": items, "families": families, "propagationSecs": 30 });
    let etag = etag_of(&body);
    Ok(with_etag(Json(body), &etag))
}

async fn one(
    state: &AppState,
    key: &str,
) -> Result<(Value, Option<SettingPolicyRow>), AdminApiError> {
    let def = def_of(key)?;
    let policy = state.store.get_setting_policy(key).await?;
    let counts = counts_for(state).await?;
    let empty = Map::new();
    Ok((
        describe_key(def, key, policy.as_ref(), counts.get(key).unwrap_or(&empty)),
        policy,
    ))
}

async fn get_one(
    State(state): State<AppState>,
    _ctx: AdminCtx,
    UrlPath(key): UrlPath<String>,
) -> Result<Response, AdminApiError> {
    let (item, _) = one(&state, &key).await?;
    let etag = item["etag"].as_str().unwrap_or_default().to_string();
    Ok(with_etag(Json(item), &etag))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GlobalPut {
    /// Absent = keep; `null` = back to the default.
    #[serde(default, with = "present")]
    global_value: Option<Value>,
    allow_user_override: Option<bool>,
    locked: Option<bool>,
    #[serde(default)]
    dry_run: bool,
}

/// Tells an absent member (`None`) from an explicit `null` (`Some(Null)`).
mod present {
    use serde::{Deserialize, Deserializer};
    use serde_json::Value;

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Value>, D::Error> {
        Value::deserialize(d).map(Some)
    }
}

/// How many narrower values a lock or an override switch-off would overrule.
/// One consequence of a write: how many, and the first [`IMPACT_NAMES`] names
/// (the shape the console's `parseImpact` reads).
fn impact_item(count: usize, names: Vec<String>) -> Value {
    json!({ "count": count, "names": names })
}

/// Names listed per consequence in a dry run (UX6: the first 20).
const IMPACT_NAMES: usize = 20;

async fn global_impact(state: &AppState, key: &str) -> Result<Value, AdminApiError> {
    let mut impact = Map::new();
    for scope in SettingScope::ALL {
        let rows = state
            .store
            .list_setting_values(key, scope, None, 500)
            .await?;
        if rows.is_empty() {
            continue;
        }
        let names = rows
            .iter()
            .take(IMPACT_NAMES)
            .map(|r| r.scope_id.clone())
            .collect();
        impact.insert(
            format!("{}Values", camel(scope)),
            impact_item(rows.len(), names),
        );
    }
    Ok(Value::Object(impact))
}

fn camel(scope: SettingScope) -> &'static str {
    match scope {
        SettingScope::Domain => "domain",
        SettingScope::Profile => "profile",
        SettingScope::Group => "group",
        SettingScope::AdminUser => "adminUser",
        SettingScope::User => "user",
    }
}

async fn put_global(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    UrlPath(key): UrlPath<String>,
    AdminJson(body): AdminJson<GlobalPut>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    let def = def_of(&key)?;
    registry::refuse_if_dedicated(def).map_err(refusal)?;
    let current = state.store.get_setting_policy(&key).await?;
    let stored = policy_json(current.as_ref());
    check_if_match(
        &headers,
        &etag_of(&stored),
        current.as_ref().map(|p| p.updated_by.as_str()),
        current.as_ref().map(|p| p.updated_at),
    )?;
    let global_value = match &body.global_value {
        None => current.as_ref().and_then(|p| p.global_value.clone()),
        Some(v) => {
            registry::refuse_if_pinned(def, &registry::process_env).map_err(refusal)?;
            if v.is_null() {
                None
            } else {
                Some(
                    registry::validate(def, v, "/globalValue")
                        .map_err(refusal)?
                        .to_string(),
                )
            }
        }
    };
    if let Some(v) = &body.global_value
        && !v.is_null()
    {
        registry::ceiling(
            &state.store,
            &actor_of(&ctx, Capability::SettingsWrite),
            &key,
            CeilingTarget::Global,
            v,
        )
        .await
        .map_err(ceiling_err)?;
    }
    let user_scoped = def.scopes == registry::Scopes::GlobalDomainUser;
    let allow_user_override = match body.allow_user_override {
        Some(true) if !user_scoped => {
            return Err(AdminApiError::invalid(
                "notUserScoped",
                format!("{key} cannot be set by users"),
                "/allowUserOverride",
            ));
        }
        Some(b) => b,
        None => current
            .as_ref()
            .map_or(def.user_override, |p| p.allow_user_override),
    } && user_scoped;
    let global_locked = body
        .locked
        .unwrap_or_else(|| current.as_ref().is_some_and(|p| p.global_locked));
    let was_locked = current.as_ref().is_some_and(|p| p.global_locked);
    let was_open = current
        .as_ref()
        .map_or(def.user_override, |p| p.allow_user_override);
    if body.dry_run {
        let tightening = (global_locked && !was_locked) || (was_open && !allow_user_override);
        let impact = if tightening {
            global_impact(&state, &key).await?
        } else {
            json!({})
        };
        return Ok(Json(json!({ "dryRun": true, "impact": impact })).into_response());
    }
    let row = SettingPolicyRow {
        key: key.clone(),
        global_value,
        allow_user_override,
        global_locked,
        updated_by: ctx.username.clone(),
        updated_at: now(),
    };
    if !state.store.compare_setting_policy(&key, current.as_ref(), Some(&row)).await? {
        return Err(AdminApiError::stale(None, None));
    }
    audit(
        &state,
        &ctx,
        &key,
        json!({ "scope": "global", "before": stored, "after": policy_json(Some(&row)) }),
    )
    .await;
    let (item, _) = one(&state, &key).await?;
    let etag = item["etag"].as_str().unwrap_or_default().to_string();
    Ok(with_etag(Json(item), &etag))
}

async fn delete_global(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    UrlPath(key): UrlPath<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    let def = def_of(&key)?;
    registry::refuse_if_dedicated(def).map_err(refusal)?;
    let current = state.store.get_setting_policy(&key).await?;
    let stored = policy_json(current.as_ref());
    check_if_match(
        &headers,
        &etag_of(&stored),
        current.as_ref().map(|p| p.updated_by.as_str()),
        current.as_ref().map(|p| p.updated_at),
    )?;
    if !state.store.compare_setting_policy(&key, current.as_ref(), None).await? {
        return Err(AdminApiError::stale(None, None));
    }
    if current.is_some() {
        audit(
            &state,
            &ctx,
            &key,
            json!({ "scope": "global", "before": stored, "after": null }),
        )
        .await;
    }
    let (item, _) = one(&state, &key).await?;
    let etag = item["etag"].as_str().unwrap_or_default().to_string();
    Ok(with_etag(Json(item), &etag))
}

// ─── admin: narrower values ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ValuesQuery {
    scope: String,
    cursor: Option<String>,
    limit: Option<i64>,
}

async fn list_values(
    State(state): State<AppState>,
    ctx: AdminCtx,
    UrlPath(key): UrlPath<String>,
    Query(q): Query<ValuesQuery>,
) -> Result<Response, AdminApiError> {
    let def = def_of(&key)?;
    let scope = SettingScope::parse(&q.scope).ok_or_else(|| {
        AdminApiError::invalid(
            "invalidScope",
            "scope must be domain, profile, group, admin_user or user",
            "/scope",
        )
    })?;
    let user_scope = matches!(scope, SettingScope::AdminUser | SettingScope::User);
    if user_scope {
        ctx.require(Capability::SettingsUserValues)?;
    }
    let filter = ctx.domain_filter(if user_scope {
        Capability::SettingsUserValues
    } else {
        Capability::SettingsRead
    });
    if filter.is_some()
        && (matches!(scope, SettingScope::Profile | SettingScope::Group)
            || (user_scope && def.scope_id == registry::ScopeId::Account))
    {
        // Rows that are not keyed by a domain or an address cannot be filtered
        // to a domain-scoped admin's domains.
        ctx.require_unscoped(if user_scope {
            Capability::SettingsUserValues
        } else {
            Capability::SettingsRead
        })?;
    }
    let limit = q.limit.unwrap_or(100).clamp(1, 500);
    let rows = state
        .store
        .list_setting_values(&key, scope, q.cursor.as_deref(), limit)
        .await?;
    let next = (rows.len() as i64 == limit).then(|| rows.last().map(|r| r.scope_id.clone()));
    let mut visible = Vec::new();
    for row in &rows {
        if let Some(set) = filter {
            let domain = if scope == SettingScope::Domain {
                state.store.canonical_policy_domain(&row.scope_id).await?
            } else {
                user_domain(&state, &row.scope_id)
                    .await?
                    .unwrap_or_default()
            };
            if !set.contains(&domain) {
                continue;
            }
        }
        let mut value = value_json(Some(row));
        value["etag"] = Value::String(etag_of(&value_json(Some(row))));
        visible.push(value);
    }
    Ok(Json(json!({ "values": visible, "nextCursor": next.flatten() })).into_response())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ScopedPut {
    value: Value,
    locked: Option<bool>,
    #[serde(default)]
    dry_run: bool,
}

/// The common refusals of a write to a narrower scope.
fn check_scoped_write(
    def: &SettingDef,
    key: &str,
    scope: SettingScope,
) -> Result<(), AdminApiError> {
    registry::refuse_if_dedicated(def).map_err(refusal)?;
    if !def.scopes.allows(scope) {
        return Err(AdminApiError::invalid(
            "scopeNotAllowed",
            format!("{key} can be set {} only", def.scopes.as_str()),
            "/value",
        ));
    }
    registry::refuse_if_pinned(def, &registry::process_env).map_err(refusal)
}

async fn put_domain(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    UrlPath((key, domain)): UrlPath<(String, String)>,
    AdminJson(body): AdminJson<ScopedPut>,
) -> Result<Response, AdminApiError> {
    let domain = state.store.canonical_policy_domain(&domain).await?;
    ctx.require_domain(&domain)?;
    let def = def_of(&key)?;
    check_scoped_write(def, &key, SettingScope::Domain)?;
    let value = registry::validate(def, &body.value, "/value").map_err(refusal)?;
    let current = state
        .store
        .get_setting_value(SettingScope::Domain, &domain, &key)
        .await?;
    let before = value_json(current.as_ref());
    check_if_match(
        &headers,
        &etag_of(&before),
        current.as_ref().map(|r| r.updated_by.as_str()),
        current.as_ref().map(|r| r.updated_at),
    )?;
    let unscoped = ctx.has_unscoped(Capability::SettingsWrite);
    if !unscoped {
        if current.as_ref().is_some_and(|r| r.locked) {
            return Err(refusal(Refusal::locked(format!(
                "{key} is locked for {domain} by an administrator for all domains"
            ))));
        }
        if body.locked == Some(true) {
            return Err(AdminApiError::unscoped_required(Capability::SettingsWrite).at("/locked"));
        }
    }
    let actor = actor_of(&ctx, Capability::SettingsWrite);
    registry::ceiling(
        &state.store,
        &actor,
        &key,
        CeilingTarget::Domain(&domain),
        &value,
    )
    .await
    .map_err(ceiling_err)?;
    let locked = body
        .locked
        .unwrap_or_else(|| current.as_ref().is_some_and(|r| r.locked));
    if body.dry_run {
        let impact = if locked && !current.as_ref().is_some_and(|r| r.locked) {
            domain_impact(&state, &key, &domain).await?
        } else {
            json!({})
        };
        return Ok(Json(json!({ "dryRun": true, "impact": impact })).into_response());
    }
    let row = SettingValueRow {
        scope: SettingScope::Domain,
        scope_id: domain.clone(),
        key: key.clone(),
        value: value.to_string(),
        locked,
        updated_by: ctx.username.clone(),
        updated_at: now(),
    };
    state.store.put_setting_value(&row).await?;
    let after = value_json(Some(&row));
    audit(
        &state,
        &ctx,
        &key,
        json!({ "scope": "domain", "domain": domain, "before": before, "after": after }),
    )
    .await;
    let etag = etag_of(&after);
    let mut out = after;
    out["etag"] = Value::String(etag.clone());
    Ok(with_etag(Json(out), &etag))
}

/// How many per-user values in `domain` a domain lock would overrule.
async fn domain_impact(state: &AppState, key: &str, domain: &str) -> Result<Value, AdminApiError> {
    let mut impact = Map::new();
    for scope in [SettingScope::AdminUser, SettingScope::User] {
        let mut n = 0;
        let mut names = Vec::new();
        for r in state
            .store
            .list_setting_values(key, scope, None, 500)
            .await?
        {
            if user_domain(state, &r.scope_id).await?.as_deref() == Some(domain) {
                n += 1;
                if names.len() < IMPACT_NAMES {
                    names.push(r.scope_id.clone());
                }
            }
        }
        if n > 0 {
            impact.insert(format!("{}Values", camel(scope)), impact_item(n, names));
        }
    }
    Ok(Value::Object(impact))
}

/// The domain of a per-user scope id: the address's domain, or the domain of the
/// account's sign-in name.
async fn user_domain(state: &AppState, id: &str) -> Result<Option<String>, mw_store::StoreError> {
    let raw = if id.contains('@') {
        Some(domain_of(id))
    } else {
        match state.store.get_account(id).await {
            Ok(a) if a.username.contains('@') => Some(domain_of(&a.username)),
            Ok(_) | Err(mw_store::StoreError::NotFound) => None,
            Err(e) => return Err(e),
        }
    };
    match raw {
        Some(domain) => Ok(Some(state.store.canonical_policy_domain(&domain).await?)),
        None => Ok(None),
    }
}

async fn delete_domain(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    UrlPath((key, domain)): UrlPath<(String, String)>,
) -> Result<Response, AdminApiError> {
    let domain = state.store.canonical_policy_domain(&domain).await?;
    ctx.require_domain(&domain)?;
    let def = def_of(&key)?;
    registry::refuse_if_dedicated(def).map_err(refusal)?;
    let current = state
        .store
        .get_setting_value(SettingScope::Domain, &domain, &key)
        .await?;
    let before = value_json(current.as_ref());
    check_if_match(
        &headers,
        &etag_of(&before),
        current.as_ref().map(|r| r.updated_by.as_str()),
        current.as_ref().map(|r| r.updated_at),
    )?;
    if current.as_ref().is_some_and(|r| r.locked) && !ctx.has_unscoped(Capability::SettingsWrite) {
        return Err(refusal(Refusal::locked(format!(
            "{key} is locked for {domain} by an administrator for all domains"
        ))));
    }
    if current.is_some() {
        state
            .store
            .delete_setting_value(SettingScope::Domain, &domain, &key)
            .await?;
        audit(
            &state,
            &ctx,
            &key,
            json!({ "scope": "domain", "domain": domain, "before": before, "after": null }),
        )
        .await;
    }
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UserPut {
    value: Value,
}

/// The per-user scope id for `def`: the account id as given, or the address
/// lower-cased (t32-q A15).
async fn user_scope_id(
    store: &mw_store::Store,
    def: &SettingDef,
    id: &str,
) -> Result<String, mw_store::StoreError> {
    match def.scope_id {
        registry::ScopeId::Account => Ok(id.to_string()),
        registry::ScopeId::Address => store.canonical_policy_address(id).await,
    }
}

/// Check that the caller may act on the user `id`, and return that user's domain
/// when it is known.
async fn require_user(
    state: &AppState,
    ctx: &AdminCtx,
    id: &str,
) -> Result<Option<String>, AdminApiError> {
    let domain = user_domain(state, id).await?;
    match &domain {
        Some(d) => ctx.require_domain(d)?,
        None => ctx.require_unscoped(Capability::SettingsUserValues)?,
    }
    Ok(domain)
}

async fn put_user(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    UrlPath((key, id)): UrlPath<(String, String)>,
    AdminJson(body): AdminJson<UserPut>,
) -> Result<Response, AdminApiError> {
    let def = def_of(&key)?;
    let id = user_scope_id(&state.store, def, &id).await?;
    let domain = require_user(&state, &ctx, &id).await?;
    check_scoped_write(def, &key, SettingScope::AdminUser)?;
    let value = registry::validate(def, &body.value, "/value").map_err(refusal)?;
    let actor = actor_of(&ctx, Capability::SettingsUserValues);
    registry::ceiling(
        &state.store,
        &actor,
        &key,
        CeilingTarget::User {
            domain: domain.as_deref().unwrap_or_default(),
        },
        &value,
    )
    .await
    .map_err(ceiling_err)?;
    let current = state
        .store
        .get_setting_value(SettingScope::AdminUser, &id, &key)
        .await?;
    let before = value_json(current.as_ref());
    check_if_match(
        &headers,
        &etag_of(&before),
        current.as_ref().map(|r| r.updated_by.as_str()),
        current.as_ref().map(|r| r.updated_at),
    )?;
    let row = SettingValueRow {
        scope: SettingScope::AdminUser,
        scope_id: id.clone(),
        key: key.clone(),
        value: value.to_string(),
        locked: false,
        updated_by: ctx.username.clone(),
        updated_at: now(),
    };
    state.store.put_setting_value(&row).await?;
    let after = value_json(Some(&row));
    audit(
        &state,
        &ctx,
        &key,
        json!({ "scope": "admin_user", "user": id, "before": before, "after": after }),
    )
    .await;
    let etag = etag_of(&after);
    let mut out = after;
    out["etag"] = Value::String(etag.clone());
    Ok(with_etag(Json(out), &etag))
}

async fn delete_user(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    UrlPath((key, id)): UrlPath<(String, String)>,
) -> Result<Response, AdminApiError> {
    let def = def_of(&key)?;
    let id = user_scope_id(&state.store, def, &id).await?;
    require_user(&state, &ctx, &id).await?;
    registry::refuse_if_dedicated(def).map_err(refusal)?;
    let current = state
        .store
        .get_setting_value(SettingScope::AdminUser, &id, &key)
        .await?;
    let before = value_json(current.as_ref());
    check_if_match(
        &headers,
        &etag_of(&before),
        current.as_ref().map(|r| r.updated_by.as_str()),
        current.as_ref().map(|r| r.updated_at),
    )?;
    if current.is_some() {
        state
            .store
            .delete_setting_value(SettingScope::AdminUser, &id, &key)
            .await?;
        audit(
            &state,
            &ctx,
            &key,
            json!({ "scope": "admin_user", "user": id, "before": before, "after": null }),
        )
        .await;
    }
    Ok(StatusCode::NO_CONTENT.into_response())
}

// ─── user ─────────────────────────────────────────────────────────────────────

/// The resolution subject of a signed-in user: the domain and canonical address
/// from the session's sign-in name, and the account id.
pub(crate) struct UserSubject {
    pub domain: String,
    pub account_id: String,
    pub address: String,
}

impl UserSubject {
    pub(crate) fn of(session: &mw_store::Session) -> Self {
        let name = session.username.trim().to_ascii_lowercase();
        let (address, domain) = match name.rsplit_once('@') {
            Some((_, d)) if !d.is_empty() => (name.clone(), d.to_string()),
            _ => (String::new(), String::new()),
        };
        UserSubject {
            domain,
            account_id: session.account_id.clone(),
            address,
        }
    }

    pub(crate) async fn resolved(
        store: &mw_store::Store,
        session: &mw_store::Session,
    ) -> Result<Self, mw_store::StoreError> {
        let raw = Self::of(session);
        let canonical = store.canonical_setting_subject(&raw.as_subject()).await?;
        Ok(Self {
            domain: canonical.domain,
            account_id: canonical.account_id,
            address: canonical.address,
        })
    }

    pub(crate) fn as_subject(&self) -> SettingSubject<'_> {
        SettingSubject {
            domain: &self.domain,
            account_id: &self.account_id,
            address: &self.address,
        }
    }
}

/// `403 {code:"setting_locked"}` in the webmail error shape.
fn user_locked(key: &str, e: &Effective) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "code": "setting_locked",
            "error": format!("{key} is set by your administrator"),
            "lockedBy": e.locked_by.map(|s| s.as_str()),
        })),
    )
        .into_response()
}

fn user_error(status: StatusCode, code: &str, detail: String, field: Option<&str>) -> Response {
    (
        status,
        Json(json!({ "code": code, "error": detail, "field": field })),
    )
        .into_response()
}

fn server_error(e: impl std::fmt::Display) -> Response {
    tracing::error!("settings: {e}");
    (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response()
}

/// The user-writable entry for `key`, or the refusal to answer.
fn user_def(key: &str) -> Result<&'static SettingDef, Response> {
    match registry::lookup(key) {
        Some(d) if d.user_visible() => Ok(d),
        _ => Err(user_error(
            StatusCode::NOT_FOUND,
            "unknownSetting",
            format!("no setting named {key}"),
            None,
        )),
    }
}

async fn user_entry(
    state: &AppState,
    subject: &UserSubject,
    key: &str,
) -> Result<Effective, Response> {
    let extra = user_layer_rows(state, &subject.account_id)
        .await
        .map_err(server_error)?;
    match registry::effective_for_with(&state.store, key, &subject.as_subject(), &extra).await {
        Ok(Some(e)) => Ok(e),
        Ok(None) => Err(user_error(
            StatusCode::NOT_FOUND,
            "unknownSetting",
            format!("no setting named {key}"),
            None,
        )),
        Err(e) => Err(server_error(e)),
    }
}

/// The keys whose user value lives outside `setting_value`, and the route that
/// writes it (and applies the locks there).
pub(crate) const USER_LAYER_ELSEWHERE: &[(&str, &str)] = &[
    ("appearance.theme", "/api/account/appearance"),
    ("appearance.accent", "/api/account/appearance"),
    ("appearance.density", "/api/account/appearance"),
    ("appearance.font", "/api/account/appearance"),
    ("mail.readReceipts", "/api/account/mdn-policy"),
    ("notifications.enabled", "/api/account/notifications"),
];

/// `409 {code:"useDedicatedRoute", route}` for a key whose user value is written
/// elsewhere.
fn refuse_elsewhere(key: &str) -> Result<(), Response> {
    match USER_LAYER_ELSEWHERE.iter().find(|(k, _)| *k == key) {
        Some((_, route)) => Err((
            StatusCode::CONFLICT,
            Json(json!({
                "code": "useDedicatedRoute",
                "error": format!("{key} is changed through {route}"),
                "route": route,
            })),
        )
            .into_response()),
        None => Ok(()),
    }
}

fn user_row(account_id: &str, key: &str, value: Value) -> SettingValueRow {
    SettingValueRow {
        scope: SettingScope::User,
        scope_id: account_id.to_string(),
        key: key.to_string(),
        value: value.to_string(),
        locked: false,
        updated_by: account_id.to_string(),
        updated_at: 0,
    }
}

/// The user's own values of the [`USER_LAYER_ELSEWHERE`] keys, as `user` rows:
/// the appearance object's theme (once the user chose one), accent and density,
/// the read-receipt policy, and whether notifications are on.
pub(crate) async fn user_layer_rows(
    state: &AppState,
    account_id: &str,
) -> Result<Vec<SettingValueRow>, mw_store::StoreError> {
    let mut rows = Vec::new();
    if account_id.is_empty() {
        return Ok(rows);
    }
    if let Some(prefs) = state
        .store
        .get_setting(&format!("appearance:{account_id}"))
        .await?
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .and_then(|env| env.get("prefs").cloned())
        .filter(Value::is_object)
    {
        let chosen = prefs.get("chosen").and_then(Value::as_bool) != Some(false);
        if chosen && let Some(theme) = prefs.get("theme").and_then(Value::as_str) {
            rows.push(user_row(account_id, "appearance.theme", json!(theme)));
        }
        if let Some(accent) = prefs.get("accentId").and_then(Value::as_str)
            && !accent.is_empty()
        {
            rows.push(user_row(account_id, "appearance.accent", json!(accent)));
        }
        if let Some(density) = prefs.get("density").and_then(Value::as_str) {
            rows.push(user_row(account_id, "appearance.density", json!(density)));
        }
        if let Some(font) = prefs.get("font").and_then(Value::as_str) {
            rows.push(user_row(account_id, "appearance.font", json!(font)));
        }
    }
    if let Some(policy) = state
        .store
        .get_setting(&mw_engine::Engine::mdn_policy_key(account_id))
        .await?
        .filter(|p| mw_engine::Engine::MDN_POLICIES.contains(&p.as_str()))
    {
        rows.push(user_row(account_id, "mail.readReceipts", json!(policy)));
    }
    if let Some(n) = state.store.get_notification_rules(account_id).await? {
        rows.push(user_row(
            account_id,
            "notifications.enabled",
            json!(n.enabled),
        ));
    }
    Ok(rows)
}

/// The effective value of `key` for the session's user, with the user layer read
/// from where it lives. `None` for a key the registry does not know.
pub(crate) async fn effective_for_session(
    state: &AppState,
    session: &mw_store::Session,
    key: &str,
) -> Result<Option<Effective>, mw_store::StoreError> {
    let subject = UserSubject::resolved(&state.store, session).await?;
    let extra = user_layer_rows(state, &subject.account_id).await?;
    registry::effective_for_with(&state.store, key, &subject.as_subject(), &extra).await
}

async fn user_list(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let subject = match UserSubject::resolved(&state.store, &session).await {
        Ok(s) => s,
        Err(e) => return server_error(e),
    };
    let extra = match user_layer_rows(&state, &subject.account_id).await {
        Ok(rows) => rows,
        Err(e) => return server_error(e),
    };
    match registry::effective_all_with(
        &state.store,
        &subject.as_subject(),
        SettingDef::user_visible,
        &extra,
    )
    .await
    {
        Ok(all) => {
            let map: Map<String, Value> = all
                .iter()
                .map(|e| (e.key.clone(), e.to_user_json()))
                .collect();
            Json(json!({ "settings": map })).into_response()
        }
        Err(e) => server_error(e),
    }
}

#[derive(Debug, Deserialize)]
struct UserValue {
    value: Value,
}

async fn user_put(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(key): UrlPath<String>,
    body: Result<Json<UserValue>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let def = match user_def(&key) {
        Ok(d) => d,
        Err(r) => return r,
    };
    if let Err(r) = refuse_elsewhere(&key) {
        return r;
    }
    let Ok(Json(body)) = body else {
        return user_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalidBody",
            "the body must be {\"value\": …}".into(),
            None,
        );
    };
    let subject = match UserSubject::resolved(&state.store, &session).await {
        Ok(s) => s,
        Err(e) => return server_error(e),
    };
    let current = match user_entry(&state, &subject, &key).await {
        Ok(e) => e,
        Err(r) => return r,
    };
    if !current.user_may_override || !def.scopes.allows(SettingScope::User) {
        return user_locked(&key, &current);
    }
    let value = match registry::validate(def, &body.value, "/value") {
        Ok(v) if !v.is_null() || matches!(def.kind, registry::Kind::FolderRole) => v,
        Ok(_) => {
            return user_error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalidValue",
                "a value is required".into(),
                Some("/value"),
            );
        }
        Err(r) => {
            let status = StatusCode::from_u16(r.status).unwrap_or(StatusCode::UNPROCESSABLE_ENTITY);
            return user_error(status, r.code, r.detail, r.field.as_deref());
        }
    };
    let row = SettingValueRow {
        scope: SettingScope::User,
        scope_id: subject.account_id.clone(),
        key: key.clone(),
        value: value.to_string(),
        locked: false,
        updated_by: subject.account_id.clone(),
        updated_at: now(),
    };
    if let Err(e) = state.store.put_setting_value(&row).await {
        return server_error(e);
    }
    match user_entry(&state, &subject, &key).await {
        Ok(e) => Json(e.to_user_json()).into_response(),
        Err(r) => r,
    }
}

async fn user_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(key): UrlPath<String>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if let Err(r) = user_def(&key).and_then(|_| refuse_elsewhere(&key)) {
        return r;
    }
    let subject = match UserSubject::resolved(&state.store, &session).await {
        Ok(s) => s,
        Err(e) => return server_error(e),
    };
    if let Err(e) = state
        .store
        .delete_setting_value(SettingScope::User, &subject.account_id, &key)
        .await
    {
        return server_error(e);
    }
    match user_entry(&state, &subject, &key).await {
        Ok(e) => Json(e.to_user_json()).into_response(),
        Err(r) => r,
    }
}
