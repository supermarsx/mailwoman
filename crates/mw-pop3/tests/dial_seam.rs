//! The dial seam (t32-ext §J): a POP3 backend given a dialer opens its sessions
//! through it, and a dialer refusal is final (no direct fallback).

mod common;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use common::{MockServer, mock_config};
use mw_engine::backend::AccountBackend;
use mw_pop3::{LeavePolicy, Pop3Auth, Pop3Backend};
use tokio::net::TcpStream;

const CAPA: &str = include_str!("../../../fixtures/pop3/capa.transcript");

/// Sends every connection to one loopback address, recording what was asked.
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

#[tokio::test]
async fn sessions_open_through_the_given_dialer() {
    let server = MockServer::start(CAPA).await;
    let dialer = Arc::new(RedirectDialer {
        to: server.addr,
        asked: Mutex::new(Vec::new()),
    });
    let mut cfg = mock_config(server.addr, Pop3Auth::UserPass, "u", "p", LeavePolicy::Keep);
    cfg.host = "pop.tunnel.invalid".into();
    cfg.port = 110;
    let caps = Pop3Backend::new(cfg)
        .with_dialer(Some(dialer.clone()))
        .capabilities()
        .await
        .expect("capabilities through the dialer");
    assert!(caps.sasl_plain);
    assert_eq!(
        *dialer.asked.lock().unwrap(),
        vec![("pop.tunnel.invalid".to_string(), 110)]
    );
    server.finish().await;
}

#[tokio::test]
async fn a_dialer_refusal_is_final() {
    let server = MockServer::start(CAPA).await;
    let cfg = mock_config(server.addr, Pop3Auth::UserPass, "u", "p", LeavePolicy::Keep);
    let err = Pop3Backend::new(cfg)
        .with_dialer(Some(Arc::new(RefusingDialer)))
        .capabilities()
        .await
        .expect_err("refused");
    assert!(err.to_string().contains("target not allowed"), "{err}");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert!(
        server.commands().is_empty(),
        "no direct connection was made"
    );
}
