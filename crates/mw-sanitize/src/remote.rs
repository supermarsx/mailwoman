//! Remote content in mail: placeholders, trackers and link cleaning (t32 e31a,
//! plan R).
//!
//! # Placeholders
//! In [`crate::RemoteImages::Placeholder`] mode the sanitizer does not drop a
//! remote image reference; it replaces the URL with `mw-rc:<n>` and appends a
//! hidden marker carrying `n` and the original URL. Nothing loads from a
//! placeholder: the scheme is not one the body frame's CSP or any browser
//! fetches. The reader asks the server to sign the URLs it may load
//! (`POST /api/rc/sign`) and swaps `mw-rc:<n>` for the signed same-origin path.
//! A message's own HTML cannot produce a placeholder: the URL-scheme allowlist
//! runs before this module's rewrite and refuses `mw-rc:` in the input.
//!
//! # Trackers
//! An image is a tracker when its host is on `data/trackers.txt` (or a subdomain
//! of one), when its URL matches a shape in `data/beacons.txt`, or when it is
//! declared at most 2×2 pixels by attributes or inline CSS. The sanitizer marks
//! trackers; with tracker protection on it never produces a loadable reference
//! for one, and the server refuses to sign one.
//!
//! # Link cleaning
//! [`clean_link`] removes the query parameters listed in
//! `data/tracking_params.txt` from an http(s) link and keeps everything else of
//! the URL as written.

use std::sync::OnceLock;

/// The scheme of a placeholder reference.
pub const PLACEHOLDER_SCHEME: &str = "mw-rc";

/// Largest width or height, in CSS pixels, at which a declared image counts as a
/// tracking pixel.
pub const PIXEL_MAX: f32 = 2.0;

const TRACKERS: &str = include_str!("../data/trackers.txt");
const BEACONS: &str = include_str!("../data/beacons.txt");
const TRACKING_PARAMS: &str = include_str!("../data/tracking_params.txt");

fn entries(src: &'static str) -> impl Iterator<Item = &'static str> {
    src.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
}

/// The placeholder for reference `n`.
pub fn placeholder(n: usize) -> String {
    format!("{PLACEHOLDER_SCHEME}:{n}")
}

/// The reference number of a placeholder, or `None` when `value` is not one.
pub fn parse_placeholder(value: &str) -> Option<usize> {
    value
        .strip_prefix(PLACEHOLDER_SCHEME)?
        .strip_prefix(':')?
        .parse()
        .ok()
}

/// Whether `value` is an absolute `http`/`https` URL (scheme compared without case).
pub fn is_http_url(value: &str) -> bool {
    let v = value.trim_start();
    v.len() > 8
        && (v[..7].eq_ignore_ascii_case("http://") || v[..8].eq_ignore_ascii_case("https://"))
}

/// The parts of an absolute http(s) URL this module needs: lower-cased host, the
/// path, and the query (without `?`). `None` when `url` is not one.
struct Parts<'a> {
    host: String,
    path: &'a str,
    query: Option<&'a str>,
}

fn parts(url: &str) -> Option<Parts<'_>> {
    if !is_http_url(url) {
        return None;
    }
    let after = url.trim_start().split_once("://")?.1;
    let end = after.find(['/', '?', '#']).unwrap_or(after.len());
    let authority = &after[..end];
    let rest = &after[end..];
    let host_port = authority.rsplit_once('@').map_or(authority, |(_, hp)| hp);
    let host = match host_port.strip_prefix('[') {
        Some(v6) => v6.split(']').next().unwrap_or(v6),
        None => host_port.split(':').next().unwrap_or(host_port),
    };
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
    if host.is_empty() {
        return None;
    }
    let rest = rest.split('#').next().unwrap_or(rest);
    let (path, query) = match rest.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (rest, None),
    };
    Some(Parts { host, path, query })
}

/// The lower-cased host of an absolute http(s) URL.
pub fn http_url_host(url: &str) -> Option<String> {
    parts(url).map(|p| p.host)
}

fn tracker_hosts() -> &'static [&'static str] {
    static LIST: OnceLock<Vec<&'static str>> = OnceLock::new();
    LIST.get_or_init(|| entries(TRACKERS).collect())
}

/// Whether `host` (lower-cased) is on the tracker list or below a listed host.
pub fn is_tracker_host(host: &str) -> bool {
    tracker_hosts().iter().any(|t| {
        host == *t
            || host
                .strip_suffix(t)
                .is_some_and(|prefix| prefix.ends_with('.'))
    })
}

enum Beacon {
    PathEnd(String),
    Segment(String),
    Query(Vec<String>),
}

