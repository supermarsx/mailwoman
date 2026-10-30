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

    /// Everything the admin Users table shows besides the directory row itself
    /// (t107), read as a fixed set of whole-table queries: the cost does not grow
    /// with a lookup per user. Each correlated subquery hits a primary key or an
    /// index (`totp_secrets`, `quotas` PK; `idx_webauthn_credentials_account`);
    /// the linked-mailbox counts are one grouped pass. Bounded like
    /// [`Self::directory_accounts`].
    pub async fn user_table_facts(&self) -> Result<UserTableFacts, StoreError> {
        let mut facts = UserTableFacts::default();
        let rows = q("SELECT ids.account_id AS account_id,\
             (SELECT COUNT(*) FROM totp_secrets t WHERE t.account_id=ids.account_id AND t.confirmed=1) AS totp,\
             (SELECT COUNT(*) FROM webauthn_credentials w WHERE w.account_id=ids.account_id) AS passkeys,\
             (SELECT qa.bytes_limit FROM quotas qa WHERE qa.account_id=ids.account_id) AS bytes_limit,\
             (SELECT qa.msg_limit FROM quotas qa WHERE qa.account_id=ids.account_id) AS msg_limit,\
             (SELECT m.first_seen FROM mail_user_account m WHERE m.account_id=ids.account_id) AS first_seen,             COALESCE(lc.total,0) AS linked \
             ,(SELECT a.last_seen_at FROM user_activity a WHERE a.account_id=ids.account_id) AS last_activity \
             FROM (SELECT account_id FROM mail_user_account UNION SELECT account_id FROM signin_identity UNION SELECT username AS account_id FROM admin_users) ids \
             LEFT JOIN (SELECT b.subject AS subject,COUNT(*) AS total FROM identity_binding b JOIN linked_account l ON l.owner_principal_id=b.principal_id WHERE b.kind='mailbox' AND b.retired_at IS NULL GROUP BY b.subject) lc ON lc.subject=ids.account_id \
             ORDER BY ids.account_id LIMIT 15001")
            .fetch_all(&self.backend)
            .await?;
        if rows.len() > 15000 {
            return Err(StoreError::Corrupt(
                "user directory exceeds bounded listing limit".into(),
            ));
        }
        for row in rows {
            let bytes = row.get_opt_i64("bytes_limit");
            let msgs = row.get_opt_i64("msg_limit");
            facts.accounts.insert(
                row.get_string("account_id"),
                AccountFacts {
                    totp: row.get_i64("totp") > 0,
                    passkeys: row.get_i64("passkeys"),
                    quota: bytes.zip(msgs),
                    linked: row.get_i64("linked"),
                    first_seen: row.get_opt_i64("first_seen"),
                    last_seen_at: row
                        .get_opt_string("last_activity")
                        .and_then(|at| chrono::DateTime::parse_from_rfc3339(&at).ok())
                        .map(|at| at.timestamp()),
                },
            );
        }
        for row in q("SELECT address,display_name,created_at,origin FROM mail_user ORDER BY address LIMIT 15001")
            .fetch_all(&self.backend)
            .await?
        {
            facts.people.insert(
                row.get_string("address"),
                PersonFacts {
                    display_name: row
                        .get_opt_string("display_name")
                        .filter(|n| !n.trim().is_empty()),
                    created_at: row.get_i64("created_at"),
                    origin: row.get_string("origin"),
                },
            );
        }
        // Users provisioned in the console (`admin_users` holds the provisioned
        // account id in `username`); `created_at` is RFC 3339 text there.
        for row in q("SELECT username,created_at FROM admin_users ORDER BY username LIMIT 15001")
            .fetch_all(&self.backend)
            .await?
        {
            let created = chrono::DateTime::parse_from_rfc3339(&row.get_string("created_at"))
                .ok()
                .map(|t| t.timestamp());
            facts
                .provisioned
                .insert(row.get_string("username"), created);
        }
        // The presentation-only native login spelling (`verified_native_directory_login`),
        // for every engine account at once.
        for row in q("SELECT u.account_id,u.address,a.username FROM mail_user_account u JOIN accounts a ON a.id=u.account_id WHERE u.mode='engine' AND a.kind IN ('imap','pop3') AND EXISTS (SELECT 1 FROM identity_binding b JOIN identity_principal p ON p.id=b.principal_id WHERE b.kind='mailbox' AND b.subject=a.id AND b.authority=(a.kind || ':' || lower(a.host) || ':' || CAST(a.port AS TEXT)) AND b.retired_at IS NULL AND p.state='active')")
            .fetch_all(&self.backend)
            .await?
        {
            facts.native_logins.insert(
                (row.get_string("account_id"), row.get_string("address")),
                row.get_string("username"),
            );
        }
        for row in q("SELECT alias,domain FROM domain_alias ORDER BY alias")
            .fetch_all(&self.backend)
            .await?
        {
            facts
                .aliases
                .insert(row.get_string("alias"), row.get_string("domain"));
        }
        for row in q("SELECT key,value FROM settings WHERE key >= ?1 AND key < ?2")
            .bind(FLAGS_FROM)
            .bind(FLAGS_TO)
            .fetch_all(&self.backend)
            .await?
        {
            let key = row.get_string("key");
            facts
                .flags
                .insert(key[FLAGS_FROM.len()..].to_string(), row.get_string("value"));
        }
        for row in q("SELECT lower(a.username) AS username,MIN(r.name) AS role FROM admin_account a JOIN admin_account_role ar ON ar.account_id=a.id JOIN admin_role r ON r.id=ar.role_id WHERE a.disabled=0 GROUP BY lower(a.username)")
            .fetch_all(&self.backend)
            .await?
        {
            facts
                .admin_roles
                .insert(row.get_string("username"), row.get_string("role"));
        }
        for row in q("SELECT scope_kind,scope_id,profile_id,priority FROM feature_profile_assignment ORDER BY priority DESC,scope_kind,scope_id,profile_id")
            .fetch_all(&self.backend)
            .await?
        {
            let key = (row.get_string("scope_kind"), row.get_string("scope_id"));
            // Highest priority first: the first assignment of a scope wins.
            facts
                .profile_assignments
                .entry(key)
                .or_insert_with(|| (row.get_string("profile_id"), row.get_i64("priority")));
        }
        for row in
            q("SELECT m.group_id,m.account_id,g.domain FROM user_group_member m JOIN user_group g ON g.id=m.group_id ORDER BY m.group_id,m.account_id")
                .fetch_all(&self.backend)
                .await?
        {
            facts.group_domains.insert(row.get_string("group_id"), row.get_string("domain"));
            facts
                .memberships
                .entry(row.get_string("account_id"))
                .or_default()
                .push(row.get_string("group_id"));
        }
        for row in q("SELECT id,name FROM feature_profile ORDER BY name,id")
            .fetch_all(&self.backend)
            .await?
        {
            facts
                .profile_names
                .push((row.get_string("id"), row.get_string("name")));
        }
        Ok(facts)
    }
}

