//! Pure request inspection: headers, target, method and version in, typed
//! findings out. No I/O, no header value ever leaves this module: a finding is a
//! class plus a static code, so nothing a client sent can reach a log or a reply.
use super::config::{Class, Config, Mode, normalise_host};
use axum::http::{HeaderMap, HeaderName, Method, StatusCode, Uri, Version, header};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Finding {
    pub class: Class,
    pub code: &'static str,
}
impl Finding {
    fn new(class: Class, code: &'static str) -> Self {
        Self { class, code }
    }
    /// The reject status for this finding when its class enforces.
    pub fn status(&self) -> StatusCode {
        match self.code {
            "path-long" | "query-long" => StatusCode::URI_TOO_LONG,
            "too-many-headers" | "header-value-long" | "cookie-large" => {
                StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE
            }
            "body-too-large" => StatusCode::PAYLOAD_TOO_LARGE,
            "method-not-allowed" => StatusCode::METHOD_NOT_ALLOWED,
            "expect-unknown" => StatusCode::EXPECTATION_FAILED,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

/// What the guard may look at.
pub(crate) struct View<'a> {
    pub method: &'a Method,
    pub uri: &'a Uri,
    pub version: Version,
    pub headers: &'a HeaderMap,
    /// The socket peer is inside `MW_TRUSTED_PROXIES` (the `scope_mw::proxy` authority).
    pub peer_trusted: bool,
    /// Any trusted proxy is configured at all.
    pub proxies_configured: bool,
    pub peer_loopback: bool,
}

fn http1(version: Version) -> bool {
    matches!(
        version,
        Version::HTTP_09 | Version::HTTP_10 | Version::HTTP_11
    )
}
fn text(value: &header::HeaderValue) -> Option<&str> {
    value.to_str().ok()
}

/// Parse a `Content-Length` header set. `Err` is a syntax error; `Ok` carries the
/// distinct values and how many times a value was stated.
fn content_lengths(headers: &HeaderMap) -> Result<(Vec<u64>, usize), ()> {
    let mut values = Vec::new();
    let mut stated = 0;
    for value in headers.get_all(header::CONTENT_LENGTH) {
        for token in text(value).ok_or(())?.split(',') {
            let token = token.trim();
            if token.is_empty() || !token.bytes().all(|b| b.is_ascii_digit()) {
                return Err(());
            }
            let n: u64 = token.parse().map_err(|_| ())?;
            stated += 1;
            if !values.contains(&n) {
                values.push(n);
            }
        }
    }
    Ok((values, stated))
}

/// A syntactically valid `Host`: `reg-name`, IPv4 or `[IPv6]`, optional numeric
/// port, optional trailing dot. No userinfo, whitespace, control or path characters.
pub(crate) fn host_syntax_ok(value: &str) -> bool {
    if value.is_empty() || value.len() > 255 {
        return false;
    }
    let (host, port) = if let Some(rest) = value.strip_prefix('[') {
        let Some((inside, after)) = rest.split_once(']') else {
            return false;
        };
        if inside.is_empty()
            || !inside
                .bytes()
                .all(|b| b.is_ascii_hexdigit() || matches!(b, b':' | b'.'))
            || !inside.contains(':')
        {
            return false;
        }
        match after {
            "" => ("x", None),
            _ => match after.strip_prefix(':') {
                Some(port) => ("x", Some(port)),
                None => return false,
            },
        }
    } else {
        match value.rsplit_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (value, None),
        }
    };
    if let Some(port) = port
        && (port.is_empty()
            || port.len() > 5
            || !port.bytes().all(|b| b.is_ascii_digit())
            || port.parse::<u32>().map_or(true, |p| p > 65535))
    {
        return false;
    }
    host.bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
        && !host.starts_with('.')
        && !host.contains("..")
}

const FORWARDING: &[&str] = &[
    "forwarded",
    "x-forwarded-for",
    "x-forwarded-proto",
    "x-forwarded-host",
    "x-forwarded-port",
    "x-real-ip",
];
/// Headers that carry one value; a second one means two parsers can disagree.
const SINGLETON: &[&str] = &[
    "content-type",
    "authorization",
    "origin",
    "referer",
    "user-agent",
    "sec-fetch-site",
    "sec-fetch-mode",
    "sec-fetch-dest",
    "sec-fetch-user",
    "if-match",
    "if-none-match",
    "range",
];
/// Connection-listed names that strip a header a later hop depends on.
const PROTECTED_FROM_HOP: &[&str] = &[
    "content-length",
    "host",
    "transfer-encoding",
    "authorization",
    "cookie",
    "x-forwarded-for",
    "forwarded",
];

fn is_websocket_route(path: &str) -> bool {
    path.ends_with("/jmap/ws")
}

pub(crate) fn inspect(cfg: &Config, view: &View<'_>) -> Vec<Finding> {
    let mut out = Vec::new();
    let on = |class: Class| cfg.mode(class) != Mode::Off;
    let path = view.uri.path();
    let headers = view.headers;
    let exempt = cfg.exempts(path);
    let heuristic = |class: Class| on(class) && !(exempt && class.exemptible());

    // (2) framing: body length must be unambiguous.
    if on(Class::Framing) {
        let te: Vec<_> = headers.get_all(header::TRANSFER_ENCODING).iter().collect();
        let has_cl = headers.contains_key(header::CONTENT_LENGTH);
        if has_cl && !te.is_empty() {
            out.push(Finding::new(Class::Framing, "cl-and-te"));
        }
        if !te.is_empty() {
            if !http1(view.version) || view.version == Version::HTTP_10 {
                out.push(Finding::new(Class::Framing, "te-version"));
            }
            let exact = te.len() == 1
                && text(te[0]).is_some_and(|v| v.trim().eq_ignore_ascii_case("chunked"));
            if !exact {
                out.push(Finding::new(Class::Framing, "te-value"));
            }
        }
        match content_lengths(headers) {
            Err(()) => out.push(Finding::new(Class::Framing, "cl-syntax")),
            Ok((values, _)) if values.len() > 1 => {
                out.push(Finding::new(Class::Framing, "cl-conflict"))
            }
            _ => {}
        }
    }

    // (3) Host.
    if on(Class::Host) {
        let hosts: Vec<_> = headers.get_all(header::HOST).iter().collect();
        match hosts.len() {
            0 => {
                if view.version == Version::HTTP_11 && view.uri.authority().is_none() {
                    out.push(Finding::new(Class::Host, "host-missing"));
                }
            }
            1 => match text(hosts[0]) {
                Some("") => out.push(Finding::new(Class::Host, "host-empty")),
                Some(value) if host_syntax_ok(value) => {}
                _ => out.push(Finding::new(Class::Host, "host-syntax")),
            },
            _ => out.push(Finding::new(Class::Host, "host-multiple")),
        }
    }
    if heuristic(Class::HostPinning) && !cfg.hosts.is_empty() {
        let named = headers
            .get(header::HOST)
            .and_then(text)
            .map(normalise_host)
            .or_else(|| view.uri.host().map(normalise_host));
        if let Some(named) = named.filter(|h| !h.is_empty()) {
            let loopback = matches!(named.as_str(), "localhost" | "127.0.0.1" | "::1");
            if !cfg.hosts.contains(&named) && !(loopback && view.peer_loopback) {
                out.push(Finding::new(Class::HostPinning, "host-unpinned"));
            }
        }
    }

    // (4) request target. Hyper already refuses raw control characters, spaces
    // and non-ASCII in the target; these are the encoded forms that survive.
    if on(Class::PathControl) {
        let lower = path.to_ascii_lowercase();
        if path.bytes().any(|b| b < 0x20 || b == 0x7f) {
            out.push(Finding::new(Class::PathControl, "path-control-char"));
        } else if lower.contains("%00") {
            out.push(Finding::new(Class::PathControl, "path-nul"));
        } else if lower.contains("%0d") || lower.contains("%0a") {
            out.push(Finding::new(Class::PathControl, "path-crlf"));
        }
    }
    if heuristic(Class::Target) {
        let lower = path.to_ascii_lowercase();
        let code = if path.contains('\\') || lower.contains("%5c") {
            Some("path-backslash")
        } else if lower.contains("%25") && ["%252e", "%252f", "%255c", "%2500"].iter().any(|p| lower.contains(p)) {
            Some("path-double-encoded")
        } else if lower.contains("%2e") {
            Some("path-encoded-dot")
        } else if ["%c0%", "%c1%", "%e0%80", "%f0%80"].iter().any(|p| lower.contains(p)) {
            Some("path-overlong-utf8")
        } else if path.split('/').any(|s| s == "." || s == "..")
            || lower.contains("%2f..")
            || lower.contains("..%2f")
        {
            Some("path-dot-segment")
        } else if path.contains("//") {
            Some("path-empty-segment")
        } else if !path.is_ascii() {
            // Browsers percent-encode; hyper passes raw high bytes through.
            Some("path-non-ascii")
        } else if (path.is_empty() || view.uri.path_and_query().is_none())
            && *view.method != Method::CONNECT
        {
            Some("path-empty")
        } else {
            None
        };
        if let Some(code) = code {
            out.push(Finding::new(Class::Target, code));
        }
        if let (Some(authority), Some(host)) = (view.uri.authority(), headers.get(header::HOST).and_then(text))
            && normalise_host(authority.as_str()) != normalise_host(host)
        {
            out.push(Finding::new(Class::Target, "authority-mismatch"));
        }
    }
    if heuristic(Class::PathLength) {
        if path.len() > cfg.max_path {
            out.push(Finding::new(Class::PathLength, "path-long"));
        } else if view.uri.query().is_some_and(|q| q.len() > cfg.max_query) {
            out.push(Finding::new(Class::PathLength, "query-long"));
        }
    }

    // (5) method.
    if heuristic(Class::Method) && !cfg.methods.contains(view.method.as_str()) {
        out.push(Finding::new(Class::Method, "method-not-allowed"));
    }

    // (1) duplicate singleton headers.
    if heuristic(Class::DuplicateHeaders) {
        if let Ok((_, stated)) = content_lengths(headers)
            && stated > 1
        {
            out.push(Finding::new(Class::DuplicateHeaders, "content-length-repeated"));
        }
        if SINGLETON
            .iter()
            .any(|name| headers.get_all(HeaderName::from_static(name)).iter().count() > 1)
        {
            out.push(Finding::new(Class::DuplicateHeaders, "singleton-repeated"));
        }
        if http1(view.version) && headers.get_all(header::COOKIE).iter().count() > 1 {
            out.push(Finding::new(Class::DuplicateHeaders, "cookie-repeated"));
        }
    }
    if heuristic(Class::Forwarded)
        && !view.peer_trusted
        && FORWARDING
            .iter()
            .any(|name| headers.contains_key(HeaderName::from_static(name)))
    {
        out.push(Finding::new(Class::Forwarded, "forwarded-from-untrusted-peer"));
    }

    // (6) header hygiene.
    if heuristic(Class::HeaderHygiene) {
        if headers.len() > cfg.max_headers {
            out.push(Finding::new(Class::HeaderHygiene, "too-many-headers"));
        }
        if headers.values().any(|v| v.len() > cfg.max_header_value) {
            out.push(Finding::new(Class::HeaderHygiene, "header-value-long"));
        }
        let cookie: usize = headers.get_all(header::COOKIE).iter().map(|v| v.len()).sum();
        if cookie > cfg.max_cookie {
            out.push(Finding::new(Class::HeaderHygiene, "cookie-large"));
        }
        if headers.values().any(|v| v.to_str().is_err()) {
            out.push(Finding::new(Class::HeaderHygiene, "header-opaque-bytes"));
        }
        let upgrade = headers.get(header::UPGRADE);
        let tokens: Vec<String> = headers
            .get_all(header::CONNECTION)
            .iter()
            .filter_map(text)
            .flat_map(|v| v.split(','))
            .map(|t| t.trim().to_ascii_lowercase())
            .collect();
        let connection_upgrade = tokens.iter().any(|t| t == "upgrade");
        if http1(view.version) {
            if let Some(upgrade) = upgrade {
                let websocket = text(upgrade).is_some_and(|v| v.trim().eq_ignore_ascii_case("websocket"));
                if !(websocket && is_websocket_route(path)) {
                    out.push(Finding::new(Class::HeaderHygiene, "upgrade-unexpected"));
                }
            }
            if upgrade.is_some() != connection_upgrade {
                out.push(Finding::new(Class::HeaderHygiene, "upgrade-connection-mismatch"));
            }
        }
        if tokens.iter().any(|t| PROTECTED_FROM_HOP.contains(&t.as_str())) {
            out.push(Finding::new(Class::HeaderHygiene, "connection-strips-header"));
        }
    }

    // (7) body framing that is visible to the application.
    if heuristic(Class::Body) {
        let (lengths, _) = content_lengths(headers).unwrap_or_default();
        let declared = lengths.iter().copied().max().unwrap_or(0);
        let has_body = declared > 0 || headers.contains_key(header::TRANSFER_ENCODING);
        if matches!(*view.method, Method::GET | Method::HEAD) && has_body {
            out.push(Finding::new(Class::Body, "safe-method-body"));
        }
        if declared > cfg.max_body {
            out.push(Finding::new(Class::Body, "body-too-large"));
        }
        if let Some(expect) = headers.get(header::EXPECT) {
            if !text(expect).is_some_and(|v| v.trim().eq_ignore_ascii_case("100-continue")) {
                out.push(Finding::new(Class::Body, "expect-unknown"));
            } else if !has_body {
                out.push(Finding::new(Class::Body, "expect-without-body"));
            }
        }
    }
    out
}
