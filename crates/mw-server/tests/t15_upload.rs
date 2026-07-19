//! t15-E-e2e — LEG 1+2+3: NEW-FILE blob upload round-trip over REAL infrastructure.
//!
//! The 26.15 headline: `POST /jmap/upload/{accountId}` accepts arbitrary bytes, the
//! store SEALS them and writes them to a real filesystem [`FsUploadBackend`] object
//! (NEVER plaintext, NEVER in the DB), records only metadata + the server-minted
//! `storage_key` in the 0012 `uploaded_blobs` table, and returns a `U`+64-hex blobId
//! that becomes a real MIME attachment on an outgoing `Email/set` create.
//!
//! "unit-green != wired": the unit tests in `mw-store/src/upload.rs` cover the store
//! methods over a `tempfile` backend on in-memory SQLite. THIS leg proves the same
//! seam end-to-end against REAL infrastructure — a real on-disk FS backend AND (when
//! `MW_E14_PG_DSN` is set) a real Postgres metadata store, driving the full
//! upload -> `fetch_blob` -> `compose_from_spec` -> SEND path so the uploaded bytes
//! actually ride the wire as an attachment. Postgres exercises the 0012 migration +
//! SQL in the second dialect (the class of wiring bug live-E2E exists to catch).
//!
//! ## Legs
//!   1. HEADLINE: `store_upload` -> `U`-blobId; the sealed object is on the real FS
//!      backend (NOT plaintext); an `Email/set` create referencing that blobId + inline
//!      `EmailSubmission/set` sends a message whose bytes carry EXACTLY the uploaded
//!      payload as a named/typed attachment. Metadata lands in the real store.
//!   3. ISOLATION: account B cannot resolve account A's blobId (`get_upload` -> None,
//!      the 404 the HTTP handler returns).
//!   4. GC: an aged unreferenced upload is reclaimed by `sweep_uploads` (backing
//!      `mailwoman maintenance gc-uploads`) AND its on-disk object removed; a fresh one
//!      is kept; a re-sweep is a no-op.
//!
//! (The HTTP 200/413/403 surface — over-`maxSizeUpload` -> 413, foreign account -> 403 —
//! is proven over a real loopback socket by `tests/integration.rs`
//! {`upload_over_max_size_is_413`, `upload_route_rejects_foreign_account`,
//! `upload_route_proxies_to_upstream`}; those checks are mode-independent in the handler.)
//!
//! ## Running
//!   cargo test -p mw-server --test t15_upload                          # SQLite + real FS
//!   docker compose -f docker-compose.ci.yml up -d --wait postgres
//!   MW_E14_PG_DSN=postgres://mailwoman:mailwoman@localhost:5432/mailwoman \
//!     cargo test -p mw-server --test t15_upload -- --nocapture --test-threads=1

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::Engine;
use mw_engine::account::{AccountRuntime, MailSubmitter};
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MailboxRole,
    MessageRef, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result as BackendResult,
    SyncCursor, WatchHandle,
};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, FsUploadBackend, NewAccount, ServerKey, Store};

const UIDVALIDITY: u32 = 100;

/// The known upload payload and its base64 encoding (as mail-builder emits it for a
/// binary attachment). The bytes are ASCII but carried under `application/octet-stream`,
/// so they are base64-transfer-encoded on the wire.
const PAYLOAD: &[u8] = b"mailwoman-upload-e2e-payload-0123456789";
const PAYLOAD_B64: &str = "bWFpbHdvbWFuLXVwbG9hZC1lMmUtcGF5bG9hZC0wMTIzNDU2Nzg5";

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

/// A unique temp directory for one driver's real FS upload backend.
fn temp_root(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("mw-t15-upload-{tag}-{}-{nanos}", unique()))
}

/// A minimal backend: one empty INBOX so `resync` provisions the mailbox structure the
/// compose/send path needs. `append` is a best-effort no-op (Sent).
struct EmptyBackend;

