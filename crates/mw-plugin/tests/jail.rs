//! Integration tests for the `mw-plugin` wasmtime WASI-p2 jail (t7-e1), driven
//! against a REAL committed component (`tests/fixtures/guest.wasm`, built from
//! `tests/guest-fixture/`). These prove the security core end-to-end:
//!
//! * a capability NOT granted is denied (host import refuses; guest cannot act);
//! * `http-fetch` outside the `net_allowlist` is refused;
//! * a wall-clock deadline / fuel / memory trip is a typed `LimitExceeded` and the
//!   host SURVIVES (no panic);
//! * an unsigned component fails to load without `allow_unsigned`, and loads (with
//!   the banner signal) under it; a signed round-trip verifies;
//! * the account-backend adapter round-trips a mock guest through the WIT boundary.

use std::sync::Arc;

use async_trait::async_trait;
use ed25519_dalek::{Signer, SigningKey};
use mw_engine::{MessageRef, RawMailboxRef};
use mw_plugin::{
    Capability, Grant, HostServices, HttpFetcher, HttpReq, HttpResp, PluginError, PluginHost,
    PluginLimits, PluginManifest, TrustRoot,
};

const GUEST: &[u8] = include_bytes!("fixtures/guest.wasm");

fn manifest(
    caps: Vec<Capability>,
    allowlist: &[&str],
    limits: PluginLimits,
    signature: Option<String>,
) -> PluginManifest {
    PluginManifest {
        id: "test-guest".into(),
        name: "Test Guest".into(),
        version: "0".into(),
        signature,
        capabilities: caps,
        net_allowlist: allowlist.iter().map(|s| s.to_string()).collect(),
        limits,
    }
}

fn grant(caps: Vec<Capability>, allow_unsigned: bool) -> Grant {
    Grant {
        plugin_id: "test-guest".into(),
        capabilities: caps,
        granted_by: "admin@test".into(),
        allow_unsigned,
    }
}

fn fast_limits() -> PluginLimits {
    PluginLimits {
        memory_mb: 64,
        deadline_ms: 5_000,
        fuel: None,
    }
}

struct MockHttp;
#[async_trait]
impl HttpFetcher for MockHttp {
    async fn fetch(&self, _req: HttpReq) -> Result<HttpResp, String> {
        Ok(HttpResp {
            status: 200,
            headers: vec![],
            body: b"ok".to_vec(),
        })
    }
}

fn host_with_mock_http() -> PluginHost {
    let services = HostServices {
        http: Arc::new(MockHttp),
        ..HostServices::default()
    };
    PluginHost::try_new(services, TrustRoot::empty()).unwrap()
}

// ── account-backend adapter round-trip THROUGH the WIT boundary ────────────────

