//! Whole-database encryption at rest for the SQLite store (SQLCipher 4).
//!
//! # What this covers
//!
//! * The SQLite main file, its `-wal` and `-journal` files are encrypted page by
//!   page (AES-256-CBC with a per-page HMAC-SHA512) when a database key is in use.
//! * The key never lives in the database or in the settings table. It is resolved
//!   from the environment: `MW_DB_KEY`, `MW_DB_KEY_FILE` or `MW_DB_KEY_COMMAND`.
//! * `MW_DB_ENCRYPTION` is `off`, `optional` (default) or `required`.
//!
//! # What this does not cover
//!
//! * Postgres: the server's disk is not reachable from the application. See
//!   [`crate::Store::postgres_encryption_report`] for what can be observed.
//! * Files outside the database (search index, logs, spool); those are reported by
//!   the server's encryption status endpoint.
//! * A process that holds the key, memory contents, or the key source itself.
//!
//! The crypto provider is the Rust one patched into the vendored `libsqlite3-sys`
//! (no OpenSSL). Plain SQLite files still open when no key is configured.

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use sha2::{Digest, Sha256, Sha512};
use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};
use sqlx::{ConnectOptions, Connection, Row};
use zeroize::Zeroizing;

/// First 16 bytes of every plaintext SQLite database file.
pub const SQLITE_MAGIC: &[u8; 16] = b"SQLite format 3\0";

/// Fixed salt for turning a passphrase into a 256-bit key. A raw 64-hex key skips this.
const PASSPHRASE_SALT: &[u8] = b"mailwoman/db-key/v1";
const PASSPHRASE_ITERATIONS: u32 = 310_000;

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("database encryption key problem: {0}")]
    Key(String),
    #[error("{0}")]
    Config(String),
    #[error(
        "db.encryption=required but the database file {0} is not encrypted; run `mailwoman db encrypt` first"
    )]
    PlaintextButRequired(String),
    #[error("db.encryption=required but no database key is configured (MW_DB_KEY / MW_DB_KEY_FILE / MW_DB_KEY_COMMAND)")]
    KeyMissingButRequired,
    #[error("the database file {0} is encrypted but no database key is configured")]
    EncryptedNoKey(String),
    #[error("the database key does not open {0} (wrong key, or the file is not a database)")]
    WrongKey(String),
    #[error("encryption migration failed: {0}")]
    Migration(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("database error: {0}")]
    Db(String),
}

impl From<std::io::Error> for EncryptionError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<sqlx::Error> for EncryptionError {
    fn from(e: sqlx::Error) -> Self {
        Self::Db(e.to_string())
    }
}

/// `db.encryption`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EncryptionMode {
    /// Never encrypt; an encrypted file is refused with a clear error.
    Off,
    /// Encrypt when a key is configured; an existing plaintext file keeps opening.
    Optional,
    /// A key is mandatory and the file must already be encrypted.
    Required,
}

impl EncryptionMode {
    pub fn parse(s: &str) -> Result<Self, EncryptionError> {
        match s.trim().to_ascii_lowercase().as_str() {
            "" | "optional" => Ok(Self::Optional),
            "off" | "false" | "no" | "0" => Ok(Self::Off),
            "required" | "require" | "on" | "true" | "yes" | "1" => Ok(Self::Required),
            other => Err(EncryptionError::Config(format!(
                "MW_DB_ENCRYPTION must be off, optional or required, got {other:?}"
            ))),
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Optional => "optional",
            Self::Required => "required",
        }
    }
}

/// A 256-bit database key. Never printed, never serialised.
#[derive(Clone)]
pub struct DbKey(Zeroizing<[u8; 32]>);

impl std::fmt::Debug for DbKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DbKey(fingerprint={})", self.fingerprint())
    }
}

