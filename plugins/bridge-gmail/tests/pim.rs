//! t10 — the HONEST negative PIM path for the Gmail bridge, proven end-to-end through
//! the real `mw-plugin` wasmtime jail.
//!
//! The Gmail bridge targets `world plugin-pim`, so the committed
//! `fixtures/bridge-gmail.wasm` EXPORTS the three optional PIM interfaces
//! (`calendar` / `tasks` / `bridge-parity`) — the host can PROBE and BIND them
//! (`advertises_pim()` + every `as_bridge_*()` is `Some`). But the Gmail bridge is
//! **mail-scoped**: Google Calendar / Google Tasks are separate Google APIs outside
//! this bridge's OAuth scope, and Gmail has no Outlook reaction/voting/recall/Focused
//! parity. So every honest `supports-*()` returns `false`, which is exactly what keeps
//! the engine on its byte-unchanged standards fallback (native CalDAV/CardDAV):
//! e13 (MOUNT) wires an `as_bridge_*` trait object only when the matching
//! `supports-* == true`, so here it wires NONE.
//!
//! This is the deliberate "binds-interface-but-honest-supports-keeps-fallback"
//! demonstrator: interface presence (coarse) is decoupled from honest capability.

use std::sync::Arc;

use async_trait::async_trait;
use mw_engine::{MessageRef, RecallOutcome};
use mw_plugin::{
    Capability, Grant, HostServices, HttpFetcher, HttpReq, HttpResp, OAuthTokenProvider,
    PluginHost, PluginLimits, PluginManifest, TrustRoot,
};

const COMPONENT: &[u8] = include_bytes!("../fixtures/bridge-gmail.wasm");

/// A do-nothing HTTP fetcher — the PIM negative path never reaches out (every
/// `supports-*()` is `false`, and the data funcs return `unsupported` without I/O).
struct NoHttp;
#[async_trait]
impl HttpFetcher for NoHttp {
    async fn fetch(&self, _req: HttpReq) -> Result<HttpResp, String> {
        Ok(HttpResp {
            status: 404,
            headers: vec![],
            body: b"{}".to_vec(),
        })
    }
}

struct MockOAuth;
#[async_trait]
impl OAuthTokenProvider for MockOAuth {
    async fn token(&self, _account: &str) -> Result<String, String> {
        Ok("test-access-token".to_string())
    }
}

fn manifest() -> PluginManifest {
    PluginManifest {
        id: "bridge-gmail".into(),
        name: "Gmail API bridge".into(),
        version: "26.10.0".into(),
        signature: None,
        capabilities: vec![Capability::AccountBackend, Capability::Net],
        net_allowlist: vec!["gmail.googleapis.com".into()],
        limits: PluginLimits {
            memory_mb: 64,
            deadline_ms: 15_000,
            fuel: None,
        },
    }
}

fn grant() -> Grant {
    Grant {
        plugin_id: "bridge-gmail".into(),
        capabilities: vec![Capability::AccountBackend, Capability::Net],
        granted_by: "admin@test".into(),
        allow_unsigned: true,
    }
}

fn load() -> mw_plugin::PluginHandle {
    let services = HostServices {
        http: Arc::new(NoHttp),
        oauth: Arc::new(MockOAuth),
        ..HostServices::default()
    };
    let host = PluginHost::try_new(services, TrustRoot::empty()).unwrap();
    host.load(COMPONENT, &manifest(), &grant()).unwrap()
}

/// The three PIM interfaces ARE exported, so the host probes them present and — with
/// the bridge (`account-backend`) capability granted — can bind an adapter for each.
/// This is the "host binds the interface" half of the negative path.
#[test]
fn gmail_exports_and_binds_all_three_pim_interfaces() {
    let handle = load();
    assert!(
        handle.advertises_pim(),
        "bridge-gmail targets world plugin-pim ⇒ the host probes the PIM interfaces present"
    );
    // Interface present + account-backend granted ⇒ the host CAN bind each adapter
    // (whether the engine USES it is decided by the honest supports-* probe below).
    assert!(
        handle.as_bridge_calendar().is_some(),
        "calendar interface bindable"
    );
    assert!(
        handle.as_bridge_tasks().is_some(),
        "tasks interface bindable"
    );
    assert!(
        handle.as_bridge_reactions().is_some(),
        "reactions interface bindable"
    );
    assert!(
        handle.as_bridge_voting().is_some(),
        "voting interface bindable"
    );
    assert!(
        handle.as_bridge_recall().is_some(),
        "recall interface bindable"
    );
    assert!(
        handle.as_bridge_focused_sync().is_some(),
        "focused-sync interface bindable"
    );
}

/// The HONEST half: every `supports-*()` returns `false` (Gmail is mail-only), so
/// e13 wires no `as_bridge_*` trait object into a `BridgeCapabilitySource` and the
/// engine keeps its byte-unchanged standards fallback for calendar/tasks/parity.
#[tokio::test]
async fn gmail_honest_supports_are_all_false_keeping_engine_on_fallback() {
    let handle = load();

    assert!(
        !handle.bridge_supports_calendar().await.unwrap(),
        "Gmail bridge does not implement Google Calendar (separate API, out of scope)"
    );
    assert!(
        !handle.bridge_supports_tasks().await.unwrap(),
        "Gmail bridge does not implement Google Tasks (separate API, out of scope)"
    );

    // The Outlook-parity subset the engine reads via BridgeCapabilitySource::caps —
    // all false, mirroring GmailBackend::capabilities().
    let caps = handle.bridge_parity_caps().await.unwrap();
    assert!(!caps.reactions, "no Gmail reactions");
    assert!(!caps.voting, "no Gmail voting");
    assert!(!caps.recall, "no Gmail recall");
    assert!(!caps.focused_sync, "no Gmail Focused-Inbox sync");
}

/// Recall crosses the seam as the honest §10.3 outcome: `Unsupported` (Gmail has no
/// server-side recall) — NOT `Requested` (nothing was attempted) and NOT `Failed`.
#[tokio::test]
async fn gmail_recall_reports_unsupported_not_requested() {
    let handle = load();
    let msg = MessageRef::Plugin {
        raw: "gmail-msg-1".into(),
    };
    let outcome = handle.bridge_recall(&msg).await.unwrap();
    assert_eq!(
        outcome,
        RecallOutcome::Unsupported,
        "the honesty matrix: no recall ⇒ Unsupported, never Requested/Failed"
    );
}

/// The unsupported data funcs return a typed `unsupported` error through the jail
/// (never a host panic), for the calendar/tasks/parity read paths.
#[tokio::test]
async fn gmail_pim_data_funcs_return_unsupported_not_panic() {
    let handle = load();

    // calendar list ⇒ Unsupported (mapped to the host's typed error, host survives).
    assert!(
        handle.bridge_list_calendars().await.is_err(),
        "list-calendars is unsupported on the mail-only Gmail bridge"
    );

    // sync-events ⇒ Unsupported.
    assert!(
        handle.bridge_sync_events("cal-1", &[]).await.is_err(),
        "sync-events is unsupported on the mail-only Gmail bridge"
    );

    // get-reactions ⇒ Unsupported.
    let msg = MessageRef::Plugin {
        raw: "gmail-msg-1".into(),
    };
    assert!(
        handle.bridge_get_reactions(&msg).await.is_err(),
        "get-reactions is unsupported on the mail-only Gmail bridge"
    );
}
