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
use mw_store::feature_governance::{FeatureMutation, FeatureSnapshot, FeatureWriteError};
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
            // Account-scoped plugin hosts are native account capabilities. A raw
            // upstream account ID or a guessed session name cannot establish one.
            Err(mw_store::StoreError::NotFound) => {
                return Err("native plugin identity unavailable".into());
            }
            Err(_) => return Err("plugin identity unavailable".into()),
        }
    };
    let subject = mw_store::settings_policy::SettingSubject {
        personal_owner: None,
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

/// One user-facing feature a profile can switch or limit (t98).
///
/// * `capability_keys` — boolean registry keys; the feature is on when all are
///   `true` for the subject. A feature with none is a pure limit or list.
/// * `limit_keys` — numeric limits (`0` = no limit) the editor shows beside it.
/// * `setting_keys` — other profile-able keys of the feature (allowed values,
///   policy switches).
/// * `rest_entry_points` / `jmap_families` — where the server refuses it; the
///   guard itself is [`crate::feature_gate`].
/// * `enforcement` — `enforced` (refused by the server), `partial` (some of the
///   limits are not yet counted by the server) or `ui` (only hidden in the web
///   client, because the same data stays reachable another way).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FeatureDef {
    pub id: &'static str,
    pub category: &'static str,
    pub control: &'static str,
    pub capability_keys: &'static [&'static str],
    pub limit_keys: &'static [&'static str],
    pub setting_keys: &'static [&'static str],
    pub rest_entry_points: &'static [&'static str],
    pub jmap_families: &'static [&'static str],
    pub enforcement: &'static str,
}

const fn toggle(
    id: &'static str,
    category: &'static str,
    caps: &'static [&'static str],
    rest: &'static [&'static str],
    jmap: &'static [&'static str],
) -> FeatureDef {
    FeatureDef {
        id,
        category,
        control: "toggle",
        capability_keys: caps,
        limit_keys: &[],
        setting_keys: &[],
        rest_entry_points: rest,
        jmap_families: jmap,
        enforcement: "enforced",
    }
}
const fn limits(mut f: FeatureDef, keys: &'static [&'static str]) -> FeatureDef {
    f.limit_keys = keys;
    f
}
const fn settings(mut f: FeatureDef, keys: &'static [&'static str]) -> FeatureDef {
    f.setting_keys = keys;
    f
}
const fn level(mut f: FeatureDef, enforcement: &'static str) -> FeatureDef {
    f.enforcement = enforcement;
    f
}
const fn control(mut f: FeatureDef, control: &'static str) -> FeatureDef {
    f.control = control;
    f
}

/// The categories, in the editor's order.
pub(crate) const CATEGORIES: &[&str] = &[
    "compose",
    "reading",
    "views",
    "organise",
    "pim",
    "data",
    "accounts",
    "integrations",
    "developer",
    "personal",
];

