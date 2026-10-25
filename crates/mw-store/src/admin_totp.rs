//! Atomic administrator self-enrollment, separate from mailbox MFA.
use crate::{Store, StoreError, backend::q};

impl Store {
    /// Confirm a first TOTP factor and seed recovery codes in the same transaction.
    /// `None` means the account/session is no longer authorized or already enrolled.
    /// `Some(true)` means these recovery hashes were newly stored and may be shown once.
    pub async fn enroll_admin_totp_with_recovery(
        &self,
        account: &str,
        session: &str,
        secret: &[u8],
        step: i64,
        now: i64,
        hashes: &[String],
    ) -> Result<Option<bool>, StoreError> {
        let sealed = self.key.seal(secret)?;
        let mut tx = self.backend.begin().await?;
        if q("UPDATE admin_account SET updated_at=updated_at WHERE id=?1 AND disabled=0")
            .bind(account)
            .execute_tx(&mut tx)
            .await?
            != 1
        {
            return Ok(None);
        }
        if q("UPDATE admin_sessions SET last_seen=last_seen WHERE token_hash=?1 AND admin_id=(SELECT username FROM admin_account WHERE id=?2) AND expires_at>?3 AND absolute_expires_at>?3")
            .bind(session).bind(account).bind(now).execute_tx(&mut tx).await? != 1 {
            return Ok(None);
        }
        if q("INSERT INTO admin_totp(account_id,sealed_secret,last_step,enrolled_at) VALUES(?1,?2,?3,?4) ON CONFLICT(account_id) DO NOTHING")
            .bind(account).bind(sealed).bind(step).bind(now).execute_tx(&mut tx).await? != 1 {
            return Ok(None);
        }
        // Used hashes count too: re-enrollment must not silently replace a previously
        // issued recovery set. Explicit rotation is a separate sensitive action.
        let count = q("SELECT COUNT(*) AS n FROM admin_recovery_codes WHERE account_id=?1")
            .bind(account)
            .fetch_all_tx(&mut tx)
            .await?
            .first()
            .map_or(0, |r| r.get_i64("n"));
        let seeded = count == 0 && !hashes.is_empty();
        if seeded {
            for hash in hashes {
                q("INSERT INTO admin_recovery_codes(account_id,code_hash,used) VALUES(?1,?2,0)")
                    .bind(account)
                    .bind(hash)
                    .execute_tx(&mut tx)
                    .await?;
            }
        }
        tx.commit().await?;
        Ok(Some(seeded))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn exercise(store: Store) {
        let id = format!("totp-self-{:032x}", rand::random::<u128>());
        let session = format!("session-{id}");
        let account = crate::AdminAccountRow {
            id: id.clone(),
            username: id.clone(),
            display_name: id.clone(),
            password_hash: Some("synthetic-hash".into()),
            disabled: false,
            source: "db".into(),
            created_at: 100,
            updated_at: 100,
            last_login_at: None,
        };
        store.insert_admin_account(&account).await.unwrap();
        let hashes = vec!["synthetic-recovery-a".into(), "synthetic-recovery-b".into()];
        assert_eq!(
            store
                .enroll_admin_totp_with_recovery(&id, &session, b"secret-a", 3, 100, &hashes)
                .await
                .unwrap(),
            None
        );
        assert!(store.get_admin_totp(&id).await.unwrap().is_none());
        store
            .put_admin_session_with_timeouts(&session, &id, 100, 3600, 7200)
            .await
            .unwrap();
        let (a, b) = tokio::join!(
            store.enroll_admin_totp_with_recovery(&id, &session, b"secret-a", 3, 100, &hashes),
            store.enroll_admin_totp_with_recovery(&id, &session, b"secret-b", 3, 100, &hashes),
        );
        let outcomes = [a.unwrap(), b.unwrap()];
        assert_eq!(outcomes.iter().filter(|v| **v == Some(true)).count(), 1);
        assert_eq!(outcomes.iter().filter(|v| v.is_none()).count(), 1);
        assert_eq!(store.admin_recovery_hashes(&id).await.unwrap().len(), 2);
        assert!(
            !store.advance_admin_totp_step(&id, 3).await.unwrap(),
            "enrollment code already consumed"
        );
        assert!(store.consume_admin_recovery(&id, &hashes[0]).await.unwrap());
        store.delete_admin_totp(&id).await.unwrap();
        assert_eq!(
            store
                .enroll_admin_totp_with_recovery(&id, &session, b"secret-c", 4, 100, &hashes)
                .await
                .unwrap(),
            Some(false)
        );
        assert_eq!(
            store.admin_recovery_hashes(&id).await.unwrap(),
            vec![hashes[1].clone()]
        );
        store.delete_admin_totp(&id).await.unwrap();
        store.delete_admin_session(&session).await.unwrap();
        assert_eq!(
            store
                .enroll_admin_totp_with_recovery(&id, &session, b"secret-d", 5, 100, &hashes)
                .await
                .unwrap(),
            None
        );
        assert!(store.get_admin_totp(&id).await.unwrap().is_none());
        store.delete_admin_account(&id).await.unwrap();
    }

    #[tokio::test]
    async fn admin_totp_atomic_enrollment_sqlite() {
        exercise(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn admin_totp_atomic_enrollment_postgres() {
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
