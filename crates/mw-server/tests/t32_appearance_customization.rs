use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;
async fn serve(router: axum::Router) -> String {
    let socket = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", socket.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(
            socket,
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    base
}
async fn read(client: &reqwest::Client, url: &str) -> (Value, String) {
    let response = client.get(url).send().await.unwrap();
    assert_eq!(response.status(), 200);
    let tag = response.headers()["etag"].to_str().unwrap().to_owned();
    (response.json().await.unwrap(), tag)
}
async fn policy(client: &reqwest::Client, base: &str, key: &str, value: Value) {
    let url = format!("{base}/admin/settings/{key}");
    let (_, tag) = read(client, &url).await;
    let response = client
        .put(url)
        .header("if-match", tag)
        .json(&value)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
}
#[tokio::test]
async fn customization_is_owner_bound_cas_gated_and_independently_sandboxed() {
    let upstream = serve(mw_mock_jmap::router()).await;
    let (router, _) = build_app_full(
        AppConfig {
            db_path: common::test_db::unique_db_path("appearance-customization")
                .to_string_lossy()
                .into(),
            server_key_hex: None,
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(upstream.clone()),
                jmap_upstreams: Some(vec![upstream]),
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
    let admin = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    let user = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    assert_eq!(
        user.get(format!(
            "{base}/api/account/appearance/customization?accountId=x"
        ))
        .send()
        .await
        .unwrap()
        .status(),
        401
    );
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
    // Console and mailbox share this source IP; respect the sign-in cooldown.
    tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    let login = user
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200);
    let me: Value = user
        .get(format!("{base}/api/me"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let authority = me["storageAuthority"].as_str().unwrap();
    let account = login.json::<Value>().await.unwrap()["accountId"]
        .as_str()
        .unwrap()
        .to_owned();
    let endpoint = format!("{base}/api/account/appearance/customization?accountId={account}");
    let (initial, tag) = read(&user, &endpoint).await;
    assert_eq!(initial["cssEnabled"], false);
    assert_eq!(initial["scriptEnabled"], false);
    let document = json!({"version":1,"css":"body{color:red}","script":"appearance.set({density:'compact'});"});
    let mutation = json!({"ownerId":account,"document":document});
    assert_eq!(
        user.put(&endpoint)
            .header("x-mailwoman-storage-authority", authority)
            .json(&mutation)
            .send()
            .await
            .unwrap()
            .status(),
        428
    );
    let saved = user
        .put(&endpoint)
        .header("x-mailwoman-storage-authority", authority)
        .header("if-match", &tag)
        .json(&mutation)
        .send()
        .await
        .unwrap();
    assert_eq!(saved.status(), 200);
    let saved = saved.json::<Value>().await.unwrap();
    assert_eq!(saved["document"], document);
    assert_eq!(saved["compiledCss"], "");
    assert_eq!(
        user.put(&endpoint)
            .header("x-mailwoman-storage-authority", authority)
            .header("if-match", &tag)
            .json(&mutation)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    assert_eq!(
        user.get(format!(
            "{base}/api/account/appearance/customization?accountId=foreign"
        ))
        .send()
        .await
        .unwrap()
        .status(),
        409
    );
    let (_, tag) = read(&user, &endpoint).await;
    assert_eq!(
        user.put(&endpoint)
            .header("x-mailwoman-storage-authority", authority)
            .header("if-match", &tag)
            .json(&json!({"ownerId":"foreign","document":document}))
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    assert_eq!(user.put(&endpoint).header("x-mailwoman-storage-authority", authority).header("if-match",&tag).json(&json!({"ownerId":account,"document":{"version":1,"css":"@import 'https://evil.test/x';","script":""}})).send().await.unwrap().status(),422);
    policy(
        &admin,
        &base,
        "appearance.customCssEnabled",
        json!({"globalValue":true}),
    )
    .await;
    policy(
        &admin,
        &base,
        "appearance.customJsEnabled",
        json!({"globalValue":true}),
    )
    .await;
    let (enabled, tag) = read(&user, &endpoint).await;
    assert_eq!(enabled["cssEnabled"], true);
    assert_eq!(enabled["scriptEnabled"], true);
    assert!(
        enabled["compiledCss"]
            .as_str()
            .unwrap()
            .starts_with(".mw-custom-mail{")
    );
    let left = user.put(&endpoint).header("if-match", &tag).header("x-mailwoman-storage-authority", authority).json(
        &json!({"ownerId":account,"document":{"version":1,"css":"body{color:blue}","script":""}}),
    );
    let right = user.put(&endpoint).header("if-match", &tag).header("x-mailwoman-storage-authority", authority).json(
        &json!({"ownerId":account,"document":{"version":1,"css":"body{color:green}","script":""}}),
    );
    let (left, right) = tokio::join!(left.send(), right.send());
    let mut statuses = [
        left.unwrap().status().as_u16(),
        right.unwrap().status().as_u16(),
    ];
    statuses.sort();
    assert_eq!(statuses, [200, 412]);
    policy(
        &admin,
        &base,
        "appearance.accent",
        json!({"globalValue":"blue","locked":true}),
    )
    .await;
    let held_accent = user
        .put(format!("{base}/api/account/appearance"))
        .header("x-mailwoman-storage-authority", authority)
        .json(
            &json!({"appearance":{"accentId":"red","customAccent":"#ff0000","density":"compact"}}),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(held_accent.status(), 200);
    let held: Value = user
        .get(format!("{base}/api/account/appearance"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(held["appearance"]["accentId"], "blue");
    assert_eq!(held["appearance"]["customAccent"], "");
    assert_eq!(held["appearance"]["density"], "compact");
    let (locked, _) = read(&user, &endpoint).await;
    assert_eq!(locked["cssAllowed"], true);
    assert_eq!(locked["cssEnabled"], false);
    assert_eq!(locked["compiledCss"], "");
    assert_eq!(locked["lockedFields"], json!(["accent"]));
    let admin_endpoint = format!("{base}/admin/me/appearance-customization?username=root");
    let (own, tag) = read(&admin, &admin_endpoint).await;
    assert_eq!(own["document"]["css"], "");
    assert_eq!(own["cssEnabled"], true);
    assert_eq!(
        admin
            .put(&admin_endpoint)
            .header("if-match", tag)
            .json(&json!({"ownerId":own["ownerId"],"document":document}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        admin
            .get(format!(
                "{base}/admin/me/appearance-customization?username=other"
            ))
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    let (mail, _) = read(&user, &endpoint).await;
    assert_ne!(mail["document"]["css"], document["css"]);
    let (_, tag) = read(&admin, &format!("{base}/admin/me/preferences")).await;
    let left = admin
        .put(format!("{base}/admin/me/preferences"))
        .header("if-match", &tag)
        .json(&json!({"density":"compact"}));
    let right = admin
        .put(format!("{base}/admin/me/preferences"))
        .header("if-match", &tag)
        .json(&json!({"density":"cozy"}));
    let (left, right) = tokio::join!(left.send(), right.send());
    let mut statuses = [
        left.unwrap().status().as_u16(),
        right.unwrap().status().as_u16(),
    ];
    statuses.sort();
    assert_eq!(statuses, [200, 412]);
    policy(
        &admin,
        &base,
        "appearance.customJsEnabled",
        json!({"globalValue":false}),
    )
    .await;
    assert_eq!(read(&user, &endpoint).await.0["scriptEnabled"], false);
    let frame = user
        .get(format!("{base}/api/appearance/sandbox"))
        .send()
        .await
        .unwrap();
    assert_eq!(frame.status(), 200);
    let csp = frame.headers()["content-security-policy"]
        .to_str()
        .unwrap()
        .to_owned();
    for rule in [
        "sandbox allow-scripts",
        "connect-src 'none'",
        "worker-src blob:",
        "frame-ancestors 'self'",
    ] {
        assert!(csp.contains(rule));
    }
    assert!(!csp.contains("allow-same-origin"));
    assert_eq!(
        frame.headers()["cross-origin-resource-policy"],
        "cross-origin"
    );
    assert_eq!(
        frame.headers()["cross-origin-embedder-policy"],
        "credentialless"
    );
    let body = frame.text().await.unwrap();
    let script = body
        .split_once("<script>")
        .unwrap()
        .1
        .split_once("</script>")
        .unwrap()
        .0;
    use base64::Engine as _;
    use sha2::Digest as _;
    assert!(
        !script.contains('\r'),
        "HTML parser line-ending normalization must not invalidate CSP hashes"
    );
    let hash =
        base64::engine::general_purpose::STANDARD.encode(sha2::Sha256::digest(script.as_bytes()));
    assert!(csp.contains(&format!("'sha256-{hash}'")));
    let shell = user
        .get(format!("{base}/healthz"))
        .header("appearance-frame", "true")
        .send()
        .await
        .unwrap();
    assert_eq!(shell.headers()["x-frame-options"], "DENY");
    assert!(
        !shell.headers()["content-security-policy"]
            .to_str()
            .unwrap()
            .contains(" 'unsafe-eval'")
    );
}
