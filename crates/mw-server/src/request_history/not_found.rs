//! Which 404 answers are counted, and the redacted form a not-found path is kept in.
//!
//! **Counted.** Every 404 from any client, except the app's own designed misses
//! ([`expected`]): browser-automatic icon requests and the few routes the web client
//! calls knowing a 404 means "nothing here" (see `docs/admin/latency-metrics.md`).
//! JMAP `unknownMethod` is a method error inside an HTTP 200, so it never is a 404.
//!
//! **Redacted.** [`normalise`] keeps the shape of a path and drops what could
//! identify a person or authorise a request: the query and fragment are dropped;
//! email addresses, numbers, UUIDs, hex, base64/base64url and other long or token-like
//! segments, `key=value` values and the signed `/api/rc/*` tokens become
//! placeholders; anything outside printable ASCII becomes `_`; the result is capped
//! at 16 segments and [`NOT_FOUND_PATH_MAX_BYTES`]. No address, header or account is
//! ever read here.
use axum::http::Method;
use mw_store::request_metrics::NOT_FOUND_PATH_MAX_BYTES;

const MAX_SEGMENTS: usize = 16;
const MAX_SEGMENT_BYTES: usize = 40;

/// A bounded method label: the methods this server answers, or `OTHER`.
pub(crate) fn method_label(method: &Method) -> &'static str {
    const KNOWN: [&str; 16] = [
        "GET",
        "HEAD",
        "POST",
        "PUT",
        "PATCH",
        "DELETE",
        "OPTIONS",
        "PROPFIND",
        "PROPPATCH",
        "REPORT",
        "MKCOL",
        "MKCALENDAR",
        "COPY",
        "MOVE",
        "LOCK",
        "UNLOCK",
    ];
    KNOWN
        .iter()
        .find(|known| **known == method.as_str())
        .copied()
        .unwrap_or("OTHER")
}

/// For a 404 answer: `None` when it is one of the app's designed misses, else the
/// bounded method label and the redacted path it is counted under.
pub(crate) fn counted(method: &Method, path: &str) -> Option<(&'static str, String)> {
    (!expected(method, path)).then(|| (method_label(method), normalise(path)))
}

/// The app's own designed 404s, which are not counted. `path` is relative to the
/// deploy prefix (as the metrics layer sees it).
pub(crate) fn expected(method: &Method, path: &str) -> bool {
    let path = path.split(['?', '#']).next().unwrap_or("");
    // Icons browsers request on their own, whether or not a page links them.
    let icon = path == "/favicon.ico"
        || path == "/favicon.svg"
        || path == "/favicon.png"
        || (path.starts_with("/apple-touch-icon")
            && path.ends_with(".png")
            && !path[1..].contains('/'));
    if icon && (method == Method::GET || method == Method::HEAD) {
        return true;
    }
    if method != Method::GET {
        return false;
    }
    let one_segment_under = |prefix: &str| {
        path.strip_prefix(prefix)
            .is_some_and(|rest| !rest.is_empty() && !rest.contains('/'))
    };
    // 404 = no receipt decision recorded for this message (`api/receipts.ts`).
    one_segment_under("/api/account/mdn-decision/")
        // 404 = no tracking pixel bound to this message (`api/tracking.ts`).
        || path == "/api/track/open"
        // Polled until the other device relays the envelope (`zeroaccess/pairing.ts`).
        || one_segment_under("/api/zeroaccess/pair/envelope/")
        // The command palette's optional server search; unmounted, it answers 404
        // and the palette says only local results are shown (`CommandPalette.tsx`).
        || path == mw_admin::config::admin_route("/search")
}

