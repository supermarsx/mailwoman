//! Bounded diagnostics history and probes that never return stored credentials.
use crate::{
    Backend, Store, StoreError,
    backend::{Tx, q},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticRun {
    pub id: String,
    pub suite: String,
    pub started_at: i64,
    pub finished_at: i64,
    pub status: String,
    pub results: serde_json::Value,
    pub triggered_by: String,
}

impl Store {
    pub async fn diagnostic_ping(&self) -> Result<(), StoreError> {
        q("SELECT 1").fetch_one(&self.backend).await?;
        Ok(())
    }

    pub async fn diagnostic_migrations(&self) -> Result<(i64, usize), StoreError> {
        let applied = q("SELECT COUNT(*) FROM _sqlx_migrations WHERE success=TRUE")
            .fetch_scalar_i64(&self.backend)
            .await?;
        let expected = match &self.backend {
            Backend::Sqlite(_) => sqlx::migrate!("./migrations").migrations.len(),
            Backend::Postgres(_) => sqlx::migrate!("./migrations_pg").migrations.len(),
        };
        Ok((applied, expected))
    }

    /// Verify real write access, then explicitly roll back the scratch row.
    pub async fn diagnostic_write_probe(&self, nonce: &str) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings (key,value) VALUES (?1,?2)")
            .bind(format!("diagnostic-scratch:{nonce}"))
            .bind("scratch")
            .execute_tx(&mut tx)
            .await?;
        match tx {
            Tx::Sqlite(tx) => tx.rollback().await?,
            Tx::Postgres(tx) => tx.rollback().await?,
        }
        Ok(())
    }

    /// A synthetic seal/open check; does not claim to validate every stored row.
    pub fn diagnostic_seal_probe(&self) -> Result<(), StoreError> {
        let sample = b"mailwoman diagnostic sample";
        let sealed = self.key.seal(sample)?;
        if self.key.open(&sealed)? != sample {
            return Err(StoreError::Corrupt("seal round trip failed".into()));
        }
        Ok(())
    }

    pub async fn save_diagnostic_run(&self, run: &DiagnosticRun) -> Result<(), StoreError> {
        let results =
            serde_json::to_string(&run.results).map_err(|e| StoreError::Corrupt(e.to_string()))?;
        if results.len() > 64 * 1024
            || !run.results.is_array()
            || !matches!(run.status.as_str(), "pass" | "warn" | "fail" | "error")
        {
            return Err(StoreError::Corrupt("invalid diagnostic result".into()));
        }
        q("INSERT INTO diag_run (id,suite,started_at,finished_at,status,results_json,triggered_by) VALUES (?1,?2,?3,?4,?5,?6,?7)")
            .bind(run.id.as_str()).bind(run.suite.as_str()).bind(run.started_at).bind(run.finished_at)
            .bind(run.status.as_str()).bind(results).bind(run.triggered_by.as_str()).execute(&self.backend).await?;
        Ok(())
    }

    pub async fn diagnostic_runs(
        &self,
        before: Option<(i64, &str)>,
        limit: usize,
    ) -> Result<Vec<DiagnosticRun>, StoreError> {
        let (at, id) = before.unwrap_or((i64::MAX, ""));
        let rows = q("SELECT id,suite,started_at,finished_at,status,results_json,triggered_by FROM diag_run WHERE finished_at IS NOT NULL AND (started_at<?1 OR (started_at=?1 AND id<?2)) ORDER BY started_at DESC,id DESC LIMIT ?3")
            .bind(at).bind(id).bind(limit.clamp(1, 101) as i64).fetch_all(&self.backend).await?;
        rows.into_iter()
            .map(|r| {
                Ok(DiagnosticRun {
                    id: r.get_string("id"),
                    suite: r.get_string("suite"),
                    started_at: r.get_i64("started_at"),
                    finished_at: r.get_i64("finished_at"),
                    status: r.get_string("status"),
                    results: serde_json::from_str(&r.get_string("results_json"))
                        .map_err(|_| StoreError::Corrupt("invalid diagnostic history".into()))?,
                    triggered_by: r.get_string("triggered_by"),
                })
            })
            .collect()
    }

    /// Delete only completed history older than the configured retention cutoff.
    pub async fn prune_diagnostic_runs(&self, cutoff: i64) -> Result<(), StoreError> {
        q("DELETE FROM diag_run WHERE finished_at IS NOT NULL AND finished_at<?1")
            .bind(cutoff)
            .execute(&self.backend)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn probes_roll_back_and_history_pages_stably() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        store.diagnostic_ping().await.unwrap();
        store.diagnostic_write_probe("test").await.unwrap();
        assert!(
            store
                .get_setting("diagnostic-scratch:test")
                .await
                .unwrap()
                .is_none()
        );
        store.diagnostic_seal_probe().unwrap();
        let (actual, expected) = store.diagnostic_migrations().await.unwrap();
        assert_eq!(actual, expected as i64);
        for id in ["a", "b", "c"] {
            store
                .save_diagnostic_run(&DiagnosticRun {
                    id: id.into(),
                    suite: "server".into(),
                    started_at: 1,
                    finished_at: 2,
                    status: "pass".into(),
                    results: serde_json::json!([]),
                    triggered_by: "admin".into(),
                })
                .await
                .unwrap();
        }
        let first = store.diagnostic_runs(None, 2).await.unwrap();
        assert_eq!(
            first.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
            vec!["c", "b"]
        );
        assert_eq!(
            store.diagnostic_runs(Some((1, "b")), 2).await.unwrap()[0].id,
            "a"
        );
        store.prune_diagnostic_runs(2).await.unwrap();
        assert_eq!(store.diagnostic_runs(None, 10).await.unwrap().len(), 3);
        store.prune_diagnostic_runs(3).await.unwrap();
        assert!(store.diagnostic_runs(None, 10).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn probes_expose_missing_migration_and_read_only_database() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        q("DELETE FROM _sqlx_migrations WHERE version=(SELECT MAX(version) FROM _sqlx_migrations)")
            .execute(&store.backend)
            .await
            .unwrap();
        let (actual, expected) = store.diagnostic_migrations().await.unwrap();
        assert_ne!(actual, expected as i64);
        q("PRAGMA query_only=ON")
            .execute(&store.backend)
            .await
            .unwrap();
        assert!(store.diagnostic_write_probe("readonly").await.is_err());
        assert!(
            store
                .get_setting("diagnostic-scratch:readonly")
                .await
                .unwrap()
                .is_none()
        );
        if let Backend::Sqlite(pool) = &store.backend {
            pool.close().await;
        }
        assert!(store.diagnostic_ping().await.is_err());
    }
}
