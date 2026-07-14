//! V6 (0007) repository methods — additive, dual-backend (t6-e11 MOUNT).
//!
//! The 0007 tables (`api_keys`/`oauth_*`/`webhooks`/`audit_log`/`domains`/
//! `quotas`/`admin_users`/`admin_sessions`/`zeroaccess_accounts`/`cache_scope`)
//! were created by e0's migration but their typed repo methods were deliberately
//! deferred by the Batch-B crates (e3/e5/e9 backed their persistence traits with
//! in-memory doubles while `mw-store` was mid-refactor). This module fills that
//! gap so the MOUNT executor (e11) can back the `OAuthStore`/`AdminBackend`/
//! `WebhookRegistry` seams over the real store.
//!
//! It is **purely additive**: it adds new `Store` methods + plain row structs and
//! touches no existing query or the frozen public API. Every query is authored in
//! the SQLite `?n` style and runs identically on SQLite or Postgres through the
//! frozen [`crate::backend`] dispatch layer (so the mounted surface is
//! backend-parity-identical for free).
//!
//! Sealed columns (`webhooks.secret_sealed`, `zeroaccess_accounts.wrapped_root_key`)
//! are stored as opaque bytes; the caller (e11) seals/unseals via [`ServerKey`].

use crate::backend::q;
use crate::{Store, StoreError};

// ─── Row structs (plain data; the mw-server adapters map to the trait types) ──

/// An `api_keys` row (0007). The scope's ip-allowlist/expiry/rate-limit live
/// inside `scopes_json`; the dedicated columns are left NULL (informational).
#[derive(Debug, Clone)]
pub struct ApiKeyRow {
    pub id: String,
    pub key_prefix: String,
    pub key_hash: String,
    pub account_id: String,
    pub scopes_json: String,
    pub unattended_send: bool,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub revoked_at: Option<String>,
}

/// An `oauth_clients` row (0007).
#[derive(Debug, Clone)]
pub struct OAuthClientRow {
    pub client_id: String,
    pub name: String,
    pub redirect_uris_json: String,
    pub approved_by: String,
    pub created_at: String,
}

/// An `oauth_tokens` row (0007).
#[derive(Debug, Clone)]
pub struct OAuthTokenRow {
    pub token_hash: String,
    pub client_id: String,
    pub account_id: String,
    pub scopes_json: String,
    pub resource: Option<String>,
    pub kind: String,
    pub expires_at: String,
    pub created_at: String,
    pub revoked_at: Option<String>,
    pub pkce_challenge: Option<String>,
}

/// A `webhooks` row (0007). `secret_sealed` is opaque sealed bytes.
#[derive(Debug, Clone)]
pub struct WebhookRow {
    pub id: String,
    pub account_id: String,
    pub url: String,
    pub secret_sealed: Vec<u8>,
    pub event_filter_json: String,
    pub created_at: String,
}

/// An `audit_log` row (0007). Append-only — no update/delete method exists.
#[derive(Debug, Clone)]
pub struct AuditRow {
    pub id: String,
    pub ts: String,
    pub actor: String,
    pub actor_kind: String,
    pub action: String,
    pub target: Option<String>,
    pub detail_json: String,
    pub ip: Option<String>,
}

/// A `domains` row (0007).
#[derive(Debug, Clone)]
pub struct DomainRow {
    pub name: String,
    pub upstream_json: String,
    pub allowlist_json: String,
    pub blocklist_json: String,
}

/// An `admin_users` row (0007). We store the full `account_id` in `username`.
#[derive(Debug, Clone)]
pub struct AdminUserRow {
    pub username: String,
    pub password_hash: Option<String>,
    pub created_at: String,
}

/// A `quotas` row (0007).
#[derive(Debug, Clone, Copy)]
pub struct QuotaRow {
    pub bytes_limit: i64,
    pub msg_limit: i64,
}

/// A `zeroaccess_accounts` row (0007).
#[derive(Debug, Clone)]
pub struct ZeroAccessRow {
    pub account_id: String,
    pub enabled: bool,
    pub wrapped_root_key: Vec<u8>,
    pub kdf_params_json: String,
    pub recovery_wrapped: Option<Vec<u8>>,
    pub paired_devices_json: String,
}

/// A `cache_scope` row (0007).
#[derive(Debug, Clone)]
pub struct CacheScopeRow {
    pub class: String,
    pub layers_json: String,
    pub ttl_secs: i64,
}

