//! Pure CardDAV XML request-body builders (RFC 6352, plan §1.3/§2.3).
//!
//! Every function is a pure `&…` → `String` transform with no I/O, so the wire
//! bodies are unit-tested directly (no live server). This is the CardDAV-facing
//! half of the thin seam described in `Cargo.toml`: it mirrors the shape of
//! `mw-dav`'s shared request builders but keyed to the CardDAV namespace. When
//! `mw-dav` exposes generic `PROPFIND`/`REPORT` primitives (`DavKind::CardDav`),
//! these collapse to delegation (integration point noted in [`crate::transport`]).

/// `DAV:` core namespace.
pub const NS_DAV: &str = "DAV:";
/// CardDAV namespace (RFC 6352).
pub const NS_CARDDAV: &str = "urn:ietf:params:xml:ns:carddav";
/// CalendarServer extension namespace — carries `getctag` (the fallback sync key).
pub const NS_CALSRV: &str = "http://calendarserver.org/ns/";

const DECL: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n";

/// Escape the five XML metacharacters for safe interpolation into element text.
pub fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// `PROPFIND` body requesting `current-user-principal` (discovery step 1, §2.3).
pub fn propfind_current_user_principal() -> String {
    format!(
        "{DECL}<d:propfind xmlns:d=\"{NS_DAV}\">\
<d:prop><d:current-user-principal/></d:prop></d:propfind>"
    )
}

/// `PROPFIND` body requesting the CardDAV `addressbook-home-set` (discovery
/// step 2, §2.3).
pub fn propfind_addressbook_home_set() -> String {
    format!(
        "{DECL}<d:propfind xmlns:d=\"{NS_DAV}\" xmlns:c=\"{NS_CARDDAV}\">\
<d:prop><c:addressbook-home-set/></d:prop></d:propfind>"
    )
}

/// `PROPFIND` (Depth: 1) body enumerating a home-set's collections with the
/// properties the engine feature-detects on: `displayname`, `resourcetype`
/// (to keep only `addressbook` collections), `getctag`, `sync-token`
/// (discovery step 3, §2.3).
pub fn propfind_collections() -> String {
    format!(
        "{DECL}<d:propfind xmlns:d=\"{NS_DAV}\" xmlns:c=\"{NS_CARDDAV}\" \
xmlns:cs=\"{NS_CALSRV}\">\
<d:prop>\
<d:resourcetype/>\
<d:displayname/>\
<d:sync-token/>\
<cs:getctag/>\
</d:prop></d:propfind>"
    )
}

/// `PROPFIND` (Depth: 1) body listing member `getetag`s plus the collection
/// `getctag` — the ctag + etag-diff fallback pull when `sync-collection` is
/// unadvertised (§2.3).
pub fn propfind_etag_list() -> String {
    format!(
        "{DECL}<d:propfind xmlns:d=\"{NS_DAV}\" xmlns:cs=\"{NS_CALSRV}\">\
<d:prop><d:getetag/><cs:getctag/></d:prop></d:propfind>"
    )
}

/// `sync-collection` REPORT body (RFC 6578) from `sync_token` (an empty/`None`
/// token requests the initial full enumeration, §2.3).
pub fn report_sync_collection(sync_token: Option<&str>) -> String {
    let token = sync_token.map(xml_escape).unwrap_or_default();
    format!(
        "{DECL}<d:sync-collection xmlns:d=\"{NS_DAV}\">\
<d:sync-token>{token}</d:sync-token>\
<d:sync-level>1</d:sync-level>\
<d:prop><d:getetag/></d:prop></d:sync-collection>"
    )
}

/// `addressbook-query` REPORT body (RFC 6352) — the full enumeration / fallback
/// pull, requesting `getetag` + `address-data` for every card in a collection
/// (§2.3). Returns hrefs + etags + vCard bodies.
pub fn addressbook_query() -> String {
    format!(
        "{DECL}<c:addressbook-query xmlns:d=\"{NS_DAV}\" xmlns:c=\"{NS_CARDDAV}\">\
<d:prop><d:getetag/><c:address-data/></d:prop>\
<c:filter/></c:addressbook-query>"
    )
}

/// `addressbook-multiget` REPORT body (RFC 6352) over a set of hrefs — pulls the
/// `getetag` + `address-data` vCard bodies for changed cards (§2.3).
pub fn addressbook_multiget(hrefs: &[String]) -> String {
    let mut body = format!(
        "{DECL}<c:addressbook-multiget xmlns:d=\"{NS_DAV}\" xmlns:c=\"{NS_CARDDAV}\">\
<d:prop><d:getetag/><c:address-data/></d:prop>"
    );
    for href in hrefs {
        body.push_str(&format!("<d:href>{}</d:href>", xml_escape(href)));
    }
    body.push_str("</c:addressbook-multiget>");
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_metacharacters() {
        assert_eq!(xml_escape("a&b<c>\"'"), "a&amp;b&lt;c&gt;&quot;&apos;");
    }

    #[test]
    fn sync_collection_embeds_token() {
        let b = report_sync_collection(Some("http://radicale.org/ns/sync/42"));
        assert!(b.contains("<d:sync-token>http://radicale.org/ns/sync/42</d:sync-token>"));
        assert!(b.contains("<d:sync-level>1</d:sync-level>"));
    }

    #[test]
    fn sync_collection_initial_is_empty_token() {
        let b = report_sync_collection(None);
        assert!(b.contains("<d:sync-token></d:sync-token>"));
    }

    #[test]
    fn multiget_lists_hrefs_and_address_data() {
        let b = addressbook_multiget(&["/a/1.vcf".to_string(), "/a/2.vcf".to_string()]);
        assert!(b.contains("<c:addressbook-multiget"));
        assert!(b.contains("xmlns:c=\"urn:ietf:params:xml:ns:carddav\""));
        assert!(b.contains("<c:address-data/>"));
        assert!(b.contains("<d:href>/a/1.vcf</d:href>"));
        assert!(b.contains("<d:href>/a/2.vcf</d:href>"));
    }

    #[test]
    fn query_requests_address_data() {
        let b = addressbook_query();
        assert!(b.contains("<c:addressbook-query"));
        assert!(b.contains("<c:address-data/>"));
        assert!(b.contains("<c:filter/>"));
    }

    #[test]
    fn home_set_is_carddav_namespaced() {
        let b = propfind_addressbook_home_set();
        assert!(b.contains("<c:addressbook-home-set/>"));
        assert!(b.contains("xmlns:c=\"urn:ietf:params:xml:ns:carddav\""));
    }

    #[test]
    fn collections_propfind_requests_ctag_and_sync_token() {
        let b = propfind_collections();
        assert!(b.contains("<cs:getctag/>"));
        assert!(b.contains("<d:sync-token/>"));
        assert!(b.contains("<d:resourcetype/>"));
    }
}
