//! Nextcloud authorities and account-owned credentials. No secrets in metadata.
use crate::{
    Store, StoreError,
    backend::{Row, q},
    seal,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NextcloudInstance {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub ca_pem: String,
    pub enabled: bool,
    pub revision: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NextcloudLink {
    pub account_id: String,
    pub instance_id: String,
    pub instance_revision: String,
    pub username: String,
    pub revision: String,
}

fn instance(row: &Row) -> NextcloudInstance {
    NextcloudInstance {
        id: row.get_string("id"),
        name: row.get_string("name"),
        base_url: row.get_string("base_url"),
        ca_pem: row.get_string("ca_pem"),
        enabled: row.get_i64("enabled") != 0,
        revision: row.get_string("revision"),
    }
}
fn link(row: &Row) -> NextcloudLink {
    NextcloudLink {
        account_id: row.get_string("account_id"),
        instance_id: row.get_string("instance_id"),
        instance_revision: row.get_string("instance_revision"),
        username: row.get_string("username"),
        revision: row.get_string("revision"),
    }
}

impl Store {
    pub async fn nextcloud_instances(&self) -> Result<Vec<NextcloudInstance>, StoreError> {
        Ok(
            q("SELECT * FROM nextcloud_instance ORDER BY name,id LIMIT 65")
                .fetch_all(&self.backend)
                .await?
                .iter()
                .map(instance)
                .collect(),
        )
    }
    pub async fn nextcloud_instance(
        &self,
        id: &str,
    ) -> Result<Option<NextcloudInstance>, StoreError> {
        Ok(q("SELECT * FROM nextcloud_instance WHERE id=?1")
            .bind(id)
            .fetch_optional(&self.backend)
            .await?
            .as_ref()
            .map(instance))
    }
    /// CAS under the same lock used by link creation. A verified link cannot be
    /// persisted against an authority changed while the network check ran.
    pub async fn put_nextcloud_instance(
        &self,
        desired: &NextcloudInstance,
        expected: Option<&str>,
    ) -> Result<Option<NextcloudInstance>, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings(key,value) VALUES('nextcloud:authority-lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='nextcloud:authority-lock'")
            .execute_tx(&mut tx)
            .await?;
        let rows = q("SELECT * FROM nextcloud_instance ORDER BY id")
            .fetch_all_tx(&mut tx)
            .await?;
        let old = rows.iter().find(|row| row.get_string("id") == desired.id);
        if old.map(|row| row.get_string("revision")).as_deref() != expected
            || (old.is_none() && rows.len() >= 64)
        {
            return Ok(None);
        }
        let mut saved = desired.clone();
        saved.revision = seal::random_token();
        q("INSERT INTO nextcloud_instance(id,name,base_url,enabled,revision,ca_pem) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET name=excluded.name,base_url=excluded.base_url,enabled=excluded.enabled,revision=excluded.revision,ca_pem=excluded.ca_pem")
            .bind(&saved.id).bind(&saved.name).bind(&saved.base_url).bind(i64::from(saved.enabled)).bind(&saved.revision).bind(&saved.ca_pem).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(Some(saved))
    }
    pub async fn nextcloud_links(&self, account: &str) -> Result<Vec<NextcloudLink>, StoreError> {
        Ok(q("SELECT * FROM nextcloud_account_link WHERE account_id=?1 ORDER BY instance_id LIMIT 65").bind(account).fetch_all(&self.backend).await?.iter().map(link).collect())
    }
    /// Call only after authentication succeeds. Failed verification never replaces
    /// a working link. A revision change requires explicit verification again.
    pub async fn put_verified_nextcloud_link(
        &self,
        desired: &NextcloudLink,
        password: &str,
        expected: Option<&str>,
    ) -> Result<Option<NextcloudLink>, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings(key,value) VALUES('nextcloud:authority-lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='nextcloud:authority-lock'")
            .execute_tx(&mut tx)
            .await?;
        let authorities =
            q("SELECT * FROM nextcloud_instance WHERE id=?1 AND enabled=1 AND revision=?2")
                .bind(&desired.instance_id)
                .bind(&desired.instance_revision)
                .fetch_all_tx(&mut tx)
                .await?;
        if authorities.is_empty() {
            return Ok(None);
        }
        let old =
            q("SELECT revision FROM nextcloud_account_link WHERE account_id=?1 AND instance_id=?2")
                .bind(&desired.account_id)
                .bind(&desired.instance_id)
                .fetch_all_tx(&mut tx)
                .await?;
        if old.first().map(|row| row.get_string("revision")).as_deref() != expected {
            return Ok(None);
        }
        let mut saved = desired.clone();
        saved.revision = seal::random_token();
        let sealed = self.key.seal_bound(
            password.as_bytes(),
            "nextcloud-app-password",
            &[
                &saved.account_id,
                &saved.instance_id,
                &saved.instance_revision,
                &saved.username,
            ],
        )?;
        q("INSERT INTO nextcloud_account_link(account_id,instance_id,instance_revision,username,sealed_password,revision) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(account_id,instance_id) DO UPDATE SET instance_revision=excluded.instance_revision,username=excluded.username,sealed_password=excluded.sealed_password,revision=excluded.revision")
            .bind(&saved.account_id).bind(&saved.instance_id).bind(&saved.instance_revision).bind(&saved.username).bind(sealed).bind(&saved.revision).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(Some(saved))
    }
    /// Joined snapshot: disabled/repointed instances cannot release credentials.
    pub async fn nextcloud_authority(
        &self,
        account: &str,
        id: &str,
    ) -> Result<Option<(NextcloudInstance, NextcloudLink, String)>, StoreError> {
        let Some(row) = q("SELECT l.*,i.id,i.name,i.base_url,i.enabled,i.ca_pem FROM nextcloud_account_link l JOIN nextcloud_instance i ON i.id=l.instance_id AND i.revision=l.instance_revision WHERE l.account_id=?1 AND l.instance_id=?2 AND i.enabled=1")
            .bind(account).bind(id).fetch_optional(&self.backend).await? else { return Ok(None); };
        let linked = link(&row);
        let authority = NextcloudInstance {
            id: id.into(),
            name: row.get_string("name"),
            base_url: row.get_string("base_url"),
            ca_pem: row.get_string("ca_pem"),
            enabled: true,
            revision: linked.instance_revision.clone(),
        };
        let password = String::from_utf8(self.key.open_bound(
            &row.get_blob("sealed_password"),
            "nextcloud-app-password",
            &[account, id, &linked.instance_revision, &linked.username],
        )?)
        .map_err(|_| seal::SealError::Decrypt)?;
        Ok(Some((authority, linked, password)))
    }
    pub async fn delete_nextcloud_link(
        &self,
        account: &str,
        id: &str,
        revision: &str,
    ) -> Result<bool, StoreError> {
        Ok(q("DELETE FROM nextcloud_account_link WHERE account_id=?1 AND instance_id=?2 AND revision=?3").bind(account).bind(id).bind(revision).execute(&self.backend).await? > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn account_isolation_authority_change_and_stale_verification() {
        exercise(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn postgres_account_isolation_and_authority_cas() {
        let Ok(dsn) = std::env::var("MW_TEST_PG") else {
            return;
        };
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&dsn)
            .await
            .unwrap();
        let schema = format!("nextcloud_links_{}", rand::random::<u64>());
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
        let desired = NextcloudInstance {
            id: "cloud".into(),
            name: "Cloud".into(),
            base_url: "https://cloud.example/".into(),
            ca_pem: String::new(),
            enabled: true,
            revision: String::new(),
        };
        let first = store
            .put_nextcloud_instance(&desired, None)
            .await
            .unwrap()
            .unwrap();
        let link = NextcloudLink {
            account_id: "alice".into(),
            instance_id: first.id.clone(),
            instance_revision: first.revision.clone(),
            username: "remote-alice".into(),
            revision: String::new(),
        };
        let saved = store
            .put_verified_nextcloud_link(&link, "private-app-password", None)
            .await
            .unwrap()
            .unwrap();
        assert!(
            store
                .nextcloud_authority("bob", "cloud")
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(
            store
                .nextcloud_authority("alice", "cloud")
                .await
                .unwrap()
                .unwrap()
                .2,
            "private-app-password"
        );
        assert!(
            store
                .put_verified_nextcloud_link(&link, "wrong-overwrite", None)
                .await
                .unwrap()
                .is_none()
        );
        let mut updated = first.clone();
        updated.ca_pem = "replacement CA certificate".into();
        let changed = store
            .put_nextcloud_instance(&updated, Some(&first.revision))
            .await
            .unwrap()
            .unwrap();
        assert_ne!(changed.revision, first.revision);
        assert_eq!(changed.ca_pem, updated.ca_pem);
        assert_eq!(
            store
                .nextcloud_instance("cloud")
                .await
                .unwrap()
                .unwrap()
                .ca_pem,
            updated.ca_pem
        );
        assert!(
            store
                .nextcloud_authority("alice", "cloud")
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .put_verified_nextcloud_link(
                    &link,
                    "stale-verified-password",
                    Some(&saved.revision)
                )
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            !store
                .delete_nextcloud_link("bob", "cloud", &saved.revision)
                .await
                .unwrap()
        );
        assert!(
            store
                .delete_nextcloud_link("alice", "cloud", &saved.revision)
                .await
                .unwrap()
        );
    }
}
