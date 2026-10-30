//! The local country database, managed from the console (t96).
//!
//! Country policies (`security.signin.geo*`) and the Received-hop and session
//! country labels read one local MaxMind DB file. Before t96 the only way to give
//! the server one was `MW_GEOIP_DB` plus a restart. This module adds a managed
//! copy in a data directory that an administrator can download from an open
//! provider (DB-IP Lite by default), update on a schedule, upload, or roll back,
//! all without a restart:
//!
//! * **files** — `<dir>/active.mmdb` and `<dir>/previous.mmdb`, each with a JSON
//!   sidecar describing where it came from. A new file is written to a temporary
//!   name, validated, then renamed over `active.mmdb` (an atomic replace); the
//!   file it replaces is first copied to `previous.mmdb` for rollback;
//! * **memory** — the validated reader is installed in
//!   [`mw_engine::security::geoip`], whose swap keeps in-flight lookups on the
//!   database they started with;
//! * **precedence** — `MW_GEOIP_DB`, when set, owns the database. The page turns
//!   read-only ("managed by the environment") and nothing here writes;
//! * **fail closed** — no change here relaxes the policy: with an active country
//!   policy, a missing or invalid database still refuses sign-in
//!   ([`crate::signin_geography`]).
//!
//! The directory is `MW_GEOIP_DIR`, else `geoip/` beside a SQLite database file.
//! A Postgres or in-memory database names no directory; downloads then need
//! `MW_GEOIP_DIR`.

pub(crate) mod download;
pub(crate) mod provider;
mod routes;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};

use mw_engine::security::geoip::{self, CountryDatabase, DatabaseInfo, InvalidDatabase};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::AppState;
pub(crate) use routes::register;

/// The data directory variable.
pub(crate) const ENV_DIR: &str = "MW_GEOIP_DIR";
/// A database file to install when the data directory has none (deploy.sh puts
/// the DB-IP file it fetched during `init` here).
pub(crate) const ENV_SEED: &str = "MW_GEOIP_SEED";
/// The `settings` row holding the update schedule.
const SCHEDULE_KEY: &str = "admin.geoip.schedule.v1";
/// A database built longer ago than this is reported as stale.
pub(crate) const STALE_AFTER_SECS: i64 = 45 * 86_400;
/// How often the background loop looks at the schedule and the files (ms).
static LOOP_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(60_000);

/// Test seam: change how often every background loop in this process wakes.
#[doc(hidden)]
pub fn set_loop_interval(d: std::time::Duration) {
    LOOP_MS.store(d.as_millis().clamp(10, 3_600_000) as u64, Ordering::Relaxed);
}

/// Where a file came from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Sidecar {
    /// `dbip`, `maxmind`, `ipinfo`, `custom`, `upload` or `seed`.
    pub source: String,
    /// The provider's release name, when it has one (DB-IP `YYYY-MM`).
    #[serde(default)]
    pub release: Option<String>,
    /// The download URL without its query (never the key).
    #[serde(default)]
    pub label: Option<String>,
    pub installed_at: i64,
    pub installed_by: String,
    pub sha256: String,
    #[serde(default)]
    pub archive_sha256: Option<String>,
    #[serde(default)]
    pub checksum_verified: bool,
    pub database_type: String,
    pub build_epoch: u64,
    pub size_bytes: u64,
    pub networks: u64,
}

/// The update schedule and the outcome of the last attempt.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Schedule {
    pub last_check_at: Option<i64>,
    pub last_success_at: Option<i64>,
    pub next_check_at: Option<i64>,
    pub last_error: Option<String>,
    pub failures: u32,
    /// A run holds this until it finishes, so two instances do not both download.
    pub claimed_until: Option<i64>,
}

/// Process state: the directory and the outcome of loading `active.mmdb`.
struct Runtime {
    dir: RwLock<Option<PathBuf>>,
    /// Why `active.mmdb` did not load, when it did not.
    load_error: RwLock<Option<String>>,
    /// `(len, mtime)` of `active.mmdb` when it was loaded, to notice a file
    /// replaced by another instance sharing the directory.
    loaded_stamp: RwLock<Option<(u64, i64)>>,
    /// One write operation (download, upload, rollback) at a time.
    op: tokio::sync::Mutex<()>,
    busy: std::sync::atomic::AtomicBool,
}

fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| Runtime {
        dir: RwLock::new(None),
        load_error: RwLock::new(None),
        loaded_stamp: RwLock::new(None),
        op: tokio::sync::Mutex::new(()),
        busy: std::sync::atomic::AtomicBool::new(false),
    })
}

/// The data directory for a database DSN: `MW_GEOIP_DIR`, else `geoip/` beside
/// a SQLite file. `None` for Postgres and in-memory databases.
pub(crate) fn data_dir(db_path: &str) -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os(ENV_DIR).filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(dir));
    }
    if db_path.starts_with("postgres://") || db_path.starts_with("postgresql://") {
        return None;
    }
    let file = db_path
        .strip_prefix("sqlite://")
        .or_else(|| db_path.strip_prefix("sqlite:"))
        .unwrap_or(db_path);
    let file = file.split('?').next().unwrap_or(file);
    if file.is_empty() || file.contains(":memory:") {
        return None;
    }
    let parent = Path::new(file)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    Some(parent.join("geoip"))
}

pub(crate) fn dir() -> Option<PathBuf> {
    runtime()
        .dir
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

fn active_path(dir: &Path) -> PathBuf {
    dir.join("active.mmdb")
}
fn previous_path(dir: &Path) -> PathBuf {
    dir.join("previous.mmdb")
}
fn sidecar_of(file: &Path) -> PathBuf {
    file.with_extension("json")
}

fn read_sidecar(file: &Path) -> Option<Sidecar> {
    let raw = std::fs::read(sidecar_of(file)).ok()?;
    serde_json::from_slice(&raw).ok()
}

fn stamp(file: &Path) -> Option<(u64, i64)> {
    let meta = std::fs::metadata(file).ok()?;
    let mtime = meta
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_nanos() as i64;
    Some((meta.len(), mtime))
}

/// Write `bytes` to `path` through a temporary file in the same directory and a
/// rename, so `path` is always either the old or the new complete file.
fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let tmp = path.with_extension(format!("tmp-{}", uuid::Uuid::new_v4().simple()));
    let result = (|| {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Why an operation could not run.
#[derive(Debug)]
pub(crate) enum OpError {
    /// `MW_GEOIP_DB` owns the database.
    Environment,
    /// No data directory (Postgres without `MW_GEOIP_DIR`).
    NoDirectory,
    /// Another download, upload or rollback is running.
    Busy,
    /// Nothing to roll back to.
    NoPrevious,
    /// The file is not a usable country database.
    Invalid(InvalidDatabase),
    /// The download failed.
    Fetch(String),
    /// The configuration cannot be used (a missing key).
    Config(String),
    /// A filesystem or store failure.
    Io(String),
}

impl std::fmt::Display for OpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpError::Environment => f.write_str(
                "MW_GEOIP_DB is set, so the environment manages the database; unset it to manage the database here",
            ),
            OpError::NoDirectory => f.write_str(
                "no data directory: set MW_GEOIP_DIR (required with a PostgreSQL database)",
            ),
            OpError::Busy => f.write_str("another database operation is running"),
            OpError::NoPrevious => f.write_str("there is no previous database to roll back to"),
            OpError::Invalid(e) => write!(f, "{e}"),
            OpError::Fetch(m) | OpError::Config(m) | OpError::Io(m) => f.write_str(m),
        }
    }
}

/// Validate `bytes` off the runtime.
async fn validate(bytes: Vec<u8>) -> Result<(CountryDatabase, geoip::Sample), InvalidDatabase> {
    tokio::task::spawn_blocking(move || CountryDatabase::open_validated(bytes))
        .await
        .unwrap_or(Err(InvalidDatabase::Unreadable))
}

fn sidecar_for(
    db: &CountryDatabase,
    source: &str,
    by: &str,
    release: Option<String>,
    label: Option<String>,
    archive_sha256: Option<String>,
    checksum_verified: bool,
) -> Sidecar {
    let info = db.info();
    Sidecar {
        source: source.into(),
        release,
        label,
        installed_at: now(),
        installed_by: by.into(),
        sha256: info.sha256.clone(),
        archive_sha256,
        checksum_verified,
        database_type: info.database_type.clone(),
        build_epoch: info.build_epoch,
        size_bytes: info.size_bytes,
        networks: db.country_networks().unwrap_or(0),
    }
}

