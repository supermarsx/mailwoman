//! Pure DAV multistatus response parsers (RFC 6352/6578, plan §2.3).
//!
//! Namespace-prefix-agnostic: every element is matched on its lowercased local
//! name, so Radicale (`<D:` / default-ns) and Google (`<d:` + full-URL hrefs)
//! parse through the same walker. All functions are pure `&str` → value with no
//! I/O and are unit-tested over recorded fixtures (`fixtures/carddav/`).

use mw_dav::{Collection, Resource, SyncDelta};
use quick_xml::Reader;
use quick_xml::events::{BytesText, Event};

use crate::{Error, Result};

fn xml_err(e: impl std::fmt::Display) -> Error {
    Error::Xml(e.to_string())
}

/// Decode a text node to a `String`, resolving XML entities. quick-xml 0.41 no
/// longer unescapes `Event::Text` in place, so we decode bytes → str then run
/// entity unescaping explicitly.
fn decode_text(e: &BytesText<'_>) -> Result<String> {
    let decoded = e.decode().map_err(xml_err)?;
    Ok(quick_xml::escape::unescape(&decoded)
        .map_err(xml_err)?
        .into_owned())
}

/// Lowercased local name (namespace prefix stripped) of a raw qualified name.
fn local(name: &[u8]) -> String {
    let s = String::from_utf8_lossy(name);
    let l = s.rsplit(':').next().unwrap_or(&s);
    l.to_ascii_lowercase()
}

/// One accumulated `<response>` element.
#[derive(Default)]
struct RespAcc {
    href: Option<String>,
    etag: Option<String>,
    data: Option<String>,
    /// The `<response>/<status>` direct-child status (marks a sync removal).
    resource_status: Option<String>,
    display_name: Option<String>,
    ctag: Option<String>,
    /// The collection's own `sync-token` (per-response, inside its prop).
    sync_token: Option<String>,
    resourcetypes: Vec<String>,
}

/// The walked multistatus: every `<response>` plus the top-level `sync-token`
/// (present in a `sync-collection` REPORT reply).
struct Multistatus {
    responses: Vec<RespAcc>,
    top_sync_token: Option<String>,
}

/// Walk a DAV multistatus document once into its responses (§2.3).
fn walk(xml: &str) -> Result<Multistatus> {
    let mut reader = Reader::from_str(xml);
    let mut stack: Vec<String> = Vec::new();
    let mut responses: Vec<RespAcc> = Vec::new();
    let mut current: Option<RespAcc> = None;
    let mut top_sync_token: Option<String> = None;

    loop {
        match reader.read_event().map_err(xml_err)? {
            Event::Eof => break,
            Event::Start(e) => {
                let name = local(e.local_name().as_ref());
                if name == "response" {
                    current = Some(RespAcc::default());
                }
                stack.push(name);
            }
            Event::End(e) => {
                let name = local(e.local_name().as_ref());
                if name == "response"
                    && let Some(acc) = current.take()
                {
                    responses.push(acc);
                }
                stack.pop();
            }
            Event::Empty(e) => {
                let name = local(e.local_name().as_ref());
                // resourcetype children arrive as empty elements.
                if stack.last().map(String::as_str) == Some("resourcetype")
                    && let Some(acc) = current.as_mut()
                {
                    acc.resourcetypes.push(name);
                }
            }
            Event::Text(e) => {
                let text = decode_text(&e)?;
                assign(&stack, current.as_mut(), &mut top_sync_token, text);
            }
            Event::CData(e) => {
                let text = String::from_utf8_lossy(e.as_ref()).into_owned();
                assign(&stack, current.as_mut(), &mut top_sync_token, text);
            }
            _ => {}
        }
    }

    Ok(Multistatus {
        responses,
        top_sync_token,
    })
}

/// Route a text node to the field named by its enclosing element.
fn assign(
    stack: &[String],
    current: Option<&mut RespAcc>,
    top_sync_token: &mut Option<String>,
    text: String,
) {
    let elem = match stack.last() {
        Some(e) => e.as_str(),
        None => return,
    };
    let parent = stack.get(stack.len().wrapping_sub(2)).map(String::as_str);

    // sync-token can be top-level (the delta cursor) or per-collection (in a prop).
    if elem == "sync-token" {
        if parent == Some("multistatus") {
            *top_sync_token = Some(text.trim().to_string());
        } else if let Some(acc) = current {
            acc.sync_token = Some(text.trim().to_string());
        }
        return;
    }

    let acc = match current {
        Some(a) => a,
        None => return,
    };
    match elem {
        "href" if parent == Some("response") => acc.href = Some(text.trim().to_string()),
        "getetag" => acc.etag = Some(text.trim().to_string()),
        "address-data" | "calendar-data" => acc.data = Some(text.trim().to_string()),
        "displayname" => acc.display_name = Some(text.trim().to_string()),
        "getctag" => acc.ctag = Some(text.trim().to_string()),
        "status" if parent == Some("response") => {
            acc.resource_status = Some(text.trim().to_string())
        }
        _ => {}
    }
}