#[async_trait]
impl AccountBackend for EmptyBackend {
    async fn capabilities(&self) -> BackendResult<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> BackendResult<Vec<RawMailbox>> {
        Ok(vec![RawMailbox {
            mailbox_ref: RawMailboxRef {
                name: "INBOX".into(),
                uidvalidity: UIDVALIDITY,
            },
            role: MailboxRole::Inbox,
            parent: None,
            uidnext: 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
        }])
    }
    async fn sync_mailbox(
        &self,
        _mbox: &RawMailboxRef,
        _cursor: &SyncCursor,
    ) -> BackendResult<MailboxDelta> {
        Ok(MailboxDelta {
            added: Vec::new(),
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: UIDVALIDITY,
                uidnext: 1,
            },
        })
    }
    async fn fetch_raw(&self, _refs: &[MessageRef]) -> BackendResult<Vec<RawMessage>> {
        Ok(Vec::new())
    }
    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _rm: &[Flag]) -> BackendResult<()> {
        Ok(())
    }
    async fn move_messages(
        &self,
        _r: &[MessageRef],
        _to: &RawMailboxRef,
    ) -> BackendResult<MoveOutcome> {
        Err(EngineError::Unsupported("empty backend".into()))
    }
    async fn append(
        &self,
        _mbox: &RawMailboxRef,
        _raw: &[u8],
        _flags: &[Flag],
    ) -> BackendResult<MessageRef> {
        Err(EngineError::Unsupported("empty backend".into()))
    }
    async fn watch(&self, _sink: ChangeSink) -> BackendResult<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

/// Captures the `Outgoing` that reaches the wire.
#[derive(Default)]
struct CaptureInner {
    last: Mutex<Option<Outgoing>>,
    calls: AtomicUsize,
}

#[async_trait]
impl MailSubmitter for CaptureInner {
    async fn submit(&self, msg: Outgoing) -> BackendResult<SubmissionResult> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let accepted = msg.rcpt_to.clone();
        *self.last.lock().unwrap() = Some(msg);
        Ok(SubmissionResult {
            accepted,
            rejected: Vec::new(),
        })
    }
}

async fn jmap(engine: &Engine, account_id: &str, calls: Value) -> Value {
    engine
        .handle_jmap(account_id, &json!({ "methodCalls": calls }))
        .await
}

fn method_result<'a>(resp: &'a Value, call_id: &str) -> &'a Value {
    resp["methodResponses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r[2] == call_id)
        .map(|r| &r[1])
        .unwrap_or(&Value::Null)
}

/// Compose a message that attaches the uploaded `blob_id` and submit it inline.
fn upload_and_submit(blob_id: &str) -> Value {
    json!([
        ["Email/set", { "create": { "draft": {
            "to": [{ "email": "boss@example.org" }],
            "subject": "New-file upload attached",
            "from": [{ "email": "me@example.org" }],
            "bodyValues": { "1": { "value": "here is the uploaded file" } },
            "textBody": [{ "partId": "1", "type": "text/plain" }],
            "attachments": [{ "blobId": blob_id, "type": "application/octet-stream", "name": "payload.bin" }]
        } } }, "c1"],
        ["EmailSubmission/set", { "create": { "sub1": {
            "emailId": "#draft",
            "mailwomanHoldSeconds": 0
        } } }, "c2"]
    ])
}

