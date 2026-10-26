use mw_dav::{DavClient, DavConfig, DavError, DavTransport, TransportRequest, TransportResponse};
use std::sync::{Arc, Mutex};

struct Recording {
    status: u16,
    requests: Mutex<Vec<TransportRequest>>,
}
impl DavTransport for Recording {
    fn send(&self, request: TransportRequest) -> mw_dav::transport::TransportFuture<'_> {
        Box::pin(async move {
            self.requests.lock().unwrap().push(request);
            Ok(TransportResponse {
                status: self.status,
                etag: Some("\"new\"".into()),
                body: String::new(),
            })
        })
    }
}

#[tokio::test]
async fn injected_transport_preserves_conditional_writes_and_conflicts_without_network_fallback() {
    let transport = Arc::new(Recording {
        status: 412,
        requests: Mutex::new(vec![]),
    });
    let client = DavClient::with_transport(
        DavConfig {
            base_url: "https://must-not-be-contacted.invalid/".into(),
            ..Default::default()
        },
        transport.clone(),
    );
    assert!(matches!(
        client
            .put_resource("/calendar/item.ics", "calendar", Some("\"old\""))
            .await,
        Err(DavError::Conflict)
    ));
    assert!(matches!(
        client
            .put_resource("/calendar/new.ics", "calendar", None)
            .await,
        Err(DavError::Conflict)
    ));
    assert!(matches!(
        client
            .delete_resource("/calendar/item.ics", Some("\"old\""))
            .await,
        Err(DavError::Conflict)
    ));
    let requests = transport.requests.lock().unwrap();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].if_match.as_deref(), Some("\"old\""));
    assert_eq!(requests[1].if_none_match.as_deref(), Some("*"));
    assert_eq!(requests[2].method, "DELETE");
    assert_eq!(requests[2].if_match.as_deref(), Some("\"old\""));
}

#[test]
fn permission_failures_are_not_sync_tombstones() {
    for status in [401, 403, 412, 500, 503] {
        let xml = format!(
            "<d:multistatus xmlns:d='DAV:'><d:response><d:href>/cal/kept.ics</d:href><d:status>HTTP/1.1 {status} Refused</d:status></d:response></d:multistatus>"
        );
        assert!(
            matches!(mw_dav::response::parse_sync_delta(&xml),Err(DavError::Status {status:actual,..}) if actual==status)
        );
    }
    let xml = "<d:multistatus xmlns:d='DAV:'><d:response><d:href>/cal/gone.ics</d:href><d:status>HTTP/1.1 404 Not Found</d:status></d:response></d:multistatus>";
    assert_eq!(
        mw_dav::response::parse_sync_delta(xml).unwrap().removed,
        vec!["/cal/gone.ics"]
    );
}

#[test]
fn stored_resource_paths_resolve_against_the_owning_collection() {
    let base = "https://cloud.example/remote.php/dav/calendars/alice/work/";
    assert_eq!(
        mw_dav::resolve_resource_href(base, "/remote.php/dav/calendars/alice/work/event.ics")
            .unwrap(),
        format!("{base}event.ics")
    );
    assert_eq!(
        mw_dav::resolve_resource_href(base, "event.ics").unwrap(),
        format!("{base}event.ics")
    );
    assert!(mw_dav::resolve_resource_href("not a URL", "event.ics").is_err());
}
