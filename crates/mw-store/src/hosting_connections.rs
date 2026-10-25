//! Deployment-local panel connections, atomically paired with sealed credentials.
use crate::{
    Store, StoreError,
    backend::{Row, q},
    seal,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostingConnectionRow {
    pub id: String,
    pub config_json: String,
    pub credential_set: bool,
    pub revision: String,
    pub updated_at: i64,
}
/// This type deliberately has neither Debug nor Serialize.
pub enum HostingCredentialChange {
    Keep,
    Set(String),
    Clear,
}
pub enum HostingMutation {
    Applied(HostingConnectionRow),
    Missing,
    Conflict,
}
fn row(r: &Row) -> HostingConnectionRow {
    HostingConnectionRow {
        id: r.get_string("id"),
        config_json: r.get_string("config_json"),
        credential_set: r.get_i64("credential_set") != 0,
        revision: r.get_string("revision"),
        updated_at: r.get_i64("updated_at"),
    }
}
const SELECT: &str = "SELECT id,config_json,revision,updated_at,CAST(CASE WHEN sealed_credential IS NULL THEN 0 ELSE 1 END AS BIGINT) AS credential_set FROM hosting_connection ORDER BY id";
const BY_ID: &str = "SELECT id,config_json,revision,updated_at,CAST(CASE WHEN sealed_credential IS NULL THEN 0 ELSE 1 END AS BIGINT) AS credential_set FROM hosting_connection WHERE id=?1";

impl Store {
    pub async fn hosting_connections(&self) -> Result<Vec<HostingConnectionRow>, StoreError> {
        Ok(q(SELECT)
            .fetch_all(&self.backend)
            .await?
            .iter()
            .map(row)
            .collect())
    }
    pub async fn hosting_connection(
        &self,
        id: &str,
    ) -> Result<Option<HostingConnectionRow>, StoreError> {
        Ok(q(BY_ID)
            .bind(id)
            .fetch_optional(&self.backend)
            .await?
            .as_ref()
            .map(row))
    }
    /// One database snapshot binds the credential to the exact saved authority.
    /// Callers must never combine a separately-read config with a newer secret.
    pub async fn hosting_connection_authority(
        &self,
        id: &str,
    ) -> Result<Option<(HostingConnectionRow, Option<String>)>, StoreError> {
        let Some(r)=q("SELECT *,CAST(CASE WHEN sealed_credential IS NULL THEN 0 ELSE 1 END AS BIGINT) AS credential_set FROM hosting_connection WHERE id=?1").bind(id).fetch_optional(&self.backend).await? else{return Ok(None)};
        let metadata = row(&r);
        let secret = if metadata.credential_set {
            Some(
                String::from_utf8(self.key.open(&r.get_blob("sealed_credential"))?)
                    .map_err(|_| seal::SealError::Decrypt)?,
            )
        } else {
            None
        };
        Ok(Some((metadata, secret)))
    }
    /// None is insert-only; Some(revision) is update-only and atomically checked.
    pub async fn put_hosting_connection(
        &self,
        id: &str,
        config_json: &str,
        expected: Option<&str>,
        credential: HostingCredentialChange,
        now: i64,
    ) -> Result<HostingMutation, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings(key,value) VALUES('ssh:authority-lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='ssh:authority-lock'")
            .execute_tx(&mut tx)
            .await?;
        q("INSERT INTO settings(key,value) VALUES('hosting:write-lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='hosting:write-lock'")
            .execute_tx(&mut tx)
            .await?;
        let old = q(BY_ID)
            .bind(id)
            .fetch_all_tx(&mut tx)
            .await?
            .first()
            .map(row);
        match (expected, old.as_ref()) {
            (None, Some(_)) => return Ok(HostingMutation::Conflict),
            (Some(_), None) => return Ok(HostingMutation::Missing),
            (Some(expected), Some(old)) if old.revision != expected => {
                return Ok(HostingMutation::Conflict);
            }
            _ => {}
        }
        let config: serde_json::Value = serde_json::from_str(config_json)
            .map_err(|_| StoreError::Corrupt("invalid hosting configuration".into()))?;
        if config.pointer("/connection/kind").and_then(|v| v.as_str()) == Some("ssh") {
            let Some(tunnel) = config
                .pointer("/connection/tunnelId")
                .and_then(|v| v.as_str())
            else {
                return Ok(HostingMutation::Conflict);
            };
            if q("SELECT id FROM ssh_tunnel WHERE id=?1 AND enabled=1 AND length(sealed_private_key)>0")
                .bind(tunnel).fetch_opt_scalar_string_tx(&mut tx).await?.is_none() {
                return Ok(HostingMutation::Conflict);
            }
        }
        let revision = seal::random_token();
        q("INSERT INTO hosting_connection(id,config_json,revision,updated_at) VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET config_json=excluded.config_json,revision=excluded.revision,updated_at=excluded.updated_at").bind(id).bind(config_json).bind(&revision).bind(now).execute_tx(&mut tx).await?;
        let credential_set = match credential {
            HostingCredentialChange::Keep => old.as_ref().is_some_and(|r| r.credential_set),
            HostingCredentialChange::Clear => {
                q("UPDATE hosting_connection SET sealed_credential=NULL WHERE id=?1")
                    .bind(id)
                    .execute_tx(&mut tx)
                    .await?;
                false
            }
            HostingCredentialChange::Set(value) => {
                let sealed = self.key.seal(value.as_bytes())?;
                q("UPDATE hosting_connection SET sealed_credential=?2 WHERE id=?1")
                    .bind(id)
                    .bind(sealed)
                    .execute_tx(&mut tx)
                    .await?;
                true
            }
        };
        tx.commit().await?;
        Ok(HostingMutation::Applied(HostingConnectionRow {
            id: id.into(),
            config_json: config_json.into(),
            credential_set,
            revision,
            updated_at: now,
        }))
    }
    pub async fn delete_hosting_connection(
        &self,
        id: &str,
        expected: &str,
    ) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings(key,value) VALUES('ssh:authority-lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='ssh:authority-lock'")
            .execute_tx(&mut tx)
            .await?;
        let deleted = q("DELETE FROM hosting_connection WHERE id=?1 AND revision=?2")
            .bind(id)
            .bind(expected)
            .execute_tx(&mut tx)
            .await?
            == 1;
        tx.commit().await?;
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn applied(result: HostingMutation) -> HostingConnectionRow {
        match result {
            HostingMutation::Applied(row) => row,
            _ => panic!("mutation refused"),
        }
    }
    async fn exercise(store: Store) {
        let first = applied(
            store
                .put_hosting_connection(
                    "panel",
                    r#"{"name":"one"}"#,
                    None,
                    HostingCredentialChange::Set("secret-one".into()),
                    1,
                )
                .await
                .unwrap(),
        );
        assert!(first.credential_set);
        assert!(!format!("{first:?}").contains("secret-one"));
        assert!(matches!(
            store
                .put_hosting_connection("panel", "{}", None, HostingCredentialChange::Clear, 2)
                .await
                .unwrap(),
            HostingMutation::Conflict
        ));
        let second = applied(
            store
                .put_hosting_connection(
                    "panel",
                    r#"{"name":"two"}"#,
                    Some(&first.revision),
                    HostingCredentialChange::Keep,
                    2,
                )
                .await
                .unwrap(),
        );
        let (metadata, secret) = store
            .hosting_connection_authority("panel")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(metadata, second);
        assert_eq!(secret.as_deref(), Some("secret-one"));
        assert!(matches!(
            store
                .put_hosting_connection(
                    "panel",
                    "{}",
                    Some(&first.revision),
                    HostingCredentialChange::Set("must-not-win".into()),
                    3
                )
                .await
                .unwrap(),
            HostingMutation::Conflict
        ));
        assert!(
            !store
                .delete_hosting_connection("panel", &first.revision)
                .await
                .unwrap()
        );
        let third = applied(
            store
                .put_hosting_connection(
                    "panel",
                    "{}",
                    Some(&second.revision),
                    HostingCredentialChange::Set("replacement".into()),
                    3,
                )
                .await
                .unwrap(),
        );
        assert_eq!(
            store
                .hosting_connection_authority("panel")
                .await
                .unwrap()
                .unwrap()
                .1
                .as_deref(),
            Some("replacement")
        );
        let fourth = applied(
            store
                .put_hosting_connection(
                    "panel",
                    "{}",
                    Some(&third.revision),
                    HostingCredentialChange::Clear,
                    4,
                )
                .await
                .unwrap(),
        );
        assert!(!fourth.credential_set);
        assert!(
            store
                .hosting_connection_authority("panel")
                .await
                .unwrap()
                .unwrap()
                .1
                .is_none()
        );
        assert!(
            store
                .delete_hosting_connection("panel", &fourth.revision)
                .await
                .unwrap()
        );
        assert!(store.hosting_connections().await.unwrap().is_empty());
        assert!(matches!(
            store
                .put_hosting_connection(
                    "panel",
                    "{}",
                    Some(&fourth.revision),
                    HostingCredentialChange::Keep,
                    5
                )
                .await
                .unwrap(),
            HostingMutation::Missing
        ));
    }
    #[tokio::test]
    async fn sealed_connection_cas_sqlite() {
        exercise(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn sealed_connection_cas_postgres() {
        let Ok(dsn) = std::env::var("MW_HOSTING_TEST_PG_DSN") else {
            return;
        };
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&dsn)
            .await
            .unwrap();
        let schema = format!("hosting_{}", rand::random::<u64>());
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
}