impl DbKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(Zeroizing::new(bytes))
    }

    pub fn generate() -> Self {
        use rand::RngCore;
        let mut k = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut k);
        Self::from_bytes(k)
    }

    /// A 64-character hex string is the key itself; anything else is a passphrase
    /// stretched with PBKDF2-HMAC-SHA512 (310 000 rounds, fixed application salt).
    pub fn from_secret(secret: &str) -> Result<Self, EncryptionError> {
        let s = secret.trim();
        if s.is_empty() {
            return Err(EncryptionError::Key("the key is empty".into()));
        }
        if s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()) {
            let mut k = [0u8; 32];
            for (i, c) in s.as_bytes().chunks(2).enumerate() {
                k[i] = u8::from_str_radix(std::str::from_utf8(c).unwrap_or("00"), 16)
                    .map_err(|_| EncryptionError::Key("bad hex".into()))?;
            }
            return Ok(Self::from_bytes(k));
        }
        if s.len() < 16 {
            return Err(EncryptionError::Key(
                "a passphrase must be at least 16 characters (or give a 64-character hex key)".into(),
            ));
        }
        let mut k = [0u8; 32];
        pbkdf2::pbkdf2_hmac::<Sha512>(s.as_bytes(), PASSPHRASE_SALT, PASSPHRASE_ITERATIONS, &mut k);
        Ok(Self::from_bytes(k))
    }

    /// Short identifier safe to display: lets an operator tell two keys apart.
    pub fn fingerprint(&self) -> String {
        let mut h = Sha256::new();
        h.update(b"mailwoman/db-key-fp/v1");
        h.update(&self.0[..]);
        let d = h.finalize();
        d[..8].iter().map(|b| format!("{b:02x}")).collect()
    }

    pub fn to_hex(&self) -> Zeroizing<String> {
        Zeroizing::new(self.0.iter().map(|b| format!("{b:02x}")).collect())
    }

    /// The value for `PRAGMA key`: a raw key, so no per-connection PBKDF2 round.
    fn pragma_value(&self) -> Zeroizing<String> {
        Zeroizing::new(format!("\"x'{}'\"", self.to_hex().as_str()))
    }

    /// The `KEY` argument of `ATTACH ... KEY`.
    fn attach_literal(&self) -> Zeroizing<String> {
        Zeroizing::new(format!("\"x'{}'\"", self.to_hex().as_str()))
    }
}

/// Where a key came from, for the status page. Contains no secret.
#[derive(Debug, Clone, serde::Serialize)]
pub struct KeySourceInfo {
    /// `env`, `file`, `command` or `test-env`.
    pub kind: &'static str,
    /// The variable name, file path, or command's program name.
    pub detail: String,
    /// Seconds since the key file was last modified, when the source is a file.
    pub age_secs: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    pub mode: EncryptionMode,
    pub key: Option<DbKey>,
    pub source: Option<KeySourceInfo>,
}

impl EncryptionConfig {
    pub fn off() -> Self {
        Self {
            mode: EncryptionMode::Off,
            key: None,
            source: None,
        }
    }

    pub fn with_key(mode: EncryptionMode, key: DbKey) -> Self {
        Self {
            mode,
            key: Some(key),
            source: None,
        }
    }

    /// Read `MW_DB_ENCRYPTION` and the key variables.
    pub fn from_env() -> Result<Self, EncryptionError> {
        let mode = match std::env::var("MW_DB_ENCRYPTION") {
            Ok(v) => EncryptionMode::parse(&v)?,
            Err(_) => EncryptionMode::Optional,
        };
        let (key, source) = resolve_key_from_env(
            "MW_DB_KEY",
            "MW_DB_KEY_FILE",
            "MW_DB_KEY_COMMAND",
            Some("MW_TEST_DB_KEY"),
        )?;
        Ok(Self { mode, key, source })
    }

    /// The key to use when opening `path`, after applying the mode rules.
    /// `Ok(None)` means "open as plaintext".
    pub fn key_for_open(&self, path: &Path) -> Result<Option<&DbKey>, EncryptionError> {
        let state = probe_file(path)?;
        let shown = path.display().to_string();
        match (self.mode, state, self.key.as_ref()) {
            (EncryptionMode::Required, _, None) => Err(EncryptionError::KeyMissingButRequired),
            (EncryptionMode::Required, FileState::Plaintext, Some(_)) => {
                Err(EncryptionError::PlaintextButRequired(shown))
            }
            (EncryptionMode::Required, _, Some(k)) => Ok(Some(k)),
            (EncryptionMode::Off, FileState::NotPlaintext, _) => Err(EncryptionError::Config(
                format!(
                    "{shown} is not a plain SQLite file but db.encryption=off; set MW_DB_ENCRYPTION=optional|required and provide the key, or run `mailwoman db decrypt`"
                ),
            )),
            (EncryptionMode::Off, _, _) => Ok(None),
            (EncryptionMode::Optional, FileState::NotPlaintext, None) => {
                Err(EncryptionError::EncryptedNoKey(shown))
            }
            (EncryptionMode::Optional, FileState::Plaintext, _) => Ok(None),
            (EncryptionMode::Optional, _, k) => Ok(k),
        }
    }
}

