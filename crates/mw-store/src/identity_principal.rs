//! Durable primary identities. Callers must supply a verified backend subject,
//! never an unverified email, alias, request parameter, or selected child mailbox.
use crate::{
    Store, StoreError,
    backend::{Row, q},
};
use serde::Serialize;

/// Authentication evidence constructed by a trusted sign-in handler.
/// The account must be the original authenticated session account. `authority`
/// identifies a stable configured realm (or a verified OIDC issuer/provider).
pub struct VerifiedPrimary<'a> {
    pub account_id: &'a str,
    pub kind: &'a str,
    pub authority: &'a str,
    pub subject: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrimaryPrincipal {
    pub principal_id: String,
    pub binding_id: String,
    pub primary_account_id: String,
    pub auth_time: i64,
    pub authentication_method: String,
}

/// Verified policy identity. Native compatibility is explicit, never inferred
/// from an upstream account ID that happens to exist in the local account table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedPolicySubject {
    pub personal_subject: String,
    pub native_account: Option<String>,
}

fn row(value: &Row) -> PrimaryPrincipal {
    PrimaryPrincipal {
        principal_id: value.get_string("principal_id"),
        binding_id: value.get_string("binding_id"),
        primary_account_id: value.get_string("primary_account_id"),
        auth_time: value.get_i64("auth_time"),
        authentication_method: value.get_string("authentication_method"),
    }
}

fn personal_subject(parts: Vec<String>) -> String {
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    digest.update(b"mailwoman-personal-subject-v1\0");
    for part in parts {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part.as_bytes());
    }
    let encoded: String = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("po1_{encoded}")
}

impl Store {
    /// Opaque browser-storage namespace. Raw upstream account IDs are not globally
    /// unique and must never serve as an authority for persisted private data.
    async fn verified_personal_parts(
        &self,
        session: &str,
    ) -> Result<Option<Vec<String>>, StoreError> {
        let Some(value) = q("SELECT sp.binding_id,sp.primary_account_id,s.jmap_url,s.api_url,b.kind,CAST(CASE WHEN a.id IS NULL THEN 0 ELSE 1 END AS BIGINT) AS native_account FROM session_principal sp JOIN sessions s ON s.id=sp.session_id AND s.account_id=sp.primary_account_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=p.id LEFT JOIN accounts a ON a.id=sp.primary_account_id WHERE sp.session_id=?1 AND p.state='active' AND b.retired_at IS NULL")
            .bind(session).fetch_optional(&self.backend).await? else { return Ok(None) };
        let account = value.get_string("primary_account_id");
        let binding = value.get_string("binding_id");
        let service = value.get_string("jmap_url");
        let api = value.get_string("api_url");
        let native = ((service == "engine" && api == "engine")
            || (service == "sso" && api == "sso"))
            && value.get_i64("native_account") == 1;
        let parts = if native {
            vec!["native".to_owned(), account]
        } else if service == "sso" && api == "sso" && value.get_string("kind") == "sso" {
            vec!["sso".to_owned(), binding, account]
        } else {
            let mut parts = vec!["proxy".to_owned(), binding, account];
            for endpoint in [&service, &api] {
                let Ok(mut url) = url::Url::parse(endpoint) else {
                    return Ok(None);
                };
                if !matches!(url.scheme(), "http" | "https")
                    || url.host_str().is_none()
                    || !url.username().is_empty()
                    || url.password().is_some()
                {
                    return Ok(None);
                }
                url.set_fragment(None);
                parts.push(url.to_string());
            }
            parts
        };
        Ok(Some(parts))
    }

    pub async fn session_storage_authority(
        &self,
        session: &str,
    ) -> Result<Option<String>, StoreError> {
        let Some(parts) = self.verified_personal_parts(session).await? else {
            return Ok(None);
        };
        // Length-prefixed fields prevent delimiter ambiguity. Neither realm URLs
        // nor external subjects are exposed to the browser.
        let mut message = Vec::new();
        for part in parts {
            message.extend_from_slice(&(part.len() as u64).to_be_bytes());
            message.extend_from_slice(part.as_bytes());
        }
        let digest: String =
            crate::seal::hmac_sha256(&self.key.derive("mw-client-storage-authority-v1"), &message)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
        Ok(Some(format!("msa1_{digest}")))
    }

