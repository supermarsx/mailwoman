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

/// Records the request the guest made, and answers with a canned status + body so
/// the guest's response handling can be driven down each branch.
struct MockOcs {
    last: Mutex<Option<HttpReq>>,
    reply: Mutex<(u16, Vec<u8>)>,
}

impl Default for MockOcs {
    fn default() -> Self {
        Self {
            last: Mutex::new(None),
            reply: Mutex::new((200, OCS_JSON.to_vec())),
        }
    }
}

#[async_trait]
impl HttpFetcher for MockOcs {
    async fn fetch(&self, req: HttpReq) -> Result<HttpResp, String> {
        *self.last.lock().unwrap() = Some(req);
        let (status, body) = self.reply.lock().unwrap().clone();
        Ok(HttpResp {
            status,
            headers: vec![("content-type".into(), "application/json".into())],
            body,
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

// ── response handling: the branches a real Nextcloud will exercise (t19-e9) ────
//
// The guest's error variants (`protocol` / `transport` / `unsupported`) all arrive
// host-side as `PluginError::Runtime` — `mw_plugin::adapter::wit_to_plugin_err`
// flattens every non-limit, non-capability guest error into that one variant, so
// the MESSAGE is the only thing that distinguishes them here.

/// A loaded component with the full grant and the Nextcloud host allowlisted,
/// reused across the cases in one test. Loading compiles the component, which
/// dominates the runtime of this suite, so each test pays for it once.
struct Harness {
    http: Arc<MockOcs>,
    handle: mw_plugin::PluginHandle,
    _host: PluginHost,
}

fn harness() -> Harness {
    let http = Arc::new(MockOcs::default());
    let host = host_with(http.clone());
    let caps = vec![Capability::MessagePipeline, Capability::Net];
    let m = manifest(caps.clone(), &[NC_HOST]);
    let handle = host.load(COMPONENT, &m, &grant(caps)).unwrap();
    Harness {
        http,
        handle,
        _host: host,
    }
}

impl Harness {
    /// Drive one share request against a mock replying `status` + `body`.
    async fn share(
        &self,
        request: &[u8],
        status: u16,
        body: &[u8],
    ) -> Result<Vec<u8>, PluginError> {
        *self.http.reply.lock().unwrap() = (status, body.to_vec());
        self.http.last.lock().unwrap().take();
        self.handle.call_message_out(request.to_vec()).await
    }

    fn last(&self) -> Option<HttpReq> {
        self.http.last.lock().unwrap().clone()
    }

    /// The urlencoded OCS form body of the most recent request.
    fn last_form(&self) -> String {
        String::from_utf8(self.last().expect("a request was made").body.unwrap()).unwrap()
    }
}

fn runtime_message(err: &PluginError) -> String {
    match err {
        PluginError::Runtime(m) => m.clone(),
        other => panic!("expected a guest Runtime error, got {other:?}"),
    }
}

/// **OCS answers HTTP 200 even when the share was refused** — the real outcome is
/// in `ocs.meta.statuscode`. A guest that only checked the HTTP status would hand
/// the composer a "successful" share with no URL, so this is the branch that
/// matters most. Both OCS success codes (v1 `100`, v2 `200`) are accepted, and a
/// response with no `meta` at all is trusted when it carries a URL.
#[tokio::test]
async fn ocs_verdict_is_read_from_the_body_not_the_http_status() {
    let h = harness();

    // A logical failure under HTTP 200: the code AND the operator-facing message
    // must both survive into the error.
    let err = h
        .share(
            SHARE_REQUEST,
            200,
            br#"{ "ocs": { "meta": {
                 "status": "failure", "statuscode": 404,
                 "message": "Wrong path, file/folder doesn't exist"
               }, "data": [] } }"#,
        )
        .await
        .unwrap_err();
    let msg = runtime_message(&err);
    assert!(msg.contains("404"), "the OCS code must survive: {msg}");
    assert!(
        msg.contains("Wrong path"),
        "the OCS message must survive: {msg}"
    );
    assert!(
        h.last().is_some(),
        "the request WAS made — only the OCS verdict failed"
    );

    // A failure with no `message` still errors, with a fallback description.
    let msg = runtime_message(
        &h.share(
            SHARE_REQUEST,
            200,
            br#"{ "ocs": { "meta": { "statuscode": 403 } } }"#,
        )
        .await
        .unwrap_err(),
    );
    assert!(msg.contains("403"), "{msg}");
    assert!(msg.contains("share creation failed"), "{msg}");

    // OCS v1 success code.
    let url = h
        .share(
            SHARE_REQUEST,
            200,
            br#"{ "ocs": {
                 "meta": { "status": "ok", "statuscode": 100, "message": "OK" },
                 "data": { "url": "https://cloud.example.com/s/V1Token" }
               } }"#,
        )
        .await
        .expect("OCS v1 status 100 is a success");
    assert_eq!(
        String::from_utf8(url).unwrap(),
        "https://cloud.example.com/s/V1Token"
    );

    // No `meta` block: the status check is skipped rather than failing closed.
    let url = h
        .share(
            SHARE_REQUEST,
            200,
            br#"{ "ocs": { "data": { "url": "https://cloud.example.com/s/NoMeta" } } }"#,
        )
        .await
        .expect("a URL with no meta block is usable");
    assert_eq!(
        String::from_utf8(url).unwrap(),
        "https://cloud.example.com/s/NoMeta"
    );
}

