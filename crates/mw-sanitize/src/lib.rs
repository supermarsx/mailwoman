#![forbid(unsafe_code)]
//! HTML email sanitizer per SPEC §7.2.
//!
//! Pipeline:
//! - Real HTML5 parsing via ammonia/html5ever — never regex.
//! - `<script>`, `<object>`, `<embed>`, `<form>`, `<iframe>`, `<svg>`, `<math>`
//!   removed (content of `<script>`/`<style>` dropped from the body flow).
//! - All event-handler attributes stripped (they are not on the allowlist).
//! - URL schemes restricted to http/https/mailto/cid; `javascript:` and
//!   `data:` URLs are neutralized by the scheme allowlist.
//! - Remote images are OFF by default: any `<img src>` that is not a `cid:`
//!   reference has its `src` removed (SPEC §7.2 remote-content policy).
//! - When a remote image is stripped, its host is recorded and a hidden block
//!   marker (`data-mw-blocked-host`, plus `data-mw-tracker` when the host is a
//!   known tracker) is appended to the body (t16 S9). The strip default is
//!   unchanged — the marker only *reports* what was already blocked, so the web
//!   reader can surface "N trackers blocked" without an extra round-trip. The
//!   marker never carries a loadable URL.
//!
//! CSS rewrite (SPEC §7.2 item 3), instead of stripping CSS wholesale:
//! - Inline `style="…"` and `<style>…</style>` blocks are parsed with a real
//!   CSS parser (`cssparser`), never regex.
//! - Declarations are filtered against a property allowlist; unknown/vendor
//!   properties are dropped.
//! - `position:fixed` / `position:sticky` are dropped (overlay/clickjacking).
//! - `@import` (and every at-rule other than `@media`/`@supports`) is dropped.
//! - External `url(…)` is dropped; only internal `cid:` references survive.
//! - `expression(…)` / `javascript:` values are dropped.
//! - `z-index` is clamped to [`MAX_Z_INDEX`].
//! - `<style>` selectors are namespaced under the message container
//!   ([`CONTAINER_CLASS`]) so a message's stylesheet cannot leak out of the
//!   rendered body; the sanitized output is wrapped in a
//!   `<div class="mw-email-body">` when it carries a scoped stylesheet.

use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use cssparser::{
    AtRuleParser, BasicParseErrorKind, CowRcStr, DeclarationParser, ParseError, Parser,
    ParserInput, ParserState, QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser,
    StyleSheetParser, ToCss, Token,
};

// The wasm-bindgen surface (plan §1.3): sanitize decrypted E2EE HTML in the browser
// crypto worker, never on the server. Gated on the wasm32 target so the native build
// never links wasm-bindgen and the engine consumers (mw-render/mw-export/mw-server)
// stay unchanged; the sanitize policy below is target-agnostic (ammonia + cssparser
// are pure-Rust + wasm-compatible). e8b builds it to wasm via `scripts/build-wasm.*`
// into `apps/web/src/wasm/mw-sanitize`.
#[cfg(target_arch = "wasm32")]
mod wasm;

/// Class applied to the wrapper `<div>` that scopes a message's stylesheet.
/// `<style>` selectors are rewritten to sit under `.mw-email-body …`.
pub const CONTAINER_CLASS: &str = "mw-email-body";

/// Upper bound enforced on any `z-index` declaration. The rendered body already
/// lives in a sandboxed, opaque-origin iframe, so this is defense-in-depth
/// against overlay tricks within the message itself.
pub const MAX_Z_INDEX: i64 = 1000;

/// A remote resource the sanitizer stripped from a message body: the host it would
/// have loaded from, and whether that host was classified as a tracker. Surfaced to
/// the body as a hidden block marker so the web reader can report it (t16 S9).
#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockedResource {
    host: String,
    tracker: bool,
}

