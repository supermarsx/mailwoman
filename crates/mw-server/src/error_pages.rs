//! Error answers for requests no handler took (t32-h10).
//!
//! Two shapes, chosen by who is asking:
//!
//! * **A browser navigation** (see [`is_navigation`]) gets a small HTML page in the
//!   app's style: the name, the status, one plain sentence and a link back to the
//!   mail app (and to the admin console where that is where the person was). It
//!   carries the real status, never a `200`. The page has no script and no inline
//!   style, so it renders under the shell [`crate::CSP`] unchanged; its stylesheet
//!   is served by this server at [`STYLESHEET_PATH`], so a missing or broken web
//!   build cannot take the error page down with it.
//! * **Everything else** (`fetch()`, API clients, subresources, writes) gets
//!   `{"error": "...", "code": "..."}` as `application/json`, never HTML.
//!
//! Both are `Cache-Control: no-store`. Neither repeats anything from the request:
//! no path, no query, no header value.
//!
//! The text lives in one table, [`COPY`]. It is English only for now: server-side
//! translation is planned as `server.ftl` (t32-ext.md §15, L5), and this table is
//! the one place that will read from it.
//!
//! The module also holds two input checks that guard the same edges:
//! [`safe_return_path`] (a post-login or post-action return target taken from a
//! request) and [`is_unsafe_asset_path`] (a static-file path that could name a file
//! outside the web directory).

use std::panic::AssertUnwindSafe;
use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use futures_util::FutureExt;

/// The error page's stylesheet, served by this server at `{base}/__mw_error.css`.
pub(crate) const STYLESHEET_PATH: &str = "__mw_error.css";

/// The errors this module answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErrorKind {
    BadRequest,
    /// No route answers a navigation with 403 or 502 yet; the rows are here so a
    /// refusal or an upstream failure that reaches a person uses the same page.
    #[allow(dead_code)]
    Forbidden,
    NotFound,
    MethodNotAllowed,
    Internal,
    #[allow(dead_code)]
    BadGateway,
    Unavailable,
}

/// What a person or a client is told for one [`ErrorKind`].
struct Copy {
    status: StatusCode,
    /// Stable machine-readable code (the JSON `code`).
    code: &'static str,
    /// The page title and the JSON `error`.
    title: &'static str,
    /// The page's second sentence: what to do next.
    hint: &'static str,
}

/// All error text in one place (see the module docs for the translation plan).
const COPY: &[(ErrorKind, Copy)] = &[
    (
        ErrorKind::BadRequest,
        Copy {
            status: StatusCode::BAD_REQUEST,
            code: "bad_request",
            title: "This address is not valid.",
            hint: "Check the address and try again.",
        },
    ),
    (
        ErrorKind::Forbidden,
        Copy {
            status: StatusCode::FORBIDDEN,
            code: "forbidden",
            title: "You do not have access to this page.",
            hint: "If you think you should, ask your administrator.",
        },
    ),
    (
        ErrorKind::NotFound,
        Copy {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            title: "This page does not exist.",
            hint: "Check the address, or go back to your mail.",
        },
    ),
    (
        ErrorKind::MethodNotAllowed,
        Copy {
            status: StatusCode::METHOD_NOT_ALLOWED,
            code: "method_not_allowed",
            title: "This address cannot be opened this way.",
            hint: "Go back to your mail and try again from there.",
        },
    ),
    (
        ErrorKind::Internal,
        Copy {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            title: "Something went wrong on the server.",
            hint: "Try again in a moment. If it keeps happening, tell your administrator.",
        },
    ),
    (
        ErrorKind::BadGateway,
        Copy {
            status: StatusCode::BAD_GATEWAY,
            code: "bad_gateway",
            title: "The mail server did not answer.",
            hint: "Try again in a few minutes.",
        },
    ),
    (
        ErrorKind::Unavailable,
        Copy {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "unavailable",
            title: "Mailwoman cannot be reached right now.",
            hint: "Check your connection, or try again in a few minutes.",
        },
    ),
];

fn copy(kind: ErrorKind) -> &'static Copy {
    COPY.iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, c)| c)
        .expect("every ErrorKind has a COPY row")
}

