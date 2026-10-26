use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
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
fn client(last: u8) -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .local_address(IpAddr::V4(Ipv4Addr::new(127, 0, 0, last)))
        .build()
        .unwrap()
}
async fn post(c: &reqwest::Client, base: &str, path: &str, body: Value) -> reqwest::Response {
    c.post(format!("{base}{path}"))
        .json(&body)
        .send()
        .await
        .unwrap()
}
#[tokio::test]
async fn temporary_locks_are_durable_surface_specific_pre_upstream_and_proof_guarded() {
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    let upstream = serve(mw_mock_jmap::router().layer(axum::middleware::from_fn(
        move |request: axum::extract::Request, next: axum::middleware::Next| {
            let seen = seen.clone();
            async move {
                seen.fetch_add(1, Ordering::SeqCst);
                next.run(request).await
            }
        },
    )))
    .await;
    let db = common::test_db::unique_db_path("signin-guard");
    let key = "5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";
    let store = mw_store::Store::open(
        db.to_str().unwrap(),
        mw_store::ServerKey::from_hex(key).unwrap(),
    )
    .await
    .unwrap();
    for (key, value) in [
        ("signin.retryCooldownSecs", 0),
        ("security.signin.adminFailureLimit", 2),
        ("security.signin.mailboxFailureLimit", 2),
    ] {
        store
            .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
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
    let base = serve(router).await;
    let admin = client(2);
    let source = client(1);
    assert_eq!(
        post(
            &admin,
            &base,
            "/admin/login",
            json!({"username":"root","password":"hunter2"})
        )
        .await
        .status(),
        200
    );
    assert_eq!(
        source
            .get(format!("{base}/admin/bans/automatic"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    // Invalid/expired ceremonies are not evidence of a rejected credential.
    for _ in 0..3 {
        for (path, body) in [
            (
                "/admin/login/passkey/finish",
                json!({"pendingToken":"expired","credentialId":"missing","clientDataJson":"","authenticatorData":"","signature":""}),
            ),
            (
                "/admin/login/recovery",
                json!({"pendingToken":"expired","code":"missing"}),
            ),
        ] {
            assert_eq!(post(&source, &base, path, body).await.status(), 401);
        }
    }
    assert!(
        store
            .auth_source_lock(
                "admin",
                "127.0.0.1".parse().unwrap(),
                chrono::Utc::now().timestamp()
            )
            .await
            .unwrap()
            .is_none()
    );
    for _ in 0..2 {
        assert_eq!(
            post(
                &source,
                &base,
                "/admin/login",
                json!({"username":"root","password":"wrong"})
            )
            .await
            .status(),
            401
        );
    }
    let blocked = source
        .post(format!("{base}/admin/login"))
        .header("x-forwarded-for", "203.0.113.55")
        .json(&json!({"username":"root","password":"hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(blocked.status(), 429);
    assert!(
        blocked.headers()["retry-after"]
            .to_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
            > 0
    );
    assert_eq!(blocked.headers()["cache-control"], "no-store");
    assert_eq!(
        blocked.json::<Value>().await.unwrap()["code"],
        "signinLocked"
    );
    for (path, body) in [
        (
            "/admin/login/totp",
            json!({"pendingToken":"missing","code":"000000"}),
        ),
        (
            "/admin/login/recovery",
            json!({"pendingToken":"missing","code":"missing"}),
        ),
        (
            "/admin/login/passkey/finish",
            json!({"pendingToken":"missing","credentialId":"missing","clientDataJson":"","authenticatorData":"","signature":""}),
        ),
    ] {
        assert_eq!(
            post(&source, &base, path, body).await.status(),
            429,
            "{path}"
        );
    }
    let lock = store
        .auth_source_lock(
            "admin",
            "127.0.0.1".parse().unwrap(),
            chrono::Utc::now().timestamp(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(lock.failures, 2);
    // A mailbox login does not inherit or reset the administrator lock.
    let good = json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS});
    assert_eq!(
        post(&source, &base, "/api/login", good.clone())
            .await
            .status(),
        200
    );
    assert!(
        store
            .auth_source_lock(
                "admin",
                "127.0.0.1".parse().unwrap(),
                chrono::Utc::now().timestamp()
            )
            .await
            .unwrap()
            .is_some()
    );
    for _ in 0..2 {
        assert_eq!(
            post(
                &source,
                &base,
                "/api/login",
                json!({"username":mw_mock_jmap::USER,"password":"wrong"})
            )
            .await
            .status(),
            401
        );
    }
    let before = calls.load(Ordering::SeqCst);
    assert_eq!(
        post(&source, &base, "/api/login", good.clone())
            .await
            .status(),
        429
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        before,
        "locked credentials never reach upstream"
    );
    for path in [
        "/api/sso/missing/begin",
        "/api/sso/missing/callback?state=unused",
    ] {
        assert_eq!(
            source
                .get(format!("{base}{path}"))
                .send()
                .await
                .unwrap()
                .status(),
            429,
            "SSO must apply the same source lock before provider resolution"
        );
    }
    let listed = admin
        .get(format!("{base}/admin/bans/automatic"))
        .send()
        .await
        .unwrap();
    assert_eq!(listed.headers()["cache-control"], "private, no-store");
    let listed: Value = listed.json().await.unwrap();
    assert_eq!(listed["locks"].as_array().unwrap().len(), 2);
    let mailbox = listed["locks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["surface"] == "webmail")
        .unwrap();
    let revision = mailbox["revision"].as_str().unwrap();
    let url = format!("{base}/admin/bans/automatic/webmail/127.0.0.1");
    assert_eq!(
        admin
            .delete(&url)
            .header("if-match", format!("\"{revision}\""))
            .send()
            .await
            .unwrap()
            .status(),
        403,
        "fresh proof required"
    );
    assert_eq!(
        post(
            &admin,
            &base,
            "/admin/step-up",
            json!({"password":"hunter2"})
        )
        .await
        .status(),
        200
    );
    assert_eq!(admin.delete(&url).send().await.unwrap().status(), 428);
    assert_eq!(
        admin
            .delete(&url)
            .header("if-match", "\"stale\"")
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
    assert_eq!(
        admin
            .delete(&url)
            .header("if-match", format!("\"{revision}\""))
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    assert_eq!(post(&source, &base, "/api/login", good).await.status(), 200);
    let after = store
        .auth_source_lock(
            "admin",
            "127.0.0.1".parse().unwrap(),
            chrono::Utc::now().timestamp(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(after.locked_until, lock.locked_until);
    assert_eq!(
        after.revision, lock.revision,
        "blocked retries do not mutate the existing lock"
    );
    let role: Value = post(
        &admin,
        &base,
        "/admin/roles",
        json!({
            "name":"Scoped lock operator", "capabilities":["bans.read","bans.write"]
        }),
    )
    .await
    .json()
    .await
    .unwrap();
    for (username, roles) in [
        (
            "scoped-locks",
            json!([{"roleId":role["id"],"domains":["example.org"]}]),
        ),
        ("no-lock-grants", json!([])),
    ] {
        assert_eq!(
            post(
                &admin,
                &base,
                "/admin/admins",
                json!({
                    "username":username,"password":"long test password","roles":roles
                })
            )
            .await
            .status(),
            201
        );
        let actor = client(3);
        assert_eq!(
            post(
                &actor,
                &base,
                "/admin/login",
                json!({"username":username,"password":"long test password"})
            )
            .await
            .status(),
            200
        );
        assert_eq!(
            actor
                .get(format!("{base}/admin/bans/automatic"))
                .send()
                .await
                .unwrap()
                .status(),
            403
        );
        assert_eq!(
            actor
                .delete(format!("{base}/admin/bans/automatic/admin/127.0.0.1"))
                .header("if-match", format!("\"{}\"", lock.revision))
                .send()
                .await
                .unwrap()
                .status(),
            403
        );
    }
}
