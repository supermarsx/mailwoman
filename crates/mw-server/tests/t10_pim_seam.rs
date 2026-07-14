//! t10-e14 backend live-E2E — bridge PIM through the REAL jail + engine (the headline).
//!
//! Proves the §6.5 bridge-PIM seam end-to-end through the *real* wasmtime jail, the real
//! digest-verified first-party bridge components (`plugins/dist/*.wasm`), the real boot
//! loader (`v7_mount::load_plugin_backends`), and the real engine PIM accessors — NOT a
//! mock. A boot-loaded Graph bridge account must route calendar/tasks/reactions/voting/
//! recall/Focused-sync to the bridge (honest per-interface `supports-*`), while a plain
//! IMAP account keeps every PIM path on the byte-unchanged standards fallback (the hard
//! regression gate, plan risk #2). The honest support matrix — Graph = all six, EWS =
//! calendar+tasks only, Gmail = none — is asserted through the mounted `BridgeCaps`
//! source exactly as it is served to JMAP/DAV at runtime.
//!
//! Needs no external services: the jail + components + engine are all real and in-tree,
//! so this runs in the default `cargo test` gate. The Graph guest's account-backend HTTP
//! is served from the committed `plugins/bridge-graph/fixtures/*.json` recordings (the
//! guest never dials Microsoft); the PIM `supports-*` probes are pure guest functions and
//! need no network at all.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::Value;

use mw_engine::{Engine, V7Hooks};
use mw_plugin::{
    HostServices, HttpFetcher, HttpReq, HttpResp, OAuthTokenProvider, PluginHost, TrustRoot,
};
use mw_store::{
    AccountKind, BridgeAccountRow, Credentials, NewAccount, PluginRow, ServerKey, Store,
};

const FIXTURE_TOKEN: &str = "FIXTURE.ACCESS.TOKEN";

/// Point `v7_mount::resolve_component` at the canonical shipped layout so the boot loader
/// reads + digest-verifies the real `.wasm` bytes (identical to v7_boot_load.rs).
fn point_plugin_dir_at_shipped_layout() {
    unsafe {
        std::env::set_var(
            "MW_PLUGIN_DIR",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../plugins/dist"),
        );
    }
}

/// Replay of the committed Graph fixture request→response pairs (longest `url_contains`
/// first), so the boot-loaded Graph guest's account-backend calls resolve without network.
struct GraphFixtureHttp {
    fixtures: Vec<(String, String, u16, Vec<u8>)>,
}

impl GraphFixtureHttp {
    fn load() -> Self {
        let dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../plugins/bridge-graph/fixtures");
        let mut fixtures = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for entry in rd.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                let v: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                let method = v["method"].as_str().unwrap().to_string();
                let url_contains = v["url_contains"].as_str().unwrap().to_string();
                let status = v["status"].as_u64().unwrap() as u16;
                let body = if let Some(j) = v.get("body_json").filter(|j| !j.is_null()) {
                    serde_json::to_vec(j).unwrap()
                } else if let Some(t) = v.get("body_text").and_then(Value::as_str) {
                    t.as_bytes().to_vec()
                } else {
                    Vec::new()
                };
                fixtures.push((method, url_contains, status, body));
            }
        }
        fixtures.sort_by_key(|f| std::cmp::Reverse(f.1.len()));
        Self { fixtures }
    }
}