/// Parse an `addressbook-multiget` / `addressbook-query` multistatus into the
/// resources it carries (href + etag + vCard body, §2.3). Removal/404 responses
/// are skipped.
pub fn parse_resources(xml: &str) -> Result<Vec<Resource>> {
    let ms = walk(xml)?;
    let mut out = Vec::new();
    for r in ms.responses {
        let href = match r.href {
            Some(h) if !h.is_empty() => h,
            _ => continue,
        };
        if is_404(r.resource_status.as_deref()) {
            continue;
        }
        out.push(Resource {
            href,
            etag: r.etag,
            body: r.data,
        });
    }
    Ok(out)
}

/// Parse a `sync-collection` REPORT reply (RFC 6578) into the incremental delta:
/// the new `sync-token`, the changed hrefs (+etags; bodies are pulled by a
/// follow-up multiget), and the removed hrefs (§2.3).
pub fn parse_sync_delta(xml: &str) -> Result<SyncDelta> {
    let ms = walk(xml)?;
    let mut changed = Vec::new();
    let mut removed = Vec::new();
    for r in ms.responses {
        let href = match r.href {
            Some(h) if !h.is_empty() => h,
            _ => continue,
        };
        if is_404(r.resource_status.as_deref()) {
            removed.push(href);
        } else {
            changed.push(Resource {
                href,
                etag: r.etag,
                body: r.data,
            });
        }
    }
    Ok(SyncDelta {
        new_sync_token: ms.top_sync_token,
        changed,
        removed,
    })
}

/// Parse a discovery `PROPFIND` (Depth: 1) into the address-book collections it
/// enumerates — keeping only responses whose `resourcetype` includes
/// `addressbook` (§2.3).
pub fn parse_collections(xml: &str) -> Result<Vec<Collection>> {
    let ms = walk(xml)?;
    let mut out = Vec::new();
    for r in ms.responses {
        if !r.resourcetypes.iter().any(|t| t == "addressbook") {
            continue;
        }
        let href = match r.href {
            Some(h) if !h.is_empty() => h,
            _ => continue,
        };
        out.push(Collection {
            href,
            display_name: r.display_name.unwrap_or_default(),
            color: None,
            ctag: r.ctag,
            sync_token: r.sync_token,
            components: Vec::new(),
        });
    }
    Ok(out)
}

/// The collection `getctag` + the `(href, etag)` of every member resource.
pub type EtagList = (Option<String>, Vec<(String, String)>);

/// Parse a ctag-fallback `PROPFIND` (Depth: 1) etag listing into the collection
/// `getctag` plus the `(href, etag)` of every member resource — diffed against
/// stored etags when `sync-collection` is unadvertised (§2.3).
pub fn parse_etag_list(xml: &str) -> Result<EtagList> {
    let ms = walk(xml)?;
    let mut ctag = None;
    let mut members = Vec::new();
    for r in ms.responses {
        if ctag.is_none()
            && let Some(c) = r.ctag
        {
            ctag = Some(c);
        }
        if let (Some(href), Some(etag)) = (r.href, r.etag)
            && !href.is_empty()
        {
            members.push((href, etag));
        }
    }
    Ok((ctag, members))
}

/// Extract the first `<href>` nested inside the first `<wrapper_local>` element —
/// the discovery step-1/2 chain (`current-user-principal`, `addressbook-home-set`).
pub fn first_href_in(xml: &str, wrapper_local: &str) -> Result<Option<String>> {
    let mut reader = Reader::from_str(xml);
    let mut depth_in_wrapper: usize = 0;
    let mut inside = false;
    loop {
        match reader.read_event().map_err(xml_err)? {
            Event::Eof => break,
            Event::Start(e) => {
                let name = local(e.local_name().as_ref());
                if inside {
                    depth_in_wrapper += 1;
                } else if name == wrapper_local {
                    inside = true;
                    depth_in_wrapper = 0;
                }
            }
            Event::End(_) if inside => {
                if depth_in_wrapper == 0 {
                    inside = false;
                } else {
                    depth_in_wrapper -= 1;
                }
            }
            Event::Text(e) if inside => {
                // href text lives one element below the wrapper.
                let t = decode_text(&e)?;
                let t = t.trim();
                if !t.is_empty() {
                    return Ok(Some(t.to_string()));
                }
            }
            _ => {}
        }
    }
    Ok(None)
}

fn is_404(status: Option<&str>) -> bool {
    status.map(|s| s.contains("404")).unwrap_or(false)
}
