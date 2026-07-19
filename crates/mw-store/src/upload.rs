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

    /// Resolve an upload for `account_id`, unsealing the object bytes. Account-scoped:
    /// a `blob_id` owned by a different account (or an unknown id) resolves to `None`,
    /// so one account can never read another's upload.
    pub async fn get_upload(
        &self,
        account_id: &str,
        blob_id: &str,
    ) -> Result<Option<Upload>, StoreError> {
        let row = q("SELECT blob_id, account_id, content_type, size, storage_key
                       FROM uploaded_blobs WHERE blob_id = ?1 AND account_id = ?2")
        .bind(blob_id)
        .bind(account_id)
        .fetch_optional(&self.backend)
        .await?;
        let Some(r) = row else { return Ok(None) };
        let storage_key = r.get_string("storage_key");
        let account_key = account_dir_token(account_id);
        let sealed = match self.uploads.get(&account_key, &storage_key).await? {
            Some(s) => s,
            None => return Ok(None),
        };
        let bytes = self.key.open(&sealed)?;
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

    /// Sweep every upload older than `older_than` (by `created_at`), removing both the
    /// backend object and the row. Returns the number of rows reclaimed. Backs the
    /// explicit `mailwoman maintenance gc-uploads` one-shot (never automatic). A backend
    /// object that is already gone does not stall reclamation of the row.
    pub async fn sweep_uploads(&self, older_than: chrono::Duration) -> Result<u64, StoreError> {
        let cutoff = (Utc::now() - older_than).to_rfc3339();
        let rows = q("SELECT account_id, storage_key FROM uploaded_blobs WHERE created_at < ?1")
            .bind(&cutoff)
            .fetch_all(&self.backend)
            .await?;
        for r in &rows {
            let account_key = account_dir_token(&r.get_string("account_id"));
            let storage_key = r.get_string("storage_key");
            // Best-effort object removal: a missing/failed object must not block the row
            // reclamation that is the sweep's actual job.
            let _ = self.uploads.delete(&account_key, &storage_key).await;
        }
        let n = q("DELETE FROM uploaded_blobs WHERE created_at < ?1")
            .bind(&cutoff)
            .execute(&self.backend)
            .await?;
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    /// A unique temp directory for one test's FS backend (avoids a `tempfile`
    /// dev-dependency, mirroring the file-backed store test in `lib.rs`).
    fn temp_root(tag: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("mw-upload-{tag}-{unique}"))
    }

    async fn fs_store(root: &PathBuf) -> Store {
        Store::open_in_memory(ServerKey::generate())
            .await
            .unwrap()
            .with_upload_backend(Arc::new(FsUploadBackend::new(root.clone())))
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
