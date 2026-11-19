//! Shared special-folder target contract; parsing needs no sync engine.
use crate::Mailbox;
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub const ROLES: [&str; 6] = ["inbox", "drafts", "sent", "trash", "junk", "archive"];
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged, deny_unknown_fields)]
pub enum RoleTarget {
    Id { id: String },
    Path { path: String },
}
impl RoleTarget {
    pub fn matches(&self, mailbox: &Mailbox) -> bool {
        match self {
            Self::Id { id } => mailbox.id == *id,
            Self::Path { path } => mailbox.name == *path,
        }
    }
}
pub fn parse_target(value: &Value) -> std::result::Result<Option<RoleTarget>, &'static str> {
    let target: Option<RoleTarget> = serde_json::from_value(value.clone())
        .map_err(|_| "must be null or exactly one mailbox id/path")?;
    if let Some(target) = &target {
        let text = match target {
            RoleTarget::Id { id } => id,
            RoleTarget::Path { path } => path,
        };
        if text.is_empty() || text.len() > 512 || text.chars().any(char::is_control) {
            return Err("mailbox target must contain 1–512 bytes without control characters");
        }
    }
    Ok(target)
}
