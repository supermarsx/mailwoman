//! In-jail integration for the t10 PIM / Outlook-parity exports: load the REAL
//! committed `bridge-graph` component (now targeting `world plugin-pim`) in the
//! `mw-plugin` wasmtime host and drive its `calendar` / `tasks` / `bridge-parity`
//! exports through the engine `Bridge*` trait seam (t10-e1's `adapter_pim`) against
//! recorded Graph fixtures — no live Microsoft 365 tenant (plan §2.5).
//!
//! What this proves end-to-end through the real jail + WIT boundary + host PIM adapter:
//!  * the component is probed as a `plugin-pim` guest (`advertises_pim()`), and the six
//!    `Bridge*` accessors bind (present + `account-backend` granted);
//!  * Graph's HONEST support surfaces: `supports-{calendar,tasks,reactions,voting,
//!    recall,focused}() == true` (all genuine or best-effort on Graph);
//!  * calendar delta, room resources, free/busy, task list + delta, reaction set,
//!    vote cast, Focused-Inbox get/set, and the recall HONESTY MATRIX all round-trip;
//!  * recall is NEVER reported as guaranteed: an unread message ⇒ `Requested`, an
//!    already-read message ⇒ `Failed(<honest note>)` (never a false success).
//!
//! The pure Graph→WIT mapping is covered in `tests/mapping.rs`; the account-backend
//! seam in `tests/jail.rs`. This file covers ONLY the new PIM/parity export wiring.

use std::sync::Arc;

use async_trait::async_trait;
use mw_engine::{FocusedState, MessageRef, RecallOutcome};
use mw_plugin::{
    Capability, Grant, HostServices, HttpFetcher, HttpReq, HttpResp, OAuthTokenProvider,
    PluginHost, PluginLimits, PluginManifest, TrustRoot,
};

use bridge_graph::fixtures::{FixtureSet, FIXTURE_TOKEN};

/// The committed component, rebuilt by `build.sh` from `src/guest.rs`.
const COMPONENT: &[u8] = include_bytes!("fixtures/bridge-graph.wasm");

/// An `HttpFetcher` replaying the recorded Graph fixtures, plus two canned answers the
/// PIM parity reads need that have no standalone JSON fixture: the Focused-Inbox
/// `inferenceClassification` read (`get-focused`) and the fire-and-forget writes
/// (`set-focused` PATCH / reaction `unreact`). Kept local to this test file so the
/// shared `fixtures/` set stays byte-unchanged.
struct PimHttp {
    set: FixtureSet,
}

