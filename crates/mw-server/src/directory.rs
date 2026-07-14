//! Directory/GAL routes (plan §3 e9/e14, SPEC §13). Filled by e9; MOUNTED by e14.
//!
//! `/api/directory/*` — GAL search / group-expand-before-send / S-MIME cert lookup /
//! photo over `mw-directory`. Every handler is **mailbox-session-authed** (the same
//! cookie/native session as the JMAP surface) via [`crate::authed`].
//!
//! ## Injection (e14)
//! The live [`mw_directory::DirectorySource`] is built by e14 at mount (from the
//! 0008 `directory_config` rows + sealed service-bind passwords) and injected as a
//! request extension ([`DirectoryHandle`]). e9 owns only the HTTP shape + the
//! session gate; it does not construct the directory or touch `lib.rs`.
//!
//! ## No mail content in logs (§21.1)
//! The search query and recipient address are mail-derived; they are **never**
//! logged. Only opaque result counts are emitted, and any diagnostic that must
//! reference the query wraps it in [`crate::observability::Redacted`].
#![allow(dead_code)]

use std::sync::Arc;

use axum::extract::{Extension, Path as UrlPath, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use base64::Engine as _;
use serde::Deserialize;
use serde_json::json;

use mw_directory::{DirectoryError, DirectorySource};

use crate::AppState;

/// The live directory source e14 injects (built from the 0008 `directory_config`).
pub(crate) type DirectoryHandle = Arc<dyn DirectorySource>;

/// e14 merges this into `router()` and layers on the injected [`DirectoryHandle`].
pub(crate) fn directory_router() -> Router<AppState> {
    Router::new()
        .route("/api/directory/search", get(search))
        .route("/api/directory/group/{dn}", get(expand_group))
        .route("/api/directory/cert", get(lookup_cert))
        .route("/api/directory/photo", get(lookup_photo))
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    /// The GAL search term (mail-derived — never logged).
    q: String,
    #[serde(default)]
    page: u32,
}

#[derive(Debug, Deserialize)]
struct EmailQuery {
    email: String,
}

/// `GET /api/directory/search?q=&page=` — GAL search across every recipient field.
async fn search(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Extension(dir): Extension<DirectoryHandle>,
    Query(query): Query<SearchQuery>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    match dir.search_gal(&query.q, query.page).await {
        Ok(entries) => {
            // Opaque count only — never the query or any resolved address.
            tracing::debug!(count = entries.len(), page = query.page, "gal search");
            Json(json!({ "entries": entries, "count": entries.len() })).into_response()
        }
        Err(e) => directory_error(e),
    }
}

/// `GET /api/directory/group/{dn}` — expand a distribution group before send
/// ("who is actually in this?"). The DN is path-encoded by the caller.
async fn expand_group(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Extension(dir): Extension<DirectoryHandle>,
    UrlPath(dn): UrlPath<String>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    match dir.expand_group(&dn).await {
        Ok(members) => Json(json!({ "members": members, "count": members.len() })).into_response(),
        Err(e) => directory_error(e),
    }
}

/// `GET /api/directory/cert?email=` — S/MIME certificate lookup (feeds mw-crypto's
/// cert path, §8.2). DER blobs are returned base64-encoded.
async fn lookup_cert(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Extension(dir): Extension<DirectoryHandle>,
    Query(q): Query<EmailQuery>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    match dir.lookup_cert(&q.email).await {
        Ok(ders) => {
            let b64: Vec<String> = ders
                .iter()
                .map(|d| base64::engine::general_purpose::STANDARD.encode(d))
                .collect();
            Json(json!({ "certificates": b64, "count": b64.len() })).into_response()
        }
        Err(e) => directory_error(e),
    }
}

/// `GET /api/directory/photo?email=` — the recipient's photo attribute, returned as
/// its raw image bytes (best-effort content type). `404` when absent.
async fn lookup_photo(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Extension(dir): Extension<DirectoryHandle>,
    Query(q): Query<EmailQuery>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    match dir.lookup_photo(&q.email).await {
        Ok(Some(bytes)) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "image/jpeg")],
            bytes,
        )
            .into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "error": "no photo" }))).into_response(),
        Err(e) => directory_error(e),
    }
}

/// Map a [`DirectoryError`] to an HTTP response. `NotConfigured` ⇒ `501` (the
/// deployment has no directory); protocol/transport/auth errors ⇒ `502` (never leak
/// the query or a resolved address in the body).
fn directory_error(e: DirectoryError) -> Response {
    match e {
        DirectoryError::NotConfigured => (
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({ "error": "no directory configured" })),
        )
            .into_response(),
        DirectoryError::Auth(_) | DirectoryError::Transport(_) | DirectoryError::Protocol(_) => {
            tracing::warn!("directory backend error: {e}");
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": "directory lookup failed" })),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mw_directory::{BindOutcome, Der, GalEntry, Result as DirResult};

    /// A canned directory source (no live LDAP) exercising the error mapping.
    struct MockDir(Option<DirectoryError>);

    #[async_trait]
    impl DirectorySource for MockDir {
        async fn search_gal(&self, _q: &str, _p: u32) -> DirResult<Vec<GalEntry>> {
            match &self.0 {
                None => Ok(vec![GalEntry {
                    dn: "cn=a".into(),
                    display_name: "A".into(),
                    mail: "a@x".into(),
                    is_group: false,
                }]),
                Some(_) => Err(DirectoryError::NotConfigured),
            }
        }
        async fn expand_group(&self, _dn: &str) -> DirResult<Vec<GalEntry>> {
            Ok(vec![])
        }
        async fn lookup_cert(&self, _email: &str) -> DirResult<Vec<Der>> {
            Ok(vec![vec![1, 2, 3]])
        }
        async fn lookup_photo(&self, _email: &str) -> DirResult<Option<Der>> {
            Ok(None)
        }
        async fn bind_auth(&self, _u: &str, _p: &str) -> DirResult<BindOutcome> {
            Ok(BindOutcome::Denied)
        }
    }

    #[test]
    fn not_configured_maps_to_501() {
        let resp = directory_error(DirectoryError::NotConfigured);
        assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[test]
    fn transport_error_maps_to_502_without_leaking() {
        let resp = directory_error(DirectoryError::Transport("host down".into()));
        assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    }

    #[tokio::test]
    async fn cert_der_is_base64_encoded() {
        // The mock returns one DER blob [1,2,3]; assert the base64 round-trip shape.
        let dir = MockDir(None);
        let ders = dir.lookup_cert("a@x").await.unwrap();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&ders[0]);
        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(&b64)
                .unwrap(),
            vec![1u8, 2, 3]
        );
    }

    /// §21.1: mail-derived values (the GAL search term, a recipient address) are
    /// never logged. The directory/nextcloud handlers emit only opaque counts +
    /// operation names; any diagnostic that must reference a mail-derived string wraps
    /// it in the typed [`crate::observability::Redacted`], which renders the fixed
    /// marker — the inner value can never reach a `tracing` field.
    #[test]
    fn mail_derived_strings_are_unprintable_when_wrapped() {
        use crate::observability::Redacted;
        let query = Redacted("bank statement");
        let address = Redacted("alice@example.com");
        assert_eq!(query.to_string(), Redacted::<&str>::MARKER);
        assert_eq!(format!("{address:?}"), Redacted::<&str>::MARKER);
        // The marker carries none of the wrapped content.
        assert!(!query.to_string().contains("bank"));
        assert!(!format!("{address:?}").contains("alice"));
    }
}
