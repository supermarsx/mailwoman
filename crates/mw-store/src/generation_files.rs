//! Cooperative SQLite lifecycle fencing and durable generation selection.
//!
//! Open databases are never renamed. The active manifest is the only published
//! pointer; its absence after generation mode was activated is an error, never
//! permission to reopen a potentially stale legacy database. Non-cooperating
//! external SQLite writers are unsupported during conversion.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
};

const VERSION: u32 = 1;
const LEGACY: &str = "legacy";
const MANIFEST_LIMIT: u64 = 16384;
const MARKER: &[u8] = b"mailwoman-sqlite-generations-v1\n";

#[path = "generation_retirement.rs"]
pub(crate) mod retirement;

#[derive(Debug, thiserror::Error)]
pub(crate) enum GenerationError {
    #[error("the SQLite installation is already open or undergoing maintenance: {0}")]
    Locked(String),
    #[error("invalid SQLite generation metadata: {0}")]
    Invalid(&'static str),
    #[error(
        "SQLite generation publication may have happened; keep database access fenced until recovery: {0}"
    )]
    PublicationUncertain(#[source] io::Error),
    #[error("the active SQLite generation changed before publication")]
    Changed,
    #[error("SQLite generation filesystem operation failed: {0}")]
    Io(#[from] io::Error),
}

/// A process must retain this guard for as long as it can access the logical
/// database. Store's same-process registry reuses a single guard across clones.
#[derive(Debug)]
pub(crate) struct GenerationFiles {
    logical: PathBuf,
    lease: Mutex<LifecycleLease>,
}
#[derive(Debug, PartialEq, Eq)]
enum LeaseMode {
    Shared,
    OfflineExclusive,
    ConversionExclusive,
    Unfenced,
}
#[derive(Debug)]
struct LifecycleLease {
    file: File,
    mode: LeaseMode,
    // If OS locking fails while restoring shared access, retain admission so
    // cooperating processes cannot enter an accidentally unfenced installation.
    admission_fence: Option<File>,
}
/// Exclusive conversion ownership lasts through every private copy connection.
pub(crate) struct ConversionLease {
    files: Arc<GenerationFiles>,
}
impl Drop for ConversionLease {
    fn drop(&mut self) {
        if let Err(error) = self.files.downgrade_conversion() {
            eprintln!(
                "[mw-store] SQLite lifecycle fence retained after downgrade failure: {error}"
            );
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ActiveGeneration {
    pub id: String,
    pub path: PathBuf,
}
#[derive(Debug)]
pub(crate) struct ReservedGeneration {
    pub id: String,
    pub path: PathBuf,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    version: u32,
    generation_id: String,
    file_name: String,
    source_generation: Option<String>,
    parent_digest: Option<String>,
}

impl GenerationFiles {
    /// Ordinary SQLite processes coexist under shared lifetime leases.
    pub fn shared(path: &Path) -> Result<Self, GenerationError> {
        Self::open_lease(path, false)
    }
    /// Offline CLI operations require exclusive ownership of the installation.
    /// This method never waits for another process to exit.
    pub fn lock(path: &Path) -> Result<Self, GenerationError> {
        Self::open_lease(path, true)
    }
    fn admission(logical: &Path, wait: bool) -> Result<File, GenerationError> {
        let admission = open_private_lock(&sidecar(logical, ".mw-lifecycle-admission.lock"))?;
        if wait {
            admission.lock().map_err(GenerationError::Io)?;
        } else {
            admission
                .try_lock()
                .map_err(|error| GenerationError::Locked(error.to_string()))?;
        }
        Ok(admission)
    }
    fn open_lease(path: &Path, exclusive: bool) -> Result<Self, GenerationError> {
        let logical = canonical_logical_path(path)?;
        // Concurrent ordinary process startup should serialize this tiny critical
        // section rather than mistake another admission attempt for maintenance.
        let until = std::time::Instant::now() + std::time::Duration::from_millis(250);
        let _admission = loop {
            match Self::admission(&logical, false) {
                Ok(file) => break file,
                Err(GenerationError::Locked(_)) if std::time::Instant::now() < until => {
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
                Err(error) => return Err(error),
            }
        };
        let lease = open_private_lock(&sidecar(&logical, ".mw-lifecycle.lock"))?;
        if exclusive {
            lease.try_lock()
        } else {
            lease.try_lock_shared()
        }
        .map_err(|error| GenerationError::Locked(error.to_string()))?;
        Ok(Self {
            logical,
            lease: Mutex::new(LifecycleLease {
                file: lease,
                mode: if exclusive {
                    LeaseMode::OfflineExclusive
                } else {
                    LeaseMode::Shared
                },
                admission_fence: None,
            }),
        })
    }
    /// Conversion is safe only when no other process has a live shared lease.
    /// Admission closes the unlock/relock gap required by Windows locking.
    pub fn acquire_conversion(self: &Arc<Self>) -> Result<ConversionLease, GenerationError> {
        let mut lease = self.lease.lock().unwrap_or_else(|p| p.into_inner());
        if lease.mode != LeaseMode::Shared {
            return Err(GenerationError::Locked(
                "SQLite conversion already owns this installation".into(),
            ));
        }
        let admission = Self::admission(&self.logical, false)?;
        if let Err(error) = lease.file.unlock() {
            lease.mode = LeaseMode::Unfenced;
            lease.admission_fence = Some(admission);
            return Err(GenerationError::Io(error));
        }
        match lease.file.try_lock() {
            Ok(()) => {
                lease.mode = LeaseMode::ConversionExclusive;
                Ok(ConversionLease {
                    files: self.clone(),
                })
            }
            Err(error) => {
                if let Err(restore) = lease.file.lock_shared() {
                    lease.mode = LeaseMode::Unfenced;
                    lease.admission_fence = Some(admission);
                    return Err(GenerationError::Io(restore));
                }
                Err(GenerationError::Locked(format!(
                    "another SQLite process is active: {error}"
                )))
            }
        }
    }
    fn downgrade_conversion(&self) -> Result<(), GenerationError> {
        let mut lease = self.lease.lock().unwrap_or_else(|p| p.into_inner());
        if lease.mode != LeaseMode::ConversionExclusive {
            return Ok(());
        }
        // If opening admission fails, retain the exclusive lifetime lock.
        let admission = Self::admission(&self.logical, true)?;
        if let Err(error) = lease.file.unlock() {
            lease.mode = LeaseMode::Unfenced;
            lease.admission_fence = Some(admission);
            return Err(GenerationError::Io(error));
        }
        if let Err(error) = lease.file.lock_shared() {
            lease.mode = LeaseMode::Unfenced;
            lease.admission_fence = Some(admission);
            return Err(GenerationError::Io(error));
        }
        lease.mode = LeaseMode::Shared;
        Ok(())
    }
    pub fn conversion_active(&self) -> bool {
        matches!(
            self.lease.lock().unwrap_or_else(|p| p.into_inner()).mode,
            LeaseMode::ConversionExclusive | LeaseMode::Unfenced
        )
    }
    pub fn logical_path(&self) -> &Path {
        &self.logical
    }
    pub fn manifest_path(&self) -> PathBuf {
        sidecar(&self.logical, ".mw-current.json")
    }
    fn marker_path(&self) -> PathBuf {
        sidecar(&self.logical, ".mw-generations")
    }
    fn parent(&self) -> &Path {
        self.logical
            .parent()
            .expect("canonical database path has a parent")
    }
    fn logical_name(&self) -> &str {
        self.logical
            .file_name()
            .and_then(|name| name.to_str())
            .expect("logical filename validated")
    }
    fn generation_name(&self, id: &str) -> String {
        format!("{}.mw-generation-{id}.sqlite", self.logical_name())
    }

    /// Resolve only a fully validated manifest. Invalid, dangling, linked or
    /// truncated metadata is an error; there is no fallback to an older file.
    pub fn resolve_active(&self) -> Result<ActiveGeneration, GenerationError> {
        let Some((manifest, _)) = self.read_manifest()? else {
            if self.marker_exists()? {
                return Err(GenerationError::Invalid(
                    "CURRENT is missing after generation mode was activated",
                ));
            }
            if let Ok(metadata) = std::fs::symlink_metadata(&self.logical) {
                check_regular(&self.logical, &metadata)?;
            }
            return Ok(ActiveGeneration {
                id: LEGACY.into(),
                path: self.logical.clone(),
            });
        };
        let path = self.validate_manifest(&manifest)?;
        let metadata = std::fs::symlink_metadata(&path)?;
        check_regular(&path, &metadata)?;
        if manifest.generation_id != LEGACY && metadata.len() == 0 {
            return Err(GenerationError::Invalid("active generation is empty"));
        }
        Ok(ActiveGeneration {
            id: manifest.generation_id,
            path,
        })
    }

    /// Reserve a private, uniquely named file in the logical database directory.
    /// Callers may open it with the same SQLCipher key/settings and build a
    /// verified shadow there. Dropping the reservation never deletes the file.
    pub fn reserve_generation(&self) -> Result<ReservedGeneration, GenerationError> {
        self.activate_protocol()?;
        let id = uuid::Uuid::new_v4().to_string();
        let path = self.parent().join(self.generation_name(&id));
        let file = create_private(&path)?;
        file.sync_all()?;
        sync_parent(self.parent())?;
        Ok(ReservedGeneration { id, path })
    }

    /// Reconstruct an exact-owned filename from a recorded generation ID for
    /// recovery. This neither creates nor opens the file and does not establish
    /// that it is safe to delete; cleanup must still validate the active source.
    pub fn reserved_generation(&self, id: &str) -> Result<ReservedGeneration, GenerationError> {
        if !valid_id(id) {
            return Err(GenerationError::Invalid("invalid reserved generation ID"));
        }
        Ok(ReservedGeneration {
            id: id.to_owned(),
            path: self.parent().join(self.generation_name(id)),
        })
    }

    /// Publish a verified, durable shadow. The host must hold its exclusive
    /// database-operation gate, catch up all writes, validate the database and
    /// flush the target before this call. All potentially stale source handles
    /// stay fenced until the host swaps its active pool to the returned path.
    pub fn publish(
        &self,
        expected: &ActiveGeneration,
        target: &ReservedGeneration,
    ) -> Result<ActiveGeneration, GenerationError> {
        self.activate_protocol()?;
        if self.resolve_active()? != *expected {
            return Err(GenerationError::Changed);
        }
        if !valid_id(&target.id)
            || target.path != self.parent().join(self.generation_name(&target.id))
        {
            return Err(GenerationError::Invalid(
                "target is not an owned same-directory generation",
            ));
        }
        let metadata = std::fs::symlink_metadata(&target.path)?;
        check_regular(&target.path, &metadata)?;
        if metadata.len() == 0 {
            return Err(GenerationError::Invalid("target generation is empty"));
        }
        let (_, bytes) = self.read_manifest()?.ok_or(GenerationError::Invalid(
            "CURRENT disappeared before publication",
        ))?;
        let parent_digest = hex_digest(&bytes);
        let manifest = Manifest {
            version: VERSION,
            generation_id: target.id.clone(),
            file_name: self.generation_name(&target.id),
            source_generation: Some(expected.id.clone()),
            parent_digest: Some(parent_digest),
        };
        self.write_manifest(&manifest)?;
        self.resolve_active()
    }

    /// After a publication error, make the currently selected manifest durable
    /// again while the caller still holds its exclusive database gate. The
    /// returned generation, not an assumed rollback, decides which pool to use.
    pub fn recover_publication(&self) -> Result<ActiveGeneration, GenerationError> {
        let file = open_existing(&self.manifest_path(), true)?;
        file.sync_all()?;
        sync_parent(self.parent())?;
        self.resolve_active()
    }

    /// Remove one explicitly owned abandoned reservation. Active and previously
    /// published generations are retained; this never scans or deletes a tree.
    pub fn discard_reserved(&self, target: ReservedGeneration) -> Result<(), GenerationError> {
        if !valid_id(&target.id)
            || target.path != self.parent().join(self.generation_name(&target.id))
        {
            return Err(GenerationError::Invalid(
                "reservation does not belong to this database",
            ));
        }
        if self.resolve_active()?.id == target.id {
            return Err(GenerationError::Invalid("cannot discard active generation"));
        }
        // Only empty reservations can be removed here. Non-empty SQLite shadows
        // may own WAL/SHM or still-open handles and require coordinated cleanup.
        let metadata = std::fs::symlink_metadata(&target.path)?;
        check_regular(&target.path, &metadata)?;
        if metadata.len() != 0 {
            return Err(GenerationError::Invalid(
                "non-empty shadow needs coordinated cleanup",
            ));
        }
        std::fs::remove_file(&target.path)?;
        sync_parent(self.parent())?;
        Ok(())
    }

    /// Remove only this failed job's unselected shadow and its exact SQLite
    /// sidecars. The caller MUST first close every target SQLite handle. On Unix
    /// an open handle cannot be detected reliably by inspecting filenames.
    /// Never use this for a previously published generation or rollback copy.
    pub fn discard_closed_generation(
        &self,
        expected_active: &ActiveGeneration,
        target: &ReservedGeneration,
    ) -> Result<(), GenerationError> {
        if !valid_id(&target.id)
            || target.path != self.parent().join(self.generation_name(&target.id))
        {
            return Err(GenerationError::Invalid(
                "reservation does not belong to this database",
            ));
        }
        if self.resolve_active()? != *expected_active {
            return Err(GenerationError::Changed);
        }
        if target.id == expected_active.id || target.path == expected_active.path {
            return Err(GenerationError::Invalid("cannot discard active generation"));
        }
        let paths = [
            sidecar(&target.path, "-wal"),
            sidecar(&target.path, "-shm"),
            sidecar(&target.path, "-journal"),
            target.path.clone(),
        ];
        // Preflight the entire exact-owned set before deleting anything. This
        // deliberately does not enumerate neighbouring files or follow links.
        let mut present = Vec::new();
        for path in &paths {
            match std::fs::symlink_metadata(path) {
                Ok(metadata) => {
                    check_regular(path, &metadata)?;
                    present.push(path);
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        for path in present {
            std::fs::remove_file(path)?;
        }
        sync_parent(self.parent())?;
        Ok(())
    }

    fn read_manifest(&self) -> Result<Option<(Manifest, Vec<u8>)>, GenerationError> {
        let path = self.manifest_path();
        let metadata = match std::fs::symlink_metadata(&path) {
            Ok(meta) => meta,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        check_regular(&path, &metadata)?;
        if metadata.len() == 0 || metadata.len() > MANIFEST_LIMIT {
            return Err(GenerationError::Invalid("CURRENT size is invalid"));
        }
        let mut file = open_existing(&path, false)?;
        check_same_file(&metadata, &file.metadata()?)?;
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(MANIFEST_LIMIT + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MANIFEST_LIMIT {
            return Err(GenerationError::Invalid("CURRENT exceeds its size limit"));
        }
        let manifest: Manifest = serde_json::from_slice(&bytes)
            .map_err(|_| GenerationError::Invalid("CURRENT is malformed"))?;
        self.validate_manifest(&manifest)?;
        Ok(Some((manifest, bytes)))
    }
    fn validate_manifest(&self, manifest: &Manifest) -> Result<PathBuf, GenerationError> {
        if manifest.version != VERSION {
            return Err(GenerationError::Invalid("unsupported CURRENT version"));
        }
        let is_legacy = manifest.generation_id == LEGACY;
        if !is_legacy && !valid_id(&manifest.generation_id) {
            return Err(GenerationError::Invalid("invalid generation ID"));
        }
        let expected = if is_legacy {
            self.logical_name().to_owned()
        } else {
            self.generation_name(&manifest.generation_id)
        };
        if manifest.file_name != expected
            || Path::new(&manifest.file_name).components().count() != 1
            || !matches!(
                Path::new(&manifest.file_name).components().next(),
                Some(Component::Normal(_))
            )
        {
            return Err(GenerationError::Invalid(
                "CURRENT filename is not an owned relative basename",
            ));
        }
        if is_legacy {
            if manifest.source_generation.is_some() || manifest.parent_digest.is_some() {
                return Err(GenerationError::Invalid(
                    "legacy CURRENT has unexpected lineage",
                ));
            }
        } else {
            let source = manifest
                .source_generation
                .as_deref()
                .ok_or(GenerationError::Invalid("CURRENT has no source generation"))?;
            if source != LEGACY && !valid_id(source) || source == manifest.generation_id {
                return Err(GenerationError::Invalid("invalid source generation"));
            }
            if !manifest.parent_digest.as_deref().is_some_and(|value| {
                value.len() == 64
                    && value
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            }) {
                return Err(GenerationError::Invalid("invalid parent digest"));
            }
        }
        Ok(self.parent().join(&manifest.file_name))
    }
    fn marker_exists(&self) -> Result<bool, GenerationError> {
        let path = self.marker_path();
        let metadata = match std::fs::symlink_metadata(&path) {
            Ok(meta) => meta,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error.into()),
        };
        check_regular(&path, &metadata)?;
        let file = open_existing(&path, false)?;
        check_same_file(&metadata, &file.metadata()?)?;
        let mut bytes = Vec::new();
        file.take(128).read_to_end(&mut bytes)?;
        if bytes != MARKER {
            return Err(GenerationError::Invalid(
                "generation activation marker is malformed",
            ));
        }
        Ok(true)
    }
    fn activate_protocol(&self) -> Result<(), GenerationError> {
        let active = self.resolve_active()?;
        if self.read_manifest()?.is_none() {
            let legacy = Manifest {
                version: VERSION,
                generation_id: LEGACY.into(),
                file_name: self.logical_name().into(),
                source_generation: None,
                parent_digest: None,
            };
            self.write_manifest(&legacy)?;
        }
        if !self.marker_exists()? {
            let mut marker = create_private(&self.marker_path())?;
            marker.write_all(MARKER)?;
            marker.sync_all()?;
            sync_parent(self.parent())?;
        }
        if self.resolve_active()? != active {
            return Err(GenerationError::Changed);
        }
        Ok(())
    }
    fn write_manifest(&self, manifest: &Manifest) -> Result<(), GenerationError> {
        self.validate_manifest(manifest)?;
        let bytes = serde_json::to_vec(manifest)
            .map_err(|_| GenerationError::Invalid("CURRENT serialization failed"))?;
        let path = sidecar(
            &self.logical,
            &format!(".mw-current-{}.tmp", uuid::Uuid::new_v4()),
        );
        let mut file = create_private(&path)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        // A failed publication deliberately leaves only this owned temporary;
        // recovery never treats it as active or deletes unrelated files.
        atomic_replace(&path, &self.manifest_path())
            .map_err(GenerationError::PublicationUncertain)?;
        sync_parent(self.parent()).map_err(GenerationError::PublicationUncertain)?;
        Ok(())
    }
}

pub(crate) fn canonical_logical_path(path: &Path) -> Result<PathBuf, GenerationError> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .ok_or(GenerationError::Invalid(
            "database path has no UTF-8 filename",
        ))?;
    if matches!(name, "." | "..")
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(GenerationError::Invalid(
            "database path contains parent traversal",
        ));
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = std::fs::canonicalize(parent)?;
    if !parent.is_dir() {
        return Err(GenerationError::Invalid(
            "database parent is not a directory",
        ));
    }
    Ok(parent.join(name))
}
fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}
fn valid_id(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok_and(|id| id.to_string() == value)
}
fn hex_digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn create_private(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}
fn open_existing(path: &Path, write: bool) -> Result<File, GenerationError> {
    let mut options = OpenOptions::new();
    options.read(true).write(write);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x00200000);
    }
    let file = options.open(path)?;
    check_regular(path, &file.metadata()?)?;
    Ok(file)
}
pub(crate) fn open_private_lock(path: &Path) -> Result<File, GenerationError> {
    match create_private(path) {
        Ok(file) => Ok(file),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            let metadata = std::fs::symlink_metadata(path)?;
            check_regular(path, &metadata)?;
            let file = open_existing(path, true)?;
            check_same_file(&metadata, &file.metadata()?)?;
            Ok(file)
        }
        Err(error) => Err(error.into()),
    }
}
fn check_regular(path: &Path, metadata: &std::fs::Metadata) -> Result<(), GenerationError> {
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(GenerationError::Invalid(
            "generation path is not a regular unlinked file",
        ));
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(GenerationError::Invalid(
                "reparse points are not allowed for generation files",
            ));
        }
        windows_single_link(path)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            return Err(GenerationError::Invalid(
                "hard-linked generation files are not supported",
            ));
        }
    }
    if std::fs::canonicalize(path)?.parent() != path.parent() {
        return Err(GenerationError::Invalid(
            "generation path escaped the logical directory",
        ));
    }
    Ok(())
}
#[cfg(windows)]
fn windows_single_link(path: &Path) -> Result<(), GenerationError> {
    mw_platform_fs::verify_single_link(path).map_err(GenerationError::Io)
}

fn check_same_file(
    before: &std::fs::Metadata,
    after: &std::fs::Metadata,
) -> Result<(), GenerationError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if before.dev() != after.dev() || before.ino() != after.ino() {
            return Err(GenerationError::Invalid(
                "generation file changed while opening",
            ));
        }
    }
    #[cfg(not(unix))]
    {
        if before.len() != after.len() || before.file_type() != after.file_type() {
            return Err(GenerationError::Invalid(
                "generation file changed while opening",
            ));
        }
    }
    Ok(())
}
fn atomic_replace(from: &Path, to: &Path) -> io::Result<()> {
    mw_platform_fs::atomic_replace(from, to)
}

