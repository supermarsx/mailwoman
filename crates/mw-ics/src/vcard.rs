//! vCard 3/4 parse+emit against the frozen §2.1 `ContactCard` projection.
//!
//! Reading uses the `vcard4` crate (tolerant of both 3.0 and 4.0); writing is
//! hand-rolled over the vCard 4.0 grammar so `TYPE`/`PREF` parameters on
//! emails/phones survive the round-trip (the builder does not expose them).
//! `vcard_raw` is the round-trip source of truth (plan risk #13).

use serde_json::{Value, json};
use vcard4::parameter::Parameters;
use vcard4::parse_loose;
use vcard4::property::{Kind, Property as _, TextOrUriProperty};

use crate::{IcsError, Result};

/// One parsed contact: the `ContactCard` projection + verbatim `vcard_raw`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedVcard {
    pub vcard_raw: String,
    pub json: Value,
}

fn ctx_from(params: Option<&Parameters>) -> String {
    params
        .and_then(|p| p.types.as_ref())
        .and_then(|t| t.first())
        .map(|t| t.to_string().to_ascii_lowercase())
        .unwrap_or_default()
}

fn pref_from(params: Option<&Parameters>) -> i64 {
    params.and_then(|p| p.pref).map(i64::from).unwrap_or(0)
}

fn tou_string(p: &TextOrUriProperty) -> String {
    p.to_string()
}

/// Parse a vCard 3/4 document into per-card Mailwoman projections.
pub fn parse_vcard(bytes: &[u8]) -> Result<Vec<ParsedVcard>> {
    let text = String::from_utf8_lossy(bytes);
    let cards = parse_loose(text.as_ref()).map_err(|e| IcsError::Vcard(format!("{e}")))?;
    let mut out = vec![];
    for card in cards {
        let full = card
            .formatted_name
            .first()
            .map(|p| p.value.clone())
            .unwrap_or_default();
        let (given, surname, prefix, suffix) = match &card.name {
            Some(n) => {
                let v = &n.value;
                (
                    v.get(1).cloned().unwrap_or_default(),
                    v.first().cloned().unwrap_or_default(),
                    v.get(3).cloned().unwrap_or_default(),
                    v.get(4).cloned().unwrap_or_default(),
                )
            }
            None => Default::default(),
        };
        let kind = match card.kind.as_ref().map(|k| &k.value) {
            Some(Kind::Org) | Some(Kind::Group) => "org",
            _ => "individual",
        };
        let emails: Vec<Value> = card
            .email
            .iter()
            .map(|e| {
                json!({
                    "context": ctx_from(e.parameters.as_ref()),
                    "value": e.value,
                    "pref": pref_from(e.parameters.as_ref()),
                })
            })
            .collect();
        let phones: Vec<Value> = card
            .tel
            .iter()
            .map(|t| json!({ "context": ctx_from(t.parameters()), "value": tou_string(t) }))
            .collect();
        let online: Vec<Value> = card
            .impp
            .iter()
            .map(|s| json!({ "context": ctx_from(s.parameters.as_ref()), "value": s.value.to_string() }))
            .collect();
        let organizations: Vec<Value> = card
            .org
            .iter()
            .map(|o| Value::String(o.value.join(";")))
            .collect();
        let titles: Vec<Value> = card
            .title
            .iter()
            .map(|t| Value::String(t.value.clone()))
            .collect();
        let nicknames: Vec<Value> = card
            .nickname
            .iter()
            .map(|n| Value::String(n.value.clone()))
            .collect();
        let notes = card
            .note
            .first()
            .map(|n| n.value.clone())
            .unwrap_or_default();
        let uid = card.uid.as_ref().map(tou_string).unwrap_or_default();
        let pgp_key = card
            .key
            .first()
            .map(tou_string)
            .map(Value::String)
            .unwrap_or(Value::Null);
        let mut anniversaries: Vec<Value> = vec![];
        if let Some(b) = &card.bday {
            anniversaries.push(json!({ "kind": "birthday", "date": b.to_string() }));
        }

        let json = json!({
            "id": uid,
            "addressBookId": "",
            "uid": uid,
            "kind": kind,
            "name": {
                "full": full, "given": given, "surname": surname,
                "prefix": prefix, "suffix": suffix,
            },
            "nicknames": nicknames,
            "organizations": organizations,
            "titles": titles,
            "emails": emails,
            "phones": phones,
            "onlineServices": online,
            "addresses": [],
            "anniversaries": anniversaries,
            "notes": notes,
            "photoBlobId": Value::Null,
            "isFavorite": false,
            "groupIds": [],
            "pgpKey": pgp_key,
            "smimeCert": Value::Null,
            "etag": Value::Null,
        });
        out.push(ParsedVcard {
            vcard_raw: card.to_string(),
            json,
        });
    }
    Ok(out)
}

