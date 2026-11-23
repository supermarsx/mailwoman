//! The seven table-driven export sections (mailstate, tasks, security,
//! integrations, delegation, usage, records) are in a default export, each can
//! be switched off, URLs lose their query string and user info, and a second
//! account's rows never reach the first person's export.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::settings_policy::SettingPolicyRow;
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool, sqlite::SqliteConnectOptions};
use std::net::SocketAddr;

mod common;
use common::auth;

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const TABLE_SECTIONS: [&str; 7] = ["mailstate", "tasks", "security", "integrations", "delegation", "usage", "records"];
/// Every text value seeded for the second account starts with this.
const BRAVO: &str = "bravo-only";
const BRAVO_ADDRESS: &str = "bravo.second@example.test";

async fn serve(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    });
    format!("http://{addr}")
}

async fn set(store: &mw_store::Store, key: &str, value: Value) {
    store
        .put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "test".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}

/// Values a CHECK constraint needs that dummy values do not meet.
const HINTS: &[(&str, &str, &str)] = &[
    ("delegation_grant", "scope_kind", "account"),
    ("account_provider", "provider", "ews"),
];
static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Insert one row; NOT NULL columns without a default that `set` does not name
/// get `<tag>-<column>-<n>` (text), 1 (integer) or one zero byte (blob).
async fn seed(pool: &SqlitePool, table: &str, set: &[(&str, &str)], tag: &str) {
    let cols = sqlx::query(&format!("SELECT name,type,\"notnull\",dflt_value,pk FROM pragma_table_info('{table}')"))
        .fetch_all(pool)
        .await
        .unwrap();
    assert!(!cols.is_empty(), "{table} is not a table");
    let mut names = Vec::new();
    let mut vals = Vec::new();
    for c in cols {
        let name: String = c.get("name");
        let ty = c.get::<String, _>("type").to_ascii_uppercase();
        let notnull: i64 = c.get("notnull");
        let dflt: Option<String> = c.get("dflt_value");
        let pk: i64 = c.get("pk");
        let hint = HINTS.iter().find(|(t, c, _)| *t == table && *c == name).map(|(_, c, v)| (*c, *v));
        let given = set.iter().find(|(k, _)| *k == name).copied().or(hint);
        if given.is_none() && pk == 0 && (notnull == 0 || dflt.is_some()) {
            continue;
        }
        names.push(format!("\"{name}\""));
        vals.push(match given {
            Some((_, v)) => format!("'{}'", v.replace('\'', "''")),
            None if ty.contains("INT") => "1".into(),
            None if ty.contains("BLOB") => "x'00'".into(),
            None => format!("'{tag}-{name}-{}'", SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)),
        });
    }
    let sql = format!("INSERT INTO \"{table}\" ({}) VALUES ({})", names.join(","), vals.join(","));
    sqlx::query(&sql).execute(pool).await.unwrap_or_else(|e| panic!("{table}: {e}"));
}

struct Fixture {
    base: String,
    db: String,
    store: mw_store::Store,
    user: reqwest::Client,
    admin: reqwest::Client,
}

async fn fixture() -> Fixture {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("t48-dsr-export-sections");
    let store = mw_store::Store::open(db.to_str().unwrap(), mw_store::ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    set(&store, "signin.retryCooldownSecs", json!(0)).await;
    set(&store, "dsr.admin.stepUp", json!(false)).await;
    set(&store, "dsr.confirmation.required", json!(false)).await;
    let (router, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(KEY.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(upstream.clone()),
                jmap_upstreams: Some(vec![upstream]),
                ..SecurityConfig::default()
            },
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("hunter2".into()),
            redis_url: None,
        },
    )
    .await
    .unwrap();
    let base = serve(router).await;
    let signed = auth::sign_in(&base, mw_mock_jmap::USER, mw_mock_jmap::PASS).await;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::COOKIE, signed.cookie().parse().unwrap());
    if let Some(a) = &signed.storage_authority {
        headers.insert(auth::STORAGE_AUTHORITY_HEADER, a.parse().unwrap());
    }
    let user = reqwest::Client::builder().no_proxy().default_headers(headers).build().unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(3100)).await;
    let admin = reqwest::Client::builder().no_proxy().cookie_store(true).build().unwrap();
    let r = admin
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":"root","password":"hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    Fixture {
        base,
        db: db.to_string_lossy().into(),
        store,
        user,
        admin,
    }
}

