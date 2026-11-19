//! Private local control for a running server. No database connection, network
//! listener, command-line secret or service stop is needed by the deployment CLI.
use crate::{
    ServerKey, Store,
    backend::Backend,
    generation_files,
    online_maintenance::{ConversionOptions, ConversionReport},
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
const PURPOSE: &str = "mw-online-maintenance-command-v1";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Command {
    id: String,
    issued_at: i64,
    operation: Operation,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "options")]
enum Operation {
    Start(ConversionOptions),
    Cancel,
}
fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    name.into()
}

pub fn online_conversion_status_path(path: &Path) -> Option<ConversionReport> {
    let path = generation_files::canonical_logical_path(path).ok()?;
    let bytes =
        generation_files::bounded_read(&sidecar(&path, ".mw-online-status.json"), 65536).ok()?;
    serde_json::from_slice(&bytes).ok()
}
/// Durable local receipt, separate from the active conversion's progress report.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommandAcknowledgement {
    pub id: String,
    pub accepted: bool,
    pub error: Option<String>,
    pub acknowledged_at: i64,
}
pub fn online_conversion_acknowledgement(path: &Path, id: &str) -> Option<CommandAcknowledgement> {
    let path = generation_files::canonical_logical_path(path).ok()?;
    let bytes =
        generation_files::bounded_read(&sidecar(&path, ".mw-online-acks.json"), 65536).ok()?;
    let acknowledgements: Vec<CommandAcknowledgement> = serde_json::from_slice(&bytes).ok()?;
    acknowledgements.into_iter().find(|ack| ack.id == id)
}
fn acknowledge(path: &Path, id: String, result: Result<(), String>) -> Result<(), String> {
    let _lock = command_lock(path)?;
    let ack_path = sidecar(path, ".mw-online-acks.json");
    let mut acknowledgements: Vec<CommandAcknowledgement> =
        if ack_path.try_exists().map_err(|e| e.to_string())? {
            serde_json::from_slice(
                &generation_files::bounded_read(&ack_path, 65536).map_err(|e| e.to_string())?,
            )
            .map_err(|_| "Invalid local maintenance acknowledgement state")?
        } else {
            Vec::new()
        };
    let at = chrono::Utc::now().timestamp();
    acknowledgements.retain(|ack| at.saturating_sub(ack.acknowledged_at) <= 125 && ack.id != id);
    if acknowledgements.len() >= 64 {
        acknowledgements.drain(..acknowledgements.len() - 63);
    }
    acknowledgements.push(CommandAcknowledgement {
        id,
        accepted: result.is_ok(),
        error: result.err().map(|error| error.chars().take(128).collect()),
        acknowledged_at: at,
    });
    generation_files::private_atomic_write(
        &ack_path,
        &serde_json::to_vec(&acknowledgements).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
pub fn submit_online_conversion(
    path: &Path,
    key: &ServerKey,
    options: ConversionOptions,
) -> Result<String, String> {
    submit(path, key, Operation::Start(options))
}
pub fn submit_online_cancellation(path: &Path, key: &ServerKey) -> Result<String, String> {
    submit(path, key, Operation::Cancel)
}
fn submit(path: &Path, key: &ServerKey, operation: Operation) -> Result<String, String> {
    let path = generation_files::canonical_logical_path(path).map_err(|e| e.to_string())?;
    let _lock = command_lock(&path)?;
    let pending = sidecar(&path, ".mw-online-request");
    if pending.try_exists().map_err(|e| e.to_string())? {
        return Err(
            "A local maintenance command is pending; wait for the running server to accept it"
                .into(),
        );
    }
    let command = Command {
        id: uuid::Uuid::new_v4().to_string(),
        issued_at: chrono::Utc::now().timestamp(),
        operation,
    };
    let bytes = serde_json::to_vec(&command).map_err(|e| e.to_string())?;
    let authority = path.to_str().ok_or("Database path must be UTF-8")?;
    let sealed = key
        .seal_bound(&bytes, PURPOSE, &[authority])
        .map_err(|e| e.to_string())?;
    generation_files::private_atomic_write(&pending, &sealed).map_err(|e| e.to_string())?;
    Ok(command.id)
}
fn command_lock(path: &Path) -> Result<std::fs::File, String> {
    let file = generation_files::open_private_lock(&sidecar(path, ".mw-online-command.lock"))
        .map_err(|e| e.to_string())?;
    file.try_lock()
        .map_err(|_| "Another local maintenance command is being handled".to_string())?;
    Ok(file)
}
fn decode(path: &Path, key: &ServerKey, sealed: &[u8], at: i64) -> Result<Command, String> {
    let authority = path.to_str().ok_or("Database path must be UTF-8")?;
    let bytes = key
        .open_bound(sealed, PURPOSE, &[authority])
        .map_err(|_| "Local maintenance command authentication failed")?;
    let command: Command =
        serde_json::from_slice(&bytes).map_err(|_| "Invalid local maintenance command")?;
    if uuid::Uuid::parse_str(&command.id).is_err()
        || command.issued_at > at + 5
        || at.saturating_sub(command.issued_at) > 120
    {
        return Err("Expired or invalid local maintenance command".into());
    }
    Ok(command)
}
impl Store {
    pub async fn poll_online_conversion_command(&self) -> Result<(), String> {
        let Backend::Sqlite(live) = &self.backend else {
            return Ok(());
        };
        if !live.is_managed() {
            return Ok(());
        }
        let path = live.logical_path();
        let pending = sidecar(path, ".mw-online-request");
        if !pending.try_exists().map_err(|e| e.to_string())? {
            return Ok(());
        }
        let (command, claimed) = {
            let _lock = command_lock(path)?;
            let claimed = sidecar(
                path,
                &format!(".mw-online-claimed-{}", uuid::Uuid::new_v4()),
            );
            let bytes =
                generation_files::bounded_read(&pending, 65536).map_err(|e| e.to_string())?;
            // Remove the authenticated envelope from the pending slot before awaiting.
            std::fs::rename(&pending, &claimed).map_err(|e| e.to_string())?;
            let at = chrono::Utc::now().timestamp();
            let command = decode(path, &self.key, &bytes, at).and_then(|command| {
                let consumed_path = sidecar(path, ".mw-online-consumed.json");
                let mut consumed: Vec<(String, i64)> =
                    if consumed_path.try_exists().map_err(|e| e.to_string())? {
                        serde_json::from_slice(
                            &generation_files::bounded_read(&consumed_path, 65536)
                                .map_err(|e| e.to_string())?,
                        )
                        .map_err(|_| "Invalid local command replay state")?
                    } else {
                        Vec::new()
                    };
                consumed.retain(|(_, time)| at.saturating_sub(*time) <= 125);
                if consumed.iter().any(|(id, _)| id == &command.id) {
                    return Ok(None);
                }
                if consumed.len() >= 512 {
                    return Err("Local maintenance command limit reached".into());
                }
                consumed.push((command.id.clone(), at));
                generation_files::private_atomic_write(
                    &consumed_path,
                    &serde_json::to_vec(&consumed).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                Ok(Some(command))
            });
            (command, claimed)
        };
        let _ = std::fs::remove_file(claimed);
        let Some(command) = command? else {
            return Ok(());
        };
        if self
            .online_conversion_status()
            .is_some_and(|r| r.id == command.id)
        {
            return acknowledge(path, command.id, Ok(()));
        }
        let result = match command.operation {
            Operation::Start(options) => self
                .start_online_conversion_with_id(options, command.id.clone())
                .await
                .map(|_| ()),
            Operation::Cancel => {
                if self.cancel_online_conversion() {
                    Ok(())
                } else {
                    Err("No cancellable online conversion is running".into())
                }
            }
        };
        // The replay fence is persisted before execution. The receipt records
        // validation/busy failures without replacing another job's status.
        acknowledge(path, command.id, result.clone())?;
        result?;
        Ok(())
    }
    /// Call once during server startup after abandoned-copy recovery succeeds.
    pub fn start_online_conversion_control(&self) {
        let Backend::Sqlite(live) = &self.backend else {
            return;
        };
        if !live.is_managed() {
            return;
        }
        let store = self.clone();
        tokio::spawn(async move {
            loop {
                if let Err(error) = store.poll_online_conversion_command().await {
                    eprintln!("[mw-store] Local database maintenance command rejected: {error}");
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn envelopes_are_bound_to_key_path_and_lifetime() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("mail.db");
        let canonical = generation_files::canonical_logical_path(&path).unwrap();
        let key = ServerKey::generate();
        submit_online_conversion(&path, &key, ConversionOptions::default()).unwrap();
        let bytes =
            generation_files::bounded_read(&sidecar(&canonical, ".mw-online-request"), 65536)
                .unwrap();
        let now = chrono::Utc::now().timestamp();
        assert!(decode(&canonical, &key, &bytes, now).is_ok());
        assert!(decode(&canonical, &ServerKey::generate(), &bytes, now).is_err());
        assert!(decode(&root.path().join("other.db"), &key, &bytes, now).is_err());
        assert!(decode(&canonical, &key, &bytes, now + 121).is_err());
        assert!(submit_online_cancellation(&path, &key).is_err());
    }
    #[test]
    fn acknowledgement_history_and_errors_are_bounded() {
        let root = tempfile::tempdir().unwrap();
        let path = generation_files::canonical_logical_path(&root.path().join("mail.db")).unwrap();
        let mut first = String::new();
        let mut last = String::new();
        for index in 0..90 {
            let id = uuid::Uuid::new_v4().to_string();
            if index == 0 {
                first = id.clone();
            }
            last = id.clone();
            acknowledge(&path, id, Err("\u{0001}".repeat(1024))).unwrap();
        }
        assert!(online_conversion_acknowledgement(&path, &first).is_none());
        assert!(
            !online_conversion_acknowledgement(&path, &last)
                .unwrap()
                .accepted
        );
        let bytes =
            generation_files::bounded_read(&sidecar(&path, ".mw-online-acks.json"), 65536).unwrap();
        let entries: Vec<CommandAcknowledgement> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(entries.len(), 64);
        assert!(
            entries
                .iter()
                .all(|ack| ack.error.as_ref().unwrap().chars().count() == 128)
        );
    }
    #[tokio::test]
    async fn rejected_commands_have_receipts_without_replacing_active_job() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("mail.db");
        let key = ServerKey::generate();
        let store = Store::open(path.to_str().unwrap(), key.clone())
            .await
            .unwrap();
        let invalid = ConversionOptions {
            batch_rows: 0,
            ..ConversionOptions::default()
        };
        let invalid_id = submit_online_conversion(&path, &key, invalid).unwrap();
        let canonical = generation_files::canonical_logical_path(&path).unwrap();
        let pending = sidecar(&canonical, ".mw-online-request");
        let rejected_envelope = generation_files::bounded_read(&pending, 65536).unwrap();
        assert!(store.poll_online_conversion_command().await.is_err());
        let rejected = online_conversion_acknowledgement(&path, &invalid_id).unwrap();
        assert!(!rejected.accepted);
        assert!(rejected.error.is_some());
        assert!(store.online_conversion_status().is_none());
        // Replaying a rejected command neither starts a job nor loses its receipt.
        generation_files::private_atomic_write(&pending, &rejected_envelope).unwrap();
        store.poll_online_conversion_command().await.unwrap();
        assert!(
            !online_conversion_acknowledgement(&path, &invalid_id)
                .unwrap()
                .accepted
        );
        assert!(store.online_conversion_status().is_none());
        let active_id =
            submit_online_conversion(&path, &key, ConversionOptions::default()).unwrap();
        store.poll_online_conversion_command().await.unwrap();
        assert!(
            online_conversion_acknowledgement(&path, &active_id)
                .unwrap()
                .accepted
        );
        let busy_id = submit_online_conversion(&path, &key, ConversionOptions::default()).unwrap();
        assert!(store.poll_online_conversion_command().await.is_err());
        assert!(
            !online_conversion_acknowledgement(&path, &busy_id)
                .unwrap()
                .accepted
        );
        assert_eq!(store.online_conversion_status().unwrap().id, active_id);
        let cancel_id = submit_online_cancellation(&path, &key).unwrap();
        store.poll_online_conversion_command().await.unwrap();
        assert!(
            online_conversion_acknowledgement(&path, &cancel_id)
                .unwrap()
                .accepted
        );
        store.close().await;
    }
    #[tokio::test]
    async fn accepted_start_is_not_replayed() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("mail.db");
        let key = ServerKey::generate();
        let store = Store::open(path.to_str().unwrap(), key.clone())
            .await
            .unwrap();
        let id = submit_online_conversion(&path, &key, ConversionOptions::default()).unwrap();
        let canonical = generation_files::canonical_logical_path(&path).unwrap();
        let pending = sidecar(&canonical, ".mw-online-request");
        let bytes = generation_files::bounded_read(&pending, 65536).unwrap();
        store.poll_online_conversion_command().await.unwrap();
        generation_files::private_atomic_write(&pending, &bytes).unwrap();
        store.poll_online_conversion_command().await.unwrap();
        assert_eq!(store.online_conversion_status().unwrap().id, id);
        assert!(
            online_conversion_acknowledgement(&path, &id)
                .unwrap()
                .accepted
        );
        store.close().await;
    }
}
