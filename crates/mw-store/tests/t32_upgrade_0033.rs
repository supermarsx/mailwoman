//! t32-e10a — migration `0033_admin_controls` over a populated database.
//!
//! An existing deployment upgrades from a database built by the earlier
//! migrations and holding real rows. This test builds that database with the
//! migrator cut off before `0032` (the 0031 era named in the plan, `t32-admin.md`
//! §3.3) and before `0033` (the release right before this one), writes rows with
//! raw SQL into every table 0033 alters plus one it does not touch, then opens the
//! file the way the server does (`Store::open` / `Store::open_postgres`) and lets
//! the full migrator bring it forward.
//!
//! It asserts what the plan promises an upgraded install (`t32-schedule.md` §4,
//! §12.3, §13.3): every new table exists and is empty; every existing row is still
//! there with its old values; the columns added to existing tables read their
//! defaults on old rows, so behaviour is unchanged until an admin changes
//! something. Two backfills are part of that promise and are asserted too:
//! `signatures.text` takes the old `body`, and `egress_proxy.kind` takes the old
//! `scheme`, so an existing SOCKS5 proxy does not turn into an HTTP one.
//!
//! The Postgres leg runs when `DATABASE_URL_PG` (or `MW_TEST_PG`) is set, inside a
//! schema of its own that is dropped afterwards.

use mw_store::{ServerKey, Store};

#[path = "../src/test_db.rs"]
mod test_db;

fn key() -> ServerKey {
    ServerKey::from_bytes(&[5u8; 32]).unwrap()
}

fn pg_dsn() -> Option<String> {
    std::env::var("DATABASE_URL_PG")
        .ok()
        .or_else(|| std::env::var("MW_TEST_PG").ok())
        .filter(|s| !s.trim().is_empty())
}

/// Every table 0033 creates. Written out here, not read from the migration, so a
/// table the migration forgets is caught.
const NEW_TABLES: &[&str] = &[
    "setting_policy",
    "setting_value",
    "auth_failure",
    "ip_block",
    "ip_allow",
    "domain_alias",
    "domain_host",
    "ssh_tunnel",
    "admin_account",
    "admin_totp",
    "admin_role",
    "admin_role_capability",
    "admin_account_role",
    "admin_capability_override",
    "audit_chain_head",
    "audit_checkpoint",
    "audit_retention",
    "audit_sink",
    "admin_saved_query",
    "alert_rule",
    "alert_event",
    "signin_event",
    "remembered_device",
    "branding",
    "brand_asset",
    "custom_css",
    "custom_theme",
    "custom_accent",
    "sieve_script",
    "egress_internal_allow",
    "smime_trust_root",
    "pki_revocation_cache",
    "identity_signature_default",
    "domain_signature_template",
    "domain_disclaimer",
    "mdn_sender_rule",
    "receipt_status",
    "child_account",
    "provider_profile",
    "oauth_client",
    "account_oauth",
    "delegation_grant",
    "shared_mailbox",
    "org_view",
    "identity_verification",
    "template",
    "template_version",
    "user_group",
    "user_group_member",
    "feature_profile",
    "feature_profile_assignment",
    "mail_user",
    "mail_user_account",
    "signin_allow",
    "usage_counter",
    "usage_storage",
    "config_snapshot",
    "integration_config",
    "maintenance_window",
    "scheduled_task",
    "job_run",
    "diag_run",
    "diag_probe_account",
    "account_ban",
    "metric_rollup",
    "data_export",
];

#[derive(Clone)]
enum Db {
    Lite(sqlx::SqlitePool),
    Pg(sqlx::PgPool),
}