/// Install validated `bytes` as the active database: keep the current file as
/// the previous one, replace `active.mmdb` atomically, then swap the reader.
fn install_files(dir: &Path, bytes: &[u8], sidecar: &Sidecar) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let active = active_path(dir);
    let previous = previous_path(dir);
    if active.exists() {
        let current = std::fs::read(&active)?;
        write_atomic(&previous, &current)?;
        match std::fs::read(sidecar_of(&active)) {
            Ok(meta) => write_atomic(&sidecar_of(&previous), &meta)?,
            Err(_) => {
                let _ = std::fs::remove_file(sidecar_of(&previous));
            }
        }
    }
    write_atomic(&active, bytes)?;
    write_atomic(
        &sidecar_of(&active),
        &serde_json::to_vec_pretty(sidecar).expect("sidecar json"),
    )?;
    Ok(())
}

fn swap_in(db: CountryDatabase, dir: &Path) {
    geoip::install(Some(Arc::new(db)));
    *runtime()
        .loaded_stamp
        .write()
        .unwrap_or_else(|e| e.into_inner()) = stamp(&active_path(dir));
    *runtime()
        .load_error
        .write()
        .unwrap_or_else(|e| e.into_inner()) = None;
}

struct BusyGuard;
impl Drop for BusyGuard {
    fn drop(&mut self) {
        runtime().busy.store(false, Ordering::SeqCst);
    }
}

async fn begin() -> Result<(PathBuf, tokio::sync::MutexGuard<'static, ()>, BusyGuard), OpError> {
    if geoip::environment_path().is_some() {
        return Err(OpError::Environment);
    }
    let dir = dir().ok_or(OpError::NoDirectory)?;
    let guard = runtime().op.try_lock().map_err(|_| OpError::Busy)?;
    runtime().busy.store(true, Ordering::SeqCst);
    Ok((dir, guard, BusyGuard))
}

/// Validate and install an uploaded file (`.mmdb`, or gzip of one).
pub(crate) async fn install_upload(bytes: Vec<u8>, by: &str) -> Result<Sidecar, OpError> {
    let (dir, _guard, _busy) = begin().await?;
    let bytes = tokio::task::spawn_blocking(move || download::unpack(&bytes))
        .await
        .map_err(|_| OpError::Io("unpacking failed".into()))?
        .map_err(OpError::Fetch)?;
    let (db, _) = validate(bytes.clone()).await.map_err(OpError::Invalid)?;
    let sidecar = sidecar_for(&db, "upload", by, None, None, None, false);
    let d = dir.clone();
    let s = sidecar.clone();
    tokio::task::spawn_blocking(move || install_files(&d, &bytes, &s))
        .await
        .map_err(|_| OpError::Io("install failed".into()))?
        .map_err(|e| OpError::Io(format!("could not write the data directory: {e}")))?;
    swap_in(db, &dir);
    Ok(sidecar)
}

/// What an update run did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UpdateOutcome {
    Installed(Box<Sidecar>),
    /// The provider's current release is the installed one.
    UpToDate,
}

