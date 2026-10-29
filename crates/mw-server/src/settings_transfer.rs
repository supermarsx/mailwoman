//! Portable registered global policies. Secrets/dedicated resources use recovery backups.
use super::*;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

const FORMAT: &str = "mailwoman-settings";
const MAX_ENTRIES: usize = 1024;
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Entry {
    key: String,
    global_value: Value,
    locked: bool,
    allow_user_override: bool,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Bundle {
    format: String,
    version: u32,
    entries: Vec<Entry>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Import {
    bundle: Bundle,
    #[serde(default)]
    apply: bool,
    #[serde(default)]
    revisions: BTreeMap<String, String>,
}
pub(super) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/config/export",
        Capability::ConfigExport,
        export,
    );
    r.admin_endpoint(
        Method::POST,
        "/config/import",
        Capability::ConfigImport,
        import,
    );
    nextcloud_backup::register(r);
}
// Sends the portable export to an administrator-owned Nextcloud folder (t64).
#[path = "config_backup_nextcloud.rs"]
mod nextcloud_backup;
fn portable(key: &str) -> Result<(), AdminApiError> {
    let def = def_of(key)?;
    registry::refuse_if_dedicated(def).map_err(refusal)?;
    if key == "admin.allowLastSuperadminLoss" {
        return Err(AdminApiError::invalid(
            "dedicatedConfirmation",
            "Last-superadmin protection must be changed in its dedicated settings flow",
            "/key",
        ));
    }
    Ok(())
}
/// The portable bundle and the dedicated keys it leaves out.
async fn portable_bundle(state: &AppState) -> Result<(Bundle, Vec<String>), AdminApiError> {
    let rows = state.store.list_setting_policies().await?;
    if rows.len() > MAX_ENTRIES {
        return Err(AdminApiError::invalid(
            "exportLimit",
            "Too many stored policies for one portable bundle",
            "/entries",
        ));
    }
    let mut entries = Vec::new();
    let mut excluded = Vec::new();
    for row in rows {
        if portable(&row.key).is_err() {
            excluded.push(row.key);
            continue;
        }
        let value = row
            .global_value
            .as_deref()
            .map(serde_json::from_str::<Value>)
            .transpose()
            .map_err(|_| AdminApiError::internal("Invalid stored setting"))?
            .unwrap_or(Value::Null);
        entries.push(Entry {
            key: row.key,
            global_value: value,
            locked: row.global_locked,
            allow_user_override: row.allow_user_override,
        });
    }
    entries.sort_by(|a, b| a.key.cmp(&b.key));
    Ok((
        Bundle {
            format: FORMAT.into(),
            version: 1,
            entries,
        },
        excluded,
    ))
}
async fn export(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::ConfigExport)?;
    ctx.require_unscoped(Capability::SettingsRead)?;
    let (bundle, excluded) = portable_bundle(&state).await?;
    Ok((
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        Json(json!({
            "bundle": bundle,
            "excluded": excluded,
            "scope": "registered-global-policies",
            "includesSecrets": false,
            "includesDomains": false,
            "includesMail": false
        })),
    )
        .into_response())
}
fn validate_bundle(bundle: &Bundle) -> Result<(), AdminApiError> {
    if bundle.format != FORMAT || bundle.version != 1 || bundle.entries.len() > MAX_ENTRIES {
        return Err(AdminApiError::invalid(
            "invalidBundle",
            "Expected version 1 mailwoman-settings with at most 1024 entries",
            "/bundle",
        ));
    }
    let mut keys = BTreeSet::new();
    for entry in &bundle.entries {
        if !keys.insert(&entry.key) {
            return Err(AdminApiError::invalid(
                "duplicateKey",
                "Duplicate setting in import",
                "/entries",
            ));
        }
        portable(&entry.key)?;
    }
    Ok(())
}
fn body(entry: &Entry, dry_run: bool) -> GlobalPut {
    GlobalPut {
        global_value: Some(entry.global_value.clone()),
        allow_user_override: Some(entry.allow_user_override),
        locked: Some(entry.locked),
        dry_run,
        confirm_last_superadmin_loss: false,
    }
}
async fn import(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(request): AdminJson<Import>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::ConfigImport)?;
    ctx.require_unscoped(Capability::SettingsWrite)?;
    validate_bundle(&request.bundle)?;
    if request.apply {
        crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::ConfigImport)
            .await?;
    }
    let mut revisions = BTreeMap::new();
    let mut changes = Vec::new();
    // Validate the entire bundle through the existing policy writer before any write.
    for entry in &request.bundle.entries {
        let current = state.store.get_setting_policy(&entry.key).await?;
        let before = policy_json(current.as_ref());
        let tag = etag_of(&before);
        if request.apply && request.revisions.get(&entry.key) != Some(&tag) {
            return Err(AdminApiError::stale(None, None));
        }
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::IF_MATCH,
            tag.parse()
                .map_err(|_| AdminApiError::internal("Invalid revision"))?,
        );
        let result = put_global(
            State(state.clone()),
            ctx.clone(),
            headers,
            UrlPath(entry.key.clone()),
            AdminJson(body(entry, true)),
        )
        .await?;
        let bytes = axum::body::to_bytes(result.into_body(), 65536)
            .await
            .map_err(|_| AdminApiError::internal("Invalid impact response"))?;
        let impact: Value = serde_json::from_slice(&bytes)
            .map_err(|_| AdminApiError::internal("Invalid impact JSON"))?;
        changes.push(
            json!({"key": entry.key, "before": before, "after": entry, "impact": impact["impact"]}),
        );
        revisions.insert(entry.key.clone(), tag);
    }
    if !request.apply {
        return Ok((
            [(axum::http::header::CACHE_CONTROL, "private, no-store")],
            Json(json!({"preview":true,"changes":changes,"revisions":revisions,"atomic":false})),
        )
            .into_response());
    }
    let mut applied = Vec::new();
    for entry in &request.bundle.entries {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::IF_MATCH,
            revisions[&entry.key]
                .parse()
                .map_err(|_| AdminApiError::internal("Invalid revision"))?,
        );
        if let Err(error) = put_global(
            State(state.clone()),
            ctx.clone(),
            headers,
            UrlPath(entry.key.clone()),
            AdminJson(body(entry, false)),
        )
        .await
        {
            let status = error.into_response().status().as_u16();
            // The policy writer may commit before response enrichment fails.
            // Do not describe a server-error outcome as a confirmed non-write.
            return Ok(Json(json!({"complete":false,"applied":applied,"outcomeUncertain":status>=500,"reloadRequired":true,"failed":{"key":entry.key,"status":status},"notAttempted":request.bundle.entries.iter().skip(applied.len()+1).map(|e|&e.key).collect::<Vec<_>>()})).into_response());
        }
        applied.push(entry.key.clone());
    }
    audit(
        &state,
        &ctx,
        "configuration-import",
        json!({"keys":applied,"count":applied.len()}),
    )
    .await;
    Ok(Json(json!({"complete":true,"applied":applied})).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_duplicate_unknown_and_protected_imports() {
        let entry = Entry {
            key: "mail.quickReply".into(),
            global_value: json!(false),
            locked: false,
            allow_user_override: true,
        };
        let mut bundle = Bundle {
            format: FORMAT.into(),
            version: 1,
            entries: vec![entry.clone()],
        };
        assert!(validate_bundle(&bundle).is_ok());
        bundle.entries.push(entry);
        assert!(validate_bundle(&bundle).is_err());
        bundle.entries.pop();
        bundle.entries[0].key = "admin.allowLastSuperadminLoss".into();
        assert!(validate_bundle(&bundle).is_err());
        bundle.entries[0].key = "not.a.setting".into();
        assert!(validate_bundle(&bundle).is_err());
    }
}
