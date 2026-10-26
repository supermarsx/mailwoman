//! Account-owned remote collection bindings. No credentials are copied here.
use crate::{
    Store, StoreError,
    backend::{Row, q},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NextcloudPimBinding {
    pub account_id: String,
    pub collection_id: String,
    pub kind: String,
    pub instance_id: String,
    pub link_revision: String,
    pub href: String,
    pub enabled: bool,
}
pub struct NextcloudPimApply<'a> {
    pub account_id: &'a str,
    pub instance_id: &'a str,
    pub link_revision: &'a str,
    pub kind: &'a str,
    pub href: &'a str,
    pub name: &'a str,
}
fn binding(row: &Row) -> NextcloudPimBinding {
    NextcloudPimBinding {
        account_id: row.get_string("account_id"),
        collection_id: row.get_string("collection_id"),
        kind: row.get_string("kind"),
        instance_id: row.get_string("instance_id"),
        link_revision: row.get_string("link_revision"),
        href: row.get_string("href"),
        enabled: row.get_i64("enabled") != 0,
    }
}
impl Store {
    /// Bounded keyset page of accounts with currently authorized collection bindings.
    pub async fn active_nextcloud_pim_accounts(&self, after:&str)->Result<Vec<String>,StoreError>{
        Ok(q("SELECT DISTINCT b.account_id FROM nextcloud_pim_binding b JOIN nextcloud_account_link l ON l.account_id=b.account_id AND l.instance_id=b.instance_id AND l.revision=b.link_revision JOIN nextcloud_instance i ON i.id=l.instance_id AND i.revision=l.instance_revision WHERE b.account_id>?1 AND b.enabled=1 AND i.enabled=1 ORDER BY b.account_id LIMIT 64")
           .bind(after).fetch_all(&self.backend).await?.iter().map(|row|row.get_string("account_id")).collect())
    }

    pub async fn nextcloud_pim_bindings(
        &self,
        account: &str,
    ) -> Result<Vec<NextcloudPimBinding>, StoreError> {
        Ok(q("SELECT * FROM nextcloud_pim_binding WHERE account_id=?1 ORDER BY collection_id LIMIT 129")
            .bind(account).fetch_all(&self.backend).await?.iter().map(binding).collect())
    }

    /// Metadata-only live authority join; never decrypts credentials for listing.
    pub async fn active_nextcloud_pim_bindings(
        &self,
        account: &str,
    ) -> Result<Vec<NextcloudPimBinding>, StoreError> {
        Ok(q("SELECT b.* FROM nextcloud_pim_binding b JOIN nextcloud_account_link l ON l.account_id=b.account_id AND l.instance_id=b.instance_id AND l.revision=b.link_revision JOIN nextcloud_instance i ON i.id=l.instance_id AND i.revision=l.instance_revision WHERE b.account_id=?1 AND b.enabled=1 AND i.enabled=1 ORDER BY b.collection_id LIMIT 129")
            .bind(account).fetch_all(&self.backend).await?.iter().map(binding).collect())
    }

