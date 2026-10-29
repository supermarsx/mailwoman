//! Account-owned Nextcloud file operations. Every operation resolves a current
//! verified link and uses the guarded, bounded DAV/OCS transport.
//! Calendar/contact synchronization is a separate integration, not implied by these routes.
use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use crate::AppState;

// Large attachments uploaded in chunks and shared as a link (t39 L7). Kept beside
// this module so the router merge in lib.rs does not change.
#[path = "nextcloud_upload.rs"]
mod upload;

/// Legacy extension slot; request handlers resolve their own account-bound gateway.
pub(crate) type NextcloudHandle = Option<Arc<dyn NextcloudGateway>>;

/// e14 merges this into `router()` and layers on the injected [`NextcloudHandle`].
pub(crate) fn nextcloud_router() -> Router<AppState> {
    Router::new()
        .route("/api/nextcloud/attach", post(attach))
        .route("/api/nextcloud/save", post(save))
        .route("/api/nextcloud/share-link", post(share_link))
        .merge(upload::router())
}

/// Errors from a Nextcloud operation (kept coarse; no file content ever leaks).
#[derive(Debug)]
pub enum NextcloudError {
    Api(String),
}

impl std::fmt::Display for NextcloudError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NextcloudError::Api(m) => write!(f, "nextcloud rejected the request: {m}"),
        }
    }
}

impl std::error::Error for NextcloudError {}

/// A created public share link.
#[derive(Debug, Clone)]
pub struct ShareLink {
    pub url: String,
}

/// One entry in a WebDAV directory listing (the attach picker, plan §3 e14).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NextcloudEntry {
    /// The path relative to the user's files root (e.g. `/Documents/report.pdf`).
    pub path: String,
    /// The display name (last path segment).
    pub name: String,
    /// Whether this entry is a collection (folder).
    pub is_dir: bool,
    /// Size in bytes for files (0 for folders).
    pub size: u64,
}

/// Parameters for a public share-link creation.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareLinkReq {
    /// The file path within the Nextcloud user's files (e.g. `/Documents/big.zip`).
    pub path: String,
    /// Optional link password.
    #[serde(default)]
    pub password: Option<String>,
    /// Optional expiry (`YYYY-MM-DD`).
    #[serde(default, alias = "expiresAt")]
    pub expire_date: Option<String>,
    pub account_id: String,
    #[serde(default)]
    pub instance_id: Option<String>,
}

/// The Nextcloud file/share seam e14 backs with a linked account (OCS + WebDAV over
/// `reqwest`). A trait so the routes are testable without a live instance.
#[async_trait]
pub trait NextcloudGateway: Send + Sync {
    /// Fetch a file's bytes (attach-from-Nextcloud).
    async fn fetch(&self, path: &str) -> Result<Vec<u8>, NextcloudError>;
    /// Store bytes at a path (save-attachment-to-Nextcloud).
    async fn save(&self, path: &str, bytes: &[u8]) -> Result<(), NextcloudError>;
    /// Create a public share link (optionally password/expiry-protected).
    async fn create_share_link(&self, req: &ShareLinkReq) -> Result<ShareLink, NextcloudError>;
    /// List a WebDAV collection (attach picker, plan §3 e14). The default returns an
    /// empty listing; the account-bound gateway does a real PROPFIND.
    async fn list(&self, _path: &str) -> Result<Vec<NextcloudEntry>, NextcloudError> {
        Ok(Vec::new())
    }
    /// One DAV request inside the linked account's own files or upload area. The
    /// account-bound gateway builds every URL from the verified link, so a caller
    /// names only paths below that account.
    async fn dav(&self, _op: DavOp<'_>) -> Result<Vec<u8>, NextcloudError> {
        Err(NextcloudError::Api("operation not supported".into()))
    }
}

/// Where a [`DavOp`] goes: a path in the account's files, or its chunked-upload
/// area (`remote.php/dav/uploads/<user>/<transfer>/<chunk>`).
#[derive(Debug, Clone, Copy)]
pub enum DavTarget<'a> {
    File(&'a str),
    Upload {
        transfer: &'a str,
        chunk: Option<&'a str>,
    },
}

/// A DAV request for [`NextcloudGateway::dav`]. `destination` is a path in the
/// account's files (sent as an absolute same-origin `Destination`).
#[derive(Debug, Clone, Copy)]
pub struct DavOp<'a> {
    pub target: DavTarget<'a>,
    pub method: mw_egress::DavMethod,
    pub body: Option<(&'a str, &'a [u8])>,
    pub depth: Option<u8>,
    pub destination: Option<&'a str>,
}

// ── handlers ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttachReq {
    account_id: String,
    paths: Vec<String>,
    #[serde(default)]
    instance_id: Option<String>,
}