pub(crate) static CATALOGUE: &[FeatureDef] = &[
    // ── Mail and compose ──
    toggle(
        "compose.rich_text",
        "compose",
        &["cap.compose.rich_text"],
        &[],
        &["Email/set"],
    ),
    limits(
        toggle(
            "compose.attachments",
            "compose",
            &["cap.compose.attachments"],
            &["/jmap/upload"],
            &["Email/set"],
        ),
        &["limit.attachment_bytes"],
    ),
    toggle(
        "compose.share_links",
        "compose",
        &["cap.compose.share_links"],
        &["/api/nextcloud/share-link"],
        &[],
    ),
    toggle(
        "compose.scheduled_send",
        "compose",
        &["cap.compose.scheduled_send"],
        &[],
        &["EmailSubmission/set"],
    ),
    settings(
        toggle(
            "compose.read_receipts",
            "compose",
            &["cap.compose.read_receipts"],
            &[],
            &["Email/set"],
        ),
        &["receipts.dsnAllowed", "mail.readReceipts"],
    ),
    toggle(
        "compose.signatures",
        "compose",
        &["cap.signatures"],
        &["/api/account/signatures"],
        &[],
    ),
    toggle(
        "compose.identities",
        "compose",
        &["cap.identities"],
        &["/api/account/identities"],
        &[],
    ),
    level(
        control(
            limits(
                toggle("compose.sending", "compose", &[], &[], &["Email/set"]),
                &[
                    "limit.recipients_per_message",
                    "limit.send_per_hour",
                    "limit.send_per_day",
                ],
            ),
            "limit",
        ),
        "partial",
    ),
    // ── Reading and privacy ──
    toggle(
        "privacy.remote_images",
        "reading",
        &["cap.remote_images"],
        &["/api/image-proxy", "/api/remote-images/grant"],
        &[],
    ),
    toggle(
        "privacy.direct_content",
        "reading",
        &["cap.remote_content.direct"],
        &["/api/rc"],
        &[],
    ),
    settings(
        toggle(
            "privacy.trusted_links",
            "reading",
            &["cap.links.trusted"],
            &["/api/settings/links.trustedDomains"],
            &[],
        ),
        &["links.trustedDomains", "links.trustedSubdomains"],
    ),
    toggle(
        "reading.zoom",
        "reading",
        &["cap.zoom"],
        &["/api/settings/reader.zoom"],
        &[],
    ),
    // ── Views and search ──
    toggle(
        "views.focused",
        "views",
        &["cap.mail.focused_inbox"],
        &["/api/settings/mail.focusedInbox"],
        &[],
    ),
    toggle(
        "views.events",
        "views",
        &["cap.views.events"],
        &[],
        &["Email/query"],
    ),
    level(
        toggle(
            "views.attachments",
            "views",
            &["cap.views.attachments"],
            &[],
            &[],
        ),
        "ui",
    ),
    toggle(
        "search.operators",
        "views",
        &["cap.search.operators"],
        &[],
        &["Email/query"],
    ),
    toggle(
        "search.saved",
        "views",
        &["cap.saved_searches"],
        &["/api/account/saved-searches"],
        &[],
    ),
    // ── Organise ──
    toggle(
        "rules",
        "organise",
        &["cap.rules"],
        &[],
        &["MailRule", "VacationResponse"],
    ),
    toggle(
        "rules.forward",
        "organise",
        &["cap.rules.forward"],
        &[],
        &["MailRule/set"],
    ),
    settings(
        toggle(
            "sieve.raw",
            "organise",
            &["cap.sieve.raw_edit"],
            &[],
            &["SieveScript"],
        ),
        &[
            "sieve.maxScripts",
            "sieve.maxScriptBytes",
            "sieve.allowedExtensions",
        ],
    ),
    settings(
        toggle(
            "forwarding.external",
            "organise",
            &["cap.sieve.forward_external"],
            &[],
            &[],
        ),
        &["sieve.maxRedirects"],
    ),
    toggle(
        "vacation",
        "organise",
        &["cap.vacation"],
        &[],
        &["VacationResponse"],
    ),
    toggle("labels", "organise", &["cap.labels"], &[], &["Email/set"]),
    toggle("snooze", "organise", &["cap.snooze"], &[], &["Email/set"]),
    toggle(
        "follow_up",
        "organise",
        &["cap.follow_up"],
        &[],
        &["Email/set"],
    ),
    toggle("masked", "organise", &["cap.masked"], &["/api/masked"], &[]),
    // ── Calendar, contacts, tasks, notes, keys ──
    toggle(
        "calendar",
        "pim",
        &["cap.pim.calendar"],
        &["/api/calendar/subscribe", "/api/calendar/refresh"],
        &["Calendar", "CalendarEvent"],
    ),
    toggle(
        "contacts",
        "pim",
        &["cap.pim.contacts"],
        &[],
        &["AddressBook", "ContactCard", "ContactGroup"],
    ),
    toggle(
        "tasks",
        "pim",
        &["cap.pim.tasks"],
        &[],
        &["TaskList", "Task"],
    ),
    toggle(
        "notes",
        "pim",
        &["cap.pim.notes"],
        &[],
        &["Notebook", "Note"],
    ),
    toggle("keys", "pim", &["cap.keys"], &[], &["CryptoKey"]),
    // ── Data ──
    limits(
        toggle("import", "data", &["cap.import"], &["/api/import"], &[]),
        &["import.maxMessages", "import.maxMessageBytes"],
    ),
    toggle("export", "data", &["cap.export"], &["/api/export"], &[]),
    level(
        control(
            limits(
                toggle("storage", "data", &[], &[], &[]),
                &["limit.storage_bytes"],
            ),
            "limit",
        ),
        "partial",
    ),
    // ── Accounts and access ──
    limits(
        toggle(
            "accounts.linked",
            "accounts",
            &["accounts.linkingEnabled"],
            &["/api/accounts/link"],
            &[],
        ),
        &["limit.linked_mailboxes"],
    ),
    settings(
        control(
            toggle(
                "access.protocols",
                "accounts",
                &[],
                &["/jmap", "/api/v1"],
                &[],
            ),
            "list",
        ),
        &["access.protocols"],
    ),
    toggle(
        "apps.native",
        "accounts",
        &["cap.apps.native"],
        &["/jmap", "/api"],
        &[],
    ),
    toggle(
        "notifications.push",
        "accounts",
        &["notifications.pushAllowed"],
        &["/api/push/subscribe"],
        &[],
    ),
    // ── Integrations ──
    toggle(
        "integrations.nextcloud",
        "integrations",
        &["cap.integration.nextcloud"],
        &["/api/nextcloud"],
        &[],
    ),
    level(
        toggle(
            "integrations.google",
            "integrations",
            &["cap.integration.google"],
            &[],
            &[],
        ),
        "enforced",
    ),
    toggle(
        "integrations.microsoft",
        "integrations",
        &["cap.integration.microsoft"],
        &[],
        &[],
    ),
    toggle(
        "plugins.ui",
        "integrations",
        &["cap.plugins"],
        &["/api/ui-plugins"],
        &[],
    ),
    toggle(
        "packages",
        "integrations",
        &["packages.allowed"],
        &["/api/account/packages"],
        &[],
    ),
    settings(
        limits(
            toggle(
                "ai_assistant",
                "integrations",
                &["cap.ai_assistant"],
                &["/api/assist"],
                &[],
            ),
            &["limit.ai_calls_per_day"],
        ),
        &["assist.profile"],
    ),
    toggle(
        "directory",
        "integrations",
        &["cap.directory"],
        &["/api/directory"],
        &[],
    ),
    // ── Developer access ──
    toggle(
        "api_keys",
        "developer",
        &["cap.api_keys"],
        &["/api/keys", "/api/v1"],
        &[],
    ),
    toggle("mcp", "developer", &["cap.mcp"], &["/mcp"], &[]),
    toggle(
        "oauth_apps",
        "developer",
        &["cap.oauth_apps"],
        &["/oauth/consent", "/oauth/decision"],
        &[],
    ),
    // ── Personalisation ──
    toggle(
        "appearance.custom",
        "personal",
        &["cap.appearance.custom"],
        &["/api/account/appearance"],
        &[],
    ),
    toggle(
        "customisation.code",
        "personal",
        &["cap.appearance.custom_code"],
        &["/api/account/appearance/customization"],
        &[],
    ),
    toggle(
        "avatars",
        "personal",
        &["cap.avatars"],
        &["/api/accounts/layout"],
        &[],
    ),
    toggle(
        "shortcuts",
        "personal",
        &["cap.shortcuts"],
        &["/api/settings/shortcuts.*"],
        &[],
    ),
    toggle(
        "analytics",
        "personal",
        &["cap.analytics"],
        &["/api/analytics/events"],
        &[],
    ),
];

