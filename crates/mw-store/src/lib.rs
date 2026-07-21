#![forbid(unsafe_code)]
//! Local store: opaque sessions + settings, with upstream credentials
//! sealed via XChaCha20-Poly1305 (SPEC §7.3, §9). Pluggable backend (V6, t6-e1):
//! the same public API runs on **SQLite** (default) or **Postgres**, selected by
//! DSN in [`Store::open`]. Queries are authored once in the SQLite `?n` style and
//! translated per-backend by the [`backend`]/[`dialect`] helpers (no `sqlx::Any`).

mod cache;
mod redact;
mod seal;
mod v2;
mod v3;
mod v4;
mod v5;
// V6 pluggable-backend seam (plan §1.1, §2.1). `Store` holds a `Backend` and
// dispatches every query through these helpers; the public `Store` API is
// unchanged so `mw-engine`/`mw-server` are untouched.
mod backend;
mod dialect;
mod migrate;
// V6 (0007) additive repo methods (t6-e11 MOUNT). New `Store` methods + row
// structs over the 0007 tables, reusing the frozen dual-backend query layer;
// no existing query or public item is touched.
mod v6;
// V7 (0008) additive repo methods (t7-e9): the `passwd_config` table gap +
// password-change audit + coordinated credential re-seal.
mod v7;
// V7 (0008) admin-config persistence (t7-e14 MOUNT): directory/plugins/assist
// config rows + the content-free assist audit sink.
mod v7_config;
// V8 (0009) SSO config + content-free login-audit persistence (t9-e0). New `Store`
// methods over the sso_config/sso_login_audit tables; sealed secret column.
mod sso;
// V9 (0010) deferred-tail persistence (t10-e0): UI-plugin registry + grants,
// masked-email aliases, OAuth DCR policy + DCR-client metadata. New `Store` methods
// over the 0010 tables; no secret columns (signature is public, tokens hashed).
mod v9_tail;
// 0011 (26.12 t12 conformance): per-account EWS credential binding (sealed) for the
// on-prem Exchange bridge. New `Store` methods over the 0011 `ews_account_cred`
// table; sealed secret column (zero-access posture). No existing item touched.
mod ews_cred;
// 0012 (26.15 t15): new-file upload store + pluggable at-rest storage backend. New
// `Store` methods over the 0012 `uploaded_blobs` table (metadata only) + the
// `UploadBackend` trait and its filesystem impl (sealed objects on disk, never in the
// DB). Backend construction (the upload dir) lives in `mw-server` and is injected via
// `Store::with_upload_backend`.
mod upload;
// 0013 (26.15 t15): persistent per-(plugin, account) plugin KV backing the
// `store:kv-scoped` capability (replaces the non-persistent HostKv stub). Sealed at
// rest, quota-bounded, isolated by (plugin_id, account_id) derived host-side. New
// `Store` methods over the 0013 `plugin_kv` table; no existing item touched.
mod plugin_kv;
// 0014 (26.15 t15): admin-managed third-party component load allowlist
// (admin-pins-digest-at-install). New `Store` methods over the 0014 `plugin_allowlist`
// table — the trust store behind `resolve_component`'s third-party path. A SEPARATE,
// admin-managed fallback the compiled-in first-party pin always takes precedence over;
// stores only the pinned identity + admin provenance, never component bytes.
mod plugin_allowlist;
// 0015 (26.16 t16): login second-factor persistence. New `Store` methods over the
// 0015 `totp_secrets`/`webauthn_credentials`/`recovery_codes`/`twofa_policy` tables —
// TOTP secret sealed at rest, WebAuthn COSE public key stored opaque, recovery codes
// argon2-hashed + single-use, admin require-2FA policy. e3 fills the verification-side
// callers; the module body here is functional CRUD.
mod twofa;
// 0016 (26.16 t16): remote-image display grants (4-grant model: single/all/per-sender/
// per-domain + soft revoke) backing the anonymizing image proxy. New `Store` methods
// over the 0016 `remote_image_grants` table; deny-by-default, no secret columns. e6
// fills the proxy callers.
mod image_grants;
// 0017 (26.16 t16): user preferences — signature templates + notification rules/quiet
// hours. New `Store` methods over the 0017 `signatures`/`notification_rules` tables.
// Saved searches (W13) reuse the FROZEN 0003 `saved_searches` table, not a new one.
// e15/e11 fill the endpoint callers.
mod user_prefs;
// 0018 (26.16 t16): cached bridge OAuth tokens (B1). New `Store` methods over the 0018
// `bridge_oauth_tokens` table; both tokens sealed at rest. e7 fills the OAuth-client
// callers.
mod bridge_tokens;

