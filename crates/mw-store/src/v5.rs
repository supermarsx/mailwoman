//! V5 (thin shells + push) repository methods (plan §2.4) layered over [`Store`]:
//! push subscriptions (`push_subscriptions`), the server VAPID keypair
//! (`push_config`, private key SEALED at rest), and native bearer-token sessions
//! (`native_sessions`).
//!
//! Same seam discipline as [`crate::v4`]: values cross as opaque primitives;
//! enum-like fields (`transport`, `client_type`) are plain strings the engine/
//! server owns. PRIVACY (plan §2.3/risk #8): NO message content is ever stored for
//! push; the VAPID private key is sealed via [`crate::ServerKey`] (never plaintext);
//! `native_sessions` stores only a token HASH.
//!
//! e0 ships the Row types + working CRUD + the sealed VAPID round-trip so the seam
//! is real and testable. e5 wires the dispatcher (a second consumer of the engine
//! `StateChange` broadcast) + the real VAPID keygen on first boot.

use sqlx::Row;

use crate::{Store, StoreError};

/// One push subscription (`push_subscriptions`). `p256dh`/`auth` are Web-Push-only;
/// `app_id` is UnifiedPush/APNs. No content is ever attached to a subscription.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushSubscriptionRow {
    pub id: String,
    pub account_id: String,
    pub transport: String,
    pub endpoint: String,
    pub p256dh: Option<String>,
    pub auth: Option<String>,
    pub app_id: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub last_wake_at: Option<String>,
}

/// One native bearer-token session (`native_sessions`). Stores only the token hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSessionRow {
    pub token_hash: String,
    pub account_id: String,
    pub client_type: String,
    pub created_at: String,
    pub last_seen: String,
    pub rotated_from: Option<String>,
}

fn push_sub_from_row(r: &sqlx::sqlite::SqliteRow) -> PushSubscriptionRow {
    PushSubscriptionRow {
        id: r.get("id"),
        account_id: r.get("account_id"),
        transport: r.get("transport"),
        endpoint: r.get("endpoint"),
        p256dh: r.get("p256dh"),
        auth: r.get("auth"),
        app_id: r.get("app_id"),
        expires_at: r.get("expires_at"),
        created_at: r.get("created_at"),
        last_wake_at: r.get("last_wake_at"),
    }
}

fn native_session_from_row(r: &sqlx::sqlite::SqliteRow) -> NativeSessionRow {
    NativeSessionRow {
        token_hash: r.get("token_hash"),
        account_id: r.get("account_id"),
        client_type: r.get("client_type"),
        created_at: r.get("created_at"),
        last_seen: r.get("last_seen"),
        rotated_from: r.get("rotated_from"),
    }
}

impl Store {
    // ── Push subscriptions ────────────────────────────────────────────────────

    /// Idempotently store (or refresh) a subscription, keyed by its endpoint.
    pub async fn upsert_push_subscription(
        &self,
        row: &PushSubscriptionRow,
    ) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO push_subscriptions
                 (id, account_id, transport, endpoint, p256dh, auth, app_id,
                  expires_at, created_at, last_wake_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
             ON CONFLICT(endpoint) DO UPDATE SET
                 account_id=excluded.account_id, transport=excluded.transport,
                 p256dh=excluded.p256dh, auth=excluded.auth, app_id=excluded.app_id,
                 expires_at=excluded.expires_at",
        )
        .bind(&row.id)
        .bind(&row.account_id)
        .bind(&row.transport)
        .bind(&row.endpoint)
        .bind(row.p256dh.as_deref())
        .bind(row.auth.as_deref())
        .bind(row.app_id.as_deref())
        .bind(row.expires_at.as_deref())
        .bind(&row.created_at)
        .bind(row.last_wake_at.as_deref())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// All active subscriptions for an account (the dispatcher's fan-out set).
    pub async fn list_push_subscriptions(
        &self,
        account_id: &str,
    ) -> Result<Vec<PushSubscriptionRow>, StoreError> {
        let rows = sqlx::query(
            "SELECT * FROM push_subscriptions WHERE account_id = ?1 ORDER BY created_at",
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(push_sub_from_row).collect())
    }

