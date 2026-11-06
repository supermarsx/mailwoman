//! Server-side symbolication of browser stack frames with source map v3.
//!
//! The web build can ship hidden source maps (`MW_SOURCEMAP=hidden`, see
//! `apps/web/vite.config.ts`). They are part of the embedded (or `MW_WEB_DIR`)
//! asset set, but the static handler never serves a `.map` file: only this module
//! reads them, for the admin console. A frame names a bundled file such as
//! `/assets/index-Ab12.js`; its map is `assets/index-Ab12.js.map` next to it, so
//! the right map is always the one of the exact file that threw (the file name
//! carries the build's content hash).
//!
//! No crate is added: a v3 map is JSON plus a base64-VLQ `mappings` string, which
//! [`Map::lookup`] decodes in one pass over only what the query needs.
use serde::Deserialize;
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

/// A parsed source map (names, sources and the raw mappings; segments are decoded
/// per query, not stored).
#[derive(Debug)]
pub(crate) struct Map {
    sources: Vec<String>,
    names: Vec<String>,
    mappings: String,
}

#[derive(Deserialize)]
struct Raw {
    #[serde(default)]
    sources: Vec<Option<String>>,
    #[serde(default)]
    names: Vec<Option<String>>,
    #[serde(default)]
    mappings: String,
    #[serde(default)]
    version: u32,
}

/// An original position (1-based line and column, as a developer reads them).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Original {
    pub source: String,
    pub line: u32,
    pub col: u32,
    pub name: Option<String>,
}

/// Parse a source map document. Index maps (`sections`) are not supported.
pub(crate) fn parse(bytes: &[u8]) -> Option<Map> {
    let raw: Raw = serde_json::from_slice(bytes).ok()?;
    if raw.version != 3 || raw.mappings.is_empty() {
        return None;
    }
    Some(Map {
        sources: raw.sources.into_iter().map(|s| normalise_source(&s.unwrap_or_default())).collect(),
        names: raw.names.into_iter().map(Option::unwrap_or_default).collect(),
        mappings: raw.mappings,
    })
}

/// `../../src/a.ts` and `webpack:///src/a.ts` both read `src/a.ts`; the build
/// machine's absolute prefix never reaches a stored trace.
fn normalise_source(s: &str) -> String {
    let s = s.split_once("://").map_or(s, |(_, rest)| rest);
    let s = s.trim_start_matches('/');
    let mut rest = s;
    while let Some(r) = rest.strip_prefix("../").or_else(|| rest.strip_prefix("./")) {
        rest = r;
    }
    rest.to_owned()
}

fn vlq_value(c: u8) -> Option<i64> {
    Some(match c {
        b'A'..=b'Z' => i64::from(c - b'A'),
        b'a'..=b'z' => i64::from(c - b'a') + 26,
        b'0'..=b'9' => i64::from(c - b'0') + 52,
        b'+' => 62,
        b'/' => 63,
        _ => return None,
    })
}

/// Decode one segment's fields (1, 4 or 5 of them) starting at `*i`.
fn segment(bytes: &[u8], i: &mut usize, fields: &mut [i64; 5]) -> Option<usize> {
    let mut n = 0;
    while *i < bytes.len() && bytes[*i] != b',' && bytes[*i] != b';' {
        let (mut value, mut shift) = (0i64, 0u32);
        loop {
            let digit = vlq_value(*bytes.get(*i)?)?;
            *i += 1;
            value += (digit & 31) << shift;
            shift += 5;
            if digit & 32 == 0 {
                break;
            }
            if shift > 60 {
                return None;
            }
        }
        let signed = if value & 1 == 1 { -(value >> 1) } else { value >> 1 };
        if n >= 5 {
            return None;
        }
        fields[n] = signed;
        n += 1;
    }
    Some(n)
}

