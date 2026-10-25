//! The reader's remote-content mode (t32 e31a, plan R): placeholders, trackers and
//! link cleaning, and that the strip mode is unchanged.
//!
//! Run:
//!   cargo test -p mw-sanitize --test t32_remote_content -- --test-threads=1

use mw_sanitize::{RemoteImages, SanitizeOptions, sanitize_email_html, sanitize_email_html_with};

fn reader(html: &str) -> String {
    sanitize_email_html_with(html, &SanitizeOptions::READER)
}

/// Every value of `attr="…"` in `html`.
fn attr_values(html: &str, attr: &str) -> Vec<String> {
    let needle = format!(" {attr}=\"");
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(i) = rest.find(&needle) {
        let after = &rest[i + needle.len()..];
        let end = after.find('"').unwrap_or(after.len());
        out.push(after[..end].to_string());
        rest = &after[end..];
    }
    out
}

/// No attribute a browser fetches from, and no CSS `url(`, carries an http(s) URL.
fn assert_nothing_loadable(out: &str) {
    for attr in ["src", "srcset", "background", "poster"] {
        for v in attr_values(out, attr) {
            assert!(
                !v.to_ascii_lowercase().contains("http"),
                "{attr}=\"{v}\" is loadable in {out}"
            );
        }
    }
    let low = out.to_ascii_lowercase();
    let mut rest = low.as_str();
    while let Some(i) = rest.find("url(") {
        let arg = &rest[i + 4..];
        assert!(
            arg.starts_with("mw-rc:") || arg.starts_with("cid:") || arg.starts_with("\"cid:"),
            "a CSS url() is loadable in {out}"
        );
        rest = &arg[1..];
    }
}

