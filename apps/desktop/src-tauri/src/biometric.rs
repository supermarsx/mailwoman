//! Biometric app-lock commands (plan §2.1 "Biometric app-lock"; §3 e1).
//!
//! Backs the capability-layer `biometricAvailable()` / `biometricAuthenticate({reason})`
//! for the desktop app-lock. `tauri-plugin-biometric` is `#![cfg(mobile)]` (it backs
//! Android/iOS only), so the desktop path uses **Windows Hello** directly via the
//! `windows` crate's `UserConsentVerifier` (fingerprint / face / PIN). On macOS and
//! Linux there is no first-party desktop biometric here, so the commands report
//! "unavailable" and the SPA honestly falls back to the passphrase unlock.
//!
//! Commands registered by e7 (`tauri::generate_handler!`):
//!   * `mw_biometric_available()`             -> Result<bool, String>
//!   * `mw_biometric_authenticate(reason)`    -> Result<bool, String>
//!
//! `authenticate` shows a real OS consent dialog, so it is exercised in the live E2E
//! / a documented manual step, not a headless unit test; `available()` is a
//! non-interactive probe and is smoke-tested on Windows.

/// True when the OS can perform a biometric/PIN consent prompt right now.
#[tauri::command]
pub async fn mw_biometric_available() -> Result<bool, String> {
    imp::available()
}

/// Prompt the user for biometric (or Windows Hello PIN) consent. Returns `true` only
/// on a verified prompt; `false` on cancel/retry-exhausted; `Err` on a hard failure.
#[tauri::command]
pub async fn mw_biometric_authenticate(reason: String) -> Result<bool, String> {
    imp::authenticate(&reason)
}

#[cfg(windows)]
mod imp {
    use windows::Security::Credentials::UI::{
        UserConsentVerificationResult, UserConsentVerifier, UserConsentVerifierAvailability,
    };
    use windows::core::HSTRING;

    pub fn available() -> Result<bool, String> {
        let availability = UserConsentVerifier::CheckAvailabilityAsync()
            .map_err(|e| format!("Windows Hello availability check: {e}"))?
            .get()
            .map_err(|e| format!("Windows Hello availability await: {e}"))?;
        Ok(availability == UserConsentVerifierAvailability::Available)
    }

    pub fn authenticate(reason: &str) -> Result<bool, String> {
        let message = HSTRING::from(reason);
        let result = UserConsentVerifier::RequestVerificationAsync(&message)
            .map_err(|e| format!("Windows Hello request: {e}"))?
            .get()
            .map_err(|e| format!("Windows Hello await: {e}"))?;
        Ok(result == UserConsentVerificationResult::Verified)
    }
}

#[cfg(not(windows))]
mod imp {
    // macOS (LocalAuthentication) / Linux have no first-party desktop biometric wired
    // here; the SPA falls back to the passphrase unlock (honest, plan §7.6 posture).
    pub fn available() -> Result<bool, String> {
        Ok(false)
    }

    pub fn authenticate(_reason: &str) -> Result<bool, String> {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `available()` is a non-interactive probe; it must return a definite bool
    /// (never panic) regardless of whether Hello is enrolled on this machine.
    #[test]
    fn available_probe_returns_a_bool() {
        let r = imp::available();
        assert!(r.is_ok(), "availability probe should not hard-error: {r:?}");
    }

    /// On non-Windows the desktop biometric path is honestly unavailable.
    #[cfg(not(windows))]
    #[test]
    fn non_windows_is_unavailable() {
        assert_eq!(imp::available().unwrap(), false);
        assert_eq!(imp::authenticate("unlock").unwrap(), false);
    }
}
