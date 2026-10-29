//! Large attachments to the linked Nextcloud, for a share link instead of an
//! attachment (t39 L7). The browser sends the file in chunks; each chunk goes to
//! the account's own upload area with Nextcloud's chunked upload (v2: `MKCOL`
//! the transfer, `PUT` numbered chunks, `MOVE .file` onto the destination), so
//! the server never holds more than one chunk of the file.
//!
//! Every request resolves the verified link again (account fence, instance
//! revision, sealed credential) through [`crate::nextcloud_gateway`], and every
//! URL is built from that link: a caller names a file name and a transfer id,
//! never a host or another account's path.
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{post, put};
use axum::{Json, Router};
use mw_egress::DavMethod;
use serde::Deserialize;
use serde_json::json;

use super::{DavOp, DavTarget, NextcloudError, NextcloudGateway};
use crate::AppState;

/// The folder shared files are placed in, in the linked account's files.
pub(crate) const FOLDER: &str = "/Mail attachments";
/// Bytes per chunk. Nextcloud's v2 chunking wants 5 MiB or more for every chunk
/// but the last, and a guarded DAV request carries at most 8 MiB.
pub(crate) const CHUNK_BYTES: usize = 5 * 1024 * 1024;
/// The largest file offered for a share link.
pub(crate) const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024 * 1024;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/nextcloud/upload", post(start))
        .route(
            "/api/nextcloud/upload/{transfer}/{chunk}",
            put(chunk).layer(DefaultBodyLimit::max(CHUNK_BYTES + 1024)),
        )
        .route("/api/nextcloud/upload/{transfer}/finish", post(finish))
        .route("/api/nextcloud/upload/{transfer}/abort", post(abort))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartReq {
    account_id: String,
    #[serde(default)]
    instance_id: Option<String>,
    name: String,
    size: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChunkQuery {
    account_id: String,
    #[serde(default)]
    instance_id: Option<String>,
    path: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FinishReq {
    account_id: String,
    #[serde(default)]
    instance_id: Option<String>,
    path: String,
    size: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AbortReq {
    account_id: String,
    #[serde(default)]
    instance_id: Option<String>,
}

fn refused(code: StatusCode, message: &str) -> Response {
    (code, Json(json!({ "error": message }))).into_response()
}

fn cloud_error(error: &NextcloudError) -> Response {
    let NextcloudError::Api(reason) = error;
    tracing::warn!(reason = %reason, "nextcloud upload step failed");
    let (status, message) = match reason.as_str() {
        "cloud storage full" => (
            StatusCode::INSUFFICIENT_STORAGE,
            "Your Nextcloud storage is full.",
        ),
        "cloud permission denied" => (
            StatusCode::FORBIDDEN,
            "Nextcloud refused the upload for this account.",
        ),
        "cloud timeout" => (
            StatusCode::GATEWAY_TIMEOUT,
            "Nextcloud did not answer in time.",
        ),
        _ => (StatusCode::BAD_GATEWAY, "Nextcloud refused the upload."),
    };
    refused(status, message)
}

/// The account's gateway, after the session, feature and account checks.
async fn gateway(
    state: &AppState,
    headers: &HeaderMap,
    account_id: &str,
    instance_id: Option<&str>,
) -> Result<std::sync::Arc<dyn NextcloudGateway>, Response> {
    let session =
        crate::user_router::feature_session(state, headers, "integrations.nextcloud").await?;
    if session.account_id != account_id {
        return Err(refused(
            StatusCode::CONFLICT,
            "Account changed; pick the file again.",
        ));
    }
    crate::nextcloud_gateway::resolve(state, headers, instance_id).await
}

/// The last path segment of a browser file name, without control characters,
/// or `None` when nothing usable is left.
pub(crate) fn file_name(raw: &str) -> Option<String> {
    let last = raw.rsplit(['/', '\\']).next().unwrap_or("");
    let clean: String = last.chars().filter(|c| !c.is_control()).collect();
    let clean = clean.trim().trim_end_matches('.').trim().to_owned();
    if clean.is_empty() || clean == "." || clean == ".." || clean.chars().count() > 200 {
        return None;
    }
    Some(clean)
}

/// `name`, then `stem (2).ext`, `stem (3).ext` …
pub(crate) fn candidate(name: &str, n: u32) -> String {
    if n < 2 {
        return name.to_owned();
    }
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => format!("{stem} ({n}).{ext}"),
        _ => format!("{name} ({n})"),
    }
}

/// A transfer id is 32 lowercase hex characters (checked again when the URL is built).
fn transfer_id(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// The destination must be a file directly inside [`FOLDER`].
fn destination(path: &str) -> bool {
    path.strip_prefix(FOLDER)
        .and_then(|rest| rest.strip_prefix('/'))
        .is_some_and(|name| file_name(name).as_deref() == Some(name))
}

async fn exists(gateway: &dyn NextcloudGateway, path: &str) -> Result<bool, NextcloudError> {
    match gateway
        .dav(DavOp {
            target: DavTarget::File(path),
            method: DavMethod::Propfind,
            body: Some(("application/xml", crate::nextcloud_transport::PROPERTIES)),
            depth: Some(0),
            destination: None,
        })
        .await
    {
        Ok(_) => Ok(true),
        Err(NextcloudError::Api(reason)) if reason == "cloud item not found" => Ok(false),
        Err(error) => Err(error),
    }
}

/// Size of a file from a depth-0 PROPFIND.
async fn stored_size(gateway: &dyn NextcloudGateway, path: &str) -> Result<u64, NextcloudError> {
    let bytes = gateway
        .dav(DavOp {
            target: DavTarget::File(path),
            method: DavMethod::Propfind,
            body: Some(("application/xml", crate::nextcloud_transport::PROPERTIES)),
            depth: Some(0),
            destination: None,
        })
        .await?;
    let text = String::from_utf8_lossy(&bytes);
    let mut reader = quick_xml::Reader::from_str(&text);
    let mut inside = false;
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Start(tag)) => {
                inside = tag.local_name().as_ref() == b"getcontentlength";
            }
            Ok(quick_xml::events::Event::Text(value)) if inside => {
                return String::from_utf8_lossy(value.as_ref())
                    .trim()
                    .parse()
                    .map_err(|_| NextcloudError::Api("invalid size".into()));
            }
            Ok(quick_xml::events::Event::End(_)) => inside = false,
            Ok(quick_xml::events::Event::Eof) | Err(_) => {
                return Err(NextcloudError::Api("missing size".into()));
            }
            _ => {}
        }
    }
}