/// The key range of the per-account flag records (`account_gate::FLAGS_PREFIX`);
/// `;` is the byte after `:`, so the range is exactly the prefix.
const FLAGS_FROM: &str = "v6:admin:flags:";
const FLAGS_TO: &str = "v6:admin:flags;";

/// Per-account facts for the Users table (t107).
#[derive(Debug, Clone, Default)]
pub struct AccountFacts {
    /// A confirmed TOTP factor.
    pub totp: bool,
    pub passkeys: i64,
    /// `(bytes_limit, msg_limit)` when a quota is set.
    pub quota: Option<(i64, i64)>,
    /// Linked mailboxes owned through this account's identity principal.
    pub linked: i64,
    /// When the account first signed in (`mail_user_account.first_seen`).
    pub first_seen: Option<i64>,
    /// Most recent foreground activity; retained after session revocation.
    pub last_seen_at: Option<i64>,
}

/// The directory record's own columns (`mail_user`), keyed by address.
#[derive(Debug, Clone)]
pub struct PersonFacts {
    pub display_name: Option<String>,
    pub created_at: i64,
    pub origin: String,
}

#[derive(Debug, Clone, Default)]
pub struct UserTableFacts {
    pub accounts: std::collections::HashMap<String, AccountFacts>,
    pub people: std::collections::HashMap<String, PersonFacts>,
    /// Console-provisioned account ids → when they were provisioned.
    pub provisioned: std::collections::HashMap<String, Option<i64>>,
    /// `(account_id, directory address)` → the native login spelling.
    pub native_logins: std::collections::HashMap<(String, String), String>,
    /// Domain alias → target domain.
    pub aliases: std::collections::HashMap<String, String>,
    /// Folded flag subject → the stored JSON record.
    pub flags: std::collections::HashMap<String, String>,
    /// Lower-cased console username → its first role name.
    pub admin_roles: std::collections::HashMap<String, String>,
    /// `(scope_kind, scope_id)` → the highest-priority `(profile id, priority)`.
    pub profile_assignments: std::collections::HashMap<(String, String), (String, i64)>,
    /// Group id → its applicable canonical domain (empty means any domain).
    pub group_domains: std::collections::HashMap<String, String>,
    /// Account id → the groups it belongs to.
    pub memberships: std::collections::HashMap<String, Vec<String>>,
    /// Every feature profile `(id, name)`, by name.
    pub profile_names: Vec<(String, String)>,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn foreground_activity_survives_logout_without_changing_signin() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        assert!(
            store
                .register_successful_signin(signin("alice", "alice@example.test", 20))
                .await
                .unwrap()
        );
        let session = store
            .create_session(
                "alice",
                "alice@example.test",
                "engine",
                "engine",
                &crate::Credentials {
                    username: "alice".into(),
                    password: "fixture".into(),
                },
            )
            .await
            .unwrap();
        store.touch_session(&session).await.unwrap();
        assert_eq!(
            store.user_table_facts().await.unwrap().accounts["alice"].last_seen_at,
            None,
            "background telemetry is not foreground activity"
        );
        let activity = "2026-10-30T12:34:56Z";
        assert!(
            store
                .record_session_activity(&session, activity)
                .await
                .unwrap()
        );
        let expected = chrono::DateTime::parse_from_rfc3339(activity)
            .unwrap()
            .timestamp();
        assert_eq!(
            store.directory_accounts().await.unwrap()[0].last_signin_at,
            Some(20)
        );
        assert_eq!(
            store.user_table_facts().await.unwrap().accounts["alice"].last_seen_at,
            Some(expected)
        );
        assert!(
            store
                .record_session_activity(&session, "2020-01-01T00:00:00Z")
                .await
                .unwrap()
        );
        store.delete_session(&session).await.unwrap();
        assert!(
            !store
                .record_session_activity(&session, "2027-01-01T00:00:00Z")
                .await
                .unwrap()
        );
        assert_eq!(
            store.user_table_facts().await.unwrap().accounts["alice"].last_seen_at,
            Some(expected),
            "logout preserves history; late requests cannot create new activity"
        );
    }

    #[tokio::test]
    async fn historical_activity_migration_uses_foreground_clock_not_polling() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::raw_sql("CREATE TABLE sessions(account_id TEXT,created_at TEXT,activity_at TEXT,last_seen TEXT); INSERT INTO sessions VALUES('alice','2020-01-01T00:00:00Z','2020-01-02T00:00:00Z','2026-01-01T00:00:00Z');").execute(&pool).await.unwrap();
        sqlx::raw_sql(include_str!("../migrations/0079_user_activity.sql"))
            .execute(&pool)
            .await
            .unwrap();
        let activity: String =
            sqlx::query_scalar("SELECT last_seen_at FROM user_activity WHERE account_id='alice'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(activity, "2020-01-02T00:00:00Z");
    }
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

    /// t107: every Users-table column comes from the one batch, keyed right.
    #[tokio::test]
    async fn user_table_facts_batches_every_column() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        for (id, address, at) in [
            ("acct-a", "alice@a.test", 100),
            ("acct-b", "bob@b.test", 200),
        ] {
            let (local, domain) = address.split_once('@').unwrap();
            assert!(
                store
                    .register_successful_signin(SuccessfulSignin {
                        account_id: id,
                        address,
                        domain,
                        local_part: local,
                        mode: "engine",
                        now: at,
                    })
                    .await
                    .unwrap()
            );
        }
        let b = &store.backend;
        q("UPDATE mail_user SET display_name='Alice A' WHERE address='alice@a.test'")
            .execute(b)
            .await
            .unwrap();
        q("INSERT INTO admin_users(id,username,password_hash,created_at) VALUES('p1','new@a.test',NULL,'2026-01-02T03:04:05Z')").execute(b).await.unwrap();
        q("INSERT INTO totp_secrets(account_id,sealed_secret,confirmed,created_at) VALUES('acct-a',?1,1,'x'),('acct-b',?1,0,'x')").bind(vec![0u8]).execute(b).await.unwrap();
        q("INSERT INTO webauthn_credentials(credential_id,account_id,cose_public_key,created_at) VALUES('c1','acct-a',?1,'x'),('c2','acct-a',?1,'x')").bind(vec![0u8]).execute(b).await.unwrap();
        q("INSERT INTO quotas(account_id,bytes_limit,msg_limit) VALUES('acct-b',1000,5)")
            .execute(b)
            .await
            .unwrap();
        q("INSERT INTO feature_profile(id,name,values_json,builtin,updated_at) VALUES('fp1','Staff','{}',0,1),('fp2','Guests','{}',0,1)").execute(b).await.unwrap();
        q("INSERT INTO feature_profile_assignment(scope_kind,scope_id,profile_id,priority) VALUES('user','acct-a','fp1',10),('user','acct-a','fp2',5),('domain','b.test','fp2',1)").execute(b).await.unwrap();
        q("INSERT INTO user_group(id,name,domain,source,created_at) VALUES('g1','G','a.test','manual',1)").execute(b).await.unwrap();
        q("INSERT INTO user_group_member(group_id,account_id) VALUES('g1','acct-b')")
            .execute(b)
            .await
            .unwrap();
        q("INSERT INTO admin_account(id,username,display_name,source,created_at,updated_at) VALUES('adm1','Alice@A.test','Alice','db',1,1)").execute(b).await.unwrap();
        q("INSERT INTO admin_role(id,name) VALUES('r1','Helpdesk')")
            .execute(b)
            .await
            .unwrap();
        q("INSERT INTO admin_account_role(account_id,role_id) VALUES('adm1','r1')")
            .execute(b)
            .await
            .unwrap();
        q("INSERT INTO domains(name,upstream_json) VALUES('a.test','{}')")
            .execute(b)
            .await
            .unwrap();
        q("INSERT INTO domain_alias(alias,domain) VALUES('old.test','a.test')")
            .execute(b)
            .await
            .unwrap();
        q("INSERT INTO settings(key,value) VALUES('v6:admin:flags:acct-b','{\"zeroAccess\":false,\"forcePasswordChange\":true,\"remoteCacheWipe\":false,\"disabled\":false}'),('v6:admin:other','x')").execute(b).await.unwrap();

        q("INSERT INTO identity_principal(id,state,created_at) VALUES('pr1','active',1),('pr2','active',1)").execute(b).await.unwrap();
        q("INSERT INTO identity_binding(id,principal_id,kind,authority,subject,created_at) VALUES('bd1','pr1','mailbox','imap:h:993','acct-a',1),('bd2','pr2','mailbox','imap:h:993','acct-c',1)").execute(b).await.unwrap();
        q("INSERT INTO linked_account(id,owner_principal_id,child_principal_id,child_subject,account_id,display_name,sealed_session,revision,created_at) VALUES('l1','pr1','pr2','acct-c','acct-c','Child',?1,'r',1)").bind(vec![0u8]).execute(b).await.unwrap();

        let f = store.user_table_facts().await.unwrap();
        let a = &f.accounts["acct-a"];
        assert_eq!(a.linked, 1);
        assert_eq!(f.accounts["acct-b"].linked, 0);
        assert!(a.totp);
        assert_eq!(a.passkeys, 2);
        assert_eq!(a.quota, None);
        assert_eq!(a.first_seen, Some(100));
        let bob = &f.accounts["acct-b"];
        assert!(!bob.totp, "an unconfirmed TOTP enrolment is not a factor");
        assert_eq!(bob.quota, Some((1000, 5)));
        assert_eq!(f.accounts["new@a.test"].passkeys, 0);
        assert_eq!(
            f.people["alice@a.test"].display_name.as_deref(),
            Some("Alice A")
        );
        assert_eq!(f.people["bob@b.test"].created_at, 200);
        assert_eq!(f.provisioned["new@a.test"], Some(1_767_323_045));
        assert_eq!(
            f.profile_assignments[&("user".to_string(), "acct-a".to_string())],
            ("fp1".to_string(), 10)
        );
        assert_eq!(
            f.profile_assignments[&("domain".to_string(), "b.test".to_string())],
            ("fp2".to_string(), 1)
        );
        assert_eq!(f.memberships["acct-b"], vec!["g1".to_string()]);
        assert_eq!(f.group_domains["g1"], "a.test");
        assert_eq!(f.admin_roles["alice@a.test"], "Helpdesk");
        assert_eq!(f.aliases["old.test"], "a.test");
        assert_eq!(f.flags.len(), 1);
        assert!(f.flags["acct-b"].contains("\"forcePasswordChange\":true"));
        assert_eq!(
            f.profile_names,
            vec![
                ("fp2".to_string(), "Guests".to_string()),
                ("fp1".to_string(), "Staff".to_string())
            ]
        );
    }
}
