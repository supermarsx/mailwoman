//! CalDAV / CardDAV sync orchestration (plan §2.3): the engine drives
//! `mw-dav` / `mw-carddav`, persists `etag`/`sync-token`/`ctag`, and reconciles
//! the local store against the server — `sync-collection` incremental where the
//! server advertises it, else the `ctag` + etag-diff fallback (`mw-carddav`
//! degrades internally; `mw-dav` exposes both paths). Same feature-detect /
//! degrade discipline as the V1 IMAP sync engine.
//!
//! The push side lives inline in [`crate::pim::events`] / `tasks` / `contacts`
//! (`persist_*` PUTs on write); this module owns the **pull** side plus the thin
//! PUT/DELETE adapters those writers call.

use mw_dav::{DavClient, DavConfig, SyncDelta};
use serde_json::Value;

use crate::account::AccountRuntime;
use crate::backend::{EngineError, Result};
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;

use super::gen_id;

/// Map an `mw-dav` error onto the engine error type (a `412` precondition
/// failure surfaces as a protocol error the caller re-pulls on).
pub(crate) fn dav_err(e: mw_dav::DavError) -> EngineError {
    EngineError::Protocol(format!("dav: {e}"))
}

impl Engine {
    // ── write adapters (called by persist_* on the push side) ────────────────

    /// `PUT` an iCalendar resource, returning the server's new `ETag` if any.
    pub(crate) async fn dav_put(
        &self,
        base: &DavConfig,
        href: &str,
        body: &str,
        if_match: Option<&str>,
    ) -> Result<Option<String>> {
        let client = DavClient::new(base.clone()).map_err(dav_err)?;
        client
            .put_resource(href, body, if_match)
            .await
            .map_err(dav_err)
    }

    /// `DELETE` an iCalendar resource (best-effort; `412` conflict is surfaced).
    pub(crate) async fn dav_delete(
        &self,
        base: &DavConfig,
        href: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        let client = DavClient::new(base.clone()).map_err(dav_err)?;
        client
            .delete_resource(href, if_match)
            .await
            .map_err(dav_err)
    }

    /// `PUT` a vCard resource, returning the server's new `ETag` if any.
    pub(crate) async fn carddav_put(
        &self,
        base: &DavConfig,
        href: &str,
        vcard: &str,
        if_match: Option<&str>,
    ) -> Result<Option<String>> {
        let client = mw_carddav::CardDavClient::new(base.clone()).map_err(dav_err)?;
        let etag = client
            .put_contact(href, vcard, if_match)
            .await
            .map_err(dav_err)?;
        Ok((!etag.is_empty()).then_some(etag))
    }

    /// `DELETE` a vCard resource (best-effort; `412` conflict is surfaced).
    pub(crate) async fn carddav_delete(
        &self,
        base: &DavConfig,
        href: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        let client = mw_carddav::CardDavClient::new(base.clone()).map_err(dav_err)?;
        client.delete_contact(href, if_match).await.map_err(dav_err)
    }

    // ── pull side (incremental reconcile) ────────────────────────────────────

    /// Pull every DAV-backed collection for an account and reconcile the local
    /// store (plan §2.3). Mailwoman-native collections (no CalDAV/CardDAV URL)
    /// and webcal overlays (a subscribed `.ics` feed, never a DAV collection) are
    /// skipped. Best-effort per collection — one failure does not abort the rest —
    /// so the surface degrades rather than stalls; the first failure is returned
    /// after every collection was tried, so the caller can back off.
    pub async fn sync_pim(&self, account_id: &str, rt: &AccountRuntime) -> Result<()> {
        // Bridge PIM routing (plan §2.2, §1.3): when the account's backend advertises
        // a bridge calendar/tasks capability, pull PIM through the bridge. When it
        // does NOT — a plain IMAP/POP3/DAV account, or nothing attached — this is a
        // strict no-op (`bridge_calendar`/`bridge_tasks` return `None`) and the
        // standards CalDAV/CardDAV path below runs byte-for-byte unchanged.
        self.sync_pim_via_bridge(account_id).await?;

        let Some(base) = rt.dav.clone() else {
            return Ok(()); // mail-only account (incl. bridge-only accounts)
        };
        let mut first_err: Option<EngineError> = None;
        for cal in self.store().list_calendars(account_id).await? {
            if cal.caldav_url.is_none() || cal.is_overlay {
                continue;
            }
            if let Err(e) = self.pull_calendar(account_id, &cal, &base).await {
                tracing::warn!("CalDAV pull for calendar {} failed: {e}", cal.id);
                first_err.get_or_insert(e);
            }
        }
        for book in self.store().list_address_books(account_id).await? {
            if book.carddav_url.is_some()
                && let Err(e) = self.pull_address_book(account_id, &book, &base).await
            {
                tracing::warn!("CardDAV pull for book {} failed: {e}", book.id);
                first_err.get_or_insert(e);
            }
        }
        self.broadcast_state(account_id).await;
        first_err.map_or(Ok(()), Err)
    }

