//! Dual-backend parity + `migrate-store` integration tests (t6-e1; plan §2.1, §3
//! e1 acceptance). The SQLite path runs ALWAYS. The Postgres path runs only when
//! `DATABASE_URL_PG` (or `MW_TEST_PG`) points at a live server (CI provides
//! `postgres:16`, plan §11); otherwise it logs a SKIP and the SQLite assertions
//! still run — the suite never silently passes the PG path.
//!
//! `backend_parity` is the table-driven mock-vs-real discipline applied to
//! backends: a scripted sequence of repo calls runs on BOTH backends and the
//! backend-independent result snapshot must be byte-identical.

use mw_store::{
    AccountKind, AddressBookRow, CalendarRow, ContactRow, Credentials, EventInstanceRow, EventRow,
    MailboxUpsert, MessageUpsert, NewAccount, NoteRow, ServerKey, Store, StoreKeyMaterialRow,
    SubmissionRow,
};

fn key() -> ServerKey {
    ServerKey::from_bytes(&[7u8; 32]).unwrap()
}

fn pg_dsn() -> Option<String> {
    std::env::var("DATABASE_URL_PG")
        .ok()
        .or_else(|| std::env::var("MW_TEST_PG").ok())
        .filter(|s| !s.trim().is_empty())
}

/// Both PG tests share one database and each `TRUNCATE`s it; this process-wide
/// async lock keeps their PG sections from interleaving (cargo runs tests in
/// parallel threads).
fn pg_lock() -> &'static tokio::sync::Mutex<()> {
    static L: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();
    L.get_or_init(|| tokio::sync::Mutex::new(()))
}

/// Every store table, for TRUNCATE between Postgres runs.
const ALL_TABLES: &str = "sessions, settings, accounts, mailboxes, messages, bodies, threads, \
    pop3_uidl, sync_state, message_meta, tags, saved_searches, submissions, identities, changes, \
    calendars, calendar_shares, events, event_instances, tasks, notebooks, notes, address_books, \
    contacts, contact_groups, pim_changes, crypto_keys, key_associations, security_verdicts, \
    dlp_audit, sender_controls, store_key_material, push_subscriptions, push_config, native_sessions";

async fn truncate_pg(dsn: &str) {
    use sqlx::postgres::PgPoolOptions;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(dsn)
        .await
        .expect("connect pg for truncate");
    sqlx::query(&format!(
        "TRUNCATE TABLE {ALL_TABLES} RESTART IDENTITY CASCADE"
    ))
    .execute(&pool)
    .await
    .expect("truncate pg");
}

