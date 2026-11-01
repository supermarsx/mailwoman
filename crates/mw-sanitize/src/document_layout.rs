//! Preserve passive document-root presentation that fragment sanitization loses.
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};

use super::{CONTAINER_CLASS, Recorded, escape_attr, sanitize_declaration_list_with};

fn element_attributes(node: &Handle, tag: &str) -> Option<Vec<(String, String)>> {
    let NodeData::Element { name, attrs, .. } = &node.data else {
        return None;
    };
    if name.ns.as_ref() != "http://www.w3.org/1999/xhtml" || name.local.as_ref() != tag {
        return None;
    }
    Some(
        attrs
            .borrow()
            .iter()
            .filter(|attr| attr.name.ns.is_empty())
            .map(|attr| (attr.name.local.to_string(), attr.value.to_string()))
            .collect(),
    )
}

/// Both `html` and `body` selectors already map to the message container. Retain
/// their inherited presentation there too, with body attributes taking priority.
/// A real document parse avoids treating strings in scripts/comments/foreign
/// content as a document root. No content from this parse bypasses ammonia.
pub(super) fn container_attributes(input: &str, rec: &mut Recorded) -> Option<String> {
    if !super::contains_ignore_ascii_case(input, "<body")
        && !super::contains_ignore_ascii_case(input, "<html")
    {
        return None;
    }
    let dom = html5ever::parse_document(RcDom::default(), Default::default()).one(input);
    let html = dom
        .document
        .children
        .borrow()
        .iter()
        .find(|node| element_attributes(node, "html").is_some())
        .cloned()?;
    let body = html
        .children
        .borrow()
        .iter()
        .find(|node| element_attributes(node, "body").is_some())
        .cloned();
    let mut classes = String::new();
    let mut styles = String::new();
    let mut passive = std::collections::BTreeMap::new();
    for attrs in [
        element_attributes(&html, "html"),
        body.as_ref()
            .and_then(|node| element_attributes(node, "body")),
    ]
    .into_iter()
    .flatten()
    {
        // Legacy body paint is a presentation hint, overridden by inline CSS.
        for (name, value) in &attrs {
            let property = match name.as_str() {
                "bgcolor" => "background-color",
                "text" => "color",
                _ => continue,
            };
            let safe = sanitize_declaration_list_with(&format!("{property}:{value}"), Some(rec));
            if !safe.is_empty() {
                styles.push_str(&safe);
                styles.push(';');
            }
        }
        for (name, value) in attrs {
            match name.as_str() {
                "class" => {
                    classes.push(' ');
                    classes.push_str(&value);
                }
                "style" => {
                    let safe = sanitize_declaration_list_with(&value, Some(rec));
                    if !safe.is_empty() {
                        styles.push_str(&safe);
                        styles.push(';');
                    }
                }
                "id" | "lang" | "dir" => {
                    passive.insert(name, value);
                }
                _ => {}
            }
        }
    }
    if classes.is_empty() && styles.is_empty() && passive.is_empty() {
        return None;
    }
    let mut attrs = format!("class=\"{CONTAINER_CLASS}{}\"", escape_attr(&classes));
    if !styles.is_empty() {
        attrs.push_str(&format!(
            " style=\"{}\"",
            escape_attr(styles.trim_end_matches(';'))
        ));
    }
    for (name, value) in passive {
        attrs.push_str(&format!(" {name}=\"{}\"", escape_attr(&value)));
    }
    Some(attrs)
}