impl Fixture {
    /// A native account; `bind` registers it to the signed-in person's address.
    async fn account(&self, username: &str, bind: bool) -> String {
        let id = self
            .store
            .create_account(
                &mw_store::NewAccount {
                    kind: mw_store::AccountKind::Imap,
                    host: "imap.example.test",
                    port: 993,
                    tls: "tls",
                    username,
                    sync_policy_json: "{}",
                },
                &mw_store::Credentials { username: username.into(), password: "x".into() },
            )
            .await
            .unwrap();
        if bind {
            let (local, domain) = username.split_once('@').unwrap();
            self.store
                .register_successful_signin(mw_store::user_directory::SuccessfulSignin {
                    account_id: &id,
                    address: username,
                    domain,
                    local_part: local,
                    mode: "engine",
                    now: chrono::Utc::now().timestamp(),
                })
                .await
                .unwrap();
        }
        id
    }

    async fn raw(&self) -> SqlitePool {
        SqlitePool::connect_with(SqliteConnectOptions::new().filename(&self.db).foreign_keys(false))
            .await
            .unwrap()
    }

    /// File an export as the signed-in person and run it; the request id.
    async fn export(&self) -> String {
        auth::user_step_up(&self.user, &self.base, mw_mock_jmap::PASS).await;
        let r = self
            .user
            .post(format!("{}/api/privacy/requests", self.base))
            .json(&json!({"kind":"access"}))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 201);
        let body: Value = r.json().await.unwrap();
        let id = body["request"]["id"].as_str().unwrap().to_string();
        let r = self.admin.post(format!("{}/admin/dsr/{id}/run", self.base)).json(&json!({})).send().await.unwrap();
        let body: Value = r.json().await.unwrap();
        assert_eq!(body["request"]["status"], "ready", "{body}");
        id
    }

    async fn sections(&self, id: &str) -> Vec<Value> {
        let r = self.admin.get(format!("{}/admin/dsr/{id}", self.base)).send().await.unwrap();
        assert_eq!(r.status(), 200);
        let body: Value = r.json().await.unwrap();
        body["request"]["export"]["sections"].as_array().cloned().unwrap_or_default()
    }

    async fn part(&self, id: &str, name: &str) -> Option<Value> {
        let bytes = self.store.dsr_part_read(id, name).await.unwrap()?;
        Some(serde_json::from_slice(&bytes).unwrap())
    }
}

/// Seed one row per section table for `acct`, with text values tagged `tag`.
async fn seed_person(pool: &SqlitePool, acct: &str, address: &str, tag: &str) {
    let cal = format!("{tag}-cal");
    seed(pool, "tags", &[("user", acct), ("name", &format!("{tag}-tag"))], tag).await;
    seed(pool, "masked_email", &[("account_id", acct)], tag).await;
    seed(pool, "calendars", &[("id", &cal), ("account_id", acct)], tag).await;
    seed(pool, "tasks", &[("list_id", &cal)], tag).await;
    seed(pool, "api_keys", &[("account_id", acct)], tag).await;
    seed(pool, "signin_identity", &[("account_id", acct)], tag).await;
    seed(
        pool,
        "webhooks",
        &[("account_id", acct), ("url", &format!("https://hooks.example.test/in/{tag}?token={tag}-hook-token#frag"))],
        tag,
    )
    .await;
    seed(
        pool,
        "account_provider",
        &[("account_id", acct), ("endpoint_url", &format!("https://{tag}-user:{tag}-pass@ews.example.test/EWS/Exchange.asmx?sig={tag}-sig"))],
        tag,
    )
    .await;
    seed(pool, "inbound_webhooks", &[("account_id", acct)], tag).await;
    seed(pool, "user_group_member", &[("account_id", acct)], tag).await;
    seed(pool, "quotas", &[("account_id", acct)], tag).await;
    seed(pool, "usage_counter", &[("address", address)], tag).await;
    seed(pool, "account_restriction", &[("account_id", acct)], tag).await;
    seed(pool, "plugin_kv", &[("account_id", acct)], tag).await;
}

