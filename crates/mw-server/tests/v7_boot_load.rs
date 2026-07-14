//! V7 BOOT-LOAD gate (t7-fix-loadplugins) — prove that a bridge configured purely in
//! the 0008 registry (an approved+enabled `plugins` row + a `bridge_accounts` binding)
//! is auto-loaded at server boot and then serves the JMAP MAIL surface, reached through
//! the REAL boot function `mw_server::v7_mount::load_plugin_backends` — NOT a direct
//! `engine.register_plugin_backend` (that is what e16's `v7_e2e.rs` already did).
//!
//! e16 proved the *mechanism* (a plugin-backed account serves `Mailbox/get` like IMAP)
//! by registering the backend directly in the test. This file closes the remaining gap:
//! the **deployment boot path**. `load_plugin_backends` was a returns-0 stub; here we
//! seed only the persisted registry rows, call the boot loader, and assert it (a)
//! returns 1 and (b) the bound account is served by the engine's ordinary JMAP dispatch.
//!
//! ## Why a fixture host (not `build_app_full`)
//! The mount's `build_plugin_host` injects the live `reqwest`/rustls fetcher, so driving
//! the whole `build_app_full` would make the Graph bridge dial `graph.microsoft.com` for
//! real. Exactly like e16, we inject a recorded-fixture `HttpFetcher` + a fixture
//! `OAuthTokenProvider` into the `PluginHost` and then call the identical boot function
//! the mount calls at `crates/mw-server/src/lib.rs`. The store rows, the component
//! source (bundled), the load path, and the engine registration are all the real ones.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::Engine;
use mw_plugin::{
    HostServices, HttpFetcher, HttpReq, HttpResp, OAuthTokenProvider, PluginHost, TrustRoot,
};
use mw_store::{
    AccountKind, BridgeAccountRow, Credentials, NewAccount, PluginRow, ServerKey, Store,
};

/// The fake bearer token the Graph fixtures accept (real tokens never enter the guest).
const FIXTURE_TOKEN: &str = "FIXTURE.ACCESS.TOKEN";

/// A self-contained replay of the committed `plugins/bridge-graph/fixtures/*.json`
/// request→response pairs (method + longest `url_contains` first) — the same matcher
/// e16 uses, reimplemented here so the harness needs no dependency on the bridge crate.
struct GraphFixtureHttp {
    fixtures: Vec<(String, String, u16, Vec<u8>)>,
    seen: Mutex<Vec<(String, Option<String>)>>,
}