fn beacons() -> &'static [Beacon] {
    static LIST: OnceLock<Vec<Beacon>> = OnceLock::new();
    LIST.get_or_init(|| {
        entries(BEACONS)
            .filter_map(|l| {
                let (kind, value) = l.split_once(':')?;
                let value = value.trim().to_ascii_lowercase();
                match kind.trim() {
                    "path-end" => Some(Beacon::PathEnd(value)),
                    "segment" => Some(Beacon::Segment(value)),
                    "query" => Some(Beacon::Query(
                        value.split(',').map(|k| k.trim().to_string()).collect(),
                    )),
                    _ => None,
                }
            })
            .collect()
    })
}

/// Whether `url` has the shape of an open-tracking beacon.
pub fn is_beacon_url(url: &str) -> bool {
    let Some(p) = parts(url) else {
        return false;
    };
    let path = p.path.to_ascii_lowercase();
    let keys: Vec<String> = p
        .query
        .map(|q| {
            q.split('&')
                .filter_map(|pair| pair.split('=').next())
                .map(|k| percent_decode_ascii(k).to_ascii_lowercase())
                .collect()
        })
        .unwrap_or_default();
    beacons().iter().any(|b| match b {
        Beacon::PathEnd(end) => path.ends_with(end.as_str()),
        Beacon::Segment(seg) => path.split('/').any(|s| s == seg),
        Beacon::Query(want) => want.iter().all(|k| keys.iter().any(|have| have == k)),
    })
}

/// Whether an image at `url` is a tracker by its URL alone (host or beacon shape).
pub fn is_tracker_url(url: &str) -> bool {
    http_url_host(url).is_some_and(|h| is_tracker_host(&h)) || is_beacon_url(url)
}

/// Whether a declared size is a tracking pixel: both dimensions declared and each
/// at most [`PIXEL_MAX`].
pub fn is_pixel(width: Option<f32>, height: Option<f32>) -> bool {
    matches!((width, height), (Some(w), Some(h)) if w <= PIXEL_MAX && h <= PIXEL_MAX)
}

/// A size from an HTML `width`/`height` attribute (`1`, `1px`). Percentages and
/// anything else are not a declared pixel size.
pub fn attr_pixels(value: &str) -> Option<f32> {
    let v = value.trim();
    let v = v.strip_suffix("px").unwrap_or(v).trim();
    v.parse::<f32>().ok().filter(|n| n.is_finite() && *n >= 0.0)
}

fn tracking_params() -> &'static [String] {
    static LIST: OnceLock<Vec<String>> = OnceLock::new();
    LIST.get_or_init(|| {
        entries(TRACKING_PARAMS)
            .map(|l| l.to_ascii_lowercase())
            .collect()
    })
}

/// Whether a query parameter name is on the tracking list.
pub fn is_tracking_param(name: &str) -> bool {
    let name = percent_decode_ascii(name).to_ascii_lowercase();
    tracking_params().iter().any(|p| match p.strip_suffix('*') {
        Some(prefix) => name.starts_with(prefix),
        None => name == p.as_str(),
    })
}

/// `href` with the listed tracking parameters removed, or `None` when it is not
/// an http(s) link or has none of them. Everything else — scheme, host, path,
/// the other parameters in their order, the fragment — is kept as written.
pub fn clean_link(href: &str) -> Option<String> {
    if !is_http_url(href) {
        return None;
    }
    let (before_frag, frag) = match href.split_once('#') {
        Some((b, f)) => (b, Some(f)),
        None => (href, None),
    };
    let (base, query) = before_frag.split_once('?')?;
    let kept: Vec<&str> = query
        .split('&')
        .filter(|pair| {
            let name = pair.split('=').next().unwrap_or("");
            !is_tracking_param(name)
        })
        .collect();
    if kept.len() == query.split('&').count() {
        return None;
    }
    let mut out = base.to_string();
    let kept: Vec<&str> = kept.into_iter().filter(|p| !p.is_empty()).collect();
    if !kept.is_empty() {
        out.push('?');
        out.push_str(&kept.join("&"));
    }
    if let Some(f) = frag {
        out.push('#');
        out.push_str(f);
    }
    Some(out)
}

/// The candidates of a `srcset` value as `(url, descriptor)`, or `None` when any
/// candidate is malformed. Candidates are separated by commas; a URL with a comma
/// in it is not supported and makes the whole value malformed (the attribute is
/// then dropped, never partly kept).
pub fn srcset_candidates(value: &str) -> Option<Vec<(String, String)>> {
    let mut out = Vec::new();
    for candidate in value.split(',') {
        let c = candidate.trim();
        if c.is_empty() {
            return None;
        }
        let mut it = c.split_whitespace();
        let url = it.next()?.to_string();
        let descriptor: Vec<&str> = it.collect();
        if descriptor.len() > 1 {
            return None;
        }
        let descriptor = descriptor.first().copied().unwrap_or("").to_string();
        if !descriptor.is_empty() && !(descriptor.ends_with('x') || descriptor.ends_with('w')) {
            return None;
        }
        out.push((url, descriptor));
    }
    (!out.is_empty()).then_some(out)
}

