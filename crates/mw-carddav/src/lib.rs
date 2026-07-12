#![forbid(unsafe_code)]
//! `mw-carddav` — the CardDAV client for Mailwoman V3 (plan §0.4, §1.3,
//! SPEC §6.2/§13).
//!
//! A thin CardDAV surface (`addressbook-query`/`addressbook-multiget`, RFC 6352,
//! discovery, sync-token/ctag pull, `PUT`/`DELETE` with `If-Match`) built to
//! **reuse `mw-dav`'s shared DAV core** (plan §1.3). It layers the
//! CardDAV-namespaced REPORTs on top and projects vCard bodies to the frozen
//! `ContactCard` shape (§2.1).
//!
//! ## Seam / integration points (dependencies still resuming)
//! `mw-dav` (e2) and `mw-ics` (e1) build in parallel; their public bodies are
//! still `todo!()` stubs, so this crate builds against the **frozen contract**
//! (§2.1/§2.3) plus a thin local seam and integrates as they land:
//! - **DAV core:** `mw-dav`'s DTOs (`DavConfig`, `Collection`, `Resource`,
//!   `SyncDelta`, `DavError`) are frozen and reused verbatim; a live
//!   [`mw_dav::DavClient`] is held (see [`CardDavClient::dav`]) so discovery/sync
//!   collapse to delegation once `e2` exposes generic REPORT/PROPFIND helpers.
//!   Until then the REPORTs ride the in-tree `reqwest` directly ([`transport`]).
//! - **vCard:** the authoritative parser is `mw_ics::parse_vcard`; while it is a
//!   `todo!()` stub, [`vcard::from_vcard`] is a thin local projection over the
//!   common fields, producing the identical `ContactCard` wire shape so the swap
//!   is mechanical (see `vcard.rs`).

mod request;
mod response;
mod transport;
pub mod vcard;

pub use mw_dav::{Collection, DavConfig, Resource, SyncDelta};
pub use vcard::{Anniversary, ContactCard, ContactEmail, ContactName, ContactValue};

use transport::{Http, expect_multistatus, expect_write};

/// A recoverable CardDAV failure — CardDAV errors surface through the shared
/// [`mw_dav::DavError`].
pub type Error = mw_dav::DavError;

/// The convenience result alias for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// The CardDAV client: owns the shared [`mw_dav::DavClient`] (the delegation
/// seam) and the CardDAV HTTP transport, adding the address-book REPORTs
/// (plan §1.3).
pub struct CardDavClient {
    dav: mw_dav::DavClient,
    http: Http,
}

impl CardDavClient {
    /// Construct a CardDAV client over the shared DAV core (rustls `reqwest`).
    pub fn new(config: DavConfig) -> Result<Self> {
        let dav = mw_dav::DavClient::new(config.clone())?;
        let http = Http::new(config)?;
        Ok(Self { dav, http })
    }

    /// The underlying shared DAV core. Discovery + sync + `PUT`/`DELETE` will
    /// delegate here verbatim once `mw-dav` (e2) publishes its generic
    /// REPORT/PROPFIND primitives (the seam noted at the crate root).
    pub fn dav(&self) -> &mw_dav::DavClient {
        &self.dav
    }

    /// Discover the account's address-book collections: `.well-known/carddav`
    /// (followed via redirects) → `current-user-principal` → `addressbook-home-set`
    /// → `PROPFIND Depth:1`, keeping only `addressbook` resourcetypes (§2.3).
    pub async fn discover_addressbooks(&self) -> Result<Vec<Collection>> {
        // Step 1: current-user-principal off the base/well-known URL.
        let r1 = self
            .http
            .propfind("", "0", request::propfind_current_user_principal())
            .await?;
        expect_multistatus(&r1)?;
        let principal =
            response::first_href_in(&r1.body, "current-user-principal")?.unwrap_or_default();

        // Step 2: addressbook-home-set off the principal.
        let r2 = self
            .http
            .propfind(&principal, "0", request::propfind_addressbook_home_set())
            .await?;
        expect_multistatus(&r2)?;
        let home = response::first_href_in(&r2.body, "addressbook-home-set")?.unwrap_or(principal);

        // Step 3: enumerate collections under the home-set.
        let r3 = self
            .http
            .propfind(&home, "1", request::propfind_collections())
            .await?;
        expect_multistatus(&r3)?;
        response::parse_collections(&r3.body)
    }

