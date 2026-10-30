//! Read-only dashboard aggregation. No upstream probes or credential reads.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    capabilities::Capability as C,
};
use axum::{
    Json,
    extract::State,
    http::{Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    time::{Duration, UNIX_EPOCH},
};

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/dashboard", C::DashboardRead, summary);
}

fn counts(
    snapshot: &mw_store::diag::DashboardSnapshot,
    allowed: Option<&BTreeSet<String>>,
) -> Value {
    let admits = |domain: &str| {
        snapshot
            .canonical_domain(domain)
            .is_some_and(|d| allowed.is_none_or(|a| a.contains(&d)))
    };
    let domains = snapshot
        .domains
        .iter()
        .filter(|d| admits(&d.name))
        .collect::<Vec<_>>();
    let exact = |key: &str, value: usize| {
        (!snapshot.limited.contains(key) && !snapshot.limited.contains("aliases")).then_some(value)
    };
    json!({"provisionedUsers":exact("provisionedUsers", snapshot.provisioned_domains.iter().filter(|d| admits(d)).count()),
        "accountsSignedIn24h":exact("signins",snapshot.signed_in_24h.iter().filter(|a|snapshot.address_domain(a).is_some_and(|d|admits(&d))).count()),"accountsSignedIn7d":exact("signins",snapshot.signed_in_7d.iter().filter(|a|snapshot.address_domain(a).is_some_and(|d|admits(&d))).count()),
        "domains":exact("domains",domains.len()),
        "domainsWithUpstream":exact("domains",domains.iter().filter(|d|d.configured).count()),
        "domainsDisabled":exact("domains",domains.iter().filter(|d|!d.enabled).count()),
        "activeSessions":null,
        "activeAdminSessions":if allowed.is_none(){snapshot.active_admin_sessions}else{None}})
}

/// Count non-expired mailbox sessions without touching activity or contacting an upstream.
/// Unknown/truncated results stay unavailable rather than reporting a partial total.
async fn active_webmail_sessions(
    state: &AppState,
    snapshot: &mw_store::diag::DashboardSnapshot,
    allowed: Option<&BTreeSet<String>>,
) -> Result<u64, &'static str> {
    if snapshot.limited.contains("aliases") {
        return Err("Domain alias snapshot is incomplete.");
    }
    let rows = state
        .store
        .list_security_sessions(None, 1000)
        .await
        .map_err(|_| "Session storage is unavailable.")?;
    if rows.len() == 1000 {
        return Err("Session query limit was reached; incomplete totals are not reported.");
    }
    let mut policies = std::collections::BTreeMap::new();
    let now = chrono::Utc::now();
    let mut count = 0;
    for row in rows {
        let Some(domain) = snapshot
            .address_domain(&row.username)
            .and_then(|d| snapshot.canonical_domain(&d))
        else {
            continue;
        };
        if allowed.is_some_and(|domains| !domains.contains(&domain)) {
            continue;
        }
        if snapshot
            .domains
            .iter()
            .any(|configured| configured.name == domain && !configured.enabled)
        {
            continue;
        }
        let key = (row.account_id.clone(), row.username.clone());
        if !policies.contains_key(&key) {
            let policy = crate::security_center::policy(state, &row.account_id, &row.username)
                .await
                .map_err(|_| "Session expiry policy is unavailable.")?;
            let gate = crate::account_gate::for_login(
                &state.store,
                &row.account_id,
                &row.username,
                &row.username,
            )
            .await
            .map_err(|_| "Account access policy is unavailable.")?;
            policies.insert(key.clone(), (policy, gate.disabled));
        }
        let (policy, disabled) = &policies[&key];
        if !disabled
            && !crate::security_center::session_deadline_passed(
                &row.created_at,
                &row.last_activity,
                now,
                policy.idle_secs,
                policy.absolute_secs,
            )
        {
            count += 1;
        }
    }
    Ok(count)
}

