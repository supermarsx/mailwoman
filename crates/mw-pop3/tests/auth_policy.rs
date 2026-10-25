use mw_net::{AuthMechanism as M, PasswordAuthPolicy};
use mw_pop3::{LeavePolicy, Pop3Auth, Pop3Config, TlsMode, conn::Pop3Conn};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

async fn attempt(mechanisms: Vec<M>, user: bool) -> (mw_engine::backend::EngineError, String) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        socket.write_all(b"+OK ready\r\n").await.unwrap();
        let mut peer = BufReader::new(socket);
        let mut line = String::new();
        peer.read_line(&mut line).await.unwrap();
        assert_eq!(line, "CAPA\r\n");
        peer.get_mut()
            .write_all(
                format!(
                    "+OK capabilities\r\nSASL PLAIN LOGIN SCRAM-SHA-256-PLUS\r\n{}.\r\n",
                    if user { "USER\r\n" } else { "" }
                )
                .as_bytes(),
            )
            .await
            .unwrap();
        let mut auth = String::new();
        if peer.read_line(&mut auth).await.unwrap() != 0 {
            peer.get_mut()
                .write_all(b"-ERR authentication rejected\r\n")
                .await
                .unwrap();
            peer.read_to_string(&mut auth).await.unwrap();
        }
        auth
    });
    let cfg = Pop3Config {
        host: "127.0.0.1".into(),
        port: addr.port(),
        tls: TlsMode::Plain,
        auth: Pop3Auth::UserPass,
        username: "user".into(),
        secret: "secret".into(),
        leave_policy: LeavePolicy::Keep,
        poll_interval: std::time::Duration::from_secs(60),
    };
    let error = Pop3Conn::open_via_with_auth(
        &cfg,
        None,
        &mw_net::TlsOptions::default(),
        Some(&PasswordAuthPolicy::new(mechanisms).unwrap()),
    )
    .await
    .err()
    .expect("authentication refused");
    (error, server.await.unwrap())
}

#[tokio::test]
async fn policy_uses_post_transport_capabilities_and_never_retries_rejected_auth() {
    let (error, wire) = attempt(vec![M::Plain, M::Login, M::Pop3UserPass], true).await;
    assert!(matches!(error, mw_engine::backend::EngineError::Auth(_)));
    assert!(wire.starts_with("AUTH PLAIN "));
    assert_eq!(wire.lines().count(), 1);
}

#[tokio::test]
async fn unsupported_and_unbound_mechanisms_send_no_credentials() {
    for mechanism in [M::ScramSha256, M::ScramSha256Plus] {
        let (error, wire) = attempt(vec![mechanism], true).await;
        assert!(matches!(
            error,
            mw_engine::backend::EngineError::Unsupported(_)
        ));
        assert!(wire.is_empty());
    }
}

#[tokio::test]
async fn user_pass_is_refused_when_successful_capa_does_not_advertise_it() {
    let (error, wire) = attempt(vec![M::Pop3UserPass], false).await;
    assert!(matches!(
        error,
        mw_engine::backend::EngineError::Unsupported(_)
    ));
    assert!(wire.is_empty());
}
