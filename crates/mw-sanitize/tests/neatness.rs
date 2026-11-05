//! Output neatness (no empty/redundant wrappers) with fidelity (text, links, images,
//! tables unchanged) and the processing limits.
use mw_sanitize::{MAX_NESTING_DEPTH, SanitizeOptions, sanitize_email_html, sanitize_email_html_with};

fn text_of(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

#[test]
fn drops_empty_and_attribute_less_wrappers() {
    let src = "<div><div><span><font>Hello</font></span> <span></span><b></b><i> </i></div></div><p>x</p>";
    let out = sanitize_email_html(src);
    assert!(!out.contains("<span>"), "{out}");
    assert!(!out.contains("<font>"), "{out}");
    assert!(!out.contains("<b></b>"), "{out}");
    assert_eq!(count(&out, "<div>"), 1, "{out}");
    assert_eq!(text_of(&out), text_of(src));
}

#[test]
fn keeps_wrappers_with_attributes_and_stylesheet_targets() {
    let out = sanitize_email_html("<span style=\"color:red\">a</span><span class=\"k\"></span>");
    assert!(out.contains("<span style=\"color:red\">a</span>"), "{out}");
    assert!(out.contains("<span class=\"k\"></span>"), "{out}");
    let out = sanitize_email_html("<style>span{color:red}</style><span>a</span>");
    assert!(out.contains("<span>a</span>"), "a stylesheet names span: {out}");
}

#[test]
fn collapses_long_br_runs_to_two_and_whitespace_runs() {
    let out = sanitize_email_html("<p>a<br><br><br><br>b   \n\n  c</p>");
    assert_eq!(count(&out, "<br>"), 2, "{out}");
    assert!(out.contains("b c"), "{out}");
}

#[test]
fn preformatted_and_white_space_styled_text_is_untouched() {
    let out = sanitize_email_html("<pre>a   b\n\nc</pre>");
    assert!(out.contains("a   b\n\nc"), "{out}");
    let out = sanitize_email_html("<p style=\"white-space:pre\">a   b</p>");
    assert!(out.contains("a   b"), "{out}");
}

#[test]
fn links_images_tables_and_placeholders_are_preserved() {
    let src = "<table><tr><td><a href=\"https://a.example/x\"><span>one</span></a><img src=\"https://t.example/1.gif\" width=\"1\" height=\"1\"><img src=\"cid:x\"></td></tr></table><div><div><img src=\"https://t.example/2.png\"></div></div>";
    let raw = sanitize_email_html_with(src, &SanitizeOptions::READER);
    for tag in ["<table", "<tr", "<td", "<a ", "<img"] {
        assert_eq!(count(&raw, tag), count(src, tag), "{tag}: {raw}");
    }
    assert!(raw.contains("mw-rc:0") && raw.contains("mw-rc:1"), "{raw}");
    assert!(raw.find("mw-rc:0").unwrap() < raw.find("mw-rc:1").unwrap());
}

#[test]
fn output_is_deterministic_and_idempotent_enough() {
    let src = "<div><div><span>x</span><br><br><br></div></div>";
    let a = sanitize_email_html(src);
    assert_eq!(a, sanitize_email_html(src));
    assert_eq!(sanitize_email_html(&a), sanitize_email_html(&sanitize_email_html(&a)));
}

#[test]
fn pathological_nesting_falls_back_to_escaped_text_without_parsing() {
    let depth = MAX_NESTING_DEPTH + 10;
    let src = format!("{}x{}", "<div><i>".repeat(depth), "</i></div>".repeat(depth));
    let out = sanitize_email_html(&src);
    assert!(out.starts_with("<pre data-mw-body-truncated=\"true\">&lt;div&gt;"), "{}", &out[..80]);
    assert!(!out.contains("<div>"));
    // Within the limit, parsed normally.
    let ok = format!("{}x{}", "<div>".repeat(200), "</div>".repeat(200));
    assert!(!sanitize_email_html(&ok).contains("data-mw-body-truncated"));
    // Unclosed optional-end elements (old-style `<p>` lists) do not count as depth.
    let lax = "<p>a".repeat(5000);
    assert!(!sanitize_email_html(&lax).contains("data-mw-body-truncated"));
}

#[test]
fn oversized_input_is_escaped_text() {
    let src = format!("<script>alert(1)</script>{}", "a".repeat(mw_sanitize::MAX_INPUT_BYTES));
    let out = sanitize_email_html(&src);
    assert!(out.starts_with("<pre data-mw-body-truncated=\"true\">&lt;script&gt;"));
    assert!(!out.contains("<script"));
    assert!(out.len() < 1_100_000);
}
