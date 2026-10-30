//! Real-router client entry gate: independent of mailbox credentials and native headers.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};
use std::net::SocketAddr;
mod common;
const PSK: &str = "entry-test-secret-2468";
static TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}
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
async fn fixture() -> (String, Store, reqwest::Client, String) {
    let upstream = serve(mw_mock_jmap::router()).await;
    let dir = common::test_db::unique_dir("client-access");
    let web = dir.join("web");
    std::fs::create_dir_all(web.join("assets")).unwrap();
    std::fs::write(
        web.join("index.html"),
        "<!doctype html><title>Mail shell</title><main>MAIL_SHELL</main>",
    )
    .unwrap();
    std::fs::write(web.join("assets/app.js"), "console.log('asset')").unwrap();
    let key = ServerKey::generate();
    let db = dir.join("store.db");
    let store = Store::open(db.to_str().unwrap(), key.clone())
        .await
        .unwrap();
    store
        .put_setting_policy(&SettingPolicyRow {
            key: "signin.retryCooldownSecs".into(),
            global_value: Some("0".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
    let app = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(key.to_hex()),
            web_dir: Some(web),
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig {
                csrf_strict: true,
                ..Default::default()
            },
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
    let base = format!(
        "{}{}",
        serve(app).await,
        std::env::var("MW_BASE_PATH").unwrap_or_default()
    );
    let admin = client();
    let signed = admin
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":"root","password":"hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(signed.status(), 200);
    let token = signed.json::<Value>().await.unwrap()["csrfToken"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        admin
            .post(format!("{base}/admin/step-up"))
            .header("x-csrf-token", &token)
            .json(&json!({"password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    (base, store, admin, token)
}
async fn configure(
    base: &str,
    admin: &reqwest::Client,
    csrf: &str,
    body: Value,
) -> reqwest::Response {
    let current = admin
        .get(format!("{base}/admin/client-access"))
        .send()
        .await
        .unwrap();
    assert_eq!(current.status(), 200);
    let etag = current.headers()["etag"].clone();
    admin
        .put(format!("{base}/admin/client-access"))
        .header("if-match", etag)
        .header("x-csrf-token", csrf)
        .json(&body)
        .send()
        .await
        .unwrap()
}
fn policy(psk: bool, slug: Option<&str>) -> Value {
    json!({"pskEnabled":psk,"slugEnabled":slug.is_some(),"slug":slug,"grantHours":24,"psk":PSK})
}
fn nonce(html: &str) -> String {
    html.split("name=\"nonce\" value=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .into()
}
#[tokio::test]
async fn psk_gate_preserves_admin_and_rejects_native_spoofing_and_stale_grants() {
    let _serial = TEST_LOCK.lock().await;
    let (base, store, admin, csrf) = fixture().await;
    let guest = client();
    assert!(
        guest
            .get(format!("{base}/"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
            .contains("MAIL_SHELL")
    );
    let saved = configure(&base, &admin, &csrf, policy(true, None)).await;
    assert_eq!(saved.status(), 200);
    let body = saved.text().await.unwrap();
    assert!(!body.contains(PSK));
    assert!(!body.contains("argon2"));
    let stored = store
        .get_setting("security.clientAccess.v1")
        .await
        .unwrap()
        .unwrap();
    assert!(!stored.contains(PSK));
    assert!(stored.contains("$argon2id$"));
    for path in ["/api/login", "/jmap/session", "/api/me"] {
        let request = if path == "/api/login" {
            guest
                .post(format!("{base}{path}"))
                .json(&json!({"username":"parent@example.org", "password":"testpass"}))
        } else {
            guest.get(format!("{base}{path}"))
        };
        let answer = request
            .header("authorization", "Bearer forged")
            .header("origin", "tauri://localhost")
            .send()
            .await
            .unwrap();
        assert_eq!(answer.status(), 403);
        assert_eq!(
            answer.json::<Value>().await.unwrap()["code"],
            "clientAccessRequired"
        );
    }
    assert_eq!(
        admin
            .get(format!("{base}/admin/client-access"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let page = guest
        .get(format!("{base}/arbitrary/deep/path"))
        .header("accept", "text/html")
        .send()
        .await
        .unwrap();
    assert_eq!(page.headers()["x-mailwoman-client-gate"], "enabled");
    let csp = page.headers()["content-security-policy"].to_str().unwrap();
    assert!(csp.contains("style-src 'self'"));
    assert!(csp.contains("form-action 'self'"));
    assert!(csp.contains("img-src 'self'"));
    assert_eq!(page.headers()["referrer-policy"], "same-origin");
    assert!(!csp.contains("unsafe-inline"));
    assert!(
        page.headers()["cache-control"]
            .to_str()
            .unwrap()
            .contains("no-store")
    );
    let pending = page
        .headers()
        .get_all("set-cookie")
        .iter()
        .find_map(|h| {
            h.to_str()
                .ok()
                .filter(|v| v.starts_with("mw_client_access_pending="))
                .map(|v| v.split(';').next().unwrap().to_string())
        })
        .unwrap();
    let html = page.text().await.unwrap();
    assert!(!html.contains("MAIL_SHELL"));
    assert!(!html.contains("<style>"));
    assert!(html.contains("href=\"/assets/client-access.css\""));
    let style = guest
        .get(format!("{base}/assets/client-access.css"))
        .send()
        .await
        .unwrap();
    assert_eq!(style.status(), 200);
    assert!(
        style.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/css")
    );
    let nonce = nonce(&html);
    let cross = guest
        .post(format!("{base}/api/client-access/unlock"))
        .header("origin", "https://evil.test")
        .header("authorization", "Bearer fake")
        .form(&[("psk", PSK), ("nonce", &nonce)])
        .send()
        .await
        .unwrap();
    assert_eq!(cross.status(), 403);
    let bad = guest
        .post(format!("{base}/api/client-access/unlock"))
        .form(&[("psk", "wrong-entry-key-0000"), ("nonce", &nonce)])
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 403);
    // Stale mailbox/admin cookies must not make the independent public form
    // require a CSRF token bound to a session that no longer exists.
    let unlocked = guest
        .post(format!("{base}/api/client-access/unlock"))
        .header(
            "cookie",
            format!("{pending}; mw_session=expired; mw_admin_session=expired; mw_csrf=stale"),
        )
        .form(&[("psk", PSK), ("nonce", &nonce)])
        .send()
        .await
        .unwrap();
    assert_eq!(unlocked.status(), 303, "{}", unlocked.text().await.unwrap());
    let shell = guest.get(format!("{base}/")).send().await.unwrap();
    assert_eq!(shell.headers()["referrer-policy"], "no-referrer");
    assert!(
        shell.headers()["content-security-policy"]
            .to_str()
            .unwrap()
            .contains("form-action 'none'")
    );
    assert_eq!(shell.headers()["x-mailwoman-client-gate"], "enabled");
    assert!(shell.text().await.unwrap().contains("MAIL_SHELL"));
    assert_eq!(
        guest
            .get(format!("{base}/jmap/session"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let mut changed = policy(true, None);
    changed["psk"] = json!("rotated-entry-key-123456");
    assert_eq!(configure(&base, &admin, &csrf, changed).await.status(), 200);
    assert_eq!(
        guest
            .get(format!("{base}/jmap/session"))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
}
#[tokio::test]
async fn slug_is_an_entry_capability_and_both_modes_require_both_proofs() {
    let _serial = TEST_LOCK.lock().await;
    let (base, _, admin, csrf) = fixture().await;
    for psk in [false, true] {
        let guest = client();
        let slug = "private-client-entry";
        assert_eq!(
            configure(&base, &admin, &csrf, policy(psk, Some(slug)))
                .await
                .status(),
            200
        );
        for path in ["/", "/index.html", "/another/path"] {
            let denied = guest
                .get(format!("{base}{path}"))
                .header("accept", "text/html")
                .send()
                .await
                .unwrap();
            assert_eq!(denied.status(), 404);
            assert_eq!(denied.headers()["x-mailwoman-client-gate"], "enabled");
            assert!(
                denied.headers()["cache-control"]
                    .to_str()
                    .unwrap()
                    .contains("no-store")
            );
            let body = denied.text().await.unwrap();
            assert_eq!(body, "Not found");
            assert!(!body.contains(slug));
        }
        assert_eq!(
            guest
                .get(format!("{base}/assets/app.js"))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
        let entry = guest.get(format!("{base}/{slug}")).send().await.unwrap();
        if psk {
            assert_eq!(entry.status(), 200);
            let nonce = nonce(&entry.text().await.unwrap());
            assert_eq!(
                guest
                    .get(format!("{base}/jmap/session"))
                    .send()
                    .await
                    .unwrap()
                    .status(),
                403
            );
            assert_eq!(
                guest
                    .post(format!("{base}/api/client-access/unlock"))
                    .form(&[("psk", PSK), ("nonce", nonce.as_str())])
                    .send()
                    .await
                    .unwrap()
                    .status(),
                303
            );
        } else {
            assert_eq!(entry.status(), 303);
            assert_eq!(entry.headers()["location"], "/");
        }
        let bookmarked = guest.get(format!("{base}/{slug}")).send().await.unwrap();
        assert_eq!(bookmarked.status(), 303);
        assert_eq!(bookmarked.headers()["location"], "/");
        assert_eq!(
            guest
                .get(format!("{base}/jmap/session"))
                .send()
                .await
                .unwrap()
                .status(),
            401
        );
    }
}
#[tokio::test]
async fn configuration_is_conditional_redacted_and_rejects_collisions_in_both_directions() {
    let _serial = TEST_LOCK.lock().await;
    let (base, store, admin, csrf) = fixture().await;
    assert_eq!(
        client()
            .get(format!("{base}/admin/client-access"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let current = admin
        .get(format!("{base}/admin/client-access"))
        .send()
        .await
        .unwrap();
    let etag = current.headers()["etag"].clone();
    let defaults = current.json::<Value>().await.unwrap();
    assert_eq!(defaults["pskEnabled"], false);
    assert_eq!(defaults["slugEnabled"], false);
    let first = configure(
        &base,
        &admin,
        &csrf,
        policy(true, Some("secret-entry-path")),
    )
    .await;
    assert_eq!(first.status(), 200);
    assert_eq!(
        admin
            .put(format!("{base}/admin/client-access"))
            .header("if-match", etag)
            .header("x-csrf-token", &csrf)
            .json(&policy(false, None))
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    assert_eq!(
        configure(&base, &admin, &csrf, policy(false, Some("admin")))
            .await
            .status(),
        422
    );
    assert!(
        mw_server::console_path::cli(
            &store,
            mw_admin::cli::ConsolePathVerb::Set {
                path: "/secret-entry-path".into()
            }
        )
        .await
        .is_err()
    );
    store
        .set_setting("admin.consolePath", "/pending-console-path")
        .await
        .unwrap();
    assert_eq!(
        configure(
            &base,
            &admin,
            &csrf,
            policy(false, Some("pending-console-path"))
        )
        .await
        .status(),
        422
    );
    let mut preserve = policy(true, None);
    preserve["psk"] = json!("");
    let saved = configure(&base, &admin, &csrf, preserve).await;
    assert_eq!(saved.status(), 200);
    assert_eq!(saved.json::<Value>().await.unwrap()["pskConfigured"], true);
    let mut clear = policy(false, None);
    clear.as_object_mut().unwrap().remove("psk");
    clear["clearPsk"] = json!(true);
    assert_eq!(configure(&base, &admin, &csrf, clear).await.status(), 200);
}

#[tokio::test]
async fn deployment_prefix_scopes_form_action_grants_and_fixed_redirect() {
    let _serial = TEST_LOCK.lock().await;
    struct Restore(Option<std::ffi::OsString>);
    impl Drop for Restore {
        fn drop(&mut self) {
            unsafe {
                match &self.0 {
                    Some(value) => std::env::set_var("MW_BASE_PATH", value),
                    None => std::env::remove_var("MW_BASE_PATH"),
                }
            }
        }
    }
    let _restore = Restore(std::env::var_os("MW_BASE_PATH"));
    unsafe { std::env::set_var("MW_BASE_PATH", "/gatehost") };
    let (base, _, admin, csrf) = fixture().await;
    let guest = client();
    let saved = configure(&base, &admin, &csrf, policy(true, Some("private-gatehost"))).await;
    assert_eq!(saved.status(), 200);
    assert_eq!(
        saved.json::<Value>().await.unwrap()["clientEntryPath"],
        "/gatehost/private-gatehost"
    );
    let page = guest
        .get(format!("{base}/private-gatehost"))
        .send()
        .await
        .unwrap();
    assert!(
        page.headers()
            .get_all("set-cookie")
            .iter()
            .any(|v| v.to_str().unwrap().contains("Path=/gatehost/"))
    );
    let html = page.text().await.unwrap();
    assert!(html.contains("href=\"/gatehost/assets/client-access.css\""));
    assert!(html.contains("action=\"/gatehost/api/client-access/unlock\""));
    let nonce = nonce(&html);
    let unlocked = guest
        .post(format!("{base}/api/client-access/unlock"))
        .form(&[("psk", PSK), ("nonce", nonce.as_str())])
        .send()
        .await
        .unwrap();
    assert_eq!(unlocked.status(), 303);
    assert_eq!(unlocked.headers()["location"], "/gatehost/");
    assert!(
        guest
            .get(format!("{base}/"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
            .contains("MAIL_SHELL")
    );
}

#[tokio::test]
async fn rejected_keys_use_durable_existing_failure_locks_even_when_cooldown_is_off() {
    let _serial = TEST_LOCK.lock().await;
    let (base, store, admin, csrf) = fixture().await;
    let guest = client();
    store
        .put_setting_policy(&SettingPolicyRow {
            key: "security.signin.mailboxFailureLimit".into(),
            global_value: Some("2".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
    assert_eq!(
        configure(&base, &admin, &csrf, policy(true, None))
            .await
            .status(),
        200
    );
    let html = guest
        .get(format!("{base}/"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let nonce = nonce(&html);
    for expected in [403, 403, 429] {
        let answer = guest
            .post(format!("{base}/api/client-access/unlock"))
            .form(&[("psk", "wrong-key-long-enough"), ("nonce", nonce.as_str())])
            .send()
            .await
            .unwrap();
        assert_eq!(answer.status().as_u16(), expected);
        if expected == 429 {
            assert!(answer.headers().contains_key("retry-after"));
            let html = answer.text().await.unwrap();
            assert!(html.contains("Please wait before trying again"));
            assert!(html.contains("name=\"psk\""));
            assert!(html.contains(&nonce));
        }
    }
}
