//! t107: `GET /admin/users/table` — the admin Users table's server side. The
//! columns (display name, created, last sign-in or never, second factor, console
//! role, feature profile, quota, flags), the filters, the sort orders, paging, and
//! role scoping: a domain-scoped admin sees only its domains' users, and every
//! count and facet is computed after scoping.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
use std::net::SocketAddr;
mod common;
const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

async fn serve(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{addr}")
}
fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap()
}
async fn admin_login(base: &str, username: &str, password: &str) -> reqwest::Client {
    tokio::time::sleep(std::time::Duration::from_millis(3100)).await;
    let c = client();
    let r = c
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":username,"password":password}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "{}", r.text().await.unwrap());
    c
}
async fn table(c: &reqwest::Client, base: &str, query: &str) -> Value {
    let r = c
        .get(format!("{base}/admin/users/table?{query}"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    r.json().await.unwrap()
}
fn addresses(page: &Value) -> Vec<String> {
    page["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["loginAddress"].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn users_table_columns_filters_sort_paging_and_scope() {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("t107-users-table");
    let store = mw_store::Store::open(
        db.to_str().unwrap(),
        mw_store::ServerKey::from_hex(KEY).unwrap(),
    )
    .await
    .unwrap();
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
                jmap_upstreams: Some(vec![upstream.clone()]),
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

    // Signed-in users: one today, one 40 days ago, one 100 days ago; two domains.
    let now = chrono::Utc::now().timestamp();
    for (id, address, at) in [
        ("acct-alice", "alice@a.test", now - 3600),
        ("acct-amy", "amy@a.test", now - 40 * 86_400),
        ("acct-bob", "bob@b.test", now - 100 * 86_400),
    ] {
        let (local, domain) = address.split_once('@').unwrap();
        assert!(
            store
                .register_successful_signin(mw_store::user_directory::SuccessfulSignin {
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
    store
        .put_totp_secret("acct-amy", b"0123456789abcdef", true)
        .await
        .unwrap();

    let root = admin_login(&base, "root", "hunter2").await;
    // Provisioned in the console, never signed in.
    for (domain, username) in [("a.test", "newbie"), ("b.test", "fresh")] {
        let r = root
            .post(format!("{base}/admin/users"))
            .json(&json!({"domain":domain,"username":username,"quota":{"bytesLimit":5000,"msgLimit":0}}))
            .send()
            .await
            .unwrap();
        assert!(r.status().is_success(), "{}", r.text().await.unwrap());
    }

    // Explicit name ordering remains supported alongside the activity default.
    let all = table(&root, &base, "sort=name").await;
    assert_eq!(all["total"], 5);
    assert_eq!(all["unfilteredTotal"], 5);
    assert_eq!(
        addresses(&all),
        [
            "alice@a.test",
            "amy@a.test",
            "bob@b.test",
            "fresh@b.test",
            "newbie@a.test"
        ]
    );
    assert_eq!(all["facets"]["domains"], json!(["a.test", "b.test"]));
    let row = |page: &Value, address: &str| -> Value {
        page["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["loginAddress"] == address)
            .cloned()
            .unwrap()
    };
    let alice = row(&all, "alice@a.test");
    assert_eq!(alice["state"], "active");
    assert_eq!(alice["lastSigninAt"], now - 3600);
    assert_eq!(alice["lastSeenAt"], now - 3600);
    assert_eq!(alice["twoFactor"], "none");
    assert_eq!(alice["linkedMailboxes"], 0);
    assert!(alice["createdAt"].as_i64().is_some());
    assert_eq!(row(&all, "amy@a.test")["twoFactor"], "totp");
    assert_eq!(row(&all, "amy@a.test")["twoFactorMethods"], json!(["totp"]));
    let newbie = row(&all, "newbie@a.test");
    assert_eq!(newbie["state"], "pending");
    assert_eq!(newbie["lastSigninAt"], Value::Null);
    assert_eq!(newbie["lastSeenAt"], Value::Null);
    assert_eq!(newbie["quota"]["bytesLimit"], 5000);
    assert!(newbie["createdAt"].as_i64().unwrap() >= now - 60);

    let active = store
        .create_session(
            "acct-bob",
            "bob@b.test",
            "engine",
            "engine",
            &mw_store::Credentials {
                username: "bob".into(),
                password: "fixture".into(),
            },
        )
        .await
        .unwrap();
    let seen = chrono::DateTime::from_timestamp(now, 0)
        .unwrap()
        .to_rfc3339();
    assert!(store.record_session_activity(&active, &seen).await.unwrap());
    store.delete_session(&active).await.unwrap();
    let recent = table(&root, &base, "").await;
    assert_eq!(addresses(&recent)[0], "bob@b.test");
    assert_eq!(row(&recent, "bob@b.test")["lastSeenAt"], now);
    assert_eq!(
        row(&recent, "bob@b.test")["lastSigninAt"],
        now - 100 * 86_400
    );
    assert_eq!(
        addresses(&table(&root, &base, "sort=lastSeen&dir=desc").await),
        addresses(&recent)
    );

    // Last sign-in: never sorts oldest (first ascending, last descending).
    let asc = table(&root, &base, "sort=lastSignin&dir=asc").await;
    assert_eq!(
        &addresses(&asc)[2..],
        ["bob@b.test", "amy@a.test", "alice@a.test"]
    );
    let desc = table(&root, &base, "sort=lastSignin&dir=desc").await;
    assert_eq!(
        &addresses(&desc)[..3],
        ["alice@a.test", "amy@a.test", "bob@b.test"]
    );
    let by_factor = table(&root, &base, "sort=twoFactor").await;
    assert_eq!(addresses(&by_factor)[0], "amy@a.test");

    // Filters.
    let q = |query: &'static str| {
        let root = root.clone();
        let base = base.clone();
        async move { addresses(&table(&root, &base, &format!("{query}&sort=name")).await) }
    };
    assert_eq!(q("signin=never").await, ["fresh@b.test", "newbie@a.test"]);
    assert_eq!(q("signin=older:30").await, ["amy@a.test", "bob@b.test"]);
    assert_eq!(q("signin=older:90").await, ["bob@b.test"]);
    assert_eq!(q("signin=within:7").await, ["alice@a.test"]);
    assert_eq!(q("status=pending").await, ["fresh@b.test", "newbie@a.test"]);
    assert_eq!(
        q("status=active,pending&domain=b.test").await,
        ["bob@b.test", "fresh@b.test"]
    );
    assert_eq!(q("twoFactor=totp").await, ["amy@a.test"]);
    assert_eq!(
        q("twoFactor=none&domain=a.test").await,
        ["alice@a.test", "newbie@a.test"]
    );
    assert_eq!(q("role=admin").await, Vec::<String>::new());
    assert_eq!(q("profile=none").await.len(), 5);
    assert_eq!(q("q=AM").await, ["amy@a.test"]);
    assert_eq!(q("q=b.test").await, ["bob@b.test", "fresh@b.test"]);
    let filtered = table(&root, &base, "q=a.test&signin=never").await;
    assert_eq!(filtered["total"], 1);
    assert_eq!(filtered["unfilteredTotal"], 5);
    let created_from = now - 120;
    assert_eq!(
        addresses(
            &table(
                &root,
                &base,
                &format!("createdFrom={created_from}&sort=name")
            )
            .await
        ),
        ["fresh@b.test", "newbie@a.test"]
    );

    // Paging.
    let p = table(&root, &base, "limit=2&offset=2&sort=name").await;
    assert_eq!(p["total"], 5);
    assert_eq!(p["offset"], 2);
    assert_eq!(p["limit"], 2);
    assert_eq!(addresses(&p), ["bob@b.test", "fresh@b.test"]);
    let past = table(&root, &base, "limit=2&offset=99").await;
    assert_eq!(addresses(&past), Vec::<String>::new());
    assert_eq!(table(&root, &base, "limit=100000").await["limit"], 200);

    // Role scoping: an admin limited to b.test sees b.test only, counts included.
    // Creating roles and admins is a sensitive action: prove the password first.
    assert_eq!(
        root.post(format!("{base}/admin/step-up"))
            .json(&json!({"password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let role: Value = root
        .post(format!("{base}/admin/roles"))
        .json(&json!({"name":"b readers","capabilities":["users.read"]}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(role["id"].is_string(), "{role}");
    let created = root
        .post(format!("{base}/admin/admins"))
        .json(&json!({"username":"breader","password":"long scoped password","roles":[{"roleId":role["id"],"domains":["b.test"]}]}))
        .send()
        .await
        .unwrap();
    assert!(
        created.status().is_success(),
        "{}",
        created.text().await.unwrap()
    );
    store
        .put_feature_profile(
            &mw_store::feature_profiles::FeatureProfile {
                id: "private-profile".into(),
                name: "Private organisation policy".into(),
                values: Default::default(),
                builtin: false,
                updated_at: now,
            },
            "root",
        )
        .await
        .unwrap();
    let scoped = admin_login(&base, "breader", "long scoped password").await;
    let mine = table(&scoped, &base, "").await;
    assert_eq!(addresses(&mine), ["bob@b.test", "fresh@b.test"]);
    assert_eq!(mine["unfilteredTotal"], 2);
    assert_eq!(mine["facets"]["domains"], json!(["b.test"]));
    assert_eq!(
        mine["facets"]["profiles"],
        json!([]),
        "scoped readers cannot enumerate unrelated profiles"
    );
    assert_eq!(
        addresses(&table(&scoped, &base, "domain=a.test").await),
        Vec::<String>::new()
    );
    assert_eq!(
        addresses(&table(&scoped, &base, "q=alice").await),
        Vec::<String>::new()
    );

    // Without users.read: refused.
    let none: Value = root
        .post(format!("{base}/admin/roles"))
        .json(&json!({"name":"nothing","capabilities":["dashboard.read"]}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        root.post(format!("{base}/admin/admins"))
            .json(&json!({"username":"blind","password":"long blind password","roles":[{"roleId":none["id"],"domains":null}]}))
            .send()
            .await
            .unwrap()
            .status()
            .is_success()
    );
    let blind = admin_login(&base, "blind", "long blind password").await;
    assert_eq!(
        blind
            .get(format!("{base}/admin/users/table"))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        client()
            .get(format!("{base}/admin/users/table"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
}