    /// Incrementally pull one CalDAV collection (events or a VTODO task list)
    /// and reconcile it into the store, advancing the stored `sync-token` (or the
    /// `ctag`, on a server without `sync-collection`).
    pub(crate) async fn pull_calendar(
        &self,
        account_id: &str,
        cal: &mw_store::CalendarRow,
        base: &DavConfig,
    ) -> Result<()> {
        if cal.is_overlay {
            // A webcal feed: no REPORT, PUT or DELETE ever goes to it.
            return Ok(());
        }
        let href = cal.caldav_url.as_deref().unwrap_or_default();
        let tasks = cal.component == "VTODO";
        let client = DavClient::new(base.clone()).map_err(dav_err)?;
        let members = if tasks {
            self.store().task_dav_members(&cal.id).await?
        } else {
            self.store().event_dav_members(&cal.id).await?
        };
        let pulled = match client
            .sync_collection(href, cal.sync_token.as_deref())
            .await
        {
            Ok(delta) => Pulled::Delta(delta),
            Err(mw_dav::DavError::Status { status, .. }) if sync_unsupported(status) => {
                let (ctag, listed) = client.list_etags(href).await.map_err(dav_err)?;
                if ctag.is_some() && ctag == cal.ctag {
                    return Ok(()); // nothing changed since the last pull
                }
                Pulled::Listing { ctag, listed }
            }
            Err(e) => return Err(dav_err(e)),
        };
        let (delta, new_ctag) = reconcile(pulled, &members);

        // Bodies: use those inlined in the delta, multiget the rest.
        let need: Vec<String> = delta
            .changed
            .iter()
            .filter(|r| r.body.is_none())
            .map(|r| r.href.clone())
            .collect();
        let fetched = if need.is_empty() {
            Vec::new()
        } else {
            client
                .calendar_multiget(href, &need)
                .await
                .map_err(dav_err)?
        };
        let mut resources = delta.changed.clone();
        resources.retain(|r| r.body.is_some());
        resources.extend(fetched);

        for res in resources {
            let Some(body) = res.body.as_deref() else {
                continue;
            };
            let parsed = mw_ics::parse_ical(body.as_bytes()).map_err(super::events::ics_err)?;
            // One resource is one object: its first component of the collection's
            // kind is the master; overrides stay in the stored body verbatim.
            let Some(p) = parsed.into_iter().find(|p| p.component == cal.component) else {
                continue;
            };
            let uid = json_uid(&p.json);
            let known = member_id(&members, &res.href);
            let server = ServerCopy {
                body,
                href: &res.href,
            };
            if tasks {
                let id = match known {
                    Some(id) => Some(id),
                    None => self.task_id_for_uid(&cal.id, &uid).await?,
                }
                .unwrap_or_else(|| gen_id("task"));
                let is_new = self.store().get_task(&id).await?.is_none();
                self.persist_task_with(
                    account_id,
                    &cal.id,
                    &id,
                    &uid,
                    p.json,
                    res.etag.clone(),
                    None,
                    Some(server),
                )
                .await?;
                self.record_pim_change(
                    account_id,
                    ChangeType::Task,
                    &id,
                    created_or_updated(is_new),
                )
                .await?;
            } else {
                let id = match known {
                    Some(id) => Some(id),
                    None => self.event_id_for_uid(&cal.id, &uid).await?,
                }
                .unwrap_or_else(|| gen_id("ev"));
                let is_new = self.store().get_event(&id).await?.is_none();
                self.persist_event_with(
                    account_id,
                    &cal.id,
                    &id,
                    &uid,
                    p.json,
                    res.etag.clone(),
                    None,
                    Some(server),
                )
                .await?;
                self.record_pim_change(
                    account_id,
                    ChangeType::CalendarEvent,
                    &id,
                    created_or_updated(is_new),
                )
                .await?;
            }
        }

        // Tombstones: the stored href names the object; a row from before the
        // href was kept falls back to the basename-is-the-uid convention.
        for removed in &delta.removed {
            let id = match member_id(&members, removed) {
                Some(id) => Some(id),
                None if tasks => self.task_id_for_uid(&cal.id, &href_uid(removed)).await?,
                None => self.event_id_for_uid(&cal.id, &href_uid(removed)).await?,
            };
            let Some(id) = id else {
                continue;
            };
            if tasks {
                self.store().delete_task(&id).await?;
                self.record_pim_change(account_id, ChangeType::Task, &id, ChangeOp::Destroyed)
                    .await?;
            } else {
                self.store().delete_event(&id).await?;
                self.record_pim_change(
                    account_id,
                    ChangeType::CalendarEvent,
                    &id,
                    ChangeOp::Destroyed,
                )
                .await?;
            }
        }

        // Persist the advanced sync-token / ctag.
        if delta.new_sync_token.is_some() || new_ctag.is_some() {
            let mut updated = cal.clone();
            if delta.new_sync_token.is_some() {
                updated.sync_token = delta.new_sync_token;
            }
            if new_ctag.is_some() {
                updated.ctag = new_ctag;
            }
            self.store().upsert_calendar(&updated).await?;
        }
        Ok(())
    }

