//! OS keychain commands (plan §2.1 "Auth token store" + "Secure store"; §3 e1).
//!
//! Backs the capability-layer `getSessionToken`/`setSessionToken`/`clearSessionToken`
//! (bearer session for native clients, §2.2) **and** `secureGet`/`secureSet`/
//! `secureDelete` (the key-vault-passphrase wrap) with the OS-native credential
//! store via the `keyring` crate: Windows Credential Manager (DPAPI) on Windows,
//! Keychain on macOS, Secret Service on Linux. Both surfaces share one generic
//! command trio; the JS layer picks the `service` namespace so the two never
//! collide (`SERVICE_SESSION` vs `SERVICE_SECURE` below).
//!
//! Commands registered by e7 (`tauri::generate_handler!`):
//!   * `mw_keychain_set(service, key, value)`  -> Result<(), String>
//!   * `mw_keychain_get(service, key)`         -> Result<Option<String>, String>
//!   * `mw_keychain_delete(service, key)`      -> Result<(), String>
//!
//! The commands are thin `async` wrappers (Tauri schedules them off the UI thread)
//! over the synchronous `keyring` helpers below, which the unit tests drive directly
//! against the real OS store — no async runtime needed to test the credential logic.

use keyring::{Entry, Error as KeyringError};

/// Namespace for the native bearer session token (native-auth mode, §2.2). The JS
/// layer stores the token under `service = SERVICE_SESSION`.
pub const SERVICE_SESSION: &str = "mailwoman.session";
/// Namespace for the secure-store surface (key-vault passphrase wrap, arbitrary keys).
pub const SERVICE_SECURE: &str = "mailwoman.secure";

/// Build a `keyring` entry for `(service, key)`. `service` namespaces the surface
/// (session vs secure-store) and `key` is the item name within it; together they
/// address one credential in the OS store.
fn entry(service: &str, key: &str) -> Result<Entry, String> {
    Entry::new(service, key).map_err(|e| format!("keychain entry {service}/{key}: {e}"))
}

/// Store `value` under `(service, key)`, overwriting any existing secret.
fn set_secret(service: &str, key: &str, value: &str) -> Result<(), String> {
    entry(service, key)?
        .set_password(value)
        .map_err(|e| format!("keychain set {service}/{key}: {e}"))
}

/// Read the secret at `(service, key)`. `Ok(None)` = no such credential (a normal
/// "not set yet" state, not an error) so the caller degrades gracefully.
fn get_secret(service: &str, key: &str) -> Result<Option<String>, String> {
    match entry(service, key)?.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(e) => Err(format!("keychain get {service}/{key}: {e}")),
    }
}

/// Delete the credential at `(service, key)`. Deleting a missing entry is a no-op
/// success (idempotent clear, e.g. `clearSessionToken` on an already-clean store).
fn delete_secret(service: &str, key: &str) -> Result<(), String> {
    match entry(service, key)?.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
        Err(e) => Err(format!("keychain delete {service}/{key}: {e}")),
    }
}

/// Store `value` under `(service, key)` in the OS credential store.
#[tauri::command]
pub async fn mw_keychain_set(service: String, key: String, value: String) -> Result<(), String> {
    set_secret(&service, &key, &value)
}

/// Read the secret at `(service, key)`, or `null` when unset.
#[tauri::command]
pub async fn mw_keychain_get(service: String, key: String) -> Result<Option<String>, String> {
    get_secret(&service, &key)
}

/// Delete the credential at `(service, key)` (idempotent).
#[tauri::command]
pub async fn mw_keychain_delete(service: String, key: String) -> Result<(), String> {
    delete_secret(&service, &key)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Small unique suffix so concurrent/repeat runs never share a credential name.
    fn unique() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{nanos:x}")
    }

    /// Live round-trip against the real OS store (Credential Manager/DPAPI on
    /// Windows). Uses a unique key so parallel runs never clash, and always cleans
    /// up. Plan §3 e1 acceptance: "a keyring round-trip test (set->get->delete) —
    /// this actually works on Windows (DPAPI), so run it live".
    #[test]
    fn keychain_round_trips_against_the_os_store() {
        let key = format!("test-{}", unique());
        let secret = "s3cr3t-bearer-token";

        // Not present initially.
        assert_eq!(get_secret(SERVICE_SESSION, &key).unwrap(), None);

        // set -> get returns it.
        set_secret(SERVICE_SESSION, &key, secret).expect("set on the native store");
        assert_eq!(
            get_secret(SERVICE_SESSION, &key).unwrap(),
            Some(secret.to_string())
        );

        // overwrite.
        set_secret(SERVICE_SESSION, &key, "rotated").unwrap();
        assert_eq!(
            get_secret(SERVICE_SESSION, &key).unwrap(),
            Some("rotated".to_string())
        );

        // delete -> gone.
        delete_secret(SERVICE_SESSION, &key).unwrap();
        assert_eq!(get_secret(SERVICE_SESSION, &key).unwrap(), None);

        // delete again is a no-op success (idempotent clear).
        delete_secret(SERVICE_SESSION, &key).unwrap();
    }

    #[test]
    fn session_and_secure_namespaces_are_isolated() {
        let key = format!("iso-{}", unique());
        set_secret(SERVICE_SESSION, &key, "from-session").unwrap();
        // The same key under the secure namespace must not see the session value.
        assert_eq!(get_secret(SERVICE_SECURE, &key).unwrap(), None);
        delete_secret(SERVICE_SESSION, &key).unwrap();
    }
}
