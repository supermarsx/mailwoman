//! The Adaptive protection page's policy editor and simulator (t97).
//!
//! * `GET  {console}/bans/protection/policy` (`bans.read`): every
//!   `security.protection.*` rule as the settings model describes it (value,
//!   default, bounds, ETag), plus the three presets. The exemption lists are
//!   included only for holders of `settings.read`.
//! * `PUT  {console}/bans/protection/policy` (`settings.write` for every domain,
//!   plus a fresh step-up): several rules at once. Each value is validated by the
//!   settings registry, and the policy that would result is parsed as the
//!   runtime parses it, so no write leaves an unusable policy. Each key is audited
//!   as `setting-changed`, and the request as one `protection-policy-changed`
//!   security event. `null` returns a rule to its default.
//! * `POST {console}/bans/protection/simulate` (`bans.read`): what the current
//!   policy (or the current policy with a draft applied) would decide for a sample
//!   request. Nothing is stored and no counter moves.
//!
//! The runtime enforcement and the registry stay the source of truth: this module
//! only reads them and writes through the same settings path as the console's
//! Settings page.
use super::config::{Config, PREFIX};
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    capabilities::Capability,
    settings_registry,
};
use axum::{
    Json,
    extract::State,
    http::{Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use mw_store::request_protection_model::{Action, Reason, SimInput, simulate as run};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/bans/protection/policy",
        Capability::BansRead,
        read,
    );
    r.admin_endpoint(
        Method::PUT,
        "/bans/protection/policy",
        Capability::SettingsWrite,
        write,
    );
    r.admin_endpoint(
        Method::POST,
        "/bans/protection/simulate",
        Capability::BansRead,
        simulate,
    );
}

const EXEMPT: [&str; 3] = [
    "security.protection.exemptClients",
    "security.protection.exemptAccounts",
    "security.protection.exemptAdmins",
];
const MAX_CHANGES: usize = 64;

/// The presets. `balanced` is the registry default thresholds with both
/// protections enforced; in every preset a signed-in mailbox or administrator
/// gets more requests than an anonymous address; `relaxed` only records (observe) with looser
/// thresholds; `strict` enforces tighter thresholds and longer blocks. A preset
/// never touches the exemption lists or the storage-failure behaviour.
pub(crate) fn presets() -> Value {
    json!({
        "relaxed": {
            "security.protection.rateMode": "observe",
            "security.protection.adaptiveMode": "observe",
            "security.protection.clientRequests": 1200,
            "security.protection.accountRequests": 3600,
            "security.protection.adminRequests": 2400,
            "security.protection.shortAbsoluteAnomalies": 40,
            "security.protection.shortRatioPercent": 70,
            "security.protection.longAbsoluteAnomalies": 200,
            "security.protection.longRatioPercent": 40,
            "security.protection.throttleSecs": 3,
            "security.protection.blockSecs": 120,
        },
        "balanced": {
            "security.protection.rateMode": "enforce",
            "security.protection.adaptiveMode": "enforce",
            "security.protection.clientRequests": 600,
            "security.protection.accountRequests": 1800,
            "security.protection.adminRequests": 1200,
            "security.protection.shortAbsoluteAnomalies": 20,
            "security.protection.shortRatioPercent": 50,
            "security.protection.longAbsoluteAnomalies": 100,
            "security.protection.longRatioPercent": 25,
            "security.protection.throttleSecs": 3,
            "security.protection.blockSecs": 300,
        },
        "strict": {
            "security.protection.rateMode": "enforce",
            "security.protection.adaptiveMode": "enforce",
            "security.protection.clientRequests": 300,
            "security.protection.accountRequests": 900,
            "security.protection.adminRequests": 600,
            "security.protection.shortAbsoluteAnomalies": 10,
            "security.protection.shortRatioPercent": 30,
            "security.protection.longAbsoluteAnomalies": 50,
            "security.protection.longRatioPercent": 15,
            "security.protection.throttleSecs": 10,
            "security.protection.blockSecs": 1800,
        },
    })
}