/// Resolve a key from `<key_var>`, `<file_var>` or `<cmd_var>` (first one set wins).
pub fn resolve_key_from_env(
    key_var: &str,
    file_var: &str,
    cmd_var: &str,
    test_var: Option<&str>,
) -> Result<(Option<DbKey>, Option<KeySourceInfo>), EncryptionError> {
    let env_nonempty = |n: &str| std::env::var(n).ok().filter(|v| !v.trim().is_empty());
    if let Some(v) = env_nonempty(key_var) {
        return Ok((
            Some(DbKey::from_secret(&v)?),
            Some(KeySourceInfo {
                kind: "env",
                detail: key_var.into(),
                age_secs: None,
            }),
        ));
    }
    if let Some(p) = env_nonempty(file_var) {
        let path = PathBuf::from(&p);
        let meta = std::fs::metadata(&path)
            .map_err(|e| EncryptionError::Key(format!("{file_var} {}: {e}", path.display())))?;
        if !meta.is_file() || meta.len() > 4096 {
            return Err(EncryptionError::Key(format!(
                "{file_var} must name a small regular file"
            )));
        }
        let text = Zeroizing::new(std::fs::read_to_string(&path)?);
        let age = meta
            .modified()
            .ok()
            .and_then(|m| m.elapsed().ok())
            .map(|d| d.as_secs());
        return Ok((
            Some(DbKey::from_secret(&text)?),
            Some(KeySourceInfo {
                kind: "file",
                detail: p,
                age_secs: age,
            }),
        ));
    }
    if let Some(c) = env_nonempty(cmd_var) {
        // The secret-manager hook: any program that prints the key on stdout, e.g.
        // `vault kv get -field=key secret/mailwoman`. No shell is involved; the
        // command line is split on whitespace.
        let mut parts = c.split_whitespace();
        let program = parts
            .next()
            .ok_or_else(|| EncryptionError::Key(format!("{cmd_var} is empty")))?;
        let out = std::process::Command::new(program)
            .args(parts)
            .stdin(std::process::Stdio::null())
            .output()
            .map_err(|e| EncryptionError::Key(format!("{cmd_var}: cannot run {program}: {e}")))?;
        if !out.status.success() {
            return Err(EncryptionError::Key(format!(
                "{cmd_var}: {program} exited with {}",
                out.status
            )));
        }
        let text = Zeroizing::new(String::from_utf8_lossy(&out.stdout).into_owned());
        return Ok((
            Some(DbKey::from_secret(&text)?),
            Some(KeySourceInfo {
                kind: "command",
                detail: program.into(),
                age_secs: None,
            }),
        ));
    }
    if let Some(t) = test_var {
        if let Some(v) = env_nonempty(t) {
            return Ok((
                Some(DbKey::from_secret(&v)?),
                Some(KeySourceInfo {
                    kind: "test-env",
                    detail: t.into(),
                    age_secs: None,
                }),
            ));
        }
    }
    Ok((None, None))
}

static GLOBAL: RwLock<Option<Arc<EncryptionConfig>>> = RwLock::new(None);

/// Install the process-wide configuration (the server does this at startup).
pub fn set_global(cfg: EncryptionConfig) {
    *GLOBAL.write().unwrap_or_else(|e| e.into_inner()) = Some(Arc::new(cfg));
}

/// The process-wide configuration; read from the environment on first use.
pub fn global() -> Result<Arc<EncryptionConfig>, EncryptionError> {
    if let Some(c) = GLOBAL.read().unwrap_or_else(|e| e.into_inner()).as_ref() {
        return Ok(c.clone());
    }
    let cfg = Arc::new(EncryptionConfig::from_env()?);
    let mut w = GLOBAL.write().unwrap_or_else(|e| e.into_inner());
    Ok(w.get_or_insert(cfg).clone())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FileState {
    /// No file yet.
    Missing,
    /// Zero bytes.
    Empty,
    /// Starts with `SQLite format 3\0`.
    Plaintext,
    /// Anything else: an encrypted database (or a file that is not a database).
    NotPlaintext,
}

/// Classify the first 16 bytes of `path`.
pub fn probe_file(path: &Path) -> Result<FileState, EncryptionError> {
    use std::io::Read;
    let mut f = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(FileState::Missing),
        Err(e) => return Err(e.into()),
    };
    let mut head = [0u8; 16];
    let mut n = 0;
    while n < 16 {
        let r = f.read(&mut head[n..])?;
        if r == 0 {
            break;
        }
        n += r;
    }
    Ok(match n {
        0 => FileState::Empty,
        16 if &head == SQLITE_MAGIC => FileState::Plaintext,
        _ => FileState::NotPlaintext,
    })
}

/// Make sure the Rust crypto provider is linked before any connection opens.
pub fn init() {
    libsqlite3_sys::provider::ensure_linked();
}

/// Connection options for a file database, with `PRAGMA key` first when a key is given.
pub fn connect_options(
    path: &Path,
    key: Option<&DbKey>,
    create: bool,
) -> SqliteConnectOptions {
    init();
    let mut o = SqliteConnectOptions::new().filename(path).create_if_missing(create);
    if let Some(k) = key {
        o = o.pragma("key", k.pragma_value().as_str().to_owned());
    }
    o
}