/// Sanitize untrusted HTML email content. Always returns owned, safe HTML.
pub fn sanitize_email_html(input: &str) -> String {
    // S9 side-channel: the attribute filter runs inside ammonia's single parse and
    // cannot ADD attributes to an element, so it records each stripped remote
    // `<img src>` here. After cleaning we emit one hidden marker per blocked
    // resource. `Arc<Mutex<…>>` because ammonia requires the filter be
    // `Send + Sync + 'static`; there is no cross-thread contention (the clean runs
    // synchronously on this thread).
    let blocked: Arc<Mutex<Vec<BlockedResource>>> = Arc::new(Mutex::new(Vec::new()));
    let blocked_sink = Arc::clone(&blocked);

    let mut builder = ammonia::Builder::default();

    // `<script>` content is dropped entirely (tag + text). `<style>` is dropped
    // from the BODY flow here; its CSS is extracted and re-scoped separately
    // below so selectors can be namespaced under the message container.
    let mut clean_content: HashSet<&str> = HashSet::new();
    clean_content.insert("script");
    clean_content.insert("style");
    builder.clean_content_tags(clean_content);

    // Restrict URL schemes (kills javascript: and data:text/html).
    let mut schemes: HashSet<&str> = HashSet::new();
    schemes.insert("http");
    schemes.insert("https");
    schemes.insert("mailto");
    schemes.insert("cid");
    builder.url_schemes(schemes);

    // Deny relative URLs — emails have no trusted base.
    builder.url_relative(ammonia::UrlRelative::Deny);

    // Allow the presentational attributes CSS needs to target: `style` (inline,
    // sanitized in the filter below), plus `class`/`id` so namespaced `<style>`
    // selectors can match. All are inert inside the sandboxed body iframe.
    let mut generic: HashSet<&str> = HashSet::new();
    generic.insert("lang");
    generic.insert("title");
    generic.insert("style");
    generic.insert("class");
    generic.insert("id");
    builder.generic_attributes(generic);

    // One attribute filter: strip remote (non-cid) img@src, and rewrite inline
    // `style` through the CSS declaration sanitizer.
    builder.attribute_filter(move |element, attribute, value| -> Option<Cow<str>> {
        if element == "img" && attribute == "src" && !value.starts_with("cid:") {
            // Remote image: stripped as before (strip default UNCHANGED). Record its
            // host + tracker classification for the block-marker report (S9).
            if let Some(host) = url_host(value) {
                let tracker = is_tracker_host(&host);
                blocked_sink
                    .lock()
                    .expect("sanitize blocked-resource lock")
                    .push(BlockedResource { host, tracker });
            }
            return None;
        }
        if attribute == "style" {
            let sanitized = sanitize_declaration_list(value);
            if sanitized.is_empty() {
                return None;
            }
            return Some(Cow::Owned(sanitized));
        }
        Some(Cow::Borrowed(value))
    });

    // Links open nowhere implicitly; add rel hardening.
    builder.link_rel(Some("noopener noreferrer nofollow"));

    let body = builder.clean(input).to_string();

    // Extract every `<style>` block from the original input, allowlist + scope
    // its CSS under the container, and prepend the result inside the wrapper.
    let scoped_css = extract_and_scope_styles(input);

    // S9: emit one hidden block marker per stripped remote image, carrying the host
    // (and a tracker flag) the web reader reports. Inert — a `<span hidden>` with no
    // loadable URL.
    let blocked = blocked.lock().expect("sanitize blocked-resource lock");
    let mut markers = String::new();
    for r in blocked.iter() {
        markers.push_str("<span hidden data-mw-blocked-host=\"");
        markers.push_str(&escape_attr(&r.host));
        markers.push('"');
        if r.tracker {
            markers.push_str(" data-mw-tracker");
        }
        markers.push_str("></span>");
    }

    if scoped_css.is_empty() && markers.is_empty() {
        body
    } else {
        let style = if scoped_css.is_empty() {
            String::new()
        } else {
            format!("<style>{scoped_css}</style>")
        };
        format!("<div class=\"{CONTAINER_CLASS}\">{style}{body}{markers}</div>")
    }
}

/// Extract the lower-cased host from an absolute `http`/`https` URL (the only
/// remote schemes ammonia lets through to a stripped `<img src>`). Strips any
/// `user:pass@` userinfo and `:port`. Returns `None` for a URL with no host.
fn url_host(url: &str) -> Option<String> {
    // Drop the scheme (`http://` / `https://`); `//host…` protocol-relative refs are
    // denied upstream by `url_relative(Deny)`, so an absolute scheme is present.
    let after_scheme = url.split_once("://").map(|(_, rest)| rest)?;
    // Authority ends at the first `/`, `?`, or `#`.
    let authority = after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(after_scheme);
    // Drop any userinfo before an `@`.
    let host_port = authority.rsplit_once('@').map_or(authority, |(_, hp)| hp);
    // Drop a `:port` suffix. An IPv6 literal is bracketed (`[::1]`) — keep the
    // brackets' contents; there is no bare `:port` to trim inside them.
    let host = if let Some(stripped) = host_port.strip_prefix('[') {
        stripped.split(']').next().unwrap_or(stripped)
    } else {
        host_port.split(':').next().unwrap_or(host_port)
    };
    let host = host.trim().to_ascii_lowercase();
    (!host.is_empty()).then_some(host)
}

