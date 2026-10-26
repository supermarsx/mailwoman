//! Bounded DAV listing parser. No DTDs, external entities or namespace-prefix assumptions.
use quick_xml::{Reader, events::Event};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Entry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<String>,
    pub content_type: Option<String>,
}
#[derive(Default)]
struct Item {
    href: String,
    size: String,
    modified: String,
    content_type: String,
    collection: bool,
}

fn decode_path(value: &str) -> Result<String, &'static str> {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let pair = bytes.get(i + 1..i + 3).ok_or("invalid DAV escape")?;
            let pair = std::str::from_utf8(pair).map_err(|_| "invalid DAV escape")?;
            out.push(u8::from_str_radix(pair, 16).map_err(|_| "invalid DAV escape")?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| "invalid DAV path encoding")
}

pub(crate) fn entries(
    xml: &[u8],
    root: &reqwest::Url,
    requested: &reqwest::Url,
) -> Result<Vec<Entry>, &'static str> {
    if xml.len() > 2 * 1024 * 1024 {
        return Err("DAV listing too large");
    }
    let root_path = decode_path(root.path())?;
    let requested_path = decode_path(requested.path())?;
    let mut reader = Reader::from_reader(xml);
    let mut stack: Vec<Vec<u8>> = Vec::new();
    let mut current = None::<Item>;
    let mut output = Vec::new();
    let mut responses = 0;
    let mut multistatus = false;
    loop {
        match reader.read_event().map_err(|_| "invalid DAV XML")? {
            Event::DocType(_) => return Err("DAV DTD refused"),
            Event::Start(tag) => {
                if stack.len() >= 32 {
                    return Err("DAV XML too deep");
                }
                let name = tag.local_name().as_ref().to_vec();
                if stack.is_empty() {
                    if name != b"multistatus" {
                        return Err("expected DAV multistatus");
                    }
                    multistatus = true;
                }
                if name == b"response" {
                    if current.is_some() {
                        return Err("nested DAV response");
                    }
                    responses += 1;
                    if responses > 1001 {
                        return Err("DAV listing has too many entries");
                    }
                    current = Some(Item::default());
                }
                if name == b"collection"
                    && let Some(item) = &mut current
                {
                    item.collection = true;
                }
                stack.push(name);
            }
            Event::Empty(tag) => {
                if tag.local_name().as_ref() == b"collection"
                    && let Some(item) = &mut current
                {
                    item.collection = true;
                }
            }
            Event::Text(text) => {
                let raw = text.decode().map_err(|_| "invalid DAV text")?;
                let value = quick_xml::escape::unescape(&raw).map_err(|_| "invalid DAV entity")?;
                append(&mut current, stack.last().map(Vec::as_slice), &value)?;
            }
            Event::GeneralRef(reference) => {
                if let Some(character) = reference
                    .resolve_char_ref()
                    .map_err(|_| "invalid DAV character reference")?
                {
                    append(
                        &mut current,
                        stack.last().map(Vec::as_slice),
                        &character.to_string(),
                    )?;
                } else {
                    let name = reference.decode().map_err(|_| "invalid DAV entity")?;
                    let value = quick_xml::escape::resolve_predefined_entity(&name)
                        .ok_or("DAV entity refused")?;
                    append(&mut current, stack.last().map(Vec::as_slice), value)?;
                }
            }
            Event::CData(text) => {
                let value = text.decode().map_err(|_| "invalid DAV text")?;
                append(&mut current, stack.last().map(Vec::as_slice), &value)?;
            }
            Event::End(tag) => {
                if stack.pop().as_deref() != Some(tag.local_name().as_ref()) {
                    return Err("invalid DAV nesting");
                }
                if tag.local_name().as_ref() == b"response" {
                    let item = current.take().ok_or("invalid DAV response")?;
                    let url = root
                        .join(item.href.trim())
                        .map_err(|_| "invalid DAV href")?;
                    if url.origin() != root.origin()
                        || !url.username().is_empty()
                        || url.password().is_some()
                        || url.query().is_some()
                        || url.fragment().is_some()
                    {
                        return Err("DAV href escaped instance");
                    }
                    let path = decode_path(url.path())?;
                    if path.trim_end_matches('/') == requested_path.trim_end_matches('/') {
                        continue;
                    }
                    let relative = path
                        .strip_prefix(&root_path)
                        .ok_or("DAV href escaped account")?;
                    if !path.starts_with(&format!("{}/", requested_path.trim_end_matches('/'))) {
                        return Err("DAV href escaped collection");
                    }
                    if relative.split('/').any(|part| matches!(part, "." | ".."))
                        || relative.contains('\\')
                        || relative.chars().any(char::is_control)
                    {
                        return Err("invalid DAV href path");
                    }
                    output.push(Entry {
                        name: relative
                            .trim_end_matches('/')
                            .rsplit('/')
                            .next()
                            .unwrap_or_default()
                            .into(),
                        path: format!("/{relative}"),
                        is_dir: item.collection,
                        size: item.size.trim().parse().unwrap_or(0),
                        modified: (!item.modified.is_empty()).then_some(item.modified),
                        content_type: (!item.content_type.is_empty()).then_some(item.content_type),
                    });
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !multistatus || responses == 0 || !stack.is_empty() || current.is_some() {
        return Err("truncated DAV XML");
    }
    Ok(output)
}
fn append(item: &mut Option<Item>, tag: Option<&[u8]>, value: &str) -> Result<(), &'static str> {
    let Some(item) = item else {
        return Ok(());
    };
    let target = match tag {
        Some(b"href") => &mut item.href,
        Some(b"getcontentlength") => &mut item.size,
        Some(b"getlastmodified") => &mut item.modified,
        Some(b"getcontenttype") => &mut item.content_type,
        _ => return Ok(()),
    };
    if target.len() + value.len() > 8192 {
        return Err("DAV property too large");
    }
    target.push_str(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn namespaces_unicode_and_account_confinement() {
        let root: reqwest::Url = "https://cloud.test/install/remote.php/dav/files/alice/"
            .parse()
            .unwrap();
        let xml=br#"<multistatus xmlns="DAV:"><response><href>/install/remote.php/dav/files/alice/R%C3%A9sum%C3%A9%20%26%20notes</href><propstat><prop><getcontentlength>12</getcontentlength></prop></propstat></response></multistatus>"#;
        let found = entries(xml, &root, &root).unwrap();
        assert_eq!(found[0].name, "Résumé & notes");
        assert_eq!(found[0].size, 12);
        for href in [
            "https://evil.test/secret",
            "/install/remote.php/dav/files/alice2/private",
            "/install/remote.php/dav/files/alice/%2e%2e/bob/private",
        ] {
            let xml = format!(
                "<d:multistatus xmlns:d='DAV:'><d:response><d:href>{href}</d:href></d:response></d:multistatus>"
            );
            assert!(entries(xml.as_bytes(), &root, &root).is_err());
        }
        assert!(
            entries(
                b"<!DOCTYPE x [<!ENTITY secret SYSTEM 'file:///etc/passwd'>]><x/>",
                &root,
                &root
            )
            .is_err()
        );
    }
}
