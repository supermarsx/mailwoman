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

use sha2::{Digest, Sha256};

use crate::backend::q;
use crate::{Store, StoreError};

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

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
    ///
    /// `unattended_send` is written on insert only. Replacing an existing key
    /// leaves the stored value alone: the column is the admin countersign, and
    /// [`Self::set_api_key_unattended_send`] is the one statement that changes it.
    pub async fn put_api_key(&self, row: &ApiKeyRow) -> Result<(), StoreError> {
        q("INSERT INTO api_keys (id, key_prefix, key_hash, account_id, scopes, unattended_send, created_at, last_used_at, revoked_at)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
           ON CONFLICT(key_prefix) DO UPDATE SET
             key_hash = excluded.key_hash, account_id = excluded.account_id,
             scopes = excluded.scopes,
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

    /// Set or clear the admin countersign on an API key, by its opaque row id.
    /// Returns whether a row was updated: `false` for an unknown id, and — when
    /// setting — for a revoked key, which is never countersigned.
    pub async fn set_api_key_unattended_send(
        &self,
        id: &str,
        on: bool,
    ) -> Result<bool, StoreError> {
        let sql = if on {
            "UPDATE api_keys SET unattended_send = 1 WHERE id = ?1 AND revoked_at IS NULL"
        } else {
            "UPDATE api_keys SET unattended_send = 0 WHERE id = ?1"
        };
        Ok(q(sql).bind(id).execute(&self.backend).await? > 0)
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

    /// How long an admin session survives without being used, in seconds (0030).
    pub const ADMIN_SESSION_IDLE_SECS: i64 = 30 * 60;

    /// The longest an admin session lives however much it is used, in seconds
    /// (0030). `mw-server` sends the same figure as the admin cookie's `Max-Age`.
    pub const ADMIN_SESSION_MAX_SECS: i64 = 12 * 60 * 60;

    /// The smallest forward move of an admin session's idle deadline that is
    /// written back, in seconds. See [`Store::get_admin_session_at`].
    pub const ADMIN_SESSION_REFRESH_GRANULARITY_SECS: i64 = 60;

    /// Store a new admin session. It is accepted until it has gone
    /// [`Self::ADMIN_SESSION_IDLE_SECS`] without a read, and never past
    /// [`Self::ADMIN_SESSION_MAX_SECS`] from now (0030). Rows whose deadline has
    /// already passed are deleted in the same call, so abandoned sessions do not
    /// accumulate.
    pub async fn put_admin_session(
        &self,
        token_hash: &str,
        admin_id: &str,
        now: &str,
    ) -> Result<(), StoreError> {
        let unix = unix_now();
        q("DELETE FROM admin_sessions WHERE expires_at <= ?1 OR absolute_expires_at <= ?1")
            .bind(unix)
            .execute(&self.backend)
            .await?;
        q("INSERT INTO admin_sessions
               (token_hash, admin_id, created_at, last_seen, expires_at, absolute_expires_at)
           VALUES (?1, ?2, ?3, ?3, ?4, ?5)
           ON CONFLICT(token_hash) DO UPDATE SET
             last_seen = excluded.last_seen,
             expires_at = excluded.expires_at,
             absolute_expires_at = excluded.absolute_expires_at")
        .bind(token_hash)
        .bind(admin_id)
        .bind(now)
        .bind(unix + Self::ADMIN_SESSION_IDLE_SECS)
        .bind(unix + Self::ADMIN_SESSION_MAX_SECS)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// The admin a session token belongs to, or `None` when the token is unknown or
    /// its session has expired. Every admin gate in `mw-server` resolves a cookie
    /// through this one method, so the expiry applies to all of them.
    pub async fn get_admin_session(&self, token_hash: &str) -> Result<Option<String>, StoreError> {
        self.get_admin_session_at(token_hash, unix_now()).await
    }

    /// [`get_admin_session`](Self::get_admin_session) with the clock supplied
    /// (`now` in unix seconds).
    ///
    /// A session is refused once `now` reaches its idle deadline or its absolute
    /// cap, and the row is deleted. An accepted read moves the idle deadline to
    /// `now + Self::ADMIN_SESSION_IDLE_SECS`, bounded by the absolute cap; the row is
    /// rewritten only when that moves the deadline by
    /// [`Self::ADMIN_SESSION_REFRESH_GRANULARITY_SECS`] or more, so a burst of panel
    /// requests costs one write rather than one each.
    pub async fn get_admin_session_at(
        &self,
        token_hash: &str,
        now: i64,
    ) -> Result<Option<String>, StoreError> {
        let Some(row) = q("SELECT admin_id, expires_at, absolute_expires_at
               FROM admin_sessions WHERE token_hash = ?1")
        .bind(token_hash)
        .fetch_optional(&self.backend)
        .await?
        else {
            return Ok(None);
        };
        let expires_at = row.get_i64("expires_at");
        let absolute = row.get_i64("absolute_expires_at");
        if now >= expires_at || now >= absolute {
            self.delete_admin_session(token_hash).await?;
            return Ok(None);
        }
        let refreshed = (now + Self::ADMIN_SESSION_IDLE_SECS).min(absolute);
        if refreshed - expires_at >= Self::ADMIN_SESSION_REFRESH_GRANULARITY_SECS {
            q("UPDATE admin_sessions SET expires_at = ?2 WHERE token_hash = ?1")
                .bind(token_hash)
                .bind(refreshed)
                .execute(&self.backend)
                .await?;
        }
        Ok(Some(row.get_string("admin_id")))
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

    /// The one folding of an account name used wherever two spellings of a name
    /// must compare equal — the admin flag key, the names an account is known by,
    /// and the session revoke: surrounding whitespace removed, lowercased.
    pub fn fold_account_name(name: &str) -> String {
        name.trim().to_lowercase()
    }

    /// Write `value` under `key` only if the stored value is still `expected`
    /// (`None`: only if the key is absent). Returns whether the write happened.
    ///
    /// Each arm is one statement, so the comparison and the write cannot be
    /// separated by another writer on either backend. This is what a
    /// read-modify-write of a `settings` record loops on instead of holding a
    /// transaction across the read (see [`record_change`](Self::record_change) for
    /// why a read-then-upgrade transaction is avoided on SQLite).
    pub async fn compare_and_set_setting(
        &self,
        key: &str,
        expected: Option<&str>,
        value: &str,
    ) -> Result<bool, StoreError> {
        let n = match expected {
            Some(old) => {
                q("UPDATE settings SET value = ?3 WHERE key = ?1 AND value = ?2")
                    .bind(key)
                    .bind(old)
                    .bind(value)
                    .execute(&self.backend)
                    .await?
            }
            None => {
                q("INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO NOTHING")
                    .bind(key)
                    .bind(value)
                    .execute(&self.backend)
                    .await?
            }
        };
        Ok(n == 1)
    }

    /// Record that the account `account_id` is known by each of `names` (folded
    /// with [`fold_account_name`](Self::fold_account_name)). Names already recorded
    /// are kept; the record holds at most [`ACCOUNT_NAMES_MAX`] names and further
    /// ones are not added.
    ///
    /// A session row carries the name the upstream reported and, sealed, the name
    /// the credentials were presented under, but it is deleted at logout and on
    /// revoke. This record outlives the sessions, so a credential that carries
    /// only the account id (an API key, an OAuth token) can still be matched to
    /// the names an admin may have flagged. It lives in `settings`; no table of
    /// its own.
    pub async fn remember_account_names(
        &self,
        account_id: &str,
        names: &[&str],
    ) -> Result<(), StoreError> {
        if account_id.is_empty() {
            return Ok(());
        }
        let key = account_names_key(account_id);
        for _ in 0..SETTING_SWAP_ATTEMPTS {
            let current = self.get_setting(&key).await?;
            let mut known = match &current {
                Some(raw) => parse_account_names(raw)?,
                None => Vec::new(),
            };
            let before = known.len();
            for name in names {
                let folded = Self::fold_account_name(name);
                if !folded.is_empty() && !known.contains(&folded) && known.len() < ACCOUNT_NAMES_MAX
                {
                    known.push(folded);
                }
            }
            if known.len() == before {
                return Ok(());
            }
            let json = serde_json::to_string(&known)
                .map_err(|e| StoreError::Corrupt(format!("account names: {e}")))?;
            if self
                .compare_and_set_setting(&key, current.as_deref(), &json)
                .await?
            {
                return Ok(());
            }
        }
        Err(StoreError::Corrupt(format!(
            "account names record changed under {SETTING_SWAP_ATTEMPTS} successive writes"
        )))
    }

    /// The folded names recorded for `account_id` by
    /// [`remember_account_names`](Self::remember_account_names). Empty when none.
    pub async fn account_names(&self, account_id: &str) -> Result<Vec<String>, StoreError> {
        if account_id.is_empty() {
            return Ok(Vec::new());
        }
        match self.get_setting(&account_names_key(account_id)).await? {
            Some(raw) => parse_account_names(&raw),
            None => Ok(Vec::new()),
        }
    }

    /// The ids of the accounts known by `name`, compared after
    /// [`fold_account_name`](Self::fold_account_name). An account is known by:
    ///
    /// * its id,
    /// * the `username` of an `accounts` row (engine mode),
    /// * a name recorded with [`remember_account_names`](Self::remember_account_names),
    /// * the `username` of one of its sessions, and
    /// * the name its session's credentials were presented under (sealed in the
    ///   session row; a row that does not open under this key is skipped — it
    ///   cannot be used as a session either).
    ///
    /// Reads every `accounts` and `sessions` row and every names record, so it is
    /// for the admin's disable and revoke, not for a request path.
    pub async fn account_ids_known_by(&self, name: &str) -> Result<Vec<String>, StoreError> {
        let wanted = Self::fold_account_name(name);
        let mut ids: Vec<String> = Vec::new();
        if wanted.is_empty() {
            return Ok(ids);
        }
        let mut add = |id: String| {
            if !id.is_empty() && !ids.contains(&id) {
                ids.push(id);
            }
        };
        for row in q("SELECT id, username FROM accounts")
            .fetch_all(&self.backend)
            .await?
        {
            let id = row.get_string("id");
            if Self::fold_account_name(&row.get_string("username")) == wanted
                || Self::fold_account_name(&id) == wanted
            {
                add(id);
            }
        }
        let like = format!("{ACCOUNT_NAMES_PREFIX}%");
        for row in q("SELECT key, value FROM settings WHERE key LIKE ?1")
            .bind(like.as_str())
            .fetch_all(&self.backend)
            .await?
        {
            let key = row.get_string("key");
            let Some(id) = key.strip_prefix(ACCOUNT_NAMES_PREFIX) else {
                continue;
            };
            if parse_account_names(&row.get_string("value"))?.contains(&wanted) {
                add(id.to_string());
            }
        }
        for row in q("SELECT account_id, username, sealed_creds FROM sessions")
            .fetch_all(&self.backend)
            .await?
        {
            let account_id = row.get_string("account_id");
            let presented = self
                .key
                .open(&row.get_blob("sealed_creds"))
                .ok()
                .and_then(|bytes| crate::decode_creds(&bytes).ok())
                .map(|c| c.username);
            if Self::fold_account_name(&row.get_string("username")) == wanted
                || Self::fold_account_name(&account_id) == wanted
                || presented.is_some_and(|p| Self::fold_account_name(&p) == wanted)
            {
                add(account_id);
            }
        }
        Ok(ids)
    }

    /// Delete every stored session of every account known by `username` (see
    /// [`account_ids_known_by`](Self::account_ids_known_by)), together with the
    /// `native_sessions` marker of each, and every session that has no account id
    /// and carries the name itself. Returns the number of `sessions` rows removed.
    ///
    /// The admin surface names an account as `username@domain`, while
    /// `sessions.account_id` is a store-generated id (engine mode) or the upstream's
    /// account id (proxy mode), so [`delete_sessions_for_account`](Self::delete_sessions_for_account)
    /// given the admin's name matches nothing in those modes.
    ///
    /// Matching the session's own `username` column is not enough either: in proxy
    /// mode that column holds the name the upstream reported, which can differ
    /// from the one the user typed and the admin knows. So the name is resolved to
    /// account ids first and every session of those accounts goes, whatever name
    /// it was opened under. The names on the deleted rows are recorded against the
    /// account ([`remember_account_names`](Self::remember_account_names)) before
    /// the rows go.
    ///
    /// A native marker is keyed by the lowercase hex SHA-256 of the session id (the
    /// bearer token), which is what `mw-server` writes into `native_sessions.token_hash`.
    pub async fn delete_sessions_for_username(&self, username: &str) -> Result<u64, StoreError> {
        let wanted = Self::fold_account_name(username);
        if wanted.is_empty() {
            return Ok(0);
        }
        let accounts = self.account_ids_known_by(username).await?;
        let mut doomed: Vec<String> = Vec::new();
        // The names on the rows that are about to go, per account. A session
        // opened before names were recorded keeps its two names nowhere else, so
        // they are recorded first: the account's keys and tokens can then still
        // be matched to the name the admin used.
        let mut names: Vec<(String, Vec<String>)> = Vec::new();
        for row in q("SELECT id, account_id, username, sealed_creds FROM sessions")
            .fetch_all(&self.backend)
            .await?
        {
            let account_id = row.get_string("account_id");
            let reported = row.get_string("username");
            let presented = self
                .key
                .open(&row.get_blob("sealed_creds"))
                .ok()
                .and_then(|bytes| crate::decode_creds(&bytes).ok())
                .map(|c| c.username);
            // A session without an account id belongs to no account in
            // `accounts`; it is matched on its own two names.
            let named = Self::fold_account_name(&reported) == wanted
                || presented
                    .as_deref()
                    .is_some_and(|p| Self::fold_account_name(p) == wanted);
            if !(named || accounts.contains(&account_id)) {
                continue;
            }
            doomed.push(row.get_string("id"));
            if !account_id.is_empty() {
                let at = match names.iter().position(|(id, _)| *id == account_id) {
                    Some(at) => at,
                    None => {
                        names.push((account_id, Vec::new()));
                        names.len() - 1
                    }
                };
                names[at].1.push(reported);
                names[at].1.extend(presented);
            }
        }
        for (account_id, known) in &names {
            let known: Vec<&str> = known.iter().map(String::as_str).collect();
            self.remember_account_names(account_id, &known).await?;
        }
        let mut removed = 0u64;
        for id in doomed {
            let token_hash: String = Sha256::digest(id.as_bytes())
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            q("DELETE FROM native_sessions WHERE token_hash = ?1")
                .bind(&token_hash)
                .execute(&self.backend)
                .await?;
            removed += q("DELETE FROM sessions WHERE id = ?1")
                .bind(&id)
                .execute(&self.backend)
                .await?;
        }
        Ok(removed)
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

/// The `settings` key prefix of the names an account is known by.
const ACCOUNT_NAMES_PREFIX: &str = "v6:account:names:";

/// The most names one account's record holds. A mail server that accepts any
/// number of spellings for one mailbox must not be able to grow the record
/// without bound.
pub const ACCOUNT_NAMES_MAX: usize = 32;

/// How many times a compare-and-set of a `settings` record is retried when
/// another writer got in between the read and the write.
const SETTING_SWAP_ATTEMPTS: usize = 16;

fn account_names_key(account_id: &str) -> String {
    format!("{ACCOUNT_NAMES_PREFIX}{account_id}")
}

fn parse_account_names(raw: &str) -> Result<Vec<String>, StoreError> {
    serde_json::from_str(raw)
        .map_err(|e| StoreError::Corrupt(format!("account names are not valid JSON: {e}")))
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

    /// Replacing a key by prefix carries the row's other columns and leaves the
    /// countersign where it was, in both directions; only the setter moves it, and
    /// it does not set it on a revoked key.
    #[tokio::test]
    async fn api_key_upsert_does_not_move_the_countersign() {
        let s = store().await;
        let row = |prefix: &str, unattended_send: bool| ApiKeyRow {
            id: prefix.into(),
            key_prefix: prefix.into(),
            key_hash: "hash".into(),
            account_id: "a@x".into(),
            scopes_json: "{}".into(),
            unattended_send,
            created_at: "2026-01-01T00:00:00Z".into(),
            last_used_at: None,
            revoked_at: None,
        };
        let flag = |s: &Store, prefix: &'static str| {
            let s = s.clone();
            async move {
                s.get_api_key(prefix)
                    .await
                    .unwrap()
                    .unwrap()
                    .unattended_send
            }
        };

        s.put_api_key(&row("plain", false)).await.unwrap();
        let mut again = row("plain", true);
        again.key_hash = "hash-2".into();
        s.put_api_key(&again).await.unwrap();
        assert_eq!(
            s.get_api_key("plain").await.unwrap().unwrap().key_hash,
            "hash-2",
            "control: the upsert did replace the row"
        );
        assert!(!flag(&s, "plain").await, "an upsert does not set it");

        assert!(s.set_api_key_unattended_send("plain", true).await.unwrap());
        assert!(flag(&s, "plain").await, "control: the setter sets it");
        s.put_api_key(&row("plain", false)).await.unwrap();
        assert!(flag(&s, "plain").await, "an upsert does not clear it");
        assert!(s.set_api_key_unattended_send("plain", false).await.unwrap());
        assert!(!flag(&s, "plain").await);

        assert!(!s.set_api_key_unattended_send("nope", true).await.unwrap());
        s.put_api_key(&row("gone", false)).await.unwrap();
        s.revoke_api_key("gone", "2026-01-02T00:00:00Z")
            .await
            .unwrap();
        assert!(!s.set_api_key_unattended_send("gone", true).await.unwrap());
        assert!(
            !flag(&s, "gone").await,
            "a revoked key is not countersigned"
        );
    }

    /// The admin surface names an account by login name; `sessions.account_id` is
    /// a different identifier. Delete-by-account with the login name removes
    /// nothing (the control), delete-by-username removes exactly that user's rows
    /// whatever the case, and takes the native marker with them.
    ///
    /// The names carry a per-run tag so the body can run against a Postgres
    /// database that other runs have used.
    async fn assert_delete_by_username(s: &Store) {
        use crate::{Credentials, NativeSessionRow};
        let tag = crate::seal::random_token();
        let alice_mixed = format!("Alice-{tag}@Example.org");
        let alice_lower = alice_mixed.to_ascii_lowercase();
        let alice_upper = alice_mixed.to_ascii_uppercase();
        let bob_name = format!("bob-{tag}@example.org");
        let (alice_acct, bob_acct) = (format!("acct-a-{tag}"), format!("acct-b-{tag}"));
        let creds = Credentials {
            username: "u".into(),
            password: "p".into(),
        };
        let alice_1 = s
            .create_session(&alice_acct, &alice_mixed, "engine", "engine", &creds)
            .await
            .unwrap();
        let alice_2 = s
            .create_session(&alice_acct, &alice_lower, "engine", "engine", &creds)
            .await
            .unwrap();
        let bob = s
            .create_session(&bob_acct, &bob_name, "engine", "engine", &creds)
            .await
            .unwrap();
        let native_hash = |id: &str| -> String {
            Sha256::digest(id.as_bytes())
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect()
        };
        for id in [&alice_2, &bob] {
            s.create_native_session(&NativeSessionRow {
                token_hash: native_hash(id),
                account_id: "x".into(),
                client_type: "native".into(),
                created_at: "2026-01-01T00:00:00Z".into(),
                last_seen: "2026-01-01T00:00:00Z".into(),
                rotated_from: None,
            })
            .await
            .unwrap();
        }

        // Control: the pre-existing by-account delete, given the login name,
        // matches no row — this is the dead "revoke sessions" button.
        assert_eq!(
            s.delete_sessions_for_account(&alice_lower).await.unwrap(),
            0
        );
        assert!(s.get_session(&alice_1).await.is_ok());

        assert_eq!(
            s.delete_sessions_for_username(&alice_upper).await.unwrap(),
            2
        );
        assert!(s.get_session(&alice_1).await.is_err());
        assert!(s.get_session(&alice_2).await.is_err());
        assert!(
            s.get_native_session(&native_hash(&alice_2))
                .await
                .unwrap()
                .is_none(),
            "the native marker goes with its session"
        );
        // Another user's session and native marker are untouched.
        assert!(s.get_session(&bob).await.is_ok());
        assert!(
            s.get_native_session(&native_hash(&bob))
                .await
                .unwrap()
                .is_some()
        );
        // Nothing left to delete.
        assert_eq!(
            s.delete_sessions_for_username(&alice_lower).await.unwrap(),
            0
        );
        // Leave a shared database as it was found: the sessions, and the names
        // the revoke recorded for the two accounts.
        assert_eq!(s.delete_sessions_for_username(&bob_name).await.unwrap(), 1);
        for acct in [&alice_acct, &bob_acct] {
            q("DELETE FROM settings WHERE key = ?1")
                .bind(account_names_key(acct))
                .execute(&s.backend)
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn delete_sessions_for_username_matches_login_name_case_insensitively() {
        assert_delete_by_username(&store().await).await;
    }

    /// The same assertions on live Postgres, whose statement spells the case fold
    /// differently. Runs when `DATABASE_URL_PG` / `MW_TEST_PG` names a server (the
    /// convention of `tests/backend_parity.rs`); says so when it does not.
    #[tokio::test]
    async fn delete_sessions_for_username_on_postgres() {
        let Some(dsn) = std::env::var("DATABASE_URL_PG")
            .ok()
            .or_else(|| std::env::var("MW_TEST_PG").ok())
            .filter(|s| !s.trim().is_empty())
        else {
            eprintln!(
                "[mw-store] t27-e3 delete_sessions_for_username: Postgres path SKIPPED (set \
                 DATABASE_URL_PG or MW_TEST_PG to a live postgres:16 to run it). The SQLite \
                 path still asserted."
            );
            return;
        };
        let s = Store::open_postgres(&dsn, ServerKey::generate())
            .await
            .expect("DATABASE_URL_PG is set but Postgres is not reachable");
        assert_delete_by_username(&s).await;
    }

    /// O1 (t27-f1): the admin's name for an account may be the one the user typed,
    /// which a proxy-mode session keeps only inside its sealed credentials. The
    /// revoke must find the account through it and take every session of that
    /// account, including one opened under another name.
    #[tokio::test]
    async fn revoke_by_the_presented_name_takes_every_session_of_the_account() {
        use crate::Credentials;
        let s = store().await;
        let creds = |name: &str| Credentials {
            username: name.into(),
            password: "p".into(),
        };
        // The upstream reports `alice@example.org`; the user typed `Alice`.
        let typed = s
            .create_session("up-1", "alice@example.org", "u", "u", &creds("Alice"))
            .await
            .unwrap();
        // A second session of the same account, opened under the reported name.
        let other = s
            .create_session(
                "up-1",
                "alice@example.org",
                "u",
                "u",
                &creds("alice@example.org"),
            )
            .await
            .unwrap();
        let bob = s
            .create_session("up-2", "bob@example.org", "u", "u", &creds("bob"))
            .await
            .unwrap();

        assert_eq!(
            s.account_ids_known_by(" ALICE ").await.unwrap(),
            vec!["up-1".to_string()]
        );
        assert_eq!(s.delete_sessions_for_username("alice").await.unwrap(), 2);
        assert!(s.get_session(&typed).await.is_err());
        assert!(s.get_session(&other).await.is_err());
        assert!(s.get_session(&bob).await.is_ok(), "another account is kept");
        // The deleted sessions' names were recorded, so the account is still
        // found by either after its sessions are gone.
        assert_eq!(
            s.account_names("up-1").await.unwrap(),
            vec!["alice@example.org".to_string(), "alice".to_string()]
        );
        assert_eq!(
            s.account_ids_known_by("alice").await.unwrap(),
            vec!["up-1".to_string()]
        );
        assert!(s.account_names("up-2").await.unwrap().is_empty());

        // A session with no account id (an upstream that names no mail account)
        // is matched on its own names and is not one account with the others.
        let carol = s
            .create_session("", "carol@example.org", "u", "u", &creds("carol"))
            .await
            .unwrap();
        let dave = s
            .create_session("", "dave@example.org", "u", "u", &creds("dave"))
            .await
            .unwrap();
        assert_eq!(s.delete_sessions_for_username("CAROL").await.unwrap(), 1);
        assert!(s.get_session(&carol).await.is_err());
        assert!(s.get_session(&dave).await.is_ok());
    }

    /// The names record outlives the sessions and resolves a name back to the
    /// account; it is folded, de-duplicated and capped.
    #[tokio::test]
    async fn account_names_are_merged_folded_and_capped() {
        let s = store().await;
        assert!(s.account_names("up-1").await.unwrap().is_empty());
        s.remember_account_names("up-1", &["Alice", " alice@Example.org ", ""])
            .await
            .unwrap();
        s.remember_account_names("up-1", &["ALICE", "al"])
            .await
            .unwrap();
        assert_eq!(
            s.account_names("up-1").await.unwrap(),
            vec![
                "alice".to_string(),
                "alice@example.org".to_string(),
                "al".to_string()
            ]
        );
        // No session and no `accounts` row exists: the record alone resolves it.
        assert_eq!(
            s.account_ids_known_by("Alice@example.org").await.unwrap(),
            vec!["up-1".to_string()]
        );
        assert!(s.account_ids_known_by("bob").await.unwrap().is_empty());
        // An empty account id records nothing.
        s.remember_account_names("", &["ghost"]).await.unwrap();
        assert!(s.account_ids_known_by("ghost").await.unwrap().is_empty());

        let many: Vec<String> = (0..ACCOUNT_NAMES_MAX + 8)
            .map(|i| format!("n{i}"))
            .collect();
        let many: Vec<&str> = many.iter().map(String::as_str).collect();
        s.remember_account_names("up-1", &many).await.unwrap();
        let kept = s.account_names("up-1").await.unwrap();
        assert_eq!(kept.len(), ACCOUNT_NAMES_MAX);
        assert_eq!(kept[0], "alice", "earlier names are not displaced");
    }

    /// O5 (t27-f1): the write happens only over the value the caller read.
    #[tokio::test]
    async fn compare_and_set_setting_refuses_a_stale_expectation() {
        let s = store().await;
        assert!(s.compare_and_set_setting("k", None, "v1").await.unwrap());
        assert!(
            !s.compare_and_set_setting("k", None, "v2").await.unwrap(),
            "absent was expected, a value is stored"
        );
        assert!(
            !s.compare_and_set_setting("k", Some("stale"), "v2")
                .await
                .unwrap()
        );
        assert_eq!(s.get_setting("k").await.unwrap().as_deref(), Some("v1"));
        assert!(
            s.compare_and_set_setting("k", Some("v1"), "v2")
                .await
                .unwrap()
        );
        assert_eq!(s.get_setting("k").await.unwrap().as_deref(), Some("v2"));
        assert!(
            !s.compare_and_set_setting("absent", Some("x"), "y")
                .await
                .unwrap()
        );
        assert_eq!(s.get_setting("absent").await.unwrap(), None);
    }

    /// The deadlines of one admin session row: `(expires_at, absolute_expires_at)`.
    async fn admin_deadlines(s: &Store, hash: &str) -> Option<(i64, i64)> {
        q("SELECT expires_at, absolute_expires_at FROM admin_sessions WHERE token_hash = ?1")
            .bind(hash)
            .fetch_optional(&s.backend)
            .await
            .unwrap()
            .map(|r| (r.get_i64("expires_at"), r.get_i64("absolute_expires_at")))
    }

    async fn resolves(s: &Store, hash: &str, now: i64) -> bool {
        s.get_admin_session_at(hash, now).await.unwrap().as_deref() == Some("root")
    }

    async fn assert_admin_session_expiry(s: &Store, hash: &str) {
        let t0 = unix_now();
        s.put_admin_session(hash, "root", "2026-10-05T00:00:00Z")
            .await
            .unwrap();
        let (idle0, cap) = admin_deadlines(s, hash).await.expect("row written");
        assert!(
            (idle0 - t0 - Store::ADMIN_SESSION_IDLE_SECS).abs() <= 5,
            "{idle0}"
        );
        assert!(
            (cap - t0 - Store::ADMIN_SESSION_MAX_SECS).abs() <= 5,
            "{cap}"
        );

        // Control: inside the idle window the token resolves.
        assert!(resolves(s, hash, t0 + 10).await);
        // A read that moves the deadline by less than the granularity writes nothing.
        assert_eq!(admin_deadlines(s, hash).await.unwrap().0, idle0);

        // A later read inside the window moves the idle deadline forward...
        let later = idle0 - 60;
        assert!(resolves(s, hash, later).await);
        let (idle1, cap1) = admin_deadlines(s, hash).await.unwrap();
        assert_eq!(idle1, later + Store::ADMIN_SESSION_IDLE_SECS);
        assert_eq!(cap1, cap, "the absolute cap never moves");
        // ...so a time past the ORIGINAL deadline is still accepted,
        assert!(resolves(s, hash, idle0 + 30).await);

        // and the refresh is bounded by the absolute cap. Walk the session there in
        // steps shorter than the idle window, as a session in constant use would.
        let mut now = idle0 + 30;
        while now + Store::ADMIN_SESSION_IDLE_SECS / 2 < cap {
            now += Store::ADMIN_SESSION_IDLE_SECS / 2;
            assert!(resolves(s, hash, now).await, "in use at {now}");
        }
        assert_eq!(admin_deadlines(s, hash).await.unwrap().0, cap);
        assert!(resolves(s, hash, cap - 1).await);
        assert!(
            !resolves(s, hash, cap).await,
            "a session in constant use still ends at the absolute cap"
        );
        assert!(
            admin_deadlines(s, hash).await.is_none(),
            "an expired row is deleted, not left to be retried"
        );

        // Idle expiry on its own: one second before the deadline, then at it.
        s.put_admin_session(hash, "root", "2026-10-05T00:00:00Z")
            .await
            .unwrap();
        let (idle, _) = admin_deadlines(s, hash).await.unwrap();
        assert!(resolves(s, hash, idle - 1).await);
        s.put_admin_session(hash, "root", "2026-10-05T00:00:00Z")
            .await
            .unwrap();
        let (idle, _) = admin_deadlines(s, hash).await.unwrap();
        assert!(!resolves(s, hash, idle).await);
        assert!(s.get_admin_session(hash).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn admin_session_expires_when_idle_and_at_the_absolute_cap() {
        assert_admin_session_expiry(&store().await, "h-expiry").await;
    }

    /// The same on live Postgres (BIGINT columns). Runs when `DATABASE_URL_PG` /
    /// `MW_TEST_PG` names a server; says so when it does not.
    #[tokio::test]
    async fn admin_session_expiry_on_postgres() {
        let Some(dsn) = std::env::var("DATABASE_URL_PG")
            .ok()
            .or_else(|| std::env::var("MW_TEST_PG").ok())
            .filter(|s| !s.trim().is_empty())
        else {
            eprintln!(
                "[mw-store] t28-e8 admin session expiry: Postgres path SKIPPED (set \
                 DATABASE_URL_PG or MW_TEST_PG to a live postgres:16 to run it). The SQLite \
                 path still asserted."
            );
            return;
        };
        let s = Store::open_postgres(&dsn, ServerKey::generate())
            .await
            .expect("DATABASE_URL_PG is set but Postgres is not reachable");
        // A hash unique to this run: the Postgres test database is shared.
        let hash = format!("h-expiry-{}-{}", std::process::id(), unix_now());
        assert_admin_session_expiry(&s, &hash).await;
    }

    /// A new login deletes rows whose deadline has passed, whoever they belong to.
    #[tokio::test]
    async fn a_new_admin_session_purges_expired_ones() {
        let s = store().await;
        s.put_admin_session("stale", "root", "t").await.unwrap();
        s.put_admin_session("live", "root", "t").await.unwrap();
        q("UPDATE admin_sessions SET expires_at = 1 WHERE token_hash = 'stale'")
            .execute(&s.backend)
            .await
            .unwrap();
        s.put_admin_session("fresh", "root", "t").await.unwrap();
        assert!(admin_deadlines(&s, "stale").await.is_none());
        assert!(admin_deadlines(&s, "live").await.is_some());
        assert!(admin_deadlines(&s, "fresh").await.is_some());
    }

    /// 0030 over a database that already holds an admin session: the row survives
    /// the migration and reads as expired, because it carries no deadline.
    #[tokio::test]
    async fn migration_0030_expires_admin_sessions_issued_before_it() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let mut before = sqlx::migrate!("./migrations");
        before.migrations.to_mut().retain(|m| m.version < 30);
        assert!(before.migrations.iter().any(|m| m.version == 29));
        before.run(&pool).await.unwrap();
        let pre = crate::Store {
            backend: crate::backend::Backend::Sqlite(pool.clone()),
            key: ServerKey::from_bytes(&[9u8; 32]).unwrap(),
            uploads: crate::upload::fail_closed_backend(),
        };
        let has_column = q(
            "SELECT COUNT(*) AS n FROM pragma_table_info('admin_sessions')
                            WHERE name = 'expires_at'",
        )
        .fetch_one(&pre.backend)
        .await
        .unwrap()
        .get_i64("n");
        assert_eq!(has_column, 0, "the seed really is at the pre-0030 schema");
        q(
            "INSERT INTO admin_sessions (token_hash, admin_id, created_at, last_seen)
           VALUES ('old', 'root', 't', 't')",
        )
        .execute(&pre.backend)
        .await
        .unwrap();

        let after = Store::init_sqlite(pool, ServerKey::from_bytes(&[9u8; 32]).unwrap())
            .await
            .expect("0030 applies over a populated database");
        assert_eq!(admin_deadlines(&after, "old").await, Some((0, 0)));
        assert!(after.get_admin_session("old").await.unwrap().is_none());
        // A session issued after the migration works.
        after.put_admin_session("new", "root", "t").await.unwrap();
        assert_eq!(
            after.get_admin_session("new").await.unwrap().as_deref(),
            Some("root")
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
