//! Test-only external-generator control. Production routes/auth are unchanged.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
const CONTROL_LIMIT: u64 = 65536;
pub const HANDOFF_LIMIT: u64 = 64 * 1024 * 1024;
const REPORT_LIMIT: u64 = 64 * 1024 * 1024;
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
fn regular(path: &Path, limit: u64) -> Result<(), String> {
    let m = std::fs::symlink_metadata(path).map_err(|_| "external control file unavailable")?;
    if !m.is_file() || m.file_type().is_symlink() || m.len() > limit {
        return Err("invalid external control file".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if m.nlink() != 1 || m.mode() & 0o077 != 0 || m.uid() != unsafe { libc::geteuid() } {
            return Err("external control file is not private".into());
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err("external control links refused".into());
        }
    }
    Ok(())
}
fn bytes(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    regular(path, limit)?;
    let file = std::fs::File::open(path).map_err(|_| "external control read failed")?;
    let mut value = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut value)
        .map_err(|_| "external control read failed")?;
    if value.len() as u64 > limit {
        return Err("external control size limit exceeded".into());
    }
    Ok(value)
}
fn record(path: &Path) -> Result<Option<Value>, String> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("external control metadata unavailable".into()),
        Ok(_) => serde_json::from_slice(&bytes(path, CONTROL_LIMIT)?)
            .map(Some)
            .map_err(|_| "external control JSON invalid".into()),
    }
}
fn private_new(path: &Path, value: &Value, limit: u64) -> Result<(), String> {
    let content = serde_json::to_vec(value).map_err(|_| "external control encoding failed")?;
    if content.len() as u64 > limit {
        return Err("external control encoding exceeded bound".into());
    }
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| {
        let mut file = options
            .open(&temporary)
            .map_err(|_| "external control create failed")?;
        file.write_all(&content)
            .and_then(|_| file.sync_all())
            .map_err(|_| "external control write failed")?;
        drop(file);
        if path.exists() {
            return Err("external control destination exists");
        }
        std::fs::rename(&temporary, path).map_err(|_| "external control publication failed")
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result.map_err(str::to_owned)
}
/// An explicit owned scratch root must be mounted into app and generator only.
pub async fn run(
    handoff: &Value,
    label: &str,
    report_dir: &Path,
    deadline_secs: u64,
) -> Result<(), String> {
    if label.is_empty()
        || !label
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        || !(10..=1800).contains(&deadline_secs)
    {
        return Err("invalid external campaign bounds".into());
    }
    let root = PathBuf::from(
        std::env::var_os("MW_LOAD_FIXTURE_EXTERNAL_ROOT")
            .ok_or("explicit external scratch root required")?,
    );
    let m = std::fs::symlink_metadata(&root).map_err(|_| "external scratch root unavailable")?;
    if !m.is_dir() || m.file_type().is_symlink() {
        return Err("external scratch root must be an owned directory".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if m.mode() & 0o022 != 0 || m.uid() != unsafe { libc::geteuid() } {
            return Err("external scratch root must be owned and not writable by others".into());
        }
    }
    let root =
        std::fs::canonicalize(root).map_err(|_| "external scratch canonicalization failed")?;
    let run_id = uuid::Uuid::new_v4().to_string();
    let directory = root.join(format!("run-{run_id}"));
    std::fs::create_dir(&directory).map_err(|_| "external run directory create failed")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))
            .map_err(|_| "external run directory privacy failed")?;
    }
    let startup: u64 = std::env::var("MW_LOAD_FIXTURE_EXTERNAL_STARTUP_SECONDS")
        .ok()
        .map(|v| v.parse())
        .transpose()
        .map_err(|_| "invalid external startup bound")?
        .unwrap_or(60);
    if !(10..=120).contains(&startup) {
        return Err("external startup bound outside10..120".into());
    }
    let mut env = serde_json::Map::new();
    for key in [
        "MW_LOAD_FIXTURE_SQLITE_CACHE_EXPECT",
        "MW_LOAD_FIXTURE_MODE",
        "MW_LOAD_FIXTURE_LONG",
        "MW_LOAD_FIXTURE_SECONDS",
        "MW_LOAD_FIXTURE_WARMUP_SECONDS",
        "MW_LOAD_FIXTURE_DEADLINE_SECONDS",
        "MW_LOAD_FIXTURE_CONCURRENCY",
        "MW_LOAD_FIXTURE_ENDPOINTS",
        "MW_LOAD_FIXTURE_RATES",
        "MW_LOAD_FIXTURE_TIMEOUT_MS",
        "MW_LOAD_FIXTURE_MAX_ERROR_RATE",
        "MW_LOAD_FIXTURE_MAX_THROTTLE_RATE",
        "MW_LOAD_FIXTURE_MAX_P99_MS",
        "MW_LOAD_FIXTURE_ALLOW_OVERLOAD",
        "MW_LOAD_FIXTURE_ADAPTIVE",
    ] {
        if let Ok(value) = std::env::var(key) {
            if value.len() > 256 {
                return Err("external campaign option too long".into());
            }
            env.insert(key.into(), value.into());
        }
    }
    let mut handoff = handoff.clone();
    handoff["generatorPlacement"] =
        "separate container/cgroup sharing private network namespace".into();
    let secret = directory.join("handoff.private.json");
    private_new(&secret, &handoff, HANDOFF_LIMIT)?;
    let published = now_ms();
    let expires = published + (deadline_secs + startup + 10) * 1000;
    if let Err(error) = private_new(
        &directory.join("ready.json"),
        &json!({"version":1,"runId":run_id,"expiresAtMs":expires,"deadlineSeconds":deadline_secs,"handoffFile":"handoff.private.json","outputFile":"report.json","campaignEnv":env}),
        CONTROL_LIMIT,
    ) {
        let _ = std::fs::remove_file(&secret);
        return Err(error);
    }
    let outcome = async {
        let started = tokio::time::Instant::now();
        let mut claimed = false;
        let mut last_heartbeat = published;
        loop {
            if now_ms() > expires
                || started.elapsed() > Duration::from_secs(deadline_secs + startup + 10)
            {
                return Err("external campaign deadline exceeded".into());
            }
            if let Some(done) = record(&directory.join("done.json"))? {
                if done["runId"].as_str() != Some(run_id.as_str()) {
                    return Err("external completion identity mismatch".into());
                }
                if done["status"] != "completed" {
                    return Err("external generator reported failure".into());
                }
                let report = bytes(&directory.join("report.json"), REPORT_LIMIT)?;
                let digest: String = Sha256::digest(&report)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect();
                if done["reportHash"].as_str() != Some(digest.as_str())
                    || done["reportBytes"].as_u64() != Some(report.len() as u64)
                {
                    return Err("external report integrity mismatch".into());
                }
                serde_json::from_slice::<Value>(&report)
                    .map_err(|_| "external report JSON invalid")?;
                std::fs::write(report_dir.join(format!("{label}.json")), report)
                    .map_err(|_| "external report export failed")?;
                let markdown = bytes(&directory.join("report.md"), REPORT_LIMIT)?;
                std::fs::write(report_dir.join(format!("{label}.md")), markdown)
                    .map_err(|_| "external report export failed")?;
                return Ok(());
            }
            if let Some(heartbeat) = record(&directory.join("heartbeat.json"))? {
                if heartbeat["runId"].as_str() != Some(run_id.as_str()) {
                    return Err("external heartbeat identity mismatch".into());
                }
                let timestamp = heartbeat["atMs"]
                    .as_u64()
                    .ok_or("external heartbeat timestamp missing")?;
                if timestamp > now_ms() + 5000 {
                    return Err("external heartbeat timestamp invalid".into());
                }
                last_heartbeat = last_heartbeat.max(timestamp);
                claimed = true;
            }
            if !claimed && started.elapsed() > Duration::from_secs(startup) {
                return Err("external generator did not start".into());
            }
            if claimed && now_ms().saturating_sub(last_heartbeat) > 30000 {
                return Err("external generator heartbeat expired".into());
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }
    .await;
    let _ = std::fs::remove_file(secret); // exact-owned credential file only
    outcome
}