fn private(body: Value) -> Response {
    ([(header::CACHE_CONTROL, "private, no-store")], Json(body)).into_response()
}
fn protection_keys(with_exempt: bool) -> Vec<&'static str> {
    settings_registry::REGISTRY
        .iter()
        .map(|def| def.key)
        .filter(|key| key.starts_with(PREFIX) && (with_exempt || !EXEMPT.contains(key)))
        .collect()
}

async fn read(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::BansRead)?;
    let with_exempt = ctx.has_unscoped(Capability::SettingsRead);
    let mut rules = Vec::new();
    for key in protection_keys(with_exempt) {
        rules.push(crate::settings_routes::describe_setting(&state, key).await?);
    }
    Ok(private(json!({
        "rules": rules,
        "presets": presets(),
        "canEdit": ctx.has_unscoped(Capability::SettingsWrite),
        "domainScopes": false,
    })))
}

/// The effective protection values with `changes` applied (`null` = default),
/// each change validated by the registry. Errors name `/changes/<i>` or the
/// draft key.
fn candidate(
    current: Vec<settings_registry::Effective>,
    changes: &[(String, Value)],
    field: impl Fn(usize) -> String,
) -> Result<Vec<settings_registry::Effective>, AdminApiError> {
    let mut values = current;
    for (index, (key, value)) in changes.iter().enumerate() {
        let def = settings_registry::lookup(key)
            .filter(|def| def.key.starts_with(PREFIX))
            .ok_or_else(|| {
                AdminApiError::invalid(
                    "notProtectionSetting",
                    format!("{key} is not an adaptive protection rule"),
                    field(index),
                )
            })?;
        let next = if value.is_null() {
            def.default_value()
        } else {
            settings_registry::validate(def, value, &field(index)).map_err(|r| {
                AdminApiError::new(
                    StatusCode::from_u16(r.status).unwrap_or(StatusCode::UNPROCESSABLE_ENTITY),
                    r.code,
                    r.detail,
                )
                .at(field(index))
            })?
        };
        if let Some(row) = values.iter_mut().find(|row| row.key == *key) {
            row.value = next;
        }
    }
    Ok(values)
}
fn parse_or_refuse(rows: Vec<settings_registry::Effective>) -> Result<Config, AdminApiError> {
    Config::parse(rows).map_err(|()| {
        AdminApiError::invalid(
            "invalidPolicy",
            "These values together do not make a usable protection policy (check the window lengths and that every exemption is an IP address or network).",
            "/changes",
        )
    })
}
async fn effective(state: &AppState) -> Result<Vec<settings_registry::Effective>, AdminApiError> {
    Ok(
        settings_registry::effective_all(&state.store, &Default::default(), |def| {
            def.key.starts_with(PREFIX)
        })
        .await?,
    )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Change {
    key: String,
    value: Value,
    #[serde(default)]
    etag: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Write {
    changes: Vec<Change>,
    /// The preset applied, for the audit row only.
    #[serde(default)]
    preset: Option<String>,
}

async fn write(
    State(state): State<AppState>,
    ctx: AdminCtx,
    crate::admin_ctx::AdminJson(body): crate::admin_ctx::AdminJson<Write>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    if body.changes.is_empty() || body.changes.len() > MAX_CHANGES {
        return Err(AdminApiError::invalid(
            "invalidChanges",
            format!("Send between 1 and {MAX_CHANGES} changes"),
            "/changes",
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for (index, change) in body.changes.iter().enumerate() {
        if !seen.insert(change.key.as_str()) {
            return Err(AdminApiError::invalid(
                "duplicateKey",
                format!("{} is changed twice", change.key),
                format!("/changes/{index}/key"),
            ));
        }
    }
    let preset = match body.preset.as_deref() {
        None => None,
        Some(name @ ("relaxed" | "balanced" | "strict")) => Some(name),
        Some(_) => {
            return Err(AdminApiError::invalid(
                "unknownPreset",
                "Unknown preset",
                "/preset",
            ));
        }
    };
    let pairs: Vec<(String, Value)> = body
        .changes
        .iter()
        .map(|c| (c.key.clone(), c.value.clone()))
        .collect();
    // The whole resulting policy must parse as the runtime parses it.
    parse_or_refuse(candidate(effective(&state).await?, &pairs, |i| {
        format!("/changes/{i}/value")
    })?)?;
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::BanManage).await?;
    let triples: Vec<(String, Value, Option<String>)> = body
        .changes
        .into_iter()
        .map(|c| (c.key, c.value, c.etag))
        .collect();
    let rules = crate::settings_routes::write_global_values(&state, &ctx, &triples).await?;
    crate::audit::security_event(
        &state,
        crate::audit::SecurityEvent::admin(&ctx.username, "protection-policy-changed")
            .target("setting:security.protection")
            .detail(json!({
                "keys": triples.iter().map(|t| t.0.clone()).collect::<Vec<_>>(),
                "preset": preset,
            })),
    )
    .await;
    Ok(private(json!({ "rules": rules })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Simulate {
    subject_kind: String,
    #[serde(default)]
    user_agent: Option<String>,
    #[serde(default)]
    subject_value: Option<String>,
    sample: SimInput,
    /// Unsaved values to try instead of the stored ones (`key → value`).
    #[serde(default)]
    draft: Map<String, Value>,
}
fn reason(value: Reason) -> &'static str {
    match value {
        Reason::RequestRate => "request-rate",
        Reason::ShortAbsolute => "short-absolute",
        Reason::ShortRatio => "short-ratio",
        Reason::LongAbsolute => "long-absolute",
        Reason::LongRatio => "long-ratio",
    }
}
fn action(value: Action) -> &'static str {
    match value {
        Action::Allow => "allow",
        Action::Throttle => "throttle",
        Action::Block => "block",
    }
}

async fn simulate(
    State(state): State<AppState>,
    ctx: AdminCtx,
    crate::admin_ctx::AdminJson(body): crate::admin_ctx::AdminJson<Simulate>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::BansRead)?;
    if body
        .user_agent
        .as_ref()
        .is_some_and(|ua| ua.len() > super::user_agent::MAX_UA)
    {
        return Err(AdminApiError::invalid(
            "invalidUserAgent",
            "User agent must be at most 4096 bytes",
            "/userAgent",
        ));
    }
    let (kind, sample_value) = match body.subject_kind.as_str() {
        "client" => ("client", "192.0.2.1"),
        "account" => ("account", "sample-account"),
        "admin" => ("admin", "sample-admin"),
        _ => {
            return Err(AdminApiError::invalid(
                "invalidSubject",
                "The subject is client, account or admin",
                "/subjectKind",
            ));
        }
    };
    let value = body
        .subject_value
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or(sample_value);
    if value.len() > 512 || (kind == "client" && value.parse::<std::net::IpAddr>().is_err()) {
        return Err(AdminApiError::invalid(
            "invalidSubject",
            "Give an IP address for a client, or an ID for an account or administrator",
            "/subjectValue",
        ));
    }
    if body.draft.len() > MAX_CHANGES {
        return Err(AdminApiError::invalid(
            "invalidDraft",
            "Too many draft values",
            "/draft",
        ));
    }
    let draft: BTreeMap<String, Value> = body.draft.into_iter().collect();
    let pairs: Vec<(String, Value)> = draft.into_iter().collect();
    let keys: Vec<String> = pairs.iter().map(|p| p.0.clone()).collect();
    let config = parse_or_refuse(candidate(effective(&state).await?, &pairs, |i| {
        format!("/draft/{}", keys[i])
    })?)?;
    let off = if body.user_agent.is_some() {
        !config.enabled()
    } else {
        !config.legacy_enabled()
    };
    let category = body
        .user_agent
        .as_deref()
        .and_then(|ua| config.user_agent.classify(Some(ua)));
    let ua_policy = category
        .filter(|_| kind == "client" || !config.user_agent.anonymous_only)
        .and_then(|category| {
            config.user_agent_policy(category, if kind == "client" { value } else { "192.0.2.1" })
        });
    let ua_summary = body.user_agent.as_ref().map(
        |_| json!({"classification":category,"claimedOnly":true,"exempt":ua_policy.is_none()}),
    );
    let legacy = config.subject(kind, value).map(|p| run(&p, &body.sample));
    let ua = ua_policy.map(|p| run(&p, &body.sample));
    // Compare actual decisions; an exception in this dimension never bypasses
    // the existing account/client protection policy.
    let rank = |o: &mw_store::request_protection_model::SimOutcome| {
        (
            o.enforced,
            match o.action {
                Action::Allow => 0,
                Action::Throttle => 1,
                Action::Block => 2,
            },
        )
    };
    let outcome = match (legacy, ua) {
        (Some(a), Some(b)) => Some(if rank(&b) > rank(&a) { b } else { a }),
        (a, b) => a.or(b),
    };
    let Some(outcome) = outcome else {
        return Ok(private(json!({
            "userAgent": ua_summary,
            "action": "allow",
            "reason": null,
            "enforced": false,
            "durationSecs": 0,
            "signalCounted": false,
            "off": off,
            "exempt": !off,
            "rules": [],
        })));
    };
    Ok(private(json!({
        "userAgent": ua_summary,
        "action": action(outcome.action),
        "reason": outcome.reason.map(reason),
        "enforced": outcome.enforced,
        "durationSecs": outcome.duration_secs,
        "signalCounted": outcome.signal_counted,
        "off": false,
        "exempt": false,
        "rules": outcome.rules.iter().map(|r| json!({
            "rule": reason(r.rule),
            "enabled": r.enabled,
            "value": r.value,
            "threshold": r.threshold,
            "fired": r.fired,
        })).collect::<Vec<_>>(),
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_preset_value_is_a_valid_registry_value() {
        for (name, preset) in presets().as_object().unwrap() {
            for (key, value) in preset.as_object().unwrap() {
                let def = settings_registry::lookup(key).unwrap_or_else(|| panic!("{name}: {key}"));
                settings_registry::validate(def, value, "/v")
                    .unwrap_or_else(|_| panic!("{name}: {key}={value}"));
                assert!(!EXEMPT.contains(&key.as_str()));
            }
        }
    }
    #[test]
    fn strict_is_never_looser_than_balanced() {
        let p = presets();
        for key in [
            "clientRequests",
            "accountRequests",
            "adminRequests",
            "shortAbsoluteAnomalies",
            "shortRatioPercent",
            "longAbsoluteAnomalies",
            "longRatioPercent",
        ] {
            let k = format!("{PREFIX}{key}");
            assert!(
                p["strict"][&k].as_u64() <= p["balanced"][&k].as_u64(),
                "{key}"
            );
            assert!(
                p["balanced"][&k].as_u64() <= p["relaxed"][&k].as_u64(),
                "{key}"
            );
        }
    }
    #[test]
    fn signed_in_allowances_exceed_the_anonymous_one_in_defaults_and_presets() {
        let default = |key: &str| {
            settings_registry::lookup(&format!("{PREFIX}{key}"))
                .unwrap()
                .default_value()
                .as_u64()
                .unwrap()
        };
        let anonymous = default("clientRequests");
        assert!(default("accountRequests") > anonymous);
        assert!(default("adminRequests") > anonymous);
        let p = presets();
        for name in ["relaxed", "balanced", "strict"] {
            let get = |key: &str| p[name][format!("{PREFIX}{key}")].as_u64().unwrap();
            assert!(get("accountRequests") > get("clientRequests"), "{name}");
            assert!(get("adminRequests") > get("clientRequests"), "{name}");
        }
        assert_eq!(
            p["balanced"][format!("{PREFIX}clientRequests")],
            json!(anonymous)
        );
        assert_eq!(
            p["balanced"][format!("{PREFIX}accountRequests")],
            json!(default("accountRequests"))
        );
        assert_eq!(
            p["balanced"][format!("{PREFIX}adminRequests")],
            json!(default("adminRequests"))
        );
    }
}
