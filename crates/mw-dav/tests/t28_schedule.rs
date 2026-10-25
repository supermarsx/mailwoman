//! 26.20 t28-e19 (f34, t29 A3) — free/busy for other people through the caller's
//! CalDAV server (RFC 6638 §5): discovery of the scheduling outbox, the request
//! body, and the per-recipient reply, against a stub server that records what it
//! was sent.

use std::sync::{Arc, Mutex};

use mw_dav::{DavClient, DavConfig, request, response};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const PRINCIPAL: &str = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:"><d:response><d:href>/.well-known/caldav</d:href>
<d:propstat><d:prop><d:current-user-principal><d:href>/principals/me/</d:href>
</d:current-user-principal></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat>
</d:response></d:multistatus>"#;

const OUTBOX: &str = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav"><d:response>
<d:href>/principals/me/</d:href><d:propstat><d:prop><c:schedule-outbox-URL>
<d:href>/calendars/me/outbox/</d:href></c:schedule-outbox-URL></d:prop>
<d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response></d:multistatus>"#;

const NO_OUTBOX: &str = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:"><d:response><d:href>/principals/me/</d:href>
<d:propstat><d:prop/><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response>
</d:multistatus>"#;

const SCHEDULE_RESPONSE: &str = "<?xml version=\"1.0\"?>\
<C:schedule-response xmlns:D=\"DAV:\" xmlns:C=\"urn:ietf:params:xml:ns:caldav\">\
<C:response><C:recipient><D:href>mailto:Bob@Example.org</D:href></C:recipient>\
<C:request-status>2.0;Success</C:request-status>\
<C:calendar-data>BEGIN:VCALENDAR\r\nMETHOD:REPLY\r\nBEGIN:VFREEBUSY\r\n\
FREEBUSY;FBTYPE=BUSY:20260701T090000Z/20260701T100000Z\r\n\
END:VFREEBUSY\r\nEND:VCALENDAR\r\n</C:calendar-data></C:response>\
<C:response><C:recipient><D:href>mailto:nobody@example.org</D:href></C:recipient>\
<C:request-status>3.7;Invalid calendar user</C:request-status></C:response>\
</C:schedule-response>";

/// One recorded request: method, path, content type, body.
type Seen = Arc<Mutex<Vec<(String, String, String, String)>>>;

/// A stub DAV server answering the three scheduling steps. With `outbox` false
/// the principal has no `schedule-outbox-URL`.
async fn stub(outbox: bool) -> (String, Seen) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let seen: Seen = Arc::default();
    let log = seen.clone();
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let log = log.clone();
            tokio::spawn(async move {
                let mut buf = Vec::new();
                loop {
                    // Read one request (headers + Content-Length body).
                    let mut tmp = [0u8; 4096];
                    let head_end = loop {
                        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                            break i + 4;
                        }
                        let n = match sock.read(&mut tmp).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => n,
                        };
                        buf.extend_from_slice(&tmp[..n]);
                    };
                    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
                    let len = head
                        .lines()
                        .find_map(|l| {
                            l.to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .map(|v| v.trim().parse::<usize>().unwrap_or(0))
                        })
                        .unwrap_or(0);
                    while buf.len() < head_end + len {
                        let n = match sock.read(&mut tmp).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => n,
                        };
                        buf.extend_from_slice(&tmp[..n]);
                    }
                    let body = String::from_utf8_lossy(&buf[head_end..head_end + len]).to_string();
                    buf.drain(..head_end + len);
                    let mut first = head.lines().next().unwrap_or("").split_whitespace();
                    let method = first.next().unwrap_or("").to_string();
                    let path = first.next().unwrap_or("").to_string();
                    let ctype = head
                        .lines()
                        .find_map(|l| {
                            let lower = l.to_ascii_lowercase();
                            lower
                                .strip_prefix("content-type:")
                                .map(|v| v.trim().to_string())
                        })
                        .unwrap_or_default();
                    log.lock()
                        .unwrap()
                        .push((method.clone(), path.clone(), ctype, body.clone()));
                    let (status, reply) = match (method.as_str(), path.as_str()) {
                        ("PROPFIND", "/.well-known/caldav") => ("207 Multi-Status", PRINCIPAL),
                        ("PROPFIND", "/principals/me/") if outbox => ("207 Multi-Status", OUTBOX),
                        ("PROPFIND", "/principals/me/") => ("207 Multi-Status", NO_OUTBOX),
                        ("POST", "/calendars/me/outbox/") => ("200 OK", SCHEDULE_RESPONSE),
                        _ => ("404 Not Found", ""),
                    };
                    let out = format!(
                        "HTTP/1.1 {status}\r\nContent-Type: application/xml\r\nContent-Length: {}\r\n\r\n{reply}",
                        reply.len()
                    );
                    if sock.write_all(out.as_bytes()).await.is_err() {
                        return;
                    }
                }
            });
        }
    });
    (base, seen)
}

fn client(base: &str) -> DavClient {
    DavClient::new(DavConfig {
        base_url: base.to_string(),
        username: "me@example.org".into(),
        password: "pw".into(),
        pin: None,
    })
    .unwrap()
}