#[test]
fn a_remote_image_becomes_a_placeholder_with_its_url_in_a_marker() {
    let out = reader(r#"<p>Hi</p><img alt="logo" src="https://cdn.example/a.png?x=1&amp;y=2">"#);
    assert_eq!(attr_values(&out, "src"), ["mw-rc:0"], "{out}");
    assert!(out.contains(r#"data-mw-rc="0""#), "{out}");
    assert!(
        out.contains(r#"data-mw-rc-url="https://cdn.example/a.png?x=1&amp;y=2""#),
        "the original URL rides in a data- attribute, escaped: {out}"
    );
    assert!(
        out.contains(r#"data-mw-blocked-host="cdn.example""#),
        "{out}"
    );
    assert!(!out.contains("data-mw-tracker"), "{out}");
    assert_nothing_loadable(&out);
}

#[test]
fn placeholders_are_numbered_in_order_and_cid_images_are_kept() {
    let out = reader(
        r#"<img src="https://a.example/1.png"><img src="cid:part1"><img src="https://b.example/2.png">"#,
    );
    assert_eq!(
        attr_values(&out, "src"),
        ["mw-rc:0", "cid:part1", "mw-rc:1"],
        "{out}"
    );
    assert!(
        out.contains(r#"data-mw-rc="1" data-mw-rc-url="https://b.example/2.png""#),
        "{out}"
    );
}

#[test]
fn a_tracker_by_host_or_beacon_shape_is_stripped_and_counted() {
    for (html, host) in [
        (
            r#"<img src="https://us5.list-manage.com/track/open.php?u=1">"#,
            "us5.list-manage.com",
        ),
        (
            r#"<img src="https://news.example/wf/open?upn=xyz">"#,
            "news.example",
        ),
        (
            r#"<img src="https://news.example/img?e=abc&amp;u=def">"#,
            "news.example",
        ),
    ] {
        let out = reader(html);
        assert!(attr_values(&out, "src").is_empty(), "{out}");
        assert!(
            !out.contains("data-mw-rc="),
            "a tracker must not be signable: {out}"
        );
        assert!(
            out.contains(&format!(r#"data-mw-blocked-host="{host}" data-mw-tracker"#)),
            "{out}"
        );
    }
}

#[test]
fn with_tracker_protection_off_a_tracker_is_a_flagged_placeholder() {
    let opts = SanitizeOptions {
        block_trackers: false,
        ..SanitizeOptions::READER
    };
    let out = sanitize_email_html_with(r#"<img src="https://x.list-manage.com/p.png">"#, &opts);
    assert_eq!(attr_values(&out, "src"), ["mw-rc:0"]);
    assert!(out.contains("data-mw-tracker"), "{out}");
}

#[test]
fn an_image_declared_at_most_two_by_two_is_marked_a_tracker() {
    for html in [
        r#"<img src="https://cdn.example/p.png" width="1" height="1">"#,
        r#"<img width="2px" src="https://cdn.example/p.png" height="0">"#,
        r#"<img src="https://cdn.example/p.png" style="width:1px;height:1px">"#,
    ] {
        let out = reader(html);
        assert!(
            out.contains(r#"data-mw-rc="0" data-mw-rc-url="https://cdn.example/p.png" data-mw-blocked-host="cdn.example" data-mw-tracker"#),
            "{html} → {out}"
        );
    }
    // Negative control: a real image is not one.
    let out = reader(r#"<img src="https://cdn.example/p.png" width="600" height="1">"#);
    assert!(!out.contains("data-mw-tracker"), "{out}");
    let out = reader(r#"<img src="https://cdn.example/p.png" width="1">"#);
    assert!(
        !out.contains("data-mw-tracker"),
        "one dimension is not a declared pixel: {out}"
    );
}

#[test]
fn srcset_candidates_become_placeholders_or_the_attribute_goes() {
    let out = reader(
        r#"<img src="https://a.example/1.png" srcset="https://a.example/1.png 1x, https://a.example/2.png 2x">"#,
    );
    assert_eq!(
        attr_values(&out, "srcset"),
        ["mw-rc:1 1x, mw-rc:2 2x"],
        "{out}"
    );
    for bad in [
        r#"<img srcset="data:image/png;base64,AAAA 1x">"#,
        r#"<img srcset="javascript:alert(1) 1x">"#,
        r#"<img srcset="https://a.example/1.png 1x 2x">"#,
    ] {
        let out = reader(bad);
        assert!(attr_values(&out, "srcset").is_empty(), "{bad} → {out}");
    }
}

#[test]
fn the_background_attribute_becomes_a_placeholder_or_goes() {
    let out = reader(
        r#"<table background="https://a.example/bg.png"><tr><td background="cid:bg">x</td></tr></table>"#,
    );
    assert_eq!(
        attr_values(&out, "background"),
        ["mw-rc:0", "cid:bg"],
        "{out}"
    );
    let out = reader(r#"<table background="javascript:alert(1)"><tr><td>x</td></tr></table>"#);
    assert!(attr_values(&out, "background").is_empty(), "{out}");
}

#[test]
fn css_urls_in_image_properties_become_placeholders_and_others_are_dropped() {
    let out = reader(
        r#"<div style="color:red;background-image:url('https://a.example/bg.png')">x</div>"#,
    );
    assert!(out.contains("background-image:url(mw-rc:0)"), "{out}");
    assert!(out.contains("color:red"), "{out}");

    let out = reader(
        r#"<style>.hero{background:#fff url("https://a.example/h.png") no-repeat}</style><div class="hero">x</div>"#,
    );
    assert!(out.contains("url(mw-rc:0)"), "{out}");
    assert!(
        out.contains(r#"data-mw-rc="0" data-mw-rc-url="https://a.example/h.png""#),
        "{out}"
    );

    for html in [
        r#"<div style="background:url(data:image/png;base64,AAAA)">x</div>"#,
        r#"<div style="width:url(https://a.example/x)">x</div>"#,
        r#"<div style="background:url(https://x.list-manage.com/p.png)">x</div>"#,
    ] {
        let out = reader(html);
        assert!(!out.contains("url("), "{html} → {out}");
    }
    assert_nothing_loadable(&reader(
        r#"<style>@import url(https://a.example/x.css); .a{list-style-image:url(https://a.example/b.png)}</style><ul class="a"><li>x</li></ul>"#,
    ));
}

#[test]
fn a_message_cannot_forge_a_placeholder() {
    let out = reader(
        r#"<img src="mw-rc:0"><div style="background:url(mw-rc:0)">x</div><table background="mw-rc:0"><tr><td>y</td></tr></table>"#,
    );
    assert!(!out.contains("mw-rc:0"), "{out}");
    assert!(!out.contains("data-mw-rc="), "{out}");
}

#[test]
fn tracking_parameters_are_removed_from_links_and_the_original_is_kept_in_a_marker() {
    let out = reader(
        r#"<a href="https://shop.example/p?id=7&amp;utm_source=news&amp;fbclid=abc">Shop</a> <a href="https://plain.example/a?b=1">Plain</a>"#,
    );
    let hrefs = attr_values(&out, "href");
    assert_eq!(
        hrefs,
        ["https://shop.example/p?id=7", "https://plain.example/a?b=1"],
        "{out}"
    );
    assert!(
        out.contains(r#"data-mw-link-cleaned="https://shop.example/p?id=7" data-mw-link-original="https://shop.example/p?id=7&amp;utm_source=news&amp;fbclid=abc""#),
        "{out}"
    );
    assert_eq!(out.matches("data-mw-link-cleaned").count(), 1, "{out}");
    assert!(
        out.contains(r#"rel="noopener noreferrer nofollow""#),
        "{out}"
    );
}

#[test]
fn a_hostile_url_is_escaped_in_its_marker() {
    let out = reader(r#"<img src='https://a.example/"><script>alert(1)</script>'>"#);
    assert!(!out.contains("<script"), "{out}");
    assert!(
        out.contains("data-mw-rc-url=\"https://a.example/&quot;&gt;&lt;script&gt;"),
        "{out}"
    );
}

#[test]
fn strip_mode_is_what_it_was() {
    let html = r#"<img src="https://a.example/1.png" srcset="https://a.example/2.png 2x"><table background="https://a.example/bg.png"><tr><td>x</td></tr></table><a href="https://s.example/?utm_source=x">l</a><div style="background-image:url(https://a.example/b.png)">y</div>"#;
    let strip = sanitize_email_html(html);
    assert_eq!(
        strip,
        sanitize_email_html_with(html, &SanitizeOptions::STRIP)
    );
    for needle in [
        "mw-rc",
        "data-mw-rc",
        "srcset",
        "background=",
        "data-mw-link",
        "url(",
    ] {
        assert!(!strip.contains(needle), "{needle} in {strip}");
    }
    assert!(
        strip.contains("utm_source"),
        "strip mode does not clean links: {strip}"
    );
    assert!(
        strip.contains(r#"data-mw-blocked-host="a.example""#),
        "{strip}"
    );
    assert_nothing_loadable(&strip);
}

#[test]
fn placeholder_mode_without_link_cleaning_leaves_links_alone() {
    let opts = SanitizeOptions {
        remote_images: RemoteImages::Placeholder,
        block_trackers: true,
        clean_links: false,
    };
    let out = sanitize_email_html_with(r#"<a href="https://s.example/?utm_source=x">l</a>"#, &opts);
    assert!(out.contains("utm_source=x"), "{out}");
    assert!(!out.contains("data-mw-link"), "{out}");
}
