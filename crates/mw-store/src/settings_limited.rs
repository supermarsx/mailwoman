//! Explicit bounded setting reads. Default get_setting semantics are unchanged.
use crate::{Store, StoreError, backend::q};
const LIMITED_SETTING_SQL: &str = "SELECT CASE WHEN octet_length(value)<=?2 THEN value ELSE NULL END AS value,CAST(CASE WHEN value IS NULL OR octet_length(value)>?2 THEN 1 ELSE 0 END AS BIGINT) AS oversized FROM settings WHERE key=?1";
impl Store {
    /// Reject oversized/corrupt values inside SQL before returning their text.
    /// The limit is UTF-8 bytes, not characters. Missing keys alone return None.
    pub async fn get_setting_limited(
        &self,
        key: &str,
        max_bytes: u64,
    ) -> Result<Option<String>, StoreError> {
        if max_bytes > 16 * 1024 * 1024 {
            return Err(StoreError::Corrupt(
                "setting read budget exceeds supported limit".into(),
            ));
        }
        let row = q(LIMITED_SETTING_SQL)
            .bind(key)
            .bind(max_bytes as i64)
            .fetch_optional(&self.backend)
            .await?;
        let Some(row) = row else { return Ok(None) };
        if row.get_i64("oversized") != 0 {
            return Err(StoreError::Corrupt("setting exceeds read budget".into()));
        }
        row.try_get_opt_string("value")?
            .map(Some)
            .ok_or_else(|| StoreError::Corrupt("setting value is null".into()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    async fn contract(store: Store) {
        let key = format!("limited-{}", uuid::Uuid::new_v4());
        store.set_setting(&key, "🦀").await.unwrap();
        assert!(store.get_setting_limited(&key, 3).await.is_err());
        assert_eq!(
            store.get_setting_limited(&key, 4).await.unwrap().as_deref(),
            Some("🦀")
        );
        assert_eq!(
            store.get_setting(&key).await.unwrap().as_deref(),
            Some("🦀")
        );
        assert_eq!(
            store
                .get_setting_limited("missing' OR 1=1 --", 0)
                .await
                .unwrap(),
            None
        );
        store.set_setting(&key, "").await.unwrap();
        assert_eq!(
            store.get_setting_limited(&key, 0).await.unwrap().as_deref(),
            Some("")
        );
        store
            .set_setting(&key, &"x".repeat(1024 * 1024))
            .await
            .unwrap();
        assert!(store.get_setting_limited(&key, 128).await.is_err());
        // Execute the exact production SQL: the database returns a NULL payload,
        // not a megabyte String which the caller subsequently rejects.
        let row = q(LIMITED_SETTING_SQL)
            .bind(&key)
            .bind(128i64)
            .fetch_one(&store.backend)
            .await
            .unwrap();
        assert_eq!(row.try_get_opt_string("value").unwrap(), None);
        assert_eq!(row.get_i64("oversized"), 1);
        assert_eq!(
            store.get_setting(&key).await.unwrap().unwrap().len(),
            1024 * 1024
        );

        assert!(
            store
                .get_setting_limited(&key, 17 * 1024 * 1024)
                .await
                .is_err()
        );
        q("DELETE FROM settings WHERE key=?1")
            .bind(&key)
            .execute(&store.backend)
            .await
            .unwrap();
        store.close().await;
    }
    #[tokio::test]
    async fn sqlite_limited_settings_use_exact_key_and_utf8_bytes() {
        contract(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn sqlite_limited_settings_reject_invalid_text_without_lossy_fallback() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        q("INSERT INTO settings(key,value) VALUES(?1,?2)")
            .bind("invalid-text")
            .bind(vec![255u8])
            .execute(&store.backend)
            .await
            .unwrap();
        assert!(store.get_setting_limited("invalid-text", 1).await.is_err());
        store.close().await;
    }
    #[tokio::test]
    async fn postgres_limited_settings_use_exact_key_and_utf8_bytes() {
        let Ok(dsn) = std::env::var("MW_TEST_PG") else {
            eprintln!("SKIP: MW_TEST_PG absent");
            return;
        };
        use sqlx::Connection;
        let schema = format!("limited_settings_{}", uuid::Uuid::new_v4().simple());
        let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&mut admin)
            .await
            .unwrap();
        let mut scoped = url::Url::parse(&dsn).unwrap();
        scoped
            .query_pairs_mut()
            .append_pair("options", &format!("-csearch_path={schema}"));
        contract(
            Store::open(scoped.as_str(), crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&mut admin)
            .await
            .unwrap();
    }
}
