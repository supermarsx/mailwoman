//! t18-e-e2e — at-rest reclaim for the C8 Note-metadata seal (R2), LIVE.
//!
//! 26.17's C8 backfill blanked the plaintext title/tags/color/pinned columns IN PLACE,
//! so the prior bytes persisted in SQLite free pages / WAL (or Postgres dead tuples)
//! until a VACUUM. 26.18 (R2) makes the backfill return the sealed-row COUNT and, when
//! it sealed rows THIS run, runs a best-effort dialect-aware reclaim at store-open
//! (SQLite `VACUUM`; Postgres `VACUUM notes` PLAIN — never `FULL`); plus a
//! `mailwoman maintenance vacuum` CLI that runs the same reclaim UNCONDITIONALLY (the
//! operator remedy for a 26.17 DB whose backfill already ran, where the auto-path
//! cannot fire). `v3.rs` unit-tests the count-return + reclaim; THIS leg proves it
//! WIRED end-to-end:
//!   * a LEGACY (pre-0019, plaintext-in-the-clear) note reverted at rest is sealed on
//!     the next store-open AND the count-gated auto-VACUUM runs — after which the main
//!     DB file (which VACUUM rebuilds) holds NO plaintext residue (the reclaim cleared
//!     the free pages the in-place blanking left behind);
//!   * the note still round-trips (VACUUM preserved the live data);
//!   * `mailwoman maintenance vacuum` exits 0 and the note survives it.
//!
//! On SQLite unconditionally AND on live Postgres when MW_E14_PG_DSN is set.
//!
//! Run:
//!   cargo test -p mw-server --test t18_e2e_vacuum -- --nocapture --test-threads=1
//!   MW_E14_PG_DSN=postgres://mailwoman:mailwoman@localhost:5432/mailwoman \
//!     cargo test -p mw-server --test t18_e2e_vacuum -- --nocapture --test-threads=1

use std::process::Command;

use mw_store::{AccountKind, Credentials, NewAccount, NoteRow, ServerKey, Store};
use sqlx::Row as _;
use sqlx::sqlite::SqliteConnectOptions;

// A FIXED key so a re-opened store (and the CLI) unseal what the first open sealed.
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

