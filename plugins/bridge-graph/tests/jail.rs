//! In-jail integration: load the REAL committed `bridge-graph` component in the
//! `mw-plugin` wasmtime host and drive it through the engine `AccountBackend` seam
//! against recorded Graph fixtures — no live Microsoft 365 tenant (plan §2.5).
//!
//! What this proves end-to-end through the real jail + WIT boundary + host adapter:
//!  * the component loads (unsigned ⇒ banner signal) and is capability-gated;
//!  * `capabilities()` advertises the Outlook-parity caps;
//!  * `list_mailboxes()` serves the Graph folder tree (roles) through the seam;
//!  * `addrbook-source::search` serves contacts + GAL through the seam;
//!  * **OAuth tokens never live in the guest** — token acquisition is the host
//!    `oauth-token` import (the guest never contacts login.microsoftonline.com), and
//!    every Graph request carries the host-provided transient bearer token.
//!
//! The full mail sync / fetch / flags / Focused-Inbox / calendar / To-Do / recall
//! mapping is covered in `tests/mapping.rs`. Message-ref- and cursor-bearing methods
//! (sync/fetch/flags/move/submit) do NOT round-trip through the CURRENT host adapter
//! because it JSON-encodes `mw_engine::{MessageRef, SyncCursor}` into the WIT opaque
//! and `mw_engine::MessageRef` has no plugin variant — a host-side gap flagged for
//! e8/e14 (see the e10 report). It is NOT a WIT-ABI limitation: `message-ref.raw` is
//! documented as "serialized IMAP uid or Graph id".

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use mw_engine::MailboxRole;
use mw_plugin::{
    Capability, Grant, HostServices, HttpFetcher, HttpReq, HttpResp, OAuthTokenProvider,
    PluginHost, PluginLimits, PluginManifest, TrustRoot,
};

use bridge_graph::fixtures::{FixtureSet, FIXTURE_TOKEN};

/// The committed component, rebuilt by `build.sh` from `src/guest.rs`.
const COMPONENT: &[u8] = include_bytes!("fixtures/bridge-graph.wasm");

#[derive(Debug, Clone)]
struct SeenRequest {
    url: String,
    authorization: Option<String>,
}

/// An `mw_plugin::HttpFetcher` that replays the recorded Graph fixtures AND records
/// every request the guest emitted (so the test can assert the OAuth posture).
struct RecordingHttp {
    set: FixtureSet,
    seen: Mutex<Vec<SeenRequest>>,
}

#[async_trait]
impl HttpFetcher for RecordingHttp {
    async fn fetch(&self, req: HttpReq) -> Result<HttpResp, String> {
        let authorization = req
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
            .map(|(_, v)| v.clone());
        self.seen.lock().unwrap().push(SeenRequest {
            url: req.url.clone(),
            authorization,
        });
        match self.set.match_response(&req.method, &req.url) {
            Some(r) => Ok(HttpResp {
                status: r.status,
                headers: r.headers,
                body: r.body,
            }),
            None => Err(format!("no fixture for {} {}", req.method, req.url)),
        }
    }
}

/// A host OAuth provider — stands in for the device-code/auth-code flow the host
/// owns. The guest calls this via the `oauth-token` import; the secret never crosses
/// into the guest, only a short-lived access token does.
struct FixtureOAuth {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl OAuthTokenProvider for FixtureOAuth {
    async fn token(&self, _account: &str) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(FIXTURE_TOKEN.to_string())
    }
}

fn manifest(caps: Vec<Capability>) -> PluginManifest {
    PluginManifest {
        id: "bridge-graph".into(),
        name: "Microsoft Graph bridge".into(),
        version: "26.8.0".into(),
        signature: None,
        capabilities: caps,
        net_allowlist: vec![
            "graph.microsoft.com".into(),
            "login.microsoftonline.com".into(),
        ],
        limits: PluginLimits::default(),
    }
}

fn grant(caps: Vec<Capability>) -> Grant {
    Grant {
        plugin_id: "bridge-graph".into(),
        capabilities: caps,
        granted_by: "admin@vogue-homes.com".into(),
        allow_unsigned: true,
    }
}

struct Harness {
    host: PluginHost,
    seen: Arc<RecordingHttp>,
    oauth_calls: Arc<AtomicUsize>,
}