/// Map the driver's "file is not a database" to a clear error.
pub fn classify_open_error(path: &Path, e: sqlx::Error, keyed: bool) -> EncryptionError {
    let msg = e.to_string();
    if msg.contains("file is not a database") || msg.contains("not a database") {
        if keyed {
            EncryptionError::WrongKey(path.display().to_string())
        } else {
            EncryptionError::EncryptedNoKey(path.display().to_string())
        }
    } else {
        EncryptionError::Db(msg)
    }
}

/// Open one connection and prove the key works by reading the schema.
pub async fn open_verified(
    path: &Path,
    key: Option<&DbKey>,
    create: bool,
) -> Result<SqliteConnection, EncryptionError> {
    let mut conn = connect_options(path, key, create)
        .connect()
        .await
        .map_err(|e| classify_open_error(path, e, key.is_some()))?;
    sqlx::query("SELECT count(*) FROM sqlite_master")
        .fetch_one(&mut conn)
        .await
        .map_err(|e| classify_open_error(path, e, key.is_some()))?;
    Ok(conn)
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

async fn table_names(conn: &mut SqliteConnection, schema: &str) -> Result<Vec<String>, EncryptionError> {
    let sql = format!(
        "SELECT name FROM {}.sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        quote_ident(schema)
    );
    let rows = sqlx::query(&sql).fetch_all(conn).await?;
    Ok(rows.iter().map(|r| r.get::<String, _>(0)).collect())
}

/// `(table, row count)` for every user table, in name order.
pub async fn table_counts(conn: &mut SqliteConnection) -> Result<Vec<(String, i64)>, EncryptionError> {
    let mut out = Vec::new();
    for t in table_names(conn, "main").await? {
        let n: i64 = sqlx::query(&format!("SELECT count(*) FROM {}", quote_ident(&t)))
            .fetch_one(&mut *conn)
            .await
            .map(|r| r.get(0))?;
        out.push((t, n));
    }
    Ok(out)
}

/// Content digest of one table, order-independent of storage layout: SHA-256 over
/// each row's text rendering, sorted. Used to prove a copy is byte-for-byte equal in content.
async fn table_digest(conn: &mut SqliteConnection, table: &str) -> Result<String, EncryptionError> {
    let cols: Vec<String> = sqlx::query(&format!("PRAGMA table_info({})", quote_ident(table)))
        .fetch_all(&mut *conn)
        .await?
        .iter()
        .map(|r| r.get::<String, _>("name"))
        .collect();
    if cols.is_empty() {
        return Ok(String::new());
    }
    let expr = cols
        .iter()
        .map(|c| format!("quote({})", quote_ident(c)))
        .collect::<Vec<_>>()
        .join(" || char(31) || ");
    let sql = format!(
        "SELECT {expr} AS r FROM {} ORDER BY 1",
        quote_ident(table)
    );
    let rows = sqlx::query(&sql).fetch_all(&mut *conn).await?;
    let mut h = Sha256::new();
    for r in rows {
        let s: String = r.try_get(0).unwrap_or_default();
        h.update(s.as_bytes());
        h.update([0x1e]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TransformReport {
    pub operation: String,
    pub dry_run: bool,
    pub source: String,
    pub from_state: &'static str,
    pub to_state: &'static str,
    pub tables: Vec<(String, i64)>,
    pub rows_total: i64,
    pub integrity: String,
    /// Plaintext/previous copy kept for rollback (absent for dry runs or when purged).
    pub rollback_copy: Option<String>,
    pub new_key_fingerprint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TransformOptions {
    pub dry_run: bool,
    /// Keep the original beside the new file as a rollback copy (default true).
    pub keep_rollback_copy: bool,
}

impl Default for TransformOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            keep_rollback_copy: true,
        }
    }
}

fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}

/// Export `src` (opened with `from`) to a brand-new file `dst` keyed with `to`
/// (`None` = plaintext). Returns the per-table counts of the source.
async fn export_copy(
    src: &Path,
    from: Option<&DbKey>,
    dst: &Path,
    to: Option<&DbKey>,
) -> Result<Vec<(String, i64)>, EncryptionError> {
    // The destination is created by ATTACH, which needs the connection to be allowed
    // to create files; the source's existence is checked by every caller.
    if !src.exists() {
        return Err(EncryptionError::Io(format!("{} does not exist", src.display())));
    }
    let mut conn = open_verified(src, from, true).await?;
    // Fold any WAL into the main file so the export sees everything.
    sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&mut conn)
        .await
        .ok();
    let counts = table_counts(&mut conn).await?;
    let user_version: i64 = sqlx::query("PRAGMA user_version")
        .fetch_one(&mut conn)
        .await
        .map(|r| r.get(0))
        .unwrap_or(0);
    let key_lit: String = match to {
        Some(k) => k.attach_literal().as_str().to_owned(),
        None => "''".to_owned(),
    };
    sqlx::query(&format!("ATTACH DATABASE ?1 AS mwexport KEY {key_lit}"))
        .bind(dst.to_string_lossy().into_owned())
        .execute(&mut conn)
        .await?;
    let exported = async {
        sqlx::query("SELECT sqlcipher_export('mwexport')")
            .execute(&mut conn)
            .await?;
        sqlx::query(&format!("PRAGMA mwexport.user_version = {user_version}"))
            .execute(&mut conn)
            .await?;
        Ok::<(), sqlx::Error>(())
    }
    .await;
    let detach = sqlx::query("DETACH DATABASE mwexport").execute(&mut conn).await;
    exported?;
    detach?;
    conn.close().await.ok();
    Ok(counts)
}

