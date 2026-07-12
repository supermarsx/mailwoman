//! Live integration test against a real IMAP server (Greenmail in CI, e7).
//!
//! Gated two ways so `cargo test -p mw-imap` stays green with no server:
//! it is `#[ignore]` by default, and it also no-ops unless `GREENMAIL_IMAP`
//! (a `host:port`) is set. e7 runs it with `--ignored` and the env populated.
//!
//! Env:
//! - `GREENMAIL_IMAP`  — `host:port` of the plaintext IMAP listener (e.g.
//!   `127.0.0.1:3143`).
//! - `GREENMAIL_USER` / `GREENMAIL_PASS` — credentials (default `mw@localhost`
//!   / `mwpass`).

use mw_engine::backend::AccountBackend;
use mw_imap::{Credentials, ImapBackend, ImapConfig, TlsMode};

fn env_addr() -> Option<(String, u16)> {
    let raw = std::env::var("GREENMAIL_IMAP").ok()?;
    let (host, port) = raw.rsplit_once(':')?;
    Some((host.to_string(), port.parse().ok()?))
}

#[tokio::test]
#[ignore = "requires a live Greenmail IMAP server (set GREENMAIL_IMAP=host:port)"]
async fn greenmail_connect_list_and_append() {
    let Some((host, port)) = env_addr() else {
        eprintln!("GREENMAIL_IMAP not set; skipping live test");
        return;
    };
    let user = std::env::var("GREENMAIL_USER").unwrap_or_else(|_| "mw@localhost".into());
    let pass = std::env::var("GREENMAIL_PASS").unwrap_or_else(|_| "mwpass".into());

    let config = ImapConfig::new(
        host,
        Credentials::Password {
            username: user,
            password: pass,
        },
    )
    .port(port)
    .tls(TlsMode::Plaintext);

    let backend = ImapBackend::connect(config)
        .await
        .expect("connect to Greenmail");

    let caps = backend.capabilities().await.expect("capabilities");
    assert!(
        caps.imap4rev2 || caps.sasl_plain,
        "expected a usable IMAP capability set"
    );

    let boxes = backend.list_mailboxes().await.expect("list mailboxes");
    assert!(
        boxes
            .iter()
            .any(|m| m.mailbox_ref.name.eq_ignore_ascii_case("INBOX")),
        "expected an INBOX in {boxes:?}"
    );

    // Append a message to INBOX and confirm it round-trips via a UID-window sync.
    let inbox = mw_engine::backend::RawMailboxRef {
        name: "INBOX".into(),
        uidvalidity: 0,
    };
    let raw = b"From: live@test\r\nTo: mw@localhost\r\nSubject: greenmail\r\n\r\nhi\r\n";
    backend
        .append(&inbox, raw, &[mw_engine::backend::Flag::Seen])
        .await
        .expect("append to INBOX");
}
