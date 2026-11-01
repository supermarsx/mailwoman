use mw_sanitize::{SanitizeOptions, sanitize_email_html_reader, sanitize_email_html_reader_safe};

#[test]
fn preserves_legacy_email_table_spacing_dimensions_and_fonts() {
    let input = r##"<table width="600" cellpadding="20" cellspacing="0" border="0" bgcolor="#fefefe" align="center"><tr><td width="300" height="50" align="right" valign="middle" bgcolor="#ffeecc"><font face="Arial, sans-serif" color="#cc0000" size="4">Readable email</font></td></tr></table>"##;
    for opts in [SanitizeOptions::READER, SanitizeOptions::STRIP] {
        for out in [
            sanitize_email_html_reader(input, &opts),
            sanitize_email_html_reader_safe(input, &opts),
        ] {
            for expected in [
                "width=\"600\"",
                "cellpadding=\"20\"",
                "cellspacing=\"0\"",
                "border=\"0\"",
                "bgcolor=\"#fefefe\"",
                "width=\"300\"",
                "height=\"50\"",
                "valign=\"middle\"",
                "bgcolor=\"#ffeecc\"",
                "<font",
                "face=\"Arial, sans-serif\"",
                "color=\"#cc0000\"",
                "size=\"4\"",
            ] {
                assert!(out.contains(expected), "missing {expected}: {out}");
            }
            assert!(!out.contains("data-mw-body-blocked"), "{out}");
        }
    }
}

#[test]
fn preserves_logical_spacing_direction_and_safe_font_properties() {
    let input = r#"<style>.content{margin-block:12px;padding-inline:24px;max-inline-size:600px;font-variant-numeric:tabular-nums}</style><div class="content" dir="rtl" style="margin-inline:auto;padding-block:8px;font-kerning:normal;text-decoration-thickness:2px">Readable email</div>"#;
    let out = sanitize_email_html_reader_safe(input, &SanitizeOptions::READER);
    for expected in [
        "dir=\"rtl\"",
        "margin-block:12px",
        "padding-inline:24px",
        "max-inline-size:600px",
        "font-variant-numeric:tabular-nums",
        "margin-inline:auto",
        "padding-block:8px",
        "font-kerning:normal",
        "text-decoration-thickness:2px",
    ] {
        assert!(out.contains(expected), "missing {expected}: {out}");
    }
    assert!(out.contains(".mw-email-body .content{"), "{out}");
}

#[test]
fn preserves_real_document_root_fonts_and_spacing_on_the_scoped_container() {
    let input = r##"<!doctype html><html lang="en" style="font-family:Georgia;font-size:18px"><head><style>html body .content{padding:24px}html > body{line-height:1.4}:root body.content{margin:0}</style></head><body class="content" id="newsletter" dir="rtl" bgcolor="#eee" style="padding:24px;font-size:20px;background-image:url(https://images.test/bg.png)"><p>Root formatting</p></body></html>"##;
    let out = sanitize_email_html_reader_safe(input, &SanitizeOptions::READER);
    for expected in [
        "class=\"mw-email-body content\"",
        "lang=\"en\"",
        "dir=\"rtl\"",
        "id=\"newsletter\"",
        "font-family:Georgia",
        "font-size:18px",
        "font-size:20px",
        "padding:24px",
        "background-color:#eee",
        "background-image:url(mw-rc:",
        ".mw-email-body .content{padding:24px}",
        ".mw-email-body{line-height:1.4}",
        ".mw-email-body.content{margin:0}",
    ] {
        assert!(out.contains(expected), "missing {expected}: {out}");
    }
    assert!(!out.contains(".mw-email-body body"), "{out}");
    assert!(!out.contains("url(https:"), "{out}");
    assert!(
        out.contains("data-mw-rc-url=\"https://images.test/bg.png\""),
        "{out}"
    );
}

#[test]
fn document_root_normalization_does_not_promote_fake_body_tags_or_active_attributes() {
    let input = r#"<html><head><script>const text='<body style="font-family:evil" onload="evil()">';</script></head><body style="font-family:Georgia;position:fixed;background:url(javascript:evil())" onload="evil()"><p>Readable text</p><svg><body style="font-family:foreign"/></svg></body></html>"#;
    let out = sanitize_email_html_reader_safe(input, &SanitizeOptions::READER);
    assert!(out.contains("font-family:Georgia"), "{out}");
    for forbidden in [
        "font-family:evil",
        "font-family:foreign",
        "onload",
        "position:fixed",
        "javascript:",
        "<script",
        "<svg",
    ] {
        assert!(!out.contains(forbidden), "retained {forbidden}: {out}");
    }
    assert!(
        sanitize_email_html_reader(input, &SanitizeOptions::READER)
            .starts_with("<pre data-mw-body-blocked=\"true\">")
    );
}

#[test]
fn preserving_layout_does_not_admit_active_content_or_bypass_remote_policy() {
    let input = r#"<table width="600" background="https://images.test/background.png" onclick="evil()"><tr><td cellpadding="20"><font face="Arial" onclick="evil()">Text</font><img src="https://images.test/logo.png" border="0" hspace="8" onerror="evil()"><a href="javascript:evil()">Unsafe link</a><form><input name="secret"></form><span style="padding-inline:expression(evil());position:fixed">Text</span></td></tr></table>"#;
    assert!(
        sanitize_email_html_reader(input, &SanitizeOptions::READER)
            .starts_with("<pre data-mw-body-blocked=\"true\">")
    );
    for opts in [SanitizeOptions::READER, SanitizeOptions::STRIP] {
        let out = sanitize_email_html_reader_safe(input, &opts);
        for forbidden in [
            "onclick",
            "onerror",
            "javascript:",
            "expression(",
            "position:fixed",
            "<script",
            "<form",
            "<input",
            "src=\"https:",
            "background=\"https:",
        ] {
            assert!(!out.contains(forbidden), "retained {forbidden}: {out}");
        }
        assert!(out.contains("width=\"600\""), "{out}");
        assert!(out.contains("face=\"Arial\""), "{out}");
        assert!(out.contains("hspace=\"8\""), "{out}");
        if opts == SanitizeOptions::READER {
            assert!(out.contains("mw-rc:"), "{out}");
        } else {
            assert!(!out.contains("mw-rc:"), "{out}");
        }
    }
}
