//! Administrator credentials have a separate namespace from mailbox credentials.
use crate::{Store, StoreError, WebauthnCredentialRow, WebauthnState, backend::q};

#[derive(Clone)]
pub struct AdminPasskey {
    pub credential: WebauthnCredentialRow,
    pub state: WebauthnState,
}

/// A verified factor must still exist when the session is minted.
pub enum AdminSessionFactor<'a> {
    Passkey(&'a AdminPasskey),
    Recovery { account: &'a str, hash: &'a str },
}

// Call after the invariant lock and before any account row lock. Recovery
// codes and TOTP require a password in the admin login flow; they are not an
// independent replacement for the last passwordless passkey.
async fn protects_credential_removal(tx: &mut crate::backend::Tx, account: &str, credential: Option<&str>) -> Result<bool, StoreError> {
    if !crate::admin_accounts::last_super_protected(tx, account).await? { return Ok(false); }
    let rows = q("SELECT password_hash FROM admin_account WHERE id=?1").bind(account).fetch_all_tx(tx).await?;
    if rows.first().and_then(|r| r.get_opt_string("password_hash")).is_some_and(|p| !p.is_empty()) { return Ok(false); }
    if let Some(credential) = credential {
        let others = q("SELECT credential_id FROM admin_webauthn_credentials WHERE account_id=?1 AND credential_id<>?2")
            .bind(account).bind(credential).fetch_all_tx(tx).await?;
        if !others.is_empty() { return Ok(false); }
    }
    Ok(true)
}

fn row(r: &crate::Row) -> AdminPasskey {
    AdminPasskey {
        credential: WebauthnCredentialRow {
            credential_id: r.get_string("credential_id"),
            account_id: r.get_string("account_id"),
            cose_public_key: r.get_blob("cose_public_key"),
            sign_count: r.get_i64("sign_count"),
            transports: r.get_string("transports"),
            label: r.get_string("label"),
            created_at: r.get_string("created_at"),
        },
        state: WebauthnState {
            backup_eligible: Some(r.get_i64("backup_eligible") != 0),
            backup_state: r.get_i64("backup_state") != 0,
            uv_initialized: r.get_i64("uv_initialized") != 0,
            revision: r.get_i64("state_revision"),
        },
    }
}

