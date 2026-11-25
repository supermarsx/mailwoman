//! 0012 new-file upload store + pluggable at-rest storage backend (26.15 t15).
//!
//! `POST /jmap/upload/{accountId}` accepts arbitrary bytes; the store SEALS them
//! (XChaCha20-Poly1305 under the store [`ServerKey`], the same zero-access posture as
//! bodies/creds) and hands the opaque sealed bytes to an [`UploadBackend`] object on
//! disk — the bytes are NEVER written plaintext and NEVER stored in the DB. The 0012
//! `uploaded_blobs` table holds only metadata + the server-minted `storage_key`
//! locating the object + `backend_kind`. A `U`+64-hex `blob_id` (collision-free vs the
//! pure 64-hex stableIds) is returned to the client and resolved by
//! `Engine::fetch_blob`.
//!
//! Storage backend (Design decision 1): a trait with a **filesystem** default impl
//! ([`FsUploadBackend`]). S3 is DEFERRED — the trait boundary is kept clean so an S3
//! impl can slot in later behind the same seam, but 26.15 pulls NO S3 dependency
//! (license floor). The backend construction (the upload directory) lives in
//! `mw-server` and is injected via [`Store::with_upload_backend`]; a store with no
//! injected backend is fail-closed (every put/get/delete errors rather than silently
//! dropping bytes).
//!
//! On-disk isolation: objects are laid out `root/<account_key>/<storage_key>`, where
//! `account_key` is the deterministic lowercase-hex SHA-256 of the account id (a
//! server-minted, fixed-length, path-safe per-account subdirectory token) and
//! `storage_key` is a server-minted 64-hex token. Both are validated to be pure
//! lowercase hex before any path join, so a traversal-y key (`..`, absolute, or
//! separator-bearing) can never escape the configured root. Storage keys are never
//! guest/client input.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::backend::q;
use crate::{Store, StoreError, seal};

/// A backend read/write error, kept dialect-free so `mw-server` can construct the
/// filesystem backend without depending on any concrete store internals.
#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    /// A storage/account key was not the expected server-minted lowercase-hex shape.
    #[error("invalid upload storage key")]
    InvalidKey,
    /// No storage backend was injected into the store (see [`Store::with_upload_backend`]).
    #[error("upload storage backend not configured")]
    NotConfigured,
    /// An underlying I/O failure from the backend.
    #[error("upload storage backend io error: {0}")]
    Io(String),
}

/// Pluggable at-rest storage for sealed upload objects. The store seals the plaintext
/// and hands opaque sealed bytes here; the backend only ever sees ciphertext.
///
/// Keys are server-minted lowercase-hex tokens (never guest input): `account_key`
/// isolates one account's objects, `storage_key` names one object. Implementations
/// MUST validate both are pure hex before touching any path.
#[async_trait]
pub trait UploadBackend: Send + Sync {
    /// A short, stable name recorded in `uploaded_blobs.backend_kind` (e.g. `"fs"`).
    fn kind(&self) -> &'static str;

    /// Write sealed object bytes under `(account_key, storage_key)`.
    async fn put(
        &self,
        account_key: &str,
        storage_key: &str,
        sealed: Vec<u8>,
    ) -> Result<(), UploadError>;

    /// Read the sealed object bytes, or `None` if no such object exists.
    async fn get(
        &self,
        account_key: &str,
        storage_key: &str,
    ) -> Result<Option<Vec<u8>>, UploadError>;

    /// Remove the object. A missing object is not an error (idempotent).
    async fn delete(&self, account_key: &str, storage_key: &str) -> Result<(), UploadError>;

    /// Write a sealed object given in parts, pulled from `parts` until it
    /// returns `None` (t55). An error from `parts` aborts the write and leaves no
    /// object. The default collects the parts and calls [`UploadBackend::put`];
    /// a backend that can write incrementally (as [`FsUploadBackend`] does)
    /// overrides it so memory stays at one part.
    async fn put_parts(
        &self,
        account_key: &str,
        storage_key: &str,
        parts: &mut (dyn PartSource + Send),
    ) -> Result<(), UploadError> {
        let mut all = Vec::new();
        while let Some(part) = parts.next_part().await? {
            all.extend_from_slice(&part);
        }
        self.put(account_key, storage_key, all).await
    }

    /// Read an object in pieces (t55), or `None` if it does not exist. The
    /// default reads it whole with [`UploadBackend::get`]; [`FsUploadBackend`]
    /// reads the file piece by piece.
    async fn open_parts(
        &self,
        account_key: &str,
        storage_key: &str,
    ) -> Result<Option<Box<dyn PartReader>>, UploadError> {
        Ok(self
            .get(account_key, storage_key)
            .await?
            .map(|data| Box::new(WholeReader { data, at: 0 }) as Box<dyn PartReader>))
    }
}

/// Reads an object in pieces ([`UploadBackend::open_parts`]).
#[async_trait]
pub trait PartReader: Send {
    /// Up to `len` further bytes (fewer only at the end); empty at the end.
    async fn read(&mut self, len: usize) -> Result<Vec<u8>, UploadError>;
}

/// [`PartReader`] over bytes already in memory (the default `open_parts`).
struct WholeReader {
    data: Vec<u8>,
    at: usize,
}

#[async_trait]
impl PartReader for WholeReader {
    async fn read(&mut self, len: usize) -> Result<Vec<u8>, UploadError> {
        let end = self.data.len().min(self.at.saturating_add(len));
        let part = self.data[self.at..end].to_vec();
        self.at = end;
        Ok(part)
    }
}

/// The parts of an object being written by [`UploadBackend::put_parts`].
#[async_trait]
pub trait PartSource {
    /// The next part, or `None` when the object is complete.
    async fn next_part(&mut self) -> Result<Option<Vec<u8>>, UploadError>;
}

