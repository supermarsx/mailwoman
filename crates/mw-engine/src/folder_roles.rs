//! Account-local special-folder overrides layered over immutable backend roles.
//! Persisted mailbox roles remain discovery data; resetting a preference restores
//! that data without contacting the upstream or rewriting another account.
use crate::{
    Engine,
    backend::{EngineError, Result},
};
use mw_store::Mailbox;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};

pub const ROLES: [&str; 6] = ["inbox", "drafts", "sent", "trash", "junk", "archive"];
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged, deny_unknown_fields)]
pub enum RoleTarget {
    Id { id: String },
    Path { path: String },
}
impl RoleTarget {
    fn matches(&self, mailbox: &Mailbox) -> bool {
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
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RoleProblem {
    pub role: String,
    pub reason: &'static str,
}
pub struct RoleView {
    pub mailboxes: Vec<Mailbox>,
    /// Backend special folders remain protected even when their effective role is overridden.
    pub management_mailboxes: Vec<Mailbox>,
    pub problems: Vec<RoleProblem>,
    pub revision: Option<String>,
}
impl RoleView {
    pub fn state(&self, base: &str) -> String {
        match &self.revision {
            Some(revision) => format!("{base}~{revision}"),
            None => base.to_owned(),
        }
    }
    pub fn base_state<'a>(&self, state: &'a str) -> Option<&'a str> {
        match (&self.revision, state.split_once('~')) {
            (None, None) => Some(state),
            (Some(revision), Some((base, old))) if revision == old => Some(base),
            _ => None,
        }
    }
}
fn project(
    mut mailboxes: Vec<Mailbox>,
    targets: &BTreeMap<String, Option<RoleTarget>>,
) -> RoleView {
    let mut management_mailboxes = mailboxes.clone();
    let mut selected = HashMap::<String, usize>::new();
    let mut problems = Vec::new();
    // Include other server-defined roles when detecting collisions: mapping
    // Trash onto the automatic Inbox/All folder must never enable expunge.
    let mut roles = ROLES
        .iter()
        .map(|role| (*role).to_owned())
        .collect::<Vec<_>>();
    for mailbox in &mailboxes {
        if let Some(role) = &mailbox.role
            && !roles.contains(role)
        {
            roles.push(role.clone());
        }
    }
    for role in &roles {
        let target = targets.get(role).and_then(Option::as_ref);
        let matches = mailboxes
            .iter()
            .enumerate()
            .filter(|(_, mailbox)| match target {
                Some(target) => mailbox.uidvalidity != 0 && target.matches(mailbox),
                None => mailbox.role.as_ref() == Some(role),
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [index] => {
                selected.insert(role.clone(), *index);
            }
            [] if target.is_some() => problems.push(RoleProblem {
                role: role.clone(),
                reason: "targetNotFound",
            }),
            [] => {}
            _ => problems.push(RoleProblem {
                role: role.clone(),
                reason: "ambiguousTarget",
            }),
        }
    }
    let mut collisions = HashMap::<usize, usize>::new();
    for index in selected.values() {
        *collisions.entry(*index).or_default() += 1;
    }
    for mailbox in &mut mailboxes {
        mailbox.role = None;
    }
    for (role, index) in selected {
        if collisions[&index] > 1 {
            problems.push(RoleProblem {
                role,
                reason: "conflictingRoles",
            });
        } else {
            mailboxes[index].role = Some(role);
        }
    }
    for (protected, effective) in management_mailboxes.iter_mut().zip(&mailboxes) {
        if protected.role.is_none() {
            protected.role.clone_from(&effective.role);
        }
    }
    problems.sort_by(|a, b| a.role.cmp(&b.role));
    let revision = if targets.values().any(Option::is_some) || !problems.is_empty() {
        let projection = mailboxes
            .iter()
            .map(|mailbox| (&mailbox.id, &mailbox.role))
            .collect::<BTreeMap<_, _>>();
        let digest = Sha256::digest(
            serde_json::to_vec(&(targets, projection, &problems))
                .expect("role projection serializable"),
        );
        Some(
            digest[..12]
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        )
    } else {
        None
    };
    RoleView {
        mailboxes,
        management_mailboxes,
        problems,
        revision,
    }
}
impl Engine {
    pub async fn mailbox_role_view(&self, account: &str) -> Result<RoleView> {
        use mw_store::settings_policy::{ResolveInput, SettingSubject, resolve};
        let identity = self.store().get_account(account).await?.username;
        let subject = SettingSubject { personal_owner: None,
            domain: identity.rsplit_once('@').map_or("", |(_, domain)| domain),
            account_id: account,
            address: &identity,
        };
        let rows = self.store().setting_values_for_subject(&subject).await?;
        let keys = ROLES.map(|role| format!("mail.folderRole.{role}"));
        let policies = self
            .store()
            .get_setting_policies_batch(&keys.iter().map(String::as_str).collect::<Vec<_>>())
            .await?;
        let mut targets = BTreeMap::new();
        for (index, role) in ROLES.iter().enumerate() {
            let values = rows
                .iter()
                .filter(|row| row.key == keys[index])
                .collect::<Vec<_>>();
            let resolved = resolve(&ResolveInput {
                floor: "null",
                policy: policies.iter().find(|policy| policy.key == keys[index]),
                values: &values,
                ..Default::default()
            });
            let value = serde_json::from_str(&resolved.value)
                .map_err(|_| EngineError::Protocol("invalid folder role policy".into()))?;
            let target = parse_target(&value)
                .map_err(|_| EngineError::Protocol("invalid folder role target".into()))?;
            targets.insert((*role).to_owned(), target);
        }
        Ok(project(
            self.store().list_mailboxes(account).await?,
            &targets,
        ))
    }
    pub async fn effective_mailboxes(&self, account: &str) -> Result<Vec<Mailbox>> {
        Ok(self.mailbox_role_view(account).await?.mailboxes)
    }
    pub async fn effective_mailbox(&self, account: &str, id: &str) -> Result<Mailbox> {
        self.effective_mailboxes(account)
            .await?
            .into_iter()
            .find(|mailbox| mailbox.id == id)
            .ok_or(EngineError::MailboxNotFound(id.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mailbox(id: &str, name: &str, role: Option<&str>) -> Mailbox {
        Mailbox {
            id: id.into(),
            account_id: "account".into(),
            name: name.into(),
            role: role.map(str::to_owned),
            uidvalidity: 55,
            uidnext: 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        }
    }
    fn targets(role: &str, value: Value) -> BTreeMap<String, Option<RoleTarget>> {
        BTreeMap::from([(role.into(), parse_target(&value).unwrap())])
    }
    #[test]
    fn target_schema_is_bounded_and_excludes_ambiguous_or_control_values() {
        for value in [
            serde_json::json!({}),
            serde_json::json!({"id":"one","path":"Trash"}),
            serde_json::json!({"path":"Trash\nINBOX"}),
            serde_json::json!({"id":""}),
            serde_json::json!({"path":"x".repeat(513)}),
            serde_json::json!("Trash"),
        ] {
            assert!(parse_target(&value).is_err(), "{value}");
        }
        assert!(parse_target(&serde_json::json!({"path":"INBOX/Eliminados"})).is_ok());
        assert_eq!(parse_target(&Value::Null), Ok(None));
    }
    #[test]
    fn explicit_target_is_owned_and_missing_never_falls_back_to_automatic_trash() {
        let raw = vec![
            mailbox("inbox", "INBOX", Some("inbox")),
            mailbox("trash", "Trash", Some("trash")),
            mailbox("custom", "Deleted", None),
        ];
        let selected = project(
            raw.clone(),
            &targets("trash", serde_json::json!({"id":"custom"})),
        );
        assert_eq!(selected.mailboxes[1].role, None);
        assert_eq!(selected.mailboxes[2].role.as_deref(), Some("trash"));
        assert!(selected.problems.is_empty());
        let missing = project(
            raw.clone(),
            &targets("trash", serde_json::json!({"id":"foreign-account-id"})),
        );
        assert!(
            missing
                .mailboxes
                .iter()
                .all(|mailbox| mailbox.role.as_deref() != Some("trash"))
        );
        assert_eq!(missing.problems[0].reason, "targetNotFound");
        let reset = project(raw, &BTreeMap::new());
        assert_eq!(reset.mailboxes[1].role.as_deref(), Some("trash"));
        assert!(reset.revision.is_none());
    }
    #[test]
    fn duplicate_role_targets_remove_both_roles_instead_of_enabling_inbox_expunge() {
        let result = project(
            vec![
                mailbox("inbox", "INBOX", Some("inbox")),
                mailbox("trash", "Trash", Some("trash")),
            ],
            &targets("trash", serde_json::json!({"id":"inbox"})),
        );
        assert!(
            result
                .mailboxes
                .iter()
                .all(|mailbox| mailbox.role.is_none())
        );
        assert_eq!(result.problems.len(), 2);
        assert_eq!(
            result.management_mailboxes[0].role.as_deref(),
            Some("inbox")
        );
        assert_eq!(
            result.management_mailboxes[1].role.as_deref(),
            Some("trash")
        );
        assert!(
            result
                .problems
                .iter()
                .all(|problem| problem.reason == "conflictingRoles")
        );
    }
    #[test]
    fn mapping_revision_invalidates_stale_mailbox_changes_tokens() {
        let raw = vec![
            mailbox("trash", "Trash", Some("trash")),
            mailbox("other", "Deleted", None),
        ];
        let before = project(raw.clone(), &BTreeMap::new());
        let after = project(
            raw,
            &targets("trash", serde_json::json!({"path":"Deleted"})),
        );
        assert_eq!(before.state("42"), "42");
        assert_eq!(after.base_state("42"), None);
        let current = after.state("42");
        assert_eq!(after.base_state(&current), Some("42"));
        assert_eq!(before.base_state(&current), None);
    }
}