/// Download from the configured provider. With `force` a release that is
/// already installed is downloaded and installed again; without it ("check for
/// updates", the schedule) an unchanged release is left alone.
pub(crate) async fn update(
    state: &AppState,
    by: &str,
    force: bool,
) -> Result<UpdateOutcome, OpError> {
    let (dir, _guard, _busy) = begin().await?;
    let config = provider::load(&state.store)
        .await
        .map_err(|_| OpError::Config("the country database configuration is unreadable".into()))?;
    let plans = provider::plans(&config, &state.store, chrono::Utc::now())
        .map_err(|m| OpError::Config(m.into()))?;
    let installed = read_sidecar(&active_path(&dir)).filter(|_| geoip::installed().is_some());
    let mut last_error = OpError::Fetch("no download was attempted".into());
    for plan in &plans {
        if !force
            && let Some(current) = &installed
            && current.source == config.provider.as_str()
        {
            if plan.release.is_some() && current.release == plan.release {
                return Ok(UpdateOutcome::UpToDate);
            }
            if plan.checksum_url.is_some()
                && let Ok(Some(sum)) = download::published_checksum(plan).await
                && current.archive_sha256.as_deref() == Some(sum.as_str())
            {
                return Ok(UpdateOutcome::UpToDate);
            }
        }
        let fetched = match download::fetch(plan).await {
            Ok(f) => f,
            Err(download::FetchError::NotFound) if plan.release.is_some() => {
                last_error = OpError::Fetch(download::FetchError::NotFound.to_string());
                continue;
            }
            Err(e) => return Err(OpError::Fetch(e.to_string())),
        };
        let (db, _) = validate(fetched.database.clone())
            .await
            .map_err(OpError::Invalid)?;
        if !force
            && installed
                .as_ref()
                .is_some_and(|c| c.sha256 == db.info().sha256)
        {
            return Ok(UpdateOutcome::UpToDate);
        }
        let sidecar = sidecar_for(
            &db,
            config.provider.as_str(),
            by,
            plan.release.clone(),
            Some(plan.label.clone()),
            Some(fetched.archive_sha256.clone()),
            fetched.checksum_verified,
        );
        let d = dir.clone();
        let s = sidecar.clone();
        let bytes = fetched.database;
        tokio::task::spawn_blocking(move || install_files(&d, &bytes, &s))
            .await
            .map_err(|_| OpError::Io("install failed".into()))?
            .map_err(|e| OpError::Io(format!("could not write the data directory: {e}")))?;
        swap_in(db, &dir);
        return Ok(UpdateOutcome::Installed(Box::new(sidecar)));
    }
    Err(last_error)
}

/// Swap `active.mmdb` and `previous.mmdb` (validating the previous one first).
pub(crate) async fn rollback(by: &str) -> Result<Sidecar, OpError> {
    let (dir, _guard, _busy) = begin().await?;
    let previous = previous_path(&dir);
    if !previous.exists() {
        return Err(OpError::NoPrevious);
    }
    let p = previous.clone();
    let (db, _) = tokio::task::spawn_blocking(move || CountryDatabase::open_validated_path(&p))
        .await
        .unwrap_or(Err(InvalidDatabase::Unreadable))
        .map_err(OpError::Invalid)?;
    let d = dir.clone();
    let by = by.to_string();
    let sidecar = tokio::task::spawn_blocking(move || -> std::io::Result<Sidecar> {
        let active = active_path(&d);
        let previous = previous_path(&d);
        let old_bytes = std::fs::read(&previous)?;
        let old_meta = read_sidecar(&previous);
        let cur_bytes = std::fs::read(&active).ok();
        let cur_meta = std::fs::read(sidecar_of(&active)).ok();
        // The previous file becomes active (atomic replace), then the old
        // active file becomes the previous one, so a second rollback undoes it.
        write_atomic(&active, &old_bytes)?;
        let mut meta = old_meta.unwrap_or_else(|| Sidecar {
            source: "unknown".into(),
            release: None,
            label: None,
            installed_at: now(),
            installed_by: by.clone(),
            sha256: download::sha256_hex(&old_bytes),
            archive_sha256: None,
            checksum_verified: false,
            database_type: String::new(),
            build_epoch: 0,
            size_bytes: old_bytes.len() as u64,
            networks: 0,
        });
        meta.installed_at = now();
        meta.installed_by = by;
        write_atomic(
            &sidecar_of(&active),
            &serde_json::to_vec_pretty(&meta).expect("sidecar json"),
        )?;
        if let Some(bytes) = cur_bytes {
            write_atomic(&previous, &bytes)?;
            match cur_meta {
                Some(m) => write_atomic(&sidecar_of(&previous), &m)?,
                None => {
                    let _ = std::fs::remove_file(sidecar_of(&previous));
                }
            }
        }
        Ok(meta)
    })
    .await
    .map_err(|_| OpError::Io("rollback failed".into()))?
    .map_err(|e| OpError::Io(format!("could not write the data directory: {e}")))?;
    swap_in(db, &dir);
    Ok(sidecar)
}

