//! t17-e-e2e — C8 Note-metadata sealing at rest (26.17 headline).
//!
//! "unit-green != wired." `v3.rs` unit-tests the seal/unseal + sort in isolation;
//! THIS leg drives the real `Store` on a real on-disk SQLite DB *and* live Postgres,
//! and asserts the security-relevant facts end-to-end:
//!
//!   * AT REST (the headline): after `upsert_note`, the note's `title` / `tags` /
//!     `color` / `pinned` never appear in plaintext anywhere in the persisted store —
//!     the frozen plaintext columns are BLANK and the sealed columns are ciphertext.
//!     On SQLite we scan every raw DB byte on disk; on Postgres we read the raw
//!     columns back over sqlx.
//!   * ORDER PRESERVED: `list_notes` still returns pinned-first, then `updated_at`
//!     DESC within each pinned group — the exact pre-seal visual order, now produced
//!     by the Rust stable sort after decrypt (the SQL `ORDER BY pinned` is gone).
//!   * ROUND-TRIP: `get_note` decrypts all four metadata fields + the body back.
//!   * STORE-OPEN BACKFILL: a *legacy* row (pre-0019, `title_sealed IS NULL`, metadata
//!     in the clear) is sealed + blanked the next time the store opens, and a second
//!     open is a no-op (the sealed bytes are byte-identical — idempotent).
//!
//! Run:
//!   cargo test -p mw-server --test t17_note_seal -- --nocapture --test-threads=1
//!   MW_E14_PG_DSN=postgres://mailwoman:mailwoman@localhost:5432/mailwoman \
//!     cargo test -p mw-server --test t17_note_seal -- --nocapture --test-threads=1

use mw_store::{AccountKind, Credentials, NewAccount, NoteRow, ServerKey, Store};
use sqlx::Row as _;
use sqlx::sqlite::SqliteConnectOptions;

// A FIXED key so a re-opened store unseals what the first open sealed.
const KEY_HEX: &str = "1122334455667788990011223344556677889900112233445566778899001122";

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

fn key() -> ServerKey {
    ServerKey::from_hex(KEY_HEX).unwrap()
}

