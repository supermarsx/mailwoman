use mw_egress::plan::{EgressPlan, Request, send_with};
use mw_egress::{InternalAllowList, OutBody, Outgoing, TrafficClass, WebPushHeaders};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

#[tokio::test]
async fn push_refuses_plaintext_private_metadata_and_credentials() {
    let plan = EgressPlan::new(vec![], InternalAllowList::empty()).unwrap();
    let request = Request::web_push("vapid t=token, k=key", b"ciphertext").unwrap();
    for url in [
        "http://8.8.8.8/wake",
        "https://127.0.0.1/wake",
        "https://169.254.169.254/wake",
        "https://[::1]/wake",
        "https://user:secret@8.8.8.8/wake",
    ] {
        assert!(
            send_with(&plan, url.parse().unwrap(), &request)
                .await
                .outcome
                .is_err(),
            "{url}"
        );
    }
    assert!(!TrafficClass::Push.internal_allow_eligible());
    assert!(Request::web_push("vapid token\r\nCookie: secret", b"x").is_err());
}

#[tokio::test]
async fn typed_push_headers_cross_only_the_origin_exchange() {
    let (client, server) = tokio::io::duplex(4096);
    let read = tokio::spawn(async move {
        let mut server = BufReader::new(server);
        let mut headers = String::new();
        loop {
            let mut line = String::new();
            server.read_line(&mut line).await.unwrap();
            headers.push_str(&line);
            if line == "\r\n" {
                break;
            }
        }
        let mut body = [0; 10];
        server.read_exact(&mut body).await.unwrap();
        server
            .get_mut()
            .write_all(b"HTTP/1.1 201 Created\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
        (headers, body)
    });
    let out = Outgoing {
        web_push: Some(WebPushHeaders {
            authorization: "vapid t=token, k=key",
        }),
        response_metadata: None,
        authorized_api: None,
        authorized_dav: None,
        response_limit: 8192,
        accept: "*/*",
        user_agent: "Mailwoman",
        body: Some(OutBody {
            content_type: "application/octet-stream",
            bytes: b"ciphertext",
        }),
    };
    let result = mw_egress::proxy::http::exchange_with(
        client,
        &"https://push.example/wake".parse().unwrap(),
        &out,
    )
    .await;
    assert!(result.is_ok());
    let (headers, body) = read.await.unwrap();
    let headers = headers.to_ascii_lowercase();
    assert!(headers.starts_with("post /wake http/1.1"));
    assert!(headers.contains("authorization: vapid t=token, k=key"));
    assert!(headers.contains("content-encoding: aes128gcm"));
    assert!(headers.contains("ttl: 2419200"));
    assert!(!headers.contains("cookie:"));
    assert!(!headers.contains("proxy-authorization:"));
    assert_eq!(&body, b"ciphertext");
}

#[tokio::test]
async fn selected_push_proxy_failure_has_no_direct_fallback_or_credential_leak() {
    use mw_egress::plan::{PlanRoute, RouteScope};
    use mw_egress::proxy::{ProxyRoute, ProxyScheme};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let proxy = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let mut socket = BufReader::new(socket);
        let mut headers = String::new();
        loop {
            let mut line = String::new();
            socket.read_line(&mut line).await.unwrap();
            headers.push_str(&line);
            if line == "\r\n" {
                break;
            }
        }
        socket
            .get_mut()
            .write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(100), listener.accept())
                .await
                .is_err()
        );
        headers
    });
    let route = ProxyRoute {
        id: "push-test".into(),
        scheme: ProxyScheme::HttpConnect,
        host: "127.0.0.1".into(),
        port,
        auth: None,
        allow_plaintext: false,
    };
    let plan = EgressPlan::new(
        vec![PlanRoute {
            scope: RouteScope::Global,
            classes: [TrafficClass::Push].into_iter().collect(),
            route,
        }],
        InternalAllowList::empty(),
    )
    .unwrap();
    let request = Request::web_push("vapid t=private-token, k=key", b"ciphertext").unwrap();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        send_with(&plan, "https://8.8.8.8/wake".parse().unwrap(), &request),
    )
    .await
    .unwrap();
    assert!(result.outcome.is_err());
    let headers = proxy.await.unwrap();
    assert!(headers.starts_with("CONNECT 8.8.8.8:443 HTTP/1.1"));
    assert!(!headers.contains("private-token"));
    assert!(!headers.contains("ciphertext"));
}

#[tokio::test]
async fn web_push_redirect_is_returned_without_following_or_reposting() {
    let (client, server) = tokio::io::duplex(4096);
    let read = tokio::spawn(async move {
        let mut server = BufReader::new(server);
        loop {
            let mut line = String::new();
            server.read_line(&mut line).await.unwrap();
            if line == "\r\n" {
                break;
            }
        }
        let mut body = [0; 1];
        server.read_exact(&mut body).await.unwrap();
        server.get_mut().write_all(b"HTTP/1.1 307 Temporary Redirect\r\nLocation: https://127.0.0.1/steal\r\nContent-Length: 0\r\n\r\n").await.unwrap();
    });
    let out = Outgoing {
        web_push: Some(WebPushHeaders {
            authorization: "vapid t=secret, k=key",
        }),
        response_metadata: None,
        authorized_api: None,
        authorized_dav: None,
        response_limit: 8192,
        accept: "*/*",
        user_agent: "Mailwoman",
        body: Some(OutBody {
            content_type: "application/octet-stream",
            bytes: b"x",
        }),
    };
    let result = mw_egress::proxy::http::exchange_with(
        client,
        &"https://push.example/wake".parse().unwrap(),
        &out,
    )
    .await
    .unwrap();
    assert!(matches!(result, mw_egress::Hop::Redirect(_)));
    read.await.unwrap();
}