async fn make_account(store: &Store) -> String {
    let uname = format!("me-{}@example.org", unique());
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: &uname,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: uname.clone(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap()
}

/// The full upload -> attach -> send round-trip + isolation, over a real FS backend and
/// the given real store.
async fn drive_upload(store: Store, root: &Path, dialect: &str) {
    let account_id = make_account(&store).await;
    let other_id = make_account(&store).await;

    let engine = Arc::new(Engine::new(store));
    let inner = Arc::new(CaptureInner::default());
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            Arc::new(EmptyBackend) as Arc<dyn AccountBackend>,
            inner.clone() as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    engine.resync(&account_id).await.expect("resync provisions");

    // LEG 1a — upload a real new file: a `U`+64-hex blobId comes back.
    let blob_id = engine
        .store_upload(&account_id, "application/octet-stream", PAYLOAD)
        .await
        .expect("store_upload seals + writes + records");
    assert!(
        blob_id.starts_with('U') && blob_id.len() == 65,
        "[{dialect}] blobId is U+64hex: {blob_id}"
    );

    // LEG 1b — the sealed object exists on the REAL filesystem backend and is NOT
    // plaintext: no on-disk object carries the payload marker.
    let mut found_object = false;
    let mut plaintext_leaked = false;
    for account_dir in std::fs::read_dir(root).unwrap() {
        for obj in std::fs::read_dir(account_dir.unwrap().path()).unwrap() {
            found_object = true;
            let bytes = std::fs::read(obj.unwrap().path()).unwrap();
            if bytes.windows(PAYLOAD.len()).any(|w| w == PAYLOAD) {
                plaintext_leaked = true;
            }
        }
    }
    assert!(
        found_object,
        "[{dialect}] a sealed object was written to disk"
    );
    assert!(
        !plaintext_leaked,
        "[{dialect}] upload bytes must be sealed at rest, never plaintext on disk"
    );

    // LEG 3 — isolation: account B cannot resolve account A's blobId (the 404 path).
    assert!(
        engine
            .fetch_blob(&other_id, &blob_id)
            .await
            .unwrap()
            .is_none(),
        "[{dialect}] a foreign account must not resolve another account's upload"
    );

    // LEG 1c — reference the blobId on an Email/set create + submit inline; the SENT
    // message carries exactly the uploaded bytes as the named/typed attachment.
    let resp = jmap(&engine, &account_id, upload_and_submit(&blob_id)).await;
    assert!(
        method_result(&resp, "c1")["created"]["draft"]["id"].is_string(),
        "[{dialect}] upload draft created (blob resolved, no notCreated): {resp}"
    );
    assert!(
        method_result(&resp, "c1").get("notCreated").is_none(),
        "[{dialect}] no notCreated — the U-blob resolved: {resp}"
    );
    assert!(
        method_result(&resp, "c2")["created"]["sub1"]["id"].is_string(),
        "[{dialect}] submission accepted + sent inline: {resp}"
    );
    assert_eq!(
        inner.calls.load(Ordering::SeqCst),
        1,
        "[{dialect}] the message reached the wire exactly once"
    );

    let sent = inner
        .last
        .lock()
        .unwrap()
        .clone()
        .expect("captured Outgoing");
    let raw = String::from_utf8_lossy(&sent.raw);
    assert!(
        raw.contains("multipart/"),
        "[{dialect}] sent message is multipart: {raw}"
    );
    assert!(
        raw.contains("payload.bin"),
        "[{dialect}] sent message carries the declared attachment filename: {raw}"
    );
    // The uploaded bytes ride the wire (base64, whitespace-folded by the MIME emitter).
    let raw_nows: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        raw_nows.contains(PAYLOAD_B64),
        "[{dialect}] sent message carries EXACTLY the uploaded bytes (base64 {PAYLOAD_B64}): {raw}"
    );

    let _ = std::fs::remove_dir_all(root);
}

/// Count sealed object files under a FS-backend root (one temp root per driver, so it
/// holds only this test's objects — `.tmp` write-siblings are never counted).
fn object_count(root: &Path) -> usize {
    let mut n = 0;
    if let Ok(accounts) = std::fs::read_dir(root) {
        for a in accounts.flatten() {
            if let Ok(objs) = std::fs::read_dir(a.path()) {
                for o in objs.flatten() {
                    if o.path().is_file() {
                        n += 1;
                    }
                }
            }
        }
    }
    n
}