impl Db {
    fn name(&self) -> &'static str {
        match self {
            Db::Lite(_) => "sqlite",
            Db::Pg(_) => "postgres",
        }
    }

    /// `B'hex'` in `sql` becomes this dialect's blob literal.
    fn dialect_sql(&self, sql: &str) -> String {
        let mut out = String::new();
        let mut rest = sql;
        while let Some(i) = rest.find("B'") {
            out.push_str(&rest[..i]);
            let tail = &rest[i + 2..];
            let end = tail.find('\'').expect("blob literal end");
            let hex = &tail[..end];
            match self {
                Db::Lite(_) => out.push_str(&format!("x'{hex}'")),
                Db::Pg(_) => out.push_str(&format!("'\\x{hex}'::bytea")),
            }
            rest = &tail[end + 1..];
        }
        out.push_str(rest);
        out
    }

    async fn exec(&self, sql: &str) {
        let sql = self.dialect_sql(sql);
        match self {
            Db::Lite(p) => sqlx::query(&sql).execute(p).await.map(|_| ()),
            Db::Pg(p) => sqlx::query(&sql).execute(p).await.map(|_| ()),
        }
        .unwrap_or_else(|e| panic!("{}: {sql}: {e}", self.name()));
    }

    /// Rows of a query whose every column is cast to TEXT by the caller.
    async fn texts(&self, sql: &str) -> Vec<Vec<Option<String>>> {
        use sqlx::Row as _;
        match self {
            Db::Lite(p) => sqlx::query(sql)
                .fetch_all(p)
                .await
                .unwrap_or_else(|e| panic!("sqlite: {sql}: {e}"))
                .iter()
                .map(|r| {
                    (0..r.len())
                        .map(|i| r.get::<Option<String>, _>(i))
                        .collect()
                })
                .collect(),
            Db::Pg(p) => sqlx::query(sql)
                .fetch_all(p)
                .await
                .unwrap_or_else(|e| panic!("postgres: {sql}: {e}"))
                .iter()
                .map(|r| {
                    (0..r.len())
                        .map(|i| r.get::<Option<String>, _>(i))
                        .collect()
                })
                .collect(),
        }
    }

    async fn one(&self, sql: &str) -> Vec<Option<String>> {
        let mut rows = self.texts(sql).await;
        assert_eq!(rows.len(), 1, "{}: {sql}: expected one row", self.name());
        rows.remove(0)
    }

    async fn count(&self, sql: &str) -> i64 {
        self.one(sql).await[0]
            .as_deref()
            .expect("count")
            .parse()
            .unwrap()
    }

    async fn table_exists(&self, table: &str) -> bool {
        let sql = match self {
            Db::Lite(_) => format!(
                "SELECT CAST(COUNT(*) AS TEXT) FROM sqlite_master \
                  WHERE type = 'table' AND name = '{table}'"
            ),
            Db::Pg(_) => format!(
                "SELECT CAST(COUNT(*) AS TEXT) FROM information_schema.tables \
                  WHERE table_schema = current_schema() AND table_name = '{table}'"
            ),
        };
        self.count(&sql).await == 1
    }

    async fn index_exists(&self, index: &str) -> bool {
        let sql = match self {
            Db::Lite(_) => format!(
                "SELECT CAST(COUNT(*) AS TEXT) FROM sqlite_master \
                  WHERE type = 'index' AND name = '{index}'"
            ),
            Db::Pg(_) => format!(
                "SELECT CAST(COUNT(*) AS TEXT) FROM pg_indexes \
                  WHERE schemaname = current_schema() AND indexname = '{index}'"
            ),
        };
        self.count(&sql).await == 1
    }

    async fn close(&self) {
        match self {
            Db::Lite(p) => p.close().await,
            Db::Pg(p) => p.close().await,
        }
    }
}

fn s(v: &str) -> Option<String> {
    Some(v.to_string())
}

