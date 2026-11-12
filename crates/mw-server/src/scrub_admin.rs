//! Per-person scrubbing and backup erasure markings: settings, the scrub of
//! logs and audit history, and the admin routes.
//!
//! * `GET  /privacy/scrub`            settings, key generations, audit chain check,
//!   chain anchors, tombstone watermark and registered backups.
//! * `POST /privacy/scrub/preview`    `{address}`: counts of what a scrub would change.
//! * `POST /privacy/scrub/run`        `{address}`: replace the person's identifiers in
//!   audit and log history with a keyed pseudonym (see `mw_store::scrub`).
//! * `POST /privacy/scrub/rotate-key` add a pseudonym key generation.
//! * `POST /privacy/scrub/seal-audit` give unsealed audit rows their chain position.
//! * `POST /privacy/backups`          register a backup (label, location, watermark).
//! * `DELETE /privacy/backups/{id}`   forget a registered backup.
//!
//! Rotated log files are rewritten with the identifiers replaced; `.gz` files are
//! read and written back compressed. Every run is an audit entry that names the
//! pseudonym, never the address.

use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson},
    admin_router::AdminRegistrar,
    capabilities::Capability,
    dsr_data::Subj,
    settings_registry,
};
use axum::{
    Json,
    extract::{Path as AxPath, State},
    http::Method,
    response::{IntoResponse, Response},
};
use mw_store::{
    scrub::{ScrubOptions, ScrubReport},
    settings_policy::SettingSubject,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::Path,
};

/// The effective `scrub.*` and `backup.*` values.
#[derive(Clone, Debug)]
pub(crate) struct ScrubConfig {
    pub enabled: bool,
    pub algorithm: String,
    pub include_logs: bool,
    pub auto_on_erasure: bool,
    pub key_retention_days: i64,
    pub propagate_erasure: bool,
    pub tombstone_retention_days: i64,
}

impl ScrubConfig {
    pub(crate) async fn load(state: &AppState) -> Self {
        let subject = SettingSubject::default();
        let rows = settings_registry::effective_all(&state.store, &subject, |d| {
            d.key.starts_with("scrub.") || d.key.starts_with("backup.")
        })
        .await
        .unwrap_or_default();
        let map: BTreeMap<String, Value> = rows.into_iter().map(|e| (e.key, e.value)).collect();
        let get = |k: &str| {
            map.get(k)
                .cloned()
                .or_else(|| settings_registry::lookup(k).map(|d| d.default_value()))
                .unwrap_or(Value::Null)
        };
        Self {
            enabled: get("scrub.enabled").as_bool().unwrap_or(true),
            algorithm: get("scrub.pseudonymAlgorithm")
                .as_str()
                .unwrap_or(mw_store::scrub::DEFAULT_ALGORITHM)
                .to_string(),
            include_logs: get("scrub.includeLogs").as_bool().unwrap_or(true),
            auto_on_erasure: get("scrub.autoRunOnErasure").as_bool().unwrap_or(false),
            key_retention_days: get("scrub.keyRetentionDays").as_i64().unwrap_or(0),
            propagate_erasure: get("backup.propagateErasure").as_bool().unwrap_or(true),
            tombstone_retention_days: get("backup.tombstoneRetentionDays").as_i64().unwrap_or(3650),
        }
    }

    fn json(&self) -> Value {
        json!({
            "enabled": self.enabled, "pseudonymAlgorithm": self.algorithm,
            "includeLogs": self.include_logs, "autoRunOnErasure": self.auto_on_erasure,
            "keyRetentionDays": self.key_retention_days,
            "propagateErasure": self.propagate_erasure,
            "tombstoneRetentionDays": self.tombstone_retention_days,
        })
    }
}

/// Remove erasure tombstones past `backup.tombstoneRetentionDays` and pseudonym
/// keys past their destroy time. Run by the retention sweeper.
pub(crate) async fn housekeeping(state: &AppState) {
    let cfg = ScrubConfig::load(state).await;
    let now = chrono::Utc::now().timestamp();
    if let Err(e) = state
        .store
        .erasure_tombstone_purge(now, cfg.tombstone_retention_days)
        .await
    {
        tracing::warn!("erasure tombstone retention failed: {e}");
    }
    if let Err(e) = state.store.scrub_destroy_expired_keys(now).await {
        tracing::warn!("pseudonym key retention failed: {e}");
    }
}

