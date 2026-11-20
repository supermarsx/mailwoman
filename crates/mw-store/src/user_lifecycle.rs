//! Local access holds retain directory/account data and never erase upstream mail.
use crate::{Store, StoreError, q};
use serde::Serialize;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserLifecycle {
    pub directory_address: String,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub changed_at: Option<i64>,
    pub revision: String,
}
#[derive(Debug)]
pub enum LifecycleChange {
    Applied {
        state: UserLifecycle,
        sessions: Vec<String>,
    },
    Stale,
    Deleted,
    NotFound,
}
pub(crate) fn access_identity(address: &str) -> Result<String, StoreError> {
    let folded = Store::fold_account_name(address);
    match folded.rsplit_once('@') {
        Some((local, domain)) => Ok(format!(
            "{local}@{}",
            crate::settings_policy::normalize_policy_domain(domain)?
        )),
        None => Ok(folded),
    }
}
/// Pure normalization shared by the fresh final gate; no cached hold decision.
pub fn final_gate_access_identity(address: &str) -> Result<String, StoreError> {
    access_identity(address)
}
fn key(address: &str) -> String {
    format!("mail-user-lifecycle:{address}")
}
async fn matches_address(
    name: &str,
    address: &str,
    tx: &mut crate::backend::Tx,
) -> Result<bool, StoreError> {
    let name = access_identity(name)?;
    let address = access_identity(address)?;
    if name == address {
        return Ok(true);
    }
    let Some((local, source)) = name.rsplit_once('@') else {
        return Ok(false);
    };
    let Some((target_local, _)) = address.rsplit_once('@') else {
        return Ok(false);
    };
    if local != target_local {
        return Ok(false);
    }
    let mut domain = crate::settings_policy::normalize_policy_domain(source)?;
    let mut seen = Vec::new();
    loop {
        if seen.len() >= 16 || seen.contains(&domain) {
            return Err(StoreError::Corrupt("cyclic directory alias".into()));
        }
        seen.push(domain.clone());
        if format!("{local}@{domain}") == address {
            return Ok(true);
        }
        let Some(parent) = q("SELECT domain FROM domain_alias WHERE alias=?1")
            .bind(&domain)
            .fetch_opt_scalar_string_tx(tx)
            .await?
        else {
            return Ok(false);
        };
        domain = crate::settings_policy::normalize_policy_domain(&parent)?;
    }
}
impl Store {
    pub async fn user_lifecycle(&self, address: &str) -> Result<Option<UserLifecycle>, StoreError> {
        if let Some(row) =
            q("SELECT address,status,deleted_at,status_changed_at FROM mail_user WHERE address=?1")
                .bind(address)
                .fetch_optional(&self.backend)
                .await?
        {
            return Ok(Some(UserLifecycle {
                directory_address: row.get_string("address"),
                status: row.get_string("status"),
                deleted_at: row.get_opt_i64("deleted_at"),
                changed_at: row.get_opt_i64("status_changed_at"),
                revision: self.get_setting(&key(address)).await?.unwrap_or_default(),
            }));
        }
        Ok(q("SELECT username FROM admin_users WHERE username=?1")
            .bind(address)
            .fetch_optional(&self.backend)
            .await?
            .map(|_| UserLifecycle {
                directory_address: address.into(),
                status: "active".into(),
                deleted_at: None,
                changed_at: None,
                revision: String::new(),
            }))
    }
    pub async fn directory_access_held(&self, address: &str) -> Result<bool, StoreError> {
        self.directory_access_held_any(&[address]).await
    }

