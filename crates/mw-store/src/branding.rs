//! Global branding shares the registry's text authority and serializes all writes.
use crate::{
    Store, StoreError,
    backend::{Tx, q},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrandImages {
    pub logo_id: Option<String>,
    pub icon_master_id: Option<String>,
    pub login_background_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct BrandingSnapshot {
    pub images: BrandImages,
    pub name_json: Option<String>,
    pub notice_json: Option<String>,
}

pub struct BrandAsset {
    pub id: String,
    pub kind: String,
    pub data: Vec<u8>,
    pub width: i64,
    pub height: i64,
}

pub struct BrandingUpdate<'a> {
    pub images: &'a BrandImages,
    pub name_json: &'a str,
    pub notice_json: &'a str,
    pub actor: &'a str,
    pub now: i64,
}

pub(crate) async fn lock(tx: &mut Tx) -> Result<(), StoreError> {
    q("INSERT INTO branding(scope,config_json,updated_by,updated_at) VALUES('global','{}','',0) ON CONFLICT(scope) DO NOTHING").execute_tx(tx).await?;
    q("UPDATE branding SET updated_at=updated_at WHERE scope='global'")
        .execute_tx(tx)
        .await?;
    Ok(())
}

async fn snapshot(tx: &mut Tx) -> Result<BrandingSnapshot, StoreError> {
    let rows = q("SELECT config_json FROM branding WHERE scope='global'")
        .fetch_all_tx(tx)
        .await?;
    let images = match rows.first() {
        Some(row) => serde_json::from_str(&row.get_string("config_json"))
            .map_err(|_| StoreError::Corrupt("global branding configuration".into()))?,
        None => BrandImages::default(),
    };
    let rows = q("SELECT key,global_value FROM setting_policy WHERE key IN ('appearance.brandName','login.notice')").fetch_all_tx(tx).await?;
    let value = |key: &str| {
        rows.iter()
            .find(|r| r.get_string("key") == key)
            .and_then(|r| r.get_opt_string("global_value"))
    };
    Ok(BrandingSnapshot {
        images,
        name_json: value("appearance.brandName"),
        notice_json: value("login.notice"),
    })
}

impl Store {
    pub async fn branding_snapshot(&self) -> Result<BrandingSnapshot, StoreError> {
        let mut tx = self.backend.begin().await?;
        lock(&mut tx).await?;
        let result = snapshot(&mut tx).await?;
        tx.commit().await?;
        Ok(result)
    }

    /// Compare the complete prior value under the same lock used by registry writes.
    /// Invalid asset references are refused before changing any text or image.
    pub async fn put_branding(
        &self,
        expected: &BrandingSnapshot,
        update: BrandingUpdate<'_>,
    ) -> Result<bool, StoreError> {
        let BrandingUpdate {
            images,
            name_json,
            notice_json,
            actor,
            now,
        } = update;
        let mut tx = self.backend.begin().await?;
        lock(&mut tx).await?;
        if &snapshot(&mut tx).await? != expected {
            return Ok(false);
        }
        for (id, kind) in [
            (&images.logo_id, "logo"),
            (&images.icon_master_id, "icon_master"),
            (&images.login_background_id, "background"),
        ] {
            if let Some(id) = id {
                let rows = q("SELECT id FROM brand_asset WHERE id=?1 AND scope='global' AND owner_account IS NULL AND kind=?2").bind(id).bind(kind).fetch_all_tx(&mut tx).await?;
                if rows.is_empty() {
                    return Ok(false);
                }
            }
        }
        for (key, value) in [
            ("appearance.brandName", name_json),
            ("login.notice", notice_json),
        ] {
            q("INSERT INTO setting_policy(key,global_value,allow_user_override,global_locked,updated_by,updated_at) VALUES(?1,?2,0,0,?3,?4) ON CONFLICT(key) DO UPDATE SET global_value=excluded.global_value,updated_by=excluded.updated_by,updated_at=excluded.updated_at")
                .bind(key).bind(value).bind(actor).bind(now).execute_tx(&mut tx).await?;
        }
        q("UPDATE branding SET config_json=?1,updated_by=?2,updated_at=?3 WHERE scope='global'")
            .bind(serde_json::to_string(images).expect("serializable images"))
            .bind(actor)
            .bind(now)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(true)
    }

    pub async fn put_brand_asset(
        &self,
        asset: &BrandAsset,
        actor: &str,
        now: i64,
    ) -> Result<(), StoreError> {
        q("INSERT INTO brand_asset(id,scope,kind,owner_account,sha256,content_type,bytes,width,height,data,created_by,created_at) VALUES(?1,'global',?2,NULL,?1,'image/png',?3,?4,?5,?6,?7,?8) ON CONFLICT(id) DO NOTHING")
            .bind(&asset.id).bind(&asset.kind).bind(asset.data.len() as i64).bind(asset.width).bind(asset.height).bind(&asset.data).bind(actor).bind(now).execute(&self.backend).await?;
        Ok(())
    }

    /// Uploads remain private until referenced by the published global configuration.
    pub async fn published_brand_asset(&self, id: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let mut tx = self.backend.begin().await?;
        lock(&mut tx).await?;
        let images = snapshot(&mut tx).await?.images;
        if ![
            images.logo_id,
            images.icon_master_id,
            images.login_background_id,
        ]
        .iter()
        .any(|v| v.as_deref() == Some(id))
        {
            return Ok(None);
        }
        let rows = q(
            "SELECT data FROM brand_asset WHERE id=?1 AND scope='global' AND owner_account IS NULL",
        )
        .bind(id)
        .fetch_all_tx(&mut tx)
        .await?;
        let result = rows.first().map(|r| r.get_blob("data"));
        tx.commit().await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_policy::SettingPolicyRow;
    async fn exercise(store: Store) {
        let initial = store.branding_snapshot().await.unwrap();
        let id = format!("brand-test-{:032x}", rand::random::<u128>());
        store
            .put_brand_asset(
                &BrandAsset {
                    id: id.clone(),
                    kind: "logo".into(),
                    data: vec![1, 2, 3],
                    width: 1,
                    height: 1,
                },
                "admin",
                1,
            )
            .await
            .unwrap();
        assert!(store.published_brand_asset(&id).await.unwrap().is_none());
        let wrong = BrandImages {
            icon_master_id: Some(id.clone()),
            ..Default::default()
        };
        assert!(
            !store
                .put_branding(
                    &initial,
                    BrandingUpdate {
                        images: &wrong,
                        name_json: "\"Bad\"",
                        notice_json: "null",
                        actor: "admin",
                        now: 2
                    }
                )
                .await
                .unwrap()
        );
        assert_eq!(store.branding_snapshot().await.unwrap(), initial);
        let image = BrandImages {
            logo_id: Some(id.clone()),
            ..Default::default()
        };
        let save = || BrandingUpdate {
            images: &image,
            name_json: "\"Acme\"",
            notice_json: "\"Welcome\"",
            actor: "admin",
            now: 3,
        };
        let (a, b) = tokio::join!(
            store.put_branding(&initial, save()),
            store.put_branding(&initial, save())
        );
        assert_ne!(a.unwrap(), b.unwrap(), "one compare-and-swap writer wins");
        assert_eq!(
            store.published_brand_asset(&id).await.unwrap(),
            Some(vec![1, 2, 3])
        );
        let stale = store.branding_snapshot().await.unwrap();
        store
            .put_setting_policy(&SettingPolicyRow {
                key: "appearance.brandName".into(),
                global_value: Some("\"Other\"".into()),
                allow_user_override: false,
                global_locked: true,
                updated_by: "policy".into(),
                updated_at: 4,
            })
            .await
            .unwrap();
        assert!(!store.put_branding(&stale, save()).await.unwrap());
        let fresh = store.branding_snapshot().await.unwrap();
        assert!(
            store
                .put_branding(
                    &fresh,
                    BrandingUpdate {
                        images: &BrandImages::default(),
                        ..save()
                    }
                )
                .await
                .unwrap()
        );
        assert!(
            store
                .get_setting_policy("appearance.brandName")
                .await
                .unwrap()
                .unwrap()
                .global_locked
        );
        assert!(store.published_brand_asset(&id).await.unwrap().is_none());
    }
    #[tokio::test]
    async fn branding_cas_and_asset_publication_sqlite() {
        exercise(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn branding_cas_and_asset_publication_postgres() {
        if let Ok(url) = std::env::var("MW_TEST_PG") {
            exercise(
                Store::open(&url, crate::ServerKey::generate())
                    .await
                    .unwrap(),
            )
            .await;
        }
    }
}