async fn verify_copy(
    original: &Path,
    from: Option<&DbKey>,
    copy: &Path,
    to: Option<&DbKey>,
) -> Result<(Vec<(String, i64)>, String), EncryptionError> {
    let mut a = open_verified(original, from, false).await?;
    let mut b = open_verified(copy, to, false).await?;
    let ca = table_counts(&mut a).await?;
    let cb = table_counts(&mut b).await?;
    if ca != cb {
        return Err(EncryptionError::Migration(format!(
            "row counts differ between source and copy: {ca:?} vs {cb:?}"
        )));
    }
    for (t, _) in &ca {
        if table_digest(&mut a, t).await? != table_digest(&mut b, t).await? {
            return Err(EncryptionError::Migration(format!(
                "content of table {t} differs between source and copy"
            )));
        }
    }
    let integrity: String = sqlx::query("PRAGMA integrity_check")
        .fetch_one(&mut b)
        .await
        .map(|r| r.get(0))?;
    if integrity != "ok" {
        return Err(EncryptionError::Migration(format!(
            "integrity_check on the copy returned {integrity}"
        )));
    }
    // The copy must really be in the target state at the byte level.
    let state = probe_file(copy)?;
    match (to.is_some(), state) {
        (true, FileState::NotPlaintext) | (false, FileState::Plaintext) => {}
        _ => {
            return Err(EncryptionError::Migration(format!(
                "the copy is in unexpected on-disk state {state:?}"
            )));
        }
    }
    a.close().await.ok();
    b.close().await.ok();
    Ok((ca, integrity))
}