/// Rows written at the pre-0033 schema. Only columns that existed before 0033 are
/// named, so nothing here can depend on the migration under test.
const SEED: &[&str] = &[
    "INSERT INTO settings (key, value) VALUES ('theme', 'grove-dark')",
    "INSERT INTO accounts (id, kind, host, port, tls, username, sealed_creds, sync_policy_json) \
     VALUES ('acct-old', 'imap', 'imap.example.test', 993, 'implicit', 'old@example.test', \
             B'0102', '{}')",
    "INSERT INTO bodies (blob_ref, account_id, sealed_bytes) VALUES ('blob-old', 'acct-old', B'0a0b')",
    "INSERT INTO identities (id, account_id, name, email, source) \
     VALUES ('ident-old', 'acct-old', 'Old', 'old@example.test', 'configured')",
    "INSERT INTO domains (name, upstream_json, allowlist, blocklist) \
     VALUES ('old.test', '{\"imap\":\"imap.old.test\"}', '[\"a@old.test\"]', '[]')",
    "INSERT INTO domains (name, upstream_json) VALUES ('second.test', '{}')",
    "INSERT INTO audit_log (id, ts, actor, actor_kind, action, target, detail_json, ip) \
     VALUES ('audit-old', '2026-01-01T00:00:00Z', 'root', 'admin', 'domain.update', 'old.test', \
             '{}', '192.0.2.1')",
    "INSERT INTO sessions (id, account_id, username, jmap_url, api_url, sealed_creds, created_at, \
                           last_seen) \
     VALUES ('sess-old', 'acct-old', 'old@example.test', 'http://j', 'http://a', B'0304', 't0', 't1')",
    "INSERT INTO webauthn_credentials (credential_id, account_id, cose_public_key, created_at) \
     VALUES ('cred-old', 'acct-old', B'a1', 't0')",
    "INSERT INTO egress_proxy (id, scheme, host, port, created_at, updated_at) \
     VALUES ('px-http', 'http', 'proxy.example.test', 3128, 't0', 't0')",
    "INSERT INTO egress_proxy (id, scheme, host, port, created_at, updated_at) \
     VALUES ('px-socks', 'socks5', 'socks.example.test', 1080, 't0', 't0')",
    "INSERT INTO signatures (account_id, name, body, is_default, rule_json, updated_at) \
     VALUES ('acct-old', 'Work', 'Regards,\nOld', 1, '', 't0')",
    "INSERT INTO saved_searches (id, \"user\", name, query_json, as_folder) \
     VALUES ('ss-old', 'old@example.test', 'Unread', '{\"unread\":true}', 1)",
    "INSERT INTO directory_config (id, url, base_dn, bind_dn, tls) \
     VALUES ('dir-old', 'ldaps://ldap.example.test', 'dc=example,dc=test', 'cn=svc', 'ldaps')",
];

