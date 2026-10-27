//! Typed personal records; legacy raw-account tables are never an implicit fallback.
use crate::{
    SavedSearchRow, SignatureRow, Store, StoreError,
    backend::{Row, q},
};

impl Store {
    pub async fn upsert_personal_signature(&self, row: &SignatureRow) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        // Take the established cross-process identity write lock before touching
        // personal rows, including the first signature for a previously empty owner.
        // A unique partial index independently enforces the persisted invariant.
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        if row.is_default {
            q("UPDATE personal_signature SET is_default=0 WHERE owner_subject=?1 AND is_default<>0 AND name<>?2")
                .bind(&row.account_id).bind(&row.name).execute_tx(&mut tx).await?;
        }
        q("INSERT INTO personal_signature(owner_subject,name,body,is_default,rule_json,updated_at,html,text,text_edited) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT(owner_subject,name) DO UPDATE SET body=excluded.body,is_default=excluded.is_default,rule_json=excluded.rule_json,updated_at=excluded.updated_at,html=excluded.html,text=excluded.text,text_edited=excluded.text_edited")
            .bind(&row.account_id).bind(&row.name).bind(&row.body).bind(i64::from(row.is_default)).bind(&row.rule_json).bind(chrono::Utc::now().to_rfc3339()).bind(row.html.as_deref()).bind(row.text.as_deref()).bind(i64::from(row.text_edited)).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn list_personal_signatures(
        &self,
        owner: &str,
    ) -> Result<Vec<SignatureRow>, StoreError> {
        Ok(q("SELECT owner_subject,name,body,is_default,rule_json,updated_at,html,text,text_edited FROM personal_signature WHERE owner_subject=?1 ORDER BY name").bind(owner).fetch_all(&self.backend).await?.iter().map(|row| SignatureRow {
            account_id:row.get_string("owner_subject"),name:row.get_string("name"),body:row.get_string("body"),is_default:row.get_i64("is_default")!=0,rule_json:row.get_string("rule_json"),updated_at:row.get_string("updated_at"),html:row.get_opt_string("html"),text:row.get_opt_string("text"),text_edited:row.get_i64("text_edited")!=0,
        }).collect())
    }
    pub async fn delete_personal_signature(
        &self,
        owner: &str,
        name: &str,
    ) -> Result<(), StoreError> {
        q("DELETE FROM personal_signature WHERE owner_subject=?1 AND name=?2")
            .bind(owner)
            .bind(name)
            .execute(&self.backend)
            .await?;
        Ok(())
    }
    pub async fn upsert_personal_saved_search(
        &self,
        row: &SavedSearchRow,
    ) -> Result<bool, StoreError> {
        Ok(q("INSERT INTO personal_saved_search(id,owner_subject,name,query_json,as_folder) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET name=excluded.name,query_json=excluded.query_json,as_folder=excluded.as_folder WHERE personal_saved_search.owner_subject=excluded.owner_subject")
            .bind(&row.id).bind(&row.user).bind(&row.name).bind(&row.query_json).bind(i64::from(row.as_folder)).execute(&self.backend).await?==1)
    }
    pub async fn list_personal_saved_searches(
        &self,
        owner: &str,
    ) -> Result<Vec<SavedSearchRow>, StoreError> {
        Ok(q("SELECT id,owner_subject,name,query_json,as_folder FROM personal_saved_search WHERE owner_subject=?1 ORDER BY name,id").bind(owner).fetch_all(&self.backend).await?.iter().map(saved_search).collect())
    }
    pub async fn get_personal_saved_search(
        &self,
        id: &str,
    ) -> Result<Option<SavedSearchRow>, StoreError> {
        Ok(q("SELECT id,owner_subject,name,query_json,as_folder FROM personal_saved_search WHERE id=?1").bind(id).fetch_optional(&self.backend).await?.as_ref().map(saved_search))
    }
    pub async fn delete_personal_saved_search(
        &self,
        owner: &str,
        id: &str,
    ) -> Result<bool, StoreError> {
        Ok(
            q("DELETE FROM personal_saved_search WHERE id=?1 AND owner_subject=?2")
                .bind(id)
                .bind(owner)
                .execute(&self.backend)
                .await?
                == 1,
        )
    }
}
fn saved_search(row: &Row) -> SavedSearchRow {
    SavedSearchRow {
        id: row.get_string("id"),
        user: row.get_string("owner_subject"),
        name: row.get_string("name"),
        query_json: row.get_string("query_json"),
        as_folder: row.get_i64("as_folder") != 0,
    }
}

