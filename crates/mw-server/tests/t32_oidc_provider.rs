use base64::Engine as _;
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{ServerKey, Store, oidc_provider::OidcClientRow};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::net::SocketAddr;
mod common;
async fn serve(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap()
    });
    format!("http://{addr}")
}
#[tokio::test]
async fn issuer_real_login_pkce_consent_replay_and_browser_binding() {
    let db = common::test_db::unique_db_path("oidc-http");
    let key = ServerKey::generate();
    let keyhex = key.to_hex();
    let store = Store::open(db.to_str().unwrap(), key).await.unwrap();
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "signin.retryCooldownSecs".into(),
            global_value: Some("0".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "fixture".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    let issuer = "https://issuer.example/oidc";
    store
        .put_oidc_provider_config(issuer, true, None, 1)
        .await
        .unwrap();
    let (pem, jwk) = mw_sso::issuer::generate_rs256_key("http-key").unwrap();
    store
        .rotate_oidc_signing_key("http-key", issuer, pem.as_bytes(), &jwk, 1)
        .await
        .unwrap();
    store.create_oidc_client(&OidcClientRow{client_id:"rp".into(),config:json!({"redirectUris":["https://rp.example/callback"],"postLogoutRedirectUris":["https://rp.example/logout"],"scopes":["openid","email","offline_access"],"authMethod":"none"}),enabled:true,revision:String::new(),updated_at:1,secret_hash:None},"Test relying party","fixture").await.unwrap();
    let web = db.with_extension("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(
        web.join("index.html"),
        r#"<html><head><script src="./assets/entry.js"></script></head><body></body></html>"#,
    )
    .unwrap();
    let upstream = serve(mw_mock_jmap::router()).await;
    let app = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(keyhex),
            web_dir: Some(web),
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
    let browser = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let rp = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    assert_eq!(
        browser
            .post(format!("{base}/api/login"))
            .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let verifier = "a".repeat(43);
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(Sha256::digest(verifier.as_bytes()));
    let shell = rp
        .get(format!("{base}/oidc/consent?flow=fixture"))
        .header("accept", "text/html")
        .send()
        .await
        .unwrap();
    assert_eq!(shell.status(), 200);
    let html = shell.text().await.unwrap();
    assert!(html.contains("src=\"/assets/entry.js\""));
    assert!(!html.contains("mw-console"));
    let discovery = rp
        .get(format!("{base}/oidc/.well-known/openid-configuration"))
        .send()
        .await
        .unwrap();
    assert_eq!(discovery.headers()["access-control-allow-origin"], "*");
    let authorize = format!("{base}/oidc/authorize");
    let params = [
        ("client_id", "rp"),
        ("redirect_uri", "https://rp.example/callback"),
        ("response_type", "code"),
        ("scope", "openid email offline_access"),
        ("code_challenge", challenge.as_str()),
        ("code_challenge_method", "S256"),
        ("state", "state-proof"),
        ("nonce", "nonce-proof"),
    ];
    let response = browser.get(&authorize).query(&params).send().await.unwrap();
    assert_eq!(response.status(), 303);
    let location = response.headers()["location"].to_str().unwrap();
    let flow = location.split("flow=").nth(1).unwrap().to_owned();
    assert_eq!(
        rp.get(format!("{base}/oidc/context"))
            .query(&[("flow", &flow)])
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    let context = browser
        .get(format!("{base}/oidc/context"))
        .query(&[("flow", &flow)])
        .send()
        .await
        .unwrap();
    assert_eq!(context.status(), 200);
    let context: Value = context.json().await.unwrap();
    assert_eq!(context["loginRequired"], false);
    let csrf = context["csrfToken"].as_str().unwrap();
    assert_eq!(
        browser
            .post(format!("{base}/oidc/decision"))
            .json(&json!({"flow":flow,"approve":true}))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    let decision = browser
        .post(format!("{base}/oidc/decision"))
        .header("x-csrf-token", csrf)
        .json(&json!({"flow":flow,"approve":true}))
        .send()
        .await
        .unwrap();
    assert_eq!(decision.status(), 200);
    let result: Value = decision.json().await.unwrap();
    let url = reqwest::Url::parse(result["redirectUri"].as_str().unwrap()).unwrap();
    let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    assert_eq!(pairs["state"], "state-proof");
    let code = &pairs["code"];
    let token = [
        ("grant_type", "authorization_code"),
        ("client_id", "rp"),
        ("code", code.as_str()),
        ("redirect_uri", "https://rp.example/callback"),
        ("code_verifier", verifier.as_str()),
    ];
    let response = rp
        .post(format!("{base}/oidc/token"))
        .header("origin", "https://rp.example")
        .form(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers()["access-control-allow-origin"],
        "https://rp.example"
    );
    let issued: Value = response.json().await.unwrap();
    let verified = mw_sso::issuer::verify_id_token(
        issued["id_token"].as_str().unwrap(),
        issuer,
        "rp",
        json!({"keys":[jwk]}),
        Some("nonce-proof"),
    )
    .unwrap();
    assert_eq!(verified["sub"], context["subject"]);
    let access = issued["access_token"].as_str().unwrap();
    let info = rp
        .get(format!("{base}/oidc/userinfo"))
        .bearer_auth(access)
        .send()
        .await
        .unwrap();
    assert_eq!(info.status(), 200);
    assert_eq!(info.json::<Value>().await.unwrap()["sub"], verified["sub"]);
    assert_eq!(
        rp.get(format!("{base}/api/me"))
            .bearer_auth(access)
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        browser
            .get(format!("{base}/oidc/userinfo"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        rp.post(format!("{base}/oidc/token"))
            .form(&token)
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    assert_eq!(
        rp.get(format!("{base}/oidc/userinfo"))
            .bearer_auth(access)
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let prompt = browser
        .get(&authorize)
        .query(&params)
        .query(&[("prompt", "login")])
        .send()
        .await
        .unwrap();
    let flow = prompt.headers()["location"]
        .to_str()
        .unwrap()
        .split("flow=")
        .nth(1)
        .unwrap();
    let context: Value = browser
        .get(format!("{base}/oidc/context"))
        .query(&[("flow", flow)])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(context["reauthenticationRequired"], true);
    let next_code = issue_code(&browser, &base, &params).await;
    let response = rp
        .post(format!("{base}/oidc/token"))
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", "rp"),
            ("code", next_code.as_str()),
            ("redirect_uri", "https://rp.example/callback"),
            ("code_verifier", verifier.as_str()),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let next: Value = response.json().await.unwrap();
    let denied_scope = rp
        .post(format!("{base}/oidc/token"))
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", "rp"),
            ("refresh_token", next["refresh_token"].as_str().unwrap()),
            ("scope", "openid mail.read"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(denied_scope.status(), 400);
    assert_eq!(
        denied_scope.json::<Value>().await.unwrap()["error"],
        "invalid_scope"
    );
    let refresh = [
        ("scope", "openid offline_access"),
        ("grant_type", "refresh_token"),
        ("client_id", "rp"),
        ("refresh_token", next["refresh_token"].as_str().unwrap()),
    ];
    let refreshed = rp
        .post(format!("{base}/oidc/token"))
        .form(&refresh)
        .send()
        .await
        .unwrap();
    assert_eq!(refreshed.status(), 200);
    let refreshed: Value = refreshed.json().await.unwrap();
    assert_ne!(refreshed["refresh_token"], next["refresh_token"]);
    let narrowed: Value = rp
        .get(format!("{base}/oidc/userinfo"))
        .bearer_auth(refreshed["access_token"].as_str().unwrap())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(narrowed.get("email").is_none());
    assert_eq!(
        rp.post(format!("{base}/oidc/token"))
            .form(&refresh)
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    assert_eq!(
        rp.get(format!("{base}/oidc/userinfo"))
            .bearer_auth(refreshed["access_token"].as_str().unwrap())
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    // Confidential clients must prove their configured authentication method;
    // a wrong secret must not consume the valid PKCE-bound code.
    let secret = "oidc_secret_fixture-with-enough-entropy-for-test";
    let secret_hash: String = Sha256::digest(secret.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    for method in ["client_secret_basic", "client_secret_post"] {
        let id = format!("confidential-{method}");
        store.create_oidc_client(&OidcClientRow {client_id:id.clone(),config:json!({"redirectUris":["https://rp.example/callback"],"postLogoutRedirectUris":[],"scopes":["openid"],"authMethod":method}),enabled:true,revision:String::new(),updated_at:1,secret_hash:Some(secret_hash.clone())},"Confidential RP","fixture").await.unwrap();
        let confidential_params: Vec<(&str, &str)> = params
            .iter()
            .map(|(key, value)| {
                (
                    *key,
                    if *key == "client_id" {
                        id.as_str()
                    } else if *key == "scope" {
                        "openid"
                    } else {
                        *value
                    },
                )
            })
            .collect();
        let code = issue_code(&browser, &base, &confidential_params).await;
        let fields = [
            ("grant_type", "authorization_code"),
            ("client_id", id.as_str()),
            ("code", code.as_str()),
            ("redirect_uri", "https://rp.example/callback"),
            ("code_verifier", verifier.as_str()),
        ];
        let wrong = rp
            .post(format!("{base}/oidc/token"))
            .basic_auth(&id, Some("wrong"))
            .form(&fields)
            .send()
            .await
            .unwrap();
        assert_eq!(wrong.status(), 401);
        assert_eq!(
            wrong.json::<Value>().await.unwrap()["error"],
            "invalid_client"
        );
        let request = rp.post(format!("{base}/oidc/token"));
        let request = if method == "client_secret_basic" {
            request.basic_auth(&id, Some(secret)).form(&fields)
        } else {
            let mut form = fields.to_vec();
            form.push(("client_secret", secret));
            request.form(&form)
        };
        let response = request.send().await.unwrap();
        assert_eq!(response.status(), 200);
        assert!(
            response.json::<Value>().await.unwrap()["id_token"]
                .as_str()
                .is_some()
        );
    }
    let mail_request = json!({"clientId":"rp","redirectUri":"https://rp.example/callback","codeChallenge":challenge,"resource":"https://mail.example/","scope":null});
    assert_eq!(
        browser
            .post(format!("{base}/oauth/consent"))
            .json(&mail_request)
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    assert_eq!(
        browser
            .post(format!("{base}/oauth/decision"))
            .json(&json!({"approve":true,"params":mail_request}))
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    let refused = rp
        .post(format!("{base}/oauth/token"))
        .json(&json!({"grant_type":"authorization_code","client_id":"rp","code":"none"}))
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), 400);
    assert_eq!(
        refused.json::<Value>().await.unwrap()["error"],
        "invalid_client"
    );
    let owned = store.oidc_client("rp").await.unwrap().unwrap();
    let revision = owned.revision.clone();
    let disabled = OidcClientRow {
        enabled: false,
        ..owned
    };
    store
        .put_oidc_client(&disabled, Some(&revision))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        browser
            .post(format!("{base}/oauth/consent"))
            .json(&mail_request)
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    let current = store.oidc_client("rp").await.unwrap().unwrap();
    let revision = current.revision.clone();
    store
        .put_oidc_client(
            &OidcClientRow {
                enabled: true,
                ..current
            },
            Some(&revision),
        )
        .await
        .unwrap()
        .unwrap();
    // Only registered RP origins receive protocol CORS; credentials are never allowed.
    let preflight = rp
        .request(reqwest::Method::OPTIONS, format!("{base}/oidc/token"))
        .header("origin", "https://rp.example")
        .header("access-control-request-method", "POST")
        .header(
            "access-control-request-headers",
            "content-type, authorization",
        )
        .send()
        .await
        .unwrap();
    assert_eq!(preflight.status(), 204);
    assert!(
        preflight
            .headers()
            .get("access-control-allow-credentials")
            .is_none()
    );
    assert_eq!(
        rp.request(reqwest::Method::OPTIONS, format!("{base}/oidc/token"))
            .header("origin", "https://evil.example")
            .header("access-control-request-method", "POST")
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    // The initial cross-site GET may omit a Strict session cookie. It only
    // prepares a browser-bound confirmation; it never signs the user out.
    let logout = browser
        .get(format!("{base}/oidc/logout"))
        .header("cookie", "")
        .query(&[
            ("id_token_hint", issued["id_token"].as_str().unwrap()),
            ("post_logout_redirect_uri", "https://rp.example/logout"),
            ("state", "logout-state"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(logout.status(), 303);
    let handle = logout.headers()["location"]
        .to_str()
        .unwrap()
        .split("logout=")
        .nth(1)
        .unwrap();
    assert_eq!(
        browser
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let review = browser
        .get(format!("{base}/oidc/logout/context"))
        .query(&[("flow", handle)])
        .send()
        .await
        .unwrap();
    assert_eq!(review.status(), 200);
    let review: Value = review.json().await.unwrap();
    let signout = browser
        .post(format!("{base}/oidc/logout"))
        .header("x-csrf-token", review["csrfToken"].as_str().unwrap())
        .json(&json!({"flow":handle,"approve":true}))
        .send()
        .await
        .unwrap();
    assert_eq!(signout.status(), 200);
    assert_eq!(
        signout.json::<Value>().await.unwrap()["redirectUri"],
        "https://rp.example/logout?state=logout-state"
    );
    assert_eq!(
        browser
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let admin = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    assert_eq!(
        admin
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let body = json!({"name":"Managed RP","enabled":true,"config":{"redirectUris":["https://managed.example/cb"],"postLogoutRedirectUris":[],"scopes":["openid"],"authMethod":"client_secret_basic"},"secretAction":"rotate"});
    assert_eq!(
        admin
            .post(format!("{base}/admin/oidc/clients"))
            .json(&body)
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
    let created = admin
        .post(format!("{base}/admin/oidc/clients"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), 201);
    let created: Value = created.json().await.unwrap();
    assert!(
        created["clientSecret"]
            .as_str()
            .unwrap()
            .starts_with("oidc_secret_")
    );
    let id = created["clientId"].as_str().unwrap();
    let detail = admin
        .get(format!("{base}/admin/oidc/clients/{id}"))
        .send()
        .await
        .unwrap();
    let etag = detail.headers()["etag"].to_str().unwrap().to_owned();
    let detail: Value = detail.json().await.unwrap();
    assert_eq!(detail["secretConfigured"], true);
    assert!(detail.get("clientSecret").is_none());
    assert!(detail.get("secretHash").is_none());
    assert_eq!(
        admin
            .put(format!("{base}/admin/oidc/clients/{id}"))
            .header("if-match", "\"stale\"")
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    assert_eq!(
        admin
            .delete(format!("{base}/admin/oidc/clients/{id}"))
            .header("if-match", etag)
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    let mut limited = false;
    for _ in 0..40 {
        let response = rp
            .post(format!("{base}/oidc/token"))
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_id", "rp"),
                ("code", "unknown"),
            ])
            .send()
            .await
            .unwrap();
        if response.status() == 429 {
            assert_eq!(response.headers()["retry-after"], "1");
            assert_eq!(
                response.json::<Value>().await.unwrap()["error"],
                "temporarily_unavailable"
            );
            limited = true;
            break;
        }
    }
    assert!(limited, "token admission must bound repeated requests");
}

async fn issue_code(browser: &reqwest::Client, base: &str, params: &[(&str, &str)]) -> String {
    let response = browser
        .get(format!("{base}/oidc/authorize"))
        .query(params)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 303);
    let flow = response.headers()["location"]
        .to_str()
        .unwrap()
        .split("flow=")
        .nth(1)
        .unwrap();
    let context: Value = browser
        .get(format!("{base}/oidc/context"))
        .query(&[("flow", flow)])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let response = browser
        .post(format!("{base}/oidc/decision"))
        .header("x-csrf-token", context["csrfToken"].as_str().unwrap())
        .json(&json!({"flow":flow,"approve":true}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let result: Value = response.json().await.unwrap();
    reqwest::Url::parse(result["redirectUri"].as_str().unwrap())
        .unwrap()
        .query_pairs()
        .find(|(key, _)| key == "code")
        .unwrap()
        .1
        .to_string()
}
