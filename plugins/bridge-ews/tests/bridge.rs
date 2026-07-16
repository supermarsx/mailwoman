//! Host integration test for the EWS bridge (t7-e11). Loads the committed
//! `wasm32-wasip2` component (`fixtures/bridge-ews.wasm`) through the REAL
//! `mw-plugin` jail and drives it as an `mw_engine::AccountBackend` against recorded
//! SOAP fixtures — indistinguishable from `mw-imap` to the engine. The injected
//! `HttpFetcher` performs the NTLMv2 401 challenge/response dance and dispatches each
//! SOAP request to the matching fixture, so the NTLM handshake is exercised
//! end-to-end through the sandbox.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use base64::Engine as _;
use mw_engine::{Flag, MailboxRole, SyncCursor};
use mw_plugin::{
    BasicCredentialProvider, BasicCredentials, Capability, Grant, HostServices, HttpFetcher,
    HttpReq, HttpResp, PluginHost, PluginLimits, PluginManifest, TrustRoot,
};

const GUEST: &[u8] = include_bytes!("../fixtures/bridge-ews.wasm");

const HIER: &str = include_str!("../fixtures/sync_folder_hierarchy.xml");
const ITEMS: &str = include_str!("../fixtures/sync_folder_items.xml");
const GETITEM: &str = include_str!("../fixtures/get_item_mime.xml");
const CREATED: &str = include_str!("../fixtures/create_item_sent.xml");
const RESOLVE: &str = include_str!("../fixtures/resolve_names.xml");

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// A minimal but well-formed NTLM CHALLENGE (Type 2) with a server challenge and a
/// 4-byte (EOL-only) TargetInfo blob at offset 48.
fn ntlm_type2() -> Vec<u8> {
    let mut m = Vec::new();
    m.extend_from_slice(b"NTLMSSP\0");
    m.extend_from_slice(&2u32.to_le_bytes());
    m.extend_from_slice(&[0u8; 8]); // TargetNameFields
    m.extend_from_slice(&0x0008_0201u32.to_le_bytes()); // flags (unicode|ntlm|target-info)
    m.extend_from_slice(&[0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]); // server challenge
    m.extend_from_slice(&[0u8; 8]); // reserved
    m.extend_from_slice(&4u16.to_le_bytes()); // TargetInfo len
    m.extend_from_slice(&4u16.to_le_bytes());
    m.extend_from_slice(&48u32.to_le_bytes()); // TargetInfo offset
    assert_eq!(m.len(), 48);
    m.extend_from_slice(&[0, 0, 0, 0]); // MsvAvEOL
    m
}

/// A fixture EWS credential provider standing in for the host's sealed per-account
/// credential store (t12: the guest pulls endpoint + creds through `basic-credentials`
/// exactly where OAuth bridges pull `oauth-token`). A non-empty `domain` selects the
/// NTLMv2 path; an empty `domain` selects HTTP Basic — the per-account rule the guest
/// applies. These are FIXTURE-SUPPLIED, non-placeholder values.
struct FixtureCreds {
    domain: String,
}

#[async_trait]
impl BasicCredentialProvider for FixtureCreds {
    async fn credentials(&self, _account: &str) -> Result<BasicCredentials, String> {
        Ok(BasicCredentials {
            user: "svc-mailwoman".into(),
            domain: self.domain.clone(),
            password: "fixture-secret".into(),
            workstation: "MAILWOMAN".into(),
            endpoint: "https://ews.example.com/EWS/Exchange.asmx".into(),
        })
    }
}

/// A fake EWS server: enforces the NTLM handshake (Type 1 → 401 challenge → Type 3 →
/// 200) OR accepts an up-front HTTP Basic header, and returns the fixture matching
/// the SOAP operation in the request body.
struct FakeEws {
    saw_type1: AtomicBool,
    saw_type3: AtomicBool,
    saw_basic: AtomicBool,
}