/// Best-effort classification of a remote-image host as an email tracker (open/read
/// pixel), used only to set the OPTIONAL `data-mw-tracker` marker. The honest
/// primary signal is the blocked COUNT (every stripped remote image); this subset
/// flag is heuristic, matching a curated set of known bulk-mail/analytics tracking
/// domains plus a small set of conventional tracking sub-domain labels. It never
/// gates loading — an image is blocked (or granted) regardless of this flag.
fn is_tracker_host(host: &str) -> bool {
    // Known tracker / bulk-mail-analytics domains (matched as a domain suffix so
    // `x.list-manage.com` counts). Not exhaustive; a reporting aid, not a blocklist.
    const TRACKER_SUFFIXES: &[&str] = &[
        "list-manage.com",
        "mailchimp.com",
        "sendgrid.net",
        "sendgrid.com",
        "mailgun.org",
        "mandrillapp.com",
        "sparkpostmail.com",
        "sendinblue.com",
        "sibautomation.com",
        "hubspot.com",
        "hubspotemail.net",
        "doubleclick.net",
        "google-analytics.com",
        "googletagmanager.com",
        "mixpanel.com",
        "segment.com",
        "branch.io",
        "mailtrack.io",
        "bananatag.com",
        "yesware.com",
        "streak.com",
        "getnotify.com",
        "convertkit-mail.com",
        "constantcontact.com",
        "rs6.net",
        "exct.net",
        "mktoresp.com",
        "pardot.com",
        "cmail19.com",
        "createsend.com",
    ];
    if TRACKER_SUFFIXES
        .iter()
        .any(|s| host == *s || host.ends_with(&format!(".{s}")))
    {
        return true;
    }
    // Conventional tracking sub-domain labels (`track.`, `click.`, `pixel.`, …).
    const TRACKER_LABELS: &[&str] = &[
        "track", "tracking", "trk", "click", "clicks", "pixel", "beacon", "open", "opens", "email",
        "mailstat", "mtrack",
    ];
    let first_label = host.split('.').next().unwrap_or(host);
    TRACKER_LABELS.contains(&first_label)
}

/// Minimal HTML attribute-value escaping for the block-marker host (already a
/// parsed, lower-cased host, but escaped defensively so a hostile label cannot break
/// out of the double-quoted attribute).
fn escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// Parse the input, isolate its `<style>` blocks, and return one scoped, filtered
/// stylesheet (or an empty string when there are none).
fn extract_and_scope_styles(input: &str) -> String {
    // Cheap guard: no `<style` substring means no stylesheet to process.
    if !contains_ignore_ascii_case(input, "<style") {
        return String::new();
    }

    // Re-run the real HTML5 parser (ammonia) keeping ONLY `<style>` elements and
    // their raw text. Everything else (including `<script>`) is dropped, so the
    // result is a well-formed string whose only tags are `<style>…</style>` with
    // verbatim (raw-text, unescaped) CSS — safe to slice by tag boundary.
    let isolated = ammonia::Builder::empty()
        .add_tags(&["style"])
        .rm_clean_content_tags(&["style"])
        .clean(input)
        .to_string();

    let mut out = String::new();
    let mut rest = isolated.as_str();
    while let Some(open) = rest.find("<style") {
        let after_open = &rest[open..];
        let Some(gt) = after_open.find('>') else {
            break;
        };
        let content_start = open + gt + 1;
        let tail = &rest[content_start..];
        // Raw-text parsing guarantees `</style` cannot appear inside the CSS.
        let Some(close) = tail.find("</style") else {
            break;
        };
        let css = &tail[..close];
        out.push_str(&sanitize_stylesheet(css, CONTAINER_CLASS));
        let after_close = &tail[close..];
        let Some(cgt) = after_close.find('>') else {
            break;
        };
        rest = &tail[close + cgt + 1..];
    }
    out
}

fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    let needle = needle.as_bytes();
    if needle.is_empty() {
        return true;
    }
    haystack
        .as_bytes()
        .windows(needle.len())
        .any(|w| w.eq_ignore_ascii_case(needle))
}

// ── CSS property allowlist ───────────────────────────────────────────────────