    /// Durable database subject independent of the encryption key. Migration
    /// copies the verified tuple; this identifier is never an authorization credential.
    pub async fn session_personal_subject(
        &self,
        session: &str,
    ) -> Result<Option<String>, StoreError> {
        Ok(self
            .verified_personal_parts(session)
            .await?
            .map(personal_subject))
    }

    pub async fn session_policy_subject(
        &self,
        session: &str,
    ) -> Result<Option<VerifiedPolicySubject>, StoreError> {
        let Some(parts) = self.verified_personal_parts(session).await? else {
            return Ok(None);
        };
        let native_account =
            (parts.first().map(String::as_str) == Some("native")).then(|| parts[1].clone());
        Ok(Some(VerifiedPolicySubject {
            personal_subject: personal_subject(parts),
            native_account,
        }))
    }

    /// Only native engine/background callers with authoritative account access
    /// may use this adapter. Session routes must use session_personal_subject.
    pub async fn native_personal_subject(
        &self,
        account: &str,
    ) -> Result<Option<String>, StoreError> {
        if q("SELECT id FROM accounts WHERE id=?1")
            .bind(account)
            .fetch_optional(&self.backend)
            .await?
            .is_none()
        {
            return Ok(None);
        }
        Ok(Some(personal_subject(vec![
            "native".into(),
            account.into(),
        ])))
    }

    /// Atomically attach a verified primary identity, or refuse stale, retired,
    /// unverified or already differently bound sessions. No email-based merging.
    /// Authentication age is captured once and is not extended by a repeated call.
    pub async fn bind_verified_primary(
        &self,
        session: &str,
        verified: VerifiedPrimary<'_>,
        now: i64,
    ) -> Result<Option<PrimaryPrincipal>, StoreError> {
        if now < 0
            || !["mailbox", "sso", "header"].contains(&verified.kind)
            || [verified.account_id, verified.authority, verified.subject]
                .iter()
                .any(|v| v.is_empty() || v.len() > 4096 || v.chars().any(char::is_control))
        {
            return Ok(None);
        }
        let mut tx = self.backend.begin().await?;
        // Cross-process serialization covers first-login races and retirement.
        // Both backends acquire the write lock before any authority reads.
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        if q("UPDATE sessions SET last_seen=last_seen WHERE id=?1 AND account_id=?2 AND authentication_method IN ('password','password+factor','remembered-device','passkey','header-auth','sso')")
            .bind(session).bind(verified.account_id).execute_tx(&mut tx).await? != 1 { return Ok(None); }
        let methods = q("SELECT authentication_method FROM sessions WHERE id=?1")
            .bind(session)
            .fetch_all_tx(&mut tx)
            .await?;
        let method = methods[0].get_string("authentication_method");
        if (verified.kind == "sso") != (method == "sso")
            || (verified.kind == "header") != (method == "header-auth")
        {
            return Ok(None);
        }
        let bindings = q("SELECT b.id,b.principal_id,b.retired_at,p.state FROM identity_binding b JOIN identity_principal p ON p.id=b.principal_id WHERE b.kind=?1 AND b.authority=?2 AND b.subject=?3")
            .bind(verified.kind).bind(verified.authority).bind(verified.subject).fetch_all_tx(&mut tx).await?;
        let (binding, principal) = if let Some(binding) = bindings.first() {
            if binding.get_opt_i64("retired_at").is_some()
                || binding.get_string("state") != "active"
            {
                return Ok(None);
            }
            (binding.get_string("id"), binding.get_string("principal_id"))
        } else {
            let principal = uuid::Uuid::new_v4().to_string();
            let binding = uuid::Uuid::new_v4().to_string();
            q("INSERT INTO identity_principal(id,state,created_at) VALUES(?1,'active',?2)")
                .bind(&principal)
                .bind(now)
                .execute_tx(&mut tx)
                .await?;
            q("INSERT INTO identity_binding(id,principal_id,kind,authority,subject,created_at) VALUES(?1,?2,?3,?4,?5,?6)")
                .bind(&binding).bind(&principal).bind(verified.kind).bind(verified.authority).bind(verified.subject).bind(now).execute_tx(&mut tx).await?;
            (binding, principal)
        };
        q("INSERT INTO session_principal(session_id,principal_id,binding_id,primary_account_id,auth_time,authentication_method) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(session_id) DO NOTHING")
            .bind(session).bind(&principal).bind(&binding).bind(verified.account_id).bind(now).bind(&method).execute_tx(&mut tx).await?;
        let values = q("SELECT principal_id,binding_id,primary_account_id,auth_time,authentication_method FROM session_principal WHERE session_id=?1")
            .bind(session).fetch_all_tx(&mut tx).await?;
        let value = row(&values[0]);
        if value.principal_id != principal
            || value.binding_id != binding
            || value.primary_account_id != verified.account_id
        {
            return Ok(None);
        }
        tx.commit().await?;
        Ok(Some(value))
    }

