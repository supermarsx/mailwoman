//! Atomic cross-key admission configuration. A client slug and console path
//! cannot both validate against an older value and then collide.
use crate::{Store, StoreError, backend::q};
impl Store {
    pub async fn compare_entry_setting(
        &self,
        key: &str,
        expected: Option<&str>,
        replacement: &str,
        other_expected: Option<&str>,
    ) -> Result<bool, StoreError> {
        let other = match key {
            "security.clientAccess.v1" => "admin.consolePath",
            "admin.consolePath" => "security.clientAccess.v1",
            _ => {
                return Err(StoreError::Corrupt(
                    "invalid entry configuration key".into(),
                ));
            }
        };
        if replacement.len() > 8192 {
            return Err(StoreError::Corrupt(
                "entry configuration exceeds limit".into(),
            ));
        }
        let mut tx = self.backend.begin().await?;
        // Reuse the existing durable settings mutation lock on SQLite and PG.
        crate::branding::lock(&mut tx).await?;
        for (read_key, wanted) in [(key, expected), (other, other_expected)] {
            let rows = q("SELECT value FROM settings WHERE key=?1")
                .bind(read_key)
                .fetch_all_tx(&mut tx)
                .await?;
            let actual = rows.first().map(|r| r.get_string("value"));
            if actual.as_deref() != wanted {
                return Ok(false);
            }
        }
        q("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value").bind(key).bind(replacement).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(true)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn concurrent_cross_key_writes_cannot_both_use_stale_validation() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        let (a, b) = tokio::join!(
            store.compare_entry_setting("security.clientAccess.v1", None, "client", None),
            store.compare_entry_setting("admin.consolePath", None, "console", None)
        );
        assert_ne!(a.unwrap(), b.unwrap());
        assert!(
            store
                .compare_entry_setting("unrelated", None, "bad", None)
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn postgres_cross_replica_entry_writes_share_the_durable_lock() {
        let Ok(dsn) = std::env::var("MW_TEST_PG").or_else(|_| std::env::var("DATABASE_URL_PG"))
        else {
            assert!(
                std::env::var_os("MW_REQUIRE_LIVE").is_none(),
                "PostgreSQL is required"
            );
            eprintln!("SKIP PostgreSQL entry-setting CAS: no test DSN");
            return;
        };
        let schema = format!("entry_{}", uuid::Uuid::new_v4().simple());
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&dsn)
            .await
            .unwrap();
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&pool)
            .await
            .unwrap();
        let scoped = format!(
            "{dsn}{}options=-csearch_path%3D{schema}",
            if dsn.contains('?') { "&" } else { "?" }
        );
        let key = crate::ServerKey::generate();
        let first = Store::open(&scoped, key.clone()).await.unwrap();
        let second = Store::open(&scoped, key).await.unwrap();
        let (a, b) = tokio::join!(
            first.compare_entry_setting("security.clientAccess.v1", None, "client", None),
            second.compare_entry_setting("admin.consolePath", None, "console", None)
        );
        assert_ne!(a.unwrap(), b.unwrap());
        drop(first);
        drop(second);
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;
    }
}
