//! V9 (0010) deferred-tail repository methods — additive, dual-backend (t10-e0,
//! plan §2.4). New `Store` methods + row structs over the 0010 `ui_plugins` /
//! `ui_plugin_grants` / `masked_email` / `oauth_dcr` / `oauth_client_meta` tables,
//! reusing the frozen dual-backend query layer; no existing query or public item is
//! touched.
//!
//! These are the persistence seams the tail owners fill:
//!   * `ui_plugins` / `ui_plugin_grants` → e11 (UI-plugin registry + admin approval).
//!   * `masked_email` → e7 (alias lifecycle).
//!   * `oauth_dcr` (policy) + `oauth_client_meta` → e8 (RFC 7591 DCR).
//!
//! No column here holds a secret: `signature` is a public detached signature stored
//! verbatim; the DCR registration-access-token is stored ONLY as a hash the caller
//! computes. `mw-store` stays free of any `mw-plugin`/`mw-oauth` dependency — the
//! rows carry plain fields + opaque JSON strings the server maps to typed models.

use crate::backend::q;
use crate::{Store, StoreError};

/// The singleton id for the [`OAuthDcrPolicyRow`] (there is one DCR policy).
pub const OAUTH_DCR_POLICY_ID: &str = "default";

// ── ui_plugins ────────────────────────────────────────────────────────────────

/// A `ui_plugins` row (0010). `manifest_json`/`capabilities_json`/
/// `extension_points_json` are opaque JSON; `signature` is the (public) detached
/// signature over the bundle, `None` when unsigned; `approved_by` is the admin
/// operator id, `None` until approved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiPluginRow {
    pub id: String,
    pub name: String,
    pub version: String,
    pub manifest_json: String,
    pub signature: Option<Vec<u8>>,
    pub approved_by: Option<String>,
    pub enabled: bool,
    pub capabilities_json: String,
    pub extension_points_json: String,
    pub created_at: String,
}

/// A `ui_plugin_grants` row (0010): one granted capability for a plugin. `params_json`
/// carries the capability's scoped config (e.g. the `net:host-allowlist` host set).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiPluginGrantRow {
    pub plugin_id: String,
    pub capability: String,
    pub params_json: String,
    pub granted_by: String,
    pub granted_at: String,
}

/// A `masked_email` row (0010). `state` is `'enabled'|'disabled'|'deleted'`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaskedEmailRow {
    pub id: String,
    pub account_id: String,
    pub alias_addr: String,
    pub target_desc: String,
    pub state: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

/// The singleton `oauth_dcr` policy row (0010). DEFAULT DISABLED. `default_scope_json`
/// is the serialized `mw-oauth` Scope granted to DCR-issued clients (no escalation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthDcrPolicyRow {
    pub enabled: bool,
    pub require_initial_access_token: bool,
    pub allowed_redirect_host_suffixes_json: String,
    pub default_scope_json: String,
    pub updated_at: String,
}

/// An `oauth_client_meta` row (0010): RFC 7591 metadata for a DCR-issued client. The
/// client itself lives in the 0007 `oauth_clients` table. `registration_access_token_hash`
/// is a HASH of the registration-access-token (never the raw token).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthClientMetaRow {
    pub client_id: String,
    pub registration_access_token_hash: Option<String>,
    pub software_id: Option<String>,
    pub software_version: Option<String>,
    pub contacts_json: String,
    pub created_via: String,
    pub created_at: String,
}

impl Store {
    // ── ui_plugins ──────────────────────────────────────────────────────────

    fn map_ui_plugin_row(r: &crate::Row) -> UiPluginRow {
        UiPluginRow {
            id: r.get_string("id"),
            name: r.get_string("name"),
            version: r.get_string("version"),
            manifest_json: r.get_string("manifest"),
            signature: r.get_opt_blob("signature"),
            approved_by: r.get_opt_string("approved_by"),
            enabled: r.get_i64("enabled") != 0,
            capabilities_json: r.get_string("capabilities"),
            extension_points_json: r.get_string("extension_points"),
            created_at: r.get_string("created_at"),
        }
    }

