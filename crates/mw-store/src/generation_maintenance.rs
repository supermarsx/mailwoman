//! Exclusive offline access to the currently selected SQLite generation.
//!
//! Retain the lease until every raw SQLite handle has closed. Cooperating
//! Mailwoman processes fail promptly while it is held; external SQLite writers
//! must still be stopped by the operator.
use crate::generation_files::GenerationFiles;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
#[error("offline SQLite maintenance is unavailable: {0}")]
pub struct OfflineDatabaseError(String);

#[derive(Debug)]
pub struct OfflineDatabaseLease {
    files: GenerationFiles,
    active: PathBuf,
}

impl OfflineDatabaseLease {
    pub fn acquire(path: &Path) -> Result<Self, OfflineDatabaseError> {
        let files =
            GenerationFiles::lock(path).map_err(|error| OfflineDatabaseError(error.to_string()))?;
        let active = files
            .resolve_active()
            .map_err(|error| OfflineDatabaseError(error.to_string()))?
            .path;
        Ok(Self { files, active })
    }

    pub fn active_path(&self) -> &Path {
        &self.active
    }
    pub fn logical_path(&self) -> &Path {
        self.files.logical_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lease_fences_opens_and_resolves_published_target() {
        let dir = std::env::temp_dir().join(format!("mw-offline-lease-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let logical = dir.join("mailwoman.db");
        std::fs::write(&logical, b"original").unwrap();
        let files = GenerationFiles::lock(&logical).unwrap();
        let source = files.resolve_active().unwrap();
        let target = files.reserve_generation().unwrap();
        std::fs::write(&target.path, b"verified target").unwrap();
        files.publish(&source, &target).unwrap();
        assert!(OfflineDatabaseLease::acquire(&logical).is_err());
        drop(files);
        let lease = OfflineDatabaseLease::acquire(&logical).unwrap();
        assert_eq!(lease.active_path(), target.path);
        assert_eq!(
            lease.logical_path(),
            std::fs::canonicalize(&logical).unwrap()
        );
        assert!(OfflineDatabaseLease::acquire(&logical).is_err());
        drop(lease);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