fn enforcement(id: &str) -> &'static str {
    CATALOGUE
        .iter()
        .find(|f| f.id == id)
        .map_or("pending", |f| f.enforcement)
}

/// The feature a catalogue id names.
pub(crate) fn feature(id: &str) -> Option<&'static FeatureDef> {
    CATALOGUE.iter().find(|f| f.id == id)
}

/// Every key the catalogue refers to.
pub(crate) fn catalogue_keys() -> impl Iterator<Item = &'static str> {
    CATALOGUE.iter().flat_map(|f| {
        f.capability_keys
            .iter()
            .chain(f.limit_keys)
            .chain(f.setting_keys)
            .copied()
    })
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
        Method::GET,
        "/features/effective",
        Capability::SettingsRead,
        admin_effective,
    );
    r.admin_endpoint(
        Method::POST,
        "/features/profiles/import",
        Capability::SettingsRead,
        import_profile,
    );
    r.admin_endpoint(
        Method::GET,
        "/features/profiles/{id}/export",
        Capability::SettingsRead,
        export_profile,
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

/// `GET /api/features` — every catalogue feature for the signed-in mailbox: on or
/// off, its limits and allowed values, with provenance. The web client hides what
/// is off; the routes refuse it regardless ([`crate::feature_gate`]).
async fn user_features(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    let subject = match UserSubject::resolved(&state.store, &session).await {
        Ok(v) => v,
        Err(_) => return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let resolved = match resolve_features(&state.store, &subject.as_subject()).await {
        Ok(v) => v,
        Err(_) => return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let mut features = serde_json::Map::new();
    for s in resolved {
        let pick = |keys: &[&str]| -> serde_json::Map<String, Value> {
            s.values
                .iter()
                .filter(|v| keys.contains(&v.key.as_str()))
                .map(|v| (v.key.clone(), v.to_user_json()))
                .collect()
        };
        features.insert(
            s.feature.id.into(),
            json!({
                "enabled":s.enabled,"category":s.feature.category,
                "capabilities":pick(s.feature.capability_keys),
                "limits":pick(s.feature.limit_keys),
                "settings":pick(s.feature.setting_keys),
                "enforcement":enforcement(s.feature.id),
            }),
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
    let snapshot = state.store.feature_governance_snapshot().await?;
    let mut value = serde_json::to_value(&snapshot)
        .map_err(|_| AdminApiError::internal("invalid feature catalogue"))?;
    if !ctx.has_unscoped(Capability::SettingsUserValues) {
        value["nativeTargets"] = json!([]);
    }
    value["catalogue"] = json!(catalogue);
    value["integrationPolicyKeys"] = json!(plugin_keys);
    // t98: what the editor needs beside the profiles: the category order, every
    // key a profile may hold (with its kind and bounds), the value each key has
    // with no profile (the diff baseline) and the built-in presets.
    value["categories"] = json!(CATEGORIES);
    value["profileKeys"] = json!(profile_keys());
    value["defaults"] = json!(default_values(&state).await?);
    value["presets"] = json!(
        presets()
            .into_iter()
            .map(|(id, values)| json!({"id":id,"values":values}))
            .collect::<Vec<_>>()
    );
    value["documentFormat"] = json!({"format":PROFILE_FORMAT,"version":PROFILE_FORMAT_VERSION});
    Ok(catalogue_response(&snapshot.revision, value))
}

/// The JSON document a profile is exported as and imported from (t98).
pub(crate) const PROFILE_FORMAT: &str = "mailwoman.featureProfile";
pub(crate) const PROFILE_FORMAT_VERSION: u64 = 1;

/// Check a profile's name and values exactly as a save does: every key is a
/// registered setting that a profile may hold, not owned by a dedicated route
/// and not pinned by the environment, and every value passes the registry's
/// validation (normalised in place). `at` prefixes the JSON pointer of a refusal.
pub(crate) fn validate_profile(
    name: &str,
    values: &mut BTreeMap<String, ProfileValue>,
    at: &str,
) -> Result<(), AdminApiError> {
    if name.trim().is_empty() || name.len() > 128 || name.chars().any(char::is_control) {
        return Err(AdminApiError::invalid(
            "invalidProfile",
            "invalid profile size or name",
            format!("{at}/name"),
        ));
    }
    if values.len() > 256 {
        return Err(AdminApiError::invalid(
            "invalidProfile",
            "a profile holds at most 256 values",
            format!("{at}/values"),
        ));
    }
    for (key, value) in values.iter_mut() {
        let field = format!("{at}/values/{key}");
        let def = registry::lookup(key).ok_or_else(|| {
            AdminApiError::invalid("unknownSetting", "unknown setting", field.clone())
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
                field,
            ));
        }
        value.value =
            registry::validate(def, &value.value, &format!("{field}/value")).map_err(refusal)?;
    }
    Ok(())
}

/// Every registry key a profile may hold, as the admin API describes it.
fn profile_keys() -> Vec<Value> {
    registry::REGISTRY
        .iter()
        .filter(|d| {
            !d.is_family()
                && d.scopes
                    .allows(mw_store::settings_policy::SettingScope::Profile)
                && d.dedicated_route.is_none()
        })
        .map(registry::SettingDef::describe)
        .collect()
}

fn on(keys: &[&str], value: bool) -> BTreeMap<String, ProfileValue> {
    keys.iter()
        .map(|k| {
            (
                (*k).to_string(),
                ProfileValue {
                    value: Value::Bool(value),
                    locked: false,
                },
            )
        })
        .collect()
}

/// Every boolean capability key of the catalogue.
fn capability_keys() -> Vec<&'static str> {
    let mut keys: Vec<&'static str> = CATALOGUE
        .iter()
        .flat_map(|f| f.capability_keys.iter().copied())
        .collect();
    keys.sort_unstable();
    keys.dedup();
    keys
}

/// The built-in presets (t98): starting points for a new profile, never stored
/// as profiles themselves. `Full` allows every feature; `Standard` is the
/// defaults without developer access, own code, raw Sieve, external forwarding
/// and linked mailboxes; `Mail only` turns off calendar, contacts, tasks, notes,
/// keys and the integrations; `Minimal / kiosk` keeps reading and plain
/// composing with attachments.
pub(crate) fn presets() -> Vec<(&'static str, BTreeMap<String, ProfileValue>)> {
    let caps = capability_keys();
    let except = |off: &[&str]| -> BTreeMap<String, ProfileValue> {
        caps.iter()
            .map(|k| {
                (
                    (*k).to_string(),
                    ProfileValue {
                        value: Value::Bool(!off.contains(k)),
                        locked: false,
                    },
                )
            })
            .collect()
    };
    let all_protocols = json!(registry::ACCESS_PROTOCOLS);
    let mut full = on(&caps, true);
    full.insert(
        "access.protocols".into(),
        ProfileValue {
            value: all_protocols,
            locked: false,
        },
    );
    let standard = except(&[
        "cap.api_keys",
        "cap.mcp",
        "cap.oauth_apps",
        "cap.appearance.custom_code",
        "cap.sieve.raw_edit",
        "cap.sieve.forward_external",
        "accounts.linkingEnabled",
    ]);
    let mail_only = except(&[
        "cap.pim.calendar",
        "cap.pim.contacts",
        "cap.pim.tasks",
        "cap.pim.notes",
        "cap.keys",
        "cap.integration.nextcloud",
        "cap.integration.google",
        "cap.integration.microsoft",
        "cap.plugins",
        "packages.allowed",
        "cap.ai_assistant",
        "cap.directory",
        "cap.compose.share_links",
        "cap.api_keys",
        "cap.mcp",
        "cap.oauth_apps",
        "cap.appearance.custom_code",
        "cap.sieve.raw_edit",
        "cap.sieve.forward_external",
        "accounts.linkingEnabled",
    ]);
    let keep = [
        "cap.compose.rich_text",
        "cap.compose.attachments",
        "cap.remote_images",
        "cap.export",
    ];
    let kiosk: BTreeMap<String, ProfileValue> = caps
        .iter()
        .map(|k| {
            (
                (*k).to_string(),
                ProfileValue {
                    value: Value::Bool(keep.contains(k)),
                    locked: true,
                },
            )
        })
        .collect();
    vec![
        ("full", full),
        ("standard", standard),
        ("mail-only", mail_only),
        ("minimal", kiosk),
    ]
}

/// The value each catalogue key has with no profile: the server's global layer
/// (a stored global value, else the built-in default). The editor's diff
/// compares a profile with this.
async fn default_values(state: &AppState) -> Result<BTreeMap<String, Value>, AdminApiError> {
    let policies = state.store.list_setting_policies().await?;
    let mut out = BTreeMap::new();
    for d in registry::REGISTRY.iter().filter(|d| {
        !d.is_family()
            && d.scopes
                .allows(mw_store::settings_policy::SettingScope::Profile)
    }) {
        let stored = policies
            .iter()
            .find(|p| p.key == d.key)
            .and_then(|p| p.global_value.as_deref())
            .and_then(|v| serde_json::from_str(v).ok());
        out.insert(
            d.key.to_string(),
            stored.unwrap_or_else(|| d.default_value()),
        );
    }
    Ok(out)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProfileDocument {
    format: String,
    version: u64,
    name: String,
    values: BTreeMap<String, ProfileValue>,
}

/// `POST /features/profiles/import` — check an exported profile document and
/// answer its normalised name and values. Nothing is stored: the console saves
/// the result with the ordinary `PUT`, under the catalogue revision.
async fn import_profile(
    ctx: AdminCtx,
    AdminJson(body): AdminJson<Value>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsRead)?;
    let doc = parse_profile_document(body)?;
    Ok(Json(json!({"name":doc.0,"values":doc.1})).into_response())
}

/// Parse and validate a profile document (`format`, `version`, `name`, `values`).
pub(crate) fn parse_profile_document(
    body: Value,
) -> Result<(String, BTreeMap<String, ProfileValue>), AdminApiError> {
    if body.get("format").and_then(Value::as_str) != Some(PROFILE_FORMAT) {
        return Err(AdminApiError::invalid(
            "invalidFormat",
            "not a Mailwoman feature profile document",
            "/format",
        ));
    }
    if body.get("version").and_then(Value::as_u64) != Some(PROFILE_FORMAT_VERSION) {
        return Err(AdminApiError::invalid(
            "unsupportedVersion",
            "unsupported feature profile document version",
            "/version",
        ));
    }
    let mut doc: ProfileDocument = serde_json::from_value(body).map_err(|e| {
        AdminApiError::invalid("invalidProfile", format!("invalid document: {e}"), "/")
    })?;
    let _ = (&doc.format, doc.version);
    validate_profile(&doc.name, &mut doc.values, "")?;
    Ok((doc.name.trim().to_string(), doc.values))
}

/// `GET /features/profiles/{id}/export` — one profile as an importable document.
async fn export_profile(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsRead)?;
    let snapshot = state.store.feature_governance_snapshot().await?;
    let profile = snapshot
        .profiles
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| AdminApiError::not_found("unknown profile"))?;
    Ok((
        [(axum::http::header::CACHE_CONTROL, "no-store")],
        Json(profile_document(&profile.name, &profile.values)),
    )
        .into_response())
}

pub(crate) fn profile_document(name: &str, values: &BTreeMap<String, ProfileValue>) -> Value {
    json!({"format":PROFILE_FORMAT,"version":PROFILE_FORMAT_VERSION,"name":name,"values":values})
}

#[derive(Deserialize)]
struct EffectiveQuery {
    #[serde(default)]
    account: Option<String>,
    #[serde(default)]
    domain: Option<String>,
}

/// `GET /features/effective?account=<native id>` or `?domain=<name>` — what each
/// feature resolves to for that user or for a user of that domain, with where
/// each value came from (`source`, `lockedBy`, and the `profile` that supplied
/// a profile value). Uses the same resolver as every route guard.
async fn admin_effective(
    State(state): State<AppState>,
    ctx: AdminCtx,
    axum::extract::Query(q): axum::extract::Query<EffectiveQuery>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsRead)?;
    let (account_id, address, domain) = match (&q.account, &q.domain) {
        (Some(account), None) if !account.is_empty() => {
            let row = match state.store.get_account(account).await {
                Ok(row) => row,
                Err(mw_store::StoreError::NotFound) => {
                    return Err(AdminApiError::not_found("unknown account"));
                }
                Err(e) => return Err(e.into()),
            };
            let domain = row
                .username
                .rsplit_once('@')
                .map(|(_, d)| d.to_ascii_lowercase())
                .unwrap_or_default();
            (account.clone(), row.username.to_ascii_lowercase(), domain)
        }
        (None, Some(domain)) if !domain.is_empty() => {
            let domain =
                mw_store::settings_policy::normalize_policy_domain(domain).map_err(|_| {
                    AdminApiError::invalid("invalidDomain", "invalid domain", "/domain")
                })?;
            (String::new(), String::new(), domain)
        }
        _ => {
            return Err(AdminApiError::invalid(
                "invalidSubject",
                "name exactly one of account or domain",
                "/",
            ));
        }
    };
    let subject = mw_store::settings_policy::SettingSubject {
        personal_owner: None,
        domain: &domain,
        account_id: &account_id,
        address: &address,
    };
    let preview = resolve_features(&state.store, &subject).await?;
    let (profiles, groups) = state
        .store
        .setting_profile_and_group_ids(&account_id, &domain)
        .await?;
    let snapshot = state.store.feature_governance_snapshot().await?;
    let mut features = Vec::new();
    for state in preview {
        let mut keys = serde_json::Map::new();
        for e in &state.values {
            let mut v = e.to_user_json();
            if e.source == mw_store::settings_policy::SettingSource::Profile
                && let Some(p) = profiles.iter().find(|id| {
                    snapshot
                        .profiles
                        .iter()
                        .any(|p| &p.id == *id && p.values.contains_key(&e.key))
                })
            {
                v["profile"] = json!(p);
            }
            keys.insert(e.key.clone(), v);
        }
        features.push(json!({"id":state.feature.id,"category":state.feature.category,"enabled":state.enabled,"enforcement":state.feature.enforcement,"keys":keys}));
    }
    Ok((
        [(axum::http::header::CACHE_CONTROL, "no-store")],
        Json(json!({
            "subject":{"account":account_id,"address":address,"domain":domain},
            "profiles":profiles,"groups":groups,"features":features,
        })),
    )
        .into_response())
}

