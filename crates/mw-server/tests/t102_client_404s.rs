//! t102 — clients of an older build do not hit 404s the server can avoid.
//!
//! 1. **Retained assets.** Start on build A, then start again on build B with A's
//!    files gone from the web directory: A's hashed chunks are still served, with
//!    the immutable headers they had, from the retained directory beside the
//!    database. A name that never existed is still a 404. `index.html` stays
//!    `no-cache`.
//! 2. **Worker script URLs.** `/sw.js` is the real worker (t63 manifest kept). A
//!    worker-script fetch of a `sw.js` the build does not have (an earlier deploy
//!    prefix, or a web directory without it) gets the self-removing cleanup worker;
//!    the same URL without the browser's worker-script headers is still a 404.
//! 3. **Renew.** `POST /api/accounts/{id}/renew` is a mounted route behind the
//!    session check (no alias is needed): unauthenticated it is 401, not 404.
//! 4. **Statistics.** The not-found table tags each path `client` or `external`.
use std::path::{Path, PathBuf};

use mw_server::{AppConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};

mod common;
use common::test_db;

const KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

fn shell(entry: &str) -> String {
    format!(
        "<!doctype html><html><head><meta charset=\"UTF-8\" />\
<script type=\"module\" crossorigin src=\"./assets/{entry}\"></script></head>\
<body><div id=\"root\"></div></body></html>"
    )
}

fn write_build(web: &Path, files: &[(&str, &str)], entry: &str) {
    let _ = std::fs::remove_dir_all(web);
    std::fs::create_dir_all(web.join("assets")).unwrap();
    std::fs::write(web.join("index.html"), shell(entry)).unwrap();
    for (name, body) in files {
        std::fs::write(web.join("assets").join(name), body).unwrap();
    }
    let sw = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/web/public/sw.js");
    std::fs::write(web.join("sw.js"), std::fs::read(sw).unwrap()).unwrap();
}

async fn start(db: &Path, web: &Path) -> String {
    let (app, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into_owned(),
            server_key_hex: Some(KEY_HEX.into()),
            web_dir: Some(web.to_path_buf()),
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: Default::default(),
            security: SecurityConfig::default(),
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("synthetic-test-password".into()),
            redis_url: None,
        },
    )
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    base
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn get(c: &reqwest::Client, url: &str, headers: &[(&str, &str)]) -> reqwest::Response {
    let mut req = c.get(url);
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    req.send().await.unwrap()
}

const WORKER_FETCH: &[(&str, &str)] = &[
    ("service-worker", "script"),
    ("sec-fetch-dest", "serviceworker"),
    ("sec-fetch-mode", "same-origin"),
];

