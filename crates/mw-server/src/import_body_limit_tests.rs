//! t55: an import refused before its body is read (over the 64 MiB body limit,
//! or import capacity busy) must reach the client as the 413 / 429 it is, not
//! as a connection reset in the middle of the upload. Served through
//! [`crate::inbound_http::serve`], the production HTTP stack.
use super::*;
use axum::routing::post;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

const OVERSIZED: usize = 67 * 1024 * 1024;
/// Well over the limit: the body extractor alone stops reading at 64 MiB, so
/// this is the size whose upload used to end in a reset.
const FAR_OVERSIZED: usize = 200 * 1024 * 1024;

/// The mbox route's shape: the same body limit, rejection mapping and
/// admission middleware, without the session/engine parts that run after the
/// body is read.
fn import_like_router(capacity: Arc<tokio::sync::Semaphore>) -> axum::Router {
    axum::Router::new()
        .route(
            "/api/import/mbox",
            post(|body: Result<Bytes, BytesRejection>| async move {
                match body {
                    Ok(_) => StatusCode::OK.into_response(),
                    Err(rejection) => body_rejected(rejection),
                }
            })
            .layer(DefaultBodyLimit::max(MAX_MESSAGE_BYTES)),
        )
        .layer(axum::middleware::from_fn(move |request, next| {
            let capacity = capacity.clone();
            async move { import_admission(&capacity, request, next).await }
        }))
}

async fn serve(capacity: Arc<tokio::sync::Semaphore>) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(crate::inbound_http::serve(
        listener,
        import_like_router(capacity),
        crate::inbound_http::HttpLimits::default(),
        std::future::pending(),
    ));
    address
}

/// POST `OVERSIZED` bytes with a plain client that does not send
/// `Expect: 100-continue` (as browsers and most HTTP libraries do not).
async fn post_oversized(address: SocketAddr) -> std::result::Result<u16, String> {
    let response = reqwest::Client::new()
        .post(format!("http://{address}/api/import/mbox"))
        .header("content-type", "application/mbox")
        .body(vec![b'x'; OVERSIZED])
        .send()
        .await
        .map_err(|e| format!("{e:?}"))?;
    Ok(response.status().as_u16())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_oversized_upload_receives_the_413() {
    let address = serve(Arc::new(tokio::sync::Semaphore::new(2))).await;
    for attempt in 0..3 {
        assert_eq!(post_oversized(address).await, Ok(413), "attempt {attempt}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_busy_import_capacity_upload_receives_the_429() {
    let capacity = Arc::new(tokio::sync::Semaphore::new(0));
    let address = serve(capacity).await;
    let response = reqwest::Client::new()
        .post(format!("http://{address}/api/import/mbox"))
        .body(vec![b'x'; 16 * 1024 * 1024])
        .send()
        .await
        .map_err(|e| format!("{e:?}"));
    assert_eq!(response.map(|r| r.status().as_u16()), Ok(429));
}

/// A client that sends `Expect: 100-continue` gets the 413 instead of a
/// `100 Continue`, before it sends any of the body.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expect_continue_gets_the_413_before_any_body_is_sent() {
    let address = serve(Arc::new(tokio::sync::Semaphore::new(2))).await;
    let mut connection = tokio::net::TcpStream::connect(address).await.unwrap();
    connection
        .write_all(
            format!(
                "POST /api/import/mbox HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/mbox\r\n\
                 Content-Length: {OVERSIZED}\r\nExpect: 100-continue\r\n\r\n"
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    let mut reader = tokio::io::BufReader::new(connection);
    let mut status = String::new();
    tokio::time::timeout(std::time::Duration::from_secs(5), reader.read_line(&mut status))
        .await
        .expect("no answer without the body")
        .unwrap();
    assert!(status.starts_with("HTTP/1.1 413"), "{status:?}");
    let mut rest = Vec::new();
    let _ = tokio::time::timeout(std::time::Duration::from_secs(2), reader.read_to_end(&mut rest)).await;
    let text = String::from_utf8_lossy(&rest);
    assert!(text.contains("maxMessageBytes"), "{text}");
}

/// A client that writes the whole body before reading (as a plain socket
/// writer or a streaming upload does) must still read the refusal.
async fn write_all_then_read(address: SocketAddr, body_len: usize) -> std::result::Result<String, String> {
    let mut connection = tokio::net::TcpStream::connect(address).await.unwrap();
    connection
        .write_all(
            format!("POST /api/import/mbox HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/mbox\r\nContent-Length: {body_len}\r\n\r\n")
                .as_bytes(),
        )
        .await
        .map_err(|e| format!("headers: {e}"))?;
    let chunk = vec![b'x'; 1024 * 1024];
    let mut sent = 0;
    while sent < body_len {
        let n = chunk.len().min(body_len - sent);
        connection.write_all(&chunk[..n]).await.map_err(|e| format!("body write after {sent} bytes: {e}"))?;
        sent += n;
    }
    let mut reader = tokio::io::BufReader::new(connection);
    let mut status = String::new();
    tokio::time::timeout(std::time::Duration::from_secs(10), reader.read_line(&mut status))
        .await
        .map_err(|_| "no status line".to_string())?
        .map_err(|e| format!("read: {e}"))?;
    Ok(status)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_client_that_writes_the_whole_oversized_body_first_reads_the_413() {
    let address = serve(Arc::new(tokio::sync::Semaphore::new(2))).await;
    for size in [OVERSIZED, FAR_OVERSIZED] {
        let status = write_all_then_read(address, size).await;
        assert!(status.as_deref().is_ok_and(|s| s.starts_with("HTTP/1.1 413")), "{size}: {status:?}");
    }
}

/// A body declared over the drain bound is answered at once, without being read.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_body_declared_over_the_drain_bound_is_answered_without_reading_it() {
    let address = serve(Arc::new(tokio::sync::Semaphore::new(2))).await;
    let mut connection = tokio::net::TcpStream::connect(address).await.unwrap();
    connection
        .write_all(b"POST /api/import/mbox HTTP/1.1\r\nHost: localhost\r\nContent-Length: 10000000000\r\n\r\n")
        .await
        .unwrap();
    let mut reader = tokio::io::BufReader::new(connection);
    let mut status = String::new();
    tokio::time::timeout(std::time::Duration::from_secs(5), reader.read_line(&mut status))
        .await
        .expect("answered without the body")
        .unwrap();
    assert!(status.starts_with("HTTP/1.1 413"), "{status:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_client_that_writes_the_whole_body_first_reads_the_429() {
    let address = serve(Arc::new(tokio::sync::Semaphore::new(0))).await;
    let status = write_all_then_read(address, 16 * 1024 * 1024).await;
    assert!(status.as_deref().is_ok_and(|s| s.starts_with("HTTP/1.1 429")), "{status:?}");
}
