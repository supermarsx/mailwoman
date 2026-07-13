//! Biometric app-lock for the mobile shell (§2.1 "Biometric app-lock"; frozen
//! `tauri.ts` names `mw_biometric_available` / `mw_biometric_authenticate`).
//!
//! On Android/iOS the real prompt is `tauri-plugin-biometric` (fingerprint / face /
//! device credential). The frozen `platform/tauri.ts` invokes the bare `mw_biometric_*`
//! command names (matching desktop's Windows-Hello commands), NOT the biometric
//! plugin's own `plugin:biometric|*` commands — so these thin app-level commands
//! adapt the plugin's Rust API (`BiometricExt`) to the frozen names. `authenticate`
//! resolves `true` only on a verified prompt and `false` on cancel/failure, mirroring
//! the desktop contract.
//!
//! `tauri-plugin-biometric` is `#![cfg(mobile)]` (it does not exist on the desktop
//! host build used for `cargo build --workspace`), so both commands are gated: the
//! real path is `#[cfg(mobile)]`, and the host build degrades to
//! "unavailable"/`false` (the SPA then falls back to the passphrase unlock — honest,
//! §7.6 posture). The device path is exercised in the CI Android build.

use tauri::{AppHandle, Runtime};

/// True when the device can perform a biometric / device-credential prompt now
/// (§2.1 `biometricAvailable`).
#[tauri::command]
pub async fn mw_biometric_available<R: Runtime>(app: AppHandle<R>) -> Result<bool, String> {
    #[cfg(mobile)]
    {
        use tauri_plugin_biometric::BiometricExt;
        match app.biometric().status() {
            Ok(status) => Ok(status.is_available),
            Err(e) => Err(format!("biometric status: {e}")),
        }
    }
    #[cfg(not(mobile))]
    {
        let _ = app;
        Ok(false)
    }
}

/// Prompt the user for biometric (or device-credential) consent (§2.1
/// `biometricAuthenticate`). Returns `true` on a verified prompt, `false` on
/// cancel/failure.
#[tauri::command]
pub async fn mw_biometric_authenticate<R: Runtime>(
    app: AppHandle<R>,
    reason: String,
) -> Result<bool, String> {
    #[cfg(mobile)]
    {
        use tauri_plugin_biometric::{AuthOptions, BiometricExt};
        // Allow the device PIN/pattern/password as a fallback so the app-lock still
        // works on devices without enrolled biometrics.
        let options = AuthOptions {
            allow_device_credential: true,
            ..Default::default()
        };
        Ok(app.biometric().authenticate(reason, options).is_ok())
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, reason);
        Ok(false)
    }
}