    /// Remove a subscription by endpoint (client unsubscribe / expired endpoint).
    pub async fn delete_push_subscription(&self, endpoint: &str) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM push_subscriptions WHERE endpoint = ?1")
            .bind(endpoint)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Remove a subscription by its id (the `POST /api/push/unsubscribe {id}` path).
    pub async fn delete_push_subscription_by_id(&self, id: &str) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM push_subscriptions WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Record that an opaque wake was last sent to a subscription (rate/telemetry).
    pub async fn touch_push_wake(&self, id: &str, at: &str) -> Result<(), StoreError> {
        sqlx::query("UPDATE push_subscriptions SET last_wake_at = ?2 WHERE id = ?1")
            .bind(id)
            .bind(at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ── VAPID keypair (private SEALED at rest) ────────────────────────────────

    /// Persist the VAPID keypair, sealing the private key with the server key so it
    /// is never stored in plaintext. e5 calls this after generating the keypair on
    /// first boot; the public key is served (public-only) via `/api/push/vapid`.
    pub async fn store_vapid_keypair(
        &self,
        public: &str,
        private_plaintext: &[u8],
        created_at: &str,
    ) -> Result<(), StoreError> {
        let sealed = self.key.seal(private_plaintext)?;
        sqlx::query(
            "INSERT INTO push_config (id, vapid_public, vapid_private_sealed, created_at)
             VALUES (1, ?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET
                 vapid_public=excluded.vapid_public,
                 vapid_private_sealed=excluded.vapid_private_sealed",
        )
        .bind(public)
        .bind(sealed)
        .bind(created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Load the VAPID keypair, unsealing the private key. `None` before first init.
    pub async fn load_vapid_keypair(&self) -> Result<Option<(String, Vec<u8>)>, StoreError> {
        let row =
            sqlx::query("SELECT vapid_public, vapid_private_sealed FROM push_config WHERE id = 1")
                .fetch_optional(&self.pool)
                .await?;
        match row {
            Some(r) => {
                let public: String = r.get("vapid_public");
                let sealed: Vec<u8> = r.get("vapid_private_sealed");
                let private = self.key.open(&sealed)?;
                Ok(Some((public, private)))
            }
            None => Ok(None),
        }
    }

    /// The PUBLIC VAPID key only (what `/api/push/vapid` serves). Never the private.
    pub async fn vapid_public(&self) -> Result<Option<String>, StoreError> {
        let row = sqlx::query("SELECT vapid_public FROM push_config WHERE id = 1")
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.get("vapid_public")))
    }

    // ── Native bearer-token sessions ──────────────────────────────────────────

    /// Create a native session row (stores the token HASH only).
    pub async fn create_native_session(&self, row: &NativeSessionRow) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO native_sessions
                 (token_hash, account_id, client_type, created_at, last_seen, rotated_from)
             VALUES (?1,?2,?3,?4,?5,?6)",
        )
        .bind(&row.token_hash)
        .bind(&row.account_id)
        .bind(&row.client_type)
        .bind(&row.created_at)
        .bind(&row.last_seen)
        .bind(row.rotated_from.as_deref())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Resolve a native session by token hash (bearer-auth path).
    pub async fn get_native_session(
        &self,
        token_hash: &str,
    ) -> Result<Option<NativeSessionRow>, StoreError> {
        let row = sqlx::query("SELECT * FROM native_sessions WHERE token_hash = ?1")
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(native_session_from_row))
    }

    /// Delete a native session by token hash (logout / rotation).
    pub async fn delete_native_session(&self, token_hash: &str) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM native_sessions WHERE token_hash = ?1")
            .bind(token_hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{ServerKey, Store};

    use super::*;

    async fn store() -> Store {
        let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        // A parent account row so the FK holds (mirrors cache.rs's insert shape).
        sqlx::query(
            "INSERT INTO accounts (id, kind, host, port, tls, username, sealed_creds, sync_policy_json)
             VALUES ('acct', 'imap', 'mail.example', 993, 1, 'u@e', X'00', '{}')",
        )
        .execute(&s.pool)
        .await
        .unwrap();
        s
    }

    #[tokio::test]
    async fn push_subscription_round_trips_and_is_idempotent() {
        let s = store().await;
        let row = PushSubscriptionRow {
            id: "sub1".into(),
            account_id: "acct".into(),
            transport: "webpush".into(),
            endpoint: "https://push.example/abc".into(),
            p256dh: Some("p".into()),
            auth: Some("a".into()),
            app_id: None,
            expires_at: None,
            created_at: "2026-07-13T00:00:00Z".into(),
            last_wake_at: None,
        };
        s.upsert_push_subscription(&row).await.unwrap();
        // Re-subscribe with the same endpoint → still one row (idempotent).
        s.upsert_push_subscription(&row).await.unwrap();
        let subs = s.list_push_subscriptions("acct").await.unwrap();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].endpoint, "https://push.example/abc");

        s.delete_push_subscription("https://push.example/abc")
            .await
            .unwrap();
        assert!(s.list_push_subscriptions("acct").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn vapid_private_key_is_sealed_at_rest() {
        let s = store().await;
        let private = b"vapid-private-key-bytes";
        s.store_vapid_keypair("PUBLIC_KEY", private, "2026-07-13T00:00:00Z")
            .await
            .unwrap();

        // The public key is retrievable on its own.
        assert_eq!(
            s.vapid_public().await.unwrap().as_deref(),
            Some("PUBLIC_KEY")
        );

        // The stored private blob is ciphertext, NOT the plaintext.
        let sealed: Vec<u8> =
            sqlx::query("SELECT vapid_private_sealed FROM push_config WHERE id = 1")
                .fetch_one(&s.pool)
                .await
                .unwrap()
                .get("vapid_private_sealed");
        assert_ne!(sealed.as_slice(), private.as_slice());

        // Unsealing recovers the original private key.
        let (public, recovered) = s.load_vapid_keypair().await.unwrap().unwrap();
        assert_eq!(public, "PUBLIC_KEY");
        assert_eq!(recovered, private);
    }

    #[tokio::test]
    async fn native_session_crud() {
        let s = store().await;
        let row = NativeSessionRow {
            token_hash: "hash1".into(),
            account_id: "acct".into(),
            client_type: "native".into(),
            created_at: "2026-07-13T00:00:00Z".into(),
            last_seen: "2026-07-13T00:00:00Z".into(),
            rotated_from: None,
        };
        s.create_native_session(&row).await.unwrap();
        assert_eq!(
            s.get_native_session("hash1")
                .await
                .unwrap()
                .unwrap()
                .account_id,
            "acct"
        );
        s.delete_native_session("hash1").await.unwrap();
        assert!(s.get_native_session("hash1").await.unwrap().is_none());
    }
}
