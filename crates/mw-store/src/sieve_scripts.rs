//! Account-scoped local Sieve scripts. Mutations and activation are transactional.
use crate::backend::q;
use crate::{Store, StoreError};

#[derive(Clone, PartialEq, Eq)]
pub struct SieveScriptRow {
    pub name: String,
    pub content: String,
    pub active: bool,
    pub state_json: String,
    pub updated_at: i64,
}

impl std::fmt::Debug for SieveScriptRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SieveScriptRow")
            .field("name", &self.name)
            .field("active", &self.active)
            .field("content", &"[REDACTED]")
            .finish()
    }
}

impl SieveScriptRow {
    /// An immutable opaque id, persisted beside interpreter state, survives rename.
    pub fn id(&self) -> String {
        serde_json::from_str::<serde_json::Value>(&self.state_json)
            .ok()
            .and_then(|v| {
                v.get("_jmapId")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SieveReplaceResult {
    Applied,
    StateMismatch,
    OverQuota,
}

impl Store {
    pub async fn has_sieve_authority(&self, account: &str) -> Result<bool, StoreError> {
        Ok(self
            .get_setting(&format!("sieveAuthority:{account}"))
            .await?
            .as_deref()
            == Some("1"))
    }
    async fn mark_sieve_authority(
        account: &str,
        tx: &mut crate::backend::Tx,
    ) -> Result<(), StoreError> {
        q("INSERT INTO settings (key,value) VALUES (?1,'1') ON CONFLICT (key) DO UPDATE SET value='1'")
            .bind(format!("sieveAuthority:{account}")).execute_tx(tx).await?;
        Ok(())
    }
    /// Reserve a vacation response before SMTP submission. A crash can suppress
    /// one reply, but cannot cause an automatic-response loop on restart.
    pub async fn claim_sieve_vacation(
        &self,
        account: &str,
        fingerprint: &str,
        now: i64,
        seconds: u64,
    ) -> Result<bool, StoreError> {
        let mut tx = self.sieve_tx(account).await?;
        let key = format!("sieveVacation:{account}:{fingerprint}");
        let rows = q("SELECT value FROM settings WHERE key = ?1")
            .bind(&key)
            .fetch_all_tx(&mut tx)
            .await?;
        if rows
            .first()
            .and_then(|r| r.get_string("value").parse::<i64>().ok())
            .is_some_and(|until| until > now)
        {
            tx.commit().await?;
            return Ok(false);
        }
        let until = now
            .saturating_add(i64::try_from(seconds).unwrap_or(i64::MAX))
            .to_string();
        q("INSERT INTO settings (key,value) VALUES (?1,?2) ON CONFLICT (key) DO UPDATE SET value=excluded.value")
            .bind(&key).bind(&until).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(true)
    }
    // The first write obtains SQLite's write lock or PostgreSQL's row lock.
    // Every script mutation uses this lock before reading: count/activation/CAS
    // checks therefore remain atomic across Store clones and server processes.
    async fn sieve_tx(&self, account_id: &str) -> Result<crate::backend::Tx, StoreError> {
        let mut tx = self.backend.begin().await?;
        let key = format!("sieveWriteLock:{account_id}");
        q("INSERT INTO settings (key,value) VALUES (?1,'0') ON CONFLICT (key) DO NOTHING")
            .bind(&key)
            .execute_tx(&mut tx)
            .await?;
        q("UPDATE settings SET value = value WHERE key = ?1")
            .bind(&key)
            .execute_tx(&mut tx)
            .await?;
        Ok(tx)
    }

    async fn sieve_rows_tx(
        &self,
        account_id: &str,
        tx: &mut crate::backend::Tx,
    ) -> Result<Vec<SieveScriptRow>, StoreError> {
        let rows = q("SELECT name, content, active, state_json, updated_at FROM sieve_script WHERE account_id = ?1 ORDER BY name")
            .bind(account_id).fetch_all_tx(tx).await?;
        let mut result = Vec::new();
        for row in rows {
            let mut script = SieveScriptRow {
                name: row.get_string("name"),
                content: row.get_string("content"),
                active: row.get_i64("active") != 0,
                state_json: row.get_string("state_json"),
                updated_at: row.get_i64("updated_at"),
            };
            if script.id().is_empty() {
                let mut state: serde_json::Value = serde_json::from_str(&script.state_json)
                    .map_err(|_| StoreError::Corrupt("invalid Sieve state".into()))?;
                let object = state
                    .as_object_mut()
                    .ok_or_else(|| StoreError::Corrupt("invalid Sieve state".into()))?;
                object.insert("_jmapId".into(), crate::seal::random_token().into());
                script.state_json = state.to_string();
                q("UPDATE sieve_script SET state_json = ?1 WHERE account_id = ?2 AND name = ?3")
                    .bind(&script.state_json)
                    .bind(account_id)
                    .bind(&script.name)
                    .execute_tx(tx)
                    .await?;
            }
            result.push(script);
        }
        Ok(result)
    }

    pub async fn list_sieve_scripts(
        &self,
        account_id: &str,
    ) -> Result<Vec<SieveScriptRow>, StoreError> {
        let mut tx = self.sieve_tx(account_id).await?;
        let rows = self.sieve_rows_tx(account_id, &mut tx).await?;
        if !rows.is_empty() {
            Self::mark_sieve_authority(account_id, &mut tx).await?;
        }
        tx.commit().await?;
        Ok(rows)
    }

    pub async fn put_sieve_script(
        &self,
        account_id: &str,
        name: &str,
        content: &str,
        activate: bool,
    ) -> Result<(), StoreError> {
        self.put_sieve_script_limited(account_id, name, content, activate, None)
            .await?;
        Ok(())
    }

    /// Quota and mutation share one write transaction. False means over quota.
    pub async fn put_sieve_script_limited(
        &self,
        account_id: &str,
        name: &str,
        content: &str,
        activate: bool,
        max: Option<usize>,
    ) -> Result<bool, StoreError> {
        let mut tx = self.sieve_tx(account_id).await?;
        let rows = self.sieve_rows_tx(account_id, &mut tx).await?;
        let existing = rows.iter().find(|s| s.name == name);
        if existing.is_none() && max.is_some_and(|max| rows.len() >= max) {
            return Ok(false);
        }
        let mut state: serde_json::Value = existing
            .and_then(|s| serde_json::from_str(&s.state_json).ok())
            .unwrap_or_else(|| serde_json::json!({"_jmapId":crate::seal::random_token()}));
        state["_remote"] = false.into();
        if existing.is_some_and(|s| s.content != content) {
            state.as_object_mut().unwrap().remove("_blobId");
        }
        let state = state.to_string();
        if activate {
            q("UPDATE sieve_script SET active = 0 WHERE account_id = ?1 AND active = 1")
                .bind(account_id)
                .execute_tx(&mut tx)
                .await?;
        }
        q("INSERT INTO sieve_script (account_id, name, content, active, size, state_json, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) ON CONFLICT (account_id, name) DO UPDATE SET content = excluded.content, size = excluded.size, state_json = excluded.state_json, active = CASE WHEN excluded.active = 1 THEN 1 ELSE sieve_script.active END, updated_at = excluded.updated_at")
            .bind(account_id).bind(name).bind(content).bind(i64::from(activate)).bind(content.len() as i64).bind(state).bind(chrono::Utc::now().timestamp_millis()).execute_tx(&mut tx).await?;
        Self::mark_sieve_authority(account_id, &mut tx).await?;
        tx.commit().await?;
        Ok(true)
    }

    /// Apply a JMAP snapshot only if it is still current. Names may change, ids and
    /// interpreter state are supplied unchanged by the authenticated caller.
    pub async fn replace_sieve_scripts(
        &self,
        account_id: &str,
        expected: &[SieveScriptRow],
        rows: &[SieveScriptRow],
        max: Option<usize>,
    ) -> Result<SieveReplaceResult, StoreError> {
        let mut tx = self.sieve_tx(account_id).await?;
        if self.sieve_rows_tx(account_id, &mut tx).await? != expected {
            return Ok(SieveReplaceResult::StateMismatch);
        }
        if max.is_some_and(|n| rows.len() > n) {
            return Ok(SieveReplaceResult::OverQuota);
        }
        let ids: std::collections::BTreeSet<_> = rows.iter().map(SieveScriptRow::id).collect();
        let names: std::collections::BTreeSet<_> = rows.iter().map(|s| &s.name).collect();
        if ids.len() != rows.len()
            || ids.contains("")
            || names.len() != rows.len()
            || rows.iter().filter(|s| s.active).count() > 1
        {
            return Err(StoreError::Corrupt("invalid Sieve snapshot".into()));
        }
        q("DELETE FROM sieve_script WHERE account_id = ?1")
            .bind(account_id)
            .execute_tx(&mut tx)
            .await?;
        for row in rows {
            q("INSERT INTO sieve_script (account_id,name,content,active,size,state_json,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)")
                .bind(account_id).bind(&row.name).bind(&row.content).bind(i64::from(row.active)).bind(row.content.len() as i64).bind(&row.state_json).bind(row.updated_at).execute_tx(&mut tx).await?;
        }
        if !rows.is_empty() || !expected.is_empty() {
            Self::mark_sieve_authority(account_id, &mut tx).await?;
        }
        tx.commit().await?;
        Ok(SieveReplaceResult::Applied)
    }

    pub async fn activate_sieve_script(
        &self,
        account_id: &str,
        name: Option<&str>,
    ) -> Result<bool, StoreError> {
        let mut tx = self.sieve_tx(account_id).await?;
        let rows = self.sieve_rows_tx(account_id, &mut tx).await?;
        if name.is_some_and(|name| !rows.iter().any(|s| s.name == name)) {
            return Ok(false);
        }
        q("UPDATE sieve_script SET active = 0 WHERE account_id = ?1 AND active = 1")
            .bind(account_id)
            .execute_tx(&mut tx)
            .await?;
        if let Some(name) = name {
            q("UPDATE sieve_script SET active = 1 WHERE account_id = ?1 AND name = ?2")
                .bind(account_id)
                .bind(name)
                .execute_tx(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(true)
    }

    pub async fn delete_sieve_script(
        &self,
        account_id: &str,
        name: &str,
    ) -> Result<bool, StoreError> {
        let mut tx = self.sieve_tx(account_id).await?;
        let changed =
            q("DELETE FROM sieve_script WHERE account_id = ?1 AND name = ?2 AND active = 0")
                .bind(account_id)
                .bind(name)
                .execute_tx(&mut tx)
                .await?;
        tx.commit().await?;
        Ok(changed > 0)
    }
}

#[cfg(test)]
mod tests {
    use crate::{ServerKey, Store};
    #[tokio::test]
    async fn concurrent_quota_and_snapshot_cas_preserve_account_state() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let (a, b) = tokio::join!(
            store.put_sieve_script_limited("alice", "a", "keep;", false, Some(1)),
            store.put_sieve_script_limited("alice", "b", "keep;", false, Some(1))
        );
        assert_ne!(a.unwrap(), b.unwrap());
        let before = store.list_sieve_scripts("alice").await.unwrap();
        let mut after = before.clone();
        after[0].name = "renamed".into();
        assert_eq!(
            store
                .replace_sieve_scripts("alice", &before, &after, Some(1))
                .await
                .unwrap(),
            super::SieveReplaceResult::Applied
        );
        assert_eq!(
            store.list_sieve_scripts("alice").await.unwrap()[0].id(),
            before[0].id()
        );
        assert_eq!(
            store
                .replace_sieve_scripts("alice", &before, &[], Some(1))
                .await
                .unwrap(),
            super::SieveReplaceResult::StateMismatch
        );
        assert_eq!(
            store
                .replace_sieve_scripts("bob", &before, &[], Some(1))
                .await
                .unwrap(),
            super::SieveReplaceResult::StateMismatch
        );
        assert_eq!(store.list_sieve_scripts("alice").await.unwrap(), after);
    }
    #[tokio::test]
    async fn scripts_are_scoped_activation_is_atomic_and_source_is_redacted() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        store
            .put_sieve_script("alice", "one", "secret-script-one", true)
            .await
            .unwrap();
        store
            .put_sieve_script("alice", "two", "secret-script-two", false)
            .await
            .unwrap();
        store
            .put_sieve_script("bob", "one", "bob-secret", true)
            .await
            .unwrap();
        assert!(
            !store
                .activate_sieve_script("alice", Some("missing"))
                .await
                .unwrap()
        );
        assert!(store.list_sieve_scripts("alice").await.unwrap()[0].active);
        assert!(
            store
                .activate_sieve_script("alice", Some("two"))
                .await
                .unwrap()
        );
        let scripts = store.list_sieve_scripts("alice").await.unwrap();
        assert_eq!(scripts.iter().filter(|s| s.active).count(), 1);
        assert_eq!(scripts.iter().find(|s| s.active).unwrap().name, "two");
        assert!(!format!("{scripts:?}").contains("secret"));
        assert!(!store.delete_sieve_script("alice", "two").await.unwrap());
        assert!(store.delete_sieve_script("alice", "one").await.unwrap());
        assert!(store.list_sieve_scripts("bob").await.unwrap()[0].active);
        assert!(store.activate_sieve_script("alice", None).await.unwrap());
        assert!(store.delete_sieve_script("alice", "two").await.unwrap());
    }
}
