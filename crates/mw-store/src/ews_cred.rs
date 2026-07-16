//! 0011 per-account EWS credential repository (t12 e-ews; SPEC §6.5/§27).
//!
//! Additive, dual-backend `Store` methods over the `ews_account_cred` table (0011,
//! both dialects). This is the sealed, host-held credential store for the on-prem
//! Exchange (EWS) bridge: the bridge's `wasm32-wasip2` guest is NTLM/Basic and needs
//! the CLEARTEXT password to derive NTOWFv2 (an OAuth bearer cannot serve it), so the
//! host holds the secret SEALED at rest and unseals it only to answer the guest's
//! gated `basic-credentials` import (`mw_plugin::BasicCredentialProvider`), which
//! e-mount backs with these rows at bridge mount.
//!
//! The `{user, domain, password, workstation}` quad is sealed as a unit (the same
//! zero-access posture as `sessions.sealed_creds` / `accounts.sealed_creds`); the
//! non-secret `endpoint` + `endpoint_host` stay plaintext (the host mirrors
//! `endpoint_host` into the bridge `net_allowlist`). The auth scheme is DERIVED at
//! use time — empty NT `domain` ⇒ Basic, non-empty ⇒ NTLMv2 — so no scheme column is
//! persisted. Authored in the SQLite `?n` style so it runs identically on SQLite or
//! Postgres through the frozen [`crate::backend`] dispatch layer.

use chrono::Utc;

use crate::backend::q;
use crate::{Store, StoreError, seal};

/// A per-account EWS credential binding (0011). The `user`/`domain`/`password`/
/// `workstation` fields are held decrypted only in memory; at rest they are sealed
/// together in the `sealed_cred` column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EwsAccountCred {
    pub account_id: String,
    /// The account's EWS SOAP endpoint URL (non-secret).
    pub endpoint: String,
    /// Host of `endpoint` (non-secret; mirrored into the bridge `net_allowlist`).
    pub endpoint_host: String,
    pub user: String,
    /// NT domain. Empty ⇒ the account uses HTTP Basic; non-empty ⇒ NTLMv2.
    pub domain: String,
    pub password: String,
    pub workstation: String,
    pub enabled: bool,
}

/// Seal the secret quad `{user, domain, password, workstation}` as a JSON tuple
/// (mirrors `encode_creds` for the 2-field session credential).
fn seal_ews_secret(key: &seal::ServerKey, c: &EwsAccountCred) -> Result<Vec<u8>, StoreError> {
    let plain = serde_json::to_vec(&(&c.user, &c.domain, &c.password, &c.workstation))
        .expect("ews credential encode");
    Ok(key.seal(&plain)?)
}

/// Open a sealed secret quad back into its four fields.
fn open_ews_secret(
    key: &seal::ServerKey,
    sealed: &[u8],
) -> Result<(String, String, String, String), StoreError> {
    let plain = key.open(sealed)?;
    serde_json::from_slice(&plain).map_err(|_| StoreError::Corrupt("ews credential decode".into()))
}

