#![forbid(unsafe_code)]
//! `mw-carddav` — the CardDAV client for Mailwoman V3 (plan §0.4, §1.3,
//! SPEC §6.2/§13).
//!
//! Thin CardDAV surface (`addressbook-query`/`addressbook-multiget`, RFC 6352,
//! discovery, sync-token/ctag pull, `PUT`/`DELETE` with `If-Match`) built by
//! **reusing `mw-dav`'s shared DAV core** — no duplicated HTTP/XML plumbing
//! (plan §1.3). vCard bodies (de)serialize through `mw-ics`.
//!
//! ## Scaffolder note (e0)
//! e0 freezes the module layout + the public client signatures; **e3** fills
//! every `todo!()` body and adds the recorded-fixture unit tests (Radicale +
//! Google quirk). No logic yet.

use mw_dav::{DavClient, DavConfig, Resource, SyncDelta};

/// A recoverable CardDAV failure — CardDAV errors surface through the shared
/// [`mw_dav::DavError`].
pub type Error = mw_dav::DavError;

/// The convenience result alias for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// The CardDAV client: owns a shared [`DavClient`] and adds the address-book
/// specific REPORTs on top (plan §1.3).
pub struct CardDavClient {
    dav: DavClient,
}

impl CardDavClient {
    /// Construct a CardDAV client over the shared DAV core (e3).
    pub fn new(config: DavConfig) -> Result<Self> {
        Ok(Self {
            dav: DavClient::new(config)?,
        })
    }

    /// The underlying shared DAV core (discovery + sync + PUT/DELETE are reused
    /// verbatim; `.well-known/carddav` discovery runs through it).
    pub fn dav(&self) -> &DavClient {
        &self.dav
    }

    /// Incremental pull of an address-book collection (sync-token where
    /// advertised, ctag+etag-diff fallback) — delegates to the shared core
    /// (§2.3). Present so the engine drives calendars + address books through a
    /// uniform seam.
    pub async fn sync_addressbook(
        &self,
        _collection_href: &str,
        _sync_token: Option<&str>,
    ) -> Result<SyncDelta> {
        todo!("e3: addressbook sync via the shared mw-dav sync-collection core")
    }

    /// `addressbook-multiget` vCard bodies for a set of hrefs (RFC 6352, §2.3).
    pub async fn addressbook_multiget(
        &self,
        _collection_href: &str,
        _hrefs: &[String],
    ) -> Result<Vec<Resource>> {
        todo!("e3: addressbook-multiget REPORT")
    }

    /// `addressbook-query` a collection (RFC 6352 filter → matching cards, §2.3).
    pub async fn addressbook_query(&self, _collection_href: &str) -> Result<Vec<Resource>> {
        todo!("e3: addressbook-query REPORT")
    }
}
