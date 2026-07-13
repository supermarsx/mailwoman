//! Sender-control family (`SenderControl/set`, frozen §2.2). Wires to REAL
//! mechanisms, NOT localStorage (plan §1.9 / risk #14):
//! - `block` → a real `MailRule`/Sieve rule (`From is addr` → Move Junk + Stop),
//!   persisted via `rules.rs` (`set_rules`), engine-applied at ingest (and
//!   uploaded via ManageSieve where advertised — e7).
//! - `silence` → a per-sender notify-suppress flag (a setting).
//! - `ignore-conversation` → an auto-archive control scoped to a `threadId`
//!   (recorded in `sender_controls`; the engine archives matching messages).
//! - `report-phishing`/`report-junk` → a spam-trainer keyword on the message +
//!   (when `abuseReport`) a best-effort ARF emit via the account submitter to the
//!   configured abuse address (`MW_ABUSE_ADDRESS`); the abuse-relay endpoint is e7.

use mw_sieve::{Action, Condition, MatchOp, Rule, StringTest};
use serde_json::{Value, json};

use crate::account::AccountRuntime;
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;

use super::{gen_id, server_fail};

impl Engine {
    /// `SenderControl/set {emailId|address|threadId, action, abuseReport?}` →
    /// `{updated, mailRuleId?}` — applies the §1.9 real mechanism.
    pub(crate) async fn sender_control_set(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let action = args.get("action").and_then(Value::as_str).unwrap_or("");
        let address = args
            .get("address")
            .and_then(Value::as_str)
            .map(str::to_string);
        let thread_id = args
            .get("threadId")
            .and_then(Value::as_str)
            .map(str::to_string);
        let email_id = args.get("emailId").and_then(Value::as_str);
        let abuse_report = args
            .get("abuseReport")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        // Resolve the sender address from an emailId when not supplied directly.
        let resolved_address = match &address {
            Some(a) => Some(a.clone()),
            None => match email_id {
                Some(id) => self.sender_address_of(id).await,
                None => None,
            },
        };

        let now = chrono::Utc::now().to_rfc3339();
        let mut mail_rule_id: Option<String> = None;

        match action {
            "block" => {
                let Some(addr) = &resolved_address else {
                    return server_fail("block requires an address or a resolvable emailId");
                };
                match self.create_block_rule(account_id, rt, addr).await {
                    Ok(id) => mail_rule_id = Some(id),
                    Err(e) => return server_fail(e),
                }
            }
            "silence" => {
                if let Some(addr) = &resolved_address {
                    let key = format!("notify-suppress:{account_id}:{}", addr.to_lowercase());
                    let _ = self.store().set_setting(&key, "true").await;
                }
            }
            "ignore-conversation" => {
                // Recorded as a thread-scoped control; the engine auto-archives
                // matching messages (Sieve cannot match a threadId, plan §1.9).
                if thread_id.is_none() && email_id.is_none() {
                    return server_fail("ignore-conversation requires a threadId or emailId");
                }
            }
            "report-phishing" | "report-junk" => {
                if let Some(id) = email_id {
                    let keyword = if action == "report-phishing" {
                        "$Phishing"
                    } else {
                        "$Junk"
                    };
                    let _ = self.add_spam_keyword(rt, id, keyword).await;
                    if abuse_report {
                        let _ = self.emit_arf(rt, id, action).await;
                    }
                }
            }
            other => {
                return server_fail(format!("unknown sender-control action {other}"));
            }
        }

        // Record the control (linked to the real MailRule it made, if any).
        let _ = self
            .store()
            .insert_sender_control(&mw_store::SenderControlRow {
                account_id: account_id.to_string(),
                address: resolved_address,
                thread_id,
                action: action.to_string(),
                mail_rule_id: mail_rule_id.clone(),
                at: now,
            })
            .await;

        match &mail_rule_id {
            Some(id) => json!({ "updated": true, "mailRuleId": id }),
            None => json!({ "updated": true }),
        }
    }

