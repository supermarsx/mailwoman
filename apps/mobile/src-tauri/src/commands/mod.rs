//! Mailwoman mobile capability commands (Tauri v2 · plan t7 §3 e2).
//!
//! These are the Android capability commands backing the frozen `Platform`
//! interface (§2.1): **UnifiedPush** registration (`push`), **share targets +
//! file handlers** (`share`), **badge counts** (`badge`) and **multi-server
//! config** (`config`). They are packaged as a self-contained **Tauri plugin**
//! (`init()` below) so the shared shell entry point (`lib.rs`, owned by e0/e7)
//! only has to `.plugin(commands::init())` — it never has to edit an
//! `invoke_handler`, keeping this executor's surface disjoint from e4
//! (FLAG_SECURE) and e7 (mount/wire).
//!
//! **Toolchain reality (plan §1.11 / e0 probe):** this Windows machine has the
//! Android SDK + NDK but **no JDK on PATH**, so the Android target cannot be
//! cross-compiled here. Everything in this module is written to `cargo check`
//! on the **desktop host** (where the native plugin is absent → commands
//! degrade to "unsupported"); the real Android/iOS bridge is exercised in CI
//! (e8's `android-apk` job). The `#[cfg(mobile)]` / `#[cfg(target_os=...)]`
//! gates below are the exact seam between host-checkable Rust and the
//! Kotlin/Swift that only builds under the mobile toolchain.

pub mod badge;
pub mod config;
pub mod push;
pub mod share;

use tauri::Runtime;
use tauri::plugin::{Builder, TauriPlugin};

#[cfg(mobile)]
use tauri::Manager;
#[cfg(mobile)]
use tauri::plugin::PluginHandle;

/// Java package of the custom Android plugin — must match the `@TauriPlugin`
/// class in `android-src/MailwomanMobilePlugin.kt`.
#[cfg(target_os = "android")]
const ANDROID_PLUGIN_IDENTIFIER: &str = "com.mailwoman.mobile";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_mailwoman_mobile);

/// Bridge to the native mobile plugin (Android Kotlin / iOS Swift), managed as
/// Tauri state by [`init`]'s `setup`. Exists only on the mobile targets — on the
/// desktop host build there is no native plugin, and every command takes the
/// "unsupported on this platform" branch without ever touching this state.
#[cfg(mobile)]
pub struct MobileBridge<R: Runtime> {
    handle: PluginHandle<R>,
}

#[cfg(mobile)]
impl<R: Runtime> MobileBridge<R> {
    /// Invoke a command on the native mobile plugin and deserialize its reply.
    pub(crate) fn run<T, P>(&self, name: &str, payload: P) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
        P: serde::Serialize,
    {
        self.handle
            .run_mobile_plugin(name, payload)
            .map_err(|e| e.to_string())
    }
}

/// Build the Mailwoman mobile capability plugin.
///
/// e7 registers this in the shell entry point with `.plugin(commands::init())`.
/// The plugin owns its own `invoke_handler`, so no shared registration is edited
/// by this executor (per the e2 task boundary).
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("mailwoman-mobile")
        .invoke_handler(tauri::generate_handler![
            config::config_list_servers,
            config::config_add_server,
            config::config_remove_server,
            config::config_select_server,
            config::config_get_selected,
            push::push_get_distributor,
            push::push_register,
            push::push_unregister,
            share::share_take_pending,
            badge::set_badge_count,
        ])
        .setup(|app, _api| {
            #[cfg(target_os = "android")]
            {
                let handle = _api
                    .register_android_plugin(ANDROID_PLUGIN_IDENTIFIER, "MailwomanMobilePlugin")?;
                app.manage(MobileBridge { handle });
            }
            #[cfg(target_os = "ios")]
            {
                // Best-effort iOS: requires the Swift plugin, which cannot be
                // built on this Windows machine (documented gap, plan §1.9).
                let handle = _api.register_ios_plugin(init_plugin_mailwoman_mobile)?;
                app.manage(MobileBridge { handle });
            }
            // Desktop host: no native plugin to register; commands degrade.
            let _ = app;
            Ok(())
        })
        .build()
}
