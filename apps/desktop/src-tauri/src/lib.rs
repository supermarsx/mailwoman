//! Mailwoman desktop shell (Tauri v2 · plan §3 e7 — MOUNT/WIRE).
//!
//! THIN client (SPEC §16 / plan §0): this crate embeds the hash-verified
//! `apps/web/dist` SPA (the byte-identical bundle `mw-server` serves via
//! `rust-embed`) and adds OS integration only — **no protocol logic and no forked
//! UI**. The shell loads the same SPA a browser would, verifies the bundle's
//! integrity (§7.4), injects the frozen `globalThis.__MW_CONFIG__` handshake, and
//! points the SPA at the user's Mailwoman server (or the self-contained loopback
//! `mw-server`) over the identical JMAP surface. The native capability layer
//! (`apps/web/src/platform/tauri.ts`) reaches the OS through the `mw_*` commands
//! registered below.
//!
//! e7 (this file) consolidates the Batch-B modules into a running app:
//!   * e1 — notifications / keychain / deep-link / biometric / badge / drag-out /
//!     multi-server config (`badge`, `biometric`, `deeplink`, `dragout`, `keychain`,
//!     `notifications`, `serverconfig`),
//!   * e3 — self-contained local-`mw-server` spawn (`selfcontained`),
//!   * e4 — screen-capture protection (`capture`),
//!   * e7 — desktop push honest-degrade (`push`) + the bundle-hash gate + the
//!     `invoke_handler` registration + the `setup` event bridges + the `__MW_CONFIG__`
//!     injection + the quit-time server shutdown.

pub mod badge;
pub mod biometric;
pub mod capture;
pub mod deeplink;
pub mod dragout;
pub mod keychain;
pub mod notifications;
pub mod push;
pub mod selfcontained;
pub mod serverconfig;

use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use tauri::{Manager, RunEvent, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_deep_link::DeepLinkExt;

/// The frozen JS handshake (§2.1/§2.5): the shell injects `globalThis.__MW_CONFIG__`
/// before the SPA executes so the capability layer (`platform/index.ts`) knows it is
/// running natively, where its server is, and that the native capability consumers
/// are active. `serverUrl` is `null` at boot (the SPA's multi-server picker / e6's
/// `mw_server_*` sets it; self-contained mode / e3 points it at the loopback
/// `mw-server`). camelCase matches the existing `__MW_CONFIG__` precedent
/// (`viewers/max-security.ts`).
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
        "native": true,
        // Turn on the passive native consumers (notifications/badge/deep-link/push)
        // in the SPA (`platform/capabilities.ts`). In a Tauri shell `isTauri()`
        // already enables them; this is belt-and-suspenders so the contract is
        // explicit and a non-Tauri embedding of the same bundle stays byte-identical.
        "capabilities": true,
    });
    // Assign early so import-time reads (e.g. `readAdminFloor()`) see it.
    format!("globalThis.__MW_CONFIG__ = Object.assign(globalThis.__MW_CONFIG__ || {{}}, {cfg});")
}

/// The build-emitted UI-bundle manifest (`scripts/emit-bundle-hash.mjs`, §7.4 /
/// §2.2). Compiled into the binary via `include_str!` so it cannot be swapped after
/// build; `files` maps each dist-relative path to its SHA-256.
#[derive(Debug, Deserialize)]
struct BundleManifest {
    #[serde(rename = "bundleHash")]
    bundle_hash: String,
    #[serde(rename = "fileCount")]
    file_count: usize,
    files: std::collections::BTreeMap<String, String>,
}

/// The manifest for THIS build's embedded SPA. `include_str!` resolves at compile
/// time against `apps/desktop/src-tauri/bundle-hash.json` — the same file
/// `emit-bundle-hash.mjs` writes from the dist that `frontendDist` embeds, so a
/// correctly-sequenced build (web build → emit-bundle-hash → tauri build) compiles
/// the manifest and the assets from one consistent dist.
const BUNDLE_MANIFEST_JSON: &str = include_str!("../bundle-hash.json");

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let digest = h.finalize();
    let mut s = String::with_capacity(digest.len() * 2);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Verify every file the manifest records against the bytes the shell will actually