    /// Incremental pull of an address-book collection: `sync-collection`
    /// (RFC 6578) where advertised, else the `ctag` + etag-diff fallback
    /// (a full member listing the engine diffs against stored etags, §2.3).
    pub async fn sync_addressbook(
        &self,
        collection_href: &str,
        sync_token: Option<&str>,
    ) -> Result<SyncDelta> {
        let resp = self
            .http
            .report(
                collection_href,
                "1",
                request::report_sync_collection(sync_token),
            )
            .await?;
        if resp.status == 207 {
            return response::parse_sync_delta(&resp.body);
        }
        // Fallback: sync-collection unsupported (400/403/409/501) → ctag+etag list.
        self.ctag_fallback(collection_href).await
    }

    /// The `ctag` + etag-diff fallback pull: `PROPFIND Depth:1` for member etags,
    /// returned as an all-`changed` [`SyncDelta`] (no `sync-token`) that the
    /// engine reconciles against its stored etags (§2.3, plan risk #2).
    async fn ctag_fallback(&self, collection_href: &str) -> Result<SyncDelta> {
        let resp = self
            .http
            .propfind(collection_href, "1", request::propfind_etag_list())
            .await?;
        expect_multistatus(&resp)?;
        let (_ctag, members) = response::parse_etag_list(&resp.body)?;
        let changed = members
            .into_iter()
            .map(|(href, etag)| Resource {
                href,
                etag: Some(etag),
                body: None,
            })
            .collect();
        Ok(SyncDelta {
            new_sync_token: None,
            changed,
            removed: Vec::new(),
        })
    }

    /// `addressbook-query` a collection (RFC 6352 → every card's href + etag +
    /// vCard body, §2.3).
    pub async fn addressbook_query(&self, collection_href: &str) -> Result<Vec<Resource>> {
        let resp = self
            .http
            .report(collection_href, "1", request::addressbook_query())
            .await?;
        expect_multistatus(&resp)?;
        response::parse_resources(&resp.body)
    }

    /// `addressbook-multiget` vCard bodies for a set of hrefs (RFC 6352, §2.3).
    pub async fn addressbook_multiget(
        &self,
        collection_href: &str,
        hrefs: &[String],
    ) -> Result<Vec<Resource>> {
        if hrefs.is_empty() {
            return Ok(Vec::new());
        }
        let resp = self
            .http
            .report(collection_href, "1", request::addressbook_multiget(hrefs))
            .await?;
        expect_multistatus(&resp)?;
        response::parse_resources(&resp.body)
    }

    /// `addressbook-multiget` + project each vCard to a [`ContactCard`] (§2.1) —
    /// the `addressbook-multiget → ContactCard` path. The resource href seeds the
    /// card `id`; `collection_href` seeds `addressBookId`. vCard projection is the
    /// `mw_ics::parse_vcard` seam (see [`vcard`]).
    pub async fn fetch_cards(
        &self,
        collection_href: &str,
        hrefs: &[String],
    ) -> Result<Vec<ContactCard>> {
        let resources = self.addressbook_multiget(collection_href, hrefs).await?;
        Ok(resources_to_cards(collection_href, &resources))
    }

    /// `PUT` a vCard: `If-Match:<etag>` updates, `If-None-Match:*` creates; a
    /// `412` maps to [`Error::Conflict`] (re-pull required, §2.3). Returns the
    /// new `ETag` if the server reported one (empty otherwise — the caller
    /// re-`GET`s to learn it).
    pub async fn put_contact(
        &self,
        href: &str,
        vcard: &str,
        if_match: Option<&str>,
    ) -> Result<String> {
        let resp = self.http.put(href, vcard.to_string(), if_match).await?;
        expect_write(&resp)?;
        Ok(resp.etag.unwrap_or_default())
    }

    /// `DELETE` a resource with `If-Match:<etag>`; a `412` maps to
    /// [`Error::Conflict`] (§2.3).
    pub async fn delete_contact(&self, href: &str, if_match: Option<&str>) -> Result<()> {
        let resp = self.http.delete(href, if_match).await?;
        expect_write(&resp)
    }
}

