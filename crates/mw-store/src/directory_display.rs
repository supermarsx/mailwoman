//! Read-only native login evidence for directory presentation; never rebinds users.
use crate::{Store, StoreError, q};
impl Store {
    /// A matching raw account ID is insufficient: require the recorded engine
    /// binding and an active verified principal for this exact native authority.
    /// The caller still checks the current alias configuration before presenting it.
    pub async fn verified_native_directory_login(
        &self,
        account: &str,
        directory_address: &str,
    ) -> Result<Option<String>, StoreError> {
        q("SELECT a.username FROM mail_user_account u JOIN accounts a ON a.id=u.account_id WHERE u.account_id=?1 AND u.address=?2 AND u.mode='engine' AND a.kind IN ('imap','pop3') AND EXISTS (SELECT 1 FROM identity_binding b JOIN identity_principal p ON p.id=b.principal_id WHERE b.kind='mailbox' AND b.subject=a.id AND b.authority=(a.kind || ':' || lower(a.host) || ':' || CAST(a.port AS TEXT)) AND b.retired_at IS NULL AND p.state='active')")
            .bind(account).bind(directory_address).fetch_opt_scalar_string(&self.backend).await.map_err(Into::into)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn native_display_needs_verified_exact_authority_and_engine_directory_binding() {
        exercise(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn native_display_postgres_authority_fences() {
        let Ok(dsn) = std::env::var("MW_DIRECTORY_TEST_PG_DSN") else {
            eprintln!("SKIP: MW_DIRECTORY_TEST_PG_DSN unset");
            return;
        };
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&dsn)
            .await
            .unwrap();
        let schema = format!("directory_display_{}", rand::random::<u64>());
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
    async fn exercise(store: Store) {
        q("INSERT INTO accounts(id,kind,host,port,tls,username,sealed_creds) VALUES('native','imap','mail.parent.test',993,'implicit','alice@alias.test',?1)").bind(vec![0_u8]).execute(&store.backend).await.unwrap();
        assert!(
            store
                .register_successful_signin(crate::user_directory::SuccessfulSignin {
                    account_id: "native",
                    address: "alice@parent.test",
                    domain: "parent.test",
                    local_part: "alice",
                    mode: "engine",
                    now: 1
                })
                .await
                .unwrap()
        );
        assert!(
            store
                .verified_native_directory_login("native", "alice@parent.test")
                .await
                .unwrap()
                .is_none()
        );
        let session = store
            .create_session(
                "native",
                "alice@alias.test",
                "engine",
                "engine",
                &crate::Credentials {
                    username: "alice@alias.test".into(),
                    password: "test".into(),
                },
            )
            .await
            .unwrap();
        store
            .set_session_authentication(&session, "password")
            .await
            .unwrap();
        let principal = store
            .bind_verified_primary(
                &session,
                crate::identity_principal::VerifiedPrimary {
                    account_id: "native",
                    kind: "mailbox",
                    authority: "imap:mail.parent.test:993",
                    subject: "native",
                },
                2,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            store
                .verified_native_directory_login("native", "alice@parent.test")
                .await
                .unwrap()
                .as_deref(),
            Some("alice@alias.test")
        );
        assert!(
            store
                .verified_native_directory_login("native", "other@parent.test")
                .await
                .unwrap()
                .is_none()
        );
        q("UPDATE mail_user_account SET mode='proxy' WHERE account_id='native'")
            .execute(&store.backend)
            .await
            .unwrap();
        assert!(
            store
                .verified_native_directory_login("native", "alice@parent.test")
                .await
                .unwrap()
                .is_none()
        );
        q("UPDATE mail_user_account SET mode='engine' WHERE account_id='native'")
            .execute(&store.backend)
            .await
            .unwrap();
        q("UPDATE accounts SET host='different.test' WHERE id='native'")
            .execute(&store.backend)
            .await
            .unwrap();
        assert!(
            store
                .verified_native_directory_login("native", "alice@parent.test")
                .await
                .unwrap()
                .is_none()
        );
        q("UPDATE accounts SET host='mail.parent.test' WHERE id='native'")
            .execute(&store.backend)
            .await
            .unwrap();
        store
            .retire_principal(&principal.principal_id, 3)
            .await
            .unwrap();
        assert!(
            store
                .verified_native_directory_login("native", "alice@parent.test")
                .await
                .unwrap()
                .is_none()
        );
    }
}
