//! Drag-out attachment command (plan §2.1 "Share / drag-out"; §3 e1).
//!
//! Backs the capability-layer `startDragOut(files)` — dragging an attachment OUT of
//! Mailwoman onto the desktop or another app. The OS drag itself is initiated by the
//! webview from a real file path, so the Rust side's job is to materialize each
//! attachment's bytes into a temp file and hand back the paths; e6 then starts the
//! native drag with them. Attachment bytes are the SPA's already-decrypted data
//! (blob-id resolution happens in the SPA before this call), so nothing secret is
//! invented here — the bytes are written to a per-app temp dir the shell owns.
//!
//! Commands registered by e7 (`tauri::generate_handler!`):
//!   * `mw_dragout_materialize(app, files: Vec<DragOutFile>)` -> Result<Vec<String>, String>
//!
//! The pure logic (name sanitization + materialization) is unit-tested against a
//! temp dir; the command is a thin wrapper resolving the app's temp dir.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{Manager, Runtime};

/// A file the user drags out (mirrors `DragOutFile` in `platform/index.ts`). The
/// SPA resolves `blobId` to `bytes` before calling, so `bytes` is required here.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DragOutFile {
    pub name: String,
    #[serde(default)]
    pub mime: String,
    /// Raw attachment bytes (the SPA supplies these; `blobId` is resolved SPA-side).
    #[serde(default)]
    pub bytes: Option<Vec<u8>>,
}

/// Reduce an attachment name to a safe basename: strip any directory components and
/// reject empties so a hostile/odd filename can never escape the drag-out temp dir
/// (path-traversal guard). `..` and separators collapse to the trailing segment.
pub fn sanitize_name(name: &str) -> String {
    let base = name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches('.');
    if base.is_empty() {
        "attachment".to_string()
    } else {
        base.to_string()
    }
}

/// Write each file's bytes into `dir` under its sanitized name, returning the paths.
/// A file missing its bytes is an error (the SPA must resolve `blobId` first).
pub fn materialize_to(dir: &Path, files: &[DragOutFile]) -> Result<Vec<PathBuf>, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    let mut paths = Vec::with_capacity(files.len());
    for f in files {
        let bytes = f
            .bytes
            .as_ref()
            .ok_or_else(|| format!("drag-out file {} has no bytes", f.name))?;
        let path = dir.join(sanitize_name(&f.name));
        std::fs::write(&path, bytes).map_err(|e| format!("write {}: {e}", path.display()))?;
        paths.push(path);
    }
    Ok(paths)
}

/// Materialize the drag-out files into the shell's temp dir; returns their absolute
/// paths for the webview to start an OS drag with.
#[tauri::command]
pub async fn mw_dragout_materialize<R: Runtime>(
    app: tauri::AppHandle<R>,
    files: Vec<DragOutFile>,
) -> Result<Vec<String>, String> {
    let dir = app
        .path()
        .temp_dir()
        .map_err(|e| format!("temp_dir: {e}"))?
        .join("mailwoman-dragout");
    let paths = materialize_to(&dir, &files)?;
    Ok(paths
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_strips_directories_and_traversal() {
        assert_eq!(sanitize_name("report.pdf"), "report.pdf");
        assert_eq!(sanitize_name("../../etc/passwd"), "passwd");
        assert_eq!(sanitize_name(r"C:\Windows\System32\cmd.exe"), "cmd.exe");
        assert_eq!(sanitize_name("sub/dir/photo.png"), "photo.png");
    }

    #[test]
    fn sanitize_falls_back_for_empty_or_dotty_names() {
        assert_eq!(sanitize_name(""), "attachment");
        assert_eq!(sanitize_name("   "), "attachment");
        assert_eq!(sanitize_name(".."), "attachment");
        assert_eq!(sanitize_name("/"), "attachment");
    }

    #[test]
    fn materializes_bytes_into_files() {
        let dir = std::env::temp_dir().join(format!("mw-dragout-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let files = vec![
            DragOutFile {
                name: "invoice.pdf".into(),
                mime: "application/pdf".into(),
                bytes: Some(b"%PDF-1.7 fake".to_vec()),
            },
            DragOutFile {
                name: "../escape.txt".into(),
                mime: "text/plain".into(),
                bytes: Some(b"hello".to_vec()),
            },
        ];
        let paths = materialize_to(&dir, &files).unwrap();
        assert_eq!(paths.len(), 2);
        // Traversal name was contained to the drag-out dir.
        assert_eq!(paths[1].parent().unwrap(), dir);
        assert_eq!(std::fs::read(&paths[0]).unwrap(), b"%PDF-1.7 fake");
        assert_eq!(std::fs::read(&paths[1]).unwrap(), b"hello");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn errors_when_bytes_missing() {
        let dir = std::env::temp_dir().join(format!("mw-dragout-nobytes-{}", std::process::id()));
        let files = vec![DragOutFile {
            name: "x.bin".into(),
            mime: String::new(),
            bytes: None,
        }];
        assert!(materialize_to(&dir, &files).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
