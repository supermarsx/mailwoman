//! Behavioural tests for `mw-imap` driven against an in-crate mock IMAP socket
//! (a `tokio` `TcpListener` replaying scripted server lines). No live server is
//! required, so `cargo test -p mw-imap` is green without infrastructure.
//!
//! The mock echoes each command's tag into `{tag}` in its scripted response and
//! understands the four exchange shapes `mw-imap` uses: a plain tagged command,
//! a SASL `AUTHENTICATE` continuation, an `APPEND` synchronizing literal, and an
//! `IDLE`/`DONE` cycle.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use mw_engine::backend::{
    AccountBackend, ChangeEvent, ChangeSink, Flag, MailboxRole, MessageRef, MoveOutcome,
    RawMailboxRef, SyncCursor,
};
use mw_imap::{Credentials, ImapBackend, ImapConfig, TlsMode};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc;

// --- mock server ------------------------------------------------------------

/// One scripted server-side exchange.
enum Ex {
    /// Read a command line, then send `respond` (with `{tag}` substituted).
    Cmd {
        expect: &'static str,
        respond: String,
    },
    /// SASL `AUTHENTICATE`: read the command, send `+`, read the base64
    /// response line, then send `respond`.
    Auth {
        expect: &'static str,
        respond: String,
    },
    /// `APPEND` literal: read the command (with `{n}`), send `+`, read the
    /// literal payload + CRLF, then send `respond`.
    Literal {
        expect: &'static str,
        respond: String,
    },
    /// `IDLE`: read the command, send `+ idling` and `unsolicited`, read the
    /// `DONE` line, then send `respond`.
    Idle {
        unsolicited: String,
        respond: String,
    },
}

struct Script {
    greeting: String,
    steps: Vec<Ex>,
}

/// Spawn a mock accepting one connection per script; returns the bound address
/// and a shared log of every client line the server read.
async fn spawn_mock(scripts: Vec<Script>) -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let log = Arc::new(Mutex::new(Vec::new()));
    let log2 = log.clone();
    tokio::spawn(async move {
        for script in scripts {
            let (sock, _) = listener.accept().await.unwrap();
            let log = log2.clone();
            tokio::spawn(async move {
                handle_conn(sock, script, log).await;
            });
        }
    });
    (addr, log)
}

async fn read_line(rd: &mut BufReader<OwnedReadHalf>, log: &Arc<Mutex<Vec<String>>>) -> String {
    let mut line = String::new();
    let n = rd.read_line(&mut line).await.unwrap();
    if n > 0 {
        log.lock().unwrap().push(line.clone());
    }
    line
}

fn tag_of(line: &str) -> Option<String> {
    let tok = line.split_whitespace().next()?;
    // A command tag looks like `A0001`; a continuation payload has none.
    if tok.starts_with('A') && tok.len() > 1 && tok[1..].chars().all(|c| c.is_ascii_alphanumeric())
    {
        Some(tok.to_string())
    } else {
        None
    }
}

fn literal_len(line: &str) -> Option<usize> {
    let open = line.rfind('{')?;
    let close = line[open..].find('}')? + open;
    line[open + 1..close].trim_end_matches('+').parse().ok()
}

async fn handle_conn(sock: TcpStream, script: Script, log: Arc<Mutex<Vec<String>>>) {
    let (rd, mut wr) = sock.into_split();
    let mut rd = BufReader::new(rd);
    wr.write_all(script.greeting.as_bytes()).await.unwrap();
    let mut last_tag = "*".to_string();

    for step in script.steps {
        match step {
            Ex::Cmd { expect, respond } => {
                let line = read_line(&mut rd, &log).await;
                if line.is_empty() {
                    return;
                }
                if let Some(t) = tag_of(&line) {
                    last_tag = t;
                }
                assert!(line.contains(expect), "expected {expect:?} in {line:?}");
                wr.write_all(respond.replace("{tag}", &last_tag).as_bytes())
                    .await
                    .unwrap();
            }
            Ex::Auth { expect, respond } => {
                let line = read_line(&mut rd, &log).await;
                if let Some(t) = tag_of(&line) {
                    last_tag = t;
                }
                assert!(line.contains(expect), "expected {expect:?} in {line:?}");
                wr.write_all(b"+ \r\n").await.unwrap();
                let _payload = read_line(&mut rd, &log).await; // base64 initial response
                wr.write_all(respond.replace("{tag}", &last_tag).as_bytes())
                    .await
                    .unwrap();
            }
            Ex::Literal { expect, respond } => {
                let line = read_line(&mut rd, &log).await;
                if let Some(t) = tag_of(&line) {
                    last_tag = t;
                }
                assert!(line.contains(expect), "expected {expect:?} in {line:?}");
                let n = literal_len(&line).expect("APPEND literal length");
                wr.write_all(b"+ ready\r\n").await.unwrap();
                let mut buf = vec![0u8; n];
                rd.read_exact(&mut buf).await.unwrap();
                log.lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&buf).into_owned());
                let mut crlf = [0u8; 2];
                let _ = rd.read_exact(&mut crlf).await;
                wr.write_all(respond.replace("{tag}", &last_tag).as_bytes())
                    .await
                    .unwrap();
            }
            Ex::Idle {
                unsolicited,
                respond,
            } => {
                let line = read_line(&mut rd, &log).await;
                if let Some(t) = tag_of(&line) {
                    last_tag = t;
                }
                assert!(line.contains("IDLE"), "expected IDLE in {line:?}");
                wr.write_all(b"+ idling\r\n").await.unwrap();
                if !unsolicited.is_empty() {
                    wr.write_all(unsolicited.as_bytes()).await.unwrap();
                }
                let _done = read_line(&mut rd, &log).await; // DONE
                wr.write_all(respond.replace("{tag}", &last_tag).as_bytes())
                    .await
                    .unwrap();
            }
        }
    }
}

