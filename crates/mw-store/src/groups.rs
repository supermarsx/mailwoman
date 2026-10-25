//! Feature-governance groups. Membership is authoritative at each policy read.
use crate::backend::{Row, q};
use crate::{Store, StoreError};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserGroup {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub source: String,
    pub ldap_dn: Option<String>,
    pub created_at: i64,
}

fn group(row: &Row) -> UserGroup {
    UserGroup {
        id: row.get_string("id"),
        name: row.get_string("name"),
        domain: row.get_string("domain"),
        source: row.get_string("source"),
        ldap_dn: row.get_opt_string("ldap_dn"),
        created_at: row.get_i64("created_at"),
    }
}

impl Store {
    pub async fn user_groups(&self) -> Result<Vec<UserGroup>, StoreError> {
        Ok(q("SELECT id,name,domain,source,ldap_dn,created_at FROM user_group ORDER BY domain,name,id").fetch_all(&self.backend).await?.iter().map(group).collect())
    }

    pub async fn put_user_group(&self, value: &UserGroup) -> Result<(), StoreError> {
        if value.id.is_empty()
            || value.name.trim().is_empty()
            || !matches!(value.source.as_str(), "manual" | "ldap")
        {
            return Err(StoreError::Corrupt("invalid group".into()));
        }
        let domain = self.canonical_policy_domain(&value.domain).await?;
        q("INSERT INTO user_group(id,name,domain,source,ldap_dn,created_at) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET name=excluded.name,domain=excluded.domain,source=excluded.source,ldap_dn=excluded.ldap_dn")
            .bind(&value.id).bind(&value.name).bind(domain).bind(&value.source).bind(value.ldap_dn.clone()).bind(value.created_at).execute(&self.backend).await?;
        Ok(())
    }

    pub async fn user_group_members(&self, id: &str) -> Result<Vec<String>, StoreError> {
        Ok(
            q("SELECT account_id FROM user_group_member WHERE group_id=?1 ORDER BY account_id")
                .bind(id)
                .fetch_all(&self.backend)
                .await?
                .iter()
                .map(|row| row.get_string("account_id"))
                .collect(),
        )
    }

    /// Replace the complete membership atomically. LDAP groups use the same
    /// primitive after a successful directory lookup; HTTP callers must prohibit
    /// manual membership edits for those groups.
    pub async fn replace_user_group_members(
        &self,
        id: &str,
        accounts: &[String],
    ) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        q("UPDATE user_group SET name=name WHERE id=?1")
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM user_group_member WHERE group_id=?1")
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        for account in accounts {
            q("INSERT INTO user_group_member(group_id,account_id) VALUES(?1,?2) ON CONFLICT(group_id,account_id) DO NOTHING").bind(id).bind(account).execute_tx(&mut tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_user_group(&self, id: &str) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        q("DELETE FROM feature_profile_assignment WHERE scope_kind='group' AND scope_id=?1")
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM setting_value WHERE scope_kind='group' AND scope_id=?1")
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM user_group WHERE id=?1")
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
}