/// Load `active.mmdb` (or the seed file when the directory has none) into the
/// lookup slot. Failure leaves the slot empty and records why.
async fn load_active(dir: &Path) {
    let active = active_path(dir);
    if !active.exists()
        && let Some(seed) = std::env::var_os(ENV_SEED)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .filter(|p| p.exists())
    {
        let d = dir.to_path_buf();
        let seeded = tokio::task::spawn_blocking(move || -> Result<(), String> {
            let bytes = std::fs::read(&seed).map_err(|e| e.to_string())?;
            let bytes = download::unpack(&bytes)?;
            let (db, _) =
                CountryDatabase::open_validated(bytes.clone()).map_err(|e| e.to_string())?;
            let sidecar = sidecar_for(&db, "seed", "system", None, None, None, false);
            install_files(&d, &bytes, &sidecar).map_err(|e| e.to_string())
        })
        .await;
        match seeded {
            Ok(Ok(())) => {
                tracing::info!("country database: installed the seed file from MW_GEOIP_SEED")
            }
            Ok(Err(e)) => {
                tracing::warn!("country database: the MW_GEOIP_SEED file was not installed: {e}")
            }
            Err(_) => {}
        }
    }
    if !active.exists() {
        geoip::install(None);
        return;
    }
    let path = active.clone();
    let opened = tokio::task::spawn_blocking(move || CountryDatabase::open_validated_path(&path))
        .await
        .unwrap_or(Err(InvalidDatabase::Unreadable));
    match opened {
        Ok((db, _)) => swap_in(db, dir),
        Err(e) => {
            tracing::warn!("country database: {} is not usable: {e}", active.display());
            geoip::install(None);
            *runtime()
                .load_error
                .write()
                .unwrap_or_else(|e| e.into_inner()) = Some(e.to_string());
            *runtime()
                .loaded_stamp
                .write()
                .unwrap_or_else(|e| e.into_inner()) = stamp(&active);
        }
    }
}

/// Whether a country policy is in force (`security.signin.geoScope` is not off).
/// An unreadable setting counts as in force: the warning errs on the safe side.
pub(crate) async fn policy_active(state: &AppState) -> bool {
    match crate::settings_registry::effective_for(
        &state.store,
        "security.signin.geoScope",
        &Default::default(),
    )
    .await
    {
        Ok(Some(e)) => e.value.as_str() != Some("off"),
        _ => true,
    }
}

/// `loaded`, `missing` or `invalid`, with the reason when invalid.
pub(crate) fn database_state() -> (&'static str, Option<String>, Option<Arc<CountryDatabase>>) {
    if let Some(path) = geoip::environment_path() {
        if !path.exists() {
            return ("missing", None, None);
        }
        return match geoip::environment_database(&path) {
            Ok(db) => ("loaded", None, Some(db)),
            Err(InvalidDatabase::Unreadable) if !path.exists() => ("missing", None, None),
            Err(e) => ("invalid", Some(e.to_string()), None),
        };
    }
    if let Some(db) = geoip::installed() {
        return ("loaded", None, Some(db));
    }
    match runtime()
        .load_error
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
    {
        Some(e) => ("invalid", Some(e), None),
        None => ("missing", None, None),
    }
}

fn info_json(info: &DatabaseInfo, networks: Option<u64>) -> Value {
    json!({"databaseType":info.database_type,"buildEpoch":info.build_epoch,"sizeBytes":info.size_bytes,
        "sha256":info.sha256,"networks":networks,"ipVersion":info.ip_version,"description":info.description})
}

pub(crate) fn is_stale(build_epoch: u64) -> bool {
    build_epoch > 0 && now() - build_epoch as i64 > STALE_AFTER_SECS
}