// --- script builders --------------------------------------------------------

const DOVECOT_CAPS: &str = "IMAP4rev1 IMAP4rev2 AUTH=PLAIN AUTH=LOGIN AUTH=XOAUTH2 SASL-IR ID ENABLE IDLE MOVE UIDPLUS CONDSTORE QRESYNC ESEARCH LIST-STATUS SPECIAL-USE";

fn dovecot_greeting() -> String {
    format!("* OK [CAPABILITY {DOVECOT_CAPS}] Dovecot ready.\r\n")
}

/// The AUTHENTICATE + ID + ENABLE steps a full-featured connect performs.
fn dovecot_connect_steps() -> Vec<Ex> {
    vec![
        Ex::Auth {
            expect: "AUTHENTICATE PLAIN",
            respond: format!("{{tag}} OK [CAPABILITY {DOVECOT_CAPS}] Logged in\r\n"),
        },
        Ex::Cmd {
            expect: "ID",
            respond: "* ID (\"name\" \"Dovecot\")\r\n{tag} OK ID completed\r\n".into(),
        },
        Ex::Cmd {
            expect: "ENABLE",
            respond: "* ENABLED QRESYNC IMAP4rev2\r\n{tag} OK Enabled\r\n".into(),
        },
    ]
}

fn password_creds() -> Credentials {
    Credentials::Password {
        username: "alice".into(),
        password: "secret".into(),
    }
}

fn config(addr: SocketAddr, creds: Credentials) -> ImapConfig {
    ImapConfig::new(addr.ip().to_string(), creds)
        .port(addr.port())
        .tls(TlsMode::Plaintext)
}

async fn connect_dovecot(addr: SocketAddr, extra: Vec<Ex>) -> ImapBackend {
    // Caller must have spawned a mock whose first script is greeting + connect + extra.
    let _ = extra;
    ImapBackend::connect(config(addr, password_creds()))
        .await
        .expect("connect")
}

fn dovecot_script(extra: Vec<Ex>) -> Script {
    let mut steps = dovecot_connect_steps();
    steps.extend(extra);
    Script {
        greeting: dovecot_greeting(),
        steps,
    }
}

// --- tests ------------------------------------------------------------------

