//! V8 (0009) SSO config + login-audit repository methods — additive, dual-backend
//! (t9-e0). New `Store` methods + a row struct over the 0009 `sso_config` /
//! `sso_login_audit` tables, reusing the frozen dual-backend query layer and the V6
//! sealed-column pattern; no existing query or public item is touched.
//!
//! `mw-store` stays free of any `mw-sso` dependency: [`SsoConfigRow`] carries plain
//! fields + opaque JSON strings; the server (e3) maps them to the `mw_sso` types.
//! The `secret` (OIDC client_secret / SAML SP private key) is held **plaintext** in
//! the row and sealed/unsealed transparently against the store's `ServerKey` on
//! write/read — the DB only ever holds the sealed `secret_sealed` BLOB.
//!
//! `sso_login_audit` is APPEND-ONLY and CONTENT-FREE (§21.1): the caller passes a
//! `subject_hash` (a HASH of the IdP sub/NameID it computed) and an outcome — this
//! module never sees a raw subject, token, assertion, or mail content.

use chrono::Utc;

use crate::backend::q;
use crate::{Store, StoreError, seal};

/// An `sso_config` row (0009). `config_json` is the serialized kind-specific config
/// (no secrets); `secret` is the **plaintext** OIDC client_secret / SAML SP private
/// key (sealed on write, opened on read); `claim_map_json` is the serialized
/// claim/attribute mapping. `scope` is `'deployment' | 'domain:<d>'`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsoConfigRow {
    pub id: String,
    pub kind: String,
    pub display_name: String,
    pub scope: String,
    pub enabled: bool,
    pub config_json: String,
    pub secret: Option<Vec<u8>>,
    pub claim_map_json: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Store {
    // ── sso_config ───────────────────────────────────────────────────────────

    fn map_sso_row(&self, r: &crate::Row) -> Result<SsoConfigRow, StoreError> {
        let secret = match r.get_opt_blob("secret_sealed") {
            Some(sealed) => Some(self.key.open(&sealed)?),
            None => None,
        };
        Ok(SsoConfigRow {
            id: r.get_string("id"),
            kind: r.get_string("kind"),
            display_name: r.get_string("display_name"),
            scope: r.get_string("scope"),
            enabled: r.get_i64("enabled") != 0,
            config_json: r.get_string("config"),
            secret,
            claim_map_json: r.get_string("claim_map"),
            created_at: r.get_string("created_at"),
            updated_at: r.get_string("updated_at"),
        })
    }

    /// Read every SSO backend in a scope (`'deployment'` or `'domain:<d>'`),
    /// id-ordered. Secrets are unsealed under the store key.
    pub async fn list_sso_config(&self, scope: &str) -> Result<Vec<SsoConfigRow>, StoreError> {
        let rows = q(
            "SELECT id, kind, display_name, scope, enabled, config, secret_sealed,
                             claim_map, created_at, updated_at
                      FROM sso_config WHERE scope = ?1 ORDER BY id ASC",
        )
        .bind(scope)
        .fetch_all(&self.backend)
        .await?;
        rows.iter().map(|r| self.map_sso_row(r)).collect()
    }

    /// Read one SSO backend by id (secret unsealed), or `None`.
    pub async fn get_sso_config(&self, id: &str) -> Result<Option<SsoConfigRow>, StoreError> {
        let row = q(
            "SELECT id, kind, display_name, scope, enabled, config, secret_sealed,
                            claim_map, created_at, updated_at
                     FROM sso_config WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.backend)
        .await?;
        row.map(|r| self.map_sso_row(&r)).transpose()
    }

    /// Upsert an SSO backend (by `id`), sealing `secret` under the store key.
    /// `created_at` is preserved on conflict; `updated_at` is taken from the row.
    pub async fn put_sso_config(&self, row: &SsoConfigRow) -> Result<(), StoreError> {
        let sealed = match &row.secret {
            Some(s) => Some(self.key.seal(s)?),
            None => None,
        };
        q(
            "INSERT INTO sso_config (id, kind, display_name, scope, enabled, config, secret_sealed,
                                   claim_map, created_at, updated_at)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
           ON CONFLICT(id) DO UPDATE SET
             kind = excluded.kind, display_name = excluded.display_name, scope = excluded.scope,
             enabled = excluded.enabled, config = excluded.config,
             secret_sealed = excluded.secret_sealed, claim_map = excluded.claim_map,
             updated_at = excluded.updated_at",
        )
        .bind(&row.id)
        .bind(&row.kind)
        .bind(&row.display_name)
        .bind(&row.scope)
        .bind(i64::from(row.enabled))
        .bind(&row.config_json)
        .bind(sealed)
        .bind(&row.claim_map_json)
        .bind(&row.created_at)
        .bind(&row.updated_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Delete an SSO backend by id (idempotent).
    pub async fn delete_sso_config(&self, id: &str) -> Result<(), StoreError> {
        q("DELETE FROM sso_config WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ── sso_login_audit ──────────────────────────────────────────────────────

    /// Append a **content-free** SSO login-audit row (§21.1). `subject_hash` MUST be
    /// a HASH of the IdP subject/NameID (the caller computes it) — NEVER the raw
    /// value; NO tokens/assertions/mail content are ever written. `outcome` is
    /// `'ok'` or `'error:<reason>'`.
    pub async fn append_sso_login_audit(
        &self,
        provider_id: &str,
        kind: &str,
        subject_hash: &str,
        outcome: &str,
    ) -> Result<(), StoreError> {
        q(
            "INSERT INTO sso_login_audit (id, ts, provider_id, kind, subject_hash, outcome)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(seal::random_token())
        .bind(Utc::now().to_rfc3339())
        .bind(provider_id)
        .bind(kind)
        .bind(subject_hash)
        .bind(outcome)
        .execute(&self.backend)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;
    use crate::backend::q;

    fn oidc_row() -> SsoConfigRow {
        SsoConfigRow {
            id: "corp-oidc".into(),
            kind: "oidc".into(),
            display_name: "Acme SSO".into(),
            scope: "deployment".into(),
            enabled: true,
            config_json: r#"{"kind":"oidc","issuer_url":"https://idp.example"}"#.into(),
            secret: Some(b"client-secret".to_vec()),
            claim_map_json: r#"{"email":"email"}"#.into(),
            created_at: "2026-07-14T00:00:00Z".into(),
            updated_at: "2026-07-14T00:00:00Z".into(),
        }
    }

    #[tokio::test]
    async fn sso_config_round_trips_with_sealed_secret() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        assert!(store.get_sso_config("corp-oidc").await.unwrap().is_none());
        store.put_sso_config(&oidc_row()).await.unwrap();

        let got = store.get_sso_config("corp-oidc").await.unwrap().unwrap();
        assert_eq!(got, oidc_row());
        assert_eq!(got.secret.as_deref(), Some(&b"client-secret"[..]));

        // The DB column is sealed ciphertext, never the plaintext secret.
        let raw = q("SELECT secret_sealed FROM sso_config WHERE id = ?1")
            .bind("corp-oidc")
            .fetch_optional(store.backend())
            .await
            .unwrap()
            .unwrap()
            .get_blob("secret_sealed");
        assert_ne!(raw, b"client-secret");
        assert!(!raw.is_empty());
    }

    #[tokio::test]
    async fn list_is_scoped_and_upsert_updates() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        store.put_sso_config(&oidc_row()).await.unwrap();
        let mut domain = oidc_row();
        domain.id = "acme-saml".into();
        domain.kind = "saml".into();
        domain.scope = "domain:acme.test".into();
        domain.secret = None;
        store.put_sso_config(&domain).await.unwrap();

        assert_eq!(store.list_sso_config("deployment").await.unwrap().len(), 1);
        let scoped = store.list_sso_config("domain:acme.test").await.unwrap();
        assert_eq!(scoped.len(), 1);
        assert_eq!(scoped[0].id, "acme-saml");
        assert_eq!(scoped[0].secret, None);

        // Upsert flips enabled + drops the secret.
        let mut updated = oidc_row();
        updated.enabled = false;
        updated.secret = None;
        store.put_sso_config(&updated).await.unwrap();
        let got = store.get_sso_config("corp-oidc").await.unwrap().unwrap();
        assert!(!got.enabled);
        assert_eq!(got.secret, None);
    }

    #[tokio::test]
    async fn delete_removes() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        store.put_sso_config(&oidc_row()).await.unwrap();
        store.delete_sso_config("corp-oidc").await.unwrap();
        assert!(store.get_sso_config("corp-oidc").await.unwrap().is_none());
        // Idempotent.
        store.delete_sso_config("corp-oidc").await.unwrap();
    }

    #[tokio::test]
    async fn login_audit_is_append_only_and_content_free() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        store
            .append_sso_login_audit("corp-oidc", "oidc", "hash-of-subject", "ok")
            .await
            .unwrap();
        store
            .append_sso_login_audit("corp-oidc", "oidc", "hash-of-subject", "error:replay")
            .await
            .unwrap();
        let n = q("SELECT COUNT(*) FROM sso_login_audit")
            .fetch_scalar_i64(store.backend())
            .await
            .unwrap();
        assert_eq!(n, 2);
    }
}
