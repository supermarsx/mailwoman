use mw_sanitize::{SanitizeOptions, sanitize_email_html_reader, sanitize_email_html_reader_safe};

#[test]
fn protocol_relative_navigation_links_become_explicit_https_without_enabling_images() {
    let input = r#"<a href="//shop.example/order?id=7&amp;token=a%2Bb#invoice">Order</a><img src="//images.example/logo.png">"#;
    for options in [SanitizeOptions::STRIP, SanitizeOptions::READER] {
        for sanitize in [sanitize_email_html_reader, sanitize_email_html_reader_safe] {
            let output = sanitize(input, &options);
            assert!(
                output
                    .contains(r#"href="https://shop.example/order?id=7&amp;token=a%2Bb#invoice""#),
                "{output}"
            );
            assert!(!output.contains(r#"src="//"#), "{output}");
            assert!(!output.contains("data-mw-rc-url"), "{output}");
        }
    }
}

#[test]
fn ordinary_navigation_links_keep_queries_wrappers_and_safe_html5_attributes() {
    for href in [
        "https://shop.example/order?id=7&amp;token=a%2Bb#invoice",
        "https://shop.test/?name=Jane Doe&amp;query=a b",
        "https://shop.test/a report?name=Jane%20Doe&amp;encoded=a%2Bb&amp;literal=a+b",
        "HTTP://example.org/report?one=1&amp;two=2",
        "https://redirect.example/click?url=https%3A%2F%2Fdestination.example%2Fa%3Fx%3D1%26y%3D2&amp;signature=abc",
        "mailto:hello@example.org?subject=Hello%20there&amp;body=Hi",
        "mailto:hello@example.org?subject=Hello there&amp;body=Hi friend",
        "tel:+351123456789",
    ] {
        let input = format!(
            r#"<a href="{href}" title="View report" aria-label="Report" ping="https://beacon.example/" onclick="">Read</a>"#
        );
        let output = sanitize_email_html_reader_safe(&input, &SanitizeOptions::READER);
        assert!(output.contains(&format!(r#"href="{href}""#)), "{output}");
        assert!(output.contains("View report"), "{output}");
        assert!(
            !output.contains("ping=") && !output.contains("onclick="),
            "{output}"
        );
    }
}

#[test]
fn normalized_links_keep_tracking_cleanup_and_reference_metadata() {
    let input = r#"<a href="//shop.example/order?id=7&amp;utm_source=mail">Order</a>"#;
    let output = sanitize_email_html_reader(input, &SanitizeOptions::READER);
    assert!(
        output.contains(r#"href="https://shop.example/order?id=7""#),
        "{output}"
    );
    assert!(
        output.contains(r#"data-mw-link-original="//shop.example/order?id=7&amp;utm_source=mail""#),
        "{output}"
    );
    let (_, references) =
        mw_sanitize::sanitize_email_html_and_references(input, &SanitizeOptions::READER);
    assert_eq!(references.links, vec!["https://shop.example/order?id=7"]);
}

#[test]
fn link_preservation_never_admits_executable_or_unresolved_navigation() {
    for href in [
        "javascript:alert(1)",
        "java&#9;script:alert(1)",
        "vbscript:evil",
        "data:text/html,evil",
        "file:///private",
        "blob:https://example.org/id",
        "/account",
        "../invoice",
        "?invoice=7",
        "//",
        "//example.org:invalid/path",
        "//example.org/&#9;unsafe",
        "//example.org/\\unsafe",
    ] {
        let input = format!(r#"<a href="{href}">Read</a>"#);
        let output = sanitize_email_html_reader_safe(&input, &SanitizeOptions::READER);
        assert!(!output.contains("href="), "{href}: {output}");
    }
}