fn temp_db(tag: &str) -> String {
    let dir = std::env::temp_dir().join(format!("mw-t17-note-{tag}-{}", unique()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("mw.db").to_string_lossy().into_owned()
}

/// Recognisable plaintext markers that must NEVER survive at rest once sealed.
struct Markers {
    title: String,
    tag: String,
    color: String,
}
fn markers() -> Markers {
    let u = unique();
    Markers {
        title: format!("CONFIDENTIAL-TITLE-{u}"),
        tag: format!("SECRETTAG-{u}"),
        color: format!("SECRETCOLOR-{u}"),
    }
}

/// Build a note whose metadata carries the given markers.
fn note(id: &str, account: &str, m: &Markers, pinned: bool, updated_at: &str) -> NoteRow {
    NoteRow {
        id: id.to_string(),
        account_id: account.to_string(),
        notebook_id: None,
        title: m.title.clone(),
        tags_json: format!("[\"{}\"]", m.tag),
        color: m.color.clone(),
        pinned,
        body_html: format!("<p>{}</p>", m.title),
        body_text: m.title.clone(),
        links_json: "[]".into(),
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: updated_at.to_string(),
    }
}

/// Create a real account row (notes has a FK to `accounts`; sqlx enables
/// `foreign_keys` by default) and return its id.
async fn seed_account(store: &Store) -> String {
    let username = format!("note-user-{}@example.org", unique());
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "h",
                port: 993,
                tls: "implicit",
                username: &username,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: username.clone(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

/// Concatenate the SQLite main DB file plus any `-wal`/`-journal` sidecars.
fn read_all_db_bytes(db_path: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for suffix in ["", "-wal", "-journal", "-shm"] {
        let p = format!("{db_path}{suffix}");
        if let Ok(b) = std::fs::read(&p) {
            out.extend_from_slice(&b);
        }
    }
    out
}

async fn raw_sqlite(db_path: &str) -> sqlx::SqlitePool {
    let opts = SqliteConnectOptions::new().filename(db_path);
    sqlx::SqlitePool::connect_with(opts).await.unwrap()
}

// ── SQLite: at-rest seal + order + backfill ──────────────────────────────────

#[tokio::test]
async fn note_metadata_is_sealed_at_rest_and_order_is_preserved_sqlite() {
    let db = temp_db("seal");
    let store = Store::open(&db, key()).await.unwrap();
    let account = seed_account(&store).await;

    // Four notes, inserted in SHUFFLED order, spanning both pinned states so the
    // pinned-first + updated_at-DESC ordering is a real assertion (not incidental).
    // updated_at ISO strings sort lexically == chronologically.
    let m = markers();
    let na = note(
        &format!("{account}-a"),
        &account,
        &m,
        true,
        "2026-01-01T00:00:00Z",
    ); // pinned, old
    let nb = note(
        &format!("{account}-b"),
        &account,
        &m,
        true,
        "2026-03-01T00:00:00Z",
    ); // pinned, new
    let nc = note(
        &format!("{account}-c"),
        &account,
        &m,
        false,
        "2026-02-01T00:00:00Z",
    ); // unpinned
    let nd = note(
        &format!("{account}-d"),
        &account,
        &m,
        false,
        "2026-04-01T00:00:00Z",
    ); // unpinned, newest
    for n in [&nc, &na, &nd, &nb] {
        store.upsert_note(n).await.unwrap();
    }

    // ORDER: pinned first (nb, na by updated_at DESC), then unpinned (nd, nc).
    let listed = store.list_notes(&account).await.unwrap();
    let ids: Vec<&str> = listed.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            nb.id.as_str(),
            na.id.as_str(),
            nd.id.as_str(),
            nc.id.as_str()
        ],
        "list_notes is pinned-first then updated_at DESC (matches pre-seal order)"
    );

    // ROUND-TRIP: get_note decrypts every metadata field + the body.
    let got = store.get_note(&na.id).await.unwrap().unwrap();
    assert_eq!(got.title, m.title);
    assert_eq!(got.tags_json, format!("[\"{}\"]", m.tag));
    assert_eq!(got.color, m.color);
    assert!(got.pinned);
    assert_eq!(got.body_text, m.title);

    // AT REST: the plaintext metadata columns are BLANK; sealed columns are ciphertext.
    let raw = raw_sqlite(&db).await;
    let row = sqlx::query(
        "SELECT title, tags_json, color, pinned, title_sealed, pinned_sealed FROM notes WHERE id = ?1",
    )
    .bind(&na.id)
    .fetch_one(&raw)
    .await
    .unwrap();
    assert_eq!(row.get::<String, _>("title"), "", "plaintext title blanked");
    assert_eq!(
        row.get::<String, _>("tags_json"),
        "[]",
        "plaintext tags blanked"
    );
    assert_eq!(row.get::<String, _>("color"), "", "plaintext color blanked");
    assert_eq!(row.get::<i64, _>("pinned"), 0, "plaintext pinned blanked");
    let title_sealed: Vec<u8> = row.get("title_sealed");
    assert!(!title_sealed.is_empty(), "title_sealed populated");
    assert!(
        !contains(&title_sealed, m.title.as_bytes()),
        "the sealed title column is ciphertext, not plaintext"
    );
    raw.close().await;

    // Force WAL to disk, then scan EVERY raw DB byte: no plaintext marker anywhere.
    drop(store);
    let bytes = read_all_db_bytes(&db);
    for (label, needle) in [
        ("title", m.title.as_bytes()),
        ("tag", m.tag.as_bytes()),
        ("color", m.color.as_bytes()),
    ] {
        assert!(
            !contains(&bytes, needle),
            "the note {label} must never appear in plaintext at rest (sealed)"
        );
    }
}

#[tokio::test]
async fn store_open_backfill_seals_a_legacy_plaintext_row_and_is_idempotent_sqlite() {
    let db = temp_db("backfill");
    let m = markers();

    // 1. Seal a note normally (gives us valid sealed BODY blobs to keep).
    let (account, id) = {
        let store = Store::open(&db, key()).await.unwrap();
        let account = seed_account(&store).await;
        let id = format!("{account}-legacy");
        store
            .upsert_note(&note(&id, &account, &m, true, "2026-05-05T00:00:00Z"))
            .await
            .unwrap();
        drop(store);
        (account, id)
    };
    let _ = &account;

    // 2. Revert the METADATA to a pre-0019 "legacy" state via raw SQL: plaintext
    //    columns restored, the four *_sealed columns NULLed (body_*_sealed kept).
    {
        let raw = raw_sqlite(&db).await;
        sqlx::query(
            "UPDATE notes SET title = ?2, tags_json = ?3, color = ?4, pinned = 1,
                 title_sealed = NULL, tags_json_sealed = NULL,
                 color_sealed = NULL, pinned_sealed = NULL
             WHERE id = ?1",
        )
        .bind(&id)
        .bind(&m.title)
        .bind(format!("[\"{}\"]", m.tag))
        .bind(&m.color)
        .execute(&raw)
        .await
        .unwrap();
        raw.close().await;
    }

    // 3. Re-open the store → the store-open backfill seals + blanks the legacy row.
    {
        let store = Store::open(&db, key()).await.unwrap();
        // Decrypts correctly (from the freshly-sealed columns).
        let got = store.get_note(&id).await.unwrap().unwrap();
        assert_eq!(got.title, m.title, "backfilled row decrypts its title");
        assert_eq!(got.color, m.color);
        assert!(got.pinned);
        drop(store);
    }

    // Snapshot the sealed bytes + assert plaintext is now blank at rest.
    let sealed_after_first: Vec<u8> = {
        let raw = raw_sqlite(&db).await;
        let row = sqlx::query("SELECT title, pinned, title_sealed FROM notes WHERE id = ?1")
            .bind(&id)
            .fetch_one(&raw)
            .await
            .unwrap();
        assert_eq!(
            row.get::<String, _>("title"),
            "",
            "legacy plaintext blanked by backfill"
        );
        assert_eq!(row.get::<i64, _>("pinned"), 0);
        let s: Vec<u8> = row.get("title_sealed");
        assert!(!s.is_empty(), "title_sealed populated by backfill");
        raw.close().await;
        s
    };

    // 4. Re-open AGAIN → no un-backfilled rows remain → a NO-OP: the sealed bytes
    //    are byte-identical (the backfill did not re-seal an already-sealed row).
    {
        let store = Store::open(&db, key()).await.unwrap();
        drop(store);
    }
    let raw = raw_sqlite(&db).await;
    let sealed_after_second: Vec<u8> = sqlx::query("SELECT title_sealed FROM notes WHERE id = ?1")
        .bind(&id)
        .fetch_one(&raw)
        .await
        .unwrap()
        .get("title_sealed");
    raw.close().await;
    assert_eq!(
        sealed_after_first, sealed_after_second,
        "a second store-open is a no-op (idempotent backfill, sealed bytes unchanged)"
    );
}

// ── live Postgres: at-rest seal + order + backfill (same code, second dialect) ─

#[tokio::test]
async fn note_metadata_is_sealed_at_rest_and_backfilled_on_live_postgres() {
    let Ok(dsn) = std::env::var("MW_E14_PG_DSN") else {
        eprintln!(
            "\n[t17 note-seal SKIP] MW_E14_PG_DSN unset — C8 sealing + backfill not exercised on live Postgres.\n"
        );
        return;
    };
    let m = markers();

    // AT REST + ORDER on Postgres.
    let store = Store::open(&dsn, key()).await.expect("open live PG store");
    let account = seed_account(&store).await;
    let na = note(
        &format!("{account}-a"),
        &account,
        &m,
        true,
        "2026-01-01T00:00:00Z",
    );
    let nb = note(
        &format!("{account}-b"),
        &account,
        &m,
        true,
        "2026-03-01T00:00:00Z",
    );
    let nc = note(
        &format!("{account}-c"),
        &account,
        &m,
        false,
        "2026-02-01T00:00:00Z",
    );
    let nd = note(
        &format!("{account}-d"),
        &account,
        &m,
        false,
        "2026-04-01T00:00:00Z",
    );
    for n in [&nc, &na, &nd, &nb] {
        store.upsert_note(n).await.unwrap();
    }
    let ids: Vec<String> = store
        .list_notes(&account)
        .await
        .unwrap()
        .into_iter()
        .map(|n| n.id)
        .collect();
    assert_eq!(
        ids,
        vec![nb.id.clone(), na.id.clone(), nd.id.clone(), nc.id.clone()],
        "PG list_notes pinned-first then updated_at DESC"
    );
    let got = store.get_note(&na.id).await.unwrap().unwrap();
    assert_eq!(got.title, m.title);
    assert!(got.pinned);

    // Read the RAW columns back over sqlx (the Postgres "at-rest scan"): plaintext
    // blanked, sealed column is ciphertext.
    let pg = sqlx::postgres::PgPool::connect(&dsn).await.unwrap();
    let row = sqlx::query(
        "SELECT title, tags_json, color, pinned, title_sealed FROM notes WHERE id = $1",
    )
    .bind(&na.id)
    .fetch_one(&pg)
    .await
    .unwrap();
    assert_eq!(
        row.get::<String, _>("title"),
        "",
        "PG plaintext title blanked"
    );
    assert_eq!(row.get::<String, _>("tags_json"), "[]");
    assert_eq!(row.get::<String, _>("color"), "");
    assert_eq!(row.get::<i64, _>("pinned"), 0);
    let sealed: Vec<u8> = row.get("title_sealed");
    assert!(
        !sealed.is_empty() && !contains(&sealed, m.title.as_bytes()),
        "PG title_sealed is ciphertext"
    );

    // BACKFILL on Postgres: revert one row to legacy plaintext + NULL sealed, reopen.
    let id = na.id.clone();
    sqlx::query(
        "UPDATE notes SET title = $2, color = $3, pinned = 1,
             title_sealed = NULL, tags_json_sealed = NULL,
             color_sealed = NULL, pinned_sealed = NULL
         WHERE id = $1",
    )
    .bind(&id)
    .bind(&m.title)
    .bind(&m.color)
    .execute(&pg)
    .await
    .unwrap();
    drop(store);

    let store2 = Store::open(&dsn, key()).await.unwrap();
    let got = store2.get_note(&id).await.unwrap().unwrap();
    assert_eq!(got.title, m.title, "PG backfilled row decrypts");
    drop(store2);

    let row = sqlx::query("SELECT title, title_sealed FROM notes WHERE id = $1")
        .bind(&id)
        .fetch_one(&pg)
        .await
        .unwrap();
    assert_eq!(
        row.get::<String, _>("title"),
        "",
        "PG legacy plaintext blanked by backfill"
    );
    let s1: Vec<u8> = row.get("title_sealed");
    assert!(!s1.is_empty(), "PG title_sealed populated by backfill");

    // Idempotent: reopen once more → sealed bytes unchanged.
    let store3 = Store::open(&dsn, key()).await.unwrap();
    drop(store3);
    let s2: Vec<u8> = sqlx::query("SELECT title_sealed FROM notes WHERE id = $1")
        .bind(&id)
        .fetch_one(&pg)
        .await
        .unwrap()
        .get("title_sealed");
    assert_eq!(s1, s2, "PG backfill is idempotent (no-op on second open)");

    // Clean up this account's rows so re-runs on a persistent PG stay isolated.
    sqlx::query("DELETE FROM notes WHERE account_id = $1")
        .bind(&account)
        .execute(&pg)
        .await
        .unwrap();
    pg.close().await;
}
