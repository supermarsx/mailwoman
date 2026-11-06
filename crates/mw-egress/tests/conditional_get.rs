//! Conditional `GET` for the image proxy's revalidation: validators go out as
//! `If-None-Match` / `If-Modified-Since`, a `304` is `Refusal::Status(304)`, and the
//! caching headers of the answer are captured (bounded) beside the `ETag`.
use mw_egress::{
    Conditional, Hop, Outgoing, Refusal, ResponseMetadata, Target, send_hop,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn out<'a>(metadata: &'a ResponseMetadata, conditional: Option<Conditional<'a>>) -> Outgoing<'a> {
    Outgoing {
        response_metadata: Some(metadata),
        authorized_plugin: None,
        authorized_api: None,
        authorized_dav: None,
        web_push: None,
        response_limit: 1024,
        download: None,
        conditional,
        accept: "image/*",
        user_agent: "Mailwoman-Test",
        body: None,
    }
}

/// An origin that answers one request with `response` and hands back the request head.
async fn origin(response: &'static str) -> (Target, tokio::task::JoinHandle<String>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut head = Vec::new();
        while !head.ends_with(b"\r\n\r\n") {
            head.push(socket.read_u8().await.unwrap());
        }
        socket.write_all(response.as_bytes()).await.unwrap();
        socket.shutdown().await.ok();
        String::from_utf8(head).unwrap()
    });
    let target = Target {
        url: format!("http://{addr}/a.png").parse().unwrap(),
        host: addr.ip().to_string(),
        addr,
    };
    (target, task)
}

#[tokio::test]
async fn validators_are_sent_and_a_304_is_not_a_redirect_refusal() {
    let (target, request) = origin(
        "HTTP/1.1 304 Not Modified\r\nETag: \"v2\"\r\nLast-Modified: Wed, 21 Oct 2026 07:28:00 GMT\r\nCache-Control: max-age=120\r\nContent-Length: 0\r\n\r\n",
    )
    .await;
    let metadata = ResponseMetadata::default();
    let conditional = Conditional {
        etag: Some("\"v1\""),
        last_modified: Some("Tue, 20 Oct 2026 07:28:00 GMT"),
    };
    let result = send_hop(&target, &out(&metadata, Some(conditional))).await;
    assert!(matches!(result, Err(Refusal::Status(304))), "{result:?}");
    let head = request.await.unwrap().to_ascii_lowercase();
    assert!(head.contains("if-none-match: \"v1\""), "{head}");
    assert!(head.contains("if-modified-since: tue, 20 oct 2026 07:28:00 gmt"), "{head}");
    let cache = metadata.cache_headers().expect("cache headers captured");
    assert_eq!(cache.cache_control.as_deref(), Some("max-age=120"));
    assert_eq!(cache.last_modified.as_deref(), Some("Wed, 21 Oct 2026 07:28:00 GMT"));
    assert_eq!(metadata.snapshot().unwrap().etag.as_deref(), Some("\"v2\""));
}

#[tokio::test]
async fn an_unconditional_request_sends_no_validators_and_a_304_stays_a_refusal() {
    let (target, request) = origin(
        "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nETag: \"v3\"\r\nCache-Control: no-store\r\nContent-Length: 3\r\n\r\nabc",
    )
    .await;
    let metadata = ResponseMetadata::default();
    let body = send_hop(&target, &out(&metadata, None)).await.unwrap();
    assert!(matches!(body, Hop::Body(ref b) if b == b"abc"));
    let head = request.await.unwrap().to_ascii_lowercase();
    assert!(!head.contains("if-none-match") && !head.contains("if-modified-since"), "{head}");
    assert_eq!(metadata.cache_headers().unwrap().cache_control.as_deref(), Some("no-store"));

    // Without validators a 304 has no meaning to the caller: it is not reported as one.
    let (target, _request) = origin("HTTP/1.1 304 Not Modified\r\nContent-Length: 0\r\n\r\n").await;
    let metadata = ResponseMetadata::default();
    let result = send_hop(&target, &out(&metadata, None)).await;
    assert!(!matches!(result, Err(Refusal::Status(304))), "{result:?}");
}

#[tokio::test]
async fn validators_with_control_characters_are_not_sent() {
    let (target, request) = origin("HTTP/1.1 304 Not Modified\r\nContent-Length: 0\r\n\r\n").await;
    let metadata = ResponseMetadata::default();
    let conditional = Conditional { etag: Some("\"x\"\r\nCookie: stolen"), last_modified: None };
    let _ = send_hop(&target, &out(&metadata, Some(conditional))).await;
    let head = request.await.unwrap().to_ascii_lowercase();
    assert!(!head.contains("cookie"), "{head}");
    assert!(!head.contains("if-none-match"), "{head}");
}
