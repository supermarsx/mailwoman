//! Assist gateway routes (plan §3 e9/e14, SPEC §14). Filled by e9; MOUNTED by e14.
//!
//! `/api/assist/*` — the gateway HTTP surface. **The server proxies the endpoint so
//! the browser never contacts the AI host:** `invoke` calls
//! [`mw_assist::AssistGateway::invoke`], whose adapter performs the outbound request
//! server-side (in-tree `reqwest`/rustls); the tokens stream back to the browser over
//! *our* connection, staying within the SPA CSP `connect-src 'self'` (mirroring the
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

use axum::extract::{Extension, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::json;

use mw_assist::{AssistCapability, AssistError, AssistGateway, AssistInput, DataScope};

use crate::AppState;

/// The live Assist gateway e14 injects (built from the 0008 `assist_config`).
pub(crate) type AssistHandle = Arc<AssistGateway>;

/// e14 merges this into `router()` and layers on the injected [`AssistHandle`].
pub(crate) fn assist_router() -> Router<AppState> {
    Router::new()
        .route("/api/assist/config", get(config))
        .route("/api/assist/invoke", post(invoke))
}

/// `GET /api/assist/config` — whether Assist is enabled for this deployment/user.
/// When `enabled=false` the web hides ALL Assist UI (unconfigured ⇒ zero surface).
async fn config(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(gateway): Extension<AssistHandle>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    Json(json!({
        "enabled": gateway.is_enabled(),
        // The "what left the device" disclosure copy (the web renders it verbatim).
        "disclosure": "Assist sends the selected message text (never E2EE-decrypted \
                       content or attachments by default) to the configured endpoint. \
                       Sending mail is always confirmed by you.",
    }))
    .into_response()
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
async fn invoke(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(gateway): Extension<AssistHandle>,
    Json(body): Json<InvokeReq>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    // `invoke` enforces capability → ceiling → redaction → rate-limit → audit, then
    // dispatches the (server-side) adapter request. We only ever receive the already
    // redacted token stream.
    match gateway
        .invoke(body.capability, body.scope, &body.input)
        .await
    {
        Ok(stream) => {
            // Map each token chunk to an SSE event; a stream error becomes a terminal
            // `error` event (never surfaces adapter internals or content).
            let sse = stream.map(|chunk| -> std::result::Result<Event, Infallible> {
                Ok(match chunk {
                    Ok(c) => Event::default()
                        .json_data(json!({ "delta": c.delta, "done": c.done }))
                        .unwrap_or_else(|_| Event::default().data("{}")),
                    Err(_) => Event::default()
                        .event("error")
                        .data(r#"{"error":"assist stream failed"}"#),
                })
            });
            Sse::new(sse).into_response()
        }
        Err(e) => assist_error(&e),
    }
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