pub(crate) use backend::{Backend, Row, q};

pub use bridge_tokens::BridgeOauthTokenRow;
pub use ews_cred::EwsAccountCred;
pub use image_grants::RemoteImageGrantRow;
pub use plugin_allowlist::{PluginAllowlistError, PluginAllowlistRow, new_allowlist_pin};
pub use plugin_kv::{PluginKvError, PluginKvLimits};
pub use sso::SsoConfigRow;
pub use twofa::{SessionMeta, TotpSecret, TwofaPolicyRow, WebauthnCredentialRow, session_handle};
pub use upload::{FsUploadBackend, Upload, UploadBackend, UploadError};
pub use user_prefs::{NotificationRulesRow, SignatureRow};
pub use v6::{
    AdminUserRow, ApiKeyRow, AuditRow, CacheScopeRow, DomainRow, OAuthClientRow, OAuthTokenRow,
    QuotaRow, WebhookRow, ZeroAccessRow,
};
pub use v7::PasswdConfigRow;
pub use v7_config::{
    AssistConfigRow, BridgeAccountRow, DirectoryConfigRow, PluginGrantRow, PluginRow,
};
pub use v9_tail::{
    MaskedEmailRow, OAUTH_DCR_POLICY_ID, OAuthClientMetaRow, OAuthDcrPolicyRow, UiPluginGrantRow,
    UiPluginRow,
};

pub use cache::{
    Account, AccountKind, Mailbox, MailboxUpsert, Message, MessageLocation, MessageUpsert,
    NewAccount,
};
pub use redact::Redacted;
pub use seal::{SealError, ServerKey};
pub use v2::{
    ChangeRow, IdentityRow, SavedSearchRow, SnoozeDue, StoredMeta, SubmissionRow, TagRow,
};
pub use v3::{
    AddressBookRow, CalendarRow, ContactGroupRow, ContactRow, EventInstanceRow, EventRow, NoteRow,
    NotebookRow, PimChangeRow, TaskRow,
};
pub use v4::{
    CryptoChangeRow, CryptoKeyRow, DlpAuditRow, KeyAssociationRow, SecurityVerdictRow,
    SenderControlRow, StoreKeyMaterialRow,
};
pub use v5::{NativeSessionRow, PushSubscriptionRow};

use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous,
};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("seal error: {0}")]
    Seal(#[from] SealError),
    #[error("upload backend error: {0}")]
    Upload(#[from] upload::UploadError),
    #[error("session not found")]
    NotFound,
    #[error("corrupt store data: {0}")]
    Corrupt(String),
    #[error("unsupported store DSN: {0}")]
    UnsupportedDsn(String),
}

/// Per-table row counts produced by [`Store::migrate_from_sqlite`] (powers the
/// `mailwoman migrate-store` count + content parity report, plan §2.1).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MigrationReport {
    /// `(table, rows_copied)` in copy order.
    pub tables: Vec<(String, u64)>,
}

impl MigrationReport {
    /// Total rows copied across all tables.
    pub fn total_rows(&self) -> u64 {
        self.tables.iter().map(|(_, n)| *n).sum()
    }
}

