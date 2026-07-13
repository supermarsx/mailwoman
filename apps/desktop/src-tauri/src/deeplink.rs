//! Deep-link / `mailto:` commands (plan §2.1 "Deep links / mailto:"; §3 e1).
//!
//! Backs the capability-layer `registerMailtoHandler()` and the `onOpenUrl` bridge.
//! The shell registers as the OS default `mailto:` handler (and the `mailwoman:`
//! scheme) via `tauri-plugin-deep-link`; when the OS hands the app a URL (a
//! `mailto:` link clicked elsewhere, or a `mailwoman:` deep link), the shell emits
//! the frontend event `mw://open-url` carrying the raw URL string, which
//! `platform/tauri.ts`'s `onOpenUrl` listener turns into a composer/navigation.
//!
//! Commands registered by e7 (`tauri::generate_handler!`):
//!   * `mw_register_mailto_handler(app)` -> Result<(), String>
//!
//! URL delivery: `tauri-plugin-deep-link`'s `on_open_url` callback fires with the
//! URLs; e7 wires it (in the Tauri `setup`) to call [`emit_open_url`] per URL, the
//! single choke point for the `mw://open-url` event shape. Schemes handled are
//! pinned by [`HANDLED_SCHEMES`] and unit-tested here.

use tauri::{Emitter, Runtime};
use tauri_plugin_deep_link::DeepLinkExt;

/// Frontend event name the capability layer listens on for opened URLs.
pub const OPEN_URL_EVENT: &str = "mw://open-url";

/// The URL schemes the shell claims. `mailto` makes it the default mail client;
/// `mailwoman` is the app's own deep-link scheme (e.g. `mailwoman://thread/7`).
pub const HANDLED_SCHEMES: [&str; 2] = ["mailto", "mailwoman"];

/// True when `url` is one the SPA should act on (a `mailto:`/`mailwoman:` link).
/// Used to filter the OS-delivered URLs before forwarding, so unrelated schemes
/// never reach the frontend.
pub fn is_handled_url(url: &str) -> bool {
    HANDLED_SCHEMES.iter().any(|scheme| {
        url.len() > scheme.len()
            && url[..scheme.len()].eq_ignore_ascii_case(scheme)
            && url.as_bytes()[scheme.len()] == b':'
    })
}

/// Emit an opened URL to the frontend. The single choke point for the
/// `mw://open-url` event: e7's `on_open_url` callback calls this for each handled
/// URL, and the capability layer's `onOpenUrl` receives the raw string.
pub fn emit_open_url<R: Runtime>(app: &tauri::AppHandle<R>, url: &str) -> Result<(), String> {
    app.emit(OPEN_URL_EVENT, url)
        .map_err(|e| format!("emit {OPEN_URL_EVENT}: {e}"))
}

/// Register the shell as the OS handler for the [`HANDLED_SCHEMES`]. On
/// Windows/Linux this is a runtime registration (writes the per-user URL-scheme
/// association); on macOS the association is declared in `Info.plist` at bundle
/// time, so `register` is a best-effort no-op there. Returns the first registration
/// error, if any.
#[tauri::command]
pub async fn mw_register_mailto_handler<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<(), String> {
    for scheme in HANDLED_SCHEMES {
        app.deep_link()
            .register(scheme)
            .map_err(|e| format!("register deep-link scheme {scheme}: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_handled_schemes_case_insensitively() {
        assert!(is_handled_url("mailto:ada@example.com"));
        assert!(is_handled_url("MAILTO:ada@example.com"));
        assert!(is_handled_url("mailwoman://thread/7"));
        assert!(is_handled_url("mailwoman:compose"));
    }

    #[test]
    fn rejects_unrelated_and_malformed_urls() {
        assert!(!is_handled_url("https://example.com"));
        assert!(!is_handled_url("file:///etc/passwd"));
        assert!(!is_handled_url("mailto")); // no colon
        assert!(!is_handled_url("mailtox:foo")); // scheme is a prefix but not delimited
        assert!(!is_handled_url(""));
    }
}
