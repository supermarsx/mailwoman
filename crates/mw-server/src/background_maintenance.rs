//! Online housekeeping is bounded, policy-controlled and observable without persisting a sample per run.
use crate::AppState;
use mw_store::background_maintenance::{MaintenanceOptions, MaintenanceReport};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
    time::Duration,
};
static STATUS: LazyLock<Mutex<HashMap<uuid::Uuid, Value>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static RUN: LazyLock<tokio::sync::Semaphore> = LazyLock::new(|| tokio::sync::Semaphore::new(1));
pub(crate) fn status(store: &mw_store::Store) -> Value {
    STATUS
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(&store.instance_id())
        .cloned()
        .unwrap_or_else(|| json!({"running":false,"lastReport":null,"nextRunAt":null}))
}
fn save(store: &mw_store::Store, value: Value) {
    let mut map = STATUS.lock().unwrap_or_else(|p| p.into_inner());
    if map.len() >= 32 && !map.contains_key(&store.instance_id()) {
        map.clear();
    }
    map.insert(store.instance_id(), value);
}
async fn setting(store: &mw_store::Store, key: &str) -> Value {
    let default = crate::settings_registry::lookup(key)
        .expect("maintenance setting registered")
        .default_value();
    match store.get_setting_policy(key).await {
        Ok(Some(policy)) => policy
            .global_value
            .as_deref()
            .map(|value| serde_json::from_str(value).unwrap_or(Value::Null))
            .unwrap_or(default),
        Ok(None) => default,
        Err(_) => Value::Null,
    }
}
async fn number(store: &mw_store::Store, key: &str) -> u64 {
    setting(store, key).await.as_u64().unwrap_or(0)
}
const KEYS: [&str; 15] = [
    "maintenance.database.enabled",
    "maintenance.database.orphanBodies",
    "maintenance.database.expireCachedBodies",
    "maintenance.database.cachedBodyRetentionDays",
    "maintenance.database.changeLogRetentionDays",
    "maintenance.database.changeLogBatchRows",
    "maintenance.database.incrementalVacuum",
    "maintenance.database.intervalSeconds",
    "maintenance.database.batchRows",
    "maintenance.database.batchMiB",
    "maintenance.database.graceHours",
    "maintenance.database.batchPages",
    "maintenance.database.maxRunMs",
    "maintenance.database.minFreeMiB",
    "maintenance.database.maxWalMiB",
];
/// One statement observes all global maintenance policies; no per-key database round trips.
async fn settings_snapshot(store: &mw_store::Store) -> HashMap<&'static str, Value> {
    let policies = store.get_setting_policies_batch(&KEYS).await;
    KEYS.into_iter()
        .map(|key| {
            let value = match &policies {
                Ok(rows) => rows
                    .iter()
                    .find(|row| row.key == key)
                    .and_then(|row| row.global_value.as_deref())
                    .map(|text| serde_json::from_str(text).unwrap_or(Value::Null))
                    .unwrap_or_else(|| {
                        crate::settings_registry::lookup(key)
                            .expect("maintenance setting registered")
                            .default_value()
                    }),
                Err(_) => Value::Null,
            };
            (key, value)
        })
        .collect()
}
pub(crate) async fn run_once(state: &AppState) -> Result<Value, String> {
    let _permit = RUN
        .try_acquire()
        .map_err(|_| "maintenance-busy".to_string())?;
    let store = &state.store;
    let settings = settings_snapshot(store).await;
    let number = |key: &str| settings.get(key).and_then(Value::as_u64).unwrap_or(0);
    let enabled = settings.get("maintenance.database.enabled") == Some(&Value::Bool(true));
    let interval = number("maintenance.database.intervalSeconds").clamp(15, 86400);
    let options = MaintenanceOptions {
        collect_bodies: enabled
            && settings.get("maintenance.database.orphanBodies") == Some(&Value::Bool(true)),
        incremental_vacuum: enabled
            && settings.get("maintenance.database.incrementalVacuum") == Some(&Value::Bool(true)),
        batch_rows: number("maintenance.database.batchRows") as u32,
        batch_bytes: number("maintenance.database.batchMiB") * 1024 * 1024,
        grace_seconds: number("maintenance.database.graceHours") * 3600,
        batch_pages: number("maintenance.database.batchPages") as u32,
        max_run_ms: number("maintenance.database.maxRunMs"),
        min_free_bytes: number("maintenance.database.minFreeMiB") * 1024 * 1024,
        max_wal_bytes: number("maintenance.database.maxWalMiB") * 1024 * 1024,
    };
    let previous = status(store);
    let cursor = previous["lastReport"]["cursor"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let next = chrono::Utc::now().timestamp() + interval as i64;
    let cached_days=number("maintenance.database.cachedBodyRetentionDays");
    let expiry_options = mw_store::body_cache_expiry::CachedBodyExpiryOptions {
        enabled: enabled
            && settings.get("maintenance.database.expireCachedBodies") == Some(&Value::Bool(true))
            && (1..=3650).contains(&cached_days),
        retention_days: cached_days.min(3650) as u32,
        batch_rows: options.batch_rows,
        batch_bytes: options.batch_bytes,
        max_run_ms: options.max_run_ms,
        max_wal_bytes: options.max_wal_bytes,
    };
    let mut current = json!({"running":enabled,"enabled":enabled,"options":options,"lastReport":previous["lastReport"],"cachedBodyExpiry":previous["cachedBodyExpiry"],"changeLogRetention":previous["changeLogRetention"],"nextRunAt":next,"lastError":null});
    save(store, current.clone());
    if enabled {
        match store.background_maintenance(options, cursor).await {
            Ok(report) => current["lastReport"] = serde_json::to_value(report).unwrap(),
            Err(error) => {
                tracing::warn!(%error,"Background database maintenance deferred");
                current["lastError"] = json!("maintenance-unavailable");
            }
        }
        let expiry_cursor = serde_json::from_value(previous["cachedBodyExpiry"]["cursor"].clone())
            .unwrap_or_default();
        let sources = state
            .engine
            .as_ref()
            .map(|engine| engine.body_refetchable_accounts())
            .unwrap_or_default();
        match store
            .expire_cached_bodies(expiry_options, expiry_cursor, sources)
            .await
        {
            Ok(report) => {
                // Keep expiry cursors independent of the orphan collector's body-key cursor.
                let expiry = serde_json::to_value(report).unwrap();
                current["cachedBodyExpiry"] = expiry.clone();
                // A failed orphan pass must not invent a successful baseline report.
                if current["lastReport"].is_object() {
                    current["lastReport"]["cachedBodyExpiry"] = expiry;
                }
            }
            Err(error) => {
                tracing::warn!(%error,"Cached body expiry deferred");
                current["lastError"] = json!("cache-expiry-unavailable");
            }
        }
        let change_options=mw_store::change_log_retention::ChangeLogRetentionOptions{
            enabled:number("maintenance.database.changeLogRetentionDays")>0,
            retention_days:number("maintenance.database.changeLogRetentionDays") as u32,
            batch_rows:number("maintenance.database.changeLogBatchRows") as u32,
            max_run_ms:number("maintenance.database.maxRunMs"),
            max_wal_bytes:number("maintenance.database.maxWalMiB")*1024*1024,
        };
        match store.expire_change_log_resumable(change_options).await{
            Ok(report)=>current["changeLogRetention"]=serde_json::to_value(report).unwrap(),
            Err(error)=>{
                tracing::warn!(%error,"Sync history retention deferred");
                current["lastError"]=json!("change-log-retention-unavailable");
            }
        }
    }
    current["running"] = json!(false);
    save(store, current.clone());
    Ok(current)
}
pub(crate) fn start(state: AppState) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(30)).await;
        loop {
            let _ = run_once(&state).await;
            let seconds = number(&state.store, "maintenance.database.intervalSeconds")
                .await
                .clamp(15, 86400);
            tokio::time::sleep(Duration::from_secs(seconds)).await;
        }
    });
}
/// Retention never runs a full SQLite VACUUM behind an operator's back.
pub(crate) async fn reclaim_after_retention(state: &AppState) -> bool {
    if !state.store.uses_sqlite() {
        return state.store.vacuum_database().await.is_ok();
    }
    run_once(state)
        .await
        .ok()
        .and_then(|value| {
            serde_json::from_value::<MaintenanceReport>(value["lastReport"].clone()).ok()
        })
        .is_some_and(|report| report.reclaimed_pages > 0)
}

