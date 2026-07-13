//! Mailwoman desktop shell (Tauri v2 · plan §3 e0) — the **launching stub**.
//!
//! THIN client (SPEC §16 / plan §0): this crate embeds the hash-verified
//! `apps/web/dist` SPA (the byte-identical bundle `mw-server` serves via
//! `rust-embed`) and adds OS integration only. There is **no protocol logic and
//! no forked UI** here — the shell loads the same SPA a browser would and talks to
//! the user's Mailwoman server over the identical JMAP surface, selecting the
//! native capability layer (`apps/web/src/platform/tauri.ts`) via feature-detect.
//!
//! What e0 ships (this file): a window that loads the SPA and injects the frozen
//! `globalThis.__MW_CONFIG__` handshake (§2.1/§2.5) before the SPA runs, plus the
//! registered first-party plugins the later executors build on. The native
//! command surface is intentionally empty here:
//!   * e1 fills desktop capabilities (notifications/keychain/deep-link/biometric/
//!     badge/drag-out/multi-server) in this crate,
//!   * e3 adds the self-contained local-`mw-server` spawn (`selfcontained.rs`),
//!   * e4 adds screen-capture protection (`capture.rs`),
//!   * e7 mounts the bundle-hash gate + wires the Tauri commands ↔ `tauri.ts`.

use serde_json::json;
use tauri::{WebviewUrl, WebviewWindowBuilder};

/// The frozen JS handshake (§2.1/§2.5): the shell injects `globalThis.__MW_CONFIG__`
/// before the SPA executes so the capability layer (`platform/index.ts`) knows it
/// is running natively and where its server is. `serverUrl` is `null` in the stub
/// (the SPA's server-picker / e6 sets it; self-contained mode / e3 points it at the
/// loopback `mw-server`). Kept camelCase to match the existing `__MW_CONFIG__`
/// precedent (`viewers/max-security.ts`).
fn bootstrap_config_script() -> String {
    let cfg = json!({
        "platform": {
            "kind": "desktop",
            "os": std::env::consts::OS,
            "version": env!("CARGO_PKG_VERSION"),
        },
        // e6 threads this into the SPA transport; null → the SPA shows its server
        // picker. Self-contained mode (e3) overwrites it with the loopback URL.
        "serverUrl": serde_json::Value::Null,
        // e0 advertises the *shell* kind; the capability layer feature-detects the
        // real Tauri APIs at runtime and degrades per-method (§2.1).
        "native": true,
    });
    // Assign early so `readAdminFloor()`-style import-time reads see it.
    format!("globalThis.__MW_CONFIG__ = Object.assign(globalThis.__MW_CONFIG__ || {{}}, {cfg});")
}

/// UI-bundle integrity gate (§7.4 / §2.2 / risk #9). e7 replaces this stub with a
/// real check: read the build-emitted `bundle-hash.json`, hash the loaded
/// `apps/web/dist`, and refuse to point at any server on mismatch. The seam exists
/// now so the wiring point is frozen.
fn verify_bundle_integrity() -> bool {
    // Stub: e7 fills the SHA-256 comparison. Non-blocking until then.
    true
}

/// Build and run the desktop shell.
pub fn run() {
    let _ = verify_bundle_integrity();

    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // The main window is created in code (not tauri.conf.json) so the
            // `__MW_CONFIG__` handshake is registered as an initialization script
            // BEFORE the SPA loads (config-created windows would already be
            // navigating). e4 flips `content_protected` via the capture command.
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Mailwoman")
                .inner_size(1200.0, 800.0)
                .min_inner_size(720.0, 560.0)
                .content_protected(false)
                .initialization_script(bootstrap_config_script())
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the Mailwoman desktop shell");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_script_declares_desktop_native_config() {
        let s = bootstrap_config_script();
        assert!(s.contains("__MW_CONFIG__"));
        assert!(s.contains("\"kind\":\"desktop\""));
        assert!(s.contains("\"native\":true"));
        // serverUrl is null in the stub (the server picker / e3 / e6 sets it).
        assert!(s.contains("\"serverUrl\":null"));
    }

    #[test]
    fn bundle_integrity_stub_is_non_blocking() {
        assert!(verify_bundle_integrity());
    }
}