/// Re-key a database file: plaintext to encrypted (`from=None`), encrypted to
/// encrypted under another key (rotate), or encrypted to plaintext (`to=None`).
///
/// Copy-then-swap: the data is exported to `<path>.mw-new`, verified against the
/// source (row counts and a content digest for every table, `integrity_check`, and
/// the on-disk header), the original is renamed to `<path>.pre-<op>`, the new file is
/// renamed into place and opened once more with `to`. Any failure restores the
/// original. The server must be stopped; the file is also locked exclusively first.
pub async fn transform(
    path: &Path,
    from: Option<&DbKey>,
    to: Option<&DbKey>,
    opts: &TransformOptions,
) -> Result<TransformReport, EncryptionError> {
    init();
    let op = match (from.is_some(), to.is_some()) {
        (false, true) => "encrypt",
        (true, true) => "rekey",
        (true, false) => "decrypt",
        (false, false) => {
            return Err(EncryptionError::Config("nothing to do: no key on either side".into()));
        }
    };
    match (probe_file(path)?, from.is_some()) {
        (FileState::Missing | FileState::Empty, _) => {
            return Err(EncryptionError::Migration(format!("{} does not exist", path.display())));
        }
        (FileState::Plaintext, true) => {
            return Err(EncryptionError::Migration(
                "the file is plaintext but a source key was given".into(),
            ));
        }
        (FileState::NotPlaintext, false) => {
            return Err(EncryptionError::Migration(
                "the file is not plaintext; supply the current key".into(),
            ));
        }
        _ => {}
    }
    // Refuse while another process has the database open.
    {
        let mut c = open_verified(path, from, false).await?;
        // Leaving WAL mode needs the only connection to the file; with another
        // process (a running server) attached it stays in WAL, which is the signal.
        let mode: String = sqlx::query("PRAGMA journal_mode=DELETE")
            .fetch_one(&mut c)
            .await
            .map(|r| r.get(0))
            .map_err(|e| {
                EncryptionError::Migration(format!("the database is in use (stop the server first): {e}"))
            })?;
        if !mode.eq_ignore_ascii_case("delete") {
            return Err(EncryptionError::Migration(format!(
                "the database is in use (journal mode stays {mode}); stop the server first"
            )));
        }
        sqlx::query("BEGIN EXCLUSIVE").execute(&mut c).await.map_err(|e| {
            EncryptionError::Migration(format!("the database is in use (stop the server first): {e}"))
        })?;
        sqlx::query("ROLLBACK").execute(&mut c).await.ok();
        c.close().await.ok();
    }
    let new_path = sidecar(path, ".mw-new");
    for p in [&new_path, &sidecar(&new_path, "-wal"), &sidecar(&new_path, "-shm")] {
        let _ = std::fs::remove_file(p);
    }
    let result = async {
        export_copy(path, from, &new_path, to).await?;
        verify_copy(path, from, &new_path, to).await
    }
    .await;
    let (tables, integrity) = match result {
        Ok(v) => v,
        Err(e) => {
            let _ = std::fs::remove_file(&new_path);
            return Err(e);
        }
    };
    let rows_total = tables.iter().map(|(_, n)| *n).sum();
    let mut report = TransformReport {
        operation: op.into(),
        dry_run: opts.dry_run,
        source: path.display().to_string(),
        from_state: if from.is_some() { "encrypted" } else { "plaintext" },
        to_state: if to.is_some() { "encrypted" } else { "plaintext" },
        tables,
        rows_total,
        integrity,
        rollback_copy: None,
        new_key_fingerprint: to.map(DbKey::fingerprint),
    };
    if opts.dry_run {
        let _ = std::fs::remove_file(&new_path);
        return Ok(report);
    }
    // Swap.
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    let rollback = sidecar(path, &format!(".pre-{op}-{stamp}"));
    for suffix in ["-wal", "-shm"] {
        let _ = std::fs::remove_file(sidecar(path, suffix));
    }
    std::fs::rename(path, &rollback).map_err(|e| {
        let _ = std::fs::remove_file(&new_path);
        EncryptionError::Migration(format!("cannot move the original aside: {e}"))
    })?;
    if let Err(e) = std::fs::rename(&new_path, path) {
        let _ = std::fs::rename(&rollback, path);
        return Err(EncryptionError::Migration(format!("cannot move the new file into place (original restored): {e}")));
    }
    // Final proof against the file now at the live path.
    let final_check = async {
        let mut c = open_verified(path, to, false).await?;
        let ok: String = sqlx::query("PRAGMA quick_check").fetch_one(&mut c).await.map(|r| r.get(0))?;
        c.close().await.ok();
        if ok == "ok" { Ok(()) } else { Err(EncryptionError::Migration(format!("quick_check: {ok}"))) }
    }
    .await;
    if let Err(e) = final_check {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::rename(&rollback, path);
        return Err(EncryptionError::Migration(format!("post-swap check failed, original restored: {e}")));
    }
    if opts.keep_rollback_copy {
        report.rollback_copy = Some(rollback.display().to_string());
    } else {
        purge_file(&rollback)?;
    }
    write_meta(path, op, to)?;
    Ok(report)
}

