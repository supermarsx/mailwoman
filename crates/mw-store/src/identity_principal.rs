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

fn row(value: &Row) -> PrimaryPrincipal {
    PrimaryPrincipal {
        principal_id: value.get_string("principal_id"),
        binding_id: value.get_string("binding_id"),
        primary_account_id: value.get_string("primary_account_id"),
        auth_time: value.get_i64("auth_time"),
        authentication_method: value.get_string("authentication_method"),
    }
}

impl Store {
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
        tx.commit().await?;
        Ok(changed == 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
