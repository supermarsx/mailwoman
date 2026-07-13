//! App badge count (§2.1 `setBadgeCount`, plan §3 e2).
//!
//! Android has no universal launcher-badge API; the widely-supported mechanism
//! is a notification-channel badge / launcher-shortcut count, which the Kotlin
//! `MailwomanMobilePlugin.setBadge` sets (via the notification manager, honestly
//! degrading on launchers that ignore it — reported back as `supported`). On the
//! desktop host there is no launcher badge → `{ supported: false }`.

use serde::Serialize;
use tauri::{AppHandle, Runtime};

#[cfg(mobile)]
use crate::commands::MobileBridge;
#[cfg(mobile)]
use tauri::Manager;

/// Result of a capability that may be unavailable on the current OS (mirrors the
/// frozen TS `CapabilityResult`).
#[derive(Debug, Clone, Serialize)]
pub struct CapabilityResult {
    pub supported: bool,
}

/// Set the app badge to `count` (0 clears it). §2.1 `setBadgeCount`. Frozen name.
#[tauri::command]
pub fn mw_set_badge_count<R: Runtime>(
    app: AppHandle<R>,
    count: i64,
) -> Result<CapabilityResult, String> {
    let count = count.max(0);
    #[cfg(mobile)]
    {
        let res: CapabilityResultReply = app
            .state::<MobileBridge<R>>()
            .run("setBadge", serde_json::json!({ "count": count }))?;
        Ok(CapabilityResult {
            supported: res.supported,
        })
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, count);
        Ok(CapabilityResult { supported: false })
    }
}

#[cfg(mobile)]
#[derive(serde::Deserialize)]
struct CapabilityResultReply {
    supported: bool,
}
