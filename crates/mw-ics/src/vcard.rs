//! vCard 3/4 parse+emit against the frozen §2.1 `ContactCard` projection.
//!
//! Both directions are hand-rolled over the RFC 6350 content-line grammar, and
//! they are written as a pair: `parse_vcard(emit_vcard(p))` gives `p` back for
//! every projection `p` that `parse_vcard` produced.
//!
//! The reader does not go through the `vcard4` crate. That parser hands `UID`,
//! `KEY`, `TEL`, `IMPP`, `PHOTO`, `MEMBER` and every other URI-typed value to
//! `uriparse` 0.6.4, which panics on a value such as `a@b:c`; it refuses a whole
//! document over one property name it does not know (`LABEL`, any unregistered
//! name); and it gives no access to the properties its model leaves out. Nothing
//! here builds a URI, so no value can reach that code.
//!
//! A property the projection has no field for (`PHOTO`, `CATEGORIES`, `URL`,
//! `X-…`, …) is kept as its unfolded content line under [`EXTRA`] and written
//! back as it was read. `vcard_raw` — the emitted card — is the round-trip
//! source of truth (plan risk #13).
//!
//! What is not carried: the parameters of a projected property other than the
//! first `TYPE` (and `PREF` on an email), its group prefix, any component of `N`
//! past the fifth or of `ADR` past the seventh, a second `N`/`UID`/`KIND`, and
//! `PRODID`/`REV`, which describe the document this module replaces. Emitted
//! lines are not folded.

use serde_json::{Value, json};

use crate::{IcsError, Result};

/// One parsed contact: the `ContactCard` projection + `vcard_raw`, the card's
/// content lines as received (unfolded, CRLF line ends, control characters
/// other than HTAB removed).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedVcard {
    pub vcard_raw: String,
    pub json: Value,
}

/// The projection key holding the content lines of properties that have no
/// field of their own. `mw-engine` keeps this key out of API responses and does
/// not accept it from a client (`pim/contacts.rs`, `VCARD_EXTRA`).
const EXTRA: &str = "vcardExtra";

// ── content lines ────────────────────────────────────────────────────────────

/// One unfolded content line, split but not decoded.
struct ContentLine<'a> {
    /// `[group.]name` as written.
    head: &'a str,
    /// The name without its group, upper-cased.
    name: String,
    /// The parameters as written, leading `;` included; empty when there are none.
    params_raw: &'a str,
    /// `(NAME, values)` per parameter, the name upper-cased, quotes removed. A
    /// bare vCard 2.1/3.0 parameter (`TEL;WORK`) is read as a `TYPE` value.
    params: Vec<(String, Vec<String>)>,
    /// The value as written, escapes intact.
    value: &'a str,
}

fn is_token(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

/// Byte offsets of every `sep` in `s` that is outside a quoted-string.
fn unquoted(s: &str, sep: char) -> Vec<usize> {
    let mut quoted = false;
    let mut at = vec![];
    for (i, c) in s.char_indices() {
        if c == '"' {
            quoted = !quoted;
        } else if c == sep && !quoted {
            at.push(i);
        }
    }
    at
}

fn split_unquoted(s: &str, sep: char) -> Vec<&str> {
    let mut parts = vec![];
    let mut start = 0;
    for i in unquoted(s, sep) {
        parts.push(&s[start..i]);
        start = i + sep.len_utf8();
    }
    parts.push(&s[start..]);
    parts
}

/// Split `NAME;PARAM=v:VALUE` at the first `:` outside a quoted parameter
/// value. `None` for a line with no such `:` or with a name that is not
/// `[group.]token`.
fn content_line(line: &str) -> Option<ContentLine<'_>> {
    // A `"` belongs to a parameter value, so it only counts before the `:`
    // that ends the parameters; scanning stops there.
    let mut quoted = false;
    let mut colon = None;
    for (i, c) in line.char_indices() {
        if c == '"' {
            quoted = !quoted;
        } else if c == ':' && !quoted {
            colon = Some(i);
            break;
        }
    }
    let colon = colon?;
    let (front, value) = (&line[..colon], &line[colon + 1..]);
    let head_end = front.find(';').unwrap_or(front.len());
    let (head, params_raw) = front.split_at(head_end);
    let bare = match head.split_once('.') {
        Some((group, name)) if is_token(group) => name,
        Some(_) => return None,
        None => head,
    };
    if !is_token(bare) {
        return None;
    }
    let mut params = vec![];
    for seg in split_unquoted(params_raw, ';').into_iter().skip(1) {
        let unquote = |v: &str| v.replace('"', "").trim().to_string();
        match seg.split_once('=') {
            Some((k, v)) => params.push((
                k.trim().to_ascii_uppercase(),
                split_unquoted(v, ',').into_iter().map(unquote).collect(),
            )),
            None if !seg.trim().is_empty() => params.push(("TYPE".to_string(), vec![unquote(seg)])),
            None => {}
        }
    }
    Some(ContentLine {
        head,
        name: bare.to_ascii_uppercase(),
        params_raw,
        params,
        value,
    })
}

