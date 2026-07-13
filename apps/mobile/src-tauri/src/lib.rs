//! Mailwoman mobile shell (Tauri v2 · plan §3 e0) — the **launching stub**.
//!
//! THIN client, identical model to the desktop shell (SPEC §16 / plan §0): embeds
//! the hash-verified `apps/web/dist` SPA and adds mobile OS integration only —
//! **no protocol logic, no forked UI**. **Android is the buildable gate**; **iOS is
//! best-effort/documented** (needs macOS + an Apple account, plan §1.9/§9).
//!
//! What e0 ships (this file): the mobile entry point + a window that loads the SPA
//! and injects the frozen `globalThis.__MW_CONFIG__` handshake (§2.1/§2.5) with
//! `platform.kind = "android" | "ios"`. The native command surface is intentionally
//! empty here:
//!   * e2 fills mobile capabilities (UnifiedPush registration, share targets, file
//!     handlers, badge, biometric, multi-server),
//!   * e4 adds the Android `FLAG_SECURE` Kotlin plugin,
//!   * e7 mounts the bundle-hash gate + produces the APK.

use serde_json::json;
use tauri::{WebviewUrl, WebviewWindowBuilder};

// Mobile capability commands (UnifiedPush, share targets, badge, multi-server —
// plan §3 e2). Packaged as a self-contained Tauri plugin (`commands::init()`);
// e7 registers it in `run()` below with `.plugin(commands::init())`. Declared
// `pub` so the plugin is reachable from the crate's public API (and from e7's
// mount code) — this executor does NOT edit `run()`'s builder chain itself.
pub mod commands;

/// The platform kind for the running mobile target (§2.1). Compile-time on the
/// mobile targets; falls back to a neutral value on the host build.
fn platform_kind() -> &'static str {
    #[cfg(target_os = "android")]
    {
        "android"
    }
    #[cfg(target_os = "ios")]
    {
        "ios"
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        // Host build (unit tests / `cargo run` on the dev machine).
        "mobile-host"
    }
}

/// The frozen JS handshake (§2.1/§2.5): injected before the SPA runs so the
/// capability layer knows it is native and which mobile OS it is on. Mirrors the
/// desktop shell's `bootstrap_config_script` (kept camelCase per the `__MW_CONFIG__`
/// precedent).
fn bootstrap_config_script() -> String {
    let cfg = json!({
        "platform": {
            "kind": platform_kind(),
            "os": std::env::consts::OS,
            "version": env!("CARGO_PKG_VERSION"),
        },
        "serverUrl": serde_json::Value::Null,
        "native": true,
        // Enable the passive native consumers (notifications/badge/share/push) in
        // the SPA (`platform/capabilities.ts`); `isTauri()` already enables them in
        // a shell — explicit here to keep the contract symmetric with desktop.
        "capabilities": true,
    });
    format!("globalThis.__MW_CONFIG__ = Object.assign(globalThis.__MW_CONFIG__ || {{}}, {cfg});")
}

/// Build and run the mobile shell.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        // e2's mobile capability plugin (UnifiedPush / share targets / badge /
        // multi-server config). Self-contained `invoke_handler`; host-compiles
        // (commands degrade off the mobile target). Registers the Android/iOS
        // native bridge in its own `setup`.
        .plugin(commands::init());
    // `tauri-plugin-biometric` is `#![cfg(mobile)]` — it does not exist on the
    // desktop host build (used for `cargo build --workspace` / unit tests), so its
    // registration is gated to the mobile targets.
    #[cfg(mobile)]
    let builder = builder.plugin(tauri_plugin_biometric::init());
    builder
        .setup(|app| {
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Mailwoman")
                .initialization_script(bootstrap_config_script())
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the Mailwoman mobile shell");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_script_declares_native_mobile_config() {
        let s = bootstrap_config_script();
        assert!(s.contains("__MW_CONFIG__"));
        assert!(s.contains("\"native\":true"));
        assert!(s.contains("\"serverUrl\":null"));
    }
}
