//! Domain edits preserve compatibility, require authority and reject stale writes.
use mw_server::{AppConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;

async fn server() -> (String, reqwest::Client) {
    let (app, _) = build_app_full(
        AppConfig {
            db_path: common::test_db::unique_db_path("domain-admin")
                .to_string_lossy()
                .into(),
            server_key_hex: None,
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: Default::default(),
            security: SecurityConfig::default(),
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
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap();
    assert_eq!(
        client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    (base, client)
}

#[tokio::test]
async fn domain_editor_validates_previews_versions_and_protects_preference() {
    let (base, c) = server().await;
    let status = c
        .get(format!("{base}/admin/domains/status"))
        .send()
        .await
        .unwrap();
    assert_eq!(status.status(), 200);
    let status: Value = status.json().await.unwrap();
    assert_eq!(status["mode"], "proxy");
    assert_eq!(status["blockedAccounts"]["total"], 0);
    let path = format!("{base}/admin/domains/example.org");
    assert_eq!(
        c.put(&path)
            .json(&json!({"name":"example.org","upstreamJson":"{}"}))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    let response = c.get(&path).send().await.unwrap();
    assert_eq!(response.status(), 200);
    let etag = response.headers()["etag"].to_str().unwrap().to_string();
    let detail: Value = response.json().await.unwrap();
    assert_eq!(detail["mode"], "proxy");
    assert_eq!(c.put(&path).json(&json!({"upstream":{"imap":{"host":"mail.example.org","port":993,"tls":"implicit"}}}))
        .send().await.unwrap().status(),422);
    let change = json!({"upstream":{"jmap":{"sessionUrl":"https://mail.example.org/.well-known/jmap"}},"allowlist":["*@example.org"]});
    let mut preview = change.clone();
    preview["dryRun"] = json!(true);
    let response = c
        .put(&path)
        .header("if-match", &etag)
        .json(&preview)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.json::<Value>().await.unwrap()["dryRun"], true);
    assert_eq!(c.get(&path).send().await.unwrap().headers()["etag"], etag);
    assert_eq!(
        c.put(&path)
            .header("if-match", &etag)
            .json(&change)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        c.put(&path)
            .header("if-match", &etag)
            .json(&json!({"enabled":false}))
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    assert_eq!(
        c.put(format!("{path}/preferred"))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(c.delete(&path).send().await.unwrap().status(), 409);
    assert_eq!(
        c.put(format!("{path}/aliases/alias.test"))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    let detail: Value = c.get(&path).send().await.unwrap().json().await.unwrap();
    assert_eq!(detail["aliases"], json!(["alias.test"]));
    let listed: Value = c
        .get(format!("{base}/admin/domains"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(listed[0]["aliases"], json!(["alias.test"]));
    assert_eq!(
        c.delete(format!("{path}?force=true"))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(c.get(&path).send().await.unwrap().status(), 404);
}

#[tokio::test]
async fn unauthenticated_domain_mutations_and_discovery_never_run() {
    let (base, _) = server().await;
    let c = reqwest::Client::builder().no_proxy().build().unwrap();
    for (method, suffix) in [
        (reqwest::Method::PUT, ""),
        (reqwest::Method::DELETE, ""),
        (reqwest::Method::POST, "/detect"),
        (reqwest::Method::POST, "/enable"),
        (reqwest::Method::POST, "/disable"),
        (reqwest::Method::PUT, "/preferred"),
        (reqwest::Method::PUT, "/aliases/alias.test"),
        (reqwest::Method::PUT, "/hosts/mail.test"),
    ] {
        assert_eq!(
            c.request(method, format!("{base}/admin/domains/example.org{suffix}"))
                .send()
                .await
                .unwrap()
                .status(),
            401,
            "{suffix}"
        );
    }
}

#[tokio::test]
async fn mappings_and_global_preference_have_independent_cas_and_previews() {
    let (base, c) = server().await;
    let a = format!("{base}/admin/domains/a.test");
    let b = format!("{base}/admin/domains/b.test");
    for path in [&a, &b] {
        assert_eq!(c.put(path).send().await.unwrap().status(), 204);
    }
    let detail: Value = c.get(&a).send().await.unwrap().json().await.unwrap();
    let p = detail["preferenceEtag"].as_str().unwrap();
    let m = detail["mappingEtag"].as_str().unwrap();
    assert_eq!(
        c.put(format!("{a}/preferred"))
            .header("if-match", p)
            .json(&json!({"dryRun":true}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let still: Value = c.get(&a).send().await.unwrap().json().await.unwrap();
    assert_eq!(still["preferred"], false);
    assert_eq!(
        c.put(format!("{a}/preferred"))
            .header("if-match", p)
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(
        c.put(format!("{b}/preferred"))
            .header("if-match", p)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    assert_eq!(
        c.delete(format!("{base}/admin/preferred-domain"))
            .header("if-match", p)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    // Preference change is part of the domain snapshot underlying mappings.
    assert_eq!(
        c.put(format!("{a}/aliases/alias.test"))
            .header("if-match", m)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    let detail: Value = c.get(&a).send().await.unwrap().json().await.unwrap();
    let m = detail["mappingEtag"].as_str().unwrap();
    let p = detail["preferenceEtag"].as_str().unwrap();
    assert_eq!(
        c.put(format!("{a}/aliases/alias.test"))
            .header("if-match", m)
            .json(&json!({"dryRun":true}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let still: Value = c.get(&a).send().await.unwrap().json().await.unwrap();
    assert_eq!(still["aliases"], json!([]));
    assert_eq!(
        c.put(format!("{a}/aliases/alias.test"))
            .header("if-match", m)
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(
        c.put(format!("{a}/hosts/login.test"))
            .header("if-match", m)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    assert_eq!(
        c.put(format!("{b}/aliases/alias.test"))
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    assert_eq!(
        c.put(format!("{b}/aliases/a.test"))
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    assert_eq!(
        c.put(format!("{base}/admin/domains/alias.test"))
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    assert_eq!(
        c.delete(format!("{base}/admin/preferred-domain?dryRun=true"))
            .header("if-match", p)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let still: Value = c.get(&a).send().await.unwrap().json().await.unwrap();
    assert_eq!(still["preferred"], true);
    assert_eq!(
        c.delete(format!("{base}/admin/preferred-domain"))
            .header("if-match", p)
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
}

#[tokio::test]
async fn alias_rename_checks_snapshot_and_preserves_parent() {
    let (base, c) = server().await;
    let path = format!("{base}/admin/domains/parent.test");
    assert_eq!(
        c.put(&path)
            .json(&json!({"name":"parent.test","upstreamJson":"{}"}))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    let detail: Value = c.get(&path).send().await.unwrap().json().await.unwrap();
    assert_eq!(
        c.put(format!("{path}/aliases/old.test"))
            .header("if-match", detail["mappingEtag"].as_str().unwrap())
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    let detail: Value = c.get(&path).send().await.unwrap().json().await.unwrap();
    let version = detail["mappingEtag"].as_str().unwrap();
    let rename = format!("{path}/aliases/old.test");
    assert_eq!(
        c.patch(&rename)
            .json(&json!({"name":"new.test"}))
            .send()
            .await
            .unwrap()
            .status(),
        428
    );
    assert_eq!(
        c.patch(&rename)
            .header("if-match", version)
            .json(&json!({"name":"new.test"}))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(
        c.patch(&rename)
            .header("if-match", version)
            .json(&json!({"name":"late.test"}))
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    let fresh: Value = c.get(&path).send().await.unwrap().json().await.unwrap();
    assert_eq!(fresh["aliases"], json!(["new.test"]));
    assert_eq!(fresh["upstream"], detail["upstream"]);
}
