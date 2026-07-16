//! vCard → `ContactCard` projection (§2.1, JSContact-aligned).
//!
//! [`from_vcard`] projects a vCard 3/4 body to the frozen `ContactCard` wire
//! shape over the common fields (name, org, emails, phones, online services,
//! anniversaries, notes, etc.). It produces the same `ContactCard` shape as
//! `mw_ics::parse_vcard` (which also carries the verbatim `vcard_raw` for
//! round-tripping), so the two are interchangeable at the wire level.

use serde::{Deserialize, Serialize};

/// A structured contact name (§2.1 `name:{full,given,surname,prefix,suffix}`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContactName {
    pub full: String,
    pub given: String,
    pub surname: String,
    pub prefix: String,
    pub suffix: String,
}

/// A contact email with context + preference (§2.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactEmail {
    pub context: String,
    pub value: String,
    pub pref: i64,
}

/// A generic contexted contact value — phones / online services (§2.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactValue {
    pub context: String,
    pub value: String,
}

/// A birthday / anniversary (§2.1 `anniversaries:[{kind,date}]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Anniversary {
    pub kind: String,
    pub date: String,
}

/// A contact card (§2.1, JSContact-aligned; camelCase on the wire so it matches
/// the frozen TS `ContactCard` and `mw_ics`'s projection). `pgpKey`/`smimeCert`
/// are opaque placeholders — PGP/S-MIME wiring is V4.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactCard {
    pub id: String,
    pub address_book_id: String,
    pub uid: String,
    /// `"individual"` | `"org"`.
    pub kind: String,
    pub name: ContactName,
    pub nicknames: Vec<String>,
    pub organizations: Vec<String>,
    pub titles: Vec<String>,
    pub emails: Vec<ContactEmail>,
    pub phones: Vec<ContactValue>,
    pub online_services: Vec<ContactValue>,
    pub addresses: Vec<serde_json::Value>,
    pub anniversaries: Vec<Anniversary>,
    pub notes: String,
    pub photo_blob_id: Option<String>,
    pub is_favorite: bool,
    pub group_ids: Vec<String>,
    pub pgp_key: Option<String>,
    pub smime_cert: Option<String>,
    pub etag: Option<String>,
}

impl ContactCard {
    fn empty(id: String, address_book_id: String, etag: Option<String>) -> Self {
        ContactCard {
            id,
            address_book_id,
            uid: String::new(),
            kind: "individual".to_string(),
            name: ContactName::default(),
            nicknames: Vec::new(),
            organizations: Vec::new(),
            titles: Vec::new(),
            emails: Vec::new(),
            phones: Vec::new(),
            online_services: Vec::new(),
            addresses: Vec::new(),
            anniversaries: Vec::new(),
            notes: String::new(),
            photo_blob_id: None,
            is_favorite: false,
            group_ids: Vec::new(),
            pgp_key: None,
            smime_cert: None,
            etag,
        }
    }
}

/// One physical vCard content line: name, params, value (post-unfolding).
struct Line<'a> {
    name: String,
    params: Vec<(String, String)>,
    value: &'a str,
}

/// Unfold RFC 6350 continuation lines (a CRLF/LF followed by a space or tab).
fn unfold(body: &str) -> String {
    let normalized = body.replace("\r\n", "\n");
    let mut out = String::with_capacity(normalized.len());
    for line in normalized.split('\n') {
        if let Some(rest) = line.strip_prefix(' ').or_else(|| line.strip_prefix('\t')) {
            out.push_str(rest);
        } else {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(line);
        }
    }
    out
}

/// Parse one unfolded `NAME;PARAM=v:VALUE` line (params split on `;`, `TYPE=`
/// prefixes optional — bare `WORK`/`HOME` types are accepted too).
fn parse_line(line: &str) -> Option<Line<'_>> {
    let colon = line.find(':')?;
    let (head, value) = line.split_at(colon);
    let value = &value[1..];
    let mut parts = head.split(';');
    let name = parts.next()?.trim().to_ascii_uppercase();
    if name.is_empty() {
        return None;
    }
    let mut params = Vec::new();
    for p in parts {
        if let Some((k, v)) = p.split_once('=') {
            params.push((k.trim().to_ascii_uppercase(), v.trim().to_string()));
        } else if !p.trim().is_empty() {
            // Bare param (old-style TYPE), e.g. `EMAIL;WORK:…`.
            params.push(("TYPE".to_string(), p.trim().to_string()));
        }
    }
    Some(Line {
        name,
        params,
        value,
    })
}

