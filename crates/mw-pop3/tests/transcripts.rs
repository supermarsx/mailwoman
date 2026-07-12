//! Transcript-driven acceptance tests for `mw-pop3`, run against the scripted
//! mock POP3 socket in `common`. Transcripts live in `fixtures/pop3/`.

mod common;

use std::collections::BTreeSet;
use std::time::Duration;

use common::{MockServer, mock_config};
use mw_engine::backend::{
    AccountBackend, ChangeEvent, ChangeSink, MailboxRole, MessageRef, SyncCursor,
};
use mw_pop3::{LeavePolicy, Pop3Auth, Pop3Backend};
use tokio::sync::mpsc;

const CAPA: &str = include_str!("../../../fixtures/pop3/capa.transcript");
const UIDL_SYNC: &str = include_str!("../../../fixtures/pop3/uidl_sync.transcript");
const RETR: &str = include_str!("../../../fixtures/pop3/retr.transcript");
const DELETE_ON_RETRIEVAL: &str =
    include_str!("../../../fixtures/pop3/delete_on_retrieval.transcript");
const DELETE_AFTER_DAYS_OLD: &str =
    include_str!("../../../fixtures/pop3/delete_after_days_old.transcript");
const DELETE_AFTER_DAYS_RECENT: &str =
    include_str!("../../../fixtures/pop3/delete_after_days_recent.transcript");
const SASL_LOGIN: &str = include_str!("../../../fixtures/pop3/sasl_login.transcript");

fn pop3_ref(uidl: &str) -> MessageRef {
    MessageRef::Pop3 {
        uidl: uidl.to_string(),
    }
}

fn seen(uidls: &[&str]) -> SyncCursor {
    SyncCursor::Pop3Uidl {
        seen: uidls.iter().map(|s| s.to_string()).collect(),
    }
}

#[tokio::test]
async fn capabilities_parse_from_capa() {
    let server = MockServer::start(CAPA).await;
    let cfg = mock_config(server.addr, Pop3Auth::UserPass, "u", "p", LeavePolicy::Keep);
    let caps = Pop3Backend::new(cfg).capabilities().await.unwrap();

    assert!(caps.sasl_plain, "PLAIN should be detected");
    assert!(caps.sasl_login, "LOGIN should be detected");
    assert!(caps.sasl_xoauth2, "XOAUTH2 should be detected");
    // POP3 advertises no IMAP extensions.
    assert!(!caps.imap4rev2 && !caps.qresync && !caps.idle && !caps.uidplus);
    server.finish().await;
}

#[tokio::test]
async fn sync_mailbox_uidl_diff_yields_added_and_removed() {
    let server = MockServer::start(UIDL_SYNC).await;
    let cfg = mock_config(server.addr, Pop3Auth::UserPass, "u", "p", LeavePolicy::Keep);
    let backend = Pop3Backend::new(cfg);

    // Cursor already ingested UIDL-A and a now-gone UIDL-Z.
    let cursor = seen(&["UIDL-A", "UIDL-Z"]);
    let mbox = mw_engine::backend::RawMailboxRef {
        name: "INBOX".into(),
        uidvalidity: 0,
    };
    let delta = backend.sync_mailbox(&mbox, &cursor).await.unwrap();

    assert_eq!(delta.added, vec![pop3_ref("UIDL-B")], "UIDL-B is new");
    assert_eq!(delta.removed, vec![pop3_ref("UIDL-Z")], "UIDL-Z vanished");
    assert!(delta.flag_changes.is_empty());
    match delta.next_cursor {
        SyncCursor::Pop3Uidl { seen } => {
            let expected: BTreeSet<String> =
                ["UIDL-A", "UIDL-B"].iter().map(|s| s.to_string()).collect();
            assert_eq!(seen, expected);
        }
        other => panic!("expected Pop3Uidl cursor, got {other:?}"),
    }
    // Keep policy must never DELE.
    assert!(!server.commands().iter().any(|c| c.starts_with("DELE")));
    server.finish().await;
}

#[tokio::test]
async fn fetch_raw_returns_message_bytes_and_keep_leaves_server() {
    let server = MockServer::start(RETR).await;
    let cfg = mock_config(server.addr, Pop3Auth::UserPass, "u", "p", LeavePolicy::Keep);
    let backend = Pop3Backend::new(cfg);

    let msgs = backend.fetch_raw(&[pop3_ref("UIDL-B")]).await.unwrap();
    assert_eq!(msgs.len(), 1);
    let raw = String::from_utf8(msgs[0].raw.clone()).unwrap();
    assert_eq!(
        raw,
        "From: a@example.com\r\nTo: b@example.com\r\nSubject: Hello\r\n\r\nBody line one.\r\nBody line two.\r\n"
    );
    assert_eq!(msgs[0].message_ref, pop3_ref("UIDL-B"));
    assert!(msgs[0].flags.is_empty(), "POP3 carries no server flags");

    // Keep policy: RETR but no DELE.
    let cmds = server.commands();
    assert!(cmds.iter().any(|c| c == "RETR 2"));
    assert!(!cmds.iter().any(|c| c.starts_with("DELE")));
    server.finish().await;
}

#[tokio::test]
async fn delete_on_retrieval_issues_dele() {
    let server = MockServer::start(DELETE_ON_RETRIEVAL).await;
    let cfg = mock_config(
        server.addr,
        Pop3Auth::UserPass,
        "u",
        "p",
        LeavePolicy::DeleteOnRetrieval,
    );
    let backend = Pop3Backend::new(cfg);

    let msgs = backend.fetch_raw(&[pop3_ref("UIDL-B")]).await.unwrap();
    assert_eq!(msgs.len(), 1);

    let cmds = server.commands();
    assert!(cmds.iter().any(|c| c == "RETR 2"), "must retrieve");
    assert!(
        cmds.iter().any(|c| c == "DELE 2"),
        "must delete on retrieval"
    );
    server.finish().await;
}

