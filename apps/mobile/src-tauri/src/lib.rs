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
//!     handlers, badge, biometric),
//!   * t32: the server address is baked in at build time (`commands::config`),
//!   * e4 adds the Android `FLAG_SECURE` Kotlin plugin,
//!   * e7 mounts the bundle-hash gate + produces the APK.

use serde_json::json;
use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_deep_link::DeepLinkExt;

// Mobile capability commands (UnifiedPush, share targets, badge — plan §3 e2). Packaged as a self-contained Tauri plugin (`commands::init()`);
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

/// The product name: `MW_SHELL_PRODUCT_NAME` at build time (`scripts/build-shells.*`,
/// branding), else "Mailwoman". The window title.
const PRODUCT_NAME: &str = match option_env!("MW_SHELL_PRODUCT_NAME") {
    Some(name) => name,
    None => "Mailwoman",
};

/// The frozen JS handshake (§2.1/§2.5): injected before the SPA runs so the
/// capability layer knows it is native, which mobile OS it is on and which server
/// it talks to (`serverUrl`: the baked address, or `null` and the SPA says the app
/// has not been set up, t32). `platform.updater` is always false: Android and iOS
/// apps are updated by their store, not by the app (t32-cfg §4.18). Mirrors the
/// desktop shell's `bootstrap_config_script` (kept camelCase per the
/// `__MW_CONFIG__` precedent).
fn bootstrap_config_script(server_url: Option<&str>) -> String {
    let cfg = json!({
        "platform": {
            "kind": platform_kind(),
            "os": std::env::consts::OS,
            "version": env!("CARGO_PKG_VERSION"),
            "updater": false,
        },
        "serverUrl": server_url,
        "native": true,
        // Enable the passive native consumers (notifications/badge/share/push) in
        // the SPA (`platform/capabilities.ts`); `isTauri()` already enables them in
        // a shell — explicit here to keep the contract symmetric with desktop.
        "capabilities": true,
    });
    format!("globalThis.__MW_CONFIG__ = Object.assign(globalThis.__MW_CONFIG__ || {{}}, {cfg});")
}

/// Remove the service workers stored in the Android WebView profile before the
/// first webview is created (t31-e7 finding, same as the desktop shell). wry's
/// Android client serves the bundle from `WebViewClient.shouldInterceptRequest`,
/// which Android does not call for a service worker's requests, so a stored
/// worker's fetch of `http://tauri.localhost/` leaves the WebView for the network
/// and its answer runs with the shell's IPC. The SPA no longer registers a worker
/// in a shell (`apps/web/src/sw/register.ts`); this removes one an earlier build
/// registered. `data_dir` is the app's data folder (Tauri's `app_data_dir` on
/// Android, `Context.getDataDir()`); the WebView keeps its profile in
/// `app_webview/Default` there, or in `app_webview` itself on WebView versions
/// without profiles. Nothing else in the profile is touched. Returns how many
/// folders were removed.
pub fn remove_stored_service_workers(data_dir: &std::path::Path) -> Result<usize, String> {
    let webview = data_dir.join("app_webview");
    let mut removed = 0;
    for dir in [
        webview.join("Default").join("Service Worker"),
        webview.join("Service Worker"),
    ] {
        match std::fs::remove_dir_all(&dir) {
            Ok(()) => removed += 1,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("{}: {e}", dir.display())),
        }
    }
    Ok(removed)
}