/// The redacted, bounded form of a not-found path (always starts with `/`).
pub(crate) fn normalise(path: &str) -> String {
    let path = path.split(['?', '#']).next().unwrap_or("");
    let segments: Vec<&str> = path.split('/').skip(1).collect();
    let mut out = String::with_capacity(path.len().min(NOT_FOUND_PATH_MAX_BYTES) + 1);
    let mut rc = 0_u8;
    for (index, segment) in segments.iter().enumerate() {
        if index == MAX_SEGMENTS {
            out.push_str("/{more}");
            break;
        }
        out.push('/');
        // `/api/rc/<kind>/<signed token>`: the token authorises a fetch on its own.
        rc = match (rc, *segment) {
            (0, "api") => 1,
            (1, "rc") => 2,
            (2, _) => 3,
            (3, _) => {
                out.push_str("{rc-token}");
                4
            }
            _ => 5,
        };
        if rc == 4 {
            continue;
        }
        out.push_str(&redact_segment(segment));
    }
    if out.is_empty() {
        out.push('/');
    }
    if out.len() > NOT_FOUND_PATH_MAX_BYTES {
        // ASCII only, so any byte index is a char boundary.
        out.truncate(NOT_FOUND_PATH_MAX_BYTES - "/{more}".len());
        if let Some(cut) = out.rfind('/') {
            out.truncate(cut);
        }
        out.push_str("/{more}");
    }
    out
}

fn redact_segment(segment: &str) -> String {
    if segment.is_empty() {
        return String::new();
    }
    let lower = segment.to_ascii_lowercase();
    if segment.contains('@') || lower.contains("%40") {
        return "{email}".into();
    }
    if let Some((key, _)) = segment.split_once('=') {
        return format!("{}={{value}}", redact_segment(key));
    }
    if segment.len() > MAX_SEGMENT_BYTES {
        return "{long}".into();
    }
    // Keep a short file extension so `/{token}.php` still reads as a probe for PHP.
    let (stem, extension) = match segment.rsplit_once('.') {
        Some((stem, ext))
            if !stem.is_empty()
                && (1..=5).contains(&ext.len())
                && ext.bytes().all(|b| b.is_ascii_alphanumeric()) =>
        {
            (stem, Some(ext))
        }
        _ => (segment, None),
    };
    let placeholder = classify(stem);
    let mut out = match placeholder {
        Some(placeholder) => placeholder.to_string(),
        None => stem
            .bytes()
            .map(|b| {
                if (0x21..0x7f).contains(&b) && !matches!(b, b'?' | b'#' | b'@') {
                    char::from(b)
                } else {
                    '_'
                }
            })
            .collect(),
    };
    if let Some(ext) = extension {
        out.push('.');
        out.push_str(ext);
    }
    out
}

fn classify(stem: &str) -> Option<&'static str> {
    let bytes = stem.as_bytes();
    let digits = bytes.iter().filter(|b| b.is_ascii_digit()).count();
    let letters = bytes.iter().filter(|b| b.is_ascii_alphabetic()).count();
    if digits == bytes.len() {
        return Some("{n}");
    }
    if is_uuid(stem) {
        return Some("{uuid}");
    }
    if bytes.len() >= 8 && bytes.iter().all(u8::is_ascii_hexdigit) && digits > 0 {
        return Some("{hex}");
    }
    let token_chars = bytes.iter().all(|b| {
        b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'+' | b'=' | b'.' | b'~' | b'%')
    });
    // Percent-encoded payloads, base64/base64url and JWT-like strings.
    if token_chars && (stem.contains('%') && bytes.len() >= 12) {
        return Some("{token}");
    }
    if token_chars && bytes.len() >= 16 && digits > 0 && letters > 0 {
        return Some("{token}");
    }
    if token_chars && bytes.len() >= 24 {
        return Some("{token}");
    }
    // An id with a word prefix, such as `Ma1b2c3` or `u_8f3a91`.
    if bytes.len() >= 6 && digits * 3 >= bytes.len() {
        return Some("{id}");
    }
    None
}

