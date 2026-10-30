//! t97: the security centre Overview (`GET {console}/security/dashboard`) and the
//! Adaptive protection editor (`GET|PUT {console}/bans/protection/policy`,
//! `POST {console}/bans/protection/simulate`): real data, per-tile capability
//! filtering, address masking, validation, step-up, audit and role restriction.
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
async fn fixture(name: &str) -> (String, Store) {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path(name);
    let key = "6b69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";
    let store = Store::open(db.to_str().unwrap(), ServerKey::from_hex(key).unwrap())
        .await
        .unwrap();
    store
        .put_setting_policy(&SettingPolicyRow {
            key: "signin.retryCooldownSecs".into(),
            global_value: Some("0".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "fixture".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
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
async fn admin_login(caller: &reqwest::Client, base: &str, user: &str, password: &str) {
    let response = caller
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":user,"password":password}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{user}");
}
/// An admin holding exactly `caps` (for every domain, or for `domains` only).
async fn restricted(
    root: &reqwest::Client,
    base: &str,
    name: &str,
    caps: &[&str],
    domains: Option<&[&str]>,
    ip: u8,
) -> reqwest::Client {
    // Role changes need a fresh step-up.
    let proof = root
        .post(format!("{base}/admin/step-up"))
        .json(&json!({"password":"hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(proof.status(), 200);
    let roles = if caps.is_empty() {
        json!([])
    } else {
        let role: Value = root
            .post(format!("{base}/admin/roles"))
            .json(&json!({"name":format!("{name} role"),"capabilities":caps}))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        match domains {
            Some(d) => json!([{"roleId":role["id"],"domains":d}]),
            None => json!([{"roleId":role["id"]}]),
        }
    };
    let created = root
        .post(format!("{base}/admin/admins"))
        .json(&json!({"username":name,"password":"long test password","roles":roles}))
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), 201, "{}", created.text().await.unwrap());
    let actor = client(ip);
    admin_login(&actor, base, name, "long test password").await;
    actor
}
async fn get(caller: &reqwest::Client, url: &str) -> (u16, Value) {
    let r = caller.get(url).send().await.unwrap();
    let status = r.status().as_u16();
    (status, r.json().await.unwrap_or(Value::Null))
}

#[tokio::test]
async fn dashboard_reports_real_data_per_capability_and_masks_sources() {
    let (base, _store) = fixture("t97-dashboard").await;
    let url = format!("{base}/admin/security/dashboard");
    assert_eq!(client(9).get(&url).send().await.unwrap().status(), 401);
    // Seed failed mailbox sign-ins from 127.0.0.2.
    let attacker = client(2);
    for _ in 0..4 {
        let r = attacker
            .post(format!("{base}/api/login"))
            .json(&json!({"username":mw_mock_jmap::USER,"password":"wrong password"}))
            .send()
            .await
            .unwrap();
        assert!(r.status().is_client_error(), "{}", r.status());
    }
    let root = client(3);
    admin_login(&root, &base, "root", "hunter2").await;
    let response = root.get(&url).send().await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "private, no-store");
    let body: Value = response.json().await.unwrap();
    for tile in [
        "factors",
        "signins",
        "protection",
        "admins",
        "audit",
        "posture",
    ] {
        assert!(body[tile].is_object(), "{tile} missing: {body}");
    }
    let signins = &body["signins"];
    assert!(signins["failed24h"].as_i64().unwrap() >= 4, "{signins}");
    assert_eq!(signins["hourly"].as_array().unwrap().len(), 24);
    assert_eq!(signins["daily"].as_array().unwrap().len(), 7);
    let hourly_total: i64 = signins["hourly"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["failed"].as_i64().unwrap())
        .sum();
    assert_eq!(hourly_total, signins["failed24h"].as_i64().unwrap());
    assert_eq!(signins["topSources"][0]["source"], "127.0.0.2");
    assert_eq!(signins["sourcesMasked"], false);
    assert!(body["admins"]["activeSessions"].as_i64().unwrap() >= 1);
    assert_eq!(body["admins"]["recentSignins"][0]["username"], "root");
    assert_eq!(body["posture"]["consoleDefaultPath"], true);
    assert!(body["posture"]["checks"].as_i64().unwrap() > 5);
    assert!(body["factors"]["knownAccounts"].is_number());
    let text = body.to_string();
    assert!(!text.contains("hunter2") && !text.contains("wrong password"));

    // diagnostics.read only: the posture tile, without the console-path check.
    let diag = restricted(&root, &base, "diag-only", &["diagnostics.read"], None, 4).await;
    let (status, body) = get(&diag, &url).await;
    assert_eq!(status, 200);
    assert!(body["posture"].is_object());
    assert!(body["posture"].get("consoleDefaultPath").is_none());
    for tile in ["factors", "signins", "protection", "admins", "audit"] {
        assert!(
            body.get(tile).is_none(),
            "{tile} leaked to diagnostics.read"
        );
    }

    // bans.read only: sign-ins and protection, sources masked to the network.
    let bans = restricted(&root, &base, "bans-only", &["bans.read"], None, 5).await;
    let (status, body) = get(&bans, &url).await;
    assert_eq!(status, 200);
    assert_eq!(body["signins"]["sourcesMasked"], true);
    assert_eq!(body["signins"]["topSources"][0]["source"], "127.0.0.0/24");
    assert!(body["protection"].is_object());
    assert!(body.get("factors").is_none() && body.get("admins").is_none());
    assert!(!body.to_string().contains("127.0.0.2"));

    // No tile capability, or one held for a single domain only: refused.
    let none = restricted(&root, &base, "no-caps", &[], None, 6).await;
    assert_eq!(get(&none, &url).await.0, 403);
    let scoped = restricted(
        &root,
        &base,
        "scoped-bans",
        &["bans.read"],
        Some(&["example.org"]),
        7,
    )
    .await;
    assert_eq!(get(&scoped, &url).await.0, 403);
}

#[tokio::test]
async fn protection_policy_edits_are_validated_stepped_up_audited_and_restricted() {
    let (base, _store) = fixture("t97-protection-policy").await;
    let root = client(3);
    admin_login(&root, &base, "root", "hunter2").await;
    let url = format!("{base}/admin/bans/protection/policy");
    let (status, policy) = get(&root, &url).await;
    assert_eq!(status, 200);
    let rules = policy["rules"].as_array().unwrap();
    assert!(rules.len() >= 25, "{}", rules.len());
    assert!(rules.iter().all(|r| {
        r["key"]
            .as_str()
            .unwrap()
            .starts_with("security.protection.")
    }));
    assert!(
        rules
            .iter()
            .any(|r| r["key"] == "security.protection.exemptClients")
    );
    assert_eq!(policy["canEdit"], true);
    assert_eq!(policy["domainScopes"], false);
    for preset in ["relaxed", "balanced", "strict"] {
        assert!(policy["presets"][preset].is_object());
    }
    let etag = |key: &str| -> String {
        rules.iter().find(|r| r["key"] == key).unwrap()["etag"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let put = |body: Value| root.put(&url).json(&body).send();
    // Registry validation first: out of range, foreign key, unusable whole policy.
    for (body, code) in [
        (
            json!({"changes":[{"key":"security.protection.clientRequests","value":-1}]}),
            422,
        ),
        (
            json!({"changes":[{"key":"twofa.required","value":true}]}),
            422,
        ),
        (
            json!({"changes":[{"key":"security.protection.exemptClients","value":["not an address"]}]}),
            422,
        ),
        (json!({"changes":[]}), 422),
        (
            json!({"changes":[{"key":"security.protection.blockSecs","value":60}],"preset":"lenient"}),
            422,
        ),
    ] {
        let r = put(body.clone()).await.unwrap();
        assert_eq!(
            r.status().as_u16(),
            code,
            "{body}: {}",
            r.text().await.unwrap()
        );
    }
    // A valid change needs a fresh step-up.
    let change = json!({"changes":[{"key":"security.protection.adaptiveMode","value":"enforce","etag":etag("security.protection.adaptiveMode")},{"key":"security.protection.blockSecs","value":900}],"preset":"strict"});
    let refused = put(change.clone()).await.unwrap();
    assert_eq!(refused.status(), 403);
    assert_eq!(
        refused.json::<Value>().await.unwrap()["code"],
        "step_up_required"
    );
    assert_eq!(
        root.post(format!("{base}/admin/step-up"))
            .json(&json!({"password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let saved = put(change).await.unwrap();
    assert_eq!(saved.status(), 200);
    let saved: Value = saved.json().await.unwrap();
    assert_eq!(saved["rules"][0]["effective"]["value"], "enforce");
    assert_eq!(saved["rules"][1]["effective"]["value"], 900);
    // The runtime sees it (status reads the same cache the guard uses).
    let (_, status) = get(&root, &format!("{base}/admin/bans/protection")).await;
    assert_eq!(status["policy"]["adaptiveMode"], "enforce");
    // A stale ETag is refused and changes nothing.
    let stale = put(json!({"changes":[{"key":"security.protection.adaptiveMode","value":"observe","etag":etag("security.protection.adaptiveMode")}]})).await.unwrap();
    assert_eq!(stale.status(), 412);
    // Reset to default.
    let reset: Value =
        put(json!({"changes":[{"key":"security.protection.blockSecs","value":null}]}))
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
    assert_eq!(reset["rules"][0]["effective"]["value"], 300);
    // Audited per key and as one security event.
    let (_, audit) = get(&root, &format!("{base}/admin/audit?limit=100")).await;
    let actions: Vec<&str> = audit
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r["action"].as_str())
        .collect();
    assert!(
        actions.contains(&"protection-policy-changed"),
        "{actions:?}"
    );
    assert!(actions.contains(&"setting-changed"));
    // The dashboard lists it among recent security events.
    let (_, dash) = get(&root, &format!("{base}/admin/security/dashboard")).await;
    assert!(
        dash["audit"]["recent"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["action"] == "protection-policy-changed")
    );

    // bans.read: may read (without exemption lists) but not write.
    let reader = restricted(&root, &base, "protect-reader", &["bans.read"], None, 4).await;
    let (status, view) = get(&reader, &url).await;
    assert_eq!(status, 200);
    assert_eq!(view["canEdit"], false);
    assert!(!view["rules"].as_array().unwrap().iter().any(|r| {
        r["key"]
            .as_str()
            .unwrap()
            .starts_with("security.protection.exempt")
    }));
    assert_eq!(
        reader
            .put(&url)
            .json(&json!({"changes":[{"key":"security.protection.blockSecs","value":60}]}))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    let nobody = restricted(&root, &base, "protect-nobody", &[], None, 5).await;
    assert_eq!(get(&nobody, &url).await.0, 403);
}

#[tokio::test]
async fn simulator_explains_the_decision_without_moving_counters() {
    let (base, _store) = fixture("t97-protection-simulate").await;
    let root = client(3);
    admin_login(&root, &base, "root", "hunter2").await;
    let url = format!("{base}/admin/bans/protection/simulate");
    let sim = |body: Value| root.post(&url).json(&body).send();
    let sample = json!({"requestsInWindow":1,"short":{"normal":0,"anomalous":25},"long":{"normal":0,"anomalous":0},"signal":"malformed"});
    // Defaults: both protections off.
    let off: Value = sim(json!({"subjectKind":"client","sample":sample}))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        (off["action"].as_str(), off["off"].as_bool()),
        (Some("allow"), Some(true))
    );
    // A draft turns adaptive protection on: the short absolute rule throttles.
    let on: Value = sim(json!({"subjectKind":"client","subjectValue":"198.51.100.7","sample":sample,"draft":{"security.protection.adaptiveMode":"enforce"}}))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(on["action"], "throttle", "{on}");
    assert_eq!(on["reason"], "short-absolute");
    assert_eq!(on["enforced"], true);
    assert_eq!(on["durationSecs"], 3);
    assert!(
        on["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["rule"] == "short-absolute" && r["fired"] == true)
    );
    // A second crossing before the record is clean blocks.
    let mut again = sample.clone();
    again["previouslyThrottled"] = json!(true);
    let blocked: Value = sim(json!({"subjectKind":"client","sample":again,"draft":{"security.protection.adaptiveMode":"enforce"}}))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        (blocked["action"].as_str(), blocked["durationSecs"].as_u64()),
        (Some("block"), Some(300))
    );
    // An exempt network is never counted per client.
    let exempt: Value = sim(json!({"subjectKind":"client","subjectValue":"192.0.2.7","sample":sample,"draft":{"security.protection.adaptiveMode":"enforce","security.protection.exemptClients":["192.0.2.0/24"]}}))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(exempt["exempt"], true);
    // Input validation.
    for body in [
        json!({"subjectKind":"robot","sample":sample}),
        json!({"subjectKind":"client","subjectValue":"not-an-ip","sample":sample}),
        json!({"subjectKind":"client","sample":sample,"draft":{"twofa.required":true}}),
        json!({"subjectKind":"client","sample":sample,"draft":{"security.protection.throttleSecs":0}}),
    ] {
        assert_eq!(sim(body.clone()).await.unwrap().status(), 422, "{body}");
    }
    // Nothing was stored: no subject state, the stored mode is still off.
    let (_, subjects) = get(&root, &format!("{base}/admin/bans/protection/subjects")).await;
    assert!(subjects["subjects"].as_array().unwrap().is_empty());
    let (_, status) = get(&root, &format!("{base}/admin/bans/protection")).await;
    assert_eq!(status["policy"]["adaptiveMode"], "off");
    // Role restriction.
    let nobody = restricted(&root, &base, "sim-nobody", &["diagnostics.read"], None, 4).await;
    assert_eq!(
        nobody
            .post(&url)
            .json(&json!({"subjectKind":"client","sample":sample}))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
}
