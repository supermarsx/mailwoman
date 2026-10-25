//! Read-only effective configuration. No write endpoint accepts catalogue ids.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, etag_of, with_etag},
    admin_router::AdminRegistrar,
    capabilities::Capability,
    knobs, settings_registry as registry,
};
use axum::{Json, extract::State, http::Method, response::Response};
use serde_json::{Value, json};

/// Sanitized boot facts retained when the corresponding raw AppConfig is consumed.
#[derive(Clone)]
pub(crate) struct BootConfigSnapshot {
    database: Value,
    persistent_key: bool,
}

impl BootConfigSnapshot {
    pub(crate) fn from_config(config: &crate::AppConfig) -> Self {
        Self {
            database: knobs::redact(knobs::ValueKind::Dsn, Some(&config.db_path)),
            persistent_key: config.server_key_hex.is_some(),
        }
    }
}

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/config/effective",
        Capability::ConfigRead,
        effective,
    );
}

async fn effective(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::ConfigRead)?;
    let security = if ctx.has_unscoped(Capability::SecurityRead) {
        Some(
            state
                .v6
                .admin
                .effective_security_policy()
                .await
                .map_err(AdminApiError::internal)?,
        )
    } else {
        None
    };
    let mut rows = Vec::new();
    for mut row in knobs::catalogue_rows(&registry::process_env) {
        let id = row["id"].as_str().unwrap_or_default().to_owned();
        let knob = knobs::lookup(&id).expect("catalogue row");
        let cap = read_capability(knob.group);
        if !ctx.has_unscoped(cap) {
            continue;
        }
        row["readCapability"] = json!(cap.as_str());
        row["readOnly"] = json!(true);
        if id == "MW_SERVER_KEY" {
            row["persistent"] = json!(state.config_snapshot.persistent_key);
        }
        row["valueStatus"] = json!("unavailable");
        // Environment input is not evidence of what a running subsystem resolved.
        // Keep it separate from the effective value, and never fabricate a default.
        row["configuredInput"] = knobs::redact(
            knob.kind,
            knob.env_var().and_then(registry::process_env).as_deref(),
        );
        row["value"] = Value::Null;
        let security_value = security.as_ref().and_then(|p| match id.as_str() {
            "MW_ADMIN_MIN_TLS" => Some(json!(p.policy.min_tls.as_str())),
            "MW_ADMIN_REQUIRE_2FA" => Some(json!(p.require_2fa)),
            _ => None,
        });
        if let Some(value) = security_value {
            row["value"] = value;
            row["valueStatus"] = json!("resolved");
            row["source"] = json!(if security
                .as_ref()
                .is_some_and(|p| p.pinned_by_env.iter().any(|pin| pin.var == id))
            {
                "env-pin"
            } else {
                "stored"
            });
        } else if let Some(value) = runtime_value(&state, &id) {
            row["value"] = knobs::redact_json(knob.kind, &value);
            let is_constant = id.starts_with("const:");
            row["valueStatus"] = json!(if is_constant { "build" } else { "resolved" });
            row["source"] = json!(if is_constant { "build" } else { "boot" });
        }
        row["href"] = json!(format!("#/config?field={}", knob.anchor()));
        row["searchText"] = json!(format!("{} {} {}", id, knob.what, knob.configured_by));
        row["etag"] = json!(etag_of(&row));
        rows.push(row);
    }
    if ctx.has_unscoped(Capability::SettingsRead) {
        let policies = state.store.list_setting_policies().await?;
        let mut keys: Vec<_> = registry::REGISTRY
            .iter()
            .filter(|d| !d.is_family())
            .map(|d| d.key.to_string())
            .collect();
        keys.extend(
            policies
                .iter()
                .filter(|p| registry::lookup(&p.key).is_some_and(registry::SettingDef::is_family))
                .map(|p| p.key.clone()),
        );
        keys.sort();
        keys.dedup();
        for key in keys {
            let Some(def) = registry::lookup(&key) else {
                continue;
            };
            let policy = policies.iter().find(|p| p.key == key);
            let resolved = registry::resolve_loaded(
                def,
                &key,
                &Default::default(),
                policy,
                &[],
                &registry::process_env,
            );
            let source = match resolved.source.as_str() {
                "default" => "default",
                "envPin" => "env-pin",
                "envFallback" => "env-fallback",
                "boot" => "boot",
                _ => "stored",
            };
            let mut row = json!({"id":key,"group":def.category,"class":"ui","kind":def.kind.describe(),
                "value":knobs::redact_setting(def,&resolved.value),"source":source,"valueStatus":"resolvedPolicy",
                "default":knobs::redact_setting(def,&def.default_value()),"scope":def.scopes.as_str(),"hot":def.hot(),
                "restartNeeded":!def.hot(),"readOnly":true,"locked":resolved.locked_by.map(|s|s.as_str()),
                "readCapability":Capability::SettingsRead.as_str(),"docs":knobs::docs_href(&key),
                "href":format!("#/settings?field={key}"),"searchText":format!("{} {} {}",key,def.category,def.maps_to),
                "updatedBy":policy.map(|p|&p.updated_by),"updatedAt":policy.map(|p|p.updated_at)});
            if let Some((var, precedence)) = def.env {
                row["envVar"] = json!(var);
                row["precedence"] = json!(precedence.as_str());
            }
            row["etag"] = json!(etag_of(&row));
            rows.push(row);
        }
    }
    let body = Value::Array(rows);
    let etag = etag_of(&body);
    let mut response = with_etag(Json(body), &etag);
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("private, no-store"),
    );
    Ok(response)
}

