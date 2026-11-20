//! Explicit retirement of the one predecessor proven by the selected CURRENT.
//! Directory neighbours and abandoned/unrecorded shadows are never candidates.
use super::*;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetainedDatabase {
    pub generation_id: String,
    pub file_name: String,
    pub path: String,
    pub bytes: u64,
    pub modified_at: Option<u64>,
    pub file_identity: String,
    pub deletable: bool,
    pub blocked_reason: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetainedDatabaseReport {
    pub supported: bool,
    pub active_generation: Option<String>,
    pub observed_at: u64,
    pub retained: Vec<RetainedDatabase>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetireDatabaseRequest {
    pub generation_id: String,
    pub expected_active_generation: String,
    pub file_identity: String,
    pub confirmation: String,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetiredDatabase {
    pub generation_id: String,
    pub file_name: String,
    pub removed_bytes: u64,
}
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn identity(metadata: &std::fs::Metadata) -> String {
    let modified = metadata
        .modified()
        .ok()
        .and_then(|at| at.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos());
    #[cfg(unix)]
    let platform = {
        use std::os::unix::fs::MetadataExt;
        format!("{}:{}", metadata.dev(), metadata.ino())
    };
    #[cfg(windows)]
    let platform = {
        use std::os::windows::fs::MetadataExt;
        format!(
            "{}:{}",
            metadata.creation_time(),
            metadata.file_attributes()
        )
    };
    #[cfg(not(any(unix, windows)))]
    let platform = String::new();
    hex_digest(format!("{}:{modified:?}:{platform}", metadata.len()).as_bytes())
}
impl GenerationFiles {
    fn retained_predecessor(
        &self,
    ) -> Result<Option<(ActiveGeneration, RetainedDatabase)>, GenerationError> {
        let active = self.resolve_active()?;
        let Some((manifest, _)) = self.read_manifest()? else {
            return Ok(None);
        };
        if active.id != manifest.generation_id {
            return Err(GenerationError::Changed);
        }
        let Some(id) = manifest.source_generation else {
            return Ok(None);
        };
        let path = if id == LEGACY {
            self.logical.clone()
        } else {
            self.reserved_generation(&id)?.path
        };
        if id == active.id || path == active.path {
            return Err(GenerationError::Invalid(
                "cannot retire the active database",
            ));
        }
        let metadata = match std::fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        // Fail closed rather than listing a linked or unrelated object as safe.
        check_regular(&path, &metadata)?;
        let sidecars =
            ["-wal", "-shm", "-journal"]
                .into_iter()
                .try_fold(false, |present, suffix| {
                    match std::fs::symlink_metadata(sidecar(&path, suffix)) {
                        Ok(_) => Ok(true),
                        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(present),
                        Err(error) => Err(GenerationError::Io(error)),
                    }
                })?;
        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|at| at.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());
        let row = RetainedDatabase {
            generation_id: id,
            file_name: path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or(GenerationError::Invalid("invalid retained filename"))?
                .into(),
            path: path.display().to_string(),
            bytes: metadata.len(),
            modified_at,
            file_identity: identity(&metadata),
            deletable: !sidecars,
            blocked_reason: sidecars.then(|| "sqlite-sidecars".into()),
        };
        Ok(Some((active, row)))
    }
    pub(crate) fn retained_database_report(
        &self,
    ) -> Result<RetainedDatabaseReport, GenerationError> {
        let mut report = RetainedDatabaseReport {
            supported: true,
            active_generation: Some(self.resolve_active()?.id),
            observed_at: now(),
            retained: vec![],
        };
        if let Some((active, mut row)) = self.retained_predecessor()? {
            if Some(active.id) != report.active_generation {
                return Err(GenerationError::Changed);
            }
            if self.conversion_active() {
                row.deletable = false;
                row.blocked_reason = Some("maintenance-active".into());
            }
            report.retained.push(row);
        }
        Ok(report)
    }
    pub(crate) fn retire_database(
        &self,
        request: &RetireDatabaseRequest,
    ) -> Result<RetiredDatabase, GenerationError> {
        if self.lease.lock().unwrap_or_else(|p| p.into_inner()).mode
            != LeaseMode::ConversionExclusive
        {
            return Err(GenerationError::Locked(
                "retirement requires exclusive lifecycle ownership".into(),
            ));
        }
        let Some((active, row)) = self.retained_predecessor()? else {
            return Err(GenerationError::Invalid(
                "no verified retained predecessor is available",
            ));
        };
        if active.id != request.expected_active_generation {
            return Err(GenerationError::Changed);
        }
        if row.generation_id != request.generation_id || row.file_identity != request.file_identity
        {
            return Err(GenerationError::Invalid(
                "retained database changed; refresh before confirming",
            ));
        }
        if row.file_name != request.confirmation {
            return Err(GenerationError::Invalid(
                "retained filename confirmation does not match",
            ));
        }
        if !row.deletable {
            return Err(GenerationError::Invalid(
                "retained database has SQLite sidecars; it may still be in use",
            ));
        }
        // Never open this old database with SQLite or remove its WAL/journal.
        // The lifecycle lease excludes cooperating processes and the caller's
        // rebuild guard excludes conversion; CURRENT is checked again at removal.
        if self.resolve_active()? != active {
            return Err(GenerationError::Changed);
        }
        let path = if row.generation_id == LEGACY {
            self.logical.clone()
        } else {
            self.reserved_generation(&row.generation_id)?.path
        };
        let metadata = std::fs::symlink_metadata(&path)?;
        check_regular(&path, &metadata)?;
        if identity(&metadata) != request.file_identity {
            return Err(GenerationError::Changed);
        }
        std::fs::remove_file(&path)?;
        sync_parent(self.parent())?;
        Ok(RetiredDatabase {
            generation_id: row.generation_id,
            file_name: row.file_name,
            removed_bytes: row.bytes,
        })
    }
}

impl crate::Store {
    pub async fn retained_databases(&self) -> Result<RetainedDatabaseReport, String> {
        let crate::backend::Backend::Sqlite(live) = &self.backend else {
            return Ok(RetainedDatabaseReport {
                supported: false,
                active_generation: None,
                observed_at: now(),
                retained: vec![],
            });
        };
        let files = live
            .files()
            .ok_or("retained generations require a managed file database")?;
        tokio::task::spawn_blocking(move || {
            files.retained_database_report().map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())?
    }
    pub async fn retire_retained_database(
        &self,
        request: RetireDatabaseRequest,
    ) -> Result<RetiredDatabase, String> {
        let crate::backend::Backend::Sqlite(live) = &self.backend else {
            return Err("retained generations apply only to SQLite".into());
        };
        let rebuild = live.try_rebuild_guard().map_err(|e| e.to_string())?;
        let files = live
            .files()
            .ok_or("retained generations require a managed file database")?;
        // Detached blocking ownership outlives an aborted HTTP request. No pool
        // or connection is closed; the active database continues serving work.
        tokio::task::spawn_blocking(move || {
            let _rebuild = rebuild;
            let _ownership = files.acquire_conversion().map_err(|e| e.to_string())?;
            files.retire_database(&request).map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (
        tempfile::TempDir,
        Arc<GenerationFiles>,
        ActiveGeneration,
        ReservedGeneration,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mail.db");
        std::fs::write(&path, b"old database").unwrap();
        let files = Arc::new(GenerationFiles::shared(&path).unwrap());
        let source = files.resolve_active().unwrap();
        let target = files.reserve_generation().unwrap();
        std::fs::write(&target.path, b"active database").unwrap();
        files.publish(&source, &target).unwrap();
        (dir, files, source, target)
    }
    fn request(files: &GenerationFiles) -> RetireDatabaseRequest {
        let report = files.retained_database_report().unwrap();
        let row = &report.retained[0];
        RetireDatabaseRequest {
            generation_id: row.generation_id.clone(),
            expected_active_generation: report.active_generation.unwrap(),
            file_identity: row.file_identity.clone(),
            confirmation: row.file_name.clone(),
        }
    }
    #[test]
    fn retires_only_manifest_predecessor_and_preserves_current_unknown_and_selected_files() {
        let (_dir, files, source, target) = fixture();
        let request = request(&files);
        let before = std::fs::read(files.manifest_path()).unwrap();
        let unknown = files.reserve_generation().unwrap();
        std::fs::write(&unknown.path, b"unknown").unwrap();
        assert!(files.retire_database(&request).is_err());
        let _lease = files.acquire_conversion().unwrap();
        let result = files.retire_database(&request).unwrap();
        assert_eq!(result.removed_bytes, 12);
        assert!(!source.path.exists());
        assert!(unknown.path.exists());
        assert!(target.path.exists());
        assert_eq!(before, std::fs::read(files.manifest_path()).unwrap());
        assert_eq!(files.resolve_active().unwrap().path, target.path);
        assert!(
            files
                .retained_database_report()
                .unwrap()
                .retained
                .is_empty()
        );
    }
    #[test]
    fn refuses_active_stale_identity_wrong_confirmation_and_arbitrary_paths() {
        let (_dir, files, source, target) = fixture();
        let original = request(&files);
        let _lease = files.acquire_conversion().unwrap();
        let mut invalid = original.clone();
        invalid.generation_id = target.id.clone();
        assert!(files.retire_database(&invalid).is_err());
        invalid.generation_id = "../other".into();
        assert!(files.retire_database(&invalid).is_err());
        let mut invalid = original.clone();
        invalid.confirmation = "wrong".into();
        assert!(files.retire_database(&invalid).is_err());
        invalid = original.clone();
        invalid.expected_active_generation = "legacy".into();
        assert!(files.retire_database(&invalid).is_err());
        std::fs::write(&source.path, b"changed old database").unwrap();
        assert!(files.retire_database(&original).is_err());
        assert!(source.path.exists());
        assert!(target.path.exists());
    }
    #[test]
    fn any_sidecar_and_another_cooperating_reader_block_retirement() {
        let (_dir, files, source, _target) = fixture();
        let request = request(&files);
        let reader = GenerationFiles::shared(files.logical_path()).unwrap();
        assert!(files.acquire_conversion().is_err());
        drop(reader);
        for suffix in ["-wal", "-shm", "-journal"] {
            let path = sidecar(&source.path, suffix);
            std::fs::write(&path, b"").unwrap();
            assert!(!files.retained_database_report().unwrap().retained[0].deletable);
            let lease = files.acquire_conversion().unwrap();
            assert!(files.retire_database(&request).is_err());
            drop(lease);
            std::fs::remove_file(path).unwrap();
        }
        assert!(source.path.exists());
    }
    #[cfg(unix)]
    #[test]
    fn refuses_linked_predecessors() {
        let (dir, files, source, _target) = fixture();
        let request = request(&files);
        let alias = dir.path().join("hardlink");
        std::fs::hard_link(&source.path, &alias).unwrap();
        assert!(files.retained_database_report().is_err());
        let _lease = files.acquire_conversion().unwrap();
        assert!(files.retire_database(&request).is_err());
        assert!(source.path.exists());
    }
    #[tokio::test]
    async fn converted_store_retirement_preserves_live_data_and_reopens_without_legacy_file() {
        use crate::{
            ServerKey, Store,
            encryption::{self, DbKey, EncryptionConfig, EncryptionMode},
            online_maintenance::ConversionOptions,
        };
        for encrypted in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("mail.db");
            let key = ServerKey::generate();
            let config = if encrypted {
                EncryptionConfig::with_key(EncryptionMode::Required, DbKey::generate())
            } else {
                EncryptionConfig::off()
            };
            let raw = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(
                    encryption::connect_options(&path, config.key.as_ref(), true)
                        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                        .pragma("recursive_triggers", "ON"),
                )
                .await
                .unwrap();
            sqlx::migrate!("./migrations").run(&raw).await.unwrap();
            raw.close().await;
            let store = Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &config)
                .await
                .unwrap();
            store
                .set_setting("retirement-data-probe", "preserved")
                .await
                .unwrap();
            store
                .start_online_conversion(ConversionOptions {
                    pause_ms: 0,
                    max_cutover_ms: 10_000,
                    ..Default::default()
                })
                .await
                .unwrap();
            let report = tokio::time::timeout(std::time::Duration::from_secs(60), async {
                loop {
                    let status = store.online_conversion_status().unwrap();
                    assert!(
                        !matches!(status.phase.as_str(), "failed" | "cancelled"),
                        "conversion fixture failed: {:?}",
                        status.error
                    );
                    if status.phase == "completed" {
                        let report = store.retained_databases().await.unwrap();
                        if report.retained.first().is_some_and(|row| row.deletable) {
                            break report;
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            })
            .await
            .expect("conversion fixture completes and closes predecessor sidecars");
            let row = &report.retained[0];
            assert_eq!(row.generation_id, "legacy");
            store
                .retire_retained_database(RetireDatabaseRequest {
                    generation_id: row.generation_id.clone(),
                    expected_active_generation: report.active_generation.unwrap(),
                    file_identity: row.file_identity.clone(),
                    confirmation: row.file_name.clone(),
                })
                .await
                .unwrap();
            assert!(!path.exists());
            assert_eq!(
                store
                    .get_setting("retirement-data-probe")
                    .await
                    .unwrap()
                    .as_deref(),
                Some("preserved")
            );
            store.close().await;
            let reopened = Store::open_with_encryption(path.to_str().unwrap(), key, &config)
                .await
                .unwrap();
            assert_eq!(
                reopened
                    .get_setting("retirement-data-probe")
                    .await
                    .unwrap()
                    .as_deref(),
                Some("preserved")
            );
            assert!(
                reopened
                    .retained_databases()
                    .await
                    .unwrap()
                    .retained
                    .is_empty()
            );
            reopened.close().await;
        }
    }
}
