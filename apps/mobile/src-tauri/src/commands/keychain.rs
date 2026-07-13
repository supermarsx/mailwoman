//! Secure key/value store for the mobile shell (§2.1 "Auth token store" + "Secure
//! store"; frozen `tauri.ts` names `mw_keychain_get/set/delete`).
//!
//! The desktop shell backs these with the `keyring` crate (DPAPI / Keychain /
//! Secret Service). On Android there is no `keyring` backend, so the native
//! secure store is **Android's `EncryptedSharedPreferences`** (AES-256, keyed by
//! the hardware-backed Android Keystore) via the Kotlin
//! `MailwomanMobilePlugin.keychainGet/keychainSet/keychainDelete` commands, reached
//! through the [`MobileBridge`](super::MobileBridge). The JS layer picks the
//! `service` namespace (`mailwoman.session` for the bearer token, `mailwoman.secure`
//! for the key-vault-passphrase wrap) exactly like desktop, so the two never
//! collide.
//!
//! On the desktop **host** build there is no Android Keystore, so every command
//! degrades to `Ok(None)` / no-op — keeping the crate `cargo check`-clean without
//! the mobile toolchain. The host build is not a real target (it exists only for
//! `cargo build --workspace` + unit tests), so a no-op secure store there is honest.

use tauri::{AppHandle, Runtime};

#[cfg(mobile)]
use crate::commands::MobileBridge;
#[cfg(mobile)]
use tauri::Manager;

#[cfg(mobile)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetReply {
    /// The stored value, or `null` when the `(service, key)` entry is unset.
    value: Option<String>,
}

/// Read the secret at `(service, key)`, or `null` when unset (§2.1 `secureGet` /
/// `getSessionToken`). `null` is the normal "not set yet" state, not an error, so
/// the caller degrades gracefully.
#[tauri::command]
pub fn mw_keychain_get<R: Runtime>(
    app: AppHandle<R>,
    service: String,
    key: String,
) -> Result<Option<String>, String> {
    #[cfg(mobile)]
    {
        let reply: GetReply = app.state::<MobileBridge<R>>().run(
            "keychainGet",
            serde_json::json!({ "service": service, "key": key }),
        )?;
        Ok(reply.value)
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, service, key);
        Ok(None)
    }
}

/// Store `value` under `(service, key)` in the OS-backed secure store, overwriting
/// any existing secret (§2.1 `secureSet` / `setSessionToken`).
#[tauri::command]
pub fn mw_keychain_set<R: Runtime>(
    app: AppHandle<R>,
    service: String,
    key: String,
    value: String,
) -> Result<(), String> {
    #[cfg(mobile)]
    {
        let _: serde_json::Value = app.state::<MobileBridge<R>>().run(
            "keychainSet",
            serde_json::json!({ "service": service, "key": key, "value": value }),
        )?;
        Ok(())
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, service, key, value);
        Ok(())
    }
}

/// Delete the credential at `(service, key)` (§2.1 `secureDelete` /
/// `clearSessionToken`). Deleting a missing entry is an idempotent no-op success.
#[tauri::command]
pub fn mw_keychain_delete<R: Runtime>(
    app: AppHandle<R>,
    service: String,
    key: String,
) -> Result<(), String> {
    #[cfg(mobile)]
    {
        let _: serde_json::Value = app.state::<MobileBridge<R>>().run(
            "keychainDelete",
            serde_json::json!({ "service": service, "key": key }),
        )?;
        Ok(())
    }
    #[cfg(not(mobile))]
    {
        let _ = (app, service, key);
        Ok(())
    }
}