#[tokio::test]
async fn connect_detects_capabilities() {
    let (addr, _log) = spawn_mock(vec![dovecot_script(vec![])]).await;
    let backend = connect_dovecot(addr, vec![]).await;
    let caps = backend.capabilities().await.unwrap();
    assert!(caps.imap4rev2);
    assert!(caps.qresync);
    assert!(caps.condstore);
    assert!(caps.uidplus);
    assert!(caps.r#move);
    assert!(caps.special_use);
    assert!(caps.list_status);
    assert!(caps.idle);
    assert!(caps.esearch);
    assert!(caps.enable);
    assert!(caps.id);
    assert!(caps.sasl_plain);
    assert!(caps.sasl_xoauth2);
    assert!(!caps.compress);
}

#[tokio::test]
async fn list_mailboxes_maps_special_use_roles() {
    let list = "\
* LIST (\\HasChildren) \"/\" \"INBOX\"\r\n\
* LIST (\\HasNoChildren \\Sent) \"/\" \"Sent\"\r\n\
* LIST (\\HasNoChildren \\Trash) \"/\" \"Trash\"\r\n\
* LIST (\\HasNoChildren \\Junk) \"/\" \"Spam\"\r\n\
* LIST (\\HasNoChildren \\Drafts) \"/\" \"Drafts\"\r\n\
* LIST (\\HasNoChildren \\Archive) \"/\" \"Archive\"\r\n\
* STATUS \"INBOX\" (MESSAGES 3 UNSEEN 1 UIDNEXT 12 UIDVALIDITY 99 HIGHESTMODSEQ 50)\r\n\
* STATUS \"Sent\" (MESSAGES 2 UNSEEN 0 UIDNEXT 5 UIDVALIDITY 99 HIGHESTMODSEQ 40)\r\n\
{tag} OK List completed\r\n";
    let steps = vec![Ex::Cmd {
        expect: "LIST",
        respond: list.into(),
    }];
    let (addr, _log) = spawn_mock(vec![dovecot_script(steps)]).await;
    let backend = connect_dovecot(addr, vec![]).await;

    let boxes = backend.list_mailboxes().await.unwrap();
    let role = |name: &str| {
        boxes
            .iter()
            .find(|m| m.mailbox_ref.name == name)
            .map(|m| m.role)
    };
    assert_eq!(role("INBOX"), Some(MailboxRole::Inbox));
    assert_eq!(role("Sent"), Some(MailboxRole::Sent));
    assert_eq!(role("Trash"), Some(MailboxRole::Trash));
    assert_eq!(role("Spam"), Some(MailboxRole::Junk));
    assert_eq!(role("Drafts"), Some(MailboxRole::Drafts));
    assert_eq!(role("Archive"), Some(MailboxRole::Archive));

    let inbox = boxes
        .iter()
        .find(|m| m.mailbox_ref.name == "INBOX")
        .unwrap();
    assert_eq!(inbox.total, 3);
    assert_eq!(inbox.unread, 1);
    assert_eq!(inbox.uidnext, 12);
    assert_eq!(inbox.mailbox_ref.uidvalidity, 99);
    assert_eq!(inbox.highestmodseq, 50);
}

#[tokio::test]
async fn sync_qresync_reports_vanished_and_flag_changes() {
    let select = "\
* 3 EXISTS\r\n\
* OK [UIDVALIDITY 99]\r\n\
* OK [UIDNEXT 50]\r\n\
* OK [HIGHESTMODSEQ 320]\r\n\
* VANISHED (EARLIER) 41,43\r\n\
* 3 FETCH (UID 45 FLAGS (\\Seen) MODSEQ (318))\r\n\
{tag} OK [READ-WRITE] Select completed\r\n";
    let steps = vec![Ex::Cmd {
        expect: "QRESYNC",
        respond: select.into(),
    }];
    let (addr, _log) = spawn_mock(vec![dovecot_script(steps)]).await;
    let backend = connect_dovecot(addr, vec![]).await;

    let mbox = RawMailboxRef {
        name: "INBOX".into(),
        uidvalidity: 99,
    };
    let cursor = SyncCursor::Qresync {
        uidvalidity: 99,
        highestmodseq: 100,
    };
    let delta = backend.sync_mailbox(&mbox, &cursor).await.unwrap();

    let removed_uids: Vec<u32> = delta
        .removed
        .iter()
        .map(|r| match r {
            MessageRef::Imap { uid, .. } => *uid,
            _ => panic!("imap ref"),
        })
        .collect();
    assert_eq!(removed_uids, vec![41, 43]);
    assert_eq!(delta.flag_changes.len(), 1);
    let (mref, flags) = &delta.flag_changes[0];
    assert!(matches!(mref, MessageRef::Imap { uid: 45, .. }));
    assert_eq!(flags, &vec![Flag::Seen]);
    assert_eq!(
        delta.next_cursor,
        SyncCursor::Qresync {
            uidvalidity: 99,
            highestmodseq: 320
        }
    );
}

#[tokio::test]
async fn sync_condstore_reports_flag_changes() {
    let select = "\
* 3 EXISTS\r\n\
* OK [UIDVALIDITY 99]\r\n\
* OK [HIGHESTMODSEQ 400]\r\n\
{tag} OK [READ-WRITE] Select completed\r\n";
    let fetch = "\
* 2 FETCH (UID 20 FLAGS (\\Flagged) MODSEQ (390))\r\n\
* 3 FETCH (UID 21 FLAGS () MODSEQ (395))\r\n\
{tag} OK Fetch completed\r\n";
    let steps = vec![
        Ex::Cmd {
            expect: "CONDSTORE",
            respond: select.into(),
        },
        Ex::Cmd {
            expect: "SEARCH",
            respond: "* SEARCH 20 21\r\n{tag} OK Search\r\n".into(),
        },
        Ex::Cmd {
            expect: "UID FETCH 1:* (UID FLAGS)",
            respond: fetch.into(),
        },
    ];
    let (addr, _log) = spawn_mock(vec![dovecot_script(steps)]).await;
    let backend = connect_dovecot(addr, vec![]).await;

    let mbox = RawMailboxRef {
        name: "INBOX".into(),
        uidvalidity: 99,
    };
    let cursor = SyncCursor::Condstore {
        uidvalidity: 99,
        modseq: 50,
    };
    let delta = backend.sync_mailbox(&mbox, &cursor).await.unwrap();

    assert!(delta.removed.is_empty());
    assert_eq!(delta.flag_changes.len(), 2);
    assert_eq!(delta.flag_changes[0].1, vec![Flag::Flagged]);
    assert_eq!(delta.flag_changes[1].1, Vec::<Flag>::new());
    assert_eq!(
        delta.next_cursor,
        SyncCursor::ImapSnapshot {
            uidvalidity: 99,
            uidnext: 0,
            known_uids: [20, 21].into_iter().collect(),
            highestmodseq: Some(400),
        }
    );
}

#[tokio::test]
async fn sync_uid_window_reports_added_when_no_condstore() {
    // Minimal server: no CONDSTORE/QRESYNC, so the ladder degrades to UID-window.
    let greeting = "* OK [CAPABILITY IMAP4rev1 AUTH=PLAIN] ready\r\n".to_string();
    let select = "\
* 3 EXISTS\r\n\
* OK [UIDVALIDITY 1]\r\n\
* OK [UIDNEXT 13]\r\n\
{tag} OK [READ-WRITE] Selected\r\n";
    let search = "* SEARCH 10 11 12\r\n{tag} OK Search completed\r\n";
    let steps = vec![
        Ex::Auth {
            expect: "AUTHENTICATE PLAIN",
            respond: "{tag} OK [CAPABILITY IMAP4rev1 AUTH=PLAIN] Logged in\r\n".into(),
        },
        Ex::Cmd {
            expect: "SELECT",
            respond: select.into(),
        },
        Ex::Cmd {
            expect: "SEARCH",
            respond: search.into(),
        },
        Ex::Cmd {
            expect: "UID FETCH 1:* (UID FLAGS)",
            respond: "* 1 FETCH (UID 10 FLAGS ())\r\n* 2 FETCH (UID 11 FLAGS ())\r\n* 3 FETCH (UID 12 FLAGS ())\r\n{tag} OK Fetch\r\n".into(),
        },
    ];
    let (addr, _log) = spawn_mock(vec![Script { greeting, steps }]).await;
    let backend = ImapBackend::connect(config(addr, password_creds()))
        .await
        .unwrap();

    let mbox = RawMailboxRef {
        name: "INBOX".into(),
        uidvalidity: 1,
    };
    let cursor = SyncCursor::UidWindow {
        uidvalidity: 1,
        uidnext: 10,
    };
    let delta = backend.sync_mailbox(&mbox, &cursor).await.unwrap();

    let added: Vec<u32> = delta
        .added
        .iter()
        .map(|r| match r {
            MessageRef::Imap { uid, .. } => *uid,
            _ => panic!("imap ref"),
        })
        .collect();
    assert_eq!(added, vec![10, 11, 12]);
    assert_eq!(
        delta.next_cursor,
        SyncCursor::ImapSnapshot {
            uidvalidity: 1,
            uidnext: 13,
            known_uids: [10, 11, 12].into_iter().collect(),
            highestmodseq: None,
        }
    );
}

#[tokio::test]
async fn move_with_uidplus_returns_uidplus_outcome() {
    let steps = vec![
        Ex::Cmd {
            expect: "SELECT",
            respond: "{tag} OK [READ-WRITE] Selected\r\n".into(),
        },
        Ex::Cmd {
            expect: "UID MOVE",
            respond: "{tag} OK [COPYUID 42 5:6 100:101] Move completed\r\n".into(),
        },
    ];
    let (addr, _log) = spawn_mock(vec![dovecot_script(steps)]).await;
    let backend = connect_dovecot(addr, vec![]).await;

    let refs = vec![
        MessageRef::Imap {
            mailbox: RawMailboxRef {
                name: "INBOX".into(),
                uidvalidity: 42,
            },
            uidvalidity: 42,
            uid: 5,
        },
        MessageRef::Imap {
            mailbox: RawMailboxRef {
                name: "INBOX".into(),
                uidvalidity: 42,
            },
            uidvalidity: 42,
            uid: 6,
        },
    ];
    let dest = RawMailboxRef {
        name: "Archive".into(),
        uidvalidity: 42,
    };
    let outcome = backend.move_messages(&refs, &dest).await.unwrap();
    assert_eq!(
        outcome,
        MoveOutcome::Uidplus {
            uidvalidity: 42,
            uids: vec![100, 101]
        }
    );
}

#[tokio::test]
async fn move_without_uidplus_returns_rederive() {
    let greeting = "* OK [CAPABILITY IMAP4rev1 AUTH=PLAIN] ready\r\n".to_string();
    let steps = vec![
        Ex::Auth {
            expect: "AUTHENTICATE PLAIN",
            respond: "{tag} OK [CAPABILITY IMAP4rev1 AUTH=PLAIN] Logged in\r\n".into(),
        },
        Ex::Cmd {
            expect: "SELECT",
            respond: "{tag} OK [READ-WRITE] Selected\r\n".into(),
        },
        Ex::Cmd {
            expect: "UID COPY",
            respond: "{tag} OK Copy completed\r\n".into(),
        },
        Ex::Cmd {
            expect: "UID STORE",
            respond: "{tag} OK Store completed\r\n".into(),
        },
        Ex::Cmd {
            expect: "EXPUNGE",
            respond: "* 1 EXPUNGE\r\n{tag} OK Expunge completed\r\n".into(),
        },
    ];
    let (addr, _log) = spawn_mock(vec![Script { greeting, steps }]).await;
    let backend = ImapBackend::connect(config(addr, password_creds()))
        .await
        .unwrap();

    let refs = vec![MessageRef::Imap {
        mailbox: RawMailboxRef {
            name: "INBOX".into(),
            uidvalidity: 1,
        },
        uidvalidity: 1,
        uid: 5,
    }];
    let dest = RawMailboxRef {
        name: "Archive".into(),
        uidvalidity: 1,
    };
    let outcome = backend.move_messages(&refs, &dest).await.unwrap();
    assert_eq!(outcome, MoveOutcome::RederiveByMessageId);
}

#[tokio::test]
async fn xoauth2_login_sends_correct_sasl_frame() {
    let greeting = "* OK [CAPABILITY IMAP4rev1 AUTH=XOAUTH2] ready\r\n".to_string();
    let steps = vec![Ex::Auth {
        expect: "AUTHENTICATE XOAUTH2",
        respond: "{tag} OK [CAPABILITY IMAP4rev1 AUTH=XOAUTH2] Logged in\r\n".into(),
    }];
    let (addr, log) = spawn_mock(vec![Script { greeting, steps }]).await;

    let creds = Credentials::XOAuth2 {
        username: "u@x".into(),
        token: "TOK".into(),
    };
    let backend = ImapBackend::connect(config(addr, creds)).await.unwrap();
    let _ = backend.capabilities().await.unwrap();

    let expected = mw_imap::sasl::xoauth2("u@x", "TOK");
    let lines = log.lock().unwrap();
    assert!(
        lines.iter().any(|l| l.trim_end() == expected),
        "expected the XOAUTH2 base64 frame {expected:?} in {lines:?}"
    );
}

#[tokio::test]
async fn fetch_raw_returns_body_bytes() {
    let body = "From: a@b\r\nSubject: hi\r\n\r\nbody text\r\n";
    let fetch = format!(
        "* 1 FETCH (UID 5 FLAGS (\\Seen) INTERNALDATE \"01-Jan-2026 10:00:00 +0000\" RFC822.SIZE {} BODY[] {{{}}}\r\n{}){}",
        body.len(),
        body.len(),
        body,
        "\r\n{tag} OK Fetch completed\r\n"
    );
    let steps = vec![
        Ex::Cmd {
            expect: "SELECT",
            respond: "{tag} OK [READ-WRITE] Selected\r\n".into(),
        },
        Ex::Cmd {
            expect: "BODY.PEEK",
            respond: fetch,
        },
    ];
    let (addr, _log) = spawn_mock(vec![dovecot_script(steps)]).await;
    let backend = connect_dovecot(addr, vec![]).await;

    let refs = vec![MessageRef::Imap {
        mailbox: RawMailboxRef {
            name: "INBOX".into(),
            uidvalidity: 99,
        },
        uidvalidity: 99,
        uid: 5,
    }];
    let msgs = backend.fetch_raw(&refs).await.unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].raw, body.as_bytes());
    assert_eq!(msgs[0].flags, vec![Flag::Seen]);
    assert_eq!(
        msgs[0].internaldate.as_deref(),
        Some("01-Jan-2026 10:00:00 +0000")
    );
}

