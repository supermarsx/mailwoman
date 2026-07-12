//! DAV XML (`multistatus`) response parsers for the shared core (plan §1.3,
//! §2.3), hand-rolled over `quick-xml` 0.41.
//!
//! Every function here is a pure `&str` → value transform with no I/O, so the
//! recorded Radicale / Google-quirk fixtures (`fixtures/dav/`) are unit-tested
//! directly against them — no live server. The parser is **namespace-prefix and
//! case tolerant**: it keys on the lower-cased XML *local* name only, so the
//! `d:`/`D:`/`cal:` prefix soup real servers emit (Google uses uppercase `D:`,
//! full-URL hrefs, weak `W/"…"` etags) all parse the same way. `mw-carddav`
//! (e3) reuses these verbatim by passing [`DavKind::CardDav`].

use crate::request::DavKind;
use crate::{Collection, DavError, EtagList, Resource, Result, SyncDelta};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

/// Lower-cased local name of a (possibly prefixed) qualified name — `D:href`,
/// `cal:calendar-data` and `href` all collapse to `href` / `calendar-data`.
fn local(qname: &[u8]) -> Vec<u8> {
    let bare = match qname.iter().position(|&b| b == b':') {
        Some(i) => &qname[i + 1..],
        None => qname,
    };
    bare.to_ascii_lowercase()
}

/// Leaf elements whose text content we accumulate.
fn is_text_leaf(name: &[u8]) -> bool {
    matches!(
        name,
        b"href"
            | b"status"
            | b"getetag"
            | b"calendar-data"
            | b"address-data"
            | b"displayname"
            | b"getctag"
            | b"sync-token"
            | b"calendar-color"
    )
}

/// The first numeric token in an HTTP status line (`HTTP/1.1 404 Not Found`).
fn status_code(line: &str) -> Option<u16> {
    line.split_whitespace().find_map(|t| t.parse::<u16>().ok())
}

/// One `<response>` element, flattened to the props the core cares about.
#[derive(Debug, Default, Clone)]
struct RawResponse {
    href: String,
    /// Response-level `<status>` (sync-collection tombstones carry `404` here).
    status: Option<u16>,
    etag: Option<String>,
    /// `calendar-data` / `address-data` body, when present (multiget).
    data: Option<String>,
    display_name: Option<String>,
    color: Option<String>,
    ctag: Option<String>,
    sync_token: Option<String>,
    resourcetypes: Vec<String>,
    components: Vec<String>,
}

/// Decode a text event to a `String`, resolving XML entities (`&amp;` → `&`).
/// quick-xml 0.41 no longer auto-unescapes on read, so we do it explicitly.
fn decode_text(e: &quick_xml::events::BytesText) -> Result<String> {
    let raw = e.decode().map_err(|e| DavError::Xml(e.to_string()))?;
    let unescaped = quick_xml::escape::unescape(&raw).map_err(|e| DavError::Xml(e.to_string()))?;
    Ok(unescaped.into_owned())
}

/// Resolve a character/general entity reference event (quick-xml 0.41 emits
/// `&amp;` etc. as a separate [`Event::GeneralRef`]) to its text, so entity
/// content inside a captured leaf (e.g. `&amp;` in a `calendar-data` body) is
/// not silently dropped. Unknown entities resolve to the empty string.
fn resolve_ref(e: &quick_xml::events::BytesRef) -> Result<String> {
    if let Some(ch) = e
        .resolve_char_ref()
        .map_err(|e| DavError::Xml(e.to_string()))?
    {
        return Ok(ch.to_string());
    }
    let name = e.decode().map_err(|e| DavError::Xml(e.to_string()))?;
    Ok(quick_xml::escape::resolve_predefined_entity(&name)
        .unwrap_or_default()
        .to_string())
}

/// The value of an attribute (by local name) on a start/empty element.
fn attr_value(e: &quick_xml::events::BytesStart, key: &[u8]) -> Option<String> {
    e.attributes().flatten().find_map(|a| {
        if local(a.key.as_ref()) == key {
            let raw = String::from_utf8_lossy(&a.value);
            quick_xml::escape::unescape(&raw)
                .ok()
                .map(|v| v.into_owned())
        } else {
            None
        }
    })
}

