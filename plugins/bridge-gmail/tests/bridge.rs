//! End-to-end test of the Gmail bridge as a REAL `wasm32-wasip2` component loaded
//! through the `mw-plugin` wasmtime jail and driven as an `mw_engine::AccountBackend`
//! — exactly how the engine consumes it (plan §3 e12 acceptance).
//!
//! The committed `fixtures/bridge-gmail.wasm` (built by `build.sh`) runs in the jail;
//! a mock `HttpFetcher` replays the recorded Gmail REST fixtures under `fixtures/`
//! and a mock `OAuthTokenProvider` mints a short-lived access token (the guest never
//! sees a refresh/long-lived secret). This proves, through the WIT boundary:
//!
//!   * label fidelity — Gmail system labels map to mailbox roles, user labels to
//!     folders, STARRED/UNREAD to flags;
//!   * history-ID delta sync round-trips through the frozen `SyncCursor::Plugin`
//!     (the baseline snapshot's `historyId` is handed back and drives the delta);
//!   * `fetch_raw` returns the decoded RFC822 body + flags;
//!   * the OAuth access token is host-provided and carried only as a Bearer header.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use mw_engine::{AccountBackend, MailboxRole, RawMailboxRef, SyncCursor};
use mw_plugin::{
    Capability, Grant, HostServices, HttpFetcher, HttpReq, HttpResp, OAuthTokenProvider,
    PluginHost, PluginLimits, PluginManifest, TrustRoot,
};

const COMPONENT: &[u8] = include_bytes!("../fixtures/bridge-gmail.wasm");

const LABELS: &str = include_str!("../fixtures/labels.json");
const PROFILE: &str = include_str!("../fixtures/profile.json");
const MESSAGES: &str = include_str!("../fixtures/messages_inbox.json");
const HISTORY: &str = include_str!("../fixtures/history.json");
const MSG_M1_RAW: &str = include_str!("../fixtures/msg_m1_raw.json");

const ACCESS_TOKEN: &str = "test-access-token-abc123";

/// Replays the recorded Gmail REST fixtures by matching the request URL, recording
/// every request so the test can assert on the Bearer header + the delta cursor.
struct GmailMock {
    seen: Mutex<Vec<HttpReq>>,
}

#[async_trait]
impl HttpFetcher for GmailMock {
    async fn fetch(&self, req: HttpReq) -> Result<HttpResp, String> {
        self.seen.lock().unwrap().push(req.clone());
        let url = req.url.as_str();
        let json = |s: &str| {
            Ok(HttpResp {
                status: 200,
                headers: vec![("content-type".into(), "application/json".into())],
                body: s.as_bytes().to_vec(),
            })
        };
        // Order matters: a message GET (`/messages/<id>`) before the list (`/messages?`).
        if url.contains("/profile") {
            json(PROFILE)
        } else if url.contains("/labels") {
            json(LABELS)
        } else if url.contains("/history") {
            // Prove the round-trip: the delta MUST resume from the baseline head 1000.
            assert!(
                url.contains("startHistoryId=1000"),
                "history must resume from the snapshotted cursor, got {url}"
            );
            json(HISTORY)
        } else if url.contains("/messages/m1") && url.contains("format=raw") {
            json(MSG_M1_RAW)
        } else if url.contains("/messages?") {
            json(MESSAGES)
        } else {
            Ok(HttpResp {
                status: 404,
                headers: vec![],
                body: b"{}".to_vec(),
            })
        }
    }
}

struct MockOAuth;
#[async_trait]
impl OAuthTokenProvider for MockOAuth {
    async fn token(&self, _account: &str) -> Result<String, String> {
        // Host mints a short-lived ACCESS token; the guest never holds a refresh token.
        Ok(ACCESS_TOKEN.to_string())
    }
}

