//! Multi-server configuration (§2.1 "Server config": work + personal).
//!
//! Backs `getServerUrl` / `setServerUrl` / `listServers` / `selectServer` for the
//! mobile shell. Persisted as a small JSON file in the app config dir — this is
//! plain `std::fs` + `serde`, so it builds and is unit-tested on the desktop
//! **host** (no mobile toolchain needed). The server URL itself is non-secret
//! (the *session token* lives in the OS keychain via the keyring path — desktop
//! e1); this file only records which servers the user has added and which is
//! selected.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

/// A configured Mailwoman server (mirrors the frozen TS `ServerEntry`, §2.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerEntry {
    pub url: String,
    pub label: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct ServerConfig {
    #[serde(default)]
    servers: Vec<ServerEntry>,
    #[serde(default)]
    selected: Option<String>,
}

/// Absolute path of the servers config file inside the app config dir.
fn config_path<R: Runtime>(app: &AppHandle<R>) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("no app config dir: {e}"))?;
    Ok(dir.join("servers.json"))
}

fn load<R: Runtime>(app: &AppHandle<R>) -> Result<ServerConfig, String> {
    let path = config_path(app)?;
    match std::fs::read(&path) {
        Ok(bytes) => {
            serde_json::from_slice(&bytes).map_err(|e| format!("corrupt servers.json: {e}"))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(ServerConfig::default()),
        Err(e) => Err(format!("read servers.json: {e}")),
    }
}

fn store<R: Runtime>(app: &AppHandle<R>, cfg: &ServerConfig) -> Result<(), String> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create config dir: {e}"))?;
    }
    let bytes = serde_json::to_vec_pretty(cfg).map_err(|e| format!("serialize servers: {e}"))?;
    std::fs::write(&path, bytes).map_err(|e| format!("write servers.json: {e}"))
}

/// List all configured servers (§2.1 `listServers`). Frozen `tauri.ts` name.
#[tauri::command]
pub fn mw_server_list<R: Runtime>(app: AppHandle<R>) -> Result<Vec<ServerEntry>, String> {
    Ok(load(&app)?.servers)
}

/// Add (or update the label of) a server, then select it (§2.1 `setServerUrl`).
#[tauri::command]
pub fn mw_server_add<R: Runtime>(
    app: AppHandle<R>,
    url: String,
    label: String,
) -> Result<Vec<ServerEntry>, String> {
    let mut cfg = load(&app)?;
    match cfg.servers.iter_mut().find(|s| s.url == url) {
        Some(existing) => existing.label = label,
        None => cfg.servers.push(ServerEntry {
            url: url.clone(),
            label,
        }),
    }
    cfg.selected = Some(url);
    store(&app, &cfg)?;
    Ok(cfg.servers)
}

/// Remove a server; clears the selection if it was the selected one.
#[tauri::command]
pub fn mw_server_remove<R: Runtime>(
    app: AppHandle<R>,
    url: String,
) -> Result<Vec<ServerEntry>, String> {
    let mut cfg = load(&app)?;
    cfg.servers.retain(|s| s.url != url);
    if cfg.selected.as_deref() == Some(url.as_str()) {
        cfg.selected = cfg.servers.first().map(|s| s.url.clone());
    }
    store(&app, &cfg)?;
    Ok(cfg.servers)
}

/// Select an already-configured server (§2.1 `selectServer`).
#[tauri::command]
pub fn mw_server_select<R: Runtime>(app: AppHandle<R>, url: String) -> Result<(), String> {
    let mut cfg = load(&app)?;
    if !cfg.servers.iter().any(|s| s.url == url) {
        return Err(format!("unknown server: {url}"));
    }
    cfg.selected = Some(url);
    store(&app, &cfg)
}

/// The currently selected server URL, if any (§2.1 `getServerUrl`).
#[tauri::command]
pub fn mw_server_get_selected<R: Runtime>(app: AppHandle<R>) -> Result<Option<String>, String> {
    Ok(load(&app)?.selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selecting_removed_server_falls_back_to_first() {
        let mut cfg = ServerConfig {
            servers: vec![
                ServerEntry {
                    url: "https://a".into(),
                    label: "Work".into(),
                },
                ServerEntry {
                    url: "https://b".into(),
                    label: "Personal".into(),
                },
            ],
            selected: Some("https://a".into()),
        };
        // Emulate config_remove_server's selection fixup for "https://a".
        cfg.servers.retain(|s| s.url != "https://a");
        if cfg.selected.as_deref() == Some("https://a") {
            cfg.selected = cfg.servers.first().map(|s| s.url.clone());
        }
        assert_eq!(cfg.selected.as_deref(), Some("https://b"));
        assert_eq!(cfg.servers.len(), 1);
    }

    #[test]
    fn config_round_trips_through_json() {
        let cfg = ServerConfig {
            servers: vec![ServerEntry {
                url: "https://mail.example".into(),
                label: "Work".into(),
            }],
            selected: Some("https://mail.example".into()),
        };
        let bytes = serde_json::to_vec(&cfg).unwrap();
        let back: ServerConfig = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(back.servers, cfg.servers);
        assert_eq!(back.selected, cfg.selected);
    }
}
