//! Retain cross-process exclusivity when private SQLite workers may still exist.
//!
//! Dropping a SQLx connection sends shutdown to its worker; it does not confirm
//! termination. Runtime/task cancellation must therefore never automatically
//! admit another process that can select or modify this database installation.
use crate::generation_files::ConversionLease;

pub(crate) struct CopyFence {
    lease: Option<ConversionLease>,
    complete: bool,
}
impl CopyFence {
    pub(crate) fn new(lease: ConversionLease) -> Self {
        Self {
            lease: Some(lease),
            complete: false,
        }
    }
    /// Call only after all private connection closes have completed, including
    /// temporary target pools. Cancellation or an ambiguous close/open error
    /// requires retaining the fence until process restart.
    pub(crate) fn mark_complete(&mut self) {
        self.complete = true;
    }
}
impl Drop for CopyFence {
    fn drop(&mut self) {
        if !self.complete {
            if let Some(lease) = self.lease.take() {
                // Intentionally retain the OS lease and its owner. Normal work
                // through the current process's pool remains available; another
                // process/conversion must wait for a complete process restart.
                std::mem::forget(lease);
                eprintln!(
                    "[mw-store] SQLite conversion workers were not confirmed closed; lifecycle exclusivity is retained until process restart"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation_files::GenerationFiles;
    use std::{path::PathBuf, sync::Arc};

    #[test]
    fn confirmed_private_worker_closure_restores_shared_access() {
        fn assert_send<T: Send>() {}
        assert_send::<CopyFence>();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mail.db");
        let files = Arc::new(GenerationFiles::shared(&path).unwrap());
        let mut guard = CopyFence::new(files.acquire_conversion().unwrap());
        assert!(GenerationFiles::shared(&path).is_err());
        guard.mark_complete();
        drop(guard);
        assert!(GenerationFiles::shared(&path).is_ok());
        assert!(GenerationFiles::lock(&path).is_err());
    }

    #[test]
    fn unconfirmed_child() {
        let Some(path) = std::env::var_os("MW_COPY_FENCE_ABORT_FIXTURE") else {
            return;
        };
        let path = PathBuf::from(path);
        let files = Arc::new(GenerationFiles::shared(&path).unwrap());
        let guard = CopyFence::new(files.acquire_conversion().unwrap());
        drop(guard);
        drop(files);
        // Even dropping every ordinary owner cannot release an unconfirmed
        // worker fence. This subprocess must actually exit to release it.
        assert!(GenerationFiles::shared(&path).is_err());
        assert!(GenerationFiles::lock(&path).is_err());
        std::fs::write(path.with_extension("fence-verified"), b"retained").unwrap();
    }

    #[test]
    fn unconfirmed_task_drop_retains_exclusivity_until_process_exit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mail.db");
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "lifecycle_copy_guard::tests::unconfirmed_child",
                "--nocapture",
            ])
            .env("MW_COPY_FENCE_ABORT_FIXTURE", &path)
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(
            std::fs::read(path.with_extension("fence-verified")).unwrap(),
            b"retained"
        );
        // A real process exit, unlike runtime/task cancellation, closes every
        // SQLite worker and releases the intentionally retained lifecycle lock.
        drop(GenerationFiles::lock(&path).unwrap());
    }
}
