//! Host jail test for the LanguageTool component (t7-e13). Drives the REAL committed
//! `wasm32-wasip2` component (`tests/fixtures/languagetool.wasm`, built from
//! `src/component.rs` via `build.sh`) through the real `mw-plugin` wasmtime host —
//! the same jail e16 loads to prove the sandbox. Proves:
//!
//! * loads **capability-gated** and returns grammar suggestions vs a fixture
//!   LanguageTool response (host-mediated `http-fetch`, in-allowlist);
//! * the `dlp-detector` hook is **denied without the capability grant**;
//! * `http-fetch` is **denied when the target host is outside the net allowlist**
//!   (the core jail assertion) and when `net` itself is not granted;
//! * it runs **within its resource limits** (a normal call completes under a tight
//!   memory/deadline ceiling; the host survives).

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use mw_plugin::{
    Capability, Grant, HostServices, HttpFetcher, HttpReq, HttpResp, PluginError, PluginHost,
    PluginLimits, PluginManifest,
};

const COMPONENT: &[u8] = include_bytes!("fixtures/languagetool.wasm");
const LT_HOST: &str = "api.languagetool.org";

/// A fixture LanguageTool `/v2/check` response: one match with a replacement, one
/// without.
const LT_JSON: &[u8] = br#"{
  "matches": [
    { "message": "This verb form may be incorrect.",
      "replacements": [ { "value": "goes" }, { "value": "went" } ] },
    { "message": "Possible spelling mistake found.",
      "replacements": [] }
  ]
}"#;

/// Records the request the guest made so the test can assert the plugin is really
/// host-mediated (POST to the LanguageTool endpoint), then returns the fixture JSON.
#[derive(Default)]
struct RecordingHttp {
    last: Mutex<Option<HttpReq>>,
}

#[async_trait]
impl HttpFetcher for RecordingHttp {
    async fn fetch(&self, req: HttpReq) -> Result<HttpResp, String> {
        *self.last.lock().unwrap() = Some(req);
        Ok(HttpResp {
            status: 200,
            headers: vec![("content-type".into(), "application/json".into())],
            body: LT_JSON.to_vec(),
        })
    }
}

fn host_with(http: Arc<RecordingHttp>) -> PluginHost {
    let services = HostServices {
        http,
        ..HostServices::default()
    };
    PluginHost::try_new(services, mw_plugin::TrustRoot::empty()).unwrap()
}

fn manifest(caps: Vec<Capability>, allowlist: &[&str]) -> PluginManifest {
    PluginManifest {
        id: "languagetool".into(),
        name: "LanguageTool".into(),
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
        plugin_id: "languagetool".into(),
        capabilities: caps,
        granted_by: "admin@test".into(),
        allow_unsigned: true, // the committed fixture is unsigned
    }
}

// ── loads capability-gated + returns grammar suggestions vs a fixture ──────────

#[tokio::test]
async fn loads_capability_gated_and_returns_grammar_suggestions() {
    let http = Arc::new(RecordingHttp::default());
    let host = host_with(http.clone());
    let caps = vec![Capability::DlpDetector, Capability::Net];
    let m = manifest(caps.clone(), &[LT_HOST]);
    let handle = host.load(COMPONENT, &m, &grant(caps)).unwrap();

    let suggestions = handle
        .call_dlp_detect(b"He go to school every day.".to_vec())
        .await
        .expect("in-allowlist grammar check succeeds");

    assert_eq!(
        suggestions,
        vec![
            "This verb form may be incorrect. → goes".to_string(),
            "Possible spelling mistake found.".to_string(),
        ]
    );

    // Host-mediated: the guest POSTed to the LanguageTool endpoint, never a socket.
    let req = http
        .last
        .lock()
        .unwrap()
        .clone()
        .expect("a request was made");
    assert_eq!(req.method, "POST");
    assert!(req.url.contains(LT_HOST), "url = {}", req.url);
    assert!(req.url.ends_with("/v2/check"), "url = {}", req.url);
    let body = String::from_utf8(req.body.unwrap()).unwrap();
    assert!(body.starts_with("text="), "body = {body}");
}

// ── the core jail assertion: denied when the target host is outside the allowlist ─

#[tokio::test]
async fn http_fetch_denied_when_host_outside_allowlist() {
    let http = Arc::new(RecordingHttp::default());
    let host = host_with(http.clone());
    // `net` granted, but the allowlist does NOT include the LanguageTool host.
    let caps = vec![Capability::DlpDetector, Capability::Net];
    let m = manifest(caps.clone(), &["intranet.example"]);
    let handle = host.load(COMPONENT, &m, &grant(caps)).unwrap();

    let err = handle
        .call_dlp_detect(b"He go to school.".to_vec())
        .await
        .unwrap_err();
    assert!(
        matches!(err, PluginError::CapabilityDenied(_)),
        "out-of-allowlist host must be denied, got {err:?}"
    );
    // The host never dispatched the request to the injected fetcher.
    assert!(http.last.lock().unwrap().is_none());
}

#[tokio::test]
async fn http_fetch_denied_when_net_not_granted() {
    let http = Arc::new(RecordingHttp::default());
    let host = host_with(http.clone());
    // DlpDetector granted so the hook runs, but NO `net` ⇒ the http-fetch is refused.
    let caps = vec![Capability::DlpDetector];
    let m = manifest(caps.clone(), &[LT_HOST]);
    let handle = host.load(COMPONENT, &m, &grant(caps)).unwrap();

    let err = handle
        .call_dlp_detect(b"He go to school.".to_vec())
        .await
        .unwrap_err();
    assert!(
        matches!(err, PluginError::CapabilityDenied(_)),
        "no net capability must be denied, got {err:?}"
    );
    assert!(http.last.lock().unwrap().is_none());
}

// ── the hook itself is capability-gated ────────────────────────────────────────

#[tokio::test]
async fn dlp_hook_denied_without_capability() {
    let http = Arc::new(RecordingHttp::default());
    let host = host_with(http.clone());
    // Grant only `net` — the DLP-detector hook is not granted, so the host refuses to
    // call it at all (deny-by-default on the hook, before any guest code runs).
    let caps = vec![Capability::Net];
    let m = manifest(caps.clone(), &[LT_HOST]);
    let handle = host.load(COMPONENT, &m, &grant(caps)).unwrap();

    let err = handle
        .call_dlp_detect(b"He go to school.".to_vec())
        .await
        .unwrap_err();
    assert!(
        matches!(err, PluginError::CapabilityDenied(_)),
        "ungranted hook must be denied, got {err:?}"
    );
}

// ── runs within its resource limits (host survives) ───────────────────────────

#[tokio::test]
async fn respects_resource_limits() {
    let http = Arc::new(RecordingHttp::default());
    let host = host_with(http.clone());
    let caps = vec![Capability::DlpDetector, Capability::Net];
    // A tight-but-sufficient ceiling: the small grammar workload completes cleanly.
    let mut m = manifest(caps.clone(), &[LT_HOST]);
    m.limits = PluginLimits {
        memory_mb: 32,
        deadline_ms: 2_000,
        fuel: None,
    };
    let handle = host.load(COMPONENT, &m, &grant(caps)).unwrap();

    let suggestions = handle
        .call_dlp_detect(b"He go to school.".to_vec())
        .await
        .expect("completes within its resource limits");
    assert_eq!(suggestions.len(), 2);
}