#[cfg(test)]
mod signature_default_tests {
    use super::*;
    fn signature(owner: &str, name: &str, is_default: bool) -> SignatureRow {
        SignatureRow {
            account_id: owner.into(),
            name: name.into(),
            body: format!("body {name}"),
            html: Some(format!("<b>{name}</b>")),
            text: Some(format!("plain {name}")),
            text_edited: true,
            is_default,
            rule_json: "{}".into(),
            updated_at: String::new(),
        }
    }
    async fn exercise(store: Store) {
        let owner = format!("po1_default_{}", uuid::Uuid::new_v4());
        let other = format!("po1_other_{}", uuid::Uuid::new_v4());
        store
            .upsert_personal_signature(&signature(&other, "other", true))
            .await
            .unwrap();
        store
            .upsert_personal_signature(&signature(&owner, "first", true))
            .await
            .unwrap();
        store
            .upsert_personal_signature(&signature(&owner, "second", true))
            .await
            .unwrap();
        let rows = store.list_personal_signatures(&owner).await.unwrap();
        assert_eq!(
            rows.iter().filter(|row| row.is_default).count(),
            1,
            "a new default must replace the old default"
        );
        assert!(
            rows.iter()
                .find(|row| row.name == "second")
                .unwrap()
                .is_default
        );
        let mut writers = tokio::task::JoinSet::new();
        for n in 0..16 {
            let store = store.clone();
            let row = signature(&owner, &format!("concurrent-{n}"), true);
            writers.spawn(async move { store.upsert_personal_signature(&row).await });
        }
        while let Some(result) = writers.join_next().await {
            result.unwrap().unwrap();
        }
        let rows = store.list_personal_signatures(&owner).await.unwrap();
        assert_eq!(rows.len(), 18);
        assert_eq!(rows.iter().filter(|row| row.is_default).count(), 1);
        for row in &rows {
            assert_eq!(row.html, Some(format!("<b>{}</b>", row.name)));
            assert_eq!(row.text, Some(format!("plain {}", row.name)));
            assert!(row.text_edited);
        }
        let winner = rows.iter().find(|row| row.is_default).unwrap().name.clone();
        store
            .upsert_personal_signature(&signature(&owner, "ordinary", false))
            .await
            .unwrap();
        assert!(
            store
                .list_personal_signatures(&owner)
                .await
                .unwrap()
                .iter()
                .any(|row| row.name == winner && row.is_default)
        );
        store
            .upsert_personal_signature(&signature(&owner, &winner, false))
            .await
            .unwrap();
        assert!(
            !store
                .list_personal_signatures(&owner)
                .await
                .unwrap()
                .iter()
                .any(|row| row.is_default)
        );
        assert!(store.list_personal_signatures(&other).await.unwrap()[0].is_default);
    }
    #[tokio::test]
    async fn signature_defaults_sqlite_are_atomic_and_owner_scoped() {
        exercise(
            Store::open_in_memory(crate::ServerKey::from_bytes(&[91; 32]).unwrap())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn signature_defaults_postgres_are_atomic_and_owner_scoped() {
        let Ok(dsn) = std::env::var("MW_SIGNATURE_TEST_PG") else {
            return;
        };
        exercise(
            Store::open_postgres(&dsn, crate::ServerKey::from_bytes(&[91; 32]).unwrap())
                .await
                .unwrap(),
        )
        .await;
    }
    async fn upgrade_preserves_contents(store: Store) {
        q("DROP INDEX personal_signature_one_default")
            .execute(&store.backend)
            .await
            .unwrap();
        let owner = format!("po1_upgrade_{}", uuid::Uuid::new_v4());
        let other = format!("po1_upgrade_other_{}", uuid::Uuid::new_v4());
        for (subject, name, time, default) in [
            (&owner, "old", "2020-01-01T00:00:00Z", 1_i64),
            (&owner, "z", "2026-01-01T00:00:00Z", 1),
            (&owner, "a", "2026-01-01T00:00:00Z", 1),
            (&owner, "ordinary", "2027-01-01T00:00:00Z", 0),
            (&other, "untouched", "2020-01-01T00:00:00Z", 1),
        ] {
            q("INSERT INTO personal_signature(owner_subject,name,body,is_default,rule_json,updated_at,html,text,text_edited) VALUES(?1,?2,'original body',?3,'{}',?4,'<b>original</b>','independent plain',1)").bind(subject).bind(name).bind(default).bind(time).execute(&store.backend).await.unwrap();
        }
        match &store.backend {
            crate::Backend::Sqlite(pool) => {
                sqlx::raw_sql(include_str!("../migrations/0069_signature_default.sql"))
                    .execute(pool)
                    .await
                    .unwrap();
            }
            crate::Backend::Postgres(pool) => {
                sqlx::raw_sql(include_str!("../migrations_pg/0069_signature_default.sql"))
                    .execute(pool)
                    .await
                    .unwrap();
            }
        }
        let rows = store.list_personal_signatures(&owner).await.unwrap();
        assert_eq!(rows.len(), 4);
        assert_eq!(
            rows.iter()
                .filter(|row| row.is_default)
                .map(|row| row.name.as_str())
                .collect::<Vec<_>>(),
            ["a"]
        );
        for row in rows {
            assert_eq!(row.body, "original body");
            assert_eq!(row.html.as_deref(), Some("<b>original</b>"));
            assert_eq!(row.text.as_deref(), Some("independent plain"));
            assert!(row.text_edited);
        }
        assert!(store.list_personal_signatures(&other).await.unwrap()[0].is_default);
        assert!(
            q("UPDATE personal_signature SET is_default=1 WHERE owner_subject=?1 AND name='z'")
                .bind(&owner)
                .execute(&store.backend)
                .await
                .is_err(),
            "database must reject another truthy default"
        );
    }
    #[tokio::test]
    async fn signature_default_upgrade_sqlite_preserves_both_formats() {
        upgrade_preserves_contents(
            Store::open_in_memory(crate::ServerKey::from_bytes(&[92; 32]).unwrap())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn signature_default_upgrade_postgres_preserves_both_formats() {
        let Ok(dsn) = std::env::var("MW_SIGNATURE_TEST_PG") else {
            return;
        };
        upgrade_preserves_contents(
            Store::open_postgres(&dsn, crate::ServerKey::from_bytes(&[92; 32]).unwrap())
                .await
                .unwrap(),
        )
        .await;
    }
}
