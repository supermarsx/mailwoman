use mw_egress::plan::{EgressPlan, PlanRoute, Request, RouteScope};
use mw_egress::proxy::{ProxyRoute, ProxyScheme};
use mw_egress::{
    AuthorizedDav, DavMethod, InternalAllowList, OutBody, Outgoing, Refusal, TrafficClass,
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

fn auth(method: DavMethod) -> AuthorizedDav<'static> {
    AuthorizedDav {
        additional_roots: &[],
        method,
        authorization: "Basic cHJpdmF0ZTpwYXNzd29yZA==",
        depth: Some(1),
        ocs: true,
        if_match: Some("\"rev1\""),
    }
}

#[tokio::test]
async fn selected_proxy_receives_restricted_dav_headers_and_never_retries_rejection_or_redirect() {
    for (method, status) in [
        (DavMethod::Propfind, "401 Unauthorized"),
        (DavMethod::Get, "302 Found"),
    ] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let plan = EgressPlan::new(
            vec![PlanRoute {
                scope: RouteScope::Domain("mail.test".into()),
                classes: [TrafficClass::Nextcloud].into_iter().collect(),
                route: ProxyRoute {
                    id: "cloud".into(),
                    scheme: ProxyScheme::HttpConnect,
                    host: "127.0.0.1".into(),
                    port,
                    auth: None,
                    allow_plaintext: true,
                },
            }],
            InternalAllowList::empty(),
        )
        .unwrap();
        let task = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let mut socket = BufReader::new(socket);
            let mut connect = String::new();
            socket.read_line(&mut connect).await.unwrap();
            assert_eq!(connect, "CONNECT 8.8.8.8:80 HTTP/1.1\r\n");
            loop {
                let mut line = String::new();
                socket.read_line(&mut line).await.unwrap();
                if line == "\r\n" {
                    break;
                }
                assert!(!line.to_ascii_lowercase().starts_with("authorization:"));
            }
            socket
                .get_mut()
                .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .await
                .unwrap();
            let mut head = String::new();
            loop {
                let mut line = String::new();
                socket.read_line(&mut line).await.unwrap();
                head.push_str(&line);
                if line == "\r\n" {
                    break;
                }
            }
            assert!(head.starts_with(&format!(
                "{} /remote.php/dav/files/alice/ HTTP/1.1\r\n",
                method.as_str()
            )));
            let lower = head.to_ascii_lowercase();
            assert!(lower.contains("authorization: basic c"));
            assert!(lower.contains("depth: 1"));
            assert!(lower.contains("ocs-apirequest: true"));
            assert!(lower.contains("if-match: \"rev1\""));
            assert!(!lower.contains("cookie:"));
            assert!(!lower.contains("proxy-authorization:"));
            socket.get_mut().write_all(format!("HTTP/1.1 {status}\r\nLocation: http://8.8.4.4/stolen\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes()).await.unwrap();
            drop(socket);
            assert!(
                tokio::time::timeout(std::time::Duration::from_millis(150), listener.accept())
                    .await
                    .is_err()
            );
        });
        let request = Request::dav(auth(method), None)
            .unwrap()
            .for_domain("mail.test");
        let result = mw_egress::plan::send_with(
            &plan,
            "http://8.8.8.8/remote.php/dav/files/alice/"
                .parse()
                .unwrap(),
            &request,
        )
        .await;
        assert!(result.traversed_proxy);
        assert!(result.outcome.is_err());
        task.await.unwrap();
    }
}

#[tokio::test]
async fn cancelled_dav_exchange_closes_connection_pump() {
    let (client, mut server) = tokio::io::duplex(4096);
    let task = tokio::spawn(async move {
        let out = Outgoing {
            authorized_dav: Some(auth(DavMethod::Propfind)),
            web_push: None,
            response_limit: 1024,
            accept: "application/xml",
            user_agent: "Mailwoman",
            body: None,
        };
        mw_egress::proxy::http::exchange_with(client, &"http://cloud.test/".parse().unwrap(), &out)
            .await
    });
    let mut header = Vec::new();
    while !header.ends_with(b"\r\n\r\n") {
        header.push(server.read_u8().await.unwrap());
    }
    task.abort();
    let _ = task.await;
    let mut byte = [0u8];
    assert_eq!(
        tokio::time::timeout(std::time::Duration::from_secs(1), server.read(&mut byte))
            .await
            .unwrap()
            .unwrap(),
        0
    );
}

