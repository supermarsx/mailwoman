//! Repeatable sanitiser timing harness (not a CI gate).
//! `cargo test -p mw-sanitize --release --test bench -- --ignored --nocapture`
use mw_sanitize::{SanitizeOptions, sanitize_email_html, sanitize_email_html_reader};
use std::time::Instant;

pub fn fixtures() -> Vec<(&'static str, String)> {
    let small = "<p>Hi Ana,</p><p>Lunch at noon? <a href=\"https://example.com/?utm_source=x\">menu</a></p><p>-- <br>Bo</p>".to_string();
    let mut news = String::from("<html><head><style>.hd{color:#333;font-size:14px}td{padding:4px}a{color:#06c}</style></head><body style=\"margin:0\">");
    while news.len() < 200_000 {
        news.push_str("<table width=\"600\" cellpadding=\"0\" cellspacing=\"0\" style=\"border-collapse:collapse;background:#fff\"><tr><td class=\"hd\" style=\"padding:8px 12px;font-family:Arial,sans-serif\"><table><tr><td><span style=\"font-size:14px\"><font face=\"Arial\"><span>Offer <b>today</b> only</span></font></span> <a href=\"https://shop.example.com/p?id=3&utm_campaign=z\" style=\"color:#06c;text-decoration:none\">Buy</a><img src=\"https://cdn.example.com/i.png\" width=\"100\" height=\"50\" style=\"display:block\"></td></tr></table></td></tr></table>\n");
    }
    news.push_str("</body></html>");
    let mut big = String::from("<html><body>");
    while big.len() < 2_000_000 {
        big.push_str("<div style=\"padding:10px;margin:0 auto;color:#222\"><p style=\"margin:0\"><span style=\"font-weight:bold\">Sale</span> <a href=\"https://m.example.com/x?utm_medium=e\">now</a> &nbsp; text text text text</p><img src=\"https://t.example.com/p.gif\" width=\"1\" height=\"1\"></div>\n");
    }
    big.push_str("</body></html>");
    let mut mso = String::from("<html xmlns:o=\"urn:schemas-microsoft-com:office:office\"><head><!--[if gte mso 9]><xml><o:OfficeDocumentSettings/></xml><![endif]--><style>p.MsoNormal{margin:0cm;font-size:11pt}</style></head><body><div class=\"WordSection1\">");
    for i in 0..400 {
        mso.push_str(&format!("<p class=\"MsoNormal\"><span style=\"font-size:11.0pt;font-family:Calibri,sans-serif;mso-fareast-language:EN-US\">Line {i} <o:p></o:p></span></p><p class=\"MsoNormal\"><o:p>&nbsp;</o:p></p>"));
    }
    mso.push_str("</div></body></html>");
    let mut quotes = String::from("<p>top</p>");
    for i in 0..60 { quotes.push_str(&format!("<blockquote type=\"cite\" style=\"margin:0 0 0 .8ex;border-left:1px #ccc solid\"><p>wrote {i}:</p>")); }
    for _ in 0..60 { quotes.push_str("</blockquote>"); }
    let mut attrs = String::from("<div>");
    for i in 0..3000 { attrs.push_str(&format!("<span id=\"a{i}\" class=\"c{i} d e f\" title=\"t{i}\" lang=\"en\" dir=\"ltr\" style=\"color:red;background:#fff;margin:1px 2px 3px 4px;padding:0;font:12px/1 Arial\" data-x=\"{i}\" onclick=\"x()\">v</span>")); }
    attrs.push_str("</div>");
    let mut nest = String::new();
    for _ in 0..5000 { nest.push_str("<div><span>"); }
    nest.push('x');
    for _ in 0..5000 { nest.push_str("</span></div>"); }
    let mut nest1000 = String::new();
    for _ in 0..1000 { nest1000.push_str("<div><span>"); }
    nest1000.push('x');
    for _ in 0..1000 { nest1000.push_str("</span></div>"); }
    vec![("small", small), ("newsletter200k", news), ("marketing2m", big), ("outlook", mso), ("quotes", quotes), ("attrs", attrs), ("nest5000", nest), ("nest1000", nest1000)]
}

fn pct(v: &mut [f64], p: f64) -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[((v.len() as f64 - 1.0) * p).round() as usize] }

#[test]
#[ignore]
fn bench() {
    for (name, html) in fixtures() {
        let n = if html.len() > 1_000_000 { 5 } else { 20 };
        let mut s = Vec::new(); let mut r = Vec::new(); let mut out = 0;
        for _ in 0..n {
            let t = Instant::now(); let o = sanitize_email_html(&html); s.push(t.elapsed().as_secs_f64() * 1e3); out = o.len();
            let t = Instant::now(); let _ = sanitize_email_html_reader(&html, &SanitizeOptions::READER); r.push(t.elapsed().as_secs_f64() * 1e3);
        }
        println!("BENCH {name:16} in={:>8} out={:>8} sanitize p50={:7.2}ms p95={:7.2}ms | reader p50={:7.2}ms p95={:7.2}ms", html.len(), out, pct(&mut s, 0.5), pct(&mut s, 0.95), pct(&mut r, 0.5), pct(&mut r, 0.95));
    }
}