/// Overwrite then delete (best effort: flash storage and journaling filesystems can keep old blocks).
pub fn purge_file(path: &Path) -> Result<(), EncryptionError> {
    use std::io::{Seek, Write};
    if let Ok(meta) = std::fs::metadata(path) {
        if let Ok(mut f) = std::fs::OpenOptions::new().write(true).open(path) {
            let zeros = vec![0u8; 64 * 1024];
            let mut left = meta.len();
            let _ = f.seek(std::io::SeekFrom::Start(0));
            while left > 0 {
                let n = left.min(zeros.len() as u64) as usize;
                if f.write_all(&zeros[..n]).is_err() {
                    break;
                }
                left -= n as u64;
            }
            let _ = f.sync_all();
        }
    }
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// Restore the rollback copy written by [`transform`] over the live file.
pub fn rollback(path: &Path, rollback_copy: &Path) -> Result<(), EncryptionError> {
    if !rollback_copy.exists() {
        return Err(EncryptionError::Migration(format!(
            "{} does not exist",
            rollback_copy.display()
        )));
    }
    for suffix in ["-wal", "-shm"] {
        let _ = std::fs::remove_file(sidecar(path, suffix));
    }
    let aside = sidecar(path, ".rolled-back");
    if path.exists() {
        std::fs::rename(path, &aside)?;
    }
    if let Err(e) = std::fs::rename(rollback_copy, path) {
        let _ = std::fs::rename(&aside, path);
        return Err(e.into());
    }
    let _ = std::fs::remove_file(&aside);
    Ok(())
}

// ── key metadata sidecar (no secrets) ───────────────────────────────────────

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct KeyMeta {
    pub key_fingerprint: Option<String>,
    pub encrypted_at: Option<String>,
    pub last_rotated_at: Option<String>,
    pub last_operation: Option<String>,
}

pub fn meta_path(db: &Path) -> PathBuf {
    sidecar(db, ".encryption.json")
}

pub fn read_meta(db: &Path) -> Option<KeyMeta> {
    serde_json::from_slice(&std::fs::read(meta_path(db)).ok()?).ok()
}

fn write_meta(db: &Path, op: &str, to: Option<&DbKey>) -> Result<(), EncryptionError> {
    let now = chrono::Utc::now().to_rfc3339();
    let mut m = read_meta(db).unwrap_or_default();
    m.last_operation = Some(op.into());
    match (op, to) {
        ("decrypt", _) | (_, None) => {
            m.key_fingerprint = None;
        }
        ("rekey", Some(k)) => {
            m.key_fingerprint = Some(k.fingerprint());
            m.last_rotated_at = Some(now);
        }
        (_, Some(k)) => {
            m.key_fingerprint = Some(k.fingerprint());
            m.encrypted_at = Some(now.clone());
            m.last_rotated_at = Some(now);
        }
    }
    std::fs::write(meta_path(db), serde_json::to_vec_pretty(&m).unwrap_or_default())?;
    Ok(())
}

/// Record that a database was created encrypted (called by `Store::open` on first creation).
pub fn note_created(db: &Path, key: &DbKey) {
    if read_meta(db).is_none() {
        let _ = write_meta(db, "create", Some(key));
    }
}

// ── backups ─────────────────────────────────────────────────────────────────

/// Write a consistent copy of the database at `src` to `dst`, encrypted under
/// `dst_key` (`None` writes plaintext and is refused unless `allow_plaintext`).
pub async fn backup(
    src: &Path,
    src_key: Option<&DbKey>,
    dst: &Path,
    dst_key: Option<&DbKey>,
    allow_plaintext: bool,
) -> Result<Vec<(String, i64)>, EncryptionError> {
    init();
    if dst_key.is_none() && src_key.is_some() && !allow_plaintext {
        return Err(EncryptionError::Config(
            "refusing to write a plaintext backup of an encrypted database".into(),
        ));
    }
    if dst.exists() {
        return Err(EncryptionError::Io(format!("{} already exists", dst.display())));
    }
    let counts = export_copy(src, src_key, dst, dst_key).await?;
    let (c2, _) = verify_copy(src, src_key, dst, dst_key).await?;
    debug_assert_eq!(counts, c2);
    Ok(counts)
}

/// Restore a backup to `dst` (which must not exist), re-keying to `dst_key`.
pub async fn restore(
    backup_file: &Path,
    backup_key: Option<&DbKey>,
    dst: &Path,
    dst_key: Option<&DbKey>,
) -> Result<Vec<(String, i64)>, EncryptionError> {
    init();
    if dst.exists() {
        return Err(EncryptionError::Io(format!("{} already exists", dst.display())));
    }
    export_copy(backup_file, backup_key, dst, dst_key).await?;
    let (c, _) = verify_copy(backup_file, backup_key, dst, dst_key).await?;
    if dst_key.is_some() {
        write_meta(dst, "restore", dst_key)?;
    }
    Ok(c)
}

// ── verification (the admin "prove it" button) ──────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct Check {
    pub name: &'static str,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VerifyReport {
    pub path: String,
    pub passed: bool,
    pub checks: Vec<Check>,
}

