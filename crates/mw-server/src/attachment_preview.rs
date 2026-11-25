//! Authenticated, bounded attachment previews through the disposable render worker.
use crate::AppState;
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use base64::Engine as _;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

static WORKERS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(2);
const MAX_BYTES: usize = 4 * 1024 * 1024;
const MAX_OUTPUT: usize = 16 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PreviewRequest {
    format: String,
    bytes_base64: String,
}

pub(crate) fn router() -> Router<AppState> {
    Router::new().route(
        "/api/attachments/preview",
        post(preview).layer(DefaultBodyLimit::max(6 * 1024 * 1024)),
    )
}

async fn preview(State(state): State<AppState>, request: axum::extract::Request) -> Response {
    // t63: authenticate before the (up to 6 MiB) body is read or parsed. The
    // same Json extractor runs afterwards, so its rejections are unchanged.
    let headers: HeaderMap = request.headers().clone();
    let session = match super::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let body = match <Json<PreviewRequest> as axum::extract::FromRequest<AppState>>::from_request(request, &state).await {
        Ok(Json(body)) => body,
        Err(rejection) => return rejection.into_response(),
    };
    let policy = match crate::attachment_policy::effective(&state, &session).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    if !policy.preview_allowed || !policy.download_allowed {
        return crate::attachment_policy::refusal("attachmentPreviewDisabled");
    }
    if !matches!(body.format.as_str(), "eml" | "tnef" | "image") {
        return error(StatusCode::BAD_REQUEST, "unsupported preview format");
    }
    if body.format=="image" {
        let allowed=crate::settings_routes::effective_for_session(&state,&session,"reader.filePreviews").await.ok().flatten().is_some_and(|entry|entry.value["enabled"]==true&&entry.value["convertImages"]==true&&entry.value["types"]["image"]!=false);
        if !allowed{return crate::attachment_policy::refusal("attachmentPreviewDisabled");}
    }
    let bytes = match base64::engine::general_purpose::STANDARD.decode(&body.bytes_base64) {
        Ok(bytes) if !bytes.is_empty() && bytes.len() <= MAX_BYTES => bytes,
        _ => {
            return error(
                StatusCode::BAD_REQUEST,
                "attachment is empty, invalid, or exceeds 4 MiB",
            );
        }
    };
    if policy.max_bytes > 0 && bytes.len() as u64 > policy.max_bytes {
        return crate::attachment_policy::refusal("attachmentTooLarge");
    }
    drop(bytes);
    let Some(bin) = &state.render_bin else {
        return super::render_unavailable();
    };
    let wait = crate::rate_leeway::knobs(&state.store, crate::rate_leeway::domain_of(&session.username))
        .await
        .queue();
    let Some(_permit) = crate::rate_leeway::acquire_pool("attachmentPreview", &WORKERS, wait).await else {
        crate::health_probes::render_busy();
        return crate::rate_leeway::tag(
            error(
                StatusCode::TOO_MANY_REQUESTS,
                "attachment preview workers are busy; retry shortly",
            ),
            crate::rate_leeway::pool_refusal("attachmentPreview", 2),
        );
    };
    let result = crate::health_probes::track_render_timeout(
        std::time::Duration::from_secs(15),
        run(bin, &body),
    )
    .await;
    match result {
        Ok(Ok(mut value)) => {
            // The worker parses these fields from the supplied MIME bytes;
            // caller-selected names/format are not attachment metadata.
            if let Some(files) = value["preview"]["attachments"].as_array_mut() {
                files.retain(|file| {
                    let metadata = crate::attachment_policy::Metadata {
                        name: file["name"].as_str().unwrap_or("").into(),
                        mime: file["mime"]
                            .as_str()
                            .unwrap_or("application/octet-stream")
                            .into(),
                        size: file["size"].as_u64().unwrap_or(u64::MAX),
                    };
                    crate::attachment_policy::authorize(
                        &policy,
                        &metadata,
                        &crate::attachment_policy::DownloadQuery::default(),
                    )
                    .is_ok()
                });
            }
            let mut response = Json(value).into_response();
            response.headers_mut().insert(
                axum::http::header::CACHE_CONTROL,
                axum::http::HeaderValue::from_static("private, no-store"),
            );
            response
        }
        _ => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "could not preview this attachment within the safety limits",
        ),
    }
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({"error":message}))).into_response()
}

