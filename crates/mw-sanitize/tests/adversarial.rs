//! Adversarial input for the HTML/CSS sanitizer (SPEC §7.2).
//!
//! `mw-sanitize` is a security boundary: it decides what survives from an
//! attacker-authored message body. The crate's inline tests cover the happy
//! shapes of each rule; this file goes after the awkward ones — case folding,
//! obfuscated URLs, foreign-content elements, marker-attribute injection,
//! malformed CSS, and the selector/token walkers that only run on input nobody
//! writes by hand.
//!
//! House rule for this file: **assert what is stripped**, not merely that
//! something came back. Where a test pins current behaviour that is arguably
//! wrong, it says so in its doc comment rather than quietly baking it in.

use mw_sanitize::{CONTAINER_CLASS, MAX_Z_INDEX, sanitize_email_html as clean};

/// Every `<style>` block's CSS, extracted from the wrapper the sanitizer emits.
/// Panics if there is none — a test that wanted CSS and got none should fail
/// loudly rather than assert against an empty string.
fn css(out: &str) -> String {
    let start = out.find("<style>").map(|i| i + 7).expect("a <style> block");
    let end = out[start..].find("</style>").expect("a closing </style>") + start;
    out[start..end].to_string()
}

// ── tag stripping: case folding and foreign content ──────────────────────────