/// True iff `s` is a non-empty, bounded, pure lowercase-hex token — the only shape a
/// server-minted `storage_key`/`account_key` ever takes. Rejects `.`/`/`/`\`/`:`,
/// uppercase, absolute paths, and `..` traversal by construction.
fn is_hex_token(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Lowercase-hex encode raw bytes (local helper; mirrors `seal`'s private encoder).
fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// The server-minted per-account subdirectory token: lowercase-hex SHA-256 of the
/// account id. Deterministic (so an account's objects group under one directory),
/// fixed-length, and path-safe regardless of what characters the account id contains.
fn account_dir_token(account_id: &str) -> String {
    let mut h = Sha256::new();
    h.update(account_id.as_bytes());
    hex_lower(&h.finalize())
}

/// The default filesystem [`UploadBackend`]: sealed objects laid out
/// `root/<account_key>/<storage_key>`. Construction (the `root` upload directory) is
/// deployment-configured in `mw-server` (`MW_UPLOAD_DIR`).
pub struct FsUploadBackend {
    root: PathBuf,
}

impl FsUploadBackend {
    /// Construct a filesystem backend rooted at `root` (the configured upload dir).
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Validate both keys and join the object path under `root`. Returns
    /// [`UploadError::InvalidKey`] for anything that is not a pure-hex token, so a
    /// traversal-y key can never escape the root.
    fn object_path(&self, account_key: &str, storage_key: &str) -> Result<PathBuf, UploadError> {
        if !is_hex_token(account_key) || !is_hex_token(storage_key) {
            return Err(UploadError::InvalidKey);
        }
        Ok(self.root.join(account_key).join(storage_key))
    }
}

#[async_trait]
impl UploadBackend for FsUploadBackend {
    fn kind(&self) -> &'static str {
        "fs"
    }

    async fn put(
        &self,
        account_key: &str,
        storage_key: &str,
        sealed: Vec<u8>,
    ) -> Result<(), UploadError> {
        let path = self.object_path(account_key, storage_key)?;
        let dir = path
            .parent()
            .expect("a validated object path always has a parent under root");
        std::fs::create_dir_all(dir).map_err(|e| UploadError::Io(e.to_string()))?;
        // Write to a temp sibling then rename, so a concurrent reader never sees a
        // half-written object.
        let tmp = dir.join(format!("{storage_key}.tmp"));
        std::fs::write(&tmp, &sealed).map_err(|e| UploadError::Io(e.to_string()))?;
        std::fs::rename(&tmp, &path).map_err(|e| UploadError::Io(e.to_string()))?;
        Ok(())
    }

    async fn get(
        &self,
        account_key: &str,
        storage_key: &str,
    ) -> Result<Option<Vec<u8>>, UploadError> {
        let path = self.object_path(account_key, storage_key)?;
        match std::fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(UploadError::Io(e.to_string())),
        }
    }

    async fn delete(&self, account_key: &str, storage_key: &str) -> Result<(), UploadError> {
        let path = self.object_path(account_key, storage_key)?;
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(UploadError::Io(e.to_string())),
        }
    }

    async fn put_parts(
        &self,
        account_key: &str,
        storage_key: &str,
        parts: &mut (dyn PartSource + Send),
    ) -> Result<(), UploadError> {
        use std::io::Write;
        let path = self.object_path(account_key, storage_key)?;
        let dir = path
            .parent()
            .expect("a validated object path always has a parent under root");
        std::fs::create_dir_all(dir).map_err(|e| UploadError::Io(e.to_string()))?;
        // Same temp-then-rename as `put`; an aborted write removes the temp file.
        let tmp = dir.join(format!("{storage_key}.tmp"));
        let written = async {
            let mut file = std::fs::File::create(&tmp).map_err(|e| UploadError::Io(e.to_string()))?;
            while let Some(part) = parts.next_part().await? {
                file.write_all(&part).map_err(|e| UploadError::Io(e.to_string()))?;
            }
            file.sync_all().map_err(|e| UploadError::Io(e.to_string()))
        }
        .await;
        match written.and_then(|()| std::fs::rename(&tmp, &path).map_err(|e| UploadError::Io(e.to_string()))) {
            Ok(()) => Ok(()),
            Err(e) => {
                let _ = std::fs::remove_file(&tmp);
                Err(e)
            }
        }
    }

    async fn open_parts(
        &self,
        account_key: &str,
        storage_key: &str,
    ) -> Result<Option<Box<dyn PartReader>>, UploadError> {
        let path = self.object_path(account_key, storage_key)?;
        match std::fs::File::open(&path) {
            Ok(file) => Ok(Some(Box::new(FileReader(file)))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(UploadError::Io(e.to_string())),
        }
    }
}

/// [`PartReader`] over a file of the filesystem backend.
struct FileReader(std::fs::File);

#[async_trait]
impl PartReader for FileReader {
    async fn read(&mut self, len: usize) -> Result<Vec<u8>, UploadError> {
        use std::io::Read;
        let mut part = vec![0u8; len];
        let mut filled = 0;
        while filled < len {
            match self.0.read(&mut part[filled..]) {
                Ok(0) => break,
                Ok(n) => filled += n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(e) => return Err(UploadError::Io(e.to_string())),
            }
        }
        part.truncate(filled);
        Ok(part)
    }
}

/// The fail-closed default backend used by a store that has not had a real backend
/// injected. Every operation errors, so a misconfigured deployment fails loudly rather
/// than silently accepting (and losing) upload bytes.
struct FailClosedBackend;

#[async_trait]
impl UploadBackend for FailClosedBackend {
    fn kind(&self) -> &'static str {
        "unconfigured"
    }
    async fn put(&self, _: &str, _: &str, _: Vec<u8>) -> Result<(), UploadError> {
        Err(UploadError::NotConfigured)
    }
    async fn get(&self, _: &str, _: &str) -> Result<Option<Vec<u8>>, UploadError> {
        Err(UploadError::NotConfigured)
    }
    async fn delete(&self, _: &str, _: &str) -> Result<(), UploadError> {
        Err(UploadError::NotConfigured)
    }
}

/// A spool or upload read back one plaintext chunk at a time
/// ([`Store::spool_reader`]).
pub struct UploadReader {
    mode: ReadMode,
    /// The next sealed chunk, read ahead to know whether the current one is last.
    pending: Option<Vec<u8>>,
}

enum ReadMode {
    Stream { opener: seal::StreamOpener, parts: Box<dyn PartReader> },
    Whole(Option<Vec<u8>>),
}

impl UploadReader {
    /// The next plaintext chunk, or `None` at the end. A changed, reordered,
    /// truncated or extended object is an error, never silently short.
    pub async fn next_chunk(&mut self) -> Result<Option<Vec<u8>>, StoreError> {
        match &mut self.mode {
            ReadMode::Whole(data) => Ok(data.take()),
            ReadMode::Stream { opener, parts } => {
                let current = match self.pending.take() {
                    Some(chunk) => chunk,
                    None => parts.read(seal::StreamOpener::FULL).await?,
                };
                if current.is_empty() {
                    return Ok(None);
                }
                // A full-size chunk is followed by at least the (possibly
                // empty) last chunk; anything shorter is the last one.
                let last = current.len() < seal::StreamOpener::FULL;
                if !last {
                    let next = parts.read(seal::StreamOpener::FULL).await?;
                    if next.is_empty() {
                        return Err(StoreError::Seal(crate::SealError::Truncated));
                    }
                    self.pending = Some(next);
                }
                let plain = opener.open_chunk(&current, last)?;
                if last {
                    self.mode = ReadMode::Whole(None);
                    if plain.is_empty() {
                        return Ok(None);
                    }
                }
                Ok(Some(plain))
            }
        }
    }
}