impl FakeEws {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            saw_type1: AtomicBool::new(false),
            saw_type3: AtomicBool::new(false),
            saw_basic: AtomicBool::new(false),
        })
    }

    fn dispatch(body: &str) -> &'static str {
        if body.contains("SyncFolderHierarchy") {
            HIER
        } else if body.contains("SyncFolderItems") {
            ITEMS
        } else if body.contains("GetItem") {
            GETITEM
        } else if body.contains("CreateItem") {
            CREATED
        } else if body.contains("ResolveNames") {
            RESOLVE
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

        if auth.starts_with("Basic ") {
            self.saw_basic.store(true, Ordering::SeqCst);
            let body = String::from_utf8_lossy(&req.body.unwrap_or_default()).into_owned();
            return Ok(HttpResp {
                status: 200,
                headers: vec![],
                body: FakeEws::dispatch(&body).as_bytes().to_vec(),
            });
        }

        if let Some(tok) = auth.strip_prefix("NTLM ") {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(tok.trim())
                .map_err(|e| e.to_string())?;
            let msg_type = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
            match msg_type {
                1 => {
                    self.saw_type1.store(true, Ordering::SeqCst);
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
                    self.saw_type3.store(true, Ordering::SeqCst);
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
        // No/unknown auth ⇒ demand NTLM.
        Ok(HttpResp {
            status: 401,
            headers: vec![("WWW-Authenticate".to_string(), "NTLM".to_string())],
            body: vec![],
        })
    }
}

fn manifest(net: &[&str]) -> PluginManifest {
    PluginManifest {
        id: "bridge-ews".into(),
        name: "Exchange EWS bridge".into(),
        version: "26.8.0".into(),
        signature: None,
        capabilities: vec![
            Capability::AccountBackend,
            Capability::Net,
            Capability::AddrbookSource,
        ],
        net_allowlist: net.iter().map(|s| s.to_string()).collect(),
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
        capabilities: vec![
            Capability::AccountBackend,
            Capability::Net,
            Capability::AddrbookSource,
        ],
        granted_by: "admin@test".into(),
        allow_unsigned: true,
    }
}

/// Wire the fake EWS server with an NTLM-configured account (non-empty domain).
fn host_with_fake(fake: Arc<FakeEws>) -> PluginHost {
    host_with(fake, "CORP")
}

/// Wire the fake EWS server with a per-account credential provider whose `domain`
/// selects the auth scheme (empty ⇒ Basic).
fn host_with(fake: Arc<FakeEws>, domain: &str) -> PluginHost {
    let services = HostServices {
        http: fake,
        basic_creds: Arc::new(FixtureCreds {
            domain: domain.into(),
        }),
        ..HostServices::default()
    };
    PluginHost::try_new(services, TrustRoot::empty()).unwrap()
}

#[tokio::test]
async fn ews_bridge_serves_mail_through_the_engine_with_ntlm() {
    let fake = FakeEws::new();
    let host = host_with_fake(fake.clone());
    let handle = host
        .load(GUEST, &manifest(&["ews.example.com"]), &grant())
        .unwrap();
    let backend = handle
        .as_account_backend()
        .expect("account-backend granted");

    // capabilities(): EWS advertises move/voting/recall, not reactions/focused-sync.
    let caps = backend.capabilities().await.unwrap();
    assert!(caps.r#move);

    // list_mailboxes() ⇒ SyncFolderHierarchy through the NTLM-authenticated transport.
    let boxes = backend.list_mailboxes().await.unwrap();
    assert!(boxes.iter().any(|b| b.role == MailboxRole::Inbox));
    assert!(boxes.iter().any(|b| b.role == MailboxRole::Sent));
    let inbox = boxes
        .iter()
        .find(|b| b.role == MailboxRole::Inbox)
        .unwrap()
        .clone();
    assert_eq!(inbox.total, 3);
    assert_eq!(inbox.unread, 1);

    // The NTLM handshake was actually exercised through the jail.
    assert!(
        fake.saw_type1.load(Ordering::SeqCst),
        "Type 1 negotiate sent"
    );
    assert!(
        fake.saw_type3.load(Ordering::SeqCst),
        "Type 3 authenticate sent"
    );

    // sync_mailbox() ⇒ SyncFolderItems; 2 added, 1 removed, opaque Plugin cursor.
    let delta = backend
        .sync_mailbox(&inbox.mailbox_ref, &SyncCursor::Plugin { opaque: vec![] })
        .await
        .unwrap();
    assert_eq!(delta.added.len(), 2);
    assert_eq!(delta.removed.len(), 1);
    assert!(
        matches!(delta.next_cursor, SyncCursor::Plugin { .. }),
        "bridge carries its EWS SyncState in an opaque Plugin cursor"
    );

    // fetch_raw() ⇒ GetItem MimeContent, decoded to real RFC822 bytes.
    let msgs = backend.fetch_raw(&delta.added).await.unwrap();
    assert_eq!(msgs.len(), 2);
    let body = String::from_utf8_lossy(&msgs[0].raw);
    assert!(body.contains("Subject: Quarterly numbers"), "got: {body}");

    // append()/submit() ⇒ CreateItem SendAndSaveCopy returns a usable ref.
    let sent = backend
        .append(&inbox.mailbox_ref, b"From: me\r\n\r\nhi", &[Flag::Seen])
        .await
        .unwrap();
    assert!(matches!(sent, mw_engine::MessageRef::Plugin { .. }));
}

#[tokio::test]
async fn ews_bridge_serves_mail_through_basic_auth() {
    // An account configured WITHOUT an NT domain ⇒ the guest selects HTTP Basic and
    // attaches `Authorization: Basic base64(user:pass)` up front (no NTLM dance).
    let fake = FakeEws::new();
    let host = host_with(fake.clone(), "");
    let handle = host
        .load(GUEST, &manifest(&["ews.example.com"]), &grant())
        .unwrap();
    let backend = handle
        .as_account_backend()
        .expect("account-backend granted");

    let boxes = backend.list_mailboxes().await.unwrap();
    assert!(boxes.iter().any(|b| b.role == MailboxRole::Inbox));

    // Basic was exercised; the NTLM handshake was NOT taken.
    assert!(
        fake.saw_basic.load(Ordering::SeqCst),
        "Basic Authorization header sent"
    );
    assert!(
        !fake.saw_type1.load(Ordering::SeqCst),
        "no NTLM negotiate on the Basic path"
    );

    let inbox = boxes
        .iter()
        .find(|b| b.role == MailboxRole::Inbox)
        .unwrap()
        .clone();
    let delta = backend
        .sync_mailbox(&inbox.mailbox_ref, &SyncCursor::Plugin { opaque: vec![] })
        .await
        .unwrap();
    let msgs = backend.fetch_raw(&delta.added).await.unwrap();
    assert!(String::from_utf8_lossy(&msgs[0].raw).contains("Subject: Quarterly numbers"));
}

#[tokio::test]
async fn ews_bridge_resolves_gal_through_addrbook_source() {
    let host = host_with_fake(FakeEws::new());
    let handle = host
        .load(GUEST, &manifest(&["ews.example.com"]), &grant())
        .unwrap();

    let hits = handle.call_addrbook_search("john".into()).await.unwrap();
    assert_eq!(hits.len(), 2);
    assert!(hits[0].contains("john.doe@corp.example"));
}

#[tokio::test]
async fn ews_bridge_denied_when_endpoint_not_in_allowlist() {
    // The manifest omits ews.example.com ⇒ the host `http-fetch` gate refuses the
    // guest's request before it ever reaches the (fake) network.
    let host = host_with_fake(FakeEws::new());
    let handle = host
        .load(GUEST, &manifest(&["other.example"]), &grant())
        .unwrap();
    let backend = handle.as_account_backend().unwrap();
    assert!(
        backend.list_mailboxes().await.is_err(),
        "out-of-allowlist EWS endpoint must be denied"
    );
}