/// Visual/layout CSS properties permitted in email. `position` is allowed so
/// `relative`/`absolute` survive; the `fixed`/`sticky` VALUES are rejected in
/// [`check_and_finish`]. `z-index` is allowed but clamped.
fn is_allowed_property(name: &str) -> bool {
    matches!(
        name,
        // color / background
        "color"
            | "opacity"
            | "background"
            | "background-color"
            | "background-image"
            | "background-position"
            | "background-repeat"
            | "background-size"
            | "background-clip"
            | "background-origin"
            | "background-attachment"
            // font / text
            | "font"
            | "font-family"
            | "font-size"
            | "font-style"
            | "font-weight"
            | "font-variant"
            | "font-stretch"
            | "line-height"
            | "letter-spacing"
            | "word-spacing"
            | "text-align"
            | "text-align-last"
            | "text-decoration"
            | "text-decoration-color"
            | "text-decoration-line"
            | "text-decoration-style"
            | "text-transform"
            | "text-indent"
            | "text-shadow"
            | "text-overflow"
            | "white-space"
            | "word-break"
            | "word-wrap"
            | "overflow-wrap"
            | "direction"
            | "unicode-bidi"
            | "vertical-align"
            | "tab-size"
            | "quotes"
            | "content"
            // list
            | "list-style"
            | "list-style-type"
            | "list-style-position"
            | "list-style-image"
            // box model
            | "margin"
            | "margin-top"
            | "margin-right"
            | "margin-bottom"
            | "margin-left"
            | "padding"
            | "padding-top"
            | "padding-right"
            | "padding-bottom"
            | "padding-left"
            | "border"
            | "border-width"
            | "border-style"
            | "border-color"
            | "border-top"
            | "border-right"
            | "border-bottom"
            | "border-left"
            | "border-top-width"
            | "border-right-width"
            | "border-bottom-width"
            | "border-left-width"
            | "border-top-style"
            | "border-right-style"
            | "border-bottom-style"
            | "border-left-style"
            | "border-top-color"
            | "border-right-color"
            | "border-bottom-color"
            | "border-left-color"
            | "border-radius"
            | "border-top-left-radius"
            | "border-top-right-radius"
            | "border-bottom-left-radius"
            | "border-bottom-right-radius"
            | "border-collapse"
            | "border-spacing"
            | "box-sizing"
            | "box-shadow"
            | "outline"
            | "outline-color"
            | "outline-style"
            | "outline-width"
            | "outline-offset"
            // sizing / layout
            | "width"
            | "min-width"
            | "max-width"
            | "height"
            | "min-height"
            | "max-height"
            | "display"
            | "visibility"
            | "overflow"
            | "overflow-x"
            | "overflow-y"
            | "float"
            | "clear"
            | "clip"
            | "object-fit"
            | "object-position"
            | "aspect-ratio"
            // positioning (values gated separately)
            | "position"
            | "top"
            | "right"
            | "bottom"
            | "left"
            | "z-index"
            // table
            | "caption-side"
            | "empty-cells"
            | "table-layout"
            // flex / grid
            | "flex"
            | "flex-direction"
            | "flex-wrap"
            | "flex-flow"
            | "flex-grow"
            | "flex-shrink"
            | "flex-basis"
            | "justify-content"
            | "justify-items"
            | "justify-self"
            | "align-items"
            | "align-self"
            | "align-content"
            | "order"
            | "gap"
            | "row-gap"
            | "column-gap"
            | "grid"
            | "grid-template"
            | "grid-template-columns"
            | "grid-template-rows"
            | "grid-template-areas"
            | "grid-auto-columns"
            | "grid-auto-rows"
            | "grid-auto-flow"
            | "grid-column"
            | "grid-row"
            | "grid-area"
            | "place-items"
            | "place-content"
            | "place-self"
            // misc visual
            | "cursor"
            | "transform"
            | "transform-origin"
            | "transition"
            | "transition-property"
            | "transition-duration"
            | "transition-timing-function"
            | "transition-delay"
    )
}

// ── declaration list parsing (inline styles + rule bodies) ───────────────────

/// A kept declaration is `Some((property, value))`; a dropped one is `None`.
type MaybeDecl = Option<(String, String)>;

struct DeclSanitizer;

impl<'i> DeclarationParser<'i> for DeclSanitizer {
    type Declaration = MaybeDecl;
    type Error = ();

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        _start: &ParserState,
    ) -> Result<MaybeDecl, ParseError<'i, ()>> {
        let property = name.to_ascii_lowercase();
        if !is_allowed_property(&property) {
            // Consumed to the delimiter by the RuleBodyParser regardless.
            return Ok(None);
        }
        let mut value = String::new();
        if append_value_tokens(input, &mut value).is_err() {
            return Ok(None);
        }
        Ok(check_and_finish(property, value.trim().to_string()))
    }
}

// A declaration list rejects nested qualified/at rules (default impls do so).
impl<'i> AtRuleParser<'i> for DeclSanitizer {
    type Prelude = ();
    type AtRule = MaybeDecl;
    type Error = ();
}
impl<'i> QualifiedRuleParser<'i> for DeclSanitizer {
    type Prelude = ();
    type QualifiedRule = MaybeDecl;
    type Error = ();
}
impl<'i> RuleBodyItemParser<'i, MaybeDecl, ()> for DeclSanitizer {
    fn parse_declarations(&self) -> bool {
        true
    }
    fn parse_qualified(&self) -> bool {
        false
    }
}

/// Apply the value-level policy and produce the final declaration (or drop it).
fn check_and_finish(property: String, value: String) -> MaybeDecl {
    if value.is_empty() {
        return None;
    }
    let low = value.to_ascii_lowercase();
    if low.contains("expression(") || low.contains("javascript:") {
        return None;
    }
    if has_external_url(&low) {
        return None;
    }
    if property == "position" {
        let first = low.split_whitespace().next().unwrap_or("");
        if first == "fixed" || first == "sticky" {
            return None;
        }
    }
    let value = if property == "z-index" {
        clamp_z_index(&value)
    } else {
        value
    };
    Some((property, value))
}

/// True if the (lowercased) value contains a `url(…)` whose target is not a
/// `cid:` reference — i.e. an external/remote/data resource.
fn has_external_url(low: &str) -> bool {
    let mut i = 0;
    while let Some(p) = low[i..].find("url(") {
        let start = i + p + 4;
        let end = low[start..].find(')').map_or(low.len(), |e| start + e);
        let arg = low[start..end]
            .trim()
            .trim_matches(|c| c == '"' || c == '\'')
            .trim();
        if !arg.starts_with("cid:") {
            return true;
        }
        i = end;
    }
    false
}