/// load (`fetch(rel)` returns the embedded asset's ORIGINAL bytes — the caller passes
/// the Tauri asset resolver, which brotli-decompresses so the hash matches the dist
/// file `emit-bundle-hash.mjs` hashed). Per-file comparison is order-independent, so
/// it is robust to the emitter's OS-specific sort order while still catching any
/// tampered or missing SPA asset (§7.4 / risk #9). Returns the count verified, or the
/// first offending path.
fn verify_bundle_files<F>(manifest: &BundleManifest, fetch: F) -> Result<usize, String>
where
    F: Fn(&str) -> Option<Vec<u8>>,
{
    let mut checked = 0usize;
    for (rel, expected_hash) in &manifest.files {
        let bytes =
            fetch(rel).ok_or_else(|| format!("UI-bundle asset missing from shell: {rel}"))?;
        let got = sha256_hex(&bytes);
        if &got != expected_hash {
            return Err(format!(
                "UI-bundle asset hash mismatch for {rel} (expected {}…, got {}…)",
                &expected_hash[..16.min(expected_hash.len())],
                &got[..16.min(got.len())],
            ));
        }
        checked += 1;
    }
    Ok(checked)
}

/// UI-bundle integrity gate (§7.4 / §2.2 / risk #9). Reads the compiled-in manifest
/// and re-hashes every embedded SPA asset via the Tauri asset resolver; a mismatch or
/// a missing asset **aborts launch** before the shell points at any server (tamper
/// gate). Returns `Ok(())` to proceed, or `Err(reason)` to block.
fn verify_bundle_integrity<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<(), String> {
    let manifest: BundleManifest = serde_json::from_str(BUNDLE_MANIFEST_JSON)
        .map_err(|e| format!("bundle-hash.json is unreadable ({e}) — build is inconsistent"))?;

    let resolver = app.asset_resolver();
    let fetch = |rel: &str| -> Option<Vec<u8>> {
        // The resolver keys assets with a leading slash; fall back to the bare path.
        resolver
            .get(format!("/{rel}"))
            .or_else(|| resolver.get(rel.to_string()))
            .map(|a| a.bytes)
    };

    let checked = verify_bundle_files(&manifest, fetch)?;
    eprintln!(
        "[mailwoman] UI-bundle integrity OK: {checked}/{} files match (bundleHash {}…)",
        manifest.file_count,
        &manifest.bundle_hash[..16.min(manifest.bundle_hash.len())]
    );
    Ok(())
}

