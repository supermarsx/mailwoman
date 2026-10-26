use mw_dav::{DavConfig, DavError, DavTransport, TransportRequest, TransportResponse};
use std::sync::{Arc, Mutex};

struct Rejected(Mutex<Vec<TransportRequest>>);
impl DavTransport for Rejected {
    fn send(&self, request: TransportRequest) -> mw_dav::transport::TransportFuture<'_> {
        Box::pin(async move {
            self.0.lock().unwrap().push(request);
            Ok(TransportResponse {
                status: 401,
                etag: None,
                body: String::new(),
            })
        })
    }
}

#[tokio::test]
async fn rejected_credentials_do_not_trigger_fallback_queries() {
    let transport = Arc::new(Rejected(Mutex::new(vec![])));
    let client = mw_carddav::CardDavClient::with_transport(
        DavConfig {
            base_url: "https://must-not-be-contacted.invalid/".into(),
            ..Default::default()
        },
        transport.clone(),
    );
    assert!(matches!(
        client.sync_addressbook("/books/mine/", None).await,
        Err(DavError::Status { status: 401, .. })
    ));
    let requests = transport.0.lock().unwrap();
    assert_eq!(
        requests.len(),
        1,
        "no PROPFIND retry after authentication rejection"
    );
    assert_eq!(requests[0].method, "REPORT");
}
