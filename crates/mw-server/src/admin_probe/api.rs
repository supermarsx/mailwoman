//! `{console}/security/admin-probe…`: the policy, its counters and the current
//! blocks, for the Security centre (t97).
//!
//! * `GET    /security/admin-probe` — policy + counters + top offenders (`bans.read`);
//! * `PUT    /security/admin-probe/policy` — change policy keys (`bans.write`,
//!   step-up `banManage`, `If-Match`, audited `admin-probe-policy-changed`);
//! * `GET    /security/admin-probe/blocks` — active blocks (`bans.read`);
//! * `DELETE /security/admin-probe/blocks/{id}` — lift one (`bans.write`, step-up
//!   `banManage`, audited `admin-probe-unblocked`).
use super::config::{KEYS, PREFIX};
use super::{ROUTE, Runtime, display_source, iso, tracker};
use crate::admin_ctx::{AdminApiError, AdminCtx, AdminJson, check_if_match, etag_of, with_etag};
use crate::admin_router::AdminRegistrar;
use crate::capabilities::Capability;
use crate::{AppState, settings_registry};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use mw_store::settings_policy::SettingPolicyRow;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::sync::atomic::Ordering;

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, ROUTE, Capability::BansRead, overview);
    r.admin_endpoint(
        Method::PUT,
        "/security/admin-probe/policy",
        Capability::BansWrite,
        save,
    );
    r.admin_endpoint(
        Method::GET,
        "/security/admin-probe/blocks",
        Capability::BansRead,
        blocks,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/security/admin-probe/blocks/{id}",
        Capability::BansWrite,
        unblock,
    );
}

fn unavailable() -> AdminApiError {
    AdminApiError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "adminProbeUnavailable",
        "Admin-probe protection settings could not be read",
    )
}

fn private(body: Value, etag: Option<&str>) -> Response {
    let response = ([(header::CACHE_CONTROL, "private, no-store")], Json(body)).into_response();
    match etag {
        Some(tag) => with_etag(response, tag),
        None => response,
    }
}

/// One row per source, newest activity first.
fn rows(runtime: &Runtime, policy: &super::Policy, only_blocked: bool) -> Vec<(f64, Value)> {
    let now = runtime.now();
    let mut out: Vec<(f64, u64, Value)> = runtime
        .lock()
        .snapshot(now, &policy.limits)
        .into_iter()
        .filter(|(_, entry, score)| {
            if only_blocked {
                entry.blocked_until_ms.is_some()
            } else {
                entry.blocked_until_ms.is_some() || *score > 0.0 || entry.strikes > 0
            }
        })
        .map(|(key, entry, score)| {
            let rank = if entry.blocked_until_ms.is_some() {
                1e9 + score
            } else {
                score
            };
            (
                rank,
                entry.last_seen_ms,
                json!({
                    "id": runtime.id_of(key),
                    "source": display_source(key, policy.audit_ip),
                    "score": (score * 10.0).round() / 10.0,
                    "graceAllowance": policy.limits.grace,
                    "signals": entry.signals,
                    "lastSignal": entry.last_signal.map(tracker::Signal::as_str),
                    "lastSeen": iso(entry.last_seen_ms),
                    "strikes": entry.strikes,
                    "blocked": entry.blocked_until_ms.is_some(),
                    "blockedAt": entry.blocked_at_ms.and_then(iso),
                    "blockedUntil": entry.blocked_until_ms.and_then(iso),
                    "retryAfterSecs": entry.blocked_until_ms.map(|u| u.saturating_sub(now).div_ceil(1000)),
                }),
            )
        })
        .collect();
    out.sort_by(|a, b| b.0.total_cmp(&a.0).then(b.1.cmp(&a.1)));
    out.into_iter().map(|(rank, _, v)| (rank, v)).collect()
}

async fn snapshot(state: &AppState) -> Result<(Value, String), AdminApiError> {
    let runtime = &state.admin_probe;
    let policy = runtime
        .config
        .get(&state.store)
        .await
        .map_err(|_| unavailable())?;
    let values = policy.summary();
    let etag = etag_of(&values);
    let now = runtime.now();
    let stats = &runtime.stats;
    let blocks_24h = {
        let recent = stats.recent.lock().unwrap_or_else(|p| p.into_inner());
        recent
            .iter()
            .filter(|t| now.saturating_sub(**t) <= 86_400_000)
            .count()
    };
    let all = rows(runtime, &policy, false);
    let active = all
        .iter()
        .filter(|(_, v)| v["blocked"] == json!(true))
        .count();
    let (tracked, capacity, evicted) = {
        let t = runtime.lock();
        (t.len(), t.capacity(), t.evicted)
    };
    let mut by_signal = Map::new();
    for signal in tracker::Signal::ALL {
        by_signal.insert(
            signal.as_str().into(),
            json!(stats.signals[signal as usize].load(Ordering::Relaxed)),
        );
    }
    let body = json!({
        "policy": values,
        "stats": {
            "since": iso(runtime.started_ms),
            "blocksTotal": stats.blocks.load(Ordering::Relaxed),
            "blocksLast24h": blocks_24h,
            "activeBlocks": active,
            "refusedRequests": stats.refused.load(Ordering::Relaxed),
            "tarpittedRequests": stats.tarpitted.load(Ordering::Relaxed),
            "clearedBySignin": stats.cleared.load(Ordering::Relaxed),
            "signals": by_signal,
            "trackedSources": tracked,
            "capacity": capacity,
            "evictedSources": evicted,
        },
        "topOffenders": all.into_iter().take(10).map(|(_, v)| v).collect::<Vec<_>>(),
        "scope": "instance",
        "propagationSecs": 30,
    });
    Ok((body, etag))
}