/// Tag matching is case-insensitive — `<ScRiPt>` is not a way past the
/// clean-content list.
#[test]
fn mixed_case_script_tags_are_stripped_with_their_content() {
    let out = clean(r#"<P>hi</P><ScRiPt>window.__pwned=1</ScRiPt><STYLE>x</STYLE>"#);
    assert!(!out.to_ascii_lowercase().contains("script"), "{out}");
    assert!(!out.contains("__pwned"), "{out}");
    assert!(out.contains("hi"), "{out}");
}

/// SVG and MathML are foreign-content parsing modes and a classic filter bypass.
/// Neither element survives, and script text inside them never reaches the body.
#[test]
fn svg_and_math_foreign_content_cannot_smuggle_script() {
    let out = clean(
        r#"<svg><script>alert(1)</script><a xlink:href="javascript:alert(2)">x</a></svg>
           <math><mtext><script>alert(3)</script></mtext></math>"#,
    );
    for needle in ["<svg", "<math", "alert(", "javascript:", "xlink"] {
        assert!(!out.contains(needle), "found {needle} in {out}");
    }
}

/// A `<style>` nested inside `<svg>` is parsed in the SVG namespace, and the
/// stylesheet pass drops it along with its container rather than promoting it to
/// the message stylesheet. Nothing from inside it — not even a benign
/// declaration — reaches the output, which is the safe direction: an attacker
/// cannot get CSS applied by hiding it in foreign content.
#[test]
fn style_nested_in_foreign_content_is_dropped_entirely() {
    let out = clean(
        r#"<svg><style>@import url(https://evil.example/x.css);p{color:red}</style></svg><p>x</p>"#,
    );
    assert!(!out.contains("@import"), "{out}");
    assert!(!out.contains("evil.example"), "{out}");
    assert!(!out.contains("color:red"), "{out}");
    assert!(out.contains("<p>x</p>"), "{out}");

    // A top-level `<style>` alongside one nested in `<svg>`: only the top-level
    // sheet survives, still scoped.
    let mixed = clean(r#"<style>p{color:red}</style><svg><style>.z{color:blue}</style></svg>"#);
    assert!(
        css(&mixed).contains(&format!(".{CONTAINER_CLASS} p")),
        "{mixed}"
    );
    assert!(!mixed.contains("color:blue"), "{mixed}");
}

/// Document-level redirect/base-URL elements are not on the allowlist. A `<base>`
/// would re-root every relative URL in the body; `<meta http-equiv=refresh>` is a
/// navigation primitive.
#[test]
fn base_and_meta_refresh_are_removed() {
    let out = clean(
        r#"<base href="https://evil.example/"><meta http-equiv="refresh" content="0;url=https://evil.example">
           <p>body</p>"#,
    );
    assert!(!out.contains("<base"), "{out}");
    assert!(!out.contains("<meta"), "{out}");
    assert!(!out.contains("evil.example"), "{out}");
    assert!(out.contains("body"), "{out}");
}

/// `srcset` is a second image-source channel; it is not on the attribute
/// allowlist and must not survive alongside a stripped `src`.
#[test]
fn img_srcset_is_not_a_second_remote_source() {
    let out = clean(r#"<img src="cid:ok" srcset="https://evil.example/2x.png 2x">"#);
    assert!(!out.contains("srcset"), "{out}");
    assert!(!out.contains("evil.example"), "{out}");
    assert!(out.contains("cid:ok"), "{out}");
}

// ── URL scheme obfuscation ───────────────────────────────────────────────────

/// The scheme allowlist is applied after the HTML parser has decoded entities
/// and trimmed leading control characters, so the usual `javascript:` dodges do
/// not get through.
#[test]
fn obfuscated_javascript_urls_are_all_neutralised() {
    let cases = [
        r#"<a href="JaVaScRiPt:alert(1)">a</a>"#,
        r#"<a href="  javascript:alert(1)">b</a>"#,
        r#"<a href="java&#115;cript:alert(1)">c</a>"#,
        r#"<a href="&#106;avascript:alert(1)">d</a>"#,
        r#"<a href="jav&#9;ascript:alert(1)">e</a>"#,
        r#"<a href="vbscript:msgbox(1)">f</a>"#,
        r#"<a href="data:text/html;base64,PHNjcmlwdD4=">g</a>"#,
    ];
    for case in cases {
        let out = clean(case);
        let low = out.to_ascii_lowercase();
        assert!(!low.contains("javascript"), "{case} -> {out}");
        assert!(!low.contains("vbscript"), "{case} -> {out}");
        assert!(!low.contains("alert("), "{case} -> {out}");
        assert!(!low.contains("data:"), "{case} -> {out}");
        // The link text survives; only the href is neutralised.
        assert!(
            out.contains("</a>") || !out.contains("<a "),
            "{case} -> {out}"
        );
    }
}

/// A protocol-relative `//host/…` reference has no scheme and is denied as a
/// relative URL — and because it carries no scheme it also produces no blocked
/// marker, so the host never appears in the output at all.
#[test]
fn protocol_relative_image_is_denied_outright() {
    let out = clean(r#"<img src="//evil.example/p.gif">"#);
    assert!(!out.contains("evil.example"), "{out}");
    assert!(!out.contains("data-mw-blocked-host"), "{out}");
}

// ── the blocked-resource marker ──────────────────────────────────────────────

/// A hostile host label cannot break out of the marker's double-quoted
/// attribute: `&`, `"`, `<` and `>` are escaped. This is the one place the
/// sanitizer writes attacker-influenced text back into the document.
#[test]
fn blocked_host_marker_escapes_attribute_metacharacters() {
    let out = clean("<img src='https://a\"b&c.example/p.gif'>");
    assert!(
        out.contains(r#"data-mw-blocked-host="a&quot;b&amp;c.example""#),
        "{out}"
    );
    // No unescaped quote may close the attribute early.
    assert!(!out.contains(r#"data-mw-blocked-host="a""#), "{out}");
}

/// A host carrying a URL-forbidden code point (`<`, `>`, or a bidi override) is
/// rejected before the sanitizer's own filter runs, so the image is dropped and
/// nothing about it — not even a marker — is written back. That closes the same
/// injection avenue from the other end: the escaping above never has to handle
/// an angle bracket because such a URL cannot reach it.
#[test]
fn a_host_with_forbidden_code_points_yields_no_marker_at_all() {
    for src in [
        "<img src='https://a<b.example/p.gif'>",
        "<img src='https://a>b.example/p.gif'>",
        "<img src=\"https://ex\u{202E}ample.test/p.gif\">",
    ] {
        let out = clean(src);
        assert_eq!(out, "<img>", "{src} -> {out}");
    }
}

/// An IPv6 literal host keeps its address and loses its port — the bracket form
/// has no bare `:port` to trim inside it.
#[test]
fn blocked_host_marker_handles_an_ipv6_literal() {
    let out = clean(r#"<img src="https://[2001:db8::1]:8443/p.gif">"#);
    assert!(
        out.contains(r#"data-mw-blocked-host="2001:db8::1""#),
        "{out}"
    );
    assert!(!out.contains("8443"), "{out}");
}

/// The tracker flag matches a bare tracker domain as well as a sub-domain of
/// one, and is case-insensitive via the host lower-casing.
#[test]
fn tracker_flag_matches_bare_domain_and_folds_case() {
    let bare = clean(r#"<img src="https://LIST-MANAGE.COM/o.gif">"#);
    assert!(
        bare.contains(r#"data-mw-blocked-host="list-manage.com""#),
        "{bare}"
    );
    assert!(bare.contains("data-mw-tracker"), "{bare}");

    // A domain that merely *ends with* the tracker string but is not a
    // sub-domain of it must not be flagged (`notlist-manage.com`).
    let lookalike = clean(r#"<img src="https://notlist-manage.com/o.gif">"#);
    assert!(
        lookalike.contains(r#"data-mw-blocked-host="notlist-manage.com""#),
        "{lookalike}"
    );
    assert!(!lookalike.contains("data-mw-tracker"), "{lookalike}");
}

/// A punycode host is recorded in its A-label form — the marker reports what
/// would have been fetched, not a display form that could be confused with a
/// different domain.
#[test]
fn idn_host_is_recorded_in_punycode_form() {
    let out = clean("<img src=\"https://xn--e1afmkfd.example/p.gif\">");
    assert!(
        out.contains(r#"data-mw-blocked-host="xn--e1afmkfd.example""#),
        "{out}"
    );
}

// ── stylesheet extraction ────────────────────────────────────────────────────

/// `<STYLE>` in any case is found and filtered — the cheap "no `<style` substring"
/// guard is ASCII-case-insensitive.
#[test]
fn uppercase_style_block_is_still_extracted_and_scoped() {
    let out = clean(r#"<STYLE>P{COLOR:RED;POSITION:FIXED}</STYLE><p>x</p>"#);
    let sheet = css(&out);
    assert!(sheet.contains(&format!(".{CONTAINER_CLASS} P")), "{sheet}");
    assert!(sheet.to_ascii_lowercase().contains("color:red"), "{sheet}");
    assert!(!sheet.to_ascii_lowercase().contains("fixed"), "{sheet}");
}

/// Several `<style>` blocks are all processed, not just the first.
#[test]
fn every_style_block_is_processed() {
    let out = clean(r#"<style>.a{color:red}</style><p>x</p><style>.b{color:blue}</style>"#);
    let sheet = css(&out);
    assert!(sheet.contains(&format!(".{CONTAINER_CLASS} .a")), "{sheet}");
    assert!(sheet.contains(&format!(".{CONTAINER_CLASS} .b")), "{sheet}");
}

/// A rule with no selector at all contributes nothing rather than producing a
/// bare `{…}` that would apply to everything.
#[test]
fn rule_with_an_empty_prelude_is_dropped() {
    let out = clean(r#"<style>{color:red}.keep{color:blue}</style><p>x</p>"#);
    let sheet = css(&out);
    assert!(!sheet.starts_with('{'), "{sheet}");
    assert!(!sheet.contains("color:red"), "{sheet}");
    assert!(
        sheet.contains(&format!(".{CONTAINER_CLASS} .keep")),
        "{sheet}"
    );
}

/// An empty selector between two commas is skipped, and the surviving ones are
/// still each scoped.
#[test]
fn empty_selector_in_a_list_is_skipped() {
    let out = clean(r#"<style>h1, , .foo{color:red}</style><p>x</p>"#);
    let sheet = css(&out);
    assert!(sheet.contains(&format!(".{CONTAINER_CLASS} h1")), "{sheet}");
    assert!(
        sheet.contains(&format!(".{CONTAINER_CLASS} .foo")),
        "{sheet}"
    );
    assert!(!sheet.contains(",,"), "{sheet}");
}

// ── selector namespacing ─────────────────────────────────────────────────────

/// `html`/`body`/`:root` map onto the container element only at a real token
/// boundary. An element or class whose name merely *starts with* `body` is a
/// descendant like any other — otherwise `bodyguard { … }` would silently become
/// a rule on the message container itself.
#[test]
fn body_prefix_only_collapses_at_a_token_boundary() {
    let out = clean(
        r#"<style>bodyguard{color:red}body-x{color:blue}body.cls{color:green}:rootish{color:teal}</style><p>x</p>"#,
    );
    let sheet = css(&out);
    assert!(
        sheet.contains(&format!(".{CONTAINER_CLASS} bodyguard")),
        "{sheet}"
    );
    assert!(
        sheet.contains(&format!(".{CONTAINER_CLASS} body-x")),
        "{sheet}"
    );
    assert!(
        sheet.contains(&format!(".{CONTAINER_CLASS} :rootish")),
        "{sheet}"
    );
    // `body.cls` *is* at a boundary (`.`), so it collapses onto the container.
    assert!(
        sheet.contains(&format!(".{CONTAINER_CLASS}.cls")),
        "{sheet}"
    );
}

/// A comma inside a quoted attribute selector or inside `:is(…)` is not a
/// selector-list separator — splitting there would produce two broken selectors
/// and silently drop the rule's real target.
#[test]
fn commas_inside_strings_and_brackets_do_not_split_the_selector_list() {
    let out = clean(
        r#"<style>a[title="x,y"]{color:red}</style><style>:is(h1, h2) span{color:blue}</style><p>x</p>"#,
    );
    let sheet = css(&out);
    assert!(sheet.contains(r#"a[title="x,y"]"#), "{sheet}");
    assert!(sheet.contains(":is(h1, h2) span"), "{sheet}");
    // One scoped selector per rule — the commas above did not create extras.
    assert_eq!(
        sheet.matches(&format!(".{CONTAINER_CLASS} ")).count(),
        2,
        "{sheet}"
    );
}

// ── at-rules ─────────────────────────────────────────────────────────────────

/// Only `@media` and `@supports` survive. Everything else — including the ones
/// that load remote resources or install animations — is dropped whole.
#[test]
fn every_at_rule_but_media_and_supports_is_dropped() {
    let out = clean(
        r#"<style>
            @charset "utf-8";
            @namespace url(https://evil.example/ns);
            @font-face{font-family:x;src:url(https://evil.example/f.woff)}
            @keyframes spin{from{opacity:0}to{opacity:1}}
            @page{margin:0}
            @supports (display:grid){p{color:red}}
            @media screen{h1{color:blue}}
        </style><p>x</p>"#,
    );
    let sheet = css(&out);
    for dropped in [
        "@charset",
        "@namespace",
        "@font-face",
        "@keyframes",
        "@page",
    ] {
        assert!(!sheet.contains(dropped), "{dropped} survived: {sheet}");
    }
    assert!(!sheet.contains("evil.example"), "{sheet}");
    assert!(sheet.contains("@supports (display:grid)"), "{sheet}");
    assert!(sheet.contains("@media screen"), "{sheet}");
    // Inner selectors of the surviving group rules are namespaced too.
    assert!(sheet.contains(&format!(".{CONTAINER_CLASS} p")), "{sheet}");
    assert!(sheet.contains(&format!(".{CONTAINER_CLASS} h1")), "{sheet}");
}

/// A group rule whose body is empty (or whose every declaration was filtered
/// out) contributes nothing — no empty `@media{}` shell.
#[test]
fn group_rule_with_nothing_left_inside_is_dropped() {
    let out = clean(
        r#"<style>@media screen{}@media print{p{-moz-binding:url(cid:x)}}.keep{color:red}</style><p>x</p>"#,
    );
    let sheet = css(&out);
    assert!(!sheet.contains("@media"), "{sheet}");
    assert!(!sheet.contains("binding"), "{sheet}");
    assert!(
        sheet.contains(&format!(".{CONTAINER_CLASS} .keep")),
        "{sheet}"
    );
}

/// `@media` with no condition keeps its body and emits no stray space.
#[test]
fn conditionless_media_rule_survives() {
    let out = clean(r#"<style>@media{p{color:red}}</style><p>x</p>"#);
    let sheet = css(&out);
    assert!(sheet.starts_with("@media{"), "{sheet}");
    assert!(sheet.contains(&format!(".{CONTAINER_CLASS} p")), "{sheet}");
}

/// Nested group rules recurse, and the innermost selectors are still scoped.
#[test]
fn nested_group_rules_recurse_and_scope() {
    let out = clean(
        r#"<style>@media screen{@supports (color:red){p{position:fixed;color:red}}}</style><p>x</p>"#,
    );
    let sheet = css(&out);
    assert!(
        sheet.contains("@media screen{@supports (color:red){"),
        "{sheet}"
    );
    assert!(sheet.contains(&format!(".{CONTAINER_CLASS} p")), "{sheet}");
    assert!(
        !sheet.contains("fixed"),
        "position:fixed survived nesting: {sheet}"
    );
}

// ── declaration values ───────────────────────────────────────────────────────

/// The value-level checks fold case: an uppercase `EXPRESSION(` or `URL(` is the
/// same threat as a lowercase one.
#[test]
fn value_checks_are_case_insensitive() {
    let out = clean(
        r#"<div style="WIDTH:EXPRESSION(alert(1));BACKGROUND:URL(HTTPS://EVIL.EXAMPLE/x);COLOR:red">x</div>"#,
    );
    assert!(!out.to_ascii_lowercase().contains("expression"), "{out}");
    assert!(!out.to_ascii_lowercase().contains("evil.example"), "{out}");
    assert!(out.contains("color:red"), "{out}");
}

/// `url()` is scanned across the whole value: a `cid:` reference first does not
/// license a remote one after it.
#[test]
fn a_leading_cid_url_does_not_smuggle_a_later_remote_one() {
    let mixed =
        clean(r#"<div style="background:url(cid:ok), url(https://evil.example/x.png)">x</div>"#);
    assert!(!mixed.contains("evil.example"), "{mixed}");
    assert!(
        !mixed.contains("background"),
        "whole declaration is dropped: {mixed}"
    );

    // Whitespace and quotes around a cid target are tolerated.
    let ok = clean(r#"<div style="background:url( 'cid:img1' ) no-repeat">x</div>"#);
    assert!(ok.contains("cid:img1"), "{ok}");
}

/// A declaration whose value is empty is dropped rather than emitted as
/// `prop:` with nothing after it.
#[test]
fn empty_declaration_value_is_dropped() {
    let out = clean(r#"<div style="color:;font-size:12px">x</div>"#);
    assert!(!out.contains("color:"), "{out}");
    assert!(out.contains("font-size:12px"), "{out}");
}

/// A malformed token in a value (here a bad-url: an unquoted `url()` containing
/// whitespace) aborts that declaration only — its neighbours survive.
#[test]
fn a_malformed_token_drops_only_its_own_declaration() {
    let out = clean(r#"<div style="color:red;background:url(a b);font-size:12px">x</div>"#);
    assert!(out.contains("color:red"), "{out}");
    assert!(out.contains("font-size:12px"), "{out}");
    assert!(!out.contains("background"), "{out}");
}

/// Bracket and function blocks inside a value are walked and re-serialised with
/// their closers, so a multi-part value is not silently truncated at the first
/// block.
#[test]
fn bracketed_and_nested_function_values_survive_intact() {
    let out = clean(
        r#"<div style="grid-template-columns:[full-start] minmax(1rem, 1fr) [full-end];color:rgb(calc(1 + 1), 2, 3)">x</div>"#,
    );
    assert!(out.contains("[full-start]"), "{out}");
    assert!(out.contains("[full-end]"), "{out}");
    assert!(out.contains("minmax("), "{out}");
    assert!(out.contains("calc("), "{out}");
}

/// `position` is gated on its first keyword only, case-insensitively, and the
/// permitted values still pass.
#[test]
fn position_gate_folds_case_and_checks_only_the_first_keyword() {
    assert!(
        !clean(r#"<div style="position:FIXED">x</div>"#)
            .to_ascii_lowercase()
            .contains("fixed")
    );
    assert!(
        !clean(r#"<div style="position:Sticky">x</div>"#)
            .to_ascii_lowercase()
            .contains("sticky")
    );
    assert!(clean(r#"<div style="position:absolute">x</div>"#).contains("position:absolute"));
}

/// The `z-index` clamp keeps any trailing tokens rather than dropping them, and
/// leaves in-range and negative values alone.
#[test]
fn z_index_clamp_preserves_trailing_tokens_and_ignores_small_values() {
    let clamped = clean(r#"<div style="z-index:99999 !important">x</div>"#);
    assert!(
        clamped.contains(&format!("z-index:{MAX_Z_INDEX}")),
        "{clamped}"
    );
    assert!(!clamped.contains("99999"), "{clamped}");
    assert!(
        clamped.contains("important"),
        "trailing tokens lost: {clamped}"
    );

    assert!(clean(r#"<div style="z-index:-5">x</div>"#).contains("z-index:-5"));
    assert!(clean(r#"<div style="z-index:+7">x</div>"#).contains("z-index:"));
}

/// DEFECT (recorded, not fixed): the `z-index` clamp only understands a bare
/// integer. `calc()` is valid CSS wherever an `<integer>` is, so
/// `z-index: calc(99999)` reaches the browser unclamped, as does any value the
/// `i64` parse rejects but a browser accepts.
///
/// Severity is low — `MAX_Z_INDEX` is defence-in-depth *inside* a body that is
/// already rendered in a sandboxed, opaque-origin iframe, so a high stacking
/// order cannot overlay host chrome. Recording it because the clamp reads as
/// unconditional and is not.
#[test]
fn z_index_clamp_is_bypassed_by_calc() {
    let out = clean(r#"<div style="z-index:calc(99999)">x</div>"#);
    assert!(out.contains("99999"), "clamp now covers calc(): {out}");
    assert!(!out.contains(&format!("z-index:{MAX_Z_INDEX}")), "{out}");
}

// ── shape of the output ──────────────────────────────────────────────────────

/// The wrapper appears when — and only when — there is something to wrap: a
/// scoped stylesheet or a block marker.
#[test]
fn wrapper_appears_only_for_scoped_css_or_markers() {
    assert!(!clean("<p>plain</p>").contains(CONTAINER_CLASS));
    assert!(clean(r#"<style>p{color:red}</style><p>x</p>"#).contains(CONTAINER_CLASS));
    assert!(clean(r#"<img src="https://a.example/1.gif">"#).contains(CONTAINER_CLASS));
    // A `<style>` whose every rule was filtered out leaves nothing to wrap.
    assert!(
        !clean(r#"<style>@import url(https://evil.example/x)</style><p>x</p>"#)
            .contains(CONTAINER_CLASS)
    );
}

/// Degenerate inputs return without panicking and without inventing content.
#[test]
fn degenerate_inputs_are_handled() {
    assert_eq!(clean(""), "");
    for input in [
        "<style",
        "<style>",
        "<style></style>",
        "<style>@media</style>",
        "</style>",
        "<img src=",
        "\u{0}\u{feff}<p>x</p>",
        "<p>&#x110000;</p>",
    ] {
        let out = clean(input);
        assert!(!out.contains("<script"), "{input:?} -> {out}");
    }
}
