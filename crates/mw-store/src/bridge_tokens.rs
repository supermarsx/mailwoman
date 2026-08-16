//! 0018 cached bridge OAuth token repository (t16 e1 skeleton; e7 fills the OAuth-
//! client callers): additive, dual-backend `Store` methods over `bridge_oauth_tokens`
//! (0018, both dialects).
//!
//! The bridge OAuth client mints an access+refresh pair from the
//! `bridge_accounts.oauth_ref` binding (0008); this table caches that pair so a
//! refresh isn't re-run on every call. Both tokens are SECRETS: sealed at rest
//! (XChaCha20-Poly1305 under the store ServerKey), the same zero-access posture as
//! `ews_account_cred.sealed_cred`; the host unseals only to answer the bridge's gated
//! OAuth-token import. `expires_at` drives proactive refresh; `scope` is non-secret.
//! Authored in the SQLite `?n` style so it runs identically on SQLite or Postgres
//! through [`crate::backend`].

use std::fmt;

use chrono::Utc;

use crate::backend::q;
use crate::{Store, StoreError};

/// A cached bridge OAuth token pair (0018). Tokens are held decrypted only in memory;
/// at rest they live sealed. A `None` refresh token means the grant carried none.
///
/// [`fmt::Debug`] is **hand-written to redact both tokens** (t22-e12). A derived one
/// would put a live OAuth access token — and the refresh token, which is worth more —
/// into every `tracing` event, panic message and error body that ever formats this
/// row. Sealing the columns closes the at-rest exit; it does nothing about the three
/// exits a `{:?}` opens, and this type exists precisely to hold the tokens unsealed.
/// `mw_egress::proxy::ProxyAuth` and `mw_store::egress_config::EgressProxyRow` redact
/// for the same reason.
#[derive(Clone, PartialEq, Eq)]
pub struct BridgeOauthTokenRow {
    pub bridge_account_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    /// RFC 3339 access-token expiry.
    pub expires_at: String,
    pub scope: String,
    pub updated_at: String,
}

/// Hand-written so neither token can reach a log line, a panic message or an error
/// body. See the type's docs — a derived `Debug` is the leak.
impl fmt::Debug for BridgeOauthTokenRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BridgeOauthTokenRow")
            .field("bridge_account_id", &self.bridge_account_id)
            .field("access_token", &"<redacted>")
            // Shown as present-or-absent: whether a grant carried a refresh token is
            // useful for debugging and is not itself a secret.
            .field(
                "refresh_token",
                &self
                    .refresh_token
                    .as_ref()
                    .map(|_| "<redacted>")
                    .unwrap_or("None"),
            )
            .field("expires_at", &self.expires_at)
            .field("scope", &self.scope)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