/// A scripted sequence exercising a representative method from every repo module,
/// returning a backend-independent snapshot (no server-minted random ids leak in).
async fn run_ops(s: &Store) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    // ---- V0: settings + sessions ----
    s.set_setting("theme", "grove-dark").await.unwrap();
    s.set_setting("theme", "grove-light").await.unwrap();
    out.push(format!(
        "setting={:?}",
        s.get_setting("theme").await.unwrap()
    ));

    let creds = Credentials {
        username: "u@e".into(),
        password: "hunter2".into(),
    };
    let sess = s
        .create_session("acctX", "u@e", "http://j", "http://a", &creds)
        .await
        .unwrap();
    out.push(format!(
        "session_creds={:?}",
        s.get_session(&sess).await.unwrap().credentials
    ));

    // ---- V1: account / mailbox / message / body / thread / pop3 / cursor ----
    let account = s
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example",
                port: 993,
                tls: "implicit",
                username: "u@e",
                sync_policy_json: r#"{"keep":true}"#,
            },
            &creds,
        )
        .await
        .unwrap();
    let acc = s.get_account(&account).await.unwrap();
    out.push(format!(
        "account={}:{}:{}",
        acc.host, acc.port, acc.username
    ));
    out.push(format!(
        "account_creds_ok={}",
        s.account_credentials(&account).await.unwrap() == creds
    ));

    let mbox = s
        .upsert_mailbox(&MailboxUpsert {
            account_id: &account,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 100,
            uidnext: 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        })
        .await
        .unwrap();
    // Idempotent upsert refreshes counts, same row.
    let mbox2 = s
        .upsert_mailbox(&MailboxUpsert {
            account_id: &account,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 100,
            uidnext: 42,
            highestmodseq: 9,
            total: 5,
            unread: 2,
            parent_id: None,
        })
        .await
        .unwrap();
    out.push(format!("mailbox_idempotent={}", mbox == mbox2));
    let mb = s.get_mailbox(&mbox).await.unwrap();
    out.push(format!(
        "mailbox_counts={}:{}:{}:{}",
        mb.uidnext, mb.highestmodseq, mb.total, mb.unread
    ));

    let body_ref = s
        .put_body(&account, b"raw\r\n\r\nsecret-body")
        .await
        .unwrap();
    out.push(format!(
        "body_ok={}",
        s.get_body(&body_ref).await.unwrap().as_deref() == Some(&b"raw\r\n\r\nsecret-body"[..])
    ));

    let sid = s
        .upsert_message(&MessageUpsert {
            account_id: &account,
            mailbox_id: &mbox,
            uid: 5,
            uidvalidity: 100,
            message_id: Some("<a@x>"),
            thread_id: None,
            internaldate: Some("2026-07-01T10:00:00Z"),
            size: 1024,
            flags_json: r#"["Seen"]"#,
            envelope: Some(br#"{"subject":"private-subject"}"#),
            blob_ref: Some(&body_ref),
        })
        .await
        .unwrap();
    // Re-key across a UIDVALIDITY change carries the stable id.
    s.revalidate_mailbox(&mbox, 200).await.unwrap();
    let sid2 = s
        .upsert_message(&MessageUpsert {
            account_id: &account,
            mailbox_id: &mbox,
            uid: 9,
            uidvalidity: 200,
            message_id: Some("<a@x>"),
            thread_id: None,
            internaldate: Some("2026-07-01T10:00:00Z"),
            size: 1024,
            flags_json: r#"["Seen"]"#,
            envelope: None,
            blob_ref: None,
        })
        .await
        .unwrap();
    out.push(format!("stable_id_preserved={}", sid == sid2));
    out.push(format!(
        "envelope_ok={}",
        s.get_envelope(&sid).await.unwrap().as_deref()
            == Some(&br#"{"subject":"private-subject"}"#[..])
    ));
    s.set_flags(&sid, r#"["Seen","Flagged"]"#).await.unwrap();
    out.push(format!(
        "flags={}",
        s.get_message(&sid).await.unwrap().flags_json
    ));
    let loc = s.message_location(&sid).await.unwrap().unwrap();
    out.push(format!("loc={}:{}", loc.uidvalidity, loc.uid));

    let t1 = s.assign_thread(&account, "<root@x>").await.unwrap();
    let t2 = s.assign_thread(&account, "<root@x>").await.unwrap();
    out.push(format!("thread_idempotent={}", t1 == t2));

    s.record_uidl(&account, "UID-A", "stable-a").await.unwrap();
    s.record_uidl(&account, "UID-A", "stable-a").await.unwrap();
    out.push(format!(
        "seen_uidls={}",
        s.seen_uidls(&account).await.unwrap().len()
    ));

    s.save_cursor(&account, &mbox, r#"{"k":1}"#).await.unwrap();
    out.push(format!(
        "cursor={:?}",
        s.load_cursor(&account, &mbox).await.unwrap()
    ));

    // ---- V2: change log + submissions + list ordering ----
    let c1 = s
        .record_change(&account, "Email", "e1", "created")
        .await
        .unwrap();
    let c2 = s
        .record_change(&account, "Email", "e2", "created")
        .await
        .unwrap();
    out.push(format!("changes={}:{}", c1, c2));
    out.push(format!(
        "current_state={}",
        s.current_state(&account, "Email").await.unwrap()
    ));
    out.push(format!(
        "changes_since={}",
        s.changes_since(&account, "Email", 1).await.unwrap().len()
    ));
    s.insert_submission(&SubmissionRow {
        id: "sub1".into(),
        account_id: account.clone(),
        email_id: sid.clone(),
        identity_id: None,
        send_at: None,
        undo_status: "pending".into(),
        hold_seconds: 10,
        created_at: "2026-07-01T10:00:00Z".into(),
    })
    .await
    .unwrap();
    out.push(format!(
        "pending_subs={}",
        s.pending_submissions().await.unwrap().len()
    ));

    // ---- V3: calendar / event range / note seal / contact autocomplete ----
    s.upsert_calendar(&CalendarRow {
        id: "cal1".into(),
        account_id: account.clone(),
        name: "Personal".into(),
        color: "#36f".into(),
        sort_order: 0,
        is_visible: true,
        role: Some("default".into()),
        caldav_url: None,
        sync_token: None,
        ctag: None,
        is_overlay: false,
        component: "VEVENT".into(),
    })
    .await
    .unwrap();
    s.upsert_event(&EventRow {
        id: "ev1".into(),
        calendar_id: "cal1".into(),
        uid: "uid-1".into(),
        etag: Some("\"e1\"".into()),
        ical_raw: "BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n".into(),
        start_utc: Some("2026-07-11T09:00:00Z".into()),
        end_utc: Some("2026-07-11T10:00:00Z".into()),
        tzid: Some("UTC".into()),
        rrule: None,
        status: "confirmed".into(),
        json: Some(b"{}".to_vec()),
    })
    .await
    .unwrap();
    s.replace_event_instances(
        "ev1",
        &[EventInstanceRow {
            event_id: "ev1".into(),
            instance_start_utc: "2026-07-11T09:00:00Z".into(),
            instance_end_utc: "2026-07-11T10:00:00Z".into(),
        }],
    )
    .await
    .unwrap();
    out.push(format!(
        "events_in_range={}",
        s.events_in_range(&account, "2026-07-11T00:00:00Z", "2026-07-12T00:00:00Z")
            .await
            .unwrap()
            .len()
    ));

    s.upsert_note(&NoteRow {
        id: "n1".into(),
        account_id: account.clone(),
        notebook_id: None,
        title: "Groceries".into(),
        tags_json: "[\"home\"]".into(),
        color: "#fc0".into(),
        pinned: true,
        body_html: "<p>milk SUPERSECRET eggs</p>".into(),
        body_text: "milk SUPERSECRET eggs".into(),
        links_json: "[]".into(),
        created_at: "2026-07-11T00:00:00Z".into(),
        updated_at: "2026-07-11T00:00:00Z".into(),
    })
    .await
    .unwrap();
    out.push(format!(
        "note_body={}",
        s.get_note("n1").await.unwrap().unwrap().body_text
    ));

    s.upsert_address_book(&AddressBookRow {
        id: "ab1".into(),
        account_id: account.clone(),
        name: "Contacts".into(),
        is_default: true,
        carddav_url: None,
        sync_token: None,
        ctag: None,
    })
    .await
    .unwrap();
    s.upsert_contact(&ContactRow {
        id: "c1".into(),
        address_book_id: "ab1".into(),
        uid: "c1".into(),
        etag: None,
        vcard_raw: "BEGIN:VCARD\r\nFN:Ada Lovelace\r\nEMAIL:ada@x.test\r\nEND:VCARD\r\n".into(),
        json: None,
        full_name: "Ada Lovelace".into(),
        is_favorite: false,
        photo_blob_id: None,
        pgp_key: None,
        smime_cert: None,
    })
    .await
    .unwrap();
    // Case-insensitive prefix + email-substring scan must match on both dialects.
    out.push(format!(
        "autocomplete_name={}",
        s.autocomplete_contacts(&account, "ada", 10)
            .await
            .unwrap()
            .len()
    ));
    out.push(format!(
        "autocomplete_email={}",
        s.autocomplete_contacts(&account, "ADA@", 10)
            .await
            .unwrap()
            .len()
    ));

    s.record_pim_change(&account, "Note", "n1", "created")
        .await
        .unwrap();
    out.push(format!(
        "pim_state={}",
        s.current_pim_state(&account, "Note").await.unwrap()
    ));

    // ---- V4: crypto change log + store key material ----
    let k1 = s
        .record_crypto_change(&account, "CryptoKey", "k1", "created")
        .await
        .unwrap();
    out.push(format!("crypto_change={}", k1));
    s.upsert_store_key_material(&StoreKeyMaterialRow {
        id: "skm1".into(),
        wrapped_seal_key: vec![1, 2, 3, 4],
        suite: "x25519-ml-kem-768-v1".into(),
        created_at: "2026-07-11T00:00:00Z".into(),
    })
    .await
    .unwrap();
    out.push(format!(
        "store_key={:?}",
        s.get_store_key_material()
            .await
            .unwrap()
            .map(|r| r.wrapped_seal_key)
    ));

    // ---- V5: push subscription + sealed VAPID ----
    s.store_vapid_keypair("PUBLIC", b"vapid-private", "2026-07-11T00:00:00Z")
        .await
        .unwrap();
    out.push(format!(
        "vapid_roundtrip={:?}",
        s.load_vapid_keypair().await.unwrap()
    ));

    out
}

#[tokio::test]
async fn backend_parity_sqlite_and_postgres() {
    let sqlite = Store::open_in_memory(key()).await.unwrap();
    let snap_sqlite = run_ops(&sqlite).await;
    // Sanity: the SQLite snapshot is non-trivial.
    assert!(snap_sqlite.len() > 20);

    match pg_dsn() {
        Some(dsn) => {
            let _guard = pg_lock().lock().await;
            let pg = Store::open_postgres(&dsn, key()).await.unwrap();
            truncate_pg(&dsn).await;
            let snap_pg = run_ops(&pg).await;
            assert_eq!(
                snap_sqlite, snap_pg,
                "SQLite vs Postgres backend-parity snapshot mismatch"
            );
            eprintln!("[mw-store] backend-parity: Postgres path RAN and matched SQLite.");
        }
        None => {
            eprintln!(
                "[mw-store] backend-parity: Postgres path SKIPPED (set DATABASE_URL_PG or \
                 MW_TEST_PG to a live postgres:16 to run it). SQLite path asserted."
            );
        }
    }
}

#[tokio::test]
async fn migrate_store_sqlite_to_postgres() {
    let Some(dsn) = pg_dsn() else {
        eprintln!(
            "[mw-store] migrate-store: SKIPPED (set DATABASE_URL_PG or MW_TEST_PG to a live \
             postgres:16 to run it)."
        );
        return;
    };

    // Populate a temp SQLite file store via the public API.
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "mw-store-migrate-{}.sqlite",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path_str = path.to_string_lossy().to_string();
    let src = Store::open(&path_str, key()).await.unwrap();
    let _snapshot = run_ops(&src).await;
    drop(src);

    // Migrate into a freshly-truncated Postgres backend sharing the same key.
    let _guard = pg_lock().lock().await;
    let pg = Store::open_postgres(&dsn, key()).await.unwrap();
    truncate_pg(&dsn).await;
    let report = pg.migrate_from_sqlite(&path_str).await.unwrap();
    assert!(report.total_rows() > 0, "migrate copied nothing");

    // Content parity: sealed columns open under the shared key, and rows match.
    let note = pg.get_note("n1").await.unwrap().unwrap();
    assert_eq!(note.body_text, "milk SUPERSECRET eggs");
    let skm = pg.get_store_key_material().await.unwrap().unwrap();
    assert_eq!(skm.wrapped_seal_key, vec![1, 2, 3, 4]);
    let (vp, vk) = pg.load_vapid_keypair().await.unwrap().unwrap();
    assert_eq!(
        (vp.as_str(), vk.as_slice()),
        ("PUBLIC", &b"vapid-private"[..])
    );

    // Row-count parity for a couple of representative tables (via SQLite source).
    let src_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{path_str}?mode=ro"))
        .await
        .unwrap();
    for (table, _n) in &report.tables {
        let src_count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM \"{table}\""))
            .fetch_one(&src_pool)
            .await
            .unwrap();
        let copied = report
            .tables
            .iter()
            .find(|(t, _)| t == table)
            .map(|(_, n)| *n as i64)
            .unwrap();
        assert_eq!(src_count, copied, "row-count mismatch for {table}");
    }

    drop(pg);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(format!("{path_str}-wal"));
    let _ = std::fs::remove_file(format!("{path_str}-shm"));
    eprintln!("[mw-store] migrate-store: RAN against Postgres and verified content + counts.");
}
