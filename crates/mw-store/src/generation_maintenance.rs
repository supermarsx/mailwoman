//! Shared inspection and exclusive offline access to the selected SQLite generation.
//!
//! Retain either lease until every raw SQLite handle has closed. Inspection
//! coexists with running Stores; exclusive maintenance excludes them. External
//! SQLite writers must still be stopped by the operator for maintenance.
use crate::generation_files::GenerationFiles;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
#[error("offline SQLite maintenance is unavailable: {0}")]
pub struct OfflineDatabaseError(String);

#[derive(Debug, thiserror::Error)]
#[error("SQLite inspection is unavailable: {0}")]
pub struct DatabaseInspectionError(String);

/// Stable read-only generation selection that coexists with running Stores.
/// Keep this guard until every raw inspection connection has closed. It blocks
/// conversion and offline writers; it does not authorize modifying the database.
#[derive(Debug)]
pub struct DatabaseInspectionLease {
    files: GenerationFiles,
    active: PathBuf,
}
impl DatabaseInspectionLease {
    pub fn acquire(path: &Path) -> Result<Self, DatabaseInspectionError> {
        let files = GenerationFiles::shared(path)
            .map_err(|error| DatabaseInspectionError(error.to_string()))?;
        let active = files
            .resolve_active()
            .map_err(|error| DatabaseInspectionError(error.to_string()))?
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
        let inspection = DatabaseInspectionLease::acquire(&logical).unwrap();
        assert_eq!(inspection.active_path(), target.path);
        assert!(OfflineDatabaseLease::acquire(&logical).is_err());
        drop(inspection);
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
    #[tokio::test]
    async fn inspection_coexists_with_running_store_and_fences_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mail.db");
        let store = crate::Store::open_with_encryption(
            path.to_str().unwrap(),
            crate::ServerKey::generate(),
            &crate::encryption::EncryptionConfig::off(),
        )
        .await
        .unwrap();
        let crate::backend::Backend::Sqlite(live) = &store.backend else {
            unreachable!()
        };
        let files = live.files().unwrap();
        let inspection = DatabaseInspectionLease::acquire(&path).unwrap();
        assert_eq!(
            inspection.active_path(),
            files.resolve_active().unwrap().path
        );
        assert_eq!(inspection.logical_path(), live.logical_path());
        assert!(files.acquire_conversion().is_err());
        assert!(OfflineDatabaseLease::acquire(&path).is_err());
        // Ordinary application reads and writes continue while inspection holds
        // the stable physical generation selected by CURRENT.
        crate::backend::q("CREATE TABLE inspection_probe(id INTEGER PRIMARY KEY)")
            .execute(&store.backend)
            .await
            .unwrap();
        crate::backend::q("INSERT INTO inspection_probe VALUES(1)")
            .execute(&store.backend)
            .await
            .unwrap();
        assert_eq!(
            crate::backend::q("SELECT COUNT(*) FROM inspection_probe")
                .fetch_scalar_i64(&store.backend)
                .await
                .unwrap(),
            1
        );
        drop(inspection);
        let conversion = files.acquire_conversion().unwrap();
        assert!(DatabaseInspectionLease::acquire(&path).is_err());
        drop(conversion);
        assert!(DatabaseInspectionLease::acquire(&path).is_ok());
        store.close().await;
    }
}
