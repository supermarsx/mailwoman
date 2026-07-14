//! V6 store adapters (t6-e11 MOUNT): back the frozen Batch-B persistence seams
//! (`mw_oauth::OAuthStore`, `mw_admin::AdminBackend`, `crate::webhooks::WebhookRegistry`)
//! with the real `mw-store` 0007 tables.
//!
//! The Batch-B crates deliberately shipped their persistence as traits + in-memory
//! doubles (the store was mid-refactor). Here e11 supplies the production backing:
//! the additive `mw_store::Store` 0007 methods for the tabled data, the `settings`
//! table for the small config surfaces without a dedicated table (feature flags /
//! security policy / observability / ban list), and the store [`ServerKey`] to
//! unseal `webhooks.secret_sealed` at signing time.
//!
//! Row ⇄ trait-type mapping lives here so the crates stay `mw-store`-free and the
//! surface is backend-parity-identical (SQLite ⇄ Postgres) for free.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use mw_admin::store::UserRecord;
use mw_admin::{
    ActorKind, AdminBackend, AdminError, AuditLogEntry, BanEntry, CacheScopeRow as AdminCacheScope,
    Domain, ObservabilityConfig, Quota, SecurityPolicy, UserFeatureFlags,
};
use mw_oauth::{ApiKey, OAuthClient, OAuthError, OAuthStore, OAuthToken, TokenKind};
use mw_store::{
    AdminUserRow, ApiKeyRow, AuditRow, CacheScopeRow as StoreCacheScope, DomainRow, OAuthClientRow,
    OAuthTokenRow, QuotaRow, ServerKey, Store,
};

use crate::webhooks::{WebhookEndpoint, WebhookRegistry};

// Settings keys for the config surfaces without a dedicated 0007 table.
const KEY_SECURITY: &str = "v6:admin:security_policy";
const KEY_OBS: &str = "v6:admin:observability";
const KEY_BANS: &str = "v6:admin:bans";
fn flags_key(account_id: &str) -> String {
    format!("v6:admin:flags:{account_id}")
}

// ─── OAuthStore ⇄ mw-store ────────────────────────────────────────────────────

/// `mw_oauth::OAuthStore` over the 0007 `api_keys`/`oauth_*` tables.
#[derive(Clone)]
pub struct OAuthStoreAdapter {
    store: Store,
}

impl OAuthStoreAdapter {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
    pub fn store(&self) -> &Store {
        &self.store
    }
}

fn oauth_err(e: impl std::fmt::Display) -> OAuthError {
    OAuthError::Store(e.to_string())
}

fn kind_to_str(k: TokenKind) -> &'static str {
    match k {
        TokenKind::AuthCode => "auth-code",
        TokenKind::Access => "access",
        TokenKind::Refresh => "refresh",
    }
}
fn kind_from_str(s: &str) -> TokenKind {
    match s {
        "auth-code" => TokenKind::AuthCode,
        "refresh" => TokenKind::Refresh,
        _ => TokenKind::Access,
    }
}

fn api_key_to_row(key: &ApiKey) -> Result<ApiKeyRow, OAuthError> {
    Ok(ApiKeyRow {
        id: key.prefix.clone(),
        key_prefix: key.prefix.clone(),
        key_hash: key.hash.clone(),
        account_id: key.account_id.clone(),
        scopes_json: serde_json::to_string(&key.scope).map_err(oauth_err)?,
        unattended_send: key.scope.unattended_send,
        created_at: key.created_at.clone(),
        last_used_at: key.last_used_at.clone(),
        revoked_at: key.revoked_at.clone(),
    })
}

fn api_key_from_row(r: ApiKeyRow) -> Result<ApiKey, OAuthError> {
    Ok(ApiKey {
        prefix: r.key_prefix,
        hash: r.key_hash,
        account_id: r.account_id,
        scope: serde_json::from_str(&r.scopes_json).map_err(oauth_err)?,
        created_at: r.created_at,
        last_used_at: r.last_used_at,
        revoked_at: r.revoked_at,
    })
}

#[async_trait]
impl OAuthStore for OAuthStoreAdapter {
    async fn get_client(&self, client_id: &str) -> Result<Option<OAuthClient>, OAuthError> {
        let Some(r) = self
            .store
            .get_oauth_client(client_id)
            .await
            .map_err(oauth_err)?
        else {
            return Ok(None);
        };
        Ok(Some(OAuthClient {
            client_id: r.client_id,
            name: r.name,
            redirect_uris: serde_json::from_str(&r.redirect_uris_json).map_err(oauth_err)?,
            approved_by: r.approved_by,
            created_at: r.created_at,
        }))
    }

