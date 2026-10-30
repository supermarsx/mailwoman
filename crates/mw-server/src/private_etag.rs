//! Strong `ETag` + `If-None-Match`/304 for the app's private JSON reads (t99).
//!
//! The SPA reads the same small JSON resources again and again (`/jmap/session`,
//! `/api/settings`, `/api/me`, `/api/account/*`, …). Before this they carried no
//! validator, so every read sent the whole body. Now a `GET` of one of
//! [`TAGGED_PATHS`] that answers `200` with a JSON body and no caching posture
//! of its own gets:
//!
//! * `ETag`: a strong tag over the exact bytes sent (same scheme as the static
//!   assets: SHA-256, first 16 bytes, base64url);
//! * `Cache-Control: private, no-cache`: a browser may keep it but must ask the
//!   server each time; a shared cache (proxy, CDN) must not keep it at all;
//! * `Vary: Cookie, X-Mailwoman-Link`: the answer depends on who asks and for
//!   which mailbox.
//!
//! A request whose `If-None-Match` names that tag gets `304` with no body. The
//! handler still runs (the tag is over what it would send), so a 304 can never
//! be stale: it only saves the transfer and the client's parse.
//!
//! Left alone: any response that already says how it may be cached (every
//! `no-store` answer keeps its `no-store`, including t63's analytics surfaces),
//! one that sets a cookie, one with a validator of its own, a non-JSON body
//! (assets have their own ETags; streams are not buffered) and a body that is
//! not of known size or is larger than [`MAX_TAGGED_BODY`].

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use http_body_util::BodyExt;

/// Larger JSON bodies are sent untagged rather than buffered.
pub(crate) const MAX_TAGGED_BODY: u64 = 1024 * 1024;

const VARY: &str = "Cookie, X-Mailwoman-Link";

/// The mail app's JSON reads that get a validator: the ones the t99
/// measurements found read again and again, none of which takes `If-Match` on
/// a write (the console's optimistic-concurrency tags come from its own
/// handlers and are never replaced here). Matched exactly: the router sees the
/// path with the deploy prefix already removed.
const TAGGED_PATHS: &[&str] = &[
    "/jmap/session",
    "/api/me",
    "/api/settings",
    "/api/account/appearance",
    "/api/account/identities",
    "/api/account/signatures",
    "/api/account/saved-searches",
    "/api/account/notifications",
    "/api/account/nextcloud",
    "/api/assist/config",
    "/api/ui-plugins",
    "/api/remote-images/grants",
    "/api/rc/policy",
];

fn tagged_path(path: &str) -> bool {
    TAGGED_PATHS.contains(&path)
}

fn strong_etag(bytes: &[u8]) -> String {
    use base64::Engine;
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    format!(
        "\"{}\"",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&digest[..16])
    )
}

/// Does `If-None-Match` list `etag` (or `*`)? A weak form of the tag matches too.
fn none_match(raw: Option<&HeaderValue>, etag: &str) -> bool {
    let Some(raw) = raw.and_then(|v| v.to_str().ok()) else {
        return false;
    };
    raw.split(',').any(|candidate| {
        let candidate = candidate.trim();
        let candidate = candidate.strip_prefix("W/").unwrap_or(candidate);
        candidate == "*" || candidate == etag
    })
}

fn is_json(resp: &Response) -> bool {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            let v = v.trim_start().to_ascii_lowercase();
            v.starts_with("application/json") || v.starts_with("application/problem+json")
        })
}

/// The layer (`axum::middleware::from_fn(private_etag::layer)`).
pub(crate) async fn layer(req: Request, next: Next) -> Response {
    let candidate = req.method() == Method::GET && tagged_path(req.uri().path());
    let if_none_match = if candidate {
        req.headers().get(header::IF_NONE_MATCH).cloned()
    } else {
        None
    };
    let resp = next.run(req).await;
    if !candidate {
        return resp;
    }
    tag(resp, if_none_match.as_ref()).await
}