    /// Incrementally pull one CardDAV address book and reconcile it.
    pub(crate) async fn pull_address_book(
        &self,
        account_id: &str,
        book: &mw_store::AddressBookRow,
        base: &DavConfig,
    ) -> Result<()> {
        let href = book.carddav_url.as_deref().unwrap_or_default();
        let client = mw_carddav::CardDavClient::new(base.clone()).map_err(dav_err)?;
        let members = self.store().contact_dav_members(&book.id).await?;
        let delta = client
            .sync_addressbook(href, book.sync_token.as_deref())
            .await
            .map_err(dav_err)?;
        // `mw-carddav` answers a server without sync-collection with the member
        // list (every href, no sync-token, no removals): reconcile it like the
        // CalDAV etag listing, so unchanged cards are not fetched and cards gone
        // from the server are removed here too.
        let pulled = if delta.new_sync_token.is_none() {
            Pulled::Listing {
                ctag: None,
                listed: delta
                    .changed
                    .into_iter()
                    .map(|r| (r.href, r.etag.unwrap_or_default()))
                    .collect(),
            }
        } else {
            Pulled::Delta(delta)
        };
        let (delta, _) = reconcile(pulled, &members);

        let need: Vec<String> = delta
            .changed
            .iter()
            .filter(|r| r.body.is_none())
            .map(|r| r.href.clone())
            .collect();
        let fetched = if need.is_empty() {
            Vec::new()
        } else {
            client
                .addressbook_multiget(href, &need)
                .await
                .map_err(dav_err)?
        };
        let mut resources = delta.changed.clone();
        resources.retain(|r| r.body.is_some());
        resources.extend(fetched);

        for res in resources {
            let Some(body) = res.body.as_deref() else {
                continue;
            };
            let cards = mw_ics::parse_vcard(body.as_bytes()).map_err(super::events::ics_err)?;
            let Some(c) = cards.into_iter().next() else {
                continue;
            };
            let uid = json_uid(&c.json);
            let id = match member_id(&members, &res.href) {
                Some(id) => Some(id),
                None => self.contact_id_for_uid(&book.id, &uid).await?,
            }
            .unwrap_or_else(|| gen_id("contact"));
            let existing = self.store().get_contact(&id).await?;
            let is_new = existing.is_none();
            let mut json = c.json;
            // The vCard does not carry what the engine owns (favourite, groups,
            // the stored photo blob): keep the local values (state.md t28-e9 note).
            if let Some(row) = &existing {
                overlay_engine_owned(&mut json, row);
            }
            self.persist_contact_with(
                account_id,
                &book.id,
                &id,
                &uid,
                json,
                res.etag.clone(),
                None,
                Some(ServerCopy {
                    body,
                    href: &res.href,
                }),
            )
            .await?;
            self.record_pim_change(
                account_id,
                ChangeType::ContactCard,
                &id,
                created_or_updated(is_new),
            )
            .await?;
        }
        for removed in &delta.removed {
            let id = match member_id(&members, removed) {
                Some(id) => Some(id),
                None => {
                    self.contact_id_for_uid(&book.id, &href_uid(removed))
                        .await?
                }
            };
            if let Some(id) = id {
                self.store().delete_contact(&id).await?;
                self.record_pim_change(
                    account_id,
                    ChangeType::ContactCard,
                    &id,
                    ChangeOp::Destroyed,
                )
                .await?;
            }
        }
        if delta.new_sync_token.is_some() {
            let mut updated = book.clone();
            updated.sync_token = delta.new_sync_token;
            self.store().upsert_address_book(&updated).await?;
        }
        Ok(())
    }

