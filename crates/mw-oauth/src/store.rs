//! Persistence seam for API keys + OAuth state.
//!
//! `mw-oauth` deliberately carries **no `mw-store` dependency**: the `Store`
//! façade exposes no api-key/oauth methods and is frozen. Instead this trait is
//! the contract `mw-server` (e11) implements over the 0007 tables
//! (`api_keys` / `oauth_clients` / `oauth_tokens`). Keeping the seam here lets the
//! crate be exhaustively unit-tested against [`InMemoryOAuthStore`] with no DB.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;

use crate::{ApiKey, OAuthClient, OAuthError, OAuthToken};

/// Async persistence for the authorization server. All lookups are by the stored
/// hash / prefix — the plaintext credential is never handed to the store.
#[async_trait]
pub trait OAuthStore: Send + Sync {
    // ── OAuth client registry (admin-approved) ──────────────────────────────
    async fn get_client(&self, client_id: &str) -> Result<Option<OAuthClient>, OAuthError>;
    async fn put_client(&self, client: OAuthClient) -> Result<(), OAuthError>;

    // ── OAuth tokens (auth-code | access | refresh), keyed by `token_hash` ───
    async fn put_token(&self, token: OAuthToken) -> Result<(), OAuthError>;
    async fn get_token(&self, token_hash: &str) -> Result<Option<OAuthToken>, OAuthError>;
    async fn revoke_token(&self, token_hash: &str) -> Result<(), OAuthError>;

    // ── API keys, keyed by public `prefix` ──────────────────────────────────
    async fn put_api_key(&self, key: ApiKey) -> Result<(), OAuthError>;
    async fn get_api_key(&self, prefix: &str) -> Result<Option<ApiKey>, OAuthError>;
    async fn touch_api_key(&self, prefix: &str, at: &str) -> Result<(), OAuthError>;
    async fn revoke_api_key(&self, prefix: &str) -> Result<(), OAuthError>;
}

/// In-memory reference implementation — the test/dev backing for [`OAuthStore`].
/// Production wires a `mw-store`-backed impl in `mw-server` (e11).
#[derive(Default)]
pub struct InMemoryOAuthStore {
    clients: Mutex<HashMap<String, OAuthClient>>,
    tokens: Mutex<HashMap<String, OAuthToken>>,
    keys: Mutex<HashMap<String, ApiKey>>,
}

impl InMemoryOAuthStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Test helper: number of live (non-revoked) tokens held.
    pub fn token_count(&self) -> usize {
        self.tokens.lock().expect("lock").len()
    }
}

#[async_trait]
impl OAuthStore for InMemoryOAuthStore {
    async fn get_client(&self, client_id: &str) -> Result<Option<OAuthClient>, OAuthError> {
        Ok(self.clients.lock().expect("lock").get(client_id).cloned())
    }

    async fn put_client(&self, client: OAuthClient) -> Result<(), OAuthError> {
        self.clients
            .lock()
            .expect("lock")
            .insert(client.client_id.clone(), client);
        Ok(())
    }

    async fn put_token(&self, token: OAuthToken) -> Result<(), OAuthError> {
        self.tokens
            .lock()
            .expect("lock")
            .insert(token.token_hash.clone(), token);
        Ok(())
    }

    async fn get_token(&self, token_hash: &str) -> Result<Option<OAuthToken>, OAuthError> {
        Ok(self.tokens.lock().expect("lock").get(token_hash).cloned())
    }

    async fn revoke_token(&self, token_hash: &str) -> Result<(), OAuthError> {
        if let Some(t) = self.tokens.lock().expect("lock").get_mut(token_hash) {
            t.revoked_at = Some(chrono::Utc::now().to_rfc3339());
        }
        Ok(())
    }

    async fn put_api_key(&self, key: ApiKey) -> Result<(), OAuthError> {
        self.keys
            .lock()
            .expect("lock")
            .insert(key.prefix.clone(), key);
        Ok(())
    }

    async fn get_api_key(&self, prefix: &str) -> Result<Option<ApiKey>, OAuthError> {
        Ok(self.keys.lock().expect("lock").get(prefix).cloned())
    }

    async fn touch_api_key(&self, prefix: &str, at: &str) -> Result<(), OAuthError> {
        if let Some(k) = self.keys.lock().expect("lock").get_mut(prefix) {
            k.last_used_at = Some(at.to_string());
        }
        Ok(())
    }

    async fn revoke_api_key(&self, prefix: &str) -> Result<(), OAuthError> {
        if let Some(k) = self.keys.lock().expect("lock").get_mut(prefix) {
            k.revoked_at = Some(chrono::Utc::now().to_rfc3339());
        }
        Ok(())
    }
}
