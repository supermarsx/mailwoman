//! Deep-link / `mailto:` for the mobile shell (§2.1 "Deep links / mailto:"; frozen
//! `tauri.ts` name `mw_register_mailto_handler` + the `mw://open-url` bridge).
//!
//! Mirrors the desktop `deeplink` module's frozen surface. On Android the app's URL
//! associations (`mailto:`, the app scheme, and the `.eml/.ics/.vcf/.msg` file
//! handlers) are declared **statically in the manifest** (`manifest-intents.xml`),
//! so there is no runtime scheme-registration call to make: `mw_register_mailto_handler`
//! is an idempotent no-op success (the association already exists once the manifest
//! ships). When the OS hands the app a `mailto:`/app-scheme URL, `lib.rs`'s
//! `tauri-plugin-deep-link` `on_open_url` callback forwards it through
//! [`emit_open_url`] as the `mw://open-url` event that `platform/tauri.ts`'s
//! `onOpenUrl` listener turns into a composer/navigation.

use tauri::{Emitter, Runtime};

/// Frontend event name the capability layer listens on for opened URLs.
pub const OPEN_URL_EVENT: &str = "mw://open-url";

/// The URL schemes the mobile app claims (matching the desktop shell). `mailto`
/// makes it a mail-handler; `mailwoman` is the app's own deep-link scheme.
pub const HANDLED_SCHEMES: [&str; 2] = ["mailto", "mailwoman"];

/// True when `url` is one the SPA should act on (a `mailto:`/`mailwoman:` link).
pub fn is_handled_url(url: &str) -> bool {
    HANDLED_SCHEMES.iter().any(|scheme| {
        url.len() > scheme.len()
            && url[..scheme.len()].eq_ignore_ascii_case(scheme)
            && url.as_bytes()[scheme.len()] == b':'
    })
}

/// Emit an opened URL to the frontend — the single choke point for the
/// `mw://open-url` event, matching the desktop shell.
pub fn emit_open_url<R: Runtime>(app: &tauri::AppHandle<R>, url: &str) -> Result<(), String> {
    app.emit(OPEN_URL_EVENT, url)
        .map_err(|e| format!("emit {OPEN_URL_EVENT}: {e}"))
}

/// Register the app as the handler for the claimed schemes (§2.1
/// `registerMailtoHandler`). On Android the associations are declared in the
/// manifest at install time, so this is a no-op success — there is no per-user
/// runtime registration API to call.
#[tauri::command]
pub async fn mw_register_mailto_handler<R: Runtime>(
    _app: tauri::AppHandle<R>,
) -> Result<(), String> {
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
    }

    #[test]
    fn rejects_unrelated_and_malformed_urls() {
        assert!(!is_handled_url("https://example.com"));
        assert!(!is_handled_url("mailto")); // no colon
        assert!(!is_handled_url("mailtox:foo")); // scheme is a prefix but not delimited
        assert!(!is_handled_url(""));
    }
}
