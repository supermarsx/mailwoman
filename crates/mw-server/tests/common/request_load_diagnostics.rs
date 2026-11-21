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

const CACHE_AUDIT_WAIT_MS: u64 = 1000;
const CACHE_AUDIT_RETRY_MS: u64 = 10;
const CACHE_AUDIT_MAX_ATTEMPTS: u32 = 100;

fn complete_cache_audit(audit: &serde_json::Value) -> bool {
    let sample = &audit["sample"];
    sample["completeIdleCoverage"] == true
        && sample["deadlineReached"] == false
        && sample["failedConnections"] == 0
        && sample["sampledConnections"].as_u64().is_some_and(|count| {
            count > 0
                && sample["openBefore"].as_u64() == Some(count)
                && sample["openAfter"].as_u64() == Some(count)
        })
}

/// Outside every measured window, allow asynchronous lease returns and real
/// background work to settle. Never accept partial/zero coverage. Each Store
/// audit still has its original cooperative 25ms bound; this wrapper adds a
/// cooperative 1s overall deadline and a finite attempt count, without acquiring
/// extra handles or changing pool policy. Synchronous SQLite/OS work cannot be
/// preempted by a Tokio deadline.
#[derive(Debug)]
pub struct CacheAuditFailure {
    report: serde_json::Value,
}
impl CacheAuditFailure {
    /// Fixed codes and bounded aggregate outcomes only: no pool/handle IDs,
    /// credentials, SQL text, paths or arbitrary Store error strings.
    pub fn report(&self) -> &serde_json::Value {
        &self.report
    }
}
fn wait_report(
    started: tokio::time::Instant,
    attempts: u32,
    observations: Vec<serde_json::Value>,
) -> serde_json::Value {
    serde_json::json!({"attempts":attempts,"elapsedMs":started.elapsed().as_millis() as u64,
        "budgetMs":CACHE_AUDIT_WAIT_MS,"retryIntervalMs":CACHE_AUDIT_RETRY_MS,
        "deadlineIsCooperative":true,"observations":observations})
}
fn failure(
    code: &'static str,
    started: tokio::time::Instant,
    attempts: u32,
    observations: Vec<serde_json::Value>,
) -> CacheAuditFailure {
    CacheAuditFailure {
        report: serde_json::json!({"error":code,
        "sqlitePageCacheAuditWait":wait_report(started,attempts,observations)}),
    }
}
async fn cache_audit_when_quiescent(
    store: &mw_store::Store,
) -> Result<(serde_json::Value, serde_json::Value), CacheAuditFailure> {
    use std::time::Duration;
    let started = tokio::time::Instant::now();
    let deadline = started + Duration::from_millis(CACHE_AUDIT_WAIT_MS);
    let mut observations = Vec::new();
    let mut attempts = 0;
    while attempts < CACHE_AUDIT_MAX_ATTEMPTS && tokio::time::Instant::now() < deadline {
        attempts += 1;
        let audit = match tokio::time::timeout_at(deadline, store.sqlite_page_cache_audit()).await {
            Ok(Ok(audit)) => audit,
            Ok(Err(_)) => {
                return Err(failure(
                    "cache-audit-unavailable",
                    started,
                    attempts,
                    observations,
                ));
            }
            Err(_) => {
                return Err(failure(
                    "cache-audit-quiescence-timeout",
                    started,
                    attempts,
                    observations,
                ));
            }
        };
        let sample = &audit["sample"];
        observations.push(serde_json::json!({"attempt":attempts,
            "openBefore":sample["openBefore"].as_u64(),"openAfter":sample["openAfter"].as_u64(),
            "sampledConnections":sample["sampledConnections"].as_u64(),
            "failedConnections":sample["failedConnections"].as_u64(),
            "completeIdleCoverage":sample["completeIdleCoverage"].as_bool(),
            "deadlineReached":sample["deadlineReached"].as_bool()}));
        if complete_cache_audit(&audit) {
            return Ok((audit, wait_report(started, attempts, observations)));
        }
        tokio::time::sleep_until(std::cmp::min(
            deadline,
            tokio::time::Instant::now() + Duration::from_millis(CACHE_AUDIT_RETRY_MS),
        ))
        .await;
    }
    Err(failure(
        "cache-audit-quiescence-timeout",
        started,
        attempts,
        observations,
    ))
}

