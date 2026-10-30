//! t92 — folder names outside ASCII against a real Dovecot 2.4, with and
//! without UTF-8 names.
//!
//! `#[ignore]` by default and a no-op unless the address is set:
//! - `MW_T92_IMAP` (`host:port`, plaintext): a Dovecot that advertises
//!   IMAP4rev2 and UTF8=ACCEPT (`scripts/dovecot-t32`, service `full`, 38610).
//!   It sends raw UTF-8 names in quoted strings (`* LIST () "/" "Корзина"`),
//!   which was a protocol error at sign-in before t92 (t39-e7).
//! - `MW_T92_IMAP_7BIT`: the same server without both (service `no-utf8`,
//!   38615), so names travel in modified UTF-7 (`Entw&APw-rfe`).
//!
//! Account `testuser` / `testpass` (override with `MW_T92_USER` / `MW_T92_PASS`).
//! The seed creates `Entwürfe`, `Корзина` and `Archive/2026`.
//!
//! Each case lists, appends to a non-ASCII folder and reads its STATUS count
//! back, then creates, renames and deletes a non-ASCII folder, reading the
//! server's own LIST after every step.

use mw_engine::backend::{AccountBackend, RawMailbox, RawMailboxRef};
use mw_imap::{Credentials, ImapBackend, ImapConfig, TlsMode};

fn addr(var: &str) -> Option<(String, u16)> {
    let raw = std::env::var(var).ok()?;
    let (host, port) = raw.rsplit_once(':')?;
    Some((host.to_string(), port.parse().ok()?))
}

async fn connect(host: String, port: u16) -> ImapBackend {
    let user = std::env::var("MW_T92_USER").unwrap_or_else(|_| "testuser".into());
    let pass = std::env::var("MW_T92_PASS").unwrap_or_else(|_| "testpass".into());
    ImapBackend::connect(
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
    .expect("sign-in and ENABLE succeed")
}

/// Readable name -> listed mailbox.
async fn listed(backend: &ImapBackend) -> Vec<(String, RawMailbox)> {
    backend
        .list_mailboxes()
        .await
        .expect("LIST parses")
        .into_iter()
        .map(|m| (backend.decode_mailbox_name(&m.mailbox_ref.name), m))
        .collect()
}

fn find<'a>(all: &'a [(String, RawMailbox)], name: &str) -> Option<&'a RawMailbox> {
    all.iter().find(|(n, _)| n == name).map(|(_, m)| m)
}

fn suffix() -> String {
    format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            % 1_000_000
    )
}

async fn run(backend: ImapBackend, expect_utf8_wire: bool) {
    let all = listed(&backend).await;
    let names: Vec<&str> = all.iter().map(|(n, _)| n.as_str()).collect();
    for want in ["INBOX", "Entwürfe", "Корзина", "Archive/2026"] {
        assert!(names.contains(&want), "{want} missing from {names:?}");
    }
    let drafts = find(&all, "Entwürfe").unwrap();
    if expect_utf8_wire {
        assert_eq!(drafts.mailbox_ref.name, "Entwürfe", "UTF-8 on the wire");
    } else {
        assert_eq!(
            drafts.mailbox_ref.name, "Entw&APw-rfe",
            "modified UTF-7 on the wire"
        );
    }
    assert_eq!(
        backend.encode_mailbox_name("Корзина"),
        find(&all, "Корзина").unwrap().mailbox_ref.name,
        "the spelling sent in commands is the server's own"
    );

    // STATUS on a non-ASCII folder: append one message, the count follows.
    let before = drafts.total;
    let raw = b"From: a@example.org\r\nTo: testuser@example.org\r\nSubject: t92 \xc3\xbc\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nhallo\r\n";
    backend
        .append(&drafts.mailbox_ref, raw, &[])
        .await
        .expect("APPEND to a non-ASCII folder");
    let all = listed(&backend).await;
    assert_eq!(
        find(&all, "Entwürfe").unwrap().total,
        before + 1,
        "STATUS count"
    );

    // CREATE, RENAME, DELETE with names outside ASCII.
    let tag = suffix();
    let created = format!("Тест {tag}");
    let renamed = format!("Überprüft «{tag}» & co");
    backend
        .create_mailbox(&backend.encode_mailbox_name(&created))
        .await
        .expect("CREATE");
    let all = listed(&backend).await;
    let row = find(&all, &created).unwrap_or_else(|| panic!("{created} not listed"));
    let from = RawMailboxRef {
        name: row.mailbox_ref.name.clone(),
        uidvalidity: row.mailbox_ref.uidvalidity,
    };
    backend
        .rename_mailbox(&from, &backend.encode_mailbox_name(&renamed))
        .await
        .expect("RENAME");
    let all = listed(&backend).await;
    assert!(find(&all, &created).is_none(), "old name gone");
    let row = find(&all, &renamed).unwrap_or_else(|| panic!("{renamed} not listed"));
    if !expect_utf8_wire {
        assert!(row.mailbox_ref.name.is_ascii(), "{}", row.mailbox_ref.name);
    }
    backend
        .delete_mailbox(&row.mailbox_ref)
        .await
        .expect("DELETE");
    let all = listed(&backend).await;
    assert!(find(&all, &renamed).is_none(), "deleted");
}

#[tokio::test]
#[ignore = "requires Dovecot with UTF8=ACCEPT (set MW_T92_IMAP=host:port)"]
async fn t92_utf8_accept_dovecot_signs_in_lists_and_renames_non_ascii_folders() {
    let Some((host, port)) = addr("MW_T92_IMAP") else {
        eprintln!("MW_T92_IMAP not set; skipping");
        return;
    };
    let backend = connect(host, port).await;
    // UTF-8 names: decoding is the identity, a legacy spelling exists.
    assert_eq!(backend.decode_mailbox_name("Entw&APw-rfe"), "Entw&APw-rfe");
    assert_eq!(
        backend.legacy_mailbox_spelling("Entwürfe").as_deref(),
        Some("Entw&APw-rfe")
    );
    run(backend, true).await;
}

#[tokio::test]
#[ignore = "requires Dovecot without UTF-8 names (set MW_T92_IMAP_7BIT=host:port)"]
async fn t92_seven_bit_dovecot_decodes_modified_utf7_names() {
    let Some((host, port)) = addr("MW_T92_IMAP_7BIT") else {
        eprintln!("MW_T92_IMAP_7BIT not set; skipping");
        return;
    };
    let backend = connect(host, port).await;
    assert_eq!(backend.decode_mailbox_name("Entw&APw-rfe"), "Entwürfe");
    assert_eq!(backend.legacy_mailbox_spelling("Entwürfe"), None);
    run(backend, false).await;
}