async fn run(bin: &std::path::Path, request: &PreviewRequest) -> anyhow::Result<Value> {
    use std::process::Stdio;
    let mut command = tokio::process::Command::new(bin);
    command
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    // Windows needs its system directory, but no application credentials go to the worker.
    #[cfg(windows)]
    if let Some(root) = std::env::var_os("SystemRoot") {
        command.env("SystemRoot", root);
    }
    let mut child = command.spawn()?;
    let mut input = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("missing worker stdin"))?;
    let payload =
        serde_json::to_vec(&json!({"format":request.format,"bytesBase64":request.bytes_base64}))?;
    input.write_all(&payload).await?;
    input.write_all(b"\n").await?;
    input.shutdown().await?;
    drop(input);
    let output = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("missing worker stdout"))?;
    let mut bytes = Vec::new();
    output
        .take((MAX_OUTPUT + 1) as u64)
        .read_to_end(&mut bytes)
        .await?;
    anyhow::ensure!(bytes.len() <= MAX_OUTPUT, "worker output exceeds limit");
    anyhow::ensure!(child.wait().await?.success(), "worker failed");
    let value: Value = serde_json::from_slice(&bytes)?;
    let preview = value
        .get("preview")
        .filter(|value| value.is_object())
        .ok_or_else(|| anyhow::anyhow!("worker lacks preview support"))?;
    anyhow::ensure!(
        preview.get("format").and_then(Value::as_str) == Some(request.format.as_str()),
        "worker format mismatch"
    );
    Ok(json!({"preview":preview}))
}

#[cfg(test)]
mod body_before_auth_tests {
    use super::*;
    use axum::body::Body;
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
    use tower::ServiceExt;

    /// t63 (3): an unauthenticated preview request is refused before its body
    /// is read: not one chunk of a 6 MiB body is polled.
    #[tokio::test]
    async fn an_unauthenticated_preview_body_is_never_read() {
        let (router, _push) = crate::build_app_full(
            crate::AppConfig {
                db_path: std::env::temp_dir().join(format!("mw-t63-preview-{}.db", uuid::Uuid::new_v4())).to_string_lossy().into_owned(),
                server_key_hex: None,
                web_dir: None,
                cookie_secure: false,
                mode: crate::ServerMode::Engine,
                hardening: Default::default(),
                security: Default::default(),
            },
            crate::V6Config { admin_enabled: false, ..Default::default() },
        )
        .await
        .unwrap();
        let polled = Arc::new(AtomicUsize::new(0));
        let counter = polled.clone();
        let chunks = futures_util::stream::iter((0..96).map(move |_| {
            counter.fetch_add(1, Ordering::Relaxed);
            Ok::<_, std::io::Error>(axum::body::Bytes::from(vec![b'a'; 64 * 1024]))
        }));
        let mut request = axum::http::Request::builder()
            .method("POST")
            .uri("/api/attachments/preview")
            .header("host", "localhost")
            .header("origin", "http://localhost")
            .header("content-type", "application/json")
            .header("cookie", "mw_session=not-a-session; mw_csrf=x")
            .header("x-csrf-token", "x")
            .body(Body::from_stream(chunks))
            .unwrap();
        request.extensions_mut().insert(axum::extract::ConnectInfo(
            "127.0.0.1:48231".parse::<std::net::SocketAddr>().unwrap(),
        ));
        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(polled.load(Ordering::Relaxed), 0, "body chunks read before authentication");
    }
}
