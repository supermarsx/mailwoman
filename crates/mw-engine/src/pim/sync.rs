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

use mw_dav::{DavClient, DavConfig};
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
    /// are skipped. Best-effort per collection — one failure does not abort the
    /// rest — so the surface degrades rather than stalls.
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
        for cal in self.store().list_calendars(account_id).await? {
            if cal.caldav_url.is_some()
                && let Err(e) = self.pull_calendar(account_id, &cal, &base).await
            {
                tracing::warn!("CalDAV pull for calendar {} failed: {e}", cal.id);
            }
        }
        for book in self.store().list_address_books(account_id).await? {
            if book.carddav_url.is_some()
                && let Err(e) = self.pull_address_book(account_id, &book, &base).await
            {
                tracing::warn!("CardDAV pull for book {} failed: {e}", book.id);
            }
        }
        self.broadcast_state(account_id).await;
        Ok(())
    }

    /// Incrementally pull one CalDAV collection (events or a VTODO task list)
    /// and reconcile it into the store, advancing the stored `sync-token`.
    pub(crate) async fn pull_calendar(
        &self,
        account_id: &str,
        cal: &mw_store::CalendarRow,
        base: &DavConfig,
    ) -> Result<()> {
        let href = cal.caldav_url.as_deref().unwrap_or_default();
        let client = DavClient::new(base.clone()).map_err(dav_err)?;
        let delta = client
            .sync_collection(href, cal.sync_token.as_deref())
            .await
            .map_err(dav_err)?;

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
            for p in parsed {
                if p.component != cal.component {
                    continue;
                }
                let uid = json_uid(&p.json);
                if cal.component == "VTODO" {
                    let id = self
                        .task_id_for_uid(&cal.id, &uid)
                        .await?
                        .unwrap_or_else(|| gen_id("task"));
                    let is_new = self.store().get_task(&id).await?.is_none();
                    self.persist_task(
                        account_id,
                        &cal.id,
                        &id,
                        &uid,
                        p.json,
                        res.etag.clone(),
                        None,
                    )
                    .await?;
                    self.record_pim_change(
                        account_id,
                        ChangeType::Task,
                        &id,
                        if is_new {
                            ChangeOp::Created
                        } else {
                            ChangeOp::Updated
                        },
                    )
                    .await?;
                } else {
                    let id = self
                        .event_id_for_uid(&cal.id, &uid)
                        .await?
                        .unwrap_or_else(|| gen_id("ev"));
                    let is_new = self.store().get_event(&id).await?.is_none();
                    self.persist_event(
                        account_id,
                        &cal.id,
                        &id,
                        &uid,
                        p.json,
                        res.etag.clone(),
                        None,
                    )
                    .await?;
                    self.record_pim_change(
                        account_id,
                        ChangeType::CalendarEvent,
                        &id,
                        if is_new {
                            ChangeOp::Created
                        } else {
                            ChangeOp::Updated
                        },
                    )
                    .await?;
                }
            }
        }

        // Tombstones: a removed href's basename is the resource uid.
        for removed in &delta.removed {
            let uid = href_uid(removed);
            if cal.component == "VTODO" {
                if let Some(id) = self.task_id_for_uid(&cal.id, &uid).await? {
                    self.store().delete_task(&id).await?;
                    self.record_pim_change(account_id, ChangeType::Task, &id, ChangeOp::Destroyed)
                        .await?;
                }
            } else if let Some(id) = self.event_id_for_uid(&cal.id, &uid).await? {
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

        // Persist the advanced sync-token.
        if delta.new_sync_token.is_some() {
            let mut updated = cal.clone();
            updated.sync_token = delta.new_sync_token;
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
        let delta = client
            .sync_addressbook(href, book.sync_token.as_deref())
            .await
            .map_err(dav_err)?;

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
            for c in cards {
                let uid = json_uid(&c.json);
                let id = self
                    .contact_id_for_uid(&book.id, &uid)
                    .await?
                    .unwrap_or_else(|| gen_id("contact"));
                let is_new = self.store().get_contact(&id).await?.is_none();
                self.persist_contact(
                    account_id,
                    &book.id,
                    &id,
                    &uid,
                    c.json,
                    res.etag.clone(),
                    None,
                )
                .await?;
                self.record_pim_change(
                    account_id,
                    ChangeType::ContactCard,
                    &id,
                    if is_new {
                        ChangeOp::Created
                    } else {
                        ChangeOp::Updated
                    },
                )
                .await?;
            }
        }
        for removed in &delta.removed {
            let uid = href_uid(removed);
            if let Some(id) = self.contact_id_for_uid(&book.id, &uid).await? {
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

    async fn contact_id_for_uid(&self, book_id: &str, uid: &str) -> Result<Option<String>> {
        Ok(self
            .store()
            .list_contacts(book_id)
            .await?
            .into_iter()
            .find(|c| c.uid == uid)
            .map(|c| c.id))
    }
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