/// Materialize bounded cloud files into upload blobs owned by this mailbox session.
async fn attach(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AttachReq>,
) -> Response {
    let session =
        match crate::user_router::feature_session(&state, &headers, "integrations.nextcloud").await
        {
            Ok(session) => session,
            Err(response) => return response,
        };
    if body.account_id != session.account_id {
        return (
            StatusCode::CONFLICT,
            Json(json!({"error":"Account changed; reopen the file picker."})),
        )
            .into_response();
    }
    if body.paths.is_empty() || body.paths.len() > 10 {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let gateway = match crate::nextcloud_gateway::resolve(
        &state,
        &headers,
        body.instance_id.as_deref(),
    )
    .await
    {
        Ok(gateway) => gateway,
        Err(response) => return response,
    };
    let operation = async {
        let mut files = Vec::new();
        let mut total = 0usize;
        for path in &body.paths {
            let bytes = match gateway.fetch(path).await {
                Ok(bytes) => bytes,
                Err(error) => return nextcloud_error(&error),
            };
            total = total.saturating_add(bytes.len());
            if total > mw_egress::MAX_IMAGE_BYTES {
                return StatusCode::PAYLOAD_TOO_LARGE.into_response();
            }
            let name = path
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or("attachment")
                .to_owned();
            files.push((name, bytes));
        }
        let mut attachments = Vec::new();
        for (name, bytes) in files {
            let size = bytes.len();
            let content_type = "application/octet-stream";
            let blob_id = if let Some(engine) = &state.engine {
                match engine
                    .store_upload(&session.account_id, content_type, &bytes)
                    .await
                {
                    Ok(id) => id,
                    Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                }
            } else {
                let response = crate::proxy_upload(
                    &state.security,
                    &session,
                    &session.account_id,
                    content_type,
                    bytes.into(),
                )
                .await;
                if !response.status().is_success() {
                    return response;
                }
                let body = match axum::body::to_bytes(response.into_body(), 16384).await {
                    Ok(body) => body,
                    Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
                };
                let value: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(value) => value,
                    Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
                };
                if value["accountId"].as_str() != Some(&session.account_id) {
                    return StatusCode::BAD_GATEWAY.into_response();
                }
                match value["blobId"].as_str() {
                    Some(id) => id.to_owned(),
                    None => return StatusCode::BAD_GATEWAY.into_response(),
                }
            };
            attachments
                .push(json!({"name":name,"blobId":blob_id,"size":size,"contentType":content_type}));
        }
        Json(json!({"attachments":attachments})).into_response()
    };
    tokio::time::timeout(std::time::Duration::from_secs(30), operation)
        .await
        .unwrap_or_else(|_| StatusCode::GATEWAY_TIMEOUT.into_response())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveReq {
    account_id: String,
    path: String,
    email_id: String,
    blob_id: String,
    #[serde(default)]
    instance_id: Option<String>,
    #[serde(default)]
    acknowledge_warning: bool,
}

/// Export only an owned attachment that passes the current download policy.
async fn save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SaveReq>,
) -> Response {
    let session =
        match crate::user_router::feature_session(&state, &headers, "integrations.nextcloud").await
        {
            Ok(session) => session,
            Err(response) => return response,
        };
    if body.account_id != session.account_id {
        return (
            StatusCode::CONFLICT,
            Json(json!({"error":"active account changed"})),
        )
            .into_response();
    }
    let attachment = match crate::attachment_policy::read_owned_part(
        &state,
        &session,
        &body.email_id,
        &body.blob_id,
        mw_egress::MAX_IMAGE_BYTES,
        body.acknowledge_warning,
    )
    .await
    {
        Ok(Some(attachment)) => attachment,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(response) => return response,
    };
    let nc = match crate::nextcloud_gateway::resolve(&state, &headers, body.instance_id.as_deref())
        .await
    {
        Ok(gateway) => gateway,
        Err(response) => return response,
    };
    let name = attachment
        .filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("attachment");
    let path = format!("{}/{}", body.path.trim_end_matches('/'), name);
    match nc.save(&path,&attachment.bytes).await {
        Ok(())=>Json(json!({"entry":{"path":path,"name":name,"isDir":false,"size":attachment.bytes.len(),"contentType":attachment.content_type,"modified":null}})).into_response(),
        Err(error)=>nextcloud_error(&error),
    }
}

/// `POST /api/nextcloud/share-link {path, password?, expireDate?}` — create a public
/// share link for a (typically large) attachment.
async fn share_link(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ShareLinkReq>,
) -> Response {
    let session =
        match crate::user_router::feature_session(&state, &headers, "integrations.nextcloud").await
        {
            Ok(session) => session,
            Err(response) => return response,
        };
    if body.account_id != session.account_id {
        return StatusCode::CONFLICT.into_response();
    }
    let nc = match crate::nextcloud_gateway::resolve(&state, &headers, body.instance_id.as_deref())
        .await
    {
        Ok(gateway) => gateway,
        Err(response) => return response,
    };
    match nc.create_share_link(&body).await {
        Ok(link) => Json(json!({ "url": link.url,"expiresAt":body.expire_date,"passwordProtected":body.password.as_ref().is_some_and(|password|!password.is_empty()) })).into_response(),
        Err(e) => nextcloud_error(&e),
    }
}

fn nextcloud_error(e: &NextcloudError) -> Response {
    tracing::warn!("nextcloud operation failed");
    let msg = match e {
        NextcloudError::Api(_) => "nextcloud rejected the request",
    };
    (StatusCode::BAD_GATEWAY, Json(json!({ "error": msg }))).into_response()
}
