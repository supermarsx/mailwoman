use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
mod common;
async fn serve(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{address}")
}
fn client(ip: u8) -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .local_address(IpAddr::V4(Ipv4Addr::new(127, 0, 0, ip)))
        .build()
        .unwrap()
}
async fn fixture(values: &[(&str, Value)]) -> (String, Store) {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("request-protection-http");
    let key = "5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";
    let store = Store::open(db.to_str().unwrap(), ServerKey::from_hex(key).unwrap())
        .await
        .unwrap();
    for (key, value) in
        std::iter::once(("signin.retryCooldownSecs", json!(0))).chain(values.iter().cloned())
    {
        store
            .put_setting_policy(&SettingPolicyRow {
                key: key.into(),
                global_value: Some(value.to_string()),
                allow_user_override: false,
                global_locked: false,
                updated_by: "fixture".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
    }
    let (router, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(key.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(upstream.clone()),
                jmap_upstreams: Some(vec![upstream]),
                ..Default::default()
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
    (serve(router).await, store)
}
async fn login(client: &reqwest::Client, base: &str) {
    let response = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
}
#[tokio::test]
async fn aggregate_ip_budget_uses_trusted_peer_and_preserves_logout_and_health() {
    let (base, _) = fixture(&[
        ("security.protection.rateMode", json!("enforce")),
        ("security.protection.clientRequests", json!(2)),
        ("security.protection.accountRequests", json!(0)),
        ("security.protection.adminRequests", json!(0)),
        ("security.protection.rateWindowSecs", json!(3600)),
    ])
    .await;
    let first = client(2);
    for _ in 0..2 {
        assert_eq!(
            first
                .get(format!("{base}/api/me"))
                .send()
                .await
                .unwrap()
                .status(),
            401
        );
    }
    let limited = first
        .get(format!("{base}/api/me"))
        .header("x-forwarded-for", "203.0.113.1")
        .send()
        .await
        .unwrap();
    assert_eq!(limited.status(), 429);
    assert_eq!(
        first
            .get(format!("{base}/admin/bans/protection"))
            .send()
            .await
            .unwrap()
            .status(),
        429,
        "an anonymous URL choice is not an operator exemption"
    );
    assert!(limited.headers().contains_key("retry-after"));
    assert_eq!(
        limited.json::<Value>().await.unwrap()["code"],
        "requestProtectionLimited"
    );
    assert_eq!(
        client(3)
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        first
            .get(format!("{base}/healthz"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_ne!(
        first
            .post(format!("{base}/api/logout"))
            .send()
            .await
            .unwrap()
            .status(),
        429
    );
}
#[tokio::test]
async fn verified_account_budget_is_shared_across_sources_without_double_auth_charge() {
    let (base, _) = fixture(&[
        ("security.protection.rateMode", json!("enforce")),
        ("security.protection.clientRequests", json!(0)),
        ("security.protection.accountRequests", json!(2)),
        ("security.protection.adminRequests", json!(0)),
        ("security.protection.rateWindowSecs", json!(3600)),
    ])
    .await;
    let first = client(2);
    let second = client(3);
    login(&first, &base).await;
    login(&second, &base).await;
    assert_eq!(
        first
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        second
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let limited = second
        .get(format!("{base}/api/me"))
        .header("x-mailwoman-account", "untrusted-other")
        .send()
        .await
        .unwrap();
    assert_eq!(limited.status(), 429);
    assert_eq!(
        limited.json::<Value>().await.unwrap()["code"],
        "requestProtectionLimited"
    );
}
#[tokio::test]
async fn typed_json_anomalies_enforce_but_ordinary_not_found_and_observe_mode_do_not() {
    for mode in ["enforce", "observe"] {
        let (base, store) = fixture(&[
            ("security.protection.adaptiveMode", json!(mode)),
            // t97: a signed-in caller is counted on its account, not its address.
            ("security.protection.adaptiveClients", json!(false)),
            ("security.protection.adaptiveAdmins", json!(false)),
            ("security.protection.shortAbsoluteAnomalies", json!(2)),
            ("security.protection.shortRatioPercent", json!(0)),
            ("security.protection.longAbsoluteAnomalies", json!(0)),
            ("security.protection.longRatioPercent", json!(0)),
            ("security.protection.throttleSecs", json!(60)),
        ])
        .await;
        let caller = client(2);
        login(&caller, &base).await;
        for _ in 0..3 {
            let response = caller
                .get(format!("{base}/api/unknown-protection-fixture"))
                .send()
                .await
                .unwrap();
            assert_ne!(response.status(), 429);
        }
        for _ in 0..2 {
            let response = caller
                .post(format!("{base}/jmap/api"))
                .header("content-type", "application/json")
                .body("{not-json")
                .send()
                .await
                .unwrap();
            assert_eq!(response.status(), 400);
        }
        let response = caller.get(format!("{base}/api/me")).send().await.unwrap();
        assert_eq!(
            response.status().as_u16(),
            if mode == "enforce" { 429 } else { 200 }
        );
        let events = store
            .request_protection_events(chrono::Utc::now().timestamp(), None, 100)
            .await
            .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].reason, "short-absolute");
        assert_eq!(events[0].snapshot.short.anomalous, 2);
        assert_eq!(events[0].enforced, mode == "enforce");
    }
}

async fn admin_login(caller: &reqwest::Client, base: &str, user: &str, password: &str) {
    let response = caller
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":user,"password":password}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}
#[tokio::test]
async fn protection_history_is_scoped_private_and_clear_requires_fresh_proof_and_revision() {
    let (base, _) = fixture(&[
        ("security.protection.adaptiveMode", json!("observe")),
        // t97: the signed-in subject is counted on its account, not its address.
        ("security.protection.adaptiveClients", json!(false)),
        ("security.protection.adaptiveAdmins", json!(false)),
        ("security.protection.shortAbsoluteAnomalies", json!(1)),
    ])
    .await;
    let subject = client(2);
    login(&subject, &base).await;
    assert_eq!(
        subject
            .post(format!("{base}/jmap/api"))
            .body("{malformed")
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    let anonymous = client(3);
    assert_eq!(
        anonymous
            .get(format!("{base}/admin/bans/protection"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let admin = client(2);
    admin_login(&admin, &base, "root", "hunter2").await;
    let rows = admin
        .get(format!("{base}/admin/bans/protection/subjects"))
        .send()
        .await
        .unwrap();
    assert_eq!(rows.status(), 200);
    assert_eq!(rows.headers()["cache-control"], "private, no-store");
    let rows: Value = rows.json().await.unwrap();
    let row = rows["subjects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["snapshot"]["lastSignal"] == "malformed")
        .unwrap();
    let url = format!(
        "{base}/admin/bans/protection/subjects/{}",
        row["subjectKey"].as_str().unwrap()
    );
    let revision = row["revision"].as_str().unwrap();
    // The browser refreshes this endpoint to recover its CSRF token before
    // retrying a protected mutation. It must not invalidate the displayed CAS.
    assert_eq!(
        admin
            .get(format!("{base}/admin/session"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );

    assert_eq!(
        admin
            .delete(&url)
            .header("if-match", format!("\"{revision}\""))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        admin
            .post(format!("{base}/admin/step-up"))
            .json(&json!({"password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(admin.delete(&url).send().await.unwrap().status(), 428);
    assert_eq!(
        admin
            .delete(&url)
            .header("if-match", format!("\"{}\"", "0".repeat(32)))
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    assert_eq!(
        admin
            .delete(&url)
            .header("if-match", format!("\"{revision}\""))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    let events = admin
        .get(format!("{base}/admin/bans/protection/events"))
        .send()
        .await
        .unwrap();
    assert_eq!(events.status(), 200);
    assert_eq!(events.headers()["cache-control"], "private, no-store");
    let events: Value = events.json().await.unwrap();
    assert!(
        events["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["action"] == "cleared")
    );
    assert!(!events.to_string().contains("127.0.0.2"));
    for path in [
        "subjects?limit=0",
        "events?limit=101",
        "events?after=invalid",
        "subjects?after=raw-address",
    ] {
        assert_eq!(
            admin
                .get(format!("{base}/admin/bans/protection/{path}"))
                .send()
                .await
                .unwrap()
                .status(),
            422
        );
    }
    let role: Value = admin
        .post(format!("{base}/admin/roles"))
        .json(&json!({"name":"Scoped protection","capabilities":["bans.read","bans.write"]}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    for (name, roles) in [
        ("limited-protection", json!([])),
        (
            "scoped-protection",
            json!([{"roleId":role["id"],"domains":["example.org"]}]),
        ),
    ] {
        assert_eq!(
            admin
                .post(format!("{base}/admin/admins"))
                .json(&json!({"username":name,"password":"long test password","roles":roles}))
                .send()
                .await
                .unwrap()
                .status(),
            201
        );
        let actor = client(5);
        admin_login(&actor, &base, name, "long test password").await;
        for path in ["", "/subjects", "/events"] {
            assert_eq!(
                actor
                    .get(format!("{base}/admin/bans/protection{path}"))
                    .send()
                    .await
                    .unwrap()
                    .status(),
                403
            );
        }
        assert_eq!(
            actor
                .delete(&url)
                .header("if-match", format!("\"{revision}\""))
                .send()
                .await
                .unwrap()
                .status(),
            403
        );
    }
}
#[tokio::test]
async fn real_settings_write_invalidates_off_cache_before_next_request() {
    // t97: a signed-in administrator is charged to its own budget, not its address.
    let (base, _) = fixture(&[
        ("security.protection.clientRequests", json!(0)),
        ("security.protection.accountRequests", json!(0)),
        ("security.protection.adminRequests", json!(1)),
        ("security.protection.rateWindowSecs", json!(3600)),
    ])
    .await;
    let admin = client(2);
    admin_login(&admin, &base, "root", "hunter2").await;
    let url = format!("{base}/admin/settings/security.protection.rateMode");
    let current = admin.get(&url).send().await.unwrap();
    assert_eq!(current.status(), 200);
    let etag = current.headers()["etag"].clone();
    let saved = admin
        .put(&url)
        .header("if-match", etag)
        .json(&json!({"globalValue":"enforce"}))
        .send()
        .await
        .unwrap();
    assert_eq!(saved.status(), 200, "{}", saved.text().await.unwrap());
    assert_eq!(
        admin
            .get(format!("{base}/admin/me"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        admin
            .get(format!("{base}/admin/me"))
            .send()
            .await
            .unwrap()
            .status(),
        429
    );
}

#[tokio::test]
async fn user_agent_claims_are_lax_independent_and_do_not_trust_spoofed_forwarding() {
    let (base, _) = fixture(&[
        ("security.protection.userAgentWeakRequests", json!(2)),
        ("security.protection.userAgentCrawlerRequests", json!(1)),
        ("security.protection.userAgentWindowSecs", json!(3600)),
    ])
    .await;
    let weak = client(14);
    for expected in [404, 404, 429] {
        assert_eq!(
            weak.get(format!("{base}/public-document"))
                .header("user-agent", "novel-reader/1")
                .header("x-forwarded-for", format!("192.0.2.{expected}"))
                .send()
                .await
                .unwrap()
                .status()
                .as_u16(),
            expected
        );
    }
    let crawler = client(15);
    for expected in [404, 429] {
        assert_eq!(
            crawler
                .get(format!("{base}/public-document"))
                .header("user-agent", "GPTBot/1.2")
                .send()
                .await
                .unwrap()
                .status()
                .as_u16(),
            expected
        );
    }
    // A normal browser is not assigned a UA budget even from the same source.
    for _ in 0..4 {
        assert_eq!(
            weak.get(format!("{base}/api/me"))
                .header("user-agent", "Mozilla/5.0 Chrome/140.0 Safari/537.36")
                .send()
                .await
                .unwrap()
                .status(),
            401
        );
    }
}

#[tokio::test]
async fn user_agent_exception_cannot_disable_legacy_limits_and_observe_never_refuses() {
    let (base, _) = fixture(&[
        ("security.protection.rateMode", json!("enforce")),
        ("security.protection.clientRequests", json!(1)),
        (
            "security.protection.userAgentExceptions",
            json!(["^trusted-client/"]),
        ),
    ])
    .await;
    let actor = client(16);
    for expected in [401, 429] {
        assert_eq!(
            actor
                .get(format!("{base}/api/me"))
                .header("user-agent", "trusted-client/1")
                .send()
                .await
                .unwrap()
                .status()
                .as_u16(),
            expected
        );
    }
    let (base, _) = fixture(&[
        ("security.protection.userAgentMode", json!("observe")),
        ("security.protection.userAgentWeakRequests", json!(1)),
    ])
    .await;
    for _ in 0..3 {
        assert_eq!(
            actor
                .get(format!("{base}/api/me"))
                .send()
                .await
                .unwrap()
                .status(),
            401
        );
    }
}

#[tokio::test]
async fn user_agent_scope_excludes_verified_sessions_unless_all_is_selected() {
    for scope in ["anonymous", "all"] {
        let (base, _) = fixture(&[
            ("security.protection.userAgentScope", json!(scope)),
            ("security.protection.userAgentWeakRequests", json!(2)),
            ("security.protection.userAgentWindowSecs", json!(3600)),
        ])
        .await;
        let actor = client(17);
        login(&actor, &base).await;
        assert_eq!(
            actor
                .get(format!("{base}/api/me"))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
        assert_eq!(
            actor
                .get(format!("{base}/api/me"))
                .send()
                .await
                .unwrap()
                .status()
                .as_u16(),
            if scope == "all" { 429 } else { 200 }
        );
        assert_ne!(
            actor
                .post(format!("{base}/api/logout"))
                .send()
                .await
                .unwrap()
                .status(),
            429
        );
    }
}
