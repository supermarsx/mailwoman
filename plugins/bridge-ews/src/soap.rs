//! A tiny SOAP-1.1 envelope builder + a leaf-oriented XML reader over the
//! workspace `quick-xml` (0.41). EWS is a large schema; the bridge only needs to
//! (a) wrap a request body in the Exchange SOAP envelope and (b) pull specific leaf
//! texts / attributes out of responses. Keeping the extraction local-name-based
//! (namespace-prefix-insensitive) makes it robust to `t:`/`m:`/default-namespace
//! variation across Exchange 2013–2019/SE.

use quick_xml::events::Event;
use quick_xml::reader::Reader;

/// EWS SOAP namespaces (MS-OXWSCDATA). `m:` = messages, `t:` = types.
pub const NS_SOAP: &str = "http://schemas.xmlsoap.org/soap/envelope/";
pub const NS_MESSAGES: &str = "http://schemas.microsoft.com/exchange/services/2006/messages";
pub const NS_TYPES: &str = "http://schemas.microsoft.com/exchange/services/2006/types";

/// Wrap an EWS request `body` (the `<m:...>` operation element) in a SOAP envelope
/// with a `RequestServerVersion` header targeting Exchange 2013+ (`Exchange2013`).
#[must_use]
pub fn envelope(body: &str) -> String {
    format!(
        concat!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>",
            "<soap:Envelope xmlns:soap=\"{soap}\" xmlns:m=\"{m}\" xmlns:t=\"{t}\">",
            "<soap:Header>",
            "<t:RequestServerVersion Version=\"Exchange2013\"/>",
            "</soap:Header>",
            "<soap:Body>{body}</soap:Body>",
            "</soap:Envelope>"
        ),
        soap = NS_SOAP,
        m = NS_MESSAGES,
        t = NS_TYPES,
        body = body
    )
}

/// XML-escape a text value for inclusion in an element body.
#[must_use]
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

fn local_name_matches(raw: &[u8], want: &str) -> bool {
    // Strip an optional `prefix:` from the qualified name and compare the local part.
    let name = match raw.iter().position(|&b| b == b':') {
        Some(i) => &raw[i + 1..],
        None => raw,
    };
    name == want.as_bytes()
}

/// The SOAP `ResponseClass` of the first response message — `"Success"`,
/// `"Warning"`, or `"Error"`. Returns `None` if absent.
#[must_use]
pub fn response_class(xml: &str) -> Option<String> {
    // ResponseClass is an attribute on `*ResponseMessage` elements.
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let local = e.local_name();
                if local.as_ref().ends_with(b"ResponseMessage") {
                    for a in e.attributes().flatten() {
                        if a.key.local_name().as_ref() == b"ResponseClass" {
                            return Some(String::from_utf8_lossy(&a.value).into_owned());
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    None
}

/// Whether the response reports at least one `ResponseClass="Success"`.
#[must_use]
pub fn is_success(xml: &str) -> bool {
    response_class(xml).as_deref() == Some("Success")
}

/// The `MessageText` of the first error/warning response message, if any.
#[must_use]
pub fn message_text(xml: &str) -> Option<String> {
    first_text(xml, "MessageText")
}

/// Collect the leaf text of every element whose local name equals `local`.
#[must_use]
pub fn texts_of(xml: &str, local: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut out = Vec::new();
    let mut depth = 0i32; // >0 while inside a matching element
    let mut acc = String::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if local_name_matches(e.name().as_ref(), local) => {
                depth += 1;
                if depth == 1 {
                    acc.clear();
                }
            }
            Ok(Event::Text(t)) if depth > 0 => {
                acc.push_str(&t.decode().unwrap_or_default());
            }
            Ok(Event::CData(t)) if depth > 0 => {
                acc.push_str(&String::from_utf8_lossy(t.as_ref()));
            }
            Ok(Event::End(e)) if local_name_matches(e.name().as_ref(), local) => {
                if depth == 1 {
                    out.push(std::mem::take(&mut acc));
                }
                depth -= 1;
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    out
}

/// The leaf text of the first element whose local name equals `local`.
#[must_use]
pub fn first_text(xml: &str, local: &str) -> Option<String> {
    texts_of(xml, local).into_iter().next()
}

/// Collect the value of attribute `attr` on every (start/empty) element whose local
/// name equals `local`.
#[must_use]
pub fn attrs_of(xml: &str, local: &str, attr: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut out = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e))
                if local_name_matches(e.name().as_ref(), local) =>
            {
                for a in e.attributes().flatten() {
                    if a.key.local_name().as_ref() == attr.as_bytes() {
                        out.push(String::from_utf8_lossy(&a.value).into_owned());
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    out
}

/// The `attr` value of the first element whose local name equals `local`.
#[must_use]
pub fn first_attr(xml: &str, local: &str, attr: &str) -> Option<String> {
    attrs_of(xml, local, attr).into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_wraps_body() {
        let e = envelope("<m:Ping/>");
        assert!(e.contains("<soap:Body><m:Ping/></soap:Body>"));
        assert!(e.contains("Exchange2013"));
    }

    #[test]
    fn escape_encodes_markup() {
        assert_eq!(escape("a<b>&\"'"), "a&lt;b&gt;&amp;&quot;&apos;");
    }

    #[test]
    fn texts_and_attrs_are_prefix_insensitive() {
        let xml = r#"<r xmlns:t="x"><t:Subject>Hi</t:Subject>
            <t:ItemId Id="AAA" ChangeKey="CK1"/><ItemId Id="BBB"/></r>"#;
        assert_eq!(first_text(xml, "Subject").as_deref(), Some("Hi"));
        assert_eq!(attrs_of(xml, "ItemId", "Id"), vec!["AAA", "BBB"]);
        assert_eq!(
            first_attr(xml, "ItemId", "ChangeKey").as_deref(),
            Some("CK1")
        );
    }

    #[test]
    fn response_class_and_success() {
        let ok = r#"<m:GetItemResponseMessage ResponseClass="Success"/>"#;
        assert_eq!(response_class(ok).as_deref(), Some("Success"));
        assert!(is_success(ok));
        let err = r#"<m:GetItemResponseMessage ResponseClass="Error">
            <m:MessageText>boom</m:MessageText></m:GetItemResponseMessage>"#;
        assert!(!is_success(err));
        assert_eq!(message_text(err).as_deref(), Some("boom"));
    }
}