/// The successful explicit audit runs BEFORE ordinary pool diagnostics. Their
/// idle sweep drops SQLx leases whose asynchronous return can otherwise make
/// the immediately following audit falsely observe no idle serving handles.
pub async fn snapshot_with_cache_audit(
    store: &mw_store::Store,
    directory: &std::path::Path,
    audit_cache: bool,
) -> Result<serde_json::Value, CacheAuditFailure> {
    let audit = if audit_cache {
        Some(cache_audit_when_quiescent(store).await?)
    } else {
        None
    };
    let mut value = snapshot(store, directory).await;
    if let Some((audit, wait)) = audit {
        value["sqlitePageCacheAudit"] = audit;
        value["sqlitePageCacheAuditWait"] = wait;
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
        // Current-thread scheduling makes generic-sweep→audit order return no
        // handles until Drop's return task runs. The actual helper owns retries.
        let observed = super::snapshot_with_cache_audit(&store, dir.path(), true)
            .await
            .expect("actual composed fixture snapshot retains complete owned audit");
        assert!(super::complete_cache_audit(
            &observed["sqlitePageCacheAudit"]
        ));
        assert!(
            observed["sqlitePageCacheAuditWait"]["attempts"]
                .as_u64()
                .unwrap()
                >= 1
        );
        assert!(
            observed["sqlitePageCacheAudit"]["sample"]["connections"][0]["mainCacheSize"].is_i64()
        );
        assert!(observed["pool"].is_object());
        store.close().await;
    }
    async fn busy_serving_handle_contract(release_before_deadline: bool) {
        use sqlx::Connection;
        use std::time::Duration;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("busy-audit.db");
        let db_key = mw_store::encryption::DbKey::generate();
        let config = mw_store::encryption::EncryptionConfig::with_key(
            mw_store::encryption::EncryptionMode::Required,
            db_key.clone(),
        );
        let store = mw_store::Store::open_with_encryption(
            path.to_str().unwrap(),
            mw_store::ServerKey::generate(),
            &config,
        )
        .await
        .unwrap();
        // A correctly keyed independent connection holds the SQLite write lock.
        // The public Store write then owns a real SERVING-pool lease while its
        // SQLx worker waits for that lock; an unrelated pool would not prove this.
        let options = mw_store::encryption::connect_options(&path, Some(&db_key), false);
        let mut blocker = sqlx::SqliteConnection::connect_with(&options)
            .await
            .unwrap();
        let transaction = blocker.begin_with("BEGIN IMMEDIATE").await.unwrap();
        let writer_store = store.clone();
        let writer = tokio::spawn(async move {
            writer_store
                .set_setting("fixture-cache-audit-held-write", "fixture")
                .await
        });
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if store.database_pool_snapshot().await["busy"]
                    .as_u64()
                    .unwrap()
                    > 0
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("real Store write acquired serving handle");
        assert!(!writer.is_finished());
        let partial = store.sqlite_page_cache_audit().await.unwrap();
        assert!(!super::complete_cache_audit(&partial));
        let audit_store = store.clone();
        let audit_path = dir.path().to_path_buf();
        let audit = tokio::spawn(async move {
            super::snapshot_with_cache_audit(&audit_store, &audit_path, true).await
        });
        if release_before_deadline {
            tokio::time::sleep(Duration::from_millis(40)).await;
            assert!(
                !audit.is_finished(),
                "partial audit cannot acknowledge success"
            );
            transaction.rollback().await.unwrap();
            writer.await.unwrap().unwrap();
            let completed = audit
                .await
                .unwrap()
                .expect("released real serving lease becomes auditable");
            assert!(super::complete_cache_audit(
                &completed["sqlitePageCacheAudit"]
            ));
            assert!(
                completed["sqlitePageCacheAuditWait"]["attempts"]
                    .as_u64()
                    .unwrap()
                    > 1
            );
            assert_eq!(completed["sqlitePageCacheAuditWait"]["budgetMs"], 1000);
        } else {
            let result = audit.await.unwrap();
            let writer_still_busy = !writer.is_finished();
            transaction.rollback().await.unwrap();
            writer.await.unwrap().unwrap();
            let failure = result.expect_err("busy coverage must fail at bounded deadline");
            assert!(writer_still_busy);
            assert_eq!(failure.report()["error"], "cache-audit-quiescence-timeout");
            let outcomes = failure.report()["sqlitePageCacheAuditWait"]["observations"]
                .as_array()
                .unwrap();
            assert!(!outcomes.is_empty() && outcomes.len() <= 100);
            assert!(
                outcomes
                    .iter()
                    .all(|row| row["completeIdleCoverage"] == false)
            );
        }
        blocker.close().await.unwrap();
        store.close().await;
    }

    #[tokio::test]
    async fn composed_fixture_audit_waits_for_a_real_busy_serving_lease_then_requires_full_coverage()
     {
        busy_serving_handle_contract(true).await;
    }

    #[tokio::test]
    async fn composed_fixture_audit_rejects_a_real_busy_serving_lease_at_the_fixed_deadline() {
        busy_serving_handle_contract(false).await;
    }
}