#[async_trait]
impl HttpFetcher for PimHttp {
    async fn fetch(&self, req: HttpReq) -> Result<HttpResp, String> {
        // get-focused: read the genuine Graph `inferenceClassification` field.
        if req.method.eq_ignore_ascii_case("GET")
            && req.url.contains("$select=inferenceClassification")
        {
            return Ok(HttpResp {
                status: 200,
                headers: vec![("Content-Type".into(), "application/json".into())],
                body: br#"{"id":"m","inferenceClassification":"focused"}"#.to_vec(),
            });
        }
        // set-focused (PATCH /me/messages/{id}) + reaction unreact — 200 no-content.
        if req.method.eq_ignore_ascii_case("PATCH") || req.url.contains("/unreact") {
            return Ok(HttpResp {
                status: 200,
                headers: Vec::new(),
                body: Vec::new(),
            });
        }
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

struct FixtureOAuth;

#[async_trait]
impl OAuthTokenProvider for FixtureOAuth {
    async fn token(&self, _account: &str) -> Result<String, String> {
        Ok(FIXTURE_TOKEN.to_string())
    }
}

const GRAPH_CAPS: [Capability; 3] = [
    Capability::AccountBackend,
    Capability::Net,
    Capability::AddrbookSource,
];

fn manifest() -> PluginManifest {
    PluginManifest {
        id: "bridge-graph".into(),
        name: "Microsoft Graph bridge".into(),
        version: "26.10.0".into(),
        signature: None,
        capabilities: GRAPH_CAPS.to_vec(),
        net_allowlist: vec!["graph.microsoft.com".into()],
        limits: PluginLimits::default(),
    }
}

fn grant() -> Grant {
    Grant {
        plugin_id: "bridge-graph".into(),
        capabilities: GRAPH_CAPS.to_vec(),
        granted_by: "admin@vogue-homes.com".into(),
        allow_unsigned: true,
    }
}

fn host() -> PluginHost {
    let services = HostServices {
        http: Arc::new(PimHttp {
            set: FixtureSet::load_default(),
        }),
        oauth: Arc::new(FixtureOAuth),
        ..HostServices::default()
    };
    PluginHost::try_new(services, TrustRoot::empty()).unwrap()
}

fn plugin_msg(raw: &str) -> MessageRef {
    MessageRef::Plugin { raw: raw.into() }
}

#[tokio::test]
async fn probes_as_pim_guest_with_honest_graph_support() {
    let handle = host().load(COMPONENT, &manifest(), &grant()).unwrap();

    // The rebuilt component exports the three PIM interfaces ⇒ probed as a PIM guest.
    assert!(handle.advertises_pim(), "bridge-graph advertises PIM");
    assert!(handle.as_bridge_calendar().is_some());
    assert!(handle.as_bridge_tasks().is_some());
    assert!(handle.as_bridge_reactions().is_some());
    assert!(handle.as_bridge_voting().is_some());
    assert!(handle.as_bridge_recall().is_some());
    assert!(handle.as_bridge_focused_sync().is_some());

    // Graph's HONEST per-capability support (through the jail): calendar + tasks +
    // Focused-Inbox are genuine; reactions/voting/recall are best-effort — all true.
    assert!(handle.bridge_supports_calendar().await.unwrap());
    assert!(handle.bridge_supports_tasks().await.unwrap());
    let caps = handle.bridge_parity_caps().await.unwrap();
    assert!(caps.reactions && caps.voting && caps.recall && caps.focused_sync);
}

#[tokio::test]
async fn calendar_exports_round_trip() {
    let handle = host().load(COMPONENT, &manifest(), &grant()).unwrap();
    let cal = handle.as_bridge_calendar().unwrap();

    // list-calendars: own (editable) + shared (read-only).
    let cals = cal.list_calendars().await.unwrap();
    assert_eq!(cals.len(), 2);
    assert_eq!(cals[0].id, "cal-own");
    assert!(!cals[0].read_only, "own calendar is editable");
    assert!(cals[1].read_only, "shared calendar is read-only");

    // sync-events: one changed VEVENT + one removed id + the deltaLink cursor.
    let delta = cal.sync_events("cal-own", b"").await.unwrap();
    assert_eq!(delta.changed.len(), 1);
    assert_eq!(delta.changed[0].calendar_id, "cal-own");
    assert!(
        delta.changed[0].ical.contains("BEGIN:VEVENT")
            && delta.changed[0].ical.contains("SUMMARY:Sprint review"),
        "VEVENT serialized: {}",
        delta.changed[0].ical
    );
    assert_eq!(delta.removed, vec!["evt-2".to_string()]);
    assert!(String::from_utf8_lossy(&delta.next_cursor).contains("EVTNEXT"));

    // find-rooms + get-schedule (free/busy VFREEBUSY text).
    let rooms = cal.find_rooms().await.unwrap();
    assert_eq!(rooms.len(), 2);
    assert_eq!(rooms[0].name, "Everest");
    assert_eq!(rooms[0].capacity, Some(12));

    let fb = cal
        .get_schedule(
            "everest@vogue-homes.com",
            "2026-07-15T00:00:00",
            "2026-07-15T23:59:59",
        )
        .await
        .unwrap();
    assert!(
        fb.contains("BEGIN:VFREEBUSY") && fb.contains("002200"),
        "got: {fb}"
    );
}

#[tokio::test]
async fn tasks_exports_round_trip() {
    let handle = host().load(COMPONENT, &manifest(), &grant()).unwrap();
    let tasks = handle.as_bridge_tasks().unwrap();

    // sync-tasks over a specific list ⇒ a deterministic 2-task VTODO snapshot.
    let delta = tasks.sync_tasks("list-tasks", b"").await.unwrap();
    assert_eq!(delta.changed.len(), 2);
    assert_eq!(delta.changed[0].list_id, "list-tasks");
    assert!(
        delta.changed[0].ical.contains("BEGIN:VTODO")
            && delta.changed[0]
                .ical
                .contains("SUMMARY:Draft release notes"),
        "VTODO serialized: {}",
        delta.changed[0].ical
    );
    assert!(
        delta.changed[1].ical.contains("STATUS:COMPLETED"),
        "the completed task carries STATUS:COMPLETED"
    );

    // list-tasks aggregates across every To-Do list; the real tasks are present.
    let all = tasks.list_tasks().await.unwrap();
    assert!(all.iter().any(|t| t.ical.contains("Draft release notes")));
    assert!(all.iter().any(|t| t.ical.contains("Tag 26.8")));
}

#[tokio::test]
async fn parity_reactions_voting_and_focused_round_trip() {
    let handle = host().load(COMPONENT, &manifest(), &grant()).unwrap();

    // reaction set (best-effort write) + get (honest empty — Graph has no read-back).
    let reactions = handle.as_bridge_reactions().unwrap();
    reactions
        .set_reaction(&plugin_msg("react-msg"), "👍", true)
        .await
        .unwrap();
    assert!(reactions
        .get_reactions(&plugin_msg("react-msg"))
        .await
        .unwrap()
        .is_empty());

    // vote cast (reply carries the choice) + tally (honest empty — Graph won't aggregate).
    let voting = handle.as_bridge_voting().unwrap();
    voting
        .cast_vote(&plugin_msg("vote-msg"), "Approve")
        .await
        .unwrap();
    assert!(voting
        .tally(&plugin_msg("vote-msg"))
        .await
        .unwrap()
        .is_empty());

    // Focused-Inbox: read the genuine classification, then flip it (PATCH succeeds).
    let focused = handle.as_bridge_focused_sync().unwrap();
    assert_eq!(
        focused
            .focused_state(&plugin_msg("recall-unread"))
            .await
            .unwrap(),
        FocusedState::Focused
    );
    focused
        .set_focused(&plugin_msg("recall-unread"), false)
        .await
        .unwrap();
}

#[tokio::test]
async fn recall_honesty_matrix_is_preserved() {
    let handle = host().load(COMPONENT, &manifest(), &grant()).unwrap();
    let recall = handle.as_bridge_recall().unwrap();

    // Unread ⇒ the request is accepted for processing, but NEVER guaranteed.
    let unread = recall.recall(&plugin_msg("recall-unread")).await.unwrap();
    assert_eq!(unread, RecallOutcome::Requested);

    // Already read ⇒ the bridge declines rather than pretend it can recall — the honest
    // limitation rides `Failed(note)`, never a false `Requested`/success.
    let read = recall.recall(&plugin_msg("recall-read")).await.unwrap();
    match read {
        RecallOutcome::Failed { reason } => assert!(
            reason.contains("already read"),
            "honest recall note preserved: {reason}"
        ),
        other => panic!("already-read recall must decline, got {other:?}"),
    }
}