fn clamp_z_index(value: &str) -> String {
    let mut parts = value.split_whitespace();
    let first = parts.next().unwrap_or("");
    if let Ok(n) = first.parse::<i64>()
        && n > MAX_Z_INDEX
    {
        let rest: Vec<&str> = parts.collect();
        if rest.is_empty() {
            return MAX_Z_INDEX.to_string();
        }
        return format!("{} {}", MAX_Z_INDEX, rest.join(" "));
    }
    value.to_string()
}

/// Serialize a declaration value token-by-token (recursing into function and
/// bracket blocks), inserting minimal whitespace. Mirrors the value-reconstruct
/// approach ammonia uses for its own inline-style filter.
fn append_value_tokens<'i>(
    input: &mut Parser<'i, '_>,
    value: &mut String,
) -> Result<(), ParseError<'i, ()>> {
    let mut first = true;
    loop {
        let token = match input.next() {
            Ok(t) => t,
            Err(e) if matches!(e.kind, BasicParseErrorKind::EndOfInput) => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        if matches!(token, Token::BadString(_) | Token::BadUrl(_)) {
            return Err(input.new_error(BasicParseErrorKind::EndOfInput));
        }
        let is_comma = matches!(token, Token::Comma);
        let closer = match token {
            Token::Function(_) | Token::ParenthesisBlock => Some(")"),
            Token::SquareBracketBlock => Some("]"),
            Token::CurlyBracketBlock => Some("}"),
            _ => None,
        };
        if !first && !is_comma {
            value.push(' ');
        }
        first = false;
        if token.to_css(value).is_err() {
            return Err(input.new_error(BasicParseErrorKind::EndOfInput));
        }
        if let Some(close) = closer {
            input.parse_nested_block(|nested| append_value_tokens(nested, value))?;
            value.push_str(close);
        }
    }
}

/// Sanitize a bare declaration list (an inline `style="…"` value).
fn sanitize_declaration_list(src: &str) -> String {
    let mut input = ParserInput::new(src);
    let mut parser = Parser::new(&mut input);
    let mut out = String::new();
    collect_declarations(&mut parser, &mut out);
    out
}

/// Drive a [`RuleBodyParser`] over `parser`, appending kept declarations as
/// `prop:value` separated by `;`.
fn collect_declarations(parser: &mut Parser, out: &mut String) {
    let mut sink = DeclSanitizer;
    let iter = RuleBodyParser::new(parser, &mut sink);
    for item in iter {
        if let Ok(Some((prop, val))) = item {
            if !out.is_empty() {
                out.push(';');
            }
            out.push_str(&prop);
            out.push(':');
            out.push_str(&val);
        }
    }
}

// ── stylesheet parsing (`<style>` blocks) ────────────────────────────────────

struct StylesheetSanitizer<'a> {
    container: &'a str,
}

impl<'i> QualifiedRuleParser<'i> for StylesheetSanitizer<'_> {
    type Prelude = String;
    type QualifiedRule = String;
    type Error = ();

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<String, ParseError<'i, ()>> {
        let start = input.position();
        while input.next().is_ok() {}
        Ok(input.slice_from(start).to_string())
    }

    fn parse_block<'t>(
        &mut self,
        prelude: String,
        _start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<String, ParseError<'i, ()>> {
        let selectors = namespace_selectors(&prelude, self.container);
        if selectors.is_empty() {
            return Ok(String::new());
        }
        let mut decls = String::new();
        collect_declarations(input, &mut decls);
        if decls.is_empty() {
            return Ok(String::new());
        }
        Ok(format!("{selectors}{{{decls}}}"))
    }
}

impl<'i> AtRuleParser<'i> for StylesheetSanitizer<'_> {
    // (at-rule name, raw prelude text)
    type Prelude = (String, String);
    type AtRule = String;
    type Error = ();

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<(String, String), ParseError<'i, ()>> {
        let lname = name.to_ascii_lowercase();
        // Only conditional group rules are kept (their bodies get namespaced).
        // Everything else — @import, @charset, @font-face, @keyframes, @page,
        // @namespace, … — is dropped by returning an error.
        if lname == "media" || lname == "supports" {
            let start = input.position();
            while input.next().is_ok() {}
            let raw = input.slice_from(start).to_string();
            Ok((lname, raw))
        } else {
            Err(input.new_error(BasicParseErrorKind::AtRuleInvalid(name)))
        }
    }

    fn parse_block<'t>(
        &mut self,
        prelude: (String, String),
        _start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<String, ParseError<'i, ()>> {
        let inner = namespace_rules(input, self.container);
        if inner.is_empty() {
            return Ok(String::new());
        }
        let (name, raw) = prelude;
        let cond = raw.trim();
        if cond.is_empty() {
            Ok(format!("@{name}{{{inner}}}"))
        } else {
            Ok(format!("@{name} {cond}{{{inner}}}"))
        }
    }
}