/// The content lines of a document: line ends in any spelling (CRLF, a bare LF,
/// a bare CR), continuation lines joined to the line they continue, control
/// characters other than HTAB removed.
fn unfold(text: &str) -> Vec<String> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut lines: Vec<String> = vec![];
    for physical in text.replace("\r\n", "\n").replace('\r', "\n").split('\n') {
        match (physical.strip_prefix([' ', '\t']), lines.last_mut()) {
            (Some(rest), Some(last)) => last.push_str(rest),
            _ => lines.push(physical.to_string()),
        }
    }
    for l in &mut lines {
        l.retain(|c| !c.is_control() || c == '\t');
    }
    lines
}

/// Undo vCard TEXT escaping. A backslash before anything other than `\`, `,`,
/// `;`, `n` or `N` is kept as written.
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') | Some('N') => out.push('\n'),
            Some(e @ ('\\' | ',' | ';')) => out.push(e),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Split a structured or list value on every `sep` that is not escaped. The
/// parts keep their escapes.
fn split_escaped(s: &str, sep: char) -> Vec<&str> {
    let mut parts = vec![];
    let mut start = 0;
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == sep {
            parts.push(&s[start..i]);
            start = i + sep.len_utf8();
        }
    }
    parts.push(&s[start..]);
    parts
}

fn type_values<'a>(l: &'a ContentLine<'_>) -> impl Iterator<Item = String> + 'a {
    l.params
        .iter()
        .filter(|(k, _)| k == "TYPE")
        .flat_map(|(_, v)| v.iter())
        .map(|t| t.to_ascii_lowercase())
}

/// The Mailwoman `context` of a property: its first `TYPE` value, lower-cased.
/// `pref` (vCard 3.0 preference, see [`pref_of`]) and `internet` (vCard 3.0
/// address type with no 4.0 counterpart) are not contexts.
fn context_of(l: &ContentLine<'_>) -> String {
    type_values(l)
        .find(|t| !t.is_empty() && t != "pref" && t != "internet")
        .unwrap_or_default()
}

/// `PREF=n` for n in 1..=100, a vCard 3.0 `TYPE=pref` as 1, otherwise 0.
fn pref_of(l: &ContentLine<'_>) -> i64 {
    let explicit = l
        .params
        .iter()
        .find(|(k, _)| k == "PREF")
        .and_then(|(_, v)| v.first())
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|n| (1..=100).contains(n));
    match explicit {
        Some(n) => n,
        None if type_values(l).any(|t| t == "pref") => 1,
        None => 0,
    }
}

fn has_value_text(l: &ContentLine<'_>) -> bool {
    l.params
        .iter()
        .any(|(k, v)| k == "VALUE" && v.iter().any(|t| t.eq_ignore_ascii_case("text")))
}

