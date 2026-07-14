//! V7 (0008) admin-config repository methods — additive, dual-backend (t7-e14 MOUNT).
//!
//! e9 filled the `passwd_config` slice (see [`crate::v7`]); the coordinator narrowed
//! the remaining 0008 admin-config persistence to e14 (the MOUNT executor). This
//! module adds the read/write paths the mount needs to build + persist the injected
//! V7 surfaces from the 0008 tables:
//!
//!   * `directory_config` — the priority-ordered LDAP/GAL endpoints e14 maps to a
//!     `mw_directory::Directory` ([`DirectoryConfigRow`]).
//!   * `plugins` / `plugin_grants` — the signed-registry rows e14 seeds the
//!     `mw_plugin::PluginHost` from, and persists approve/enable/grant back to
//!     ([`PluginRow`]).
//!   * `assist_config` — the per-scope Assist gateway config ([`AssistConfigRow`]) and
//!     the append-only, **content-free** `assist_audit` sink.
//!
//! `mw-store` stays free of any `mw-directory`/`mw-plugin`/`mw-assist` dependency: the
//! rows are plain structs carrying opaque JSON columns; the server (e14) maps them to
//! the crate types. Authored in the SQLite `?n` style so they run identically on
//! SQLite or Postgres through the frozen [`crate::backend`] dispatch layer.

use chrono::Utc;

use crate::backend::q;
use crate::{Store, StoreError, seal};

/// A `directory_config` row (0008). `attr_map_json` is the serialized
/// `mw_directory::AttrMap`; `tls` is `'none' | 'starttls' | 'ldaps'`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryConfigRow {
    pub id: String,
    pub url: String,
    pub base_dn: String,
    pub bind_dn: Option<String>,
    pub tls: String,
    pub priority: i64,
    pub attr_map_json: String,
    pub enabled: bool,
}

/// A `plugins` registry row (0008). `signature` is the detached hex signature over
/// the component bytes (stored as its UTF-8 bytes in the BLOB column); the JSON
/// columns mirror the `plugin.toml` manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginRow {
    pub id: String,
    pub name: String,
    pub version: String,
    pub signature_hex: Option<String>,
    pub approved_by: Option<String>,
    pub enabled: bool,
    pub capabilities_json: String,
    pub net_allowlist_json: String,
    pub limits_json: String,
    pub created_at: String,
}

/// A `plugin_grants` row (0008). `account_id` empty ⇒ a deployment-wide grant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginGrantRow {
    pub plugin_id: String,
    pub account_id: String,
    pub capability: String,
    pub granted_by: String,
    pub created_at: String,
}

/// A `bridge_accounts` row (0008, §6.5): which local account is served by which
/// bridge plugin (`plugins.id`). `oauth_ref` is an opaque token reference (the
/// long-lived secret never lives here); `extra_json` carries bridge-specific
/// settings. e14's `load_plugin_backends` reads these at boot to auto-load an
/// approved+enabled bridge component as the account's backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAccountRow {
    pub account_id: String,
    pub bridge_id: String,
    pub oauth_ref: Option<String>,
    pub extra_json: String,
}

/// An `assist_config` row (0008). All-JSON except the queryable `enabled` flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistConfigRow {
    pub scope: String,
    pub adapters_json: String,
    pub capability_grants_json: String,
    pub data_ceilings_json: String,
    pub enabled: bool,
}

impl Store {
    // ── directory_config ─────────────────────────────────────────────────────

