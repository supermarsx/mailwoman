//! Assist gateway routes (plan §3 e9/e14, SPEC §14). Filled by e9; MOUNTED by e14.
//!
//! `/api/assist/*` — the gateway HTTP surface. **The server proxies the endpoint so
//! the browser never contacts the AI host:** `invoke` calls
//! [`mw_assist::AssistGateway::invoke_disclosed`], whose adapter performs the outbound
//! request server-side (in-tree `reqwest`/rustls); the tokens stream back to the browser
//! over *our* connection, staying within the SPA CSP `connect-src 'self'` (mirroring the
//! `/errors` tunnel). The AI host is never a browser origin.
//!
//! Enforcement (capability grant → data-class ceiling → **redaction** of
//! E2EE-decrypted content + attachments → rate-limit → content-free audit) all lives
//! in `mw-assist` and runs inside `invoke`; this route never bypasses it and never
//! logs mail content (§21.1). The audit row carries capability + scope summary +
//! endpoint host only.
//!
//! ## Injection (e14)
//! The live [`mw_assist::AssistGateway`] (built from the 0008 `assist_config` row +
//! its audit sink) is injected as a request extension ([`AssistHandle`]). When Assist
//! is unconfigured the gateway reports `Disabled` and the web hides all Assist UI.
#![allow(dead_code)]

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Extension, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::{Stream, StreamExt};
use serde::Deserialize;
use serde_json::json;

use mw_assist::{
    AssistCapability, AssistError, AssistGateway, AssistInput, ChatStream, DataScope,
    InvokeDisclosure, ProposalFilter, ProposedAction,
};

use crate::AppState;

/// SSE keep-alive cadence. A model turn can idle for minutes between tokens, which is
/// long enough for a reverse proxy (nginx `proxy_read_timeout` defaults to 60s) or a
/// cloud load balancer to drop the connection mid-answer. A comment frame every
/// [`KEEPALIVE`] keeps the stream alive without inventing content.
const KEEPALIVE: Duration = Duration::from_secs(15);

/// The live Assist gateway e14 injects (built from the 0008 `assist_config`).
pub(crate) type AssistHandle = Arc<AssistGateway>;

/// e14 merges this into `router()` and layers on the injected [`AssistHandle`].
pub(crate) fn assist_router() -> Router<AppState> {
    Router::new()
        .route("/api/assist/config", get(config))
        .route("/api/assist/invoke", post(invoke))
}

/// `GET /api/assist/config` — the gateway surface the web reads at boot.
///
/// The web keys its whole Assist UI off `availability` + `capabilities`: a disabled
/// gateway reports `availability: "disabled"`, no capabilities and a null endpoint
/// host, and the client renders nothing (§14 hard-hide). `enabled` + `disclosure` are
/// kept alongside for non-browser callers.
async fn config(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(gateway): Extension<AssistHandle>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    Json(config_body(&gateway)).into_response()
}

/// Build the `/api/assist/config` body. Split out so the shape is unit-testable
/// without a live router.
fn config_body(gateway: &AssistGateway) -> serde_json::Value {
    let enabled = gateway.is_enabled();
    let ceiling = gateway.data_ceiling();
    json!({
        "enabled": enabled,
        "availability": if enabled { "enabled" } else { "disabled" },
        "capabilities": gateway.granted_capabilities(),
        "endpoint_host": gateway.endpoint_host(),
        // The admin ceiling, reported as the web's flags. Both are false unless an
        // admin explicitly opted in — and always false while the gateway is off.
        "include_e2ee": enabled && ceiling.include_e2ee,
        "include_attachments": enabled && ceiling.include_attachments,
        // The "what left the device" disclosure copy (the web renders it verbatim).
        "disclosure": "Assist sends the selected message text (never E2EE-decrypted \
                       content or attachments by default) to the configured endpoint. \
                       Sending mail is always confirmed by you.",
    })
}

/// The invoke request body: which chat-shaped capability, the per-call data scope
/// (clamped to the admin ceiling inside the gateway), and the input (prompt +
/// mailbox context, redacted before anything leaves the server).
#[derive(Debug, Deserialize)]
struct InvokeReq {
    capability: AssistCapability,
    #[serde(default)]
    scope: DataScope,
    input: AssistInput,
}