// ── emit ─────────────────────────────────────────────────────────────────────

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(',', "\\,")
        .replace(';', "\\;")
        .replace('\n', "\\n")
}

fn s(v: &Value, k: &str) -> String {
    v.get(k)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn line(out: &mut String, name: &str, params: &str, value: &str) {
    out.push_str(name);
    out.push_str(params);
    out.push(':');
    out.push_str(value);
    out.push_str("\r\n");
}

/// Emit a vCard 4.0 document from a `ContactCard` projection.
pub fn emit_vcard(contact_json: &Value) -> Result<String> {
    let v = contact_json;
    let mut out = String::from("BEGIN:VCARD\r\nVERSION:4.0\r\n");

    let uid = s(v, "uid");
    if !uid.is_empty() {
        line(&mut out, "UID", "", &uid);
    }
    let kind = s(v, "kind");
    if !kind.is_empty() {
        line(&mut out, "KIND", "", &kind);
    }
    let name = v.get("name").cloned().unwrap_or(Value::Null);
    line(&mut out, "FN", "", &esc(&s(&name, "full")));
    let n = format!(
        "{};{};;{};{}",
        esc(&s(&name, "surname")),
        esc(&s(&name, "given")),
        esc(&s(&name, "prefix")),
        esc(&s(&name, "suffix")),
    );
    line(&mut out, "N", "", &n);

    for nick in v
        .get("nicknames")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(t) = nick.as_str() {
            line(&mut out, "NICKNAME", "", &esc(t));
        }
    }
    for org in v
        .get("organizations")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(t) = org.as_str() {
            // Structural `;` separators are preserved; components escaped on read.
            line(&mut out, "ORG", "", t);
        }
    }
    for title in v
        .get("titles")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(t) = title.as_str() {
            line(&mut out, "TITLE", "", &esc(t));
        }
    }
    for email in v
        .get("emails")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let mut params = String::new();
        let ctx = s(email, "context");
        if !ctx.is_empty() {
            params.push_str(&format!(";TYPE={ctx}"));
        }
        let pref = email.get("pref").and_then(Value::as_i64).unwrap_or(0);
        if pref > 0 {
            params.push_str(&format!(";PREF={pref}"));
        }
        line(&mut out, "EMAIL", &params, &s(email, "value"));
    }
    for phone in v
        .get("phones")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let ctx = s(phone, "context");
        let params = if ctx.is_empty() {
            String::new()
        } else {
            format!(";TYPE={ctx}")
        };
        line(&mut out, "TEL", &params, &s(phone, "value"));
    }
    for svc in v
        .get("onlineServices")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let ctx = s(svc, "context");
        let params = if ctx.is_empty() {
            String::new()
        } else {
            format!(";TYPE={ctx}")
        };
        line(&mut out, "IMPP", &params, &s(svc, "value"));
    }
    let notes = s(v, "notes");
    if !notes.is_empty() {
        line(&mut out, "NOTE", "", &esc(&notes));
    }
    if let Some(key) = v.get("pgpKey").and_then(Value::as_str) {
        line(&mut out, "KEY", "", key);
    }

    out.push_str("END:VCARD\r\n");
    Ok(out)
}