    pub async fn primary_principal(
        &self,
        session: &str,
    ) -> Result<Option<PrimaryPrincipal>, StoreError> {
        Ok(q("SELECT sp.principal_id,sp.binding_id,sp.primary_account_id,sp.auth_time,sp.authentication_method FROM session_principal sp JOIN sessions s ON s.id=sp.session_id AND s.account_id=sp.primary_account_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=p.id WHERE sp.session_id=?1 AND p.state='active' AND b.retired_at IS NULL")
            .bind(session).fetch_optional(&self.backend).await?.as_ref().map(row))
    }

    /// Permanently retire an identity and revoke all of its primary sessions.
    /// Callers must separately authorize this operation. No later login can
    /// reclaim its subject, even if a mailbox address has been reassigned.
    pub async fn retire_principal(&self, principal: &str, now: i64) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        let changed = q("UPDATE identity_principal SET state='retired',retired_at=?2 WHERE id=?1 AND state!='retired'")
            .bind(principal).bind(now).execute_tx(&mut tx).await?;
        q("UPDATE identity_binding SET retired_at=?2 WHERE principal_id=?1 AND retired_at IS NULL")
            .bind(principal)
            .bind(now)
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM sessions WHERE id IN (SELECT session_id FROM session_principal WHERE principal_id=?1)")
            .bind(principal).execute_tx(&mut tx).await?;
        // t48: a retired identity keeps no stored linked-mailbox credential, as
        // the owner or as the linked mailbox.
        q("DELETE FROM linked_credential WHERE owner_principal_id=?1 OR link_id IN (SELECT id FROM linked_account WHERE child_principal_id=?1)")
            .bind(principal).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(changed == 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn storage_authority_separates_realms_and_survives_rotation() {
        storage_authority_exercise(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }

    #[tokio::test]
    async fn storage_authority_postgres() {
        use sqlx::Connection;
        let Ok(dsn) = std::env::var("MW_TEST_PG") else {
            eprintln!("SKIP: MW_TEST_PG absent");
            return;
        };
        let schema = format!("storage_authority_{}", uuid::Uuid::new_v4().simple());
        let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&mut admin)
            .await
            .unwrap();
        let mut scoped = url::Url::parse(&dsn).unwrap();
        scoped
            .query_pairs_mut()
            .append_pair("options", &format!("-csearch_path={schema}"));
        storage_authority_exercise(
            Store::open(scoped.as_str(), crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&mut admin)
            .await
            .unwrap();
    }

    async fn storage_authority_exercise(store: Store) {
        let mut ids = Vec::new();
        // Same trusted-header binding and raw account ID, different actual services.
        for service in [
            "https://mail.test/a/session",
            "https://mail.test/b/session",
            "https://mail.test/a/session#ignored",
        ] {
            let id = store
                .create_session(
                    "1",
                    "same@example.test",
                    service,
                    "https://mail.test/api",
                    &crate::Credentials {
                        username: "same".into(),
                        password: "synthetic".into(),
                    },
                )
                .await
                .unwrap();
            assert!(
                store
                    .session_storage_authority(&id)
                    .await
                    .unwrap()
                    .is_none()
            );
            store
                .set_session_authentication(&id, "header-auth")
                .await
                .unwrap();
            store
                .bind_verified_primary(
                    &id,
                    VerifiedPrimary {
                        account_id: "1",
                        kind: "header",
                        authority: "trusted-header",
                        subject: "verified",
                    },
                    100,
                )
                .await
                .unwrap()
                .unwrap();
            ids.push(id);
        }
        let owner_a = store
            .session_personal_subject(&ids[0])
            .await
            .unwrap()
            .unwrap();
        let owner_b = store
            .session_personal_subject(&ids[1])
            .await
            .unwrap()
            .unwrap();
        assert_ne!(owner_a, owner_b);
        let policy_a = store
            .session_policy_subject(&ids[0])
            .await
            .unwrap()
            .unwrap();
        let policy_b = store
            .session_policy_subject(&ids[1])
            .await
            .unwrap()
            .unwrap();
        assert_eq!(policy_a.personal_subject, owner_a);
        assert_eq!(policy_b.personal_subject, owner_b);
        assert!(policy_a.native_account.is_none() && policy_b.native_account.is_none());
        assert_eq!(
            Some(owner_a.clone()),
            store.session_personal_subject(&ids[2]).await.unwrap()
        );
        // Even a legacy raw upstream ID equal to the new typed subject cannot be adopted.
        let signature = crate::SignatureRow {
            account_id: owner_a.clone(),
            name: "work".into(),
            body: "legacy secret".into(),
            html: Some("<b>rich</b>".into()),
            text: Some("plain".into()),
            text_edited: true,
            is_default: true,
            rule_json: String::new(),
            updated_at: String::new(),
        };
        store.upsert_signature(&signature).await.unwrap();
        assert!(
            store
                .list_personal_signatures(&owner_a)
                .await
                .unwrap()
                .is_empty()
        );
        store.upsert_personal_signature(&signature).await.unwrap();
        assert!(
            store
                .list_personal_signatures(&owner_b)
                .await
                .unwrap()
                .is_empty()
        );
        store
            .delete_personal_signature(&owner_b, "work")
            .await
            .unwrap();
        let typed = store.list_personal_signatures(&owner_a).await.unwrap();
        assert_eq!(typed[0].html, signature.html);
        assert_eq!(typed[0].text, signature.text);
        assert!(typed[0].text_edited);
        let mut search = crate::SavedSearchRow {
            id: "personal-test-search".into(),
            user: owner_a.clone(),
            name: "Private".into(),
            query_json: "{}".into(),
            as_folder: true,
        };
        store.upsert_saved_search(&search).await.unwrap();
        assert!(
            store
                .list_personal_saved_searches(&owner_a)
                .await
                .unwrap()
                .is_empty()
        );
        assert!(store.upsert_personal_saved_search(&search).await.unwrap());
        search.user = owner_b.clone();
        search.name = "Hijacked".into();
        assert!(!store.upsert_personal_saved_search(&search).await.unwrap());
        assert!(
            !store
                .delete_personal_saved_search(&owner_b, &search.id)
                .await
                .unwrap()
        );
        assert_eq!(
            store
                .get_personal_saved_search(&search.id)
                .await
                .unwrap()
                .unwrap()
                .name,
            "Private"
        );
        assert!(
            store
                .delete_personal_saved_search(&owner_a, &search.id)
                .await
                .unwrap()
        );
        assert!(
            store.get_saved_search(&search.id).await.unwrap().is_some(),
            "legacy bytes remain intact"
        );
        let a = store
            .session_storage_authority(&ids[0])
            .await
            .unwrap()
            .unwrap();
        assert!(a.starts_with("msa1_") && a.len() == 69);
        assert_ne!(
            Some(a.clone()),
            store.session_storage_authority(&ids[1]).await.unwrap()
        );
        assert_eq!(
            Some(a.clone()),
            store.session_storage_authority(&ids[2]).await.unwrap()
        );
        let other_api = store
            .create_session(
                "1",
                "same@example.test",
                "https://mail.test/a/session",
                "https://mail.test/other-api",
                &crate::Credentials {
                    username: "same".into(),
                    password: "synthetic".into(),
                },
            )
            .await
            .unwrap();
        store
            .set_session_authentication(&other_api, "header-auth")
            .await
            .unwrap();
        store
            .bind_verified_primary(
                &other_api,
                VerifiedPrimary {
                    account_id: "1",
                    kind: "header",
                    authority: "trusted-header",
                    subject: "verified",
                },
                100,
            )
            .await
            .unwrap()
            .unwrap();
        assert_ne!(
            Some(a.clone()),
            store.session_storage_authority(&other_api).await.unwrap()
        );
        q("UPDATE sessions SET api_url=?2 WHERE id=?1")
            .bind(&other_api)
            .bind("https://user:secret@mail.test/api")
            .execute(&store.backend)
            .await
            .unwrap();
        assert!(
            store
                .session_storage_authority(&other_api)
                .await
                .unwrap()
                .is_none(),
            "credential-bearing endpoints cannot establish a storage realm"
        );
        let replacement = store
            .create_session(
                "1",
                "same@example.test",
                "https://mail.test/a/session",
                "https://mail.test/api",
                &crate::Credentials {
                    username: "same".into(),
                    password: "synthetic".into(),
                },
            )
            .await
            .unwrap();
        assert!(
            store
                .rotate_session_security(&ids[0], &replacement)
                .await
                .unwrap()
        );
        assert_eq!(
            Some(a.clone()),
            store.session_storage_authority(&replacement).await.unwrap()
        );
        assert!(
            store
                .session_storage_authority(&ids[0])
                .await
                .unwrap()
                .is_none()
        );
        // Merely colliding with a native account ID does not enter its namespace.
        q("INSERT INTO accounts(id,kind,host,port,tls,username,sealed_creds) VALUES('1','imap','native.test',993,'implicit','same',?1)")
            .bind(vec![0u8]).execute(&store.backend).await.unwrap();
        assert_eq!(
            Some(a),
            store.session_storage_authority(&replacement).await.unwrap()
        );
        let native = session(&store, "1").await;
        store
            .bind_verified_primary(&native, proof("1", "imap:native.test:993"), 100)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            store.native_personal_subject("1").await.unwrap(),
            store.session_personal_subject(&native).await.unwrap()
        );
        assert_eq!(
            store
                .session_policy_subject(&native)
                .await
                .unwrap()
                .unwrap()
                .native_account
                .as_deref(),
            Some("1")
        );
        assert!(
            store
                .session_policy_subject(&replacement)
                .await
                .unwrap()
                .unwrap()
                .native_account
                .is_none()
        );
        assert_ne!(
            store.native_personal_subject("1").await.unwrap(),
            store.session_personal_subject(&replacement).await.unwrap()
        );
        let native_key = store
            .session_storage_authority(&native)
            .await
            .unwrap()
            .unwrap();
        assert_ne!(
            Some(native_key.clone()),
            store.session_storage_authority(&replacement).await.unwrap()
        );
        let sso = store
            .create_session(
                "1",
                "same@example.test",
                "sso",
                "sso",
                &crate::Credentials {
                    username: "same".into(),
                    password: String::new(),
                },
            )
            .await
            .unwrap();
        store.set_session_authentication(&sso, "sso").await.unwrap();
        let principal = store
            .bind_verified_primary(
                &sso,
                VerifiedPrimary {
                    account_id: "1",
                    kind: "sso",
                    authority: "verified-issuer",
                    subject: "verified-subject",
                },
                100,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            Some(native_key),
            store.session_storage_authority(&sso).await.unwrap()
        );
        store
            .retire_principal(&principal.principal_id, 200)
            .await
            .unwrap();
        assert!(
            store
                .session_storage_authority(&sso)
                .await
                .unwrap()
                .is_none()
        );
    }

    async fn session(store: &Store, account: &str) -> String {
        let id = store
            .create_session(
                account,
                account,
                "engine",
                "engine",
                &crate::Credentials {
                    username: account.into(),
                    password: "synthetic".into(),
                },
            )
            .await
            .unwrap();
        store
            .set_session_authentication(&id, "password")
            .await
            .unwrap();
        id
    }
    fn proof<'a>(account: &'a str, authority: &'a str) -> VerifiedPrimary<'a> {
        VerifiedPrimary {
            account_id: account,
            kind: "mailbox",
            authority,
            subject: "verified-subject",
        }
    }
    async fn exercise(store: Store) {
        let realm = uuid::Uuid::new_v4().to_string();
        let account = uuid::Uuid::new_v4().to_string();
        let first = session(&store, &account).await;
        let second = session(&store, &account).await;
        // Legacy/unverified sessions cannot mint a principal.
        store.set_session_authentication(&first, "").await.unwrap();
        assert!(
            store
                .bind_verified_primary(&first, proof(&account, &realm), 100)
                .await
                .unwrap()
                .is_none()
        );
        store
            .set_session_authentication(&first, "password")
            .await
            .unwrap();
        let (a, b) = tokio::join!(
            store.bind_verified_primary(&first, proof(&account, &realm), 100),
            store.bind_verified_primary(&second, proof(&account, &realm), 101)
        );
        let a = a.unwrap().unwrap();
        let b = b.unwrap().unwrap();
        assert_eq!(
            a.principal_id, b.principal_id,
            "concurrent verified sign-ins share one durable subject"
        );
        assert_eq!(
            store
                .bind_verified_primary(&first, proof(&account, &realm), 999)
                .await
                .unwrap()
                .unwrap()
                .auth_time,
            100
        );
        assert!(
            store
                .bind_verified_primary(&first, proof(&account, "other-authority"), 101)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .bind_verified_primary(&first, proof("wrong-account", &realm), 101)
                .await
                .unwrap()
                .is_none()
        );
        let third = session(&store, &account).await;
        let separate = store
            .bind_verified_primary(&third, proof(&account, &format!("other-{realm}")), 101)
            .await
            .unwrap()
            .unwrap();
        assert_ne!(
            a.principal_id, separate.principal_id,
            "same mailbox address is not an identity merge key"
        );
        assert!(
            !store.rotate_session_security(&first, &third).await.unwrap(),
            "rotation cannot overwrite independently bound identity"
        );
        let rotated = session(&store, &account).await;
        assert!(
            store
                .rotate_session_security(&first, &rotated)
                .await
                .unwrap()
        );
        assert_eq!(
            store.primary_principal(&rotated).await.unwrap(),
            Some(a.clone())
        );
        assert!(store.primary_principal(&first).await.unwrap().is_none());
        assert!(store.retire_principal(&a.principal_id, 200).await.unwrap());
        assert!(store.primary_principal(&rotated).await.unwrap().is_none());
        assert!(store.get_session(&second).await.is_err());
        assert!(store.primary_principal(&third).await.unwrap().is_some());
        let replacement = session(&store, &account).await;
        assert!(
            store
                .bind_verified_primary(&replacement, proof(&account, &realm), 201)
                .await
                .unwrap()
                .is_none(),
            "retired subjects cannot be reassigned"
        );
        store.delete_session(&replacement).await.unwrap();
        store
            .retire_principal(&separate.principal_id, 202)
            .await
            .unwrap();
        for _ in 0..24 {
            let authority = uuid::Uuid::new_v4().to_string();
            let old = session(&store, &account).await;
            let new = session(&store, &account).await;
            let principal = store
                .bind_verified_primary(&old, proof(&account, &authority), 300)
                .await
                .unwrap()
                .unwrap();
            let (rotated, retired) = tokio::join!(
                store.rotate_session_security(&old, &new),
                store.retire_principal(&principal.principal_id, 301)
            );
            assert!(retired.unwrap());
            if rotated.unwrap() {
                assert!(
                    store.get_session(&new).await.is_err(),
                    "retirement must revoke a concurrently rotated session"
                );
            } else {
                // The caller discards its unbound replacement on a failed rotation.
                store.delete_session(&new).await.unwrap();
            }
            assert!(store.get_session(&old).await.is_err());
        }
    }
    #[tokio::test]
    async fn principal_authority_sqlite() {
        exercise(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn principal_authority_postgres() {
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