    // ── uid → local id lookups (scan; the collections are small) ─────────────

    pub(crate) async fn event_id_for_uid(
        &self,
        calendar_id: &str,
        uid: &str,
    ) -> Result<Option<String>> {
        Ok(self
            .store()
            .list_events(calendar_id)
            .await?
            .into_iter()
            .find(|e| e.uid == uid)
            .map(|e| e.id))
    }

    pub(crate) async fn task_id_for_uid(&self, list_id: &str, uid: &str) -> Result<Option<String>> {
        Ok(self
            .store()
            .list_tasks(list_id)
            .await?
            .into_iter()
            .find(|t| t.uid == uid)
            .map(|t| t.id))
    }

    pub(crate) async fn contact_id_for_uid(
        &self,
        book_id: &str,
        uid: &str,
    ) -> Result<Option<String>> {
        Ok(self
            .store()
            .list_contacts(book_id)
            .await?
            .into_iter()
            .find(|c| c.uid == uid)
            .map(|c| c.id))
    }
}

/// The server's own copy of a resource, adopted verbatim by the pull (t28-e19):
/// the body becomes the stored `ical_raw`/`vcard_raw` (nothing re-emitted, so
/// properties the projection does not model survive), and the href is recorded.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ServerCopy<'a> {
    pub(crate) body: &'a str,
    pub(crate) href: &'a str,
}

/// What a pull got from the server: an RFC 6578 delta, or — from a server without
/// `sync-collection` — the full member list with each member's etag.
enum Pulled {
    Delta(SyncDelta),
    Listing {
        ctag: Option<String>,
        listed: Vec<(String, String)>,
    },
}

/// Statuses a server answers an unsupported `sync-collection` REPORT with.
fn sync_unsupported(status: u16) -> bool {
    matches!(status, 400 | 403 | 405 | 409 | 415 | 501)
}

/// A delta against what is stored: a delta passes through; a member listing
/// becomes "changed" for every member whose etag differs from the stored one (or
/// that is not stored) and "removed" for every stored href the server no longer
/// lists. Returns the delta and, for a listing, the collection's new `ctag`.
fn reconcile(
    pulled: Pulled,
    members: &[(String, String, Option<String>)],
) -> (SyncDelta, Option<String>) {
    match pulled {
        Pulled::Delta(delta) => (delta, None),
        Pulled::Listing { ctag, listed } => {
            let changed = listed
                .iter()
                .filter(|(href, etag)| {
                    !members.iter().any(|(_, stored, stored_etag)| {
                        same_href(stored, href)
                            && !etag.is_empty()
                            && stored_etag.as_deref() == Some(etag.as_str())
                    })
                })
                .map(|(href, etag)| mw_dav::Resource {
                    href: href.clone(),
                    etag: (!etag.is_empty()).then(|| etag.clone()),
                    body: None,
                })
                .collect();
            let removed = members
                .iter()
                .filter(|(_, stored, _)| !listed.iter().any(|(href, _)| same_href(stored, href)))
                .map(|(_, stored, _)| stored.clone())
                .collect();
            (
                SyncDelta {
                    new_sync_token: None,
                    changed,
                    removed,
                },
                ctag,
            )
        }
    }
}