/// What a scrub of one person changed (or would change).
pub(crate) struct SubjectScrub {
    pub counts: BTreeMap<String, u64>,
    pub errors: Vec<String>,
    pub report: ScrubReport,
}

/// Scrub one person from audit history and, with `scrub.includeLogs`, from the
/// rotated log files.
pub(crate) async fn scrub_subject(
    state: &AppState,
    s: &Subj,
    cfg: &ScrubConfig,
    dry_run: bool,
    request_id: Option<&str>,
) -> Result<SubjectScrub, String> {
    let people = crate::dsr_data::person_ids(state, s)
        .await
        .map_err(|e| e.to_string())?;
    let mut hashed = vec![crate::push_relay::hash_token(&s.address)];
    for subject in state
        .store
        .dsr_binding_subjects(&people.principals)
        .await
        .map_err(|e| e.to_string())?
    {
        hashed.push(crate::push_relay::hash_token(&subject));
    }
    let opts = ScrubOptions {
        dry_run,
        algorithm: cfg.algorithm.clone(),
        // An erasure already wrote its own tombstone.
        tombstone: !dry_run && request_id.is_none(),
    };
    let now = chrono::Utc::now().timestamp();
    let report = state
        .store
        .scrub_person(&people, &[], &hashed, &opts, request_id, now)
        .await
        .map_err(|e| e.to_string())?;
    let mut counts = report.counts.clone();
    let mut errors = report.errors.clone();
    if cfg.include_logs {
        let dir = crate::observability::log_files::policy().directory;
        let mut needles: Vec<String> = vec![s.address.clone()];
        needles.extend(s.account_ids.iter().cloned());
        needles.retain(|n| n.len() >= 4);
        let pairs: Vec<(String, String)> = needles
            .into_iter()
            .map(|n| (n, report.pseudonym.clone()))
            .collect();
        if !dry_run {
            crate::observability::log_files::release_current();
        }
        let r = scrub_log_dir(&dir, &pairs, dry_run);
        if r.lines > 0 {
            counts.insert("log_lines".into(), r.lines);
        }
        if r.files > 0 {
            counts.insert("log_files".into(), r.files);
        }
        errors.extend(r.errors);
    }
    Ok(SubjectScrub { counts, errors, report })
}

/// What a log scrub changed.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct LogScrub {
    pub files: u64,
    pub lines: u64,
    pub errors: Vec<String>,
}

fn is_log_name(name: &str) -> bool {
    name.starts_with("mailwoman") && (name.ends_with(".log") || name.ends_with(".log.gz"))
}

/// Replace the needles in every `mailwoman*.log` and `mailwoman*.log.gz` of
/// `dir`. A file is rewritten through a temporary file and renamed over the
/// original, so a failure leaves the original in place.
pub(crate) fn scrub_log_dir(dir: &Path, needles: &[(String, String)], dry_run: bool) -> LogScrub {
    let mut out = LogScrub::default();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    let mut names: Vec<String> = entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| is_log_name(n))
        .collect();
    names.sort();
    for name in names {
        let path = dir.join(&name);
        match scrub_log_file(&path, needles, dry_run) {
            Ok(0) => {}
            Ok(n) => {
                out.files += 1;
                out.lines += n;
            }
            Err(e) => out.errors.push(format!("{name}: {e}")),
        }
    }
    out
}

/// Scrub one file; the number of lines changed.
pub(crate) fn scrub_log_file(
    path: &Path,
    needles: &[(String, String)],
    dry_run: bool,
) -> std::io::Result<u64> {
    let gz = path.extension().is_some_and(|e| e == "gz");
    let raw = std::fs::read(path)?;
    let plain = if gz {
        let mut buf = Vec::new();
        flate2::read::MultiGzDecoder::new(&raw[..]).read_to_end(&mut buf)?;
        buf
    } else {
        raw
    };
    let plain=crate::observability::log_files::decode_log(&plain)?;
    let text = String::from_utf8_lossy(&plain);
    let mut changed = 0u64;
    let mut out = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        let new = mw_store::scrub::replace_needles(line, needles);
        if new != line {
            changed += 1;
        }
        out.push_str(&new);
    }
    if changed == 0 || dry_run {
        return Ok(changed);
    }
    let tmp = path.with_extension("scrub-tmp");
    {
        let mut f = std::fs::File::create(&tmp)?;
        let sealed=crate::observability::log_files::encode_log(out.as_bytes())?;
        if gz {
            let mut enc = flate2::write::GzEncoder::new(&mut f, flate2::Compression::default());
            enc.write_all(&sealed)?;
            enc.finish()?;
        } else {
            f.write_all(&sealed)?;
        }
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(changed)
}

