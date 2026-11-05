//! Admin > Compliance > Encryption: what is encrypted at rest, how that was
//! determined, and the actions that prove it.
//!
//! Every row carries a state and the evidence behind it. The states are:
//!
//! * `encrypted`: the bytes on disk are ciphertext, checked by reading them.
//! * `sealed-columns-only`: the store is readable as a whole, but the sensitive
//!   columns inside it (credentials, message bodies, uploads) are sealed.
//! * `plaintext`: readable by anyone with access to the file or directory.
//! * `memory-only` / `not-stored`: nothing is written to disk.
//! * `unknown`: the application cannot observe it (for example a Postgres server's
//!   disk).
//!
//! Nothing here claims that a configuration satisfies a law or a standard.

use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    capabilities::Capability,
    engine_mode::{self, SearchIndexPlace},
    settings_registry,
};
use axum::{
    Json,
    extract::State,
    http::Method,
    response::{IntoResponse, Response},
};
use mw_store::encryption::{self, DbKey, FileState};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

static DATABASE: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Remember the database DSN the process opened (set once in `build_app`).
pub(crate) fn remember_database(dsn: &str) {
    let _ = DATABASE.set(dsn.to_owned());
}

fn database() -> String {
    DATABASE.get().cloned().unwrap_or_default()
}

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/encryption", Capability::SettingsRead, status_route);
    r.admin_endpoint(Method::POST, "/encryption/verify", Capability::SettingsWrite, verify_route);
    r.admin_endpoint(Method::POST, "/encryption/rehearse", Capability::SettingsWrite, rehearse_route);
}

fn no_store(v: Value) -> Response {
    ([(axum::http::header::CACHE_CONTROL, "no-store")], Json(v)).into_response()
}

fn row(id: &str, state: &str, evidence: &str, args: Value) -> Value {
    json!({"id": id, "state": state, "evidence": evidence, "args": args})
}

fn is_postgres(db: &str) -> bool {
    db.starts_with("postgres://") || db.starts_with("postgresql://")
}

/// Count, and classify by first bytes, regular files under `dir` (bounded walk).
fn sample_dir(dir: &Path, limit: usize) -> (usize, usize, usize) {
    // (files seen, files that look like a known plaintext format, bytes of ciphertext shape ok)
    let mut stack = vec![dir.to_path_buf()];
    let (mut seen, mut plain, mut ok) = (0usize, 0usize, 0usize);
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if seen >= limit {
                return (seen, plain, ok);
            }
            seen += 1;
            let mut head = [0u8; 8];
            if let Ok(mut f) = std::fs::File::open(&p) {
                use std::io::Read;
                let n = f.read(&mut head).unwrap_or(0);
                let h = &head[..n];
                let looks_plain = h.starts_with(b"%PDF")
                    || h.starts_with(b"\x89PNG")
                    || h.starts_with(b"\xff\xd8\xff")
                    || h.starts_with(b"PK\x03\x04")
                    || h.starts_with(b"GIF8")
                    || h.starts_with(b"<")
                    || h.starts_with(b"{");
                if looks_plain {
                    plain += 1;
                } else if e.metadata().map(|m| m.len() >= 40).unwrap_or(false) {
                    ok += 1;
                }
            }
        }
    }
    (seen, plain, ok)
}

fn db_file(_state: &AppState) -> Option<PathBuf> {
    encryption::sqlite_file_from_dsn(&database())
}