/// Build and run the desktop shell.
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        // Self-contained mode's child-process manager (e3), reachable from the
        // `mw_*_local_server` commands and the quit handler.
        .manage(selfcontained::LocalServer::new())
        .invoke_handler(tauri::generate_handler![
            // e1 — OS keychain (session bearer + secure store).
            keychain::mw_keychain_set,
            keychain::mw_keychain_get,
            keychain::mw_keychain_delete,
            // e1 — native notifications + badge.
            notifications::mw_notify,
            badge::mw_set_badge_count,
            // e1 — deep-link / default-mailto.
            deeplink::mw_register_mailto_handler,
            // e1 — biometric app-lock (Windows Hello / honest fallback).
            biometric::mw_biometric_available,
            biometric::mw_biometric_authenticate,
            // e1 — drag-out attachments.
            dragout::mw_dragout_materialize,
            // e1 — multi-server config.
            serverconfig::mw_server_list,
            serverconfig::mw_server_get_selected,
            serverconfig::mw_server_add,
            serverconfig::mw_server_remove,
            serverconfig::mw_server_select,
            // e4 — screen-capture protection.
            capture::mw_set_capture_protection,
            // e3 — self-contained local mw-server lifecycle.
            selfcontained::mw_self_contained_status,
            selfcontained::mw_start_local_server,
            selfcontained::mw_stop_local_server,
            // e7 — desktop push (honest degrade; Web Push subscribes in the WebView).
            push::mw_push_subscribe,
            push::mw_push_unsubscribe,
        ])
        .setup(|app| {
            // §7.4 tamper gate BEFORE any window loads the SPA or points at a server.
            if let Err(reason) = verify_bundle_integrity(app.handle()) {
                eprintln!("[mailwoman] FATAL: UI-bundle integrity check failed: {reason}");
                return Err(reason.into());
            }

            // Deep-link bridge (e1): forward OS-delivered mailto:/mailwoman: URLs to
            // the SPA's `onOpenUrl` via the `mw://open-url` event. Filtered so only
            // handled schemes reach the frontend.
            let handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    let s = url.as_str();
                    if deeplink::is_handled_url(s) {
                        let _ = deeplink::emit_open_url(&handle, s);
                    }
                }
            });

            // The main window is created in code (not tauri.conf.json) so the
            // `__MW_CONFIG__` handshake runs as an initialization script BEFORE the
            // SPA loads. `content_protected(false)` is the default; e4's
            // `mw_set_capture_protection` flips it at runtime.
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Mailwoman")
                .inner_size(1200.0, 800.0)
                .min_inner_size(720.0, 560.0)
                .content_protected(false)
                .initialization_script(bootstrap_config_script())
                .build()?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building the Mailwoman desktop shell");

    app.run(|app_handle, event| {
        // On quit, kill any self-contained `mw-server` sibling so it never orphans.
        if let RunEvent::ExitRequested { .. } = event {
            let _ = app_handle.state::<selfcontained::LocalServer>().stop();
        }
    });
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
        assert!(s.contains("\"capabilities\":true"));
        // serverUrl is null at boot (the server picker / e3 / e6 sets it).
        assert!(s.contains("\"serverUrl\":null"));
    }

    #[test]
    fn compiled_in_manifest_parses_and_is_non_empty() {
        // The gate depends on `bundle-hash.json` deserializing; pin that so a broken
        // emitter output fails a unit test rather than every launch.
        let m: BundleManifest = serde_json::from_str(BUNDLE_MANIFEST_JSON).unwrap();
        assert_eq!(m.files.len(), m.file_count);
        assert!(!m.bundle_hash.is_empty());
        assert!(!m.files.is_empty(), "manifest lists no files");
    }

    #[test]
    fn verify_passes_when_every_asset_matches() {
        let m: BundleManifest = serde_json::from_str(BUNDLE_MANIFEST_JSON).unwrap();
        // Fake the resolver with a map that returns bytes hashing to each expected
        // value — proves the happy path without a live app/WebView.
        let synthetic: std::collections::BTreeMap<String, Vec<u8>> = m
            .files
            .keys()
            .map(|rel| (rel.clone(), rel.clone().into_bytes()))
            .collect();
        // Rebuild an expected-hash manifest over the synthetic bytes.
        let files = synthetic
            .iter()
            .map(|(rel, bytes)| (rel.clone(), sha256_hex(bytes)))
            .collect();
        let fake = BundleManifest {
            bundle_hash: m.bundle_hash.clone(),
            file_count: synthetic.len(),
            files,
        };
        let checked =
            verify_bundle_files(&fake, |rel| synthetic.get(rel).cloned()).expect("all match");
        assert_eq!(checked, synthetic.len());
    }

    #[test]
    fn verify_fails_on_missing_asset() {
        let manifest = BundleManifest {
            bundle_hash: "x".into(),
            file_count: 1,
            files: [("index.html".to_string(), sha256_hex(b"hello"))]
                .into_iter()
                .collect(),
        };
        let err = verify_bundle_files(&manifest, |_| None).unwrap_err();
        assert!(err.contains("missing"), "got: {err}");
    }

    #[test]
    fn verify_fails_on_tampered_asset() {
        let manifest = BundleManifest {
            bundle_hash: "x".into(),
            file_count: 1,
            files: [("app.js".to_string(), sha256_hex(b"original"))]
                .into_iter()
                .collect(),
        };
        // The shell returns DIFFERENT bytes → hash mismatch → blocked.
        let err = verify_bundle_files(&manifest, |_| Some(b"tampered".to_vec())).unwrap_err();
        assert!(err.contains("mismatch"), "got: {err}");
    }
}