async fn summary(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    let work = async {
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let global = ctx.has_unscoped(C::DashboardRead);
        let snapshot = state.store.dashboard_snapshot(now, global).await?;
        let scope = ctx.domain_filter(C::DashboardRead);
        let store = if global {
            Some(state.store.store_diagnostics().await?)
        } else {
            None
        };
        let mut unavailable = serde_json::Map::new();
        let active_sessions = match tokio::time::timeout(
            Duration::from_millis(800),
            active_webmail_sessions(&state, &snapshot, scope),
        )
        .await
        {
            Ok(Ok(count)) => Some(count),
            result => {
                let reason = match result {
                    Ok(Err(reason)) => reason,
                    _ => "Session count query budget was exceeded.",
                };
                unavailable.insert("counts.activeSessions".into(), json!(reason));
                None
            }
        };
        unavailable.insert("security".into(),json!("Authentication and access summaries are available in their dedicated administration pages."));
        unavailable.insert("blockedAccounts".into(),json!("Domain admission checks are available in Domains; this dashboard does not probe upstreams."));
        for key in &snapshot.limited {
            unavailable.insert(
                format!("bounded.{key}"),
                json!("The bounded query limit was reached; incomplete totals are not reported."),
            );
        }
        if !global {
            for key in [
                "server",
                "traffic",
                "storage",
                "health",
                "recentAudit",
                "counts.activeAdminSessions",
            ] {
                unavailable.insert(
                    key.into(),
                    json!("Requires Dashboard read access for all domains."),
                );
            }
        }
        let mut audit = Vec::new();
        if global {
            let mut projection =
                crate::audit_projection::Projection::new(&state.store, &ctx, C::DashboardRead);
            for a in snapshot.audit.iter().take(10) {
                let Some(ts) = chrono::DateTime::parse_from_rfc3339(&a.ts)
                    .map(|t| t.timestamp())
                    .ok()
                    .or_else(|| {
                        chrono::NaiveDateTime::parse_from_str(&a.ts, "%Y-%m-%d %H:%M:%S")
                            .ok()
                            .map(|t| t.and_utc().timestamp())
                    })
                else {
                    continue;
                };
                let row = mw_admin::AuditLogEntry {
                    id: a.id.clone(),
                    ts: a.ts.clone(),
                    actor: a.actor.clone(),
                    actor_kind: match a.actor_kind.as_str() {
                        "admin" => mw_admin::ActorKind::Admin,
                        "user" => mw_admin::ActorKind::User,
                        "api-key" => mw_admin::ActorKind::ApiKey,
                        _ => mw_admin::ActorKind::System,
                    },
                    action: a.action.clone(),
                    target: a.target.clone(),
                    detail_json: a.detail.clone(),
                    ip: None,
                };
                if let Some(value)=projection.decorate(&row,json!({"id":a.id,"ts":ts,"actor":a.actor,"action":a.action,"target":a.target})).await? {audit.push(value);}
            }
        }
        let domains = snapshot
            .domains
            .iter()
            .filter(|d| scope.is_none_or(|s| s.contains(&d.name)))
            .collect::<Vec<_>>();
        let checklist = if snapshot.limited.contains("domains") {
            Vec::new()
        } else {
            // Console locations follow `MW_ADMIN_PATH`: the web only renders a
            // checklist link whose href is under the console path in force.
            let domains_href = mw_admin::config::admin_route("/domains");
            vec![
                json!({"id":"domain-exists","done":!domains.is_empty(),"href":domains_href}),
                json!({"id":"domain-upstream","done":domains.iter().any(|d|d.enabled&&d.configured),"href":domains_href}),
                json!({"id":"preferred-domain","done":domains.iter().any(|d|d.preferred),"href":domains_href,"optional":true}),
            ]
        };
        // t96: a missing or stale country database while a country policy is
        // active (sign-ins are refused, or classified from old data).
        let warnings = if global {
            crate::geoip::dashboard_warnings(&state).await
        } else {
            Vec::new()
        };
        let mut totals = counts(&snapshot, scope);
        totals["activeSessions"] = json!(active_sessions);
        let value = json!({"server":global.then(||json!({"version":env!("CARGO_PKG_VERSION"),"buildSha":option_env!("MW_BUILD_SHA"),"mode":if state.engine.is_some(){"engine"}else{"proxy"},"uptimeSecs":state.started.elapsed().as_secs(),"startedAt":state.started_at.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),"basePath":state.base_path.as_ref()})),
            "counts":totals,"security":null,
            "storage":store.as_ref().map(|s|json!({"dbBytes":s.db_bytes,"walBytes":s.wal_bytes,"searchIndexBytes":null,"dataVolume":null})),
            "health":store.as_ref().map(|s|json!({"dbPingMs":s.db_ping_ms,"migration":{"applied":s.migration_applied,"expected":s.migration_expected},"engine":null,"outbox":null,"searchIndex":null,"renderSandbox":null,"pluginRuntime":null})),
            "traffic":global.then(||json!({"windowMins":60,"series":crate::http_metrics::traffic_series()})),
            "recentAudit":audit,"blockedAccounts":null,"warnings":warnings,"checklist":checklist,"unavailable":unavailable});
        Ok::<_, AdminApiError>(([(header::CACHE_CONTROL, "no-store")], Json(value)).into_response())
    };
    tokio::time::timeout(Duration::from_secs(3), work)
        .await
        .map_err(|_| {
            AdminApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "dashboardTimeout",
                "Dashboard query budget was exceeded; try again.",
            )
        })?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_canonicalize_aliases_filter_domains_and_refuse_truncated_totals() {
        use mw_store::diag::{DashboardDomain, DashboardSnapshot};
        let mut snapshot = DashboardSnapshot {
            domains: vec![
                DashboardDomain {
                    name: "a.test".into(),
                    enabled: true,
                    preferred: true,
                    configured: true,
                },
                DashboardDomain {
                    name: "b.test".into(),
                    enabled: false,
                    preferred: false,
                    configured: false,
                },
            ],
            aliases: [("alias.test".into(), "a.test".into())].into(),
            provisioned_domains: vec!["alias.test".into(), "b.test".into()],
            signed_in_24h: vec!["alice@alias.test".into()],
            signed_in_7d: vec!["alice@alias.test".into(), "bob@b.test".into()],
            audit: vec![],
            active_admin_sessions: Some(8),
            enabled_admins: Some(2),
            limited: BTreeSet::new(),
        };
        let allowed = ["a.test".to_owned()].into();
        let limited = counts(&snapshot, Some(&allowed));
        assert_eq!(limited["domains"], 1);
        assert_eq!(limited["provisionedUsers"], 1);
        assert!(limited["activeAdminSessions"].is_null());
        assert_eq!(counts(&snapshot, None)["activeAdminSessions"], 8);
        snapshot.limited.insert("domains");
        assert!(counts(&snapshot, None)["domains"].is_null());
        snapshot.limited.insert("aliases");
        assert!(counts(&snapshot, Some(&allowed))["provisionedUsers"].is_null());
    }
}