#[test]
fn dav_credentials_are_redacted_and_headers_and_body_are_bounded() {
    let auth = auth(DavMethod::Get);
    assert!(!format!("{auth:?}").contains("cHJp"));
    assert!(
        Request::dav(
            AuthorizedDav {
                authorization: "Basic secret\r\nHost: attacker",
                ..auth
            },
            None
        )
        .is_err()
    );
    assert!(
        Request::dav(
            AuthorizedDav {
                depth: Some(2),
                ..auth
            },
            None
        )
        .is_err()
    );
    let body = vec![0u8; mw_egress::MAX_IMAGE_BYTES + 1];
    assert!(matches!(
        Request::dav(
            auth,
            Some(OutBody {
                content_type: "application/octet-stream",
                bytes: &body
            })
        ),
        Err(Refusal::TooLarge)
    ));
}

#[tokio::test]
async fn additional_ca_verifies_origin_inside_proxy_without_disabling_hostname_checks() {
    use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
    use std::sync::Arc;
    for (trusted, correct_name) in [(true, true), (false, true), (true, false)] {
        let certified = rcgen::generate_simple_self_signed(vec![
            if correct_name {
                "8.8.8.8"
            } else {
                "wrong.test"
            }
            .into(),
        ])
        .unwrap();
        let certificate = certified.cert.der().to_vec();
        let config = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(certificate.clone())],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                certified.signing_key.serialize_der(),
            )),
        )
        .unwrap();
        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let plan = EgressPlan::new(
            vec![PlanRoute {
                scope: RouteScope::Global,
                classes: [TrafficClass::Nextcloud].into_iter().collect(),
                route: ProxyRoute {
                    id: "tls-cloud".into(),
                    scheme: ProxyScheme::HttpConnect,
                    host: "127.0.0.1".into(),
                    port: listener.local_addr().unwrap().port(),
                    auth: None,
                    allow_plaintext: false,
                },
            }],
            InternalAllowList::empty(),
        )
        .unwrap();
        let peer = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let mut socket = BufReader::new(socket);
            loop {
                let mut line = String::new();
                socket.read_line(&mut line).await.unwrap();
                if line == "\r\n" {
                    break;
                }
            }
            socket
                .get_mut()
                .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .await
                .unwrap();
            let Ok(stream) = acceptor.accept(socket.into_inner()).await else {
                return false;
            };
            let mut stream = BufReader::new(stream);
            let mut head = String::new();
            loop {
                let mut line = String::new();
                match stream.read_line(&mut line).await {
                    Ok(0) | Err(_) => return false,
                    _ => {}
                }
                head.push_str(&line);
                if line == "\r\n" {
                    break;
                }
            }
            assert!(head.to_ascii_lowercase().contains("authorization: basic "));
            stream
                .get_mut()
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok")
                .await
                .unwrap();
            true
        });
        let roots = if trusted {
            vec![certificate]
        } else {
            Vec::new()
        };
        let request = Request::dav(
            AuthorizedDav {
                additional_roots: &roots,
                ..auth(DavMethod::Get)
            },
            None,
        )
        .unwrap();
        let response = mw_egress::plan::send_with(
            &plan,
            "https://8.8.8.8/remote.php/dav/".parse().unwrap(),
            &request,
        )
        .await;
        assert_eq!(response.outcome.is_ok(), trusted && correct_name);
        assert_eq!(
            peer.await.unwrap(),
            trusted && correct_name,
            "credentials must not cross a failed TLS handshake"
        );
    }
}
