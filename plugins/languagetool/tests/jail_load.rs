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
/// host-mediated (POST to the LanguageTool endpoint), and answers with a canned
/// status + body so the guest's response handling can be driven down each branch.
struct RecordingHttp {
    last: Mutex<Option<HttpReq>>,
    reply: Mutex<(u16, Vec<u8>)>,
}

impl Default for RecordingHttp {
    fn default() -> Self {
        Self {
            last: Mutex::new(None),
            reply: Mutex::new((200, LT_JSON.to_vec())),
        }
    }
}

#[async_trait]
impl HttpFetcher for RecordingHttp {
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

// ── response handling + the request the guest builds (t19-e9) ─────────────────
//
// The guest's `protocol` / `transport` / `unsupported` error variants all arrive
// host-side as `PluginError::Runtime` — `mw_plugin::adapter::wit_to_plugin_err`
// flattens every non-limit, non-capability guest error into that one variant, so
// the MESSAGE is the only thing that distinguishes them at this seam.

/// A loaded component with `dlp-detector` + `net` granted and the LanguageTool
/// host allowlisted, reused across the cases in one test. Loading compiles the
/// component, which dominates this suite's runtime, so each test pays once.
struct Harness {
    http: Arc<RecordingHttp>,
    handle: mw_plugin::PluginHandle,
    _host: PluginHost,
}

fn harness() -> Harness {
    let http = Arc::new(RecordingHttp::default());
    let host = host_with(http.clone());
    let caps = vec![Capability::DlpDetector, Capability::Net];
    let m = manifest(caps.clone(), &[LT_HOST]);
    let handle = host.load(COMPONENT, &m, &grant(caps)).unwrap();
    Harness {
        http,
        handle,
        _host: host,
    }
}

impl Harness {
    /// Run one grammar check against a mock replying `status` + `body`.
    async fn check(
        &self,
        draft: &[u8],
        status: u16,
        body: &[u8],
    ) -> Result<Vec<String>, PluginError> {
        *self.http.reply.lock().unwrap() = (status, body.to_vec());
        self.http.last.lock().unwrap().take();
        self.handle.call_dlp_detect(draft.to_vec()).await
    }

    fn last(&self) -> Option<HttpReq> {
        self.http.last.lock().unwrap().clone()
    }
}

/// How a LanguageTool response becomes suggestion strings.
///
/// The shape that matters: a match with replacements renders as
/// `"<message> → <first replacement>"`, a match without renders as the message
/// alone, and a match with a blank message is DROPPED rather than emitted as an
/// empty suggestion the composer would render as a blank row.
#[tokio::test]
async fn matches_render_as_suggestions_and_blank_messages_are_dropped() {
    let h = harness();

    // No matches at all ⇒ no suggestions (a clean draft, not an error).
    assert!(
        h.check(b"All fine.", 200, br#"{ "matches": [] }"#)
            .await
            .unwrap()
            .is_empty()
    );

    // A response with no `matches` key is treated as "nothing to report".
    assert!(
        h.check(
            b"All fine.",
            200,
            br#"{ "software": { "name": "LanguageTool" } }"#
        )
        .await
        .unwrap()
        .is_empty()
    );

    let out = h
        .check(
            b"He go.",
            200,
            br#"{ "matches": [
                 { "message": "Verb form.", "replacements": [{ "value": "goes" }, { "value": "went" }] },
                 { "message": "Spelling.", "replacements": [] },
                 { "message": "  ", "replacements": [{ "value": "x" }] },
                 { "replacements": [{ "value": "y" }] },
                 { "message": "Empty replacement.", "replacements": [{ "value": "" }] },
                 { "message": "  Padded.  ", "replacements": [] }
               ] }"#,
        )
        .await
        .unwrap();

    assert_eq!(
        out,
        vec![
            // only the FIRST replacement is offered
            "Verb form. → goes".to_string(),
            "Spelling.".to_string(),
            // a blank replacement falls back to the bare message
            "Empty replacement.".to_string(),
            // messages are trimmed
            "Padded.".to_string(),
        ],
        "blank and missing messages must be dropped, not rendered empty"
    );
}

/// A malformed body is an error, and an HTTP failure from the endpoint surfaces
/// its status — neither is silently reported to the user as "no problems found",
/// which is what an empty `Ok(vec![])` would look like in the composer.
#[tokio::test]
async fn malformed_and_failed_responses_are_errors_not_empty_results() {
    let h = harness();

    let err = h
        .check(b"He go.", 200, b"<html>not json</html>")
        .await
        .unwrap_err();
    let msg = match &err {
        PluginError::Runtime(m) => m.clone(),
        other => panic!("expected a guest Runtime error, got {other:?}"),
    };
    assert!(msg.contains("malformed"), "{msg}");

    for status in [400u16, 429, 500, 503] {
        let err = h.check(b"He go.", status, b"{}").await.unwrap_err();
        let msg = match &err {
            PluginError::Runtime(m) => m.clone(),
            other => panic!("expected a guest Runtime error, got {other:?}"),
        };
        assert!(
            msg.contains(&status.to_string()),
            "HTTP {status} must surface: {msg}"
        );
    }
}

