//! Start-up check of the PostgreSQL server's own settings. It warns and never
//! refuses: PostgreSQL is configured by its administrator, and a server that
//! starts with stock settings works, only slower or with less protection than
//! the profile in `docs/admin/postgresql-tuning.md`.
//!
//! What it reads: one `SELECT name, setting, unit FROM pg_settings` for the
//! names in [`READ_SQL`] (the values `SHOW` prints, in base units). What it
//! reports, on standard error at store open:
//! - always: `fsync`, `full_page_writes` or `synchronous_commit` set to `off`
//!   (durability: acknowledged writes can be lost, and with the first two
//!   the database can be corrupted, by a crash or power loss);
//! - when this process sees at least [`RESOURCE_CPUS`] CPUs: `shared_buffers`
//!   at or below the initdb default of 128 MB, and `max_wal_size` at or below
//!   the default of 1 GB. Mailwoman cannot see the database host's memory, so
//!   its own CPU count stands in for "a deployment with resources"; on a
//!   smaller host the stock values are close to the profile anyway.
use crate::{Store, backend::q};

/// CPUs of the application host from which stock memory and WAL sizes are
/// reported as far below the profile.
pub const RESOURCE_CPUS: u32 = 4;
const STOCK_SHARED_BUFFERS: u64 = 128 * 1024 * 1024;
const STOCK_MAX_WAL_SIZE: u64 = 1024 * 1024 * 1024;
const READ_SQL: &str = "SELECT name, setting, COALESCE(unit, '') AS unit FROM pg_settings WHERE name IN ('fsync','full_page_writes','synchronous_commit','shared_buffers','max_wal_size')";
const GUIDE: &str = "see docs/admin/postgresql-tuning.md";

/// One server setting as `pg_settings` reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerSetting {
    pub name: String,
    pub setting: String,
    pub unit: String,
}

/// `setting` in bytes for the memory units `pg_settings` uses (`8kB`, `kB`,
/// `MB`, ...); `None` for anything else.
fn bytes(setting: &ServerSetting) -> Option<u64> {
    let value: u64 = setting.setting.parse().ok()?;
    let digits = setting.unit.bytes().take_while(u8::is_ascii_digit).count();
    let count: u64 = if digits == 0 { 1 } else { setting.unit[..digits].parse().ok()? };
    let scale: u64 = match &setting.unit[digits..] {
        "B" => 1,
        "kB" => 1024,
        "MB" => 1024 * 1024,
        "GB" => 1024 * 1024 * 1024,
        _ => return None,
    };
    value.checked_mul(count)?.checked_mul(scale)
}

/// The warnings for `settings` when this process sees `app_cpus` CPUs.
pub fn assess(settings: &[ServerSetting], app_cpus: u32) -> Vec<String> {
    let get = |name: &str| settings.iter().find(|s| s.name == name);
    let mut warnings = Vec::new();
    for (name, consequence) in [
        ("fsync", "a crash or power loss can lose acknowledged writes and corrupt the database"),
        ("full_page_writes", "a crash or power loss during a page write can leave a torn page the database cannot repair"),
        ("synchronous_commit", "a crash of the database server loses the most recent acknowledged writes (up to about three times wal_writer_delay)"),
    ] {
        if get(name).is_some_and(|s| s.setting == "off") {
            warnings.push(format!("{name}=off: {consequence}; {GUIDE}"));
        }
    }
    if app_cpus >= RESOURCE_CPUS {
        if let Some(value) = get("shared_buffers").and_then(bytes)
            && value <= STOCK_SHARED_BUFFERS
        {
            warnings.push(format!(
                "shared_buffers is {} MB, the initdb default; the profile uses 25% of the database server's memory; {GUIDE}",
                value / (1024 * 1024)
            ));
        }
        if let Some(value) = get("max_wal_size").and_then(bytes)
            && value <= STOCK_MAX_WAL_SIZE
        {
            warnings.push(format!(
                "max_wal_size is {} MB, the default; write load then forces a checkpoint every few seconds and a full-page image for each first page write after it; {GUIDE}",
                value / (1024 * 1024)
            ));
        }
    }
    warnings
}

