//! t63: browser analytics tags. The shell CSP widens only for a consenting,
//! signed-in user; the loader is a same-origin content-hashed file; saving needs
//! step-up and writes an audit entry that carries the hash, never the snippet.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;

const SHELL_CSP: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self' blob: data:; font-src 'self'; manifest-src 'self'; connect-src 'self' blob:; frame-src 'self'; worker-src 'self' blob:; base-uri 'none'; form-action 'none'; require-trusted-types-for 'script'";

async fn shell_headers_tag(client: &reqwest::Client, base: &str) -> Option<String> {
    let r = client.get(format!("{base}/")).send().await.unwrap();
    r.headers()
        .get("x-mw-analytics-tag")
        .map(|v| v.to_str().unwrap().to_owned())
}

async fn sql_rows(db: &str, statement: &str) -> Vec<String> {
    use sqlx::{Connection, Row};
    let mut conn = sqlx::SqliteConnection::connect(&format!("sqlite://{db}"))
        .await
        .unwrap();
    sqlx::query(statement)
        .fetch_all(&mut conn)
        .await
        .unwrap()
        .iter()
        .map(|r| r.try_get::<String, _>(0).unwrap_or_default())
        .collect()
}

fn csp(r: &reqwest::Response) -> Vec<String> {
    r.headers()
        .get_all("content-security-policy")
        .iter()
        .map(|v| v.to_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn browser_tag_is_consent_gated_hashed_stepped_up_and_audited() {
    let web = std::env::temp_dir().join(format!("mw-t63-web-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(
        web.join("index.html"),
        "<!doctype html><html><head><script type=\"module\" src=\"./assets/index.js\"></script></head><body><div id=\"root\"></div></body></html>",
    )
    .unwrap();
    let db: String = common::test_db::unique_db_path("analytics-tag")
        .to_string_lossy()
        .into();
    let mock_socket = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock_url = format!("http://{}", mock_socket.local_addr().unwrap());
    let mock_task = tokio::spawn(async move {
        axum::serve(mock_socket, mw_mock_jmap::router())
            .await
            .unwrap();
    });
    let (app, _) = build_app_full(
        AppConfig {
            db_path: db.clone(),
            server_key_hex: None,
            web_dir: Some(web.clone()),
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(mock_url.clone()),
                jmap_upstreams: Some(vec![mock_url]),
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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let client = || {
        reqwest::Client::builder()
            .no_proxy()
            .cookie_store(true)
            .build()
            .unwrap()
    };
    let admin = client();
    let login = admin
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":"root","password":"hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200);
    let marker_cookie = login
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().to_owned())
        .find(|c| c.starts_with("mw_admin_browser=1"));
    let marker_cookie = marker_cookie.expect("admin sign-in sets the admin-browser marker");
    assert!(
        marker_cookie.contains("HttpOnly")
            && marker_cookie.contains("SameSite=Strict")
            && marker_cookie.contains("Max-Age=34560000"),
        "{marker_cookie}"
    );
    let read = admin
        .get(format!("{base}/admin/analytics/tag"))
        .send()
        .await
        .unwrap();
    assert_eq!(read.status(), 200);
    let etag = read.headers()["etag"].to_str().unwrap().to_owned();
    let view: Value = read.json().await.unwrap();
    assert_eq!(view["config"]["enabled"], false);

    let secret_snippet = "window.__t63_fixture_secret_snippet = 1;";
    let custom = json!({"enabled":true,"provider":"custom","customKind":"inline","snippet":secret_snippet,
        "connectOrigins":["https://collect.example.com"],"imgOrigins":["https://px.example.com"]});
    // A fresh step-up proof is required.
    let r = admin.put(format!("{base}/admin/analytics/tag")).header("if-match", &etag).json(&json!({"acknowledgeFullAccess":true,"enabled":true,"provider":"custom","customKind":"inline","snippet":secret_snippet})).send().await.unwrap();
    assert_eq!(r.status(), 403);
    assert_eq!(r.json::<Value>().await.unwrap()["code"], "step_up_required");
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
    // The revision is required.
    assert_eq!(
        admin
            .put(format!("{base}/admin/analytics/tag"))
            .json(&custom)
            .send()
            .await
            .unwrap()
            .status(),
        428
    );
    // Without the acknowledgement a custom tag is refused.
    let r = admin
        .put(format!("{base}/admin/analytics/tag"))
        .header("if-match", &etag)
        .json(&custom)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 422);
    assert_eq!(
        r.json::<Value>().await.unwrap()["code"],
        "acknowledgeFullAccess"
    );
    // Unsafe origins are refused.
    for bad in [
        "http://collect.example.com",
        "*",
        "data:",
        "javascript:alert(1)",
        "https://*.com",
        "'unsafe-inline'",
    ] {
        let mut body = custom.clone();
        body["acknowledgeFullAccess"] = json!(true);
        body["connectOrigins"] = json!([bad]);
        assert_eq!(
            admin
                .put(format!("{base}/admin/analytics/tag"))
                .header("if-match", &etag)
                .json(&body)
                .send()
                .await
                .unwrap()
                .status(),
            422,
            "{bad}"
        );
    }
    let mut oversized = custom.clone();
    oversized["acknowledgeFullAccess"] = json!(true);
    oversized["snippet"] = json!("a".repeat(32 * 1024 + 1));
    assert_eq!(
        admin
            .put(format!("{base}/admin/analytics/tag"))
            .header("if-match", &etag)
            .json(&oversized)
            .send()
            .await
            .unwrap()
            .status(),
        422
    );

    let mut accepted = custom.clone();
    accepted["acknowledgeFullAccess"] = json!(true);
    let r = admin
        .put(format!("{base}/admin/analytics/tag"))
        .header("if-match", &etag)
        .json(&accepted)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "{}", r.text().await.unwrap());
    let view: Value = admin
        .get(format!("{base}/admin/analytics/tag"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(view["preview"]["valid"], true);
    let path = view["preview"]["path"].as_str().unwrap().to_owned();
    let sha = view["preview"]["sha256"].as_str().unwrap().to_owned();
    assert_eq!(path, format!("/assets/analytics/{sha}.js"));
    assert!(!view.to_string().contains("acknowledgeFullAccess"));

    // The audit entry records the hash and origins, never the snippet.
    let audit: Vec<Value> = admin
        .get(format!("{base}/admin/audit?limit=200"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(audit.iter().any(
        |e| e["action"] == "analytics-tag-configured" && e["target"] == "analytics:browser-tag"
    ));
    // The before/after detail is in the audit row itself (not in the list API).
    let rows = sql_rows(&db, "SELECT COALESCE(after_json,'') || ' ' || COALESCE(before_json,'') || ' ' || detail_json FROM audit_log WHERE action = 'analytics-tag-configured'").await;
    let entry = rows.join("\n");
    assert!(entry.contains(&sha), "{entry}");
    assert!(entry.contains("https://collect.example.com"), "{entry}");
    let everything = sql_rows(
        &db,
        "SELECT COALESCE(after_json,'') || COALESCE(before_json,'') || detail_json FROM audit_log",
    )
    .await
    .join("\n");
    assert!(
        !everything.contains("__t63_fixture_secret_snippet"),
        "{everything}"
    );

    // Signed out: the shell is unchanged and the loader is not served.
    let anon = client();
    let shell = anon
        .get(format!("{base}/"))
        .header("sec-fetch-dest", "document")
        .send()
        .await
        .unwrap();
    assert_eq!(csp(&shell), [SHELL_CSP]);
    assert!(!shell.text().await.unwrap().contains("mw-analytics"));
    assert_eq!(
        anon.get(format!("{base}{path}"))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );

    // Signed in without consent: unchanged.
    let user = client();
    tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    let login = user
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200);
    let login_cookie = login
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().split(';').next().unwrap().to_owned())
        .collect::<Vec<_>>()
        .join("; ");
    let shell = user.get(format!("{base}/")).send().await.unwrap();
    assert_eq!(csp(&shell), [SHELL_CSP]);
    assert!(!shell.text().await.unwrap().contains("mw-analytics"));

    // Consent to server-side counts only (the pre-t63 promise) loads nothing.
    let me: Value = user
        .get(format!("{base}/api/me"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let authority = me["storageAuthority"].as_str().unwrap().to_owned();
    let prefs = user
        .get(format!("{base}/api/account/preferences"))
        .send()
        .await
        .unwrap();
    let ptag = prefs.headers()["etag"].to_str().unwrap().to_owned();
    assert_eq!(
        user.put(format!("{base}/api/account/preferences"))
            .header("x-mailwoman-storage-authority", &authority)
            .header("if-match", ptag)
            .json(&json!({"version":1,"preferences":{"analyticsConsent":true}}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let status: Value = user
        .get(format!("{base}/api/analytics/status"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(status["browserTag"]["provider"], "custom");
    assert_eq!(status["browserTag"]["consent"], false);
    let key = status["browserTag"]["consentKey"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(key, format!("custom:{sha}"));
    let shell = user.get(format!("{base}/")).send().await.unwrap();
    assert_eq!(csp(&shell), [SHELL_CSP]);
    assert_eq!(
        user.get(format!("{base}{path}"))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
    // A malformed or foreign tag consent is refused by the preferences schema.
    let set_tag_consent = |value: Value| {
        let user = user.clone();
        let base = base.clone();
        let authority = authority.clone();
        async move {
            let prefs = user
                .get(format!("{base}/api/account/preferences"))
                .send()
                .await
                .unwrap();
            let ptag = prefs.headers()["etag"].to_str().unwrap().to_owned();
            user.put(format!("{base}/api/account/preferences")).header("x-mailwoman-storage-authority", &authority).header("if-match", ptag)
                .json(&json!({"version":1,"preferences":{"analyticsConsent":true,"analyticsTagConsent":value}})).send().await.unwrap().status()
        }
    };
    assert_eq!(set_tag_consent(json!("custom:nothex")).await, 400);
    assert_eq!(
        set_tag_consent(json!(format!("plausible:{sha}"))).await,
        200
    );
    assert_eq!(
        csp(&user.get(format!("{base}/")).send().await.unwrap()),
        [SHELL_CSP],
        "another provider's consent"
    );
    // Consent to this provider and loader.
    assert_eq!(set_tag_consent(json!(key)).await, 200);
    let status: Value = user
        .get(format!("{base}/api/analytics/status"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(status["browserTag"]["consent"], true);

    let shell = user.get(format!("{base}/")).send().await.unwrap();
    let widened = SHELL_CSP
        .replace(
            "img-src 'self' blob: data:",
            "img-src 'self' blob: data: https://px.example.com",
        )
        .replace(
            "connect-src 'self' blob:",
            "connect-src 'self' blob: https://collect.example.com",
        )
        + "; trusted-types default svelte-trusted-html";
    assert_eq!(csp(&shell), std::slice::from_ref(&widened));
    assert_eq!(shell.headers()["cache-control"], "private, no-store");
    let html = shell.text().await.unwrap();
    assert!(
        html.contains(&format!(
            r#"<meta name="mw-analytics" content="" data-consent="{key}">"#
        )),
        "{html}"
    );
    assert_eq!(shell_headers_tag(&user, &base).await, Some("1".to_owned()));
    assert!(
        html.contains(&format!(
            r#"<script defer src="{path}" data-mw-analytics></script></body>"#
        )),
        "{html}"
    );
    assert!(!widened.contains("unsafe-inline") && !widened.contains("'unsafe-eval'"));

    // The loader: same-origin, hashed, typed, nosniff, long-cached.
    let js = user.get(format!("{base}{path}")).send().await.unwrap();
    assert_eq!(js.status(), 200);
    assert_eq!(
        js.headers()["content-type"],
        "text/javascript; charset=utf-8"
    );
    assert_eq!(js.headers()["x-content-type-options"], "nosniff");
    assert_eq!(
        js.headers()["cache-control"],
        "private, max-age=31536000, immutable"
    );
    let body = js.bytes().await.unwrap();
    use sha2::Digest;
    let digest: String = sha2::Sha256::digest(&body)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(digest, sha);
    assert!(String::from_utf8_lossy(&body).ends_with(&format!("{secret_snippet}\n")));
    let wrong = format!("/assets/analytics/{}.js", "0".repeat(64));
    assert_eq!(
        user.get(format!("{base}{wrong}"))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );

    // A browser that also holds an admin console session never gets the tag (F3).
    let session_cookie = login_cookie.clone();
    let admin_cookie = format!("{}=anything", mw_admin::config::session_cookie_name());
    let raw = reqwest::Client::builder().no_proxy().build().unwrap();
    let with_admin = raw
        .get(format!("{base}/"))
        .header("cookie", format!("{session_cookie}; {admin_cookie}"))
        .send()
        .await
        .unwrap();
    assert_eq!(csp(&with_admin), [SHELL_CSP]);
    assert!(!with_admin.text().await.unwrap().contains("mw-analytics"));
    assert_eq!(
        raw.get(format!("{base}{path}"))
            .header("cookie", format!("{session_cookie}; {admin_cookie}"))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
    let without_admin = raw
        .get(format!("{base}/"))
        .header("cookie", session_cookie.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(
        csp(&without_admin),
        std::slice::from_ref(&widened),
        "the same session without an admin cookie is tagged"
    );
    // A browser that ever signed in to the console keeps a marker that refuses tags (R2).
    let marker = raw
        .get(format!("{base}/"))
        .header("cookie", format!("{session_cookie}; mw_admin_browser=1"))
        .send()
        .await
        .unwrap();
    assert_eq!(csp(&marker), [SHELL_CSP]);

    // DNT / GPC: unchanged even with consent.
    for header in ["dnt", "sec-gpc"] {
        let shell = user
            .get(format!("{base}/"))
            .header(header, "1")
            .send()
            .await
            .unwrap();
        assert_eq!(csp(&shell), [SHELL_CSP], "{header}");
    }
    // The admin console and the reader bridge frame keep their own policies.
    let console = user
        .get(format!("{base}/admin"))
        .header("sec-fetch-dest", "document")
        .header("accept", "text/html")
        .send()
        .await
        .unwrap();
    assert_eq!(csp(&console), [SHELL_CSP]);
    assert!(!console.text().await.unwrap().contains("mw-analytics"));
    let frame = user
        .get(format!("{base}/api/rc/frame"))
        .send()
        .await
        .unwrap();
    let frame_csp = csp(&frame);
    assert!(!frame_csp.is_empty());
    assert!(
        frame_csp.iter().all(|p| !p.contains("example.com")),
        "{frame_csp:?}"
    );
    let frame_anon = anon
        .get(format!("{base}/api/rc/frame"))
        .send()
        .await
        .unwrap();
    assert_eq!(csp(&frame_anon), frame_csp);

    // Disabling needs no Super admin decision and removes every addition at once.
    let etag = admin
        .get(format!("{base}/admin/analytics/tag"))
        .send()
        .await
        .unwrap()
        .headers()["etag"]
        .to_str()
        .unwrap()
        .to_owned();
    let mut off = custom.clone();
    off["enabled"] = json!(false);
    assert_eq!(
        admin
            .put(format!("{base}/admin/analytics/tag"))
            .header("if-match", &etag)
            .json(&off)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let shell = user.get(format!("{base}/")).send().await.unwrap();
    assert_eq!(csp(&shell), [SHELL_CSP]);
    assert_eq!(
        user.get(format!("{base}{path}"))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );

    // A preset: exact additions.
    let etag = admin
        .get(format!("{base}/admin/analytics/tag"))
        .send()
        .await
        .unwrap()
        .headers()["etag"]
        .to_str()
        .unwrap()
        .to_owned();
    let plausible = json!({"enabled":true,"provider":"plausible","siteId":"mail.example.com"});
    // Presets also run provider code with full page access: acknowledged too (F3).
    let r = admin
        .put(format!("{base}/admin/analytics/tag"))
        .header("if-match", &etag)
        .json(&plausible)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 422);
    let mut acked = plausible.clone();
    acked["acknowledgeFullAccess"] = json!(true);
    assert_eq!(
        admin
            .put(format!("{base}/admin/analytics/tag"))
            .header("if-match", &etag)
            .json(&acked)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    // The provider changed: the old consent does not carry over (F1).
    assert_eq!(
        csp(&user.get(format!("{base}/")).send().await.unwrap()),
        [SHELL_CSP]
    );
    let status: Value = user
        .get(format!("{base}/api/analytics/status"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(status["browserTag"]["consent"], false);
    assert_eq!(
        set_tag_consent(status["browserTag"]["consentKey"].clone()).await,
        200
    );
    let shell = user.get(format!("{base}/")).send().await.unwrap();
    assert_eq!(
        csp(&shell),
        [SHELL_CSP
            .replace(
                "script-src 'self' 'wasm-unsafe-eval'",
                "script-src 'self' 'wasm-unsafe-eval' https://plausible.io"
            )
            .replace(
                "connect-src 'self' blob:",
                "connect-src 'self' blob: https://plausible.io"
            )
            + "; trusted-types default svelte-trusted-html"]
    );
    assert!(shell.text().await.unwrap().contains(
        r#"<meta name="mw-analytics" content="https://plausible.io" data-consent="plausible:"#
    ));

    task.abort();
    mock_task.abort();
    let _ = std::fs::remove_dir_all(&web);
}