#[tokio::test]
async fn account_backend_adapter_round_trips_through_wit() {
    let host = PluginHost::new();
    let caps = vec![Capability::AccountBackend];
    let m = manifest(caps.clone(), &[], fast_limits(), None);
    let handle = host.load(GUEST, &m, &grant(caps, true)).unwrap();

    let backend = handle
        .as_account_backend()
        .expect("account-backend granted");

    // capabilities()
    let c = backend.capabilities().await.unwrap();
    assert!(c.idle, "guest advertised idle");
    assert!(c.r#move, "guest advertised move");

    // list_mailboxes() — proves record + list + string marshalling.
    let boxes = backend.list_mailboxes().await.unwrap();
    assert_eq!(boxes.len(), 2);
    assert_eq!(boxes[0].mailbox_ref.name, "INBOX");
    assert_eq!(boxes[0].role, mw_engine::MailboxRole::Inbox);
    assert_eq!(boxes[1].mailbox_ref.name, "Archive");

    // fetch_raw() — nested records + byte lists + the opaque message-ref round-trip.
    let mref = MessageRef::Imap {
        mailbox: RawMailboxRef {
            name: "INBOX".into(),
            uidvalidity: 1,
        },
        uidvalidity: 1,
        uid: 7,
    };
    let msgs = backend
        .fetch_raw(std::slice::from_ref(&mref))
        .await
        .unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(
        msgs[0].message_ref, mref,
        "message-ref survives the boundary"
    );
    assert!(msgs[0].raw.starts_with(b"Subject: hello"));
}

#[tokio::test]
async fn as_account_backend_denied_without_capability() {
    let host = PluginHost::new();
    // No AccountBackend capability granted.
    let m = manifest(vec![Capability::DlpDetector], &[], fast_limits(), None);
    let handle = host
        .load(GUEST, &m, &grant(vec![Capability::DlpDetector], true))
        .unwrap();
    assert!(handle.as_account_backend().is_none());
}

// ── capability deny + net allowlist enforcement ───────────────────────────────

#[tokio::test]
async fn net_capability_not_granted_is_denied() {
    let host = host_with_mock_http();
    // addrbook-source granted, but NOT net → the guest's http-fetch is refused.
    let caps = vec![Capability::AddrbookSource];
    let m = manifest(caps.clone(), &["allowed.example"], fast_limits(), None);
    let handle = host.load(GUEST, &m, &grant(caps, true)).unwrap();

    let err = handle
        .call_addrbook_search("https://allowed.example/x".into())
        .await
        .unwrap_err();
    assert!(
        matches!(err, PluginError::CapabilityDenied(_)),
        "got {err:?}"
    );
}

#[tokio::test]
async fn http_fetch_outside_allowlist_refused() {
    let host = host_with_mock_http();
    let caps = vec![Capability::AddrbookSource, Capability::Net];
    let m = manifest(caps.clone(), &["allowed.example"], fast_limits(), None);
    let handle = host.load(GUEST, &m, &grant(caps, true)).unwrap();

    // Outside the allowlist ⇒ denied.
    let err = handle
        .call_addrbook_search("https://evil.example/x".into())
        .await
        .unwrap_err();
    assert!(
        matches!(err, PluginError::CapabilityDenied(_)),
        "got {err:?}"
    );
}

#[tokio::test]
async fn http_fetch_inside_allowlist_allowed() {
    let host = host_with_mock_http();
    let caps = vec![Capability::AddrbookSource, Capability::Net];
    let m = manifest(caps.clone(), &["allowed.example"], fast_limits(), None);
    let handle = host.load(GUEST, &m, &grant(caps, true)).unwrap();

    let out = handle
        .call_addrbook_search("https://allowed.example/path".into())
        .await
        .unwrap();
    assert_eq!(out, vec!["status=200".to_string()]);
}

// ── resource limits: a trip is LimitExceeded and the host SURVIVES ─────────────

#[tokio::test]
async fn wall_clock_deadline_trips_and_host_survives() {
    let host = PluginHost::new();
    let caps = vec![Capability::DlpDetector];
    let limits = PluginLimits {
        memory_mb: 64,
        deadline_ms: 100, // tight
        fuel: None,
    };
    let m = manifest(caps.clone(), &[], limits, None);
    let handle = host.load(GUEST, &m, &grant(caps, true)).unwrap();

    let err = handle.call_dlp_detect(b"loop".to_vec()).await.unwrap_err();
    assert!(matches!(err, PluginError::LimitExceeded(_)), "got {err:?}");

    // Host survives: a fresh benign call on a NEW handle still works.
    let m2 = manifest(vec![Capability::DlpDetector], &[], fast_limits(), None);
    let h2 = host
        .load(GUEST, &m2, &grant(vec![Capability::DlpDetector], true))
        .unwrap();
    assert!(
        h2.call_dlp_detect(b"benign".to_vec())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn fuel_exhaustion_trips() {
    let host = PluginHost::new();
    let caps = vec![Capability::DlpDetector];
    let limits = PluginLimits {
        memory_mb: 64,
        deadline_ms: 60_000, // generous so FUEL trips first
        fuel: Some(2_000_000),
    };
    let m = manifest(caps.clone(), &[], limits, None);
    let handle = host.load(GUEST, &m, &grant(caps, true)).unwrap();

    let err = handle.call_dlp_detect(b"loop".to_vec()).await.unwrap_err();
    assert!(matches!(err, PluginError::LimitExceeded(_)), "got {err:?}");
}

#[tokio::test]
async fn memory_ceiling_trips() {
    let host = PluginHost::new();
    let caps = vec![Capability::DlpDetector];
    let limits = PluginLimits {
        memory_mb: 16, // small ceiling
        deadline_ms: 60_000,
        fuel: None,
    };
    let m = manifest(caps.clone(), &[], limits, None);
    let handle = host.load(GUEST, &m, &grant(caps, true)).unwrap();

    let err = handle.call_dlp_detect(b"alloc".to_vec()).await.unwrap_err();
    assert!(matches!(err, PluginError::LimitExceeded(_)), "got {err:?}");
}

// ── signed-registry verification + allow_unsigned banner ──────────────────────

#[test]
fn unsigned_fails_closed_without_policy() {
    let host = PluginHost::new();
    let m = manifest(vec![], &[], fast_limits(), None);
    match host.load(GUEST, &m, &grant(vec![], false)) {
        Err(PluginError::SignatureInvalid(_)) => {}
        Err(other) => panic!("expected SignatureInvalid, got {other:?}"),
        Ok(_) => panic!("unsigned component must fail closed"),
    }
}

#[test]
fn unsigned_loads_under_policy_with_banner_signal() {
    let host = PluginHost::new();
    let m = manifest(vec![], &[], fast_limits(), None);
    let handle = host.load(GUEST, &m, &grant(vec![], true)).unwrap();
    assert!(handle.is_unsigned(), "banner signal must be set");
}

#[test]
fn signed_round_trip_verifies() {
    let sk = SigningKey::from_bytes(&[3u8; 32]);
    let sig = sk.sign(GUEST);
    let hex: String = sig.to_bytes().iter().map(|b| format!("{b:02x}")).collect();

    let trust = TrustRoot::from_public_keys(&[sk.verifying_key().to_bytes()]).unwrap();
    let mut host = PluginHost::new();
    host.set_trust_root(trust);

    let m = manifest(vec![], &[], fast_limits(), Some(hex));
    let handle = host.load(GUEST, &m, &grant(vec![], false)).unwrap();
    assert!(!handle.is_unsigned(), "verified ⇒ no banner");
}

#[test]
fn bad_signature_is_rejected() {
    let sk = SigningKey::from_bytes(&[3u8; 32]);
    let other = SigningKey::from_bytes(&[4u8; 32]);
    let sig = sk.sign(GUEST);
    let hex: String = sig.to_bytes().iter().map(|b| format!("{b:02x}")).collect();

    // Trust a DIFFERENT key ⇒ the signature must not verify.
    let trust = TrustRoot::from_public_keys(&[other.verifying_key().to_bytes()]).unwrap();
    let mut host = PluginHost::new();
    host.set_trust_root(trust);

    let m = manifest(vec![], &[], fast_limits(), Some(hex));
    match host.load(GUEST, &m, &grant(vec![], false)) {
        Err(PluginError::SignatureInvalid(_)) => {}
        Err(other) => panic!("expected SignatureInvalid, got {other:?}"),
        Ok(_) => panic!("bad signature must be rejected"),
    }
}
