//! Screen-capture protection for the mobile shell (§7.6 / §2.1; frozen `tauri.ts`
//! name `mw_set_capture_protection`).
//!
//! On Android the genuine OS primitive is `FLAG_SECURE`, owned by e4's custom Kotlin
//! plugin `FlagSecurePlugin.kt` (a SEPARATE `@TauriPlugin` from the main
//! capability plugin). This command adapts the frozen `mw_set_capture_protection`
//! invoke to that plugin's `setCaptureProtection` command through the
//! [`CaptureBridge`](super::CaptureBridge) handle (registered in `commands::init`'s
//! `setup`). Android returns `{ supported: true }` — FLAG_SECURE is a real control.
//!
//! On the desktop **host** build there is no FlagSecure plugin, so the command
//! degrades to `{ supported: false }` → the SPA keeps the V4 watermark (honest,
//! §7.6). iOS has no prevention API (only detection, the documented best-effort
//! skeleton), so it also reports `{ supported: false }`.

use serde::Serialize;
use tauri::{AppHandle, Runtime};

#[cfg(target_os = "android")]
use crate::commands::CaptureBridge;
#[cfg(target_os = "android")]
use tauri::Manager;

/// Result of the capture-protection capability (mirrors the frozen TS
/// `CapabilityResult`). `supported: false` means the OS cannot exclude the window
/// from capture, so the caller keeps the watermark — it is NOT an error.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct CaptureResult {
    pub supported: bool,
}

#[cfg(target_os = "android")]
#[derive(serde::Deserialize)]
struct CaptureReply {
    supported: bool,
}

/// Turn the Android FLAG_SECURE screen-capture exclusion on/off (§2.1
/// `setCaptureProtection`). `supported: true` on Android (FLAG_SECURE is enforced by
/// the OS); `false` everywhere else (host / iOS) → the SPA keeps the watermark.
#[tauri::command]
pub fn mw_set_capture_protection<R: Runtime>(
    app: AppHandle<R>,
    enabled: bool,
) -> Result<CaptureResult, String> {
    #[cfg(target_os = "android")]
    {
        let reply: CaptureReply = app.state::<CaptureBridge<R>>().run(
            "setCaptureProtection",
            serde_json::json!({ "enabled": enabled }),
        )?;
        Ok(CaptureResult {
            supported: reply.supported,
        })
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (app, enabled);
        Ok(CaptureResult { supported: false })
    }
}