#[tokio::test]
async fn append_captures_appenduid() {
    let steps = vec![Ex::Literal {
        expect: "APPEND",
        respond: "{tag} OK [APPENDUID 55 9] Append completed\r\n".into(),
    }];
    let (addr, _log) = spawn_mock(vec![dovecot_script(steps)]).await;
    let backend = connect_dovecot(addr, vec![]).await;

    let dest = RawMailboxRef {
        name: "Sent".into(),
        uidvalidity: 55,
    };
    let raw = b"From: a@b\r\nSubject: sent\r\n\r\nhello\r\n";
    let mref = backend.append(&dest, raw, &[Flag::Seen]).await.unwrap();
    match mref {
        MessageRef::Imap {
            uidvalidity, uid, ..
        } => {
            assert_eq!(uidvalidity, 55);
            assert_eq!(uid, 9);
        }
        _ => panic!("expected imap ref"),
    }
}

#[tokio::test]
async fn append_refuses_server_no_instead_of_reporting_a_fake_uid() {
    let steps = vec![Ex::Literal {
        expect: "APPEND",
        respond: "{tag} NO [OVERQUOTA] mailbox is full\r\n".into(),
    }];
    let (addr, _) = spawn_mock(vec![dovecot_script(steps)]).await;
    let backend = connect_dovecot(addr, vec![]).await;
    let dest = RawMailboxRef {
        name: "INBOX".into(),
        uidvalidity: 55,
    };
    assert!(
        backend
            .append(&dest, b"Subject: test\r\n\r\nbody", &[])
            .await
            .is_err()
    );
}