    /// One fresh bounded scan for all names of an already-resolved principal.
    /// No held rows, results or authentication decisions survive this call.
    pub async fn directory_access_held_any(&self, addresses: &[&str]) -> Result<bool, StoreError> {
        let folded = addresses
            .iter()
            .map(|address| access_identity(address))
            .collect::<Result<std::collections::HashSet<_>, _>>()?;
        if folded.is_empty() {
            return Ok(false);
        }
        // SQLite LOWER is ASCII-only. Match the same Unicode folding as login
        // flags, and retain the original status-indexed legacy scan and bound.
        let rows = q("SELECT address FROM mail_user WHERE status IN ('disabled','deleting','deleted') ORDER BY address LIMIT 5001")
            .fetch_all(&self.backend).await?;
        if rows.len() > 5000 {
            return Err(StoreError::Corrupt(
                "directory access holds exceed bounded lookup limit".into(),
            ));
        }
        for row in rows {
            if folded.contains(&access_identity(&row.get_string("address"))?) {
                return Ok(true);
            }
        }
        Ok(false)
    }
    pub async fn lifecycle_account_bindings(
        &self,
        address: &str,
    ) -> Result<Vec<String>, StoreError> {
        let rows =
            q("SELECT account_id FROM mail_user_account WHERE address=?1 OR address=?2 LIMIT 1001")
                .bind(address)
                .bind(access_identity(address)?)
                .fetch_all(&self.backend)
                .await?;
        if rows.len() > 1000 {
            return Err(StoreError::Corrupt("too many directory bindings".into()));
        }
        Ok(rows
            .into_iter()
            .map(|row| row.get_string("account_id"))
            .collect())
    }
    /// The row write lock serializes this operation with successful sign-in registration.
    /// Revocation uses proven login names, not an upstream account ID alone.
    pub async fn change_user_lifecycle(
        &self,
        expected: &UserLifecycle,
        status: &str,
        actor: &str,
        now: i64,
    ) -> Result<LifecycleChange, StoreError> {
        if !["active", "disabled", "deleted"].contains(&status) {
            return Err(StoreError::Corrupt("invalid directory lifecycle".into()));
        }
        let address = &expected.directory_address;
        let mut tx = self.backend.begin().await?;
        if q("UPDATE mail_user SET status=status WHERE address=?1")
            .bind(address)
            .execute_tx(&mut tx)
            .await?
            != 1
        {
            if q("SELECT username FROM admin_users WHERE username=?1")
                .bind(address)
                .fetch_opt_scalar_string_tx(&mut tx)
                .await?
                .is_none()
            {
                return Ok(LifecycleChange::NotFound);
            }
            let (local, domain) = address.rsplit_once('@').unwrap_or((address, ""));
            q("INSERT INTO mail_user(address,domain,local_part,origin,status,created_at) VALUES(?1,?2,?3,'provisioned','active',?4) ON CONFLICT(address) DO NOTHING").bind(address).bind(domain).bind(local).bind(now).execute_tx(&mut tx).await?;
            q("UPDATE mail_user SET status=status WHERE address=?1")
                .bind(address)
                .execute_tx(&mut tx)
                .await?;
        }
        let row = q("SELECT status,deleted_at,status_changed_at FROM mail_user WHERE address=?1")
            .bind(address)
            .fetch_all_tx(&mut tx)
            .await?
            .pop()
            .ok_or(StoreError::NotFound)?;
        let revision = q("SELECT value FROM settings WHERE key=?1")
            .bind(key(address))
            .fetch_opt_scalar_string_tx(&mut tx)
            .await?
            .unwrap_or_default();
        if row.get_string("status") != expected.status
            || row.get_opt_i64("deleted_at") != expected.deleted_at
            || row.get_opt_i64("status_changed_at") != expected.changed_at
            || revision != expected.revision
        {
            return Ok(LifecycleChange::Stale);
        }
        if expected.status == "deleted" {
            return Ok(LifecycleChange::Deleted);
        }
        let mut sessions = Vec::new();
        if status != "active" {
            let rows=q("SELECT id,username,sealed_creds FROM sessions WHERE lower(username)=?2 OR account_id IN (SELECT account_id FROM mail_user_account WHERE address=?1 OR address=?2) LIMIT 5001").bind(address).bind(access_identity(address)?).fetch_all_tx(&mut tx).await?;
            if rows.len() > 5000 {
                return Err(StoreError::Corrupt(
                    "too many sessions for bounded lifecycle operation".into(),
                ));
            }
            for row in rows {
                let reported = row.get_string("username");
                let login = self
                    .open_session_creds(&row.get_blob("sealed_creds"))
                    .map(|credentials| credentials.username);
                let matches_reported = matches_address(&reported, address, &mut tx).await?;
                let matches_login = if let Some(login) = login.as_deref() {
                    matches_address(login, address, &mut tx).await?
                } else {
                    false
                };
                if matches_reported || matches_login {
                    let id = row.get_string("id");
                    q("DELETE FROM sessions WHERE id=?1")
                        .bind(&id)
                        .execute_tx(&mut tx)
                        .await?;
                    sessions.push(id);
                }
            }
        }
        let deleted_at = (status == "deleted").then_some(now);
        let revision = uuid::Uuid::new_v4().to_string();
        q("UPDATE mail_user SET status=?2,status_changed_at=?3,status_changed_by=?4,deleted_at=?5 WHERE address=?1").bind(address).bind(status).bind(now).bind(actor).bind(deleted_at).execute_tx(&mut tx).await?;
        q("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value").bind(key(address)).bind(&revision).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(LifecycleChange::Applied {
            state: UserLifecycle {
                directory_address: address.clone(),
                status: status.into(),
                deleted_at,
                changed_at: Some(now),
                revision,
            },
            sessions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn batched_directory_holds_preserve_unicode_errors_and_one_fresh_scan() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        q("INSERT INTO mail_user(address,domain,local_part,origin,status,created_at) VALUES('NÉW@BÜCHER.Test','xn--bcher-kva.test','néw','import','disabled',1)")
            .execute(&store.backend).await.unwrap();
        let (held, counts) =
            crate::pool_telemetry::measure_queries(store.directory_access_held_any(&[
                "clear@example.test",
                "néw@xn--bcher-kva.test",
                "NÉW@BÜCHER.TEST",
            ]))
            .await;
        assert!(held.unwrap());
        assert_eq!(counts.read.calls, 1);
        assert!(
            store
                .directory_access_held("néw@xn--bcher-kva.test")
                .await
                .unwrap()
        );
        assert!(
            !store
                .directory_access_held_any(&["clear@example.test", "other@example.test"])
                .await
                .unwrap()
        );
        q("INSERT INTO mail_user(address,domain,local_part,origin,status,created_at) VALUES('broken@bad/path','example.test','broken','import','deleted',1)")
            .execute(&store.backend).await.unwrap();
        assert!(
            store
                .directory_access_held_any(&["clear@example.test"])
                .await
                .is_err()
        );
        // Finding a valid hold before a corrupt later row may return true, but
        // a corrupt legacy row must never turn a refusal into admission.
        assert!(!matches!(
            store
                .directory_access_held_any(&["NÉW@BÜCHER.TEST", "clear@example.test"])
                .await,
            Ok(false)
        ));
        assert!(
            store
                .directory_access_held_any(&["user@bad/path"])
                .await
                .is_err()
        );
        let (empty, counts) =
            crate::pool_telemetry::measure_queries(store.directory_access_held_any(&[])).await;
        assert!(!empty.unwrap());
        assert_eq!(counts.read.calls, 0);
        store.close().await;
        assert!(
            store
                .directory_access_held_any(&["clear@example.test"])
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn batched_directory_holds_keep_the_5001_fail_closed_bound() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        q("WITH RECURSIVE n(value) AS (SELECT 1 UNION ALL SELECT value+1 FROM n WHERE value<5000) INSERT INTO mail_user(address,domain,local_part,origin,status,created_at) SELECT 'held' || CAST(value AS TEXT) || '@example.test','example.test','held','import','disabled',1 FROM n")
            .execute(&store.backend).await.unwrap();
        assert!(
            !store
                .directory_access_held_any(&["clear@example.test"])
                .await
                .unwrap()
        );
        q("INSERT INTO mail_user(address,domain,local_part,origin,status,created_at) VALUES('extra@example.test','example.test','extra','import','disabled',1)")
            .execute(&store.backend).await.unwrap();
        assert!(
            store
                .directory_access_held_any(&["clear@example.test"])
                .await
                .is_err()
        );
        assert!(
            store
                .directory_access_held_any(&["held1@example.test"])
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn independent_directory_writer_revokes_without_invalidation() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("holds.db");
        let key = crate::ServerKey::generate();
        let reader = Store::open(path.to_str().unwrap(), key.clone())
            .await
            .unwrap();
        let writer = Store::open(path.to_str().unwrap(), key).await.unwrap();
        assert!(
            !reader
                .directory_access_held_any(&["user@example.test"])
                .await
                .unwrap()
        );
        q("INSERT INTO mail_user(address,domain,local_part,origin,status,created_at) VALUES('user@example.test','example.test','user','import','disabled',1)")
            .execute(&writer.backend).await.unwrap();
        assert!(
            reader
                .directory_access_held_any(&["USER@EXAMPLE.TEST"])
                .await
                .unwrap()
        );
        q("UPDATE mail_user SET status='active' WHERE address='user@example.test'")
            .execute(&writer.backend)
            .await
            .unwrap();
        assert!(
            !reader
                .directory_access_held_any(&["USER@EXAMPLE.TEST"])
                .await
                .unwrap()
        );
        reader.close().await;
        writer.close().await;
        drop(reader);
        drop(writer);
    }

    #[tokio::test]
    async fn lifecycle_cas_retains_data_revokes_proven_sessions_and_never_reenrolls() {
        exercise(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn postgres_lifecycle_cas_retains_data_and_refuses_reenrollment() {
        let Ok(dsn) = std::env::var("MW_LIFECYCLE_TEST_PG") else {
            eprintln!("SKIP: MW_LIFECYCLE_TEST_PG unset");
            return;
        };
        exercise(
            Store::open_postgres(&dsn, crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    async fn exercise(store: Store) {
        let address = "alice@example.test";
        let signup = || crate::user_directory::SuccessfulSignin {
            account_id: "shared-upstream-id",
            address,
            domain: "example.test",
            local_part: "alice",
            mode: "proxy",
            now: 1,
        };
        assert!(store.register_successful_signin(signup()).await.unwrap());
        let owned = store
            .create_session(
                "shared-upstream-id",
                address,
                "https://one.test/session",
                "https://one.test/jmap",
                &crate::Credentials {
                    username: address.into(),
                    password: "secret".into(),
                },
            )
            .await
            .unwrap();
        let other = store
            .create_session(
                "shared-upstream-id",
                "bob@other.test",
                "https://two.test/session",
                "https://two.test/jmap",
                &crate::Credentials {
                    username: "bob@other.test".into(),
                    password: "other".into(),
                },
            )
            .await
            .unwrap();
        q("INSERT INTO domains(name,upstream_json) VALUES('example.test','{}')")
            .execute(&store.backend)
            .await
            .unwrap();
        store
            .put_domain_alias("alias.test", "example.test")
            .await
            .unwrap();
        let alias = store
            .create_session(
                "shared-upstream-id",
                "alice@alias.test",
                "https://one.test/session",
                "https://one.test/jmap",
                &crate::Credentials {
                    username: "alice@alias.test".into(),
                    password: "secret".into(),
                },
            )
            .await
            .unwrap();
        let initial = store.user_lifecycle(address).await.unwrap().unwrap();
        let LifecycleChange::Applied {
            state: blocked,
            sessions,
        } = store
            .change_user_lifecycle(&initial, "disabled", "operator", 2)
            .await
            .unwrap()
        else {
            panic!("block applied")
        };
        assert_eq!(sessions.len(), 2);
        assert!(sessions.contains(&owned));
        assert!(sessions.contains(&alias));
        assert!(matches!(
            store.get_session(&alias).await,
            Err(StoreError::NotFound)
        ));
        assert!(matches!(
            store.get_session(&owned).await,
            Err(StoreError::NotFound)
        ));
        assert!(store.get_session(&other).await.is_ok());
        assert!(!store.register_successful_signin(signup()).await.unwrap());
        assert!(matches!(
            store
                .change_user_lifecycle(&initial, "deleted", "operator", 2)
                .await
                .unwrap(),
            LifecycleChange::Stale
        ));
        let LifecycleChange::Applied { state: active, .. } = store
            .change_user_lifecycle(&blocked, "active", "operator", 2)
            .await
            .unwrap()
        else {
            panic!("unblock applied")
        };
        assert_ne!(initial.revision, active.revision);
        let LifecycleChange::Applied { state: deleted, .. } = store
            .change_user_lifecycle(&active, "deleted", "operator", 3)
            .await
            .unwrap()
        else {
            panic!("delete applied")
        };
        assert_eq!(deleted.deleted_at, Some(3));
        assert!(!store.register_successful_signin(signup()).await.unwrap());
        assert!(matches!(
            store
                .change_user_lifecycle(&deleted, "active", "operator", 4)
                .await
                .unwrap(),
            LifecycleChange::Deleted
        ));
        assert_eq!(
            q("SELECT address FROM mail_user_account WHERE account_id='shared-upstream-id'")
                .fetch_opt_scalar_string(&store.backend)
                .await
                .unwrap()
                .as_deref(),
            Some(address)
        );
        assert_eq!(
            q("SELECT created_at FROM mail_user WHERE address=?1")
                .bind(address)
                .fetch_optional(&store.backend)
                .await
                .unwrap()
                .unwrap()
                .get_i64("created_at"),
            1
        );
        exercise_provisioned(store).await;
    }
    #[tokio::test]
    async fn provisioned_without_signin_gets_a_nonrevivable_local_access_record() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        exercise_provisioned(store).await;
    }
    async fn exercise_provisioned(store: Store) {
        q("INSERT INTO admin_users(id,username,password_hash,created_at) VALUES('provisioned-id','NÉW@BÜCHER.Test','test','2026-01-01')").execute(&store.backend).await.unwrap();
        assert!(
            store
                .register_successful_signin(crate::user_directory::SuccessfulSignin {
                    account_id: "existing-folded-binding",
                    address: "néw@xn--bcher-kva.test",
                    domain: "xn--bcher-kva.test",
                    local_part: "néw",
                    mode: "proxy",
                    now: 1,
                })
                .await
                .unwrap()
        );
        let session = store
            .create_session(
                "existing-folded-binding",
                "néw@xn--bcher-kva.test",
                "https://one.test/session",
                "https://one.test/jmap",
                &crate::Credentials {
                    username: "néw@xn--bcher-kva.test".into(),
                    password: "secret".into(),
                },
            )
            .await
            .unwrap();
        let original = store
            .user_lifecycle("NÉW@BÜCHER.Test")
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(
            store
                .change_user_lifecycle(&original, "deleted", "operator", 10)
                .await
                .unwrap(),
            LifecycleChange::Applied { .. }
        ));
        assert!(matches!(
            store.get_session(&session).await,
            Err(StoreError::NotFound)
        ));
        assert_eq!(
            store
                .user_lifecycle("NÉW@BÜCHER.Test")
                .await
                .unwrap()
                .unwrap()
                .status,
            "deleted"
        );
        assert!(
            store
                .directory_access_held("néw@xn--bcher-kva.test")
                .await
                .unwrap()
        );
        assert!(
            !store
                .register_successful_signin(crate::user_directory::SuccessfulSignin {
                    account_id: "new-lowercase-binding",
                    address: "néw@xn--bcher-kva.test",
                    domain: "xn--bcher-kva.test",
                    local_part: "néw",
                    mode: "proxy",
                    now: 11,
                })
                .await
                .unwrap()
        );
        assert!(
            store
                .get_admin_user("NÉW@BÜCHER.Test")
                .await
                .unwrap()
                .is_some()
        );
    }
}
