//! 0016 remote-image display grants repository (t16 e1 skeleton; e6 fills the proxy
//! callers): additive, dual-backend `Store` methods over `remote_image_grants` (0016,
//! both dialects) backing the anonymizing image proxy's 4-grant model.
//!
//! Deny-by-default: a remote image loads only when a matching, non-revoked grant
//! exists. `scope_kind` selects the breadth — `single` (one message), `all` (account-
//! wide), `per-sender` (a sender address), `per-domain` (a sender domain) — and
//! `scope_value` carries the message id / sender / domain ('' for `all`). Revocation
//! is soft (the row is kept for audit). No secret columns. Authored in the SQLite `?n`
//! style so it runs identically on SQLite or Postgres through [`crate::backend`].

use chrono::Utc;

use crate::backend::q;
use crate::{Store, StoreError};

/// A remote-image display grant (0016).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteImageGrantRow {
    pub account_id: String,
    /// "single" | "all" | "per-sender" | "per-domain".
    pub scope_kind: String,
    /// Message id / sender / domain; "" for the account-wide `all` grant.
    pub scope_value: String,
    pub granted_at: String,
    pub revoked: bool,
}

impl Store {
    /// Grant remote-image loading for a scope (idempotent upsert; un-revokes and
    /// refreshes `granted_at` if the row already existed).
    pub async fn grant_remote_image(
        &self,
        account_id: &str,
        scope_kind: &str,
        scope_value: &str,
    ) -> Result<(), StoreError> {
        let now = Utc::now().to_rfc3339();
        q("INSERT INTO remote_image_grants (account_id, scope_kind, scope_value, granted_at, revoked)
             VALUES (?1, ?2, ?3, ?4, 0)
             ON CONFLICT(account_id, scope_kind, scope_value) DO UPDATE SET
                 granted_at = excluded.granted_at, revoked = 0")
        .bind(account_id)
        .bind(scope_kind)
        .bind(scope_value)
        .bind(&now)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Soft-revoke a grant (kept for audit). No-op if the row does not exist.
    pub async fn revoke_remote_image(
        &self,
        account_id: &str,
        scope_kind: &str,
        scope_value: &str,
    ) -> Result<(), StoreError> {
        q("UPDATE remote_image_grants SET revoked = 1
             WHERE account_id = ?1 AND scope_kind = ?2 AND scope_value = ?3")
        .bind(account_id)
        .bind(scope_kind)
        .bind(scope_value)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Whether a specific grant is active (present and not revoked).
    pub async fn is_remote_image_granted(
        &self,
        account_id: &str,
        scope_kind: &str,
        scope_value: &str,
    ) -> Result<bool, StoreError> {
        let n = q("SELECT COUNT(*) FROM remote_image_grants
                     WHERE account_id = ?1 AND scope_kind = ?2 AND scope_value = ?3 AND revoked = 0")
            .bind(account_id)
            .bind(scope_kind)
            .bind(scope_value)
            .fetch_scalar_i64(&self.backend)
            .await?;
        Ok(n > 0)
    }

    /// Active (non-revoked) grants for an account.
    pub async fn list_active_image_grants(
        &self,
        account_id: &str,
    ) -> Result<Vec<RemoteImageGrantRow>, StoreError> {
        let rows = q(
            "SELECT account_id, scope_kind, scope_value, granted_at, revoked
                      FROM remote_image_grants WHERE account_id = ?1 AND revoked = 0
                      ORDER BY granted_at DESC",
        )
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(grant_from_row).collect())
    }
}

fn grant_from_row(r: &crate::backend::Row) -> RemoteImageGrantRow {
    RemoteImageGrantRow {
        account_id: r.get_string("account_id"),
        scope_kind: r.get_string("scope_kind"),
        scope_value: r.get_string("scope_value"),
        granted_at: r.get_string("granted_at"),
        revoked: r.get_i64("revoked") != 0,
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
    async fn grant_revoke_and_query() {
        let s = store().await;
        assert!(!s.is_remote_image_granted("a1", "all", "").await.unwrap());

        s.grant_remote_image("a1", "per-domain", "example.com")
            .await
            .unwrap();
        s.grant_remote_image("a1", "all", "").await.unwrap();
        assert!(s.is_remote_image_granted("a1", "all", "").await.unwrap());
        assert_eq!(s.list_active_image_grants("a1").await.unwrap().len(), 2);

        s.revoke_remote_image("a1", "all", "").await.unwrap();
        assert!(!s.is_remote_image_granted("a1", "all", "").await.unwrap());
        assert_eq!(s.list_active_image_grants("a1").await.unwrap().len(), 1);

        // Re-granting un-revokes.
        s.grant_remote_image("a1", "all", "").await.unwrap();
        assert!(s.is_remote_image_granted("a1", "all", "").await.unwrap());
        assert_eq!(s.list_active_image_grants("a1").await.unwrap().len(), 2);
    }
}
