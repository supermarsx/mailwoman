//! Fixed panel operations; remote command names and argument positions are not caller input.
use crate::hosting_connections::{PanelConfig, PanelProvider};
use serde::{Deserialize, Serialize};

// No Debug: several variants contain a one-time replacement password.
#[derive(Deserialize)]
#[serde(
    tag = "action",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum PanelOperation {
    ListDomains,
    ListMailboxes {
        domain: String,
    },
    CreateDomain {
        domain: String,
        parent: Option<String>,
        owner: Option<String>,
        password: Option<String>,
    },
    SetDomainEnabled {
        domain: String,
        enabled: bool,
    },
    DeleteDomain {
        domain: String,
    },
    CreateMailbox {
        domain: String,
        account: String,
        password: String,
        quota_mi_b: Option<u32>,
        display_name: Option<String>,
    },
    SetMailboxQuota {
        domain: String,
        account: String,
        quota_mi_b: u32,
    },
    ResetMailboxPassword {
        domain: String,
        account: String,
        password: String,
    },
    SetMailboxEnabled {
        domain: String,
        account: String,
        enabled: bool,
    },
    DeleteMailbox {
        domain: String,
        account: String,
    },
    AddAlias {
        domain: String,
        account: String,
        alias: String,
    },
    RemoveAlias {
        domain: String,
        account: String,
        alias: String,
    },
}
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PanelResult {
    pub domains: Vec<PanelDomain>,
    pub mailboxes: Vec<PanelMailbox>,
    pub changed: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PanelDomain {
    pub name: String,
    pub owner: Option<String>,
    pub enabled: Option<bool>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PanelMailbox {
    pub account: String,
    pub address: String,
    pub enabled: Option<bool>,
    pub quota_mi_b: Option<u32>,
    pub aliases: Vec<String>,
}
#[derive(Debug)]
pub(crate) struct PanelError {
    pub code: &'static str,
    pub result_uncertain: bool,
}
impl PanelError {
    pub fn new(code: &'static str) -> Self {
        Self {
            code,
            result_uncertain: false,
        }
    }
}
impl std::fmt::Display for PanelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code)
    }
}
impl std::error::Error for PanelError {}

fn argument(value: &str, max: usize) -> Result<(), PanelError> {
    if value.is_empty() || value.len() > max || value.chars().any(char::is_control) {
        Err(PanelError::new("invalid-argument"))
    } else {
        Ok(())
    }
}
fn account(value: &str) -> Result<(), PanelError> {
    if value.is_empty()
        || value.len() > 128
        || value.starts_with('-')
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b'+'))
    {
        Err(PanelError::new("invalid-account"))
    } else {
        Ok(())
    }
}
fn domain(value: &str) -> Result<(), PanelError> {
    crate::domain_config::normalize_domain(value).map_err(|_| PanelError::new("invalid-domain"))?;
    if value.starts_with('-') || value.chars().any(char::is_control) {
        return Err(PanelError::new("invalid-domain"));
    }
    Ok(())
}
fn mailbox_account(value: &str, provider: PanelProvider) -> Result<(), PanelError> {
    if provider == PanelProvider::Virtualmin && value.contains('@') {
        let (local, realm) = value
            .split_once('@')
            .ok_or_else(|| PanelError::new("invalid-account"))?;
        account(local)?;
        domain(realm)
    } else {
        account(value)
    }
}
impl PanelOperation {
    pub fn is_mutation(&self) -> bool {
        !matches!(self, Self::ListDomains | Self::ListMailboxes { .. })
    }
    pub fn domain(&self) -> Option<&str> {
        match self {
            Self::ListDomains => None,
            Self::ListMailboxes { domain }
            | Self::CreateDomain { domain, .. }
            | Self::SetDomainEnabled { domain, .. }
            | Self::DeleteDomain { domain }
            | Self::CreateMailbox { domain, .. }
            | Self::SetMailboxQuota { domain, .. }
            | Self::ResetMailboxPassword { domain, .. }
            | Self::SetMailboxEnabled { domain, .. }
            | Self::DeleteMailbox { domain, .. }
            | Self::AddAlias { domain, .. }
            | Self::RemoveAlias { domain, .. } => Some(domain),
        }
    }
    pub fn validate(&self, config: &PanelConfig) -> Result<(), PanelError> {
        if let Some(value) = self.domain() {
            domain(value)?;
        }
        match self {
            Self::CreateMailbox { account: a, .. }
            | Self::SetMailboxQuota { account: a, .. }
            | Self::ResetMailboxPassword { account: a, .. }
            | Self::SetMailboxEnabled { account: a, .. }
            | Self::DeleteMailbox { account: a, .. }
            | Self::AddAlias { account: a, .. }
            | Self::RemoveAlias { account: a, .. } => mailbox_account(a, config.provider)?,
            _ => (),
        }
        match self {
            Self::CreateMailbox {
                password,
                display_name,
                ..
            } => {
                argument(password, 1024)?;
                if let Some(name) = display_name {
                    argument(name, 256)?;
                    if config.provider == PanelProvider::Hestia {
                        return Err(PanelError::new("display-name-unsupported"));
                    }
                }
            }
            Self::ResetMailboxPassword { password, .. } => argument(password, 1024)?,
            Self::AddAlias { alias, .. } | Self::RemoveAlias { alias, .. } => account(alias)?,
            Self::CreateDomain {
                parent,
                owner,
                password,
                ..
            } => {
                if let Some(parent) = parent {
                    domain(parent)?;
                }
                if let Some(owner) = owner {
                    account(owner)?;
                }
                if let Some(password) = password {
                    argument(password, 1024)?;
                }
                if config.provider == PanelProvider::Virtualmin
                    && parent.is_none()
                    && (owner.is_none() || password.is_none())
                {
                    return Err(PanelError::new("domain-owner-required"));
                }
                if config.provider == PanelProvider::Hestia
                    && (parent.is_some() || owner.is_some() || password.is_some())
                {
                    return Err(PanelError::new("use-configured-hestia-owner"));
                }
            }
            _ => (),
        }
        if config.provider == PanelProvider::Virtualmin
            && matches!(
                self,
                Self::CreateDomain { .. }
                    | Self::SetDomainEnabled { .. }
                    | Self::DeleteDomain { .. }
            )
            && !config.allow_domain_lifecycle
        {
            return Err(PanelError::new("domain-lifecycle-disabled"));
        }
        Ok(())
    }
}

