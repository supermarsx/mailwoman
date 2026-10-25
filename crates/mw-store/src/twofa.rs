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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WebauthnState {
    pub backup_eligible: Option<bool>,
    pub backup_state: bool,
    pub uv_initialized: bool,
    pub revision: i64,
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

/// Non-secret metadata for one active session (S11 session-management UI). The raw
/// session id is the bearer credential and is NEVER exposed; `handle` is a stable,
/// non-reversible truncated SHA-256 of it, safe to render and to revoke against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionMeta {
    /// Stable non-reversible display id (first 8 bytes of SHA-256 of the session id).
    pub handle: String,
    pub username: String,
    pub created_at: String,
    pub last_seen: String,
}

/// Stable, non-reversible display handle for a session id (first 8 bytes of its
/// SHA-256, hex). Used so the session-management surface can list and revoke
/// sessions without ever handing the opaque bearer token back to the client.
pub fn session_handle(id: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(id.as_bytes());
    digest[..8].iter().map(|b| format!("{b:02x}")).collect()
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

    /// The last TOTP step (RFC 6238 counter) a login consumed for this account, or
    /// `0` if none has been consumed / no secret exists (0021 `last_step`). A code
    /// whose matched step is `<= ` this has already been used and must be rejected.
    pub async fn totp_last_step(&self, account_id: &str) -> Result<i64, StoreError> {
        let row = q("SELECT last_step FROM totp_secrets WHERE account_id = ?1")
            .bind(account_id)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.map(|r| r.get_i64("last_step")).unwrap_or(0))
    }

    /// Advance the last-consumed TOTP step to `step`, but ONLY if it strictly
    /// increases the stored value (compare-and-swap over 0021 `last_step`, mirroring
    /// the single-use [`consume_recovery_code`](Self::consume_recovery_code) guard).
    /// Returns `true` iff this call won the advance — i.e. the step had not already
    /// been consumed, so the code is fresh. A replay of an already-consumed step, and
    /// two concurrent logins racing the same code, all resolve to exactly one winner
    /// because the DB `last_step < ?` predicate admits a single row update.
    pub async fn advance_totp_last_step(
        &self,
        account_id: &str,
        step: i64,
    ) -> Result<bool, StoreError> {
        let affected =
            q("UPDATE totp_secrets SET last_step = ?2 WHERE account_id = ?1 AND last_step < ?3")
                .bind(account_id)
                .bind(step)
                .bind(step)
                .execute(&self.backend)
                .await?;
        Ok(affected == 1)
    }

    // ---- WebAuthn ------------------------------------------------------------

    /// Register a WebAuthn credential (0015), setting `created_at` to now.
    pub async fn add_webauthn_credential(
        &self,
        cred: &WebauthnCredentialRow,
    ) -> Result<(), StoreError> {
        self.register_webauthn_credential(cred, &WebauthnState::default(), false)
            .await
    }

    /// Insert-only registration: an existing credential id never authorizes replacing a key.
    pub async fn register_webauthn_credential(
        &self,
        cred: &WebauthnCredentialRow,
        state: &WebauthnState,
        discoverable: bool,
    ) -> Result<(), StoreError> {
        let now = Utc::now().to_rfc3339();
        q("INSERT INTO webauthn_credentials
                 (credential_id, account_id, cose_public_key, sign_count, transports, label, created_at, backup_eligible, backup_state, uv_initialized, discoverable)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)")
        .bind(&cred.credential_id)
        .bind(&cred.account_id)
        .bind(&cred.cose_public_key)
        .bind(cred.sign_count)
        .bind(&cred.transports)
        .bind(&cred.label)
        .bind(&now)
        .bind(state.backup_eligible.map(i64::from))
        .bind(i64::from(state.backup_state))
        .bind(i64::from(state.uv_initialized))
        .bind(i64::from(discoverable))
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    pub async fn webauthn_state(
        &self,
        credential_id: &str,
    ) -> Result<Option<WebauthnState>, StoreError> {
        let row=q("SELECT backup_eligible,backup_state,uv_initialized,state_revision FROM webauthn_credentials WHERE credential_id=?1").bind(credential_id).fetch_optional(&self.backend).await?;
        Ok(row.map(|r| WebauthnState {
            backup_eligible: r.get_opt_i64("backup_eligible").map(|v| v != 0),
            backup_state: r.get_i64("backup_state") != 0,
            uv_initialized: r.get_i64("uv_initialized") != 0,
            revision: r.get_i64("state_revision"),
        }))
    }

    /// Commit exactly the verified credential version; no counter rollback, key
    /// replacement, revocation race, or immutable backup-eligibility change.
    pub async fn advance_webauthn_state(
        &self,
        cred: &WebauthnCredentialRow,
        previous: &WebauthnState,
        next: &WebauthnState,
        sign_count: i64,
    ) -> Result<bool, StoreError> {
        if !(0..=i64::from(u32::MAX)).contains(&sign_count)
            || sign_count < cred.sign_count
            || next.backup_eligible.is_none()
            || (next.backup_state && next.backup_eligible != Some(true))
            || previous
                .backup_eligible
                .is_some_and(|v| Some(v) != next.backup_eligible)
        {
            return Ok(false);
        }
        let changed=q("UPDATE webauthn_credentials SET sign_count=?1,backup_eligible=?2,backup_state=?3,uv_initialized=?4,state_revision=state_revision+1 WHERE credential_id=?5 AND account_id=?6 AND cose_public_key=?7 AND sign_count=?8 AND state_revision=?9 AND created_at=?10")
            .bind(sign_count).bind(next.backup_eligible.map(i64::from)).bind(i64::from(next.backup_state)).bind(i64::from(previous.uv_initialized||next.uv_initialized))
            .bind(&cred.credential_id).bind(&cred.account_id).bind(&cred.cose_public_key).bind(cred.sign_count).bind(previous.revision).bind(&cred.created_at).execute(&self.backend).await?;
        Ok(changed == 1)
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
        q("UPDATE webauthn_credentials SET sign_count = ?2 WHERE credential_id = ?1 AND sign_count < ?2")
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

impl Store {
    // ---- session management (S11) --------------------------------------------

    /// Non-secret metadata for every active session of `account_id`, most-recently
    /// seen first. Never returns the raw session id (see [`SessionMeta`]).
    pub async fn list_session_meta(
        &self,
        account_id: &str,
    ) -> Result<Vec<SessionMeta>, StoreError> {
        let rows = q("SELECT id, username, created_at, last_seen
                      FROM sessions WHERE account_id = ?1 ORDER BY last_seen DESC")
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| SessionMeta {
                handle: session_handle(&r.get_string("id")),
                username: r.get_string("username"),
                created_at: r.get_string("created_at"),
                last_seen: r.get_string("last_seen"),
            })
            .collect())
    }

    /// Revoke one session of `account_id` by its non-secret [`session_handle`].
    /// Returns the raw id iff a session was deleted, so the caller can drop the
    /// matching in-process guard entry; `None` means no session had that handle.
    pub async fn revoke_session_by_handle(
        &self,
        account_id: &str,
        handle: &str,
    ) -> Result<Option<String>, StoreError> {
        for id in self.session_ids_for_account(account_id).await? {
            if session_handle(&id) == handle {
                self.delete_session(&id).await?;
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// Revoke every session of `account_id` EXCEPT the one whose handle is
    /// `keep_handle` ("sign out everywhere else"). Returns the raw ids deleted.
    pub async fn revoke_other_sessions(
        &self,
        account_id: &str,
        keep_handle: &str,
    ) -> Result<Vec<String>, StoreError> {
        let mut deleted = Vec::new();
        for id in self.session_ids_for_account(account_id).await? {
            if session_handle(&id) != keep_handle {
                self.delete_session(&id).await?;
                deleted.push(id);
            }
        }
        Ok(deleted)
    }

    /// The raw session ids for an account (server-side only; never leaves the host).
    async fn session_ids_for_account(&self, account_id: &str) -> Result<Vec<String>, StoreError> {
        let rows = q("SELECT id FROM sessions WHERE account_id = ?1")
            .bind(account_id)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(|r| r.get_string("id")).collect())
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
    use crate::{Credentials, ServerKey};

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
    async fn totp_last_step_cas_is_monotonic() {
        let s = store().await;
        s.put_totp_secret("a1", b"12345678901234567890", true)
            .await
            .unwrap();

        // No step consumed yet.
        assert_eq!(s.totp_last_step("a1").await.unwrap(), 0);

        // First accept at step 100 wins and advances the marker.
        assert!(s.advance_totp_last_step("a1", 100).await.unwrap());
        assert_eq!(s.totp_last_step("a1").await.unwrap(), 100);

        // Replaying the same step is refused (not a strict increase) …
        assert!(!s.advance_totp_last_step("a1", 100).await.unwrap());
        // … as is any earlier step within a still-live ±window.
        assert!(!s.advance_totp_last_step("a1", 99).await.unwrap());
        assert_eq!(s.totp_last_step("a1").await.unwrap(), 100);

        // A later step still advances (a fresh code in a later window).
        assert!(s.advance_totp_last_step("a1", 101).await.unwrap());
        assert_eq!(s.totp_last_step("a1").await.unwrap(), 101);

        // Two logins racing the SAME fresh code (step 102, last_step is 101) — the
        // CAS admits EXACTLY ONE winner; the loser sees the already-advanced marker.
        let (r1, r2) = tokio::join!(
            s.advance_totp_last_step("a1", 102),
            s.advance_totp_last_step("a1", 102),
        );
        assert!(r1.unwrap() ^ r2.unwrap());
        assert_eq!(s.totp_last_step("a1").await.unwrap(), 102);
    }

    #[tokio::test]
    async fn passkey_registration_cannot_replace_another_account_key() {
        let s = store().await;
        let original = WebauthnCredentialRow {
            credential_id: "public-id".into(),
            account_id: "victim".into(),
            cose_public_key: vec![1],
            sign_count: 8,
            transports: String::new(),
            label: "original".into(),
            created_at: String::new(),
        };
        s.add_webauthn_credential(&original).await.unwrap();
        let mut attacker = original.clone();
        attacker.account_id = "attacker".into();
        attacker.cose_public_key = vec![2];
        attacker.sign_count = 0;
        assert!(
            s.add_webauthn_credential(&attacker).await.is_err(),
            "a public credential id cannot authorize replacing its owner key"
        );
        let stored = s
            .get_webauthn_credential("public-id")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.account_id, "victim");
        assert_eq!(stored.cose_public_key, vec![1]);
        assert_eq!(stored.sign_count, 8);
        assert!(
            s.add_webauthn_credential(&original).await.is_err(),
            "re-registration must not reset an existing credential counter"
        );
    }

    async fn assertion_state_cas(s: Store) {
        let id = format!("passkey-cas-{:032x}", rand::random::<u128>());
        let row = WebauthnCredentialRow {
            credential_id: id.clone(),
            account_id: "cas-owner".into(),
            cose_public_key: vec![9],
            sign_count: 5,
            transports: String::new(),
            label: String::new(),
            created_at: String::new(),
        };
        let initial = WebauthnState {
            backup_eligible: Some(true),
            backup_state: false,
            uv_initialized: true,
            revision: 0,
        };
        s.register_webauthn_credential(&row, &initial, true)
            .await
            .unwrap();
        let cred = s.get_webauthn_credential(&id).await.unwrap().unwrap();
        let old = s.webauthn_state(&id).await.unwrap().unwrap();
        let next = WebauthnState {
            backup_state: true,
            uv_initialized: false,
            ..old.clone()
        };
        let (a, b) = tokio::join!(
            s.advance_webauthn_state(&cred, &old, &next, 6),
            s.advance_webauthn_state(&cred, &old, &next, 6)
        );
        assert!(
            a.unwrap() ^ b.unwrap(),
            "only one assertion can commit a verified revision"
        );
        let updated = s.webauthn_state(&id).await.unwrap().unwrap();
        assert_eq!(updated.revision, 1);
        assert!(updated.backup_state);
        assert!(updated.uv_initialized);
        s.update_webauthn_sign_count(&id, 1).await.unwrap();
        let cred = s.get_webauthn_credential(&id).await.unwrap().unwrap();
        assert_eq!(cred.sign_count, 6);
        let changed = WebauthnState {
            backup_eligible: Some(false),
            backup_state: false,
            ..updated.clone()
        };
        assert!(
            !s.advance_webauthn_state(&cred, &updated, &changed, 7)
                .await
                .unwrap()
        );
        s.delete_webauthn_credential(&id).await.unwrap();
        assert!(
            !s.advance_webauthn_state(&cred, &updated, &next, 7)
                .await
                .unwrap()
        );
        let impostor = WebauthnCredentialRow {
            account_id: "other-owner".into(),
            cose_public_key: vec![8],
            ..cred.clone()
        };
        s.register_webauthn_credential(&impostor, &initial, true)
            .await
            .unwrap();
        assert!(
            !s.advance_webauthn_state(&cred, &initial, &next, 7)
                .await
                .unwrap()
        );
        s.delete_webauthn_credential(&id).await.unwrap();
    }

    #[tokio::test]
    async fn passkey_assertion_cas_sqlite() {
        assertion_state_cas(store().await).await;
    }

    #[tokio::test]
    async fn passkey_assertion_cas_postgres() {
        if let Ok(url) = std::env::var("MW_TEST_PG") {
            assertion_state_cas(Store::open(&url, ServerKey::generate()).await.unwrap()).await;
        }
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
    async fn session_meta_lists_and_revokes_by_handle() {
        let s = store().await;
        let creds = Credentials {
            username: "u@ex".into(),
            password: "pw".into(),
        };
        let a = s
            .create_session("acct", "u@ex", "j", "a", &creds)
            .await
            .unwrap();
        let b = s
            .create_session("acct", "u@ex", "j", "a", &creds)
            .await
            .unwrap();
        // A different account's session must never appear in acct's listing.
        let _other = s
            .create_session("other", "o@ex", "j", "a", &creds)
            .await
            .unwrap();

        let metas = s.list_session_meta("acct").await.unwrap();
        assert_eq!(metas.len(), 2);
        // The raw id is never surfaced; the handle is its truncated hash.
        assert!(metas.iter().all(|m| m.handle != a && m.handle != b));
        assert_eq!(session_handle(&a).len(), 16);

        // Revoke everything but a's session.
        let deleted = s
            .revoke_other_sessions("acct", &session_handle(&a))
            .await
            .unwrap();
        assert_eq!(deleted, vec![b.clone()]);
        assert_eq!(s.list_session_meta("acct").await.unwrap().len(), 1);
        assert!(s.get_session(&a).await.is_ok());
        assert!(s.get_session(&b).await.is_err());

        // Revoke a's session by handle; unknown handle is a no-op.
        assert!(
            s.revoke_session_by_handle("acct", "deadbeef")
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(
            s.revoke_session_by_handle("acct", &session_handle(&a))
                .await
                .unwrap(),
            Some(a.clone())
        );
        assert!(s.list_session_meta("acct").await.unwrap().is_empty());
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
