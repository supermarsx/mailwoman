//! Live POP3 integration against a real server (Greenmail in e7's CI stack).
//!
//! Ignored by default so `cargo test -p mw-pop3` is green with no server. e7
//! runs it by exporting `GREENMAIL_POP3=host:port` (plus optional
//! `GREENMAIL_USER` / `GREENMAIL_PASS`, default `mailwoman`/`mailwoman`) and
//! invoking `cargo test -p mw-pop3 -- --ignored`.

use std::time::Duration;

use mw_engine::backend::{AccountBackend, SyncCursor};
use mw_pop3::{LeavePolicy, Pop3Auth, Pop3Backend, Pop3Config, TlsMode};

fn live_config() -> Option<Pop3Config> {
    let addr = std::env::var("GREENMAIL_POP3").ok()?;
    let (host, port) = addr.rsplit_once(':')?;
    let user = std::env::var("GREENMAIL_USER").unwrap_or_else(|_| "mailwoman".into());
    let pass = std::env::var("GREENMAIL_PASS").unwrap_or_else(|_| "mailwoman".into());
    Some(Pop3Config {
        host: host.to_string(),
        port: port.parse().ok()?,
        // Greenmail's default POP3 listener is plaintext; e7 may flip to :995.
        tls: if std::env::var("GREENMAIL_POP3_TLS").is_ok() {
            TlsMode::Implicit
        } else {
            TlsMode::Plain
        },
        auth: Pop3Auth::UserPass,
        username: user,
        secret: pass,
        leave_policy: LeavePolicy::Keep,
        poll_interval: Duration::from_secs(30),
    })
}

#[tokio::test]
#[ignore = "requires a live POP3 server; set GREENMAIL_POP3=host:port"]
async fn live_list_and_sync() {
    let Some(cfg) = live_config() else {
        eprintln!("GREENMAIL_POP3 unset; skipping live test");
        return;
    };
    let backend = Pop3Backend::new(cfg);

    let boxes = backend.list_mailboxes().await.expect("list INBOX");
    assert_eq!(boxes.len(), 1);
    let mbox = boxes[0].mailbox_ref.clone();

    // First sync from an empty cursor: every present message is "added".
    let empty = SyncCursor::Pop3Uidl {
        seen: Default::default(),
    };
    let delta = backend.sync_mailbox(&mbox, &empty).await.expect("sync");
    assert_eq!(delta.added.len(), boxes[0].total as usize);

    // Fetch the raw bytes for whatever is there; must be non-empty RFC822.
    if let Some(first) = delta.added.first() {
        let msgs = backend
            .fetch_raw(std::slice::from_ref(first))
            .await
            .unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(!msgs[0].raw.is_empty());
    }
}