/// Is this request a person's browser opening a page?
///
/// `GET`/`HEAD` only, then Fetch Metadata where it is present: a document (or
/// frame) destination, or `Sec-Fetch-Mode: navigate`, which is what a navigation a
/// service worker fetched again carries next to `Sec-Fetch-Dest: empty`. A client
/// with no Fetch Metadata counts only when it asks for `text/html` by name. Unlike
/// the SPA-shell decision, a file extension does not matter here: a person who
/// opens `/missing.png` in the address bar is still better served by a page.
pub(crate) fn is_navigation(method: &Method, headers: &HeaderMap) -> bool {
    if !matches!(*method, Method::GET | Method::HEAD) {
        return false;
    }
    let dest = crate::header_str(headers, "sec-fetch-dest");
    let mode = crate::header_str(headers, "sec-fetch-mode");
    if dest.is_some() || mode.is_some() {
        return matches!(
            dest,
            Some("document" | "iframe" | "frame" | "embed" | "object")
        ) || mode.is_some_and(|m| m.eq_ignore_ascii_case("navigate"));
    }
    crate::accepts_html(headers)
}

/// The error answer for this request: the page for a navigation, JSON otherwise.
/// `path` is the request path with the deploy prefix removed (and without the
/// leading `/`); it only decides whether the page also links the admin console.
pub(crate) fn respond(
    kind: ErrorKind,
    method: &Method,
    headers: &HeaderMap,
    base: &str,
    path: &str,
) -> Response {
    // A subresource-shaped path (`.woff2`, `.js`, ...) gets the page only when the
    // browser says outright that it is opening a document; a worker-fetched
    // `navigate` + `empty` request for one gets JSON, as `wants_spa_shell` decides.
    let explicit_document = matches!(
        crate::header_str(headers, "sec-fetch-dest"),
        Some("document" | "iframe" | "frame" | "embed" | "object")
    );
    let navigation = is_navigation(method, headers)
        && (explicit_document || !crate::has_subresource_extension(path));
    if navigation {
        page(kind, base, is_admin_path(path))
    } else {
        json(kind)
    }
}

fn is_admin_path(path: &str) -> bool {
    let path = path.trim_start_matches('/');
    path == "admin" || path.starts_with("admin/")
}

/// `{"error": "...", "code": "..."}` with the kind's status.
pub(crate) fn json(kind: ErrorKind) -> Response {
    let c = copy(kind);
    let body = serde_json::json!({ "error": c.title, "code": c.code });
    let mut resp = (c.status, axum::Json(body)).into_response();
    no_store(&mut resp);
    resp
}

/// The HTML error page. `base` is `""` or a `normalize_base_path`-validated prefix,
/// so it holds no quote, `<` or `&` and goes into the attributes as it is.
pub(crate) fn page(kind: ErrorKind, base: &str, admin_link: bool) -> Response {
    let c = copy(kind);
    let admin = if admin_link {
        format!(
            "\n      <a class=\"mw-error__link\" href=\"{base}/admin\">Go to the admin console</a>"
        )
    } else {
        String::new()
    };
    let html = format!(
        r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="dark light">
<meta name="robots" content="noindex">
<title>{title} · Mailwoman</title>
<link rel="icon" href="{base}/favicon.svg" type="image/svg+xml">
<link rel="stylesheet" href="{base}/{STYLESHEET_PATH}">
</head>
<body>
  <main class="mw-error" data-error-code="{code}">
    <p class="mw-error__brand">
      <svg class="mw-error__mark" viewBox="0 0 16 16" width="32" height="32" aria-hidden="true" focusable="false">
        <rect width="16" height="16" fill="#111111"/>
        <polygon points="2,4 8,9 14,4 14,12 2,12" fill="#4d7cff"/>
        <polygon points="3.56,4 12.44,4 8,7.7" fill="#f5c400"/>
      </svg>
      <span>Mailwoman</span>
    </p>
    <p class="mw-error__status">Error {status}</p>
    <h1 class="mw-error__title">{title}</h1>
    <p class="mw-error__hint">{hint}</p>
    <nav class="mw-error__links" aria-label="Where to go next">
      <a class="mw-error__link mw-error__link--primary" href="{base}/">Go to your mail</a>{admin}
    </nav>
  </main>
</body>
</html>
"##,
        title = c.title,
        hint = c.hint,
        code = c.code,
        status = c.status.as_u16(),
    );
    let mut resp = (c.status, html).into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    no_store(&mut resp);
    resp
}

fn no_store(resp: &mut Response) {
    resp.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
}