fn parent_of(p: &Path) -> PathBuf {
    p.parent()
        .filter(|x| !x.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Plaintext rollback copies left beside the database by `mailwoman db encrypt`.
fn leftover_copies(db: &Path) -> Vec<Value> {
    let Some(name) = db.file_name().and_then(|n| n.to_str()) else {
        return vec![];
    };
    let prefix = format!("{name}.pre-");
    let mut out = vec![];
    if let Ok(rd) = std::fs::read_dir(parent_of(db)) {
        for e in rd.flatten() {
            let fname = e.file_name().to_string_lossy().into_owned();
            if fname.starts_with(&prefix) {
                let state = match encryption::probe_file(&e.path()) {
                    Ok(FileState::Plaintext) => "plaintext",
                    Ok(FileState::NotPlaintext) => "encrypted",
                    _ => "unknown",
                };
                out.push(json!({"name": fname, "state": state}));
            }
        }
    }
    out
}

async fn pg_report(state: &AppState) -> Value {
    match state.store.postgres_encryption_report().await {
        Ok(v) => v,
        Err(e) => json!({"error": e.to_string()}),
    }
}

pub(crate) async fn snapshot(state: &AppState) -> Value {
    let db = database();
    let cfg = encryption::global().ok();
    let mode = cfg.as_ref().map(|c| c.mode.as_str()).unwrap_or("off");
    let key_source = cfg.as_ref().and_then(|c| c.source.clone());
    let fingerprint = cfg.as_ref().and_then(|c| c.key.as_ref()).map(DbKey::fingerprint);
    let file = db_file(state);
    let meta = file.as_deref().and_then(encryption::read_meta);
    let rotation_days = settings_registry::effective_for(&state.store, "encryption.rotationDays", &Default::default())
        .await
        .ok()
        .flatten()
        .and_then(|v| v.value.as_i64())
        .unwrap_or(365);
    let last_rotated = meta
        .as_ref()
        .and_then(|m| m.last_rotated_at.clone())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));
    let due_at = last_rotated.filter(|_| rotation_days > 0).map(|d| d + chrono::Duration::days(rotation_days));
    let overdue = due_at.is_some_and(|d| d < chrono::Utc::now());

    let mut stores: Vec<Value> = vec![];
    let mut pg = Value::Null;

    // 1. The database.
    if is_postgres(&db) {
        pg = pg_report(state).await;
        stores.push(row(
            "database",
            "sealed-columns-only",
            "evidence-db-postgres",
            json!({"ssl": pg["connectionSsl"], "tde": pg["tdeIndicators"]}),
        ));
    } else if let Some(f) = &file {
        match encryption::probe_file(f) {
            Ok(FileState::NotPlaintext) => stores.push(row(
                "database",
                "encrypted",
                "evidence-db-header",
                json!({"cipher": "SQLCipher 4, AES-256-CBC, HMAC-SHA512 per page", "includesWal": true}),
            )),
            Ok(FileState::Plaintext) => stores.push(row("database", "sealed-columns-only", "evidence-db-plain-header", json!({}))),
            Ok(_) => stores.push(row("database", "unknown", "evidence-db-empty", json!({}))),
            Err(e) => stores.push(row("database", "unknown", "evidence-error", json!({"error": e.to_string()}))),
        }
    } else {
        stores.push(row("database", "memory-only", "evidence-db-memory", json!({})));
    }

    // 2. Search index.
    let search_env = std::env::var("MW_SEARCH_DIR").ok();
    match engine_mode::search_index_place(&db, search_env.as_deref()) {
        SearchIndexPlace::Disk(dir) => {
            let (seen, _, _) = sample_dir(&dir, 200);
            stores.push(row(
                "searchIndex",
                if seen == 0 { "unknown" } else { "plaintext" },
                "evidence-search-disk",
                json!({"files": seen}),
            ));
        }
        SearchIndexPlace::Memory(_) => stores.push(row("searchIndex", "memory-only", "evidence-search-memory", json!({}))),
    }

    // 3. Uploaded blobs.
    match std::env::var_os("MW_UPLOAD_DIR").filter(|s| !s.is_empty()).map(PathBuf::from) {
        Some(dir) => {
            let (seen, plain, ok) = sample_dir(&dir, 200);
            let st = if plain > 0 {
                "plaintext"
            } else if seen == 0 {
                "sealed-columns-only"
            } else {
                "encrypted"
            };
            stores.push(row("uploads", st, "evidence-uploads", json!({"sampled": seen, "plainLooking": plain, "ciphertextShaped": ok})));
        }
        None => stores.push(row("uploads", "not-stored", "evidence-uploads-off", json!({}))),
    }

    // 4. Backups: only what the server can see.
    let leftovers = file.as_deref().map(leftover_copies).unwrap_or_default();
    let plain_left = leftovers.iter().filter(|l| l["state"] == "plaintext").count();
    stores.push(row(
        "backups",
        if plain_left > 0 { "plaintext" } else { "unknown" },
        if plain_left > 0 { "evidence-backups-leftover" } else { "evidence-backups-unseen" },
        json!({"leftovers": leftovers}),
    ));

    // 5. Logs.
    let logs = crate::observability::log_files::status();
    if logs["fileEnabled"].as_bool().unwrap_or(false) {
        stores.push(row("logs", "plaintext", "evidence-logs-file", json!({"directory": logs["directory"], "redaction": true})));
    } else {
        stores.push(row("logs", "not-stored", "evidence-logs-stdout", json!({})));
    }

    // 6. Exports and archive.
    let export_sealed = settings_registry::effective_for(&state.store, "dsr.export.encrypt", &Default::default())
        .await
        .ok()
        .flatten()
        .and_then(|v| v.value.as_bool())
        .unwrap_or(false);
    let db_encrypted = stores[0]["state"] == "encrypted";
    stores.push(row(
        "exports",
        if db_encrypted {
            "encrypted"
        } else if export_sealed {
            "sealed-columns-only"
        } else {
            "plaintext"
        },
        "evidence-exports",
        json!({"dsrExportSealed": export_sealed, "storedIn": "database"}),
    ));
    stores.push(row("archive", "plaintext", "evidence-archive", json!({"integrity": "hmac-sha256", "confidentiality": false})));

    // 7. Secrets.
    let key_file = std::env::var_os("MW_SERVER_KEY_FILE").map(PathBuf::from).or_else(|| {
        file.as_ref().map(|f| {
            let mut s = f.clone().into_os_string();
            s.push(".server-key");
            PathBuf::from(s)
        })
    });
    let server_key_in_env = std::env::var_os("MW_SERVER_KEY").is_some();
    let key_beside_db = match (&key_file, &file, &key_source) {
        (_, Some(f), Some(k)) if k.kind == "file" => Path::new(&k.detail).parent() == f.parent(),
        _ => false,
    };
    stores.push(row(
        "secrets",
        "plaintext",
        "evidence-secrets",
        json!({"serverKey": if server_key_in_env { "environment" } else if key_file.is_some() { "file" } else { "generated-per-process" },
               "dbKeyBesideDatabase": key_beside_db}),
    ));

    // Warnings feed the security posture page.
    let mut warnings: Vec<&str> = vec![];
    let db_state = stores[0]["state"].as_str().unwrap_or("unknown").to_string();
    if db_state != "encrypted" && !is_postgres(&db) && file.is_some() {
        warnings.push("databasePlaintext");
    }
    if mode == "optional" && db_state == "encrypted" {
        // Optional would silently start on a replaced plaintext file.
        warnings.push("modeNotRequired");
    }
    if stores.iter().any(|s| s["id"] == "searchIndex" && s["state"] == "plaintext") {
        warnings.push("searchIndexPlaintext");
    }
    if plain_left > 0 {
        warnings.push("plaintextRollbackCopy");
    }
    if overdue {
        warnings.push("rotationOverdue");
    }
    if key_beside_db {
        warnings.push("keyBesideDatabase");
    }
    if is_postgres(&db) && pg["connectionSsl"] == json!(false) {
        warnings.push("postgresNoTls");
    }

    json!({
        "mode": mode,
        "backend": if is_postgres(&db) { "postgres" } else if file.is_some() { "sqlite" } else { "memory" },
        "databaseFile": file.as_ref().and_then(|f| f.file_name()).map(|n| n.to_string_lossy().into_owned()),
        "key": {
            "configured": cfg.as_ref().is_some_and(|c| c.key.is_some()),
            "source": key_source,
            "fingerprint": fingerprint,
        },
        "rotation": {
            "days": rotation_days,
            "lastRotatedAt": last_rotated.map(|d| d.to_rfc3339()),
            "encryptedAt": meta.as_ref().and_then(|m| m.encrypted_at.clone()),
            "dueAt": due_at.map(|d| d.to_rfc3339()),
            "overdue": overdue,
        },
        "stores": stores,
        "postgres": pg,
        "columnCoverage": mw_store::encryption_report::SEALED_COLUMN_COVERAGE,
        "warnings": warnings,
        "keys": ["encryption.rotationDays", "encryption.warnings.enabled"],
        "commands": {
            "dryRun": "mailwoman db encrypt --dry-run",
            "encrypt": "mailwoman db encrypt",
            "rekey": "mailwoman db rekey",
            "decrypt": "mailwoman db decrypt",
            "verify": "mailwoman db verify",
            "backup": "mailwoman db backup --to FILE",
            "keygen": "mailwoman db keygen",
        },
    })
}

