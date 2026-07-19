//! 0015 login second-factor repository (t16 e1 skeleton; e3 fills verification-side
//! callers): additive, dual-backend `Store` methods over `totp_secrets`,
//! `webauthn_credentials`, `recovery_codes`, and `twofa_policy` (0015, both dialects).
//!
//! Split of secrets: the TOTP shared key is SEALED at rest (XChaCha20-Poly1305 under
//! the store ServerKey), the same zero-access posture as `sessions.sealed_creds`; the
//! WebAuthn COSE public key is a verification key, stored opaque but NOT sealed;
//! recovery codes are stored only as argon2 hashes (the hash is produced by the
//! caller in `mw-mfa`) and are single-use via the `used` flag. All authored in the
//! SQLite `?n` style so they run identically on SQLite or Postgres through the frozen
//! [`crate::backend`] dispatch layer.

use chrono::Utc;

use crate::backend::q;
use crate::{Store, StoreError};

/// A TOTP authenticator (0015). `secret` is the raw shared HMAC key, held decrypted
/// only in memory; at rest it lives sealed in `sealed_secret`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TotpSecret {
    pub account_id: String,
    /// Raw TOTP shared key bytes (unsealed in memory only).
    pub secret: Vec<u8>,
    /// Enrolment verified (a valid code was presented at least once).
    pub confirmed: bool,
}

/// A registered WebAuthn credential (0015). The COSE public key is public
/// (verification only); `sign_count` is the last accepted signature counter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebauthnCredentialRow {
    pub credential_id: String,
    pub account_id: String,
    pub cose_public_key: Vec<u8>,
    pub sign_count: i64,
    pub transports: String,
    pub label: String,
    pub created_at: String,
}

/// An admin require-2FA policy row (0015): global (`scope_value` = "") or per-domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TwofaPolicyRow {
    /// "global" or "domain".
    pub scope_kind: String,
    /// "" for the global scope; the domain for a per-domain policy.
    pub scope_value: String,
    pub require_2fa: bool,
    pub updated_by: String,
    pub updated_at: String,
}

impl Store {
    // ---- TOTP ----------------------------------------------------------------

