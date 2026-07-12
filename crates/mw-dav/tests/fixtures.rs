//! Unit tests for the shared DAV core over **recorded** Radicale / Google-quirk
//! XML fixtures (`fixtures/dav/`) — no live server (plan §3 e2 acceptance):
//! discovery, sync-token incremental, ctag+etag fallback, multiget parse,
//! free-busy parse, and CardDAV reuse via `DavKind::CardDav`. The live path is
//! covered by the env-gated `#[ignore]` test at the bottom.

use mw_dav::request::DavKind;
use mw_dav::response;

/// Load a recorded fixture from the repo-root `fixtures/dav/` directory.
fn fixture(name: &str) -> String {
    let path = format!("{}/../../fixtures/dav/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

#[test]
fn discovery_extracts_principal_and_home_set() {
    let principal =
        response::parse_current_user_principal(&fixture("discovery_principal.xml")).unwrap();
    assert_eq!(
        principal.as_deref(),
        Some("/dav/principals/user@example.com/")
    );

    let home =
        response::parse_home_set(&fixture("discovery_homeset.xml"), DavKind::CalDav).unwrap();
    assert_eq!(home.as_deref(), Some("/dav/calendars/user@example.com/"));
}

#[test]
fn discovery_enumerates_calendars_only_with_props() {
    let cols =
        response::parse_collections(&fixture("collections_radicale.xml"), DavKind::CalDav).unwrap();
    // The non-calendar "Home" collection is filtered out; two calendars remain.
    assert_eq!(cols.len(), 2);

    let personal = cols.iter().find(|c| c.display_name == "Personal").unwrap();
    assert_eq!(personal.href, "/dav/calendars/user@example.com/personal/");
    assert_eq!(personal.color.as_deref(), Some("#FF5733FF"));
    assert_eq!(
        personal.ctag.as_deref(),
        Some("http://radicale.org/ns/sync/abc123")
    );
    assert_eq!(
        personal.sync_token.as_deref(),
        Some("http://radicale.org/ns/sync/abc123")
    );
    assert_eq!(personal.components, vec!["VEVENT".to_string()]);

    let tasks = cols.iter().find(|c| c.display_name == "Tasks").unwrap();
    assert_eq!(tasks.components, vec!["VTODO".to_string()]);
}

#[test]
fn sync_collection_radicale_splits_changed_and_removed() {
    let delta = response::parse_sync_delta(&fixture("sync_collection_radicale.xml")).unwrap();
    assert_eq!(
        delta.new_sync_token.as_deref(),
        Some("http://radicale.org/ns/sync/xyz789")
    );
    assert_eq!(delta.changed.len(), 2);
    assert_eq!(delta.removed.len(), 1);
    assert!(delta.removed[0].ends_with("event-deleted.ics"));
    assert_eq!(delta.changed[0].etag.as_deref(), Some("\"1a2b3c\""));
    // sync-collection returns etags only; bodies are fetched later via multiget.
    assert!(delta.changed[0].body.is_none());
}

#[test]
fn sync_collection_google_quirks_parse() {
    // Google: uppercase `D:` prefix, absolute hrefs, weak `W/"…"` etags.
    let delta = response::parse_sync_delta(&fixture("sync_collection_google.xml")).unwrap();
    assert_eq!(
        delta.new_sync_token.as_deref(),
        Some("\"google-sync-98765\"")
    );
    assert_eq!(delta.changed.len(), 1);
    assert_eq!(delta.removed.len(), 1);
    assert!(
        delta.changed[0]
            .href
            .starts_with("https://apidata.googleusercontent.com/")
    );
    assert_eq!(delta.changed[0].etag.as_deref(), Some("W/\"xyz-123\""));
}

#[test]
fn ctag_etag_fallback_list_parses() {
    let (ctag, members) = response::parse_etag_list(&fixture("etag_list.xml")).unwrap();
    assert_eq!(ctag.as_deref(), Some("http://radicale.org/ns/sync/abc123"));
    assert_eq!(members.len(), 2);
    assert!(
        members
            .iter()
            .any(|(h, e)| h.ends_with("event-1.ics") && e == "\"1a2b3c\"")
    );
    assert!(
        members
            .iter()
            .any(|(h, e)| h.ends_with("event-2.ics") && e == "\"4d5e6f\"")
    );
}

#[test]
fn calendar_multiget_yields_bodies_and_unescapes() {
    let res = response::parse_multiget(&fixture("multiget_radicale.xml"), DavKind::CalDav).unwrap();
    assert_eq!(res.len(), 2);

    let e1 = res
        .iter()
        .find(|r| r.href.ends_with("event-1.ics"))
        .unwrap();
    assert_eq!(e1.etag.as_deref(), Some("\"1a2b3c\""));
    let body1 = e1.body.as_deref().unwrap();
    assert!(body1.contains("BEGIN:VEVENT"));
    assert!(body1.contains("SUMMARY:Standup"));

    let e2 = res
        .iter()
        .find(|r| r.href.ends_with("event-2.ics"))
        .unwrap();
    // `&amp;` in the XML must be unescaped back to a literal `&` in the ICS body.
    assert!(
        e2.body
            .as_deref()
            .unwrap()
            .contains("SUMMARY:Review & Retro")
    );
}

#[test]
fn addressbook_multiget_reuses_the_shared_core() {
    // The same parser drives CardDAV (e3) via `DavKind::CardDav` + `address-data`.
    let res = response::parse_multiget(&fixture("multiget_carddav.xml"), DavKind::CardDav).unwrap();
    assert_eq!(res.len(), 1);
    let body = res[0].body.as_deref().unwrap();
    assert!(body.contains("BEGIN:VCARD"));
    assert!(body.contains("FN:Ada Lovelace"));
    assert_eq!(res[0].etag.as_deref(), Some("\"c1\""));
}

#[test]
fn free_busy_query_reply_parses_intervals() {
    let fb = response::parse_free_busy(&fixture("freebusy.ics")).unwrap();
    // Two FREEBUSY lines, the second carrying two comma-separated periods.
    assert_eq!(fb.len(), 3);
    assert_eq!(fb[0].start_utc, "2026-01-15T09:00:00Z");
    assert_eq!(fb[0].end_utc, "2026-01-15T10:00:00Z");
    assert!(fb.iter().all(|i| i.status == "busy"));
    assert_eq!(fb[2].start_utc, "2026-01-15T16:00:00Z");
}

/// Live smoke test against a real Radicale (RFC 6764 discovery → sync). Gated on
/// `RADICALE_URL` (+ optional `RADICALE_USER` / `RADICALE_PASS`) and `#[ignore]`
/// so CI unit runs never touch the network (plan §3 e2: "a live test env-gated,
/// #[ignore]").
#[tokio::test]
#[ignore = "requires a live Radicale at RADICALE_URL"]
async fn live_discover_and_sync() {
    let Ok(base_url) = std::env::var("RADICALE_URL") else {
        return;
    };
    let config = mw_dav::DavConfig {
        base_url,
        username: std::env::var("RADICALE_USER").unwrap_or_else(|_| "test".into()),
        password: std::env::var("RADICALE_PASS").unwrap_or_else(|_| "test".into()),
    };
    let client = mw_dav::DavClient::new(config).unwrap();
    let calendars = client.discover_calendars().await.expect("discovery");
    assert!(!calendars.is_empty(), "expected at least one calendar");
    let cal = &calendars[0];
    let delta = client
        .sync_collection(&cal.href, cal.sync_token.as_deref())
        .await
        .expect("initial sync");
    // Initial sync returns the collection contents (possibly empty) + a token.
    assert!(delta.new_sync_token.is_some() || delta.changed.is_empty());
}