    async fn put_client(&self, client: OAuthClient) -> Result<(), OAuthError> {
        self.store
            .put_oauth_client(&OAuthClientRow {
                client_id: client.client_id,
                name: client.name,
                redirect_uris_json: serde_json::to_string(&client.redirect_uris)
                    .map_err(oauth_err)?,
                approved_by: client.approved_by,
                created_at: client.created_at,
            })
            .await
            .map_err(oauth_err)
    }

    async fn put_token(&self, token: OAuthToken) -> Result<(), OAuthError> {
        self.store
            .put_oauth_token(&OAuthTokenRow {
                token_hash: token.token_hash,
                client_id: token.client_id,
                account_id: token.account_id,
                scopes_json: serde_json::to_string(&token.scope).map_err(oauth_err)?,
                resource: token.resource,
                kind: kind_to_str(token.kind).to_string(),
                expires_at: token.expires_at,
                created_at: token.created_at,
                revoked_at: token.revoked_at,
                pkce_challenge: token.pkce_challenge,
            })
            .await
            .map_err(oauth_err)
    }

    async fn get_token(&self, token_hash: &str) -> Result<Option<OAuthToken>, OAuthError> {
        let Some(r) = self
            .store
            .get_oauth_token(token_hash)
            .await
            .map_err(oauth_err)?
        else {
            return Ok(None);
        };
        Ok(Some(OAuthToken {
            token_hash: r.token_hash,
            client_id: r.client_id,
            account_id: r.account_id,
            scope: serde_json::from_str(&r.scopes_json).map_err(oauth_err)?,
            resource: r.resource,
            kind: kind_from_str(&r.kind),
            expires_at: r.expires_at,
            created_at: r.created_at,
            revoked_at: r.revoked_at,
            pkce_challenge: r.pkce_challenge,
        }))
    }

    async fn revoke_token(&self, token_hash: &str) -> Result<(), OAuthError> {
        self.store
            .revoke_oauth_token(token_hash, &Utc::now().to_rfc3339())
            .await
            .map_err(oauth_err)
    }

    async fn put_api_key(&self, key: ApiKey) -> Result<(), OAuthError> {
        self.store
            .put_api_key(&api_key_to_row(&key)?)
            .await
            .map_err(oauth_err)
    }

    async fn get_api_key(&self, prefix: &str) -> Result<Option<ApiKey>, OAuthError> {
        match self.store.get_api_key(prefix).await.map_err(oauth_err)? {
            Some(r) => Ok(Some(api_key_from_row(r)?)),
            None => Ok(None),
        }
    }

    async fn touch_api_key(&self, prefix: &str, at: &str) -> Result<(), OAuthError> {
        self.store
            .touch_api_key(prefix, at)
            .await
            .map_err(oauth_err)
    }

    async fn revoke_api_key(&self, prefix: &str) -> Result<(), OAuthError> {
        self.store
            .revoke_api_key(prefix, &Utc::now().to_rfc3339())
            .await
            .map_err(oauth_err)
    }
}

// ─── AdminBackend ⇄ mw-store ──────────────────────────────────────────────────

/// `mw_admin::AdminBackend` over the 0007 admin tables + the `settings` table for
/// the config surfaces (flags / security policy / observability / bans) that have
/// no dedicated 0007 table.
#[derive(Clone)]
pub struct AdminBackendAdapter {
    store: Store,
}

impl AdminBackendAdapter {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

fn admin_err(e: impl std::fmt::Display) -> AdminError {
    AdminError::Store(e.to_string())
}

fn actor_kind_to_str(k: ActorKind) -> &'static str {
    match k {
        ActorKind::Admin => "admin",
        ActorKind::User => "user",
        ActorKind::ApiKey => "api-key",
        ActorKind::System => "system",
    }
}
fn actor_kind_from_str(s: &str) -> ActorKind {
    match s {
        "admin" => ActorKind::Admin,
        "user" => ActorKind::User,
        "api-key" => ActorKind::ApiKey,
        _ => ActorKind::System,
    }
}