#[cfg(test)]
mod policy_tests {
    use super::*;
    #[tokio::test]
    async fn batched_global_snapshot_preserves_defaults_overrides_and_fail_closed_values() {
        let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let defaults = settings_snapshot(&store).await;
        assert_eq!(defaults["maintenance.database.enabled"], json!(true));
        for (key, value) in [
            ("maintenance.database.enabled", Some("false")),
            ("maintenance.database.batchRows", Some("23")),
            ("maintenance.database.maxRunMs", Some("malformed JSON")),
            ("maintenance.database.graceHours", None),
        ] {
            store
                .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
                    key: key.into(),
                    global_value: value.map(str::to_owned),
                    allow_user_override: false,
                    global_locked: true,
                    updated_by: "fixture".into(),
                    updated_at: 1,
                })
                .await
                .unwrap();
        }
        let loaded = settings_snapshot(&store).await;
        assert_eq!(loaded["maintenance.database.enabled"], json!(false));
        assert_eq!(loaded["maintenance.database.batchRows"], json!(23));
        assert_eq!(loaded["maintenance.database.maxRunMs"], Value::Null);
        assert_eq!(
            loaded["maintenance.database.graceHours"],
            defaults["maintenance.database.graceHours"]
        );
        for key in KEYS {
            assert_eq!(loaded[key], setting(&store, key).await, "{key}");
        }
        store.close().await;
        let unavailable = settings_snapshot(&store).await;
        assert!(unavailable.values().all(Value::is_null));
    }
}
