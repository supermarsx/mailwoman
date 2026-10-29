//! t39 R7 — trusted link domains (`links.trustedDomains`,
//! `links.trustedSubdomains`).
//!
//! * The matcher agrees with the web reader on the shared vectors
//!   (`apps/web/src/viewers/linkTrust.vectors.json`): exact domain, optional
//!   subdomains on a label boundary, never a lookalike suffix, an IP literal
//!   or a single-label entry.
//! * The registry stores only canonical entries and is global/domain/user
//!   with user overrides, so an administrator can set a default and lock it.
//! * Over the real router: the admin default reaches the user, a lookalike or
//!   wildcard entry is refused naming the item, and a lock refuses the user's
//!   write and keeps the administrator's list.

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::settings_registry::{self, Scopes, link_trust};
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};

mod common;
use common::test_db;

const KEY_HEX: &str = "5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

fn vectors() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/web/src/viewers/linkTrust.vectors.json");
    serde_json::from_str(&std::fs::read_to_string(&path).expect("shared vectors")).unwrap()
}

#[test]
fn entries_normalize_as_the_web_reader_does() {
    let v = vectors();
    let entries = v["entries"].as_array().unwrap();
    assert!(entries.len() >= 10);
    for case in entries {
        let input = case["input"].as_str().unwrap();
        let want = case["normalized"].as_str();
        assert_eq!(link_trust::normalize(input).as_deref(), want, "{input:?}");
    }
}

#[test]
fn hosts_match_on_whole_labels_as_the_web_reader_does() {
    let v = vectors();
    let matches = v["matches"].as_array().unwrap();
    assert!(matches.len() >= 10);
    for case in matches {
        let host = case["host"].as_str().unwrap();
        let domains: Vec<&str> = case["domains"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d.as_str().unwrap())
            .collect();
        let subdomains = case["subdomains"].as_bool().unwrap();
        assert_eq!(
            link_trust::is_trusted(host, &domains, subdomains),
            case["trusted"].as_bool().unwrap(),
            "{host} against {domains:?} (subdomains {subdomains})"
        );
    }
}

#[test]
fn lookalike_suffixes_never_match_even_with_subdomains() {
    let trusted = ["example.com"];
    for host in [
        "evilexample.com",
        "example.com.evil.test",
        "example-com.test",
        "xexample.com",
        "example.comm",
        "example.co",
        "com",
    ] {
        assert!(!link_trust::is_trusted(host, &trusted, true), "{host}");
    }
    assert!(link_trust::is_trusted(
        "deep.mail.example.com",
        &trusted,
        true
    ));
    assert!(!link_trust::is_trusted(
        "deep.mail.example.com",
        &trusted,
        false
    ));
}

#[test]
fn the_registry_keys_are_user_overridable_and_store_only_canonical_entries() {
    let list = settings_registry::lookup("links.trustedDomains").expect("links.trustedDomains");
    let sub =
        settings_registry::lookup("links.trustedSubdomains").expect("links.trustedSubdomains");
    for def in [list, sub] {
        assert_eq!(def.scopes, Scopes::GlobalDomainUser, "{}", def.key);
        assert!(def.user_override, "{}", def.key);
    }
    assert_eq!(list.default, "[]");
    assert_eq!(sub.default, "false");

    let ok = settings_registry::validate(
        list,
        &json!(["example.com", "xn--bcher-kva.example"]),
        "value",
    );
    assert_eq!(ok.unwrap(), json!(["example.com", "xn--bcher-kva.example"]));
    for bad in [
        json!(["*.example.com"]),
        json!(["com"]),
        json!(["10.0.0.1"]),
        json!(["-example.com"]),
        json!(["example..com"]),
        json!(["Example.com"]),
        json!(["example.com."]),
        json!(["example.com", "example.com"]),
    ] {
        assert!(
            settings_registry::validate(list, &bad, "value").is_err(),
            "{bad}"
        );
    }
    let too_many: Vec<String> = (0..101).map(|i| format!("d{i}.example")).collect();
    assert!(settings_registry::validate(list, &json!(too_many), "value").is_err());
}

// ── over the real router ───────────────────────────────────────────────────────

fn config(db: &str, upstream: String) -> AppConfig {
    let web: PathBuf = PathBuf::from(db).parent().unwrap().join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    AppConfig {
        db_path: db.to_string(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(vec![upstream.clone()]),
            upstream_default: Some(upstream),
            ..SecurityConfig::default()
        },
    }
}

async fn serve(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{addr}")
}