impl Store {
    /// Upsert a per-account EWS credential binding (by `account_id`), sealing the
    /// secret quad. Preserves `created_at` on conflict, bumps `updated_at`.
    pub async fn put_ews_account_cred(&self, cred: &EwsAccountCred) -> Result<(), StoreError> {
        let sealed = seal_ews_secret(&self.key, cred)?;
        let now = Utc::now().to_rfc3339();
        q("INSERT INTO ews_account_cred
                 (account_id, endpoint, endpoint_host, sealed_cred, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
             ON CONFLICT(account_id) DO UPDATE SET
                 endpoint = excluded.endpoint, endpoint_host = excluded.endpoint_host,
                 sealed_cred = excluded.sealed_cred, enabled = excluded.enabled,
                 updated_at = excluded.updated_at")
        .bind(&cred.account_id)
        .bind(&cred.endpoint)
        .bind(&cred.endpoint_host)
        .bind(sealed)
        .bind(i64::from(cred.enabled))
        .bind(&now)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Read + unseal the EWS credential binding for one account, if present.
    pub async fn get_ews_account_cred(
        &self,
        account_id: &str,
    ) -> Result<Option<EwsAccountCred>, StoreError> {
        let row = q(
            "SELECT account_id, endpoint, endpoint_host, sealed_cred, enabled
                     FROM ews_account_cred WHERE account_id = ?1",
        )
        .bind(account_id)
        .fetch_optional(&self.backend)
        .await?;
        let Some(r) = row else { return Ok(None) };
        let (user, domain, password, workstation) =
            open_ews_secret(&self.key, &r.get_blob("sealed_cred"))?;
        Ok(Some(EwsAccountCred {
            account_id: r.get_string("account_id"),
            endpoint: r.get_string("endpoint"),
            endpoint_host: r.get_string("endpoint_host"),
            user,
            domain,
            password,
            workstation,
            enabled: r.get_i64("enabled") != 0,
        }))
    }

    /// List every enabled EWS credential binding (account-id-ordered). e-mount uses
    /// this at boot to build the per-account credential provider + net_allowlist.
    pub async fn list_ews_account_creds(&self) -> Result<Vec<EwsAccountCred>, StoreError> {
        let rows = q(
            "SELECT account_id, endpoint, endpoint_host, sealed_cred, enabled
                      FROM ews_account_cred WHERE enabled <> 0 ORDER BY account_id ASC",
        )
        .fetch_all(&self.backend)
        .await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in &rows {
            let (user, domain, password, workstation) =
                open_ews_secret(&self.key, &r.get_blob("sealed_cred"))?;
            out.push(EwsAccountCred {
                account_id: r.get_string("account_id"),
                endpoint: r.get_string("endpoint"),
                endpoint_host: r.get_string("endpoint_host"),
                user,
                domain,
                password,
                workstation,
                enabled: r.get_i64("enabled") != 0,
            });
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    fn ntlm_cred() -> EwsAccountCred {
        EwsAccountCred {
            account_id: "acct-ntlm".into(),
            endpoint: "https://mail.corp.example/EWS/Exchange.asmx".into(),
            endpoint_host: "mail.corp.example".into(),
            user: "svc-mailwoman".into(),
            domain: "CORP".into(),
            password: "s3cr3t-ntlm".into(),
            workstation: "MAILWOMAN".into(),
            enabled: true,
        }
    }

    fn basic_cred() -> EwsAccountCred {
        EwsAccountCred {
            account_id: "acct-basic".into(),
            endpoint: "https://ews.example.com/EWS/Exchange.asmx".into(),
            endpoint_host: "ews.example.com".into(),
            user: "user@example.com".into(),
            domain: String::new(), // empty ⇒ Basic
            password: "s3cr3t-basic".into(),
            workstation: String::new(),
            enabled: true,
        }
    }

    #[tokio::test]
    async fn ews_cred_round_trips_sealed() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        assert!(
            store
                .get_ews_account_cred("acct-ntlm")
                .await
                .unwrap()
                .is_none()
        );

        store.put_ews_account_cred(&ntlm_cred()).await.unwrap();
        let got = store
            .get_ews_account_cred("acct-ntlm")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got, ntlm_cred());
        assert!(!got.domain.is_empty(), "NTLM account carries an NT domain");

        // The secret is NOT stored in plaintext — the persisted blob must not contain
        // the cleartext password bytes.
        let blob = q("SELECT sealed_cred FROM ews_account_cred WHERE account_id = ?1")
            .bind("acct-ntlm")
            .fetch_one(store.backend())
            .await
            .unwrap()
            .get_blob("sealed_cred");
        assert!(
            !blob
                .windows(b"s3cr3t-ntlm".len())
                .any(|w| w == b"s3cr3t-ntlm"),
            "password must be sealed, never plaintext at rest"
        );
    }

    #[tokio::test]
    async fn ews_cred_upsert_and_list_enabled_only() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        store.put_ews_account_cred(&ntlm_cred()).await.unwrap();
        store.put_ews_account_cred(&basic_cred()).await.unwrap();

        let all = store.list_ews_account_creds().await.unwrap();
        assert_eq!(all.len(), 2);
        // The Basic account carries an empty domain (⇒ Basic scheme at use time).
        let basic = all.iter().find(|c| c.account_id == "acct-basic").unwrap();
        assert!(basic.domain.is_empty());

        // Disable the NTLM binding via upsert; list drops it.
        let mut off = ntlm_cred();
        off.enabled = false;
        off.password = "rotated".into();
        store.put_ews_account_cred(&off).await.unwrap();
        let enabled = store.list_ews_account_creds().await.unwrap();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].account_id, "acct-basic");
        // The rotated secret round-trips through get().
        let got = store
            .get_ews_account_cred("acct-ntlm")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got.password, "rotated");
        assert!(!got.enabled);
    }
}