/// `POST /api/assist/invoke` — run a chat-shaped capability and stream the reply back
/// as Server-Sent Events. The gateway does the outbound request; the browser only
/// ever talks to us.
///
/// ## The wire contract (SPEC §14.3)
/// Three frame kinds, always in this order:
/// 1. `event: disclosure` — `{endpoint_host, sent[], withheld[]}`, what redaction
///    actually forwarded and withheld on THIS call. Sent before any token, so the UI
///    can state it while the reply streams.
/// 2. default events — `{"delta": "…", "done": false}`, one per visible token run.
/// 3. `event: done` — `{"actions": [{id, tool, summary, would_send}]}`, terminal. The
///    actions are the ones the model **proposed**; nothing here executes them (R4).
///
/// A transport failure ends the stream with `event: error` instead, carrying no
/// adapter internals and no content.
async fn invoke(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(gateway): Extension<AssistHandle>,
    Json(body): Json<InvokeReq>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    let cap = body.capability;
    // `invoke_disclosed` enforces capability → ceiling → redaction → rate-limit →
    // audit, then dispatches the (server-side) adapter request. We only ever receive
    // the already redacted token stream, plus the redaction outcome to disclose.
    match gateway.invoke_disclosed(cap, body.scope, &body.input).await {
        Ok((stream, disclosure)) => {
            let frames = invoke_frames(stream, ProposalFilter::for_capability(cap), disclosure);
            let sse = frames.map(|(name, data)| -> std::result::Result<Event, Infallible> {
                let event = match name {
                    Some(n) => Event::default().event(n),
                    None => Event::default(),
                };
                Ok(event.data(data))
            });
            let mut resp = Sse::new(sse)
                .keep_alive(KeepAlive::new().interval(KEEPALIVE).text("keep-alive"))
                .into_response();
            let h = resp.headers_mut();
            // Ask nginx (and anything honouring the convention) not to buffer, and
            // forbid any intermediary from transforming the stream. Without these a
            // proxy can hold the whole reply until the turn ends.
            h.insert("x-accel-buffering", HeaderValue::from_static("no"));
            h.insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("no-cache, no-transform"),
            );
            resp
        }
        Err(e) => assist_error(&e),
    }
}