// ─── routes ───────────────────────────────────────────────────────────────────

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/privacy/scrub", Capability::PrivacyRead, get_scrub);
    r.admin_endpoint(Method::POST, "/privacy/scrub/preview", Capability::PrivacyWrite, preview);
    r.admin_endpoint(Method::POST, "/privacy/scrub/run", Capability::PrivacyWrite, run);
    r.admin_endpoint(Method::POST, "/privacy/scrub/rotate-key", Capability::PrivacyWrite, rotate_key);
    r.admin_endpoint(Method::POST, "/privacy/scrub/seal-audit", Capability::PrivacyWrite, seal_audit);
    r.admin_endpoint(Method::POST, "/privacy/backups", Capability::PrivacyWrite, register_backup);
    r.admin_endpoint(Method::DELETE, "/privacy/backups/{id}", Capability::PrivacyWrite, delete_backup);
}

fn no_store(v: Value) -> Response {
    ([(axum::http::header::CACHE_CONTROL, "no-store")], Json(v)).into_response()
}

async fn get_scrub(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::PrivacyRead)?;
    let cfg = ScrubConfig::load(&state).await;
    let chain = state.store.audit_chain_verify().await?;
    Ok(no_store(json!({
        "settings": cfg.json(),
        "keys": state.store.scrub_key_infos().await?,
        "auditChain": chain,
        "anchors": state.store.audit_rechains(20).await?,
        "tombstoneWatermark": state.store.erasure_watermark().await?,
        "backups": state.store.backup_register_list().await?,
        "algorithms": mw_store::scrub::ALGORITHMS,
    })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Who {
    address: String,
}

async fn subject_of(state: &AppState, address: &str) -> Result<Subj, AdminApiError> {
    let address = address.trim().to_lowercase();
    if address.is_empty() || !address.contains('@') {
        return Err(AdminApiError::invalid("invalidAddress", "enter the person's address", "/address"));
    }
    let account_ids = crate::dsr::account_ids_of(state, &address, "").await;
    let owners = crate::dsr::owners_of(state, &account_ids, &[]).await;
    Ok(Subj { address, account_ids, owners })
}

fn result_json(r: &SubjectScrub, dry_run: bool) -> Value {
    json!({
        "dryRun": dry_run, "counts": r.counts, "errors": r.errors,
        "pseudonym": r.report.pseudonym, "keyGen": r.report.key_gen,
        "anchor": r.report.rechain_id, "tombstone": r.report.tombstone_seq,
    })
}

async fn preview(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(input): AdminJson<Who>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::PrivacyWrite)?;
    let cfg = ScrubConfig::load(&state).await;
    if !cfg.enabled {
        return Err(AdminApiError::conflict("scrubDisabled", "scrubbing is switched off in the scrub settings"));
    }
    let s = subject_of(&state, &input.address).await?;
    let r = scrub_subject(&state, &s, &cfg, true, None)
        .await
        .map_err(AdminApiError::internal)?;
    Ok(no_store(result_json(&r, true)))
}

async fn run(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(input): AdminJson<Who>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::PrivacyWrite)?;
    let cfg = ScrubConfig::load(&state).await;
    if !cfg.enabled {
        return Err(AdminApiError::conflict("scrubDisabled", "scrubbing is switched off in the scrub settings"));
    }
    let s = subject_of(&state, &input.address).await?;
    let r = scrub_subject(&state, &s, &cfg, false, None)
        .await
        .map_err(AdminApiError::internal)?;
    audit(&state, &ctx.username, "scrub-person", &r.report.pseudonym, json!({"counts": r.counts, "errors": r.errors.len(), "anchor": r.report.rechain_id})).await;
    Ok(no_store(result_json(&r, false)))
}

async fn rotate_key(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::PrivacyWrite)?;
    let cfg = ScrubConfig::load(&state).await;
    let now = chrono::Utc::now().timestamp();
    let generation = state
        .store
        .scrub_rotate(now, Some(cfg.key_retention_days))
        .await?;
    audit(&state, &ctx.username, "scrub-key-rotated", &format!("gen-{generation}"), json!({"generation": generation})).await;
    Ok(no_store(json!({"generation": generation, "keys": state.store.scrub_key_infos().await?})))
}