async fn overview(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::BansRead)?;
    let (body, etag) = snapshot(&state).await?;
    Ok(private(body, Some(&etag)))
}

async fn blocks(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::BansRead)?;
    let runtime = &state.admin_probe;
    let policy = runtime
        .config
        .get(&state.store)
        .await
        .map_err(|_| unavailable())?;
    let list: Vec<Value> = rows(runtime, &policy, true)
        .into_iter()
        .map(|(_, v)| v)
        .collect();
    Ok(private(json!({ "blocks": list }), None))
}

async fn unblock(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::BansWrite)?;
    if id.len() != 32
        || !id
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(AdminApiError::invalid(
            "invalidSource",
            "Unknown source id",
            "/id",
        ));
    }
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::BanManage).await?;
    let runtime = &state.admin_probe;
    let policy = runtime
        .config
        .get(&state.store)
        .await
        .map_err(|_| unavailable())?;
    let now = runtime.now();
    let (found, source) = {
        let mut t = runtime.lock();
        let key = t
            .snapshot(now, &policy.limits)
            .into_iter()
            .map(|(key, _, _)| key)
            .find(|key| runtime.id_of(*key) == id);
        match key {
            Some(key) => (
                t.unblock(key, now),
                Some(display_source(key, policy.audit_ip)),
            ),
            None => (false, None),
        }
    };
    if !found {
        return Err(AdminApiError::new(
            StatusCode::NOT_FOUND,
            "notBlocked",
            "That source is not blocked",
        ));
    }
    runtime.save(&state.store, &policy.limits).await;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "admin-probe-unblocked",
        &format!("source:{id}"),
        Some(json!({ "source": source, "blocked": true })),
        Some(json!({ "blocked": false })),
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Change {
    /// Key (without the `security.adminProbe.` prefix) to value; `null` restores
    /// the default.
    values: Map<String, Value>,
}

async fn save(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(change): AdminJson<Change>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::BansWrite)?;
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::BanManage).await?;
    if change.values.is_empty() {
        return Err(AdminApiError::invalid(
            "noChange",
            "Name at least one value",
            "/values",
        ));
    }
    let before_policy = state
        .admin_probe
        .config
        .get(&state.store)
        .await
        .map_err(|_| unavailable())?;
    let before = before_policy.summary();
    check_if_match(&headers, &etag_of(&before), None, None)?;
    // Validate everything before writing anything.
    let mut next = before.as_object().cloned().unwrap_or_default();
    let mut writes = Vec::new();
    for (name, value) in &change.values {
        let field = format!("/values/{name}");
        if !KEYS.contains(&name.as_str()) {
            return Err(AdminApiError::invalid(
                "unknownKey",
                format!("{name} is not an admin-probe setting"),
                &field,
            ));
        }
        let key = format!("{PREFIX}{name}");
        let def = settings_registry::lookup(&key).ok_or_else(unavailable)?;
        if let Some((var, _)) = def.env
            && std::env::var_os(var).is_some()
        {
            return Err(AdminApiError::conflict(
                "pinnedByEnvironment",
                format!("{var} sets {name}. Change or remove it on the server."),
            ));
        }
        let stored = if value.is_null() {
            next.insert(name.clone(), def.default_value());
            None
        } else {
            let valid = settings_registry::validate(def, value, &field).map_err(|r| {
                AdminApiError::invalid(r.code, r.detail, r.field.as_deref().unwrap_or(&field))
            })?;
            next.insert(name.clone(), valid.clone());
            Some(valid)
        };
        writes.push((key, stored));
    }
    let number = |k: &str| next.get(k).and_then(Value::as_u64).unwrap_or(0);
    if number("maxBlockSecs") < number("blockSecs") {
        return Err(AdminApiError::invalid(
            "invalidValue",
            "maxBlockSecs must be at least blockSecs",
            "/values/maxBlockSecs",
        ));
    }
    let now = chrono::Utc::now().timestamp();
    for (key, stored) in writes {
        let current = state.store.get_setting_policy(&key).await?;
        let row = stored.map(|value| SettingPolicyRow {
            key: key.clone(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: false,
            updated_by: ctx.username.clone(),
            updated_at: now,
        });
        if current.is_none() && row.is_none() {
            continue;
        }
        if !state
            .store
            .compare_setting_policy(&key, current.as_ref(), row.as_ref())
            .await?
        {
            state.admin_probe.invalidate();
            return Err(AdminApiError::stale(None, None));
        }
    }
    state.admin_probe.invalidate();
    let (body, etag) = snapshot(&state).await?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "admin-probe-policy-changed",
        "setting:security.adminProbe",
        Some(before),
        Some(body["policy"].clone()),
    )
    .await;
    Ok(private(body, Some(&etag)))
}
