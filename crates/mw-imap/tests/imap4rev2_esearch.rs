//! t32-h1 (plan D-2): a server that advertises IMAP4rev2 answers SEARCH with
//! `* ESEARCH` once rev2 is enabled (RFC 9051 §7.3.4). `imap-proto` 0.16.7
//! cannot parse ESEARCH, so the session routes extended search replies through
//! the bounded extension parser. The mock requires both ENABLE IMAP4rev2 and
//! RETURN (ALL COUNT MIN MAX), proving this parser is used by real syncs.

use std::net::SocketAddr;

use mw_engine::backend::{AccountBackend, MessageRef, RawMailboxRef, SyncCursor};
use mw_imap::{Credentials, ImapBackend, ImapConfig, TlsMode};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const CAPS: &str = "IMAP4rev1 IMAP4rev2 AUTH=PLAIN ENABLE ESEARCH";

/// Serve one connection, switching SEARCH to ESEARCH after `ENABLE IMAP4rev2`.
async fn spawn_rev2_mock(search_override: Option<&'static str>) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (sock, _) = listener.accept().await.unwrap();
        let (rd, mut wr) = sock.into_split();
        let mut rd = BufReader::new(rd);
        wr.write_all(format!("* OK [CAPABILITY {CAPS}] ready\r\n").as_bytes())
            .await
            .unwrap();
        let mut rev2 = false;
        loop {
            let mut line = String::new();
            if rd.read_line(&mut line).await.unwrap_or(0) == 0 {
                return;
            }
            let tag = line.split_whitespace().next().unwrap_or("*").to_string();
            let cmd = line.to_ascii_uppercase();
            let reply = if cmd.contains("AUTHENTICATE PLAIN") {
                wr.write_all(b"+ \r\n").await.unwrap();
                let mut payload = String::new();
                rd.read_line(&mut payload).await.unwrap();
                format!("{tag} OK [CAPABILITY {CAPS}] Logged in\r\n")
            } else if cmd.contains(" ENABLE ") {
                rev2 = cmd.contains("IMAP4REV2");
                let enabled = if rev2 { "* ENABLED IMAP4rev2\r\n" } else { "" };
                format!("{enabled}{tag} OK Enabled\r\n")
            } else if cmd.contains(" SELECT ") {
                format!(
                    "* 3 EXISTS\r\n* OK [UIDVALIDITY 1]\r\n* OK [UIDNEXT 13]\r\n\
                     {tag} OK [READ-WRITE] Selected\r\n"
                )
            } else if cmd.contains(" SEARCH ") {
                if !rev2 || !cmd.contains("RETURN (ALL COUNT MIN MAX)") {
                    wr.write_all(
                        format!("{tag} BAD expected enabled rev2 extended search\r\n").as_bytes(),
                    )
                    .await
                    .unwrap();
                    continue;
                }
                if let Some(reply) = search_override {
                    format!("{reply}\r\n{tag} OK Search completed\r\n")
                } else if rev2 {
                    format!(
                        "* ESEARCH (TAG \"{tag}\") UID ALL 10:12\r\n{tag} OK Search completed\r\n"
                    )
                } else {
                    format!("* SEARCH 10 11 12\r\n{tag} OK Search completed\r\n")
                }
            } else if cmd.contains(" FETCH ") {
                format!(
                    "* 1 FETCH (UID 10 FLAGS ())\r\n* 2 FETCH (UID 11 FLAGS ())\r\n* 3 FETCH (UID 12 FLAGS ())\r\n{tag} OK Fetch\r\n"
                )
            } else if cmd.contains(" LOGOUT") {
                format!("* BYE\r\n{tag} OK Logout\r\n")
            } else {
                format!("{tag} BAD unexpected\r\n")
            };
            wr.write_all(reply.as_bytes()).await.unwrap();
        }
    });
    addr
}

#[tokio::test]
async fn uid_window_sync_works_against_a_server_that_advertises_imap4rev2() {
    let addr = spawn_rev2_mock(None).await;
    let config = ImapConfig::new(
        addr.ip().to_string(),
        Credentials::Password {
            username: "alice".into(),
            password: "secret".into(),
        },
    )
    .port(addr.port())
    .tls(TlsMode::Plaintext);
    let backend = ImapBackend::connect(config).await.expect("connect");

    let mbox = RawMailboxRef {
        name: "INBOX".into(),
        uidvalidity: 1,
    };
    let cursor = SyncCursor::UidWindow {
        uidvalidity: 1,
        uidnext: 10,
    };
    let delta = backend
        .sync_mailbox(&mbox, &cursor)
        .await
        .expect("SEARCH-based sync against an IMAP4rev2 server");

    let added: Vec<u32> = delta
        .added
        .iter()
        .map(|r| match r {
            MessageRef::Imap { uid, .. } => *uid,
            _ => panic!("imap ref"),
        })
        .collect();
    assert_eq!(added, vec![10, 11, 12]);
}

#[tokio::test]
async fn esearch_rejects_wrong_correlators_sequence_numbers_and_unbounded_sets() {
    for reply in [
        "* ESEARCH (TAG \"wrong\") UID ALL 10:12",
        "* ESEARCH ALL 10:12",
        "* ESEARCH UID ALL 1:4294967295",
        "* ESEARCH UID ALL broken",
        "* ESEARCH UID COUNT 5",
    ] {
        let addr = spawn_rev2_mock(Some(reply)).await;
        let backend = ImapBackend::connect(
            ImapConfig::new(
                addr.ip().to_string(),
                Credentials::Password {
                    username: "alice".into(),
                    password: "secret".into(),
                },
            )
            .port(addr.port())
            .tls(TlsMode::Plaintext),
        )
        .await
        .unwrap();
        let result = backend
            .sync_mailbox(
                &RawMailboxRef {
                    name: "INBOX".into(),
                    uidvalidity: 1,
                },
                &SyncCursor::UidWindow {
                    uidvalidity: 1,
                    uidnext: 10,
                },
            )
            .await;
        assert!(result.is_err(), "accepted invalid search response: {reply}");
    }
}
