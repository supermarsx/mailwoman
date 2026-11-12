//! Tantivy files encrypted at rest; plaintext buffers never use temporary files.
use mw_store::ServerKey;
use std::{
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    sync::Arc,
};
use tantivy::directory::error::{DeleteError, LockError, OpenReadError, OpenWriteError};
use tantivy::directory::{
    AntiCallToken, Directory, DirectoryLock, FileHandle, FileSlice, Lock, MmapDirectory,
    TerminatingWrite, WatchCallback, WatchHandle, WritePtr,
};

#[derive(Clone, Debug)]
pub(crate) struct SealedDirectory {
    inner: MmapDirectory,
    key: ServerKey,
}
fn invalid(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}
fn index_file(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or("");
    name == "meta.json"
        || name == ".managed.json"
        || (stem.len() == 32 && stem.bytes().all(|b| b.is_ascii_hexdigit()) && name.len() > 33)
}
impl SealedDirectory {
    pub fn open(dir: &Path, key: ServerKey) -> io::Result<Self> {
        let inner = MmapDirectory::open(dir).map_err(invalid)?;
        let result = Self { inner, key };
        // Hold Tantivy's real OS writer lock during the resumable in-place migration.
        let _lock = result
            .inner
            .acquire_lock(&tantivy::directory::INDEX_WRITER_LOCK)
            .map_err(invalid)?;
        let mut plain = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if !index_file(name) {
                continue;
            }
            if !entry.file_type()?.is_file() {
                return Err(invalid("search files must be regular files"));
            }
            let bytes = std::fs::read(entry.path())?;
            if bytes.starts_with(b"MWB1") {
                result.decode(Path::new(name), &bytes)?;
            } else {
                plain.push(PathBuf::from(name));
            }
        }
        // Validate existing encrypted files before touching any legacy files: a wrong
        // key must not partly migrate an existing index under a different key.
        for path in plain {
            let bytes = std::fs::read(dir.join(&path))?;
            result.atomic_write(&path, &bytes)?;
        }
        result.inner.sync_directory()?;
        Ok(result)
    }
    fn encode(&self, path: &Path, bytes: &[u8]) -> io::Result<Vec<u8>> {
        self.key
            .seal_bound(bytes, "search-file-v1", &[&path.to_string_lossy()])
            .map_err(invalid)
    }
    fn decode(&self, path: &Path, bytes: &[u8]) -> io::Result<Vec<u8>> {
        self.key
            .open_bound(bytes, "search-file-v1", &[&path.to_string_lossy()])
            .map_err(invalid)
    }
}
struct Writer {
    directory: SealedDirectory,
    path: PathBuf,
    bytes: Vec<u8>,
}
impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl TerminatingWrite for Writer {
    fn terminate_ref(&mut self, _: AntiCallToken) -> io::Result<()> {
        let bytes = self.directory.encode(&self.path, &self.bytes)?;
        // The underlying WORM writer reserved this name; only encrypted bytes are
        // exposed. atomic_write replaces that empty reservation durably.
        self.directory.inner.atomic_write(&self.path, &bytes)
    }
}
impl Directory for SealedDirectory {
    fn get_file_handle(&self, path: &Path) -> Result<Arc<dyn FileHandle>, OpenReadError> {
        Ok(Arc::new(FileSlice::from(self.atomic_read(path)?)))
    }
    fn delete(&self, path: &Path) -> Result<(), DeleteError> {
        self.inner.delete(path)
    }
    fn exists(&self, path: &Path) -> Result<bool, OpenReadError> {
        self.inner.exists(path)
    }
    fn open_write(&self, path: &Path) -> Result<WritePtr, OpenWriteError> {
        // Preserve create-new semantics and cross-process exclusion, without writing
        // any plaintext, even if the caller drops an unterminated writer.
        let reservation = self.inner.open_write(path)?;
        drop(reservation);
        Ok(BufWriter::new(Box::new(Writer {
            directory: self.clone(),
            path: path.into(),
            bytes: Vec::new(),
        })))
    }
    fn atomic_read(&self, path: &Path) -> Result<Vec<u8>, OpenReadError> {
        let raw = self.inner.atomic_read(path)?;
        self.decode(path, &raw)
            .map_err(|e| OpenReadError::wrap_io_error(e, path.into()))
    }
    fn atomic_write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        self.inner.atomic_write(path, &self.encode(path, bytes)?)
    }
    fn sync_directory(&self) -> io::Result<()> {
        self.inner.sync_directory()
    }
    fn acquire_lock(&self, lock: &Lock) -> Result<DirectoryLock, LockError> {
        self.inner.acquire_lock(lock)
    }
    fn watch(&self, callback: WatchCallback) -> tantivy::Result<WatchHandle> {
        self.inner.watch(callback)
    }
}