/// The form is held only in memory and must never be logged.
pub(crate) fn form(
    config: &PanelConfig,
    secret: &str,
    op: &PanelOperation,
) -> Result<Vec<(String, String)>, PanelError> {
    op.validate(config)?;
    argument(secret, 8192)?;
    let pair = |a: &str, b: &str| (a.to_owned(), b.to_owned());
    let mut params = Vec::new();
    match config.provider {
        PanelProvider::Virtualmin => {
            params.push(pair("json", "1"));
            let program = match op {
                PanelOperation::ListDomains => {
                    params.push(pair("with-feature", "mail"));
                    "list-domains"
                }
                PanelOperation::ListMailboxes { .. } => "list-users",
                PanelOperation::CreateDomain {
                    parent,
                    owner,
                    password,
                    ..
                } => {
                    params.push(pair("mail", ""));
                    params.push(pair("dir", ""));
                    if let Some(parent) = parent {
                        params.push(pair("parent", parent));
                    } else {
                        params.push(pair("unix", ""));
                        params.push(pair("user", owner.as_deref().unwrap_or_default()));
                        params.push(pair("pass", password.as_deref().unwrap_or_default()));
                    }
                    "create-domain"
                }
                PanelOperation::SetDomainEnabled { enabled, .. } => {
                    if *enabled {
                        "enable-domain"
                    } else {
                        "disable-domain"
                    }
                }
                PanelOperation::DeleteDomain { .. } => "delete-domain",
                PanelOperation::CreateMailbox {
                    account,
                    password,
                    quota_mi_b,
                    display_name,
                    ..
                } => {
                    params.push(pair("user", account));
                    params.push(pair("pass", password));
                    if let Some(quota) = quota_mi_b {
                        params.push(pair(
                            "quota",
                            &u64::from(*quota).saturating_mul(1024).to_string(),
                        ));
                    }
                    if let Some(name) = display_name {
                        params.push(pair("real", name));
                    }
                    "create-user"
                }
                PanelOperation::SetMailboxQuota {
                    account,
                    quota_mi_b,
                    ..
                } => {
                    params.push(pair("user", account));
                    params.push(pair(
                        "quota",
                        &u64::from(*quota_mi_b).saturating_mul(1024).to_string(),
                    ));
                    "modify-user"
                }
                PanelOperation::ResetMailboxPassword {
                    account, password, ..
                } => {
                    params.push(pair("user", account));
                    params.push(pair("pass", password));
                    "modify-user"
                }
                PanelOperation::SetMailboxEnabled {
                    account, enabled, ..
                } => {
                    params.push(pair("user", account));
                    params.push(pair(if *enabled { "enable" } else { "disable" }, ""));
                    "modify-user"
                }
                PanelOperation::DeleteMailbox { account, .. } => {
                    params.push(pair("user", account));
                    "delete-user"
                }
                PanelOperation::AddAlias {
                    domain,
                    account,
                    alias,
                }
                | PanelOperation::RemoveAlias {
                    domain,
                    account,
                    alias,
                } => {
                    params.push(pair("user", account));
                    params.push(pair(
                        if matches!(op, PanelOperation::AddAlias { .. }) {
                            "add-email"
                        } else {
                            "remove-email"
                        },
                        &format!("{alias}@{domain}"),
                    ));
                    "modify-user"
                }
            };
            params.push(pair("program", program));
            if let Some(domain) = op.domain() {
                params.push(pair("domain", domain));
            }
        }
        PanelProvider::Hestia => {
            params.push(pair("hash", &format!("{}:{secret}", config.access_key)));
            params.push(pair(
                "returncode",
                if op.is_mutation() { "yes" } else { "no" },
            ));
            let mut args = vec![config.owner.clone()];
            if let Some(domain) = op.domain() {
                args.push(domain.into());
            }
            let cmd = match op {
                PanelOperation::ListDomains => {
                    args.push("json".into());
                    "v-list-mail-domains"
                }
                PanelOperation::ListMailboxes { .. } => {
                    args.push("json".into());
                    "v-list-mail-accounts"
                }
                PanelOperation::CreateDomain { .. } => "v-add-mail-domain",
                PanelOperation::SetDomainEnabled { enabled, .. } => {
                    if *enabled {
                        "v-unsuspend-mail-domain"
                    } else {
                        "v-suspend-mail-domain"
                    }
                }
                PanelOperation::DeleteDomain { .. } => "v-delete-mail-domain",
                PanelOperation::CreateMailbox {
                    account,
                    password,
                    quota_mi_b,
                    ..
                } => {
                    args.push(account.clone());
                    args.push(password.clone());
                    if let Some(q) = quota_mi_b {
                        args.push(q.to_string());
                    }
                    "v-add-mail-account"
                }
                PanelOperation::SetMailboxQuota {
                    account,
                    quota_mi_b,
                    ..
                } => {
                    args.push(account.clone());
                    args.push(quota_mi_b.to_string());
                    "v-change-mail-account-quota"
                }
                PanelOperation::ResetMailboxPassword {
                    account, password, ..
                } => {
                    args.push(account.clone());
                    args.push(password.clone());
                    "v-change-mail-account-password"
                }
                PanelOperation::SetMailboxEnabled {
                    account, enabled, ..
                } => {
                    args.push(account.clone());
                    if *enabled {
                        "v-unsuspend-mail-account"
                    } else {
                        "v-suspend-mail-account"
                    }
                }
                PanelOperation::DeleteMailbox { account, .. } => {
                    args.push(account.clone());
                    "v-delete-mail-account"
                }
                PanelOperation::AddAlias { account, alias, .. }
                | PanelOperation::RemoveAlias { account, alias, .. } => {
                    args.push(account.clone());
                    args.push(alias.clone());
                    if matches!(op, PanelOperation::AddAlias { .. }) {
                        "v-add-mail-account-alias"
                    } else {
                        "v-delete-mail-account-alias"
                    }
                }
            };
            params.push(pair("cmd", cmd));
            params.extend(
                args.into_iter()
                    .enumerate()
                    .map(|(i, v)| (format!("arg{}", i + 1), v)),
            );
        }
    }
    Ok(params)
}

