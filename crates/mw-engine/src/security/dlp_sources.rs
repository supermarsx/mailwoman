//! Fresh owned policy sources; file I/O never occupies a CPU executor thread.
use super::{DlpError, DlpRule, MAX_POLICY_BYTES, SECURITY_POLICY_SETTINGS_KEY, validate_rules};
use mw_store::Store;
use std::{
    io::Read,
    sync::{Arc, OnceLock},
    time::Duration,
};
use tokio::sync::Semaphore;
const MAX_STORED_BYTES: usize = 8 * 1024 * 1024;
#[derive(PartialEq, Eq)]
pub(super) struct Snapshot {
    pub(super) environment: Option<String>,
    pub(super) stored: Option<String>,
}
impl Snapshot {
    pub(super) fn bytes(&self) -> usize {
        self.environment.as_ref().map_or(0, String::capacity)
            + self.stored.as_ref().map_or(0, String::capacity)
            + 1024
    }
}
pub(super) async fn read(
    store: &Store,
    environment: bool,
    budget: &crate::security::cpu_work::InputBudget,
) -> Result<Snapshot, DlpError> {
    let environment = if environment {
        match std::env::var("MW_DLP_RULES") {
            Ok(value) if value.trim().is_empty() => None,
            Ok(value) if value.trim_start().starts_with('[') => {
                if value.len() > MAX_POLICY_BYTES {
                    return Err(DlpError::PolicyUnavailable);
                }
                Some(value)
            }
            Ok(value) => {
                static IO: OnceLock<Arc<Semaphore>> = OnceLock::new();
                let permit = tokio::time::timeout(
                    Duration::from_secs(2),
                    IO.get_or_init(|| Arc::new(Semaphore::new(2)))
                        .clone()
                        .acquire_owned(),
                )
                .await
                .map_err(|_| DlpError::Busy)?
                .map_err(|_| DlpError::Busy)?;
                let worker_budget = budget.clone();
                let task = tokio::task::spawn_blocking(move || {
                    let _permit = permit;
                    let file = std::fs::File::open(value.trim())
                        .map_err(|_| DlpError::PolicyUnavailable)?;
                    if !file
                        .metadata()
                        .map_err(|_| DlpError::PolicyUnavailable)?
                        .is_file()
                    {
                        return Err(DlpError::PolicyUnavailable);
                    }
                    let mut text = String::new();
                    file.take(MAX_POLICY_BYTES as u64 + 1)
                        .read_to_string(&mut text)
                        .map_err(|_| DlpError::PolicyUnavailable)?;
                    if text.len() > MAX_POLICY_BYTES {
                        return Err(DlpError::PolicyUnavailable);
                    }
                    Ok((text, worker_budget))
                });
                let (text, _worker_budget) = tokio::time::timeout(Duration::from_secs(5), task)
                    .await
                    .map_err(|_| DlpError::Busy)?
                    .map_err(|error| {
                        if error.is_cancelled() {
                            DlpError::Busy
                        } else {
                            DlpError::PolicyUnavailable
                        }
                    })??;
                Some(text)
            }
            Err(std::env::VarError::NotPresent) => None,
            Err(_) => return Err(DlpError::PolicyUnavailable),
        }
    } else {
        None
    };
    let stored = store
        .get_setting_limited(SECURITY_POLICY_SETTINGS_KEY, MAX_STORED_BYTES as u64)
        .await
        .map_err(|_| DlpError::PolicyUnavailable)?;
    if stored
        .as_ref()
        .is_some_and(|value| value.len() > MAX_STORED_BYTES)
    {
        return Err(DlpError::PolicyUnavailable);
    }
    Ok(Snapshot {
        environment,
        stored,
    })
}
pub(super) fn decode(source: &Snapshot, merge_environment: bool) -> Result<Vec<DlpRule>, DlpError> {
    let mut rules = match &source.environment {
        Some(value) => validate_rules(value).map_err(|_| DlpError::PolicyUnavailable)?,
        None => Vec::new(),
    };
    if let Some(row) = &source.stored {
        let value: serde_json::Value =
            serde_json::from_str(row).map_err(|_| DlpError::PolicyUnavailable)?;
        if !value.is_object() {
            return Err(DlpError::PolicyUnavailable);
        }
        if let Some(field) = value.get("dlp_rules_json") {
            let text = field.as_str().ok_or(DlpError::PolicyUnavailable)?;
            if text.len() > MAX_POLICY_BYTES {
                return Err(DlpError::PolicyUnavailable);
            }
            for rule in validate_rules(text).map_err(|_| DlpError::PolicyUnavailable)? {
                if !merge_environment || !rules.iter().any(|prior| prior.id == rule.id) {
                    rules.push(rule);
                }
            }
        }
    }
    super::rules_bytes(&rules)?;
    Ok(rules)
}