async fn server() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    let upstream = format!("http://{addr}");
    let db = test_db::unique_dir("mw-t39-link-trust")
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let admin = V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    };
    serve(
        build_app_full(config(&db, upstream), admin)
            .await
            .expect("server boots")
            .0,
    )
    .await
}

fn client(
    headers: reqwest::header::HeaderMap,
    jar: std::sync::Arc<reqwest::cookie::Jar>,
) -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_provider(jar)
        .default_headers(headers)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

async fn admin(base: &str) -> reqwest::Client {
    let c = client(Default::default(), Default::default());
    let r = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "admin login");
    c
}

async fn user(base: &str) -> reqwest::Client {
    let jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
    let login = client(Default::default(), jar.clone());
    let me: Value = login
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(me["ok"], json!(true), "mailbox login: {me}");
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "x-mailwoman-storage-authority",
        me["storageAuthority"].as_str().unwrap().parse().unwrap(),
    );
    client(headers, jar)
}

async fn send(req: reqwest::RequestBuilder) -> (u16, Value) {
    let r = req.send().await.unwrap();
    let status = r.status().as_u16();
    let text = r.text().await.unwrap();
    (
        status,
        serde_json::from_str(&text).unwrap_or(Value::String(text)),
    )
}

async fn entry(u: &reqwest::Client, base: &str, key: &str) -> Value {
    let (status, body) = send(u.get(format!("{base}/api/settings"))).await;
    assert_eq!(status, 200, "{body}");
    body["settings"][key].clone()
}

#[tokio::test]
async fn an_admin_default_and_lock_govern_the_users_trusted_link_domains() {
    let base = &server().await;
    let a = admin(base).await;
    let u = user(base).await;
    let list = "links.trustedDomains";
    let sub = "links.trustedSubdomains";

    // Built-in defaults: nothing is trusted.
    assert_eq!(entry(&u, base, list).await["value"], json!([]));
    assert_eq!(entry(&u, base, sub).await["value"], json!(false));

    // The administrator's default reaches the user, unlocked.
    let (status, body) = send(
        a.put(format!("{base}/admin/settings/{list}"))
            .json(&json!({ "globalValue": ["corp.example"], "locked": false })),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let e = entry(&u, base, list).await;
    assert_eq!(e["value"], json!(["corp.example"]));
    assert_eq!(e["locked"], json!(false));

    // A lookalike-shaped entry is refused and names the item; nothing is stored.
    for bad in [
        "*.partner.example",
        "partner",
        "10.1.2.3",
        "https://partner.example",
    ] {
        let (status, body) = send(
            u.put(format!("{base}/api/settings/{list}"))
                .json(&json!({ "value": ["partner.example", bad] })),
        )
        .await;
        assert_eq!(status, 422, "{bad}: {body}");
        assert_eq!(body["field"], json!("/value/1"), "{bad}: {body}");
        assert_eq!(body["code"], json!("invalidValue"), "{bad}: {body}");
    }
    assert_eq!(
        entry(&u, base, list).await["value"],
        json!(["corp.example"])
    );

    // Control: while free, the user's own list and switch are stored.
    let (status, body) = send(
        u.put(format!("{base}/api/settings/{list}"))
            .json(&json!({ "value": ["corp.example", "partner.example"] })),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let (status, body) = send(
        u.put(format!("{base}/api/settings/{sub}"))
            .json(&json!({ "value": true })),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        entry(&u, base, list).await["value"],
        json!(["corp.example", "partner.example"])
    );
    assert_eq!(entry(&u, base, sub).await["value"], json!(true));

    // Locked: the administrator's values win and the user's writes are refused.
    for (key, value) in [(list, json!(["corp.example"])), (sub, json!(false))] {
        let (status, body) = send(
            a.put(format!("{base}/admin/settings/{key}"))
                .json(&json!({ "globalValue": value, "locked": true })),
        )
        .await;
        assert_eq!(status, 200, "{key}: {body}");
    }
    let (status, body) = send(
        u.put(format!("{base}/api/settings/{list}"))
            .json(&json!({ "value": ["evil.example"] })),
    )
    .await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "setting_locked");
    let (status, body) = send(
        u.put(format!("{base}/api/settings/{sub}"))
            .json(&json!({ "value": true })),
    )
    .await;
    assert_eq!(status, 403, "{body}");
    let e = entry(&u, base, list).await;
    assert_eq!(e["value"], json!(["corp.example"]));
    assert_eq!(e["locked"], json!(true));
    let e = entry(&u, base, sub).await;
    assert_eq!(e["value"], json!(false));
    assert_eq!(e["locked"], json!(true));
}
