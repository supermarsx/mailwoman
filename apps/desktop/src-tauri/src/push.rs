//! Desktop push commands (plan §2.1 "Push"; §3 e7 mount reconciliation).
//!
//! The capability layer (`apps/web/src/platform/tauri.ts`) invokes
//! `mw_push_subscribe` / `mw_push_unsubscribe` on EVERY native platform (the same
//! `tauri.ts` runs on desktop + mobile). Mobile backs these with UnifiedPush (e2's
//! plugin); the DESKTOP shell needs matching commands so every name `tauri.ts`
//! invokes resolves to a registered Rust command (plan §2 / the e7 punch-list).
//!
//! Honest posture (§7.6 / plan §1.7): the committed desktop push transport is Web
//! Push (VAPID), and a Web Push subscription is created **in the WebView by the SPA's
//! browser fallback** (`browser.ts` via `ServiceWorkerRegistration.pushManager`),
//! not in Rust — there is no OS push-registration API for a desktop Tauri app to
//! call. WebView2/WebKitGTK do not expose a functioning browser push service in the
//! Tauri asset context, so a background wake is not deliverable to a closed desktop
//! app today; while the app is open, the existing WS/SSE realtime path already
//! drains the same `StateChange` broadcast. So the desktop command **honestly
//! reports "no native push transport"** (`mw_push_subscribe` → `null`), and the SPA
//! keeps its foreground realtime updates. This is a documented desktop-background
//! limitation, not an error — Web Push (browser) + UnifiedPush (Android) are the
//! committed live wake paths.
//!
//! Commands registered by `lib.rs`:
//!   * `mw_push_subscribe()`   -> Result<Option<serde_json::Value>, String>  (→ null)
//!   * `mw_push_unsubscribe()` -> Result<(), String>

/// Desktop push subscribe. Returns `null` (no OS-level push transport to register
/// against on desktop; the SPA's Web Push fallback owns any in-WebView subscription).
/// Kept as a registered command so the frozen `tauri.ts` `pushSubscribe()` invoke
/// resolves natively rather than throwing.
#[tauri::command]
pub async fn mw_push_subscribe() -> Result<Option<serde_json::Value>, String> {
    Ok(None)
}

/// Desktop push unsubscribe — a no-op success (nothing was registered natively).
#[tauri::command]
pub async fn mw_push_unsubscribe() -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::async_runtime::block_on;

    #[test]
    fn subscribe_reports_no_native_transport() {
        // Honest degrade: desktop has no native push registration; the SPA keeps its
        // foreground realtime path. `null` (not an error) so `tauri.ts` returns null.
        assert_eq!(block_on(mw_push_subscribe()).unwrap(), None);
    }

    #[test]
    fn unsubscribe_is_a_noop_success() {
        assert!(block_on(mw_push_unsubscribe()).is_ok());
    }
}