/// Plaintext upstream credentials, only ever held decrypted in memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub account_id: String,
    pub username: String,
    pub jmap_url: String,
    pub api_url: String,
    pub credentials: Credentials,
}

#[derive(Clone)]
pub struct Store {
    backend: Backend,
    key: ServerKey,
    /// At-rest storage for sealed upload objects (0012). Defaults to a fail-closed
    /// backend; `mw-server` injects a real [`FsUploadBackend`] via
    /// [`Store::with_upload_backend`] at build time.
    uploads: Arc<dyn UploadBackend>,
}

impl Store {
    /// Open a store, selecting the backend by DSN (plan §1.1):
    ///
    /// * `postgres://…` / `postgresql://…` → the Postgres backend
    ///   (`migrations_pg`, pure-Rust rustls TLS).
    /// * anything else — a bare filesystem path (the historical argument) or a
    ///   `sqlite:` URL — → the SQLite backend, **byte-identical** to prior
    ///   releases and the default.
    ///
    /// The SQLite connection is tuned for the engine's concurrent access pattern
    /// (many logins + the sync loop hammering the same file): WAL journalling lets
    /// a reader and the writer coexist, `busy_timeout` makes a contended writer
    /// wait rather than fail fast with "database is locked", and
    /// `synchronous=NORMAL` is the WAL-safe durability tier. The pool keeps
    /// several connections so reads don't queue behind the writer.
    pub async fn open(path: &str, key: ServerKey) -> Result<Self, StoreError> {
        if path.starts_with("postgres://") || path.starts_with("postgresql://") {
            return Self::open_postgres(path, key).await;
        }
        let url = if path.starts_with("sqlite:") {
            path.to_string()
        } else {
            format!("sqlite://{path}?mode=rwc")
        };
        let opts = SqliteConnectOptions::from_str(&url)?
            .busy_timeout(Duration::from_secs(5))
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(opts)
            .await?;
        Self::init_sqlite(pool, key).await
    }

    /// Open the Postgres backend at `dsn` (`postgres://…`) and migrate.
    pub async fn open_postgres(dsn: &str, key: ServerKey) -> Result<Self, StoreError> {
        let pool = PgPoolOptions::new().max_connections(5).connect(dsn).await?;
        let store = Self {
            backend: Backend::Postgres(pool),
            key,
            uploads: upload::fail_closed_backend(),
        };
        sqlx::migrate!("./migrations_pg")
            .run(store.pg_pool())
            .await?;
        // 0019 (26.17): seal any pre-existing plaintext Note metadata at rest.
        let sealed = store.seal_note_metadata_backfill().await?;
        // 26.18 (R2): if the backfill just blanked plaintext in place, reclaim the
        // dead tuples it left behind. Best-effort — a VACUUM failure must never brick
        // store-open (residue stays until `mailwoman maintenance vacuum`).
        store.reclaim_after_note_backfill(sealed).await;
        Ok(store)
    }

