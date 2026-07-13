//! Mailwoman mobile capability commands (Tauri v2 · plan t7 §3 e2 + e8 reconciliation).
//!
//! These are the Android capability commands backing the frozen `Platform`
//! interface (§2.1). Every command uses the frozen **bare `mw_*` name** the SPA's
//! `platform/tauri.ts` invokes (multi-server `config` → `mw_server_*`, `push` →
//! `mw_push_*`, `badge` → `mw_set_badge_count`, plus the mobile bindings
//! `mw_keychain_*`, `mw_notify`, `mw_biometric_*`, `mw_register_mailto_handler`,
//! `mw_dragout_materialize`, `mw_set_capture_protection`, `mw_self_contained_*`).
//!
//! **Registration model (t7-e8 command-name reconciliation):** the commands are
//! registered at the **app level** in `lib.rs`'s `invoke_handler`, NOT in a plugin
//! `invoke_handler` — a plugin handler would expose them only as
//! `plugin:mailwoman-mobile|…`, which the frozen bare-name contract does not use.
//! [`init`] below is therefore a **setup-only** plugin: it just registers the
//! native Android/iOS bridge handles ([`MobileBridge`], [`CaptureBridge`]) as Tauri
//! state. App-level commands are not ACL-gated, so no per-command capability grant
//! is required (this resolves the e7-flagged mobile-plugin-ACL gap).
//!
//! **Toolchain reality (plan §1.11 / e0 probe):** this Windows machine has the
//! Android SDK + NDK but **no JDK on PATH**, so the Android target cannot be
//! cross-compiled here. Everything in this module `cargo check`s on the **desktop
//! host** (where the native plugin is absent → commands `#[cfg(mobile)]`-degrade to
//! "unsupported"); the real Android/iOS bridge is exercised in CI (e8's
//! `android-apk` job). The `#[cfg(mobile)]` / `#[cfg(target_os=...)]` gates are the
//! exact seam between host-checkable Rust and the Kotlin/Swift that only builds
//! under the mobile toolchain.

pub mod badge;
pub mod biometric;
pub mod capture;
pub mod config;
pub mod deeplink;
pub mod dragout;
pub mod keychain;
pub mod notifications;
pub mod push;
pub mod selfcontained;
pub mod share;

use tauri::Runtime;
use tauri::plugin::{Builder, TauriPlugin};

#[cfg(mobile)]
use tauri::Manager;
#[cfg(mobile)]
use tauri::plugin::PluginHandle;

/// Java package of the custom Android plugins — must match the `@TauriPlugin`
/// classes in `android-src/MailwomanMobilePlugin.kt` and `FlagSecurePlugin.kt`.
#[cfg(target_os = "android")]
const ANDROID_PLUGIN_IDENTIFIER: &str = "com.mailwoman.mobile";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_mailwoman_mobile);

/// Bridge to the native mobile capability plugin (Android Kotlin `MailwomanMobilePlugin`
/// / iOS Swift), managed as Tauri state by [`init`]'s `setup`. Exists only on the
/// mobile targets — on the desktop host build there is no native plugin, and every
/// command takes the "unsupported on this platform" branch without touching this state.
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

/// Bridge to e4's separate Android FLAG_SECURE plugin (`FlagSecurePlugin.kt`),
/// registered as its own `@TauriPlugin`. Android-only — desktop host and iOS report
/// `{ supported: false }` for capture protection and never touch this state.
#[cfg(target_os = "android")]
pub struct CaptureBridge<R: Runtime> {
    handle: PluginHandle<R>,
}

#[cfg(target_os = "android")]
impl<R: Runtime> CaptureBridge<R> {
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

/// Build the Mailwoman mobile native-bridge plugin.
///
/// This is a **setup-only** plugin: its sole job is to register the native
/// Android/iOS plugin handles ([`MobileBridge`] for the capability plugin,
/// [`CaptureBridge`] for e4's FLAG_SECURE plugin) as managed Tauri state. The
/// capability COMMANDS themselves are registered at the APP level in `lib.rs`'s
/// `invoke_handler` with their frozen bare `mw_*` names, so the SPA's
/// `invoke('mw_server_list')` etc. resolve directly (a plugin `invoke_handler`
/// would only expose them under the `plugin:mailwoman-mobile|…` namespace, which
/// the frozen `platform/tauri.ts` does NOT use — this is the command-name
/// reconciliation, t7-e8). `run_mobile_plugin` (Rust → Kotlin) is internal and not
/// subject to the JS ACL, so no per-command capability grant is required.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("mailwoman-mobile")
        .setup(|app, _api| {
            #[cfg(target_os = "android")]
            {
                let handle = _api
                    .register_android_plugin(ANDROID_PLUGIN_IDENTIFIER, "MailwomanMobilePlugin")?;
                app.manage(MobileBridge { handle });
                // e4's FLAG_SECURE plugin is a separate @TauriPlugin class.
                let capture =
                    _api.register_android_plugin(ANDROID_PLUGIN_IDENTIFIER, "FlagSecurePlugin")?;
                app.manage(CaptureBridge { handle: capture });
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
