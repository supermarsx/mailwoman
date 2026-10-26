//! Typed personal records; legacy raw-account tables are never an implicit fallback.
use crate::{
    SavedSearchRow, SignatureRow, Store, StoreError,
    backend::{Row, q},
};

impl Store {
    pub async fn upsert_personal_signature(&self, row: &SignatureRow) -> Result<(), StoreError> {
        q("INSERT INTO personal_signature(owner_subject,name,body,is_default,rule_json,updated_at,html,text,text_edited) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT(owner_subject,name) DO UPDATE SET body=excluded.body,is_default=excluded.is_default,rule_json=excluded.rule_json,updated_at=excluded.updated_at,html=excluded.html,text=excluded.text,text_edited=excluded.text_edited")
            .bind(&row.account_id).bind(&row.name).bind(&row.body).bind(i64::from(row.is_default)).bind(&row.rule_json).bind(chrono::Utc::now().to_rfc3339()).bind(row.html.as_deref()).bind(row.text.as_deref()).bind(i64::from(row.text_edited)).execute(&self.backend).await?;
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