async fn seal_audit(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::PrivacyWrite)?;
    let sealed = state.store.audit_chain_seal().await?;
    Ok(no_store(json!({"sealed": sealed, "auditChain": state.store.audit_chain_verify().await?})))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BackupInput {
    label: String,
    location: String,
    #[serde(default)]
    kind: Option<String>,
    /// Tombstone watermark at the time of the backup; the current one when absent.
    #[serde(default)]
    tombstone_watermark: Option<i64>,
    #[serde(default)]
    created_at: Option<i64>,
}

async fn register_backup(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(input): AdminJson<BackupInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::PrivacyWrite)?;
    let label: String = input.label.trim().chars().take(120).collect();
    let location: String = input.location.trim().chars().take(500).collect();
    if label.is_empty() || location.is_empty() {
        return Err(AdminApiError::invalid("invalidBackup", "a label and a location are required", "/label"));
    }
    let now = chrono::Utc::now().timestamp();
    let watermark = match input.tombstone_watermark {
        Some(w) if w >= 0 => w,
        _ => state.store.erasure_watermark().await?,
    };
    let kind = input.kind.unwrap_or_else(|| "external".into());
    let manifest = json!({"format": mw_store::erasure::BACKUP_FORMAT, "label": label, "createdAt": input.created_at.unwrap_or(now), "tombstoneWatermark": watermark});
    let id = state
        .store
        .backup_register_add(&kind, &label, &location, input.created_at.unwrap_or(now), watermark, &manifest)
        .await?;
    audit(&state, &ctx.username, "backup-registered", &id, json!({"label": label, "watermark": watermark})).await;
    Ok(no_store(json!({"id": id, "backups": state.store.backup_register_list().await?})))
}

async fn delete_backup(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AxPath(id): AxPath<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::PrivacyWrite)?;
    if !state.store.backup_register_delete(&id).await? {
        return Err(AdminApiError::not_found("unknown backup"));
    }
    audit(&state, &ctx.username, "backup-unregistered", &id, json!({})).await;
    Ok(no_store(json!({"backups": state.store.backup_register_list().await?})))
}

async fn audit(state: &AppState, actor: &str, action: &str, target: &str, detail: Value) {
    let entry = mw_admin::AuditLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor: actor.to_string(),
        actor_kind: mw_admin::ActorKind::Admin,
        action: action.into(),
        target: Some(format!("privacy:{target}")),
        detail_json: mw_admin::redact_detail(&detail),
        ip: None,
    };
    if let Err(e) = state.v6.admin.audit(entry).await {
        tracing::warn!("scrub audit entry not recorded: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn needles() -> Vec<(String, String)> {
        vec![("ana@example.org".into(), "scrubbed-1-abc".into())]
    }

    #[test]
    fn plain_and_gzip_logs_are_rewritten_and_others_left_alone() {
        let dir = std::env::temp_dir().join(format!("mw-scrub-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("mailwoman.log"), "sign-in ana@example.org ok\nother line\n").unwrap();
        let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        enc.write_all(b"x ANA@example.org y\nz\n").unwrap();
        std::fs::write(dir.join("mailwoman-100-0.log.gz"), enc.finish().unwrap()).unwrap();
        std::fs::write(dir.join("notes.txt"), "ana@example.org").unwrap();

        let dry = scrub_log_dir(&dir, &needles(), true);
        assert_eq!((dry.files, dry.lines), (2, 2));
        assert!(std::fs::read_to_string(dir.join("mailwoman.log")).unwrap().contains("ana@example.org"));

        let done = scrub_log_dir(&dir, &needles(), false);
        assert_eq!((done.files, done.lines), (2, 2));
        assert!(done.errors.is_empty());
        let plain = std::fs::read_to_string(dir.join("mailwoman.log")).unwrap();
        assert_eq!(plain, "sign-in scrubbed-1-abc ok\nother line\n");
        let mut text = String::new();
        flate2::read::GzDecoder::new(&std::fs::read(dir.join("mailwoman-100-0.log.gz")).unwrap()[..])
            .read_to_string(&mut text)
            .unwrap();
        assert_eq!(text, "x scrubbed-1-abc y\nz\n");
        assert!(std::fs::read_to_string(dir.join("notes.txt")).unwrap().contains("ana@example.org"));
        assert_eq!(scrub_log_dir(&dir, &needles(), false), LogScrub::default());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
