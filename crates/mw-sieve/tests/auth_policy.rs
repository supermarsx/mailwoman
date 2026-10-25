use mw_net::{AuthMechanism as M, PasswordAuthPolicy};
use mw_sieve::{client::ManageSieveClient, managesieve::Credentials, transport::TlsMode};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

async fn attempt(mechanisms: Vec<M>) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        socket
            .write_all(b"\"SASL\" \"PLAIN LOGIN\"\r\nOK \"ready\"\r\n")
            .await
            .unwrap();
        let mut peer = BufReader::new(socket);
        let mut wire = String::new();
        if peer.read_line(&mut wire).await.unwrap() != 0 {
            peer.get_mut()
                .write_all(b"NO \"authentication rejected\"\r\n")
                .await
                .unwrap();
            peer.read_to_string(&mut wire).await.unwrap();
        }
        wire
    });
    let socket = tokio::net::TcpStream::connect(addr).await.unwrap();
    let result = ManageSieveClient::from_stream_with_auth(
        Box::new(socket),
        "127.0.0.1",
        TlsMode::Plaintext,
        Credentials::Plain {
            username: "user".into(),
            password: "secret".into(),
        },
        &mw_net::TlsOptions::default(),
        Some(&PasswordAuthPolicy::new(mechanisms).unwrap()),
    )
    .await;
    assert!(result.is_err());
    server.await.unwrap()
}

#[tokio::test]
async fn advertised_order_is_used_once_without_auth_failure_fallback() {
    let wire = attempt(vec![M::Login, M::Plain]).await;
    assert!(wire.starts_with("AUTHENTICATE \"LOGIN\""));
    assert_eq!(wire.lines().count(), 1);
}

#[tokio::test]
async fn unsupported_password_mechanism_sends_no_credentials() {
    assert!(attempt(vec![M::ScramSha256]).await.is_empty());
}
