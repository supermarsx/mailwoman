//! Share targets + file handlers (§2.1 `onShareTarget`, plan §3 e2).
//!
//! Android delivers a share (`ACTION_SEND`) or a file open (`ACTION_VIEW` on a
//! `.eml`/`.ics`/`.vcf`/`.msg` — see `android-src/manifest-intents.xml`) as an
//! `Intent` to the activity. The Kotlin `MailwomanMobilePlugin` captures that
//! intent (on launch and via `onNewIntent`), reads the shared text / file bytes,
//! and (a) triggers the `shareTarget` plugin event and (b) caches the payload so
//! a late-subscribing webview can pull it.
//!
//! `share_take_pending` returns and clears that cached payload. The shape is
//! **loose by design** (matching the frozen TS `ShareTargetPayload`): the Kotlin
//! side emits `{ title?, text?, url?, files?: [{ name, mime, bytesB64 }] }`, and
//! e6's `tauri.ts` maps `bytesB64` → `Uint8Array` for the frozen interface. We
//! forward it as opaque JSON so the Rust surface does not have to re-freeze the
//! payload schema. On the desktop host there is no share intent → `Ok(None)`.

use tauri::{AppHandle, Runtime};

#[cfg(mobile)]
use crate::commands::MobileBridge;
#[cfg(mobile)]
use tauri::Manager;

/// Return and clear any pending shared/opened payload captured from the launch
/// or a subsequent intent. `null` when nothing is pending. §2.1 `onShareTarget`.
#[tauri::command]
pub fn share_take_pending<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Option<serde_json::Value>, String> {
    #[cfg(mobile)]
    {
        // Kotlin resolves `{ payload: <obj|null> }` (its `resolve` needs an
        // object); null `payload` means nothing is pending.
        let reply: serde_json::Value =
            app.state::<MobileBridge<R>>().run("takePendingShare", ())?;
        Ok(match reply.get("payload") {
            Some(p) if !p.is_null() => Some(p.clone()),
            _ => None,
        })
    }
    #[cfg(not(mobile))]
    {
        let _ = app;
        Ok(None)
    }
}