/// A real HTTP failure from the Nextcloud host is reported with its status, and
/// every shape of unusable success body is an error rather than an empty or
/// partial link handed to the composer.
#[tokio::test]
async fn http_failures_and_unusable_bodies_never_yield_a_link() {
    let h = harness();

    for status in [400u16, 401, 500, 503] {
        let msg = runtime_message(
            &h.share(SHARE_REQUEST, status, b"<html>nope</html>")
                .await
                .unwrap_err(),
        );
        assert!(
            msg.contains(&status.to_string()),
            "HTTP {status} must surface: {msg}"
        );
    }

    let cases: [(&str, &[u8]); 4] = [
        ("not JSON at all", b"<?xml version=\"1.0\"?><ocs/>"),
        (
            "no `ocs` envelope",
            br#"{ "data": { "url": "https://x/y" } }"#,
        ),
        (
            "ok but no url",
            br#"{ "ocs": { "meta": { "statuscode": 200 }, "data": {} } }"#,
        ),
        (
            "url present but empty",
            br#"{ "ocs": { "meta": { "statuscode": 200 }, "data": { "url": "" } } }"#,
        ),
    ];
    for (why, body) in cases {
        assert!(
            h.share(SHARE_REQUEST, 200, body).await.is_err(),
            "{why} must not yield a share link"
        );
    }
}

/// A share request missing a required field is refused **before** any HTTP is
/// attempted — the guest never speaks to Nextcloud on an incomplete request.
#[tokio::test]
async fn incomplete_share_requests_are_refused_without_a_fetch() {
    let h = harness();
    let cases: [(&str, &[u8]); 3] = [
        ("missing base_url", br#"{ "path": "/a.zip" }"#),
        (
            "missing path",
            br#"{ "base_url": "https://cloud.example.com" }"#,
        ),
        ("not JSON", b"base_url=https://cloud.example.com"),
    ];
    for (why, request) in cases {
        assert!(
            h.share(request, 200, OCS_JSON).await.is_err(),
            "{why} must be refused"
        );
        assert!(h.last().is_none(), "{why} must not reach the network");
    }
}

/// What the guest puts on the wire: the public-link share type, a trimmed base
/// URL, and the optional fields only when non-empty.
///
/// The empty-password case is the one with teeth. An empty string must not become
/// `password=` — Nextcloud reads that as "set an empty password", which publishes
/// an unprotected link the user believed was protected.
#[tokio::test]
async fn the_wire_format_of_a_share_request() {
    let h = harness();

    // shareType 3 = public link. Any other integer makes a user/group/e-mail share
    // that the recipient of the message cannot open.
    h.share(SHARE_REQUEST, 200, OCS_JSON)
        .await
        .expect("share created");
    assert_eq!(nextcloud_plugin::SHARE_TYPE_PUBLIC_LINK, 3);
    assert!(
        h.last_form().contains(&format!(
            "shareType={}",
            nextcloud_plugin::SHARE_TYPE_PUBLIC_LINK
        )),
        "form = {}",
        h.last_form()
    );

    // A trailing slash on base_url must not produce `//ocs/...`, which 404s.
    h.share(
        br#"{ "base_url": "https://cloud.example.com/", "path": "/a.zip" }"#,
        200,
        OCS_JSON,
    )
    .await
    .expect("share created");
    let url = h.last().unwrap().url;
    assert_eq!(
        url,
        format!(
            "https://cloud.example.com{}?format=json",
            nextcloud_plugin::OCS_SHARES_PATH
        ),
        "trailing slash must be trimmed"
    );

    // Optional fields present and percent-encoded.
    h.share(
        br#"{
          "base_url": "https://cloud.example.com",
          "path": "/Documents/big.zip",
          "password": "hunter2 &=?",
          "expiry": "2027-01-31"
        }"#,
        200,
        OCS_JSON,
    )
    .await
    .expect("share created");
    let form = h.last_form();
    assert!(
        form.contains("password=hunter2%20%26%3D%3F"),
        "form = {form}"
    );
    assert!(form.contains("expireDate=2027-01-31"), "form = {form}");

    // Optional fields present but EMPTY are omitted entirely.
    h.share(
        br#"{
          "base_url": "https://cloud.example.com",
          "path": "/Documents/big.zip",
          "password": "",
          "expiry": ""
        }"#,
        200,
        OCS_JSON,
    )
    .await
    .expect("share created");
    let form = h.last_form();
    assert!(
        !form.contains("password="),
        "an empty password must be omitted, not sent blank: {form}"
    );
    assert!(!form.contains("expireDate="), "form = {form}");
}

/// The manifest id is the key the host resolves a first-party component by
/// (`FIRST_PARTY_DIGESTS` in `mw-server/src/v7_mount.rs`), and it is NOT the
/// package name — the crate is `nextcloud-plugin`, the plugin is `nextcloud`, and
/// the host carries an explicit alias for that gap. Pinned so a rename of one
/// without the others cannot pass unnoticed.
#[test]
fn manifest_id_matches_the_crate_constant() {
    let manifest_toml = include_str!("../plugin.toml");
    assert!(
        manifest_toml.contains(&format!("id = \"{}\"", nextcloud_plugin::PLUGIN_ID)),
        "plugin.toml `id` must equal PLUGIN_ID ({})",
        nextcloud_plugin::PLUGIN_ID
    );
    assert_eq!(nextcloud_plugin::plugin_id(), nextcloud_plugin::PLUGIN_ID);
}

/// The shipped `net_allowlist` is EMPTY by design: Nextcloud is self-hosted, so
/// there is no host the plugin may reach until an admin names one. A default
/// entry appearing here would silently grant outbound reach to every deployment
/// that never edited the manifest.
#[test]
fn shipped_net_allowlist_is_empty_by_design() {
    let manifest_toml = include_str!("../plugin.toml");
    assert!(
        manifest_toml.contains("net_allowlist = []"),
        "the shipped manifest must not pre-authorise any host"
    );
}