/// Parse a WebDAV `multistatus` document into its flattened responses plus the
/// top-level `sync-token` (present only in a `sync-collection` REPORT reply).
fn parse_multistatus(xml: &str) -> Result<(Vec<RawResponse>, Option<String>)> {
    let mut reader = Reader::from_str(xml);
    let mut stack: Vec<Vec<u8>> = Vec::new();
    let mut responses: Vec<RawResponse> = Vec::new();
    let mut cur: Option<RawResponse> = None;
    let mut top_sync_token: Option<String> = None;
    let mut buf = String::new();
    let mut capturing = false;

    loop {
        match reader
            .read_event()
            .map_err(|e| DavError::Xml(e.to_string()))?
        {
            Event::Eof => break,
            Event::Start(e) => {
                let name = local(e.name().as_ref());
                let parent = stack.last().cloned();
                handle_open(&name, parent.as_deref(), &e, &mut cur);
                capturing = is_text_leaf(&name);
                buf.clear();
                stack.push(name);
            }
            Event::Empty(e) => {
                let name = local(e.name().as_ref());
                let parent = stack.last().map(|v| v.as_slice());
                handle_open(&name, parent, &e, &mut cur);
            }
            Event::Text(e) if capturing => buf.push_str(&decode_text(&e)?),
            Event::CData(e) if capturing => buf.push_str(&String::from_utf8_lossy(&e.into_inner())),
            Event::GeneralRef(e) if capturing => buf.push_str(&resolve_ref(&e)?),
            Event::End(_) => {
                let name = stack.pop().unwrap_or_default();
                let parent = stack.last().map(|v| v.as_slice());
                handle_close(
                    &name,
                    parent,
                    buf.trim(),
                    &mut cur,
                    &mut responses,
                    &mut top_sync_token,
                );
                capturing = false;
                buf.clear();
            }
            _ => {}
        }
    }
    Ok((responses, top_sync_token))
}

/// Element-open side effects: start a response, record resourcetype children and
/// `supported-calendar-component-set` entries (both usually empty elements).
fn handle_open(
    name: &[u8],
    parent: Option<&[u8]>,
    e: &quick_xml::events::BytesStart,
    cur: &mut Option<RawResponse>,
) {
    if name == b"response" {
        *cur = Some(RawResponse::default());
        return;
    }
    if parent == Some(b"resourcetype")
        && let Some(c) = cur.as_mut()
    {
        c.resourcetypes
            .push(String::from_utf8_lossy(name).into_owned());
    }
    if name == b"comp"
        && let (Some(c), Some(v)) = (cur.as_mut(), attr_value(e, b"name"))
    {
        c.components.push(v);
    }
}

/// Element-close side effects: fold a completed leaf's text into the current
/// response (or the top-level sync-token when outside any response).
fn handle_close(
    name: &[u8],
    parent: Option<&[u8]>,
    text: &str,
    cur: &mut Option<RawResponse>,
    responses: &mut Vec<RawResponse>,
    top_sync_token: &mut Option<String>,
) {
    if name == b"response" {
        if let Some(r) = cur.take() {
            responses.push(r);
        }
        return;
    }
    match name {
        b"sync-token" => match cur.as_mut() {
            Some(c) => c.sync_token = Some(text.to_string()),
            None => *top_sync_token = Some(text.to_string()),
        },
        _ => {
            let Some(c) = cur.as_mut() else { return };
            match name {
                b"href" if parent == Some(b"response") => c.href = text.to_string(),
                b"getetag" => c.etag = Some(text.to_string()),
                b"calendar-data" | b"address-data" => c.data = Some(text.to_string()),
                b"displayname" => c.display_name = Some(text.to_string()),
                b"getctag" => c.ctag = Some(text.to_string()),
                b"calendar-color" => c.color = Some(text.to_string()),
                b"status" if parent == Some(b"response") => c.status = status_code(text),
                _ => {}
            }
        }
    }
}

/// Parse a discovery `PROPFIND` (Depth: 1) reply into the collections of `kind`
/// (those whose `resourcetype` advertises `calendar` / `addressbook`), §2.3.
pub fn parse_collections(xml: &str, kind: DavKind) -> Result<Vec<Collection>> {
    let want = kind.resource_type();
    let (responses, _) = parse_multistatus(xml)?;
    Ok(responses
        .into_iter()
        .filter(|r| r.resourcetypes.iter().any(|t| t == want))
        .map(|r| Collection {
            href: r.href,
            display_name: r.display_name.unwrap_or_default(),
            color: r.color,
            ctag: r.ctag,
            sync_token: r.sync_token,
            components: r.components,
        })
        .collect())
}

