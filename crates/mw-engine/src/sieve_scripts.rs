//! Local raw-Sieve persistence and ingest execution for accounts without ManageSieve.
use crate::backend::{EngineError, Flag, RawMailboxRef, Result};
use crate::engine::Engine;
use crate::mapping::{flags_from_json, flags_to_json};
use mw_sieve::local::{Action, FlagOperation, Message};
use mw_sieve::policy::{EditKind, Policy};
#[path = "sieve_vacation_runtime.rs"]
mod vacation_runtime;

impl Engine {
    /// Runtime policy uses the same admin-only GDA resolution and built-in
    /// defaults as the public registry. Read on each delivery for hot revocation.
    pub async fn resolved_sieve_policy(&self, account: &str) -> Result<Policy> {
        use mw_store::settings_policy::{ResolveInput, SettingScope, SettingSubject, resolve};
        let identity = self.store().get_account(account).await?.username;
        let domain = identity.rsplit_once('@').map_or("", |(_, d)| d);
        let subject = SettingSubject {
            domain,
            account_id: account,
            address: &identity,
        };
        let rows = self.store().setting_values_for_subject(&subject).await?;
        let mut values = std::collections::HashMap::new();
        for (key, floor) in [
            ("sieve.maxScriptBytes", "65536"),
            ("sieve.maxScripts", "5"),
            ("sieve.maxRedirects", "4"),
            ("sieve.vacationAllowed", "false"),
            ("sieve.allowedExtensions", "null"),
            ("cap.sieve.raw_edit", "false"),
            ("cap.sieve.forward_external", "false"),
        ] {
            let mut policy = self.store().get_setting_policy(key).await?;
            if let Some(policy) = &mut policy {
                policy.allow_user_override = false;
            }
            let mine: Vec<_> = rows
                .iter()
                .filter(|r| {
                    r.key == key
                        && r.scope != SettingScope::User
                        && (r.scope != SettingScope::AdminUser || r.scope_id == account)
                })
                .collect();
            let resolved = resolve(&ResolveInput {
                floor,
                policy: policy.as_ref(),
                values: &mine,
                ..Default::default()
            });
            values.insert(
                key,
                serde_json::from_str::<serde_json::Value>(&resolved.value)
                    .map_err(|_| EngineError::Protocol("invalid runtime Sieve policy".into()))?,
            );
        }
        Ok(Policy {
            max_script_bytes: Some(
                values["sieve.maxScriptBytes"].as_u64().unwrap_or(65536) as usize
            ),
            max_scripts: Some(values["sieve.maxScripts"].as_u64().unwrap_or(5) as usize),
            max_redirects: Some(values["sieve.maxRedirects"].as_u64().unwrap_or(4) as usize),
            vacation_allowed: values["sieve.vacationAllowed"].as_bool().unwrap_or(false),
            raw_edit: values["cap.sieve.raw_edit"].as_bool().unwrap_or(false),
            forward_external: values["cap.sieve.forward_external"]
                .as_bool()
                .unwrap_or(false),
            allowed_extensions: values["sieve.allowedExtensions"].as_array().map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            }),
            own_domains: self.store().policy_domain_family(domain).await?,
            advertised: Some(
                mw_sieve::local::EXTENSIONS
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
        })
    }
    /// Store a locally executable script after policy validation. Policy is
    /// derived by the authenticated server from the account, never client JSON.
    pub async fn put_local_sieve_script(
        &self,
        account_id: &str,
        name: &str,
        source: &str,
        activate: bool,
        policy: &Policy,
        edit: EditKind,
    ) -> Result<()> {
        if name.is_empty()
            || name.len() > 512
            || name
                .chars()
                .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
        {
            return Err(EngineError::Protocol("invalid Sieve script name".into()));
        }
        if source.is_empty() {
            return Err(EngineError::Protocol(
                "Sieve source must not be empty".into(),
            ));
        }
        mw_sieve::policy::enforce(source, policy, edit).map_err(|errors| {
            EngineError::Protocol(
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        })?;
        mw_sieve::local::validate(source).map_err(|e| EngineError::Protocol(e.to_string()))?;
        if !self
            .store()
            .put_sieve_script_limited(account_id, name, source, activate, policy.max_scripts)
            .await?
        {
            return Err(EngineError::Protocol(
                "Sieve script count quota exceeded".into(),
            ));
        }
        Ok(())
    }

    /// Returns true when an active raw script replaces the legacy GUI rules.
    pub(crate) async fn apply_local_sieve_at_ingest(
        &self,
        account_id: &str,
        mailbox_id: &str,
        stable_id: &str,
        email: &mw_jmap::Email,
        raw: &[u8],
    ) -> Result<bool> {
        if self.store().get_mailbox(mailbox_id).await?.role.as_deref() != Some("inbox") {
            return Ok(false);
        }
        let scripts = self.store().list_sieve_scripts(account_id).await?;
        let Some(script) = scripts.iter().find(|s| s.active) else {
            return Ok(self.store().has_sieve_authority(account_id).await?);
        };
        // ManageSieve snapshots are for JMAP discovery only: the delivery server
        // already executed them. Never replay redirects or vacation locally.
        if serde_json::from_str::<serde_json::Value>(&script.state_json)
            .ok()
            .is_some_and(|v| v["_remote"] == true)
        {
            return Ok(true);
        }
        let policy = self.resolved_sieve_policy(account_id).await?;
        mw_sieve::policy::enforce(&script.content, &policy, EditKind::Builder).map_err(|_| {
            EngineError::Unsupported("active Sieve script is refused by current policy".into())
        })?;
        let rt = self
            .runtime(account_id)
            .ok_or_else(|| EngineError::Unsupported("account is not connected".into()))?;
        let mut message = local_message(raw, email);
        message.flags = flags_from_json(&self.store().get_message(stable_id).await?.flags_json)
            .iter()
            .map(|f| match f {
                Flag::Seen => "\\Seen".into(),
                Flag::Answered => "\\Answered".into(),
                Flag::Flagged => "\\Flagged".into(),
                Flag::Deleted => "\\Deleted".into(),
                Flag::Draft => "\\Draft".into(),
                Flag::Recent => "\\Recent".into(),
                Flag::Keyword(v) => v.clone(),
            })
            .collect();
        let actions = mw_sieve::local::evaluate(&script.content, &message)
            .map_err(|e| EngineError::Protocol(e.to_string()))?;
        let mailboxes = self.store().list_mailboxes(account_id).await?;
        let keep = actions.iter().any(|a| matches!(a, Action::Keep));
        let last_move = actions
            .iter()
            .rposition(|a| matches!(a, Action::FileInto { copy: false, .. }));
        let mut discard = false;
        for (index, action) in actions.iter().enumerate() {
            match action {
                Action::Vacation(vacation) => {
                    self.send_local_vacation(account_id, &rt, &message, email, vacation)
                        .await?;
                }
                Action::Keep => {}
                Action::Discard => discard = true,
                Action::FileInto {
                    mailbox,
                    copy,
                    flags,
                } => {
                    let dest = mailboxes
                        .iter()
                        .find(|m| {
                            m.name.eq_ignore_ascii_case(mailbox)
                                || m.role
                                    .as_deref()
                                    .is_some_and(|r| r.eq_ignore_ascii_case(mailbox))
                        })
                        .ok_or_else(|| {
                            EngineError::Protocol("Sieve destination mailbox not found".into())
                        })?;
                    let current = self.store().get_message(stable_id).await?;
                    let old = flags_from_json(&current.flags_json);
                    let delivery_flags: Vec<_> = flags
                        .as_ref()
                        .map(|v| v.iter().map(|f| local_flag(f)).collect())
                        .unwrap_or_else(|| old.clone());
                    if !copy && !keep && last_move == Some(index) && flags.is_some() {
                        let add: Vec<_> = delivery_flags
                            .iter()
                            .filter(|f| !old.contains(f))
                            .cloned()
                            .collect();
                        let remove: Vec<_> = old
                            .iter()
                            .filter(|f| !delivery_flags.contains(f))
                            .cloned()
                            .collect();
                        if let Some(reference) = self.imap_ref_for(stable_id).await? {
                            rt.backend.store_flags(&[reference], &add, &remove).await?;
                        }
                        self.store()
                            .set_flags(stable_id, &flags_to_json(&delivery_flags))
                            .await?;
                        self.invalidate_message_cache(stable_id).await;
                        self.reindex_message(stable_id).await;
                    }
                    if dest.id == mailbox_id {
                        continue;
                    }
                    if !copy && !keep && last_move == Some(index) {
                        self.move_email(&rt, stable_id, &dest.id).await?;
                    } else {
                        rt.backend
                            .append(
                                &RawMailboxRef {
                                    name: dest.name.clone(),
                                    uidvalidity: dest.uidvalidity,
                                },
                                raw,
                                &delivery_flags,
                            )
                            .await?;
                    }
                }
                Action::Flags { operation, flags } => {
                    let current = self.store().get_message(stable_id).await?;
                    let old = flags_from_json(&current.flags_json);
                    let requested: Vec<_> = flags.iter().map(|flag| local_flag(flag)).collect();
                    let mut next = match operation {
                        FlagOperation::Set => requested.clone(),
                        _ => old.clone(),
                    };
                    if *operation == FlagOperation::Add {
                        for flag in &requested {
                            if !next.contains(flag) {
                                next.push(flag.clone());
                            }
                        }
                    }
                    if *operation == FlagOperation::Remove {
                        next.retain(|flag| !requested.contains(flag));
                    }
                    let add: Vec<_> = next.iter().filter(|f| !old.contains(f)).cloned().collect();
                    let remove: Vec<_> =
                        old.iter().filter(|f| !next.contains(f)).cloned().collect();
                    if let Some(reference) = self.imap_ref_for(stable_id).await? {
                        rt.backend.store_flags(&[reference], &add, &remove).await?;
                    }
                    self.store()
                        .set_flags(stable_id, &flags_to_json(&next))
                        .await?;
                    self.invalidate_message_cache(stable_id).await;
                    self.reindex_message(stable_id).await;
                }
                Action::Redirect { address, copy } => {
                    // Recheck the resolved runtime policy, including hot revocation.
                    let domain = rt.identity.rsplit_once('@').map(|(_, domain)| domain);
                    if !policy.forward_external
                        && (domain.is_none()
                            || address.rsplit_once('@').is_none_or(|(_, target)| {
                                !domain.unwrap_or("").eq_ignore_ascii_case(target)
                            }))
                    {
                        return Err(EngineError::Unsupported(
                            "external local Sieve forwarding is disabled".into(),
                        ));
                    }
                    if message
                        .headers
                        .iter()
                        .any(|(name, _)| name.eq_ignore_ascii_case("X-Mailwoman-Loop"))
                    {
                        continue;
                    }
                    let mut forwarded = b"X-Mailwoman-Loop: local-sieve\r\n".to_vec();
                    forwarded.extend_from_slice(raw);
                    let outgoing = mw_smtp::Outgoing {
                        mail_from: rt.identity.clone(),
                        rcpt_to: vec![address.clone()],
                        raw: forwarded,
                    };
                    outgoing
                        .validate()
                        .map_err(|e| EngineError::Protocol(e.to_string()))?;
                    let result = rt.submitter.submit(outgoing).await?;
                    if result.accepted.is_empty() {
                        return Err(EngineError::Protocol("Sieve redirect was refused".into()));
                    }
                    discard |= !copy;
                }
            }
        }
        if discard && !keep && last_move.is_none() {
            let trash = mailboxes
                .iter()
                .find(|m| m.role.as_deref() == Some("trash"))
                .ok_or_else(|| {
                    EngineError::Unsupported("local discard requires a Trash mailbox".into())
                })?;
            self.move_email(&rt, stable_id, &trash.id).await?;
        }
        Ok(true)
    }
}

