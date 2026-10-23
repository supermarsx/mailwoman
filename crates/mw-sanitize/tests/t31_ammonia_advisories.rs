//! The payloads of the three 2026 ammonia advisories, fed through
//! `sanitize_email_html` (t31-e1).
//!
//! - RUSTSEC-2021-0157: quotation marks inside CSS `url("")` were not escaped
//!   by ammonia's `filter_style_properties`, so a `style` value could break out
//!   of one `url()` and inject further declarations.
//! - RUSTSEC-2026-0373: ammonia matched the CSS `url` function
//!   case-sensitively, so `URL(...)` passed its URL checks.
//! - RUSTSEC-2021-0158: SVG `animate*` elements could set `href` to a
//!   `javascript:` URL.
//!
//! The two CSS advisories need ammonia's own `filter_style_properties`, which
//! `mw-sanitize` does not use: it runs its own cssparser-based filter from the
//! attribute filter, and it removes `<svg>` with its content. These tests
//! document that this configuration does not let the payloads through; they
//! passed against ammonia 4.1.4 as well as 4.1.8, so they do not prove the
//! upgrade (the t31-e1 log records both runs).

use mw_sanitize::sanitize_email_html as clean;

/// RUSTSEC-2021-0157: a `\"` inside one `url("")` must not end it and open a
/// second declaration that loads a remote URL.
#[test]
fn css_url_quote_breakout_does_not_inject_a_declaration() {
    let input = r#"<div style='background:url("cid:\");x:y;background:url(\"https://evil.example/q")'>t</div>"#;
    let out = clean(input);
    assert!(
        out.contains(">t</div>"),
        "the element's text survives: {out}"
    );
    assert!(
        !out.contains("x:y"),
        "the injected declaration is dropped: {out}"
    );
    assert!(
        !out.contains("url(\"https://evil.example") && !out.contains("url(https://evil.example"),
        "no remote url() survives in a style value: {out}"
    );
}

/// The plan's literal payload for RUSTSEC-2021-0157 (both halves `cid:`).
#[test]
fn css_url_quote_breakout_with_cid_halves_does_not_inject_a_declaration() {
    let input = r#"<div style='background:url("cid:\");x:y;background:url(\"cid:")'>t</div>"#;
    let out = clean(input);
    assert!(
        out.contains(">t</div>"),
        "the element's text survives: {out}"
    );
    assert!(
        !out.contains("x:y"),
        "the injected declaration is dropped: {out}"
    );
}

/// RUSTSEC-2026-0373: an upper-case `URL(` is still a remote load and is
/// removed like a lower-case one.
#[test]
fn upper_case_css_url_function_is_not_let_through() {
    let input = r#"<div style='background:URL("https://evil.example/p")'>t</div>"#;
    let out = clean(input);
    assert!(
        out.contains(">t</div>"),
        "the element's text survives: {out}"
    );
    let lower = out.to_ascii_lowercase();
    assert!(
        !lower.contains("url(\"https://evil.example")
            && !lower.contains("url(https://evil.example"),
        "no remote url() survives in a style value: {out}"
    );
}

/// RUSTSEC-2021-0158: an SVG `animate` that sets `href` to `javascript:` does
/// not survive in any form.
#[test]
fn svg_animate_with_javascript_href_is_removed() {
    let input = r#"<p>a</p><svg><animate attributeName="href" values="javascript:alert(1)"/></svg><p>b</p>"#;
    let out = clean(input);
    assert!(
        out.contains("<p>a</p>") && out.contains("<p>b</p>"),
        "surrounding text survives: {out}"
    );
    let lower = out.to_ascii_lowercase();
    assert!(
        !lower.contains("javascript:"),
        "no javascript: URL survives: {out}"
    );
    assert!(
        !lower.contains("animate"),
        "no animate element survives: {out}"
    );
    assert!(!lower.contains("<svg"), "no svg element survives: {out}");
}

/// The other two element names the advisory names.
#[test]
fn svg_animate_motion_and_transform_are_removed() {
    let input = r#"<svg><animateMotion attributeName="href" values="javascript:alert(1)"/><animateTransform attributeName="href" values="javascript:alert(2)"/><animateColor attributeName="href" values="javascript:alert(3)"/></svg>"#;
    let lower = clean(input).to_ascii_lowercase();
    assert!(
        !lower.contains("javascript:"),
        "no javascript: URL survives: {lower}"
    );
    assert!(
        !lower.contains("animate"),
        "no animate* element survives: {lower}"
    );
}
