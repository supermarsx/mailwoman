//! Fixture-only adapter. The baseline overlay substitutes a compatible adapter
//! without changing any production backend source.
pub async fn snapshot(store: &mw_store::Store, directory: &std::path::Path) -> serde_json::Value {
    serde_json::json!({
        "pool":store.database_pool_snapshot().await,
        "resources":mw_observability::resource_snapshot::snapshot(directory),
        "requestProfile":mw_server::fixture_profile::snapshot(),
        "profileEnabled":mw_server::fixture_profile::enabled(),
        "adapter":"current"
    })
}
pub fn reset() {
    mw_server::fixture_profile::reset();
}

/// An explicit cache audit must run BEFORE ordinary pool diagnostics. Their
/// idle sweep drops SQLx leases whose asynchronous return can otherwise make
/// the immediately following audit falsely observe no idle serving handles.
pub async fn snapshot_with_cache_audit(
    store: &mw_store::Store,
    directory: &std::path::Path,
    audit_cache: bool,
) -> Result<serde_json::Value, ()> {
    let audit = if audit_cache {
        Some(store.sqlite_page_cache_audit().await.map_err(|_| ())?)
    } else {
        None
    };
    let mut value = snapshot(store, directory).await;
    if let Some(audit) = audit {
        value["sqlitePageCacheAudit"] = audit;
    }
    Ok(value)
}
#[cfg(test)]
mod cache_audit_tests {
    #[tokio::test]
    async fn composed_fixture_snapshot_audits_before_generic_idle_sweep() {
        let dir = tempfile::tempdir().unwrap();
        let config = mw_store::encryption::EncryptionConfig::with_key(
            mw_store::encryption::EncryptionMode::Required,
            mw_store::encryption::DbKey::generate(),
        );
        let store = mw_store::Store::open_with_encryption(
            dir.path().join("audit.db").to_str().unwrap(),
            mw_store::ServerKey::generate(),
            &config,
        )
        .await
        .unwrap();
        // Current-thread scheduling makes the old generic-sweep→audit order
        // deterministically return zero handles: Drop's return task has not run.
        // A bounded retry here only lets independent startup leases finish; it
        // cannot repair the wrong ordering INSIDE the actual composed helper.
        let observed = tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                let snapshot = super::snapshot_with_cache_audit(&store, dir.path(), true)
                    .await
                    .unwrap();
                if snapshot["sqlitePageCacheAudit"]["sample"]["completeIdleCoverage"] == true
                    && snapshot["sqlitePageCacheAudit"]["sample"]["sampledConnections"]
                        .as_u64()
                        .is_some_and(|n| n > 0)
                {
                    break snapshot;
                }
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("actual composed fixture snapshot retains complete owned audit");
        assert!(
            observed["sqlitePageCacheAudit"]["sample"]["connections"][0]["mainCacheSize"].is_i64()
        );
        assert!(observed["pool"].is_object());
        store.close().await;
    }
}
