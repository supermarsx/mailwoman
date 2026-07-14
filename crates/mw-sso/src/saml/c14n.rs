//! Exclusive XML Canonicalization (exc-C14N, W3C REC `xml-exc-c14n#`) — the top
//! risk of the hand-rolled SAML SP (plan §9 R1).
//!
//! XML-DSig signs the *canonical byte form* of an element, not its serialized
//! bytes. To validate a signature we must reproduce, byte-for-byte, the exact octet
//! stream the IdP signed. Exclusive canonicalization (as opposed to inclusive
//! C14N 1.0) is the SAML/xmldsig default: an element renders a namespace
//! declaration **only** when the prefix is *visibly utilized* by the element or one
//! of its attributes (or is named in an `InclusiveNamespaces` `PrefixList`), instead
//! of dragging every in-scope declaration down the tree. Getting this wrong makes
//! every real-IdP signature fail.
//!
//! This module parses XML into a small faithful DOM (preserving prefixes, attribute
//! values, text, and per-element namespace declarations) and emits the exclusive
//! canonical form. It is intentionally scoped to the SAML subset: elements, text,
//! CDATA (rendered as text), attributes, and namespace nodes; comments and
//! processing instructions are dropped (exc-C14N **without** comments, the xmldsig
//! default).
//!
//! References: W3C "Exclusive XML Canonicalization 1.0" + "Canonical XML 1.0".

use std::collections::{BTreeMap, BTreeSet};

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::SsoError;

/// The implicit binding of the reserved `xml` prefix (never emitted as a
/// declaration — it is well-known to every XML processor).
const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";

/// A non-namespace attribute (namespace declarations are held separately).
#[derive(Debug, Clone)]
pub struct Attr {
    /// Namespace prefix (`""` = unprefixed / no namespace).
    pub prefix: String,
    /// Local name.
    pub local: String,
    /// Attribute value with entity/char references already expanded.
    pub value: String,
}

/// A DOM child: a nested element or a run of character data.
#[derive(Debug, Clone)]
pub enum Node {
    /// A child element.
    Element(Element),
    /// Character data (already unescaped; re-escaped per C14N on render).
    Text(String),
}

/// A parsed XML element with everything canonicalization needs.
#[derive(Debug, Clone)]
pub struct Element {
    /// Stable id assigned at parse (used to omit the enveloped `Signature`).
    pub nid: u32,
    /// Element name prefix (`""` = default namespace / unprefixed).
    pub prefix: String,
    /// Element local name.
    pub local: String,
    /// Namespace declarations that appear *on* this element: `(prefix, uri)` with
    /// `prefix == ""` for the default namespace and `uri == ""` for an undeclaration
    /// (`xmlns=""`).
    pub ns_decls: Vec<(String, String)>,
    /// Non-namespace attributes.
    pub attrs: Vec<Attr>,
    /// Full in-scope namespace map (inherited ancestors + this element's own
    /// declarations), keyed by prefix (`""` = default). Used to resolve prefix →
    /// URI during canonicalization.
    pub scope: BTreeMap<String, String>,
    /// Child nodes in document order.
    pub children: Vec<Node>,
}

impl Element {
    /// The resolved namespace URI of this element's own name.
    pub fn ns_uri(&self) -> &str {
        self.scope
            .get(&self.prefix)
            .map(String::as_str)
            .unwrap_or("")
    }