/// Build and run the mobile shell.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        // e2's mobile native-bridge plugin: a SETUP-ONLY plugin that registers the
        // Android/iOS `MobileBridge` + e4's `CaptureBridge` (FLAG_SECURE) handles.
        // The capability COMMANDS are registered at the app level below with their
        // frozen bare `mw_*` names (t7-e8 command reconciliation).
        .plugin(commands::init())
        // App-level command surface: every name the frozen `platform/tauri.ts`
        // invokes has a mobile registration here (grep-verified against desktop's
        // `lib.rs`), so the SPA's bare `invoke('mw_*')` resolves natively on Android.
        // Commands `#[cfg(mobile)]`-degrade on the desktop host build.
        .invoke_handler(tauri::generate_handler![
            // OS-backed secure store (Android EncryptedSharedPreferences).
            commands::keychain::mw_keychain_get,
            commands::keychain::mw_keychain_set,
            commands::keychain::mw_keychain_delete,
            // Native notifications + badge.
            commands::notifications::mw_notify,
            commands::badge::mw_set_badge_count,
            // Deep-link / default-mailto (manifest-declared on Android).
            commands::deeplink::mw_register_mailto_handler,
            // Biometric app-lock (tauri-plugin-biometric on device).
            commands::biometric::mw_biometric_available,
            commands::biometric::mw_biometric_authenticate,
            // Drag-out (no-op on mobile).
            commands::dragout::mw_dragout_materialize,
            // Screen-capture protection (Android FLAG_SECURE via e4's plugin).
            commands::capture::mw_set_capture_protection,
            // Push (Android UnifiedPush).
            commands::push::mw_push_subscribe,
            commands::push::mw_push_unsubscribe,
            commands::push::mw_push_get_distributor,
            // Share-target pull (companion to the `mw://share-target` event).
            commands::share::mw_share_take_pending,
            // Self-contained lifecycle (desktop-only capability; honest-degrade here).
            commands::selfcontained::mw_self_contained_status,
            commands::selfcontained::mw_start_local_server,
            commands::selfcontained::mw_stop_local_server,
        ]);
    // `tauri-plugin-biometric` is `#![cfg(mobile)]` — it does not exist on the
    // desktop host build (used for `cargo build --workspace` / unit tests), so its
    // registration is gated to the mobile targets.
    #[cfg(mobile)]
    let builder = builder.plugin(tauri_plugin_biometric::init());
    builder
        .setup(|app| {
            // Before the first webview exists: no stored service worker may answer
            // the start-up navigation (t31-e7). Not proven on a device.
            #[cfg(target_os = "android")]
            {
                use tauri::Manager;
                match app.path().app_data_dir() {
                    Ok(dir) => match remove_stored_service_workers(&dir) {
                        Ok(0) => {}
                        Ok(n) => eprintln!("[mailwoman] removed stored service workers ({n})"),
                        Err(e) => eprintln!("[mailwoman] service workers not removed: {e}"),
                    },
                    Err(e) => eprintln!("[mailwoman] service workers not removed: {e}"),
                }
            }
            // Deep-link bridge: forward OS-delivered mailto:/mailwoman: URLs to the
            // SPA's `onOpenUrl` via `mw://open-url` (matching desktop's e7 bridge).
            // Filtered so only handled schemes reach the frontend.
            let handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    let s = url.as_str();
                    if commands::deeplink::is_handled_url(s) {
                        let _ = commands::deeplink::emit_open_url(&handle, s);
                    }
                }
            });

            let server = commands::config::server_url();
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title(PRODUCT_NAME)
                .initialization_script(bootstrap_config_script(server.as_deref()))
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
    fn removes_only_the_stored_service_workers() {
        let root = std::env::temp_dir().join(format!("mw-mobile-sw-{}", std::process::id()));
        let webview = root.join("app_webview");
        std::fs::create_dir_all(
            webview
                .join("Default")
                .join("Service Worker")
                .join("Database"),
        )
        .unwrap();
        std::fs::create_dir_all(webview.join("Service Worker")).unwrap();
        std::fs::create_dir_all(webview.join("Default").join("Local Storage")).unwrap();
        std::fs::write(
            webview.join("Default").join("Local Storage").join("keep"),
            b"x",
        )
        .unwrap();

        assert_eq!(remove_stored_service_workers(&root), Ok(2));
        assert!(!webview.join("Default").join("Service Worker").exists());
        assert!(!webview.join("Service Worker").exists());
        assert!(
            webview
                .join("Default")
                .join("Local Storage")
                .join("keep")
                .exists()
        );
        assert_eq!(remove_stored_service_workers(&root), Ok(0));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn bootstrap_script_declares_native_mobile_config() {
        let s = bootstrap_config_script(None);
        assert!(s.contains("__MW_CONFIG__"));
        assert!(s.contains("\"native\":true"));
        assert!(s.contains("\"serverUrl\":null"));
        assert!(s.contains("\"updater\":false"));
        let s = bootstrap_config_script(Some("https://mail.example.org"));
        assert!(s.contains("\"serverUrl\":\"https://mail.example.org\""));
    }
}
