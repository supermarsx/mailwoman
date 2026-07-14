//! Nextcloud routes (plan §3 e9/e14, SPEC §18.4). Filled by e9; MOUNTED by e14.
//!
//! `/api/nextcloud/*` — attach-from-Nextcloud (WebDAV GET), save-attachment-to-
//! Nextcloud (WebDAV PUT), and large-attachment public **share-link** creation (OCS
//! Sharing API, optional password/expiry). All routes are **mailbox-session-authed**.
//! CalDAV/CardDAV/tasks already work in core (`mw-dav`); this is only the file-share
//! surface.
//!
//! The server performs the OCS/WebDAV calls with the in-tree `reqwest`/rustls; the
//! browser never talks to the Nextcloud instance directly (same posture as the other
//! V7 proxies). No file content is logged (§21.1) — only the operation + status.
//!
//! ## Injection (e14)
//! The linked Nextcloud account (base URL + app-password auth) is built by e14 and
//! injected as an optional request extension ([`NextcloudHandle`]). When no Nextcloud
//! account is linked the handle is `None` and every route returns `501` (the web
//! hides the Nextcloud UI).
#![allow(dead_code)]

use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::{Extension, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use base64::Engine as _;
use serde::Deserialize;
use serde_json::json;

use crate::AppState;

/// The linked Nextcloud gateway e14 injects (`None` ⇒ no account linked ⇒ `501`).
pub(crate) type NextcloudHandle = Option<Arc<dyn NextcloudGateway>>;

/// e14 merges this into `router()` and layers on the injected [`NextcloudHandle`].
pub(crate) fn nextcloud_router() -> Router<AppState> {
    Router::new()
        .route("/api/nextcloud/attach", post(attach))
        .route("/api/nextcloud/save", post(save))
        .route("/api/nextcloud/share-link", post(share_link))
}

/// Errors from a Nextcloud operation (kept coarse; no file content ever leaks).
#[derive(Debug)]
pub enum NextcloudError {
    Transport(String),
    Api(String),
}

impl std::fmt::Display for NextcloudError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NextcloudError::Transport(m) => write!(f, "nextcloud transport error: {m}"),
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
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareLinkReq {
    /// The file path within the Nextcloud user's files (e.g. `/Documents/big.zip`).
    pub path: String,
    /// Optional link password.
    #[serde(default)]
    pub password: Option<String>,
    /// Optional expiry (`YYYY-MM-DD`).
    #[serde(default)]
    pub expire_date: Option<String>,
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
    /// empty listing; [`OcsNextcloud`] does a real PROPFIND.
    async fn list(&self, _path: &str) -> Result<Vec<NextcloudEntry>, NextcloudError> {
        Ok(Vec::new())
    }
}

// ── handlers ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttachReq {
    path: String,
}

/// `POST /api/nextcloud/attach {path}` — fetch a Nextcloud file to attach to a
/// compose. Returns the bytes (best-effort octet-stream).
async fn attach(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(nc): Extension<NextcloudHandle>,
    Json(body): Json<AttachReq>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    let Some(nc) = nc else {
        return not_linked();
    };
    match nc.fetch(&body.path).await {
        Ok(bytes) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/octet-stream")],
            bytes,
        )
            .into_response(),
        Err(e) => nextcloud_error(&e),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveReq {
    path: String,
    /// The attachment bytes, base64-encoded.
    content_base64: String,
}

/// `POST /api/nextcloud/save {path, contentBase64}` — save an attachment to Nextcloud.
async fn save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(nc): Extension<NextcloudHandle>,
    Json(body): Json<SaveReq>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    let Some(nc) = nc else {
        return not_linked();
    };
    let Ok(bytes) =
        base64::engine::general_purpose::STANDARD.decode(body.content_base64.as_bytes())
    else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid base64 content" })),
        )
            .into_response();
    };
    match nc.save(&body.path, &bytes).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => nextcloud_error(&e),
    }
}

/// `POST /api/nextcloud/share-link {path, password?, expireDate?}` — create a public
/// share link for a (typically large) attachment.
async fn share_link(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(nc): Extension<NextcloudHandle>,
    Json(body): Json<ShareLinkReq>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    let Some(nc) = nc else {
        return not_linked();
    };
    match nc.create_share_link(&body).await {
        Ok(link) => Json(json!({ "url": link.url })).into_response(),
        Err(e) => nextcloud_error(&e),
    }
}