    /// Explicit apply after discovery, serialized against authority edits and
    /// credential replacement. Creates the local collection and binding atomically.
    pub async fn apply_nextcloud_pim(
        &self,
        input: NextcloudPimApply<'_>,
    ) -> Result<Option<NextcloudPimBinding>, StoreError> {
        let NextcloudPimApply {
            account_id: account,
            instance_id: instance,
            link_revision,
            kind,
            href,
            name,
        } = input;
        if !matches!(kind, "calendar" | "tasks" | "contacts")
            || href.len() > 8192
            || name.len() > 512
        {
            return Ok(None);
        }
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings(key,value) VALUES('nextcloud:authority-lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='nextcloud:authority-lock'")
            .execute_tx(&mut tx)
            .await?;
        if q("UPDATE nextcloud_account_link SET revision=revision WHERE account_id=?1 AND instance_id=?2 AND revision=?3")
            .bind(account).bind(instance).bind(link_revision).execute_tx(&mut tx).await? == 0 { return Ok(None); }
        if q("SELECT l.revision FROM nextcloud_account_link l JOIN nextcloud_instance i ON i.id=l.instance_id AND i.revision=l.instance_revision WHERE l.account_id=?1 AND l.instance_id=?2 AND i.enabled=1")
            .bind(account).bind(instance).fetch_all_tx(&mut tx).await?.is_empty() { return Ok(None); }
        let rows = q("SELECT * FROM nextcloud_pim_binding WHERE account_id=?1 ORDER BY collection_id LIMIT 129")
            .bind(account).fetch_all_tx(&mut tx).await?;
        let prior = rows.iter().find(|row| {
            row.get_string("instance_id") == instance
                && row.get_string("kind") == kind
                && row.get_string("href") == href
        });
        if prior.is_none() && rows.len() >= 128 {
            return Ok(None);
        }
        let id = prior
            .map(|row| row.get_string("collection_id"))
            .unwrap_or_else(|| format!("cloud_{}", crate::seal::random_token()));
        if kind == "contacts" {
            q("INSERT INTO address_books(id,account_id,name,is_default,carddav_url,sync_token,ctag) VALUES(?1,?2,?3,0,?4,NULL,NULL) ON CONFLICT(id) DO UPDATE SET name=excluded.name,carddav_url=excluded.carddav_url")
                .bind(&id).bind(account).bind(name).bind(href).execute_tx(&mut tx).await?;
        } else {
            let component = if kind == "tasks" { "VTODO" } else { "VEVENT" };
            q("INSERT INTO calendars(id,account_id,name,color,sort_order,is_visible,role,caldav_url,sync_token,ctag,is_overlay,component) VALUES(?1,?2,?3,'#3366ff',0,1,NULL,?4,NULL,NULL,0,?5) ON CONFLICT(id) DO UPDATE SET name=excluded.name,caldav_url=excluded.caldav_url,component=excluded.component")
                .bind(&id).bind(account).bind(name).bind(href).bind(component).execute_tx(&mut tx).await?;
        }
        q("INSERT INTO nextcloud_pim_binding(account_id,collection_id,kind,instance_id,link_revision,href,enabled) VALUES(?1,?2,?3,?4,?5,?6,1) ON CONFLICT(account_id,collection_id) DO UPDATE SET link_revision=excluded.link_revision,enabled=1")
            .bind(account).bind(&id).bind(kind).bind(instance).bind(link_revision).bind(href).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(Some(NextcloudPimBinding {
            account_id: account.into(),
            collection_id: id,
            kind: kind.into(),
            instance_id: instance.into(),
            link_revision: link_revision.into(),
            href: href.into(),
            enabled: true,
        }))
    }

    /// Disconnect keeps cached data; future remote writes are refused.
    pub async fn disconnect_nextcloud_pim(
        &self,
        account: &str,
        collection: &str,
    ) -> Result<bool, StoreError> {
        Ok(
            q("UPDATE nextcloud_pim_binding SET enabled=0 WHERE account_id=?1 AND collection_id=?2")
                .bind(account)
                .bind(collection)
                .execute(&self.backend)
                .await? > 0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ServerKey,
        nextcloud_links::{NextcloudInstance, NextcloudLink},
    };
    #[tokio::test]
    async fn sqlite_bindings_are_atomic_stable_and_account_owned() {
        exercise(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
    }
    #[tokio::test]
    async fn postgres_bindings_are_atomic_stable_and_account_owned() {
        let Ok(dsn) = std::env::var("MW_TEST_PG") else {
            return;
        };
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&dsn)
            .await
            .unwrap();
        let schema = format!("nextcloud_pim_{}", rand::random::<u64>());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&pool)
            .await
            .unwrap();
        let sep = if dsn.contains('?') { '&' } else { '?' };
        let scoped = format!("{dsn}{sep}options=-c%20search_path%3D{schema}");
        let task = tokio::spawn(async move {
            exercise(
                Store::open_postgres(&scoped, ServerKey::generate())
                    .await
                    .unwrap(),
            )
            .await
        })
        .await;
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&pool)
            .await
            .unwrap();
        task.unwrap();
    }
    async fn exercise(store: Store) {
        let account = store
            .create_account(
                &crate::NewAccount {
                    kind: crate::AccountKind::Imap,
                    host: "mail.test",
                    port: 993,
                    tls: "implicit",
                    username: "a@test",
                    sync_policy_json: "{}",
                },
                &crate::Credentials {
                    username: "a@test".into(),
                    password: "fixture".into(),
                },
            )
            .await
            .unwrap();
        let instance = store
            .put_nextcloud_instance(
                &NextcloudInstance {
                    id: "cloud".into(),
                    name: "Cloud".into(),
                    base_url: "https://cloud.test/".into(),
                    ca_pem: String::new(),
                    enabled: true,
                    revision: String::new(),
                },
                None,
            )
            .await
            .unwrap()
            .unwrap();
        let link = store
            .put_verified_nextcloud_link(
                &NextcloudLink {
                    account_id: account.clone(),
                    instance_id: instance.id.clone(),
                    instance_revision: instance.revision.clone(),
                    username: "remote".into(),
                    revision: String::new(),
                },
                "fixture-password",
                None,
            )
            .await
            .unwrap()
            .unwrap();
        let input = || NextcloudPimApply {
            account_id: &account,
            instance_id: "cloud",
            link_revision: &link.revision,
            kind: "calendar",
            href: "https://cloud.test/remote.php/dav/calendars/remote/work/",
            name: "Work",
        };
        let first = store.apply_nextcloud_pim(input()).await.unwrap().unwrap();
        let same = store.apply_nextcloud_pim(input()).await.unwrap().unwrap();
        assert_eq!(first.collection_id, same.collection_id);
        assert_eq!(store.active_nextcloud_pim_accounts("").await.unwrap(), vec![account.clone()]);
        assert!(store.active_nextcloud_pim_accounts(&account).await.unwrap().is_empty());
        assert_eq!(
            store
                .active_nextcloud_pim_bindings(&account)
                .await
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            store
                .get_calendar(&first.collection_id)
                .await
                .unwrap()
                .unwrap()
                .account_id,
            account
        );
        assert!(
            store
                .nextcloud_pim_bindings("foreign")
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            !store
                .disconnect_nextcloud_pim("foreign", &first.collection_id)
                .await
                .unwrap()
        );
        assert!(
            store
                .disconnect_nextcloud_pim(&account, &first.collection_id)
                .await
                .unwrap()
        );
        assert!(!store.nextcloud_pim_bindings(&account).await.unwrap()[0].enabled);
        assert!(store.active_nextcloud_pim_accounts("").await.unwrap().is_empty());
        assert!(
            store
                .active_nextcloud_pim_bindings(&account)
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            store
                .get_calendar(&first.collection_id)
                .await
                .unwrap()
                .is_some(),
            "disconnect preserves cached objects"
        );
        assert!(
            store
                .apply_nextcloud_pim(NextcloudPimApply {
                    link_revision: "stale",
                    ..input()
                })
                .await
                .unwrap()
                .is_none()
        );
        let disabled = NextcloudInstance {
            enabled: false,
            ..instance.clone()
        };
        store
            .put_nextcloud_instance(&disabled, Some(&instance.revision))
            .await
            .unwrap()
            .unwrap();
        assert!(store.apply_nextcloud_pim(input()).await.unwrap().is_none());
    }
}
