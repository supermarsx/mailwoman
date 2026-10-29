//! Live QUOTA (RFC 9208) check against a real IMAP server (t39-e7, L10).
//!
//! `#[ignore]` by default and a no-op unless `MW_QUOTA_IMAP` (`host:port`,
//! plaintext) is set. Env:
//! - `MW_QUOTA_USER` / `MW_QUOTA_PASS` — the account (default `testuser` /
//!   `testpass`, the Dovecot fixtures' account).
//! - `MW_QUOTA_EXPECT` — `quota` (default: the server sets a STORAGE limit),
//!   `none` (QUOTA advertised, no quota root) or `unsupported` (no QUOTA).
//! - `MW_QUOTA_SETQUOTA` — a raw `SETQUOTA` argument sent first on its own
//!   connection, for servers that start without a limit (GreenMail:
//!   `INBOX (STORAGE 512)`).
//!
//! Dovecot with the quota plugin: `scripts/dovecot-t32` (10 MiB "User quota").

use mw_engine::backend::{AccountBackend, EngineError, Flag, RawMailboxRef};
use mw_imap::{Credentials, ImapBackend, ImapConfig, TlsMode};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

fn env_addr() -> Option<(String, u16)> {
    let raw = std::env::var("MW_QUOTA_IMAP").ok()?;
    let (host, port) = raw.rsplit_once(':')?;
    Some((host.to_string(), port.parse().ok()?))
}

async fn raw_setquota(host: &str, port: u16, user: &str, pass: &str, arg: &str) {
    let stream = tokio::net::TcpStream::connect((host, port)).await.unwrap();
    let (rd, mut wr) = stream.into_split();
    let mut rd = BufReader::new(rd);
    let mut line = String::new();
    rd.read_line(&mut line).await.unwrap();
    for (tag, cmd) in [
        ("s1", format!("LOGIN {user} {pass}")),
        ("s2", format!("SETQUOTA {arg}")),
        ("s3", "LOGOUT".to_string()),
    ] {
        wr.write_all(format!("{tag} {cmd}\r\n").as_bytes())
            .await
            .unwrap();
        loop {
            line.clear();
            if rd.read_line(&mut line).await.unwrap() == 0 {
                return;
            }
            if line.starts_with(&format!("{tag} ")) {
                assert!(line.contains(" OK"), "{cmd}: {line}");
                break;
            }
        }
    }
}

#[tokio::test]
#[ignore = "requires a live IMAP server (set MW_QUOTA_IMAP=host:port)"]
async fn live_getquotaroot_reports_the_server_quota() {
    let Some((host, port)) = env_addr() else {
        eprintln!("MW_QUOTA_IMAP not set; skipping live test");
        return;
    };
    let user = std::env::var("MW_QUOTA_USER").unwrap_or_else(|_| "testuser".into());
    let pass = std::env::var("MW_QUOTA_PASS").unwrap_or_else(|_| "testpass".into());
    let expect = std::env::var("MW_QUOTA_EXPECT").unwrap_or_else(|_| "quota".into());
    if let Ok(arg) = std::env::var("MW_QUOTA_SETQUOTA") {
        raw_setquota(&host, port, &user, &pass, &arg).await;
    }

    let backend = ImapBackend::connect(
        ImapConfig::new(
            host,
            Credentials::Password {
                username: user,
                password: pass,
            },
        )
        .port(port)
        .tls(TlsMode::Plaintext),
    )
    .await
    .expect("connect");

    let first = backend.quota().await;
    eprintln!("GETQUOTAROOT INBOX -> {first:?}");
    match expect.as_str() {
        "unsupported" => {
            assert!(
                matches!(first, Err(EngineError::Unsupported(_))),
                "{first:?}"
            );
        }
        "none" => assert_eq!(first.expect("quota"), vec![]),
        _ => {
            let first = first.expect("quota");
            let storage = |q: &[mw_engine::backend::QuotaUsage]| {
                q.iter()
                    .find(|r| r.resource == "STORAGE")
                    .cloned()
                    .expect("a STORAGE resource")
            };
            let before = storage(&first);
            assert!(before.limit > 0, "{before:?}");
            // Usage follows the mailbox: a 64 KiB append raises it.
            let body = "x".repeat(64 * 1024);
            let raw = format!(
                "From: a@example.org\r\nTo: b@example.org\r\nSubject: quota probe\r\n\r\n{body}\r\n"
            );
            backend
                .append(
                    &RawMailboxRef {
                        name: "INBOX".into(),
                        uidvalidity: 0,
                    },
                    raw.as_bytes(),
                    &[Flag::Seen],
                )
                .await
                .expect("append");
            let after = storage(&backend.quota().await.expect("quota after append"));
            eprintln!("after a 64 KiB append: {after:?}");
            assert!(after.usage > before.usage, "{before:?} -> {after:?}");
            assert_eq!(after.limit, before.limit);
        }
    }
}