/// Project multiget resources (with vCard bodies) to [`ContactCard`]s (§2.1).
/// Resources without a body (delete tombstones) are skipped.
fn resources_to_cards(collection_href: &str, resources: &[Resource]) -> Vec<ContactCard> {
    resources
        .iter()
        .filter_map(|r| {
            let body = r.body.as_deref()?;
            Some(vcard::from_vcard(
                body,
                &r.href,
                collection_href,
                r.etag.clone(),
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        let path = format!("{}/fixtures/carddav/{}", env!("CARGO_MANIFEST_DIR"), name);
        // Fixtures live at the workspace root; hop up from the crate dir.
        let root = format!(
            "{}/../../fixtures/carddav/{}",
            env!("CARGO_MANIFEST_DIR"),
            name
        );
        std::fs::read_to_string(&root)
            .or_else(|_| std::fs::read_to_string(&path))
            .unwrap_or_else(|e| panic!("read fixture {name}: {e}"))
    }

    #[test]
    fn discovery_keeps_only_addressbook_collections() {
        let books = response::parse_collections(&fixture("discovery_collections.xml")).unwrap();
        // The fixture has an addressbook + a non-addressbook (calendar/home) —
        // only the addressbook survives the resourcetype filter.
        assert_eq!(books.len(), 1);
        let b = &books[0];
        assert_eq!(b.href, "/addressbooks/ada/contacts/");
        assert_eq!(b.display_name, "Contacts");
        assert_eq!(b.ctag.as_deref(), Some("3145"));
        assert_eq!(
            b.sync_token.as_deref(),
            Some("http://radicale.org/ns/sync/3145")
        );
    }

    #[test]
    fn discovery_principal_and_home_set_chain() {
        assert_eq!(
            response::first_href_in(
                &fixture("discovery_principal.xml"),
                "current-user-principal"
            )
            .unwrap()
            .as_deref(),
            Some("/principals/ada/")
        );
        assert_eq!(
            response::first_href_in(&fixture("discovery_home_set.xml"), "addressbook-home-set")
                .unwrap()
                .as_deref(),
            Some("/addressbooks/ada/")
        );
    }

    #[test]
    fn sync_collection_reports_changed_removed_and_token() {
        let delta = response::parse_sync_delta(&fixture("sync_collection.xml")).unwrap();
        assert_eq!(
            delta.new_sync_token.as_deref(),
            Some("http://radicale.org/ns/sync/3146")
        );
        // Two changed cards + one removed tombstone (404).
        assert_eq!(delta.changed.len(), 2);
        assert_eq!(delta.changed[0].href, "/addressbooks/ada/contacts/ada.vcf");
        assert_eq!(delta.changed[0].etag.as_deref(), Some("\"e-ada-2\""));
        assert_eq!(delta.removed, vec!["/addressbooks/ada/contacts/old.vcf"]);
    }

    #[test]
    fn ctag_fallback_lists_member_etags() {
        let (ctag, members) = response::parse_etag_list(&fixture("etag_list.xml")).unwrap();
        assert_eq!(ctag.as_deref(), Some("3146"));
        assert_eq!(members.len(), 2);
        assert!(
            members
                .iter()
                .any(|(h, e)| h == "/addressbooks/ada/contacts/ada.vcf" && e == "\"e-ada-2\"")
        );
    }

    #[test]
    fn multiget_projects_to_contact_cards() {
        let resources = response::parse_resources(&fixture("multiget.xml")).unwrap();
        assert_eq!(resources.len(), 2);
        let cards = resources_to_cards("/addressbooks/ada/contacts/", &resources);
        assert_eq!(cards.len(), 2);
        let ada = cards
            .iter()
            .find(|c| c.name.full == "Ada Lovelace")
            .expect("ada card");
        assert_eq!(ada.address_book_id, "/addressbooks/ada/contacts/");
        assert_eq!(ada.emails.len(), 1);
        assert_eq!(ada.emails[0].value, "ada@example.org");
        assert_eq!(ada.etag.as_deref(), Some("\"e-ada-2\""));
        assert_eq!(ada.uid, "urn-ada");
    }

    #[test]
    fn google_quirk_full_url_hrefs_and_prefixes_parse() {
        // Google returns full-URL hrefs + different ns prefixes; the local-name
        // walker handles both (plan risk #2 — quirk fixtures).
        let resources = response::parse_resources(&fixture("google_multiget.xml")).unwrap();
        assert_eq!(resources.len(), 1);
        assert!(resources[0].href.starts_with("https://"));
        assert!(
            resources[0]
                .body
                .as_deref()
                .unwrap()
                .contains("BEGIN:VCARD")
        );
        let cards = resources_to_cards("book", &resources);
        assert_eq!(cards[0].name.full, "Grace Hopper");
    }

    #[test]
    fn put_conflict_maps_to_conflict_error() {
        // Unit-level assertion of the 412→Conflict mapping used by put/delete.
        let resp = transport::HttpResponse {
            status: 412,
            etag: None,
            body: String::new(),
        };
        match transport::expect_write(&resp) {
            Err(Error::Conflict) => {}
            other => panic!("expected Conflict, got {other:?}"),
        }
    }
}
