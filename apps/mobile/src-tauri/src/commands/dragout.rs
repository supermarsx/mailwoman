//! Drag-out attachments — mobile no-op (§2.1 "Share / drag-out"; frozen `tauri.ts`
//! name `mw_dragout_materialize`).
//!
//! Drag-out (dragging an attachment OUT of the app onto the desktop) is a desktop
//! affordance; Android/iOS have no equivalent OS drag surface. The command exists so
//! the frozen `platform/tauri.ts` invoke resolves natively on mobile, and it honestly
//! materializes nothing: it returns an empty path list. (Mobile sharing OUT of the
//! app is the OS share sheet, a separate future capability, not drag-out.)

use serde::{Deserialize, Serialize};

/// A file the SPA would drag out (mirrors `DragOutFile` in `platform/index.ts`).
/// Accepted for wire-compatibility with the frozen call; unused on mobile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DragOutFile {
    pub name: String,
    #[serde(default)]
    pub mime: String,
    #[serde(default)]
    pub bytes: Option<Vec<u8>>,
}

/// Materialize drag-out files — a no-op on mobile, returning an empty path list.
#[tauri::command]
pub async fn mw_dragout_materialize(files: Vec<DragOutFile>) -> Result<Vec<String>, String> {
    let _ = files;
    Ok(Vec::new())
}
