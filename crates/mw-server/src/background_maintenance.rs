//! Online housekeeping is bounded, policy-controlled and observable without persisting a sample per run.
use crate::AppState;
use mw_store::background_maintenance::{MaintenanceOptions, MaintenanceReport};
use serde_json::{json, Value};
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
pub(crate) async fn run_once(state: &AppState) -> Result<Value, String> {
    let _permit = RUN
        .try_acquire()
        .map_err(|_| "maintenance-busy".to_string())?;
    let store = &state.store;
    let prefix = "maintenance.database";
    let interval = number(store, &format!("{prefix}.intervalSeconds"))
        .await
        .clamp(15, 86400);
    let enabled = setting(store, &format!("{prefix}.enabled")).await == true;
    let options = MaintenanceOptions {
        collect_bodies: enabled && setting(store, &format!("{prefix}.orphanBodies")).await == true,
        incremental_vacuum: enabled
            && setting(store, &format!("{prefix}.incrementalVacuum")).await == true,
        batch_rows: number(store, &format!("{prefix}.batchRows")).await as u32,
        batch_bytes: number(store, &format!("{prefix}.batchMiB")).await * 1024 * 1024,
        grace_seconds: number(store, &format!("{prefix}.graceHours")).await * 3600,
        batch_pages: number(store, &format!("{prefix}.batchPages")).await as u32,
        max_run_ms: number(store, &format!("{prefix}.maxRunMs")).await,
        min_free_bytes: number(store, &format!("{prefix}.minFreeMiB")).await * 1024 * 1024,
        max_wal_bytes: number(store, &format!("{prefix}.maxWalMiB")).await * 1024 * 1024,
    };
    let previous = status(store);
    let cursor = previous["lastReport"]["cursor"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let next = chrono::Utc::now().timestamp() + interval as i64;
    let mut current = json!({"running":enabled,"enabled":enabled,"options":options,"lastReport":previous["lastReport"],"nextRunAt":next,"lastError":null});
    save(store, current.clone());
    if enabled {
        match store.background_maintenance(options, cursor).await {
            Ok(report) => current["lastReport"] = serde_json::to_value(report).unwrap(),
            Err(error) => {
                tracing::warn!(%error,"Background database maintenance deferred");
                current["lastError"] = json!("maintenance-unavailable");
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
