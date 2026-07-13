//! Self-contained mode — mobile honest-degrade (§4.1 / §2.1; frozen `tauri.ts` names
//! `mw_self_contained_status` / `mw_start_local_server` / `mw_stop_local_server`).
//!
//! Self-contained mode (spawning a bundled `mw-server` sibling process) is a DESKTOP
//! capability (SPEC §4.1 / plan §3 e3): a laptop user running serverless. On mobile
//! there is no bundled server — the phone always points at a remote (or self-hosted)
//! Mailwoman server. These commands exist so the frozen `platform/tauri.ts` invokes
//! resolve natively on mobile, and they degrade honestly: status is always `"off"`
//! and start/stop are no-ops. (`platform/index.ts` already reports `"off"` for
//! mobile; this keeps the native command surface symmetric with desktop so a bare
//! invoke never hits an unregistered command.)

/// Always `"off"` on mobile — there is no bundled local server to manage.
#[tauri::command]
pub async fn mw_self_contained_status() -> Result<String, String> {
    Ok("off".to_string())
}

/// No-op on mobile: mobile never spawns a local server. Returns `null` (no loopback
/// URL) so the caller keeps pointing at the configured remote server.
#[tauri::command]
pub async fn mw_start_local_server() -> Result<Option<String>, String> {
    Ok(None)
}

/// No-op on mobile.
#[tauri::command]
pub async fn mw_stop_local_server() -> Result<(), String> {
    Ok(())
}
