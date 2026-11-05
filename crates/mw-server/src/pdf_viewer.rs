//! Fixed, self-hosted PDF.js viewer document. PDF bytes arrive from the parent
//! after attachment authorization; query strings never select a document.
use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
};

#[derive(Clone)]
pub(crate) struct PdfViewerResponse;

pub(crate) const PATH: &str = "pdfjs/web/mailwoman.html";
const CSP: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self' data:; style-src 'self' 'unsafe-inline'; img-src 'self' blob: data:; font-src 'self' blob: data:; connect-src 'self' blob: data:; media-src blob:; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'self'; require-trusted-types-for 'script'; trusted-types default";

pub(crate) fn response(method: &Method, headers: &HeaderMap) -> Response {
    if !matches!(*method, Method::GET | Method::HEAD) {
        return (
            StatusCode::METHOD_NOT_ALLOWED,
            [(header::ALLOW, "GET, HEAD")],
        )
            .into_response();
    }
    // Reproducibly generated from the pinned upstream HTML by the vendor script.
    // The only executable entry configures PDF.js before importing its viewer.
    let html = include_str!("../../../apps/web/public/pdfjs/web/mailwoman.html");
    let len = html.len();
    let mut response = super::asset_response(
        PATH,
        "text/html; charset=utf-8",
        html.as_bytes().to_vec(),
        headers,
    );
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(CSP),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    response.extensions_mut().insert(PdfViewerResponse);
    if *method == Method::HEAD {
        if response.status() == StatusCode::OK {
            response
                .headers_mut()
                .insert(header::CONTENT_LENGTH, HeaderValue::from(len));
        }
        *response.body_mut() = Body::empty();
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_document_has_only_configured_bootstrap_and_independent_policy() {
        let entry = include_str!("../../../apps/web/public/pdfjs/web/mailwoman.html");
        assert!(entry.contains("../bootstrap.mjs"));
        assert!(!entry.contains("<script src=\"viewer.mjs\""));
        assert!(!entry.contains("connect-src *"));
        let original = include_str!("../../../apps/web/public/pdfjs/web/viewer.html");
        assert_eq!(
            original
                .matches("<script src=\"../build/pdf.mjs\" type=\"module\"></script>")
                .count(),
            1
        );
        assert_eq!(
            original
                .matches("<script src=\"viewer.mjs\" type=\"module\"></script>")
                .count(),
            1
        );
        let response = response(&Method::GET, &HeaderMap::new());
        assert!(response.extensions().get::<PdfViewerResponse>().is_some());
        assert_eq!(response.headers()[header::CONTENT_SECURITY_POLICY], CSP);
        assert!(CSP.contains("frame-ancestors 'self'"));
        assert!(CSP.contains("require-trusted-types-for 'script'"));
        assert!(!CSP.contains("connect-src *"));
        assert_eq!(
            super::response(&Method::POST, &HeaderMap::new()).status(),
            StatusCode::METHOD_NOT_ALLOWED
        );
    }
}