/// One feature as resolved for one subject.
pub(crate) struct FeatureState {
    pub feature: &'static FeatureDef,
    /// Every capability key is `true`.
    pub enabled: bool,
    pub values: Vec<registry::Effective>,
}

/// THE feature resolver (t98): every catalogue feature for one subject, from one
/// read of the stored policy. The order is the settings chain
/// ([`mw_store::settings_policy::resolve`]): environment pin > global lock >
/// the broadest lock among domain, profile, group, admin per-user > the
/// narrowest stored value (user, admin per-user, group, profile, domain) > the
/// global value > the built-in default. Profiles are ordered by assignment
/// priority, then user over group over domain assignments.
pub(crate) async fn resolve_features(
    store: &mw_store::Store,
    subject: &mw_store::settings_policy::SettingSubject<'_>,
) -> Result<Vec<FeatureState>, mw_store::StoreError> {
    let wanted: Vec<&str> = catalogue_keys().collect();
    let values = registry::effective_all(store, subject, |d| wanted.contains(&d.key)).await?;
    Ok(CATALOGUE
        .iter()
        .map(|f| {
            let mine: Vec<registry::Effective> = values
                .iter()
                .filter(|v| {
                    f.capability_keys.contains(&v.key.as_str())
                        || f.limit_keys.contains(&v.key.as_str())
                        || f.setting_keys.contains(&v.key.as_str())
                })
                .cloned()
                .collect();
            let enabled = f.capability_keys.iter().all(|key| {
                mine.iter()
                    .any(|v| v.key == *key && v.value.as_bool() == Some(true))
            });
            FeatureState {
                feature: f,
                enabled,
                values: mine,
            }
        })
        .collect())
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
    headers: HeaderMap,
    Path(id): Path<String>,
    AdminJson(mut input): AdminJson<ProfileInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    let expected = expected_revision(&headers)?;
    validate_profile(&input.name, &mut input.values, "")?;
    let changed = state
        .store
        .mutate_feature_governance(
            &expected,
            FeatureMutation::PutProfile(FeatureProfile {
                id: id.clone(),
                name: input.name,
                values: input.values,
                builtin: false,
                updated_at: chrono::Utc::now().timestamp(),
            }),
            &ctx.username,
        )
        .await
        .map_err(feature_error)?;
    audit(&state, &ctx, "profile-changed", &id).await;
    Ok(write_response(changed))
}
async fn delete_profile(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    let expected = expected_revision(&headers)?;
    let changed = state
        .store
        .mutate_feature_governance(
            &expected,
            FeatureMutation::DeleteProfile(id.clone()),
            &ctx.username,
        )
        .await
        .map_err(feature_error)?;
    audit(&state, &ctx, "profile-deleted", &id).await;
    Ok(write_response(changed))
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
    headers: HeaderMap,
    Path(id): Path<String>,
    AdminJson(input): AdminJson<GroupInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    let expected = expected_revision(&headers)?;
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
    let changed = state
        .store
        .mutate_feature_governance(
            &expected,
            FeatureMutation::PutGroup(UserGroup {
                id: id.clone(),
                name: input.name,
                domain: input.domain,
                source: "manual".into(),
                ldap_dn: None,
                created_at: chrono::Utc::now().timestamp(),
            }),
            &ctx.username,
        )
        .await
        .map_err(feature_error)?;
    audit(&state, &ctx, "group-changed", &id).await;
    Ok(write_response(changed))
}
async fn delete_group(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    let expected = expected_revision(&headers)?;
    let changed = state
        .store
        .mutate_feature_governance(
            &expected,
            FeatureMutation::DeleteGroup(id.clone()),
            &ctx.username,
        )
        .await
        .map_err(feature_error)?;
    audit(&state, &ctx, "group-deleted", &id).await;
    Ok(write_response(changed))
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
    let snapshot = state.store.feature_governance_snapshot().await?;
    let members = snapshot
        .memberships
        .get(&id)
        .ok_or_else(|| AdminApiError::not_found("unknown group"))?;
    Ok(catalogue_response(
        &snapshot.revision,
        json!({"members":members,"revision":snapshot.revision}),
    ))
}
async fn put_members(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    Path(id): Path<String>,
    AdminJson(input): AdminJson<MembersInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsUserValues)?;
    let expected = expected_revision(&headers)?;
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
    let changed = state
        .store
        .mutate_feature_governance(
            &expected,
            FeatureMutation::Members {
                group: id.clone(),
                members: input.members,
            },
            &ctx.username,
        )
        .await
        .map_err(feature_error)?;
    audit(&state, &ctx, "group-members-changed", &id).await;
    Ok(write_response(changed))
}
async fn put_assignment(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(input): AdminJson<ProfileAssignment>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsUserValues)?;
    let expected = expected_revision(&headers)?;
    let changed = state
        .store
        .mutate_feature_governance(
            &expected,
            FeatureMutation::PutAssignment(input.clone()),
            &ctx.username,
        )
        .await
        .map_err(feature_error)?;
    audit(&state, &ctx, "profile-assigned", &input.profile_id).await;
    Ok(write_response(changed))
}
async fn delete_assignment(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(input): AdminJson<ProfileAssignment>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsUserValues)?;
    let expected = expected_revision(&headers)?;
    let changed = state
        .store
        .mutate_feature_governance(
            &expected,
            FeatureMutation::DeleteAssignment(input.clone()),
            &ctx.username,
        )
        .await
        .map_err(feature_error)?;
    audit(&state, &ctx, "profile-unassigned", &input.profile_id).await;
    Ok(write_response(changed))
}