/// A `BDAY`/`ANNIVERSARY` value as the projection's `date`, trimmed: a
/// `VALUE=text` value is unescaped first, `YYYYMMDD` becomes `YYYY-MM-DD`,
/// anything else is kept.
fn read_date(l: &ContentLine<'_>) -> String {
    let v = if has_value_text(l) {
        unescape(l.value)
    } else {
        l.value.to_string()
    };
    let v = v.trim();
    if v.len() == 8 && v.bytes().all(|b| b.is_ascii_digit()) {
        format!("{}-{}-{}", &v[..4], &v[4..6], &v[6..])
    } else {
        v.to_string()
    }
}

const ADR_PARTS: [&str; 7] = [
    "pobox", "ext", "street", "locality", "region", "postcode", "country",
];

/// The units of an `organizations` entry: split on `;`, without the blank units
/// at the end.
fn org_units(org: &str) -> Vec<&str> {
    let mut units: Vec<&str> = org.split(';').collect();
    while units.last().is_some_and(|u| u.trim().is_empty()) {
        units.pop();
    }
    units
}

/// The projection's `kind` for a `KIND` value (or for whatever a caller put in
/// the projection): the lower-cased token, `individual` when it is not one.
fn kind_of(raw: &str) -> String {
    let k = raw.trim().to_ascii_lowercase();
    match k.as_str() {
        "organization" => "org".to_string(),
        _ if is_token(&k) => k,
        _ => "individual".to_string(),
    }
}

/// Whether a property that is not projected is carried under [`EXTRA`]. The
/// names refused here are the card's own framing, the two properties that
/// describe the replaced document, and every projected property — except `FN`
/// and `KEY`, whose occurrences after the first have no field and are carried.
fn carried_as_written(name: &str) -> bool {
    !matches!(
        name,
        "BEGIN"
            | "END"
            | "VERSION"
            | "PRODID"
            | "REV"
            | "UID"
            | "KIND"
            | "N"
            | "NICKNAME"
            | "ORG"
            | "TITLE"
            | "EMAIL"
            | "TEL"
            | "IMPP"
            | "ADR"
            | "BDAY"
            | "ANNIVERSARY"
            | "MEMBER"
            | "NOTE"
    )
}

/// Parse a vCard 3/4 document into per-card Mailwoman projections.
///
/// An `Err` means the document could not be delimited: it holds no card, a card
/// is not closed by `END:VCARD`, or one card begins inside another. Inside a
/// card nothing is an error: a line that is not `name[;params]:value` is
/// skipped, and every other line is either projected or carried as written.
pub fn parse_vcard(bytes: &[u8]) -> Result<Vec<ParsedVcard>> {
    let text = String::from_utf8_lossy(bytes);
    let mut out = vec![];
    let mut open: Option<Vec<String>> = None;
    for line in unfold(text.as_ref()) {
        let marker = line.trim();
        if marker.eq_ignore_ascii_case("BEGIN:VCARD") {
            if open.is_some() {
                return Err(IcsError::Vcard("BEGIN:VCARD inside a card".into()));
            }
            open = Some(vec![]);
        } else if marker.eq_ignore_ascii_case("END:VCARD") {
            if let Some(lines) = open.take() {
                out.push(project(&lines));
            }
        } else if let Some(lines) = open.as_mut()
            && !marker.is_empty()
        {
            lines.push(line);
        }
    }
    if open.is_some() {
        return Err(IcsError::Vcard("card is not closed by END:VCARD".into()));
    }
    if out.is_empty() {
        return Err(IcsError::Vcard("no BEGIN:VCARD found".into()));
    }
    Ok(out)
}