impl Store {
    /// Upsert the cached OAuth token pair for a bridge account, sealing both tokens.
    pub async fn put_bridge_oauth_token(
        &self,
        token: &BridgeOauthTokenRow,
    ) -> Result<(), StoreError> {
        let sealed_access = self.key.seal(token.access_token.as_bytes())?;
        // A missing refresh token is sealed as empty bytes and decoded back to `None`.
        let refresh_plain = token.refresh_token.as_deref().unwrap_or("");
        let sealed_refresh = self.key.seal(refresh_plain.as_bytes())?;
        let now = Utc::now().to_rfc3339();
        q("INSERT INTO bridge_oauth_tokens
                 (bridge_account_id, sealed_access_token, sealed_refresh_token, expires_at, scope, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(bridge_account_id) DO UPDATE SET
                 sealed_access_token = excluded.sealed_access_token,
                 sealed_refresh_token = excluded.sealed_refresh_token,
                 expires_at = excluded.expires_at, scope = excluded.scope,
                 updated_at = excluded.updated_at")
        .bind(&token.bridge_account_id)
        .bind(sealed_access)
        .bind(sealed_refresh)
        .bind(&token.expires_at)
        .bind(&token.scope)
        .bind(&now)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Read + unseal the cached token pair for a bridge account, if present.
    pub async fn get_bridge_oauth_token(
        &self,
        bridge_account_id: &str,
    ) -> Result<Option<BridgeOauthTokenRow>, StoreError> {
        let row = q("SELECT bridge_account_id, sealed_access_token, sealed_refresh_token, expires_at, scope, updated_at
                     FROM bridge_oauth_tokens WHERE bridge_account_id = ?1")
            .bind(bridge_account_id)
            .fetch_optional(&self.backend)
            .await?;
        let Some(r) = row else { return Ok(None) };
        let access = decode_token(&self.key.open(&r.get_blob("sealed_access_token"))?)?;
        let refresh_bytes = self.key.open(&r.get_blob("sealed_refresh_token"))?;
        let refresh = if refresh_bytes.is_empty() {
            None
        } else {
            Some(decode_token(&refresh_bytes)?)
        };
        Ok(Some(BridgeOauthTokenRow {
            bridge_account_id: r.get_string("bridge_account_id"),
            access_token: access,
            refresh_token: refresh,
            expires_at: r.get_string("expires_at"),
            scope: r.get_string("scope"),
            updated_at: r.get_string("updated_at"),
        }))
    }

    /// Drop the cached token pair for a bridge account (disconnect / forced re-auth).
    pub async fn delete_bridge_oauth_token(
        &self,
        bridge_account_id: &str,
    ) -> Result<(), StoreError> {
        q("DELETE FROM bridge_oauth_tokens WHERE bridge_account_id = ?1")
            .bind(bridge_account_id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }
}

fn decode_token(bytes: &[u8]) -> Result<String, StoreError> {
    String::from_utf8(bytes.to_vec()).map_err(|_| StoreError::Corrupt("oauth token decode".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    #[tokio::test]
    async fn token_pair_round_trips_sealed() {
        let s = store().await;
        assert!(s.get_bridge_oauth_token("b1").await.unwrap().is_none());

        let tok = BridgeOauthTokenRow {
            bridge_account_id: "b1".into(),
            access_token: "ACCESS-s3cr3t".into(),
            refresh_token: Some("REFRESH-s3cr3t".into()),
            expires_at: "2026-07-19T00:00:00Z".into(),
            scope: "Mail.Read offline_access".into(),
            updated_at: String::new(),
        };
        s.put_bridge_oauth_token(&tok).await.unwrap();

        let got = s.get_bridge_oauth_token("b1").await.unwrap().unwrap();
        assert_eq!(got.access_token, "ACCESS-s3cr3t");
        assert_eq!(got.refresh_token.as_deref(), Some("REFRESH-s3cr3t"));
        assert_eq!(got.scope, "Mail.Read offline_access");
        assert!(!got.updated_at.is_empty());

        // Neither token is stored in plaintext at rest.
        let (a, rf) = q("SELECT sealed_access_token, sealed_refresh_token FROM bridge_oauth_tokens WHERE bridge_account_id = ?1")
            .bind("b1")
            .fetch_one(s.backend())
            .await
            .map(|r| (r.get_blob("sealed_access_token"), r.get_blob("sealed_refresh_token")))
            .unwrap();
        assert!(
            !a.windows(9)
                .any(|w| w == b"s3cr3t".as_slice() || w == b"ACCESS-s3".as_slice())
        );
        assert!(!a.windows(6).any(|w| w == b"s3cr3t"));
        assert!(!rf.windows(6).any(|w| w == b"s3cr3t"));
    }

    #[tokio::test]
    async fn absent_refresh_token_round_trips_as_none() {
        let s = store().await;
        s.put_bridge_oauth_token(&BridgeOauthTokenRow {
            bridge_account_id: "b2".into(),
            access_token: "A".into(),
            refresh_token: None,
            expires_at: "2026-07-19T00:00:00Z".into(),
            scope: String::new(),
            updated_at: String::new(),
        })
        .await
        .unwrap();
        let got = s.get_bridge_oauth_token("b2").await.unwrap().unwrap();
        assert_eq!(got.access_token, "A");
        assert!(got.refresh_token.is_none());

        s.delete_bridge_oauth_token("b2").await.unwrap();
        assert!(s.get_bridge_oauth_token("b2").await.unwrap().is_none());
    }

    #[test]
    fn debug_renders_neither_token() {
        // Sealing the columns closes the at-rest exit. It does nothing about the
        // three a `{:?}` opens — `tracing` events, panic messages and error bodies —
        // and this type exists precisely to hold the tokens UNSEALED in memory.
        const ACCESS: &str = "ya29-access-token-do-not-log";
        const REFRESH: &str = "1ARf-refresh-token-worth-more";
        let row = BridgeOauthTokenRow {
            bridge_account_id: "b1".into(),
            access_token: ACCESS.into(),
            refresh_token: Some(REFRESH.into()),
            expires_at: "2026-07-19T00:00:00Z".into(),
            scope: "mail.read".into(),
            updated_at: "2026-07-18T00:00:00Z".into(),
        };

        // NEGATIVE CONTROL, asserted first: the struct really does carry both tokens.
        // Without this, a type that had simply stopped storing them would pass the
        // redaction assertions below — "absent" and "redacted" are indistinguishable
        // in the rendered output, and only one of them is the property we want.
        assert_eq!(row.access_token, ACCESS);
        assert_eq!(row.refresh_token.as_deref(), Some(REFRESH));

        let rendered = format!("{row:?}");
        assert!(
            !rendered.contains(ACCESS),
            "Debug leaked the access token: {rendered}"
        );
        assert!(
            !rendered.contains(REFRESH),
            "Debug leaked the refresh token: {rendered}"
        );
        assert!(
            rendered.matches("<redacted>").count() == 2,
            "both tokens must render as redacted rather than be omitted — an absent \
             field reads as 'this type holds no secret': {rendered}"
        );
        // The non-secret fields stay legible, or the redaction has cost the type its
        // usefulness in a debug line and someone will reach for the raw fields instead.
        assert!(rendered.contains("b1") && rendered.contains("mail.read"));
    }

    #[test]
    fn debug_distinguishes_an_absent_refresh_token_from_a_redacted_one() {
        let row = BridgeOauthTokenRow {
            bridge_account_id: "b2".into(),
            access_token: "A".into(),
            refresh_token: None,
            expires_at: String::new(),
            scope: String::new(),
            updated_at: String::new(),
        };
        let rendered = format!("{row:?}");
        assert!(
            rendered.contains("refresh_token: \"None\""),
            "whether a grant carried a refresh token is useful for debugging and is \
             not itself a secret: {rendered}"
        );
        assert_eq!(rendered.matches("<redacted>").count(), 1);
    }
}
