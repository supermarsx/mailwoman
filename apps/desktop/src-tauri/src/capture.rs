//! Screen-capture protection — the REAL native version (SPEC §7.6 / plan §3 e4).
//!
//! This is the native counterpart of V4's honest web watermark. Where the OS
//! provides a genuine capture-exclusion primitive, Mailwoman uses it; where it does
//! not, it says so plainly (returns `{ supported: false }`) and the caller keeps the
//! V4 watermark. No security theatre — see `docs/security/screen-capture.md`.
//!
//! Desktop coverage (this module):
//!   * **Windows** — `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)`, applied by
//!     Tauri's `WebviewWindow::set_content_protected(true)`. A screenshot or screen
//!     recording of the protected window captures **black**. → `supported: true`.
//!   * **macOS** — `NSWindow.sharingType = .none`, also via `set_content_protected`.
//!     Excludes the window from `ScreenCaptureKit`/screenshots. → `supported: true`.
//!   * **Linux** — WebKitGTK/X11/Wayland expose no reliable capture-exclusion API;
//!     `set_content_protected` is a no-op there. We do NOT call it and report
//!     `supported: false` so the SPA keeps the watermark (honest, no over-claim).
//!
//! Mobile FLAG_SECURE (Android) lives in the custom Kotlin plugin under
//! `apps/mobile/src-tauri/android-src/FlagSecurePlugin.kt`; iOS capture *detection*
//! (there is no prevention API) is the documented best-effort skeleton in
//! `apps/mobile/src-tauri/ios-src/`.
//!
//! ## Command surface (for e7 to register — do NOT wire it here)
//! This module owns the logic only; the shared `invoke_handler` registration in
//! `lib.rs` is e7's. e7 adds `mod capture;` and
//! `tauri::generate_handler![capture::set_capture_protection, ...]`. The SPA's
//! `platform/tauri.ts` (e6) calls it as:
//!
//! ```ignore
//! import { invoke } from '@tauri-apps/api/core';
//! const { supported } = await invoke<{ supported: boolean }>(
//!   'set_capture_protection', { enabled: true },
//! );
//! ```

use serde::Serialize;

/// Result of the `setCaptureProtection` capability (frozen §2.1 `CapabilityResult`).
/// `supported: false` means this OS cannot exclude the window from capture, so the
/// caller must fall back to the visible watermark — it is NOT an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CaptureResult {
    pub supported: bool,
}

/// Whether the host OS provides a genuine capture-exclusion primitive that Tauri's
/// `set_content_protected` drives. Kept honest per §7.6: only Windows and macOS
/// qualify today; everything else (Linux, and — via their own paths — the browser
/// and iOS) returns `false` so the watermark stays.
pub const fn capture_supported() -> bool {
    cfg!(any(target_os = "windows", target_os = "macos"))
}

/// The window operation this module needs, abstracted so the decision logic is unit-
/// testable without a live WebView2/Wry window or the `tauri` `test` feature. The
/// blanket impl below forwards to Tauri; tests use a recording fake.
pub trait ContentProtect {
    fn set_content_protected(&self, protected: bool) -> Result<(), String>;
}

impl<R: tauri::Runtime> ContentProtect for tauri::WebviewWindow<R> {
    fn set_content_protected(&self, protected: bool) -> Result<(), String> {
        tauri::WebviewWindow::set_content_protected(self, protected).map_err(|e| e.to_string())
    }
}

/// Core decision, parameterised on `supported` so both branches are testable on any
/// platform: when the OS supports it, drive the native content-protection call; when
/// it does not, do nothing and report `supported: false` (the caller keeps the
/// watermark). Applies to both enable and disable so toggling off is also native.
fn apply_capture_protection(
    window: &impl ContentProtect,
    enabled: bool,
    supported: bool,
) -> Result<CaptureResult, String> {
    if supported {
        window.set_content_protected(enabled)?;
    }
    Ok(CaptureResult { supported })
}

/// Turn the OS screen-capture exclusion on/off for the app window.
///
/// Backs the frozen `Platform.setCaptureProtection(enabled)` (§2.1). On Windows and
/// macOS this calls `WebviewWindow::set_content_protected(enabled)` (→
/// `WDA_EXCLUDEFROMCAPTURE` / `NSWindow` sharing-type `.none`) and returns
/// `{ supported: true }`. On every other desktop OS it makes no OS call and returns
/// `{ supported: false }` so the SPA keeps the V4 watermark.
#[tauri::command]
pub async fn set_capture_protection<R: tauri::Runtime>(
    window: tauri::WebviewWindow<R>,
    enabled: bool,
) -> Result<CaptureResult, String> {
    apply_capture_protection(&window, enabled, capture_supported())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// Records every `set_content_protected` call so tests can assert the call path
    /// without a real window. `fail` forces the error path.
    struct FakeWindow {
        calls: RefCell<Vec<bool>>,
        fail: bool,
    }

    impl FakeWindow {
        fn new() -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
                fail: false,
            }
        }
        fn failing() -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
                fail: true,
            }
        }
    }

    impl ContentProtect for FakeWindow {
        fn set_content_protected(&self, protected: bool) -> Result<(), String> {
            self.calls.borrow_mut().push(protected);
            if self.fail {
                Err("mock failure".to_string())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn supported_enable_calls_content_protected_true() {
        let w = FakeWindow::new();
        let r = apply_capture_protection(&w, true, true).unwrap();
        assert!(r.supported);
        assert_eq!(w.calls.borrow().as_slice(), &[true]);
    }

    #[test]
    fn supported_disable_calls_content_protected_false() {
        let w = FakeWindow::new();
        let r = apply_capture_protection(&w, false, true).unwrap();
        assert!(r.supported);
        assert_eq!(w.calls.borrow().as_slice(), &[false]);
    }

    #[test]
    fn unsupported_makes_no_os_call_and_reports_false() {
        // Honest degrade (§7.6): on an OS without capture exclusion we never touch
        // the window and tell the caller to keep the watermark.
        let w = FakeWindow::new();
        let r = apply_capture_protection(&w, true, false).unwrap();
        assert!(!r.supported);
        assert!(w.calls.borrow().is_empty());
    }

    #[test]
    fn os_error_propagates() {
        let w = FakeWindow::failing();
        let err = apply_capture_protection(&w, true, true).unwrap_err();
        assert!(err.contains("mock failure"));
    }

    #[test]
    fn capture_supported_matches_target_os() {
        // Compile-time honesty matrix: Win/macOS true, everything else false.
        assert_eq!(
            capture_supported(),
            cfg!(any(target_os = "windows", target_os = "macos")),
        );
    }

    #[test]
    fn result_serializes_to_supported_field() {
        let json = serde_json::to_string(&CaptureResult { supported: true }).unwrap();
        assert_eq!(json, r#"{"supported":true}"#);
    }
}
