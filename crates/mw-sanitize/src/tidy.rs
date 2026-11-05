//! Output tidying: drops markup that carries no presentation, on the sanitiser's own
//! serialised output (well-formed by construction, so a small tokenizer suffices).
//!
//! What it never touches: elements with any attribute, `<img>` (the reader hot-patches
//! images by index, so their order and count are fixed), links, tables and the hidden
//! `data-mw-*` markers (those are appended after this pass). What it removes:
//! - attribute-less `<span>`/`<font>` wrappers (unwrapped) and attribute-less empty
//!   `<span>/<font>/<div>/<b>/<i>/<u>/<em>/<strong>` (dropped), unless the message's own
//!   stylesheet names that element (then a selector could depend on it);
//! - a doubled attribute-less `<div><div>…</div></div>` (outer one unwrapped);
//! - runs of three or more `<br>` (kept at two);
//! - runs of ASCII whitespace in text (one space), unless the body has `<pre>`,
//!   `<textarea>`, `<listing>` or any `white-space` styling.
//!
//! Comments and conditional comments never reach this pass (ammonia strips them).

const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];
const UNWRAP: &[&str] = &["span", "font"];
const EMPTY_DROP: &[&str] = &["span", "font", "div", "b", "i", "u", "em", "strong"];

pub(crate) fn is_void(name: &str) -> bool {
    VOID.contains(&name)
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Open,
    Close,
    Void,
    Text,
}

struct Tok<'a> {
    kind: Kind,
    name: &'a str,
    raw: &'a str,
    bare: bool,
    live: bool,
    pair: usize,
}

fn tokenize(s: &str) -> Option<Vec<Tok<'_>>> {
    let bytes = s.as_bytes();
    let mut toks = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            let end = s[i..].find('<').map_or(s.len(), |n| i + n);
            toks.push(Tok { kind: Kind::Text, name: "", raw: &s[i..end], bare: false, live: true, pair: usize::MAX });
            i = end;
            continue;
        }
        // Find the closing `>` outside quoted attribute values.
        let mut j = i + 1;
        let mut quote = 0u8;
        while j < bytes.len() {
            let b = bytes[j];
            if quote != 0 {
                if b == quote {
                    quote = 0;
                }
            } else if b == b'"' || b == b'\'' {
                quote = b;
            } else if b == b'>' {
                break;
            }
            j += 1;
        }
        if j >= bytes.len() {
            return None;
        }
        let raw = &s[i..=j];
        let inner = &raw[1..raw.len() - 1];
        let closing = inner.starts_with('/');
        let body = inner.trim_start_matches('/');
        let name_end = body
            .find(|c: char| c.is_ascii_whitespace() || c == '/')
            .unwrap_or(body.len());
        let name = &body[..name_end];
        if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric()) {
            return None; // not something we emit; leave the document untouched
        }
        let kind = if closing {
            Kind::Close
        } else if VOID.contains(&name) {
            Kind::Void
        } else {
            Kind::Open
        };
        let bare = !closing && body.len() == name.len();
        toks.push(Tok { kind, name, raw, bare, live: true, pair: usize::MAX });
        i = j + 1;
    }
    // Pair opens with closes.
    let mut stack: Vec<usize> = Vec::new();
    for k in 0..toks.len() {
        match toks[k].kind {
            Kind::Open => stack.push(k),
            Kind::Close => {
                let o = stack.pop()?;
                if toks[o].name != toks[k].name {
                    return None;
                }
                toks[o].pair = k;
                toks[k].pair = o;
            }
            _ => {}
        }
    }
    stack.is_empty().then_some(toks)
}

fn next_live(toks: &[Tok<'_>], from: usize) -> Option<usize> {
    (from + 1..toks.len()).find(|&k| toks[k].live)
}
fn prev_live(toks: &[Tok<'_>], from: usize) -> Option<usize> {
    (0..from).rev().find(|&k| toks[k].live)
}

fn collapse_ws(text: &str, out: &mut String) {
    let mut prev_space = false;
    for c in text.chars() {
        if matches!(c, ' ' | '\t' | '\r' | '\n' | '\x0c') {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(c);
            prev_space = false;
        }
    }
}

/// Tidy `body`. `css` is the message's scoped stylesheet (element names it mentions are
/// left alone). Returns the input unchanged if it is not the shape the serialiser emits.
pub(crate) fn tidy(body: &str, css: &str) -> String {
    if body.len() < 16 {
        return body.to_string();
    }
    let Some(mut toks) = tokenize(body) else {
        return body.to_string();
    };
    let css = css.to_ascii_lowercase();
    let styled = |tag: &str| css.contains(tag);

    // Pass 1: empty / redundant wrappers, to a fixed point (bounded).
    for _ in 0..4 {
        let mut changed = false;
        for o in 0..toks.len() {
            if !toks[o].live || toks[o].kind != Kind::Open || !toks[o].bare {
                continue;
            }
            let c = toks[o].pair;
            let name = toks[o].name;
            let empty = next_live(&toks, o) == Some(c);
            if EMPTY_DROP.contains(&name) && empty && !styled(name) {
                toks[o].live = false;
                toks[c].live = false;
                changed = true;
            } else if UNWRAP.contains(&name) && !styled(name) {
                toks[o].live = false;
                toks[c].live = false;
                changed = true;
            } else if name == "div"
                && !styled("div")
                && let Some(n) = next_live(&toks, o)
                && toks[n].kind == Kind::Open
                && toks[n].name == "div"
                && toks[n].bare
                && prev_live(&toks, c) == Some(toks[n].pair)
            {
                toks[o].live = false;
                toks[c].live = false;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // Pass 2: `<br>` runs (whitespace-only text between breaks does not break a run).
    let mut run = 0;
    let mut k = 0;
    while k < toks.len() {
        if !toks[k].live {
            k += 1;
            continue;
        }
        match toks[k].kind {
            Kind::Void if toks[k].name == "br" => {
                run += 1;
                if run > 2 {
                    toks[k].live = false;
                }
            }
            Kind::Text if toks[k].raw.trim().is_empty() && run > 0 => {}
            _ => run = 0,
        }
        k += 1;
    }

    let keep_ws = ["<pre", "<textarea", "<listing", "white-space"]
        .iter()
        .any(|needle| body.contains(needle));
    let mut out = String::with_capacity(body.len());
    for t in &toks {
        if !t.live {
            continue;
        }
        if t.kind == Kind::Text && !keep_ws {
            collapse_ws(t.raw, &mut out);
        } else {
            out.push_str(t.raw);
        }
    }
    out
}