/// Prove the database file is encrypted and the key works:
/// 1. the first 16 bytes are not `SQLite format 3\0`;
/// 2. a fixed random wrong key fails to open the file;
/// 3. the real key opens it and reads the schema;
/// 4. a known row is written, read back through a fresh connection and removed.
///
/// Step 4 writes to the table `mw_encryption_probe`, created on demand.
pub async fn verify(path: &Path, key: Option<&DbKey>) -> VerifyReport {
    init();
    let mut checks = Vec::new();
    let header = probe_file(path);
    checks.push(match &header {
        Ok(FileState::NotPlaintext) => Check {
            name: "header-not-sqlite",
            passed: true,
            detail: "the first 16 bytes are not 'SQLite format 3'".into(),
        },
        Ok(s) => Check {
            name: "header-not-sqlite",
            passed: false,
            detail: format!("file state is {s:?}; the SQLite header is readable in the clear"),
        },
        Err(e) => Check {
            name: "header-not-sqlite",
            passed: false,
            detail: e.to_string(),
        },
    });
    if let Some(k) = key {
        let wrong = DbKey::generate();
        let r = open_verified(path, Some(&wrong), false).await;
        checks.push(Check {
            name: "wrong-key-rejected",
            passed: matches!(r, Err(EncryptionError::WrongKey(_))),
            detail: match r {
                Err(EncryptionError::WrongKey(_)) => "a random key was refused".into(),
                Err(e) => format!("unexpected error: {e}"),
                Ok(_) => "a random key opened the database".into(),
            },
        });
        let opened = open_verified(path, Some(k), false).await;
        checks.push(Check {
            name: "right-key-opens",
            passed: opened.is_ok(),
            detail: match &opened {
                Ok(_) => format!("opened with key {}", k.fingerprint()),
                Err(e) => e.to_string(),
            },
        });
        if let Ok(mut c) = opened {
            let marker = format!("probe-{}", DbKey::generate().fingerprint());
            let rt = async {
                sqlx::query("CREATE TABLE IF NOT EXISTS mw_encryption_probe(id INTEGER PRIMARY KEY, marker TEXT NOT NULL)")
                    .execute(&mut c)
                    .await?;
                sqlx::query("INSERT INTO mw_encryption_probe(marker) VALUES (?1)")
                    .bind(&marker)
                    .execute(&mut c)
                    .await?;
                sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)").execute(&mut c).await.ok();
                c.close().await.ok();
                let mut c2 = open_verified(path, Some(k), false).await.map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
                let got: Option<String> = sqlx::query("SELECT marker FROM mw_encryption_probe WHERE marker=?1")
                    .bind(&marker)
                    .fetch_optional(&mut c2)
                    .await?
                    .map(|r| r.get(0));
                sqlx::query("DELETE FROM mw_encryption_probe WHERE marker=?1")
                    .bind(&marker)
                    .execute(&mut c2)
                    .await?;
                sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)").execute(&mut c2).await.ok();
                c2.close().await.ok();
                Ok::<_, sqlx::Error>(got)
            }
            .await;
            checks.push(match rt {
                Ok(Some(m)) if m == marker => Check {
                    name: "row-round-trip",
                    passed: true,
                    detail: "wrote a marker row, read it back on a new connection, removed it".into(),
                },
                Ok(_) => Check {
                    name: "row-round-trip",
                    passed: false,
                    detail: "the marker row did not come back".into(),
                },
                Err(e) => Check {
                    name: "row-round-trip",
                    passed: false,
                    detail: e.to_string(),
                },
            });
            // The marker must not be present in the bytes on disk.
            let leaked = std::fs::read(path)
                .map(|b| b.windows(marker.len()).any(|w| w == marker.as_bytes()))
                .unwrap_or(false);
            checks.push(Check {
                name: "marker-not-in-file",
                passed: !leaked,
                detail: if leaked {
                    "the marker text is readable in the database file".into()
                } else {
                    "the marker text does not appear in the database file bytes".into()
                },
            });
        }
    } else {
        checks.push(Check {
            name: "key-configured",
            passed: false,
            detail: "no database key is configured".into(),
        });
    }
    let passed = checks.iter().all(|c| c.passed);
    VerifyReport {
        path: path.display().to_string(),
        passed,
        checks,
    }
}

/// Resolve the file path out of a SQLite DSN / bare path. `None` for in-memory databases.
pub fn sqlite_file_from_dsn(dsn: &str) -> Option<PathBuf> {
    let url = if dsn.starts_with("sqlite:") {
        dsn.to_string()
    } else {
        format!("sqlite://{dsn}?mode=rwc")
    };
    if url.contains(":memory:") || url.contains("mode=memory") {
        return None;
    }
    SqliteConnectOptions::from_str(&url)
        .ok()
        .map(|o| o.get_filename().to_path_buf())
}

/// The `PRAGMA key` value for sqlx connect options.
pub fn key_pragma(k: &DbKey) -> String {
    k.pragma_value().as_str().to_owned()
}

/// The key to open an existing database file that is only being read (a migration
/// source): none for a plaintext file, the configured key for an encrypted one.
pub fn source_key(path: &Path) -> Result<Option<DbKey>, EncryptionError> {
    match probe_file(path)? {
        FileState::NotPlaintext => global()?
            .key
            .clone()
            .map(Some)
            .ok_or_else(|| EncryptionError::EncryptedNoKey(path.display().to_string())),
        _ => Ok(None),
    }
}

/// Connection options for code that reads a store's database file with its own pool
/// (tests, one-off tools): the database key is applied when the file is encrypted.
/// Panics if the file is encrypted and no key is configured.
pub fn raw_options(path: &str, read_only: bool) -> SqliteConnectOptions {
    init();
    let file = Path::new(path);
    let mut o = SqliteConnectOptions::new().filename(file).read_only(read_only);
    if let Some(k) = source_key(file).expect("database key for an encrypted file") {
        o = o.pragma("key", key_pragma(&k));
    }
    o
}
