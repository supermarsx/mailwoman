//! Multi-server URL config storage (plan §2.1 "Server config"; §3 e1).
//!
//! Backs the capability-layer `getServerUrl`/`setServerUrl`/`listServers`/
//! `selectServer` — the "work + personal" server list a desktop user points the
//! thin shell at (the SPA then talks the identical JMAP surface to whichever is
//! selected, §2.2). The list is plain config (URLs + labels, not secrets), stored
//! as `servers.json` in the app config dir; the bearer *token* for each server
//! lives in the OS keychain ([`crate::keychain`]), never here.
//!
//! Commands registered by e7 (`tauri::generate_handler!`):
//!   * `mw_server_list(app)`               -> Result<Vec<ServerEntry>, String>
//!   * `mw_server_get_selected(app)`       -> Result<Option<String>, String>
//!   * `mw_server_add(app, url, label)`    -> Result<Vec<ServerEntry>, String>
//!   * `mw_server_remove(app, url)`        -> Result<Vec<ServerEntry>, String>
//!   * `mw_server_select(app, url)`        -> Result<(), String>
//!
//! The pure list operations live on [`ServerStore`] and are unit-tested against a
//! temp file; the commands are thin wrappers that resolve the config path.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{Manager, Runtime};

/// A configured Mailwoman server (mirrors `ServerEntry` in `platform/index.ts`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ServerEntry {
    pub url: String,
    pub label: String,
}

/// The persisted multi-server config: the known servers plus which one is active.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ServerStore {
    pub servers: Vec<ServerEntry>,
    /// The selected server's URL; `None` until the user picks one.
    #[serde(default)]
    pub selected: Option<String>,
}

impl ServerStore {
    /// Load the store from `path`, treating a missing file as an empty store (first
    /// run). A corrupt file surfaces as an error rather than silently wiping config.
    pub fn load(path: &Path) -> Result<Self, String> {
        match std::fs::read(path) {
            Ok(bytes) => {
                serde_json::from_slice(&bytes).map_err(|e| format!("parse {}: {e}", path.display()))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(format!("read {}: {e}", path.display())),
        }
    }

    /// Persist the store to `path`, creating the parent config dir if needed.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        let json = serde_json::to_vec_pretty(self).map_err(|e| format!("serialize: {e}"))?;
        std::fs::write(path, json).map_err(|e| format!("write {}: {e}", path.display()))
    }

    /// Add a server (or update the label of an existing URL) and select it. Adding
    /// the first server makes it the selection. URLs are the identity key.
    pub fn add(&mut self, url: String, label: String) {
        match self.servers.iter_mut().find(|s| s.url == url) {
            Some(existing) => existing.label = label,
            None => self.servers.push(ServerEntry {
                url: url.clone(),
                label,
            }),
        }
        self.selected = Some(url);
    }

    /// Remove the server with `url`. If it was selected, the selection falls back to
    /// the first remaining server (or `None` when the list is now empty).
    pub fn remove(&mut self, url: &str) {
        self.servers.retain(|s| s.url != url);
        if self.selected.as_deref() == Some(url) {
            self.selected = self.servers.first().map(|s| s.url.clone());
        }
    }

    /// Select `url` if it is a known server; selecting an unknown URL is rejected so
    /// the SPA never points at a server the user did not add.
    pub fn select(&mut self, url: &str) -> Result<(), String> {
        if self.servers.iter().any(|s| s.url == url) {
            self.selected = Some(url.to_string());
            Ok(())
        } else {
            Err(format!("unknown server: {url}"))
        }
    }
}

/// Resolve `servers.json` in the app config dir for this platform.
fn store_path<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("app_config_dir: {e}"))?;
    Ok(dir.join("servers.json"))
}

fn load<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(PathBuf, ServerStore), String> {
    let path = store_path(app)?;
    let store = ServerStore::load(&path)?;
    Ok((path, store))
}

/// List all configured servers.
#[tauri::command]
pub async fn mw_server_list<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<ServerEntry>, String> {
    Ok(load(&app)?.1.servers)
}

/// The currently selected server URL (or `null`).
#[tauri::command]
pub async fn mw_server_get_selected<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<String>, String> {
    Ok(load(&app)?.1.selected)
}

/// Add (or relabel) a server and select it; returns the updated list.
#[tauri::command]
pub async fn mw_server_add<R: Runtime>(
    app: tauri::AppHandle<R>,
    url: String,
    label: String,
) -> Result<Vec<ServerEntry>, String> {
    let (path, mut store) = load(&app)?;
    store.add(url, label);
    store.save(&path)?;
    Ok(store.servers)
}

/// Remove a server; returns the updated list.
#[tauri::command]
pub async fn mw_server_remove<R: Runtime>(
    app: tauri::AppHandle<R>,
    url: String,
) -> Result<Vec<ServerEntry>, String> {
    let (path, mut store) = load(&app)?;
    store.remove(&url);
    store.save(&path)?;
    Ok(store.servers)
}

/// Select a known server as the active one.
#[tauri::command]
pub async fn mw_server_select<R: Runtime>(
    app: tauri::AppHandle<R>,
    url: String,
) -> Result<(), String> {
    let (path, mut store) = load(&app)?;
    store.select(&url)?;
    store.save(&path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_first_server_selects_it() {
        let mut s = ServerStore::default();
        s.add("https://work.example".into(), "Work".into());
        assert_eq!(s.servers.len(), 1);
        assert_eq!(s.selected.as_deref(), Some("https://work.example"));
    }

    #[test]
    fn add_existing_url_updates_label_not_duplicates() {
        let mut s = ServerStore::default();
        s.add("https://work.example".into(), "Work".into());
        s.add("https://work.example".into(), "Office".into());
        assert_eq!(s.servers.len(), 1);
        assert_eq!(s.servers[0].label, "Office");
    }

    #[test]
    fn remove_selected_falls_back_to_first_remaining() {
        let mut s = ServerStore::default();
        s.add("https://work.example".into(), "Work".into());
        s.add("https://home.example".into(), "Home".into()); // now selected
        s.remove("https://home.example");
        assert_eq!(s.selected.as_deref(), Some("https://work.example"));
        s.remove("https://work.example");
        assert_eq!(s.selected, None);
        assert!(s.servers.is_empty());
    }

    #[test]
    fn select_rejects_unknown_url() {
        let mut s = ServerStore::default();
        s.add("https://work.example".into(), "Work".into());
        assert!(s.select("https://home.example").is_err());
        assert!(s.select("https://work.example").is_ok());
    }

    #[test]
    fn round_trips_through_a_file_and_treats_missing_as_empty() {
        let dir = std::env::temp_dir().join(format!("mw-servercfg-{}", std::process::id()));
        let path = dir.join("servers.json");
        let _ = std::fs::remove_dir_all(&dir);

        // Missing file -> empty store.
        assert_eq!(ServerStore::load(&path).unwrap(), ServerStore::default());

        let mut s = ServerStore::default();
        s.add("https://work.example".into(), "Work".into());
        s.add("https://home.example".into(), "Home".into());
        s.save(&path).unwrap();

        let loaded = ServerStore::load(&path).unwrap();
        assert_eq!(loaded, s);
        assert_eq!(loaded.selected.as_deref(), Some("https://home.example"));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn entry_serializes_camelcase() {
        let json = serde_json::to_string(&ServerEntry {
            url: "https://x".into(),
            label: "X".into(),
        })
        .unwrap();
        assert_eq!(json, r#"{"url":"https://x","label":"X"}"#);
    }
}
