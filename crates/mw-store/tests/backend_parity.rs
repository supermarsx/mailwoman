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
    MailboxUpsert, MessageUpsert, NewAccount, NoteRow, ServerKey, SsoConfigRow, Store,
    StoreKeyMaterialRow, SubmissionRow,
};

// Test-support helper; not part of the shipped `mw-store` library, so it is
// reached by path rather than through the crate root. See its module docs.
#[path = "../src/test_db.rs"]
mod test_db;

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
    dlp_audit, sender_controls, store_key_material, push_subscriptions, push_config, \
    native_sessions, sso_config, sso_login_audit";

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

    // ---- V8: SSO config (sealed secret) + content-free login audit ----
    s.put_sso_config(&SsoConfigRow {
        id: "corp-oidc".into(),
        kind: "oidc".into(),
        display_name: "Acme SSO".into(),
        scope: "deployment".into(),
        enabled: true,
        config_json: r#"{"kind":"oidc","issuer_url":"https://idp.example"}"#.into(),
        secret: Some(b"client-secret".to_vec()),
        claim_map_json: r#"{"email":"email"}"#.into(),
        created_at: "2026-07-14T00:00:00Z".into(),
        updated_at: "2026-07-14T00:00:00Z".into(),
    })
    .await
    .unwrap();
    out.push(format!(
        "sso_secret_unseal_ok={}",
        s.get_sso_config("corp-oidc")
            .await
            .unwrap()
            .unwrap()
            .secret
            .as_deref()
            == Some(&b"client-secret"[..])
    ));
    out.push(format!(
        "sso_scoped={}",
        s.list_sso_config("deployment").await.unwrap().len()
    ));
    s.append_sso_login_audit("corp-oidc", "oidc", "hash-of-subject", "ok")
        .await
        .unwrap();
    out.push("sso_audit_appended".into());

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
    let path = test_db::unique_file_path("mw-store-migrate", "src.sqlite");
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

// ─────────────────────────────────────────────────────────────────────────────
// `migrate-store` schema-coverage gate.
//
// `migrate_from_sqlite` copies the tables named in `migrate.rs`'s `TABLES`, and
// builds its `MigrationReport` from that same list. Asserting row-count parity
// over the report therefore cannot detect a table the migrator never mentions —
// the oracle is the thing under test. The gate below asserts instead against the
// LIVE schema (`sqlite_master`), so every table the migrations create must be
// accounted for: copied, or named in exactly one of the two lists here.
//
// The two lists are deliberately separate and are NOT equivalent. The first is a
// decision; the second is an open question that has not been decided yet. Both
// are tolerated by the gate today (t24 landed the detection, not the data fix),
// so the backlog is visible in source rather than silently passing.
// ─────────────────────────────────────────────────────────────────────────────

/// DELIBERATELY NOT MIGRATED — live session state that cannot outlive the old
/// deployment, or an admin/deployment surface the operator re-configures on the
/// new host. `(table, reason)`.
const NOT_MIGRATED_DELIBERATELY: &[(&str, &str)] = &[
    (
        "admin_users",
        "0007: the admin panel is a separate identity domain from mail accounts; the \
         operator bootstraps the admin login on the new deployment.",
    ),
    (
        "admin_sessions",
        "0007: live admin bearer-token hashes. Sessions are re-established after a move.",
    ),
    (
        "oauth_clients",
        "0007: admin-approved OAuth clients whose redirect_uris name the OLD host; \
         re-approved against the new one.",
    ),
    (
        "oauth_tokens",
        "0007: live auth-code/access/refresh token hashes. Clients re-authorize.",
    ),
    (
        "oauth_client_meta",
        "0010: RFC 7591 side table keyed by oauth_clients.client_id — meaningless without \
         its parent, which is itself not copied.",
    ),
    (
        "oauth_dcr",
        "0010: the singleton dynamic-client-registration POLICY row, default-disabled; a \
         deployment setting, not account data.",
    ),
    (
        "api_keys",
        "0007: per-deployment API credentials (Argon2id hash + ip_allowlist + rate_limit). \
         Re-minted against the new host.",
    ),
    (
        "webhooks",
        "0007: outbound webhook endpoints and their sealed HMAC secrets, registered against \
         the old deployment's delivery surface.",
    ),
    (
        "domains",
        "0007 (SPEC §19): managed-domain upstream routing + allow/blocklists — operator \
         configuration of the deployment, re-entered in the admin UI.",
    ),
    (
        "directory_config",
        "0008 (SPEC §13): LDAP/GAL endpoint URLs, bind DNs and attribute maps — deployment \
         configuration.",
    ),
    (
        "egress_proxy",
        "0026: the outbound proxy host/port/sealed credentials of the OLD deployment's \
         network position.",
    ),
    (
        "sso_config",
        "0009: IdP endpoints, sealed client secret and claim map, registered against the \
         old deployment's redirect/ACS URLs.",
    ),
    (
        "plugins",
        "0008: the installed WASM plugin registry (admin-approved, enabled deny-by-default). \
         Plugin bundles live outside the store; the operator re-approves them.",
    ),
    (
        "plugin_allowlist",
        "0014: admin-pinned plugin SHA-256 digests. Omission fails CLOSED — an un-pinned \
         plugin will not load.",
    ),
    (
        "ui_plugins",
        "0010: the UI plugin registry (manifest + signature, enabled deny-by-default). Same \
         re-approval story as `plugins`.",
    ),
    (
        "ui_plugin_grants",
        "0010: admin-granted UI plugin capabilities. Omission fails CLOSED — no capability \
         is granted until an admin grants it again.",
    ),
];