#[tokio::test]
async fn older_builds_keep_loading_and_stale_worker_urls_clean_up() {
    // SAFETY: this binary has one test; nothing else reads these meanwhile.
    unsafe {
        std::env::remove_var("MW_BASE_PATH");
        std::env::remove_var("MW_RETAINED_ASSETS_DIR");
    }
    let dir: PathBuf = test_db::unique_dir("t102-client-404s");
    let web = dir.join("web");
    let db = dir.join("mw.db");
    let c = client();

    // Build A.
    write_build(
        &web,
        &[
            ("index-AAAA1111.js", "/* entry A */"),
            ("Dashboard-CdQvJhSt.js", "export default 'dashboard A';"),
            ("dashboard-Ab12Cd34.css", ".a{}"),
        ],
        "index-AAAA1111.js",
    );
    let a = start(&db, &web).await;
    let r = get(&c, &format!("{a}/assets/Dashboard-CdQvJhSt.js"), &[]).await;
    assert_eq!(r.status(), 200);
    assert!(
        dir.join("retained-assets/assets/Dashboard-CdQvJhSt.js")
            .exists(),
        "build A retained at start"
    );

    // Build B replaces A in the web directory (a deploy), on a new start.
    write_build(
        &web,
        &[
            ("index-BBBB2222.js", "/* entry B */"),
            ("Dashboard-Zz9Yy8Xx.js", "export default 'dashboard B';"),
        ],
        "index-BBBB2222.js",
    );
    let b = start(&db, &web).await;
    // A tab still on build A lazy-loads its chunk: served, immutable, same bytes.
    let r = get(
        &c,
        &format!("{b}/assets/Dashboard-CdQvJhSt.js"),
        &[("sec-fetch-dest", "script")],
    )
    .await;
    assert_eq!(r.status(), 200, "retained chunk of build A");
    assert_eq!(
        r.headers()["cache-control"],
        "public, max-age=31536000, immutable"
    );
    assert!(
        r.headers()["content-type"]
            .to_str()
            .unwrap()
            .contains("javascript")
    );
    assert_eq!(r.text().await.unwrap(), "export default 'dashboard A';");
    assert_eq!(
        get(&c, &format!("{b}/assets/dashboard-Ab12Cd34.css"), &[])
            .await
            .status(),
        200
    );
    // Build B's own files, and a hashed name no build ever had.
    assert_eq!(
        get(&c, &format!("{b}/assets/Dashboard-Zz9Yy8Xx.js"), &[])
            .await
            .status(),
        200
    );
    assert_eq!(
        get(&c, &format!("{b}/assets/Never-Qq1Ww2Ee.js"), &[])
            .await
            .status(),
        404
    );
    // The shell names B's entry and is never kept stale.
    let r = get(&c, &format!("{b}/index.html"), &[]).await;
    assert_eq!(r.headers()["cache-control"], "no-cache");
    assert!(r.text().await.unwrap().contains("index-BBBB2222.js"));
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(dir.join("retained-assets/builds.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["builds"].as_array().unwrap().len(), 2);

    // The real worker at /sw.js: the t63 manifest the build wrote, no cleanup.
    let r = get(&c, &format!("{b}/sw.js"), WORKER_FETCH).await;
    assert_eq!(r.status(), 200);
    assert!(r.headers().get("x-mailwoman-worker").is_none());
    assert_eq!(r.headers()["cache-control"], "no-cache");
    let body = r.text().await.unwrap();
    assert!(body.contains("ASSET_DIGESTS"), "the real worker");
    // A registration from an earlier deploy prefix: cleanup worker.
    let r = get(&c, &format!("{b}/mail/sw.js"), WORKER_FETCH).await;
    assert_eq!(r.status(), 200);
    assert_eq!(r.headers()["x-mailwoman-worker"], "cleanup");
    assert_eq!(r.headers()["cache-control"], "no-cache");
    assert!(
        r.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/javascript")
    );
    let cleanup = r.text().await.unwrap();
    assert!(cleanup.contains("registration.unregister()"));
    assert!(!cleanup.contains("'fetch'"));
    // Same URL, not a worker-script fetch (a scanner, a curl): still a 404.
    assert_eq!(get(&c, &format!("{b}/mail/sw.js"), &[]).await.status(), 404);
    // A web directory without sw.js: browsers are told to drop the worker.
    std::fs::remove_file(web.join("sw.js")).unwrap();
    let r = get(&c, &format!("{b}/sw.js"), WORKER_FETCH).await;
    assert_eq!(r.status(), 200);
    assert_eq!(r.headers()["x-mailwoman-worker"], "cleanup");
    assert_eq!(get(&c, &format!("{b}/sw.js"), &[]).await.status(), 404);

    // Renew is mounted and behind the session check: 401, never a route 404.
    let r = c
        .post(format!(
            "{b}/api/accounts/550e8400-e29b-41d4-a716-446655440000/renew"
        ))
        .header("origin", &b)
        .json(&json!({"password":"x"}))
        .send()
        .await
        .unwrap();
    // 401 (no session) or 403 (the write guard, before any handler): never 404.
    assert!(
        matches!(r.status().as_u16(), 401 | 403),
        "renew requires a session: {}",
        r.status()
    );

    // Statistics: one client miss and one probe, tagged apart.
    for _ in 0..2 {
        get(&c, &format!("{b}/assets/Gone-Pp0Oo9Ii.js"), &[]).await;
        get(&c, &format!("{b}/wp-login.php"), &[]).await;
    }
    assert_eq!(
        c.post(format!("{b}/admin/login"))
            .json(&json!({"username":"root","password":"synthetic-test-password"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    // Let the periodic writer flush the middleware's buckets.
    tokio::time::sleep(std::time::Duration::from_secs(17)).await;
    let body: Value = c
        .get(format!("{b}/admin/metrics/not-found"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let source_of = |path: &str| {
        body["paths"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["path"] == path)
            .map(|row| row["source"].clone())
    };
    assert_eq!(
        source_of("/assets/Gone-Pp0Oo9Ii.js"),
        Some(json!("client")),
        "{body}"
    );
    assert_eq!(
        source_of("/wp-login.php"),
        Some(json!("external")),
        "{body}"
    );
    assert!(
        body["listedBySource"]["client"].as_u64().unwrap() >= 2,
        "{body}"
    );
    assert!(
        body["listedBySource"]["external"].as_u64().unwrap() >= 2,
        "{body}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
