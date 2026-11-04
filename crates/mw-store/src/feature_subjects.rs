//! Durable feature targets for verified non-native primary identities.
//! Directory IDs and addresses are display metadata, never authentication proof.
use crate::{Store, StoreError, backend::q};

const BACKFILL_LIMIT: usize = 5000;

impl Store {
    /// Remember the policy owner verified by this primary session. Native users
    /// retain their existing account-based policy namespace. No caller-supplied
    /// raw account, address, or opaque owner is accepted here.
    pub async fn remember_feature_subject(&self, session: &str) -> Result<(), StoreError> {
        let Some(binding) = q("SELECT sp.binding_id,sp.primary_account_id,s.jmap_url,s.api_url,COALESCE(a.address,s.username) AS address FROM session_principal sp JOIN sessions s ON s.id=sp.session_id AND s.account_id=sp.primary_account_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=p.id LEFT JOIN mail_user_account a ON a.account_id=sp.primary_account_id AND lower(a.address)=lower(s.username) AND a.mode<>'child' WHERE sp.session_id=?1 AND s.linked_internal=0 AND p.state='active' AND b.retired_at IS NULL")
            .bind(session).fetch_optional(&self.backend).await? else { return Ok(()) };
        let Some(policy) = self.session_policy_subject(session).await? else {
            return Ok(());
        };
        if policy.native_account.is_some() {
            return Ok(());
        }
        let account = binding.get_string("primary_account_id");
        let binding_id = binding.get_string("binding_id");
        let address = self
            .canonical_policy_address(&binding.get_string("address"))
            .await?;
        // Recheck the verified tuple atomically at the write. Retirement, logout,
        // a selected child mailbox, or changed service endpoints cannot register
        // a new target after the authority used above has disappeared.
        q("INSERT INTO feature_user_subject(subject,account_id,address,binding_id) SELECT ?1,sp.primary_account_id,?2,sp.binding_id FROM session_principal sp JOIN sessions s ON s.id=sp.session_id AND s.account_id=sp.primary_account_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=p.id WHERE sp.session_id=?3 AND sp.primary_account_id=?4 AND sp.binding_id=?5 AND s.jmap_url=?6 AND s.api_url=?7 AND s.linked_internal=0 AND p.state='active' AND b.retired_at IS NULL ON CONFLICT(subject) DO UPDATE SET address=excluded.address WHERE feature_user_subject.account_id=excluded.account_id AND feature_user_subject.binding_id=excluded.binding_id")
            .bind(policy.personal_subject).bind(address).bind(session).bind(account).bind(binding_id)
            .bind(binding.get_string("jmap_url")).bind(binding.get_string("api_url"))
            .execute(&self.backend).await?;
        Ok(())
    }

    /// Upgrade existing verified sessions without adopting raw directory rows.
    /// Distinct service tuples are retained: the same binding/account may be
    /// authenticated against different proxy realms, each with its own owner.
    /// The bound is checked before any writes, so an unexpectedly large set
    /// cannot partially populate the registry or monopolize a request.
    pub async fn backfill_feature_subjects(&self) -> Result<(), StoreError> {
        let rows = q("SELECT MIN(sp.session_id) AS session_id FROM session_principal sp JOIN sessions s ON s.id=sp.session_id AND s.account_id=sp.primary_account_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=p.id WHERE s.linked_internal=0 AND p.state='active' AND b.retired_at IS NULL GROUP BY sp.binding_id,sp.primary_account_id,s.jmap_url,s.api_url ORDER BY MIN(sp.session_id) LIMIT 5001")
            .fetch_all(&self.backend).await?;
        if rows.len() > BACKFILL_LIMIT {
            return Err(StoreError::Corrupt(
                "feature subject backfill exceeds verified target limit".into(),
            ));
        }
        for row in rows {
            let session = row.get_string("session_id");
            let Some(policy) = self.session_policy_subject(&session).await? else {
                // Legacy sessions can contain malformed service URLs. Skip
                // them without adopting a raw ID or blocking unrelated users.
                if self.primary_principal(&session).await?.is_some() {
                    eprintln!(
                        "[mw-store] legacy feature profile target unavailable until fresh verified sign-in"
                    );
                }
                continue;
            };
            if policy.native_account.is_some() {
                continue;
            }
            if q("SELECT subject FROM feature_user_subject WHERE subject=?1")
                .bind(policy.personal_subject)
                .fetch_optional(&self.backend)
                .await?
                .is_some()
            {
                continue;
            }
            self.remember_feature_subject(&session).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Credentials, ServerKey, identity_principal::VerifiedPrimary};

    async fn session(store: &Store, service: &str) -> String {
        let id = store
            .create_session(
                "shared",
                "user@example.test",
                service,
                "https://mail.test/api",
                &Credentials {
                    username: "user".into(),
                    password: "synthetic".into(),
                },
            )
            .await
            .unwrap();
        store
            .set_session_authentication(&id, "header-auth")
            .await
            .unwrap();
        store
            .bind_verified_primary(
                &id,
                VerifiedPrimary {
                    account_id: "shared",
                    kind: "header",
                    authority: "trusted-header",
                    subject: "verified-user",
                },
                100,
            )
            .await
            .unwrap()
            .unwrap();
        id
    }

    async fn registered(store: &Store) -> Vec<String> {
        q("SELECT subject FROM feature_user_subject ORDER BY subject")
            .fetch_all(&store.backend)
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.get_string("subject"))
            .collect()
    }