/// The error page's stylesheet. Colours are the Dark and Light packs' values from
/// `apps/web/src/theme/tokens.ts` (bg, text, textDim, borderStrong, accent, link,
/// focus); the Jost face is used when the system has it, otherwise the same
/// fallback stack as the app. Square corners, 2px rules, no shadow blur.
pub(crate) const STYLESHEET: &str = r#":root {
  color-scheme: dark light;
  --mw-e-bg: #111111;
  --mw-e-surface: #1a1a1a;
  --mw-e-text: #f2efe6;
  --mw-e-dim: #a39e94;
  --mw-e-rule: #bdb8ad;
  --mw-e-accent: #4d7cff;
  --mw-e-accent-text: #0b0b0b;
  --mw-e-link: #8fb0ff;
  --mw-e-focus: #f5c400;
  --mw-e-signal: #f5c400;
  --mw-e-shadow: #000000;
}
@media (prefers-color-scheme: light) {
  :root {
    --mw-e-bg: #f4f1ea;
    --mw-e-surface: #fbf9f4;
    --mw-e-text: #111111;
    --mw-e-dim: #4a4740;
    --mw-e-rule: #111111;
    --mw-e-accent: #1d3fbf;
    --mw-e-accent-text: #ffffff;
    --mw-e-link: #1d3fbf;
    --mw-e-focus: #111111;
    --mw-e-shadow: #111111;
  }
}
*, *::before, *::after { box-sizing: border-box; }
html, body { margin: 0; min-height: 100%; }
body {
  background: var(--mw-e-bg);
  color: var(--mw-e-text);
  font: 400 16px/1.4 "Jost", system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  padding: 16px;
}
.mw-error {
  width: 100%;
  max-width: 34rem;
  background: var(--mw-e-surface);
  border: 2px solid var(--mw-e-rule);
  border-top: 8px solid var(--mw-e-signal);
  box-shadow: 6px 6px 0 0 var(--mw-e-shadow);
  padding: 24px;
}
.mw-error__brand {
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 0 0 24px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  font-size: 14px;
}
.mw-error__mark { display: block; flex: none; }
.mw-error__status {
  margin: 0 0 8px;
  color: var(--mw-e-dim);
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  font-size: 13px;
}
.mw-error__title {
  margin: 0 0 12px;
  font-size: 28px;
  line-height: 1.2;
  font-weight: 800;
}
.mw-error__hint { margin: 0 0 24px; color: var(--mw-e-dim); }
.mw-error__links { display: flex; flex-wrap: wrap; gap: 12px; }
.mw-error__link {
  display: inline-flex;
  align-items: center;
  min-height: 44px;
  padding: 8px 16px;
  border: 2px solid var(--mw-e-rule);
  color: var(--mw-e-link);
  font-weight: 700;
  text-decoration: none;
}
.mw-error__link--primary {
  background: var(--mw-e-accent);
  border-color: var(--mw-e-accent);
  color: var(--mw-e-accent-text);
}
.mw-error__link:hover { text-decoration: underline; }
.mw-error__link:focus-visible { outline: 3px solid var(--mw-e-focus); outline-offset: 2px; }
@media (forced-colors: active) {
  .mw-error, .mw-error__link { border-color: CanvasText; box-shadow: none; }
  .mw-error__link--primary { background: ButtonFace; color: LinkText; }
}
"#;

/// Turn a handler panic into a plain 500 instead of a dropped connection. The
/// panic message is logged and never sent: it can carry a path, a value or a
/// library's internals.
pub(crate) async fn catch_panic(
    State(base): State<Arc<str>>,
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let headers = req.headers().clone();
    match AssertUnwindSafe(next.run(req)).catch_unwind().await {
        Ok(resp) => resp,
        Err(panic) => {
            let msg = panic
                .downcast_ref::<&str>()
                .map(|s| (*s).to_string())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "non-string panic".to_string());
            tracing::error!(panic = %msg, "request handler panicked");
            if is_navigation(&method, &headers) {
                page(ErrorKind::Internal, &base, false)
            } else {
                json(ErrorKind::Internal)
            }
        }
    }
}

/// A return target taken from a request (`relayState`, a `next`-style parameter),
/// reduced to a same-origin path under the deploy prefix, or `None`.
///
/// `raw` must be an app path: it starts with exactly one `/` and names a place in
/// this app relative to the prefix (`/` is the mail app, `/admin` the console).
/// Refused, raw or after any depth of percent-decoding:
///
/// * anything that is not printable ASCII: CR/LF (header splitting), NUL, and tab,
///   which a browser strips from a URL, so `/\t/evil.example` becomes
///   `//evil.example`;
/// * a second leading `/` or a `\` anywhere (`//host`, `/\host`, `/%2f/host`,
///   `/%5chost`): a browser reads both as the start of a host;
/// * a scheme (`https://other`, `javascript:`), which the leading-`/` rule already
///   excludes;
/// * a `.` or `..` path segment, which would climb out of the prefix;
/// * a decoding that is not UTF-8 (overlong forms such as `%c0%af`).
///
/// The returned value is `raw` with the prefix in front, unchanged otherwise.
pub(crate) fn safe_return_path(raw: &str, base: &str) -> Option<String> {
    if raw.is_empty() || raw.len() > 2048 {
        return None;
    }
    if !raw.bytes().all(|b| (0x21..=0x7e).contains(&b)) {
        return None;
    }
    let decoded = decode_fully(raw)?;
    for form in [raw, decoded.as_str()] {
        if !form.starts_with('/') || form.starts_with("//") {
            return None;
        }
        if form.chars().any(|c| c == '\\' || c.is_control()) {
            return None;
        }
        let path = form.split(['?', '#']).next().unwrap_or_default();
        if path.split('/').any(|seg| seg == "." || seg == "..") {
            return None;
        }
    }
    Some(format!("{base}{raw}"))
}