/// The per-account directory token for spools, distinct from uploads.
fn spool_dir_token(account_id: &str) -> String {
    account_dir_token(&format!("spool\u{0}{account_id}"))
}

/// What [`Store::put_upload_stream`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamedUpload {
    /// Sealed and stored: the minted `U`+64-hex blob id and the plaintext size.
    Stored { blob_id: String, size: u64 },
    /// The body went past the cap; nothing was stored.
    TooLarge,
    /// The body could not be read (the client went away); nothing was stored.
    BodyFailed,
}

enum Refused {
    TooLarge,
    Body,
}

/// Pulls the request body, seals it chunk by chunk, and hands the sealed parts
/// to the backend: the header first, then one part per [`seal::STREAM_CHUNK`].
struct BodyParts<'a, S> {
    body: &'a mut S,
    sealer: seal::StreamSealer,
    header: Option<Vec<u8>>,
    buf: Vec<u8>,
    total: u64,
    cap: u64,
    done: bool,
    refused: Option<Refused>,
}

#[async_trait]
impl<S, T, E> PartSource for BodyParts<'_, S>
where
    S: futures_util::Stream<Item = Result<T, E>> + Unpin + Send,
    T: AsRef<[u8]> + Send,
    E: std::fmt::Display + Send,
{
    async fn next_part(&mut self) -> Result<Option<Vec<u8>>, UploadError> {
        use futures_util::StreamExt;
        if let Some(header) = self.header.take() {
            return Ok(Some(header));
        }
        loop {
            if self.done {
                return Ok(None);
            }
            if self.buf.len() >= seal::STREAM_CHUNK {
                let rest = self.buf.split_off(seal::STREAM_CHUNK);
                let chunk = std::mem::replace(&mut self.buf, rest);
                return self
                    .sealer
                    .seal_chunk(&chunk, false)
                    .map(Some)
                    .map_err(|e| UploadError::Io(e.to_string()));
            }
            match self.body.next().await {
                Some(Ok(data)) => {
                    let data = data.as_ref();
                    self.total += data.len() as u64;
                    if self.total > self.cap {
                        self.refused = Some(Refused::TooLarge);
                        return Err(UploadError::Io("upload exceeds the size limit".into()));
                    }
                    self.buf.extend_from_slice(data);
                }
                Some(Err(e)) => {
                    self.refused = Some(Refused::Body);
                    return Err(UploadError::Io(format!("upload body: {e}")));
                }
                None => {
                    self.done = true;
                    let last = std::mem::take(&mut self.buf);
                    return self
                        .sealer
                        .seal_chunk(&last, true)
                        .map(Some)
                        .map_err(|e| UploadError::Io(e.to_string()));
                }
            }
        }
    }
}

/// The default (fail-closed) backend for a freshly-opened store; `mw-server` replaces
/// it with a real [`FsUploadBackend`] via [`Store::with_upload_backend`].
pub(crate) fn fail_closed_backend() -> Arc<dyn UploadBackend> {
    Arc::new(FailClosedBackend)
}

/// A resolved upload: metadata + the unsealed plaintext bytes. Returned by
/// [`Store::get_upload`] for `Engine::fetch_blob` to turn into a MIME attachment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upload {
    pub blob_id: String,
    pub account_id: String,
    pub content_type: String,
    /// Plaintext byte length (as recorded at upload time).
    pub size: i64,
    pub bytes: Vec<u8>,
}

impl Store {
    /// Whether upload storage was injected at startup. This is configuration
    /// readiness, not a promise that the filesystem will accept a future write.
    pub fn uploads_configured(&self) -> bool {
        self.uploads.kind() != "unconfigured"
    }