fn temp_db() -> String {
    let dir = std::env::temp_dir().join(format!("mw-t18-vacuum-{}", unique()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("mw.db").to_string_lossy().into_owned()
}

struct Markers {
    title: String,
    tag: String,
    color: String,
}
fn markers() -> Markers {
    let u = unique();
    Markers {
        title: format!("VACUUM-RESIDUE-TITLE-{u}"),
        tag: format!("VACUUMTAG-{u}"),
        color: format!("VACUUMCOLOR-{u}"),
    }
}

fn note(id: &str, account: &str, m: &Markers, updated_at: &str) -> NoteRow {
    NoteRow {
        id: id.to_string(),
        account_id: account.to_string(),
        notebook_id: None,
        title: m.title.clone(),
        tags_json: format!("[\"{}\"]", m.tag),
        color: m.color.clone(),
        pinned: true,
        body_html: format!("<p>{}</p>", m.title),
        body_text: m.title.clone(),
        links_json: "[]".into(),
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: updated_at.to_string(),
    }
}

async fn seed_account(store: &Store) -> String {
    let username = format!("vac-user-{}@example.org", unique());
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

/// The MAIN database file only (no `-wal`/`-journal`/`-shm` sidecars). A plain SQLite
/// `VACUUM` rebuilds THIS file (reclaiming its free pages); the rolling WAL sidecar is
/// a reused log SQLite does not scrub, so it is not the durable at-rest artifact.
fn read_main_db_bytes(db_path: &str) -> Vec<u8> {
    std::fs::read(db_path).unwrap_or_default()
}

async fn raw_sqlite(db_path: &str) -> sqlx::SqlitePool {
    let opts = SqliteConnectOptions::new().filename(db_path);
    sqlx::SqlitePool::connect_with(opts).await.unwrap()
}

/// Run `mailwoman maintenance --db-path <db> --server-key <hex> vacuum` and assert it
/// exits 0 (the operator remedy path).
fn run_maintenance_vacuum(db_path: &str) {
    let bin = env!("CARGO_BIN_EXE_mailwoman");
    let out = Command::new(bin)
        .args([
            "maintenance",
            "--db-path",
            db_path,
            "--server-key",
            KEY_HEX,
            "vacuum",
        ])
        .output()
        .expect("spawn `mailwoman maintenance vacuum`");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "`maintenance vacuum` must exit 0 (status {:?})\nstdout: {stdout}\nstderr: {stderr}",
        out.status.code()
    );
    assert!(
        stdout.contains("vacuum"),
        "the CLI prints a reclaim summary: {stdout}"
    );
    eprintln!("[t18 vacuum CLI] {}", stdout.trim());
}

// ── SQLite: backfill → count-gated auto-VACUUM → residue reclaimed + CLI ──────────

#[tokio::test]
async fn note_backfill_auto_vacuums_and_cli_reclaims_sqlite() {
    let db = temp_db();
    let m = markers();

    // 1. Seal a note normally (valid sealed BODY blobs to keep on the legacy revert).
    let (account, id) = {
        let store = Store::open(&db, key()).await.unwrap();
        let account = seed_account(&store).await;
        let id = format!("{account}-legacy");
        store
            .upsert_note(&note(&id, &account, &m, "2026-05-05T00:00:00Z"))
            .await
            .unwrap();
        drop(store);
        (account, id)
    };
    let _ = &account;

    // 2. Revert the METADATA to a pre-0019 "legacy" state: plaintext markers written
    //    back into the file, the four *_sealed columns NULLed (body_*_sealed kept). This
    //    physically places the plaintext markers into the DB file — the residue R2 must
    //    reclaim once the backfill blanks them again.
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

    // Sanity: the plaintext markers ARE physically in the file now (so the post-reclaim
    // absence is meaningful, not vacuous).
    assert!(
        contains(&read_all_db_bytes(&db), m.title.as_bytes()),
        "precondition: legacy plaintext is present in the file before the reclaim"
    );

    // 3. Re-open the store → the backfill seals + blanks the legacy row AND, because it
    //    sealed >0 rows this run, the count-gated auto-VACUUM fires (best-effort).
    {
        let store = Store::open(&db, key()).await.unwrap();
        let got = store.get_note(&id).await.unwrap().unwrap();
        assert_eq!(
            got.title, m.title,
            "backfilled row still decrypts its title"
        );
        assert_eq!(got.color, m.color);
        assert!(got.pinned);
        drop(store);
    }

    // 4. AT REST after the auto-VACUUM: the live cell is blank + ciphertext.
    {
        let raw = raw_sqlite(&db).await;
        let row = sqlx::query("SELECT title, pinned, title_sealed FROM notes WHERE id = ?1")
            .bind(&id)
            .fetch_one(&raw)
            .await
            .unwrap();
        assert_eq!(row.get::<String, _>("title"), "", "live plaintext blanked");
        assert_eq!(row.get::<i64, _>("pinned"), 0);
        assert!(
            !row.get::<Vec<u8>, _>("title_sealed").is_empty(),
            "title_sealed populated (ciphertext)"
        );
        raw.close().await;
    }

    // 5. The operator remedy: `maintenance vacuum` runs the same reclaim
    //    unconditionally and exits 0 (for a 26.17 DB whose backfill already ran, this is
    //    the only path that clears residue). The note survives it.
    run_maintenance_vacuum(&db);
    {
        let store = Store::open(&db, key()).await.unwrap();
        let got = store.get_note(&id).await.unwrap().unwrap();
        assert_eq!(got.title, m.title, "note round-trips after the CLI VACUUM");
        assert!(got.pinned);
        drop(store);
    }

    // 6. The R2 headline: the plaintext residue the in-place blanking left in the
    //    DATABASE FILE's free pages is reclaimed by the VACUUM. Scan the MAIN db file
    //    (VACUUM rebuilds it) — the marker must be gone. (The rolling `-wal` sidecar is
    //    a reused log SQLite does not scrub and is not a durable at-rest artifact; a
    //    per-sidecar breakdown is printed for transparency.)
    for suffix in ["", "-wal", "-journal", "-shm"] {
        let p = format!("{db}{suffix}");
        if let Ok(b) = std::fs::read(&p) {
            eprintln!(
                "[t18 vacuum] {p}: {} bytes, title-marker present = {}",
                b.len(),
                contains(&b, m.title.as_bytes())
            );
        }
    }
    let main_bytes = read_main_db_bytes(&db);
    for (label, needle) in [
        ("title", m.title.as_bytes()),
        ("tag", m.tag.as_bytes()),
        ("color", m.color.as_bytes()),
    ] {
        assert!(
            !contains(&main_bytes, needle),
            "note {label} plaintext residue must be reclaimed from the DB file by the VACUUM"
        );
    }
}

// ── live Postgres: same reclaim on the second dialect (VACUUM notes PLAIN) ─────────

#[tokio::test]
async fn note_backfill_reclaim_runs_on_live_postgres() {
    let Ok(dsn) = std::env::var("MW_E14_PG_DSN") else {
        eprintln!(
            "\n[t18 vacuum SKIP] MW_E14_PG_DSN unset — R2 PG `VACUUM notes` reclaim + CLI not exercised on live Postgres.\n"
        );
        return;
    };
    let m = markers();

    let store = Store::open(&dsn, key()).await.expect("open live PG store");
    let account = seed_account(&store).await;
    let id = format!("{account}-legacy");
    store
        .upsert_note(&note(&id, &account, &m, "2026-05-05T00:00:00Z"))
        .await
        .unwrap();

    // Revert to legacy plaintext + NULL sealed via sqlx on live PG.
    let pg = sqlx::postgres::PgPool::connect(&dsn).await.unwrap();
    sqlx::query(
        "UPDATE notes SET title = $2, tags_json = $3, color = $4, pinned = 1,
             title_sealed = NULL, tags_json_sealed = NULL,
             color_sealed = NULL, pinned_sealed = NULL
         WHERE id = $1",
    )
    .bind(&id)
    .bind(&m.title)
    .bind(format!("[\"{}\"]", m.tag))
    .bind(&m.color)
    .execute(&pg)
    .await
    .unwrap();
    drop(store);

    // Re-open → backfill seals the legacy row + the count-gated auto-reclaim runs
    // `VACUUM notes` PLAIN (out of transaction, never FULL) best-effort.
    let store2 = Store::open(&dsn, key()).await.unwrap();
    let got = store2.get_note(&id).await.unwrap().unwrap();
    assert_eq!(got.title, m.title, "PG backfilled row decrypts");
    // At rest: plaintext blanked, sealed column ciphertext.
    let row = sqlx::query("SELECT title, pinned, title_sealed FROM notes WHERE id = $1")
        .bind(&id)
        .fetch_one(&pg)
        .await
        .unwrap();
    assert_eq!(
        row.get::<String, _>("title"),
        "",
        "PG live plaintext blanked"
    );
    assert_eq!(row.get::<i64, _>("pinned"), 0);
    let sealed: Vec<u8> = row.get("title_sealed");
    assert!(
        !sealed.is_empty() && !contains(&sealed, m.title.as_bytes()),
        "PG title_sealed is ciphertext"
    );

    // The PG plain-VACUUM reclaim path runs without error directly (mirrors the CLI).
    store2
        .reclaim_note_metadata_residue()
        .await
        .expect("PG `VACUUM notes` PLAIN reclaim runs out-of-transaction");
    drop(store2);

    // And the CLI runs the same reclaim against the live DSN, exit 0.
    run_maintenance_vacuum(&dsn);

    // Note still round-trips after both reclaims.
    let store3 = Store::open(&dsn, key()).await.unwrap();
    let got = store3.get_note(&id).await.unwrap().unwrap();
    assert_eq!(got.title, m.title, "PG note round-trips after VACUUM notes");
    drop(store3);

    // Clean up this account's rows so re-runs on a persistent PG stay isolated.
    sqlx::query("DELETE FROM notes WHERE account_id = $1")
        .bind(&account)
        .execute(&pg)
        .await
        .unwrap();
    pg.close().await;
}