/// UNCLASSIFIED — NOT blessed. Tables whose omission has not been ruled a
/// deliberate design choice: they hold per-account state, security enrolments,
/// append-only audit history, or key material, and dropping them is at least
/// arguably data loss. `(table, the open question)`.
///
/// Each entry is a question for the follow-up task that decides, table by table,
/// whether the migrator should copy it. Presence here means "undecided", never
/// "fine to leave behind".
const NOT_MIGRATED_UNCLASSIFIED: &[(&str, &str)] = &[
    (
        "zeroaccess_accounts",
        "0007: the per-account WRAPPED client-derived root key (+ recovery-wrapped copy, \
         paired devices). Nothing else holds it — if it is not copied, zero-access mail on \
         the destination is undecryptable. Strongest data-loss candidate.",
    ),
    (
        "totp_secrets",
        "0015: per-account sealed TOTP secrets. Not copying silently un-enrols every user's \
         authenticator app.",
    ),
    (
        "webauthn_credentials",
        "0015: per-account passkeys/security keys. Same silent un-enrolment.",
    ),
    (
        "recovery_codes",
        "0015: per-account 2FA recovery code hashes — the fallback when the factors above \
         are gone.",
    ),
    (
        "twofa_policy",
        "0015: the admin require-2FA policy (global or per-domain). Omission fails OPEN: a \
         required second factor silently becomes optional after the move.",
    ),
    (
        "quotas",
        "0007: per-account byte/message limits. Also fails OPEN — limits silently lift.",
    ),
    (
        "passwd_config",
        "0008: per-account password policy and the force-change-on-next-login flag; the flag \
         silently clears.",
    ),
    (
        "signatures",
        "0017: user-authored signature bodies and their auto-apply rules. Author content, not \
         deployment config. (`identities.signature_*` IS copied — these are not.)",
    ),
    (
        "notification_rules",
        "0017: per-account notification rules and quiet hours.",
    ),
    (
        "remote_image_grants",
        "0016: per-account remote-image privacy decisions. Fails closed (images re-prompt) but \
         is still user state.",
    ),
    (
        "masked_email",
        "0010 (SPEC §28.4): per-account alias addresses. Losing the rows does not stop mail \
         arriving at the alias, so the destination cannot attribute or manage it.",
    ),
    (
        "uploaded_blobs",
        "0012: metadata for uploaded attachment objects. The objects live on the upload \
         backend; without these rows they are orphaned and unfetchable.",
    ),
    (
        "message_embeddings",
        "0022: per-message sealed vectors. Derived data — recomputable, but only by re-running \
         the embedder over the whole store.",
    ),
    (
        "crypto_changes",
        "0005: the crypto object change-feed. Its siblings `changes` (0001) and `pim_changes` \
         (0004) ARE copied, so this looks like an omission rather than a decision.",
    ),
    (
        "ews_account_cred",
        "0011: per-account EWS endpoint + sealed credential. An account binding, in the same \
         family as `accounts.sealed_creds`, which IS copied.",
    ),
    (
        "bridge_accounts",
        "0008: which account is served by which bridge plugin, plus its settings.",
    ),
    (
        "bridge_oauth_tokens",
        "0018: sealed bridge OAuth access/refresh tokens. Not copying forces every bridged \
         account through re-consent.",
    ),
    (
        "plugin_grants",
        "0008: plugin capability grants, which may be account-scoped (a non-empty account_id) \
         as well as deployment-wide.",
    ),
    (
        "plugin_kv",
        "0013: per-plugin, per-account SEALED plugin state with quota accounting — application \
         data a plugin cannot regenerate.",
    ),
    (
        "assist_config",
        "0008: Assist configuration keyed by scope — 'deployment' AND 'user:<account_id>'. The \
         per-user rows are not deployment config.",
    ),
    (
        "cache_scope",
        "0007: the per-CacheClass layer/TTL matrix. Reads as deployment tuning (a \
         reclassification candidate), but it was never decided.",
    ),
    (
        "audit_log",
        "0007 (SPEC §21): the append-only admin audit log, which by invariant has no delete \
         path — yet a migration drops all of it.",
    ),
    (
        "sso_login_audit",
        "0009: append-only SSO login outcomes (hashed subjects). Same history loss.",
    ),
    (
        "password_change_audit",
        "0008: append-only password-change outcomes. Same history loss.",
    ),
    (
        "assist_audit",
        "0008: append-only, content-free Assist capability audit. Same history loss.",
    ),
];

