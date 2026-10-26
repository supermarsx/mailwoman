use base64::Engine as _;
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
        .unwrap()
    });
    format!("http://{address}")
}
async fn setting(store: &mw_store::Store, key: &str, value: Value) {
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
#[tokio::test]
async fn geography_uses_trusted_source_and_stops_before_upstream() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/geoip/GeoIP2-Country-Test.mmdb");
    let owned_database =
        common::test_db::unique_db_path("signin-geo-fixture").with_extension("mmdb");
    std::fs::copy(&fixture, &owned_database).unwrap();
    // This integration executable contains one test; environment is configured
    // before any server/provider is constructed and never changed while serving.
    unsafe {
        std::env::set_var("MW_GEOIP_DB", &owned_database);
        std::env::set_var("MW_TRUSTED_PROXIES", "127.0.0.1");
        std::env::set_var("MW_FORWARDED_MODE", "xff");
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let refused_calls = Arc::new(AtomicUsize::new(0));
    let refused_count = refused_calls.clone();
    let refused_authorization = format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD
            .encode(format!("{}:geography-denied-proof", mw_mock_jmap::USER))
    );
    let upstream = serve(mw_mock_jmap::router().layer(axum::middleware::from_fn(
        move |req: axum::extract::Request, next: axum::middleware::Next| {
            let count = count.clone();
            let refused_count = refused_count.clone();
            let refused_authorization = refused_authorization.clone();
            async move {
                // Existing signed-in accounts may run unrelated JMAP background
                // refreshes. Count the credential-bearing session discovery
                // endpoint used by this login path, not those later API calls.
                if matches!(req.uri().path(), "/.well-known/jmap" | "/jmap/session") {
                    count.fetch_add(1, Ordering::SeqCst);
                }
                if req
                    .headers()
                    .get("authorization")
                    .and_then(|value| value.to_str().ok())
                    == Some(refused_authorization.as_str())
                {
                    refused_count.fetch_add(1, Ordering::SeqCst);
                }
                next.run(req).await
            }
        },
    )))
    .await;
    let db = common::test_db::unique_db_path("signin-geography");
    let key = "12".repeat(32);
    let store = mw_store::Store::open(
        db.to_str().unwrap(),
        mw_store::ServerKey::from_hex(&key).unwrap(),
    )
    .await
    .unwrap();
    setting(&store, "signin.retryCooldownSecs", json!(0)).await;
    setting(&store, "security.signin.geoScope", json!("mailbox")).await;
    setting(&store, "security.signin.geoCountries", json!(["GB"])).await;
    let app = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(key),
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
    .unwrap()
    .0;
    let base = serve(app).await;
    let trusted = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .local_address(IpAddr::V4(Ipv4Addr::LOCALHOST))
        .build()
        .unwrap();
    let untrusted = reqwest::Client::builder()
        .no_proxy()
        .local_address(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)))
        .build()
        .unwrap();
    let body = json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS});
    let denied = untrusted
        .post(format!("{base}/api/login"))
        .header("x-forwarded-for", "81.2.69.160")
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), 403);
    assert_eq!(
        denied.json::<Value>().await.unwrap()["code"],
        "signinGeographyDenied"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let accepted = trusted
        .post(format!("{base}/api/login"))
        .header("x-forwarded-for", "81.2.69.160")
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(accepted.status(), 200);
    assert!(calls.load(Ordering::SeqCst) > 0);
    setting(&store, "security.signin.geoMode", json!("deny")).await;
    let refused_body =
        json!({"username": mw_mock_jmap::USER, "password": "geography-denied-proof"});
    assert_eq!(
        trusted
            .post(format!("{base}/api/login"))
            .header("x-forwarded-for", "81.2.69.160")
            .json(&refused_body)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(refused_calls.load(Ordering::SeqCst), 0);
    setting(&store, "security.signin.geoUnknown", json!("allow")).await;
    assert_eq!(
        untrusted
            .post(format!("{base}/api/login"))
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    // Mailbox-only policy cannot lock out the administrator diagnostic.
    assert_eq!(
        trusted
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    // Exercise the actual administrator validation/CAS route, not just a direct
    // fixture write: country codes intentionally differ from lowercase domains.
    let policy_url = format!("{base}/admin/settings/security.signin.geoCountries");
    let read = trusted.get(&policy_url).send().await.unwrap();
    assert_eq!(read.status(), 200);
    let tag = read.headers()["etag"].to_str().unwrap().to_owned();
    let saved = trusted
        .put(&policy_url)
        .header("if-match", &tag)
        .json(&json!({"globalValue":["GB","PT"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        saved.status(),
        200,
        "valid uppercase country list must be configurable"
    );
    let read = trusted.get(&policy_url).send().await.unwrap();
    let tag = read.headers()["etag"].to_str().unwrap().to_owned();
    assert_eq!(
        store
            .get_setting_policy("security.signin.geoCountries")
            .await
            .unwrap()
            .unwrap()
            .global_value
            .as_deref(),
        Some("[\"GB\",\"PT\"]")
    );
    for invalid in [
        json!(["gb"]),
        json!(["GB", "GB"]),
        json!(["*"]),
        json!(vec!["GB"; 250]),
    ] {
        let refused = trusted
            .put(&policy_url)
            .header("if-match", &tag)
            .json(&json!({"globalValue":invalid}))
            .send()
            .await
            .unwrap();
        assert_eq!(refused.status(), 422);
    }
    let diagnostic = trusted
        .get(format!("{base}/admin/bans/geography"))
        .header("x-forwarded-for", "81.2.69.160")
        .send()
        .await
        .unwrap();
    assert_eq!(diagnostic.headers()["cache-control"], "private, no-store");
    let diagnostic: Value = diagnostic.json().await.unwrap();
    assert_eq!(diagnostic["country"], "GB");
    assert_eq!(diagnostic["administratorDecision"], "off");
    assert_eq!(diagnostic["mailboxDecision"], "deny");
    setting(&store, "security.signin.geoScope", json!("all")).await;
    assert_eq!(
        trusted
            .post(format!("{base}/admin/login"))
            .header("x-forwarded-for", "81.2.69.160")
            .json(&json!({"username":"root","password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        trusted
            .get(format!("{base}/admin/me"))
            .send()
            .await
            .unwrap()
            .status(),
        200,
        "the initial scope does not revoke existing authenticated sessions"
    );
    assert!(
        store
            .auth_source_lock(
                "admin",
                "81.2.69.160".parse().unwrap(),
                chrono::Utc::now().timestamp()
            )
            .await
            .unwrap()
            .is_none(),
        "country refusals are not rejected credentials"
    );
    // Existing mixed mailbox/admin cookies are scoped by the requested surface.
    assert_eq!(
        trusted
            .get(format!("{base}/api/me"))
            .header("x-forwarded-for", "81.2.69.160")
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    setting(&store, "security.signin.geoEnforceSessions", json!(true)).await;
    setting(&store, "security.signin.geoScope", json!("mailbox")).await;
    let blocked = trusted
        .get(format!("{base}/api/me"))
        .header("x-forwarded-for", "81.2.69.160")
        .send()
        .await
        .unwrap();
    assert_eq!(blocked.status(), 403);
    assert_eq!(blocked.headers()["cache-control"], "private, no-store");
    assert_eq!(
        blocked.json::<Value>().await.unwrap()["code"],
        "sessionGeographyDenied"
    );
    assert_eq!(
        trusted
            .get(format!("{base}/admin/me"))
            .header("x-forwarded-for", "81.2.69.160")
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    // Cookieless signed reader authority cannot skip source policy.
    setting(&store, "security.signin.geoUnknown", json!("deny")).await;
    let invalid = untrusted
        .get(format!("{base}/api/rc/p/untrusted-token"))
        .send()
        .await
        .unwrap();
    assert_eq!(invalid.status(), 403);
    assert_ne!(
        invalid.json::<Value>().await.unwrap()["code"],
        "sessionGeographyDenied"
    );
    use hmac::{Hmac, KeyInit, Mac};
    let key = store.derive_key("mw-remote-content-v1");
    let payload=base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json!({"v":1,"k":"p","a":"fixture","s":"fixture","e":"fixture","u":"fixture","x":chrono::Utc::now().timestamp()+60}).to_string());
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(&key).unwrap();
    mac.update(b"mw-rc-token\0");
    mac.update(payload.as_bytes());
    let signed_token = format!(
        "{payload}%2E{}",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
    );
    let signed = untrusted
        .get(format!("{base}/api/rc/p/{signed_token}"))
        .header("x-forwarded-for", "81.2.69.160")
        .send()
        .await
        .unwrap();
    assert_eq!(signed.status(), 403);
    assert_eq!(
        signed.json::<Value>().await.unwrap()["code"],
        "sessionGeographyDenied"
    );
    setting(&store, "security.signin.geoScope", json!("admin")).await;
    assert_eq!(
        trusted
            .get(format!("{base}/api/me"))
            .header("x-forwarded-for", "81.2.69.160")
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        trusted
            .get(format!("{base}/admin/me"))
            .header("x-forwarded-for", "81.2.69.160")
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        trusted
            .get(format!("{base}/healthz"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        trusted
            .get(format!("{base}/api/public-config"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    // Policy failure does not prevent signing out of the affected admin session.
    assert!(
        trusted
            .post(format!("{base}/admin/logout"))
            .json(&json!({}))
            .send()
            .await
            .unwrap()
            .status()
            .is_success()
    );
    setting(&store, "security.signin.geoScope", json!("all")).await;
    std::fs::remove_file(&owned_database).unwrap();
    let unavailable = untrusted
        .post(format!("{base}/api/login"))
        .json(&refused_body)
        .send()
        .await
        .unwrap();
    assert_eq!(unavailable.status(), 503);
    assert_eq!(
        unavailable.json::<Value>().await.unwrap()["code"],
        "signinGeographyUnavailable"
    );
    assert_eq!(
        refused_calls.load(Ordering::SeqCst),
        0,
        "unknown=allow cannot bypass a missing database"
    );
    setting(&store, "security.signin.geoScope", json!("off")).await;
    assert_eq!(
        untrusted
            .post(format!("{base}/api/login"))
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
}