    /// First attribute with the given local name (any/no prefix), unescaped value.
    pub fn attr(&self, local: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|a| a.local == local)
            .map(|a| a.value.as_str())
    }

    /// Iterate direct child elements.
    pub fn child_elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|n| match n {
            Node::Element(e) => Some(e),
            Node::Text(_) => None,
        })
    }

    /// First direct child element matching `(ns_uri, local)`.
    pub fn child(&self, ns_uri: &str, local: &str) -> Option<&Element> {
        self.child_elements()
            .find(|e| e.local == local && e.ns_uri() == ns_uri)
    }

    /// First descendant (self excluded, depth-first) matching `(ns_uri, local)`.
    pub fn descendant(&self, ns_uri: &str, local: &str) -> Option<&Element> {
        for c in self.child_elements() {
            if c.local == local && c.ns_uri() == ns_uri {
                return Some(c);
            }
            if let Some(found) = c.descendant(ns_uri, local) {
                return Some(found);
            }
        }
        None
    }

    /// All direct children matching `(ns_uri, local)`.
    pub fn children_named<'a>(&'a self, ns_uri: &'a str, local: &'a str) -> Vec<&'a Element> {
        self.child_elements()
            .filter(|e| e.local == local && e.ns_uri() == ns_uri)
            .collect()
    }

    /// Concatenated text of all direct text children, trimmed of surrounding ASCII
    /// whitespace (SAML leaf values — `DigestValue`, `NameID`, attribute values — are
    /// whitespace-insignificant single tokens).
    pub fn text(&self) -> String {
        let mut s = String::new();
        for n in &self.children {
            if let Node::Text(t) = n {
                s.push_str(t);
            }
        }
        s.trim().to_string()
    }

    /// Depth-first search (self included) for the element whose `ID` attribute equals
    /// `id` — the target of a dsig `Reference URI="#id"`.
    pub fn find_by_id(&self, id: &str) -> Option<&Element> {
        if self.attr("ID") == Some(id) {
            return Some(self);
        }
        for c in self.child_elements() {
            if let Some(found) = c.find_by_id(id) {
                return Some(found);
            }
        }
        None
    }
}

/// Parse an XML document into its root [`Element`]. Comments / PIs / the XML
/// declaration are discarded (exc-C14N-without-comments).
pub fn parse_document(xml: &str) -> Result<Element, SsoError> {
    let mut reader = Reader::from_str(xml);
    // Preserve every text node verbatim — canonical form is whitespace-significant.
    reader.config_mut().trim_text(false);

    let mut counter: u32 = 0;
    let mut root_scope: BTreeMap<String, String> = BTreeMap::new();
    root_scope.insert("xml".to_string(), XML_NS.to_string());

    let mut stack: Vec<Element> = Vec::new();
    let mut root: Option<Element> = None;

    loop {
        let ev = reader
            .read_event()
            .map_err(|e| SsoError::TokenValidation(format!("xml parse: {e}")))?;
        match ev {
            Event::Eof => break,
            Event::Start(e) => {
                let inherited = stack
                    .last()
                    .map(|p| p.scope.clone())
                    .unwrap_or_else(|| root_scope.clone());
                let el = build_element(&e, &mut counter, inherited)?;
                stack.push(el);
            }
            Event::Empty(e) => {
                let inherited = stack
                    .last()
                    .map(|p| p.scope.clone())
                    .unwrap_or_else(|| root_scope.clone());
                let el = build_element(&e, &mut counter, inherited)?;
                attach(&mut stack, &mut root, el)?;
            }
            Event::End(_) => {
                let el = stack
                    .pop()
                    .ok_or_else(|| SsoError::TokenValidation("xml: unbalanced end tag".into()))?;
                attach(&mut stack, &mut root, el)?;
            }
            Event::Text(e) => {
                let raw = e
                    .decode()
                    .map_err(|e| SsoError::TokenValidation(format!("xml text: {e}")))?;
                let text = quick_xml::escape::unescape(&raw)
                    .map_err(|e| SsoError::TokenValidation(format!("xml unescape: {e}")))?
                    .into_owned();
                push_text(&mut stack, text);
            }
            Event::CData(e) => {
                // CDATA is literal; render as escaped text.
                push_text(&mut stack, String::from_utf8_lossy(e.as_ref()).into_owned());
            }
            Event::GeneralRef(e) => {
                // quick-xml 0.41 emits entity references (`&lt;`, `&#169;`) as their
                // own events — resolve so entity content is not silently dropped.
                push_text(&mut stack, resolve_ref(&e)?);
            }
            // Comments, PIs, DOCTYPE, XML declaration: discarded.
            _ => {}
        }
    }

    root.ok_or_else(|| SsoError::TokenValidation("xml: no root element".into()))
}

