//! The dial seam (t32-ext §J): a submitter given a dialer connects through it,
//! and a dialer refusal is final (no direct fallback, nothing sent).

mod common;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use mw_smtp::{Credentials, Outgoing, Security, SubmitConfig, Submitter};
use tokio::net::TcpStream;

#[derive(Debug)]
struct RedirectDialer {
    to: SocketAddr,
    asked: Mutex<Vec<(String, u16)>>,
}

impl mw_net::Dialer for RedirectDialer {
    fn dial<'a>(
        &'a self,
        host: &'a str,
        port: u16,
    ) -> mw_net::BoxFuture<'a, std::io::Result<mw_net::BoxStream>> {
        Box::pin(async move {
            self.asked.lock().unwrap().push((host.to_string(), port));
            let tcp = TcpStream::connect(self.to).await?;
            Ok(Box::new(tcp) as mw_net::BoxStream)
        })
    }
}

#[derive(Debug)]
struct RefusingDialer;

impl mw_net::Dialer for RefusingDialer {
    fn dial<'a>(
        &'a self,
        _host: &'a str,
        _port: u16,
    ) -> mw_net::BoxFuture<'a, std::io::Result<mw_net::BoxStream>> {
        Box::pin(async {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "target not allowed",
            ))
        })
    }
}

fn fixture(name: &str) -> Vec<String> {
    common::load_script(&format!(
        "{}/../../fixtures/smtp/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
}

fn config(host: &str, port: u16) -> SubmitConfig {
    SubmitConfig {
        host: host.into(),
        port,
        security: Security::Plaintext,
        credentials: Credentials::Plain {
            user: "alice@example.com".into(),
            pass: "s3cret".into(),
        },
        ehlo_name: "client.test".to_string(),
    }
}

fn message() -> Outgoing {
    Outgoing {
        mail_from: "sender@example.com".into(),
        rcpt_to: vec!["good@example.com".into(), "bad@example.com".into()],
        raw: b"From: sender@example.com\r\nSubject: t\r\n\r\nHello.\r\n".to_vec(),
    }
}

#[tokio::test]
async fn submission_runs_through_the_given_dialer() {
    let mock = common::start(fixture("auth_plain_session.txt")).await;
    let dialer = Arc::new(RedirectDialer {
        to: mock.addr,
        asked: Mutex::new(Vec::new()),
    });
    let sub = Submitter::new(config("smtp.tunnel.invalid", 587)).with_dialer(Some(dialer.clone()));
    let result = sub.submit(message()).await.expect("submission");
    assert_eq!(result.accepted, vec!["good@example.com".to_string()]);
    let sent = mock.captured();
    mock.join().await;
    assert!(sent.iter().any(|l| l == "EHLO client.test"));
    assert_eq!(
        *dialer.asked.lock().unwrap(),
        vec![("smtp.tunnel.invalid".to_string(), 587)]
    );
}

#[tokio::test]
async fn a_dialer_refusal_is_final() {
    let mock = common::start(fixture("auth_plain_session.txt")).await;
    let sub = Submitter::new(config(&mock.addr.ip().to_string(), mock.addr.port()))
        .with_dialer(Some(Arc::new(RefusingDialer)));
    let err = sub.submit(message()).await.expect_err("refused");
    assert!(err.to_string().contains("target not allowed"), "{err}");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert_eq!(mock.connections(), 0, "no direct connection was made");
}