/// Create the folder (if missing), choose a free name in it and open a transfer.
pub(crate) async fn begin(
    gateway: &dyn NextcloudGateway,
    name: &str,
) -> Result<(String, String), NextcloudError> {
    match gateway
        .dav(DavOp {
            target: DavTarget::File(&format!("{FOLDER}/")),
            method: DavMethod::Mkcol,
            body: None,
            depth: None,
            destination: None,
        })
        .await
    {
        Ok(_) => {}
        Err(NextcloudError::Api(reason)) if reason == "cloud item already exists" => {}
        Err(error) => return Err(error),
    }
    let mut path = None;
    for n in 1..=50 {
        let next = format!("{FOLDER}/{}", candidate(name, n));
        if !exists(gateway, &next).await? {
            path = Some(next);
            break;
        }
    }
    let path = path.ok_or_else(|| NextcloudError::Api("no free file name".into()))?;
    let transfer = uuid::Uuid::new_v4().simple().to_string();
    gateway
        .dav(DavOp {
            target: DavTarget::Upload {
                transfer: &transfer,
                chunk: None,
            },
            method: DavMethod::Mkcol,
            body: None,
            depth: None,
            destination: Some(&path),
        })
        .await?;
    Ok((transfer, path))
}

/// Assemble the chunks onto `path` and check the stored size.
pub(crate) async fn complete(
    gateway: &dyn NextcloudGateway,
    transfer: &str,
    path: &str,
    size: u64,
) -> Result<(), NextcloudError> {
    gateway
        .dav(DavOp {
            target: DavTarget::Upload {
                transfer,
                chunk: Some(".file"),
            },
            method: DavMethod::Move,
            body: None,
            depth: None,
            destination: Some(path),
        })
        .await?;
    let stored = stored_size(gateway, path).await?;
    if stored != size {
        return Err(NextcloudError::Api("stored size differs".into()));
    }
    Ok(())
}

/// `POST /api/nextcloud/upload {accountId, instanceId?, name, size}` → `{transfer, path, chunkBytes}`.
async fn start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<StartReq>,
) -> Response {
    let nc = match gateway(
        &state,
        &headers,
        &body.account_id,
        body.instance_id.as_deref(),
    )
    .await
    {
        Ok(nc) => nc,
        Err(response) => return response,
    };
    let Some(name) = file_name(&body.name) else {
        return refused(StatusCode::BAD_REQUEST, "The file name cannot be used.");
    };
    if body.size == 0 || body.size > MAX_FILE_BYTES {
        return refused(
            StatusCode::PAYLOAD_TOO_LARGE,
            "The file is empty or larger than 4 GiB.",
        );
    }
    match begin(nc.as_ref(), &name).await {
        Ok((transfer, path)) => {
            Json(json!({"transfer": transfer, "path": path, "chunkBytes": CHUNK_BYTES}))
                .into_response()
        }
        Err(error) => cloud_error(&error),
    }
}