/// The path of an href (absolute URLs and root-relative paths compare equal),
/// without a trailing slash.
fn href_path(href: &str) -> &str {
    let path = match href.find("://") {
        Some(i) => {
            let rest = &href[i + 3..];
            rest.find('/').map_or("/", |j| &rest[j..])
        }
        None => href,
    };
    path.trim_end_matches('/')
}

fn same_href(a: &str, b: &str) -> bool {
    href_path(a) == href_path(b)
}

/// The local id stored for a server href, if any.
fn member_id(members: &[(String, String, Option<String>)], href: &str) -> Option<String> {
    members
        .iter()
        .find(|(_, stored, _)| same_href(stored, href))
        .map(|(id, ..)| id.clone())
}

fn created_or_updated(is_new: bool) -> ChangeOp {
    if is_new {
        ChangeOp::Created
    } else {
        ChangeOp::Updated
    }
}

/// Keep the fields the engine owns, which a vCard does not carry, from the stored
/// contact: favourite, group membership, the stored photo blob.
fn overlay_engine_owned(json: &mut Value, row: &mw_store::ContactRow) {
    let Some(obj) = json.as_object_mut() else {
        return;
    };
    obj.insert("isFavorite".into(), Value::Bool(row.is_favorite));
    if let Some(photo) = &row.photo_blob_id {
        obj.insert("photoBlobId".into(), Value::String(photo.clone()));
    }
    if let Some(groups) = row
        .json
        .as_deref()
        .and_then(|b| serde_json::from_slice::<Value>(b).ok())
        .and_then(|v| v.get("groupIds").cloned())
    {
        obj.insert("groupIds".into(), groups);
    }
}

// ── merging a local edit into the server's copy (t28-e19, hazard 4) ──────────

/// iCalendar properties the event/task projection controls: on a local edit the
/// emitted value wins, and one the projection no longer has is dropped. Every
/// other property of the server's copy (`X-*`, `CATEGORIES`, `ATTACH`, `URL`,
/// `CLASS`, …) is kept.
const ICS_MODELLED: &[&str] = &[
    "UID",
    "SUMMARY",
    "DESCRIPTION",
    "LOCATION",
    "DTSTART",
    "DTEND",
    "DUE",
    "DURATION",
    "RRULE",
    "RDATE",
    "EXDATE",
    "STATUS",
    "TRANSP",
    "PRIORITY",
    "PERCENT-COMPLETE",
    "SEQUENCE",
    "ORGANIZER",
    "ATTENDEE",
    "RELATED-TO",
    "RECURRENCE-ID",
];

/// vCard properties the contact projection controls (see [`ICS_MODELLED`]).
/// `PHOTO`, `URL`, `X-*`, `CATEGORIES`, … of the server's copy are kept.
const VCARD_MODELLED: &[&str] = &[
    "VERSION",
    "UID",
    "FN",
    "N",
    "KIND",
    "NICKNAME",
    "ORG",
    "TITLE",
    "NOTE",
    "EMAIL",
    "TEL",
    "ADR",
    "IMPP",
    "BDAY",
    "ANNIVERSARY",
    "KEY",
    "MEMBER",
];

/// RFC 5545/6350 logical lines of `text` (folded continuations joined).
fn unfold(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for raw in text.split('\n') {
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        if let Some(rest) = raw.strip_prefix([' ', '\t'])
            && let Some(last) = lines.last_mut()
        {
            last.push_str(rest);
        } else if !raw.is_empty() {
            lines.push(raw.to_string());
        }
    }
    lines
}

/// A content line folded at 75 octets on character boundaries, CRLF-terminated.
fn fold_into(out: &mut String, line: &str) {
    let mut width = 0;
    for c in line.chars() {
        let len = c.len_utf8();
        if width + len > 75 {
            out.push_str("\r\n ");
            width = 1;
        }
        out.push(c);
        width += len;
    }
    out.push_str("\r\n");
}