#[async_trait]
impl AdminBackend for AdminBackendAdapter {
    async fn append_audit(&self, entry: AuditLogEntry) -> Result<(), AdminError> {
        self.store
            .append_audit(&AuditRow {
                id: entry.id,
                ts: entry.ts,
                actor: entry.actor,
                actor_kind: actor_kind_to_str(entry.actor_kind).to_string(),
                action: entry.action,
                target: entry.target,
                detail_json: entry.detail_json,
                ip: entry.ip,
            })
            .await
            .map_err(admin_err)
    }

    async fn list_audit(&self, limit: usize) -> Result<Vec<AuditLogEntry>, AdminError> {
        let rows = self
            .store
            .list_audit(limit as i64)
            .await
            .map_err(admin_err)?;
        Ok(rows
            .into_iter()
            .map(|r| AuditLogEntry {
                id: r.id,
                ts: r.ts,
                actor: r.actor,
                actor_kind: actor_kind_from_str(&r.actor_kind),
                action: r.action,
                target: r.target,
                detail_json: r.detail_json,
                ip: r.ip,
            })
            .collect())
    }

    async fn upsert_domain(&self, domain: Domain) -> Result<(), AdminError> {
        self.store
            .upsert_domain(&DomainRow {
                name: domain.name,
                upstream_json: domain.upstream_json,
                allowlist_json: serde_json::to_string(&domain.allowlist).map_err(admin_err)?,
                blocklist_json: serde_json::to_string(&domain.blocklist).map_err(admin_err)?,
            })
            .await
            .map_err(admin_err)
    }

    async fn get_domain(&self, name: &str) -> Result<Option<Domain>, AdminError> {
        match self.store.get_domain(name).await.map_err(admin_err)? {
            Some(r) => Ok(Some(domain_from_row(r)?)),
            None => Ok(None),
        }
    }

    async fn list_domains(&self) -> Result<Vec<Domain>, AdminError> {
        self.store
            .list_domains()
            .await
            .map_err(admin_err)?
            .into_iter()
            .map(domain_from_row)
            .collect()
    }

    async fn delete_domain(&self, name: &str) -> Result<(), AdminError> {
        self.store.delete_domain(name).await.map_err(admin_err)
    }

    async fn upsert_user(&self, user: UserRecord) -> Result<(), AdminError> {
        let account_id = mw_admin::account_id(&user.username, &user.domain);
        self.store
            .upsert_admin_user(&AdminUserRow {
                username: account_id,
                password_hash: user.password_hash,
                created_at: Utc::now().to_rfc3339(),
            })
            .await
            .map_err(admin_err)
    }

    async fn get_user(&self, account_id: &str) -> Result<Option<UserRecord>, AdminError> {
        let Some(r) = self
            .store
            .get_admin_user(account_id)
            .await
            .map_err(admin_err)?
        else {
            return Ok(None);
        };
        let (username, domain) = split_account(&r.username);
        Ok(Some(UserRecord {
            username,
            domain,
            password_hash: r.password_hash,
        }))
    }

    async fn set_quota(&self, account_id: &str, quota: Quota) -> Result<(), AdminError> {
        self.store
            .set_quota(
                account_id,
                QuotaRow {
                    bytes_limit: quota.bytes_limit,
                    msg_limit: quota.msg_limit,
                },
            )
            .await
            .map_err(admin_err)
    }

    async fn get_quota(&self, account_id: &str) -> Result<Option<Quota>, AdminError> {
        Ok(self
            .store
            .get_quota(account_id)
            .await
            .map_err(admin_err)?
            .map(|q| Quota {
                bytes_limit: q.bytes_limit,
                msg_limit: q.msg_limit,
            }))
    }

    async fn set_flags(&self, account_id: &str, flags: UserFeatureFlags) -> Result<(), AdminError> {
        let json = serde_json::to_string(&flags).map_err(admin_err)?;
        self.store
            .set_setting(&flags_key(account_id), &json)
            .await
            .map_err(admin_err)
    }

    async fn get_flags(&self, account_id: &str) -> Result<UserFeatureFlags, AdminError> {
        match self
            .store
            .get_setting(&flags_key(account_id))
            .await
            .map_err(admin_err)?
        {
            Some(s) => serde_json::from_str(&s).map_err(admin_err),
            None => Ok(UserFeatureFlags::default()),
        }
    }

    async fn revoke_sessions(&self, account_id: &str) -> Result<u64, AdminError> {
        self.store
            .delete_sessions_for_account(account_id)
            .await
            .map_err(admin_err)
    }