/// The console's status object.
pub(crate) async fn status(state: &AppState) -> Value {
    let (db_state, error, db) = database_state();
    let environment = geoip::environment_path().is_some();
    let dir = dir();
    let database = match &db {
        Some(db) => {
            let d = Arc::clone(db);
            let networks = tokio::task::spawn_blocking(move || d.country_networks().ok())
                .await
                .ok()
                .flatten();
            let mut v = info_json(db.info(), networks);
            let sidecar = if environment {
                None
            } else {
                dir.as_ref().and_then(|d| read_sidecar(&active_path(d)))
            };
            v["source"] = json!(if environment {
                "environment".to_string()
            } else {
                sidecar
                    .as_ref()
                    .map(|s| s.source.clone())
                    .unwrap_or_else(|| "unknown".into())
            });
            if let Some(s) = sidecar {
                v["release"] = json!(s.release);
                v["label"] = json!(s.label);
                v["installedAt"] = json!(s.installed_at);
                v["installedBy"] = json!(s.installed_by);
                v["checksumVerified"] = json!(s.checksum_verified);
            }
            v["stale"] = json!(is_stale(db.info().build_epoch));
            v
        }
        None => Value::Null,
    };
    let previous = match (&dir, environment) {
        (Some(d), false) if previous_path(d).exists() => read_sidecar(&previous_path(d))
            .map(|s| json!({"source":s.source,"release":s.release,"buildEpoch":s.build_epoch,"sizeBytes":s.size_bytes,"installedAt":s.installed_at,"sha256":s.sha256}))
            .unwrap_or_else(|| json!({"source":"unknown"})),
        _ => Value::Null,
    };
    let config = provider::load(&state.store).await.unwrap_or_default();
    let schedule = load_schedule(state).await.0;
    let has_history = !environment
        && dir
            .as_ref()
            .and_then(|d| read_sidecar(&active_path(d)))
            .is_some_and(|s| s.source == config.provider.as_str());
    json!({
        "managedBy": if environment { "environment" } else { "console" },
        "state": db_state,
        "error": error,
        "dataDirectory": dir.as_ref().map(|d| d.display().to_string()),
        "writable": !environment && dir.is_some(),
        "database": database,
        "previous": previous,
        "policyActive": policy_active(state).await,
        "busy": runtime().busy.load(Ordering::SeqCst),
        "config": config.public(),
        "providers": provider::Provider::ALL.iter().map(|p| p.describe()).collect::<Vec<_>>(),
        "schedule": {
            "autoUpdate": config.auto_update && !environment,
            "lastCheckAt": schedule.last_check_at,
            "lastSuccessAt": schedule.last_success_at,
            "nextCheckAt": if config.auto_update && !environment && has_history { schedule.next_check_at } else { None },
            "lastError": schedule.last_error,
            "failures": schedule.failures,
            "waitingForFirstDownload": !has_history,
        },
        "staleAfterDays": STALE_AFTER_SECS / 86_400,
    })
}

// ── schedule ──────────────────────────────────────────────────────────────────

async fn load_schedule(state: &AppState) -> (Schedule, Option<String>) {
    let raw = state.store.get_setting(SCHEDULE_KEY).await.ok().flatten();
    let schedule = raw
        .as_deref()
        .and_then(|r| serde_json::from_str(r).ok())
        .unwrap_or_default();
    (schedule, raw)
}

async fn save_schedule(state: &AppState, expected: Option<&str>, schedule: &Schedule) -> bool {
    let value = serde_json::to_string(schedule).expect("schedule json");
    state
        .store
        .compare_and_set_setting(SCHEDULE_KEY, expected, &value)
        .await
        .unwrap_or(false)
}

/// Record the outcome of a check (manual or scheduled) in the schedule.
pub(crate) async fn record_check(state: &AppState, result: Result<(), String>, interval_days: u32) {
    for _ in 0..3 {
        let (mut s, raw) = load_schedule(state).await;
        let t = now();
        s.last_check_at = Some(t);
        s.claimed_until = None;
        match &result {
            Ok(()) => {
                s.last_success_at = Some(t);
                s.last_error = None;
                s.failures = 0;
                s.next_check_at = Some(t + i64::from(interval_days) * 86_400);
            }
            Err(e) => {
                s.last_error = Some(e.chars().take(300).collect());
                s.failures = s.failures.saturating_add(1);
                s.next_check_at = Some(t + backoff_secs(s.failures));
            }
        }
        if save_schedule(state, raw.as_deref(), &s).await {
            return;
        }
    }
}

/// Retry delay after `failures` failed attempts in a row: 1 h, 2 h, 4 h … 24 h.
pub(crate) fn backoff_secs(failures: u32) -> i64 {
    let hours = 1i64 << failures.saturating_sub(1).min(5);
    hours.min(24) * 3600
}