fn build_element(
    e: &quick_xml::events::BytesStart,
    counter: &mut u32,
    inherited: BTreeMap<String, String>,
) -> Result<Element, SsoError> {
    let name = e.name();
    let (prefix, local) = split_qname(name.as_ref())?;

    let mut ns_decls: Vec<(String, String)> = Vec::new();
    let mut attrs: Vec<Attr> = Vec::new();

    for a in e.attributes() {
        let a = a.map_err(|e| SsoError::TokenValidation(format!("xml attr: {e}")))?;
        let key = a.key.as_ref();
        let raw = String::from_utf8_lossy(&a.value);
        let value = quick_xml::escape::unescape(&raw)
            .map_err(|e| SsoError::TokenValidation(format!("xml attr unescape: {e}")))?
            .into_owned();
        if key == b"xmlns" {
            ns_decls.push((String::new(), value));
        } else if let Some(rest) = key.strip_prefix(b"xmlns:") {
            let p = std::str::from_utf8(rest)
                .map_err(|_| SsoError::TokenValidation("xml: bad prefix".into()))?
                .to_string();
            ns_decls.push((p, value));
        } else {
            let (ap, al) = split_qname(key)?;
            attrs.push(Attr {
                prefix: ap,
                local: al,
                value,
            });
        }
    }

    let mut scope = inherited;
    for (p, u) in &ns_decls {
        if u.is_empty() {
            scope.remove(p);
        } else {
            scope.insert(p.clone(), u.clone());
        }
    }

    *counter += 1;
    Ok(Element {
        nid: *counter,
        prefix,
        local,
        ns_decls,
        attrs,
        scope,
        children: Vec::new(),
    })
}

fn attach(stack: &mut [Element], root: &mut Option<Element>, el: Element) -> Result<(), SsoError> {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(Node::Element(el));
    } else if root.is_none() {
        *root = Some(el);
    } else {
        return Err(SsoError::TokenValidation(
            "xml: multiple root elements".into(),
        ));
    }
    Ok(())
}

fn push_text(stack: &mut [Element], text: String) {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(Node::Text(text));
    }
    // Whitespace outside the document element is not part of any element: dropped.
}

fn resolve_ref(e: &quick_xml::events::BytesRef) -> Result<String, SsoError> {
    if let Some(ch) = e
        .resolve_char_ref()
        .map_err(|e| SsoError::TokenValidation(format!("xml char ref: {e}")))?
    {
        return Ok(ch.to_string());
    }
    let name = e
        .decode()
        .map_err(|e| SsoError::TokenValidation(format!("xml entity: {e}")))?;
    quick_xml::escape::resolve_predefined_entity(&name)
        .map(str::to_string)
        .ok_or_else(|| SsoError::TokenValidation(format!("unknown XML entity '{name}'")))
}

fn split_qname(name: &[u8]) -> Result<(String, String), SsoError> {
    let s = std::str::from_utf8(name)
        .map_err(|_| SsoError::TokenValidation("xml: non-utf8 name".into()))?;
    match s.split_once(':') {
        Some((p, l)) => Ok((p.to_string(), l.to_string())),
        None => Ok((String::new(), s.to_string())),
    }
}

/// Canonicalize `el` and its subtree to the exclusive canonical byte form.
///
/// * `inclusive` — the `InclusiveNamespaces` `PrefixList` (prefixes, or `#default`
///   for the default namespace) that must be treated inclusively (rendered when in
///   scope, like C14N 1.0), per the transform/canonicalization-method parameters.
/// * `skip_nid` — the [`Element::nid`] of an enveloped `ds:Signature` to omit
///   (the enveloped-signature transform); `None` to include everything.
pub fn canonicalize(el: &Element, inclusive: &BTreeSet<String>, skip_nid: Option<u32>) -> String {
    let mut out = String::new();
    let rendered: BTreeMap<String, String> = BTreeMap::new();
    render(el, &rendered, inclusive, skip_nid, &mut out);
    out
}