#[tokio::test]
async fn store_flags_issues_uid_store() {
    let steps = vec![
        Ex::Cmd {
            expect: "SELECT",
            respond: "{tag} OK [READ-WRITE] Selected\r\n".into(),
        },
        Ex::Cmd {
            expect: "UID STORE",
            respond: "{tag} OK Store completed\r\n".into(),
        },
    ];
    let (addr, log) = spawn_mock(vec![dovecot_script(steps)]).await;
    let backend = connect_dovecot(addr, vec![]).await;

    let refs = vec![MessageRef::Imap {
        mailbox: RawMailboxRef {
            name: "INBOX".into(),
            uidvalidity: 99,
        },
        uidvalidity: 99,
        uid: 5,
    }];
    backend
        .store_flags(&refs, &[Flag::Seen], &[])
        .await
        .unwrap();
    let lines = log.lock().unwrap();
    assert!(
        lines
            .iter()
            .any(|l| l.contains("UID STORE 5 +FLAGS.SILENT (\\Seen)")),
        "expected a +FLAGS STORE in {lines:?}"
    );
}

#[tokio::test]
async fn watch_emits_mailbox_changed_on_idle_activity() {
    // Two connections: the command connection (connect) and the watch connection.
    let cmd_script = dovecot_script(vec![]);
    let watch_steps = {
        let mut s = dovecot_connect_steps();
        s.push(Ex::Cmd {
            expect: "SELECT",
            respond: "{tag} OK [READ-WRITE] Selected\r\n".into(),
        });
        s.push(Ex::Idle {
            unsolicited: "* 5 EXISTS\r\n".into(),
            respond: "{tag} OK IDLE terminated\r\n".into(),
        });
        // A second IDLE the loop enters after emitting; ended by the stop signal.
        s.push(Ex::Idle {
            unsolicited: String::new(),
            respond: "{tag} OK IDLE terminated\r\n".into(),
        });
        s.push(Ex::Cmd {
            expect: "LOGOUT",
            respond: "* BYE bye\r\n{tag} OK Logout\r\n".into(),
        });
        s
    };
    let watch_script = Script {
        greeting: dovecot_greeting(),
        steps: watch_steps,
    };
    let (addr, _log) = spawn_mock(vec![cmd_script, watch_script]).await;
    let backend = connect_dovecot(addr, vec![]).await;

    let (tx, mut rx) = mpsc::unbounded_channel();
    let handle = backend.watch(ChangeSink::new(tx)).await.unwrap();

    let event = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
        .await
        .expect("watch event within timeout")
        .expect("channel open");
    assert!(matches!(event, ChangeEvent::MailboxChanged { .. }));
    handle.stop();
}