impl Store {
    pub async fn put_admin_session_for_factor(
        &self,
        token_hash: &str,
        username: &str,
        now: i64,
        timeouts: (i64, i64),
        factor: AdminSessionFactor<'_>,
    ) -> Result<bool, StoreError> {
        // Match ordinary admin sign-in retention without holding another account's lock.
        q("DELETE FROM admin_sessions WHERE expires_at<=?1 OR absolute_expires_at<=?1")
            .bind(now)
            .execute(&self.backend)
            .await?;
        let account = match &factor {
            AdminSessionFactor::Passkey(key) => key.credential.account_id.as_str(),
            AdminSessionFactor::Recovery { account, .. } => account,
        };
        let mut tx = self.backend.begin().await?;
        if q("UPDATE admin_account SET updated_at=updated_at WHERE id=?1 AND username=?2 AND disabled=0")
            .bind(account).bind(username).execute_tx(&mut tx).await? != 1 {
            // Dropping the transaction rolls back the account lock.
            return Ok(false);
        }
        let valid = match factor {
            AdminSessionFactor::Passkey(key) => {
                q("SELECT credential_id FROM admin_webauthn_credentials WHERE account_id=?1 AND credential_id=?2 AND cose_public_key=?3 AND created_at=?4 AND state_revision>=?5")
                    .bind(account).bind(&key.credential.credential_id).bind(&key.credential.cose_public_key)
                    .bind(&key.credential.created_at).bind(key.state.revision + 1).fetch_opt_scalar_string_tx(&mut tx).await?.is_some()
            }
            AdminSessionFactor::Recovery { hash, .. } => q("SELECT code_hash FROM admin_recovery_codes WHERE account_id=?1 AND code_hash=?2 AND used=1")
                .bind(account).bind(hash).fetch_opt_scalar_string_tx(&mut tx).await?.is_some(),
        };
        if !valid {
            // Dropping the transaction rolls back the account lock.
            return Ok(false);
        }
        let (idle, absolute) = (timeouts.0.clamp(300, 7200), timeouts.1.clamp(3600, 86400));
        let stamp = chrono::DateTime::from_timestamp(now, 0)
            .ok_or_else(|| StoreError::Corrupt("invalid session timestamp".into()))?
            .to_rfc3339();
        q("INSERT INTO admin_sessions(token_hash,admin_id,created_at,last_seen,expires_at,absolute_expires_at) VALUES(?1,?2,?3,?3,?4,?5)")
            .bind(token_hash).bind(username).bind(stamp).bind(now+idle.min(absolute)).bind(now+absolute).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(true)
    }
    /// Revoke every administrator passkey, recovery code and active session atomically.
    pub async fn revoke_admin_passkeys(&self, account: &str) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        crate::admin_accounts::lock_admin_accounts(&mut tx).await?;
        if protects_credential_removal(&mut tx, account, None).await? { return Ok(false); }
        q("UPDATE admin_account SET updated_at=updated_at WHERE id=?1")
            .bind(account)
            .execute_tx(&mut tx)
            .await?;
        for sql in [
            "DELETE FROM admin_webauthn_credentials WHERE account_id=?1",
            "DELETE FROM admin_recovery_codes WHERE account_id=?1",
            "DELETE FROM admin_sessions WHERE admin_id=(SELECT username FROM admin_account WHERE id=?1)",
        ] {
            q(sql).bind(account).execute_tx(&mut tx).await?;
        }
        tx.commit().await?;
        Ok(true)
    }
    pub async fn list_admin_passkeys(
        &self,
        account: &str,
    ) -> Result<Vec<AdminPasskey>, StoreError> {
        Ok(q("SELECT * FROM admin_webauthn_credentials WHERE account_id=?1 ORDER BY credential_id").bind(account).fetch_all(&self.backend).await?.iter().map(row).collect())
    }
    pub async fn get_admin_passkey(&self, id: &str) -> Result<Option<AdminPasskey>, StoreError> {
        Ok(
            q("SELECT * FROM admin_webauthn_credentials WHERE credential_id=?1")
                .bind(id)
                .fetch_optional(&self.backend)
                .await?
                .as_ref()
                .map(row),
        )
    }
    /// Insert only, bound to an enabled administrator, with an atomic per-account cap.
    pub async fn register_admin_passkey(
        &self,
        key: &AdminPasskey,
        recovery_hashes: &[String],
    ) -> Result<(bool, bool), StoreError> {
        let c = &key.credential;
        let s = &key.state;
        if !s.uv_initialized
            || s.backup_eligible.is_none()
            || (s.backup_state && s.backup_eligible != Some(true))
        {
            return Ok((false, false));
        }
        let mut tx = self.backend.begin().await?;
        if q("UPDATE admin_account SET updated_at=updated_at WHERE id=?1 AND disabled=0")
            .bind(&c.account_id)
            .execute_tx(&mut tx)
            .await?
            != 1
        {
            return Ok((false, false));
        }
        let inserted=q("INSERT INTO admin_webauthn_credentials (credential_id,account_id,cose_public_key,sign_count,transports,label,created_at,backup_eligible,backup_state,uv_initialized) SELECT ?1,?2,?3,?4,?5,?6,?7,?8,?9,1 WHERE (SELECT COUNT(*) FROM admin_webauthn_credentials WHERE account_id=?2)<20 ON CONFLICT(credential_id) DO NOTHING")
            .bind(&c.credential_id).bind(&c.account_id).bind(&c.cose_public_key).bind(c.sign_count).bind(&c.transports).bind(&c.label).bind(chrono::Utc::now().to_rfc3339()).bind(i64::from(s.backup_eligible.unwrap_or(false))).bind(i64::from(s.backup_state)).execute_tx(&mut tx).await? == 1;
        let mut seeded = false;
        if inserted {
            let count = q("SELECT COUNT(*) AS n FROM admin_recovery_codes WHERE account_id=?1")
                .bind(&c.account_id)
                .fetch_all_tx(&mut tx)
                .await?
                .first()
                .map_or(0, |r| r.get_i64("n"));
            if count == 0 {
                for hash in recovery_hashes {
                    q("INSERT INTO admin_recovery_codes(account_id,code_hash,used) VALUES(?1,?2,0)").bind(&c.account_id).bind(hash).execute_tx(&mut tx).await?;
                }
                seeded = true;
            }
        }
        tx.commit().await?;
        Ok((inserted, seeded))
    }
    pub async fn advance_admin_passkey(
        &self,
        old: &AdminPasskey,
        backup_state: bool,
        sign_count: i64,
    ) -> Result<bool, StoreError> {
        let c = &old.credential;
        if sign_count < c.sign_count
            || sign_count > i64::from(u32::MAX)
            || (backup_state && old.state.backup_eligible != Some(true))
        {
            return Ok(false);
        }
        Ok(q("UPDATE admin_webauthn_credentials SET sign_count=?1,backup_state=?2,state_revision=state_revision+1 WHERE credential_id=?3 AND account_id=?4 AND cose_public_key=?5 AND sign_count=?6 AND state_revision=?7 AND created_at=?8 AND EXISTS(SELECT 1 FROM admin_account WHERE id=?4 AND disabled=0)")
            .bind(sign_count).bind(i64::from(backup_state)).bind(&c.credential_id).bind(&c.account_id).bind(&c.cose_public_key).bind(c.sign_count).bind(old.state.revision).bind(&c.created_at).execute(&self.backend).await? == 1)
    }
    /// None means the final superadmin credential is protected; Some(false)
    /// means no owned credential matched. The decision and delete are atomic.
    pub async fn delete_admin_passkey(&self, account: &str, credential: &str) -> Result<Option<bool>, StoreError> {
        let mut tx = self.backend.begin().await?;
        crate::admin_accounts::lock_admin_accounts(&mut tx).await?;
        // Serialize with verified-factor session minting, which locks this row.
        q("UPDATE admin_account SET updated_at=updated_at WHERE id=?1").bind(account).execute_tx(&mut tx).await?;
        if protects_credential_removal(&mut tx, account, Some(credential)).await? { return Ok(None); }
        let removed = q("DELETE FROM admin_webauthn_credentials WHERE account_id=?1 AND credential_id=?2")
            .bind(account).bind(credential).execute_tx(&mut tx).await? == 1;
        tx.commit().await?;
        Ok(Some(removed))
    }
    pub async fn admin_recovery_hashes(&self, account: &str) -> Result<Vec<String>, StoreError> {
        Ok(
            q("SELECT code_hash FROM admin_recovery_codes WHERE account_id=?1 AND used=0")
                .bind(account)
                .fetch_all(&self.backend)
                .await?
                .iter()
                .map(|r| r.get_string("code_hash"))
                .collect(),
        )
    }
    pub async fn consume_admin_recovery(
        &self,
        account: &str,
        hash: &str,
    ) -> Result<bool, StoreError> {
        Ok(q("UPDATE admin_recovery_codes SET used=1 WHERE account_id=?1 AND code_hash=?2 AND used=0").bind(account).bind(hash).execute(&self.backend).await? == 1)
    }
    pub async fn replace_admin_recovery(
        &self,
        account: &str,
        hashes: &[String],
    ) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        q("UPDATE admin_account SET updated_at=updated_at WHERE id=?1")
            .bind(account)
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM admin_recovery_codes WHERE account_id=?1")
            .bind(account)
            .execute_tx(&mut tx)
            .await?;
        for hash in hashes {
            q("INSERT INTO admin_recovery_codes(account_id,code_hash,used) VALUES(?1,?2,0)")
                .bind(account)
                .bind(hash)
                .execute_tx(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    async fn exercise(store: Store) {
        let id = format!("admin-passkey-{:032x}", rand::random::<u128>());
        let account = crate::AdminAccountRow {
            id: id.clone(),
            username: id.clone(),
            display_name: id.clone(),
            password_hash: Some("fixture-hash".into()),
            disabled: false,
            source: "db".into(),
            created_at: 1,
            updated_at: 1,
            last_login_at: None,
        };
        store.insert_admin_account(&account).await.unwrap();
        let key = AdminPasskey {
            credential: WebauthnCredentialRow {
                credential_id: format!("key-{id}"),
                account_id: id.clone(),
                cose_public_key: vec![1, 2, 3],
                sign_count: 1,
                transports: "internal".into(),
                label: "proof".into(),
                created_at: String::new(),
            },
            state: WebauthnState {
                backup_eligible: Some(true),
                backup_state: false,
                uv_initialized: true,
                revision: 0,
            },
        };
        assert_eq!(
            store
                .register_admin_passkey(&key, &["recovery-hash".into()])
                .await
                .unwrap(),
            (true, true)
        );
        let mut replacement = key.clone();
        replacement.credential.cose_public_key = vec![7];
        assert_eq!(
            store
                .register_admin_passkey(&replacement, &[])
                .await
                .unwrap(),
            (false, false)
        );
        let other_id = format!("{id}-other");
        store
            .insert_admin_account(&crate::AdminAccountRow {
                id: other_id.clone(),
                username: other_id.clone(),
                ..account.clone()
            })
            .await
            .unwrap();
        replacement.credential.account_id = other_id.clone();
        assert_eq!(
            store
                .register_admin_passkey(&replacement, &[])
                .await
                .unwrap(),
            (false, false)
        );
        assert!(
            store
                .list_admin_passkeys(&other_id)
                .await
                .unwrap()
                .is_empty()
        );
        store.delete_admin_account(&other_id).await.unwrap();
        let stored = store
            .get_admin_passkey(&key.credential.credential_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            stored.credential.cose_public_key,
            key.credential.cose_public_key
        );
        assert!(
            store
                .get_webauthn_credential(&key.credential.credential_id)
                .await
                .unwrap()
                .is_none(),
            "mailbox verifier cannot see administrator credentials"
        );
        let (a, b) = tokio::join!(
            store.advance_admin_passkey(&stored, true, 2),
            store.advance_admin_passkey(&stored, true, 2)
        );
        assert!(a.unwrap() ^ b.unwrap());
        assert!(
            !store
                .delete_admin_passkey("wrong-account", &key.credential.credential_id)
                .await
                .unwrap().unwrap()
        );
        let (a, b) = tokio::join!(
            store.consume_admin_recovery(&id, "recovery-hash"),
            store.consume_admin_recovery(&id, "recovery-hash")
        );
        assert!(a.unwrap() ^ b.unwrap());
        store
            .replace_admin_recovery(&id, &["new-hash".into()])
            .await
            .unwrap();
        assert_eq!(
            store.admin_recovery_hashes(&id).await.unwrap(),
            vec!["new-hash"]
        );
        store.update_admin_account(&id, &id, true, 2).await.unwrap();
        let current = store
            .get_admin_passkey(&key.credential.credential_id)
            .await
            .unwrap()
            .unwrap();
        assert!(
            !store
                .advance_admin_passkey(&current, true, 3)
                .await
                .unwrap()
        );
        let session = format!("session-{id}");
        store
            .put_admin_session_with_timeouts(&session, &id, 100, 3600, 7200)
            .await
            .unwrap();
        store.revoke_admin_passkeys("nonexistent").await.unwrap();
        assert!(
            store
                .get_admin_passkey(&key.credential.credential_id)
                .await
                .unwrap()
                .is_some()
        );
        store
            .update_admin_account(&id, &id, false, 3)
            .await
            .unwrap();
        assert!(
            store
                .advance_admin_passkey(&current, true, 3)
                .await
                .unwrap()
        );
        assert!(store.consume_admin_recovery(&id, "new-hash").await.unwrap());
        let recovery_session = format!("recovery-{id}");
        assert!(
            store
                .put_admin_session_for_factor(
                    &recovery_session,
                    &id,
                    100,
                    (3600, 7200),
                    AdminSessionFactor::Recovery {
                        account: &id,
                        hash: "new-hash"
                    }
                )
                .await
                .unwrap()
        );
        let racing_session = format!("racing-{id}");
        let (mint, revoke) = tokio::join!(
            store.put_admin_session_for_factor(
                &racing_session,
                &id,
                100,
                (3600, 7200),
                AdminSessionFactor::Passkey(&current)
            ),
            store.revoke_admin_passkeys(&id),
        );
        mint.unwrap();
        revoke.unwrap();
        assert!(
            store
                .get_admin_session_with_idle(&recovery_session, 101, 3600)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .get_admin_session_with_idle(&racing_session, 101, 3600)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            !store
                .put_admin_session_for_factor(
                    "late-key",
                    &id,
                    100,
                    (3600, 7200),
                    AdminSessionFactor::Passkey(&current)
                )
                .await
                .unwrap()
        );
        assert!(
            !store
                .put_admin_session_for_factor(
                    "late-recovery",
                    &id,
                    100,
                    (3600, 7200),
                    AdminSessionFactor::Recovery {
                        account: &id,
                        hash: "new-hash"
                    }
                )
                .await
                .unwrap()
        );
        assert!(
            store
                .get_admin_session_with_idle(&session, 101, 3600)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            !store
                .advance_admin_passkey(&current, true, 3)
                .await
                .unwrap()
        );
        store.delete_admin_account(&id).await.unwrap();
        assert!(
            store
                .get_admin_passkey(&key.credential.credential_id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(store.admin_recovery_hashes(&id).await.unwrap().is_empty());
    }
    #[tokio::test]
    async fn admin_passkey_isolation_and_cas_sqlite() {
        exercise(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn admin_passkey_isolation_and_cas_postgres() {
        if let Ok(url) = std::env::var("MW_TEST_PG") {
            exercise(
                Store::open(&url, crate::ServerKey::generate())
                    .await
                    .unwrap(),
            )
            .await;
        }
    }
}