    /// Create the real block rule for `addr` (`From is addr` → Move Junk + Stop),
    /// persist it via `rules.rs`, and record the `MailRule` change. Returns the id.
    async fn create_block_rule(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        addr: &str,
    ) -> Result<String, String> {
        let mut rules = self
            .get_rules(account_id)
            .await
            .map_err(|e| e.to_string())?;
        let id = gen_id("mr");
        rules.push(Rule {
            id: id.clone(),
            name: format!("Block {addr}"),
            match_all: true,
            conditions: vec![Condition::From(StringTest {
                op: MatchOp::Is,
                value: addr.to_string(),
            })],
            actions: vec![
                Action::Move {
                    mailbox: "Junk".into(),
                },
                Action::Stop,
            ],
            enabled: true,
        });
        self.set_rules(account_id, &rules)
            .await
            .map_err(|e| e.to_string())?;
        // Best-effort ManageSieve upload where advertised (e7 owns the transport).
        let _ = self.upload_sieve_if_supported(rt, &rules).await;
        let _ = self
            .record_crypto_change(account_id, ChangeType::MailRule, &id, ChangeOp::Created)
            .await;
        self.broadcast_state(account_id).await;
        Ok(id)
    }

    /// Resolve the From address of a stored message (for `SenderControl` by emailId).
    async fn sender_address_of(&self, email_id: &str) -> Option<String> {
        let raw = {
            let msg = self.store().get_message(email_id).await.ok()?;
            let blob = msg.blob_ref.as_ref()?;
            self.store().get_body(blob).await.ok().flatten()?
        };
        let parsed = mw_mime::parse(&raw).ok()?.email;
        parsed
            .from
            .as_ref()
            .and_then(|f| f.first())
            .map(|a| a.email.clone())
            .filter(|s| !s.is_empty())
    }

    /// Tag a message with a spam-trainer keyword (`$Junk`/`$Phishing`), best-effort.
    async fn add_spam_keyword(
        &self,
        rt: &AccountRuntime,
        email_id: &str,
        keyword: &str,
    ) -> Result<(), String> {
        self.add_keyword_local(rt, email_id, keyword)
            .await
            .map_err(|e| e.to_string())
    }

    /// Emit a best-effort ARF abuse report for `email_id` via the account submitter
    /// to `MW_ABUSE_ADDRESS`. A no-op when the address is unset; the full abuse
    /// relay + `report` feature wiring is e7's endpoint (plan §3 e7). Non-fatal.
    async fn emit_arf(
        &self,
        rt: &AccountRuntime,
        email_id: &str,
        action: &str,
    ) -> Result<(), String> {
        let Ok(abuse) = std::env::var("MW_ABUSE_ADDRESS") else {
            return Ok(());
        };
        let raw = {
            let msg = self
                .store()
                .get_message(email_id)
                .await
                .map_err(|e| e.to_string())?;
            match msg.blob_ref.as_ref() {
                Some(b) => self
                    .store()
                    .get_body(b)
                    .await
                    .map_err(|e| e.to_string())?
                    .unwrap_or_default(),
                None => Vec::new(),
            }
        };
        let feedback_type = if action == "report-phishing" {
            "fraud"
        } else {
            "abuse"
        };
        let arf = build_arf(&rt.identity, &abuse, feedback_type, &raw);
        let _ = rt
            .submitter
            .submit(mw_smtp::Outgoing {
                mail_from: rt.identity.clone(),
                rcpt_to: vec![abuse],
                raw: arf,
            })
            .await;
        Ok(())
    }
}

/// A minimal RFC 5965 ARF (`multipart/report; report-type=feedback-report`).
/// Content-free beyond the reported headers; the abuse address decides handling.
fn build_arf(from: &str, to: &str, feedback_type: &str, original: &[u8]) -> Vec<u8> {
    let boundary = "mw-arf-boundary";
    let date = chrono::Utc::now().to_rfc2822();
    let original_str = String::from_utf8_lossy(original);
    format!(
        "From: {from}\r\n\
         To: {to}\r\n\
         Subject: Abuse report\r\n\
         Date: {date}\r\n\
         MIME-Version: 1.0\r\n\
         Content-Type: multipart/report; report-type=feedback-report; boundary=\"{boundary}\"\r\n\
         \r\n\
         --{boundary}\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\r\n\
         This is an email abuse report for a message received from {from}.\r\n\
         \r\n\
         --{boundary}\r\n\
         Content-Type: message/feedback-report\r\n\r\n\
         Feedback-Type: {feedback_type}\r\n\
         User-Agent: Mailwoman/1.0\r\n\
         Version: 1\r\n\
         \r\n\
         --{boundary}\r\n\
         Content-Type: message/rfc822\r\n\r\n\
         {original_str}\r\n\
         --{boundary}--\r\n"
    )
    .into_bytes()
}