#[tokio::test]
async fn free_busy_for_attendees_goes_through_the_scheduling_outbox() {
    let (base, seen) = stub(true).await;
    let replies = client(&base)
        .schedule_free_busy(
            "me@example.org",
            &["bob@example.org".into(), "nobody@example.org".into()],
            "2026-07-01T00:00:00Z",
            "2026-07-02T00:00:00Z",
        )
        .await
        .expect("the server does scheduling");

    let bob = replies
        .iter()
        .find(|r| r.recipient == "bob@example.org")
        .expect("a reply for bob, lower-cased");
    let busy = bob.busy.as_ref().expect("bob's busy blocks");
    assert_eq!(busy.len(), 1);
    assert_eq!(busy[0].start_utc, "2026-07-01T09:00:00Z");
    assert_eq!(busy[0].end_utc, "2026-07-01T10:00:00Z");
    let nobody = replies
        .iter()
        .find(|r| r.recipient == "nobody@example.org")
        .expect("a reply for the unknown address");
    assert_eq!(nobody.busy, None, "a 3.x status is not free time");

    let seen = seen.lock().unwrap().clone();
    let post = seen
        .iter()
        .find(|(m, ..)| m == "POST")
        .expect("one POST to the outbox");
    assert_eq!(post.1, "/calendars/me/outbox/");
    assert!(post.2.starts_with("text/calendar"), "{}", post.2);
    assert!(post.3.contains("METHOD:REQUEST\r\n"));
    assert!(post.3.contains("BEGIN:VFREEBUSY\r\n"));
    assert!(post.3.contains("DTSTART:20260701T000000Z\r\n"));
    assert!(post.3.contains("DTEND:20260702T000000Z\r\n"));
    assert!(post.3.contains("ORGANIZER:mailto:me@example.org\r\n"));
    assert!(post.3.contains("ATTENDEE:mailto:bob@example.org\r\n"));
    assert!(post.3.contains("ATTENDEE:mailto:nobody@example.org\r\n"));
}

#[tokio::test]
async fn a_server_without_an_outbox_is_a_discovery_error_and_nothing_is_posted() {
    let (base, seen) = stub(false).await;
    let err = client(&base)
        .schedule_free_busy(
            "me@example.org",
            &["bob@example.org".into()],
            "2026-07-01T00:00:00Z",
            "2026-07-02T00:00:00Z",
        )
        .await
        .unwrap_err();
    assert!(matches!(err, mw_dav::DavError::Discovery(_)), "{err:?}");
    assert!(!seen.lock().unwrap().iter().any(|(m, ..)| m == "POST"));
}

#[test]
fn an_address_cannot_add_a_line_to_the_request() {
    let body = request::free_busy_request(
        "me@example.org",
        &["bob@example.org\r\nATTENDEE:mailto:eve@example.org".into()],
        "20260701T000000Z",
        "20260702T000000Z",
        "u1",
        "20260701T000000Z",
    );
    assert_eq!(
        body.lines().filter(|l| l.starts_with("ATTENDEE:")).count(),
        1,
        "{body}"
    );
}

#[test]
fn schedule_response_parses_per_recipient() {
    let replies = response::parse_schedule_response(SCHEDULE_RESPONSE).unwrap();
    assert_eq!(replies.len(), 2);
    assert!(replies[0].ok() && !replies[1].ok());
    assert_eq!(
        response::parse_schedule_outbox(OUTBOX).unwrap().as_deref(),
        Some("/calendars/me/outbox/")
    );
    assert_eq!(response::parse_schedule_outbox(NO_OUTBOX).unwrap(), None);
}

/// With a pin, the client connects to the checked address even though the base
/// URL's host does not resolve, and refuses a URL on another host.
#[tokio::test]
async fn a_pinned_client_connects_to_the_checked_address_only() {
    let (base, seen) = stub(true).await;
    let addr: std::net::SocketAddr = base.trim_start_matches("http://").parse().unwrap();
    let pinned = DavClient::new(DavConfig {
        base_url: format!("http://dav.invalid:{}", addr.port()),
        username: "me@example.org".into(),
        password: "pw".into(),
        pin: Some(addr),
    })
    .unwrap();
    let replies = pinned
        .schedule_free_busy(
            "me@example.org",
            &["bob@example.org".into()],
            "2026-07-01T00:00:00Z",
            "2026-07-02T00:00:00Z",
        )
        .await
        .expect("the pinned address answers for dav.invalid");
    assert!(replies.iter().any(|r| r.recipient == "bob@example.org"));
    let before = seen.lock().unwrap().len();

    // Another host is refused before anything is sent.
    let err = pinned
        .sync_collection("http://elsewhere.invalid/cal/", None)
        .await
        .unwrap_err();
    assert!(
        matches!(err, mw_dav::DavError::Transport(ref m) if m.contains("another host")),
        "{err:?}"
    );
    assert_eq!(seen.lock().unwrap().len(), before, "nothing was sent");
    assert!(mw_dav::url_allowed(
        &DavConfig::default(),
        "http://anywhere.invalid/"
    ));
}