/// One SSE frame: an optional event name (`None` ⇒ the default token event) plus its
/// JSON payload. Producing frames rather than [`Event`]s keeps the contract
/// unit-testable — an `Event` exposes nothing once built.
type Frame = (Option<&'static str>, String);

/// Which part of the reply the frame stream is emitting.
enum Stage {
    Disclosure(InvokeDisclosure),
    Body,
    /// The stream ended: release any text the proposal filter was holding.
    Flush,
    /// Emit the terminal frame with the proposals parsed out of the reply.
    Finish(Vec<ProposedAction>),
    End,
}

struct FrameState {
    inner: ChatStream,
    /// Taken at [`Stage::Flush`] (finishing consumes the filter).
    filter: Option<ProposalFilter>,
    stage: Stage,
}

/// Turn one adapter token stream into the `/api/assist/invoke` frame sequence.
fn invoke_frames(
    stream: ChatStream,
    filter: ProposalFilter,
    disclosure: InvokeDisclosure,
) -> impl Stream<Item = Frame> + Send {
    let init = FrameState {
        inner: stream,
        filter: Some(filter),
        stage: Stage::Disclosure(disclosure),
    };
    futures_util::stream::unfold(init, |mut st| async move {
        loop {
            match std::mem::replace(&mut st.stage, Stage::Body) {
                Stage::Disclosure(d) => {
                    let data = serde_json::to_string(&d).unwrap_or_else(|_| "{}".into());
                    return Some(((Some("disclosure"), data), st));
                }
                Stage::Body => match st.inner.next().await {
                    None => st.stage = Stage::Flush,
                    Some(Ok(chunk)) => {
                        let visible = match st.filter.as_mut() {
                            Some(f) => f.push(&chunk.delta),
                            None => chunk.delta.clone(),
                        };
                        if chunk.done {
                            st.stage = Stage::Flush;
                        }
                        if !visible.is_empty() {
                            let data = json!({ "delta": visible, "done": false }).to_string();
                            return Some(((None, data), st));
                        }
                    }
                    Some(Err(_)) => {
                        st.stage = Stage::End;
                        return Some((
                            (
                                Some("error"),
                                r#"{"error":"assist stream failed"}"#.to_string(),
                            ),
                            st,
                        ));
                    }
                },
                Stage::Flush => {
                    let (trailing, actions) = st
                        .filter
                        .take()
                        .map_or_else(|| (String::new(), Vec::new()), ProposalFilter::finish);
                    st.stage = Stage::Finish(actions);
                    if !trailing.is_empty() {
                        let data = json!({ "delta": trailing, "done": false }).to_string();
                        return Some(((None, data), st));
                    }
                }
                Stage::Finish(actions) => {
                    st.stage = Stage::End;
                    let data = json!({ "actions": actions }).to_string();
                    return Some(((Some("done"), data), st));
                }
                Stage::End => return None,
            }
        }
    })
}

/// Map an [`AssistError`] to an HTTP response. `Disabled` ⇒ `404` (the feature is
/// off for this scope — the web hides it anyway); denied capability ⇒ `403`;
/// rate-limit ⇒ `429`; endpoint/transport ⇒ `502`. No mail content is ever included.
pub(crate) fn assist_error(e: &AssistError) -> Response {
    let (code, msg): (StatusCode, String) = match e {
        AssistError::Disabled => (StatusCode::NOT_FOUND, "assist disabled".into()),
        AssistError::CapabilityDenied(_) => (
            StatusCode::FORBIDDEN,
            "assist capability not granted".into(),
        ),
        AssistError::ScopeExceeded(_) => (
            StatusCode::FORBIDDEN,
            "assist data-class ceiling exceeded".into(),
        ),
        AssistError::RateLimited => (
            StatusCode::TOO_MANY_REQUESTS,
            "assist rate limit exceeded".into(),
        ),
        AssistError::Endpoint(_) => {
            tracing::warn!("assist endpoint error");
            (StatusCode::BAD_GATEWAY, "assist endpoint error".into())
        }
        AssistError::Unimplemented => {
            (StatusCode::NOT_IMPLEMENTED, "assist not implemented".into())
        }
    };
    (code, Json(json!({ "error": msg }))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mw_assist::{
        AdapterConfig, AssistConfig, ChatPayload, ChatStream, ContentKind, ContextItem,
        EndpointAdapter, Result as AssistResult, StreamChunk,
    };
    use std::sync::Mutex;

    #[test]
    fn error_mapping_is_coarse_and_content_free() {
        assert_eq!(
            assist_error(&AssistError::Disabled).status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            assist_error(&AssistError::CapabilityDenied(AssistCapability::Draft)).status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            assist_error(&AssistError::RateLimited).status(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            assist_error(&AssistError::Endpoint("boom".into())).status(),
            StatusCode::BAD_GATEWAY
        );
    }

    /// A fake adapter that records the redacted payload it was handed + its host, so
    /// the test can assert E2EE content never reached it (the server-proxy path).
    struct SpyAdapter {
        seen: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl EndpointAdapter for SpyAdapter {
        async fn chat(&self, payload: &ChatPayload) -> AssistResult<ChatStream> {
            self.seen.lock().unwrap().push(payload.prompt.clone());
            let chunks = vec![
                Ok(StreamChunk {
                    delta: "hello".into(),
                    done: false,
                }),
                Ok(StreamChunk {
                    delta: String::new(),
                    done: true,
                }),
            ];
            Ok(futures_util::stream::iter(chunks).boxed())
        }
        async fn embed(&self, _input: &str) -> AssistResult<Vec<f32>> {
            Ok(vec![])
        }
        async fn transcribe(&self, _audio: &[u8], _mime: &str) -> AssistResult<String> {
            Ok(String::new())
        }
        fn host(&self) -> String {
            "spy.internal".into()
        }
    }

    /// The route drives the gateway, which redacts before dispatch: an E2EE-decrypted
    /// context item is NEVER present in the payload the adapter (⇒ the AI host) sees,
    /// even though the plain item is. This is the server-proxy + redaction guarantee.
    #[tokio::test]
    async fn invoke_redacts_e2ee_before_it_reaches_the_endpoint() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let gateway = AssistGateway::new(AssistConfig {
            enabled: true,
            capability_grants: vec![AssistCapability::Summarize],
            data_ceiling: DataScope {
                accounts: vec!["acct".into()],
                folders: vec![],
                include_e2ee: false,
                include_attachments: false,
            },
            adapter: Some(AdapterConfig::LocalProcess {
                program: "unused".into(),
                args: vec![],
            }),
            rate_limit_per_min: None,
        })
        .with_adapter(Arc::new(SpyAdapter { seen: seen.clone() }));

        let input = AssistInput {
            prompt: "summarize".into(),
            context: vec![
                ContextItem {
                    account: "acct".into(),
                    folder: String::new(),
                    text: "PLAIN-VISIBLE".into(),
                    kind: ContentKind::Plain,
                },
                ContextItem {
                    account: "acct".into(),
                    folder: String::new(),
                    text: "SECRET-E2EE".into(),
                    kind: ContentKind::E2eeDecrypted,
                },
            ],
        };
        let scope = DataScope {
            accounts: vec!["acct".into()],
            ..Default::default()
        };
        let mut stream = gateway
            .invoke(AssistCapability::Summarize, scope, &input)
            .await
            .expect("invoke succeeds");
        // Drain so the adapter runs.
        while stream.next().await.is_some() {}

        let payload = seen.lock().unwrap().join("\n");
        assert!(payload.contains("PLAIN-VISIBLE"), "plain content forwarded");
        assert!(
            !payload.contains("SECRET-E2EE"),
            "E2EE content must NEVER reach the endpoint by default"
        );
    }

    /// The config body the web actually keys off: a disabled gateway reports the
    /// hard-hide shape, and the legacy `enabled`/`disclosure` fields stay put.
    #[test]
    fn config_body_hides_everything_when_disabled() {
        let gateway = AssistGateway::new(AssistConfig::default());
        let body = config_body(&gateway);
        assert_eq!(body["availability"], json!("disabled"));
        assert_eq!(body["capabilities"], json!([]));
        assert_eq!(body["endpoint_host"], json!(null));
        assert_eq!(body["include_e2ee"], json!(false));
        assert_eq!(body["include_attachments"], json!(false));
        assert_eq!(body["enabled"], json!(false));
        assert!(body["disclosure"].is_string());
    }

    /// An enabled gateway reports the granted capabilities in the web's wire spelling
    /// plus the endpoint host, which is what the disclosure names.
    #[test]
    fn config_body_reports_grants_and_endpoint_when_enabled() {
        let gateway = AssistGateway::new(AssistConfig {
            enabled: true,
            capability_grants: vec![
                AssistCapability::Assistant,
                AssistCapability::SearchSemantic,
            ],
            ..AssistConfig::default()
        })
        .with_adapter(Arc::new(SpyAdapter {
            seen: Arc::new(Mutex::new(Vec::new())),
        }));
        let body = config_body(&gateway);
        assert_eq!(body["availability"], json!("enabled"));
        assert_eq!(
            body["capabilities"],
            json!(["assistant", "search-semantic"]),
            "capabilities use the kebab-case wire names the web declares"
        );
        assert_eq!(body["endpoint_host"], json!("spy.internal"));
        assert_eq!(
            body["include_e2ee"],
            json!(false),
            "the default ceiling still excludes E2EE"
        );
    }

    /// Build a canned adapter stream from token runs.
    fn canned(parts: &[&str]) -> ChatStream {
        let mut chunks: Vec<mw_assist::Result<StreamChunk>> = parts
            .iter()
            .map(|p| {
                Ok(StreamChunk {
                    delta: (*p).to_string(),
                    done: false,
                })
            })
            .collect();
        chunks.push(Ok(StreamChunk {
            delta: String::new(),
            done: true,
        }));
        futures_util::stream::iter(chunks).boxed()
    }

    async fn frames_of(stream: ChatStream, cap: AssistCapability) -> Vec<Frame> {
        invoke_frames(
            stream,
            ProposalFilter::for_capability(cap),
            InvokeDisclosure {
                endpoint_host: "spy.internal".into(),
                sent: vec!["your prompt".into()],
                withheld: vec!["1 attachment(s)".into()],
            },
        )
        .collect()
        .await
    }

    /// The full `/api/assist/invoke` contract on one assistant turn: the disclosure
    /// leads, the proposal block never reaches the user as text, and the terminal
    /// `done` frame carries the proposals for human review.
    #[tokio::test]
    async fn invoke_frames_emit_disclosure_then_deltas_then_actions() {
        let stream = canned(&[
            "I can look ",
            "that up.\n<<<MW_",
            "ACTIONS\n[{\"tool\":\"mail.search\",\"summary\":\"Search for Bob\"}]",
        ]);
        let frames = frames_of(stream, AssistCapability::Assistant).await;

        assert_eq!(
            frames[0].0,
            Some("disclosure"),
            "disclosure leads the stream"
        );
        let disclosure: serde_json::Value = serde_json::from_str(&frames[0].1).unwrap();
        assert_eq!(disclosure["endpoint_host"], json!("spy.internal"));
        assert_eq!(disclosure["withheld"], json!(["1 attachment(s)"]));

        let text: String = frames
            .iter()
            .filter(|(name, _)| name.is_none())
            .map(|(_, data)| {
                serde_json::from_str::<serde_json::Value>(data).unwrap()["delta"]
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .collect();
        assert_eq!(text, "I can look that up.\n");
        assert!(
            !text.contains("MW_ACTIONS") && !text.contains("mail.search"),
            "the proposal block is never shown as prose"
        );

        let last = frames.last().expect("terminal frame");
        assert_eq!(last.0, Some("done"));
        let done: serde_json::Value = serde_json::from_str(&last.1).unwrap();
        assert_eq!(done["actions"][0]["tool"], json!("mail.search"));
        assert_eq!(done["actions"][0]["id"], json!("act-1"));
        assert_eq!(done["actions"][0]["would_send"], json!(false));
    }

    /// A non-assistant capability streams its text verbatim and proposes nothing.
    #[tokio::test]
    async fn non_assistant_capabilities_propose_nothing() {
        let frames = frames_of(canned(&["Summary."]), AssistCapability::Summarize).await;
        let last = frames.last().expect("terminal frame");
        assert_eq!(last.0, Some("done"));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&last.1).unwrap()["actions"],
            json!([])
        );
    }

    /// A mid-stream transport failure ends the stream with the coarse `error` frame —
    /// no adapter internals, no content, and no terminal `done`.
    #[tokio::test]
    async fn a_stream_error_terminates_with_the_error_frame() {
        let chunks: Vec<mw_assist::Result<StreamChunk>> = vec![
            Ok(StreamChunk {
                delta: "partial".into(),
                done: false,
            }),
            Err(AssistError::Endpoint("secret adapter detail".into())),
        ];
        let frames = frames_of(
            futures_util::stream::iter(chunks).boxed(),
            AssistCapability::Assistant,
        )
        .await;
        let last = frames.last().expect("terminal frame");
        assert_eq!(last.0, Some("error"));
        assert!(
            !last.1.contains("secret adapter detail"),
            "adapter internals never reach the browser"
        );
    }

    #[tokio::test]
    async fn disabled_gateway_reports_disabled() {
        let gateway = AssistGateway::new(AssistConfig::default());
        assert!(!gateway.is_enabled());
        // `invoke`'s Ok type (ChatStream) is not Debug, so match rather than unwrap.
        let result = gateway
            .invoke(
                AssistCapability::Draft,
                DataScope::default(),
                &AssistInput::prompt("x"),
            )
            .await;
        assert!(matches!(result, Err(AssistError::Disabled)));
    }
}