#[tokio::test]
async fn recorded_fixtures_parse_without_panic() {
    // Every recorded transcript in fixtures/imap/ must feed the parser wrapper
    // without panicking (the fuzz invariant, exercised over real transcripts).
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/imap");
    let mut count = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("txt") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let wire = text.replace("\r\n", "\n").replace('\n', "\r\n");
        mw_imap::fuzz_parse_responses(wire.as_bytes());
        count += 1;
    }
    assert!(count > 0, "expected recorded fixtures in {dir:?}");
}

// --- dial seam (t32-ext §J) -------------------------------------------------

/// A dialer that sends every connection to one loopback address and records
/// the `(host, port)` it was asked for, standing in for an SSH tunnel.
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

#[tokio::test]
async fn connect_via_a_dialer_runs_both_connections_through_it() {
    let cmd_script = dovecot_script(vec![]);
    let watch_steps = {
        let mut s = dovecot_connect_steps();
        s.push(Ex::Cmd {
            expect: "SELECT",
            respond: "{tag} OK [READ-WRITE] Selected\r\n".into(),
        });
        s.push(Ex::Idle {
            unsolicited: "* 5 EXISTS\r\n".into(),
            respond: "{tag} OK IDLE terminated\r\n".into(),
        });
        s.push(Ex::Idle {
            unsolicited: String::new(),
            respond: "{tag} OK IDLE terminated\r\n".into(),
        });
        s
    };
    let (addr, _log) = spawn_mock(vec![
        cmd_script,
        Script {
            greeting: dovecot_greeting(),
            steps: watch_steps,
        },
    ])
    .await;
    let dialer = Arc::new(RedirectDialer {
        to: addr,
        asked: Mutex::new(Vec::new()),
    });
    // The configured name does not resolve; only the dialer can reach the server.
    let config = ImapConfig::new("imap.tunnel.invalid", password_creds())
        .port(143)
        .tls(TlsMode::Plaintext);
    let backend = ImapBackend::connect_via(config, Some(dialer.clone()))
        .await
        .expect("connect through the dialer");
    assert!(backend.capabilities().await.unwrap().imap4rev2);

    let (tx, mut rx) = mpsc::unbounded_channel();
    let handle = backend.watch(ChangeSink::new(tx)).await.unwrap();
    let event = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
        .await
        .expect("watch event within timeout")
        .expect("channel open");
    assert!(matches!(event, ChangeEvent::MailboxChanged { .. }));
    handle.stop();

    assert_eq!(
        *dialer.asked.lock().unwrap(),
        vec![
            ("imap.tunnel.invalid".to_string(), 143),
            ("imap.tunnel.invalid".to_string(), 143)
        ],
        "the command and the IDLE connection both went through the dialer"
    );
}