/// Parse a `sync-collection` REPORT (RFC 6578) reply: the new `sync-token` plus
/// changed resources (2xx / no status) and removed hrefs (`404`), §2.3.
pub fn parse_sync_delta(xml: &str) -> Result<SyncDelta> {
    let (responses, new_sync_token) = parse_multistatus(xml)?;
    let mut delta = SyncDelta {
        new_sync_token,
        ..Default::default()
    };
    for r in responses {
        if r.href.is_empty() {
            continue;
        }
        if matches!(r.status, Some(s) if s >= 400) {
            delta.removed.push(r.href);
        } else {
            delta.changed.push(Resource {
                href: r.href,
                etag: r.etag,
                body: r.data,
            });
        }
    }
    Ok(delta)
}

/// Parse a `calendar-multiget` / `addressbook-multiget` reply into resources
/// carrying their `ETag` + body (RFC 4791 / RFC 6352), §2.3.
pub fn parse_multiget(xml: &str, _kind: DavKind) -> Result<Vec<Resource>> {
    let (responses, _) = parse_multistatus(xml)?;
    Ok(responses
        .into_iter()
        .filter(|r| !r.href.is_empty())
        .map(|r| Resource {
            href: r.href,
            etag: r.etag,
            body: r.data,
        })
        .collect())
}

/// Parse a `calendar-query` reply (hrefs + etags, no bodies) — the initial /
/// fallback enumeration before a multiget (§2.3).
pub fn parse_resource_list(xml: &str) -> Result<Vec<Resource>> {
    parse_multiget(xml, DavKind::CalDav)
}

/// Parse the ctag + member-etag fallback `PROPFIND` (Depth: 1) reply: the
/// collection `getctag` plus every member `(href, etag)` for etag-diff sync
/// when `sync-collection` is unadvertised (§2.3).
pub fn parse_etag_list(xml: &str) -> Result<EtagList> {
    let (responses, _) = parse_multistatus(xml)?;
    let ctag = responses.iter().find_map(|r| r.ctag.clone());
    let members = responses
        .into_iter()
        .filter_map(|r| match (r.href.is_empty(), r.etag) {
            (false, Some(etag)) => Some((r.href, etag)),
            _ => None,
        })
        .collect();
    Ok((ctag, members))
}

/// Find the first `<href>` nested inside the first `target` element (used for
/// `current-user-principal` and the home-set, whose href is nested in the prop,
/// not at response level).
fn first_href_in(xml: &str, target: &[u8]) -> Result<Option<String>> {
    let target = target.to_ascii_lowercase();
    let mut reader = Reader::from_str(xml);
    let mut inside = false;
    let mut capturing = false;
    let mut buf = String::new();
    loop {
        match reader
            .read_event()
            .map_err(|e| DavError::Xml(e.to_string()))?
        {
            Event::Eof => break,
            Event::Start(e) => {
                let n = local(e.name().as_ref());
                if n == target {
                    inside = true;
                } else if inside && n == b"href" {
                    capturing = true;
                    buf.clear();
                }
            }
            Event::Text(e) if capturing => buf.push_str(&decode_text(&e)?),
            Event::GeneralRef(e) if capturing => buf.push_str(&resolve_ref(&e)?),
            Event::End(e) => {
                let n = local(e.name().as_ref());
                if capturing && n == b"href" {
                    return Ok(Some(buf.trim().to_string()));
                }
                if n == target {
                    inside = false;
                }
            }
            _ => {}
        }
    }
    Ok(None)
}

/// Extract the principal href from a `current-user-principal` `PROPFIND` reply.
pub fn parse_current_user_principal(xml: &str) -> Result<Option<String>> {
    first_href_in(xml, b"current-user-principal")
}

/// Extract the collection home-set href for `kind` from a home-set `PROPFIND`.
pub fn parse_home_set(xml: &str, kind: DavKind) -> Result<Option<String>> {
    first_href_in(xml, kind.home_set_prop().as_bytes())
}