    async fn set_security_policy(&self, policy: SecurityPolicy) -> Result<(), AdminError> {
        let json = serde_json::to_string(&policy).map_err(admin_err)?;
        self.store
            .set_setting(KEY_SECURITY, &json)
            .await
            .map_err(admin_err)
    }

    async fn get_security_policy(&self) -> Result<Option<SecurityPolicy>, AdminError> {
        match self
            .store
            .get_setting(KEY_SECURITY)
            .await
            .map_err(admin_err)?
        {
            Some(s) => Ok(Some(serde_json::from_str(&s).map_err(admin_err)?)),
            None => Ok(None),
        }
    }

    async fn set_observability(&self, cfg: ObservabilityConfig) -> Result<(), AdminError> {
        let json = serde_json::to_string(&cfg).map_err(admin_err)?;
        self.store
            .set_setting(KEY_OBS, &json)
            .await
            .map_err(admin_err)
    }

    async fn get_observability(&self) -> Result<Option<ObservabilityConfig>, AdminError> {
        match self.store.get_setting(KEY_OBS).await.map_err(admin_err)? {
            Some(s) => Ok(Some(serde_json::from_str(&s).map_err(admin_err)?)),
            None => Ok(None),
        }
    }

    async fn upsert_cache_scope(&self, row: AdminCacheScope) -> Result<(), AdminError> {
        self.store
            .upsert_cache_scope(&StoreCacheScope {
                class: row.class,
                layers_json: row.layers_json,
                ttl_secs: row.ttl_secs,
            })
            .await
            .map_err(admin_err)
    }

    async fn list_cache_scope(&self) -> Result<Vec<AdminCacheScope>, AdminError> {
        Ok(self
            .store
            .list_cache_scope()
            .await
            .map_err(admin_err)?
            .into_iter()
            .map(|r| AdminCacheScope {
                class: r.class,
                layers_json: r.layers_json,
                ttl_secs: r.ttl_secs,
            })
            .collect())
    }

    async fn add_ban(&self, ban: BanEntry) -> Result<(), AdminError> {
        let mut bans = self.load_bans().await?;
        bans.retain(|b| b.ip != ban.ip);
        bans.push(ban);
        self.save_bans(&bans).await
    }

    async fn remove_ban(&self, ip: &str) -> Result<(), AdminError> {
        let mut bans = self.load_bans().await?;
        bans.retain(|b| b.ip != ip);
        self.save_bans(&bans).await
    }

    async fn list_bans(&self) -> Result<Vec<BanEntry>, AdminError> {
        self.load_bans().await
    }

    async fn is_banned(&self, ip: &str) -> Result<bool, AdminError> {
        Ok(self.load_bans().await?.iter().any(|b| b.ip == ip))
    }
}

impl AdminBackendAdapter {
    async fn load_bans(&self) -> Result<Vec<BanEntry>, AdminError> {
        match self.store.get_setting(KEY_BANS).await.map_err(admin_err)? {
            Some(s) => serde_json::from_str(&s).map_err(admin_err),
            None => Ok(Vec::new()),
        }
    }
    async fn save_bans(&self, bans: &[BanEntry]) -> Result<(), AdminError> {
        let json = serde_json::to_string(bans).map_err(admin_err)?;
        self.store
            .set_setting(KEY_BANS, &json)
            .await
            .map_err(admin_err)
    }
}

fn domain_from_row(r: DomainRow) -> Result<Domain, AdminError> {
    Ok(Domain {
        name: r.name,
        upstream_json: r.upstream_json,
        allowlist: serde_json::from_str(&r.allowlist_json).map_err(admin_err)?,
        blocklist: serde_json::from_str(&r.blocklist_json).map_err(admin_err)?,
    })
}

/// Split a canonical `username@domain` account id. A malformed id (no `@`) keeps
/// the whole string as the username with an empty domain.
fn split_account(account_id: &str) -> (String, String) {
    match account_id.rsplit_once('@') {
        Some((u, d)) => (u.to_string(), d.to_string()),
        None => (account_id.to_string(), String::new()),
    }
}

// ─── WebhookRegistry ⇄ mw-store (unseal secret via ServerKey) ─────────────────