/// The draft text is percent-encoded into the form body, so a draft containing
/// `&`, `=` or a newline cannot inject extra form parameters (e.g. its own
/// `language=`), and non-ASCII survives as UTF-8 percent-escapes.
#[tokio::test]
async fn draft_text_is_percent_encoded_into_the_form_body() {
    let h = harness();
    h.check(
        "a&language=xx=1\nrésumé".as_bytes(),
        200,
        br#"{ "matches": [] }"#,
    )
    .await
    .unwrap();

    let body = String::from_utf8(h.last().unwrap().body.unwrap()).unwrap();
    assert!(body.starts_with("text="), "body = {body}");
    assert!(body.ends_with("&language=auto"), "body = {body}");
    assert!(
        body.matches("language=").count() == 1,
        "the draft must not inject a second `language=`: {body}"
    );
    assert!(body.contains("%26"), "`&` must be escaped: {body}");
    assert!(body.contains("%0A"), "newline must be escaped: {body}");
    assert!(body.contains("r%C3%A9sum%C3%A9"), "UTF-8 escapes: {body}");
}

/// ⚠️ RECORDS A DEFECT — the endpoint the guest posts to is COMPILED IN.
///
/// `src/component.rs` builds the URL from `DEFAULT_ENDPOINT_HOST`, so the manifest
/// `net_allowlist` is only ever a *filter*, never a *destination*. Pointing a
/// deployment at a self-hosted LanguageTool by editing `net_allowlist` — which
/// `plugin.toml` and `src/lib.rs` both describe as the way to do it — makes every
/// check fail `capability-denied` instead of reaching the internal server.
///
/// This test pins the behaviour as it actually is so the gap stays visible; it is
/// NOT an endorsement. Reported by t19-e9, not fixed (changing it means rebuilding
/// the committed component and regenerating its first-party digest).
#[tokio::test]
async fn endpoint_host_is_compiled_in_not_manifest_driven() {
    let h = harness();
    h.check(b"He go.", 200, br#"{ "matches": [] }"#)
        .await
        .unwrap();

    let url = h.last().unwrap().url;
    assert_eq!(
        url,
        format!(
            "https://{}{}",
            languagetool::DEFAULT_ENDPOINT_HOST,
            languagetool::CHECK_PATH
        ),
        "the guest always posts to the compiled-in endpoint"
    );

    // Self-hosting by allowlist alone: the guest still calls api.languagetool.org,
    // so the host refuses it. The internal server is never contacted.
    let self_hosted = Arc::new(RecordingHttp::default());
    let host = host_with(self_hosted.clone());
    let caps = vec![Capability::DlpDetector, Capability::Net];
    let m = manifest(caps.clone(), &["lt.internal.example"]);
    let handle = host.load(COMPONENT, &m, &grant(caps)).unwrap();

    let err = handle
        .call_dlp_detect(b"He go.".to_vec())
        .await
        .unwrap_err();
    assert!(
        matches!(err, PluginError::CapabilityDenied(_)),
        "allowlisting only the self-hosted host yields a denial, got {err:?}"
    );
    assert!(self_hosted.last.lock().unwrap().is_none());
}

/// The manifest id, the crate constant, and the net allowlist entry the component
/// actually needs must agree. `id` is the key `FIRST_PARTY_DIGESTS` in
/// `mw-server/src/v7_mount.rs` resolves the component by, and the allowlist entry
/// is the only thing standing between the guest and a `capability-denied` on every
/// call — a mismatch in either is a silent, total failure of the plugin.
#[test]
fn manifest_agrees_with_the_crate_constants() {
    let manifest_toml = include_str!("../plugin.toml");

    assert!(
        manifest_toml.contains(&format!("id = \"{}\"", languagetool::PLUGIN_ID)),
        "plugin.toml `id` must equal PLUGIN_ID ({})",
        languagetool::PLUGIN_ID
    );
    assert_eq!(languagetool::plugin_id(), languagetool::PLUGIN_ID);
    assert!(
        manifest_toml.contains(&format!("\"{}\"", languagetool::DEFAULT_ENDPOINT_HOST)),
        "the shipped net_allowlist must contain the endpoint the guest posts to ({})",
        languagetool::DEFAULT_ENDPOINT_HOST
    );
    assert!(
        languagetool::CHECK_PATH.starts_with('/'),
        "CHECK_PATH is appended to `https://<host>` and must be rooted"
    );
}
