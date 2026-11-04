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
    // These suites exercise the user-agent budgets; the trust-tier class actions
    // (security.trust.ua*, default block for declared AI crawlers) are covered by
    // t101_trust_geo, so the AI classes stay on the budget path here.
    let budget_path = ["uaAiTraining", "uaAiSearch", "uaAiUser"]
        .map(|class| (format!("security.trust.{class}"), json!("limit")));
    for (key, value) in std::iter::once(("signin.retryCooldownSecs".to_owned(), json!(0)))
        .chain(budget_path)
        .chain(values.iter().map(|(k, v)| ((*k).to_owned(), v.clone())))
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
async fn aggregate_source_country_uses_local_geoip_and_is_returned_in_lists_and_history() {
    use mw_engine::security::geoip_fixture::{Entry, country_mmdb};
    // The GeoIP provider is process-wide. Isolate its environment before any
    // child threads start instead of mutating it beside other parallel tests.
    if std::env::var_os("MW_PROTECTION_COUNTRY_TEST_CHILD").is_none() {
        let file = common::test_db::unique_file_path("protection-country", "country.mmdb");
        std::fs::write(
            &file,
            country_mmdb(&[Entry::v4("127.0.0.48/32", "PT")], 1_790_000_000),
        )
        .unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "aggregate_source_country_uses_local_geoip_and_is_returned_in_lists_and_history",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("MW_PROTECTION_COUNTRY_TEST_CHILD", "1")
            .env("MW_GEOIP_DB", &file)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let (base, store) = fixture(&[
        ("security.adminProbe.enabled", json!(false)),
        ("security.protection.adaptiveMode", json!("enforce")),
    ])
    .await;
    let known = client(48);
    assert_eq!(
        known
            .get(format!("{base}/.git/config"))
            .send()
            .await
            .unwrap()
            .status(),
        429
    );
    assert_eq!(
        client(49)
            .get(format!("{base}/cmd_sco"))
            .send()
            .await
            .unwrap()
            .status(),
        429
    );
    let known_key = store
        .request_protection_subject("client", "127.0.0.48")
        .unwrap();
    let unknown_key = store
        .request_protection_subject("client", "127.0.0.49")
        .unwrap();
    let operator = client(50);
    admin_login(&operator, &base, "root", "hunter2").await;
    let response = operator
        .get(format!("{base}/admin/bans/protection/subjects"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    let rows = body["subjects"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .find(|row| row["subjectKey"] == known_key)
            .unwrap()["snapshot"]["countryCode"],
        "PT"
    );
    assert!(
        rows.iter()
            .find(|row| row["subjectKey"] == unknown_key)
            .unwrap()["snapshot"]["countryCode"]
            .is_null()
    );
    let response = operator
        .get(format!("{base}/admin/bans/protection/events"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    let events = body["events"].as_array().unwrap();
    let event = events
        .iter()
        .find(|row| row["subjectKey"] == known_key)
        .unwrap();
    assert_eq!(event["reason"], "probe-path");
    assert_eq!(event["snapshot"]["countryCode"], "PT");
    assert_eq!(event["snapshot"]["sourceIp"], "127.0.0.48");
    assert_eq!(
        rows.iter()
            .find(|row| row["subjectKey"] == known_key)
            .unwrap()["snapshot"]["sourceIp"],
        "127.0.0.48"
    );
    assert_eq!(
        rows.iter()
            .find(|row| row["subjectKey"] == unknown_key)
            .unwrap()["snapshot"]["sourceIp"],
        "127.0.0.49"
    );
}
#[tokio::test]
async fn scanner_paths_block_the_source_on_the_first_request_and_audit_the_reason() {
    let (base, store) = fixture(&[
        ("security.adminProbe.enabled", json!(false)),
        ("security.protection.adaptiveMode", json!("enforce")),
        ("security.protection.signals", json!([])),
        ("security.protection.blockSecs", json!(120)),
    ])
    .await;
    for (index, path) in [
        "/.env",
        "/backup/.env.production.local",
        "/.%65nv",
        "/assets/.env",
        "/wp-admin/",
        "/wordpress/wp-login.php",
        "/xmlrpc.php",
        "/wp-config.php.bak",
        "/administrator/",
        "/phpMyAdmin/index.php",
        "/core/install.php",
        "/sites/default/settings.php",
        "/.git/config",
        "/.git/HEAD",
        "/.gitconfig",
        "/cmd_sco",
        "/%63md_sco",
        "/nested/cmd_sco/execute",
        "/api/cmd_sco",
        "/admin/.git/config",
    ]
    .iter()
    .enumerate()
    {
        let scanner = client(20 + index as u8);
        let first = scanner
            .get(format!("{base}{path}"))
            .header("x-forwarded-for", "203.0.113.99")
            .send()
            .await
            .unwrap();
        assert_eq!(first.status(), 429, "first request to {path}");
        assert_eq!(first.headers()["retry-after"], "120");
        assert_eq!(
            first.json::<Value>().await.unwrap()["code"],
            "requestProtectionLimited"
        );
        for followup in ["/", "/api/me", "/unrelated-scanner.php"] {
            assert_eq!(
                scanner
                    .get(format!("{base}{followup}"))
                    .send()
                    .await
                    .unwrap()
                    .status(),
                429,
                "{path} must block its source on {followup}"
            );
        }
    }
    // Spoofed forwarding headers never select the subject or poison another IP.
    assert_eq!(
        client(90)
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let rows = store
        .request_protection_rows(chrono::Utc::now().timestamp(), None, 100)
        .await
        .unwrap();
    let rows: Vec<_> = rows
        .iter()
        .filter(|row| {
            row.snapshot.last_signal == Some(mw_store::request_protection_model::Signal::ProbePath)
        })
        .collect();
    assert_eq!(rows.len(), 20);
    assert!(rows.iter().all(|row| row.subject_kind == "client"
        && row.snapshot.stage == 2
        && row.snapshot.rate_count == 0
        && row.snapshot.last_signal
            == Some(mw_store::request_protection_model::Signal::ProbePath)));
    let events = store
        .request_protection_events(chrono::Utc::now().timestamp(), None, 100)
        .await
        .unwrap();
    assert_eq!(events.len(), 20);
    assert!(
        events
            .iter()
            .all(|event| event.reason == "probe-path" && event.action == "block" && event.enforced)
    );
}

#[tokio::test]
async fn probe_blocks_remain_source_scoped_with_valid_sessions_and_preserve_recovery() {
    let (base, store) = fixture(&[
        ("security.adminProbe.enabled", json!(false)),
        ("security.protection.adaptiveMode", json!("enforce")),
        (
            "security.protection.exemptClients",
            json!(["127.0.0.44/32"]),
        ),
    ])
    .await;
    let actor = client(43);
    login(&actor, &base).await;
    assert_eq!(
        actor
            .get(format!("{base}/cmd_sco"))
            .send()
            .await
            .unwrap()
            .status(),
        429
    );
    assert_eq!(
        actor
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        429,
        "a valid cookie cannot bypass the source's probe block"
    );
    assert_eq!(
        actor
            .get(format!("{base}/healthz"))
            .send()
            .await
            .unwrap()
            .status(),
        200
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
    let exempt = client(44);
    assert_ne!(
        exempt
            .get(format!("{base}/.git/config"))
            .send()
            .await
            .unwrap()
            .status(),
        429
    );
    assert_eq!(
        exempt
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let other_device = client(45);
    login(&other_device, &base).await;
    assert_eq!(
        other_device
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let events = store
        .request_protection_events(chrono::Utc::now().timestamp(), None, 100)
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].subject_kind, "client");
}

#[tokio::test]
async fn explicit_probes_respect_modes_and_do_not_treat_normal_paths_or_query_text_as_scans() {
    for mode in ["off", "observe", "enforce"] {
        let (base, store) = fixture(&[
            ("security.adminProbe.enabled", json!(false)),
            ("security.protection.adaptiveMode", json!(mode)),
        ])
        .await;
        let ordinary = client(46);
        for path in [
            "/admin",
            "/login",
            "/user/login",
            "/.gitattributes",
            "/.environment",
            "/cmd_sco.js",
            "/api/files/.env",
            "/api/files/.git/config",
            "/?next=/.env",
        ] {
            assert_ne!(
                ordinary
                    .get(format!("{base}{path}"))
                    .send()
                    .await
                    .unwrap()
                    .status(),
                429,
                "{mode}: {path}"
            );
        }
        assert!(
            store
                .request_protection_events(chrono::Utc::now().timestamp(), None, 100)
                .await
                .unwrap()
                .is_empty()
        );
        let scanner = client(47);
        let status = scanner
            .get(format!("{base}/cmd_sco"))
            .send()
            .await
            .unwrap()
            .status();
        if mode == "enforce" {
            assert_eq!(status, 429);
        } else {
            assert_ne!(status, 429);
        }
        let events = store
            .request_protection_events(chrono::Utc::now().timestamp(), None, 100)
            .await
            .unwrap();
        assert_eq!(events.len(), usize::from(mode != "off"));
        if let Some(event) = events.first() {
            assert_eq!(event.reason, "probe-path");
            assert_eq!(event.enforced, mode == "enforce");
        }
    }
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
        assert_eq!(events[0].snapshot.source_ip.as_deref(), Some("127.0.0.2"));
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
    assert!(
        events["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["snapshot"]["sourceIp"] == "127.0.0.2")
    );
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
        ("security.protection.userAgentBlockBots", json!(false)),
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

#[tokio::test]
async fn bot_user_agents_are_blocked_on_the_first_request_by_default_even_for_files() {
    let (base, store) = fixture(&[
        ("security.protection.blockSecs", json!(120)),
        ("security.protection.userAgentAdaptive", json!(false)),
        ("security.protection.userAgentMaxAction", json!("throttle")),
    ])
    .await;
    for (index, (agent, path)) in [
        ("GPTBot/1.2", "/api/me"),
        ("Googlebot/2.1", "/public-document"),
        ("AhrefsBot/7", "/robots.txt"),
        ("Scrapy/2.0", "/assets/app.js"),
        ("curl/8.0", "/wp-content/missing.png"),
        ("Claude-User/1.0", "/api/me"),
        ("CustomCrawler/1.0", "/api/me"),
    ]
    .iter()
    .enumerate()
    {
        let actor = client(70 + index as u8);
        let first = actor
            .get(format!("{base}{path}"))
            .header("user-agent", *agent)
            .send()
            .await
            .unwrap();
        assert_eq!(first.status(), 429, "{agent}: {path}");
        assert_eq!(first.headers()["retry-after"], "120");
        assert_eq!(
            first.json::<Value>().await.unwrap()["code"],
            "requestProtectionLimited"
        );
        // The user-agent policy rejects bot requests; a changed claim does not
        // turn this category-specific decision into a global source ban.
        assert_eq!(
            actor
                .get(format!("{base}/api/me"))
                .header("user-agent", "Mozilla/5.0 Chrome/140.0 Safari/537.36")
                .send()
                .await
                .unwrap()
                .status(),
            401
        );
        assert_eq!(
            actor
                .get(format!("{base}/healthz"))
                .header("user-agent", *agent)
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
    }
    let events = store
        .request_protection_events(chrono::Utc::now().timestamp(), None, 100)
        .await
        .unwrap();
    assert_eq!(events.len(), 7);
    assert!(events.iter().all(|event| event.reason == "bot-user-agent"
        && event.action == "block"
        && event.enforced
        && event.subject_kind.starts_with("ua-")));
}

#[tokio::test]
async fn first_hit_bot_blocks_respect_modes_categories_and_explicit_exceptions() {
    for mode in ["off", "observe", "enforce"] {
        let (base, store) = fixture(&[
            ("security.protection.userAgentMode", json!(mode)),
            (
                "security.protection.userAgentExceptions",
                json!(["^AllowedBot/"]),
            ),
        ])
        .await;
        let actor = client(80);
        assert_ne!(
            actor
                .get(format!("{base}/api/me"))
                .header("user-agent", "AllowedBot/1")
                .send()
                .await
                .unwrap()
                .status(),
            429
        );
        let response = actor
            .get(format!("{base}/api/me"))
            .header("user-agent", "GPTBot/1")
            .send()
            .await
            .unwrap();
        assert_eq!(
            response.status().as_u16(),
            if mode == "enforce" { 429 } else { 401 }
        );
        let events = store
            .request_protection_events(chrono::Utc::now().timestamp(), None, 100)
            .await
            .unwrap();
        assert_eq!(events.len(), usize::from(mode != "off"));
        if let Some(event) = events.first() {
            assert_eq!(event.reason, "bot-user-agent");
            assert_eq!(event.enforced, mode == "enforce");
        }
    }
    let (base, _) = fixture(&[(
        "security.protection.userAgentCategories",
        json!(["missing", "uncommon"]),
    )])
    .await;
    assert_eq!(
        client(81)
            .get(format!("{base}/api/me"))
            .header("user-agent", "GPTBot/1")
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
}

#[tokio::test]
async fn bot_simulation_uses_default_first_hit_policy_for_headers_and_direct_samples() {
    let (base, _) = fixture(&[]).await;
    let operator = client(82);
    admin_login(&operator, &base, "root", "hunter2").await;
    for body in [
        json!({"subjectKind":"client","userAgent":"GPTBot/1","sample":{"requestsInWindow":1,"signal":"normal"}}),
        json!({"subjectKind":"client","sample":{"requestsInWindow":1,"signal":"bot-user-agent"}}),
    ] {
        let response = operator
            .post(format!("{base}/admin/bans/protection/simulate"))
            .json(&body)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let decision: Value = response.json().await.unwrap();
        assert_eq!(decision["action"], "block", "{decision}");
        assert_eq!(decision["reason"], "bot-user-agent");
        assert_eq!(decision["enforced"], true);
        assert_eq!(decision["signalCounted"], true);
    }
}