/// A dialer that refuses every target, as a tunnel refuses one outside its set.
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
async fn a_dialer_refusal_fails_the_connect_without_a_direct_fallback() {
    let (addr, log) = spawn_mock(vec![dovecot_script(vec![])]).await;
    // The configured address is reachable directly; the refusal must still win.
    let err = ImapBackend::connect_via(
        config(addr, password_creds()),
        Some(Arc::new(RefusingDialer)),
    )
    .await
    .err()
    .expect("refused");
    assert!(err.to_string().contains("target not allowed"), "{err}");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert!(
        log.lock().unwrap().is_empty(),
        "no direct connection was made"
    );
}

#[tokio::test]
async fn first_sync_promotes_zero_uidvalidity_to_qresync() {
    let steps = vec![
        Ex::Cmd { expect: "SELECT", respond: "* OK [UIDVALIDITY 99]\r\n* OK [UIDNEXT 12]\r\n* OK [HIGHESTMODSEQ 50]\r\n{tag} OK Selected\r\n".into() },
        Ex::Cmd { expect: "SEARCH", respond: "* SEARCH 10 11\r\n{tag} OK Search\r\n".into() },
        Ex::Cmd { expect: "QRESYNC (99 50)", respond: "* OK [UIDVALIDITY 99]\r\n* OK [HIGHESTMODSEQ 51]\r\n* VANISHED (EARLIER) 10\r\n* 1 FETCH (UID 11 FLAGS (\\Seen) MODSEQ (51))\r\n{tag} OK Selected\r\n".into() },
    ];
    let (addr, _) = spawn_mock(vec![dovecot_script(steps)]).await;
    let backend = connect_dovecot(addr, vec![]).await;
    let mailbox = RawMailboxRef {
        name: "INBOX".into(),
        uidvalidity: 0,
    };
    let first = backend
        .sync_mailbox(
            &mailbox,
            &SyncCursor::UidWindow {
                uidvalidity: 0,
                uidnext: 1,
            },
        )
        .await
        .unwrap();
    assert_eq!(
        first.next_cursor,
        SyncCursor::Qresync {
            uidvalidity: 99,
            highestmodseq: 50
        }
    );
    assert_eq!(first.added.len(), 2);
    let second = backend
        .sync_mailbox(&mailbox, &first.next_cursor)
        .await
        .unwrap();
    assert!(matches!(
        second.removed.as_slice(),
        [MessageRef::Imap { uid: 10, .. }]
    ));
    assert_eq!(second.flag_changes[0].1, vec![Flag::Seen]);
}

#[tokio::test]
async fn persisted_snapshot_reports_flags_and_expunges_after_reconnect() {
    let greeting = "* OK [CAPABILITY IMAP4rev1 AUTH=PLAIN] ready\r\n".to_string();
    let steps = vec![
        Ex::Auth { expect: "AUTHENTICATE PLAIN", respond: "{tag} OK [CAPABILITY IMAP4rev1 AUTH=PLAIN] Logged in\r\n".into() },
        Ex::Cmd { expect: "SELECT", respond: "* OK [UIDVALIDITY 1]\r\n* OK [UIDNEXT 14]\r\n{tag} OK Selected\r\n".into() },
        Ex::Cmd { expect: "UID SEARCH ALL", respond: "* SEARCH 11 13\r\n{tag} OK Search\r\n".into() },
        Ex::Cmd { expect: "UID FETCH 1:* (UID FLAGS)", respond: "* 1 FETCH (UID 11 FLAGS (\\Seen))\r\n* 2 FETCH (UID 13 FLAGS ())\r\n{tag} OK Fetch\r\n".into() },
    ];
    let (addr, _) = spawn_mock(vec![Script { greeting, steps }]).await;
    let backend = ImapBackend::connect(config(addr, password_creds()))
        .await
        .unwrap();
    let persisted = mw_engine::mapping::cursor_to_json(&SyncCursor::ImapSnapshot {
        uidvalidity: 1,
        uidnext: 13,
        known_uids: [10, 11, 12].into_iter().collect(),
        highestmodseq: None,
    });
    let cursor = mw_engine::mapping::cursor_from_json(&persisted).unwrap();
    let delta = backend
        .sync_mailbox(
            &RawMailboxRef {
                name: "INBOX".into(),
                uidvalidity: 1,
            },
            &cursor,
        )
        .await
        .unwrap();
    assert!(matches!(
        delta.added.as_slice(),
        [MessageRef::Imap { uid: 13, .. }]
    ));
    assert_eq!(delta.removed.len(), 2);
    assert!(matches!(delta.removed[0], MessageRef::Imap { uid: 10, .. }));
    assert!(matches!(delta.removed[1], MessageRef::Imap { uid: 12, .. }));
    assert_eq!(delta.flag_changes.len(), 1);
    assert!(matches!(
        delta.flag_changes[0].0,
        MessageRef::Imap { uid: 11, .. }
    ));
    assert_eq!(delta.flag_changes[0].1, vec![Flag::Seen]);
}