/// GC leg (backs `mailwoman maintenance gc-uploads`): the TTL boundary reclaims rows +
/// on-disk objects in BOTH directions against the real store/backend. A just-uploaded
/// blob is NOT reclaimed by a 24h sweep (created_at is not older than now-24h); forcing
/// the cutoff PAST creation (a negative `older_than` ⇒ cutoff in the future) exercises the
/// exact reclamation SQL + FS-object delete, removing both the row and the disk object.
async fn drive_gc(store: Store, root: &Path, dialect: &str) {
    let account_id = make_account(&store).await;

    // Fresh-kept: a just-uploaded blob survives a 24h TTL sweep.
    let fresh = store
        .put_upload(&account_id, "text/plain", b"fresh-bytes")
        .await
        .unwrap();
    store
        .sweep_uploads(chrono::Duration::hours(24))
        .await
        .unwrap();
    assert!(
        store
            .get_upload(&account_id, &fresh)
            .await
            .unwrap()
            .is_some(),
        "[{dialect}] a fresh upload is NOT reclaimed by a 24h sweep"
    );
    assert!(
        object_count(root) >= 1,
        "[{dialect}] the fresh object is still on disk after the 24h sweep"
    );

    // Aged-reclaimed: a second upload, then a sweep whose cutoff is forced past both rows'
    // creation (older_than = -5s ⇒ cutoff = now + 5s). Everything is reclaimed: rows gone,
    // objects deleted from the real backend.
    store
        .put_upload(&account_id, "text/plain", b"target-bytes")
        .await
        .unwrap();
    let reclaimed = store
        .sweep_uploads(chrono::Duration::seconds(-5))
        .await
        .unwrap();
    assert!(
        reclaimed >= 2,
        "[{dialect}] the forced-past-cutoff sweep reclaims the uploads ({reclaimed})"
    );
    assert!(
        store
            .get_upload(&account_id, &fresh)
            .await
            .unwrap()
            .is_none(),
        "[{dialect}] the reclaimed row is gone after gc"
    );
    assert_eq!(
        object_count(root),
        0,
        "[{dialect}] gc removed the on-disk objects, not just the rows"
    );

    // Re-sweep of the now-empty window reclaims nothing new (idempotent).
    let again = store
        .sweep_uploads(chrono::Duration::seconds(-5))
        .await
        .unwrap();
    assert_eq!(again, 0, "[{dialect}] a re-sweep is a no-op");

    let _ = std::fs::remove_dir_all(root);
}

// ── SQLite (always) ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn upload_round_trip_and_send_sqlite() {
    let root = temp_root("rt-sqlite");
    let store = Store::open_in_memory(ServerKey::generate())
        .await
        .unwrap()
        .with_upload_backend(Arc::new(FsUploadBackend::new(root.clone())));
    drive_upload(store, &root, "sqlite").await;
}

#[tokio::test]
async fn upload_gc_reclaims_aged_only_sqlite() {
    let root = temp_root("gc-sqlite");
    let store = Store::open_in_memory(ServerKey::generate())
        .await
        .unwrap()
        .with_upload_backend(Arc::new(FsUploadBackend::new(root.clone())));
    drive_gc(store, &root, "sqlite").await;
}

// ── Postgres (live via MW_E14_PG_DSN, else loud-skip) ────────────────────────────────

#[tokio::test]
async fn upload_round_trip_and_send_postgres() {
    let Ok(dsn) = std::env::var("MW_E14_PG_DSN") else {
        eprintln!(
            "\n[t15 upload SKIP] MW_E14_PG_DSN unset — live Postgres upload round-trip not driven.\n"
        );
        return;
    };
    let root = temp_root("rt-pg");
    let store = Store::open(&dsn, ServerKey::generate())
        .await
        .expect("open live Postgres store")
        .with_upload_backend(Arc::new(FsUploadBackend::new(root.clone())));
    drive_upload(store, &root, "postgres").await;
}

#[tokio::test]
async fn upload_gc_reclaims_aged_only_postgres() {
    let Ok(dsn) = std::env::var("MW_E14_PG_DSN") else {
        eprintln!("\n[t15 upload gc SKIP] MW_E14_PG_DSN unset — live Postgres gc not driven.\n");
        return;
    };
    let root = temp_root("gc-pg");
    let store = Store::open(&dsn, ServerKey::generate())
        .await
        .expect("open live Postgres store")
        .with_upload_backend(Arc::new(FsUploadBackend::new(root.clone())));
    drive_gc(store, &root, "postgres").await;
}
