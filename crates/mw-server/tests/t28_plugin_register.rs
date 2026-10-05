//! t28-e14 (audit §5.2, §13 rows 16-17) — an engine plugin can be registered, and
//! what it then runs with is what the administrator granted.
//!
//! Before 26.20 there was no registration route at all (`plugins` rows had no
//! writer outside tests), the loader built every grant from the manifest's whole
//! capability list without reading `plugin_grants`, and every load passed
//! `allow_unsigned: true`. So the first plugin anyone managed to register would have
//! run with every capability it declared, whatever the grant screen said.
//!
//! Every leg drives the real router over HTTP in engine mode and loads the real
//! `plugins/dist/spam-rspamd.wasm` into the wasmtime host. "The guest may not use
//! the network" is measured at a loopback HTTP server standing in for rspamd: a
//! refused call is no request arriving there, not a flag reading back.
//!
//! Run:
//!   cargo test -p mw-server --test t28_plugin_register --locked -- --test-threads=1

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};

mod common;
use common::test_db;

const KEY_HEX: &str = "28e14a1b2c3d4e5f60718293a4b5c6d728e14a1b2c3d4e5f60718293a4b5c6d7";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T28</div>";

/// The seven first-party components `plugins/dist/` ships.
const FIRST_PARTY: [&str; 7] = [
    "bridge-ews",
    "bridge-gmail",
    "bridge-graph",
    "languagetool",
    "nextcloud",
    "spam-rspamd",
    "spam-spamassassin",
];

// ── harness ──────────────────────────────────────────────────────────────────

fn admin_v6() -> V6Config {
    V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    }
}

/// The directory third-party components are read from. `MW_THIRDPARTY_PLUGIN_DIR`
/// is process-wide, so it is set once for this binary, before any server boots;
/// the gate runs these tests on one thread.
fn third_party_dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = test_db::unique_dir("mw-t28-e14-thirdparty");
        std::fs::create_dir_all(&dir).unwrap();
        // SAFETY: set before the first server is built, single-threaded test binary.
        unsafe { std::env::set_var("MW_THIRDPARTY_PLUGIN_DIR", &dir) };
        dir
    })
}

async fn boot(mode: ServerMode) -> String {
    third_party_dir();
    let dir = test_db::unique_dir("mw-t28-e14-plugins");
    let web = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let app = mw_server::build_app_full(config, admin_v6())
        .await
        .expect("build_app_full")
        .0;
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

fn browser() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

/// An admin session against `base`.
struct Admin {
    http: reqwest::Client,
    base: String,
}

impl Admin {
    async fn login(base: &str) -> Self {
        let http = browser();
        let r = http
            .post(format!("{base}/admin/login"))
            .json(&json!({ "username": "root", "password": "hunter2" }))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 200, "admin login");
        Self {
            http,
            base: base.to_string(),
        }
    }

    /// POST `body` (or nothing) and return the status and the JSON answer.
    async fn post(&self, path: &str, body: Option<Value>) -> (u16, Value) {
        let mut req = self.http.post(format!("{}{path}", self.base));
        if let Some(body) = body {
            req = req.json(&body);
        }
        let r = req.send().await.unwrap();
        let status = r.status().as_u16();
        (status, r.json().await.unwrap_or(Value::Null))
    }

    /// POST that must answer `200`/`201` with `{ "plugin": … }`; returns the plugin.
    async fn change(&self, path: &str, body: Option<Value>) -> Value {
        let (status, answer) = self.post(path, body).await;
        assert!(
            status == 200 || status == 201,
            "POST {path} answered {status}: {answer}"
        );
        answer["plugin"].clone()
    }

    async fn plugins(&self) -> Vec<Value> {
        let r = self
            .http
            .get(format!("{}/admin/plugins", self.base))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 200, "GET /admin/plugins");
        let body: Value = r.json().await.unwrap();
        body["plugins"].as_array().expect("plugins array").clone()
    }

    async fn plugin(&self, id: &str) -> Value {
        self.plugins()
            .await
            .into_iter()
            .find(|p| p["id"] == id)
            .unwrap_or_else(|| panic!("plugin '{id}' is not listed"))
    }

    /// The `change` names of the audit rows written for `id`, oldest first.
    async fn audit_changes(&self, id: &str) -> Vec<String> {
        let r = self
            .http
            .get(format!("{}/admin/audit", self.base))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 200, "GET /admin/audit");
        let body: Value = r.json().await.unwrap();
        let rows = body
            .as_array()
            .or_else(|| body["entries"].as_array())
            .or_else(|| body["audit"].as_array())
            .unwrap_or_else(|| panic!("audit answer is not a list: {body}"))
            .clone();
        let mut out: Vec<(String, String)> = rows
            .iter()
            .filter(|row| row["target"] == id)
            .filter_map(|row| {
                let detail = row
                    .get("detail")
                    .cloned()
                    .or_else(|| {
                        row.get("detailJson")
                            .and_then(Value::as_str)
                            .and_then(|s| serde_json::from_str(s).ok())
                    })
                    .unwrap_or(Value::Null);
                let detail = match detail {
                    Value::String(s) => serde_json::from_str(&s).unwrap_or(Value::Null),
                    other => other,
                };
                let change = detail.get("change")?.as_str()?.to_string();
                Some((row["ts"].as_str().unwrap_or_default().to_string(), change))
            })
            .collect();
        out.sort();
        out.into_iter().map(|(_, change)| change).collect()
    }
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap_or_else(|| panic!("not an array: {v}"))
        .iter()
        .map(|s| s.as_str().unwrap_or_default().to_string())
        .collect()
}

