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

async fn preview(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PreviewRequest>,
) -> Response {
    let session = match super::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let policy = match crate::attachment_policy::effective(&state, &session).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    if !policy.preview_allowed || !policy.download_allowed {
        return crate::attachment_policy::refusal("attachmentPreviewDisabled");
    }
    if !matches!(body.format.as_str(), "eml" | "tnef") {
        return error(StatusCode::BAD_REQUEST, "unsupported preview format");
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
    let Ok(_permit) = WORKERS.try_acquire() else {
        return error(
            StatusCode::TOO_MANY_REQUESTS,
            "attachment preview workers are busy; retry shortly",
        );
    };
    let result = tokio::time::timeout(std::time::Duration::from_secs(15), run(bin, &body)).await;
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