/// Project the content lines of one card (everything between `BEGIN` and `END`).
fn project(lines: &[String]) -> ParsedVcard {
    let mut uid: Option<String> = None;
    let mut kind: Option<String> = None;
    let mut full: Option<String> = None;
    let mut n: Option<Vec<String>> = None;
    let mut pgp_key: Option<String> = None;
    let mut notes: Vec<String> = vec![];
    let mut nicknames: Vec<Value> = vec![];
    let mut organizations: Vec<Value> = vec![];
    let mut titles: Vec<Value> = vec![];
    let mut emails: Vec<Value> = vec![];
    let mut phones: Vec<Value> = vec![];
    let mut online: Vec<Value> = vec![];
    let mut addresses: Vec<Value> = vec![];
    let mut anniversaries: Vec<Value> = vec![];
    let mut members: Vec<Value> = vec![];
    let mut extra: Vec<Value> = vec![];

    for line in lines {
        let Some(l) = content_line(line) else {
            continue;
        };
        let text = || unescape(l.value);
        match l.name.as_str() {
            "UID" if uid.is_none() => uid = Some(text()),
            "KIND" if kind.is_none() => kind = Some(kind_of(l.value)),
            "FN" if full.is_none() => full = Some(text()),
            "N" if n.is_none() => {
                n = Some(
                    split_escaped(l.value, ';')
                        .into_iter()
                        .map(unescape)
                        .collect(),
                )
            }
            "KEY" if pgp_key.is_none() => pgp_key = Some(text()),
            "NICKNAME" => nicknames.extend(
                split_escaped(l.value, ',')
                    .into_iter()
                    .map(unescape)
                    .filter(|v| !v.trim().is_empty())
                    .map(Value::String),
            ),
            "ORG" => {
                // The units stay `;`-separated in one string, as the projection
                // has always had them. A `;` that was escaped inside a unit is
                // therefore a separator from here on.
                let joined = split_escaped(l.value, ';')
                    .into_iter()
                    .map(unescape)
                    .collect::<Vec<_>>()
                    .join(";");
                let units = org_units(&joined);
                if !units.is_empty() {
                    organizations.push(Value::String(units.join(";")));
                }
            }
            // An entry whose value is blank is not an entry; `emit_vcard`
            // applies the same test to the same decoded text.
            "TITLE" | "EMAIL" | "TEL" | "IMPP" | "MEMBER" if text().trim().is_empty() => {}
            "TITLE" => titles.push(Value::String(text())),
            "EMAIL" => emails.push(json!({
                "context": context_of(&l),
                "value": text(),
                "pref": pref_of(&l),
            })),
            "TEL" => phones.push(json!({ "context": context_of(&l), "value": text() })),
            "IMPP" => online.push(json!({ "context": context_of(&l), "value": text() })),
            "ADR" => {
                let parts: Vec<String> = split_escaped(l.value, ';')
                    .into_iter()
                    .map(unescape)
                    .collect();
                if parts.iter().any(|p| !p.trim().is_empty()) {
                    let mut adr = serde_json::Map::new();
                    adr.insert("context".into(), Value::String(context_of(&l)));
                    for (i, key) in ADR_PARTS.iter().enumerate() {
                        let part = parts.get(i).cloned().unwrap_or_default();
                        adr.insert((*key).into(), Value::String(part));
                    }
                    addresses.push(Value::Object(adr));
                }
            }
            "BDAY" | "ANNIVERSARY" => {
                let date = read_date(&l);
                if !date.trim().is_empty() {
                    let kind = if l.name == "BDAY" {
                        "birthday"
                    } else {
                        "anniversary"
                    };
                    anniversaries.push(json!({ "kind": kind, "date": date }));
                }
            }
            "MEMBER" => members.push(Value::String(text())),
            "NOTE" => notes.push(text()),
            name if carried_as_written(name) => extra.push(Value::String(line.clone())),
            _ => {}
        }
    }

    let uid = uid.unwrap_or_default();
    let n = n.unwrap_or_default();
    let part = |i: usize| n.get(i).cloned().unwrap_or_default();
    let json = json!({
        "id": uid,
        "addressBookId": "",
        "uid": uid,
        "kind": kind.unwrap_or_else(|| "individual".to_string()),
        "name": {
            "full": full.unwrap_or_default(),
            "given": part(1),
            "surname": part(0),
            "additional": part(2),
            "prefix": part(3),
            "suffix": part(4),
        },
        "nicknames": nicknames,
        "organizations": organizations,
        "titles": titles,
        "emails": emails,
        "phones": phones,
        "onlineServices": online,
        "addresses": addresses,
        "anniversaries": anniversaries,
        "members": members,
        "notes": notes.join("\n"),
        "photoBlobId": Value::Null,
        "isFavorite": false,
        "groupIds": [],
        "pgpKey": pgp_key.map(Value::String).unwrap_or(Value::Null),
        "smimeCert": Value::Null,
        "etag": Value::Null,
        EXTRA: extra,
    });
    let mut vcard_raw = String::from("BEGIN:VCARD\r\n");
    for l in lines {
        vcard_raw.push_str(l);
        vcard_raw.push_str("\r\n");
    }
    vcard_raw.push_str("END:VCARD\r\n");
    ParsedVcard { vcard_raw, json }
}