fn not_linked() -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({ "error": "no nextcloud account linked" })),
    )
        .into_response()
}

fn nextcloud_error(e: &NextcloudError) -> Response {
    tracing::warn!("nextcloud operation failed");
    let msg = match e {
        NextcloudError::Transport(_) => "nextcloud unreachable",
        NextcloudError::Api(_) => "nextcloud rejected the request",
    };
    (StatusCode::BAD_GATEWAY, Json(json!({ "error": msg }))).into_response()
}

// ── OCS/WebDAV reqwest gateway (e14 constructs from the linked account) ──────────

/// A linked-account Nextcloud gateway over OCS + WebDAV (`reqwest`/rustls). Pure URL
/// construction is factored into [`webdav_url`] / [`ocs_shares_url`] so it is unit
/// tested without a live instance.
pub struct OcsNextcloud {
    client: reqwest::Client,
    /// Instance base, no trailing slash (e.g. `https://cloud.example.org`).
    base_url: String,
    username: String,
    app_password: String,
}

impl OcsNextcloud {
    #[must_use]
    pub fn new(
        client: reqwest::Client,
        base_url: impl Into<String>,
        username: impl Into<String>,
        app_password: impl Into<String>,
    ) -> Self {
        Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            username: username.into(),
            app_password: app_password.into(),
        }
    }
}