/// The property name of a content line, upper-cased, without a vCard group.
fn prop_name(line: &str) -> String {
    let end = line.find([';', ':']).unwrap_or(line.len());
    let name = &line[..end];
    let bare = name.rsplit('.').next().unwrap_or(name);
    bare.to_ascii_uppercase()
}

/// The property key as written (group included), upper-cased.
fn prop_key(line: &str) -> String {
    let end = line.find([';', ':']).unwrap_or(line.len());
    line[..end].to_ascii_uppercase()
}

/// The value of a content line (after the first `:` outside a quoted parameter).
fn prop_value(line: &str) -> &str {
    let mut quoted = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => quoted = !quoted,
            ':' if !quoted => return &line[i + 1..],
            _ => {}
        }
    }
    ""
}

/// One iCalendar component: its name, property lines and sub-components.
#[derive(Debug, Clone, Default)]
struct Comp {
    name: String,
    props: Vec<String>,
    subs: Vec<Comp>,
}

impl Comp {
    fn prop(&self, name: &str) -> Option<&str> {
        self.props
            .iter()
            .find(|p| prop_name(p) == name)
            .map(|p| prop_value(p))
    }

    fn has(&self, name: &str) -> bool {
        self.props.iter().any(|p| prop_name(p) == name)
    }

    /// `(UID, RECURRENCE-ID)` of an event/task/journal.
    fn key(&self) -> (String, String) {
        (
            self.prop("UID").unwrap_or_default().to_string(),
            self.prop("RECURRENCE-ID").unwrap_or_default().to_string(),
        )
    }

    fn write(&self, out: &mut String) {
        fold_into(out, &format!("BEGIN:{}", self.name));
        for p in &self.props {
            fold_into(out, p);
        }
        for s in &self.subs {
            s.write(out);
        }
        fold_into(out, &format!("END:{}", self.name));
    }
}

/// Parse iCalendar text into its top-level components (normally one
/// `VCALENDAR`). `None` when BEGIN/END do not nest.
fn parse_comps(text: &str) -> Option<Vec<Comp>> {
    let mut stack: Vec<Comp> = Vec::new();
    let mut top = Vec::new();
    for line in unfold(text) {
        if let Some(name) = line.strip_prefix("BEGIN:") {
            stack.push(Comp {
                name: name.trim().to_ascii_uppercase(),
                ..Comp::default()
            });
        } else if let Some(name) = line.strip_prefix("END:") {
            let done = stack.pop()?;
            if done.name != name.trim().to_ascii_uppercase() {
                return None;
            }
            match stack.last_mut() {
                Some(parent) => parent.subs.push(done),
                None => top.push(done),
            }
        } else {
            stack.last_mut()?.props.push(line);
        }
    }
    stack.is_empty().then_some(top)
}

fn is_item(name: &str) -> bool {
    matches!(name, "VEVENT" | "VTODO" | "VJOURNAL")
}

