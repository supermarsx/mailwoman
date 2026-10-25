//! The dial seam (t32-ext §J): `ManageSieveClient::from_stream` runs a session
//! over a stream the caller already holds (an egress tunnel or an SSH channel),
//! and `connect_via` dials through a given dialer with no direct fallback.

use std::sync::Mutex;

use mw_sieve::{Credentials, ManageSieveClient, TlsMode};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const GREETING: &str = concat!(
    "\"IMPLEMENTATION\" \"Example ManageSieve v1\"\r\n",
    "\"SIEVE\" \"fileinto vacation\"\r\n",
    "\"SASL\" \"PLAIN\"\r\n",
    "\"VERSION\" \"1.0\"\r\n",
    "OK \"ManageSieve ready.\"\r\n",
);

fn creds() -> Credentials {
    Credentials::Plain {
        username: "alice".into(),
        password: "secret".into(),
    }
}

#[tokio::test]
async fn a_session_runs_over_a_stream_the_caller_holds() {
    let (client, mut server) = tokio::io::duplex(1 << 16);
    server.write_all(GREETING.as_bytes()).await.unwrap();
    server.write_all(b"OK \"Logged in.\"\r\n").await.unwrap();
    server
        .write_all(b"\"main\" ACTIVE\r\nOK \"Listscripts completed.\"\r\n")
        .await
        .unwrap();

    let mut session = ManageSieveClient::from_stream(
        Box::new(client),
        "sieve.example",
        TlsMode::Plaintext,
        creds(),
    )
    .await
    .expect("session over the held stream");
    assert!(session.capabilities().sieve.iter().any(|e| e == "vacation"));
    let scripts = session.list_scripts().await.expect("listscripts");
    assert_eq!(scripts.len(), 1);
    assert!(scripts[0].active);

    drop(session);
    let mut wire = String::new();
    server.read_to_string(&mut wire).await.unwrap();
    assert!(wire.starts_with("AUTHENTICATE \"PLAIN\""), "{wire}");
    assert!(wire.contains("LISTSCRIPTS\r\n"), "{wire}");
}

#[derive(Debug, Default)]
struct RefusingDialer(Mutex<Vec<(String, u16)>>);

impl mw_net::Dialer for RefusingDialer {
    fn dial<'a>(
        &'a self,
        host: &'a str,
        port: u16,
    ) -> mw_net::BoxFuture<'a, std::io::Result<mw_net::BoxStream>> {
        Box::pin(async move {
            self.0.lock().unwrap().push((host.to_string(), port));
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "target not allowed",
            ))
        })
    }
}

#[tokio::test]
async fn connect_via_uses_the_dialer_and_its_refusal_is_final() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let dialer = RefusingDialer::default();
    let host = addr.ip().to_string();
    let attempt = ManageSieveClient::connect_via(
        Some(&dialer),
        &host,
        addr.port(),
        TlsMode::Plaintext,
        creds(),
    );
    // A direct connection would wait for a greeting that never comes.
    let err = tokio::time::timeout(std::time::Duration::from_secs(5), attempt)
        .await
        .expect("answered without waiting on a direct connection")
        .err()
        .expect("refused");
    assert!(err.to_string().contains("target not allowed"), "{err}");
    assert_eq!(dialer.0.lock().unwrap().len(), 1);
    let accepted =
        tokio::time::timeout(std::time::Duration::from_millis(100), listener.accept()).await;
    assert!(accepted.is_err(), "no direct connection was made");
}

#[tokio::test]
async fn checked_save_refuses_external_redirect_before_any_script_command() {
    let (client, mut server) = tokio::io::duplex(1 << 16);
    server.write_all(GREETING.as_bytes()).await.unwrap();
    server.write_all(b"OK \"Logged in.\"\r\n").await.unwrap();
    let mut session = ManageSieveClient::from_stream(
        Box::new(client),
        "sieve.example",
        TlsMode::Plaintext,
        creds(),
    )
    .await
    .unwrap();
    let policy = mw_sieve::policy::Policy {
        raw_edit: true,
        own_domains: vec!["example.org".into()],
        ..Default::default()
    };
    let error = session
        .put_script_checked(
            "main",
            "redirect \"user@outside.test\";",
            &policy,
            mw_sieve::policy::EditKind::Raw,
        )
        .await
        .unwrap_err();
    assert!(matches!(error, mw_sieve::client::SaveError::Policy(_)));
    drop(session);
    let mut wire = String::new();
    server.read_to_string(&mut wire).await.unwrap();
    assert!(wire.starts_with("AUTHENTICATE"));
    assert!(!wire.contains("PUTSCRIPT"));
    assert!(!wire.contains("LISTSCRIPTS"));
    assert!(!wire.contains("outside.test"));
}
