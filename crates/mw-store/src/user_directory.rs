//! Successful authentication registration; never reactivates held identities.
use crate::{Store, StoreError, q};

pub struct SuccessfulSignin<'a> {
    pub account_id: &'a str,
    pub address: &'a str,
    pub domain: &'a str,
    pub local_part: &'a str,
    pub mode: &'a str,
    pub now: i64,
}
pub struct DirectoryAccount {
    pub account_id: String,
    pub address: String,
    pub last_signin_at: Option<i64>,
    pub status: String,
    pub deleted_at: Option<i64>,
}
impl Store {
    /// Credential-free, bounded listing including earlier successful identities.
    pub async fn directory_accounts(&self) -> Result<Vec<DirectoryAccount>, StoreError> {
        let mut accounts = std::collections::BTreeMap::<String, DirectoryAccount>::new();
        for sql in [
            "SELECT a.account_id,a.address,a.last_seen AS last_signin_at,COALESCE(u.status,'active') AS status,u.deleted_at FROM mail_user_account a LEFT JOIN mail_user u ON u.address=a.address ORDER BY a.account_id LIMIT 5001",
            "SELECT a.account_id,a.address,a.seen_at AS last_signin_at,COALESCE(u.status,'active') AS status,u.deleted_at FROM signin_identity a LEFT JOIN mail_user u ON u.address=a.address ORDER BY a.account_id LIMIT 5001",
            "SELECT a.username AS account_id,a.username AS address,NULL AS last_signin_at,COALESCE(u.status,'active') AS status,u.deleted_at FROM admin_users a LEFT JOIN mail_user u ON u.address=a.username ORDER BY a.username LIMIT 5001",
        ] {
            let rows = q(sql).fetch_all(&self.backend).await?;
            if rows.len() > 5000 {
                return Err(StoreError::Corrupt(
                    "user directory exceeds bounded listing limit".into(),
                ));
            }
            for row in rows {
                let id = row.get_string("account_id");
                let address = row.get_string("address");
                let last = row.get_opt_i64("last_signin_at");
                if let Some(existing) = accounts.get_mut(&id) {
                    if existing.address == address {
                        existing.last_signin_at = existing.last_signin_at.max(last);
                    }
                } else {
                    accounts.insert(
                        id.clone(),
                        DirectoryAccount {
                            account_id: id,
                            address,
                            last_signin_at: last,
                            status: row.get_string("status"),
                            deleted_at: row.get_opt_i64("deleted_at"),
                        },
                    );
                }
            }
            if accounts.len() > 5000 {
                return Err(StoreError::Corrupt(
                    "user directory exceeds bounded listing limit".into(),
                ));
            }
        }
        let mut rows = accounts.into_values().collect::<Vec<_>>();
        rows.sort_by(|a, b| {
            a.address
                .cmp(&b.address)
                .then(a.account_id.cmp(&b.account_id))
        });
        Ok(rows)
    }

    /// Atomically register a successful identity. False means a hold or conflicting
    /// account binding won; callers must not issue a session in that case.
    pub async fn register_successful_signin(
        &self,
        s: SuccessfulSignin<'_>,
    ) -> Result<bool, StoreError> {
        let policy_address = s.address;
        self.register_successful_signin_with_policy(s, policy_address)
            .await
    }