fn render(
    el: &Element,
    rendered: &BTreeMap<String, String>,
    inclusive: &BTreeSet<String>,
    skip_nid: Option<u32>,
    out: &mut String,
) {
    // 1. Prefixes visibly utilized by this element: its own prefix, each prefixed
    //    attribute's prefix, plus every prefix named in the InclusiveNamespaces list.
    let mut util: BTreeSet<String> = BTreeSet::new();
    util.insert(el.prefix.clone());
    for a in &el.attrs {
        if !a.prefix.is_empty() {
            util.insert(a.prefix.clone());
        }
    }
    for p in inclusive {
        util.insert(if p == "#default" {
            String::new()
        } else {
            p.clone()
        });
    }

    // 2. Decide which namespace nodes to emit (only those whose in-scope binding
    //    differs from the nearest already-rendered ancestor binding).
    let mut new_rendered = rendered.clone();
    let mut ns_out: Vec<(String, String)> = Vec::new();
    for p in &util {
        if p == "xml" {
            // The reserved `xml` prefix is implicitly declared — never emitted.
            continue;
        }
        let val = el.scope.get(p).cloned().unwrap_or_default();
        let prev = rendered.get(p).cloned().unwrap_or_default();
        if p.is_empty() {
            // Default namespace.
            if val.is_empty() {
                // No default in scope: emit `xmlns=""` only to cancel an ancestor's.
                if !prev.is_empty() {
                    ns_out.push((String::new(), String::new()));
                    new_rendered.insert(String::new(), String::new());
                }
            } else if prev != val {
                ns_out.push((String::new(), val.clone()));
                new_rendered.insert(String::new(), val.clone());
            }
        } else if !val.is_empty() && prev != val {
            ns_out.push((p.clone(), val.clone()));
            new_rendered.insert(p.clone(), val.clone());
        }
    }
    // Namespace declarations sort by prefix; the default (`""`) sorts first.
    ns_out.sort_by(|a, b| a.0.cmp(&b.0));

    // 3. Attributes sort by (namespace URI, local name); unprefixed = empty URI.
    let mut attrs: Vec<&Attr> = el.attrs.iter().collect();
    attrs.sort_by(|a, b| {
        let ua = attr_ns(a, el);
        let ub = attr_ns(b, el);
        ua.cmp(ub).then_with(|| a.local.cmp(&b.local))
    });

    // 4. Emit the start tag.
    let q = qname(&el.prefix, &el.local);
    out.push('<');
    out.push_str(&q);
    for (p, u) in &ns_out {
        if p.is_empty() {
            out.push_str(" xmlns=\"");
        } else {
            out.push_str(" xmlns:");
            out.push_str(p);
            out.push_str("=\"");
        }
        out.push_str(&escape_attr(u));
        out.push('"');
    }
    for a in &attrs {
        out.push(' ');
        out.push_str(&qname(&a.prefix, &a.local));
        out.push_str("=\"");
        out.push_str(&escape_attr(&a.value));
        out.push('"');
    }
    out.push('>');

    // 5. Children (omitting the enveloped signature), then the end tag. Empty
    //    elements always render as a start/end pair.
    for child in &el.children {
        match child {
            Node::Text(t) => out.push_str(&escape_text(t)),
            Node::Element(c) => {
                if Some(c.nid) == skip_nid {
                    continue;
                }
                render(c, &new_rendered, inclusive, skip_nid, out);
            }
        }
    }
    out.push_str("</");
    out.push_str(&q);
    out.push('>');
}

fn attr_ns<'a>(a: &Attr, el: &'a Element) -> &'a str {
    if a.prefix.is_empty() {
        ""
    } else {
        el.scope.get(&a.prefix).map(String::as_str).unwrap_or("")
    }
}

fn qname(prefix: &str, local: &str) -> String {
    if prefix.is_empty() {
        local.to_string()
    } else {
        format!("{prefix}:{local}")
    }
}

/// C14N character-data escaping: `& < >` and CR.
fn escape_text(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '\r' => o.push_str("&#xD;"),
            _ => o.push(c),
        }
    }
    o
}

/// C14N attribute-value escaping: `& < "` and TAB/LF/CR.
fn escape_attr(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '"' => o.push_str("&quot;"),
            '\t' => o.push_str("&#x9;"),
            '\n' => o.push_str("&#xA;"),
            '\r' => o.push_str("&#xD;"),
            _ => o.push(c),
        }
    }
    o
}

