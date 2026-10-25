use mw_imap::{Credentials, ImapError, Session, transport::TlsMode};
use mw_net::{AuthMechanism as M, PasswordAuthPolicy};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

async fn attempt(mechanisms: Vec<M>, capabilities: &'static str) -> (ImapError, String) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        socket
            .write_all(format!("* OK [CAPABILITY IMAP4rev1 {capabilities}] ready\r\n").as_bytes())
            .await
            .unwrap();
        let mut peer = BufReader::new(socket);
        let mut wire = String::new();
        if peer.read_line(&mut wire).await.unwrap() != 0 {
            let tag = wire.split_whitespace().next().unwrap();
            peer.get_mut()
                .write_all(format!("{tag} NO authentication rejected\r\n").as_bytes())
                .await
                .unwrap();
            peer.read_to_string(&mut wire).await.unwrap();
        }
        wire
    });
    let mut session = Session::connect("127.0.0.1", addr.port(), TlsMode::Plaintext)
        .await
        .unwrap();
    let error = session
        .login(&Credentials::PasswordWithPolicy {
            username: "user".into(),
            password: "secret".into(),
            policy: PasswordAuthPolicy::new(mechanisms).unwrap(),
        })
        .await
        .unwrap_err();
    drop(session);
    (error, server.await.unwrap())
}

#[tokio::test]
async fn actual_capabilities_choose_requested_order_and_rejection_never_retries() {
    let (error, wire) = attempt(vec![M::Login, M::Plain], "AUTH=PLAIN AUTH=LOGIN").await;
    assert!(matches!(error, ImapError::Auth(_)));
    assert!(wire.contains(" AUTHENTICATE LOGIN\r\n"), "{wire}");
    assert_eq!(wire.lines().count(), 1);
}

#[tokio::test]
async fn unavailable_or_unbound_or_disabled_mechanisms_send_no_credentials() {
    for (mechanism, caps) in [
        (M::ScramSha256, "AUTH=PLAIN"),
        (M::ScramSha256Plus, "AUTH=SCRAM-SHA-256-PLUS"),
        (M::ImapLogin, "LOGINDISABLED AUTH=PLAIN"),
    ] {
        let (error, wire) = attempt(vec![mechanism], caps).await;
        assert!(matches!(error, ImapError::Unsupported(_)));
        assert!(wire.is_empty());
    }
}