/// One pass of the background loop: notice a file another instance replaced,
/// then run a scheduled update when one is due.
async fn tick(state: &AppState) {
    if geoip::environment_path().is_some() {
        return;
    }
    let Some(dir) = dir() else { return };
    let current = stamp(&active_path(&dir));
    let loaded = *runtime()
        .loaded_stamp
        .read()
        .unwrap_or_else(|e| e.into_inner());
    if current != loaded && !runtime().busy.load(Ordering::SeqCst) {
        load_active(&dir).await;
    }
    let Ok(config) = provider::load(&state.store).await else {
        return;
    };
    // Only after an administrator downloaded from this provider once: the
    // schedule keeps a managed copy fresh; it never makes the first request.
    let has_history =
        read_sidecar(&active_path(&dir)).is_some_and(|s| s.source == config.provider.as_str());
    if !config.auto_update || !has_history {
        return;
    }
    let (mut schedule, raw) = load_schedule(state).await;
    let t = now();
    let due = schedule.next_check_at.is_none_or(|next| t >= next);
    if !due || schedule.claimed_until.is_some_and(|until| until > t) {
        return;
    }
    schedule.claimed_until = Some(t + 900);
    if !save_schedule(state, raw.as_deref(), &schedule).await {
        return; // another instance claimed this run
    }
    let outcome = update(state, "scheduler", false).await;
    let detail = match &outcome {
        Ok(UpdateOutcome::Installed(s)) => {
            json!({"result":"installed","source":s.source,"release":s.release,"sha256":s.sha256,"buildEpoch":s.build_epoch,"checksumVerified":s.checksum_verified})
        }
        Ok(UpdateOutcome::UpToDate) => json!({"result":"up-to-date","source":config.provider}),
        Err(e) => json!({"result":"failed","source":config.provider,"reason":e.to_string()}),
    };
    crate::audit::security_event(
        state,
        crate::audit::SecurityEvent::system("geoip-database-update")
            .target("geoip:database")
            .detail(detail),
    )
    .await;
    record_check(
        state,
        outcome.map(|_| ()).map_err(|e| e.to_string()),
        config.interval_days,
    )
    .await;
}

/// Load the database for this process and start the background loop. Called
/// once per app from `build_app`.
pub(crate) fn start(state: AppState, dir: Option<PathBuf>) {
    *runtime().dir.write().unwrap_or_else(|e| e.into_inner()) = dir.clone();
    tokio::spawn(async move {
        if geoip::environment_path().is_none()
            && let Some(dir) = &dir
        {
            load_active(dir).await;
        }
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(
                LOOP_MS.load(Ordering::Relaxed),
            ))
            .await;
            tick(&state).await;
        }
    });
}

/// Log, at most once per five minutes, that a sign-in was refused because the
/// policy is active and the database is not usable.
pub(crate) fn note_refusal() {
    static LAST: AtomicI64 = AtomicI64::new(0);
    let t = now();
    let last = LAST.load(Ordering::Relaxed);
    if t - last >= 300
        && LAST
            .compare_exchange(last, t, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
    {
        tracing::warn!(
            "sign-in refused: a country policy is active and the local country database is missing or invalid. Open the admin console at Security > Country database to download or upload one (or fix MW_GEOIP_DB when it is set)."
        );
    }
}

/// Dashboard warnings: a missing, invalid or stale database while a country
/// policy is active.
pub(crate) async fn dashboard_warnings(state: &AppState) -> Vec<Value> {
    if !policy_active(state).await {
        return Vec::new();
    }
    match database_state() {
        ("loaded", _, Some(db)) if is_stale(db.info().build_epoch) => {
            vec![json!({"code":"geoip-database-stale","count":1})]
        }
        ("loaded", _, _) => Vec::new(),
        _ => vec![json!({"code":"geoip-database-missing","count":1})],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_dir_follows_the_database() {
        // SAFETY of the env read: MW_GEOIP_DIR is not set in unit tests.
        if std::env::var_os(ENV_DIR).is_some() {
            return;
        }
        assert_eq!(
            data_dir("/data/mailwoman.db"),
            Some(PathBuf::from("/data/geoip"))
        );
        assert_eq!(
            data_dir("sqlite:///data/mw.db?mode=rwc"),
            Some(PathBuf::from("/data/geoip"))
        );
        assert_eq!(data_dir("mailwoman.db"), Some(PathBuf::from("./geoip")));
        assert_eq!(data_dir("postgres://u@h/db"), None);
        assert_eq!(data_dir("sqlite::memory:"), None);
    }

    #[test]
    fn backoff_doubles_to_a_day() {
        assert_eq!(backoff_secs(1), 3600);
        assert_eq!(backoff_secs(2), 7200);
        assert_eq!(backoff_secs(3), 14_400);
        assert_eq!(backoff_secs(6), 24 * 3600);
        assert_eq!(backoff_secs(40), 24 * 3600);
    }
}