#[cfg(unix)]
fn sync_parent(parent: &Path) -> io::Result<()> {
    File::open(parent)?.sync_all()
}
#[cfg(not(unix))]
fn sync_parent(_parent: &Path) -> io::Result<()> {
    Ok(())
}

/// Atomic private sidecar publication for authenticated control requests or
/// payload-free status. The caller owns the chosen sidecar path; input bytes
/// are bounded and never interpreted here.
pub(crate) fn private_atomic_write(path: &Path, bytes: &[u8]) -> Result<(), GenerationError> {
    if bytes.len() > 65536 {
        return Err(GenerationError::Invalid("control sidecar is too large"));
    }
    let path = canonical_logical_path(path)?;
    match std::fs::symlink_metadata(&path) {
        Ok(metadata) => check_regular(&path, &metadata)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let temp = sidecar(&path, &format!(".{}-tmp", uuid::Uuid::new_v4()));
    let mut file = create_private(&temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    atomic_replace(&temp, &path)?;
    sync_parent(path.parent().expect("canonical sidecar parent"))?;
    Ok(())
}
/// Read one regular private sidecar without following a supplied link or
/// allocating according to an attacker-controlled file size.
pub(crate) fn bounded_read(path: &Path, max_bytes: u64) -> Result<Vec<u8>, GenerationError> {
    let path = canonical_logical_path(path)?;
    let max_bytes = max_bytes.clamp(1, 1024 * 1024);
    let metadata = std::fs::symlink_metadata(&path)?;
    check_regular(&path, &metadata)?;
    if metadata.len() > max_bytes {
        return Err(GenerationError::Invalid(
            "control sidecar exceeds its size limit",
        ));
    }
    let mut file = open_existing(&path, false)?;
    check_same_file(&metadata, &file.metadata()?)?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(max_bytes + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(GenerationError::Invalid(
            "control sidecar grew beyond its size limit",
        ));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_processes_coexist_but_conversion_and_offline_maintenance_are_exclusive() {
        fn assert_send<T: Send>() {}
        assert_send::<ConversionLease>();
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("mail.db");
        let first = Arc::new(GenerationFiles::shared(&path).unwrap());
        let second = GenerationFiles::shared(&path).unwrap();
        assert!(GenerationFiles::lock(&path).is_err());
        assert!(first.acquire_conversion().is_err());
        // A refused upgrade must restore its shared lease before admission opens.
        let third = GenerationFiles::shared(&path).unwrap();
        drop(second);
        drop(third);
        let conversion = first.acquire_conversion().unwrap();
        assert!(first.acquire_conversion().is_err());
        assert!(GenerationFiles::shared(&path).is_err());
        assert!(GenerationFiles::lock(&path).is_err());
        drop(conversion);
        let second = GenerationFiles::shared(&path).unwrap();
        assert!(GenerationFiles::lock(&path).is_err());
        drop(second);
        drop(first);
        let offline = GenerationFiles::lock(&path).unwrap();
        assert!(GenerationFiles::shared(&path).is_err());
        drop(offline);
        drop(GenerationFiles::shared(&path).unwrap());
    }

    #[test]
    fn admission_serializes_new_opens_and_upgrade_without_releasing_existing_lease() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("mail.db");
        let live = Arc::new(GenerationFiles::shared(&path).unwrap());
        let admission = GenerationFiles::admission(live.logical_path(), false).unwrap();
        assert!(GenerationFiles::shared(&path).is_err());
        assert!(GenerationFiles::lock(&path).is_err());
        assert!(live.acquire_conversion().is_err());
        drop(admission);
        let conversion = live.acquire_conversion().unwrap();
        drop(conversion);
        assert!(GenerationFiles::lock(&path).is_err());
    }

    fn fixture() -> (PathBuf, GenerationFiles) {
        let dir =
            std::env::temp_dir().join(format!("mw-generation-files-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let logical = dir.join("mailwoman.db");
        std::fs::write(&logical, b"original SQLite stand-in").unwrap();
        let files = GenerationFiles::lock(&logical).unwrap();
        (dir, files)
    }
    #[test]
    fn reconstructing_reservation_requires_canonical_uuid_and_never_creates_file() {
        let (dir, files) = fixture();
        let id = "12abcdef-1234-4567-89ab-123456789abc".to_string();
        let target = files.reserved_generation(&id).unwrap();
        assert_eq!(target.id, id);
        assert_eq!(
            target.path,
            files
                .parent()
                .join(format!("mailwoman.db.mw-generation-{id}.sqlite"))
        );
        assert!(!target.path.exists());
        assert!(!files.manifest_path().exists());
        for invalid in [
            "legacy",
            "",
            "../outside",
            "/outside",
            &id.to_uppercase(),
            &id.replace('-', ""),
        ] {
            assert!(files.reserved_generation(invalid).is_err(), "{invalid:?}");
        }
        drop(files);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn lifecycle_lease_fences_other_opens_until_all_owner_handles_close() {
        let (dir, files) = fixture();
        let logical = files.logical_path().to_owned();
        assert!(matches!(
            GenerationFiles::lock(&logical),
            Err(GenerationError::Locked(_))
        ));
        drop(files);
        drop(GenerationFiles::lock(&logical).unwrap());
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn generation_publication_preserves_source_and_persists_across_reopen() {
        let (dir, files) = fixture();
        let source = files.resolve_active().unwrap();
        let next = files.reserve_generation().unwrap();
        std::fs::write(&next.path, b"verified target stand-in").unwrap();
        let active = files.publish(&source, &next).unwrap();
        assert_eq!(active.path, next.path);
        assert_eq!(
            std::fs::read(&source.path).unwrap(),
            b"original SQLite stand-in"
        );
        assert!(matches!(
            files.publish(&source, &next),
            Err(GenerationError::Changed)
        ));
        let logical = files.logical_path().to_owned();
        drop(files);
        let files = GenerationFiles::lock(&logical).unwrap();
        assert_eq!(files.resolve_active().unwrap(), active);
        drop(files);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn missing_or_corrupt_current_never_falls_back_after_activation() {
        let (dir, files) = fixture();
        let next = files.reserve_generation().unwrap();
        files.discard_reserved(next).unwrap();
        std::fs::remove_file(files.manifest_path()).unwrap();
        assert!(files.resolve_active().is_err());
        std::fs::write(files.manifest_path(), b"bad JSON").unwrap();
        assert!(files.resolve_active().is_err());
        drop(files);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn rejects_traversal_dangling_wrong_version_and_empty_active_generations() {
        let (dir, files) = fixture();
        let next = files.reserve_generation().unwrap();
        assert!(
            files
                .publish(&files.resolve_active().unwrap(), &next)
                .is_err()
        );
        let original = std::fs::read(files.manifest_path()).unwrap();
        for patch in [
            serde_json::json!({"fileName":"../outside.sqlite"}),
            serde_json::json!({"version":2}),
        ] {
            let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
            for (key, value_patch) in patch.as_object().unwrap() {
                value[key] = value_patch.clone();
            }
            std::fs::write(files.manifest_path(), serde_json::to_vec(&value).unwrap()).unwrap();
            assert!(files.resolve_active().is_err());
        }
        drop(files);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn abandoning_an_empty_reservation_only_removes_that_exact_file() {
        let (dir, files) = fixture();
        let reservation = files.reserve_generation().unwrap();
        let path = reservation.path.clone();
        let unrelated = dir.join("keep.txt");
        std::fs::write(&unrelated, b"keep").unwrap();
        files.discard_reserved(reservation).unwrap();
        assert!(!path.exists());
        assert!(unrelated.exists());
        assert!(files.logical_path().exists());
        drop(files);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn private_files_and_symlink_rejection() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let (dir, files) = fixture();
        let next = files.reserve_generation().unwrap();
        assert_eq!(
            std::fs::metadata(&next.path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        std::fs::remove_file(&next.path).unwrap();
        symlink(files.logical_path(), &next.path).unwrap();
        assert!(
            files
                .publish(&files.resolve_active().unwrap(), &next)
                .is_err()
        );
        drop(files);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn missing_active_target_is_not_replaced_with_legacy_source() {
        let (dir, files) = fixture();
        let source = files.resolve_active().unwrap();
        let target = files.reserve_generation().unwrap();
        std::fs::write(&target.path, b"verified target").unwrap();
        files.publish(&source, &target).unwrap();
        std::fs::remove_file(&target.path).unwrap();
        assert!(files.resolve_active().is_err());
        assert!(source.path.exists());
        drop(files);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn closed_shadow_cleanup_removes_exact_sidecars_and_preserves_source() {
        let (dir, files) = fixture();
        let source = files.resolve_active().unwrap();
        let target = files.reserve_generation().unwrap();
        std::fs::write(&target.path, b"failed shadow").unwrap();
        for suffix in ["-wal", "-shm", "-journal"] {
            std::fs::write(sidecar(&target.path, suffix), b"closed sidecar").unwrap();
        }
        let unrelated = sidecar(&target.path, ".keep");
        std::fs::write(&unrelated, b"keep").unwrap();
        files.discard_closed_generation(&source, &target).unwrap();
        files.discard_closed_generation(&source, &target).unwrap();
        assert!(!target.path.exists());
        assert!(unrelated.exists());
        assert_eq!(
            std::fs::read(&source.path).unwrap(),
            b"original SQLite stand-in"
        );
        drop(files);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn closed_shadow_cleanup_rejects_active_or_changed_source() {
        let (dir, files) = fixture();
        let source = files.resolve_active().unwrap();
        let target = files.reserve_generation().unwrap();
        std::fs::write(&target.path, b"verified target").unwrap();
        let active = files.publish(&source, &target).unwrap();
        assert!(matches!(
            files.discard_closed_generation(&source, &target),
            Err(GenerationError::Changed)
        ));
        assert!(files.discard_closed_generation(&active, &target).is_err());
        assert!(target.path.exists());
        drop(files);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn closed_shadow_cleanup_preflights_hardlinked_sidecars_before_deleting() {
        let (dir, files) = fixture();
        let source = files.resolve_active().unwrap();
        let target = files.reserve_generation().unwrap();
        std::fs::write(&target.path, b"failed shadow").unwrap();
        let wal = sidecar(&target.path, "-wal");
        std::fs::write(&wal, b"closed wal").unwrap();
        let shm = sidecar(&target.path, "-shm");
        std::fs::hard_link(&source.path, &shm).unwrap();
        assert!(files.discard_closed_generation(&source, &target).is_err());
        assert!(wal.exists());
        assert!(target.path.exists());
        drop(files);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn private_sidecars_replace_atomically_and_reads_are_bounded() {
        let (dir, files) = fixture();
        let path = dir.join("control.status");
        private_atomic_write(&path, b"first").unwrap();
        private_atomic_write(&path, b"second").unwrap();
        assert_eq!(bounded_read(&path, 64).unwrap(), b"second");
        assert!(bounded_read(&path, 2).is_err());
        assert!(private_atomic_write(&path, &vec![0; 65537]).is_err());
        drop(files);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
