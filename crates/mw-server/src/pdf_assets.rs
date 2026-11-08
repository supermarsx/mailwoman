//! The PDF.js viewer's static library, embedded in the binary.
//!
//! The viewer document (`pdfjs/web/mailwoman.html`) has always been embedded, but
//! the files it loads (`bootstrap.mjs`, `mailwoman.css`, `web/viewer.css`, the
//! library, worker, cmaps, fonts, wasm...) used to come from `MW_WEB_DIR`. A stale
//! or incomplete deployed web folder then answered them with the JSON 404 body, and
//! the browser refused it under `nosniff` (`application/json` is not a script or a
//! stylesheet). Everything the viewer needs is now compiled in, so the document and
//! its assets can never be of different versions.
//!
//! `MW_WEB_DIR` may still supply a library file, but only when its
//! `pdfjs/manifest.json` is byte-identical to the embedded one (the version stamp:
//! upstream version plus every pinned file hash) and the file exists. Our own
//! files (`bootstrap.mjs`, `mailwoman.css`, the entry document) and the worker are
//! always the embedded copy: they are coupled to the embedded document's CSP.
//!
//! Access: these are public, immutable library files that contain no mail. They sit
//! behind the same edge layers as the app shell (source bans, client-access gate,
//! rate limits). The PDF bytes never come from here: the parent fetches them
//! through the authenticated attachment route and hands them to the frame.
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::SystemTime,
};

use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
};

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../apps/web/public"]
#[include = "pdfjs/**"]
#[include = "pdf.worker.mjs"]
struct PdfjsAssets;

const WORKER: &str = "pdf.worker.mjs";
const MANIFEST: &str = "pdfjs/manifest.json";

/// Is this (prefix-relative) path one this module owns?
pub(crate) fn owns(path: &str) -> bool {
    path == WORKER || path.starts_with("pdfjs/")
}

/// The Content-Type each embedded file is served with. Explicit, never guessed:
/// under `nosniff` a script or stylesheet with another type is refused outright.
pub(crate) fn mime_for(path: &str) -> &'static str {
    let name = path.rsplit('/').next().unwrap_or(path);
    let Some((_, ext)) = name.rsplit_once('.') else {
        // LICENSE, LICENSE_FOXIT...
        return if name.starts_with("LICENSE") {
            "text/plain; charset=utf-8"
        } else {
            "application/octet-stream"
        };
    };
    match ext.to_ascii_lowercase().as_str() {
        "mjs" | "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        "json" => "application/json",
        "wasm" => "application/wasm",
        "svg" => "image/svg+xml",
        "gif" => "image/gif",
        "png" => "image/png",
        "ttf" => "font/ttf",
        "ftl" => "text/plain; charset=utf-8",
        // Binary CMaps, Type 1 fonts and ICC profiles are fetched as bytes.
        _ => "application/octet-stream",
    }
}

fn manifest_files(bytes: &[u8]) -> Option<Vec<String>> {
    let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    Some(value.get("files")?.as_object()?.keys().cloned().collect())
}

/// Per-file override from `MW_WEB_DIR`, valid only for pinned upstream files and
/// only when the deployed manifest equals the embedded one.
fn deployed(dir: &Path, path: &str, pinned: &[String]) -> Option<Vec<u8>> {
    let rel = path.strip_prefix("pdfjs/")?;
    if !pinned.iter().any(|f| f == rel) || !manifest_matches(dir) {
        return None;
    }
    std::fs::read(dir.join(path)).ok()
}

type StampCache = Mutex<HashMap<PathBuf, (SystemTime, u64, bool)>>;

/// Is the deployed `pdfjs/manifest.json` identical to the embedded one? Cached by
/// (mtime, length) so a viewer load does not re-read it for every asset.
fn manifest_matches(dir: &Path) -> bool {
    static CACHE: OnceLock<StampCache> = OnceLock::new();
    let file = dir.join(MANIFEST);
    let Ok(meta) = std::fs::metadata(&file) else {
        return false;
    };
    let stamp = (meta.modified().unwrap_or(SystemTime::UNIX_EPOCH), meta.len());
    let cache = CACHE.get_or_init(Default::default);
    if let Ok(map) = cache.lock()
        && let Some((m, l, ok)) = map.get(&file)
        && (*m, *l) == stamp
    {
        return *ok;
    }
    let ok = match (std::fs::read(&file), PdfjsAssets::get(MANIFEST)) {
        (Ok(disk), Some(embedded)) => disk == embedded.data.as_ref(),
        _ => false,
    };
    if let Ok(mut map) = cache.lock() {
        map.insert(file, (stamp.0, stamp.1, ok));
    }
    ok
}