    /// Upsert a UI plugin by `id`. `created_at` is preserved on conflict.
    pub async fn put_ui_plugin(&self, row: &UiPluginRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO ui_plugins (id, name, version, manifest, signature, approved_by,
                                     enabled, capabilities, extension_points, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name, version = excluded.version, manifest = excluded.manifest,
               signature = excluded.signature, approved_by = excluded.approved_by,
               enabled = excluded.enabled, capabilities = excluded.capabilities,
               extension_points = excluded.extension_points",
        )
        .bind(&row.id)
        .bind(&row.name)
        .bind(&row.version)
        .bind(&row.manifest_json)
        .bind(row.signature.clone())
        .bind(row.approved_by.clone())
        .bind(i64::from(row.enabled))
        .bind(&row.capabilities_json)
        .bind(&row.extension_points_json)
        .bind(&row.created_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Read one UI plugin by id, or `None`.
    pub async fn get_ui_plugin(&self, id: &str) -> Result<Option<UiPluginRow>, StoreError> {
        let row = q(
            "SELECT id, name, version, manifest, signature, approved_by, enabled,
                    capabilities, extension_points, created_at
             FROM ui_plugins WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.map(|r| Self::map_ui_plugin_row(&r)))
    }

    /// Every UI plugin, id-ordered.
    pub async fn list_ui_plugins(&self) -> Result<Vec<UiPluginRow>, StoreError> {
        let rows = q(
            "SELECT id, name, version, manifest, signature, approved_by, enabled,
                    capabilities, extension_points, created_at
             FROM ui_plugins ORDER BY id ASC",
        )
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(Self::map_ui_plugin_row).collect())
    }

    /// Flip a UI plugin's `enabled` flag (idempotent).
    pub async fn set_ui_plugin_enabled(&self, id: &str, enabled: bool) -> Result<(), StoreError> {
        q("UPDATE ui_plugins SET enabled = ?2 WHERE id = ?1")
            .bind(id)
            .bind(i64::from(enabled))
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Delete a UI plugin and all its grants (idempotent).
    pub async fn delete_ui_plugin(&self, id: &str) -> Result<(), StoreError> {
        q("DELETE FROM ui_plugin_grants WHERE plugin_id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        q("DELETE FROM ui_plugins WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ── ui_plugin_grants ──────────────────────────────────────────────────────

    /// Upsert a capability grant for a plugin (by `plugin_id` + `capability`).
    pub async fn put_ui_plugin_grant(&self, row: &UiPluginGrantRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO ui_plugin_grants (plugin_id, capability, params, granted_by, granted_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(plugin_id, capability) DO UPDATE SET
               params = excluded.params, granted_by = excluded.granted_by,
               granted_at = excluded.granted_at",
        )
        .bind(&row.plugin_id)
        .bind(&row.capability)
        .bind(&row.params_json)
        .bind(&row.granted_by)
        .bind(&row.granted_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Every grant for a plugin, capability-ordered.
    pub async fn list_ui_plugin_grants(
        &self,
        plugin_id: &str,
    ) -> Result<Vec<UiPluginGrantRow>, StoreError> {
        let rows = q(
            "SELECT plugin_id, capability, params, granted_by, granted_at
             FROM ui_plugin_grants WHERE plugin_id = ?1 ORDER BY capability ASC",
        )
        .bind(plugin_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| UiPluginGrantRow {
                plugin_id: r.get_string("plugin_id"),
                capability: r.get_string("capability"),
                params_json: r.get_string("params"),
                granted_by: r.get_string("granted_by"),
                granted_at: r.get_string("granted_at"),
            })
            .collect())
    }

    /// Revoke one capability grant (idempotent).
    pub async fn delete_ui_plugin_grant(
        &self,
        plugin_id: &str,
        capability: &str,
    ) -> Result<(), StoreError> {
        q("DELETE FROM ui_plugin_grants WHERE plugin_id = ?1 AND capability = ?2")
            .bind(plugin_id)
            .bind(capability)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ── masked_email ──────────────────────────────────────────────────────────

    fn map_masked_row(r: &crate::Row) -> MaskedEmailRow {
        MaskedEmailRow {
            id: r.get_string("id"),
            account_id: r.get_string("account_id"),
            alias_addr: r.get_string("alias_addr"),
            target_desc: r.get_string("target_desc"),
            state: r.get_string("state"),
            created_at: r.get_string("created_at"),
            last_used_at: r.get_opt_string("last_used_at"),
        }
    }

    /// Upsert a masked-email alias by `id`. `created_at` is preserved on conflict.
    pub async fn put_masked_email(&self, row: &MaskedEmailRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO masked_email (id, account_id, alias_addr, target_desc, state,
                                       created_at, last_used_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
               account_id = excluded.account_id, alias_addr = excluded.alias_addr,
               target_desc = excluded.target_desc, state = excluded.state,
               last_used_at = excluded.last_used_at",
        )
        .bind(&row.id)
        .bind(&row.account_id)
        .bind(&row.alias_addr)
        .bind(&row.target_desc)
        .bind(&row.state)
        .bind(&row.created_at)
        .bind(row.last_used_at.clone())
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Read one masked-email alias by id, or `None`.
    pub async fn get_masked_email(&self, id: &str) -> Result<Option<MaskedEmailRow>, StoreError> {
        let row = q(
            "SELECT id, account_id, alias_addr, target_desc, state, created_at, last_used_at
             FROM masked_email WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.map(|r| Self::map_masked_row(&r)))
    }

    /// Every masked-email alias for an account, newest-first.
    pub async fn list_masked_email(
        &self,
        account_id: &str,
    ) -> Result<Vec<MaskedEmailRow>, StoreError> {
        let rows = q(
            "SELECT id, account_id, alias_addr, target_desc, state, created_at, last_used_at
             FROM masked_email WHERE account_id = ?1 ORDER BY created_at DESC",
        )
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(Self::map_masked_row).collect())
    }

    /// Set an alias's lifecycle `state` (`'enabled'|'disabled'|'deleted'`; idempotent).
    pub async fn set_masked_email_state(&self, id: &str, state: &str) -> Result<(), StoreError> {
        q("UPDATE masked_email SET state = ?2 WHERE id = ?1")
            .bind(id)
            .bind(state)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Record a `last_used_at` timestamp for an alias (idempotent).
    pub async fn touch_masked_email(&self, id: &str, at: &str) -> Result<(), StoreError> {
        q("UPDATE masked_email SET last_used_at = ?2 WHERE id = ?1")
            .bind(id)
            .bind(at)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ── oauth_dcr (singleton policy) ──────────────────────────────────────────

    /// The DCR policy, or `None` when never configured (⇒ DCR is disabled).
    pub async fn get_oauth_dcr_policy(&self) -> Result<Option<OAuthDcrPolicyRow>, StoreError> {
        let row = q(
            "SELECT enabled, require_initial_access_token, allowed_redirect_host_suffixes,
                    default_scope, updated_at
             FROM oauth_dcr WHERE id = ?1",
        )
        .bind(OAUTH_DCR_POLICY_ID)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.map(|r| OAuthDcrPolicyRow {
            enabled: r.get_i64("enabled") != 0,
            require_initial_access_token: r.get_i64("require_initial_access_token") != 0,
            allowed_redirect_host_suffixes_json: r.get_string("allowed_redirect_host_suffixes"),
            default_scope_json: r.get_string("default_scope"),
            updated_at: r.get_string("updated_at"),
        }))
    }

    /// Upsert the singleton DCR policy.
    pub async fn put_oauth_dcr_policy(&self, row: &OAuthDcrPolicyRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO oauth_dcr (id, enabled, require_initial_access_token,
                                    allowed_redirect_host_suffixes, default_scope, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
               enabled = excluded.enabled,
               require_initial_access_token = excluded.require_initial_access_token,
               allowed_redirect_host_suffixes = excluded.allowed_redirect_host_suffixes,
               default_scope = excluded.default_scope, updated_at = excluded.updated_at",
        )
        .bind(OAUTH_DCR_POLICY_ID)
        .bind(i64::from(row.enabled))
        .bind(i64::from(row.require_initial_access_token))
        .bind(&row.allowed_redirect_host_suffixes_json)
        .bind(&row.default_scope_json)
        .bind(&row.updated_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    // ── oauth_client_meta ─────────────────────────────────────────────────────

    /// Upsert DCR client metadata by `client_id`. `created_at` is preserved on conflict.
    pub async fn put_oauth_client_meta(&self, row: &OAuthClientMetaRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO oauth_client_meta (client_id, registration_access_token_hash,
                                            software_id, software_version, contacts,
                                            created_via, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(client_id) DO UPDATE SET
               registration_access_token_hash = excluded.registration_access_token_hash,
               software_id = excluded.software_id, software_version = excluded.software_version,
               contacts = excluded.contacts, created_via = excluded.created_via",
        )
        .bind(&row.client_id)
        .bind(row.registration_access_token_hash.clone())
        .bind(row.software_id.clone())
        .bind(row.software_version.clone())
        .bind(&row.contacts_json)
        .bind(&row.created_via)
        .bind(&row.created_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Read DCR client metadata by id, or `None`.
    pub async fn get_oauth_client_meta(
        &self,
        client_id: &str,
    ) -> Result<Option<OAuthClientMetaRow>, StoreError> {
        let row = q(
            "SELECT client_id, registration_access_token_hash, software_id, software_version,
                    contacts, created_via, created_at
             FROM oauth_client_meta WHERE client_id = ?1",
        )
        .bind(client_id)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.map(|r| OAuthClientMetaRow {
            client_id: r.get_string("client_id"),
            registration_access_token_hash: r.get_opt_string("registration_access_token_hash"),
            software_id: r.get_opt_string("software_id"),
            software_version: r.get_opt_string("software_version"),
            contacts_json: r.get_string("contacts"),
            created_via: r.get_string("created_via"),
            created_at: r.get_string("created_at"),
        }))
    }

    /// Delete DCR client metadata by id (idempotent; the 0007 `oauth_clients` row is
    /// deleted separately by the caller).
    pub async fn delete_oauth_client_meta(&self, client_id: &str) -> Result<(), StoreError> {
        q("DELETE FROM oauth_client_meta WHERE client_id = ?1")
            .bind(client_id)
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
    async fn ui_plugin_and_grants_round_trip() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        assert!(store.get_ui_plugin("p1").await.unwrap().is_none());

        let row = UiPluginRow {
            id: "p1".into(),
            name: "Snooze Buttons".into(),
            version: "1.0.0".into(),
            manifest_json: r#"{"id":"p1"}"#.into(),
            signature: Some(vec![1, 2, 3]),
            approved_by: None,
            enabled: false,
            capabilities_json: r#"["ui:message-toolbar"]"#.into(),
            extension_points_json: r#"["message-toolbar"]"#.into(),
            created_at: "2026-07-14T00:00:00Z".into(),
        };
        store.put_ui_plugin(&row).await.unwrap();
        assert_eq!(store.get_ui_plugin("p1").await.unwrap().unwrap(), row);

        // Deny-by-default until approved + enabled.
        store.set_ui_plugin_enabled("p1", true).await.unwrap();
        assert!(store.get_ui_plugin("p1").await.unwrap().unwrap().enabled);
        assert_eq!(store.list_ui_plugins().await.unwrap().len(), 1);

        let grant = UiPluginGrantRow {
            plugin_id: "p1".into(),
            capability: "net:host-allowlist".into(),
            params_json: r#"{"hosts":["api.example.com"]}"#.into(),
            granted_by: "admin".into(),
            granted_at: "2026-07-14T00:00:00Z".into(),
        };
        store.put_ui_plugin_grant(&grant).await.unwrap();
        assert_eq!(
            store.list_ui_plugin_grants("p1").await.unwrap(),
            vec![grant]
        );

        // Delete cascades the grants.
        store.delete_ui_plugin("p1").await.unwrap();
        assert!(store.get_ui_plugin("p1").await.unwrap().is_none());
        assert!(store.list_ui_plugin_grants("p1").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn masked_email_lifecycle() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let row = MaskedEmailRow {
            id: "m1".into(),
            account_id: "a1".into(),
            alias_addr: "x7f2@masked.example".into(),
            target_desc: "shopping".into(),
            state: "enabled".into(),
            created_at: "2026-07-14T00:00:00Z".into(),
            last_used_at: None,
        };
        store.put_masked_email(&row).await.unwrap();
        assert_eq!(store.get_masked_email("m1").await.unwrap().unwrap(), row);
        assert_eq!(store.list_masked_email("a1").await.unwrap().len(), 1);

        store
            .set_masked_email_state("m1", "disabled")
            .await
            .unwrap();
        store
            .touch_masked_email("m1", "2026-07-15T00:00:00Z")
            .await
            .unwrap();
        let got = store.get_masked_email("m1").await.unwrap().unwrap();
        assert_eq!(got.state, "disabled");
        assert_eq!(got.last_used_at.as_deref(), Some("2026-07-15T00:00:00Z"));
        assert!(store.list_masked_email("nope").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn oauth_dcr_policy_and_client_meta() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        // Absent ⇒ DCR disabled.
        assert!(store.get_oauth_dcr_policy().await.unwrap().is_none());

        let policy = OAuthDcrPolicyRow {
            enabled: true,
            require_initial_access_token: false,
            allowed_redirect_host_suffixes_json: r#"["example.com"]"#.into(),
            default_scope_json: r#"{"read":true}"#.into(),
            updated_at: "2026-07-14T00:00:00Z".into(),
        };
        store.put_oauth_dcr_policy(&policy).await.unwrap();
        assert_eq!(store.get_oauth_dcr_policy().await.unwrap().unwrap(), policy);

        let meta = OAuthClientMetaRow {
            client_id: "c1".into(),
            registration_access_token_hash: Some("hash".into()),
            software_id: Some("sw".into()),
            software_version: None,
            contacts_json: r#"["ops@example.com"]"#.into(),
            created_via: "dcr".into(),
            created_at: "2026-07-14T00:00:00Z".into(),
        };
        store.put_oauth_client_meta(&meta).await.unwrap();
        assert_eq!(
            store.get_oauth_client_meta("c1").await.unwrap().unwrap(),
            meta
        );
        store.delete_oauth_client_meta("c1").await.unwrap();
        assert!(store.get_oauth_client_meta("c1").await.unwrap().is_none());
    }
}