/// A loopback stand-in for the rspamd scan worker: answers every request with a
/// `reject` verdict and counts the requests that reach it.
async fn fake_rspamd() -> (u16, Arc<AtomicUsize>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let hits = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&hits);
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let seen = Arc::clone(&seen);
            tokio::spawn(async move {
                // Read the head and whatever body arrives with it; the answer does
                // not depend on the message.
                let mut buf = vec![0u8; 16 * 1024];
                let _ = sock.read(&mut buf).await;
                seen.fetch_add(1, Ordering::SeqCst);
                let body = r#"{"action":"reject","score":15.0,"required_score":6.0,"symbols":{}}"#;
                let head = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\
                     content-length: {}\r\nconnection: close\r\n\r\n",
                    body.len()
                );
                let _ = sock.write_all(head.as_bytes()).await;
                let _ = sock.write_all(body.as_bytes()).await;
                let _ = sock.shutdown().await;
            });
        }
    });
    (port, hits)
}

fn dist_wasm(id: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../plugins/dist")
        .join(format!("{id}.wasm"));
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// ── 1. registration, and the grant a loaded instance runs with ───────────────

#[tokio::test]
async fn a_registered_plugin_runs_with_the_stored_grant_and_nothing_more() {
    let base = boot(ServerMode::Engine).await;
    let (rspamd_port, rspamd_hits) = fake_rspamd().await;
    let admin = Admin::login(&base).await;

    // Precondition: nothing is registered, and the route is the admin's.
    assert!(admin.plugins().await.is_empty(), "registry starts empty");
    let anonymous = browser()
        .post(format!("{base}/admin/plugins"))
        .json(&json!({ "id": "spam-rspamd" }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        anonymous.status(),
        401,
        "registration needs an admin session"
    );
    assert!(admin.plugins().await.is_empty());

    // Register by id: the manifest is the server's, the hosts are this deployment's.
    let p = admin
        .change(
            "/admin/plugins",
            Some(json!({ "id": "spam-rspamd", "netAllowlist": ["127.0.0.1"] })),
        )
        .await;
    assert_eq!(p["firstParty"], true);
    assert_eq!(p["trust"], "first-party-digest");
    assert_eq!(
        strings(&p["capabilities"]),
        ["spam-action", "net", "store-kv-scoped"]
    );
    assert_eq!(strings(&p["netAllowlist"]), ["127.0.0.1"]);
    assert_eq!(
        p["limits"],
        json!({ "memoryMb": 32, "deadlineMs": 10000, "fuel": null })
    );
    assert_eq!(
        (&p["approved"], &p["enabled"]),
        (&json!(false), &json!(false))
    );
    assert!(
        strings(&p["granted"]).is_empty(),
        "registration grants nothing"
    );
    assert_eq!(
        (&p["loaded"], &p["notLoadedReason"]),
        (&json!(false), &json!("not-approved"))
    );

    let (status, again) = admin
        .post("/admin/plugins", Some(json!({ "id": "spam-rspamd" })))
        .await;
    assert_eq!(
        (status, &again["code"]),
        (409, &json!("already-registered"))
    );

    // Enable is refused before approval; approval alone loads nothing.
    let (status, early) = admin.post("/admin/plugins/spam-rspamd/enable", None).await;
    assert_eq!((status, &early["code"]), (400, &json!("not-approved")));
    let p = admin
        .change("/admin/plugins/spam-rspamd/approve", None)
        .await;
    assert_eq!(
        (&p["approved"], &p["notLoadedReason"]),
        (&json!(true), &json!("disabled"))
    );

    // Enabled with no grant row: not loaded. This is the line the unfixed loader
    // fails — it ran every enabled plugin with its whole manifest capability list.
    let p = admin
        .change("/admin/plugins/spam-rspamd/enable", None)
        .await;
    assert_eq!(p["enabled"], true);
    assert_eq!(
        (&p["loaded"], &p["notLoadedReason"]),
        (&json!(false), &json!("no-grant")),
        "an enabled plugin with no stored grant must not be loaded"
    );
    let (status, not_loaded) = admin.post("/admin/plugins/spam-rspamd/test", None).await;
    assert_eq!((status, &not_loaded["code"]), (409, &json!("not-loaded")));

    // Point the classifier at the stand-in daemon.
    let endpoint = format!("http://127.0.0.1:{rspamd_port}");
    let p = admin
        .change(
            "/admin/plugins/spam-rspamd/settings",
            Some(json!({ "endpoint": endpoint })),
        )
        .await;
    assert_eq!(p["endpoint"], json!(endpoint));

    // Grant everything it declares except `net`. It loads at once, without `net`.
    let p = admin
        .change(
            "/admin/plugins/spam-rspamd/grant",
            Some(json!({ "capabilities": ["spam-action", "store-kv-scoped"] })),
        )
        .await;
    assert_eq!(
        p["loaded"], true,
        "a granted, enabled classifier loads without a restart"
    );
    assert_eq!(
        (&p["restartRequired"], &p["notLoadedReason"]),
        (&json!(false), &Value::Null)
    );
    assert_eq!(
        strings(&p["loadedCapabilities"]),
        ["spam-action", "store-kv-scoped"],
        "the loaded instance's grant must not include net"
    );

    // The guest's network call is refused by the host: nothing reaches the daemon.
    let (status, verdict) = admin.post("/admin/plugins/spam-rspamd/test", None).await;
    assert_eq!(status, 200, "{verdict}");
    assert_eq!(verdict["verdict"], "unknown", "{verdict}");
    assert!(
        verdict["detail"]
            .to_string()
            .contains("net capability not granted"),
        "the guest reports the host's refusal: {verdict}"
    );
    assert_eq!(
        rspamd_hits.load(Ordering::SeqCst),
        0,
        "a guest without the net grant reached the network"
    );

    // A grant can name only what the manifest declares, from the closed set.
    for bad in [
        json!({ "capabilities": ["spam-action", "account-backend"] }),
        json!({ "capabilities": ["spam-action", "everything"] }),
        json!({ "capabilities": ["spam-action"], "widen": true }),
        json!({ "capability": "net" }),
    ] {
        let (status, _) = admin
            .post("/admin/plugins/spam-rspamd/grant", Some(bad.clone()))
            .await;
        assert_eq!(status, 400, "{bad} must be refused");
    }
    assert_eq!(
        strings(&admin.plugin("spam-rspamd").await["granted"]),
        ["spam-action", "store-kv-scoped"],
        "a refused grant request stores nothing"
    );

    // Grant `net`: the same call now reaches the daemon and returns its verdict.
    let p = admin
        .change(
            "/admin/plugins/spam-rspamd/grant",
            Some(json!({ "capabilities": ["spam-action", "store-kv-scoped", "net"] })),
        )
        .await;
    assert_eq!(
        strings(&p["loadedCapabilities"]),
        ["net", "spam-action", "store-kv-scoped"]
    );
    let (status, verdict) = admin.post("/admin/plugins/spam-rspamd/test", None).await;
    assert_eq!(
        (status, &verdict["verdict"]),
        (200, &json!("spam")),
        "{verdict}"
    );
    assert_eq!(rspamd_hits.load(Ordering::SeqCst), 1);

    // Leaving `net` out of the next grant revokes it, again without a restart.
    let p = admin
        .change(
            "/admin/plugins/spam-rspamd/grant",
            Some(json!({ "capabilities": ["spam-action", "store-kv-scoped"] })),
        )
        .await;
    assert_eq!(
        strings(&p["loadedCapabilities"]),
        ["spam-action", "store-kv-scoped"]
    );
    let (_, verdict) = admin.post("/admin/plugins/spam-rspamd/test", None).await;
    assert_eq!(verdict["verdict"], "unknown", "{verdict}");
    assert_eq!(rspamd_hits.load(Ordering::SeqCst), 1, "net was revoked");

    // A first-party component has no allow-unsigned flag to set.
    let (status, refused) = admin
        .post(
            "/admin/plugins/spam-rspamd/allow-unsigned",
            Some(json!({ "allow": true })),
        )
        .await;
    assert_eq!((status, &refused["code"]), (400, &json!("first-party")));

    // Disable unloads it; uninstall removes the registration.
    let p = admin
        .change("/admin/plugins/spam-rspamd/disable", None)
        .await;
    assert_eq!(
        (&p["loaded"], &p["notLoadedReason"]),
        (&json!(false), &json!("disabled"))
    );
    let (status, _) = admin.post("/admin/plugins/spam-rspamd/test", None).await;
    assert_eq!(status, 409);

    // Every change above left an audit row.
    assert_eq!(
        admin.audit_changes("spam-rspamd").await,
        [
            "plugin-registered",
            "plugin-approved",
            "plugin-enabled",
            "plugin-settings",
            "plugin-granted",
            "plugin-granted",
            "plugin-granted",
            "plugin-disabled",
        ]
    );

    let (status, gone) = admin
        .post("/admin/plugins/spam-rspamd/uninstall", None)
        .await;
    assert_eq!((status, &gone["unregistered"]), (200, &json!(true)));
    assert!(admin.plugins().await.is_empty());
    // Registered again, it starts from nothing: no approval, no grant.
    let p = admin
        .change("/admin/plugins", Some(json!({ "id": "spam-rspamd" })))
        .await;
    assert_eq!(p["approved"], false);
    assert!(strings(&p["granted"]).is_empty());
    assert_eq!(
        strings(&p["netAllowlist"]),
        ["rspamd"],
        "the compiled-in default host"
    );
}

// ── 2. third-party: pinned digest, unsigned flag, tampered file ──────────────

#[tokio::test]
async fn a_third_party_plugin_needs_a_pinned_digest_and_the_unsigned_flag() {
    let base = boot(ServerMode::Engine).await;
    let admin = Admin::login(&base).await;
    let dir = third_party_dir();
    // A real component under a third-party id, so that a permitted load succeeds.
    let bytes = dist_wasm("spam-rspamd");
    let file = dir.join("acme-spam.wasm");
    std::fs::write(&file, &bytes).unwrap();
    let manifest = json!({
        "id": "acme-spam",
        "name": "Acme spam filter",
        "version": "1.0.0",
        "capabilities": ["spam-action", "net"],
        "netAllowlist": ["scan.acme.example"],
    });

    // The file is on disk but its digest is not pinned: refused, nothing stored.
    let (status, refused) = admin.post("/admin/plugins", Some(manifest.clone())).await;
    assert_eq!(
        (status, &refused["code"]),
        (403, &json!("digest-not-approved"))
    );
    assert!(admin.plugins().await.is_empty());

    let (status, _) = admin
        .post(
            "/admin/plugins/allowlist",
            Some(json!({ "pluginId": "acme-spam", "digestHex": sha256_hex(&bytes) })),
        )
        .await;
    assert_eq!(status, 200, "pin the digest");

    // Unknown capabilities and the first-party-only one are refused up front.
    for caps in [
        json!(["spam-action", "root"]),
        json!(["account-backend", "net"]),
    ] {
        let mut bad = manifest.clone();
        bad["capabilities"] = caps.clone();
        let (status, _) = admin.post("/admin/plugins", Some(bad)).await;
        assert_eq!(status, 400, "capabilities {caps} must be refused");
    }
    assert!(admin.plugins().await.is_empty());

    let p = admin.change("/admin/plugins", Some(manifest.clone())).await;
    assert_eq!(
        (&p["firstParty"], &p["trust"]),
        (&json!(false), &json!("admin-pinned-digest"))
    );
    assert_eq!(
        (&p["signed"], &p["allowUnsigned"]),
        (&json!(false), &json!(false))
    );

    admin.change("/admin/plugins/acme-spam/approve", None).await;
    admin
        .change(
            "/admin/plugins/acme-spam/grant",
            Some(json!({ "capabilities": ["spam-action"] })),
        )
        .await;

    // Approved, granted, digest pinned — and still refused: it is unsigned and no
    // administrator has allowed that. The unfixed loader passed `allow_unsigned:
    // true` for every plugin.
    let (status, refused) = admin.post("/admin/plugins/acme-spam/enable", None).await;
    assert_eq!(
        (status, &refused["code"]),
        (403, &json!("unsigned-not-allowed"))
    );
    let p = admin.plugin("acme-spam").await;
    assert_eq!(
        (&p["enabled"], &p["loaded"]),
        (&json!(false), &json!(false))
    );

    // The flag takes `{allow}`, and setting it enables nothing by itself.
    let (status, _) = admin
        .post("/admin/plugins/acme-spam/allow-unsigned", None)
        .await;
    assert_eq!(status, 400, "a body is required");
    let p = admin
        .change(
            "/admin/plugins/acme-spam/allow-unsigned",
            Some(json!({ "allow": true })),
        )
        .await;
    assert_eq!(
        (&p["allowUnsigned"], &p["enabled"]),
        (&json!(true), &json!(false))
    );
    let p = admin.change("/admin/plugins/acme-spam/enable", None).await;
    assert_eq!(p["loaded"], true, "{p}");
    assert_eq!(strings(&p["loadedCapabilities"]), ["spam-action"]);

    // Clearing the flag unloads it at once.
    let p = admin
        .change(
            "/admin/plugins/acme-spam/allow-unsigned",
            Some(json!({ "allow": false })),
        )
        .await;
    assert_eq!(
        (&p["loaded"], &p["notLoadedReason"]),
        (&json!(false), &json!("unsigned-not-allowed"))
    );
    admin
        .change(
            "/admin/plugins/acme-spam/allow-unsigned",
            Some(json!({ "allow": true })),
        )
        .await;
    assert_eq!(admin.plugin("acme-spam").await["loaded"], true);

    // Tamper with the file on disk. The next load reads it, finds a digest nobody
    // pinned, and refuses it.
    let mut tampered = bytes.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 0x01;
    std::fs::write(&file, &tampered).unwrap();
    admin.change("/admin/plugins/acme-spam/disable", None).await;
    let p = admin.change("/admin/plugins/acme-spam/enable", None).await;
    assert_eq!(
        (&p["loaded"], &p["notLoadedReason"]),
        (&json!(false), &json!("component-unavailable")),
        "a tampered component must be refused by its digest"
    );
    // Positive control: the pinned bytes load again.
    std::fs::write(&file, &bytes).unwrap();
    admin.change("/admin/plugins/acme-spam/disable", None).await;
    assert_eq!(
        admin.change("/admin/plugins/acme-spam/enable", None).await["loaded"],
        true
    );

    // A tampered file is refused at registration too.
    let other = dir.join("acme-two.wasm");
    std::fs::write(&other, &bytes).unwrap();
    let (status, _) = admin
        .post(
            "/admin/plugins/allowlist",
            Some(json!({ "pluginId": "acme-two", "digestHex": sha256_hex(&bytes) })),
        )
        .await;
    assert_eq!(status, 200);
    std::fs::write(&other, &tampered).unwrap();
    let mut two = manifest.clone();
    two["id"] = json!("acme-two");
    let (status, refused) = admin.post("/admin/plugins", Some(two)).await;
    assert_eq!(
        (status, &refused["code"]),
        (403, &json!("digest-not-approved"))
    );

    // Revoking the pin disables and unloads the plugin.
    let (status, _) = admin
        .post(
            &format!(
                "/admin/plugins/allowlist/acme-spam/{}/revoke",
                sha256_hex(&bytes)
            ),
            None,
        )
        .await;
    assert_eq!(status, 200);
    let p = admin.plugin("acme-spam").await;
    assert_eq!(
        (&p["enabled"], &p["loaded"]),
        (&json!(false), &json!(false))
    );
}

// ── 3. all seven register; what is and is not loaded is said plainly ─────────

#[tokio::test]
async fn every_first_party_component_registers_and_reports_whether_it_runs() {
    let base = boot(ServerMode::Engine).await;
    let admin = Admin::login(&base).await;
    for id in FIRST_PARTY {
        let p = admin
            .change("/admin/plugins", Some(json!({ "id": id })))
            .await;
        assert_eq!(p["firstParty"], true, "{id}");
        let declared = strings(&p["capabilities"]);
        assert!(!declared.is_empty(), "{id} declares capabilities");
        admin
            .change(&format!("/admin/plugins/{id}/approve"), None)
            .await;
        admin
            .change(
                &format!("/admin/plugins/{id}/grant"),
                Some(json!({ "capabilities": declared })),
            )
            .await;
        admin
            .change(&format!("/admin/plugins/{id}/enable"), None)
            .await;
    }
    let listed: Vec<String> = admin
        .plugins()
        .await
        .iter()
        .map(|p| p["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(listed, FIRST_PARTY, "all seven are registered, in id order");

    let state = |p: &Value| {
        (
            p["loaded"].as_bool().unwrap(),
            p["restartRequired"].as_bool().unwrap(),
            p["notLoadedReason"].as_str().map(str::to_string),
        )
    };
    // One classifier seat: the first spam plugin in id order holds it.
    assert_eq!(
        state(&admin.plugin("spam-rspamd").await),
        (true, false, None)
    );
    assert_eq!(
        state(&admin.plugin("spam-spamassassin").await),
        (false, false, Some("another-classifier-active".into()))
    );
    // A bridge runs per bound account; none is bound here.
    for id in ["bridge-ews", "bridge-gmail", "bridge-graph"] {
        assert_eq!(
            state(&admin.plugin(id).await),
            (false, false, Some("no-account-binding".into())),
            "{id}"
        );
    }
    // Nothing in the server calls these two hooks, so they are never loaded.
    for id in ["languagetool", "nextcloud"] {
        assert_eq!(
            state(&admin.plugin(id).await),
            (false, false, Some("no-host-caller".into())),
            "{id}"
        );
    }

    // Disabling the seated classifier seats the next one.
    admin
        .change("/admin/plugins/spam-rspamd/disable", None)
        .await;
    assert_eq!(
        state(&admin.plugin("spam-spamassassin").await),
        (true, false, None)
    );
}

// ── 4. proxy mode has no engine, so nothing is reported as loaded ────────────

#[tokio::test]
async fn proxy_mode_registers_but_never_reports_a_plugin_as_loaded() {
    let base = boot(ServerMode::Proxy).await;
    let admin = Admin::login(&base).await;
    admin
        .change("/admin/plugins", Some(json!({ "id": "spam-rspamd" })))
        .await;
    admin
        .change("/admin/plugins/spam-rspamd/approve", None)
        .await;
    admin
        .change(
            "/admin/plugins/spam-rspamd/grant",
            Some(json!({ "capabilities": ["spam-action", "net"] })),
        )
        .await;
    let p = admin
        .change("/admin/plugins/spam-rspamd/enable", None)
        .await;
    assert_eq!(p["enabled"], true);
    assert_eq!(
        (&p["loaded"], &p["notLoadedReason"]),
        (&json!(false), &json!("proxy-mode"))
    );
    let (status, _) = admin.post("/admin/plugins/spam-rspamd/test", None).await;
    assert_eq!(status, 409);
}