// ── emit ─────────────────────────────────────────────────────────────────────

/// vCard TEXT escaping (backslash, comma, semicolon, newline). A line break in
/// any spelling — CRLF, a bare LF or a bare CR — is written as `\n`.
fn esc(s: &str) -> String {
    s.replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\\', "\\\\")
        .replace(',', "\\,")
        .replace(';', "\\;")
        .replace('\n', "\\n")
}

/// Escaping for a value that is a URI or free text and is neither a list nor
/// structured (`UID`, `TEL`, `IMPP`, `MEMBER`, `KEY`): backslash and line breaks
/// only. A `,` or `;` is written as it is, which is what a URI needs
/// (`data:application/pgp-keys;base64,…`) and what [`unescape`] reads back
/// unchanged.
fn esc_uri(s: &str) -> String {
    s.replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
}

fn s(v: &Value, k: &str) -> String {
    v.get(k)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn items<'a>(v: &'a Value, k: &str) -> impl Iterator<Item = &'a Value> {
    v.get(k).and_then(Value::as_array).into_iter().flatten()
}

/// A parameter value that stays one value of its parameter: control characters
/// other than HTAB are left out, a DQUOTE (which a quoted-string cannot hold)
/// becomes `'`, and a value holding `:`, `;` or `,` is written as a
/// quoted-string. Empty when nothing is left to write.
fn param_value(v: &str) -> String {
    let v: String = v
        .chars()
        .filter(|c| !c.is_control() || *c == '\t')
        .map(|c| if c == '"' { '\'' } else { c })
        .collect();
    if v.contains([':', ';', ',']) {
        format!("\"{v}\"")
    } else {
        v
    }
}

/// `;TYPE=<context>` for an entry of `emails`/`phones`/`onlineServices`/
/// `addresses`, or nothing when it has no context.
fn type_param(entry: &Value) -> String {
    let ctx = param_value(&s(entry, "context"));
    if ctx.is_empty() {
        String::new()
    } else {
        format!(";TYPE={ctx}")
    }
}

/// Append one content line. Control characters other than HTAB (CR, LF, NUL,
/// DEL, C1 …) are left out of `params` and `value`, so neither can end the
/// line. Nothing else is escaped or quoted here: the caller passes `params`
/// built with [`param_value`] or read by [`content_line`], and a `value` that
/// is already escaped for its type.
fn line(out: &mut String, name: &str, params: &str, value: &str) {
    let keep = |c: &char| !c.is_control() || *c == '\t';
    out.push_str(name);
    out.extend(params.chars().filter(keep));
    out.push(':');
    out.extend(value.chars().filter(keep));
    out.push_str("\r\n");
}