async fn status_route(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsRead)?;
    Ok(no_store(snapshot(&state).await))
}

async fn verify_route(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    let Some(file) = db_file(&state) else {
        return Ok(no_store(json!({
            "passed": false,
            "checks": [{"name": "sqlite-file", "passed": false, "detail": "the database is not a SQLite file; see the Postgres report"}],
        })));
    };
    let cfg = encryption::global().map_err(AdminApiError::internal)?;
    let report = encryption::verify(&file, cfg.key.as_ref()).await;
    let at = chrono::Utc::now().to_rfc3339();
    let _ = state
        .store
        .set_setting("encryption.lastVerify", &json!({"at": at, "passed": report.passed}).to_string())
        .await;
    crate::admin_accounts::audit(&state, &ctx, "encryption-verified", "database", None, Some(json!({"passed": report.passed}))).await;
    Ok(no_store(serde_json::to_value(&report).unwrap_or(Value::Null)))
}

/// Dry run of a migration against a consistent copy of the live data: the file is
/// exported to a scratch file next to the database, compared table by table, and
/// removed. The live database is only read.
async fn rehearse_route(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    let Some(file) = db_file(&state) else {
        return Err(AdminApiError::conflict("notSqlite", "only a SQLite file can be rehearsed"));
    };
    let cfg = encryption::global().map_err(AdminApiError::internal)?;
    let started = std::time::Instant::now();
    let (from, to) = match cfg.key.as_ref() {
        Some(k) => (Some(k.clone()), Some(DbKey::generate())),
        None => (None, Some(DbKey::generate())),
    };
    let scratch = {
        let mut s = file.clone().into_os_string();
        s.push(".rehearse");
        PathBuf::from(s)
    };
    let _ = std::fs::remove_file(&scratch);
    let result = encryption::backup(&file, from.as_ref(), &scratch, to.as_ref(), false).await;
    let size = std::fs::metadata(&scratch).map(|m| m.len()).ok();
    let _ = std::fs::remove_file(&scratch);
    let _ = std::fs::remove_file(format!("{}-wal", scratch.display()));
    let _ = std::fs::remove_file(format!("{}-shm", scratch.display()));
    crate::admin_accounts::audit(&state, &ctx, "encryption-rehearsed", "database", None, Some(json!({"ok": result.is_ok()}))).await;
    match result {
        Ok(tables) => Ok(no_store(json!({
            "ok": true,
            "operation": if from.is_some() { "rekey" } else { "encrypt" },
            "tables": tables.len(),
            "rows": tables.iter().map(|(_, n)| *n).sum::<i64>(),
            "bytes": size,
            "millis": started.elapsed().as_millis() as u64,
        }))),
        Err(e) => Ok(no_store(json!({"ok": false, "error": e.to_string()}))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "mw-encstatus-{tag}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn upload_sampling_flags_files_that_look_like_known_plain_formats() {
        let d = scratch("up");
        std::fs::write(d.join("sealed"), vec![0xA7u8; 80]).unwrap();
        std::fs::write(d.join("plain.pdf"), b"%PDF-1.7 hello hello hello hello hello hello").unwrap();
        let (seen, plain, ok) = sample_dir(&d, 100);
        assert_eq!((seen, plain, ok), (2, 1, 1));
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn leftover_rollback_copies_are_listed_with_their_state() {
        let d = scratch("left");
        let db = d.join("mw.db");
        std::fs::write(&db, vec![0x11u8; 64]).unwrap();
        let mut plain = b"SQLite format 3\0".to_vec();
        plain.extend_from_slice(&[0u8; 64]);
        std::fs::write(d.join("mw.db.pre-encrypt-20260101T000000Z"), plain).unwrap();
        std::fs::write(d.join("mw.db.pre-rekey-20260102T000000Z"), vec![0x22u8; 64]).unwrap();
        let mut found = leftover_copies(&db);
        found.sort_by_key(|v| v["name"].as_str().unwrap_or_default().to_owned());
        assert_eq!(found.len(), 2);
        assert_eq!(found[0]["state"], "plaintext");
        assert_eq!(found[1]["state"], "encrypted");
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn row_carries_state_and_evidence() {
        let r = row("database", "encrypted", "evidence-db-header", json!({"a": 1}));
        assert_eq!(r["state"], "encrypted");
        assert_eq!(r["evidence"], "evidence-db-header");
    }
}