fn expected_revision(headers: &HeaderMap) -> Result<String, AdminApiError> {
    let value = headers.get(axum::http::header::IF_MATCH).ok_or_else(|| {
        AdminApiError::new(
            axum::http::StatusCode::PRECONDITION_REQUIRED,
            "preconditionRequired",
            "reload the feature catalogue before saving",
        )
    })?;
    let value = value.to_str().unwrap_or("");
    if headers.get_all(axum::http::header::IF_MATCH).iter().count() != 1
        || value.len() != 70
        || !value.starts_with("\"fg1_")
        || !value.ends_with('"')
        || !value[5..69].bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(AdminApiError::new(
            axum::http::StatusCode::PRECONDITION_FAILED,
            "preconditionFailed",
            "the catalogue revision must match exactly",
        ));
    }
    Ok(value[1..value.len() - 1].to_string())
}
fn feature_error(error: FeatureWriteError) -> AdminApiError {
    match error {
        FeatureWriteError::Store(error) => error.into(),
        FeatureWriteError::Stale => AdminApiError::new(
            axum::http::StatusCode::PRECONDITION_FAILED,
            "preconditionFailed",
            "the feature catalogue changed; reload before saving",
        ),
        FeatureWriteError::Invalid(code) => {
            AdminApiError::invalid(code, "invalid feature governance change", "/")
        }
    }
}
fn catalogue_response(revision: &str, value: Value) -> Response {
    (
        [
            (axum::http::header::ETAG, format!("\"{revision}\"")),
            (axum::http::header::CACHE_CONTROL, "no-store".into()),
        ],
        Json(value),
    )
        .into_response()
}
fn write_response(snapshot: FeatureSnapshot) -> Response {
    catalogue_response(
        &snapshot.revision,
        json!({"ok":true,"revision":snapshot.revision}),
    )
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

#[cfg(test)]
mod t98_tests {
    use super::*;
    use mw_store::feature_profiles::{FeatureProfile, ProfileAssignment, ProfileValue};
    use mw_store::settings_policy::{
        SettingPolicyRow, SettingScope, SettingSource, SettingSubject, SettingValueRow,
    };

    async fn store() -> mw_store::Store {
        let s = mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        s.put_domain_config(&mw_store::DomainConfigRow {
            name: "example.test".into(),
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
        s
    }
    fn profile(id: &str, values: &[(&str, Value, bool)]) -> FeatureProfile {
        FeatureProfile {
            id: id.into(),
            name: id.into(),
            values: values
                .iter()
                .map(|(k, v, locked)| {
                    (
                        (*k).to_string(),
                        ProfileValue {
                            value: v.clone(),
                            locked: *locked,
                        },
                    )
                })
                .collect(),
            builtin: false,
            updated_at: 1,
        }
    }
    async fn assign(s: &mw_store::Store, kind: &str, id: &str, profile: &str, priority: i64) {
        s.put_feature_profile_assignment(&ProfileAssignment {
            scope_kind: kind.into(),
            scope_id: id.into(),
            profile_id: profile.into(),
            priority,
        })
        .await
        .unwrap();
    }
    const SUBJECT: SettingSubject<'static> = SettingSubject {
        personal_owner: None,
        domain: "example.test",
        account_id: "acct-1",
        address: "ana@example.test",
    };
    async fn state_of(s: &mw_store::Store, id: &str) -> FeatureState {
        resolve_features(s, &SUBJECT)
            .await
            .unwrap()
            .into_iter()
            .find(|f| f.feature.id == id)
            .unwrap()
    }
    fn value(scope: SettingScope, id: &str, key: &str, v: &str, locked: bool) -> SettingValueRow {
        SettingValueRow {
            scope,
            scope_id: id.into(),
            key: key.into(),
            value: v.into(),
            locked,
            updated_by: "t".into(),
            updated_at: 1,
        }
    }
    fn policy(key: &str, global: &str, locked: bool) -> SettingPolicyRow {
        SettingPolicyRow {
            key: key.into(),
            global_value: Some(global.into()),
            allow_user_override: false,
            global_locked: locked,
            updated_by: "t".into(),
            updated_at: 1,
        }
    }

    /// The documented order: default < global < domain < profile < admin per-user,
    /// profiles by priority, and the broadest lock wins over every narrower scope.
    #[tokio::test]
    async fn resolver_precedence_default_global_domain_profile_user_and_locks() {
        let s = store().await;
        let key = "cap.masked";
        let source = |f: &FeatureState| f.values.iter().find(|v| v.key == key).unwrap().source;
        let f = state_of(&s, "masked").await;
        assert!(f.enabled);
        assert_eq!(source(&f), SettingSource::Default);
        s.put_setting_policy(&policy(key, "false", false))
            .await
            .unwrap();
        let f = state_of(&s, "masked").await;
        assert!(!f.enabled);
        assert_eq!(source(&f), SettingSource::Global);
        s.put_setting_value(&value(
            SettingScope::Domain,
            "example.test",
            key,
            "true",
            false,
        ))
        .await
        .unwrap();
        let f = state_of(&s, "masked").await;
        assert!(f.enabled);
        assert_eq!(source(&f), SettingSource::Domain);
        s.put_feature_profile(&profile("low", &[(key, json!(false), false)]), "t")
            .await
            .unwrap();
        s.put_feature_profile(&profile("high", &[(key, json!(true), false)]), "t")
            .await
            .unwrap();
        assign(&s, "domain", "example.test", "low", 1).await;
        let f = state_of(&s, "masked").await;
        assert!(!f.enabled);
        assert_eq!(source(&f), SettingSource::Profile);
        assign(&s, "domain", "example.test", "high", 5).await;
        assert!(state_of(&s, "masked").await.enabled, "higher priority wins");
        s.put_setting_value(&value(
            SettingScope::AdminUser,
            "acct-1",
            key,
            "false",
            false,
        ))
        .await
        .unwrap();
        let f = state_of(&s, "masked").await;
        assert!(!f.enabled);
        assert_eq!(source(&f), SettingSource::AdminUser);
        s.put_setting_value(&value(
            SettingScope::Domain,
            "example.test",
            key,
            "true",
            true,
        ))
        .await
        .unwrap();
        let f = state_of(&s, "masked").await;
        assert!(f.enabled, "a domain lock beats profiles and the user value");
        assert_eq!(f.values[0].locked_by, Some(SettingSource::Domain));
        s.put_setting_policy(&policy(key, "false", true))
            .await
            .unwrap();
        let f = state_of(&s, "masked").await;
        assert!(!f.enabled, "a global lock beats the domain lock");
        assert_eq!(f.values[0].locked_by, Some(SettingSource::Global));
    }

    /// Upgrade: a profile written before t98 holds only the old keys. It keeps
    /// what it switched off, and every feature added by t98 resolves to allowed.
    #[tokio::test]
    async fn existing_profiles_keep_behaviour_and_new_features_default_to_allowed() {
        let s = store().await;
        s.put_feature_profile(
            &profile(
                "legacy",
                &[
                    ("cap.pim.contacts", json!(false), true),
                    ("cap.import", json!(false), false),
                ],
            ),
            "t",
        )
        .await
        .unwrap();
        assign(&s, "domain", "example.test", "legacy", 1).await;
        let all = resolve_features(&s, &SUBJECT).await.unwrap();
        let on = |id: &str| all.iter().find(|f| f.feature.id == id).unwrap().enabled;
        assert!(!on("contacts"));
        assert!(!on("import"));
        for id in [
            "compose.rich_text",
            "compose.attachments",
            "compose.share_links",
            "compose.scheduled_send",
            "compose.read_receipts",
            "compose.signatures",
            "compose.identities",
            "privacy.trusted_links",
            "reading.zoom",
            "views.focused",
            "views.events",
            "views.attachments",
            "search.operators",
            "search.saved",
            "vacation",
            "labels",
            "snooze",
            "follow_up",
            "masked",
            "keys",
            "apps.native",
            "directory",
            "mcp",
            "appearance.custom",
            "customisation.code",
            "avatars",
            "shortcuts",
            "analytics",
            "calendar",
            "tasks",
            "notes",
        ] {
            assert!(on(id), "{id} must stay allowed after the upgrade");
        }
        for f in CATALOGUE {
            for key in f.capability_keys {
                let def = registry::lookup(key).unwrap();
                if key.starts_with("cap.")
                    && !matches!(
                        *key,
                        "cap.sieve.raw_edit"
                            | "cap.sieve.forward_external"
                            | "cap.remote_content.direct"
                    )
                {
                    assert_eq!(def.default, "true", "{key}");
                }
            }
        }
        assert_eq!(
            registry::lookup("limit.linked_mailboxes").unwrap().default,
            "0"
        );
        assert_eq!(
            registry::lookup("access.protocols")
                .unwrap()
                .default_value(),
            json!(["imap", "pop3", "ews"])
        );
    }

    fn keys_of(f: &FeatureDef) -> Vec<&'static str> {
        f.capability_keys
            .iter()
            .chain(f.limit_keys)
            .chain(f.setting_keys)
            .copied()
            .collect()
    }

    #[test]
    fn catalogue_keys_exist_and_profiles_may_hold_them() {
        let mut ids = std::collections::BTreeSet::new();
        for f in CATALOGUE {
            assert!(ids.insert(f.id), "duplicate feature {}", f.id);
            assert!(CATEGORIES.contains(&f.category), "{}", f.id);
            assert!(
                matches!(f.enforcement, "enforced" | "partial" | "ui"),
                "{}",
                f.id
            );
            for key in keys_of(f) {
                let def = registry::lookup(key).unwrap_or_else(|| panic!("{key}"));
                assert!(
                    def.scopes.allows(SettingScope::Profile),
                    "{key} must be profile-able"
                );
            }
        }
    }

    #[test]
    fn presets_validate_like_a_saved_profile() {
        let ids: Vec<_> = presets().into_iter().map(|(id, _)| id).collect();
        assert_eq!(ids, ["full", "standard", "mail-only", "minimal"]);
        for (id, mut values) in presets() {
            validate_profile(id, &mut values, "").unwrap_or_else(|e| panic!("{id}: {}", e.code));
        }
        let mail_only = presets()
            .into_iter()
            .find(|(id, _)| *id == "mail-only")
            .unwrap()
            .1;
        for key in [
            "cap.pim.calendar",
            "cap.pim.tasks",
            "cap.pim.notes",
            "cap.pim.contacts",
        ] {
            assert_eq!(mail_only[key].value, json!(false), "{key}");
        }
        assert_eq!(mail_only["cap.compose.rich_text"].value, json!(true));
        let full = presets()
            .into_iter()
            .find(|(id, _)| *id == "full")
            .unwrap()
            .1;
        assert!(
            full.values()
                .filter(|v| v.value.is_boolean())
                .all(|v| v.value == json!(true))
        );
    }

    #[test]
    fn import_documents_are_validated_and_round_trip() {
        let values: BTreeMap<String, ProfileValue> = [(
            "cap.pim.calendar".to_string(),
            ProfileValue {
                value: json!(false),
                locked: true,
            },
        )]
        .into();
        let doc = profile_document("Mail only", &values);
        let (name, back) = parse_profile_document(doc.clone()).unwrap();
        assert_eq!((name.as_str(), &back), ("Mail only", &values));
        let code = |edit: &dyn Fn(&mut Value)| {
            let mut d = doc.clone();
            edit(&mut d);
            parse_profile_document(d).unwrap_err().code.to_string()
        };
        assert_eq!(code(&|d| d["format"] = json!("other")), "invalidFormat");
        assert_eq!(code(&|d| d["version"] = json!(2)), "unsupportedVersion");
        assert_eq!(code(&|d| d["name"] = json!("")), "invalidProfile");
        assert_eq!(code(&|d| d["extra"] = json!(1)), "invalidProfile");
        assert_eq!(
            code(&|d| d["values"]["nope.key"] = json!({"value":true})),
            "unknownSetting"
        );
        assert_eq!(
            code(&|d| d["values"]["cap.pim.calendar"] = json!({"value":"yes"})),
            "invalidValue"
        );
        assert_eq!(
            code(&|d| d["values"]["access.protocols"] = json!({"value":["imap","smtp"]})),
            "invalidValue"
        );
        assert_eq!(
            code(&|d| d["values"]["accounts.maxLinked"] = json!({"value":3})),
            "invalidScope"
        );
    }
}
