//! Account-scoped authoritative loaders for ephemeral mail caches.
use crate::{Store, StoreError, backend::q};
impl Store {
    /// Ephemeral Redis mail is authenticated and bound to its complete cache
    /// namespace (class + account + revision/content address), never plaintext.
    pub fn seal_mail_cache(
        &self,
        namespace: &str,
        bytes: &[u8],
    ) -> Result<Vec<u8>, crate::SealError> {
        self.key
            .seal_bound(bytes, "mw-ephemeral-mail-cache-v2", &[namespace])
    }
    pub fn open_mail_cache(
        &self,
        namespace: &str,
        sealed: &[u8],
    ) -> Result<Vec<u8>, crate::SealError> {
        self.key
            .open_bound(sealed, "mw-ephemeral-mail-cache-v2", &[namespace])
    }
    pub async fn get_account_envelope(
        &self,
        account_id: &str,
        stable_id: &str,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let row = q("SELECT envelope_json FROM messages WHERE account_id=?1 AND stable_id=?2")
            .bind(account_id)
            .bind(stable_id)
            .fetch_optional(&self.backend)
            .await?;
        row.and_then(|r| r.get_opt_blob("envelope_json"))
            .map(|sealed| self.key.open(&sealed).map_err(StoreError::from))
            .transpose()
    }
    pub async fn get_account_body(
        &self,
        account_id: &str,
        blob_ref: &str,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let row = q("SELECT sealed_bytes FROM bodies WHERE account_id=?1 AND blob_ref=?2")
            .bind(account_id)
            .bind(blob_ref)
            .fetch_optional(&self.backend)
            .await?;
        row.map(|r| {
            self.key
                .open(&r.get_blob("sealed_bytes"))
                .map_err(StoreError::from)
        })
        .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey};
    async fn exercise(store: Store) {
        let user = format!("cache-{}@example.org", uuid::Uuid::new_v4());
        let account = store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "example.org",
                    port: 993,
                    tls: "implicit",
                    username: &user,
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: user.clone(),
                    password: "fixture".into(),
                },
            )
            .await
            .unwrap();
        let mailbox = store
            .upsert_mailbox(&MailboxUpsert {
                account_id: &account,
                name: "INBOX",
                role: Some("inbox"),
                uidvalidity: 1,
                uidnext: 2,
                highestmodseq: 1,
                total: 1,
                unread: 1,
                parent_id: None,
            })
            .await
            .unwrap();
        let message = MessageUpsert {
            account_id: &account,
            mailbox_id: &mailbox,
            uid: 1,
            uidvalidity: 1,
            message_id: Some("cache-fixture@example.org"),
            thread_id: None,
            internaldate: None,
            size: 8,
            flags_json: "[]",
            envelope: Some(b"old envelope"),
            blob_ref: None,
        };
        let (id, _) = store
            .upsert_message_with_body(&message, b"old body")
            .await
            .unwrap();
        let first = store.get_message(&id).await.unwrap();
        assert!(!first.cache_revision.is_empty());
        q("UPDATE messages SET flags_json=?1 WHERE stable_id=?2")
            .bind("[\"Seen\"]")
            .bind(&id)
            .execute(&store.backend)
            .await
            .unwrap();
        let second = store.get_message(&id).await.unwrap();
        assert_ne!(first.cache_revision, second.cache_revision);
        let mut tx = store.backend.begin().await.unwrap();
        q("UPDATE messages SET size=size+1 WHERE stable_id=?1")
            .bind(&id)
            .execute_tx(&mut tx)
            .await
            .unwrap();
        let in_tx = q("SELECT cache_revision FROM messages WHERE stable_id=?1")
            .bind(&id)
            .fetch_opt_scalar_string_tx(&mut tx)
            .await
            .unwrap()
            .unwrap();
        assert_ne!(in_tx, second.cache_revision);
        match tx {
            crate::backend::Tx::Sqlite(tx) => tx.rollback().await.unwrap(),
            crate::backend::Tx::Postgres(tx) => tx.rollback().await.unwrap(),
        }
        assert_eq!(
            store.get_message(&id).await.unwrap().cache_revision,
            second.cache_revision
        );
        let blob = first.blob_ref.as_deref().unwrap();
        assert!(
            store
                .repair_message_envelope(&account, &id, blob, b"repaired")
                .await
                .unwrap()
        );
        let repaired = store.get_message(&id).await.unwrap();
        assert_ne!(repaired.cache_revision, second.cache_revision);
        assert_eq!(
            store.get_account_envelope(&account, &id).await.unwrap(),
            Some(b"repaired".to_vec())
        );
        assert!(
            store
                .get_account_envelope("other-account", &id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .get_account_body("other-account", blob)
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(
            store.get_account_body(&account, blob).await.unwrap(),
            Some(b"old body".to_vec())
        );
        let (same, _) = store
            .upsert_message_with_body(&message, b"replacement body")
            .await
            .unwrap();
        assert_eq!(same, id);
        let replaced = store.get_message(&id).await.unwrap();
        assert_ne!(replaced.blob_ref, first.blob_ref);
        assert_ne!(replaced.cache_revision, repaired.cache_revision);
        // Raw SQL delete/reinsert of the exact same stable ID cannot resurrect
        // a previous revision, including with recursive_triggers enabled.
        q("DELETE FROM messages WHERE stable_id=?1")
            .bind(&id)
            .execute(&store.backend)
            .await
            .unwrap();
        q("INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity) VALUES(?1,?2,?3,1,1)")
            .bind(&id).bind(&account).bind(&mailbox).execute(&store.backend).await.unwrap();
        let reinserted = store.get_message(&id).await.unwrap();
        assert_ne!(reinserted.cache_revision, replaced.cache_revision);
        assert!(!reinserted.cache_revision.is_empty());
        let batch = store
            .get_messages(&[id.clone(), "missing".into(), id.clone()])
            .await
            .unwrap();
        assert_eq!(
            batch[0].as_ref().unwrap().cache_revision,
            reinserted.cache_revision
        );
        assert!(batch[1].is_none());
        assert_eq!(batch[2], batch[0]);
        q("DELETE FROM accounts WHERE id=?1")
            .bind(&account)
            .execute(&store.backend)
            .await
            .unwrap();
        store.close().await;
    }
    #[tokio::test]
    async fn shared_mail_payloads_are_bound_to_server_key_class_account_and_revision() {
        let first = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let other = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let namespace = "header-windows:mail-v2:account/message/revision";
        let sealed = first
            .seal_mail_cache(namespace, b"private mail bytes")
            .unwrap();
        assert_ne!(sealed, b"private mail bytes");
        assert_eq!(
            first.open_mail_cache(namespace, &sealed).unwrap(),
            b"private mail bytes"
        );
        assert_eq!(
            first.clone().open_mail_cache(namespace, &sealed).unwrap(),
            b"private mail bytes"
        );
        assert!(other.open_mail_cache(namespace, &sealed).is_err());
        for wrong in [
            "message-bodies:mail-v2:account/message/revision",
            "header-windows:mail-v2:other-account/message/revision",
            "header-windows:mail-v2:account/message/other-revision",
        ] {
            assert!(first.open_mail_cache(wrong, &sealed).is_err());
        }
        let mut tampered = sealed;
        tampered[30] ^= 1;
        assert!(first.open_mail_cache(namespace, &tampered).is_err());
    }

    #[tokio::test]
    async fn sqlite_cache_revision_covers_raw_writes_repair_rollback_and_aba() {
        exercise(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
    }
    #[tokio::test]
    async fn postgres_cache_revision_covers_raw_writes_repair_rollback_and_aba() {
        let url = std::env::var("MW_TEST_PG")
            .ok()
            .or_else(|| std::env::var("DATABASE_URL_PG").ok());
        let Some(url) = url else {
            mw_test_gate::skip("message cache revision: PostgreSQL fixture unavailable");
            return;
        };
        exercise(
            Store::open_postgres(&url, ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
}
