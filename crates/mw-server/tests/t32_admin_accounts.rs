//! Recovery coverage for the database admin account and role routes.
use std::net::SocketAddr;

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use reqwest::{Client, Response};
use serde_json::{Value, json};

mod common;
use common::test_db;

async fn server() -> String {
    let web = test_db::unique_dir("admin-accounts-web");
    std::fs::write(web.join("index.html"), "<!doctype html><title>Test</title>").unwrap();
    let config = AppConfig {
        db_path: test_db::unique_db_path("admin-accounts")
            .to_string_lossy()
            .into(),
        server_key_hex: None,
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let (app, _) = build_app_full(
        config,
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("hunter2".into()),
            redis_url: None,
        },
    )
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{addr}")
}

fn client() -> Client {
    Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn login(c: &Client, base: &str, username: &str, password: &str) -> Response {
    c.post(format!("{base}/admin/login"))
        .json(&json!({"username": username, "password": password}))
        .send()
        .await
        .unwrap()
}

async fn confirm(c: &Client, base: &str, password: &str) {
    let response = c
        .post(format!("{base}/admin/step-up"))
        .json(&json!({"password": password}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        200,
        "step-up: {}",
        response.text().await.unwrap()
    );
}

async fn root(base: &str) -> Client {
    let c = client();
    assert_eq!(login(&c, base, "root", "hunter2").await.status(), 200);
    confirm(&c, base, "hunter2").await;
    c
}

#[tokio::test]
async fn bootstrap_preserves_last_super_admin_and_hides_secrets() {
    let base = server().await;
    let c = root(&base).await;
    let list: Value = c
        .get(format!("{base}/admin/admins"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = list["admins"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    let admin = &rows[0];
    assert_eq!(admin["username"], "root");
    assert!(admin.get("passwordHash").is_none());
    assert!(!list.to_string().contains("hunter2"));
    let response = c
        .delete(format!(
            "{base}/admin/admins/{}",
            admin["id"].as_str().unwrap()
        ))
        .header("If-Match", admin["etag"].as_str().unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 409);
    assert_eq!(
        response.json::<Value>().await.unwrap()["code"],
        "lastSuperAdmin"
    );
    assert_eq!(
        c.get(format!("{base}/admin/session"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
}

#[tokio::test]
async fn database_password_authenticates_but_no_roles_grant_no_admin_actions() {
    let base = server().await;
    let root = root(&base).await;
    let created = root
        .post(format!("{base}/admin/admins"))
        .json(
            &json!({"username": "reader", "password": "a sufficiently long password", "roles": []}),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), 201);
    let account: Value = created.json().await.unwrap();
    assert!(account.get("passwordHash").is_none());
    let c = client();
    assert_eq!(login(&c, &base, "reader", "wrong").await.status(), 401);
    assert_eq!(
        login(&c, &base, "reader", "a sufficiently long password")
            .await
            .status(),
        200
    );
    assert_eq!(
        c.get(format!("{base}/admin/me"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        c.get(format!("{base}/admin/admins"))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        c.post(format!("{base}/admin/roles"))
            .json(&json!({"name": "Escalation", "capabilities": ["admins.manage"]}))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        c.post(format!("{base}/admin/logout"))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(
        c.get(format!("{base}/admin/session"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
}

async fn create(c: &Client, base: &str, name: &str, roles: Value) -> Value {
    let response = c
        .post(format!("{base}/admin/admins"))
        .json(&json!({"username": name, "password":"long test password", "roles":roles}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 201);
    // Read the list because the response above was consumed for diagnostic output.
    let list: Value = c
        .get(format!("{base}/admin/admins"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    list["admins"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["username"] == name)
        .unwrap()
        .clone()
}

#[tokio::test]
async fn limited_manager_cannot_take_over_a_super_admin_or_grant_unheld_capabilities() {
    let base = server().await;
    let root = root(&base).await;
    let role: Value = root.post(format!("{base}/admin/roles")).json(&json!({"name":"Managers", "capabilities":["admins.read","admins.manage","roles.manage"]})).send().await.unwrap().json().await.unwrap();
    create(&root, &base, "manager", json!([{"roleId":role["id"]}])).await;
    let manager = client();
    assert_eq!(
        login(&manager, &base, "manager", "long test password")
            .await
            .status(),
        200
    );
    confirm(&manager, &base, "long test password").await;
    let list: Value = root
        .get(format!("{base}/admin/admins"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = list["admins"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["username"] == "root")
        .unwrap()["id"]
        .as_str()
        .unwrap();
    for suffix in ["reset-password", "totp"] {
        let response = manager
            .post(format!("{base}/admin/admins/{id}/{suffix}"))
            .json(&json!({"password":"stolen password long"}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 403);
    }
    let response = manager
        .post(format!("{base}/admin/roles"))
        .json(&json!({"name":"Escalation", "capabilities":["settings.write"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 403);
    assert_eq!(
        response.json::<Value>().await.unwrap()["code"],
        "capabilityNotHeld"
    );
}

#[tokio::test]
async fn totp_challenge_is_single_use_and_password_reset_invalidates_pending_login() {
    use mw_mfa::totp::{TotpParams, base32_decode, totp_at};
    let base = server().await;
    let root = root(&base).await;
    let account = create(&root, &base, "factor", json!([])).await;
    let id = account["id"].as_str().unwrap();
    let factor: Value = root
        .post(format!("{base}/admin/admins/{id}/totp"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let secret = base32_decode(factor["secret"].as_str().unwrap()).unwrap();
    let c = client();
    let response = login(&c, &base, "factor", "long test password").await;
    assert!(response.headers().get("set-cookie").is_none());
    let challenge: Value = response.json().await.unwrap();
    assert_eq!(challenge["twofaRequired"], true);
    let code = totp_at(
        &secret,
        chrono::Utc::now().timestamp() as u64,
        &TotpParams::default(),
    );
    let body = json!({"pendingToken":challenge["pendingToken"],"code":code});
    assert_eq!(
        c.post(format!("{base}/admin/login/totp"))
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        c.post(format!("{base}/admin/login/totp"))
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        410
    );
    let pending: Value = login(&client(), &base, "factor", "long test password")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(
        root.post(format!("{base}/admin/admins/{id}/reset-password"))
            .json(&json!({"password":"changed long password"}))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(
        c.get(format!("{base}/admin/session"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        client()
            .post(format!("{base}/admin/login/totp"))
            .json(&json!({"pendingToken":pending["pendingToken"],"code":code}))
            .send()
            .await
            .unwrap()
            .status(),
        410
    );
}

#[tokio::test]
async fn simultaneous_demotions_preserve_an_enabled_super_admin() {
    let base = server().await;
    let root = root(&base).await;
    let other = create(&root, &base, "other", json!([{"roleId":"super_admin"}])).await;
    let list: Value = root
        .get(format!("{base}/admin/admins"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let first = list["admins"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["username"] == "root")
        .unwrap();
    let a = root
        .put(format!(
            "{base}/admin/admins/{}/roles",
            first["id"].as_str().unwrap()
        ))
        .json(&json!({"roles":[]}))
        .send();
    let b = root
        .put(format!(
            "{base}/admin/admins/{}/roles",
            other["id"].as_str().unwrap()
        ))
        .json(&json!({"roles":[]}))
        .send();
    let (a, b) = tokio::join!(a, b);
    let a = a.unwrap().status();
    let b = b.unwrap().status();
    assert!(
        (a == 200 && (b == 409 || b == 403)) || (b == 200 && (a == 409 || a == 403)),
        "{a}, {b}"
    );
}

#[tokio::test]
async fn every_registered_capability_route_rejects_missing_capability_and_non_admin_sessions() {
    use mw_server::{
        admin_router::{Access, registry},
        capabilities::Capability,
    };
    let base = server().await;
    let root = root(&base).await;
    let role: Value = root
        .post(format!("{base}/admin/roles"))
        .json(&json!({"name":"Matrix", "capabilities":[]}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let role_id = role["id"].as_str().unwrap();
    create(&root, &base, "matrix", json!([{"roleId":role_id}])).await;
    let actor = client();
    assert_eq!(
        login(&actor, &base, "matrix", "long test password")
            .await
            .status(),
        200
    );
    confirm(&actor, &base, "long test password").await;
    let anonymous = client();
    let endpoints = registry();
    let mut checked = 0;
    for cap in Capability::ALL {
        let routes: Vec<_> = endpoints
            .iter()
            .filter(|e| e.access == Access::Cap(*cap))
            .collect();
        if routes.is_empty() {
            continue;
        }
        let caps: Vec<_> = Capability::ALL
            .iter()
            .filter(|c| *c != cap)
            .map(|c| c.as_str())
            .collect();
        let response = root
            .put(format!("{base}/admin/roles/{role_id}"))
            .json(&json!({"name":"Matrix","capabilities":caps}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        for route in routes {
            let suffix = route
                .suffix
                .split('/')
                .map(|segment| {
                    if segment.starts_with('{') {
                        "missing"
                    } else {
                        segment
                    }
                })
                .collect::<Vec<_>>()
                .join("/");
            let url = format!("{base}/admin{suffix}");
            let method: reqwest::Method = route.method.as_str().parse().unwrap();
            let response = actor
                .request(method.clone(), &url)
                .json(&json!({}))
                .send()
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                403,
                "{} {} without {}",
                route.method,
                route.suffix,
                cap.as_str()
            );
            for cookie in ["", "mw_session=mailbox-only"] {
                let response = anonymous
                    .request(method.clone(), &url)
                    .header("Cookie", cookie)
                    .json(&json!({}))
                    .send()
                    .await
                    .unwrap();
                assert_eq!(
                    response.status(),
                    401,
                    "{} {} without admin session",
                    route.method,
                    route.suffix
                );
            }
            checked += 1;
        }
    }
    assert_eq!(
        checked,
        endpoints
            .iter()
            .filter(|e| matches!(e.access, Access::Cap(_)))
            .count()
    );
    assert!(checked > 0, "catalogue must contain protected endpoints");
}

#[tokio::test]
async fn domain_roles_filter_lists_and_refuse_cross_domain_writes() {
    let base = server().await;
    let root = root(&base).await;
    for domain in ["allowed.example", "other.example"] {
        assert_eq!(
            root.put(format!("{base}/admin/domains/{domain}"))
                .send()
                .await
                .unwrap()
                .status(),
            204
        );
    }
    create(
        &root,
        &base,
        "scoped",
        json!([{"roleId":"domain_admin","domains":["allowed.example"]}]),
    )
    .await;
    let scoped = client();
    assert_eq!(
        login(&scoped, &base, "scoped", "long test password")
            .await
            .status(),
        200
    );
    confirm(&scoped, &base, "long test password").await;
    let rows: Value = scoped
        .get(format!("{base}/admin/domains"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(rows.to_string().contains("allowed.example"));
    assert!(!rows.to_string().contains("other.example"));
    assert_eq!(
        scoped
            .put(format!("{base}/admin/domains/allowed.example"))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(
        scoped
            .put(format!("{base}/admin/domains/other.example"))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        scoped
            .get(format!("{base}/admin/admins"))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
}