fn local_flag(flag: &str) -> Flag {
    match flag.to_ascii_lowercase().as_str() {
        "\\seen" => Flag::Seen,
        "\\answered" => Flag::Answered,
        "\\flagged" => Flag::Flagged,
        "\\deleted" => Flag::Deleted,
        "\\draft" => Flag::Draft,
        "\\recent" => Flag::Recent,
        _ => Flag::Keyword(flag.to_string()),
    }
}
fn local_message(raw: &[u8], email: &mw_jmap::Email) -> Message {
    let text = String::from_utf8_lossy(raw);
    let mut headers: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            break;
        }
        if line.starts_with([' ', '\t']) {
            if let Some((_, value)) = headers.last_mut() {
                value.push(' ');
                value.push_str(line.trim());
            }
        } else if let Some((name, value)) = line.split_once(':') {
            headers.push((name.to_string(), value.trim().to_string()));
        }
    }
    let envelope_from = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("Return-Path"))
        .map(|(_, value)| value.trim_matches(['<', '>']).to_string());
    let envelope_to = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("Delivered-To"))
        .map(|(_, value)| value.clone());
    Message {
        current_time: None,
        headers,
        flags: Vec::new(),
        body: email
            .body_values
            .values()
            .map(|value| value.value.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        size: raw.len() as u64,
        envelope_from,
        envelope_to,
    }
}