/// Parse a rule list from `parser`, namespacing every qualified rule's selectors
/// under `container` and recursing into `@media`/`@supports`.
fn namespace_rules(parser: &mut Parser, container: &str) -> String {
    let mut sink = StylesheetSanitizer { container };
    let mut out = String::new();
    let iter = StyleSheetParser::new(parser, &mut sink);
    for result in iter {
        if let Ok(rule) = result
            && !rule.is_empty()
        {
            out.push_str(&rule);
        }
    }
    out
}

fn sanitize_stylesheet(src: &str, container: &str) -> String {
    let mut input = ParserInput::new(src);
    let mut parser = Parser::new(&mut input);
    namespace_rules(&mut parser, container)
}

/// Rewrite a (possibly comma-separated) selector list so every selector is
/// scoped under `.{container}`. A leading `html`/`body`/`:root` maps onto the
/// container element itself; everything else becomes a descendant.
fn namespace_selectors(raw: &str, container: &str) -> String {
    let mut out = String::new();
    for sel in split_top_level_commas(raw) {
        let s = sel.trim();
        if s.is_empty() {
            continue;
        }
        let scoped = namespace_one(s, container);
        if !out.is_empty() {
            out.push(',');
        }
        out.push_str(&scoped);
    }
    out
}

fn namespace_one(selector: &str, container: &str) -> String {
    let lower = selector.to_ascii_lowercase();
    for kw in ["html", "body", ":root"] {
        if lower.starts_with(kw) {
            let after = &selector[kw.len()..];
            let boundary = after
                .chars()
                .next()
                .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '\\'));
            if boundary {
                return format!(".{container}{after}");
            }
        }
    }
    format!(".{container} {selector}")
}

/// Split on commas that are not nested inside `()`/`[]` or a string.
fn split_top_level_commas(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0usize;
    let mut in_str: Option<char> = None;
    for (i, c) in s.char_indices() {
        match in_str {
            Some(q) => {
                if c == q {
                    in_str = None;
                }
            }
            None => match c {
                '"' | '\'' => in_str = Some(c),
                '(' | '[' => depth += 1,
                ')' | ']' => depth = depth.saturating_sub(1),
                ',' if depth == 0 => {
                    parts.push(&s[start..i]);
                    start = i + 1;
                }
                _ => {}
            },
        }
    }
    parts.push(&s[start..]);
    parts
}

#[cfg(test)]
mod tests {
    use super::sanitize_email_html as clean;
    use super::{CONTAINER_CLASS, MAX_Z_INDEX, sanitize_declaration_list, sanitize_stylesheet};