async fn assert_upgraded(db: &Db) -> Vec<String> {
    let d = db.name();
    let mut bad = Vec::new();

    let applied = db
        .count("SELECT CAST(COUNT(*) AS TEXT) FROM _sqlx_migrations WHERE version = 33 AND success")
        .await;
    if applied != 1 {
        bad.push(format!("{d}: migration 33 is not recorded as applied"));
    }

    for t in NEW_TABLES {
        if !db.table_exists(t).await {
            bad.push(format!("{d}: table {t} missing after the upgrade"));
            continue;
        }
        let n = db
            .count(&format!("SELECT CAST(COUNT(*) AS TEXT) FROM {t}"))
            .await;
        if n != 0 {
            bad.push(format!(
                "{d}: new table {t} holds {n} rows after the upgrade"
            ));
        }
    }
    if !db.index_exists("idx_bodies_account").await {
        bad.push(format!("{d}: idx_bodies_account missing"));
    }
    if !bad.is_empty() {
        return bad;
    }

    let mut check = |what: &str, got: Vec<Option<String>>, want: Vec<Option<String>>| {
        if got != want {
            bad.push(format!("{d}: {what}: got {got:?}, want {want:?}"));
        }
    };

    // Untouched tables and old columns keep their values.
    check(
        "settings row",
        db.one("SELECT value FROM settings WHERE key = 'theme'")
            .await,
        vec![s("grove-dark")],
    );
    check(
        "accounts row",
        db.one("SELECT host, CAST(port AS TEXT), username FROM accounts WHERE id = 'acct-old'")
            .await,
        vec![s("imap.example.test"), s("993"), s("old@example.test")],
    );
    check(
        "bodies row",
        db.one("SELECT account_id, CAST(length(sealed_bytes) AS TEXT) FROM bodies")
            .await,
        vec![s("acct-old"), s("2")],
    );

    // domains: old values intact; new columns at their defaults.
    check(
        "domains old.test",
        db.one(
            "SELECT upstream_json, allowlist, blocklist, CAST(enabled AS TEXT), \
                    CAST(preferred AS TEXT), settings_json, CAST(updated_at AS TEXT) \
               FROM domains WHERE name = 'old.test'",
        )
        .await,
        vec![
            s("{\"imap\":\"imap.old.test\"}"),
            s("[\"a@old.test\"]"),
            s("[]"),
            s("1"),
            s("0"),
            s("{}"),
            None,
        ],
    );
    check(
        "domains count",
        vec![Some(
            db.count("SELECT CAST(COUNT(*) AS TEXT) FROM domains")
                .await
                .to_string(),
        )],
        vec![s("2")],
    );

    check(
        "audit_log row",
        db.one(
            "SELECT ts, actor, action, target, ip, admin_role, capability, category, request_id, \
                    before_json, after_json, prev_hash, hash, CAST(seq AS TEXT) \
               FROM audit_log WHERE id = 'audit-old'",
        )
        .await,
        vec![
            s("2026-01-01T00:00:00Z"),
            s("root"),
            s("domain.update"),
            s("old.test"),
            s("192.0.2.1"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
    );

    check(
        "sessions row",
        db.one(
            "SELECT username, created_at, last_seen, user_agent, ip, client_type, device_label, \
                    client_version \
               FROM sessions WHERE id = 'sess-old'",
        )
        .await,
        vec![
            s("old@example.test"),
            s("t0"),
            s("t1"),
            None,
            None,
            None,
            None,
            None,
        ],
    );

    check(
        "webauthn_credentials row",
        db.one(
            "SELECT account_id, CAST(sign_count AS TEXT), CAST(discoverable AS TEXT) \
               FROM webauthn_credentials WHERE credential_id = 'cred-old'",
        )
        .await,
        vec![s("acct-old"), s("0"), s("0")],
    );

    let proxy_cols = "SELECT scheme, host, kind, tunnel_id, CAST(tls_to_proxy AS TEXT), \
                             CAST(remote_dns AS TEXT), remote_dns_risk_accepted_by, \
                             CAST(remote_dns_risk_accepted_at AS TEXT), chain_next_id, scope, \
                             traffic_classes_json, CAST(active AS TEXT) \
                        FROM egress_proxy WHERE id = ";
    let proxy_tail = |scheme: &str, host: &str| {
        vec![
            s(scheme),
            s(host),
            s(scheme),
            None,
            s("0"),
            s("0"),
            None,
            None,
            None,
            s("global"),
            s("[\"remote_content\",\"ics_feeds\"]"),
            s("0"),
        ]
    };
    check(
        "egress_proxy http row (kind backfilled from scheme)",
        db.one(&format!("{proxy_cols}'px-http'")).await,
        proxy_tail("http", "proxy.example.test"),
    );
    check(
        "egress_proxy socks5 row (kind backfilled from scheme)",
        db.one(&format!("{proxy_cols}'px-socks'")).await,
        proxy_tail("socks5", "socks.example.test"),
    );

    check(
        "signatures row (text backfilled from body)",
        db.one(
            "SELECT body, CAST(is_default AS TEXT), html, text, CAST(text_edited AS TEXT), \
                    images_json \
               FROM signatures WHERE account_id = 'acct-old' AND name = 'Work'",
        )
        .await,
        vec![
            s("Regards,\nOld"),
            s("1"),
            None,
            s("Regards,\nOld"),
            s("0"),
            s("[]"),
        ],
    );

    check(
        "saved_searches row",
        db.one(
            "SELECT name, query_json, CAST(as_folder AS TEXT), query_text, builder_json, \
                    accounts_json, settings_json, CAST(position AS TEXT), icon, accent_id, \
                    CAST(start_page AS TEXT), source, org_view_id \
               FROM saved_searches WHERE id = 'ss-old'",
        )
        .await,
        vec![
            s("Unread"),
            s("{\"unread\":true}"),
            s("1"),
            None,
            None,
            None,
            s("{}"),
            s("0"),
            None,
            None,
            s("0"),
            s("user"),
            None,
        ],
    );

    check(
        "identities row",
        db.one(
            "SELECT email, source, auto_bcc, color, pgp_key_id, smime_cert_id, \
                    CAST(request_receipt AS TEXT), CAST(verified_at AS TEXT), sender_header \
               FROM identities WHERE id = 'ident-old'",
        )
        .await,
        vec![
            s("old@example.test"),
            s("configured"),
            None,
            None,
            None,
            None,
            s("0"),
            None,
            None,
        ],
    );

    check(
        "directory_config row",
        db.one(
            "SELECT url, bind_dn, CAST(enabled AS TEXT), \
                    CAST((sealed_bind_password IS NULL) AS TEXT) \
               FROM directory_config WHERE id = 'dir-old'",
        )
        .await,
        vec![
            s("ldaps://ldap.example.test"),
            s("cn=svc"),
            s("1"),
            // SQLite renders the boolean as 1, Postgres as true.
            s(if matches!(db, Db::Lite(_)) {
                "1"
            } else {
                "true"
            }),
        ],
    );

    bad
}

fn report(bad: &[String]) {
    assert!(
        bad.is_empty(),
        "upgrade to 0033 broke {} promise(s):\n  - {}",
        bad.len(),
        bad.join("\n  - ")
    );
}

fn cut(mut m: sqlx::migrate::Migrator, before: i64) -> sqlx::migrate::Migrator {
    m.migrations.to_mut().retain(|mig| mig.version < before);
    assert!(
        m.migrations.iter().any(|mig| mig.version == before - 1)
            && m.migrations.iter().all(|mig| mig.version < before),
        "the cut-off migrator ends at {}",
        before - 1
    );
    m
}

async fn sqlite_upgrade_from(before: i64) {
    let path = test_db::unique_file_path(&format!("mw-store-t32-upgrade-{before}"), "u.sqlite");
    let path_str = path.to_string_lossy().to_string();
    let url = format!("sqlite://{path_str}?mode=rwc");

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    cut(sqlx::migrate!("./migrations"), before)
        .run(&pool)
        .await
        .unwrap();
    let db = Db::Lite(pool);
    assert!(
        !db.table_exists("mail_user").await,
        "the seed really is at the pre-0033 schema"
    );
    for stmt in SEED {
        db.exec(stmt).await;
    }
    db.close().await;

    drop(
        Store::open(&path_str, key())
            .await
            .expect("0033 applies over a populated database"),
    );

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    let db = Db::Lite(pool);
    let bad = assert_upgraded(&db).await;
    db.close().await;
    let _ = std::fs::remove_file(&path_str);
    let _ = std::fs::remove_file(format!("{path_str}-wal"));
    let _ = std::fs::remove_file(format!("{path_str}-shm"));
    report(&bad);
    eprintln!(
        "[mw-store] t32 upgrade from the pre-{before:04} schema (SQLite): {} new tables empty, \
         old rows intact, new columns at their defaults.",
        NEW_TABLES.len()
    );
}

async fn postgres_upgrade_from(before: i64) {
    let Some(dsn) = pg_dsn() else {
        mw_test_gate::skip(format!(
            "[mw-store] t32 upgrade from the pre-{before:04} schema: Postgres path not driven \
             (set DATABASE_URL_PG or MW_TEST_PG to a live postgres:16 to run it)."
        ));
        return;
    };
    let schema = format!(
        "t32_e10a_up{before}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .expect("DATABASE_URL_PG is set but Postgres is not reachable");
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin)
        .await
        .unwrap();
    let sep = if dsn.contains('?') { '&' } else { '?' };
    let scoped = format!("{dsn}{sep}options=-c%20search_path%3D{schema}");

    let outcome = tokio::spawn(async move {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&scoped)
            .await
            .unwrap();
        cut(sqlx::migrate!("./migrations_pg"), before)
            .run(&pool)
            .await
            .unwrap();
        let db = Db::Pg(pool);
        assert!(
            !db.table_exists("mail_user").await,
            "the seed really is at the pre-0033 schema"
        );
        for stmt in SEED {
            db.exec(stmt).await;
        }
        db.close().await;

        drop(
            Store::open_postgres(&scoped, key())
                .await
                .expect("0033 applies over a populated database"),
        );

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&scoped)
            .await
            .unwrap();
        let db = Db::Pg(pool);
        let bad = assert_upgraded(&db).await;
        db.close().await;
        bad
    })
    .await;
    let _ = sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&admin)
        .await;
    let bad = outcome.expect("the Postgres leg panicked");
    report(&bad);
    eprintln!(
        "[mw-store] t32 upgrade from the pre-{before:04} schema (Postgres): RAN; {} new tables \
         empty, old rows intact, new columns at their defaults.",
        NEW_TABLES.len()
    );
}

/// The 0031 era: the database as it stood before 0032 and 0033.
#[tokio::test]
async fn upgrade_from_0031_era_sqlite() {
    sqlite_upgrade_from(32).await;
}

/// The 0032 era: the release right before this one.
#[tokio::test]
async fn upgrade_from_0032_era_sqlite() {
    sqlite_upgrade_from(33).await;
}

#[tokio::test]
async fn upgrade_from_0031_era_postgres() {
    postgres_upgrade_from(32).await;
}

#[tokio::test]
async fn upgrade_from_0032_era_postgres() {
    postgres_upgrade_from(33).await;
}