impl GraphFixtureHttp {
    fn load() -> Self {
        let dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../plugins/bridge-graph/fixtures");
        let mut fixtures = Vec::new();
        for entry in std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("read graph fixtures {}: {e}", dir.display()))
            .flatten()
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let v: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            let method = v["method"].as_str().unwrap().to_string();
            let url_contains = v["url_contains"].as_str().unwrap().to_string();
            let status = v["status"].as_u64().unwrap() as u16;
            let body = if let Some(j) = v.get("body_json") {
                if j.is_null() {
                    Vec::new()
                } else {
                    serde_json::to_vec(j).unwrap()
                }
            } else if let Some(t) = v.get("body_text").and_then(Value::as_str) {
                t.as_bytes().to_vec()
            } else {
                Vec::new()
            };
            fixtures.push((method, url_contains, status, body));
        }
        fixtures.sort_by_key(|f| std::cmp::Reverse(f.1.len()));
        assert!(!fixtures.is_empty(), "graph fixtures loaded");
        Self {
            fixtures,
            seen: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl HttpFetcher for GraphFixtureHttp {
    async fn fetch(&self, req: HttpReq) -> Result<HttpResp, String> {
        let authorization = req
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
            .map(|(_, v)| v.clone());
        self.seen
            .lock()
            .unwrap()
            .push((req.url.clone(), authorization));
        for (method, url_contains, status, body) in &self.fixtures {
            if method.eq_ignore_ascii_case(&req.method) && req.url.contains(url_contains) {
                return Ok(HttpResp {
                    status: *status,
                    headers: vec![("Content-Type".into(), "application/json".into())],
                    body: body.clone(),
                });
            }
        }
        Err(format!("no fixture for {} {}", req.method, req.url))
    }
}

struct FixtureOAuth;
#[async_trait]
impl OAuthTokenProvider for FixtureOAuth {
    async fn token(&self, _account: &str) -> Result<String, String> {
        Ok(FIXTURE_TOKEN.to_string())
    }
}

/// A `PluginHost` wired with the recorded-fixture services, exactly as
/// `v7_mount::build_plugin_host` would build it in a deployment (minus the live
/// reqwest fetcher). No registry seeding is needed: `load_plugin_backends` loads the
/// component bytes directly via `PluginHost::load`.
fn fixture_registry(http: Arc<GraphFixtureHttp>) -> mw_server::plugins::PluginRegistry {
    let host = PluginHost::try_new(
        HostServices {
            http,
            oauth: Arc::new(FixtureOAuth),
            ..HostServices::default()
        },
        TrustRoot::empty(),
    )
    .unwrap();
    Arc::new(Mutex::new(host))
}

async fn jmap(engine: &Engine, account_id: &str, calls: Value) -> Value {
    engine
        .handle_jmap(account_id, &json!({ "methodCalls": calls }))
        .await
}

/// **The headline proof.** Seed ONLY the 0008 persisted rows a real admin would create
/// (an approved+enabled `bridge-graph` plugin + a `bridge_accounts` binding + the
/// account), then reach the bridge through the BOOT loader — `load_plugin_backends` —
/// and assert the bridged account is served `Mailbox/get` by the engine like IMAP.
#[tokio::test]
async fn boot_loaded_bridge_serves_mailbox_get_through_the_engine() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();

    // The account the bridge backs (created exactly as any account is).
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "graph.microsoft.com",
                port: 443,
                tls: "implicit",
                username: "me@vogue-homes.com",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "me@vogue-homes.com".into(),
                password: "unused-bridge-uses-oauth".into(),
            },
        )
        .await
        .unwrap();

    // The signed-registry row: APPROVED + ENABLED, advertising the account-backend
    // (+ net/addrbook) capabilities. Unsigned (committed fixture) — the boot grant
    // allows unsigned for a bundled first-party component.
    store
        .put_plugin(&PluginRow {
            id: "bridge-graph".into(),
            name: "Microsoft Graph bridge".into(),
            version: "26.8.0".into(),
            signature_hex: None,
            approved_by: Some("admin@vogue-homes.com".into()),
            enabled: true,
            capabilities_json: r#"["account-backend","net","addrbook-source"]"#.into(),
            net_allowlist_json: r#"["graph.microsoft.com","login.microsoftonline.com"]"#.into(),
            limits_json: "{}".into(),
            created_at: "2026-07-14T00:00:00Z".into(),
        })
        .await
        .unwrap();

    // The account ↔ bridge binding.
    store
        .put_bridge_account(&BridgeAccountRow {
            account_id: account_id.clone(),
            bridge_id: "bridge-graph".into(),
            oauth_ref: Some("graph-token-ref".into()),
            extra_json: "{}".into(),
        })
        .await
        .unwrap();

    // Build the engine + a fixture-backed plugin host, then run the REAL boot loader.
    // (`Store` is a cheap `Arc` clone; the engine and the loader share the same rows.)
    let http = Arc::new(GraphFixtureHttp::load());
    let registry = fixture_registry(http.clone());
    let engine = Arc::new(Engine::new(store.clone()));

    let loaded = mw_server::v7_mount::load_plugin_backends(&engine, &registry, &store).await;
    assert_eq!(
        loaded, 1,
        "the boot loader auto-loaded exactly one bridge backend"
    );

    // The engine now treats the account as plugin-backed — reached via the boot path,
    // never a direct register_plugin_backend in this test.
    assert!(engine.is_plugin_backed(&account_id));
    assert_eq!(
        engine.plugin_backend_id(&account_id).as_deref(),
        Some("bridge-graph")
    );

    // resync lists+upserts mailboxes through the real jail (message sync may follow);
    // the mailbox surface is populated regardless.
    let _ = engine.resync(&account_id).await;

    let mb = jmap(&engine, &account_id, json!([["Mailbox/get", {}, "mb"]])).await;
    let mailboxes = mb["methodResponses"][0][1]["list"]
        .as_array()
        .expect("Mailbox/get served like imap");
    assert!(
        mailboxes.iter().any(|m| m["role"] == "inbox"),
        "the boot-loaded Graph bridge serves the folder tree (inbox role) via JMAP: {mb}"
    );

    // OAuth posture: the guest reached only Graph and carried the host-minted token
    // (tokens never live in the guest) — proving the injected HostServices reached the
    // boot-loaded component.
    let seen = http.seen.lock().unwrap().clone();
    assert!(
        !seen.is_empty(),
        "the boot-loaded guest made host-mediated calls"
    );
    for (url, auth) in &seen {
        assert!(
            url.contains("graph.microsoft.com") && !url.contains("login.microsoftonline.com"),
            "guest reached only Graph, never the token host: {url}"
        );
        assert_eq!(
            auth.as_deref(),
            Some(format!("Bearer {FIXTURE_TOKEN}").as_str()),
            "every Graph call carried the host-minted transient token"
        );
    }
}