/// Could this static-file path name something other than a plain file inside the
/// web directory? `path` is the request path with the leading `/` removed, still
/// percent-encoded as it arrived.
///
/// Checked raw and fully decoded: a `.`/`..` segment, a `\` (a separator on
/// Windows), a `:` (a drive or an NTFS stream), a control character such as NUL, or
/// a decoding that is not UTF-8 (overlong forms). No file the web build ships has
/// any of these in its name.
pub(crate) fn is_unsafe_asset_path(path: &str) -> bool {
    let Some(decoded) = decode_fully(path) else {
        return true;
    };
    [path, decoded.as_str()].iter().any(|form| {
        form.chars()
            .any(|c| c == '\\' || c == ':' || c.is_control())
            || form.split('/').any(|seg| seg == "." || seg == "..")
    })
}

/// Percent-decode until nothing changes (at most four rounds, so `%252e` cannot
/// hide a `.` behind a second encoding). `None` when a round is not UTF-8 or the
/// encoding is still nested after the last round.
fn decode_fully(s: &str) -> Option<String> {
    let mut cur = s.to_string();
    for _ in 0..4 {
        let next = percent_decode(&cur)?;
        if next == cur {
            return Some(cur);
        }
        cur = next;
    }
    None
}

fn percent_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let (Some(h), Some(l)) = (hex(bytes[i + 1]), hex(bytes[i + 2]))
        {
            out.push(h << 4 | l);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).ok()
}

fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// `GET {base}/__mw_error.css`.
pub(crate) fn stylesheet_response(req: &HeaderMap) -> Response {
    crate::asset_response(
        STYLESHEET_PATH,
        "text/css; charset=utf-8",
        STYLESHEET.as_bytes().to_vec(),
        req,
    )
}