/// `PUT /api/nextcloud/upload/{transfer}/{n}?accountId&instanceId&path` with the chunk bytes.
async fn chunk(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((transfer, index)): Path<(String, u32)>,
    Query(query): Query<ChunkQuery>,
    bytes: Bytes,
) -> Response {
    if !transfer_id(&transfer) || !(1..=10_000).contains(&index) || !destination(&query.path) {
        return refused(StatusCode::BAD_REQUEST, "Invalid upload step.");
    }
    if bytes.is_empty() || bytes.len() > CHUNK_BYTES {
        return refused(StatusCode::PAYLOAD_TOO_LARGE, "Invalid chunk size.");
    }
    let nc = match gateway(
        &state,
        &headers,
        &query.account_id,
        query.instance_id.as_deref(),
    )
    .await
    {
        Ok(nc) => nc,
        Err(response) => return response,
    };
    let chunk = index.to_string();
    match nc
        .dav(DavOp {
            target: DavTarget::Upload {
                transfer: &transfer,
                chunk: Some(&chunk),
            },
            method: DavMethod::Put,
            body: Some(("application/octet-stream", &bytes)),
            depth: None,
            destination: Some(&query.path),
        })
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => cloud_error(&error),
    }
}

/// `POST /api/nextcloud/upload/{transfer}/finish {accountId, instanceId?, path, size}`.
async fn finish(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(transfer): Path<String>,
    Json(body): Json<FinishReq>,
) -> Response {
    if !transfer_id(&transfer) || !destination(&body.path) {
        return refused(StatusCode::BAD_REQUEST, "Invalid upload step.");
    }
    let nc = match gateway(
        &state,
        &headers,
        &body.account_id,
        body.instance_id.as_deref(),
    )
    .await
    {
        Ok(nc) => nc,
        Err(response) => return response,
    };
    match complete(nc.as_ref(), &transfer, &body.path, body.size).await {
        Ok(()) => {
            let name = body.path.rsplit('/').next().unwrap_or_default();
            Json(json!({"path": body.path, "name": name, "size": body.size})).into_response()
        }
        Err(error) => cloud_error(&error),
    }
}

/// `POST /api/nextcloud/upload/{transfer}/abort` drops the partial chunks.
async fn abort(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(transfer): Path<String>,
    Json(body): Json<AbortReq>,
) -> Response {
    if !transfer_id(&transfer) {
        return refused(StatusCode::BAD_REQUEST, "Invalid upload step.");
    }
    let nc = match gateway(
        &state,
        &headers,
        &body.account_id,
        body.instance_id.as_deref(),
    )
    .await
    {
        Ok(nc) => nc,
        Err(response) => return response,
    };
    match nc
        .dav(DavOp {
            target: DavTarget::Upload {
                transfer: &transfer,
                chunk: None,
            },
            method: DavMethod::Delete,
            body: None,
            depth: None,
            destination: None,
        })
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(NextcloudError::Api(reason)) if reason == "cloud item not found" => {
            StatusCode::NO_CONTENT.into_response()
        }
        Err(error) => cloud_error(&error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names_keep_one_segment_and_free_names_count_up() {
        assert_eq!(
            file_name("C:\\fakepath\\report.zip").as_deref(),
            Some("report.zip")
        );
        assert_eq!(file_name("../../etc/passwd").as_deref(), Some("passwd"));
        assert_eq!(file_name("a\u{0}b\r\n.bin").as_deref(), Some("ab.bin"));
        for bad in ["", "..", ".", "dir/", "   "] {
            assert_eq!(file_name(bad), None, "{bad:?}");
        }
        assert_eq!(file_name(&"x".repeat(201)), None);
        assert_eq!(candidate("report.zip", 1), "report.zip");
        assert_eq!(candidate("report.zip", 3), "report (3).zip");
        assert_eq!(candidate("README", 2), "README (2)");
        assert_eq!(candidate(".env", 2), ".env (2)");
    }

    #[test]
    fn upload_steps_accept_only_own_transfer_ids_and_folder_files() {
        assert!(transfer_id(&"a".repeat(32)));
        assert!(!transfer_id(&"A".repeat(32)));
        assert!(!transfer_id("../x"));
        assert!(destination("/Mail attachments/report.zip"));
        assert!(!destination("/Mail attachments/sub/report.zip"));
        assert!(!destination("/Mail attachments/../Documents/x"));
        assert!(!destination("/Documents/report.zip"));
        assert!(!destination("/Mail attachments/"));
    }
}