fn read_capability(group: knobs::Group) -> Capability {
    use knobs::Group::*;
    match group {
        Signin | Admin => Capability::SecurityRead,
        Integrations => Capability::IntegrationsRead,
        Mail => Capability::DomainsRead,
        Observability => Capability::ObservabilityRead,
        StoredConfig => Capability::SettingsRead,
        _ => Capability::ConfigRead,
    }
}

fn runtime_value(state: &AppState, id: &str) -> Option<Value> {
    Some(match id {
        "const:REFRESH_EVERY" => {
            json!(format!("{} s", crate::admin_apply::REFRESH_EVERY.as_secs()))
        }
        "MW_DB_PATH" => state.config_snapshot.database.clone(),
        "MW_SERVER_KEY" => json!(true),
        "MW_BASE_PATH" => json!(state.base_path.as_ref()),
        "MW_MODE" => json!(if state.engine.is_some() {
            "engine"
        } else {
            "proxy"
        }),
        "MW_WEB_DIR" => json!(
            state
                .web_dir
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned())
        ),
        "MW_RENDER_BIN" => json!(
            state
                .render_bin
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned())
        ),
        "MW_RENDER_JAIL" => json!(state.render_jail_required),
        "MW_ADMIN_ENABLED" => json!(state.v6.admin_enabled),
        "MW_COEP" => json!(state.hardening.coep),
        "MW_CSRF_STRICT" => json!(state.hardening.csrf_strict),
        "MW_WKD_DIR" => json!(
            state
                .security
                .wkd_dir
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned())
        ),
        "MW_ABUSE_SPOOL" => json!(
            state
                .security
                .abuse_spool
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned())
        ),
        "MW_JMAP_UPSTREAMS" => json!(state.security.jmap_upstreams),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin_ctx::DomainScope;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn restricted_grants_cannot_read_other_areas_or_global_configuration() {
        let ctx = AdminCtx::with_grants(
            "auditor",
            BTreeMap::from([(Capability::ConfigRead, DomainScope::All)]),
        );
        assert!(ctx.require_unscoped(Capability::ConfigRead).is_ok());
        assert!(!ctx.has_unscoped(read_capability(knobs::Group::Signin)));
        assert!(!ctx.has_unscoped(read_capability(knobs::Group::Mail)));
        assert!(!ctx.has_unscoped(Capability::SettingsRead));
        let domain = AdminCtx::with_grants(
            "domain",
            BTreeMap::from([(
                Capability::ConfigRead,
                DomainScope::Domains(BTreeSet::from(["example.org".into()])),
            )]),
        );
        assert!(domain.require_unscoped(Capability::ConfigRead).is_err());
    }
}
