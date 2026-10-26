//! Domain editor API. Upstream changes require an unscoped grant.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, check_if_match, etag_of, with_etag},
    admin_router::AdminRegistrar,
    capabilities::Capability as C,
    domain_config::{UpstreamConfig, normalize_domain},
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
};
use mw_store::{DomainConfigRow, DomainMutation};
use serde::Deserialize;
use serde_json::{Value, json};

fn invalid(field: &str, detail: impl Into<String>) -> AdminApiError {
    AdminApiError::invalid("invalidDomain", detail, field)
}

async fn stop_domain_accounts(state: &AppState, domain: &str) -> Result<(), AdminApiError> {
    let ids = domain_account_ids(state, domain).await?;
    if let Some(engine) = &state.engine {
        for id in ids {
            engine.unregister(&id);
        }
    }
    Ok(())
}
async fn domain_account_ids(state: &AppState, domain: &str) -> Result<Vec<String>, AdminApiError> {
    let mut ids = Vec::new();
    if state.engine.is_some() {
        for account in state.store.list_accounts().await? {
            if crate::domain_config::canonical(&state.store, &account.username, None)
                .await
                .is_ok_and(|identity| identity.domain == domain)
            {
                ids.push(account.id);
            }
        }
    }
    Ok(ids)
}

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::POST,
        "/domains/username-preview",
        C::DomainsUpstreamWrite,
        username_preview,
    );
    r.admin_endpoint(Method::GET, "/domains/status", C::DomainsRead, status);
    r.admin_endpoint(
        Method::POST,
        "/domains/{name}/detect",
        C::DomainsUpstreamWrite,
        crate::domain_detect::detect,
    );
    r.admin_endpoint(Method::GET, "/domains", C::DomainsRead, list);
    r.admin_endpoint(Method::GET, "/domains/{name}", C::DomainsRead, get);
    r.admin_endpoint(Method::PUT, "/domains/{name}", C::DomainsWrite, put);
    r.admin_endpoint(Method::DELETE, "/domains/{name}", C::DomainsDelete, delete);
    r.admin_endpoint(
        Method::POST,
        "/domains/{name}/enable",
        C::DomainsWrite,
        enable,
    );
    r.admin_endpoint(
        Method::POST,
        "/domains/{name}/disable",
        C::DomainsWrite,
        disable,
    );
    r.admin_endpoint(
        Method::PUT,
        "/domains/{name}/preferred",
        C::DomainsWrite,
        prefer,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/preferred-domain",
        C::DomainsWrite,
        unprefer,
    );
    r.admin_endpoint(
        Method::PUT,
        "/domains/{name}/aliases/{alias}",
        C::DomainsWrite,
        alias_put,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/domains/{name}/aliases/{alias}",
        C::DomainsWrite,
        alias_delete,
    );
    r.admin_endpoint(
        Method::PATCH,
        "/domains/{name}/aliases/{alias}",
        C::DomainsWrite,
        alias_rename,
    );
    r.admin_endpoint(
        Method::PUT,
        "/domains/{name}/hosts/{host}",
        C::DomainsWrite,
        host_put,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/domains/{name}/hosts/{host}",
        C::DomainsWrite,
        host_delete,
    );
}