#[async_trait]
impl NextcloudGateway for OcsNextcloud {
    async fn fetch(&self, path: &str) -> Result<Vec<u8>, NextcloudError> {
        let url = webdav_url(&self.base_url, &self.username, path);
        let resp = self
            .client
            .get(url)
            .basic_auth(&self.username, Some(&self.app_password))
            .send()
            .await
            .map_err(|e| NextcloudError::Transport(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(NextcloudError::Api(resp.status().to_string()));
        }
        Ok(resp
            .bytes()
            .await
            .map_err(|e| NextcloudError::Transport(e.to_string()))?
            .to_vec())
    }

    async fn save(&self, path: &str, bytes: &[u8]) -> Result<(), NextcloudError> {
        let url = webdav_url(&self.base_url, &self.username, path);
        let resp = self
            .client
            .put(url)
            .basic_auth(&self.username, Some(&self.app_password))
            .body(bytes.to_vec())
            .send()
            .await
            .map_err(|e| NextcloudError::Transport(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(NextcloudError::Api(resp.status().to_string()));
        }
        Ok(())
    }

    async fn create_share_link(&self, req: &ShareLinkReq) -> Result<ShareLink, NextcloudError> {
        let url = ocs_shares_url(&self.base_url);
        // shareType 3 = public link.
        let mut form = vec![("path", req.path.clone()), ("shareType", "3".to_string())];
        if let Some(pw) = &req.password {
            form.push(("password", pw.clone()));
        }
        if let Some(exp) = &req.expire_date {
            form.push(("expireDate", exp.clone()));
        }
        let resp = self
            .client
            .post(url)
            .basic_auth(&self.username, Some(&self.app_password))
            .header("OCS-APIRequest", "true")
            .header(header::ACCEPT, "application/json")
            .form(&form)
            .send()
            .await
            .map_err(|e| NextcloudError::Transport(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(NextcloudError::Api(resp.status().to_string()));
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| NextcloudError::Transport(e.to_string()))?;
        parse_share_url(&body)
            .map(|url| ShareLink { url })
            .ok_or_else(|| NextcloudError::Api("no share url in response".into()))
    }

    async fn list(&self, path: &str) -> Result<Vec<NextcloudEntry>, NextcloudError> {
        let url = webdav_url(&self.base_url, &self.username, path);
        let resp = self
            .client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), url)
            .basic_auth(&self.username, Some(&self.app_password))
            .header("Depth", "1")
            .header(header::CONTENT_TYPE, "application/xml")
            .body(PROPFIND_BODY)
            .send()
            .await
            .map_err(|e| NextcloudError::Transport(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(NextcloudError::Api(resp.status().to_string()));
        }
        let xml = resp
            .text()
            .await
            .map_err(|e| NextcloudError::Transport(e.to_string()))?;
        Ok(parse_propfind(&xml, &self.username))
    }
}

const PROPFIND_BODY: &str = r#"<?xml version="1.0"?><d:propfind xmlns:d="DAV:"><d:prop><d:resourcetype/><d:getcontentlength/></d:prop></d:propfind>"#;

/// Extract entries from a WebDAV multistatus PROPFIND body (best-effort; namespace-
/// prefix + case tolerant). The first `<response>` is the queried collection itself
/// and is dropped. The picker's live fidelity is exercised by e16.
fn parse_propfind(xml: &str, user: &str) -> Vec<NextcloudEntry> {
    let prefix = format!("/remote.php/dav/files/{user}");
    let lower = xml.to_lowercase();
    let mut out = Vec::new();
    let mut search = 0usize;
    while let Some(rel) = lower[search..].find(":response") {
        let start = search + rel;
        let end = lower[start..]
            .find("</")
            .map(|e| {
                start
                    + e
                    + lower[start + e..]
                        .find(":response>")
                        .map(|x| x + ":response>".len())
                        .unwrap_or(0)
            })
            .unwrap_or(lower.len());
        let block = &xml[start..end.min(xml.len())];
        let block_lower = &lower[start..end.min(lower.len())];
        search = end.max(start + 1);

        let Some(href) = extract_tag(block, ":href>") else {
            continue;
        };
        let decoded = percent_decode(href.trim());
        let Some(rest) = decoded.strip_prefix(&prefix) else {
            continue;
        };
        let clean = rest.trim_end_matches('/');
        if clean.is_empty() {
            continue; // the collection itself
        }
        let is_dir = block_lower.contains(":collection");
        let size = extract_tag(block, ":getcontentlength>")
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let name = clean.rsplit('/').next().unwrap_or(clean).to_string();
        out.push(NextcloudEntry {
            path: rest.to_string(),
            name,
            is_dir,
            size,
        });
    }
    out
}

/// Extract the text between the first `…{tag}` and its closing `</…>` (namespace
/// tolerant: `tag` is the suffix like `:href>`).
fn extract_tag(block: &str, tag: &str) -> Option<String> {
    let lower = block.to_lowercase();
    let open = lower.find(tag)? + tag.len();
    let close = lower[open..].find("</")? + open;
    Some(block[open..close].to_string())
}

/// Minimal percent-decoding for WebDAV hrefs.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16)
        {
            out.push(b);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The WebDAV URL for a user's file: `{base}/remote.php/dav/files/{user}/{path}`.
fn webdav_url(base: &str, user: &str, path: &str) -> String {
    let p = path.trim_start_matches('/');
    format!("{base}/remote.php/dav/files/{user}/{p}")
}

/// The OCS Sharing API endpoint for creating shares.
fn ocs_shares_url(base: &str) -> String {
    format!("{base}/ocs/v2.php/apps/files_sharing/api/v1/shares")
}

/// Extract `ocs.data.url` from an OCS JSON share-create response.
fn parse_share_url(body: &serde_json::Value) -> Option<String> {
    body.get("ocs")?
        .get("data")?
        .get("url")?
        .as_str()
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webdav_url_is_well_formed() {
        assert_eq!(
            webdav_url("https://cloud.example.org", "alice", "/Docs/big.zip"),
            "https://cloud.example.org/remote.php/dav/files/alice/Docs/big.zip"
        );
        // A path without a leading slash is handled identically.
        assert_eq!(
            webdav_url("https://c.io", "bob", "a.txt"),
            "https://c.io/remote.php/dav/files/bob/a.txt"
        );
    }

    #[test]
    fn ocs_shares_url_is_the_sharing_api() {
        assert_eq!(
            ocs_shares_url("https://c.io"),
            "https://c.io/ocs/v2.php/apps/files_sharing/api/v1/shares"
        );
    }

    #[test]
    fn parses_share_url_from_ocs_json() {
        let body = json!({ "ocs": { "data": { "url": "https://c.io/s/AbCdEf" } } });
        assert_eq!(
            parse_share_url(&body).as_deref(),
            Some("https://c.io/s/AbCdEf")
        );
        assert!(parse_share_url(&json!({ "ocs": {} })).is_none());
    }

    #[test]
    fn unlinked_handle_returns_501() {
        assert_eq!(not_linked().status(), StatusCode::NOT_IMPLEMENTED);
    }
}