/// The page body, for tests that look at it without a server.
#[cfg(test)]
async fn page_body(kind: ErrorKind, base: &str, admin: bool) -> String {
    let body: axum::body::Body = page(kind, base, admin).into_body();
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_has_copy() {
        for kind in [
            ErrorKind::BadRequest,
            ErrorKind::Forbidden,
            ErrorKind::NotFound,
            ErrorKind::MethodNotAllowed,
            ErrorKind::Internal,
            ErrorKind::BadGateway,
            ErrorKind::Unavailable,
        ] {
            let c = copy(kind);
            assert!(c.status.is_client_error() || c.status.is_server_error());
            assert!(!c.title.is_empty() && !c.hint.is_empty());
        }
    }

    #[tokio::test]
    async fn the_page_has_no_script_and_no_inline_style() {
        let body = page_body(ErrorKind::NotFound, "/mail", true).await;
        assert!(!body.contains("<script"), "{body}");
        assert!(!body.contains("<style"), "{body}");
        assert!(!body.contains("style="), "{body}");
        for attr in [" onload=", " onerror=", " onclick=", "javascript:"] {
            assert!(!body.contains(attr), "{attr} in {body}");
        }
        assert!(body.contains(r#"href="/mail/""#));
        assert!(body.contains(r#"href="/mail/admin""#));
        assert!(body.contains(r#"href="/mail/__mw_error.css""#));
        // Nothing loaded from elsewhere.
        assert!(!body.contains("http://") && !body.contains("https://"));
    }

    #[tokio::test]
    async fn the_admin_link_appears_only_when_asked() {
        assert!(
            !page_body(ErrorKind::NotFound, "", false)
                .await
                .contains("/admin")
        );
        assert!(
            page_body(ErrorKind::NotFound, "", true)
                .await
                .contains(r#"href="/admin""#)
        );
    }

    #[test]
    fn navigation_detection() {
        let h = |pairs: &[(&'static str, &'static str)]| {
            let mut m = HeaderMap::new();
            for (k, v) in pairs {
                m.insert(*k, HeaderValue::from_static(v));
            }
            m
        };
        assert!(is_navigation(
            &Method::GET,
            &h(&[("sec-fetch-dest", "document")])
        ));
        assert!(is_navigation(
            &Method::GET,
            &h(&[("sec-fetch-dest", "empty"), ("sec-fetch-mode", "navigate")])
        ));
        assert!(!is_navigation(
            &Method::GET,
            &h(&[("sec-fetch-dest", "empty"), ("sec-fetch-mode", "cors")])
        ));
        assert!(!is_navigation(
            &Method::GET,
            &h(&[("sec-fetch-dest", "script"), ("accept", "text/html")])
        ));
        assert!(is_navigation(&Method::GET, &h(&[("accept", "text/html")])));
        assert!(!is_navigation(&Method::GET, &h(&[("accept", "*/*")])));
        assert!(!is_navigation(&Method::GET, &h(&[])));
        assert!(!is_navigation(
            &Method::POST,
            &h(&[("sec-fetch-dest", "document")])
        ));
        assert!(is_navigation(
            &Method::HEAD,
            &h(&[("sec-fetch-dest", "document")])
        ));
    }

    #[tokio::test]
    async fn a_panic_becomes_a_plain_500() {
        use axum::routing::get;
        async fn boom() -> &'static str {
            panic!("secret detail at /srv/mailwoman/src/x.rs:1");
        }
        let app = axum::Router::new().route("/boom", get(boom)).layer(
            axum::middleware::from_fn_with_state(Arc::<str>::from("/mail"), catch_panic),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        let c = reqwest::Client::new();
        let api = c.get(format!("http://{addr}/boom")).send().await.unwrap();
        assert_eq!(api.status(), 500);
        let body = api.text().await.unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&body).unwrap()["code"],
            "internal_error"
        );
        assert!(!body.contains("secret") && !body.contains(".rs"), "{body}");

        let page = c
            .get(format!("http://{addr}/boom"))
            .header("sec-fetch-dest", "document")
            .send()
            .await
            .unwrap();
        assert_eq!(page.status(), 500);
        let body = page.text().await.unwrap();
        assert!(
            body.contains("Something went wrong on the server."),
            "{body}"
        );
        assert!(body.contains(r#"href="/mail/""#), "{body}");
        assert!(!body.contains("secret"), "{body}");
    }

    #[test]
    fn return_paths() {
        let ok = [
            ("/", "", "/"),
            ("/admin", "", "/admin"),
            ("/admin#/users", "", "/admin#/users"),
            ("/#/calendar/week", "/mail", "/mail/#/calendar/week"),
            ("/inbox?x=1", "/mail", "/mail/inbox?x=1"),
            ("/a%20b", "", "/a%20b"),
            ("/?next=//x", "", "/?next=//x"),
        ];
        for (raw, base, want) in ok {
            assert_eq!(safe_return_path(raw, base).as_deref(), Some(want), "{raw}");
        }
        let refused = [
            "",
            "inbox",
            "//evil.example",
            "///evil.example",
            "/\\evil.example",
            "\\\\evil.example",
            "/%2fevil.example",
            "/%2F/evil.example",
            "%2f%2fevil.example",
            "/%5cevil.example",
            "/%5Cevil.example",
            "/%252f/evil.example",
            "https://evil.example",
            "http:/evil.example",
            "javascript:alert(1)",
            "/\t/evil.example",
            "/%09/evil.example",
            "/x\r\nSet-Cookie:a=b",
            "/x%0d%0aSet-Cookie:a=b",
            "/x%00",
            "/../other",
            "/a/../../other",
            "/%2e%2e/other",
            "/%c0%afevil",
            "/caf\u{e9}",
            "/ space",
        ];
        for raw in refused {
            assert_eq!(
                safe_return_path(raw, "/mail"),
                None,
                "{raw:?} must be refused"
            );
        }
        assert_eq!(
            safe_return_path(&format!("/{}", "a".repeat(3000)), ""),
            None
        );
    }

    #[test]
    fn asset_paths() {
        for ok in [
            "index.html",
            "assets/index-abc.js",
            "fonts/x.woff2",
            "a%20b.png",
        ] {
            assert!(!is_unsafe_asset_path(ok), "{ok}");
        }
        for bad in [
            "../secret",
            "a/../../secret",
            "..%2fsecret",
            "%2e%2e/secret",
            "%2e%2e%2fsecret",
            "..\\secret",
            "..%5csecret",
            "C:/Windows/win.ini",
            "index.html::$DATA",
            "index.html%00.png",
            "%c0%ae%c0%ae/secret",
            "%252e%252e/secret",
            "./index.html",
        ] {
            assert!(is_unsafe_asset_path(bad), "{bad} must be refused");
        }
    }
}