fn manifest() -> PluginManifest {
    PluginManifest {
        id: "bridge-gmail".into(),
        name: "Gmail API bridge".into(),
        version: "26.8.0".into(),
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

fn host_and_backend() -> (Arc<GmailMock>, Arc<dyn AccountBackend>) {
    let mock = Arc::new(GmailMock {
        seen: Mutex::new(Vec::new()),
    });
    let services = HostServices {
        http: mock.clone(),
        oauth: Arc::new(MockOAuth),
        ..HostServices::default()
    };
    let host = PluginHost::try_new(services, TrustRoot::empty()).unwrap();
    let handle = host.load(COMPONENT, &manifest(), &grant()).unwrap();
    assert!(
        handle.is_unsigned(),
        "in-tree component is unsigned ⇒ banner signal set"
    );
    let backend = handle
        .as_account_backend()
        .expect("account-backend capability granted");
    (mock, backend)
}

fn inbox() -> RawMailboxRef {
    RawMailboxRef {
        name: "INBOX".into(),
        uidvalidity: 1,
    }
}

#[tokio::test]
async fn capabilities_advertise_move_not_outlook_caps() {
    let (_mock, backend) = host_and_backend();
    let caps = backend.capabilities().await.unwrap();
    assert!(caps.r#move, "Gmail supports label re-tagging (move)");
    assert!(!caps.idle, "Gmail is HTTP-poll, no IDLE");
}

#[tokio::test]
async fn label_fidelity_maps_gmail_labels_to_roles_and_folders() {
    let (_mock, backend) = host_and_backend();
    let boxes = backend.list_mailboxes().await.unwrap();

    let role_of = |name: &str| {
        boxes
            .iter()
            .find(|m| m.mailbox_ref.name == name)
            .map(|m| m.role)
    };
    assert_eq!(role_of("INBOX"), Some(MailboxRole::Inbox));
    assert_eq!(role_of("SENT"), Some(MailboxRole::Sent));
    assert_eq!(role_of("DRAFT"), Some(MailboxRole::Drafts));
    assert_eq!(role_of("TRASH"), Some(MailboxRole::Trash));
    assert_eq!(role_of("SPAM"), Some(MailboxRole::Junk));
    assert_eq!(role_of("All Mail"), Some(MailboxRole::All));
    // User label surfaces as a folder by its display name.
    assert_eq!(role_of("Receipts"), Some(MailboxRole::None));
    // Flag-only / hidden labels are NEVER mailboxes.
    for hidden in ["STARRED", "UNREAD", "IMPORTANT", "CATEGORY_PROMOTIONS"] {
        assert!(
            !boxes.iter().any(|m| m.mailbox_ref.name == hidden),
            "{hidden} must not be a mailbox"
        );
    }
    // INBOX counts flow through from labels.list.
    let ib = boxes
        .iter()
        .find(|m| m.mailbox_ref.name == "INBOX")
        .unwrap();
    assert_eq!(ib.total, 2);
    assert_eq!(ib.unread, 1);
}

#[tokio::test]
async fn history_id_delta_sync_round_trips_through_plugin_cursor() {
    let (mock, backend) = host_and_backend();

    // 1) Baseline sync: an empty plugin cursor ⇒ full enumeration + profile head.
    let full = backend
        .sync_mailbox(&inbox(), &SyncCursor::Plugin { opaque: vec![] })
        .await
        .unwrap();
    assert_eq!(full.added.len(), 2, "INBOX baseline lists m1 + m2");
    // The next cursor carries the Gmail historyId losslessly as SyncCursor::Plugin.
    match &full.next_cursor {
        SyncCursor::Plugin { opaque } => assert_eq!(opaque, b"1000"),
        other => panic!("expected SyncCursor::Plugin, got {other:?}"),
    }

    // 2) Feed the cursor back ⇒ the history-ID delta path (the mock asserts the URL
    //    resumes from startHistoryId=1000, proving the round-trip end-to-end).
    let delta = backend
        .sync_mailbox(&inbox(), &full.next_cursor)
        .await
        .unwrap();
    assert_eq!(delta.added.len(), 1, "m3 added");
    assert_eq!(delta.removed.len(), 1, "m1 removed");
    assert_eq!(delta.flag_changes.len(), 1, "m2 re-labelled");
    let (_, flags) = &delta.flag_changes[0];
    assert!(flags.contains(&mw_engine::Flag::Flagged), "m2 got STARRED");
    assert!(flags.contains(&mw_engine::Flag::Seen), "m2 is read");
    // Cursor advanced to the response head.
    match &delta.next_cursor {
        SyncCursor::Plugin { opaque } => assert_eq!(opaque, b"1005"),
        other => panic!("expected SyncCursor::Plugin, got {other:?}"),
    }

    // The guest authenticated every call with the host-minted access token only.
    let seen = mock.seen.lock().unwrap();
    assert!(!seen.is_empty());
    let bearer = format!("Bearer {ACCESS_TOKEN}");
    assert!(
        seen.iter().all(|r| r
            .headers
            .iter()
            .any(|(k, v)| k.eq_ignore_ascii_case("authorization") && v == &bearer)),
        "every Gmail request carries the host-provided Bearer token"
    );
}

#[tokio::test]
async fn fetch_raw_returns_decoded_body_and_flags() {
    let (_mock, backend) = host_and_backend();

    // Get real refs from the baseline sync, then fetch the first (m1).
    let full = backend
        .sync_mailbox(&inbox(), &SyncCursor::Plugin { opaque: vec![] })
        .await
        .unwrap();
    let msgs = backend.fetch_raw(&full.added[..1]).await.unwrap();
    assert_eq!(msgs.len(), 1);
    assert!(
        msgs[0].raw.starts_with(b"Subject: Hello from Gmail"),
        "decoded RFC822 body"
    );
    // m1 is UNREAD + STARRED in the fixture ⇒ Flagged, not Seen.
    assert!(msgs[0].flags.contains(&mw_engine::Flag::Flagged));
    assert!(!msgs[0].flags.contains(&mw_engine::Flag::Seen));
    // The user label survives as a keyword (true multi-label fidelity).
    assert!(
        msgs[0]
            .flags
            .contains(&mw_engine::Flag::Keyword("Label_42".into()))
    );
}