/// Merge a local edit (`emitted`, the projection's serialisation) into the
/// server's copy (`stored`): the emitted object, plus everything of the stored
/// one the projection does not model — calendar-level properties, time zones,
/// unmodelled properties of the edited component, and recurrence overrides the
/// projection does not carry. When either side does not parse, `emitted` is
/// returned unchanged.
pub(crate) fn merge_ics(stored: &str, emitted: &str) -> String {
    let (Some(stored), Some(mut merged)) = (parse_comps(stored), parse_comps(emitted)) else {
        return emitted.to_string();
    };
    let (Some(old), Some(new)) = (
        stored.into_iter().find(|c| c.name == "VCALENDAR"),
        merged.iter_mut().find(|c| c.name == "VCALENDAR"),
    ) else {
        return emitted.to_string();
    };
    for p in &old.props {
        let n = prop_name(p);
        if !matches!(n.as_str(), "VERSION" | "PRODID" | "METHOD") && !new.has(&n) {
            new.props.push(p.clone());
        }
    }
    for sub in old.subs {
        if sub.name == "VTIMEZONE" {
            let tzid = sub.prop("TZID").unwrap_or_default().to_string();
            if !new
                .subs
                .iter()
                .any(|s| s.name == "VTIMEZONE" && s.prop("TZID") == Some(tzid.as_str()))
            {
                new.subs.insert(0, sub);
            }
        } else if is_item(&sub.name) {
            let key = sub.key();
            match new
                .subs
                .iter_mut()
                .find(|s| s.name == sub.name && s.key() == key)
            {
                Some(target) => {
                    for p in &sub.props {
                        let n = prop_name(p);
                        if !ICS_MODELLED.contains(&n.as_str()) && !target.has(&n) {
                            target.props.push(p.clone());
                        }
                    }
                    for inner in sub.subs {
                        // Alarms are modelled (the projection's `alerts`).
                        if inner.name != "VALARM"
                            && !target.subs.iter().any(|s| s.name == inner.name)
                        {
                            target.subs.push(inner);
                        }
                    }
                }
                // An override the projection does not carry stays as the server has it.
                None if !key.1.is_empty() => new.subs.push(sub),
                None => {}
            }
        } else if !new.subs.iter().any(|s| s.name == sub.name) {
            new.subs.push(sub);
        }
    }
    let mut out = String::new();
    for c in &merged {
        c.write(&mut out);
    }
    out
}

/// [`merge_ics`] for a vCard: the emitted card plus every property of the stored
/// card the projection does not model (`PHOTO`, `URL`, `X-*`, …), unless the
/// emitted card has the same property key.
pub(crate) fn merge_vcard(stored: &str, emitted: &str) -> String {
    let old = unfold(stored);
    let new = unfold(emitted);
    let (Some(end), true) = (
        new.iter()
            .rposition(|l| l.eq_ignore_ascii_case("END:VCARD")),
        old.first()
            .is_some_and(|l| l.eq_ignore_ascii_case("BEGIN:VCARD")),
    ) else {
        return emitted.to_string();
    };
    let mut lines: Vec<String> = new[..end].to_vec();
    let keys: Vec<String> = lines.iter().map(|l| prop_key(l)).collect();
    for l in &old {
        if l.eq_ignore_ascii_case("BEGIN:VCARD") || l.eq_ignore_ascii_case("END:VCARD") {
            continue;
        }
        let n = prop_name(l);
        if !VCARD_MODELLED.contains(&n.as_str()) && !keys.contains(&prop_key(l)) {
            lines.push(l.clone());
        }
    }
    lines.extend(new[end..].iter().cloned());
    let mut out = String::new();
    for l in &lines {
        fold_into(&mut out, l);
    }
    out
}