#[tokio::test]
async fn a_default_export_holds_the_seven_sections_with_only_the_persons_rows() {
    let f = fixture().await;
    let alfa = f.account(mw_mock_jmap::USER, true).await;
    let bravo = f.account(BRAVO_ADDRESS, false).await;
    let pool = f.raw().await;
    seed_person(&pool, &alfa, mw_mock_jmap::USER, "alfa").await;
    seed_person(&pool, &bravo, BRAVO_ADDRESS, BRAVO).await;
    // Delegations: bravo gave alfa access (alfa received it), and alfa gave a
    // third account access. The received grant must not name bravo.
    seed(&pool, "delegation_grant", &[("id", "grant-received"), ("owner_account_id", &bravo), ("delegate_owner_account_id", &alfa), ("mailbox_ids_json", "[]"), ("permissions_json", "[\"read\"]")], "grant").await;
    seed(&pool, "delegation_grant", &[("id", "grant-given"), ("owner_account_id", &alfa), ("delegate_owner_account_id", "acct-charlie-third"), ("mailbox_ids_json", "[]"), ("permissions_json", "[\"read\"]")], "grant").await;
    // A grant between bravo and a third account has nothing to do with alfa.
    seed(&pool, "delegation_grant", &[("id", "grant-bravo-only"), ("owner_account_id", &bravo), ("delegate_owner_account_id", "acct-charlie-third"), ("mailbox_ids_json", "[]"), ("permissions_json", "[]")], BRAVO).await;
    pool.close().await;

    let id = f.export().await;
    let sections = f.sections(&id).await;
    for name in TABLE_SECTIONS {
        let s = sections.iter().find(|s| s["name"] == name).unwrap_or_else(|| panic!("{name} missing: {sections:?}"));
        assert_eq!(s["included"], true, "{s}");
        assert!(s["records"].as_u64().unwrap() >= 1, "{s}");
    }

    let mut all = String::new();
    for name in TABLE_SECTIONS {
        let part = f.part(&id, &format!("{name}.json")).await.unwrap_or_else(|| panic!("{name}.json"));
        let text = part.to_string();
        assert!(text.contains("alfa"), "{name}: the person's own rows are exported: {text}");
        all.push_str(&text);
    }
    if let Some(n) = f.part(&id, "records/compliance-notifications.json").await {
        all.push_str(&n.to_string());
    }
    assert!(!all.contains(BRAVO), "the second account's rows are in the export: {all}");
    assert!(!all.contains(&bravo), "the second account's id is in the export");
    assert!(!all.contains(BRAVO_ADDRESS), "the second account's address is in the export");
    assert!(!all.contains("grant-bravo-only"));

    // Delegation: the given grant names the delegate, the received one names nobody.
    let delegation = f.part(&id, "delegation.json").await.unwrap();
    let rows = delegation.as_array().unwrap();
    let given = rows.iter().find(|r| r["id"] == "grant-given").expect("given grant");
    assert_eq!(given["role"], "owner");
    assert_eq!(given["delegateAccountId"], "acct-charlie-third");
    let received = rows.iter().find(|r| r["id"] == "grant-received").expect("received grant");
    assert_eq!(received["role"], "delegate");
    assert_eq!(received["permissions"], json!(["read"]));
    assert!(received.get("ownerAccountId").is_none() && received.get("delegateAccountId").is_none(), "{received}");

    // URLs keep scheme, host and path only.
    let integrations = f.part(&id, "integrations.json").await.unwrap();
    let hook = integrations.as_array().unwrap().iter().find(|r| r["table"] == "webhooks").unwrap();
    assert_eq!(hook["url"], "https://hooks.example.test/in/alfa");
    let provider = integrations.as_array().unwrap().iter().find(|r| r["table"] == "account_provider").unwrap();
    assert_eq!(provider["endpointUrl"], "https://ews.example.test/EWS/Exchange.asmx");
    let text = integrations.to_string();
    for secret in ["hook-token", "alfa-pass", "alfa-user", "alfa-sig", "frag"] {
        assert!(!text.contains(secret), "{secret} is in the export: {text}");
    }
    // No sealed value or secret column reaches the export.
    let security = f.part(&id, "security.json").await.unwrap();
    let key = security.as_array().unwrap().iter().find(|r| r["table"] == "api_keys").unwrap();
    let fields: Vec<&String> = key.as_object().unwrap().keys().collect();
    assert!(!fields.iter().any(|k| k.contains("hash") || k.contains("secret")), "{fields:?}");
}

#[tokio::test]
async fn a_section_switched_off_is_listed_as_switched_off_and_not_stored() {
    let f = fixture().await;
    let alfa = f.account(mw_mock_jmap::USER, true).await;
    let pool = f.raw().await;
    seed_person(&pool, &alfa, mw_mock_jmap::USER, "alfa").await;
    pool.close().await;
    set(&f.store, "dsr.export.include.integrations", json!(false)).await;

    let id = f.export().await;
    let sections = f.sections(&id).await;
    let off = sections.iter().find(|s| s["name"] == "integrations").unwrap();
    assert_eq!(off["included"], false);
    assert_eq!(off["reason"], "switchedOff");
    assert!(f.part(&id, "integrations.json").await.is_none());
    for name in TABLE_SECTIONS.iter().filter(|n| **n != "integrations") {
        let s = sections.iter().find(|s| s["name"] == *name).unwrap();
        assert_eq!(s["included"], true, "{s}");
    }
}
