#![forbid(unsafe_code)]
//! `mw-dav` — the shared DAV/HTTP core plus the CalDAV surface for Mailwoman V3
//! (plan §0.1, §1.3, SPEC §6.2/§11).
//!
//! Hand-rolled over `reqwest` (rustls) + `quick-xml` — no DAV framework (plan
//! §1.3). This crate owns the **shared DAV core** that `mw-carddav` reuses:
//! discovery (`.well-known` → current-user-principal → home-set), `PROPFIND`,
//! `REPORT`, `MKCALENDAR`, `PUT`/`GET`/`DELETE` with `ETag` `If-Match`/
//! `If-None-Match`, `sync-collection` (RFC 6578) with `sync-token`, and the
//! `ctag` + etag-diff fallback (§2.3). On top of it sits the **CalDAV** surface
//! (list calendars, pull events/tasks by component, push, `calendar-query`/
//! `calendar-multiget`, `free-busy-query`). Bodies (de)serialize through
//! `mw-ics`.
//!
//! ## Scaffolder note (e0)
//! e0 freezes the module layout, the DTO seams, and the public client
//! signatures; **e2** fills every `todo!()` body, adds the recorded-fixture
//! unit tests (Radicale + Google/M365 quirks), and the `cargo-fuzz` target over
//! DAV XML parsing (plan §1.9). No logic yet.

use serde::{Deserialize, Serialize};

/// A recoverable DAV/HTTP failure.
#[derive(Debug, thiserror::Error)]
pub enum DavError {
    #[error("http transport error: {0}")]
    Transport(String),
    #[error("DAV XML parse error: {0}")]
    Xml(String),
    #[error("unexpected DAV status {status}: {body}")]
    Status { status: u16, body: String },
    #[error("etag precondition failed (412) — remote changed, re-pull required")]
    Conflict,
    #[error("serialization error: {0}")]
    Ics(#[from] mw_ics::IcsError),
}

/// The convenience result alias for this crate.
pub type Result<T> = std::result::Result<T, DavError>;

/// Credentials + base URL for a DAV account. The shared core is protocol-
/// agnostic; CalDAV/CardDAV layer their collection semantics on top.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DavConfig {
    /// The server base or discovery URL (`.well-known/caldav` is tried).
    pub base_url: String,
    pub username: String,
    pub password: String,
}

/// A discovered DAV collection (calendar or address book): its href plus the
/// sync capabilities the engine feature-detects on (`sync-token` vs `ctag`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Collection {
    pub href: String,
    pub display_name: String,
    /// `calendar-color` / `addressbook` colour, if advertised.
    pub color: Option<String>,
    /// `getctag`, when the server exposes it (the fallback sync key).
    pub ctag: Option<String>,
    /// `sync-token`, when the server advertises RFC 6578 sync-collection.
    pub sync_token: Option<String>,
    /// Supported component set (`VEVENT` / `VTODO`) for a calendar collection.
    pub components: Vec<String>,
}

/// One remote resource in a collection: its href + `ETag` + body bytes. Bodies
/// are opaque here; `mw-ics` parses them at the engine seam.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    pub href: String,
    pub etag: Option<String>,
    /// The iCalendar / vCard body (absent for a delete-tombstone href).
    pub body: Option<String>,
}

/// The result of an incremental `sync-collection` (RFC 6578) pull: the new
/// `sync-token` plus the changed and removed hrefs (§2.3).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncDelta {
    pub new_sync_token: Option<String>,
    pub changed: Vec<Resource>,
    pub removed: Vec<String>,
}

/// The shared DAV/HTTP client (plan §1.3). `mw-carddav` constructs its own
/// surface over the same core methods.
pub struct DavClient {
    _config: DavConfig,
    _http: reqwest::Client,
}

impl DavClient {
    /// Construct a DAV client for an account (rustls `reqwest`, e2).
    pub fn new(config: DavConfig) -> Result<Self> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| DavError::Transport(e.to_string()))?;
        Ok(Self {
            _config: config,
            _http: http,
        })
    }

    // ── shared DAV core (reused by mw-carddav) ──────────────────────────────

    /// Discover the principal's collection home-set and enumerate collections
    /// (`.well-known` → current-user-principal → home-set → `PROPFIND`, §2.3).
    pub async fn discover_collections(&self) -> Result<Vec<Collection>> {
        todo!("e2: .well-known → current-user-principal → home-set → PROPFIND")
    }

    /// Incremental pull via `sync-collection` (RFC 6578) from `sync_token`, or
    /// the `ctag` + full-`PROPFIND` etag-diff fallback when unsupported (§2.3).
    pub async fn sync_collection(
        &self,
        _collection_href: &str,
        _sync_token: Option<&str>,
    ) -> Result<SyncDelta> {
        todo!("e2: sync-collection REPORT + ctag/etag-diff fallback")
    }

    /// `PUT` a resource with `If-Match:<etag>` (update) or `If-None-Match:*`
    /// (create); a `412` maps to [`DavError::Conflict`] (§2.3).
    pub async fn put_resource(
        &self,
        _href: &str,
        _body: &str,
        _if_match: Option<&str>,
    ) -> Result<String> {
        todo!("e2: PUT with If-Match/If-None-Match, returning the new ETag")
    }

    /// `DELETE` a resource with `If-Match:<etag>` (§2.3).
    pub async fn delete_resource(&self, _href: &str, _if_match: Option<&str>) -> Result<()> {
        todo!("e2: DELETE with If-Match")
    }

    // ── CalDAV surface ──────────────────────────────────────────────────────

    /// `MKCALENDAR` a new calendar collection (§2.3).
    pub async fn make_calendar(&self, _href: &str, _display_name: &str) -> Result<()> {
        todo!("e2: MKCALENDAR")
    }

    /// `calendar-multiget` bodies for a set of hrefs (RFC 4791, §2.3).
    pub async fn calendar_multiget(
        &self,
        _collection_href: &str,
        _hrefs: &[String],
    ) -> Result<Vec<Resource>> {
        todo!("e2: calendar-multiget REPORT")
    }

    /// `free-busy-query` a collection over a window, returning busy intervals
    /// (RFC 4791, feeds `Calendar/freeBusy`, §2.2).
    pub async fn free_busy_query(
        &self,
        _collection_href: &str,
        _window_start: &str,
        _window_end: &str,
    ) -> Result<Vec<mw_ics::BusyInterval>> {
        todo!("e2: free-busy-query REPORT")
    }
}