async fn tag(resp: Response, if_none_match: Option<&HeaderValue>) -> Response {
    use axum::body::HttpBody as _;
    let h = resp.headers();
    if resp.status() != StatusCode::OK
        || h.contains_key(header::CACHE_CONTROL)
        || h.contains_key(header::ETAG)
        || h.contains_key(header::SET_COOKIE)
        || !is_json(&resp)
    {
        return resp;
    }
    match resp.body().size_hint().exact() {
        Some(n) if n <= MAX_TAGGED_BODY => {}
        _ => return resp,
    }
    let (mut parts, body) = resp.into_parts();
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        // A body of known size that fails to read is the handler's failure;
        // it was going to fail on the wire the same way.
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let etag = strong_etag(&bytes);
    let Ok(etag_value) = HeaderValue::from_str(&etag) else {
        return Response::from_parts(parts, Body::from(bytes));
    };
    parts.headers.insert(header::ETAG, etag_value.clone());
    parts.headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-cache"),
    );
    parts
        .headers
        .append(header::VARY, HeaderValue::from_static(VARY));
    if none_match(if_none_match, &etag) {
        let mut not_modified = StatusCode::NOT_MODIFIED.into_response();
        let out = not_modified.headers_mut();
        out.insert(header::ETAG, etag_value);
        out.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, no-cache"),
        );
        out.append(header::VARY, HeaderValue::from_static(VARY));
        // Keep the security headers the inner layers set (nosniff, CSP, …).
        for name in [
            header::X_CONTENT_TYPE_OPTIONS,
            header::CONTENT_SECURITY_POLICY,
            header::REFERRER_POLICY,
        ] {
            if let Some(v) = parts.headers.get(&name) {
                out.insert(name, v.clone());
            }
        }
        return not_modified;
    }
    Response::from_parts(parts, Body::from(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;
    use axum::{Json, Router};

    fn app() -> Router {
        Router::new()
            .route(
                "/api/settings",
                get(|| async { Json(serde_json::json!({"a": 1})) })
                    .post(|| async { Json(serde_json::json!({"w": 1})) }),
            )
            .route(
                "/api/thing",
                get(|| async { Json(serde_json::json!({"a": 1})) }),
            )
            .route(
                "/api/me",
                get(|| async {
                    (
                        [(header::CACHE_CONTROL, "private, no-store")],
                        Json(serde_json::json!({"s": 1})),
                    )
                }),
            )
            .route(
                "/api/ui-plugins",
                get(|| async {
                    (
                        [(header::SET_COOKIE, "mw_session=x; HttpOnly")],
                        Json(serde_json::json!({"ok": true})),
                    )
                }),
            )
            .route("/api/rc/policy", get(|| async { "plain" }))
            .route(
                "/jmap/session",
                get(|| async { Json(serde_json::json!({"state": "s1"})) }),
            )
            .route(
                "/other",
                get(|| async { Json(serde_json::json!({"x": 1})) }),
            )
            .route(
                "/api/account/appearance",
                get(|| async { (StatusCode::NOT_FOUND, Json(serde_json::json!({"e": 1}))) }),
            )
            .layer(axum::middleware::from_fn(layer))
    }

    struct Answer {
        status: StatusCode,
        headers: axum::http::HeaderMap,
        body: Vec<u8>,
    }

    async fn serve() -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app()).await.unwrap() });
        format!("http://{addr}")
    }

    async fn request(method: reqwest::Method, path: &str, inm: Option<&str>) -> Answer {
        let base = serve().await;
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let mut req = client.request(method, format!("{base}{path}"));
        if let Some(v) = inm {
            req = req.header("if-none-match", v);
        }
        let r = req.send().await.unwrap();
        let status = StatusCode::from_u16(r.status().as_u16()).unwrap();
        let mut headers = axum::http::HeaderMap::new();
        for (k, v) in r.headers() {
            headers.append(
                axum::http::HeaderName::from_bytes(k.as_str().as_bytes()).unwrap(),
                HeaderValue::from_bytes(v.as_bytes()).unwrap(),
            );
        }
        let body = r.bytes().await.unwrap().to_vec();
        Answer {
            status,
            headers,
            body,
        }
    }

    async fn send(path: &str, inm: Option<&str>) -> Answer {
        request(reqwest::Method::GET, path, inm).await
    }

    #[tokio::test]
    async fn a_json_read_gets_a_strong_private_tag_and_304s_on_a_match() {
        let first = send("/api/settings", None).await;
        assert_eq!(first.status, StatusCode::OK);
        let etag = first.headers[header::ETAG].to_str().unwrap().to_owned();
        assert!(
            etag.starts_with('"') && !etag.starts_with("W/"),
            "strong tag: {etag}"
        );
        assert_eq!(first.headers[header::CACHE_CONTROL], "private, no-cache");
        assert_eq!(first.headers[header::VARY], VARY);
        assert_eq!(&first.body[..], br#"{"a":1}"#);

        let again = send("/api/settings", Some(&etag)).await;
        assert_eq!(again.status, StatusCode::NOT_MODIFIED);
        assert_eq!(again.headers[header::ETAG], etag.as_str());
        assert_eq!(again.headers[header::CACHE_CONTROL], "private, no-cache");
        assert!(again.body.is_empty());

        // A weak form, and a list, match too; another tag does not.
        assert_eq!(
            send("/api/settings", Some(&format!("\"x\", W/{etag}")))
                .await
                .status,
            StatusCode::NOT_MODIFIED
        );
        assert_eq!(
            send("/api/settings", Some("\"other\"")).await.status,
            StatusCode::OK
        );
        // A JSON read that is not on the list is left alone.
        let other = send("/api/thing", Some("*")).await;
        assert_eq!(other.status, StatusCode::OK);
        assert!(!other.headers.contains_key(header::ETAG));
    }

    #[tokio::test]
    async fn the_jmap_session_is_tagged() {
        let r = send("/jmap/session", None).await;
        assert!(r.headers.contains_key(header::ETAG));
    }

    #[tokio::test]
    async fn no_store_answers_keep_no_store_and_get_no_tag() {
        let r = send("/api/me", Some("*")).await;
        assert_eq!(r.status, StatusCode::OK);
        assert_eq!(r.headers[header::CACHE_CONTROL], "private, no-store");
        assert!(!r.headers.contains_key(header::ETAG));
    }

    #[tokio::test]
    async fn cookies_non_json_errors_and_other_paths_are_untouched() {
        for path in [
            "/api/ui-plugins",
            "/api/rc/policy",
            "/other",
            "/api/account/appearance",
            "/api/thing",
        ] {
            let r = send(path, Some("*")).await;
            assert_ne!(r.status, StatusCode::NOT_MODIFIED, "{path}");
            assert!(!r.headers.contains_key(header::ETAG), "{path}");
            assert!(!r.headers.contains_key(header::CACHE_CONTROL), "{path}");
        }
    }

    #[tokio::test]
    async fn a_write_is_never_answered_304() {
        let r = request(reqwest::Method::POST, "/api/settings", Some("*")).await;
        assert_eq!(r.status, StatusCode::OK);
        assert!(!r.headers.contains_key(header::ETAG));
    }
}