/// The `uid` field of a projection (empty string when absent).
fn json_uid(json: &Value) -> String {
    json.get("uid")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// The resource uid encoded in an href: its last path segment without the
/// `.ics` / `.vcf` extension (Radicale + most servers name resources `<uid>.ext`).
fn href_uid(href: &str) -> String {
    let base = href
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(href);
    base.rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(base)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const STORED: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Server//EN\r\n\
X-WR-CALNAME:Work\r\nBEGIN:VTIMEZONE\r\nTZID:Europe/Paris\r\nEND:VTIMEZONE\r\n\
BEGIN:VEVENT\r\nUID:u1\r\nSUMMARY:Old\r\nLOCATION:Room 1\r\nX-KEEP:yes\r\n\
CATEGORIES:Board\r\nBEGIN:VALARM\r\nACTION:DISPLAY\r\nTRIGGER:-PT5M\r\nEND:VALARM\r\n\
END:VEVENT\r\nBEGIN:VEVENT\r\nUID:u1\r\nRECURRENCE-ID:20260714T090000Z\r\n\
SUMMARY:Override\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

    const EMITTED: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Mailwoman//mw-ics//EN\r\n\
BEGIN:VEVENT\r\nUID:u1\r\nSUMMARY:New\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

    #[test]
    fn a_merge_keeps_what_the_projection_does_not_model_and_drops_what_it_removed() {
        let merged = merge_ics(STORED, EMITTED);
        assert!(merged.contains("SUMMARY:New\r\n"), "{merged}");
        assert!(!merged.contains("SUMMARY:Old"), "the edit wins: {merged}");
        assert!(merged.contains("X-KEEP:yes\r\n"), "{merged}");
        assert!(merged.contains("CATEGORIES:Board\r\n"), "{merged}");
        assert!(merged.contains("X-WR-CALNAME:Work\r\n"), "{merged}");
        assert!(
            merged.contains("TZID:Europe/Paris"),
            "the time zone stays: {merged}"
        );
        assert!(
            merged.contains("SUMMARY:Override"),
            "the override stays: {merged}"
        );
        assert!(
            merged.contains("PRODID:-//Mailwoman//mw-ics//EN"),
            "{merged}"
        );
        // LOCATION is modelled: the projection no longer has it, so it is gone.
        assert!(!merged.contains("LOCATION:"), "{merged}");
        // Alarms are modelled (`alerts`): none emitted, none kept.
        assert!(!merged.contains("VALARM"), "{merged}");
        assert!(parse_comps(&merged).is_some(), "the result nests");
    }

    #[test]
    fn a_merge_falls_back_to_the_emitted_text_when_either_side_does_not_parse() {
        assert_eq!(merge_ics("BEGIN:VCALENDAR\r\n", EMITTED), EMITTED);
        assert_eq!(merge_ics(STORED, "garbage"), "garbage");
    }

    #[test]
    fn long_lines_are_folded_and_unfold_back() {
        let long = format!("X-LONG:{}", "é".repeat(60));
        let mut out = String::new();
        fold_into(&mut out, &long);
        assert!(out.split("\r\n").all(|l| l.len() <= 75), "{out}");
        assert_eq!(unfold(&out), vec![long]);
    }

    #[test]
    fn a_vcard_merge_keeps_photo_url_and_extensions() {
        let stored = "BEGIN:VCARD\r\nVERSION:3.0\r\nUID:c1\r\nFN:Ann\r\nNOTE:old\r\n\
PHOTO;VALUE=uri:https://img.example/a.jpg\r\nitem1.URL:https://ann.example\r\n\
X-CUSTOM:keep\r\nEND:VCARD\r\n";
        let emitted = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:c1\r\nFN:Ann Lee\r\nEND:VCARD\r\n";
        let merged = merge_vcard(stored, emitted);
        assert!(
            merged.starts_with("BEGIN:VCARD\r\nVERSION:4.0\r\n"),
            "{merged}"
        );
        assert!(merged.contains("FN:Ann Lee\r\n") && !merged.contains("FN:Ann\r\n"));
        assert!(merged.contains("PHOTO;VALUE=uri:https://img.example/a.jpg\r\n"));
        assert!(merged.contains("item1.URL:https://ann.example\r\n"));
        assert!(merged.contains("X-CUSTOM:keep\r\n"));
        assert!(
            !merged.contains("NOTE:old"),
            "a cleared modelled field stays cleared"
        );
        assert!(merged.ends_with("END:VCARD\r\n"));
    }

    #[test]
    fn a_listing_reconciles_to_changed_and_removed() {
        let members = vec![
            (
                "e1".to_string(),
                "/cal/a.ics".to_string(),
                Some("\"1\"".to_string()),
            ),
            (
                "e2".to_string(),
                "https://dav.example/cal/b.ics".to_string(),
                Some("\"1\"".to_string()),
            ),
        ];
        let (delta, ctag) = reconcile(
            Pulled::Listing {
                ctag: Some("c9".into()),
                listed: vec![
                    ("/cal/a.ics".into(), "\"1\"".into()),
                    ("/cal/c.ics".into(), "\"5\"".into()),
                ],
            },
            &members,
        );
        assert_eq!(ctag.as_deref(), Some("c9"));
        let changed: Vec<&str> = delta.changed.iter().map(|r| r.href.as_str()).collect();
        assert_eq!(changed, ["/cal/c.ics"], "a.ics is unchanged, c.ics is new");
        assert_eq!(delta.removed, ["https://dav.example/cal/b.ics"]);
        assert_eq!(
            member_id(&members, "https://dav.example/cal/a.ics").as_deref(),
            Some("e1")
        );
        assert!(sync_unsupported(501) && !sync_unsupported(412) && !sync_unsupported(500));
    }
}