#[async_trait]
impl HttpFetcher for GraphFixtureHttp {
    async fn fetch(&self, req: HttpReq) -> Result<HttpResp, String> {
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

fn fixture_registry() -> mw_server::plugins::PluginRegistry {
    let host = PluginHost::try_new(
        HostServices {
            http: Arc::new(GraphFixtureHttp::load()),
            oauth: Arc::new(FixtureOAuth),
            ..HostServices::default()
        },
        TrustRoot::empty(),
    )
    .unwrap();
    Arc::new(Mutex::new(host))
}

/// Seed the persisted 0008 rows an admin creates to run a bridge: an approved+enabled
/// `plugins` row (account-backend + net) + a `bridge_accounts` binding + the account.
async fn seed_bridge_account(store: &Store, bridge_id: &str, net_allow: &str) -> String {
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "bridge.invalid",
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
    store
        .put_plugin(&PluginRow {
            id: bridge_id.into(),
            name: bridge_id.into(),
            version: "26.10.0".into(),
            signature_hex: None,
            approved_by: Some("admin@vogue-homes.com".into()),
            enabled: true,
            capabilities_json: r#"["account-backend","net","addrbook-source"]"#.into(),
            net_allowlist_json: format!("[\"{net_allow}\"]"),
            limits_json: "{}".into(),
            created_at: "2026-07-14T00:00:00Z".into(),
        })
        .await
        .unwrap();
    store
        .put_bridge_account(&BridgeAccountRow {
            account_id: account_id.clone(),
            bridge_id: bridge_id.into(),
            oauth_ref: Some("token-ref".into()),
            extra_json: "{}".into(),
        })
        .await
        .unwrap();
    account_id
}

/// Boot the seeded bridge accounts through the real loader and attach the returned PIM
/// source to the engine (exactly as `mw_server::lib::router` does at mount).
async fn boot(store: &Store) -> Arc<Engine> {
    let registry = fixture_registry();
    let engine = Arc::new(Engine::new(store.clone()));
    let (_loaded, bridge_caps) =
        mw_server::v7_mount::load_plugin_backends(&engine, &registry, store).await;
    let mut hooks = V7Hooks::new();
    if let Some(src) = bridge_caps {
        hooks = hooks.with_bridge_caps(src);
    }
    engine.attach_v7(hooks);
    engine
}

// ── 1. Graph bridge account routes ALL SIX PIM interfaces to the bridge ──────────
#[tokio::test]
async fn graph_bridge_account_routes_all_pim_to_the_bridge() {
    point_plugin_dir_at_shipped_layout();
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let acct = seed_bridge_account(&store, "bridge-graph", "graph.microsoft.com").await;
    let engine = boot(&store).await;

    assert!(
        engine.bridge_calendar(&acct).is_some(),
        "Graph routes calendar to the bridge (not CalDAV fallback)"
    );
    assert!(
        engine.bridge_tasks(&acct).is_some(),
        "Graph routes tasks to the bridge"
    );
    assert!(engine.bridge_reactions(&acct).is_some(), "Graph reactions");
    assert!(engine.bridge_voting(&acct).is_some(), "Graph voting");
    assert!(engine.bridge_recall(&acct).is_some(), "Graph recall");
    assert!(
        engine.bridge_focused_sync(&acct).is_some(),
        "Graph Focused-sync"
    );
    assert_eq!(
        engine.bridge_caps(&acct),
        mw_engine::BridgeCaps {
            reactions: true,
            voting: true,
            recall: true,
            focused_sync: true,
        },
        "Graph advertises every parity cap"
    );
}

// ── 2. EWS honest matrix: calendar+tasks bind; parity stays on the fallback ───────
#[tokio::test]
async fn ews_bridge_binds_calendar_tasks_only_parity_on_fallback() {
    point_plugin_dir_at_shipped_layout();
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let acct = seed_bridge_account(&store, "bridge-ews", "outlook.office365.com").await;
    let engine = boot(&store).await;

    assert!(engine.bridge_calendar(&acct).is_some(), "EWS calendar binds");
    assert!(engine.bridge_tasks(&acct).is_some(), "EWS tasks bind");
    // EWS's legacy coarse caps overclaim parity; the honest per-interface supports-* are
    // false, so parity MUST stay on the standards fallback (e3's warning, plan §6.5).
    assert!(
        engine.bridge_reactions(&acct).is_none()
            && engine.bridge_voting(&acct).is_none()
            && engine.bridge_recall(&acct).is_none()
            && engine.bridge_focused_sync(&acct).is_none(),
        "EWS parity is NOT bound — honest supports-* are false"
    );
}

// ── 3. Gmail: pure standards fallback — NO PIM interface binds ────────────────────
#[tokio::test]
async fn gmail_bridge_is_pure_standards_fallback() {
    point_plugin_dir_at_shipped_layout();
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let acct = seed_bridge_account(&store, "bridge-gmail", "gmail.googleapis.com").await;
    let engine = boot(&store).await;

    assert!(
        engine.bridge_calendar(&acct).is_none()
            && engine.bridge_tasks(&acct).is_none()
            && engine.bridge_reactions(&acct).is_none()
            && engine.bridge_voting(&acct).is_none()
            && engine.bridge_recall(&acct).is_none()
            && engine.bridge_focused_sync(&acct).is_none(),
        "Gmail binds NO PIM interface (calendar/tasks/parity all on the standards fallback)"
    );
    assert_eq!(
        engine.bridge_caps(&acct),
        mw_engine::BridgeCaps::default(),
        "Gmail advertises no parity caps"
    );
}

// ── 4. Non-bridge (plain IMAP) account is byte-unchanged: EVERY PIM path is None ──
#[tokio::test]
async fn plain_imap_account_keeps_the_byte_unchanged_standards_fallback() {
    point_plugin_dir_at_shipped_layout();
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    // A plain IMAP account with NO bridge binding — the standards-only path.
    let acct = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.com",
                port: 993,
                tls: "implicit",
                username: "plain@example.com",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "plain@example.com".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    let engine = boot(&store).await;

    assert!(
        engine.bridge_calendar(&acct).is_none()
            && engine.bridge_tasks(&acct).is_none()
            && engine.bridge_reactions(&acct).is_none()
            && engine.bridge_voting(&acct).is_none()
            && engine.bridge_recall(&acct).is_none()
            && engine.bridge_focused_sync(&acct).is_none(),
        "a non-bridge account routes NOTHING to a bridge — CalDAV/CardDAV/IMAP fallback \
         stays byte-unchanged (hard regression gate)"
    );
    assert_eq!(engine.bridge_caps(&acct), mw_engine::BridgeCaps::default());
}
