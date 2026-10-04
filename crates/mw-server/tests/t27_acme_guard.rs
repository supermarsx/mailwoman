//! t27-e3 (OH-2, decision D4) — `--acme` takes domain names; a value that is not one
//! is refused at startup, before the listener is bound.
//!
//! Five deploy recipes used to say `--acme off`. `--acme` has no "off" value: any
//! value selects ACME, so those recipes started an HTTPS listener behind a proxy
//! speaking plain HTTP and asked Let's Encrypt for a certificate for a host named
//! `off`. The docs are fixed separately (t27-e5); this is the server refusing the
//! value for anyone who already followed them.
//!
//! The check is in `TlsListener::bind`, so what is under test is that function, by
//! invocation: a valid list binds the port (the control — it shows the refusals
//! below are the validator's, not a bind that would have failed anyway), and a
//! refused list leaves the port free.
//!
//! Run:
//!   cargo test -p mw-server --test t27_acme_guard --locked -- --test-threads=1

use std::net::SocketAddr;

use mw_server::tls::{TlsConfig, TlsListener, validate_acme_domains};

mod common;
use common::test_db;

fn acme(domains: &[&str]) -> TlsConfig {
    TlsConfig::Acme {
        domains: domains.iter().map(|d| d.to_string()).collect(),
        contact: None,
        cache_dir: test_db::unique_dir("mw-t27-acme"),
        staging: true,
    }
}

/// A loopback port that was free a moment ago.
fn free_port() -> SocketAddr {
    let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    probe.local_addr().unwrap()
}

/// Whether something is listening on `addr`.
fn is_listening(addr: SocketAddr) -> bool {
    std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(500)).is_ok()
}

#[tokio::test]
async fn bind_refuses_a_non_hostname_and_leaves_the_port_unbound() {
    // Control: a real hostname is accepted and the port IS bound. (No certificate
    // is requested by binding: issuance is driven per handshake, and none is made.)
    let addr = free_port();
    let (listener, resolver) = TlsListener::bind(&addr.to_string(), &acme(&["mail.example.org"]))
        .await
        .expect("a hostname is accepted");
    assert!(resolver.is_none(), "ACME mode has no reloadable resolver");
    assert!(is_listening(addr), "control: bind() bound the port");
    drop(listener);

    for bad in [
        vec!["off"],
        vec!["none"],
        vec!["false"],
        vec![""],
        vec!["localhost"],
        vec!["mail.example.org", "off"],
        vec![],
    ] {
        let addr = free_port();
        let err = match TlsListener::bind(&addr.to_string(), &acme(&bad)).await {
            Ok(_) => panic!("--acme {bad:?} must be refused"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("MW_ACME"),
            "--acme {bad:?}: the error says what to do instead: {err}"
        );
        assert!(
            !is_listening(addr),
            "--acme {bad:?}: nothing is listening after the refusal"
        );
        // And the port is still free for the process that fixes its config.
        std::net::TcpListener::bind(addr)
            .unwrap_or_else(|e| panic!("--acme {bad:?}: the port must still be free: {e}"));
    }
}

#[test]
fn the_error_names_the_value_and_how_to_disable_acme() {
    let err = validate_acme_domains(&["off".to_string()])
        .unwrap_err()
        .to_string();
    assert!(err.contains("\"off\""), "names the offending value: {err}");
    assert!(
        err.contains("do not pass --acme") && err.contains("leave MW_ACME unset"),
        "says how to run without ACME: {err}"
    );
    assert!(validate_acme_domains(&["mail.example.org".to_string()]).is_ok());
}