fn new_id() -> String {
    crate::seal::random_token()
}

impl Store {
    // ── api_keys ─────────────────────────────────────────────────────────────

    /// Insert (or replace by prefix) a scoped API key.
    pub async fn put_api_key(&self, row: &ApiKeyRow) -> Result<(), StoreError> {
        q("INSERT INTO api_keys (id, key_prefix, key_hash, account_id, scopes, unattended_send, created_at, last_used_at, revoked_at)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
           ON CONFLICT(key_prefix) DO UPDATE SET
             key_hash = excluded.key_hash, account_id = excluded.account_id,
             scopes = excluded.scopes, unattended_send = excluded.unattended_send,
             created_at = excluded.created_at, last_used_at = excluded.last_used_at,
             revoked_at = excluded.revoked_at")
            .bind(&row.id)
            .bind(&row.key_prefix)
            .bind(&row.key_hash)
            .bind(&row.account_id)
            .bind(&row.scopes_json)
            .bind(i64::from(row.unattended_send))
            .bind(&row.created_at)
            .bind(row.last_used_at.clone())
            .bind(row.revoked_at.clone())
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Look up an API key by its public prefix.
    pub async fn get_api_key(&self, prefix: &str) -> Result<Option<ApiKeyRow>, StoreError> {
        let row = q("SELECT id, key_prefix, key_hash, account_id, scopes, unattended_send, created_at, last_used_at, revoked_at
                     FROM api_keys WHERE key_prefix = ?1")
            .bind(prefix)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.map(|r| ApiKeyRow {
            id: r.get_string("id"),
            key_prefix: r.get_string("key_prefix"),
            key_hash: r.get_string("key_hash"),
            account_id: r.get_string("account_id"),
            scopes_json: r.get_string("scopes"),
            unattended_send: r.get_i64("unattended_send") != 0,
            created_at: r.get_string("created_at"),
            last_used_at: r.get_opt_string("last_used_at"),
            revoked_at: r.get_opt_string("revoked_at"),
        }))
    }

    /// Every API key (admin oversight), newest first.
    pub async fn list_api_keys(&self) -> Result<Vec<ApiKeyRow>, StoreError> {
        let rows = q("SELECT id, key_prefix, key_hash, account_id, scopes, unattended_send, created_at, last_used_at, revoked_at
                      FROM api_keys ORDER BY created_at DESC")
            .fetch_all(&self.backend)
            .await?;
        Ok(rows
            .iter()
            .map(|r| ApiKeyRow {
                id: r.get_string("id"),
                key_prefix: r.get_string("key_prefix"),
                key_hash: r.get_string("key_hash"),
                account_id: r.get_string("account_id"),
                scopes_json: r.get_string("scopes"),
                unattended_send: r.get_i64("unattended_send") != 0,
                created_at: r.get_string("created_at"),
                last_used_at: r.get_opt_string("last_used_at"),
                revoked_at: r.get_opt_string("revoked_at"),
            })
            .collect())
    }

    /// Bookkeep last-used time on an API key.
    pub async fn touch_api_key(&self, prefix: &str, at: &str) -> Result<(), StoreError> {
        q("UPDATE api_keys SET last_used_at = ?2 WHERE key_prefix = ?1")
            .bind(prefix)
            .bind(at)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Revoke an API key by prefix (also usable by its `id` via a prefix lookup).
    pub async fn revoke_api_key(&self, prefix: &str, at: &str) -> Result<(), StoreError> {
        q("UPDATE api_keys SET revoked_at = ?2 WHERE key_prefix = ?1")
            .bind(prefix)
            .bind(at)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Revoke an API key by its opaque row id (admin oversight).
    pub async fn revoke_api_key_by_id(&self, id: &str, at: &str) -> Result<(), StoreError> {
        q("UPDATE api_keys SET revoked_at = ?2 WHERE id = ?1")
            .bind(id)
            .bind(at)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ── oauth_clients ────────────────────────────────────────────────────────

    pub async fn put_oauth_client(&self, row: &OAuthClientRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO oauth_clients (client_id, name, redirect_uris, approved_by, created_at)
           VALUES (?1, ?2, ?3, ?4, ?5)
           ON CONFLICT(client_id) DO UPDATE SET
             name = excluded.name, redirect_uris = excluded.redirect_uris,
             approved_by = excluded.approved_by",
        )
        .bind(&row.client_id)
        .bind(&row.name)
        .bind(&row.redirect_uris_json)
        .bind(&row.approved_by)
        .bind(&row.created_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    pub async fn get_oauth_client(
        &self,
        client_id: &str,
    ) -> Result<Option<OAuthClientRow>, StoreError> {
        let row = q(
            "SELECT client_id, name, redirect_uris, approved_by, created_at
                     FROM oauth_clients WHERE client_id = ?1",
        )
        .bind(client_id)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.map(|r| OAuthClientRow {
            client_id: r.get_string("client_id"),
            name: r.get_string("name"),
            redirect_uris_json: r.get_string("redirect_uris"),
            approved_by: r.get_string("approved_by"),
            created_at: r.get_string("created_at"),
        }))
    }

    // ── oauth_tokens ─────────────────────────────────────────────────────────

    pub async fn put_oauth_token(&self, row: &OAuthTokenRow) -> Result<(), StoreError> {
        q("INSERT INTO oauth_tokens (token_hash, client_id, account_id, scopes, resource, kind, expires_at, created_at, revoked_at, pkce_challenge)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
           ON CONFLICT(token_hash) DO UPDATE SET
             revoked_at = excluded.revoked_at")
            .bind(&row.token_hash)
            .bind(&row.client_id)
            .bind(&row.account_id)
            .bind(&row.scopes_json)
            .bind(row.resource.clone())
            .bind(&row.kind)
            .bind(&row.expires_at)
            .bind(&row.created_at)
            .bind(row.revoked_at.clone())
            .bind(row.pkce_challenge.clone())
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    pub async fn get_oauth_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<OAuthTokenRow>, StoreError> {
        let row = q("SELECT token_hash, client_id, account_id, scopes, resource, kind, expires_at, created_at, revoked_at, pkce_challenge
                     FROM oauth_tokens WHERE token_hash = ?1")
            .bind(token_hash)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.map(|r| OAuthTokenRow {
            token_hash: r.get_string("token_hash"),
            client_id: r.get_string("client_id"),
            account_id: r.get_string("account_id"),
            scopes_json: r.get_string("scopes"),
            resource: r.get_opt_string("resource"),
            kind: r.get_string("kind"),
            expires_at: r.get_string("expires_at"),
            created_at: r.get_string("created_at"),
            revoked_at: r.get_opt_string("revoked_at"),
            pkce_challenge: r.get_opt_string("pkce_challenge"),
        }))
    }

    pub async fn revoke_oauth_token(&self, token_hash: &str, at: &str) -> Result<(), StoreError> {
        q("UPDATE oauth_tokens SET revoked_at = ?2 WHERE token_hash = ?1")
            .bind(token_hash)
            .bind(at)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ── webhooks ─────────────────────────────────────────────────────────────

    pub async fn put_webhook(&self, row: &WebhookRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO webhooks (id, account_id, url, secret_sealed, event_filter, created_at)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6)
           ON CONFLICT(id) DO UPDATE SET
             url = excluded.url, secret_sealed = excluded.secret_sealed,
             event_filter = excluded.event_filter",
        )
        .bind(&row.id)
        .bind(&row.account_id)
        .bind(&row.url)
        .bind(&row.secret_sealed)
        .bind(&row.event_filter_json)
        .bind(&row.created_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    pub async fn list_webhooks_for_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<WebhookRow>, StoreError> {
        let rows = q(
            "SELECT id, account_id, url, secret_sealed, event_filter, created_at
                      FROM webhooks WHERE account_id = ?1",
        )
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(webhook_from_row).collect())
    }

    pub async fn list_all_webhooks(&self) -> Result<Vec<WebhookRow>, StoreError> {
        let rows = q(
            "SELECT id, account_id, url, secret_sealed, event_filter, created_at
                      FROM webhooks ORDER BY created_at DESC",
        )
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(webhook_from_row).collect())
    }

    pub async fn delete_webhook(&self, id: &str) -> Result<(), StoreError> {
        q("DELETE FROM webhooks WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ── audit_log (append-only) ──────────────────────────────────────────────

    pub async fn append_audit(&self, row: &AuditRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO audit_log (id, ts, actor, actor_kind, action, target, detail_json, ip)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .bind(&row.id)
        .bind(&row.ts)
        .bind(&row.actor)
        .bind(&row.actor_kind)
        .bind(&row.action)
        .bind(row.target.clone())
        .bind(&row.detail_json)
        .bind(row.ip.clone())
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    pub async fn list_audit(&self, limit: i64) -> Result<Vec<AuditRow>, StoreError> {
        let rows = q(
            "SELECT id, ts, actor, actor_kind, action, target, detail_json, ip
                      FROM audit_log ORDER BY ts DESC, id DESC LIMIT ?1",
        )
        .bind(limit)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| AuditRow {
                id: r.get_string("id"),
                ts: r.get_string("ts"),
                actor: r.get_string("actor"),
                actor_kind: r.get_string("actor_kind"),
                action: r.get_string("action"),
                target: r.get_opt_string("target"),
                detail_json: r.get_string("detail_json"),
                ip: r.get_opt_string("ip"),
            })
            .collect())
    }

    // ── domains ──────────────────────────────────────────────────────────────

    pub async fn upsert_domain(&self, row: &DomainRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO domains (name, upstream_json, allowlist, blocklist)
           VALUES (?1, ?2, ?3, ?4)
           ON CONFLICT(name) DO UPDATE SET
             upstream_json = excluded.upstream_json,
             allowlist = excluded.allowlist, blocklist = excluded.blocklist",
        )
        .bind(&row.name)
        .bind(&row.upstream_json)
        .bind(&row.allowlist_json)
        .bind(&row.blocklist_json)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    pub async fn get_domain(&self, name: &str) -> Result<Option<DomainRow>, StoreError> {
        let row =
            q("SELECT name, upstream_json, allowlist, blocklist FROM domains WHERE name = ?1")
                .bind(name)
                .fetch_optional(&self.backend)
                .await?;
        Ok(row.map(|r| DomainRow {
            name: r.get_string("name"),
            upstream_json: r.get_string("upstream_json"),
            allowlist_json: r.get_string("allowlist"),
            blocklist_json: r.get_string("blocklist"),
        }))
    }

    pub async fn list_domains(&self) -> Result<Vec<DomainRow>, StoreError> {
        let rows = q("SELECT name, upstream_json, allowlist, blocklist FROM domains ORDER BY name")
            .fetch_all(&self.backend)
            .await?;
        Ok(rows
            .iter()
            .map(|r| DomainRow {
                name: r.get_string("name"),
                upstream_json: r.get_string("upstream_json"),
                allowlist_json: r.get_string("allowlist"),
                blocklist_json: r.get_string("blocklist"),
            })
            .collect())
    }

    pub async fn delete_domain(&self, name: &str) -> Result<(), StoreError> {
        q("DELETE FROM domains WHERE name = ?1")
            .bind(name)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ── admin_users (holds the provisioned mail account_id in `username`) ─────

    pub async fn upsert_admin_user(&self, row: &AdminUserRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO admin_users (id, username, password_hash, created_at)
           VALUES (?1, ?2, ?3, ?4)
           ON CONFLICT(username) DO UPDATE SET password_hash = excluded.password_hash",
        )
        .bind(new_id())
        .bind(&row.username)
        .bind(row.password_hash.clone())
        .bind(&row.created_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    pub async fn get_admin_user(&self, username: &str) -> Result<Option<AdminUserRow>, StoreError> {
        let row =
            q("SELECT username, password_hash, created_at FROM admin_users WHERE username = ?1")
                .bind(username)
                .fetch_optional(&self.backend)
                .await?;
        Ok(row.map(|r| AdminUserRow {
            username: r.get_string("username"),
            password_hash: r.get_opt_string("password_hash"),
            created_at: r.get_string("created_at"),
        }))
    }

    pub async fn list_admin_users(&self) -> Result<Vec<AdminUserRow>, StoreError> {
        let rows =
            q("SELECT username, password_hash, created_at FROM admin_users ORDER BY username")
                .fetch_all(&self.backend)
                .await?;
        Ok(rows
            .iter()
            .map(|r| AdminUserRow {
                username: r.get_string("username"),
                password_hash: r.get_opt_string("password_hash"),
                created_at: r.get_string("created_at"),
            })
            .collect())
    }

    // ── admin_sessions (separate admin session domain) ───────────────────────

    pub async fn put_admin_session(
        &self,
        token_hash: &str,
        admin_id: &str,
        now: &str,
    ) -> Result<(), StoreError> {
        q(
            "INSERT INTO admin_sessions (token_hash, admin_id, created_at, last_seen)
           VALUES (?1, ?2, ?3, ?3)
           ON CONFLICT(token_hash) DO UPDATE SET last_seen = excluded.last_seen",
        )
        .bind(token_hash)
        .bind(admin_id)
        .bind(now)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    pub async fn get_admin_session(&self, token_hash: &str) -> Result<Option<String>, StoreError> {
        q("SELECT admin_id FROM admin_sessions WHERE token_hash = ?1")
            .bind(token_hash)
            .fetch_opt_scalar_string(&self.backend)
            .await
            .map_err(StoreError::from)
    }

    pub async fn delete_admin_session(&self, token_hash: &str) -> Result<(), StoreError> {
        q("DELETE FROM admin_sessions WHERE token_hash = ?1")
            .bind(token_hash)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ── quotas ───────────────────────────────────────────────────────────────

    pub async fn set_quota(&self, account_id: &str, quota: QuotaRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO quotas (account_id, bytes_limit, msg_limit) VALUES (?1, ?2, ?3)
           ON CONFLICT(account_id) DO UPDATE SET
             bytes_limit = excluded.bytes_limit, msg_limit = excluded.msg_limit",
        )
        .bind(account_id)
        .bind(quota.bytes_limit)
        .bind(quota.msg_limit)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    pub async fn get_quota(&self, account_id: &str) -> Result<Option<QuotaRow>, StoreError> {
        let row = q("SELECT bytes_limit, msg_limit FROM quotas WHERE account_id = ?1")
            .bind(account_id)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.map(|r| QuotaRow {
            bytes_limit: r.get_i64("bytes_limit"),
            msg_limit: r.get_i64("msg_limit"),
        }))
    }

    // ── sessions (revoke all for an account — reuses the 0001 table) ─────────

    /// Delete every stored session for an account (admin session-revoke). Returns
    /// the number of rows removed.
    pub async fn delete_sessions_for_account(&self, account_id: &str) -> Result<u64, StoreError> {
        let n = q("DELETE FROM sessions WHERE account_id = ?1")
            .bind(account_id)
            .execute(&self.backend)
            .await?;
        Ok(n)
    }

    // ── zeroaccess_accounts ──────────────────────────────────────────────────

    pub async fn get_zeroaccess(
        &self,
        account_id: &str,
    ) -> Result<Option<ZeroAccessRow>, StoreError> {
        let row = q("SELECT account_id, enabled, wrapped_root_key, kdf_params, recovery_wrapped, paired_devices
                     FROM zeroaccess_accounts WHERE account_id = ?1")
            .bind(account_id)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.map(|r| ZeroAccessRow {
            account_id: r.get_string("account_id"),
            enabled: r.get_i64("enabled") != 0,
            wrapped_root_key: r.get_blob("wrapped_root_key"),
            kdf_params_json: r.get_string("kdf_params"),
            recovery_wrapped: r.get_opt_blob("recovery_wrapped"),
            paired_devices_json: r.get_string("paired_devices"),
        }))
    }

    pub async fn upsert_zeroaccess(&self, row: &ZeroAccessRow) -> Result<(), StoreError> {
        q("INSERT INTO zeroaccess_accounts (account_id, enabled, wrapped_root_key, kdf_params, recovery_wrapped, paired_devices)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6)
           ON CONFLICT(account_id) DO UPDATE SET
             enabled = excluded.enabled, wrapped_root_key = excluded.wrapped_root_key,
             kdf_params = excluded.kdf_params, recovery_wrapped = excluded.recovery_wrapped,
             paired_devices = excluded.paired_devices")
            .bind(&row.account_id)
            .bind(i64::from(row.enabled))
            .bind(&row.wrapped_root_key)
            .bind(&row.kdf_params_json)
            .bind(row.recovery_wrapped.clone())
            .bind(&row.paired_devices_json)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    pub async fn set_zeroaccess_enabled(
        &self,
        account_id: &str,
        enabled: bool,
    ) -> Result<(), StoreError> {
        q("UPDATE zeroaccess_accounts SET enabled = ?2 WHERE account_id = ?1")
            .bind(account_id)
            .bind(i64::from(enabled))
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Accounts flagged zero-access (drives the engine posture source).
    pub async fn list_zeroaccess_enabled(&self) -> Result<Vec<String>, StoreError> {
        q("SELECT account_id FROM zeroaccess_accounts WHERE enabled != 0")
            .fetch_all_scalar_string(&self.backend)
            .await
            .map_err(StoreError::from)
    }

    // ── cache_scope ──────────────────────────────────────────────────────────

    pub async fn upsert_cache_scope(&self, row: &CacheScopeRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO cache_scope (class, layers, ttl_secs) VALUES (?1, ?2, ?3)
           ON CONFLICT(class) DO UPDATE SET layers = excluded.layers, ttl_secs = excluded.ttl_secs",
        )
        .bind(&row.class)
        .bind(&row.layers_json)
        .bind(row.ttl_secs)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    pub async fn list_cache_scope(&self) -> Result<Vec<CacheScopeRow>, StoreError> {
        let rows = q("SELECT class, layers, ttl_secs FROM cache_scope ORDER BY class")
            .fetch_all(&self.backend)
            .await?;
        Ok(rows
            .iter()
            .map(|r| CacheScopeRow {
                class: r.get_string("class"),
                layers_json: r.get_string("layers"),
                ttl_secs: r.get_i64("ttl_secs"),
            })
            .collect())
    }
}

fn webhook_from_row(r: &crate::backend::Row) -> WebhookRow {
    WebhookRow {
        id: r.get_string("id"),
        account_id: r.get_string("account_id"),
        url: r.get_string("url"),
        secret_sealed: r.get_blob("secret_sealed"),
        event_filter_json: r.get_string("event_filter"),
        created_at: r.get_string("created_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    #[tokio::test]
    async fn api_key_round_trip_and_revoke() {
        let s = store().await;
        let row = ApiKeyRow {
            id: "k1".into(),
            key_prefix: "abcd1234".into(),
            key_hash: "hash".into(),
            account_id: "a@x".into(),
            scopes_json: "{}".into(),
            unattended_send: true,
            created_at: "2026-01-01T00:00:00Z".into(),
            last_used_at: None,
            revoked_at: None,
        };
        s.put_api_key(&row).await.unwrap();
        let got = s.get_api_key("abcd1234").await.unwrap().unwrap();
        assert_eq!(got.account_id, "a@x");
        assert!(got.unattended_send);
        assert_eq!(s.list_api_keys().await.unwrap().len(), 1);
        s.revoke_api_key("abcd1234", "2026-01-02T00:00:00Z")
            .await
            .unwrap();
        assert!(
            s.get_api_key("abcd1234")
                .await
                .unwrap()
                .unwrap()
                .revoked_at
                .is_some()
        );
    }

    #[tokio::test]
    async fn audit_append_and_list_newest_first() {
        let s = store().await;
        for i in 0..3 {
            s.append_audit(&AuditRow {
                id: format!("id{i}"),
                ts: format!("2026-01-0{}T00:00:00Z", i + 1),
                actor: "root".into(),
                actor_kind: "admin".into(),
                action: "test".into(),
                target: None,
                detail_json: "{}".into(),
                ip: None,
            })
            .await
            .unwrap();
        }
        let rows = s.list_audit(10).await.unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].id, "id2");
    }

    #[tokio::test]
    async fn webhook_and_zeroaccess_round_trip() {
        let s = store().await;
        s.put_webhook(&WebhookRow {
            id: "w1".into(),
            account_id: "a@x".into(),
            url: "https://example/hook".into(),
            secret_sealed: vec![1, 2, 3],
            event_filter_json: "[]".into(),
            created_at: "t".into(),
        })
        .await
        .unwrap();
        assert_eq!(s.list_webhooks_for_account("a@x").await.unwrap().len(), 1);

        s.upsert_zeroaccess(&ZeroAccessRow {
            account_id: "a@x".into(),
            enabled: true,
            wrapped_root_key: vec![9, 9],
            kdf_params_json: "{}".into(),
            recovery_wrapped: None,
            paired_devices_json: "[]".into(),
        })
        .await
        .unwrap();
        assert_eq!(s.list_zeroaccess_enabled().await.unwrap(), vec!["a@x"]);
        assert!(s.get_zeroaccess("a@x").await.unwrap().unwrap().enabled);
    }
}