    /// Seal `bytes`, write the sealed object to the storage backend, record the 0012
    /// metadata row, and return the minted `U`+64-hex `blob_id`.
    pub async fn put_upload(
        &self,
        account_id: &str,
        content_type: &str,
        bytes: &[u8],
    ) -> Result<String, StoreError> {
        let storage_key = seal::random_token();
        let blob_id = format!("U{storage_key}");
        let account_key = account_dir_token(account_id);
        let sealed = self.key.seal(bytes)?;
        self.uploads.put(&account_key, &storage_key, sealed).await?;
        let now = Utc::now().to_rfc3339();
        q("INSERT INTO uploaded_blobs
                 (blob_id, account_id, content_type, size, storage_key, backend_kind, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)")
        .bind(&blob_id)
        .bind(account_id)
        .bind(content_type)
        .bind(bytes.len() as i64)
        .bind(&storage_key)
        .bind(self.uploads.kind())
        .bind(&now)
        .execute(&self.backend)
        .await?;
        Ok(blob_id)
    }

    /// Seal and store an upload read from `body`, one chunk at a time (t55): the
    /// plaintext is sealed in [`seal::STREAM_CHUNK`] pieces and written as it
    /// arrives, so memory is one chunk rather than the whole upload plus a whole
    /// sealed copy. More than `cap` bytes stops the read and stores nothing
    /// ([`StreamedUpload::TooLarge`]); the caller still owns `body` and may drain
    /// it. A body read error stores nothing either ([`StreamedUpload::BodyFailed`]).
    pub async fn put_upload_stream<S, T, E>(
        &self,
        account_id: &str,
        content_type: &str,
        body: &mut S,
        cap: u64,
    ) -> Result<StreamedUpload, StoreError>
    where
        S: futures_util::Stream<Item = Result<T, E>> + Unpin + Send,
        T: AsRef<[u8]> + Send,
        E: std::fmt::Display + Send,
    {
        let storage_key = seal::random_token();
        let blob_id = format!("U{storage_key}");
        let account_key = account_dir_token(account_id);
        let total = match self.write_streamed(&account_key, &storage_key, body, cap).await? {
            Ok(total) => total,
            Err(refused) => return Ok(refused),
        };
        let now = Utc::now().to_rfc3339();
        let inserted = q("INSERT INTO uploaded_blobs
                 (blob_id, account_id, content_type, size, storage_key, backend_kind, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)")
        .bind(&blob_id)
        .bind(account_id)
        .bind(content_type)
        .bind(i64::try_from(total).unwrap_or(i64::MAX))
        .bind(&storage_key)
        .bind(self.uploads.kind())
        .bind(&now)
        .execute(&self.backend)
        .await;
        if let Err(e) = inserted {
            let _ = self.uploads.delete(&account_key, &storage_key).await;
            return Err(e.into());
        }
        Ok(StreamedUpload::Stored { blob_id, size: total })
    }

    /// Seal `body` chunk by chunk into the object `(account_key, storage_key)`.
    /// `Ok(Err(..))` is a refusal (over `cap`, or the body failed): nothing kept.
    async fn write_streamed<S, T, E>(
        &self,
        account_key: &str,
        storage_key: &str,
        body: &mut S,
        cap: u64,
    ) -> Result<Result<u64, StreamedUpload>, StoreError>
    where
        S: futures_util::Stream<Item = Result<T, E>> + Unpin + Send,
        T: AsRef<[u8]> + Send,
        E: std::fmt::Display + Send,
    {
        let (sealer, header) = self.key.stream_sealer();
        let mut source = BodyParts {
            body,
            sealer,
            header: Some(header),
            buf: Vec::with_capacity(seal::STREAM_CHUNK),
            total: 0,
            cap,
            done: false,
            refused: None,
        };
        let written = self.uploads.put_parts(account_key, storage_key, &mut source).await;
        let (total, refused) = (source.total, source.refused.take());
        match (written, refused) {
            (_, Some(Refused::TooLarge)) => Ok(Err(StreamedUpload::TooLarge)),
            (_, Some(Refused::Body)) => Ok(Err(StreamedUpload::BodyFailed)),
            (Err(e), None) => Err(e.into()),
            (Ok(()), None) => Ok(Ok(total)),
        }
    }

    /// Spool a request body to the upload backend, sealed, with no database row
    /// (t55): a work file a route reads back in pieces ([`Store::spool_reader`])
    /// and removes ([`Store::spool_delete`]) itself. Returns the spool key and
    /// the plaintext size, or the refusal. Spools live under their own per-account
    /// directory, apart from uploads.
    pub async fn spool_put_stream<S, T, E>(
        &self,
        account_id: &str,
        body: &mut S,
        cap: u64,
    ) -> Result<Result<(String, u64), StreamedUpload>, StoreError>
    where
        S: futures_util::Stream<Item = Result<T, E>> + Unpin + Send,
        T: AsRef<[u8]> + Send,
        E: std::fmt::Display + Send,
    {
        let key = seal::random_token();
        Ok(self
            .write_streamed(&spool_dir_token(account_id), &key, body, cap)
            .await?
            .map(|size| (key, size)))
    }

    /// Read a spool back, one chunk at a time.
    pub async fn spool_reader(&self, account_id: &str, key: &str) -> Result<Option<UploadReader>, StoreError> {
        self.reader(&spool_dir_token(account_id), key).await
    }

    /// Remove a spool. A missing spool is not an error.
    pub async fn spool_delete(&self, account_id: &str, key: &str) -> Result<(), StoreError> {
        Ok(self.uploads.delete(&spool_dir_token(account_id), key).await?)
    }

    async fn reader(&self, account_key: &str, storage_key: &str) -> Result<Option<UploadReader>, StoreError> {
        let Some(mut parts) = self.uploads.open_parts(account_key, storage_key).await? else {
            return Ok(None);
        };
        let header = parts.read(crate::ServerKey::STREAM_HEADER).await?;
        let mode = match self.key.stream_opener(&header) {
            Some(opener) => ReadMode::Stream { opener, parts },
            None => {
                // One-shot seal: read the rest and open it whole.
                let mut all = header;
                loop {
                    let part = parts.read(seal::STREAM_CHUNK).await?;
                    if part.is_empty() {
                        break;
                    }
                    all.extend_from_slice(&part);
                }
                ReadMode::Whole(Some(self.key.open_owned(all)?))
            }
        };
        Ok(Some(UploadReader { mode, pending: None }))
    }

    /// Resolve an upload for `account_id`, unsealing the object bytes. Account-scoped:
    /// a `blob_id` owned by a different account (or an unknown id) resolves to `None`,
    /// so one account can never read another's upload.
    pub async fn get_upload(
        &self,
        account_id: &str,
        blob_id: &str,
    ) -> Result<Option<Upload>, StoreError> {
        self.get_upload_limited(account_id, blob_id, usize::MAX)
            .await
    }

    /// Account-scoped immutable plaintext size, without reading the object.
    pub async fn upload_size(
        &self,
        account_id: &str,
        blob_id: &str,
    ) -> Result<Option<i64>, StoreError> {
        Ok(
            q("SELECT size FROM uploaded_blobs WHERE blob_id = ?1 AND account_id = ?2")
                .bind(blob_id)
                .bind(account_id)
                .fetch_optional(&self.backend)
                .await?
                .map(|row| row.get_i64("size")),
        )
    }

    /// Refuse oversized uploads from immutable, account-owned plaintext-size
    /// metadata before reading the sealed object. The limit does not include
    /// encryption envelope overhead. Recheck actual plaintext after unsealing.
    pub async fn get_upload_limited(
        &self,
        account_id: &str,
        blob_id: &str,
        maximum: usize,
    ) -> Result<Option<Upload>, StoreError> {
        let row = q("SELECT blob_id, account_id, content_type, size, storage_key
                       FROM uploaded_blobs WHERE blob_id = ?1 AND account_id = ?2 AND size >= 0 AND size <= ?3")
        .bind(blob_id)
        .bind(account_id)
        .bind(i64::try_from(maximum).unwrap_or(i64::MAX))
        .fetch_optional(&self.backend)
        .await?;
        let Some(r) = row else { return Ok(None) };
        let storage_key = r.get_string("storage_key");
        let account_key = account_dir_token(account_id);
        let sealed = match self.uploads.get(&account_key, &storage_key).await? {
            Some(s) => s,
            None => return Ok(None),
        };
        let bytes = self.key.open_upload(sealed)?;
        if bytes.len() > maximum || i64::try_from(bytes.len()).ok() != Some(r.get_i64("size")) {
            return Ok(None);
        }
        Ok(Some(Upload {
            blob_id: r.get_string("blob_id"),
            account_id: r.get_string("account_id"),
            content_type: r.get_string("content_type"),
            size: r.get_i64("size"),
            bytes,
        }))
    }

    /// Delete one upload (row + backend object). Account-scoped; a no-op if the id is
    /// unknown or owned by another account.
    pub async fn delete_upload(&self, account_id: &str, blob_id: &str) -> Result<(), StoreError> {
        let row =
            q("SELECT storage_key FROM uploaded_blobs WHERE blob_id = ?1 AND account_id = ?2")
                .bind(blob_id)
                .bind(account_id)
                .fetch_optional(&self.backend)
                .await?;
        let Some(r) = row else { return Ok(()) };
        let storage_key = r.get_string("storage_key");
        let account_key = account_dir_token(account_id);
        self.uploads.delete(&account_key, &storage_key).await?;
        q("DELETE FROM uploaded_blobs WHERE blob_id = ?1 AND account_id = ?2")
            .bind(blob_id)
            .bind(account_id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Uploads older than `older_than` that nothing stored on this server still
    /// refers to, as `(account_id, blob_id, storage_key)`.
    ///
    /// An upload is kept when any of these names its blob ID:
    /// - an account draft, saved or with a save in progress or uncertain. The
    ///   draft's editor state is sealed, so it is opened here and searched for
    ///   upload IDs. A draft that cannot be opened keeps every upload of its
    ///   account.
    /// - a contact card's photo (`contacts.photo_blob_id`).
    /// - a submission's `email_id`. Submissions name composed messages, whose
    ///   attachments are already copied into the message, so this match is a
    ///   guard rather than a known reference.
    ///
    /// Copies the server cannot see are not covered: drafts kept only in a
    /// browser's local storage and the offline send queue of a client.
    async fn sweepable_uploads(
        &self,
        older_than: chrono::Duration,
    ) -> Result<Vec<(String, String, String)>, StoreError> {
        use std::collections::HashSet;
        let cutoff = (Utc::now() - older_than).to_rfc3339();
        let rows = q(
            "SELECT blob_id, account_id, storage_key FROM uploaded_blobs WHERE created_at < ?1",
        )
        .bind(&cutoff)
        .fetch_all(&self.backend)
        .await?;
        if rows.is_empty() {
            return Ok(Vec::new());
        }
        let mut kept: HashSet<String> = HashSet::new();
        let mut kept_accounts: HashSet<String> = HashSet::new();
        for r in q("SELECT c.photo_blob_id AS blob_id FROM contacts c WHERE c.photo_blob_id LIKE 'U%'")
            .fetch_all(&self.backend)
            .await?
        {
            kept.insert(r.get_string("blob_id"));
        }
        for r in q("SELECT email_id AS blob_id FROM submissions WHERE email_id LIKE 'U%'")
            .fetch_all(&self.backend)
            .await?
        {
            kept.insert(r.get_string("blob_id"));
        }
        let drafts = q("SELECT owner, id, account_id FROM account_draft WHERE request_hash<>''")
            .fetch_all(&self.backend)
            .await?;
        for d in &drafts {
            let account = d.get_string("account_id");
            match self
                .account_draft(&d.get_string("owner"), &d.get_string("id"))
                .await
            {
                Ok(Some(draft)) => {
                    for bytes in [&draft.state, &draft.summary] {
                        kept.extend(upload_ids_in(bytes));
                    }
                }
                Ok(None) => {}
                Err(_) => {
                    kept_accounts.insert(account);
                }
            }
        }
        Ok(rows
            .iter()
            .map(|r| {
                (
                    r.get_string("account_id"),
                    r.get_string("blob_id"),
                    r.get_string("storage_key"),
                )
            })
            .filter(|(account, blob, _)| !kept.contains(blob) && !kept_accounts.contains(account))
            .collect())
    }

    /// How many uploads [`Store::sweep_uploads`] would remove now, without
    /// removing anything.
    pub async fn count_sweepable_uploads(
        &self,
        older_than: chrono::Duration,
    ) -> Result<u64, StoreError> {
        Ok(self.sweepable_uploads(older_than).await?.len() as u64)
    }

    /// Remove uploads older than `older_than` (by `created_at`) that nothing on
    /// this server refers to (see [`Store::sweepable_uploads`]), both the backend
    /// object and the row. Returns the number of rows removed. A backend object
    /// that is already gone does not stop the row from being removed.
    pub async fn sweep_uploads(&self, older_than: chrono::Duration) -> Result<u64, StoreError> {
        let cutoff = (Utc::now() - older_than).to_rfc3339();
        let mut n = 0;
        for (account, blob, storage_key) in self.sweepable_uploads(older_than).await? {
            let account_key = account_dir_token(&account);
            // Best-effort object removal: a missing/failed object must not block the row
            // reclamation that is the sweep's actual job.
            let _ = self.uploads.delete(&account_key, &storage_key).await;
            n += q("DELETE FROM uploaded_blobs WHERE blob_id = ?1 AND account_id = ?2 AND created_at < ?3")
                .bind(&blob)
                .bind(&account)
                .bind(&cutoff)
                .execute(&self.backend)
                .await?;
        }
        Ok(n)
    }
}

/// Every upload blob ID (`U` + 64 lowercase hex) that appears in `bytes`.
fn upload_ids_in(bytes: &[u8]) -> Vec<String> {
    let hex = |b: &u8| b.is_ascii_digit() || (b'a'..=b'f').contains(b);
    let mut out = Vec::new();
    let mut i = 0;
    while i + 65 <= bytes.len() {
        if bytes[i] == b'U' && bytes[i + 1..i + 65].iter().all(hex) {
            out.push(String::from_utf8_lossy(&bytes[i..i + 65]).into_owned());
            i += 65;
        } else {
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;
    use std::path::Path;

    /// A unique temp directory for one test's FS backend (avoids a `tempfile`
    /// dev-dependency, mirroring the file-backed store test in `lib.rs`).
    fn temp_root(tag: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("mw-upload-{tag}-{unique}"))
    }

    async fn fs_store(root: &Path) -> Store {
        Store::open_in_memory(ServerKey::generate())
            .await
            .unwrap()
            .with_upload_backend(Arc::new(FsUploadBackend::new(root.to_path_buf())))
    }

    /// A backend that records the largest part it was handed while writing.
    #[derive(Default)]
    struct Recording {
        objects: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
        largest_part: std::sync::atomic::AtomicUsize,
    }
    #[async_trait]
    impl UploadBackend for Recording {
        fn kind(&self) -> &'static str {
            "recording"
        }
        async fn put(&self, _: &str, key: &str, sealed: Vec<u8>) -> Result<(), UploadError> {
            self.objects.lock().unwrap().insert(key.into(), sealed);
            Ok(())
        }
        async fn get(&self, _: &str, key: &str) -> Result<Option<Vec<u8>>, UploadError> {
            Ok(self.objects.lock().unwrap().get(key).cloned())
        }
        async fn delete(&self, _: &str, key: &str) -> Result<(), UploadError> {
            self.objects.lock().unwrap().remove(key);
            Ok(())
        }
        async fn put_parts(
            &self,
            _: &str,
            key: &str,
            parts: &mut (dyn PartSource + Send),
        ) -> Result<(), UploadError> {
            let mut all = Vec::new();
            while let Some(part) = parts.next_part().await? {
                self.largest_part.fetch_max(part.len(), std::sync::atomic::Ordering::SeqCst);
                all.extend_from_slice(&part);
            }
            self.objects.lock().unwrap().insert(key.into(), all);
            Ok(())
        }
    }

    fn pieces(total: usize, piece: usize) -> (Vec<u8>, Vec<Result<Vec<u8>, String>>) {
        let data: Vec<u8> = (0..total).map(|i| (i % 251) as u8).collect();
        let parts = data.chunks(piece).map(|c| Ok(c.to_vec())).collect();
        (data, parts)
    }

    /// t55: an upload is sealed and written one chunk at a time and reads back
    /// whole; the largest part handed to the backend is one sealed chunk.
    #[tokio::test]
    async fn a_streamed_upload_is_sealed_in_chunks_and_reads_back() {
        let backend = Arc::new(Recording::default());
        let store = Store::open_in_memory(ServerKey::generate())
            .await
            .unwrap()
            .with_upload_backend(backend.clone());
        for total in [0usize, 1, seal::STREAM_CHUNK, seal::STREAM_CHUNK + 1, 5 * 1024 * 1024 + 123] {
            let (data, parts) = pieces(total, 7 * 1024);
            let mut body = futures_util::stream::iter(parts);
            let outcome = store
                .put_upload_stream("acct-a", "application/pdf", &mut body, 50_000_000)
                .await
                .unwrap();
            let StreamedUpload::Stored { blob_id, size } = outcome else { panic!("{total}: {outcome:?}") };
            assert_eq!(size as usize, total);
            let got = store.get_upload("acct-a", &blob_id).await.unwrap().unwrap();
            assert_eq!(got.bytes, data, "{total} bytes round trip");
            assert_eq!(got.content_type, "application/pdf");
            assert!(store.get_upload("acct-b", &blob_id).await.unwrap().is_none(), "account-scoped");
        }
        let largest = backend.largest_part.load(std::sync::atomic::Ordering::SeqCst);
        assert!(largest <= seal::STREAM_CHUNK + 16, "a part held more than one chunk: {largest}");
    }

    #[tokio::test]
    async fn a_streamed_upload_over_the_cap_stores_nothing() {
        let root = temp_root("stream-cap");
        let store = fs_store(&root).await;
        let (_, parts) = pieces(300 * 1024, 64 * 1024);
        let mut body = futures_util::stream::iter(parts);
        let outcome = store
            .put_upload_stream("acct-a", "text/plain", &mut body, 200 * 1024)
            .await
            .unwrap();
        assert_eq!(outcome, StreamedUpload::TooLarge);
        let left: Vec<_> = walk(&root);
        assert!(left.is_empty(), "no object or temp file left: {left:?}");
        let mut failing = futures_util::stream::iter(vec![Ok(vec![1u8; 10]), Err("reset".to_string())]);
        let outcome = store
            .put_upload_stream("acct-a", "text/plain", &mut failing, 200 * 1024)
            .await
            .unwrap();
        assert_eq!(outcome, StreamedUpload::BodyFailed);
        assert!(walk(&root).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    fn walk(dir: &Path) -> Vec<PathBuf> {
        let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
        entries
            .flatten()
            .flat_map(|e| if e.path().is_dir() { walk(&e.path()) } else { vec![e.path()] })
            .collect()
    }

    /// A streamed object is authenticated as a whole: a changed byte, a dropped
    /// last chunk, swapped chunks or an extra chunk all fail to open; an
    /// object stored before (one-shot seal) still opens.
    #[test]
    fn a_tampered_streamed_object_fails_to_open_and_old_objects_still_open() {
        let key = ServerKey::generate();
        let (mut sealer, mut object) = key.stream_sealer();
        let chunks: Vec<Vec<u8>> = (0..3)
            .map(|i| sealer.seal_chunk(&vec![i as u8; seal::STREAM_CHUNK], false).unwrap())
            .collect();
        for c in &chunks {
            object.extend_from_slice(c);
        }
        object.extend_from_slice(&sealer.seal_chunk(b"tail", true).unwrap());
        let plain = key.open_upload(object.clone()).unwrap();
        assert_eq!(plain.len(), 3 * seal::STREAM_CHUNK + 4);
        let head = 4 + 19;
        let full = seal::STREAM_CHUNK + 16;
        let mut flipped = object.clone();
        flipped[head + 10] ^= 1;
        assert!(key.open_upload(flipped).is_err(), "changed byte");
        assert!(key.open_upload(object[..head + 3 * full].to_vec()).is_err(), "dropped last chunk");
        let mut swapped = object[..head].to_vec();
        swapped.extend_from_slice(&chunks[1]);
        swapped.extend_from_slice(&chunks[0]);
        swapped.extend_from_slice(&object[head + 2 * full..]);
        assert!(key.open_upload(swapped).is_err(), "reordered chunks");
        let mut extended = object.clone();
        extended.extend_from_slice(&chunks[0]);
        assert!(key.open_upload(extended).is_err(), "extra chunk");
        let one_shot = key.seal(b"stored before t55").unwrap();
        assert_eq!(key.open_upload(one_shot).unwrap(), b"stored before t55");
        let mut lookalike = b"MWU2".to_vec();
        lookalike.extend_from_slice(&key.seal(b"x").unwrap()[4..]);
        assert!(key.open_upload(lookalike).is_err(), "neither form: refused");
    }

    /// t55: a spool is written and read back one chunk at a time, with no row,
    /// and removed; a truncated spool is an error, never silently short.
    #[tokio::test]
    async fn a_spool_reads_back_in_chunks_and_detects_truncation() {
        let root = temp_root("spool");
        let store = fs_store(&root).await;
        let (data, parts) = pieces(3 * seal::STREAM_CHUNK + 777, 5 * 1024);
        let mut body = futures_util::stream::iter(parts);
        let (key, size) = store
            .spool_put_stream("acct-a", &mut body, 64 * 1024 * 1024)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(size as usize, data.len());
        assert!(store.upload_size("acct-a", &format!("U{key}")).await.unwrap().is_none(), "no row");
        let mut reader = store.spool_reader("acct-a", &key).await.unwrap().unwrap();
        let mut back = Vec::new();
        let mut largest = 0;
        while let Some(chunk) = reader.next_chunk().await.unwrap() {
            largest = largest.max(chunk.len());
            back.extend_from_slice(&chunk);
        }
        assert_eq!(back, data);
        assert!(largest <= seal::STREAM_CHUNK, "{largest}");
        assert!(store.spool_reader("acct-b", &key).await.unwrap().is_none(), "account-scoped");
        // Drop the last sealed chunk on disk: reading must fail, not end early.
        let file = walk(&root).into_iter().next().expect("the spool file");
        let bytes = std::fs::read(&file).unwrap();
        let full = seal::STREAM_CHUNK + 16;
        std::fs::write(&file, &bytes[..4 + 19 + 3 * full]).unwrap();
        let mut reader = store.spool_reader("acct-a", &key).await.unwrap().unwrap();
        let mut failed = false;
        loop {
            match reader.next_chunk().await {
                Ok(Some(_)) => {}
                Ok(None) => break,
                Err(_) => {
                    failed = true;
                    break;
                }
            }
        }
        assert!(failed, "a truncated spool must not read as complete");
        store.spool_delete("acct-a", &key).await.unwrap();
        assert!(walk(&root).is_empty(), "the spool is removed");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A one-shot object (stored before t55) reads back through the chunk reader.
    #[tokio::test]
    async fn the_chunk_reader_opens_one_shot_objects_too() {
        let store = Store::open_in_memory(ServerKey::generate())
            .await
            .unwrap()
            .with_upload_backend(Arc::new(Recording::default()));
        let blob = store.put_upload("acct-a", "text/plain", b"one-shot body").await.unwrap();
        let key = blob.trim_start_matches('U');
        let mut reader = store.reader(&account_dir_token("acct-a"), key).await.unwrap().unwrap();
        assert_eq!(reader.next_chunk().await.unwrap().unwrap(), b"one-shot body");
        assert!(reader.next_chunk().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn put_get_round_trips_bytes_and_type() {
        let root = temp_root("roundtrip");
        let store = fs_store(&root).await;
        let blob_id = store
            .put_upload("acct-a", "image/png", b"\x89PNG\r\n\x1a\n payload")
            .await
            .unwrap();
        assert!(blob_id.starts_with('U'), "upload id is U-prefixed");
        assert_eq!(blob_id.len(), 65, "U + 64-hex");

        let got = store.get_upload("acct-a", &blob_id).await.unwrap().unwrap();
        assert_eq!(got.content_type, "image/png");
        assert_eq!(got.bytes, b"\x89PNG\r\n\x1a\n payload");
        assert_eq!(got.size as usize, got.bytes.len());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn bounded_upload_read_checks_owner_metadata_and_actual_plaintext() {
        let root = temp_root("bounded");
        let store = fs_store(&root).await;
        let blob = store
            .put_upload("acct-a", "text/plain", b"payload")
            .await
            .unwrap();
        assert!(
            store
                .get_upload_limited("acct-b", &blob, 100)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .get_upload_limited("acct-a", &blob, 6)
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(
            store
                .get_upload_limited("acct-a", &blob, 7)
                .await
                .unwrap()
                .unwrap()
                .bytes,
            b"payload"
        );
        q("UPDATE uploaded_blobs SET size = 1 WHERE blob_id = ?1")
            .bind(&blob)
            .execute(&store.backend)
            .await
            .unwrap();
        assert!(
            store
                .get_upload_limited("acct-a", &blob, 1)
                .await
                .unwrap()
                .is_none()
        );
        q("UPDATE uploaded_blobs SET size = 7 WHERE blob_id = ?1")
            .bind(&blob)
            .execute(&store.backend)
            .await
            .unwrap();
        // An oversized object must be refused before corrupt ciphertext can be opened.
        for directory in std::fs::read_dir(&root).unwrap() {
            for object in std::fs::read_dir(directory.unwrap().path()).unwrap() {
                std::fs::write(object.unwrap().path(), b"not a sealed object").unwrap();
            }
        }
        assert!(
            store
                .get_upload_limited("acct-a", &blob, 6)
                .await
                .unwrap()
                .is_none()
        );
        assert!(store.get_upload_limited("acct-a", &blob, 7).await.is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn sealed_object_on_disk_is_not_plaintext() {
        let root = temp_root("sealed");
        let store = fs_store(&root).await;
        let secret = b"the-quick-brown-fox-secret-marker";
        let blob_id = store
            .put_upload("acct-a", "text/plain", secret)
            .await
            .unwrap();

        // Walk the FS backend root and confirm no on-disk object carries the plaintext
        // marker — the bytes are sealed before they ever hit disk.
        let mut found_object = false;
        for account_dir in std::fs::read_dir(&root).unwrap() {
            for obj in std::fs::read_dir(account_dir.unwrap().path()).unwrap() {
                found_object = true;
                let bytes = std::fs::read(obj.unwrap().path()).unwrap();
                assert!(
                    !bytes.windows(secret.len()).any(|w| w == secret),
                    "upload bytes must be sealed, never plaintext at rest"
                );
            }
        }
        assert!(found_object, "the sealed object was written to disk");
        // And it still round-trips through the key.
        let got = store.get_upload("acct-a", &blob_id).await.unwrap().unwrap();
        assert_eq!(got.bytes, secret);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn get_is_account_scoped() {
        let root = temp_root("scope");
        let store = fs_store(&root).await;
        let blob_id = store
            .put_upload("acct-a", "text/plain", b"hi")
            .await
            .unwrap();
        // Account B cannot resolve account A's blob id.
        assert!(
            store
                .get_upload("acct-b", &blob_id)
                .await
                .unwrap()
                .is_none()
        );
        // The owner still can.
        assert!(
            store
                .get_upload("acct-a", &blob_id)
                .await
                .unwrap()
                .is_some()
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn traversal_key_is_refused_by_backend() {
        let root = temp_root("traversal");
        let backend = FsUploadBackend::new(root.clone());
        // A separator/traversal-bearing or absolute or uppercase key is rejected before
        // any path join.
        for bad in [
            "..",
            "../escape",
            "a/b",
            "a\\b",
            "/etc/passwd",
            "C0FFEE",
            "spaced key",
        ] {
            assert!(
                matches!(
                    backend.put("aa", bad, vec![1, 2, 3]).await,
                    Err(UploadError::InvalidKey)
                ),
                "key {bad:?} must be refused"
            );
            assert!(matches!(
                backend.get("aa", bad).await,
                Err(UploadError::InvalidKey)
            ));
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn fail_closed_without_backend() {
        // A store with no injected backend must fail loudly rather than drop bytes.
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let err = store.put_upload("acct-a", "text/plain", b"x").await;
        assert!(matches!(
            err,
            Err(StoreError::Upload(UploadError::NotConfigured))
        ));
    }

    #[tokio::test]
    async fn sweep_reclaims_only_aged_rows_and_removes_objects() {
        let root = temp_root("sweep");
        let store = fs_store(&root).await;
        let fresh = store
            .put_upload("acct-a", "text/plain", b"fresh")
            .await
            .unwrap();
        let aged = store
            .put_upload("acct-a", "text/plain", b"aged")
            .await
            .unwrap();

        // Backdate the `aged` row well past the cutoff.
        let old = (Utc::now() - chrono::Duration::hours(48)).to_rfc3339();
        q("UPDATE uploaded_blobs SET created_at = ?1 WHERE blob_id = ?2")
            .bind(&old)
            .bind(&aged)
            .execute(store.backend())
            .await
            .unwrap();

        // Sweep everything older than 24h: reclaims exactly the aged row.
        let reclaimed = store
            .sweep_uploads(chrono::Duration::hours(24))
            .await
            .unwrap();
        assert_eq!(reclaimed, 1);
        assert!(store.get_upload("acct-a", &aged).await.unwrap().is_none());
        assert!(store.get_upload("acct-a", &fresh).await.unwrap().is_some());

        // Re-sweep is a no-op.
        assert_eq!(
            store
                .sweep_uploads(chrono::Duration::hours(24))
                .await
                .unwrap(),
            0
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Uploads named by an account draft (in progress or saved) or a contact
    /// photo survive a sweep whose cutoff is past every upload; an upload
    /// nothing names is removed, and the dry-run count agrees with the sweep.
    #[tokio::test]
    async fn sweep_keeps_uploads_still_referenced() {
        use crate::account_drafts::{DraftClaim, DraftSave};
        use crate::{AccountKind, Credentials, NewAccount};
        let root = temp_root("referenced");
        let store = fs_store(&root).await;
        let account = store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "upload.test",
                    port: 993,
                    tls: "implicit",
                    username: "a@upload.test",
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: "a@upload.test".into(),
                    password: "synthetic-only".into(),
                },
            )
            .await
            .unwrap();
        let in_draft = store.put_upload(&account, "text/plain", b"draft").await.unwrap();
        let photo = store.put_upload(&account, "image/png", b"photo").await.unwrap();
        let unused = store.put_upload(&account, "text/plain", b"unused").await.unwrap();

        let state = serde_json::to_vec(&serde_json::json!({
            "attachments": [{"blobId": in_draft, "name": "a.txt", "size": 5}]
        }))
        .unwrap();
        let claim = store
            .claim_account_draft(&DraftSave {
                owner: "realm-owner",
                account_id: &account,
                id: "draft-1",
                expected_revision: None,
                operation_id: "op-1",
                state: &state,
                summary: b"{}",
                now: 1,
            })
            .await
            .unwrap();
        assert!(matches!(claim, DraftClaim::Claimed));
        q("INSERT INTO address_books(id,account_id,name) VALUES('book-1',?1,'Contacts')")
            .bind(&account)
            .execute(store.backend())
            .await
            .unwrap();
        q("INSERT INTO contacts(id,address_book_id,uid,vcard_raw,photo_blob_id) VALUES('card-1','book-1','u1','BEGIN:VCARD\nVERSION:3.0\nEND:VCARD',?1)")
            .bind(&photo)
            .execute(store.backend())
            .await
            .unwrap();

        let past_all = chrono::Duration::seconds(-5);
        assert_eq!(store.count_sweepable_uploads(past_all).await.unwrap(), 1);
        assert_eq!(store.sweep_uploads(past_all).await.unwrap(), 1);
        assert!(store.get_upload(&account, &unused).await.unwrap().is_none());
        assert!(store.get_upload(&account, &in_draft).await.unwrap().is_some());
        assert!(store.get_upload(&account, &photo).await.unwrap().is_some());

        // The draft is saved and its state still names the upload: still kept.
        let saved = q("UPDATE account_draft SET operation_state='saved', revision='r1', sealed_state=sealed_pending_state WHERE owner='realm-owner'")
            .execute(store.backend())
            .await
            .unwrap();
        assert_eq!(saved, 1);
        // The pending seal is bound to the operation, not the revision, so the
        // copied bytes no longer open: the whole account is kept.
        assert_eq!(store.count_sweepable_uploads(past_all).await.unwrap(), 0);

        // Once the draft is discarded (request_hash cleared) the upload goes.
        q("UPDATE account_draft SET request_hash='' WHERE owner='realm-owner'")
            .execute(store.backend())
            .await
            .unwrap();
        assert_eq!(store.sweep_uploads(past_all).await.unwrap(), 1);
        assert!(store.get_upload(&account, &in_draft).await.unwrap().is_none());
        assert!(store.get_upload(&account, &photo).await.unwrap().is_some());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn upload_ids_are_found_in_editor_state() {
        let id = format!("U{}", "a1".repeat(32));
        let text = format!("{{\"blobId\":\"{id}\",\"other\":\"U12\",\"x\":\"{}\"}}", "U".repeat(3));
        assert_eq!(upload_ids_in(text.as_bytes()), vec![id]);
        assert!(upload_ids_in(b"").is_empty());
    }

    #[tokio::test]
    async fn delete_removes_row_and_object() {
        let root = temp_root("delete");
        let store = fs_store(&root).await;
        let blob_id = store
            .put_upload("acct-a", "text/plain", b"bye")
            .await
            .unwrap();
        store.delete_upload("acct-a", &blob_id).await.unwrap();
        assert!(
            store
                .get_upload("acct-a", &blob_id)
                .await
                .unwrap()
                .is_none()
        );
        // Deleting an unknown id is a no-op.
        store.delete_upload("acct-a", &blob_id).await.unwrap();

        let _ = std::fs::remove_dir_all(&root);
    }
}