    #[tokio::test]
    async fn requires_verified_primary_and_survives_logout() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        q("INSERT INTO mail_user_account(account_id,address,mode,first_seen,last_seen) VALUES('shared','someone-else@native.test','proxy',1,1)").execute(&store.backend).await.unwrap();
        let id = store
            .create_session(
                "shared",
                "user@example.test",
                "https://mail.test/session",
                "https://mail.test/api",
                &Credentials {
                    username: "user".into(),
                    password: "synthetic".into(),
                },
            )
            .await
            .unwrap();
        store.remember_feature_subject(&id).await.unwrap();
        assert!(
            registered(&store).await.is_empty(),
            "raw directory registration is not verification"
        );
        store
            .set_session_authentication(&id, "header-auth")
            .await
            .unwrap();
        store
            .bind_verified_primary(
                &id,
                VerifiedPrimary {
                    account_id: "shared",
                    kind: "header",
                    authority: "trusted-header",
                    subject: "verified-user",
                },
                100,
            )
            .await
            .unwrap()
            .unwrap();
        let subject = store
            .session_policy_subject(&id)
            .await
            .unwrap()
            .unwrap()
            .personal_subject;
        store.remember_feature_subject(&id).await.unwrap();
        let row = q("SELECT account_id,address FROM feature_user_subject WHERE subject=?1")
            .bind(&subject)
            .fetch_optional(&store.backend)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.get_string("account_id"), "shared");
        assert_eq!(
            row.get_string("address"),
            "user@example.test",
            "a raw-ID collision cannot supply another realm's policy domain"
        );
        store.delete_session(&id).await.unwrap();
        assert_eq!(
            registered(&store).await,
            vec![subject.clone()],
            "logout does not remove the durable assignment target"
        );
        let again = session(&store, "https://mail.test/session").await;
        store.remember_feature_subject(&again).await.unwrap();
        assert_eq!(
            registered(&store).await,
            vec![subject],
            "the same verified owner remains stable after login"
        );
    }

    #[tokio::test]
    async fn backfill_separates_realms_and_retired_bindings_are_not_active_targets() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let first = session(&store, "https://mail.test/a/session").await;
        let second = session(&store, "https://mail.test/b/session").await;
        let a = store
            .session_policy_subject(&first)
            .await
            .unwrap()
            .unwrap()
            .personal_subject;
        let b = store
            .session_policy_subject(&second)
            .await
            .unwrap()
            .unwrap()
            .personal_subject;
        assert_ne!(a, b);
        store.remember_feature_subject(&first).await.unwrap();
        store.backfill_feature_subjects().await.unwrap();
        assert_eq!(
            registered(&store).await.len(),
            2,
            "a known binding/account cannot hide a second realm"
        );
        let principal = store
            .primary_principal(&first)
            .await
            .unwrap()
            .unwrap()
            .principal_id;
        store.retire_principal(&principal, 200).await.unwrap();
        let active=q("SELECT f.subject FROM feature_user_subject f JOIN identity_binding b ON b.id=f.binding_id JOIN identity_principal p ON p.id=b.principal_id WHERE b.retired_at IS NULL AND p.state='active'").fetch_all(&store.backend).await.unwrap();
        assert!(
            active.is_empty(),
            "the registry is not authority after its binding is retired"
        );
        store.backfill_feature_subjects().await.unwrap();
        assert_eq!(
            registered(&store).await.len(),
            2,
            "durable tombstones do not recreate sessions or bindings"
        );
    }

    #[tokio::test]
    async fn native_namespace_is_unchanged_even_when_raw_proxy_id_collides() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        q("INSERT INTO accounts(id,kind,host,port,tls,username,sealed_creds) VALUES('shared','imap','native.test',993,'implicit','native@example.test',?1)").bind(vec![0u8]).execute(&store.backend).await.unwrap();
        let native = store
            .create_session(
                "shared",
                "native@example.test",
                "engine",
                "engine",
                &Credentials {
                    username: "native".into(),
                    password: "synthetic".into(),
                },
            )
            .await
            .unwrap();
        store
            .set_session_authentication(&native, "header-auth")
            .await
            .unwrap();
        store
            .bind_verified_primary(
                &native,
                VerifiedPrimary {
                    account_id: "shared",
                    kind: "header",
                    authority: "native-header",
                    subject: "native-user",
                },
                100,
            )
            .await
            .unwrap()
            .unwrap();
        store.remember_feature_subject(&native).await.unwrap();
        assert!(registered(&store).await.is_empty());
        let proxy = session(&store, "https://mail.test/session").await;
        store.remember_feature_subject(&proxy).await.unwrap();
        let policy = store.session_policy_subject(&proxy).await.unwrap().unwrap();
        assert!(policy.native_account.is_none());
        assert_eq!(registered(&store).await, vec![policy.personal_subject]);
    }

    #[tokio::test]
    async fn startup_backfills_existing_verified_sessions_without_a_new_login() {
        let path = std::env::temp_dir().join(format!(
            "mw-feature-subject-{}.sqlite",
            uuid::Uuid::new_v4()
        ));
        let key = ServerKey::generate();
        let store = Store::open(path.to_str().unwrap(), key.clone())
            .await
            .unwrap();
        let id = session(&store, "https://mail.test/session").await;
        let subject = store
            .session_policy_subject(&id)
            .await
            .unwrap()
            .unwrap()
            .personal_subject;
        assert!(registered(&store).await.is_empty());
        if let crate::backend::Backend::Sqlite(pool) = &store.backend {
            pool.close().await;
        }
        let reopened = Store::open(path.to_str().unwrap(), key).await.unwrap();
        assert_eq!(registered(&reopened).await, vec![subject]);
        if let crate::backend::Backend::Sqlite(pool) = &reopened.backend {
            pool.close().await;
        }
        let _ = std::fs::remove_file(path); // best effort: Windows may still hold the handle
    }

    #[tokio::test]
    async fn oversized_backfill_refuses_before_any_target_is_written() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let seed = session(&store, "https://mail.test/session").await;
        q("WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i+1 FROM n WHERE i<5000) INSERT INTO sessions(id,account_id,username,jmap_url,api_url,sealed_creds,created_at,last_seen,activity_at,authentication_method) SELECT 'extra-'||CAST(n.i AS TEXT),s.account_id,s.username,s.jmap_url||CAST(n.i AS TEXT),s.api_url,s.sealed_creds,s.created_at,s.last_seen,s.activity_at,s.authentication_method FROM n CROSS JOIN sessions s WHERE s.id=?1").bind(&seed).execute(&store.backend).await.unwrap();
        q("INSERT INTO session_principal(session_id,principal_id,binding_id,primary_account_id,auth_time,authentication_method) SELECT s.id,p.principal_id,p.binding_id,p.primary_account_id,p.auth_time,p.authentication_method FROM sessions s CROSS JOIN session_principal p WHERE p.session_id=?1 AND s.id<>?1").bind(&seed).execute(&store.backend).await.unwrap();
        assert!(store.backfill_feature_subjects().await.is_err());
        assert!(registered(&store).await.is_empty());
    }
}