/// Deny-by-default: with NO `bridge_accounts` binding (even though the plugin row is
/// approved+enabled), the boot loader registers nothing — the non-plugin path is
/// byte-unchanged.
#[tokio::test]
async fn boot_loader_loads_nothing_without_a_binding() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    store
        .put_plugin(&PluginRow {
            id: "bridge-graph".into(),
            name: "Microsoft Graph bridge".into(),
            version: "26.8.0".into(),
            signature_hex: None,
            approved_by: Some("admin@vogue-homes.com".into()),
            enabled: true,
            capabilities_json: r#"["account-backend","net"]"#.into(),
            net_allowlist_json: r#"["graph.microsoft.com"]"#.into(),
            limits_json: "{}".into(),
            created_at: "2026-07-14T00:00:00Z".into(),
        })
        .await
        .unwrap();

    let http = Arc::new(GraphFixtureHttp::load());
    let registry = fixture_registry(http);
    let engine = Arc::new(Engine::new(store.clone()));
    let loaded = mw_server::v7_mount::load_plugin_backends(&engine, &registry, &store).await;
    assert_eq!(loaded, 0, "no binding ⇒ nothing loads (deny-by-default)");
}

/// A binding to a plugin that is NOT approved+enabled loads nothing (deny-by-default).
#[tokio::test]
async fn boot_loader_skips_unapproved_or_disabled_binding() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "graph.microsoft.com",
                port: 443,
                tls: "implicit",
                username: "me@vogue-homes.com",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "me@vogue-homes.com".into(),
                password: "x".into(),
            },
        )
        .await
        .unwrap();
    // Registered + APPROVED but DISABLED.
    store
        .put_plugin(&PluginRow {
            id: "bridge-graph".into(),
            name: "Microsoft Graph bridge".into(),
            version: "26.8.0".into(),
            signature_hex: None,
            approved_by: Some("admin@vogue-homes.com".into()),
            enabled: false,
            capabilities_json: r#"["account-backend","net"]"#.into(),
            net_allowlist_json: r#"["graph.microsoft.com"]"#.into(),
            limits_json: "{}".into(),
            created_at: "2026-07-14T00:00:00Z".into(),
        })
        .await
        .unwrap();
    store
        .put_bridge_account(&BridgeAccountRow {
            account_id,
            bridge_id: "bridge-graph".into(),
            oauth_ref: None,
            extra_json: "{}".into(),
        })
        .await
        .unwrap();

    let http = Arc::new(GraphFixtureHttp::load());
    let registry = fixture_registry(http);
    let engine = Arc::new(Engine::new(store.clone()));
    let loaded = mw_server::v7_mount::load_plugin_backends(&engine, &registry, &store).await;
    assert_eq!(loaded, 0, "a disabled plugin binding loads nothing");
}