/// Emit a vCard 4.0 document from a `ContactCard` projection.
pub fn emit_vcard(contact_json: &Value) -> Result<String> {
    let v = contact_json;
    let mut out = String::from("BEGIN:VCARD\r\nVERSION:4.0\r\n");

    let uid = s(v, "uid");
    if !uid.is_empty() {
        line(&mut out, "UID", "", &esc_uri(&uid));
    }
    let kind = s(v, "kind");
    if !kind.is_empty() {
        line(&mut out, "KIND", "", &kind_of(&kind));
    }
    let name = v.get("name").cloned().unwrap_or(Value::Null);
    line(&mut out, "FN", "", &esc(&s(&name, "full")));
    let n = format!(
        "{};{};{};{};{}",
        esc(&s(&name, "surname")),
        esc(&s(&name, "given")),
        esc(&s(&name, "additional")),
        esc(&s(&name, "prefix")),
        esc(&s(&name, "suffix")),
    );
    line(&mut out, "N", "", &n);

    for nick in items(v, "nicknames").filter_map(Value::as_str) {
        if !nick.trim().is_empty() {
            line(&mut out, "NICKNAME", "", &esc(nick));
        }
    }
    for org in items(v, "organizations").filter_map(Value::as_str) {
        // A `;` in the projection separates the units; everything else in a
        // unit is escaped as TEXT.
        let units = org_units(org);
        if !units.is_empty() {
            let units: Vec<String> = units.into_iter().map(esc).collect();
            line(&mut out, "ORG", "", &units.join(";"));
        }
    }
    for title in items(v, "titles").filter_map(Value::as_str) {
        if !title.trim().is_empty() {
            line(&mut out, "TITLE", "", &esc(title));
        }
    }
    for email in items(v, "emails") {
        let value = s(email, "value");
        if value.trim().is_empty() {
            continue;
        }
        let mut params = type_param(email);
        let pref = email.get("pref").and_then(Value::as_i64).unwrap_or(0);
        if (1..=100).contains(&pref) {
            params.push_str(&format!(";PREF={pref}"));
        }
        line(&mut out, "EMAIL", &params, &esc(&value));
    }
    for (key, prop) in [("phones", "TEL"), ("onlineServices", "IMPP")] {
        for entry in items(v, key) {
            let value = s(entry, "value");
            if !value.trim().is_empty() {
                line(&mut out, prop, &type_param(entry), &esc_uri(&value));
            }
        }
    }
    for adr in items(v, "addresses") {
        let parts: Vec<String> = ADR_PARTS.iter().map(|k| s(adr, k)).collect();
        if parts.iter().any(|p| !p.trim().is_empty()) {
            let parts: Vec<String> = parts.iter().map(|p| esc(p)).collect();
            line(&mut out, "ADR", &type_param(adr), &parts.join(";"));
        }
    }
    for anniversary in items(v, "anniversaries") {
        let date = s(anniversary, "date");
        if date.trim().is_empty() {
            continue;
        }
        let prop = if s(anniversary, "kind") == "birthday" {
            "BDAY"
        } else {
            "ANNIVERSARY"
        };
        let date = date.trim();
        let d = date.as_bytes();
        let digits = |r: std::ops::Range<usize>| d[r].iter().all(u8::is_ascii_digit);
        if d.len() == 10
            && d[4] == b'-'
            && d[7] == b'-'
            && digits(0..4)
            && digits(5..7)
            && digits(8..10)
        {
            // `YYYY-MM-DD` goes out in the vCard 4.0 basic form; `read_date`
            // puts the dashes back.
            let basic: String = date.chars().filter(|c| *c != '-').collect();
            line(&mut out, prop, "", &basic);
        } else if d
            .iter()
            .all(|b| b.is_ascii_digit() || b"-T:Z+.".contains(b))
        {
            // Any other date-and-or-time spelling (`--1210`, a timestamp).
            line(&mut out, prop, "", date);
        } else {
            line(&mut out, prop, ";VALUE=text", &esc(date));
        }
    }
    for member in items(v, "members").filter_map(Value::as_str) {
        if !member.trim().is_empty() {
            line(&mut out, "MEMBER", "", &esc_uri(member));
        }
    }
    let notes = s(v, "notes");
    if !notes.is_empty() {
        line(&mut out, "NOTE", "", &esc(&notes));
    }
    if let Some(key) = v.get("pgpKey").and_then(Value::as_str) {
        line(&mut out, "KEY", "", &esc_uri(key));
    }
    // Properties with no field, as they were read. Each entry is split again
    // here, so whatever the list holds it yields one property line with a
    // well-formed name that is not one of this card's own, or nothing.
    for raw in items(v, EXTRA).filter_map(Value::as_str) {
        if let Some(l) = content_line(raw)
            && carried_as_written(&l.name)
        {
            line(&mut out, l.head, l.params_raw, l.value);
        }
    }

    out.push_str("END:VCARD\r\n");
    Ok(out)
}