/// The completeness gate. Enumerates the LIVE schema and requires every table to
/// be accounted for by exactly one of: copied by the migrator, deliberately
/// skipped, or explicitly unclassified. A table added by a new migration lands in
/// none of the three and fails here until someone classifies it.
///
/// Runs unconditionally: WHICH tables the migrator names is backend-independent,
/// so this needs no Postgres DSN — the destination is a second SQLite store.
/// Row counts are deliberately ignored; an empty table copies trivially and would
/// otherwise mask a missing one.
#[tokio::test]
async fn migrate_store_accounts_for_every_schema_table() {
    use std::collections::BTreeSet;

    // A source store with the full migration chain applied and some rows in it.
    let path = test_db::unique_file_path("mw-store-schema-gate", "src.sqlite");
    let path_str = path.to_string_lossy().to_string();
    let src = Store::open(&path_str, key()).await.unwrap();
    let _ = run_ops(&src).await;
    drop(src);

    let dest = Store::open_in_memory(key()).await.unwrap();
    let report = dest.migrate_from_sqlite(&path_str).await.unwrap();

    let src_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{path_str}?mode=ro"))
        .await
        .unwrap();
    let schema: BTreeSet<String> = sqlx::query_scalar::<_, String>(
        "SELECT name FROM sqlite_master WHERE type = 'table' \
           AND name NOT LIKE 'sqlite\\_%' ESCAPE '\\' \
           AND name <> '_sqlx_migrations'",
    )
    .fetch_all(&src_pool)
    .await
    .unwrap()
    .into_iter()
    .collect();
    drop(src_pool);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(format!("{path_str}-wal"));
    let _ = std::fs::remove_file(format!("{path_str}-shm"));

    assert!(
        schema.len() > 50,
        "schema enumeration returned {} tables — the query, not the migrator, is broken",
        schema.len()
    );

    let copied: BTreeSet<&str> = report.tables.iter().map(|(t, _)| t.as_str()).collect();
    let deliberate: BTreeSet<&str> = NOT_MIGRATED_DELIBERATELY.iter().map(|(t, _)| *t).collect();
    let unclassified: BTreeSet<&str> = NOT_MIGRATED_UNCLASSIFIED.iter().map(|(t, _)| *t).collect();

    // The lists stay honest: a renamed or dropped table must not linger in them.
    for t in deliberate.iter().chain(unclassified.iter()) {
        assert!(
            schema.contains(*t),
            "`{t}` is listed as not-migrated but no longer exists in the schema — remove the \
             stale entry"
        );
    }
    // Nothing may be both copied and listed as skipped, or in both lists.
    if let Some(t) = deliberate.intersection(&unclassified).next() {
        panic!("`{t}` appears in BOTH not-migrated lists — it is either decided or not");
    }
    for t in &copied {
        assert!(
            !deliberate.contains(t) && !unclassified.contains(t),
            "`{t}` IS copied by the migrator but is also listed as not migrated"
        );
    }

    // The gate itself.
    let unaccounted: Vec<&str> = schema
        .iter()
        .map(String::as_str)
        .filter(|t| !copied.contains(t) && !deliberate.contains(t) && !unclassified.contains(t))
        .collect();
    assert!(
        unaccounted.is_empty(),
        "`migrate-store` does not account for {} schema table(s): {:?}\nEach must be either \
         added to `TABLES` in mw-store/src/migrate.rs (so it is copied), or listed in \
         `NOT_MIGRATED_DELIBERATELY` with the reason it is safe to leave behind, or listed in \
         `NOT_MIGRATED_UNCLASSIFIED` as an open question.",
        unaccounted.len(),
        unaccounted
    );

    eprintln!(
        "[mw-store] migrate-store schema coverage: {} schema tables = {} copied + {} \
         deliberately skipped + {} UNCLASSIFIED (open questions, not blessed).",
        schema.len(),
        copied.len(),
        deliberate.len(),
        unclassified.len()
    );
}