#[tokio::test]
async fn delete_after_days_reaps_old_message() {
    let server = MockServer::start(DELETE_AFTER_DAYS_OLD).await;
    let cfg = mock_config(
        server.addr,
        Pop3Auth::UserPass,
        "u",
        "p",
        LeavePolicy::DeleteAfterDays(7),
    );
    let backend = Pop3Backend::new(cfg);

    // UIDL-A was ingested before; its Date (2018) is well past 7 days.
    let delta = backend
        .sync_mailbox(
            &mw_engine::backend::RawMailboxRef {
                name: "INBOX".into(),
                uidvalidity: 0,
            },
            &seen(&["UIDL-A"]),
        )
        .await
        .unwrap();

    assert!(delta.added.is_empty());
    assert_eq!(delta.removed, vec![pop3_ref("UIDL-A")], "old msg reaped");
    match delta.next_cursor {
        SyncCursor::Pop3Uidl { seen } => assert!(seen.is_empty(), "reaped msg left the cursor"),
        other => panic!("unexpected cursor {other:?}"),
    }

    let cmds = server.commands();
    assert!(cmds.iter().any(|c| c == "TOP 1 0"), "age probe via TOP");
    assert!(cmds.iter().any(|c| c == "DELE 1"), "old msg deleted");
    server.finish().await;
}

#[tokio::test]
async fn delete_after_days_keeps_recent_message() {
    // Substitute a fresh Date so the fixture never ages into a false positive.
    let now = chrono::Utc::now().to_rfc2822();
    let script = DELETE_AFTER_DAYS_RECENT.replace("{DATE}", &now);
    let server = MockServer::start(&script).await;
    let cfg = mock_config(
        server.addr,
        Pop3Auth::UserPass,
        "u",
        "p",
        LeavePolicy::DeleteAfterDays(7),
    );
    let backend = Pop3Backend::new(cfg);

    let delta = backend
        .sync_mailbox(
            &mw_engine::backend::RawMailboxRef {
                name: "INBOX".into(),
                uidvalidity: 0,
            },
            &seen(&["UIDL-A"]),
        )
        .await
        .unwrap();

    assert!(delta.removed.is_empty(), "recent msg must survive");
    let cmds = server.commands();
    assert!(cmds.iter().any(|c| c == "TOP 1 0"), "still probes age");
    assert!(!cmds.iter().any(|c| c.starts_with("DELE")), "no delete");
    server.finish().await;
}

#[tokio::test]
async fn sasl_login_authenticates_and_lists_inbox() {
    let server = MockServer::start(SASL_LOGIN).await;
    let cfg = mock_config(
        server.addr,
        Pop3Auth::SaslLogin,
        "alice",
        "secret",
        LeavePolicy::Keep,
    );
    let boxes = Pop3Backend::new(cfg).list_mailboxes().await.unwrap();

    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].role, MailboxRole::Inbox);
    assert_eq!(boxes[0].mailbox_ref.name, "INBOX");
    assert_eq!(boxes[0].total, 3, "STAT count");

    let cmds = server.commands();
    assert!(cmds.iter().any(|c| c == "AUTH LOGIN"));
    assert!(cmds.iter().any(|c| c == "YWxpY2U="), "base64 username");
    assert!(cmds.iter().any(|c| c == "c2VjcmV0"), "base64 password");
    server.finish().await;
}

#[tokio::test]
async fn watch_emits_on_interval_and_stops() {
    // watch() needs no server: it emits MailboxChanged on a timer.
    let cfg = mock_config(
        "127.0.0.1:1".parse::<std::net::SocketAddr>().unwrap(),
        Pop3Auth::UserPass,
        "u",
        "p",
        LeavePolicy::Keep,
    );
    let backend = Pop3Backend::new(cfg);

    let (tx, mut rx) = mpsc::unbounded_channel();
    let handle = backend.watch(ChangeSink::new(tx)).await.unwrap();

    let evt = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("watch should emit within 2s")
        .expect("channel open");
    match evt {
        ChangeEvent::MailboxChanged { mailbox } => assert_eq!(mailbox.name, "INBOX"),
        other => panic!("unexpected event {other:?}"),
    }

    handle.stop();
    // After stop, the loop terminates and eventually drops the sender.
    tokio::time::timeout(Duration::from_secs(2), async {
        while rx.recv().await.is_some() {}
    })
    .await
    .expect("watch loop should stop and close the channel");
}

#[tokio::test]
async fn folder_ops_are_unsupported_and_store_flags_is_noop() {
    let cfg = mock_config(
        "127.0.0.1:1".parse::<std::net::SocketAddr>().unwrap(),
        Pop3Auth::UserPass,
        "u",
        "p",
        LeavePolicy::Keep,
    );
    let backend = Pop3Backend::new(cfg);
    let dest = mw_engine::backend::RawMailboxRef {
        name: "Archive".into(),
        uidvalidity: 0,
    };

    // store_flags is a no-op success (engine keeps flags locally).
    backend.store_flags(&[], &[], &[]).await.unwrap();

    // move/append are structurally impossible over POP3.
    assert!(matches!(
        backend.move_messages(&[pop3_ref("X")], &dest).await,
        Err(mw_engine::backend::EngineError::Unsupported(_))
    ));
    assert!(matches!(
        backend.append(&dest, b"raw", &[]).await,
        Err(mw_engine::backend::EngineError::Unsupported(_))
    ));
}