impl Map {
    /// Original positions for generated `(line, column)` pairs, both 0-based. An
    /// entry is `None` when no mapped segment starts at or before the column.
    pub(crate) fn lookup(&self, queries: &[(u32, u32)]) -> Vec<Option<Original>> {
        let mut out = vec![None; queries.len()];
        let mut by_line: HashMap<u32, Vec<usize>> = HashMap::new();
        for (idx, (line, _)) in queries.iter().enumerate() {
            by_line.entry(*line).or_default().push(idx);
        }
        let last = by_line.keys().copied().max().unwrap_or(0);
        let bytes = self.mappings.as_bytes();
        let (mut source, mut src_line, mut src_col, mut name) = (0i64, 0i64, 0i64, 0i64);
        let (mut i, mut line) = (0usize, 0u32);
        // The best segment so far for each query on the current line.
        let mut best: HashMap<usize, (i64, i64, i64, Option<i64>)> = HashMap::new();
        while i <= bytes.len() && line <= last {
            let mut gen_col = 0i64;
            // One generated line: segments until ';' or the end.
            loop {
                if i >= bytes.len() || bytes[i] == b';' {
                    break;
                }
                if bytes[i] == b',' {
                    i += 1;
                    continue;
                }
                let mut f = [0i64; 5];
                let Some(n) = segment(bytes, &mut i, &mut f) else {
                    return out;
                };
                gen_col += f[0];
                if n < 4 {
                    continue;
                }
                source += f[1];
                src_line += f[2];
                src_col += f[3];
                let mapped_name = if n >= 5 {
                    name += f[4];
                    Some(name)
                } else {
                    None
                };
                if let Some(wanted) = by_line.get(&line) {
                    for idx in wanted {
                        if i64::from(queries[*idx].1) >= gen_col {
                            best.insert(*idx, (source, src_line, src_col, mapped_name));
                        }
                    }
                }
            }
            if let Some(wanted) = by_line.get(&line) {
                for idx in wanted {
                    if let Some((s, l, c, n)) = best.remove(idx)
                        && let Some(src) = usize::try_from(s).ok().and_then(|s| self.sources.get(s))
                    {
                        out[*idx] = Some(Original {
                            source: src.clone(),
                            line: u32::try_from(l + 1).unwrap_or(0),
                            col: u32::try_from(c + 1).unwrap_or(0),
                            name: n
                                .and_then(|n| usize::try_from(n).ok())
                                .and_then(|n| self.names.get(n))
                                .filter(|n| !n.is_empty())
                                .cloned(),
                        });
                    }
                }
            }
            if i >= bytes.len() {
                break;
            }
            i += 1; // the ';'
            line += 1;
        }
        out
    }
}

/// One parsed stack line.
struct Frame<'a> {
    /// The text before the location (`    at fn (` or `fn@`).
    head: &'a str,
    function: &'a str,
    file: &'a str,
    line: u32,
    col: u32,
    /// Whether the location was wrapped in parentheses.
    paren: bool,
}

fn parse_frame(text: &str) -> Option<Frame<'_>> {
    let trimmed = text.trim_end();
    let (body, paren) = match trimmed.strip_suffix(')') {
        Some(b) => (b, true),
        None => (trimmed, false),
    };
    let (rest, col) = body.rsplit_once(':')?;
    let (rest, line) = rest.rsplit_once(':')?;
    let (col, line) = (col.parse::<u32>().ok()?, line.parse::<u32>().ok()?);
    // The file starts after the last '(' or '@' or space.
    let start = rest.rfind(['(', '@', ' ']).map_or(0, |p| p + 1);
    let (head, file) = rest.split_at(start);
    let function = if let Some(h) = head.strip_suffix('@') {
        h.trim()
    } else {
        let h = head.trim().trim_end_matches('(').trim();
        match h.strip_prefix("at") {
            Some(rest) if rest.is_empty() || rest.starts_with(' ') => rest.trim(),
            _ => h,
        }
    };
    Some(Frame { head, function, file, line, col, paren })
}

/// The `assets/<file>.js` path of a frame's file, if it is a plain bundled script.
pub(crate) fn asset_of(file: &str) -> Option<String> {
    let path = file.split(['?', '#']).next().unwrap_or(file);
    let idx = path.rfind("/assets/")?;
    let rel = &path[idx + 1..];
    let name = rel.strip_prefix("assets/")?;
    let ok = !name.is_empty()
        && name.len() <= 160
        && !name.contains('/')
        && (name.ends_with(".js") || name.ends_with(".mjs"))
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'));
    ok.then(|| rel.to_owned())
}

/// A small cache of parsed maps, `None` for a file known to have no map.
#[derive(Default)]
pub(crate) struct MapCache {
    entries: Mutex<VecDeque<(String, Option<Arc<Map>>)>>,
}

const CACHE_SLOTS: usize = 6;

impl MapCache {
    pub(crate) fn get_or_load(
        &self,
        asset: &str,
        load: impl FnOnce(&str) -> Option<Vec<u8>>,
    ) -> Option<Arc<Map>> {
        {
            let guard = self.entries.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some((_, map)) = guard.iter().find(|(k, _)| k == asset) {
                return map.clone();
            }
        }
        let map = load(&format!("{asset}.map")).and_then(|b| parse(&b)).map(Arc::new);
        let mut guard = self.entries.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if guard.len() >= CACHE_SLOTS {
            guard.pop_front();
        }
        guard.push_back((asset.to_owned(), map.clone()));
        map
    }
}

