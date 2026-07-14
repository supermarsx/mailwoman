//! V7 (0008) repository methods — additive, dual-backend (t7-e9).
//!
//! Fills the `passwd_config` table gap e0's 0008 left (it shipped
//! `password_change_audit` but not `passwd_config`) plus the small write paths the
//! V7 password-change server flow needs:
//!
//!   * [`Store::get_passwd_config`] / [`Store::put_passwd_config`] — the per-account
//!     password-change config (policy + forced-change flag). The stored `config`
//!     column is the JSON serialization of `mw_passwd::PasswdConfig`; `mw-store`
//!     stays free of any `mw-passwd` dependency — it round-trips the opaque JSON in a
//!     plain [`PasswdConfigRow`] and the server (e9) maps it to the crate type.
//!   * [`Store::put_password_change_audit`] — an append-only, **content-free** row in
//!     the 0008 `password_change_audit` table (account/backend/outcome only).
//!   * [`Store::reseal_account_credentials`] — the coordinated re-encryption of sealed
//!     upstream credentials after a successful change: every session for the account
//!     is re-sealed under the same [`ServerKey`] carrying the new password (the
//!     username is preserved). Powers `PasswordChangeOutcome.reencrypt_credentials`.
//!
//! Purely additive: new `Store` methods + one row struct, authored in the SQLite
//! `?n` style so they run identically on SQLite or Postgres through the frozen
//! [`crate::backend`] dispatch layer (backend-parity-identical for free). No existing
//! query, table, or public item is touched.

use chrono::Utc;

use crate::backend::q;
use crate::{Credentials, Store, StoreError, encode_creds, seal};

/// A `passwd_config` row (0008). `config` is opaque JSON (the serialized
/// `mw_passwd::PasswdConfig`); `force_change` mirrors the crate flag as a queryable
/// 0/1 column so a login path can gate on it without parsing the JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswdConfigRow {
    pub account_id: String,
    pub config_json: String,
    pub force_change: bool,
    pub updated_at: String,
}

impl Store {
    // ── passwd_config ────────────────────────────────────────────────────────

    /// Read the per-account password-change config, if any.
    pub async fn get_passwd_config(
        &self,
        account_id: &str,
    ) -> Result<Option<PasswdConfigRow>, StoreError> {
        let row = q("SELECT account_id, config, force_change, updated_at
                     FROM passwd_config WHERE account_id = ?1")
        .bind(account_id)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.map(|r| PasswdConfigRow {
            account_id: r.get_string("account_id"),
            config_json: r.get_string("config"),
            force_change: r.get_i64("force_change") != 0,
            updated_at: r.get_string("updated_at"),
        }))
    }

    /// Upsert the per-account password-change config (by `account_id`).
    pub async fn put_passwd_config(&self, row: &PasswdConfigRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO passwd_config (account_id, config, force_change, updated_at)
           VALUES (?1, ?2, ?3, ?4)
           ON CONFLICT(account_id) DO UPDATE SET
             config = excluded.config,
             force_change = excluded.force_change,
             updated_at = excluded.updated_at",
        )
        .bind(&row.account_id)
        .bind(&row.config_json)
        .bind(i64::from(row.force_change))
        .bind(&row.updated_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    // ── password_change_audit (append-only, content-free) ────────────────────

    /// Record a password-change audit row (append-only; account/backend/outcome
    /// only — **never** the old/new password, which never leave the server's
    /// `Secret`/`ServerKey` boundary).
    pub async fn put_password_change_audit(
        &self,
        account_id: &str,
        backend: &str,
        outcome: &str,
    ) -> Result<(), StoreError> {
        q(
            "INSERT INTO password_change_audit (id, ts, account_id, backend, outcome)
           VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(seal::random_token())
        .bind(Utc::now().to_rfc3339())
        .bind(account_id)
        .bind(backend)
        .bind(outcome)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    // ── coordinated credential re-seal ───────────────────────────────────────

    /// Re-encrypt (re-seal) every stored session credential for `account_id` under
    /// the new password, preserving each session's username. Returns the number of
    /// sessions re-sealed. Backs `PasswordChangeOutcome.reencrypt_credentials`: when
    /// the changed password equals the sealed upstream credential, the stored copy
    /// must be updated so subsequent proxy reads authenticate with the new password.
    pub async fn reseal_account_credentials(
        &self,
        account_id: &str,
        new_password: &str,
    ) -> Result<u64, StoreError> {
        let sessions = self.sessions_by_account(account_id).await?;
        let now = Utc::now().to_rfc3339();
        let mut count = 0u64;
        for s in sessions {
            let creds = Credentials {
                username: s.credentials.username.clone(),
                password: new_password.to_string(),
            };
            let sealed = self.key.seal(&encode_creds(&creds))?;
            q("UPDATE sessions SET sealed_creds = ?2, last_seen = ?3 WHERE id = ?1")
                .bind(&s.id)
                .bind(sealed)
                .bind(&now)
                .execute(&self.backend)
                .await?;
            count += 1;
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    fn creds() -> Credentials {
        Credentials {
            username: "u@example.org".into(),
            password: "old-pass".into(),
        }
    }

    #[tokio::test]
    async fn passwd_config_round_trips() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        assert!(store.get_passwd_config("a1").await.unwrap().is_none());

        let row = PasswdConfigRow {
            account_id: "a1".into(),
            config_json: r#"{"policy":{"min_length":16},"force_change_on_next_login":true}"#.into(),
            force_change: true,
            updated_at: "2026-07-14T00:00:00Z".into(),
        };
        store.put_passwd_config(&row).await.unwrap();
        let got = store.get_passwd_config("a1").await.unwrap().unwrap();
        assert_eq!(got, row);

        // Upsert overwrites in place.
        let updated = PasswdConfigRow {
            force_change: false,
            config_json: r#"{"policy":{"min_length":20}}"#.into(),
            ..row
        };
        store.put_passwd_config(&updated).await.unwrap();
        let got = store.get_passwd_config("a1").await.unwrap().unwrap();
        assert!(!got.force_change);
        assert_eq!(got.config_json, updated.config_json);
    }

    #[tokio::test]
    async fn password_change_audit_appends_content_free() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        store
            .put_password_change_audit("a1", "ldap3062", "ok")
            .await
            .unwrap();
        store
            .put_password_change_audit("a1", "local", "error:wrong-current")
            .await
            .unwrap();
        // Two independent rows (distinct ids) — append-only.
        let n = q("SELECT COUNT(*) FROM password_change_audit WHERE account_id = ?1")
            .bind("a1")
            .fetch_scalar_i64(store.backend())
            .await
            .unwrap();
        assert_eq!(n, 2);
    }

    #[tokio::test]
    async fn reseal_updates_all_sessions_to_new_password() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        store
            .create_session("a1", "u", "http://mock", "http://mock", &creds())
            .await
            .unwrap();
        store
            .create_session("a1", "u", "http://mock", "http://mock", &creds())
            .await
            .unwrap();
        store
            .create_session("a2", "u2", "http://mock", "http://mock", &creds())
            .await
            .unwrap();

        let n = store
            .reseal_account_credentials("a1", "new-pass")
            .await
            .unwrap();
        assert_eq!(n, 2);

        // a1 sessions now open to the new password (username preserved); a2 untouched.
        for s in store.sessions_by_account("a1").await.unwrap() {
            assert_eq!(s.credentials.username, "u@example.org");
            assert_eq!(s.credentials.password, "new-pass");
        }
        for s in store.sessions_by_account("a2").await.unwrap() {
            assert_eq!(s.credentials.password, "old-pass");
        }
    }
}