/// Decode `%XX` escapes that produce ASCII; anything else is kept as written.
fn percent_decode_ascii(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    let hex = |c: u8| (c as char).to_digit(16);
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let (Some(hi), Some(lo)) = (hex(b[i + 1]), hex(b[i + 2]))
            && hi < 8
        {
            out.push(char::from((hi * 16 + lo) as u8));
            i += 3;
            continue;
        }
        // Copy one whole character, so a multi-byte one is never split.
        let ch = s[i..].chars().next().expect("i is on a char boundary");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholders_round_trip_and_nothing_else_parses() {
        assert_eq!(placeholder(7), "mw-rc:7");
        assert_eq!(parse_placeholder("mw-rc:7"), Some(7));
        for v in [
            "mw-rc:",
            "mw-rc:x",
            "mw-rcx:1",
            "https://mw-rc:1",
            "cid:mw-rc:1",
        ] {
            assert_eq!(parse_placeholder(v), None, "{v}");
        }
    }

    #[test]
    fn tracker_hosts_match_the_host_and_its_subdomains_only() {
        assert!(is_tracker_host("list-manage.com"));
        assert!(is_tracker_host("us5.list-manage.com"));
        assert!(!is_tracker_host("notlist-manage.com"));
        assert!(!is_tracker_host("list-manage.com.example"));
        assert!(!is_tracker_host("example.com"));
    }

    #[test]
    fn beacon_shapes_are_recognised_and_ordinary_images_are_not() {
        for u in [
            "https://t.example/o.gif",
            "https://mail.example/wf/open?upn=abc",
            "https://x.example/a/b/OPEN.GIF",
            "https://x.example/tracking/123.png",
            "https://x.example/img?e=1&u=2",
            "https://x.example/img?U=2&E=1",
            "https://x.example/img?%65=1&u=2",
        ] {
            assert!(is_beacon_url(u), "{u}");
        }
        for u in [
            "https://cdn.example/logo.png",
            "https://cdn.example/open/logo.png",
            "https://cdn.example/img?e=1",
            "https://cdn.example/trackingpixel.png",
            "cid:part1",
            "data:image/gif;base64,AAAA",
        ] {
            assert!(!is_beacon_url(u), "{u}");
        }
    }

    #[test]
    fn a_pixel_needs_both_dimensions_at_most_two() {
        assert!(is_pixel(Some(1.0), Some(1.0)));
        assert!(is_pixel(Some(2.0), Some(0.0)));
        assert!(!is_pixel(Some(1.0), None));
        assert!(!is_pixel(Some(3.0), Some(1.0)));
        assert_eq!(attr_pixels("1"), Some(1.0));
        assert_eq!(attr_pixels(" 2px "), Some(2.0));
        assert_eq!(attr_pixels("100%"), None);
        assert_eq!(attr_pixels("-1"), None);
    }

    #[test]
    fn links_lose_tracking_parameters_and_keep_everything_else() {
        assert_eq!(
            clean_link("https://shop.example/p?id=7&utm_source=news&utm_medium=mail#top")
                .as_deref(),
            Some("https://shop.example/p?id=7#top")
        );
        assert_eq!(
            clean_link("https://shop.example/?fbclid=abc").as_deref(),
            Some("https://shop.example/")
        );
        assert_eq!(
            clean_link("HTTPS://shop.example/?a=1&GCLID=x&b=2&mc_eid=y&_hsenc=z&oly_enc_id=q")
                .as_deref(),
            Some("HTTPS://shop.example/?a=1&b=2")
        );
        assert_eq!(
            clean_link("https://shop.example/?utm%5Fsource=x&k=v").as_deref(),
            Some("https://shop.example/?k=v")
        );
        // Nothing to remove, or not an http(s) link: unchanged.
        for h in [
            "https://shop.example/p?id=7",
            "https://shop.example/p",
            "mailto:a@example.com?subject=utm_source",
            "cid:x",
        ] {
            assert_eq!(clean_link(h), None, "{h}");
        }
    }

    #[test]
    fn srcset_candidates_parse_or_refuse_as_a_whole() {
        assert_eq!(
            srcset_candidates("https://a.example/1.png 1x, https://a.example/2.png 2x").unwrap(),
            vec![
                ("https://a.example/1.png".to_string(), "1x".to_string()),
                ("https://a.example/2.png".to_string(), "2x".to_string()),
            ]
        );
        assert_eq!(
            srcset_candidates("https://a.example/a.png").unwrap(),
            vec![("https://a.example/a.png".to_string(), String::new())]
        );
        for v in [
            "",
            ",",
            "https://a.example/a.png 1x 2x",
            "https://a.example/a.png huge",
        ] {
            assert_eq!(srcset_candidates(v), None, "{v:?}");
        }
    }
}
