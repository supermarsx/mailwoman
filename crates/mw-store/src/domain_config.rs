//! Domain configuration from migration 0033. All queries support both backends.
use crate::backend::{Row, Tx, q};
use crate::{Store, StoreError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainConfigRow {
    pub name: String,
    pub upstream_json: String,
    pub allowlist_json: String,
    pub blocklist_json: String,
    pub enabled: bool,
    pub preferred: bool,
    pub settings_json: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainSigninState {
    Enabled,
    Disabled,
    Unlisted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainMutation {
    Applied,
    Stale,
    Conflict,
    Shadow,
    Disabled,
}

async fn lock_domains(tx: &mut Tx) -> Result<(), StoreError> {
    q("INSERT INTO settings (key,value) VALUES ('v6:domains:write-lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(tx).await?;
    q("UPDATE settings SET value=value WHERE key='v6:domains:write-lock'")
        .execute_tx(tx)
        .await?;
    Ok(())
}
async fn row_tx(tx: &mut Tx, name: &str) -> Result<Option<DomainConfigRow>, StoreError> {
    Ok(q("SELECT * FROM domains WHERE name=?1")
        .bind(name)
        .fetch_all_tx(tx)
        .await?
        .first()
        .map(domain))
}
async fn mappings_tx(tx: &mut Tx, name: &str, host: bool) -> Result<Vec<String>, StoreError> {
    let (sql, column) = if host {
        (
            "SELECT host FROM domain_host WHERE domain=?1 ORDER BY host",
            "host",
        )
    } else {
        (
            "SELECT alias FROM domain_alias WHERE domain=?1 ORDER BY alias",
            "alias",
        )
    };
    Ok(q(sql)
        .bind(name)
        .fetch_all_tx(tx)
        .await?
        .iter()
        .map(|r| r.get_string(column))
        .collect())
}

/// Non-secret authority metadata used during an upstream configuration upgrade.
pub struct SessionAuthority {
    pub id: String,
    pub account_id: String,
    pub username: String,
    pub jmap_url: String,
}
pub struct DomainAccount {
    pub account_id: String,
    pub username: String,
    pub seen_host: Option<String>,
}

fn domain(row: &Row) -> DomainConfigRow {
    DomainConfigRow {
        name: row.get_string("name"),
        upstream_json: row.get_string("upstream_json"),
        allowlist_json: row.get_string("allowlist"),
        blocklist_json: row.get_string("blocklist"),
        enabled: row.get_i64("enabled") != 0,
        preferred: row.get_i64("preferred") != 0,
        settings_json: row.get_string("settings_json"),
        updated_at: row.get_opt_i64("updated_at").unwrap_or(0),
    }
}

impl Store {
    pub async fn preferred_domain_record(&self) -> Result<Option<DomainConfigRow>, StoreError> {
        Ok(q("SELECT * FROM domains WHERE preferred=1")
            .fetch_optional(&self.backend)
            .await?
            .as_ref()
            .map(domain))
    }
    /// A single write lock serializes preference and mapping resources across SQLite/Postgres.
    pub async fn compare_and_prefer_domain(
        &self,
        previous: Option<&DomainConfigRow>,
        target: Option<&DomainConfigRow>,
        dry_run: bool,
    ) -> Result<DomainMutation, StoreError> {
        let mut tx = self.backend.begin().await?;
        lock_domains(&mut tx).await?;
        // Lock domain rows against ordinary editor updates as well.
        q("UPDATE domains SET name=name WHERE preferred=1")
            .execute_tx(&mut tx)
            .await?;
        let current = q("SELECT * FROM domains WHERE preferred=1")
            .fetch_all_tx(&mut tx)
            .await?
            .first()
            .map(domain);
        if current.as_ref() != previous {
            return Ok(DomainMutation::Stale);
        }
        if let Some(target) = target {
            q("UPDATE domains SET name=name WHERE name=?1")
                .bind(&target.name)
                .execute_tx(&mut tx)
                .await?;
            if row_tx(&mut tx, &target.name).await?.as_ref() != Some(target) {
                return Ok(DomainMutation::Stale);
            }
            if !target.enabled {
                return Ok(DomainMutation::Disabled);
            }
        }
        if !dry_run {
            q("UPDATE domains SET preferred=0 WHERE preferred=1")
                .execute_tx(&mut tx)
                .await?;
            if let Some(target) = target {
                q("UPDATE domains SET preferred=1 WHERE name=?1")
                    .bind(&target.name)
                    .execute_tx(&mut tx)
                    .await?;
            }
        }
        tx.commit().await?;
        Ok(DomainMutation::Applied)
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn compare_and_map_domain(
        &self,
        previous: &DomainConfigRow,
        aliases: &[String],
        hosts: &[String],
        key: &str,
        host: bool,
        remove: bool,
        dry_run: bool,
    ) -> Result<DomainMutation, StoreError> {
        let mut tx = self.backend.begin().await?;
        lock_domains(&mut tx).await?;
        q("UPDATE domains SET name=name WHERE name=?1")
            .bind(&previous.name)
            .execute_tx(&mut tx)
            .await?;
        if row_tx(&mut tx, &previous.name).await?.as_ref() != Some(previous)
            || mappings_tx(&mut tx, &previous.name, false).await? != aliases
            || mappings_tx(&mut tx, &previous.name, true).await? != hosts
        {
            return Ok(DomainMutation::Stale);
        }
        if !host && !remove && row_tx(&mut tx, key).await?.is_some() {
            return Ok(DomainMutation::Shadow);
        }
        let owner = q(if host {
            "SELECT domain FROM domain_host WHERE host=?1"
        } else {
            "SELECT domain FROM domain_alias WHERE alias=?1"
        })
        .bind(key)
        .fetch_opt_scalar_string_tx(&mut tx)
        .await?;
        if owner.as_deref().is_some_and(|owner| owner != previous.name) {
            return Ok(DomainMutation::Conflict);
        }
        if !dry_run {
            let sql = match (host, remove) {
                (true, true) => "DELETE FROM domain_host WHERE host=?1 AND domain=?2",
                (false, true) => "DELETE FROM domain_alias WHERE alias=?1 AND domain=?2",
                (true, false) => {
                    "INSERT INTO domain_host (host,domain) VALUES (?1,?2) ON CONFLICT(host) DO NOTHING"
                }
                (false, false) => {
                    "INSERT INTO domain_alias (alias,domain) VALUES (?1,?2) ON CONFLICT(alias) DO NOTHING"
                }
            };
            q(sql)
                .bind(key)
                .bind(&previous.name)
                .execute_tx(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(DomainMutation::Applied)
    }
    /// Operator inventory survives session revocation through remembered names.
    pub async fn domain_account_inventory(&self) -> Result<Vec<DomainAccount>, StoreError> {
        let mut accounts = std::collections::BTreeMap::new();
        for account in self.list_accounts().await? {
            accounts.insert(
                account.id.clone(),
                DomainAccount {
                    account_id: account.id,
                    username: account.username,
                    seen_host: Some(account.host),
                },
            );
        }
        for session in self.session_authorities().await? {
            accounts
                .entry(session.account_id.clone())
                .or_insert(DomainAccount {
                    account_id: session.account_id,
                    username: session.username,
                    seen_host: None,
                });
        }
        for row in q("SELECT key,value FROM settings WHERE key LIKE 'v6:account:names:%'")
            .fetch_all(&self.backend)
            .await?
        {
            let key = row.get_string("key");
            let Some(id) = key.strip_prefix("v6:account:names:") else {
                continue;
            };
            let names: Vec<String> = serde_json::from_str(&row.get_string("value"))
                .map_err(|e| StoreError::Corrupt(e.to_string()))?;
            if let Some(name) = names
                .iter()
                .find(|name| name.contains('@'))
                .or(names.first())
            {
                accounts.entry(id.to_string()).or_insert(DomainAccount {
                    account_id: id.into(),
                    username: name.clone(),
                    seen_host: None,
                });
            }
        }
        Ok(accounts.into_values().collect())
    }
    /// Repoint only the session authority that was just authenticated.
    pub async fn repoint_session_authority(
        &self,
        id: &str,
        previous_url: &str,
        account: &str,
        url: &str,
        api_url: &str,
    ) -> Result<bool, StoreError> {
        Ok(q(
            "UPDATE sessions SET account_id=?3,jmap_url=?4,api_url=?5 WHERE id=?1 AND jmap_url=?2",
        )
        .bind(id)
        .bind(previous_url)
        .bind(account)
        .bind(url)
        .bind(api_url)
        .execute(&self.backend)
        .await?
            == 1)
    }
    pub async fn session_authorities(&self) -> Result<Vec<SessionAuthority>, StoreError> {
        Ok(q("SELECT id,account_id,username,jmap_url FROM sessions")
            .fetch_all(&self.backend)
            .await?
            .iter()
            .map(|row| SessionAuthority {
                id: row.get_string("id"),
                account_id: row.get_string("account_id"),
                username: row.get_string("username"),
                jmap_url: row.get_string("jmap_url"),
            })
            .collect())
    }
    pub async fn get_domain_config(
        &self,
        name: &str,
    ) -> Result<Option<DomainConfigRow>, StoreError> {
        let row = q("SELECT name, upstream_json, allowlist, blocklist, enabled, preferred, settings_json, updated_at FROM domains WHERE name = ?1")
            .bind(name).fetch_optional(&self.backend).await?;
        Ok(row.as_ref().map(domain))
    }

    pub async fn list_domain_configs(&self) -> Result<Vec<DomainConfigRow>, StoreError> {
        Ok(q("SELECT name, upstream_json, allowlist, blocklist, enabled, preferred, settings_json, updated_at FROM domains ORDER BY name")
            .fetch_all(&self.backend).await?.iter().map(domain).collect())
    }

    /// Batch alias projection for domain lists, avoiding a query per domain.
    pub async fn list_domain_aliases(&self) -> Result<Vec<(String, String)>, StoreError> {
        Ok(q("SELECT domain, alias FROM domain_alias ORDER BY domain, alias")
            .fetch_all(&self.backend).await?.iter()
            .map(|row| (row.get_string("domain"), row.get_string("alias"))).collect())
    }

    /// Preference is managed separately, so an ordinary edit cannot accidentally
    /// clear the preferred domain or collide with its unique index.
    pub async fn put_domain_config(&self, row: &DomainConfigRow) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        lock_domains(&mut tx).await?;
        if q("SELECT domain FROM domain_alias WHERE alias=?1")
            .bind(&row.name)
            .fetch_opt_scalar_string_tx(&mut tx)
            .await?
            .is_some()
        {
            return Err(StoreError::Corrupt(
                "domain name is already an alias".into(),
            ));
        }
        q("INSERT INTO domains (name, upstream_json, allowlist, blocklist, enabled, preferred, settings_json, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7) ON CONFLICT(name) DO UPDATE SET upstream_json=excluded.upstream_json, allowlist=excluded.allowlist, blocklist=excluded.blocklist, enabled=excluded.enabled, settings_json=excluded.settings_json, updated_at=excluded.updated_at")
            .bind(&row.name).bind(&row.upstream_json).bind(&row.allowlist_json).bind(&row.blocklist_json)
            .bind(i64::from(row.enabled)).bind(&row.settings_json).bind(row.updated_at).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn preferred_domain(&self) -> Result<Option<String>, StoreError> {
        Ok(
            q("SELECT name FROM domains WHERE preferred = 1 AND enabled = 1")
                .fetch_opt_scalar_string(&self.backend)
                .await?,
        )
    }

    /// Atomically replace only the version the caller read; `None` creates only.
    pub async fn compare_and_put_domain_config(
        &self,
        previous: Option<&DomainConfigRow>,
        next: &DomainConfigRow,
    ) -> Result<bool, StoreError> {
        let changed = if let Some(previous) = previous {
            if previous.name != next.name {
                return Ok(false);
            }
            q("UPDATE domains SET upstream_json=?2, allowlist=?3, blocklist=?4, enabled=?5, settings_json=?6, updated_at=?7 WHERE name=?1 AND upstream_json=?8 AND allowlist=?9 AND blocklist=?10 AND enabled=?11 AND preferred=?12 AND settings_json=?13 AND COALESCE(updated_at,0)=?14")
                .bind(&next.name).bind(&next.upstream_json).bind(&next.allowlist_json).bind(&next.blocklist_json)
                .bind(i64::from(next.enabled)).bind(&next.settings_json).bind(next.updated_at)
                .bind(&previous.upstream_json).bind(&previous.allowlist_json).bind(&previous.blocklist_json)
                .bind(i64::from(previous.enabled)).bind(i64::from(previous.preferred)).bind(&previous.settings_json).bind(previous.updated_at)
                .execute(&self.backend).await?
        } else {
            let mut tx = self.backend.begin().await?;
            lock_domains(&mut tx).await?;
            if q("SELECT domain FROM domain_alias WHERE alias=?1")
                .bind(&next.name)
                .fetch_opt_scalar_string_tx(&mut tx)
                .await?
                .is_some()
            {
                return Ok(false);
            }
            let changed = q("INSERT INTO domains (name,upstream_json,allowlist,blocklist,enabled,preferred,settings_json,updated_at) VALUES (?1,?2,?3,?4,?5,0,?6,?7) ON CONFLICT(name) DO NOTHING")
                .bind(&next.name).bind(&next.upstream_json).bind(&next.allowlist_json).bind(&next.blocklist_json)
                .bind(i64::from(next.enabled)).bind(&next.settings_json).bind(next.updated_at).execute_tx(&mut tx).await?;
            tx.commit().await?;
            changed
        };
        Ok(changed == 1)
    }

    pub async fn domain_aliases(&self, name: &str) -> Result<Vec<String>, StoreError> {
        Ok(
            q("SELECT alias FROM domain_alias WHERE domain=?1 ORDER BY alias")
                .bind(name)
                .fetch_all(&self.backend)
                .await?
                .iter()
                .map(|row| row.get_string("alias"))
                .collect(),
        )
    }

    pub async fn delete_domain_config(&self, name: &str, force: bool) -> Result<bool, StoreError> {
        let pattern = format!("%@{name}");
        let changed = q("DELETE FROM domains WHERE name=?1 AND (?3=1 OR (preferred=0 AND NOT EXISTS (SELECT 1 FROM sessions s WHERE lower(s.username) LIKE ?2 OR EXISTS (SELECT 1 FROM domain_alias a WHERE a.domain=?1 AND lower(s.username) LIKE ('%@' || a.alias)))))")
            .bind(name).bind(pattern).bind(i64::from(force)).execute(&self.backend).await?;
        Ok(changed == 1)
    }

    /// Delete the exact revision authorized by the editor, not a concurrent edit.
    pub async fn delete_domain_config_if_unchanged(
        &self,
        row: &DomainConfigRow,
        force: bool,
    ) -> Result<bool, StoreError> {
        let pattern = format!("%@{}", row.name);
        Ok(q("DELETE FROM domains WHERE name=?1 AND upstream_json=?4 AND allowlist=?5 AND blocklist=?6 AND enabled=?7 AND preferred=?8 AND settings_json=?9 AND COALESCE(updated_at,0)=?10 AND (?3=1 OR (preferred=0 AND NOT EXISTS (SELECT 1 FROM sessions s WHERE lower(s.username) LIKE ?2 OR EXISTS (SELECT 1 FROM domain_alias a WHERE a.domain=?1 AND lower(s.username) LIKE ('%@' || a.alias)))))")
            .bind(&row.name).bind(pattern).bind(i64::from(force)).bind(&row.upstream_json)
            .bind(&row.allowlist_json).bind(&row.blocklist_json).bind(i64::from(row.enabled)).bind(i64::from(row.preferred))
            .bind(&row.settings_json).bind(row.updated_at).execute(&self.backend).await? == 1)
    }

    pub async fn domain_hosts(&self, name: &str) -> Result<Vec<String>, StoreError> {
        Ok(
            q("SELECT host FROM domain_host WHERE domain=?1 ORDER BY host")
                .bind(name)
                .fetch_all(&self.backend)
                .await?
                .iter()
                .map(|row| row.get_string("host"))
                .collect(),
        )
    }

    /// Changing preference is atomic, including refusal of a missing/disabled target.
    pub async fn set_preferred_domain(&self, name: Option<&str>) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        lock_domains(&mut tx).await?;
        if let Some(name) = name {
            let exists = q("SELECT name FROM domains WHERE name = ?1 AND enabled = 1")
                .bind(name)
                .fetch_opt_scalar_string_tx(&mut tx)
                .await?;
            if exists.is_none() {
                return Err(StoreError::NotFound);
            }
        }
        q("UPDATE domains SET preferred = 0 WHERE preferred = 1")
            .execute_tx(&mut tx)
            .await?;
        if let Some(name) = name {
            let changed = q("UPDATE domains SET preferred = 1 WHERE name = ?1 AND enabled = 1")
                .bind(name)
                .execute_tx(&mut tx)
                .await?;
            if changed != 1 {
                return Err(StoreError::NotFound);
            }
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn domain_alias_target(&self, alias: &str) -> Result<Option<String>, StoreError> {
        Ok(q("SELECT domain FROM domain_alias WHERE alias = ?1")
            .bind(alias)
            .fetch_opt_scalar_string(&self.backend)
            .await?)
    }

    pub async fn domain_for_host(&self, host: &str) -> Result<Option<String>, StoreError> {
        Ok(q("SELECT domain FROM domain_host WHERE host = ?1")
            .bind(host)
            .fetch_opt_scalar_string(&self.backend)
            .await?)
    }

    async fn map_domain(
        &self,
        key: &str,
        name: &str,
        host: bool,
        remove: bool,
    ) -> Result<(), StoreError> {
        let row = self
            .get_domain_config(name)
            .await?
            .ok_or(StoreError::NotFound)?;
        let aliases = self.domain_aliases(name).await?;
        let hosts = self.domain_hosts(name).await?;
        match self
            .compare_and_map_domain(&row, &aliases, &hosts, key, host, remove, false)
            .await?
        {
            DomainMutation::Applied => Ok(()),
            result => Err(StoreError::Corrupt(format!(
                "domain mapping refused: {result:?}"
            ))),
        }
    }
    pub async fn put_domain_alias(&self, alias: &str, name: &str) -> Result<(), StoreError> {
        self.map_domain(alias, name, false, false).await
    }
    pub async fn put_domain_host(&self, host: &str, name: &str) -> Result<(), StoreError> {
        self.map_domain(host, name, true, false).await
    }
    pub async fn delete_domain_alias(&self, alias: &str, name: &str) -> Result<(), StoreError> {
        self.map_domain(alias, name, false, true).await
    }
    pub async fn delete_domain_host(&self, host: &str, name: &str) -> Result<(), StoreError> {
        self.map_domain(host, name, true, true).await
    }

    pub async fn domain_signin_state(&self, name: &str) -> Result<DomainSigninState, StoreError> {
        let canonical = self
            .domain_alias_target(name)
            .await?
            .unwrap_or_else(|| name.into());
        Ok(match self.get_domain_config(&canonical).await? {
            None => DomainSigninState::Unlisted,
            Some(row) if row.enabled => DomainSigninState::Enabled,
            Some(_) => DomainSigninState::Disabled,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(name: &str) -> DomainConfigRow {
        DomainConfigRow {
            name: name.into(),
            upstream_json: "{}".into(),
            allowlist_json: "[]".into(),
            blocklist_json: "[]".into(),
            enabled: true,
            preferred: false,
            settings_json: "{}".into(),
            updated_at: 1,
        }
    }

    #[tokio::test]
    async fn stale_editor_cannot_delete_concurrent_change_even_with_force() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        let old = row("example.test");
        store.put_domain_config(&old).await.unwrap();
        let mut new = old.clone();
        new.allowlist_json = "[\"alice@example.test\"]".into();
        store.put_domain_config(&new).await.unwrap();
        assert!(
            !store
                .delete_domain_config_if_unchanged(&old, true)
                .await
                .unwrap()
        );
        assert_eq!(
            store.get_domain_config("example.test").await.unwrap(),
            Some(new.clone())
        );
        assert!(
            store
                .delete_domain_config_if_unchanged(&new, true)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn preference_refusal_preserves_current_and_alias_observes_disable() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        let mut first = row("first.test");
        store.put_domain_config(&first).await.unwrap();
        store.put_domain_config(&row("second.test")).await.unwrap();
        store
            .set_preferred_domain(Some("first.test"))
            .await
            .unwrap();
        assert!(
            store
                .set_preferred_domain(Some("absent.test"))
                .await
                .is_err()
        );
        assert_eq!(
            store.preferred_domain().await.unwrap().as_deref(),
            Some("first.test")
        );
        store
            .put_domain_alias("alias.test", "first.test")
            .await
            .unwrap();
        assert_eq!(
            store.domain_signin_state("alias.test").await.unwrap(),
            DomainSigninState::Enabled
        );
        first.enabled = false;
        store.put_domain_config(&first).await.unwrap();
        assert_eq!(
            store.domain_signin_state("alias.test").await.unwrap(),
            DomainSigninState::Disabled
        );
        assert!(store.preferred_domain().await.unwrap().is_none());
        assert!(
            store
                .put_domain_alias("second.test", "first.test")
                .await
                .is_err()
        );
        store
            .put_domain_host("mail.test", "second.test")
            .await
            .unwrap();
        assert_eq!(
            store.domain_for_host("mail.test").await.unwrap().as_deref(),
            Some("second.test")
        );
    }
}