fn miss(path: &str) -> Response {
    tracing::warn!(
        asset = path,
        "PDF viewer asset is not in the embedded set; this build is missing a file the viewer references"
    );
    let mut resp = (
        StatusCode::NOT_FOUND,
        format!(
            "PDF viewer asset not found: /{path}\nThis server build does not include that PDF.js file. Rebuild or redeploy the server image.\n"
        ),
    )
        .into_response();
    let h = resp.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    resp
}

/// Answer one `/pdfjs/*` (or `/pdf.worker.mjs`) request. Never a JSON body.
pub(crate) fn response(
    web_dir: Option<&Path>,
    method: &Method,
    path: &str,
    headers: &HeaderMap,
) -> Response {
    if !matches!(*method, Method::GET | Method::HEAD) {
        return (
            StatusCode::METHOD_NOT_ALLOWED,
            [
                (header::ALLOW, "GET, HEAD"),
                (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            ],
            "Method not allowed\n",
        )
            .into_response();
    }
    let embedded = PdfjsAssets::get(path);
    let bytes = match (&embedded, web_dir) {
        (Some(_), Some(dir)) => {
            let pinned = PdfjsAssets::get(MANIFEST)
                .and_then(|m| manifest_files(&m.data))
                .unwrap_or_default();
            deployed(dir, path, &pinned)
        }
        _ => None,
    };
    let bytes = match (bytes, embedded) {
        (Some(b), _) => b,
        (None, Some(e)) => e.data.into_owned(),
        (None, None) => return miss(path),
    };
    let len = bytes.len();
    let mut resp = super::asset_response(path, mime_for(path), bytes, headers);
    // The URLs carry no version, so a shared long lifetime could pair a new document
    // with an old library after an upgrade. Always revalidate; the ETag makes it a 304.
    resp.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    // The viewer document is sandboxed without `allow-same-origin`, so everything it
    // loads is a cross-origin request from the opaque origin `null`: module scripts,
    // the worker's import, fonts, wasm and `fetch` need CORS, and the stylesheet
    // (a no-cors request) needs a CORP that admits it. These are public library
    // files with no mail and no cookies, so a wildcard grants nothing.
    {
        let h = resp.headers_mut();
        h.insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static("*"),
        );
        h.insert(
            "cross-origin-resource-policy",
            HeaderValue::from_static("cross-origin"),
        );
    }
    if *method == Method::HEAD {
        if resp.status() == StatusCode::OK {
            resp.headers_mut()
                .insert(header::CONTENT_LENGTH, HeaderValue::from(len));
        }
        *resp.body_mut() = Body::empty();
    }
    resp
}

/// The outcome of [`audit`].
pub(crate) struct Audit {
    /// `pass`, `warn` or `fail` (Diagnostics vocabulary).
    pub(crate) status: &'static str,
    pub(crate) summary: String,
}

/// Every file the viewer references: those named by the entry document and the
/// bootstrap, plus every pinned upstream file.
fn required() -> Vec<String> {
    let mut out: Vec<String> = vec![
        "pdfjs/web/mailwoman.html".into(),
        "pdfjs/bootstrap.mjs".into(),
        "pdfjs/polyfills.mjs".into(),
        "pdfjs/mailwoman.css".into(),
        "pdfjs/build/pdf.mjs".into(),
        "pdfjs/web/viewer.mjs".into(),
        MANIFEST.into(),
        WORKER.into(),
    ];
    if let Some(html) = PdfjsAssets::get("pdfjs/web/mailwoman.html")
        && let Ok(html) = std::str::from_utf8(&html.data)
    {
        out.extend(referenced(html));
    }
    if let Some(m) = PdfjsAssets::get(MANIFEST).and_then(|m| manifest_files(&m.data)) {
        out.extend(m.into_iter().map(|f| format!("pdfjs/{f}")));
    }
    out.sort();
    out.dedup();
    out
}

/// Local `src=`/`href=` targets of the entry document, as embedded-set paths
/// (resolved against `pdfjs/web/`).
pub(crate) fn referenced(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    for attr in ["src=\"", "href=\""] {
        for part in html.split(attr).skip(1) {
            let Some(target) = part.split('"').next() else {
                continue;
            };
            if target.is_empty()
                || target.starts_with('#')
                || target.contains(':')
                || target.starts_with("//")
            {
                continue;
            }
            let mut segs: Vec<&str> = vec!["pdfjs", "web"];
            for s in target.split(['?', '#']).next().unwrap_or("").split('/') {
                match s {
                    "." | "" => {}
                    ".." => {
                        segs.pop();
                    }
                    s => segs.push(s),
                }
            }
            out.push(segs.join("/"));
        }
    }
    out
}