/// `crate::webhooks::WebhookRegistry` over the 0007 `webhooks` table. Secrets are
/// unsealed with the store [`ServerKey`] just before an endpoint is handed to the
/// dispatcher, so the plaintext secret exists only transiently at signing time.
pub struct WebhookRegistryAdapter {
    store: Store,
    key: ServerKey,
}

impl WebhookRegistryAdapter {
    pub fn new(store: Store, key: ServerKey) -> Self {
        Self { store, key }
    }
}

#[async_trait]
impl WebhookRegistry for WebhookRegistryAdapter {
    async fn list_for_account(&self, account_id: &str) -> Vec<WebhookEndpoint> {
        let rows = match self.store.list_webhooks_for_account(account_id).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("webhook registry lookup failed: {e}");
                return Vec::new();
            }
        };
        rows.into_iter()
            .filter_map(|r| {
                let secret = self.key.open(&r.secret_sealed).ok()?;
                let events: Vec<String> =
                    serde_json::from_str(&r.event_filter_json).unwrap_or_default();
                Some(WebhookEndpoint {
                    id: r.id,
                    account_id: r.account_id,
                    url: r.url,
                    secret,
                    events,
                })
            })
            .collect()
    }
}

// ─── Engine posture source + audit feed ───────────────────────────────────────

/// Backs `mw_engine::AccountPostureSource` with the 0007 `zeroaccess_accounts`
/// table (a snapshot of enabled accounts, refreshed at build time). Any account
/// in the set is treated as zero-access so its plaintext-derived cache values are
/// forced to per-request scope (mw-cache structural exclusion).
pub struct StorePostureSource {
    zero_access: std::collections::HashSet<String>,
}

impl StorePostureSource {
    pub async fn load(store: &Store) -> Self {
        let zero_access = store
            .list_zeroaccess_enabled()
            .await
            .unwrap_or_default()
            .into_iter()
            .collect();
        Self { zero_access }
    }
}

impl mw_engine::AccountPostureSource for StorePostureSource {
    fn posture(&self, account_id: &str) -> mw_engine::AccountPosture {
        if self.zero_access.contains(account_id) {
            mw_engine::AccountPosture::ZeroAccess
        } else {
            mw_engine::AccountPosture::Standard
        }
    }
}

/// Bridges the engine's semantic [`mw_engine::AuditEvent`]s to the append-only
/// admin audit log (fire-and-forget; the engine never blocks on persistence).
pub struct AdminAuditFeed {
    admin: mw_admin::Admin,
}

impl AdminAuditFeed {
    pub fn new(admin: mw_admin::Admin) -> Self {
        Self { admin }
    }
}

impl mw_engine::AuditFeed for AdminAuditFeed {
    fn emit(&self, event: mw_engine::AuditEvent) {
        let admin = self.admin.clone();
        tokio::spawn(async move {
            let entry = AuditLogEntry {
                id: mw_store::ServerKey::generate().to_hex(),
                ts: Utc::now().to_rfc3339(),
                actor: event.account_id,
                actor_kind: ActorKind::System,
                action: event.action,
                target: event.target,
                detail_json: serde_json::to_string(&event.detail).unwrap_or_else(|_| "{}".into()),
                ip: None,
            };
            let _ = admin.audit(entry).await;
        });
    }
}

/// An `mw_oauth::AuditSink` that forwards enforcement decisions to the admin audit
/// log (fire-and-forget). Holds the `Admin` handle (Send + Sync).
pub struct AdminOAuthAudit {
    admin: mw_admin::Admin,
}

impl AdminOAuthAudit {
    pub fn new(admin: mw_admin::Admin) -> Arc<Self> {
        Arc::new(Self { admin })
    }
}

impl mw_oauth::AuditSink for AdminOAuthAudit {
    fn emit(&self, event: &mw_oauth::AuditEvent) {
        let admin = self.admin.clone();
        let entry = AuditLogEntry {
            id: mw_store::ServerKey::generate().to_hex(),
            ts: event.ts.clone(),
            actor: event.actor.clone(),
            actor_kind: if event.actor_kind == "api-key" {
                ActorKind::ApiKey
            } else {
                ActorKind::System
            },
            action: event.action.clone(),
            target: None,
            detail_json: serde_json::json!({
                "allowed": event.allowed,
                "reason": event.reason,
            })
            .to_string(),
            ip: event.ip.clone(),
        };
        tokio::spawn(async move {
            let _ = admin.audit(entry).await;
        });
    }
}