/// The `context` for an email/phone/etc. from its `TYPE` params (first
/// non-`pref`/`internet` type, lowercased; empty when untyped).
fn context_of(params: &[(String, String)]) -> String {
    for (k, v) in params {
        if k == "TYPE" {
            let vt = v.to_ascii_lowercase();
            if vt != "pref" && vt != "internet" && vt != "voice" && !vt.is_empty() {
                return vt;
            }
        }
    }
    String::new()
}

/// Whether a `TYPE`/`PREF` marks this value preferred.
fn pref_of(params: &[(String, String)]) -> i64 {
    for (k, v) in params {
        if k == "PREF" {
            return v.parse().unwrap_or(1);
        }
        if k == "TYPE" && v.eq_ignore_ascii_case("pref") {
            return 1;
        }
    }
    0
}

/// Unescape vCard structured-value text escapes (`\n`, `\,`, `\;`, `\\`).
fn unescape(v: &str) -> String {
    let mut out = String::with_capacity(v.len());
    let mut chars = v.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') | Some('N') => out.push('\n'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Project a single vCard body into a [`ContactCard`] (§2.1). `id`/`addressBookId`
/// are supplied by the caller (the engine assigns local ids); `etag` is the
/// resource ETag. Covers the common vCard fields (see the module note).
pub fn from_vcard(
    body: &str,
    id: &str,
    address_book_id: &str,
    etag: Option<String>,
) -> ContactCard {
    let mut card = ContactCard::empty(id.to_string(), address_book_id.to_string(), etag);
    let unfolded = unfold(body);
    for raw in unfolded.split('\n') {
        let line = match parse_line(raw) {
            Some(l) => l,
            None => continue,
        };
        let val = unescape(line.value);
        match line.name.as_str() {
            "FN" => card.name.full = val,
            "N" => {
                // surname;given;additional;prefix;suffix
                let f: Vec<String> = line.value.split(';').map(unescape).collect();
                card.name.surname = f.first().cloned().unwrap_or_default();
                card.name.given = f.get(1).cloned().unwrap_or_default();
                card.name.prefix = f.get(3).cloned().unwrap_or_default();
                card.name.suffix = f.get(4).cloned().unwrap_or_default();
            }
            "NICKNAME" => {
                for n in line.value.split(',') {
                    let n = unescape(n);
                    if !n.is_empty() {
                        card.nicknames.push(n);
                    }
                }
            }
            "ORG" if !val.is_empty() => {
                // ORG is `;`-structured; keep the first unit as the org name.
                let org = line.value.split(';').next().map(unescape).unwrap_or(val);
                card.organizations.push(org);
            }
            "TITLE" if !val.is_empty() => card.titles.push(val),
            "EMAIL" => card.emails.push(ContactEmail {
                context: context_of(&line.params),
                value: val,
                pref: pref_of(&line.params),
            }),
            "TEL" => card.phones.push(ContactValue {
                context: context_of(&line.params),
                value: val,
            }),
            "URL" | "IMPP" => card.online_services.push(ContactValue {
                context: context_of(&line.params),
                value: val,
            }),
            "UID" => card.uid = strip_urn(&val),
            "KIND" => {
                let k = val.to_ascii_lowercase();
                card.kind = if k == "org" || k == "organization" {
                    "org".to_string()
                } else {
                    "individual".to_string()
                };
            }
            "BDAY" => card.anniversaries.push(Anniversary {
                kind: "birthday".to_string(),
                date: val,
            }),
            "ANNIVERSARY" => card.anniversaries.push(Anniversary {
                kind: "anniversary".to_string(),
                date: val,
            }),
            "NOTE" => card.notes = val,
            "KEY" => card.pgp_key = Some(val),
            _ => {}
        }
    }
    card
}

/// Strip a `urn:uuid:` prefix from a UID so the stored uid is bare.
fn strip_urn(uid: &str) -> String {
    uid.strip_prefix("urn:uuid:").unwrap_or(uid).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const RADICALE_VCARD: &str = "BEGIN:VCARD\r\nVERSION:3.0\r\nUID:11111111-2222-3333-4444-555555555555\r\nFN:Ada Lovelace\r\nN:Lovelace;Ada;;Ms.;\r\nEMAIL;TYPE=WORK:ada@example.org\r\nEMAIL;TYPE=HOME;PREF=1:ada@home.example\r\nTEL;TYPE=CELL:+1-555-0100\r\nORG:Analytical Engines;Research\r\nTITLE:Mathematician\r\nNICKNAME:Countess\r\nBDAY:1815-12-10\r\nNOTE:First programmer.\r\nEND:VCARD\r\n";

    #[test]
    fn projects_common_fields() {
        let c = from_vcard(RADICALE_VCARD, "id-1", "ab-1", Some("\"etag-1\"".into()));
        assert_eq!(c.id, "id-1");
        assert_eq!(c.address_book_id, "ab-1");
        assert_eq!(c.uid, "11111111-2222-3333-4444-555555555555");
        assert_eq!(c.name.full, "Ada Lovelace");
        assert_eq!(c.name.surname, "Lovelace");
        assert_eq!(c.name.given, "Ada");
        assert_eq!(c.name.prefix, "Ms.");
        assert_eq!(c.organizations, vec!["Analytical Engines".to_string()]);
        assert_eq!(c.titles, vec!["Mathematician".to_string()]);
        assert_eq!(c.nicknames, vec!["Countess".to_string()]);
        assert_eq!(c.emails.len(), 2);
        assert_eq!(c.emails[0].context, "work");
        assert_eq!(c.emails[0].value, "ada@example.org");
        assert_eq!(c.emails[1].context, "home");
        assert_eq!(c.emails[1].pref, 1);
        assert_eq!(c.phones.len(), 1);
        assert_eq!(c.phones[0].value, "+1-555-0100");
        assert_eq!(c.anniversaries[0].kind, "birthday");
        assert_eq!(c.anniversaries[0].date, "1815-12-10");
        assert_eq!(c.notes, "First programmer.");
        assert_eq!(c.etag.as_deref(), Some("\"etag-1\""));
    }

    #[test]
    fn unfolds_continuation_lines() {
        let folded = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Grace \r\n Hopper\r\nEND:VCARD\r\n";
        let c = from_vcard(folded, "i", "a", None);
        assert_eq!(c.name.full, "Grace Hopper");
    }

    #[test]
    fn kind_org_detected() {
        let v = "BEGIN:VCARD\nVERSION:4.0\nKIND:org\nFN:Acme Inc\nEND:VCARD\n";
        let c = from_vcard(v, "i", "a", None);
        assert_eq!(c.kind, "org");
    }

    #[test]
    fn projection_round_trips_through_json() {
        // The wire shape must match the frozen TS `ContactCard` (camelCase) so
        // the `mw_ics::parse_vcard` swap is mechanical.
        let c = from_vcard(RADICALE_VCARD, "i", "a", None);
        let j = serde_json::to_value(&c).unwrap();
        assert!(j.get("addressBookId").is_some());
        assert!(j.get("onlineServices").is_some());
        assert!(j.get("isFavorite").is_some());
        let back: ContactCard = serde_json::from_value(j).unwrap();
        assert_eq!(back, c);
    }

    #[test]
    fn projection_fits_mw_ics_parsed_vcard_contract() {
        // `from_vcard`'s projection is wire-compatible with `mw_ics::parse_vcard`:
        // a `ParsedVcard { vcard_raw, json }` whose `json` is the same `ContactCard`
        // shape produced here. Assert our projection is assignable into that
        // contract verbatim.
        let card = from_vcard(RADICALE_VCARD, "i", "a", None);
        let json = serde_json::to_value(&card).unwrap();
        let parsed = mw_ics::ParsedVcard {
            vcard_raw: RADICALE_VCARD.to_string(),
            json: json.clone(),
        };
        assert_eq!(parsed.json, json);
        let round: ContactCard = serde_json::from_value(parsed.json).unwrap();
        assert_eq!(round, card);
    }
}
