//! Host integration test for the EWS bridge's PIM / Outlook-parity exports (t10-e3).
//! Loads the committed `wasm32-wasip2` component (`fixtures/bridge-ews.wasm`) through
//! the REAL `mw-plugin` jail and drives the `calendar`/`tasks`/`bridge-parity` WIT
//! exports the host binds via its per-interface probe (t10-e1). The injected
//! `HttpFetcher` performs the NTLMv2 401 challenge/response dance and dispatches each
//! SOAP request to the matching recorded fixture, so the whole PIM path runs
//! end-to-end through the sandbox — and the HONEST support matrix is asserted:
//! calendar + tasks SUPPORTED; reactions/voting/focused NOT EWS features; recall has
//! no third-party API → `unsupported`.

use std::sync::Arc;

use async_trait::async_trait;
use base64::Engine as _;
use mw_engine::MessageRef;
use mw_plugin::{
    BasicCredentialProvider, BasicCredentials, Capability, Grant, HostServices, HttpFetcher,
    HttpReq, HttpResp, PluginHost, PluginLimits, PluginManifest, TrustRoot,
};

const GUEST: &[u8] = include_bytes!("../fixtures/bridge-ews.wasm");

const CAL: &str = include_str!("../fixtures/calendar_events.xml");
const TASKS: &str = include_str!("../fixtures/tasks.xml");
const ROOMS: &str = include_str!("../fixtures/room_lists.xml");
const FREEBUSY: &str = include_str!("../fixtures/free_busy.xml");

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// A minimal but well-formed NTLM CHALLENGE (Type 2) — 4-byte EOL TargetInfo at 48.
fn ntlm_type2() -> Vec<u8> {
    let mut m = Vec::new();
    m.extend_from_slice(b"NTLMSSP\0");
    m.extend_from_slice(&2u32.to_le_bytes());
    m.extend_from_slice(&[0u8; 8]);
    m.extend_from_slice(&0x0008_0201u32.to_le_bytes());
    m.extend_from_slice(&[0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
    m.extend_from_slice(&[0u8; 8]);
    m.extend_from_slice(&4u16.to_le_bytes());
    m.extend_from_slice(&4u16.to_le_bytes());
    m.extend_from_slice(&48u32.to_le_bytes());
    assert_eq!(m.len(), 48);
    m.extend_from_slice(&[0, 0, 0, 0]);
    m
}

/// A fixture EWS credential provider (t12: the guest pulls endpoint + creds through
/// the `basic-credentials` host import). A non-empty domain selects the NTLMv2 path.
struct FixtureCreds;

#[async_trait]
impl BasicCredentialProvider for FixtureCreds {
    async fn credentials(&self, _account: &str) -> Result<BasicCredentials, String> {
        Ok(BasicCredentials {
            user: "svc-mailwoman".into(),
            domain: "CORP".into(),
            password: "fixture-secret".into(),
            workstation: "MAILWOMAN".into(),
            endpoint: "https://ews.example.com/EWS/Exchange.asmx".into(),
        })
    }
}

/// A fake EWS server: NTLM handshake + fixture dispatch for the PIM operations.
struct FakeEws;

impl FakeEws {
    fn dispatch(body: &str) -> &'static str {
        // Calendar and tasks both use FindItem; distinguish by the target folder id.
        if body.contains("FindItem") && body.contains("Id=\"tasks\"") {
            TASKS
        } else if body.contains("FindItem") && body.contains("Id=\"calendar\"") {
            CAL
        } else if body.contains("GetRoomLists") {
            ROOMS
        } else if body.contains("GetUserAvailabilityRequest") {
            FREEBUSY
        } else {
            "<soap:Envelope/>"
        }
    }
}

#[async_trait]
impl HttpFetcher for FakeEws {
    async fn fetch(&self, req: HttpReq) -> Result<HttpResp, String> {
        let auth = req
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default();

        if let Some(tok) = auth.strip_prefix("NTLM ") {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(tok.trim())
                .map_err(|e| e.to_string())?;
            let msg_type = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
            match msg_type {
                1 => {
                    return Ok(HttpResp {
                        status: 401,
                        headers: vec![(
                            "WWW-Authenticate".to_string(),
                            format!("NTLM {}", b64(&ntlm_type2())),
                        )],
                        body: vec![],
                    });
                }
                3 => {
                    let body = String::from_utf8_lossy(&req.body.unwrap_or_default()).into_owned();
                    return Ok(HttpResp {
                        status: 200,
                        headers: vec![],
                        body: FakeEws::dispatch(&body).as_bytes().to_vec(),
                    });
                }
                _ => {}
            }
        }
        Ok(HttpResp {
            status: 401,
            headers: vec![("WWW-Authenticate".to_string(), "NTLM".to_string())],
            body: vec![],
        })
    }
}

fn manifest() -> PluginManifest {
    PluginManifest {
        id: "bridge-ews".into(),
        name: "Exchange EWS bridge".into(),
        version: "26.10.0".into(),
        signature: None,
        capabilities: vec![Capability::AccountBackend, Capability::Net],
        net_allowlist: vec!["ews.example.com".into()],
        limits: PluginLimits {
            memory_mb: 128,
            deadline_ms: 10_000,
            fuel: None,
        },
    }
}

fn grant() -> Grant {
    Grant {
        plugin_id: "bridge-ews".into(),
        capabilities: vec![Capability::AccountBackend, Capability::Net],
        granted_by: "admin@test".into(),
        allow_unsigned: true,
    }
}

fn host() -> PluginHost {
    let services = HostServices {
        http: Arc::new(FakeEws),
        basic_creds: Arc::new(FixtureCreds),
        ..HostServices::default()
    };
    PluginHost::try_new(services, TrustRoot::empty()).unwrap()
}

#[tokio::test]
async fn ews_advertises_honest_pim_support_matrix() {
    let host = host();
    let handle = host.load(GUEST, &manifest(), &grant()).unwrap();

    // The component exports the PIM interfaces AND has account-backend granted.
    assert!(
        handle.advertises_pim(),
        "plugin-pim exports probed + granted"
    );

    // Honest supports-*(): calendar + tasks true; reactions/voting/recall/focused NOT
    // EWS features (recall has no third-party API).
    assert!(handle.bridge_supports_calendar().await.unwrap());
    assert!(handle.bridge_supports_tasks().await.unwrap());
    let parity = handle.bridge_parity_caps().await.unwrap();
    assert!(!parity.reactions, "EWS has no reactions");
    assert!(!parity.voting, "voting not exposed through the parity seam");
    assert!(!parity.recall, "no third-party recall API");
    assert!(!parity.focused_sync, "Focused Inbox is Graph-only");
}

#[tokio::test]
async fn ews_calendar_delta_through_the_exports() {
    let host = host();
    let handle = host.load(GUEST, &manifest(), &grant()).unwrap();
    let calendar = handle
        .as_bridge_calendar()
        .expect("calendar interface bound");

    // list-calendars ⇒ the primary Calendar distinguished folder.
    let cals = calendar.list_calendars().await.unwrap();
    assert_eq!(cals.len(), 1);
    assert_eq!(cals[0].id, "calendar");
    assert_eq!(cals[0].role, "calendar");

    // sync-events ⇒ FindItem CalendarView through the NTLM-authenticated transport,
    // each CalendarItem serialized to VEVENT.
    let delta = calendar.sync_events("calendar", &[]).await.unwrap();
    assert_eq!(delta.changed.len(), 2, "two calendar items in the fixture");
    assert!(delta.removed.is_empty());
    assert!(!delta.next_cursor.is_empty(), "advances an opaque cursor");
    let ev = &delta.changed[0];
    assert_eq!(ev.id, "EVT-1");
    assert_eq!(ev.calendar_id, "calendar");
    assert!(ev.ical.contains("BEGIN:VEVENT"));
    assert!(ev.ical.contains("SUMMARY:Weekly sync"));
    assert!(ev.ical.contains("DTSTART:20260720T090000Z"));
    assert_eq!(ev.start.as_deref(), Some("2026-07-20T09:00:00Z"));

    // find-rooms ⇒ GetRoomLists.
    let rooms = calendar.find_rooms().await.unwrap();
    assert_eq!(rooms.len(), 2);
    assert_eq!(rooms[0].address, "room-a@corp.example");

    // get-schedule ⇒ GetUserAvailability serialized to VFREEBUSY.
    let vfb = calendar
        .get_schedule(
            "bob@corp.example",
            "2026-07-20T00:00:00",
            "2026-07-21T00:00:00",
        )
        .await
        .unwrap();
    assert!(vfb.contains("BEGIN:VFREEBUSY"));
    assert!(vfb.contains("FREEBUSY;FBTYPE=BUSY:20260720T090000/20260720T100000"));
}

#[tokio::test]
async fn ews_task_list_through_the_exports() {
    let host = host();
    let handle = host.load(GUEST, &manifest(), &grant()).unwrap();
    let tasks = handle.as_bridge_tasks().expect("tasks interface bound");

    // list-tasks ⇒ FindItem over the Tasks distinguished folder, each Task → VTODO.
    let list = tasks.list_tasks().await.unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].list_id, "tasks");
    assert!(!list[0].completed, "first task open");
    assert!(list[1].completed, "second task done");
    assert!(list[0].ical.contains("BEGIN:VTODO"));
    assert!(list[0].ical.contains("SUMMARY:Ship 26.10"));
    assert!(list[0].ical.contains("STATUS:NEEDS-ACTION"));

    // sync-tasks ⇒ same items as a delta with an opaque cursor.
    let delta = tasks.sync_tasks("tasks", &[]).await.unwrap();
    assert_eq!(delta.changed.len(), 2);
    assert!(!delta.next_cursor.is_empty());
}

#[tokio::test]
async fn ews_recall_is_honestly_unsupported() {
    let host = host();
    let handle = host.load(GUEST, &manifest(), &grant()).unwrap();

    // The parity interface is bound (present + account-backend granted), but recall
    // honestly reports Unsupported — EWS has no third-party recall/unsend API.
    let recall = handle.as_bridge_recall().expect("parity interface bound");
    let msg = MessageRef::Plugin {
        raw: "ITEM-1\u{1f}CK1".into(),
    };
    let outcome = recall.recall(&msg).await.unwrap();
    assert_eq!(outcome, mw_engine::RecallOutcome::Unsupported);
}
