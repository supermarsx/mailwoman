//! Atomic LDAP configuration with sealed, explicitly managed service credentials.
use crate::backend::{Row, q};
use crate::{DirectoryConfigRow, Store, StoreError, seal};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryAdminRow {
    pub config: DirectoryConfigRow,
    pub connection_mode: String,
    pub allow_private: bool,
    pub password_set: bool,
    pub revision: String,
}
/// Deliberately no Debug implementation: the replacement may contain a credential.
pub enum DirectoryPasswordChange {
    Keep,
    Set(String),
    Clear,
}
pub struct DirectoryAdminWrite {
    pub row: DirectoryAdminRow,
    pub password: DirectoryPasswordChange,
}
fn read(row: &Row) -> DirectoryAdminRow {
    DirectoryAdminRow {
        config: DirectoryConfigRow {
            id: row.get_string("id"),
            url: row.get_string("url"),
            base_dn: row.get_string("base_dn"),
            bind_dn: row.get_opt_string("bind_dn"),
            tls: row.get_string("tls"),
            priority: row.get_i64("priority"),
            attr_map_json: row.get_string("attr_map"),
            enabled: row.get_i64("enabled") != 0,
        },
        connection_mode: row.get_string("connection_mode"),
        allow_private: row.get_i64("allow_private") != 0,
        password_set: row.get_i64("password_set") != 0,
        revision: row.get_string("revision"),
    }
}
const SELECT: &str = "SELECT *, CAST(CASE WHEN sealed_bind_password IS NULL THEN 0 ELSE 1 END AS BIGINT) AS password_set FROM directory_config ORDER BY priority,id";
impl Store {
    pub async fn directory_admin_rows(&self) -> Result<Vec<DirectoryAdminRow>, StoreError> {
        Ok(q(SELECT)
            .fetch_all(&self.backend)
            .await?
            .iter()
            .map(read)
            .collect())
    }
    pub async fn directory_bind_password(&self, id: &str) -> Result<Option<String>, StoreError> {
        let Some(row) = q("SELECT sealed_bind_password FROM directory_config WHERE id=?1 AND sealed_bind_password IS NOT NULL")
            .bind(id).fetch_optional(&self.backend).await? else { return Ok(None); };
        let bytes = self.key.open(&row.get_blob("sealed_bind_password"))?;
        String::from_utf8(bytes)
            .map(Some)
            .map_err(|_| seal::SealError::Decrypt.into())
    }
    /// Replace the collection under a database write lock, refusing stale snapshots.
    /// Every successful edit gets a fresh revision, including password-only edits.
    pub async fn replace_directory_admin(
        &self,
        previous: &[DirectoryAdminRow],
        writes: &[DirectoryAdminWrite],
    ) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings (key,value) VALUES ('directory:write-lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='directory:write-lock'")
            .execute_tx(&mut tx)
            .await?;
        let current: Vec<_> = q(SELECT)
            .fetch_all_tx(&mut tx)
            .await?
            .iter()
            .map(read)
            .collect();
        if current != previous {
            return Ok(false);
        }
        // Keep the sealed bytes in the database when a caller retains a credential.
        for write in writes {
            let row = &write.row;
            let cfg = &row.config;
            q("INSERT INTO directory_config (id,url,base_dn,bind_dn,tls,priority,attr_map,enabled,connection_mode,allow_private,revision) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11) ON CONFLICT(id) DO UPDATE SET url=excluded.url,base_dn=excluded.base_dn,bind_dn=excluded.bind_dn,tls=excluded.tls,priority=excluded.priority,attr_map=excluded.attr_map,enabled=excluded.enabled,connection_mode=excluded.connection_mode,allow_private=excluded.allow_private,revision=excluded.revision")
                .bind(&cfg.id).bind(&cfg.url).bind(&cfg.base_dn).bind(cfg.bind_dn.clone())
                .bind(&cfg.tls).bind(cfg.priority).bind(&cfg.attr_map_json).bind(i64::from(cfg.enabled))
                .bind(&row.connection_mode).bind(i64::from(row.allow_private)).bind(seal::random_token())
                .execute_tx(&mut tx).await?;
            match &write.password {
                DirectoryPasswordChange::Keep => {}
                DirectoryPasswordChange::Clear => {
                    q("UPDATE directory_config SET sealed_bind_password=NULL WHERE id=?1")
                        .bind(&cfg.id)
                        .execute_tx(&mut tx)
                        .await?;
                }
                DirectoryPasswordChange::Set(password) => {
                    let sealed = self.key.seal(password.as_bytes())?;
                    q("UPDATE directory_config SET sealed_bind_password=?2 WHERE id=?1")
                        .bind(&cfg.id)
                        .bind(sealed)
                        .execute_tx(&mut tx)
                        .await?;
                }
            }
        }
        for old in current {
            if !writes.iter().any(|w| w.row.config.id == old.config.id) {
                q("DELETE FROM directory_config WHERE id=?1")
                    .bind(&old.config.id)
                    .execute_tx(&mut tx)
                    .await?;
            }
        }
        tx.commit().await?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn write(password: DirectoryPasswordChange) -> DirectoryAdminWrite {
        DirectoryAdminWrite {
            row: DirectoryAdminRow {
                config: DirectoryConfigRow {
                    id: "primary".into(),
                    url: "ldaps://directory.example/".into(),
                    base_dn: "dc=example".into(),
                    bind_dn: Some("cn=service".into()),
                    tls: "ldaps".into(),
                    priority: 0,
                    attr_map_json: "{}".into(),
                    enabled: true,
                },
                connection_mode: "direct".into(),
                allow_private: false,
                password_set: true,
                revision: String::new(),
            },
            password,
        }
    }
    #[tokio::test]
    async fn sealed_password_keep_replace_clear_and_stale_snapshot() {
        exercise(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn postgres_sealed_directory_cas() {
        let Ok(dsn) = std::env::var("MW_DIRECTORY_TEST_PG_DSN") else {
            eprintln!("SKIP: MW_DIRECTORY_TEST_PG_DSN unset");
            return;
        };
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&dsn)
            .await
            .unwrap();
        let schema = format!("directory_admin_{}", rand::random::<u64>());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&pool)
            .await
            .unwrap();
        let sep = if dsn.contains('?') { '&' } else { '?' };
        let scoped = format!("{dsn}{sep}options=-c%20search_path%3D{schema}");
        let result = tokio::spawn(async move {
            exercise(
                Store::open_postgres(&scoped, crate::ServerKey::generate())
                    .await
                    .unwrap(),
            )
            .await;
        })
        .await;
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&pool)
            .await
            .unwrap();
        result.unwrap();
    }
    async fn exercise(store: Store) {
        assert!(
            store
                .replace_directory_admin(
                    &[],
                    &[write(DirectoryPasswordChange::Set("secret-one".into()))]
                )
                .await
                .unwrap()
        );
        let first = store.directory_admin_rows().await.unwrap();
        assert!(first[0].password_set);
        assert_eq!(
            store
                .directory_bind_password("primary")
                .await
                .unwrap()
                .as_deref(),
            Some("secret-one")
        );
        let raw = q("SELECT sealed_bind_password FROM directory_config WHERE id='primary'")
            .fetch_optional(&store.backend)
            .await
            .unwrap()
            .unwrap()
            .get_blob("sealed_bind_password");
        assert!(!raw.windows(10).any(|b| b == b"secret-one"));
        assert!(
            store
                .replace_directory_admin(&first, &[write(DirectoryPasswordChange::Keep)])
                .await
                .unwrap()
        );
        assert!(
            !store.replace_directory_admin(&first, &[]).await.unwrap(),
            "stale removal must not erase current configuration"
        );
        let second = store.directory_admin_rows().await.unwrap();
        assert_ne!(first[0].revision, second[0].revision);
        assert_eq!(
            store
                .directory_bind_password("primary")
                .await
                .unwrap()
                .as_deref(),
            Some("secret-one")
        );
        assert!(
            store
                .replace_directory_admin(
                    &second,
                    &[write(DirectoryPasswordChange::Set("secret-two".into()))]
                )
                .await
                .unwrap()
        );
        assert_eq!(
            store
                .directory_bind_password("primary")
                .await
                .unwrap()
                .as_deref(),
            Some("secret-two")
        );
        let third = store.directory_admin_rows().await.unwrap();
        assert!(
            store
                .replace_directory_admin(&third, &[write(DirectoryPasswordChange::Clear)])
                .await
                .unwrap()
        );
        assert!(
            store
                .directory_bind_password("primary")
                .await
                .unwrap()
                .is_none()
        );
        let fourth = store.directory_admin_rows().await.unwrap();
        assert!(!fourth[0].password_set);
        assert!(store.replace_directory_admin(&fourth, &[]).await.unwrap());
        assert!(store.directory_admin_rows().await.unwrap().is_empty());
    }
}
