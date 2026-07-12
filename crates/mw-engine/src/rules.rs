//! Engine-side rule execution at ingest (plan §0.6, §3 e9).
//!
//! Rules are authored in the GUI as [`mw_sieve::Rule`]s and stored per account
//! (as JSON under the `rules:{account}` setting). Where a backend advertises
//! ManageSieve, `mw-sieve` uploads the generated Sieve; otherwise — the
//! always-green path (plan risk #9) — the engine evaluates the rules itself as
//! each new inbox message arrives and applies the resulting actions locally.
//!
//! Only genuinely new inbox arrivals are evaluated (see [`Engine::ingest`]); our
//! own drafts/sent copies and historical re-syncs never fire rules.

use mw_jmap::Email;
use mw_sieve::{Action, ParsedEnvelope, Rule, evaluate_all};

use crate::account::AccountRuntime;
use crate::backend::{EngineError, Flag, Result};
use crate::engine::Engine;
use crate::mapping::{flags_from_json, flags_to_json, flags_to_keywords};

impl Engine {
    /// The account's stored rules (empty when none configured or unparseable).
    pub async fn get_rules(&self, account_id: &str) -> Result<Vec<Rule>> {
        let key = format!("rules:{account_id}");
        let json = self.store().get_setting(&key).await?;
        Ok(json
            .and_then(|j| serde_json::from_str(&j).ok())
            .unwrap_or_default())
    }

    /// Persist the account's rule set (GUI → engine). Overwrites in full.
    pub async fn set_rules(&self, account_id: &str, rules: &[Rule]) -> Result<()> {
        let key = format!("rules:{account_id}");
        let json = serde_json::to_string(rules).unwrap_or_else(|_| "[]".into());
        self.store().set_setting(&key, &json).await?;
        Ok(())
    }

    /// Evaluate the account's rules against a freshly-ingested inbox message and
    /// apply the actions (`Move`/`Tag`/`Mark`; other actions are best-effort or
    /// no-ops engine-side). Runs only for messages delivered into the inbox.
    pub(crate) async fn apply_rules_at_ingest(
        &self,
        account_id: &str,
        mailbox_id: &str,
        stable_id: &str,
        email: &Email,
        flags: &[Flag],
    ) -> Result<()> {
        // Only inbox deliveries are subject to rules.
        let mailbox = self
            .store()
            .get_mailbox(mailbox_id)
            .await
            .map_err(crate::backend::EngineError::Store)?;
        if mailbox.role.as_deref() != Some("inbox") {
            return Ok(());
        }
        let rules = self.get_rules(account_id).await?;
        if rules.is_empty() {
            return Ok(());
        }

        let envelope = envelope_for_rules(email, flags);
        let actions = evaluate_all(&rules, &envelope);
        if actions.is_empty() {
            return Ok(());
        }

        let Some(rt) = self.runtime(account_id) else {
            return Ok(());
        };
        // Resolve move targets against the account's mailboxes (by name or role).
        let mailboxes = self.store().list_mailboxes(account_id).await?;
        for action in &actions {
            match action {
                Action::Move { mailbox: target } => {
                    if let Some(dest) = resolve_mailbox(&mailboxes, target)
                        && dest != mailbox_id
                    {
                        self.move_email(&rt, stable_id, &dest).await?;
                    }
                }
                Action::Tag { keyword } | Action::Mark { keyword } => {
                    self.add_keyword_local(&rt, stable_id, keyword).await?;
                }
                // Copy/Forward/ReplyTemplate/Notify/Stop have no local cache
                // effect (or are handled by evaluate_all's stop semantics).
                _ => {}
            }
        }
        Ok(())
    }

    /// Add one JMAP keyword to a message: update the cache flags, re-index, and
    /// best-effort mirror upstream (a POP3/local `Unsupported` is fine).
    async fn add_keyword_local(
        &self,
        rt: &AccountRuntime,
        stable_id: &str,
        keyword: &str,
    ) -> Result<()> {
        let msg = self
            .store()
            .get_message(stable_id)
            .await
            .map_err(EngineError::Store)?;
        let mut flags = flags_from_json(&msg.flags_json);
        let kw = Flag::Keyword(keyword.to_string());
        if flags.contains(&kw) {
            return Ok(());
        }
        // Mirror upstream where addressable (best-effort).
        if let Some(mref) = self.imap_ref_for(stable_id).await?
            && let Err(e) = rt
                .backend
                .store_flags(&[mref], std::slice::from_ref(&kw), &[])
                .await
            && !matches!(e, EngineError::Unsupported(_))
        {
            return Err(e);
        }
        flags.push(kw);
        self.store()
            .set_flags(stable_id, &flags_to_json(&flags))
            .await?;
        self.reindex_message(stable_id).await;
        Ok(())
    }
}

/// The minimal envelope the Sieve evaluator matches against.
fn envelope_for_rules(email: &Email, flags: &[Flag]) -> ParsedEnvelope {
    let join = |list: &Option<Vec<mw_jmap::EmailAddress>>| -> String {
        list.as_ref()
            .map(|v| {
                v.iter()
                    .map(|a| match &a.name {
                        Some(n) if !n.is_empty() => format!("{n} <{}>", a.email),
                        _ => a.email.clone(),
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default()
    };
    let body = email
        .body_values
        .values()
        .map(|v| v.value.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    ParsedEnvelope {
        from: join(&email.from),
        to: join(&email.to),
        subject: email.subject.clone().unwrap_or_default(),
        body,
        has_attachment: email.has_attachment,
        size: email.size,
        keywords: flags_to_keywords(flags).into_keys().collect(),
    }
}

/// Resolve a rule's move target (a mailbox name or role) to a mailbox id.
fn resolve_mailbox(mailboxes: &[mw_store::Mailbox], target: &str) -> Option<String> {
    mailboxes
        .iter()
        .find(|m| {
            m.name.eq_ignore_ascii_case(target)
                || m.role
                    .as_deref()
                    .is_some_and(|r| r.eq_ignore_ascii_case(target))
        })
        .map(|m| m.id.clone())
}
