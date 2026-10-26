//! Account-owned package records and a revision-bound, sealed provider key.
use crate::{Store, StoreError, backend::q, seal};

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageProvider {
    pub enabled: bool,
    pub refresh_secs: i64,
    pub configured: bool,
    pub revision: String,
}

impl Store {
    pub async fn package_provider(&self) -> Result<Option<PackageProvider>, StoreError> {
        Ok(q("SELECT enabled,refresh_secs,revision FROM package_provider WHERE id='aftership'")
            .fetch_optional(&self.backend).await?.map(|row| PackageProvider {
                enabled: row.get_i64("enabled") != 0,
                refresh_secs: row.get_i64("refresh_secs"),
                configured: true,
                revision: row.get_string("revision"),
            }))
    }

    /// None retains the existing secret; an empty secret is never accepted.
    /// The caller must validate bounds and require fresh administrator proof.
    pub async fn put_package_provider(&self, enabled: bool, refresh_secs: i64, key: Option<&str>, expected: Option<&str>) -> Result<Option<PackageProvider>, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings(key,value) VALUES('package:authority-lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='package:authority-lock'").execute_tx(&mut tx).await?;
        let old = q("SELECT * FROM package_provider WHERE id='aftership'").fetch_all_tx(&mut tx).await?.into_iter().next();
        if old.as_ref().map(|row|row.get_string("revision")).as_deref() != expected { return Ok(None); }
        let plaintext = match (key, old.as_ref()) {
            (Some(key), _) => key.as_bytes().to_vec(),
            (None, Some(row)) => self.key.open_bound(&row.get_blob("sealed_key"), "package-provider-key", &["aftership", &row.get_string("revision")])?,
            (None, None) => return Ok(None),
        };
        if plaintext.is_empty() || plaintext.len() > 512 || !(60..=86400).contains(&refresh_secs) { return Ok(None); }
        let revision = seal::random_token();
        let sealed = self.key.seal_bound(&plaintext, "package-provider-key", &["aftership", &revision])?;
        q("INSERT INTO package_provider(id,enabled,refresh_secs,sealed_key,revision) VALUES('aftership',?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET enabled=excluded.enabled,refresh_secs=excluded.refresh_secs,sealed_key=excluded.sealed_key,revision=excluded.revision")
            .bind(i64::from(enabled)).bind(refresh_secs).bind(sealed).bind(&revision).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(Some(PackageProvider {enabled,refresh_secs,configured:true,revision}))
    }

    /// Disabled or changed authorities cannot release a key to a stale operation.
    pub async fn package_provider_key(&self, revision: &str) -> Result<Option<String>, StoreError> {
        let Some(row) = q("SELECT sealed_key FROM package_provider WHERE id='aftership' AND enabled=1 AND revision=?1").bind(revision).fetch_optional(&self.backend).await? else { return Ok(None); };
        let bytes = self.key.open_bound(&row.get_blob("sealed_key"), "package-provider-key", &["aftership", revision])?;
        Ok(Some(String::from_utf8(bytes).map_err(|_|seal::SealError::Decrypt)?))
    }

    pub async fn package_account(&self, account: &str) -> Result<Option<(String,String)>,StoreError> {
        Ok(q("SELECT state,revision FROM package_account WHERE account_id=?1").bind(account).fetch_optional(&self.backend).await?.map(|row|(row.get_string("state"),row.get_string("revision"))))
    }

    /// Bounded optimistic update; account identity comes from the session.
    pub async fn put_package_account(&self, account: &str, state: &str, expected: Option<&str>) -> Result<Option<String>,StoreError> {
        if state.len() > 2*1024*1024 { return Ok(None); }
        let revision = seal::random_token();
        let affected = match expected {
            Some(old) => q("UPDATE package_account SET state=?1,revision=?2 WHERE account_id=?3 AND revision=?4").bind(state).bind(&revision).bind(account).bind(old).execute(&self.backend).await?,
            None => q("INSERT INTO package_account(account_id,state,revision) VALUES(?1,?2,?3) ON CONFLICT(account_id) DO NOTHING").bind(account).bind(state).bind(&revision).execute(&self.backend).await?,
        };
        Ok((affected > 0).then_some(revision))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn account_rows_are_isolated_and_updates_are_conditional() {
        let store=Store::open_in_memory(crate::ServerKey::generate()).await.unwrap();
        let mut ids=Vec::new();
        for username in ["one@example.test","two@example.test"] {
            ids.push(store.create_account(&crate::NewAccount{kind:crate::AccountKind::Imap,host:"h",port:993,tls:"implicit",username,sync_policy_json:"{}"},&crate::Credentials{username:username.into(),password:"test".into()}).await.unwrap());
        }
        let first=store.put_package_account(&ids[0],"private tracking record",None).await.unwrap().unwrap();
        assert!(store.package_account(&ids[1]).await.unwrap().is_none());
        assert!(store.put_package_account(&ids[0],"stale",None).await.unwrap().is_none());
        let (left,right)=tokio::join!(store.put_package_account(&ids[0],"left",Some(&first)),store.put_package_account(&ids[0],"right",Some(&first)));
        assert_ne!(left.unwrap().is_some(),right.unwrap().is_some());
        q("DELETE FROM accounts WHERE id=?1").bind(&ids[0]).execute(&store.backend).await.unwrap();
        assert!(store.package_account(&ids[0]).await.unwrap().is_none());
    }
    #[tokio::test]
    async fn provider_key_is_sealed_bound_and_revoked_on_change() {
        let store = Store::open_in_memory(crate::ServerKey::generate()).await.unwrap();
        let first = store.put_package_provider(true,900,Some("private-provider-key"),None).await.unwrap().unwrap();
        let row = q("SELECT sealed_key FROM package_provider").fetch_optional(&store.backend).await.unwrap().unwrap();
        assert!(!row.get_blob("sealed_key").windows(20).any(|bytes|bytes == b"private-provider-key"));
        assert_eq!(store.package_provider_key(&first.revision).await.unwrap().as_deref(),Some("private-provider-key"));
        assert!(store.put_package_provider(false,900,None,None).await.unwrap().is_none());
        let second = store.put_package_provider(false,900,None,Some(&first.revision)).await.unwrap().unwrap();
        assert!(store.package_provider_key(&first.revision).await.unwrap().is_none());
        assert!(store.package_provider_key(&second.revision).await.unwrap().is_none());
        let third = store.put_package_provider(true,900,None,Some(&second.revision)).await.unwrap().unwrap();
        assert_eq!(store.package_provider_key(&third.revision).await.unwrap().as_deref(),Some("private-provider-key"));
    }
}
