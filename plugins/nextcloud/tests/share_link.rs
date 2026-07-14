//! Host test for the Nextcloud share-link component (t7-e13). Drives the REAL
//! committed `wasm32-wasip2` component (`tests/fixtures/nextcloud.wasm`, built from
//! `src/component.rs` via `build.sh`) through the real `mw-plugin` wasmtime host.
//! Proves the share link is created against a mock OCS endpoint, host-mediated and
//! under a net allowlist, and that an out-of-allowlist Nextcloud host is denied.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use mw_plugin::{
    Capability, Grant, HostServices, HttpFetcher, HttpReq, HttpResp, PluginError, PluginHost,
    PluginLimits, PluginManifest,
};

const COMPONENT: &[u8] = include_bytes!("fixtures/nextcloud.wasm");
const NC_HOST: &str = "cloud.example.com";
const SHARE_URL: &str = "https://cloud.example.com/s/AbCdEf123456";

/// A canned Nextcloud OCS create-share JSON response.
const OCS_JSON: &[u8] = br#"{
  "ocs": {
    "meta": { "status": "ok", "statuscode": 200, "message": "OK" },
    "data": {
      "id": "42",
      "share_type": 3,
      "token": "AbCdEf123456",
      "url": "https://cloud.example.com/s/AbCdEf123456"
    }
  }
}"#;

#[derive(Default)]
struct MockOcs {
    last: Mutex<Option<HttpReq>>,
}

#[async_trait]
impl HttpFetcher for MockOcs {
    async fn fetch(&self, req: HttpReq) -> Result<HttpResp, String> {
        *self.last.lock().unwrap() = Some(req);
        Ok(HttpResp {
            status: 200,
            headers: vec![("content-type".into(), "application/json".into())],
            body: OCS_JSON.to_vec(),
        })
    }
}

fn host_with(http: Arc<MockOcs>) -> PluginHost {
    let services = HostServices {
        http,
        ..HostServices::default()
    };
    PluginHost::try_new(services, mw_plugin::TrustRoot::empty()).unwrap()
}

fn manifest(caps: Vec<Capability>, allowlist: &[&str]) -> PluginManifest {
    PluginManifest {
        id: "nextcloud".into(),
        name: "Nextcloud".into(),
        version: "0".into(),
        signature: None,
        capabilities: caps,
        net_allowlist: allowlist.iter().map(|s| (*s).to_string()).collect(),
        limits: PluginLimits {
            memory_mb: 64,
            deadline_ms: 5_000,
            fuel: None,
        },
    }
}

fn grant(caps: Vec<Capability>) -> Grant {
    Grant {
        plugin_id: "nextcloud".into(),
        capabilities: caps,
        granted_by: "admin@test".into(),
        allow_unsigned: true,
    }
}

const SHARE_REQUEST: &[u8] = br#"{
  "base_url": "https://cloud.example.com",
  "path": "/Documents/big report.zip",
  "expiry": "2026-12-31"
}"#;

#[tokio::test]
async fn creates_share_link_against_mock_ocs() {
    let http = Arc::new(MockOcs::default());
    let host = host_with(http.clone());
    let caps = vec![Capability::MessagePipeline, Capability::Net];
    let m = manifest(caps.clone(), &[NC_HOST]);
    let handle = host.load(COMPONENT, &m, &grant(caps)).unwrap();

    let out = handle
        .call_message_out(SHARE_REQUEST.to_vec())
        .await
        .expect("share link created");
    assert_eq!(String::from_utf8(out).unwrap(), SHARE_URL);

    // The guest hit the OCS create-share endpoint, host-mediated, with the OCS header.
    let req = http
        .last
        .lock()
        .unwrap()
        .clone()
        .expect("a request was made");
    assert_eq!(req.method, "POST");
    assert!(
        req.url
            .contains("/ocs/v2.php/apps/files_sharing/api/v1/shares"),
        "url = {}",
        req.url
    );
    assert!(req.url.contains("format=json"), "url = {}", req.url);
    assert!(
        req.headers
            .iter()
            .any(|(k, v)| k.eq_ignore_ascii_case("ocs-apirequest") && v == "true"),
        "missing OCS-APIRequest header"
    );
    let body = String::from_utf8(req.body.unwrap()).unwrap();
    assert!(body.contains("shareType=3"), "body = {body}"); // public link
    assert!(body.contains("expireDate="), "body = {body}");
    // The space in the path is percent-encoded.
    assert!(body.contains("big%20report.zip"), "body = {body}");
}

#[tokio::test]
async fn denied_when_nextcloud_host_outside_allowlist() {
    let http = Arc::new(MockOcs::default());
    let host = host_with(http.clone());
    let caps = vec![Capability::MessagePipeline, Capability::Net];
    // Allowlist a DIFFERENT host than the request's base_url.
    let m = manifest(caps.clone(), &["other-cloud.example"]);
    let handle = host.load(COMPONENT, &m, &grant(caps)).unwrap();

    let err = handle
        .call_message_out(SHARE_REQUEST.to_vec())
        .await
        .unwrap_err();
    assert!(
        matches!(err, PluginError::CapabilityDenied(_)),
        "out-of-allowlist Nextcloud host must be denied, got {err:?}"
    );
    assert!(http.last.lock().unwrap().is_none());
}

#[tokio::test]
async fn denied_when_net_not_granted() {
    let http = Arc::new(MockOcs::default());
    let host = host_with(http.clone());
    let caps = vec![Capability::MessagePipeline];
    let m = manifest(caps.clone(), &[NC_HOST]);
    let handle = host.load(COMPONENT, &m, &grant(caps)).unwrap();

    let err = handle
        .call_message_out(SHARE_REQUEST.to_vec())
        .await
        .unwrap_err();
    assert!(
        matches!(err, PluginError::CapabilityDenied(_)),
        "no net capability must be denied, got {err:?}"
    );
    assert!(http.last.lock().unwrap().is_none());
}