    /// Upsert an account's TOTP secret (sealing it), preserving `created_at` and
    /// setting the `confirmed` flag. Enrolment writes it unconfirmed; a later
    /// [`confirm_totp`](Self::confirm_totp) flips the flag once a code verifies.
    pub async fn put_totp_secret(
        &self,
        account_id: &str,
        secret: &[u8],
        confirmed: bool,
    ) -> Result<(), StoreError> {
        let sealed = self.key.seal(secret)?;
        let now = Utc::now().to_rfc3339();
        q(
            "INSERT INTO totp_secrets (account_id, sealed_secret, confirmed, created_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(account_id) DO UPDATE SET
                 sealed_secret = excluded.sealed_secret, confirmed = excluded.confirmed",
        )
        .bind(account_id)
        .bind(sealed)
        .bind(i64::from(confirmed))
        .bind(&now)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Read + unseal an account's TOTP secret, if present.
    pub async fn get_totp_secret(
        &self,
        account_id: &str,
    ) -> Result<Option<TotpSecret>, StoreError> {
        let row = q(
            "SELECT account_id, sealed_secret, confirmed FROM totp_secrets WHERE account_id = ?1",
        )
        .bind(account_id)
        .fetch_optional(&self.backend)
        .await?;
        let Some(r) = row else { return Ok(None) };
        let secret = self.key.open(&r.get_blob("sealed_secret"))?;
        Ok(Some(TotpSecret {
            account_id: r.get_string("account_id"),
            secret,
            confirmed: r.get_i64("confirmed") != 0,
        }))
    }

    /// Mark an account's TOTP enrolment confirmed.
    pub async fn confirm_totp(&self, account_id: &str) -> Result<(), StoreError> {
        q("UPDATE totp_secrets SET confirmed = 1 WHERE account_id = ?1")
            .bind(account_id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Remove an account's TOTP secret (disenrolment).
    pub async fn delete_totp_secret(&self, account_id: &str) -> Result<(), StoreError> {
        q("DELETE FROM totp_secrets WHERE account_id = ?1")
            .bind(account_id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ---- WebAuthn ------------------------------------------------------------

    /// Register a WebAuthn credential (0015), setting `created_at` to now.
    pub async fn add_webauthn_credential(
        &self,
        cred: &WebauthnCredentialRow,
    ) -> Result<(), StoreError> {
        let now = Utc::now().to_rfc3339();
        q("INSERT INTO webauthn_credentials
                 (credential_id, account_id, cose_public_key, sign_count, transports, label, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(credential_id) DO UPDATE SET
                 cose_public_key = excluded.cose_public_key, sign_count = excluded.sign_count,
                 transports = excluded.transports, label = excluded.label")
        .bind(&cred.credential_id)
        .bind(&cred.account_id)
        .bind(&cred.cose_public_key)
        .bind(cred.sign_count)
        .bind(&cred.transports)
        .bind(&cred.label)
        .bind(&now)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// A single WebAuthn credential by its id (assertion lookup).
    pub async fn get_webauthn_credential(
        &self,
        credential_id: &str,
    ) -> Result<Option<WebauthnCredentialRow>, StoreError> {
        let row = q("SELECT credential_id, account_id, cose_public_key, sign_count, transports, label, created_at
                     FROM webauthn_credentials WHERE credential_id = ?1")
            .bind(credential_id)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.as_ref().map(webauthn_from_row))
    }

    /// Every WebAuthn credential registered for an account (creation order).
    pub async fn list_webauthn_credentials(
        &self,
        account_id: &str,
    ) -> Result<Vec<WebauthnCredentialRow>, StoreError> {
        let rows = q("SELECT credential_id, account_id, cose_public_key, sign_count, transports, label, created_at
                      FROM webauthn_credentials WHERE account_id = ?1 ORDER BY created_at ASC")
            .bind(account_id)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(webauthn_from_row).collect())
    }

    /// Advance a credential's stored signature counter after a verified assertion.
    pub async fn update_webauthn_sign_count(
        &self,
        credential_id: &str,
        sign_count: i64,
    ) -> Result<(), StoreError> {
        q("UPDATE webauthn_credentials SET sign_count = ?2 WHERE credential_id = ?1")
            .bind(credential_id)
            .bind(sign_count)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Remove a WebAuthn credential.
    pub async fn delete_webauthn_credential(&self, credential_id: &str) -> Result<(), StoreError> {
        q("DELETE FROM webauthn_credentials WHERE credential_id = ?1")
            .bind(credential_id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ---- recovery codes ------------------------------------------------------

    /// Store a fresh set of argon2-hashed recovery codes for an account. Idempotent
    /// per hash (`ON CONFLICT DO NOTHING`); callers typically
    /// [`clear_recovery_codes`](Self::clear_recovery_codes) first to rotate.
    pub async fn add_recovery_codes(
        &self,
        account_id: &str,
        code_hashes: &[String],
    ) -> Result<(), StoreError> {
        let now = Utc::now().to_rfc3339();
        for hash in code_hashes {
            q(
                "INSERT INTO recovery_codes (account_id, code_hash, used, created_at)
                 VALUES (?1, ?2, 0, ?3)
                 ON CONFLICT(account_id, code_hash) DO NOTHING",
            )
            .bind(account_id)
            .bind(hash)
            .bind(&now)
            .execute(&self.backend)
            .await?;
        }
        Ok(())
    }

    /// Unused recovery-code hashes for an account (the caller argon2-verifies a
    /// presented code against these).
    pub async fn list_unused_recovery_codes(
        &self,
        account_id: &str,
    ) -> Result<Vec<String>, StoreError> {
        let rows = q("SELECT code_hash FROM recovery_codes WHERE account_id = ?1 AND used = 0")
            .bind(account_id)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(|r| r.get_string("code_hash")).collect())
    }

    /// Consume one recovery code by its hash. Returns `true` iff a not-yet-used code
    /// was flipped (single-use is enforced by the `used = 0` guard, so a replay
    /// returns `false`).
    pub async fn consume_recovery_code(
        &self,
        account_id: &str,
        code_hash: &str,
    ) -> Result<bool, StoreError> {
        let affected =
            q("UPDATE recovery_codes SET used = 1 WHERE account_id = ?1 AND code_hash = ?2 AND used = 0")
                .bind(account_id)
                .bind(code_hash)
                .execute(&self.backend)
                .await?;
        Ok(affected == 1)
    }

    /// Delete all recovery codes for an account (rotation / disenrolment).
    pub async fn clear_recovery_codes(&self, account_id: &str) -> Result<(), StoreError> {
        q("DELETE FROM recovery_codes WHERE account_id = ?1")
            .bind(account_id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ---- policy --------------------------------------------------------------

    /// Upsert an admin require-2FA policy (global or per-domain).
    pub async fn set_twofa_policy(&self, policy: &TwofaPolicyRow) -> Result<(), StoreError> {
        let now = Utc::now().to_rfc3339();
        q("INSERT INTO twofa_policy (scope_kind, scope_value, require_2fa, updated_by, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(scope_kind, scope_value) DO UPDATE SET
                 require_2fa = excluded.require_2fa, updated_by = excluded.updated_by,
                 updated_at = excluded.updated_at")
        .bind(&policy.scope_kind)
        .bind(&policy.scope_value)
        .bind(i64::from(policy.require_2fa))
        .bind(&policy.updated_by)
        .bind(&now)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Read a single require-2FA policy row by scope.
    pub async fn get_twofa_policy(
        &self,
        scope_kind: &str,
        scope_value: &str,
    ) -> Result<Option<TwofaPolicyRow>, StoreError> {
        let row = q(
            "SELECT scope_kind, scope_value, require_2fa, updated_by, updated_at
                     FROM twofa_policy WHERE scope_kind = ?1 AND scope_value = ?2",
        )
        .bind(scope_kind)
        .bind(scope_value)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(twofa_policy_from_row))
    }

    /// Every require-2FA policy row (admin listing).
    pub async fn list_twofa_policies(&self) -> Result<Vec<TwofaPolicyRow>, StoreError> {
        let rows = q(
            "SELECT scope_kind, scope_value, require_2fa, updated_by, updated_at
                      FROM twofa_policy ORDER BY scope_kind, scope_value",
        )
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(twofa_policy_from_row).collect())
    }
}

fn webauthn_from_row(r: &crate::backend::Row) -> WebauthnCredentialRow {
    WebauthnCredentialRow {
        credential_id: r.get_string("credential_id"),
        account_id: r.get_string("account_id"),
        cose_public_key: r.get_blob("cose_public_key"),
        sign_count: r.get_i64("sign_count"),
        transports: r.get_string("transports"),
        label: r.get_string("label"),
        created_at: r.get_string("created_at"),
    }
}

fn twofa_policy_from_row(r: &crate::backend::Row) -> TwofaPolicyRow {
    TwofaPolicyRow {
        scope_kind: r.get_string("scope_kind"),
        scope_value: r.get_string("scope_value"),
        require_2fa: r.get_i64("require_2fa") != 0,
        updated_by: r.get_string("updated_by"),
        updated_at: r.get_string("updated_at"),
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
    async fn totp_secret_round_trips_sealed() {
        let s = store().await;
        assert!(s.get_totp_secret("a1").await.unwrap().is_none());

        let secret = b"12345678901234567890";
        s.put_totp_secret("a1", secret, false).await.unwrap();
        let got = s.get_totp_secret("a1").await.unwrap().unwrap();
        assert_eq!(got.secret, secret);
        assert!(!got.confirmed);

        s.confirm_totp("a1").await.unwrap();
        assert!(s.get_totp_secret("a1").await.unwrap().unwrap().confirmed);

        // The shared key is sealed, never plaintext at rest.
        let blob = q("SELECT sealed_secret FROM totp_secrets WHERE account_id = ?1")
            .bind("a1")
            .fetch_one(s.backend())
            .await
            .unwrap()
            .get_blob("sealed_secret");
        assert!(!blob.windows(secret.len()).any(|w| w == secret));

        s.delete_totp_secret("a1").await.unwrap();
        assert!(s.get_totp_secret("a1").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn webauthn_credential_crud_and_counter() {
        let s = store().await;
        let cred = WebauthnCredentialRow {
            credential_id: "cred-abc".into(),
            account_id: "a1".into(),
            cose_public_key: vec![1, 2, 3, 4],
            sign_count: 0,
            transports: "usb,nfc".into(),
            label: "YubiKey".into(),
            created_at: String::new(),
        };
        s.add_webauthn_credential(&cred).await.unwrap();

        let got = s
            .get_webauthn_credential("cred-abc")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got.cose_public_key, vec![1, 2, 3, 4]);
        assert_eq!(got.account_id, "a1");
        assert!(!got.created_at.is_empty());

        s.update_webauthn_sign_count("cred-abc", 7).await.unwrap();
        assert_eq!(
            s.get_webauthn_credential("cred-abc")
                .await
                .unwrap()
                .unwrap()
                .sign_count,
            7
        );

        assert_eq!(s.list_webauthn_credentials("a1").await.unwrap().len(), 1);
        s.delete_webauthn_credential("cred-abc").await.unwrap();
        assert!(s.list_webauthn_credentials("a1").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn recovery_codes_single_use() {
        let s = store().await;
        s.add_recovery_codes("a1", &["h1".into(), "h2".into(), "h3".into()])
            .await
            .unwrap();
        assert_eq!(s.list_unused_recovery_codes("a1").await.unwrap().len(), 3);

        assert!(s.consume_recovery_code("a1", "h2").await.unwrap());
        // Replay of the same code is refused.
        assert!(!s.consume_recovery_code("a1", "h2").await.unwrap());
        assert_eq!(s.list_unused_recovery_codes("a1").await.unwrap().len(), 2);
        // Unknown code is refused.
        assert!(!s.consume_recovery_code("a1", "nope").await.unwrap());

        s.clear_recovery_codes("a1").await.unwrap();
        assert!(s.list_unused_recovery_codes("a1").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn twofa_policy_upsert_and_list() {
        let s = store().await;
        assert!(s.get_twofa_policy("global", "").await.unwrap().is_none());

        s.set_twofa_policy(&TwofaPolicyRow {
            scope_kind: "global".into(),
            scope_value: String::new(),
            require_2fa: true,
            updated_by: "admin@example.org".into(),
            updated_at: String::new(),
        })
        .await
        .unwrap();
        s.set_twofa_policy(&TwofaPolicyRow {
            scope_kind: "domain".into(),
            scope_value: "corp.example".into(),
            require_2fa: false,
            updated_by: "admin@example.org".into(),
            updated_at: String::new(),
        })
        .await
        .unwrap();

        assert!(
            s.get_twofa_policy("global", "")
                .await
                .unwrap()
                .unwrap()
                .require_2fa
        );
        assert_eq!(s.list_twofa_policies().await.unwrap().len(), 2);
    }
}