#[tokio::test]
async fn condstore_snapshot_reports_expunges_and_uses_changed_since_after_reconnect() {
    let caps = "IMAP4rev1 AUTH=PLAIN CONDSTORE";
    let greeting = format!("* OK [CAPABILITY {caps}] ready\r\n");
    let steps = vec![
        Ex::Auth { expect: "AUTHENTICATE PLAIN", respond: format!("{{tag}} OK [CAPABILITY {caps}] Logged in\r\n") },
        Ex::Cmd { expect: "CONDSTORE", respond: "* OK [UIDVALIDITY 1]\r\n* OK [UIDNEXT 13]\r\n* OK [HIGHESTMODSEQ 51]\r\n{tag} OK Selected\r\n".into() },
        Ex::Cmd { expect: "UID SEARCH ALL", respond: "* SEARCH 11\r\n{tag} OK Search\r\n".into() },
        Ex::Cmd { expect: "CHANGEDSINCE 50", respond: "* 1 FETCH (UID 11 FLAGS (\\Seen) MODSEQ (51))\r\n{tag} OK Fetch\r\n".into() },
    ];
    let (addr, _) = spawn_mock(vec![Script { greeting, steps }]).await;
    let backend = ImapBackend::connect(config(addr, password_creds()))
        .await
        .unwrap();
    let cursor = SyncCursor::ImapSnapshot {
        uidvalidity: 1,
        uidnext: 13,
        known_uids: [10, 11, 12].into_iter().collect(),
        highestmodseq: Some(50),
    };
    let delta = backend
        .sync_mailbox(
            &RawMailboxRef {
                name: "INBOX".into(),
                uidvalidity: 1,
            },
            &cursor,
        )
        .await
        .unwrap();
    assert_eq!(delta.removed.len(), 2);
    assert_eq!(delta.flag_changes.len(), 1);
    assert_eq!(delta.flag_changes[0].1, vec![Flag::Seen]);
    assert!(matches!(
        delta.next_cursor,
        SyncCursor::ImapSnapshot {
            highestmodseq: Some(51),
            ..
        }
    ));
}

#[tokio::test]
async fn dated_import_sends_internaldate_and_requires_acknowledged_coordinates() {
    let (addr, log) = spawn_mock(vec![dovecot_script(vec![Ex::Literal {
        expect: "APPEND",
        respond: "{tag} OK [APPENDUID 55 9] saved\r\n".into(),
    }])])
    .await;
    let backend = connect_dovecot(addr, vec![]).await;
    let target = RawMailboxRef {
        name: "Archive".into(),
        uidvalidity: 55,
    };
    let result = backend
        .append_with_date(
            &target,
            b"Subject: historical\r\n\r\nbody",
            &[Flag::Seen],
            "2020-02-03T04:05:06Z",
        )
        .await
        .unwrap();
    assert!(matches!(
        result,
        MessageRef::Imap {
            uid: 9,
            uidvalidity: 55,
            ..
        }
    ));
    assert!(log.lock().unwrap().iter().any(|line|line.contains("APPEND \"Archive\" (\\Seen) \"03-Feb-2020 04:05:06 +0000\"")));
    for response in [
        "{tag} NO [OVERQUOTA] full\r\n",
        "{tag} OK saved without UID\r\n",
    ] {
        let (addr, _) = spawn_mock(vec![dovecot_script(vec![Ex::Literal {
            expect: "APPEND",
            respond: response.into(),
        }])])
        .await;
        let backend = connect_dovecot(addr, vec![]).await;
        assert!(
            backend
                .append_with_date(
                    &target,
                    b"Subject: old\r\n\r\nbody",
                    &[],
                    "2020-02-03T04:05:06Z"
                )
                .await
                .is_err()
        );
    }
}

#[tokio::test]
async fn dated_import_without_uidplus_or_with_bad_date_never_sends_append() {
    let caps = "IMAP4rev1 AUTH=PLAIN";
    let (addr, log) = spawn_mock(vec![Script {
        greeting: format!("* OK [CAPABILITY {caps}] ready\r\n"),
        steps: vec![Ex::Auth {
            expect: "AUTHENTICATE PLAIN",
            respond: format!("{{tag}} OK [CAPABILITY {caps}] logged in\r\n"),
        }],
    }])
    .await;
    let backend = connect_dovecot(addr, vec![]).await;
    let target = RawMailboxRef {
        name: "INBOX".into(),
        uidvalidity: 55,
    };
    assert!(
        backend
            .append_with_date(
                &target,
                b"Subject: test\r\n\r\nbody",
                &[],
                "2020-02-03T04:05:06Z"
            )
            .await
            .is_err()
    );
    assert!(
        !log.lock()
            .unwrap()
            .iter()
            .any(|line| line.contains("APPEND"))
    );
    let (addr, log) = spawn_mock(vec![dovecot_script(vec![])]).await;
    let backend = connect_dovecot(addr, vec![]).await;
    assert!(
        backend
            .append_with_date(&target, b"Subject: test\r\n\r\nbody", &[], "2020\r\nLOGOUT")
            .await
            .is_err()
    );
    assert!(
        !log.lock()
            .unwrap()
            .iter()
            .any(|line| line.contains("APPEND"))
    );
}