    /// Read every directory endpoint, priority-ordered (lower = queried first).
    pub async fn list_directory_config(&self) -> Result<Vec<DirectoryConfigRow>, StoreError> {
        let rows = q(
            "SELECT id, url, base_dn, bind_dn, tls, priority, attr_map, enabled
                      FROM directory_config ORDER BY priority ASC, id ASC",
        )
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| DirectoryConfigRow {
                id: r.get_string("id"),
                url: r.get_string("url"),
                base_dn: r.get_string("base_dn"),
                bind_dn: r.get_opt_string("bind_dn"),
                tls: r.get_string("tls"),
                priority: r.get_i64("priority"),
                attr_map_json: r.get_string("attr_map"),
                enabled: r.get_i64("enabled") != 0,
            })
            .collect())
    }

    /// Upsert a directory endpoint (by `id`).
    pub async fn put_directory_config(&self, row: &DirectoryConfigRow) -> Result<(), StoreError> {
        q("INSERT INTO directory_config (id, url, base_dn, bind_dn, tls, priority, attr_map, enabled)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
           ON CONFLICT(id) DO UPDATE SET
             url = excluded.url, base_dn = excluded.base_dn, bind_dn = excluded.bind_dn,
             tls = excluded.tls, priority = excluded.priority, attr_map = excluded.attr_map,
             enabled = excluded.enabled")
        .bind(&row.id)
        .bind(&row.url)
        .bind(&row.base_dn)
        .bind(row.bind_dn.clone())
        .bind(&row.tls)
        .bind(row.priority)
        .bind(&row.attr_map_json)
        .bind(i64::from(row.enabled))
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    // ── plugins / plugin_grants ──────────────────────────────────────────────

    /// Read the whole plugin registry (id-ordered).
    pub async fn list_plugins(&self) -> Result<Vec<PluginRow>, StoreError> {
        let rows = q("SELECT id, name, version, signature, approved_by, enabled,
                             capabilities, net_allowlist, limits, created_at
                      FROM plugins ORDER BY id ASC")
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| PluginRow {
                id: r.get_string("id"),
                name: r.get_string("name"),
                version: r.get_string("version"),
                signature_hex: r
                    .get_opt_blob("signature")
                    .and_then(|b| String::from_utf8(b).ok()),
                approved_by: r.get_opt_string("approved_by"),
                enabled: r.get_i64("enabled") != 0,
                capabilities_json: r.get_string("capabilities"),
                net_allowlist_json: r.get_string("net_allowlist"),
                limits_json: r.get_string("limits"),
                created_at: r.get_string("created_at"),
            })
            .collect())
    }

    /// Upsert a plugin registry row (by `id`), preserving the created_at on conflict.
    pub async fn put_plugin(&self, row: &PluginRow) -> Result<(), StoreError> {
        let sig = row.signature_hex.clone().map(String::into_bytes);
        q(
            "INSERT INTO plugins (id, name, version, signature, approved_by, enabled,
                                capabilities, net_allowlist, limits, created_at)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
           ON CONFLICT(id) DO UPDATE SET
             name = excluded.name, version = excluded.version, signature = excluded.signature,
             approved_by = excluded.approved_by, enabled = excluded.enabled,
             capabilities = excluded.capabilities, net_allowlist = excluded.net_allowlist,
             limits = excluded.limits",
        )
        .bind(&row.id)
        .bind(&row.name)
        .bind(&row.version)
        .bind(sig)
        .bind(row.approved_by.clone())
        .bind(i64::from(row.enabled))
        .bind(&row.capabilities_json)
        .bind(&row.net_allowlist_json)
        .bind(&row.limits_json)
        .bind(&row.created_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Record an admin approval on a plugin.
    pub async fn set_plugin_approved(&self, id: &str, admin: &str) -> Result<(), StoreError> {
        q("UPDATE plugins SET approved_by = ?2 WHERE id = ?1")
            .bind(id)
            .bind(admin)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Enable/disable a plugin.
    pub async fn set_plugin_enabled(&self, id: &str, enabled: bool) -> Result<(), StoreError> {
        q("UPDATE plugins SET enabled = ?2 WHERE id = ?1")
            .bind(id)
            .bind(i64::from(enabled))
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Persist a capability grant (idempotent on the composite PK).
    pub async fn put_plugin_grant(&self, row: &PluginGrantRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO plugin_grants (plugin_id, account_id, capability, granted_by, created_at)
           VALUES (?1, ?2, ?3, ?4, ?5)
           ON CONFLICT(plugin_id, account_id, capability) DO UPDATE SET
             granted_by = excluded.granted_by, created_at = excluded.created_at",
        )
        .bind(&row.plugin_id)
        .bind(&row.account_id)
        .bind(&row.capability)
        .bind(&row.granted_by)
        .bind(&row.created_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// The distinct capabilities granted to a plugin (any account scope).
    pub async fn plugin_grants(&self, plugin_id: &str) -> Result<Vec<String>, StoreError> {
        let rows = q("SELECT capability FROM plugin_grants WHERE plugin_id = ?1")
            .bind(plugin_id)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(|r| r.get_string("capability")).collect())
    }

    // ── bridge_accounts ──────────────────────────────────────────────────────

    /// Read every bridge-account binding (account ↔ bridge plugin, §6.5),
    /// account-id-ordered. e14 loads a bridge component per binding at boot.
    pub async fn list_bridge_accounts(&self) -> Result<Vec<BridgeAccountRow>, StoreError> {
        let rows = q("SELECT account_id, bridge_id, oauth_ref, extra
                      FROM bridge_accounts ORDER BY account_id ASC")
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| BridgeAccountRow {
                account_id: r.get_string("account_id"),
                bridge_id: r.get_string("bridge_id"),
                oauth_ref: r.get_opt_string("oauth_ref"),
                extra_json: r.get_string("extra"),
            })
            .collect())
    }

    /// Upsert a bridge-account binding (by `account_id`).
    pub async fn put_bridge_account(&self, row: &BridgeAccountRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO bridge_accounts (account_id, bridge_id, oauth_ref, extra)
           VALUES (?1, ?2, ?3, ?4)
           ON CONFLICT(account_id) DO UPDATE SET
             bridge_id = excluded.bridge_id, oauth_ref = excluded.oauth_ref,
             extra = excluded.extra",
        )
        .bind(&row.account_id)
        .bind(&row.bridge_id)
        .bind(row.oauth_ref.clone())
        .bind(&row.extra_json)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    // ── assist_config + assist_audit ─────────────────────────────────────────

    /// Read the Assist config for a scope (`'deployment'` or `'user:<account_id>'`).
    pub async fn get_assist_config(
        &self,
        scope: &str,
    ) -> Result<Option<AssistConfigRow>, StoreError> {
        let row = q(
            "SELECT scope, adapters, capability_grants, data_ceilings, enabled
                     FROM assist_config WHERE scope = ?1",
        )
        .bind(scope)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.map(|r| AssistConfigRow {
            scope: r.get_string("scope"),
            adapters_json: r.get_string("adapters"),
            capability_grants_json: r.get_string("capability_grants"),
            data_ceilings_json: r.get_string("data_ceilings"),
            enabled: r.get_i64("enabled") != 0,
        }))
    }

    /// Upsert the Assist config for a scope.
    pub async fn put_assist_config(&self, row: &AssistConfigRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO assist_config (scope, adapters, capability_grants, data_ceilings, enabled)
           VALUES (?1, ?2, ?3, ?4, ?5)
           ON CONFLICT(scope) DO UPDATE SET
             adapters = excluded.adapters, capability_grants = excluded.capability_grants,
             data_ceilings = excluded.data_ceilings, enabled = excluded.enabled",
        )
        .bind(&row.scope)
        .bind(&row.adapters_json)
        .bind(&row.capability_grants_json)
        .bind(&row.data_ceilings_json)
        .bind(i64::from(row.enabled))
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Append a **content-free** Assist audit row (capability + scope summary +
    /// endpoint host only — NEVER mail content, §14/R4).
    pub async fn put_assist_audit(
        &self,
        actor: &str,
        capability: &str,
        scope_summary: &str,
        endpoint_host: &str,
    ) -> Result<(), StoreError> {
        q(
            "INSERT INTO assist_audit (id, ts, actor, capability, scope_summary, endpoint_host)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(seal::random_token())
        .bind(Utc::now().to_rfc3339())
        .bind(actor)
        .bind(capability)
        .bind(scope_summary)
        .bind(endpoint_host)
        .execute(&self.backend)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    #[tokio::test]
    async fn directory_config_round_trips_priority_ordered() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        assert!(store.list_directory_config().await.unwrap().is_empty());
        store
            .put_directory_config(&DirectoryConfigRow {
                id: "b".into(),
                url: "ldaps://dir2".into(),
                base_dn: "dc=x".into(),
                bind_dn: None,
                tls: "ldaps".into(),
                priority: 5,
                attr_map_json: "{}".into(),
                enabled: true,
            })
            .await
            .unwrap();
        store
            .put_directory_config(&DirectoryConfigRow {
                id: "a".into(),
                url: "ldap://dir1".into(),
                base_dn: "dc=x".into(),
                bind_dn: Some("cn=svc".into()),
                tls: "starttls".into(),
                priority: 1,
                attr_map_json: r#"{"user_cert":"userCertificate"}"#.into(),
                enabled: true,
            })
            .await
            .unwrap();
        let rows = store.list_directory_config().await.unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, "a", "lower priority queried first");
        assert_eq!(rows[0].bind_dn.as_deref(), Some("cn=svc"));
    }

    #[tokio::test]
    async fn plugin_registry_round_trips_and_approves() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        store
            .put_plugin(&PluginRow {
                id: "languagetool".into(),
                name: "LanguageTool".into(),
                version: "1".into(),
                signature_hex: Some("deadbeef".into()),
                approved_by: None,
                enabled: false,
                capabilities_json: r#"["Net"]"#.into(),
                net_allowlist_json: r#"["api.languagetool.org"]"#.into(),
                limits_json: "{}".into(),
                created_at: "2026-07-14T00:00:00Z".into(),
            })
            .await
            .unwrap();
        store
            .set_plugin_approved("languagetool", "admin@x")
            .await
            .unwrap();
        store
            .set_plugin_enabled("languagetool", true)
            .await
            .unwrap();
        store
            .put_plugin_grant(&PluginGrantRow {
                plugin_id: "languagetool".into(),
                account_id: String::new(),
                capability: "Net".into(),
                granted_by: "admin@x".into(),
                created_at: "2026-07-14T00:00:00Z".into(),
            })
            .await
            .unwrap();
        let rows = store.list_plugins().await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].approved_by.as_deref(), Some("admin@x"));
        assert!(rows[0].enabled);
        assert_eq!(rows[0].signature_hex.as_deref(), Some("deadbeef"));
        assert_eq!(
            store.plugin_grants("languagetool").await.unwrap(),
            vec!["Net"]
        );
    }

    #[tokio::test]
    async fn assist_config_and_content_free_audit() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        assert!(
            store
                .get_assist_config("deployment")
                .await
                .unwrap()
                .is_none()
        );
        store
            .put_assist_config(&AssistConfigRow {
                scope: "deployment".into(),
                adapters_json: r#"{"OpenAiCompatible":{}}"#.into(),
                capability_grants_json: r#"["Summarize"]"#.into(),
                data_ceilings_json: "{}".into(),
                enabled: true,
            })
            .await
            .unwrap();
        let got = store
            .get_assist_config("deployment")
            .await
            .unwrap()
            .unwrap();
        assert!(got.enabled);
        assert_eq!(got.capability_grants_json, r#"["Summarize"]"#);
        // The audit row carries capability + scope summary + host — never content.
        store
            .put_assist_audit("acct", "summarize", "accounts=1", "api.openai.com")
            .await
            .unwrap();
        let n = q("SELECT COUNT(*) FROM assist_audit")
            .fetch_scalar_i64(store.backend())
            .await
            .unwrap();
        assert_eq!(n, 1);
    }
}