fn is_uuid(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 36
        && b.iter().enumerate().all(|(i, c)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                *c == b'-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries_tokens_ids_and_emails_are_redacted() {
        for (raw, want) in [
            ("/wp-login.php?user=admin&pass=x", "/wp-login.php"),
            ("/.env", "/.env"),
            ("/", "/"),
            ("", "/"),
            ("/api/users/12345", "/api/users/{n}"),
            ("/u/550e8400-e29b-41d4-a716-446655440000/x", "/u/{uuid}/x"),
            ("/blob/deadbeefcafe0123", "/blob/{hex}"),
            (
                "/reset/eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.c2ln",
                "/reset/{long}",
            ),
            ("/invite/Zm9vYmFyYmF6cXV4MTIz", "/invite/{token}"),
            ("/share/abcdefghijklmnopqrstuvwxyz", "/share/{token}"),
            ("/p/alice@example.com/inbox", "/p/{email}/inbox"),
            ("/p/alice%40example.com", "/p/{email}"),
            ("/api/rc/i/c2hvcnQ", "/api/rc/i/{rc-token}"),
            ("/api/rc/f/x/y", "/api/rc/f/{rc-token}/y"),
            ("/a;jsessionid=ABC123DEF", "/a;jsessionid={value}"),
            ("/backup/8f3a91b2.zip", "/backup/{hex}.zip"),
            ("/m/Ma1b2c3", "/m/{id}"),
        ] {
            let got = normalise(raw);
            assert_eq!(got, want, "{raw}");
            assert!(!got.contains(['?', '@']), "{got}");
        }
    }

    #[test]
    fn length_and_segment_count_are_capped_and_output_is_printable_ascii() {
        let deep = "/a".repeat(100);
        let got = normalise(&deep);
        assert!(got.ends_with("/{more}"));
        assert_eq!(got.matches('/').count(), MAX_SEGMENTS + 1);
        let wide = format!("/{}", "word-".repeat(5));
        assert!(normalise(&wide).len() <= NOT_FOUND_PATH_MAX_BYTES);
        let long: String = (0..15).map(|_| "/abcdefghij").collect();
        let got = normalise(&long);
        assert!(got.len() <= NOT_FOUND_PATH_MAX_BYTES, "{got}");
        let odd = normalise("/caf\u{e9}/\u{1}x");
        assert!(odd.bytes().all(|b| (0x21..0x7f).contains(&b)), "{odd}");
    }

    #[test]
    fn the_apps_own_designed_misses_are_excluded_and_others_are_not() {
        for (method, path) in [
            (Method::GET, "/favicon.ico"),
            (Method::HEAD, "/favicon.svg"),
            (Method::GET, "/apple-touch-icon-precomposed.png"),
            (Method::GET, "/apple-touch-icon-180x180.png"),
            (Method::GET, "/api/account/mdn-decision/M123"),
            (Method::GET, "/api/track/open"),
            (Method::GET, "/api/zeroaccess/pair/envelope/p1"),
        ] {
            assert!(expected(&method, path), "{method} {path}");
        }
        assert!(expected(
            &Method::GET,
            &mw_admin::config::admin_route("/search")
        ));
        for (method, path) in [
            (Method::GET, "/wp-login.php"),
            (Method::POST, "/favicon.ico"),
            (Method::POST, "/api/track/open"),
            (Method::GET, "/api/account/mdn-decision/a/b"),
            (Method::GET, "/icons/favicon.ico"),
            (Method::GET, "/api/nope"),
        ] {
            assert!(!expected(&method, path), "{method} {path}");
        }
    }

    #[test]
    fn methods_are_a_bounded_label_set() {
        assert_eq!(method_label(&Method::GET), "GET");
        assert_eq!(
            method_label(&Method::from_bytes(b"PROPFIND").unwrap()),
            "PROPFIND"
        );
        assert_eq!(
            method_label(&Method::from_bytes(b"XYZZY").unwrap()),
            "OTHER"
        );
    }
}
