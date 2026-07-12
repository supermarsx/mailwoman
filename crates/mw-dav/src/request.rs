//! Pure XML request-body builders for the DAV/HTTP core (plan §1.3, §2.3).
//!
//! Every function here is a pure `&…` → `String` transform with no I/O, so the
//! wire bodies are unit-tested directly (no live server). `mw-carddav` reuses
//! the generic builders ([`multiget`], [`propfind_home_set`]) by passing
//! [`DavKind::CardDav`].

/// Which DAV flavour a shared-core call is for — selects the `.well-known`
/// path, the home-set element, the multiget report + data element, and the
/// collection resourcetype the discovery filter keeps (plan §1.3, so
/// `mw-carddav` drives the same core with `CardDav`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DavKind {
    CalDav,
    CardDav,
}

impl DavKind {
    /// The `.well-known` discovery path (RFC 6764).
    pub fn well_known(self) -> &'static str {
        match self {
            DavKind::CalDav => ".well-known/caldav",
            DavKind::CardDav => ".well-known/carddav",
        }
    }

    /// The home-set property local-name enumerated during discovery.
    pub fn home_set_prop(self) -> &'static str {
        match self {
            DavKind::CalDav => "calendar-home-set",
            DavKind::CardDav => "addressbook-home-set",
        }
    }

    /// The collection resourcetype the discovery filter retains.
    pub fn resource_type(self) -> &'static str {
        match self {
            DavKind::CalDav => "calendar",
            DavKind::CardDav => "addressbook",
        }
    }

    fn ns_uri(self) -> &'static str {
        match self {
            DavKind::CalDav => NS_CALDAV,
            DavKind::CardDav => NS_CARDDAV,
        }
    }

    fn multiget_report(self) -> &'static str {
        match self {
            DavKind::CalDav => "calendar-multiget",
            DavKind::CardDav => "addressbook-multiget",
        }
    }

    /// The `*-data` element carrying the resource body in a multiget response —
    /// the generic parser keys on any element whose local name ends `-data`
    /// (see [`crate::response`]).
    pub fn data_elem(self) -> &'static str {
        match self {
            DavKind::CalDav => "calendar-data",
            DavKind::CardDav => "address-data",
        }
    }
}

pub const NS_DAV: &str = "DAV:";
pub const NS_CALDAV: &str = "urn:ietf:params:xml:ns:caldav";
pub const NS_CARDDAV: &str = "urn:ietf:params:xml:ns:carddav";
/// CalendarServer extension namespace — carries `getctag` (the fallback sync key).
pub const NS_CALSRV: &str = "http://calendarserver.org/ns/";
/// Apple iCal extension namespace — carries `calendar-color`.
pub const NS_APPLE: &str = "http://apple.com/ns/ical/";

const DECL: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n";

/// Escape the five XML metacharacters for safe interpolation into element text
/// / attribute values (tokens, hrefs, display names may contain them).
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

/// `PROPFIND` body requesting the home-set for `kind` (discovery step 2, §2.3).
pub fn propfind_home_set(kind: DavKind) -> String {
    let ns = kind.ns_uri();
    let prop = kind.home_set_prop();
    format!(
        "{DECL}<d:propfind xmlns:d=\"{NS_DAV}\" xmlns:x=\"{ns}\">\
<d:prop><x:{prop}/></d:prop></d:propfind>"
    )
}