    /// Open an in-memory SQLite store (tests).
    pub async fn open_in_memory(key: ServerKey) -> Result<Self, StoreError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        Self::init_sqlite(pool, key).await
    }

    async fn init_sqlite(pool: SqlitePool, key: ServerKey) -> Result<Self, StoreError> {
        sqlx::migrate!("./migrations").run(&pool).await?;
        let store = Self {
            backend: Backend::Sqlite(pool),
            key,
            uploads: upload::fail_closed_backend(),
        };
        // 0019 (26.17): seal any pre-existing plaintext Note metadata at rest.
        let sealed = store.seal_note_metadata_backfill().await?;
        // 26.18 (R2): reclaim residue best-effort when the backfill sealed rows (see
        // `open_postgres`). Covers a direct 26.16→26.18 upgrade where the backfill runs
        // fresh; deployed 26.17 DBs already backfilled (0 rows here) → operator CLI.
        store.reclaim_after_note_backfill(sealed).await;
        Ok(store)
    }

    /// Best-effort reclaim after the Note-metadata backfill (R2). Runs the dialect-aware
    /// [`Store::reclaim_note_metadata_residue`] only when `sealed > 0` (nothing was
    /// blanked this run → no new residue → skip the whole-DB rewrite), logging and
    /// continuing on error so a VACUUM failure can never fail store-open.
    async fn reclaim_after_note_backfill(&self, sealed: usize) {
        if sealed == 0 {
            return;
        }
        if let Err(e) = self.reclaim_note_metadata_residue().await {
            eprintln!(
                "mw-store: reclaim (VACUUM) after sealing {sealed} note row(s) failed: {e}; \
                 prior plaintext may persist in free pages until `mailwoman maintenance vacuum`"
            );
        }
    }

    /// Inject the at-rest upload storage backend (0012). `mw-server` calls this at
    /// build time with the deployment-configured [`FsUploadBackend`]; a store that
    /// never has one injected is fail-closed on every upload operation.
    pub fn with_upload_backend(mut self, backend: Arc<dyn UploadBackend>) -> Self {
        self.uploads = backend;
        self
    }

    /// The active backend (used by in-crate tests that assert through the shared
    /// query helpers). Repo methods reach the field directly.
    #[cfg(test)]
    pub(crate) fn backend(&self) -> &Backend {
        &self.backend
    }

    /// Reclaim the on-disk residue left after [`Store::seal_note_metadata_backfill`]
    /// blanked the plaintext Note-metadata columns in place (R2): SQLite `VACUUM`
    /// rewrites the whole database file, dropping the freed pages that still held the
    /// old plaintext; Postgres `VACUUM notes` reclaims the `notes` table's dead tuples.
    ///
    /// Deliberately a **plain** `VACUUM`, never `VACUUM FULL` — a plain Postgres VACUUM
    /// takes only a `SHARE UPDATE EXCLUSIVE` lock (concurrent reads/writes continue),
    /// whereas `VACUUM FULL` rewrites the table under an `ACCESS EXCLUSIVE` lock. VACUUM
    /// cannot run inside a transaction, so this executes on the pool in autocommit via
    /// the simple-query protocol (no prepared statement, no wrapping `BEGIN`).
    ///
    /// Callers at store-open run this best-effort (a VACUUM error is logged, never fatal);
    /// the `mailwoman maintenance vacuum` CLI runs it unconditionally as the operator
    /// remedy for residue predating this build.
    pub async fn reclaim_note_metadata_residue(&self) -> Result<(), StoreError> {
        use sqlx::Executor as _;
        match &self.backend {
            // SQLite VACUUM is whole-database (it cannot target a single table).
            Backend::Sqlite(pool) => {
                pool.execute("VACUUM").await?;
            }
            // Plain (non-FULL) VACUUM of just the notes table — the only table whose
            // in-place blanking left dead tuples to reclaim.
            Backend::Postgres(pool) => {
                pool.execute("VACUUM notes").await?;
            }
        }
        Ok(())
    }

    fn pg_pool(&self) -> &sqlx::PgPool {
        match &self.backend {
            Backend::Postgres(p) => p,
            _ => unreachable!("pg_pool on a non-Postgres backend"),
        }
    }

    pub async fn create_session(
        &self,
        account_id: &str,
        username: &str,
        jmap_url: &str,
        api_url: &str,
        creds: &Credentials,
    ) -> Result<String, StoreError> {
        let id = seal::random_token();
        let sealed = self.key.seal(&encode_creds(creds))?;
        let now = Utc::now().to_rfc3339();
        q(
            "INSERT INTO sessions (id, account_id, username, jmap_url, api_url, sealed_creds, created_at, last_seen)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        )
        .bind(&id)
        .bind(account_id)
        .bind(username)
        .bind(jmap_url)
        .bind(api_url)
        .bind(sealed)
        .bind(now)
        .execute(&self.backend)
        .await?;
        Ok(id)
    }

    pub async fn get_session(&self, id: &str) -> Result<Session, StoreError> {
        let row = q(
            "SELECT id, account_id, username, jmap_url, api_url, sealed_creds FROM sessions WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.backend)
        .await?
        .ok_or(StoreError::NotFound)?;

        let creds = decode_creds(&self.key.open(&row.get_blob("sealed_creds"))?)?;
        Ok(Session {
            id: row.get_string("id"),
            account_id: row.get_string("account_id"),
            username: row.get_string("username"),
            jmap_url: row.get_string("jmap_url"),
            api_url: row.get_string("api_url"),
            credentials: creds,
        })
    }

    pub async fn touch_session(&self, id: &str) -> Result<(), StoreError> {
        q("UPDATE sessions SET last_seen = ?2 WHERE id = ?1")
            .bind(id)
            .bind(Utc::now().to_rfc3339())
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    pub async fn delete_session(&self, id: &str) -> Result<(), StoreError> {
        q("DELETE FROM sessions WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// All sessions for an account, most-recently-seen first (V7 §2.7, folded V6
    /// follow-up (a)). Lets **proxy-mode headless scoped-key REST reads** resolve a
    /// session by `account_id` when there is no cookie (a scoped `mwk_…` Bearer key
    /// authorizes, but the data path needs the account's sealed upstream creds).
    /// Backed by the additive `idx_sessions_account` index (0008); no schema change
    /// to 0001. Sealed credentials open under the same `ServerKey` as
    /// [`get_session`](Self::get_session).
    pub async fn sessions_by_account(&self, account_id: &str) -> Result<Vec<Session>, StoreError> {
        let rows = q(
            "SELECT id, account_id, username, jmap_url, api_url, sealed_creds
             FROM sessions WHERE account_id = ?1 ORDER BY last_seen DESC",
        )
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let creds = decode_creds(&self.key.open(&row.get_blob("sealed_creds"))?)?;
            out.push(Session {
                id: row.get_string("id"),
                account_id: row.get_string("account_id"),
                username: row.get_string("username"),
                jmap_url: row.get_string("jmap_url"),
                api_url: row.get_string("api_url"),
                credentials: creds,
            });
        }
        Ok(out)
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<(), StoreError> {
        q("INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value")
        .bind(key)
        .bind(value)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    pub async fn get_setting(&self, key: &str) -> Result<Option<String>, StoreError> {
        let row = q("SELECT value FROM settings WHERE key = ?1")
            .bind(key)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.map(|r| r.get_string("value")))
    }
}

fn encode_creds(c: &Credentials) -> Vec<u8> {
    serde_json::to_vec(&(&c.username, &c.password)).expect("credential encode")
}

fn decode_creds(bytes: &[u8]) -> Result<Credentials, StoreError> {
    let (username, password): (String, String) =
        serde_json::from_slice(bytes).map_err(|_| StoreError::NotFound)?;
    Ok(Credentials { username, password })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn creds() -> Credentials {
        Credentials {
            username: "test@example.org".into(),
            password: "s3cr3t".into(),
        }
    }

    #[tokio::test]
    async fn session_crud_round_trip() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let id = store
            .create_session(
                "a1",
                "test@example.org",
                "http://mock/jmap",
                "http://mock/jmap",
                &creds(),
            )
            .await
            .unwrap();

        let got = store.get_session(&id).await.unwrap();
        assert_eq!(got.account_id, "a1");
        assert_eq!(got.credentials, creds());

        store.touch_session(&id).await.unwrap();
        store.delete_session(&id).await.unwrap();
        assert!(matches!(
            store.get_session(&id).await,
            Err(StoreError::NotFound)
        ));
    }

    #[tokio::test]
    async fn sessions_by_account_returns_all_for_account() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        store
            .create_session("a1", "u1", "http://mock", "http://mock", &creds())
            .await
            .unwrap();
        store
            .create_session("a1", "u1", "http://mock", "http://mock", &creds())
            .await
            .unwrap();
        store
            .create_session("a2", "u2", "http://mock", "http://mock", &creds())
            .await
            .unwrap();

        let a1 = store.sessions_by_account("a1").await.unwrap();
        assert_eq!(a1.len(), 2);
        assert!(a1.iter().all(|s| s.account_id == "a1"));
        // Sealed creds open under the same key.
        assert_eq!(a1[0].credentials, creds());
        assert_eq!(store.sessions_by_account("a2").await.unwrap().len(), 1);
        assert!(store.sessions_by_account("nope").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn credentials_are_encrypted_at_rest() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let id = store
            .create_session("a1", "u", "http://mock", "http://mock", &creds())
            .await
            .unwrap();
        // Read the raw blob and ensure the password is not present in plaintext.
        let sealed = q("SELECT sealed_creds FROM sessions WHERE id = ?1")
            .bind(&id)
            .fetch_one(store.backend())
            .await
            .unwrap()
            .get_blob("sealed_creds");
        assert!(!sealed.windows(6).any(|w| w == b"s3cr3t"));
    }

    #[tokio::test]
    async fn wrong_key_cannot_open() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let id = store
            .create_session("a1", "u", "http://mock", "http://mock", &creds())
            .await
            .unwrap();
        // Swap in a different key and confirm open fails.
        let other = Store {
            backend: store.backend().clone(),
            key: ServerKey::generate(),
            uploads: store.uploads.clone(),
        };
        assert!(other.get_session(&id).await.is_err());
    }

    #[tokio::test]
    async fn concurrent_writes_do_not_lock() {
        // Regression for the engine's "database is locked" stalls: a file-backed
        // pool under many concurrent writers must serialize (busy_timeout + WAL)
        // rather than error. In-memory can't exercise this — it needs a real file.
        // Each task drives its own account so the per-(account,type) change
        // counter never collides; the contention is across tasks/connections.
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mw-store-lock-{unique}.sqlite"));
        let path_str = path.to_string_lossy().to_string();
        let store = Store::open(&path_str, ServerKey::generate()).await.unwrap();

        let mut handles = Vec::new();
        for i in 0..16 {
            let s = store.clone();
            handles.push(tokio::spawn(async move {
                let account_id = s
                    .create_account(
                        &crate::NewAccount {
                            kind: crate::AccountKind::Imap,
                            host: "h",
                            port: 993,
                            tls: "implicit",
                            username: &format!("u{i}"),
                            sync_policy_json: "{}",
                        },
                        &Credentials {
                            username: format!("u{i}"),
                            password: "p".into(),
                        },
                    )
                    .await
                    .expect("account create under contention must not lock");
                for j in 0..25 {
                    s.set_setting(&format!("k{i}-{j}"), "v")
                        .await
                        .expect("setting write under contention must not lock");
                    // A transactional writer (the change log) is the real hot path.
                    s.record_change(&account_id, "Email", &format!("e{i}-{j}"), "created")
                        .await
                        .expect("transactional write under contention must not lock");
                }
                account_id
            }));
        }
        for h in handles {
            let account_id = h.await.unwrap();
            // Each account saw exactly its 25 serial change rows.
            assert_eq!(store.current_state(&account_id, "Email").await.unwrap(), 25);
        }

        drop(store);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(format!("{path_str}-wal"));
        let _ = std::fs::remove_file(format!("{path_str}-shm"));
    }

    #[tokio::test]
    async fn settings_get_set() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        assert_eq!(store.get_setting("theme").await.unwrap(), None);
        store.set_setting("theme", "grove-dark").await.unwrap();
        store.set_setting("theme", "grove-light").await.unwrap();
        assert_eq!(
            store.get_setting("theme").await.unwrap().as_deref(),
            Some("grove-light")
        );
    }
}