/// Check the embedded set is complete and typed, and report whether a deployed
/// `MW_WEB_DIR` copy of the library is current. Cheap enough for a diagnostics run.
pub(crate) fn audit(web_dir: Option<&Path>) -> Audit {
    let pinned = PdfjsAssets::get(MANIFEST).and_then(|m| manifest_files(&m.data));
    let Some(pinned) = pinned else {
        return Audit {
            status: "fail",
            summary: "This build has no readable PDF.js version stamp".into(),
        };
    };
    let needed = required();
    let missing: Vec<&String> = needed
        .iter()
        .filter(|p| PdfjsAssets::get(p).is_none())
        .collect();
    if !missing.is_empty() {
        return Audit {
            status: "fail",
            summary: format!(
                "This build is missing {} file(s) the PDF viewer loads (first: {})",
                missing.len(),
                missing[0]
            ),
        };
    }
    let bad_type = needed.iter().find(|p| {
        let m = mime_for(p);
        (p.ends_with(".mjs") && !m.starts_with("text/javascript"))
            || (p.ends_with(".css") && !m.starts_with("text/css"))
            || (p.ends_with(".wasm") && m != "application/wasm")
    });
    if let Some(p) = bad_type {
        return Audit {
            status: "fail",
            summary: format!("The PDF viewer file {p} would be served with a wrong type"),
        };
    }
    if let Some(dir) = web_dir
        && dir.join("pdfjs").exists()
    {
        let complete = pinned
            .iter()
            .all(|f| dir.join("pdfjs").join(f).is_file());
        if !manifest_matches(dir) || !complete {
            return Audit {
                status: "warn",
                summary: format!(
                    "The PDF viewer in the web folder is stale or incomplete and is ignored; the {} embedded files are served",
                    needed.len()
                ),
            };
        }
    }
    Audit {
        status: "pass",
        summary: format!(
            "All {} PDF viewer files are present, correctly typed and match this build",
            needed.len()
        ),
    }
}

/// Log the audit once at start-up.
pub(crate) fn log_startup_audit(web_dir: Option<&Path>) {
    let a = audit(web_dir);
    match a.status {
        "pass" => tracing::info!("{}", a.summary),
        "warn" => tracing::warn!(
            "{}. Redeploy the web build, or remove pdfjs/ from MW_WEB_DIR.",
            a.summary
        ),
        _ => tracing::error!("{}. Rebuild the server image.", a.summary),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_asset_the_entry_document_and_bootstrap_load_is_embedded_and_typed() {
        let html = std::fs::read_to_string(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../apps/web/public/pdfjs/web/mailwoman.html"),
        )
        .unwrap();
        let refs = referenced(&html);
        for expected in [
            "pdfjs/bootstrap.mjs",
            "pdfjs/mailwoman.css",
            "pdfjs/web/viewer.css",
            "pdfjs/web/locale/locale.json",
        ] {
            assert!(refs.iter().any(|r| r == expected), "{expected} not referenced: {refs:?}");
        }
        // Modules the bootstrap imports and the worker it names.
        let boot = std::fs::read_to_string(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../apps/web/public/pdfjs/bootstrap.mjs"),
        )
        .unwrap();
        for import in ["./polyfills.mjs", "./build/pdf.mjs", "./web/viewer.mjs", "../pdf.worker.mjs"] {
            assert!(boot.contains(import), "bootstrap no longer names {import}");
        }
        for path in refs.iter().chain(&required()) {
            assert!(PdfjsAssets::get(path).is_some(), "viewer references {path}, which is not vendored");
        }
        let a = audit(None);
        assert_eq!(a.status, "pass", "{}", a.summary);
    }

    #[test]
    fn script_and_style_types_are_never_json_or_guessed() {
        assert_eq!(mime_for("pdfjs/bootstrap.mjs"), "text/javascript; charset=utf-8");
        assert_eq!(mime_for("pdfjs/web/viewer.css"), "text/css; charset=utf-8");
        assert_eq!(mime_for("pdfjs/web/wasm/qcms_bg.wasm"), "application/wasm");
        assert_eq!(mime_for("pdfjs/web/cmaps/78-H.bcmap"), "application/octet-stream");
        assert_eq!(mime_for("pdfjs/web/locale/en-US/viewer.ftl"), "text/plain; charset=utf-8");
    }

    #[test]
    fn a_stale_deployed_manifest_is_ignored() {
        let dir = std::env::temp_dir().join(format!("mw-pdfjs-stale-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("pdfjs")).unwrap();
        std::fs::write(dir.join(MANIFEST), "{\"version\":\"0.0.1\",\"files\":{}}").unwrap();
        assert!(!manifest_matches(&dir));
        let a = audit(Some(&dir));
        assert_eq!(a.status, "warn");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