/// `PROPFIND` (Depth: 1) body enumerating a home-set's collections with the
/// properties the engine feature-detects on: `displayname`, `resourcetype`,
/// `calendar-color`, `supported-calendar-component-set`, `getctag`, `sync-token`
/// (discovery step 3, §2.3).
pub fn propfind_collections() -> String {
    format!(
        "{DECL}<d:propfind xmlns:d=\"{NS_DAV}\" xmlns:c=\"{NS_CALDAV}\" \
xmlns:cs=\"{NS_CALSRV}\" xmlns:ic=\"{NS_APPLE}\">\
<d:prop>\
<d:resourcetype/>\
<d:displayname/>\
<d:sync-token/>\
<cs:getctag/>\
<ic:calendar-color/>\
<c:supported-calendar-component-set/>\
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

/// Generic multiget REPORT body for `kind` over a set of hrefs (RFC 4791
/// `calendar-multiget` / RFC 6352 `addressbook-multiget`). `mw-carddav` calls
/// this with [`DavKind::CardDav`] (§2.3).
pub fn multiget(kind: DavKind, hrefs: &[String]) -> String {
    let ns = kind.ns_uri();
    let report = kind.multiget_report();
    let data = kind.data_elem();
    let mut body = format!(
        "{DECL}<c:{report} xmlns:d=\"{NS_DAV}\" xmlns:c=\"{ns}\">\
<d:prop><d:getetag/><c:{data}/></d:prop>"
    );
    for href in hrefs {
        body.push_str(&format!("<d:href>{}</d:href>", xml_escape(href)));
    }
    body.push_str(&format!("</c:{report}>"));
    body
}

/// `calendar-query` REPORT body (RFC 4791) selecting one component type
/// (`VEVENT` / `VTODO`) over an optional UTC time-range window — the initial
/// full pull / fallback enumeration (§2.3). Returns hrefs + etags.
pub fn calendar_query(component: &str, window: Option<(&str, &str)>) -> String {
    let comp = xml_escape(component);
    let range = match window {
        Some((start, end)) => format!(
            "<c:time-range start=\"{}\" end=\"{}\"/>",
            xml_escape(start),
            xml_escape(end)
        ),
        None => String::new(),
    };
    format!(
        "{DECL}<c:calendar-query xmlns:d=\"{NS_DAV}\" xmlns:c=\"{NS_CALDAV}\">\
<d:prop><d:getetag/></d:prop>\
<c:filter><c:comp-filter name=\"VCALENDAR\">\
<c:comp-filter name=\"{comp}\">{range}</c:comp-filter>\
</c:comp-filter></c:filter></c:calendar-query>"
    )
}

/// `free-busy-query` REPORT body (RFC 4791) over a UTC window — feeds
/// `Calendar/freeBusy` (§2.2). The response is a `text/calendar` VFREEBUSY,
/// parsed by [`crate::response::parse_free_busy`].
pub fn free_busy_query(window_start: &str, window_end: &str) -> String {
    format!(
        "{DECL}<c:free-busy-query xmlns:c=\"{NS_CALDAV}\">\
<c:time-range start=\"{}\" end=\"{}\"/></c:free-busy-query>",
        xml_escape(window_start),
        xml_escape(window_end)
    )
}

/// `MKCALENDAR` body creating a calendar collection with a display name (§2.3).
pub fn mkcalendar(display_name: &str) -> String {
    format!(
        "{DECL}<c:mkcalendar xmlns:d=\"{NS_DAV}\" xmlns:c=\"{NS_CALDAV}\">\
<d:set><d:prop><d:displayname>{}</d:displayname></d:prop></d:set>\
</c:mkcalendar>",
        xml_escape(display_name)
    )
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
        let b = report_sync_collection(Some("http://sabre.io/ns/sync/42"));
        assert!(b.contains("<d:sync-token>http://sabre.io/ns/sync/42</d:sync-token>"));
        assert!(b.contains("<d:sync-level>1</d:sync-level>"));
    }

    #[test]
    fn sync_collection_initial_is_empty_token() {
        let b = report_sync_collection(None);
        assert!(b.contains("<d:sync-token></d:sync-token>"));
    }

    #[test]
    fn multiget_caldav_lists_hrefs_and_calendar_data() {
        let b = multiget(
            DavKind::CalDav,
            &["/c/1.ics".to_string(), "/c/2.ics".to_string()],
        );
        assert!(b.contains("<c:calendar-multiget"));
        assert!(b.contains("<c:calendar-data/>"));
        assert!(b.contains("<d:href>/c/1.ics</d:href>"));
        assert!(b.contains("<d:href>/c/2.ics</d:href>"));
    }

    #[test]
    fn multiget_carddav_uses_address_data() {
        let b = multiget(DavKind::CardDav, &["/a/1.vcf".to_string()]);
        assert!(b.contains("<c:addressbook-multiget"));
        assert!(b.contains("xmlns:c=\"urn:ietf:params:xml:ns:carddav\""));
        assert!(b.contains("<c:address-data/>"));
    }

    #[test]
    fn home_set_prop_switches_by_kind() {
        assert!(propfind_home_set(DavKind::CalDav).contains("<x:calendar-home-set/>"));
        assert!(propfind_home_set(DavKind::CardDav).contains("<x:addressbook-home-set/>"));
    }

    #[test]
    fn calendar_query_includes_time_range_and_component() {
        let b = calendar_query("VTODO", Some(("20260101T000000Z", "20260201T000000Z")));
        assert!(b.contains("name=\"VTODO\""));
        assert!(b.contains("start=\"20260101T000000Z\""));
    }
}
