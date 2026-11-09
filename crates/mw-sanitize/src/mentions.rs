//! Portable addressed mentions, independent of recipient headers and resource loading.
use html5ever::{parse_document, tendril::TendrilSink};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::collections::BTreeSet;

fn canonical(raw: &str) -> Option<String> {
    let value = raw.trim();
    if value.len() > 320 || value.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return None;
    }
    let (local, domain) = value.split_once('@')?;
    if local.is_empty()
        || domain.is_empty()
        || domain.contains('@')
        || !domain.contains('.')
        || local
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && !".!#$%&'*+-/=?^_`{|}~".contains(c))
        || domain
            .chars()
            .any(|c| !c.is_alphanumeric() && c != '.' && c != '-')
    {
        return None;
    }
    if domain
        .split('.')
        .any(|part| part.is_empty() || part.starts_with('-') || part.ends_with('-'))
    {
        return None;
    }
    Some(value.to_lowercase())
}
fn decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let pair = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(pair, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}
fn mailto(raw: &str) -> Option<String> {
    let (scheme, path) = raw.trim().split_once(':')?;
    if !scheme.eq_ignore_ascii_case("mailto") {
        return None;
    }
    canonical(&decode(path.split(['?', '#']).next()?)?)
}
fn explicit(text: &str, out: &mut BTreeSet<String>) {
    for (at, c) in text.char_indices() {
        if c != '@' {
            continue;
        }
        if text[..at]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || "._%+-".contains(c))
        {
            continue;
        }
        let tail = &text[at + 1..];
        let end = tail
            .find(|c: char| c.is_whitespace() || "<>,;()[]{}\"'".contains(c))
            .unwrap_or(tail.len());
        if let Some(value) = canonical(tail[..end].trim_end_matches(['.', '!', '?', ':'])) {
            out.insert(value);
        }
        if out.len() >= 256 {
            break;
        }
    }
}
fn authored(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        let lower = line.trim().to_lowercase();
        if line == "-- "
            || lower == "--"
            || (lower.starts_with("on ") && lower.ends_with("wrote:"))
            || (lower.starts_with("em ") && lower.ends_with("escreveu:"))
            || lower.starts_with("-----original message-----")
        {
            break;
        }
        if !line.trim_start().starts_with('>') {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}
fn text(node: &Handle, out: &mut String, depth: usize) {
    if depth > 128 {
        return;
    }
    if let NodeData::Text { contents } = &node.data {
        out.push_str(&contents.borrow());
    }
    for child in node.children.borrow().iter() {
        text(child, out, depth + 1);
    }
}
fn walk(
    node: &Handle,
    own: &mut String,
    out: &mut BTreeSet<String>,
    visits: &mut usize,
    depth: usize,
) {
    *visits += 1;
    if *visits > 100_000 || depth > 128 || out.len() >= 256 {
        return;
    }
    match &node.data {
        NodeData::Text { contents } => own.push_str(&contents.borrow()),
        NodeData::Element { name, attrs, .. } => {
            let tag = name.local.as_ref();
            let attrs = attrs.borrow();
            if ["blockquote", "script", "style", "head", "template"].contains(&tag)
                || attrs.iter().any(|a| {
                    a.name.local.starts_with("data-mw-signature")
                        || a.name.local.as_ref() == "class"
                            && a.value.split_whitespace().any(|c| {
                                ["gmail_quote", "gmail_signature", "moz-signature"].contains(&c)
                            })
                })
            {
                return;
            }
            if tag == "a" {
                if let Some(href) = attrs.iter().find(|a| a.name.local.as_ref() == "href") {
                    let mut label = String::new();
                    text(node, &mut label, 0);
                    if label.trim().starts_with('@') {
                        if let Some(address) = mailto(&href.value) {
                            out.insert(address);
                        }
                    }
                }
            }
            drop(attrs);
            for child in node.children.borrow().iter() {
                walk(child, own, out, visits, depth + 1);
            }
            if ["p", "div", "br", "li", "tr"].contains(&tag) {
                own.push('\n');
            }
        }
        _ => {
            for child in node.children.borrow().iter() {
                walk(child, own, out, visits, depth + 1);
            }
        }
    }
}
/// Addresses explicitly mentioned in the authored HTML (or plaintext when HTML
/// is absent). This never infers a mention from To/Cc, a bare name, or quoted mail.
pub fn mentioned_addresses(html: &str, plain: &str) -> Vec<String> {
    if html.len().max(plain.len()) > 8 * 1024 * 1024 {
        return vec![];
    }
    let mut out = BTreeSet::new();
    if html.is_empty() {
        explicit(&authored(plain), &mut out);
    } else {
        let dom = parse_document(RcDom::default(), Default::default()).one(html);
        let mut own = String::new();
        walk(&dom.document, &mut own, &mut out, &mut 0, 0);
        explicit(&authored(&own), &mut out);
    }
    out.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portable_structured_and_plain_mentions_only() {
        assert_eq!(
            mentioned_addresses(
                "<p><a href='mailto:Jane%40example.org'><strong>@Jane</strong> Doe</a> @alex@example.org.</p><a href='mailto:other@example.org'>Other</a>",
                ""
            ),
            ["alex@example.org", "jane@example.org"]
        );
        assert_eq!(
            mentioned_addresses(
                "",
                "Hello @alias@example.org and jane@example.org\n> @quote@example.org\n-- \n@sig@example.org"
            ),
            ["alias@example.org"]
        );
    }
    #[test]
    fn quotes_resources_and_malformed_mailto_do_not_become_mentions() {
        let html = "<blockquote><a href='mailto:q@example.org'>@Q</a></blockquote><div class='gmail_signature'>@sig@example.org</div><script>@script@example.org</script><a href='javascript:alert(1)'>@Bad</a><a href='mailto:a@example.org,b@example.org'>@Two</a><img src='https://remote.invalid/@image@example.org'><p>ordinary jane@example.org</p>";
        assert!(mentioned_addresses(html, "").is_empty());
    }
    #[test]
    fn explicit_mentions_keep_plus_aliases_and_domain_boundaries() {
        assert_eq!(
            mentioned_addresses("", "(@Jane+work@example.org), @jane@example.org.evil!"),
            ["jane+work@example.org", "jane@example.org.evil"]
        );
    }
}