fn harness() -> Harness {
    let recording = Arc::new(RecordingHttp {
        set: FixtureSet::load_default(),
        seen: Mutex::new(Vec::new()),
    });
    let oauth_calls = Arc::new(AtomicUsize::new(0));
    let services = HostServices {
        http: recording.clone(),
        oauth: Arc::new(FixtureOAuth {
            calls: oauth_calls.clone(),
        }),
        ..HostServices::default()
    };
    Harness {
        host: PluginHost::try_new(services, TrustRoot::empty()).unwrap(),
        seen: recording,
        oauth_calls,
    }
}

const GRAPH_CAPS: [Capability; 3] = [
    Capability::AccountBackend,
    Capability::Net,
    Capability::AddrbookSource,
];

#[tokio::test]
async fn component_loads_unsigned_and_gates_account_backend() {
    let h = harness();

    // Without the AccountBackend grant, the backend adapter is unavailable.
    let m = manifest(vec![Capability::Net]);
    let handle = h
        .host
        .load(COMPONENT, &m, &grant(vec![Capability::Net]))
        .unwrap();
    assert!(handle.is_unsigned(), "loaded unsigned ⇒ banner signal");
    assert!(handle.as_account_backend().is_none(), "gated without cap");

    // With it, the adapter is available.
    let m = manifest(GRAPH_CAPS.to_vec());
    let handle = h
        .host
        .load(COMPONENT, &m, &grant(GRAPH_CAPS.to_vec()))
        .unwrap();
    assert!(handle.as_account_backend().is_some());
}

#[tokio::test]
async fn capabilities_and_mailboxes_through_the_seam() {
    let h = harness();
    let m = manifest(GRAPH_CAPS.to_vec());
    let handle = h
        .host
        .load(COMPONENT, &m, &grant(GRAPH_CAPS.to_vec()))
        .unwrap();
    let backend = handle.as_account_backend().unwrap();

    // Outlook-parity caps are advertised (the engine prefers bridge-native paths).
    let caps = backend.capabilities().await.unwrap();
    assert!(caps.idle && caps.r#move);

    // The Graph folder tree is served through the engine seam, roles mapped.
    let boxes = backend.list_mailboxes().await.unwrap();
    assert_eq!(boxes.len(), 3);
    assert_eq!(boxes[0].role, MailboxRole::Inbox);
    assert_eq!(boxes[0].mailbox_ref.name, "inbox");
    assert_eq!(boxes[1].role, MailboxRole::Archive);
    assert_eq!(boxes[2].role, MailboxRole::None);

    // The guest reached ONLY Graph (never the login/token host) and carried the
    // host-provided bearer token on every request ⇒ tokens never live in the guest.
    let seen = h.seen.seen.lock().unwrap().clone();
    assert!(!seen.is_empty());
    for r in &seen {
        assert!(
            r.url.contains("graph.microsoft.com"),
            "guest reached a non-Graph host: {}",
            r.url
        );
        assert!(
            !r.url.contains("login.microsoftonline.com"),
            "guest must not run the OAuth flow itself"
        );
        assert_eq!(
            r.authorization.as_deref(),
            Some(format!("Bearer {FIXTURE_TOKEN}").as_str()),
            "every Graph call carries the host-provided transient token"
        );
    }
    assert!(
        h.oauth_calls.load(Ordering::SeqCst) >= 1,
        "the guest acquired its token via the host oauth-token import"
    );
}

#[tokio::test]
async fn addrbook_search_serves_contacts_and_gal_through_the_seam() {
    let h = harness();
    let m = manifest(GRAPH_CAPS.to_vec());
    let handle = h
        .host
        .load(COMPONENT, &m, &grant(GRAPH_CAPS.to_vec()))
        .unwrap();

    let results = handle.call_addrbook_search("car".into()).await.unwrap();
    // Carol (contacts + people + GAL) de-duplicated; Frank via GAL UPN fallback.
    let carol = results
        .iter()
        .filter(|e| e.to_ascii_lowercase().contains("carol@vogue-homes.com"))
        .count();
    assert_eq!(carol, 1, "de-duplicated across sources: {results:?}");
    assert!(results.iter().any(|e| e.contains("frank@vogue-homes.com")));
}
