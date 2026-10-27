//! Streamed body limits on `JmapClient` (moved out of `client.rs`'s unit tests).
//!
//! It lives here, not in `src/`, because mw-jmap must never build an HTTP client
//! itself (t24 B2, `mw_jmap_does_not_build_its_own_http_client`): the caller owns the
//! address policy and the DNS pin. A test is such a caller, so it builds the client
//! and hands it in through `with_http`, exactly as `mw_server::upstream_client` does.

use mw_jmap::{JmapClient, JmapError};

#[tokio::test]
async fn streamed_limits_reject_length_and_chunked_overflow_but_allow_exact_body() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    for (wire, allowed) in [
        (
            "HTTP/1.1 200 OK\r\nContent-Length: 9\r\nConnection: close\r\n\r\n123456789",
            false,
        ),
        (
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n4\r\n1234\r\n5\r\n56789\r\n0\r\n\r\n",
            false,
        ),
        (
            "HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\n12345678",
            true,
        ),
    ] {
        for post in [false, true] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let url = format!("http://{}/", listener.local_addr().unwrap());
            tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut request = [0; 2048];
                let _ = stream.read(&mut request).await;
                stream.write_all(wire.as_bytes()).await.unwrap();
            });
            let client = JmapClient::with_http(
                reqwest::Client::builder().no_proxy().build().unwrap(),
                "user",
                "pw",
            );
            let result = if post {
                client
                    .request_raw_limited(&url, bytes::Bytes::new(), 8)
                    .await
                    .map(|(_, b)| b)
            } else {
                client.get_bytes_limited(&url, 8).await.map(|(_, _, b)| b)
            };
            if allowed {
                assert_eq!(result.unwrap().as_ref(), b"12345678");
            } else {
                assert!(matches!(result, Err(JmapError::TooLarge)), "{result:?}");
            }
        }
    }
}