impl Store {
    /// Read the server settings and return the warnings for this host. A
    /// failed read returns no warnings: the check is advice, never a gate.
    pub async fn postgres_settings_warnings(&self, app_cpus: u32) -> Vec<String> {
        if !matches!(self.backend, crate::backend::Backend::Postgres(_)) {
            return Vec::new();
        }
        let Ok(rows) = q(READ_SQL).fetch_all(&self.backend).await else {
            return Vec::new();
        };
        let settings: Vec<ServerSetting> = rows
            .iter()
            .map(|r| ServerSetting {
                name: r.get_string("name"),
                setting: r.get_string("setting"),
                unit: r.get_string("unit"),
            })
            .collect();
        assess(&settings, app_cpus)
    }

    /// Called once at PostgreSQL store open: print each warning.
    pub(crate) async fn warn_on_postgres_settings(&self) {
        let cpus = std::thread::available_parallelism()
            .map(|n| n.get().min(u32::MAX as usize) as u32)
            .unwrap_or(1);
        for warning in self.postgres_settings_warnings(cpus).await {
            eprintln!("[mw-store] PostgreSQL settings: {warning}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(name: &str, setting: &str, unit: &str) -> ServerSetting {
        ServerSetting { name: name.into(), setting: setting.into(), unit: unit.into() }
    }
    fn stock() -> Vec<ServerSetting> {
        vec![
            s("fsync", "on", ""),
            s("full_page_writes", "on", ""),
            s("synchronous_commit", "on", ""),
            s("shared_buffers", "16384", "8kB"),
            s("max_wal_size", "1024", "MB"),
        ]
    }

    #[test]
    fn units_convert_to_bytes() {
        assert_eq!(bytes(&s("shared_buffers", "16384", "8kB")), Some(128 * 1024 * 1024));
        assert_eq!(bytes(&s("max_wal_size", "1024", "MB")), Some(1024 * 1024 * 1024));
        assert_eq!(bytes(&s("work_mem", "4096", "kB")), Some(4 * 1024 * 1024));
        assert_eq!(bytes(&s("fsync", "on", "")), None);
        assert_eq!(bytes(&s("x", "1", "ms")), None);
    }

    #[test]
    fn stock_settings_warn_only_on_a_host_with_resources() {
        let warnings = assess(&stock(), 8);
        assert_eq!(warnings.len(), 2, "{warnings:?}");
        assert!(warnings[0].starts_with("shared_buffers is 128 MB"), "{warnings:?}");
        assert!(warnings[1].starts_with("max_wal_size is 1024 MB"), "{warnings:?}");
        assert!(assess(&stock(), RESOURCE_CPUS - 1).is_empty());
    }

    #[test]
    fn profile_settings_do_not_warn() {
        let tuned = vec![
            s("fsync", "on", ""),
            s("full_page_writes", "on", ""),
            s("synchronous_commit", "on", ""),
            s("shared_buffers", "262144", "8kB"),
            s("max_wal_size", "4096", "MB"),
        ];
        assert!(assess(&tuned, 64).is_empty());
    }

    #[test]
    fn durability_off_always_warns() {
        let mut settings = stock();
        for setting in &mut settings {
            if matches!(setting.name.as_str(), "fsync" | "full_page_writes" | "synchronous_commit") {
                setting.setting = "off".into();
            }
        }
        let warnings = assess(&settings, 1);
        assert_eq!(warnings.len(), 3, "{warnings:?}");
        assert!(warnings.iter().all(|w| w.contains("=off: ")), "{warnings:?}");
        // `local` and `remote_write` keep local durability: no warning.
        let mut local = stock();
        local[2].setting = "local".into();
        assert!(assess(&local, 1).is_empty());
    }

    #[tokio::test]
    async fn postgres_server_settings_are_read_and_assessed() {
        let Ok(url) = std::env::var("MW_TEST_PG") else {
            mw_test_gate::skip("PostgreSQL settings check: MW_TEST_PG not configured");
            return;
        };
        let store = Store::open_postgres(&url, crate::ServerKey::generate()).await.unwrap();
        let crate::backend::Backend::Postgres(pool) = &store.backend else { unreachable!() };
        let shared: i64 = sqlx::query_scalar("SELECT setting::bigint FROM pg_settings WHERE name='shared_buffers'")
            .fetch_one(pool)
            .await
            .unwrap();
        let warnings = store.postgres_settings_warnings(64).await;
        let stock_buffers = shared * 8192 <= STOCK_SHARED_BUFFERS as i64;
        assert_eq!(
            warnings.iter().any(|w| w.starts_with("shared_buffers is ")),
            stock_buffers,
            "{warnings:?}"
        );
        // A one-CPU host never gets the sizing advice.
        assert!(store.postgres_settings_warnings(1).await.iter().all(|w| w.contains("=off: ")));
        store.close().await;
    }
}