/// Normalise an iCalendar basic-format UTC stamp (`20260101T090000Z`) to
/// RFC3339 (`2026-01-01T09:00:00Z`); pass anything else through unchanged.
fn ical_utc_to_rfc3339(s: &str) -> String {
    let b = s.as_bytes();
    if b.len() == 16 && b[8] == b'T' && b[15] == b'Z' && b[..8].iter().all(u8::is_ascii_digit) {
        format!(
            "{}-{}-{}T{}:{}:{}Z",
            &s[0..4],
            &s[4..6],
            &s[6..8],
            &s[9..11],
            &s[11..13],
            &s[13..15],
        )
    } else {
        s.to_string()
    }
}

/// Parse a `free-busy-query` REPORT reply — a `text/calendar` VFREEBUSY — into
/// merged busy intervals feeding `Calendar/freeBusy` (RFC 4791, §2.2). Handles
/// RFC 5545 line folding and `FBTYPE` params; periods are `start/end` (an
/// explicit end; durations are left verbatim for `mw-ics` to normalise).
pub fn parse_free_busy(ics: &str) -> Result<Vec<mw_ics::BusyInterval>> {
    // Unfold continuation lines (a leading space/tab continues the prior line).
    let mut unfolded: Vec<String> = Vec::new();
    for raw in ics.split(['\r', '\n']).filter(|l| !l.is_empty()) {
        if raw.starts_with([' ', '\t'])
            && let Some(last) = unfolded.last_mut()
        {
            last.push_str(&raw[1..]);
            continue;
        }
        unfolded.push(raw.to_string());
    }

    let mut out = Vec::new();
    for line in unfolded {
        let Some(colon) = line.find(':') else {
            continue;
        };
        let (name_params, value) = line.split_at(colon);
        let value = &value[1..];
        let mut parts = name_params.split(';');
        if parts.next().map(str::to_ascii_uppercase).as_deref() != Some("FREEBUSY") {
            continue;
        }
        let status = parts
            .find_map(|p| p.strip_prefix("FBTYPE="))
            .map(|v| v.to_ascii_lowercase())
            .unwrap_or_else(|| "busy".to_string());
        for period in value.split(',') {
            if let Some((start, end)) = period.split_once('/') {
                out.push(mw_ics::BusyInterval {
                    start_utc: ical_utc_to_rfc3339(start.trim()),
                    end_utc: ical_utc_to_rfc3339(end.trim()),
                    status: status.clone(),
                });
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ical_stamp_to_rfc3339() {
        assert_eq!(
            ical_utc_to_rfc3339("20260101T090000Z"),
            "2026-01-01T09:00:00Z"
        );
        assert_eq!(ical_utc_to_rfc3339("garbage"), "garbage");
    }

    #[test]
    fn free_busy_parses_periods_and_fbtype() {
        let ics = "BEGIN:VCALENDAR\r\nBEGIN:VFREEBUSY\r\n\
FREEBUSY;FBTYPE=BUSY:20260101T090000Z/20260101T100000Z,20260101T110000Z/20260101T113000Z\r\n\
FREEBUSY;FBTYPE=BUSY-TENTATIVE:20260101T140000Z/20260101T150000Z\r\n\
END:VFREEBUSY\r\nEND:VCALENDAR\r\n";
        let fb = parse_free_busy(ics).unwrap();
        assert_eq!(fb.len(), 3);
        assert_eq!(fb[0].start_utc, "2026-01-01T09:00:00Z");
        assert_eq!(fb[0].end_utc, "2026-01-01T10:00:00Z");
        assert_eq!(fb[0].status, "busy");
        assert_eq!(fb[2].status, "busy-tentative");
    }

    #[test]
    fn free_busy_unfolds_lines() {
        let ics = "BEGIN:VFREEBUSY\r\nFREEBUSY:20260101T090000Z/\r\n 20260101T100000Z\r\nEND:VFREEBUSY\r\n";
        let fb = parse_free_busy(ics).unwrap();
        assert_eq!(fb.len(), 1);
        assert_eq!(fb[0].end_utc, "2026-01-01T10:00:00Z");
    }

    #[test]
    fn mismatched_tags_are_an_error_not_a_panic() {
        // A mismatched close tag is a hard XML error (not silently accepted).
        assert!(parse_sync_delta("<a><b></a>").is_err());
    }

    #[test]
    fn truncated_xml_does_not_panic() {
        // Untrusted/truncated input must degrade to Ok(empty) or Err, never panic.
        let _ = parse_sync_delta("<d:multistatus><d:response>");
        let _ = parse_collections("<d:multistatus", DavKind::CalDav);
    }
}