/// Parse an `InclusiveNamespaces` `PrefixList` (space-separated) into a set. A
/// `#default` token is preserved verbatim (it maps to the default namespace).
pub fn parse_prefix_list(list: &str) -> BTreeSet<String> {
    list.split_whitespace().map(str::to_string).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c14n(xml: &str) -> String {
        let root = parse_document(xml).expect("parse");
        canonicalize(&root, &BTreeSet::new(), None)
    }

    fn c14n_incl(xml: &str, prefixes: &str) -> String {
        let root = parse_document(xml).expect("parse");
        canonicalize(&root, &parse_prefix_list(prefixes), None)
    }

    #[test]
    fn empty_element_becomes_start_end_pair() {
        assert_eq!(c14n("<a/>"), "<a></a>");
        assert_eq!(c14n("<a></a>"), "<a></a>");
    }

    #[test]
    fn attributes_sorted_by_namespace_then_local() {
        // Unprefixed attrs (empty URI) sort before prefixed; then by local name.
        let out = c14n(r#"<a xmlns:x="urn:x" z="1" a="2" x:m="3"/>"#);
        assert_eq!(out, r#"<a xmlns:x="urn:x" a="2" z="1" x:m="3"></a>"#);
    }

    #[test]
    fn text_escaping_amp_lt_gt() {
        assert_eq!(
            c14n("<a>1 &lt; 2 &amp; 3 &gt; 0</a>"),
            "<a>1 &lt; 2 &amp; 3 &gt; 0</a>"
        );
    }

    #[test]
    fn whitespace_between_elements_is_preserved() {
        let out = c14n("<a>\n  <b/>\n</a>");
        assert_eq!(out, "<a>\n  <b></b>\n</a>");
    }

    #[test]
    fn exclusive_does_not_propagate_unused_ancestor_namespace() {
        // `n1` is declared on the root but only used deep inside; exclusive c14n
        // does NOT copy it onto the intermediate element that never uses it.
        let xml = r#"<r xmlns:n0="urn:0" xmlns:n1="urn:1"><n0:child><n0:leaf/></n0:child></r>"#;
        let out = c14n(xml);
        // n1 never appears (unused); n0 renders once at <n0:child> and is not
        // repeated on <n0:leaf> (already rendered by the ancestor).
        assert_eq!(
            out,
            "<r><n0:child xmlns:n0=\"urn:0\"><n0:leaf></n0:leaf></n0:child></r>"
        );
    }

    #[test]
    fn prefixed_attribute_forces_its_namespace_to_render() {
        // The `n0` prefix is used only by an attribute — exclusive c14n must still
        // render its declaration on that element.
        let xml = r#"<r xmlns:n0="urn:0"><child n0:a="v"/></r>"#;
        let out = c14n(xml);
        assert_eq!(out, "<r><child xmlns:n0=\"urn:0\" n0:a=\"v\"></child></r>");
    }

    #[test]
    fn default_namespace_cancelled_with_empty_decl() {
        // A default namespace in scope, then a child in no namespace, must emit
        // `xmlns=""` to cancel it.
        let xml = r#"<r xmlns="urn:d"><child xmlns=""><leaf/></child></r>"#;
        let out = c14n(xml);
        assert_eq!(
            out,
            "<r xmlns=\"urn:d\"><child xmlns=\"\"><leaf></leaf></child></r>"
        );
    }

    #[test]
    fn inclusive_namespaces_prefix_list_forces_render() {
        // Without the PrefixList `n1` (unused) would be dropped; naming it inclusive
        // pins it onto the apex where it is in scope.
        let xml = r#"<r xmlns:n0="urn:0" xmlns:n1="urn:1"><n0:child/></r>"#;
        let out = c14n_incl(xml, "n1");
        assert_eq!(
            out,
            "<r xmlns:n1=\"urn:1\"><n0:child xmlns:n0=\"urn:0\"></n0:child></r>"
        );
    }

    #[test]
    fn canonicalize_subtree_uses_inherited_scope() {
        // Canonicalizing a nested element resolves its prefix from an ancestor
        // declaration and re-emits it at the subtree apex.
        let root =
            parse_document(r#"<r xmlns:ds="urn:ds"><ds:SignedInfo><ds:X/></ds:SignedInfo></r>"#)
                .unwrap();
        let si = root.descendant("urn:ds", "SignedInfo").unwrap();
        let out = canonicalize(si, &BTreeSet::new(), None);
        assert_eq!(
            out,
            "<ds:SignedInfo xmlns:ds=\"urn:ds\"><ds:X></ds:X></ds:SignedInfo>"
        );
    }

    #[test]
    fn skip_nid_omits_enveloped_signature() {
        let root = parse_document(r#"<A ID="x"><B/><Signature>sig</Signature><C/></A>"#).unwrap();
        let sig = root.child("", "Signature").unwrap();
        let out = canonicalize(
            root.find_by_id("x").unwrap(),
            &BTreeSet::new(),
            Some(sig.nid),
        );
        assert_eq!(out, "<A ID=\"x\"><B></B><C></C></A>");
    }
}