    #[test]
    fn strips_script_tags_and_content() {
        let out = clean(r#"<p>hi</p><script>window.__pwned=1</script>"#);
        assert!(!out.contains("script"));
        assert!(!out.contains("__pwned"));
        assert!(out.contains("<p>hi</p>"));
    }

    #[test]
    fn strips_event_handlers() {
        let out = clean(r#"<img src="cid:x" onerror="alert(1)"><div onclick="x()">a</div>"#);
        assert!(!out.contains("onerror"));
        assert!(!out.contains("onclick"));
        assert!(!out.contains("alert(1)"));
    }

    #[test]
    fn neutralizes_javascript_urls() {
        let out = clean(r#"<a href="javascript:alert(1)">x</a>"#);
        assert!(!out.contains("javascript:"));
    }

    #[test]
    fn neutralizes_data_html_urls() {
        let out = clean(r#"<a href="data:text/html,<script>1</script>">x</a>"#);
        assert!(!out.contains("data:"));
    }

    #[test]
    fn removes_forms_iframes_objects_embeds() {
        let out = clean(
            r#"<form action="https://evil.example/steal"><input name="pw"></form>
               <iframe src="https://evil.example"></iframe>
               <object data="x"></object><embed src="x">"#,
        );
        for needle in ["<form", "<iframe", "<object", "<embed", "evil.example"] {
            assert!(!out.contains(needle), "found {needle} in {out}");
        }
    }

    #[test]
    fn blocks_remote_images_keeps_cid() {
        let out = clean(r#"<img src="https://tracker.evil/p.gif"><img src="cid:inline1">"#);
        // The remote image is NOT loaded (no remote src survives)...
        assert!(!out.contains(r#"src="https://tracker.evil"#), "{out}");
        assert!(!out.contains("p.gif"), "{out}");
        // ...but the host is surfaced in a hidden, non-loadable block marker (S9).
        assert!(
            out.contains(r#"data-mw-blocked-host="tracker.evil""#),
            "{out}"
        );
        assert!(out.contains("cid:inline1"));
    }

    #[test]
    fn emits_block_marker_per_stripped_remote_image() {
        let out = clean(
            r#"<img src="https://a.example/1.gif"><img src="https://b.example/2.png"><img src="cid:x">"#,
        );
        // One marker per blocked remote image; the cid image is not counted.
        assert_eq!(out.matches("data-mw-blocked-host=").count(), 2, "{out}");
        assert!(out.contains(r#"data-mw-blocked-host="a.example""#), "{out}");
        assert!(out.contains(r#"data-mw-blocked-host="b.example""#), "{out}");
        // Markers are hidden + carry no loadable URL.
        assert!(out.contains("<span hidden data-mw-blocked-host="), "{out}");
    }

    #[test]
    fn marker_host_drops_userinfo_and_port() {
        let out = clean(r#"<img src="https://user:pw@Host.Example:8443/p.gif?u=1">"#);
        assert!(
            out.contains(r#"data-mw-blocked-host="host.example""#),
            "{out}"
        );
        // Credentials never appear in the output.
        assert!(!out.contains("user:pw"), "{out}");
        assert!(!out.contains("8443"), "{out}");
    }

    #[test]
    fn tracker_host_gets_tracker_marker() {
        // A known tracker domain suffix is flagged.
        let out = clean(r#"<img src="https://x.list-manage.com/track/open.php?u=1">"#);
        assert!(out.contains("data-mw-tracker"), "{out}");
        // A conventional `track.` sub-domain label is flagged.
        let out2 = clean(r#"<img src="https://track.shop.example/o.gif">"#);
        assert!(out2.contains("data-mw-tracker"), "{out2}");
        // A plain remote image is blocked but NOT flagged as a tracker.
        let out3 = clean(r#"<img src="https://cdn.shop.example/logo.png">"#);
        assert!(
            out3.contains(r#"data-mw-blocked-host="cdn.shop.example""#),
            "{out3}"
        );
        assert!(!out3.contains("data-mw-tracker"), "{out3}");
    }

    #[test]
    fn no_remote_image_means_no_marker_or_wrapper() {
        let out = clean(r#"<p>hi</p><img src="cid:inline">"#);
        assert!(!out.contains("data-mw-blocked-host"), "{out}");
        assert!(!out.contains(CONTAINER_CLASS), "{out}");
    }

    #[test]
    fn hardens_link_rel() {
        let out = clean(r#"<a href="https://example.org">x</a>"#);
        assert!(out.contains("noopener"));
        assert!(out.contains("noreferrer"));
    }

    #[test]
    fn survives_malformed_soup() {
        let bomb = "<div>".repeat(2000) + "<script>1</script>" + &"</div>".repeat(1999);
        let out = clean(&bomb);
        assert!(!out.contains("script"));
    }

    // ── CSS rewrite (SPEC §7.2 item 3) ───────────────────────────────────────

    #[test]
    fn inline_style_keeps_benign_properties() {
        let out = clean(r#"<p style="color:red;font-weight:bold">hi</p>"#);
        assert!(out.contains("color:red"), "{out}");
        assert!(out.contains("font-weight:bold"), "{out}");
    }

    #[test]
    fn inline_style_drops_unknown_property() {
        // `-moz-binding` (XBL script vector) and other unknowns are dropped.
        let out = clean(r#"<p style="color:red;-moz-binding:url(cid:x)">hi</p>"#);
        assert!(out.contains("color:red"), "{out}");
        assert!(!out.contains("binding"), "{out}");
    }

    #[test]
    fn inline_style_drops_position_fixed_and_sticky() {
        let out = clean(r#"<div style="position:fixed;top:0;color:red">x</div>"#);
        assert!(!out.contains("fixed"), "{out}");
        // benign siblings survive
        assert!(out.contains("color:red"), "{out}");
        let out2 = clean(r#"<div style="position:sticky">x</div>"#);
        assert!(!out2.contains("sticky"), "{out2}");
        // relative/absolute are allowed through
        let out3 = clean(r#"<div style="position:relative">x</div>"#);
        assert!(out3.contains("position:relative"), "{out3}");
    }

    #[test]
    fn inline_style_drops_external_url_keeps_cid() {
        let ext = clean(r#"<div style="background:url(https://evil/x.png)">x</div>"#);
        assert!(!ext.contains("evil"), "{ext}");
        assert!(!ext.contains("url("), "{ext}");
        let cid = clean(r#"<div style="background:url(cid:img1) no-repeat">x</div>"#);
        assert!(cid.contains("cid:img1"), "{cid}");
    }

    #[test]
    fn inline_style_clamps_z_index() {
        let out = clean(r#"<div style="z-index:999999">x</div>"#);
        assert!(
            out.contains(&format!("z-index:{MAX_Z_INDEX}")),
            "expected clamp in {out}"
        );
        assert!(!out.contains("999999"), "{out}");
        // small values are untouched
        let ok = clean(r#"<div style="z-index:5">x</div>"#);
        assert!(ok.contains("z-index:5"), "{ok}");
    }

    #[test]
    fn inline_style_drops_ie_expression() {
        let out = clean(r#"<div style="width:expression(alert(1))">x</div>"#);
        assert!(!out.contains("expression"), "{out}");
        assert!(!out.contains("alert"), "{out}");
    }

    #[test]
    fn style_block_namespaced_and_wrapped() {
        let out = clean(r#"<style>p{color:red}</style><p>hi</p>"#);
        assert!(
            out.contains(&format!(".{CONTAINER_CLASS} p")),
            "expected namespaced selector in {out}"
        );
        assert!(out.contains("color:red"), "{out}");
        assert!(
            out.contains(&format!("class=\"{CONTAINER_CLASS}\"")),
            "expected wrapper in {out}"
        );
        // the class the selector targets survives on the element
        assert!(out.contains("<p>hi</p>"), "{out}");
    }

    #[test]
    fn style_block_scopes_class_and_id_selectors() {
        let out = clean(r#"<style>.a{color:red}#b{color:blue}</style><p class="a" id="b">x</p>"#);
        assert!(out.contains(&format!(".{CONTAINER_CLASS} .a")), "{out}");
        assert!(out.contains(&format!(".{CONTAINER_CLASS} #b")), "{out}");
        // class/id are preserved on the element so the CSS can match
        assert!(out.contains(r#"class="a""#), "{out}");
        assert!(out.contains(r#"id="b""#), "{out}");
    }

    #[test]
    fn style_block_body_maps_to_container() {
        let scoped = sanitize_stylesheet("body{margin:0}", CONTAINER_CLASS);
        assert_eq!(scoped, format!(".{CONTAINER_CLASS}{{margin:0}}"));
        let scoped2 = sanitize_stylesheet("body p{color:red}", CONTAINER_CLASS);
        assert_eq!(scoped2, format!(".{CONTAINER_CLASS} p{{color:red}}"));
    }

    #[test]
    fn style_block_drops_import() {
        let out = clean(r#"<style>@import url(https://evil/x.css);p{color:red}</style><p>x</p>"#);
        assert!(!out.contains("@import"), "{out}");
        assert!(!out.contains("evil"), "{out}");
        assert!(out.contains("color:red"), "{out}");
    }

    #[test]
    fn style_block_drops_positioning_and_external_url() {
        let out = clean(
            r#"<style>.x{position:fixed;top:0}.y{background:url(http://evil/a.png)}</style><p>x</p>"#,
        );
        assert!(!out.contains("fixed"), "{out}");
        assert!(!out.contains("evil"), "{out}");
    }

    #[test]
    fn style_block_clamps_z_index() {
        let out = clean(r#"<style>.x{z-index:2147483647}</style><p>x</p>"#);
        assert!(out.contains(&format!("z-index:{MAX_Z_INDEX}")), "{out}");
        assert!(!out.contains("2147483647"), "{out}");
    }

    #[test]
    fn style_block_child_combinator_survives() {
        // Guards against HTML-escaping of `<style>` raw text corrupting CSS `>`.
        let scoped = sanitize_stylesheet("a > b{color:red}", CONTAINER_CLASS);
        assert!(scoped.contains('>'), "combinator lost: {scoped}");
        assert!(scoped.contains("color:red"), "{scoped}");
    }

    #[test]
    fn media_query_inner_selectors_namespaced() {
        let scoped = sanitize_stylesheet("@media (max-width:600px){p{color:red}}", CONTAINER_CLASS);
        assert!(scoped.starts_with("@media (max-width:600px){"), "{scoped}");
        assert!(
            scoped.contains(&format!(".{CONTAINER_CLASS} p")),
            "{scoped}"
        );
    }

    #[test]
    fn multi_selector_list_each_namespaced() {
        let scoped = sanitize_stylesheet("h1, .foo{color:red}", CONTAINER_CLASS);
        assert!(
            scoped.contains(&format!(".{CONTAINER_CLASS} h1")),
            "{scoped}"
        );
        assert!(
            scoped.contains(&format!(".{CONTAINER_CLASS} .foo")),
            "{scoped}"
        );
    }

    #[test]
    fn no_style_block_means_no_wrapper() {
        let out = clean(r#"<p style="color:red">hi</p>"#);
        assert!(!out.contains(CONTAINER_CLASS), "{out}");
        assert!(out.contains("color:red"), "{out}");
    }

    #[test]
    fn declaration_list_helper_filters() {
        let out = sanitize_declaration_list("color:red; position:fixed; font-size:12px");
        assert!(out.contains("color:red"));
        assert!(out.contains("font-size:12px"));
        assert!(!out.contains("fixed"));
    }

    #[test]
    fn font_shorthand_with_functions_preserved() {
        let out = sanitize_declaration_list("color:rgb(1, 2, 3); width:calc(100% - 10px)");
        assert!(out.contains("rgb("), "{out}");
        assert!(out.contains("calc("), "{out}");
    }
}