/// Rewrite the frames of `stack` whose file has a map. Returns the text and how
/// many frames were resolved; `None` when nothing was.
pub(crate) fn symbolicate(
    stack: &str,
    cache: &MapCache,
    load: impl Fn(&str) -> Option<Vec<u8>>,
) -> Option<(String, usize)> {
    let mut resolved = 0usize;
    let mut out = Vec::new();
    for text in stack.lines() {
        let Some(frame) = parse_frame(text) else {
            out.push(text.to_owned());
            continue;
        };
        let original = asset_of(frame.file)
            .and_then(|asset| cache.get_or_load(&asset, &load))
            .and_then(|map| {
                map.lookup(&[(frame.line.saturating_sub(1), frame.col.saturating_sub(1))])
                    .into_iter()
                    .next()
                    .flatten()
            });
        match original {
            Some(o) => {
                resolved += 1;
                let function = o.name.as_deref().unwrap_or(frame.function);
                let location = format!("{}:{}:{}", o.source, o.line, o.col);
                let head = frame.head.trim_end_matches(['(', '@', ' ']);
                let indent: String = head.chars().take_while(|c| c.is_whitespace()).collect();
                if function.is_empty() {
                    out.push(format!("{indent}at {location}"));
                } else if frame.paren || head.contains("at ") || !frame.head.ends_with('@') {
                    out.push(format!("{indent}at {function} ({location})"));
                } else {
                    out.push(format!("{indent}{function}@{location}"));
                }
            }
            None => out.push(text.to_owned()),
        }
    }
    (resolved > 0).then(|| (out.join("\n"), resolved))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `a.js` line 0: col 0 -> src0 1:1 `main`, col 10 -> src0 2:5 (no name);
    /// line 1: col 4 -> src1 10:1.
    /// Mappings: "AAAAA,UACI;IAUA" with relative fields.
    const MAP: &str = r#"{"version":3,"file":"index-Ab12.js","sources":["../../src/app.ts","../../src/util.ts"],"names":["main"],"mappings":"AAAAA,UACI;IAEAA"}"#;

    #[test]
    fn vlq_and_lookup() {
        let m = parse(MAP.as_bytes()).unwrap();
        let r = m.lookup(&[(0, 0), (0, 12), (1, 4), (5, 0)]);
        assert_eq!(r[0], Some(Original { source: "src/app.ts".into(), line: 1, col: 1, name: Some("main".into()) }));
        let second = r[1].clone().unwrap();
        assert_eq!((second.source.as_str(), second.line, second.col), ("src/app.ts", 2, 5));
        assert!(r[2].is_some());
        assert_eq!(r[3], None);
    }

    #[test]
    fn rewrites_chrome_and_firefox_frames_and_keeps_others() {
        let cache = MapCache::default();
        let load = |p: &str| (p == "assets/index-Ab12.js.map").then(|| MAP.as_bytes().to_vec());
        let stack = "TypeError: x is undefined\n    at h (/assets/index-Ab12.js:1:1)\n    at /assets/index-Ab12.js:1:13\nrun@/assets/index-Ab12.js:2:5\n    at z (/assets/vendor-Zz.js:3:4)";
        let (text, n) = symbolicate(stack, &cache, load).unwrap();
        assert_eq!(n, 3);
        assert!(text.contains("at main (src/app.ts:1:1)"), "{text}");
        assert!(text.contains("at src/app.ts:2:5"), "{text}");
        assert!(text.contains("src/util.ts:") || text.contains("@src/"), "{text}");
        assert!(text.contains("at z (/assets/vendor-Zz.js:3:4)"), "{text}");
        assert!(text.starts_with("TypeError: x is undefined"));
    }

    #[test]
    fn refuses_odd_asset_names_and_missing_maps() {
        assert_eq!(asset_of("/assets/../etc/passwd.js"), None);
        assert_eq!(asset_of("/assets/a/b.js"), None);
        assert_eq!(asset_of("https://h/assets/index-1.js?x=1").as_deref(), Some("assets/index-1.js"));
        let cache = MapCache::default();
        assert!(symbolicate("at f (/assets/none.js:1:1)", &cache, |_| None).is_none());
        assert!(parse(b"{\"version\":2}").is_none());
    }
}