/// Shared with the dashboard: counts are computed from current routing, so a
/// corrected domain immediately clears its blocked-account warning.
pub(crate) async fn blocked_accounts(
    state: &AppState,
    ctx: &AdminCtx,
) -> Result<Value, AdminApiError> {
    let scope = ctx.domain_filter(C::DomainsRead);
    let mut domains: std::collections::BTreeMap<String, (usize, Option<String>)> =
        Default::default();
    for account in state.store.domain_account_inventory().await? {
        let domain = crate::domain_config::canonical(&state.store, &account.username, None)
            .await
            .map(|identity| identity.domain)
            .unwrap_or_default();
        if scope.is_some_and(|allowed| !allowed.contains(&domain)) {
            continue;
        }
        if crate::upstream_resolve::resolve_upstream(state, &account.username)
            .await
            .is_err()
        {
            let entry = domains.entry(domain).or_default();
            entry.0 += 1;
            if entry.1.is_none() {
                entry.1 = account.seen_host;
            }
        }
    }
    let total: usize = domains.values().map(|entry| entry.0).sum();
    Ok(
        json!({"total":total,"byDomain":domains.into_iter().map(|(domain,(accounts,host))|
        json!({"domain":domain,"accounts":accounts,"seenHost":host})).collect::<Vec<_>>()}),
    )
}
async fn status(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Json<Value>, AdminApiError> {
    let choices = if ctx.has_unscoped(C::DomainsUpstreamWrite) {
        state
            .security
            .jmap_upstreams
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|url| crate::upstream_list_origin(&url))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    Ok(Json(
        json!({"mode":if state.engine.is_some(){"engine"}else{"proxy"},
        "blockedAccounts":blocked_accounts(&state,&ctx).await?,"upstreamChoices":choices,
        "transportSecurity":{"runtimeReady":true,"auto":true,"customCa":true,"serverName":true,"protocols":["imap","pop3","smtp","sieve"]},
        "authentication":crate::domain_auth::capabilities()}),
    ))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UsernamePreview {
    identity: String,
    username: crate::username_format::UsernameFormat,
}

async fn username_preview(
    ctx: AdminCtx,
    Json(body): Json<UsernamePreview>,
) -> Result<Json<Value>, AdminApiError> {
    ctx.require_unscoped(C::DomainsUpstreamWrite)?;
    let formatted = body
        .username
        .render(&body.identity)
        .map_err(|e| invalid("/username", e))?;
    Ok(Json(
        json!({"identity": body.identity, "authenticationUsername": formatted, "credentialsSent": false}),
    ))
}

fn name(value: &str) -> Result<String, AdminApiError> {
    normalize_domain(value).map_err(|e| invalid("/name", e))
}
fn dto(row: &DomainConfigRow) -> Value {
    json!({"name":row.name,"enabled":row.enabled,"preferred":row.preferred,
        "upstream":serde_json::from_str::<Value>(&row.upstream_json).unwrap_or(Value::Null),
        "selection":UpstreamConfig::parse(&row.upstream_json).ok().and_then(|config| config.selection),
        "allowlist":serde_json::from_str::<Value>(&row.allowlist_json).unwrap_or(Value::Null),
        "blocklist":serde_json::from_str::<Value>(&row.blocklist_json).unwrap_or(Value::Null),
        "settings":serde_json::from_str::<Value>(&row.settings_json).unwrap_or(Value::Null),"updatedAt":row.updated_at})
}
async fn read(
    state: &AppState,
    ctx: &AdminCtx,
    value: &str,
) -> Result<DomainConfigRow, AdminApiError> {
    let name = name(value)?;
    ctx.require_domain(&name)?;
    state
        .store
        .get_domain_config(&name)
        .await?
        .ok_or_else(|| AdminApiError::not_found("Domain not found."))
}
async fn list(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    let scope = ctx.domain_filter(C::DomainsRead);
    let mut aliases = std::collections::BTreeMap::<String, Vec<String>>::new();
    for (domain, alias) in state.store.list_domain_aliases().await? {
        if scope.is_none_or(|domains| domains.contains(&domain)) {
            aliases.entry(domain).or_default().push(alias);
        }
    }
    let rows: Vec<Value> = state
        .store
        .list_domain_configs()
        .await?
        .iter()
        .filter(|row| scope.is_none_or(|domains| domains.contains(&row.name)))
        .map(|row| {
            let mut value = dto(row);
            value["aliases"] = json!(aliases.remove(&row.name).unwrap_or_default());
            value
        })
        .collect();
    let etag = etag_of(&json!(rows));
    Ok(with_etag(Json(rows), &etag))
}
async fn get(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(value): Path<String>,
) -> Result<Response, AdminApiError> {
    let row = read(&state, &ctx, &value).await?;
    let mut body = dto(&row);
    body["aliases"] = json!(state.store.domain_aliases(&row.name).await?);
    body["hosts"] = json!(state.store.domain_hosts(&row.name).await?);
    body["mappingEtag"] = json!(mapping_etag(&row, &body["aliases"], &body["hosts"]));
    body["preferenceEtag"] = json!(preference_etag(
        state.store.preferred_domain_record().await?.as_ref()
    ));
    body["mode"] = json!(if state.engine.is_some() {
        "engine"
    } else {
        "proxy"
    });
    Ok(with_etag(Json(body), &etag_of(&dto(&row))))
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Put {
    /// Older clients supplied these fields; the URL remains authoritative.
    name: Option<String>,
    upstream_json: Option<String>,
    upstream: Option<Value>,
    enabled: Option<bool>,
    settings: Option<Value>,
    allowlist: Option<Vec<String>>,
    blocklist: Option<Vec<String>>,
    #[serde(default)]
    dry_run: bool,
}
fn validate_list(items: &[String], field: &str) -> Result<(), AdminApiError> {
    if items.len() > 1000
        || items
            .iter()
            .any(|item| item.is_empty() || item.len() > 320 || item.chars().any(char::is_control))
    {
        return Err(invalid(
            field,
            "Use at most 1000 nonempty address patterns of 320 characters.",
        ));
    }
    Ok(())
}
async fn put(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(value): Path<String>,
    headers: HeaderMap,
    payload: Option<Json<Put>>,
) -> Result<Response, AdminApiError> {
    let name = name(&value)?;
    ctx.require_domain(&name)?;
    let previous = state.store.get_domain_config(&name).await?;
    if previous.is_none() && state.store.domain_alias_target(&name).await?.is_some() {
        return Err(invalid("/name", "This name is already a domain alias."));
    }
    let mut next = previous.clone().unwrap_or_else(|| DomainConfigRow {
        name: name.clone(),
        upstream_json: "{}".into(),
        allowlist_json: "[]".into(),
        blocklist_json: "[]".into(),
        enabled: true,
        preferred: false,
        settings_json: "{}".into(),
        updated_at: 0,
    });
    check_if_match(&headers, &etag_of(&dto(&next)), None, Some(next.updated_at))?;
    let legacy_empty = payload.is_none();
    let body = payload.map(|Json(body)| body).unwrap_or_default();
    let legacy = legacy_empty || body.upstream_json.is_some();
    if let Some(body_name) = &body.name
        && normalize_domain(body_name).ok().as_deref() != Some(name.as_str())
    {
        return Err(invalid("/name", "Name must match the URL."));
    }
    if body.upstream.is_some() && body.upstream_json.is_some() {
        return Err(invalid(
            "/upstream",
            "Supply only one upstream representation.",
        ));
    }
    let upstream = body.upstream.or(body
        .upstream_json
        .map(|raw| {
            serde_json::from_str(&raw).map_err(|_| invalid("/upstreamJson", "Invalid JSON."))
        })
        .transpose()?);
    if let Some(upstream) = upstream {
        ctx.require_unscoped(C::DomainsUpstreamWrite)?;
        let config = UpstreamConfig::parse(&upstream.to_string())
            .map_err(|error| invalid("/upstream", error))?;
        if (state.engine.is_some() && config.jmap.is_some())
            || (state.engine.is_none() && config.mail().is_some())
        {
            return Err(invalid(
                "/upstream",
                "The upstream protocol must match the server mode.",
            ));
        }
        next.upstream_json = upstream.to_string();
    }
    if let Some(enabled) = body.enabled {
        next.enabled = enabled;
    }
    if let Some(settings) = body.settings {
        if !settings.is_object() || settings.to_string().len() > 16384 {
            return Err(invalid(
                "/settings",
                "Expected a settings object no larger than 16 KiB.",
            ));
        }
        next.settings_json = settings.to_string();
    }
    if let Some(items) = body.allowlist {
        ctx.require_domain_for(C::DomainsSigninPolicy, &name)?;
        validate_list(&items, "/allowlist")?;
        next.allowlist_json = json!(items).to_string();
    }
    if let Some(items) = body.blocklist {
        ctx.require_domain_for(C::DomainsSigninPolicy, &name)?;
        validate_list(&items, "/blocklist")?;
        next.blocklist_json = json!(items).to_string();
    }
    if body.dry_run {
        return Ok(Json(
            json!({"dryRun":true,"before":previous.as_ref().map(dto),"after":dto(&next)}),
        )
        .into_response());
    }
    next.updated_at = chrono::Utc::now().timestamp();
    if !state
        .store
        .compare_and_put_domain_config(previous.as_ref(), &next)
        .await?
    {
        return Err(AdminApiError::stale(None, None));
    }
    if previous
        .as_ref()
        .is_some_and(|old| old.upstream_json != next.upstream_json || old.enabled != next.enabled)
    {
        stop_domain_accounts(&state, &name).await?;
    }
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "domain-updated",
        &format!("domain:{name}"),
        previous.as_ref().map(dto),
        Some(dto(&next)),
    )
    .await;
    if legacy {
        Ok(with_etag(StatusCode::NO_CONTENT, &etag_of(&dto(&next))))
    } else {
        Ok(with_etag(Json(dto(&next)), &etag_of(&dto(&next))))
    }
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Delete {
    #[serde(default)]
    force: bool,
    #[serde(default)]
    dry_run: bool,
}
async fn delete(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(value): Path<String>,
    headers: HeaderMap,
    Query(query): Query<Delete>,
) -> Result<Response, AdminApiError> {
    let row = read(&state, &ctx, &value).await?;
    check_if_match(&headers, &etag_of(&dto(&row)), None, Some(row.updated_at))?;
    if query.force {
        ctx.require_unscoped(C::DomainsDelete)?;
    }
    if query.dry_run {
        return Ok(Json(
            json!({"dryRun":true,"domain":row.name,"preferred":row.preferred,"force":query.force}),
        )
        .into_response());
    }
    let affected = domain_account_ids(&state, &row.name).await?;
    if !state
        .store
        .delete_domain_config_if_unchanged(&row, query.force)
        .await?
    {
        if state.store.get_domain_config(&row.name).await?.as_ref() != Some(&row) {
            return Err(AdminApiError::stale(None, None));
        }
        return Err(AdminApiError::conflict(
            "domainInUse",
            "The domain is preferred or has signed-in accounts.",
        ));
    }
    if let Some(engine) = &state.engine {
        for id in affected {
            engine.unregister(&id);
        }
    }
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "domain-deleted",
        &format!("domain:{}", row.name),
        Some(dto(&row)),
        None,
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}
async fn enabled(
    state: AppState,
    ctx: AdminCtx,
    value: String,
    headers: HeaderMap,
    enabled: bool,
) -> Result<Response, AdminApiError> {
    put(
        State(state),
        ctx,
        Path(value),
        headers,
        Some(Json(Put {
            enabled: Some(enabled),
            ..Default::default()
        })),
    )
    .await
}
async fn enable(
    State(s): State<AppState>,
    ctx: AdminCtx,
    Path(n): Path<String>,
    h: HeaderMap,
) -> Result<Response, AdminApiError> {
    enabled(s, ctx, n, h, true).await
}
async fn disable(
    State(s): State<AppState>,
    ctx: AdminCtx,
    Path(n): Path<String>,
    h: HeaderMap,
) -> Result<Response, AdminApiError> {
    enabled(s, ctx, n, h, false).await
}
fn mapping_etag(row: &DomainConfigRow, aliases: &Value, hosts: &Value) -> String {
    etag_of(&json!({"row":dto(row),"aliases":aliases,"hosts":hosts}))
}
fn preference_etag(row: Option<&DomainConfigRow>) -> String {
    etag_of(&json!(row.map(dto)))
}
fn mutation_result(result: DomainMutation) -> Result<(), AdminApiError> {
    match result {
        DomainMutation::Applied => Ok(()),
        DomainMutation::Stale => Err(AdminApiError::stale(None, None)),
        DomainMutation::Conflict => Err(AdminApiError::conflict(
            "mappingConflict",
            "This mapping belongs to another domain.",
        )),
        DomainMutation::Shadow => Err(invalid(
            "/name",
            "An alias cannot shadow a configured domain.",
        )),
        DomainMutation::Disabled => Err(AdminApiError::conflict(
            "domainDisabled",
            "Enable the domain before making it preferred.",
        )),
    }
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Mutation {
    #[serde(default)]
    dry_run: bool,
}
async fn preference(
    state: AppState,
    ctx: AdminCtx,
    target: Option<DomainConfigRow>,
    headers: HeaderMap,
    dry_run: bool,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::DomainsWrite)?;
    let previous = state.store.preferred_domain_record().await?;
    check_if_match(&headers, &preference_etag(previous.as_ref()), None, None)?;
    mutation_result(
        state
            .store
            .compare_and_prefer_domain(previous.as_ref(), target.as_ref(), dry_run)
            .await?,
    )?;
    if dry_run {
        return Ok(
            Json(json!({"dryRun":true,"preferred":target.as_ref().map(|r| &r.name)}))
                .into_response(),
        );
    }
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "domain-preference-updated",
        "domains",
        None,
        None,
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}
async fn prefer(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(value): Path<String>,
    headers: HeaderMap,
    Query(query): Query<Mutation>,
    body: Option<Json<Mutation>>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::DomainsWrite)?;
    let target = read(&state, &ctx, &value).await?;
    preference(
        state,
        ctx,
        Some(target),
        headers,
        query.dry_run || body.is_some_and(|b| b.dry_run),
    )
    .await
}
async fn unprefer(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    Query(query): Query<Mutation>,
) -> Result<Response, AdminApiError> {
    preference(state, ctx, None, headers, query.dry_run).await
}
#[allow(clippy::too_many_arguments)]
async fn mapping(
    state: AppState,
    ctx: AdminCtx,
    values: (String, String),
    host: bool,
    remove: bool,
    headers: HeaderMap,
    dry_run: bool,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::DomainsUpstreamWrite)?;
    let row = read(&state, &ctx, &values.0).await?;
    let key = name(&values.1)?;
    let aliases = state.store.domain_aliases(&row.name).await?;
    let hosts = state.store.domain_hosts(&row.name).await?;
    check_if_match(
        &headers,
        &mapping_etag(&row, &json!(aliases), &json!(hosts)),
        None,
        Some(row.updated_at),
    )?;
    mutation_result(
        state
            .store
            .compare_and_map_domain(&row, &aliases, &hosts, &key, host, remove, dry_run)
            .await?,
    )?;
    if dry_run {
        return Ok(Json(
            json!({"dryRun":true,"domain":row.name,"name":key,"host":host,"remove":remove}),
        )
        .into_response());
    }
    crate::admin_accounts::audit(
        &state,
        &ctx,
        if remove {
            "domain-mapping-deleted"
        } else {
            "domain-mapping-updated"
        },
        &format!("domain:{}", row.name),
        None,
        Some(json!({"host":host,"name":key})),
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}
async fn alias_put(
    State(s): State<AppState>,
    c: AdminCtx,
    Path(v): Path<(String, String)>,
    h: HeaderMap,
    Query(query): Query<Mutation>,
    body: Option<Json<Mutation>>,
) -> Result<Response, AdminApiError> {
    mapping(
        s,
        c,
        v,
        false,
        false,
        h,
        query.dry_run || body.is_some_and(|b| b.dry_run),
    )
    .await
}
async fn alias_delete(
    State(s): State<AppState>,
    c: AdminCtx,
    Path(v): Path<(String, String)>,
    h: HeaderMap,
    Query(query): Query<Mutation>,
) -> Result<Response, AdminApiError> {
    mapping(s, c, v, false, true, h, query.dry_run).await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RenameAlias {
    name: String,
}
async fn alias_rename(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(values): Path<(String, String)>,
    headers: HeaderMap,
    Json(input): Json<RenameAlias>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::DomainsUpstreamWrite)?;
    if !headers.contains_key("if-match") {
        return Err(AdminApiError::new(
            StatusCode::PRECONDITION_REQUIRED,
            "preconditionRequired",
            "Reload the alias before renaming it.",
        ));
    }
    let row = read(&state, &ctx, &values.0).await?;
    let old = name(&values.1)?;
    let new = name(&input.name)?;
    let aliases = state.store.domain_aliases(&row.name).await?;
    let hosts = state.store.domain_hosts(&row.name).await?;
    check_if_match(
        &headers,
        &mapping_etag(&row, &json!(aliases), &json!(hosts)),
        None,
        Some(row.updated_at),
    )?;
    mutation_result(
        state
            .store
            .compare_and_rename_domain_alias(&row, &aliases, &hosts, &old, &new)
            .await?,
    )?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "domain-alias-renamed",
        &format!("domain:{}", row.name),
        Some(json!({"alias":old})),
        Some(json!({"alias":new})),
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}
async fn host_put(
    State(s): State<AppState>,
    c: AdminCtx,
    Path(v): Path<(String, String)>,
    h: HeaderMap,
    Query(query): Query<Mutation>,
    body: Option<Json<Mutation>>,
) -> Result<Response, AdminApiError> {
    mapping(
        s,
        c,
        v,
        true,
        false,
        h,
        query.dry_run || body.is_some_and(|b| b.dry_run),
    )
    .await
}
async fn host_delete(
    State(s): State<AppState>,
    c: AdminCtx,
    Path(v): Path<(String, String)>,
    h: HeaderMap,
    Query(query): Query<Mutation>,
) -> Result<Response, AdminApiError> {
    mapping(s, c, v, true, true, h, query.dry_run).await
}