fn text(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => {
            Some(s.chars().filter(|c| !c.is_control()).take(320).collect())
        }
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        serde_json::Value::Array(v) => v.first().and_then(text),
        _ => None,
    }
}
fn field(value: &serde_json::Value, names: &[&str]) -> Option<String> {
    for object in [value.get("values"), Some(value)]
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_object)
    {
        for name in names {
            if let Some((_, v)) = object
                .iter()
                .find(|(k, _)| k.to_ascii_lowercase().replace([' ', '-'], "_") == *name)
                && let Some(v) = text(v)
            {
                return Some(v);
            }
        }
    }
    None
}
fn yes(value: Option<String>) -> Option<bool> {
    match value?.to_ascii_lowercase().as_str() {
        "yes" | "true" | "1" => Some(true),
        "no" | "false" | "0" => Some(false),
        _ => None,
    }
}
fn enabled(value: &serde_json::Value) -> Option<bool> {
    yes(field(value, &["enabled"]))
        .or_else(|| yes(field(value, &["suspended", "disabled"])).map(|b| !b))
}
/// Project only known inventory fields. Providers may include password hashes,
/// filesystem paths and arbitrary command output; none is returned to the UI.
pub(crate) fn parse_response(
    provider: PanelProvider,
    op: &PanelOperation,
    bytes: &[u8],
) -> Result<PanelResult, PanelError> {
    if provider == PanelProvider::Hestia && op.is_mutation() {
        let code = std::str::from_utf8(bytes)
            .map_err(|_| PanelError::new("invalid-response"))?
            .trim();
        return match code {
            "0" => Ok(PanelResult {
                changed: true,
                ..Default::default()
            }),
            c if c.parse::<u32>().is_ok() => Err(PanelError::new("panel-command-rejected")),
            _ => Err(PanelError::new("invalid-response")),
        };
    }
    let body: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| PanelError::new("invalid-response"))?;
    let rows: Vec<(String, &serde_json::Value)> = match provider {
        PanelProvider::Virtualmin => {
            if body.get("status").and_then(serde_json::Value::as_str) != Some("success") {
                return Err(PanelError::new("panel-command-rejected"));
            }
            if op.is_mutation() {
                return Ok(PanelResult {
                    changed: true,
                    ..Default::default()
                });
            }
            body.get("data")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| PanelError::new("invalid-response"))?
                .iter()
                .map(|row| (field(row, &["name"]).unwrap_or_default(), row))
                .collect()
        }
        PanelProvider::Hestia => body
            .as_object()
            .ok_or_else(|| PanelError::new("invalid-response"))?
            .iter()
            .map(|(k, v)| (k.clone(), v))
            .collect(),
    };
    if rows.len() > 10000 {
        return Err(PanelError::new("inventory-too-large"));
    }
    let mut result = PanelResult::default();
    for (name, row) in rows {
        if !row.is_object()
            || name.is_empty()
            || name.len() > 320
            || name.chars().any(char::is_control)
        {
            return Err(PanelError::new("invalid-inventory"));
        }
        match op {
            PanelOperation::ListDomains => result.domains.push(PanelDomain {
                name,
                owner: field(row, &["username", "user", "owner"]),
                enabled: if provider == PanelProvider::Virtualmin {
                    Some(field(row, &["disabled"]).is_none())
                } else {
                    enabled(row)
                },
            }),
            PanelOperation::ListMailboxes { domain } => {
                let email = field(row, &["email_address", "email"]).filter(|v| v.contains('@'));
                if yes(field(row, &["email_enabled"])) == Some(false)
                    || (provider == PanelProvider::Virtualmin && email.is_none())
                {
                    continue;
                }
                let account = field(row, &["unix_username", "username", "user"]).unwrap_or(name);
                let address = email.unwrap_or_else(|| {
                    if account.contains('@') {
                        account.clone()
                    } else {
                        format!("{account}@{domain}")
                    }
                });
                let quota = field(
                    row,
                    if provider == PanelProvider::Virtualmin {
                        &["home_byte_quota"]
                    } else {
                        &["quota"]
                    },
                )
                .and_then(|v| {
                    if v.eq_ignore_ascii_case("unlimited") {
                        Some(0)
                    } else {
                        v.parse::<u64>().ok()
                    }
                });
                let quota_mi_b = quota.and_then(|q| {
                    u32::try_from(if provider == PanelProvider::Virtualmin {
                        q / (1024 * 1024)
                    } else {
                        q
                    })
                    .ok()
                });
                let aliases = field(
                    row,
                    &[
                        "alias",
                        "additional_email_addresses",
                        "extra_email_addresses",
                        "extra_addresses",
                    ],
                )
                .unwrap_or_default()
                .split([',', ' '])
                .filter(|v| !v.is_empty())
                .take(100)
                .map(str::to_owned)
                .collect();
                result.mailboxes.push(PanelMailbox {
                    account,
                    address,
                    enabled: enabled(row),
                    quota_mi_b,
                    aliases,
                });
            }
            _ => return Err(PanelError::new("invalid-response")),
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hosting_connections::{PanelConnection, PanelTls};
    fn config(provider: PanelProvider) -> PanelConfig {
        PanelConfig {
            id: "fixture".into(),
            name: "Fixture".into(),
            provider,
            base_url: "https://panel.example".into(),
            enabled: true,
            username: "admin".into(),
            owner: "owner".into(),
            access_key: "access".into(),
            allow_private: false,
            connection: PanelConnection::Direct,
            tls: PanelTls::default(),
            allow_domain_lifecycle: false,
        }
    }
    #[test]
    fn hestia_uses_scoped_access_secret_form_and_exact_positional_arguments() {
        let op = PanelOperation::CreateMailbox {
            domain: "example.org".into(),
            account: "alice".into(),
            password: "p&=secret".into(),
            quota_mi_b: Some(50),
            display_name: None,
        };
        let form = form(&config(PanelProvider::Hestia), "secret", &op).unwrap();
        for (key, value) in [
            ("hash", "access:secret"),
            ("returncode", "yes"),
            ("cmd", "v-add-mail-account"),
            ("arg1", "owner"),
            ("arg2", "example.org"),
            ("arg3", "alice"),
            ("arg4", "p&=secret"),
            ("arg5", "50"),
        ] {
            assert!(form.contains(&(key.into(), value.into())));
        }
    }
    #[test]
    fn virtualmin_converts_quota_and_rejects_argument_and_lifecycle_expansion() {
        let cfg = config(PanelProvider::Virtualmin);
        let op = PanelOperation::SetMailboxQuota {
            domain: "example.org".into(),
            account: "alice".into(),
            quota_mi_b: 50,
        };
        assert!(
            form(&cfg, "secret", &op)
                .unwrap()
                .contains(&("quota".into(), "51200".into()))
        );
        assert!(
            PanelOperation::ResetMailboxPassword {
                domain: "example.org".into(),
                account: "alice".into(),
                password: "safe\n--disable".into()
            }
            .validate(&cfg)
            .is_err()
        );
        assert!(
            PanelOperation::DeleteDomain {
                domain: "example.org".into()
            }
            .validate(&cfg)
            .is_err()
        );
    }
    #[test]
    fn inventory_projection_never_exposes_hashes_command_output_or_unknown_fields() {
        let op = PanelOperation::ListMailboxes {
            domain: "example.org".into(),
        };
        let value=parse_response(PanelProvider::Hestia,&op,br#"{"alice":{"QUOTA":"50","SUSPENDED":"no","ALIAS":"a,b","PASSWORD":"super-secret-hash","PATH":"/sensitive"}}"#).unwrap();
        let serialized = serde_json::to_string(&value).unwrap();
        assert!(!serialized.contains("secret") && !serialized.contains("sensitive"));
        assert_eq!(value.mailboxes[0].quota_mi_b, Some(50));
        assert_eq!(value.mailboxes[0].enabled, Some(true));
        let virtualmin=parse_response(PanelProvider::Virtualmin,&PanelOperation::ListDomains,br#"{"status":"success","output":"secret","data":[{"name":"example.org","values":{"Username":["owner"],"Password":["secret"]}}]}"#).unwrap();
        assert_eq!(virtualmin.domains[0].owner.as_deref(), Some("owner"));
        assert_eq!(virtualmin.domains[0].enabled, Some(true));
        assert!(
            !serde_json::to_string(&virtualmin)
                .unwrap()
                .contains("secret")
        );
    }

    #[test]
    fn virtualmin_mail_inventory_uses_byte_quota_and_omits_non_mail_users() {
        let op = PanelOperation::ListMailboxes {
            domain: "example.org".into(),
        };
        let value = parse_response(PanelProvider::Virtualmin, &op, br#"{"status":"success","data":[{"name":"Alice","values":{"unix_username":["alice@example.org"],"email_address":["alice@example.org"],"home_byte_quota":["52428800"],"extra_addresses":["a@example.org b@example.org"],"disabled":["No"],"password":["secret"],"encrypted_password":["hash"]}},{"name":"ftp-only","values":{"unix_username":["ftp"]}}]}"#).unwrap();
        assert_eq!(value.mailboxes.len(), 1);
        let mailbox = &value.mailboxes[0];
        assert_eq!(mailbox.account, "alice@example.org");
        assert_eq!(mailbox.address, "alice@example.org");
        assert_eq!(mailbox.quota_mi_b, Some(50));
        assert_eq!(mailbox.enabled, Some(true));
        assert_eq!(mailbox.aliases, ["a@example.org", "b@example.org"]);
        let serialized = serde_json::to_string(&value).unwrap();
        assert!(!serialized.contains("secret") && !serialized.contains("hash"));
    }
}