    /// Preserve the authenticated spelling for new directory identities while
    /// checking the parent policy identity. Historical parent bindings are kept
    /// unchanged: authentication is not authority to merge directory records.
    pub async fn register_successful_signin_with_policy(
        &self,
        s: SuccessfulSignin<'_>,
        policy_address: &str,
    ) -> Result<bool, StoreError> {
        if s.account_id.is_empty()
            || s.address.is_empty()
            || !["engine", "proxy", "header", "sso", "child"].contains(&s.mode)
        {
            return Err(StoreError::Corrupt(
                "invalid successful sign-in identity".into(),
            ));
        }
        let mut tx = self.backend.begin().await?;
        // Legacy provisioned identities may use mixed case, including Unicode.
        // Do not create a folded duplicate that bypasses an existing local hold.
        let held = q("SELECT address FROM mail_user WHERE status IN ('disabled','deleting','deleted') ORDER BY address LIMIT 5001")
            .fetch_all_tx(&mut tx).await?;
        if held.len() > 5000 {
            return Err(StoreError::Corrupt(
                "directory access holds exceed bounded lookup limit".into(),
            ));
        }
        let identities = [
            crate::user_lifecycle::access_identity(policy_address)?,
            crate::user_lifecycle::access_identity(s.address)?,
        ];
        for row in held {
            if identities.contains(&crate::user_lifecycle::access_identity(
                &row.get_string("address"),
            )?) {
                return Ok(false);
            }
        }
        // Lock an existing parent row and preserve its hold/expiry restrictions.
        for checked_address in [policy_address, s.address] {
            let active = q("UPDATE mail_user SET last_signin_at=last_signin_at WHERE address=?1 AND status='active' AND (expires_at IS NULL OR expires_at>?2)")
                .bind(checked_address).bind(s.now).execute_tx(&mut tx).await?;
            if active == 0
                && q("SELECT address FROM mail_user WHERE address=?1")
                    .bind(checked_address)
                    .fetch_opt_scalar_string_tx(&mut tx)
                    .await?
                    .is_some()
            {
                return Ok(false);
            }
        }
        let prior = q("SELECT address FROM mail_user_account WHERE account_id=?1")
            .bind(s.account_id)
            .fetch_opt_scalar_string_tx(&mut tx)
            .await?;
        let retained = prior
            .as_deref()
            .filter(|prior| *prior == policy_address && *prior != s.address);
        let address = retained.unwrap_or(s.address);
        let (local_part, domain) = address.rsplit_once('@').unwrap_or((address, ""));
        let s = SuccessfulSignin {
            address,
            local_part,
            domain,
            ..s
        };
        q("INSERT INTO mail_user(address,domain,local_part,origin,status,created_at,first_signin_at,last_signin_at) VALUES(?1,?2,?3,?4,'active',?5,?5,?5) ON CONFLICT(address) DO NOTHING")
            .bind(s.address).bind(s.domain).bind(s.local_part).bind(if s.mode=="sso" {"sso"} else {"signin"}).bind(s.now).execute_tx(&mut tx).await?;
        let n=q("UPDATE mail_user SET first_signin_at=COALESCE(first_signin_at,?2),last_signin_at=?2 WHERE address=?1 AND status='active' AND (expires_at IS NULL OR expires_at>?2)")
            .bind(s.address).bind(s.now).execute_tx(&mut tx).await?;
        if n != 1 {
            return Ok(false);
        }
        let n=q("INSERT INTO mail_user_account(account_id,address,mode,first_seen,last_seen) VALUES(?1,?2,?3,?4,?4) ON CONFLICT(account_id) DO UPDATE SET last_seen=excluded.last_seen,mode=excluded.mode WHERE mail_user_account.address=excluded.address")
            .bind(s.account_id).bind(s.address).bind(s.mode).bind(s.now).execute_tx(&mut tx).await?;
        if n != 1 {
            return Ok(false);
        }
        tx.commit().await?;
        Ok(true)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn signin<'a>(id: &'a str, address: &'a str, now: i64) -> SuccessfulSignin<'a> {
        SuccessfulSignin {
            account_id: id,
            address,
            domain: "example.test",
            local_part: "alice",
            mode: "engine",
            now,
        }
    }
    #[tokio::test]
    async fn successful_registration_is_idempotent_and_never_rebinds_or_reactivates() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        exercise(store).await;
    }
    #[tokio::test]
    async fn postgres_successful_registration_and_safe_listing() {
        let Ok(dsn) = std::env::var("MW_DIRECTORY_TEST_PG_DSN") else {
            eprintln!("SKIP: MW_DIRECTORY_TEST_PG_DSN unset");
            return;
        };
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&dsn)
            .await
            .unwrap();
        let schema = format!("user_directory_{}", rand::random::<u64>());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&pool)
            .await
            .unwrap();
        let sep = if dsn.contains('?') { '&' } else { '?' };
        let scoped = format!("{dsn}{sep}options=-c%20search_path%3D{schema}");
        let result = tokio::spawn(async move {
            exercise(
                Store::open_postgres(&scoped, crate::ServerKey::generate())
                    .await
                    .unwrap(),
            )
            .await;
        })
        .await;
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&pool)
            .await
            .unwrap();
        result.unwrap();
    }
    async fn exercise_aliases(store: &Store) {
        let alias = "alias-user@alias.test";
        let parent = "alias-user@parent.test";
        assert!(
            store
                .register_successful_signin_with_policy(signin("alias-new", alias, 1), parent)
                .await
                .unwrap()
        );
        let rows = store.directory_accounts().await.unwrap();
        assert_eq!(
            rows.iter()
                .find(|row| row.account_id == "alias-new")
                .unwrap()
                .address,
            alias
        );
        assert!(
            store
                .register_successful_signin_with_policy(signin("alias-new", alias, 2), parent)
                .await
                .unwrap()
        );
        // Previously collapsed records are neither rebound nor duplicated.
        assert!(
            store
                .register_successful_signin(signin("alias-old", parent, 1))
                .await
                .unwrap()
        );
        assert!(
            store
                .register_successful_signin_with_policy(signin("alias-old", alias, 3), parent)
                .await
                .unwrap()
        );
        let rows = store.directory_accounts().await.unwrap();
        assert_eq!(
            rows.iter()
                .find(|row| row.account_id == "alias-old")
                .unwrap()
                .address,
            parent
        );
        for held in [parent, alias] {
            q("UPDATE mail_user SET status='disabled' WHERE address=?1")
                .bind(held)
                .execute(&store.backend)
                .await
                .unwrap();
            assert!(
                !store
                    .register_successful_signin_with_policy(signin("alias-new", alias, 4), parent)
                    .await
                    .unwrap()
            );
            assert!(
                !store
                    .register_successful_signin_with_policy(signin("alias-old", alias, 4), parent)
                    .await
                    .unwrap()
            );
            q("UPDATE mail_user SET status='active',expires_at=4 WHERE address=?1")
                .bind(held)
                .execute(&store.backend)
                .await
                .unwrap();
            assert!(
                !store
                    .register_successful_signin_with_policy(signin("alias-new", alias, 4), parent)
                    .await
                    .unwrap()
            );
            q("UPDATE mail_user SET expires_at=NULL WHERE address=?1")
                .bind(held)
                .execute(&store.backend)
                .await
                .unwrap();
        }
        assert!(
            !store
                .register_successful_signin_with_policy(
                    signin("alias-old", "unrelated@other.test", 5),
                    "unrelated@other.test"
                )
                .await
                .unwrap()
        );
        q("DELETE FROM mail_user_account WHERE account_id IN ('alias-new','alias-old')")
            .execute(&store.backend)
            .await
            .unwrap();
        q("DELETE FROM mail_user WHERE address IN (?1,?2)")
            .bind(alias)
            .bind(parent)
            .execute(&store.backend)
            .await
            .unwrap();
    }
    async fn exercise(store: Store) {
        exercise_aliases(&store).await;
        assert!(
            store
                .register_successful_signin(signin("opaque", "alice@example.test", 10))
                .await
                .unwrap()
        );
        assert!(
            store
                .register_successful_signin(signin("opaque", "alice@example.test", 20))
                .await
                .unwrap()
        );
        assert!(
            !store
                .register_successful_signin(signin("opaque", "other@example.test", 30))
                .await
                .unwrap()
        );
        assert!(
            q("SELECT address FROM mail_user WHERE address=?1")
                .bind("other@example.test")
                .fetch_optional(&store.backend)
                .await
                .unwrap()
                .is_none()
        );
        let listed = store.directory_accounts().await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].account_id, "opaque");
        assert_eq!(listed[0].address, "alice@example.test");
        assert_eq!(listed[0].last_signin_at, Some(20));
        assert_eq!(
            store
                .audit_identity("user", "opaque")
                .await
                .unwrap()
                .unwrap()
                .label,
            "alice@example.test"
        );
        for status in ["disabled", "deleting", "deleted"] {
            q("UPDATE mail_user SET status=?1 WHERE address=?2")
                .bind(status)
                .bind("alice@example.test")
                .execute(&store.backend)
                .await
                .unwrap();
            assert!(
                !store
                    .register_successful_signin(signin("opaque", "alice@example.test", 40))
                    .await
                    .unwrap()
            );
        }
        q("UPDATE mail_user SET status='active',expires_at=50 WHERE address=?1")
            .bind("alice@example.test")
            .execute(&store.backend)
            .await
            .unwrap();
        assert!(
            !store
                .register_successful_signin(signin("opaque", "alice@example.test", 50))
                .await
                .unwrap()
        );
    }
}
