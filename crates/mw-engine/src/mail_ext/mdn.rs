//! Read receipts (RFC 8098 message disposition notifications): what
//! `Email/get` reports about a request or a received report, the per-account
//! policy, and `MDN/send`.
//!
//! A receipt tells the sender that the message was displayed, from which
//! address, and when. The rules here decide when one may leave:
//!
//! - **Only on request.** The message carries exactly one
//!   `Disposition-Notification-To` naming one mailbox that passes
//!   `mw_smtp::validate_mailbox` (`mw-mime` reads it; anything else is no
//!   request). That mailbox is the only recipient a receipt can have.
//! - **Never in answer to a notification.** A message that is itself a
//!   `multipart/report`, or whose `Return-Path` is the null path `<>`, is not
//!   answered.
//! - **Never for the account's own or discarded mail.** A message in Drafts,
//!   Sent, Junk or Trash, or one carrying `$draft`, is not answered.
//! - **At most once.** A claim is written to the `settings` table before
//!   anything is built and `$mdnsent` is set on the message before the
//!   submitter is called; a failed submission releases both. A claim that was
//!   neither completed nor released (the process stopped in between) stays:
//!   the receipt is then never sent, not sent twice.
//! - **Automatic only under three conditions together** (RFC 8098 §2.1): the
//!   stored policy is `always`, the request address equals the message's
//!   `Return-Path`, and the message is not list or bulk mail. Without a
//!   `Return-Path` header the comparison fails.
//! - **From an address the message was sent to.** `Final-Recipient` and the
//!   report's `From` are an address of this account (its connected identity or
//!   a stored sending identity) that appears in the original's `To` or `Cc`.
//!   A message that reached the account through an alias, a masked address or
//!   a blind copy names no such address and is not answered, so a receipt
//!   never tells the sender an address they did not write to.
//!
//! The policy `never` does not stop a manual `MDN/send`: it is what the
//! first-party client reads to show no prompt, and with `ask` it refuses every
//! automatic one.
//!
//! The report goes out with the null reverse path (RFC 8098 §3) through the
//! account's submitter, the same one ordinary mail uses.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use mail_parser::{MessageParser, MimeHeaders};
use mw_mime::{EmailAddress, MdnActionMode, MdnDisposition, MdnInput, MdnSendingMode};
use serde_json::{Value, json};

use crate::backend::{EngineError, Flag, Result};
use crate::engine::Engine;
use crate::mapping::flags_from_json;

use super::server_fail;

/// The keyword that records a receipt was sent for a message (RFC 8621 §4.1.1,
/// IMAP `$MDNSent`).
pub(crate) const MDN_SENT_KEYWORD: &str = "$mdnsent";

/// Mailbox roles whose messages are never answered with a receipt.
const NO_RECEIPT_ROLES: [&str; 4] = ["drafts", "sent", "junk", "trash"];

/// Ledger value of a claim that was given up: the receipt was not sent and may
/// be tried again.
const RELEASED: &str = "released";

/// A read-receipt request this engine would answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReceiptRequest {
    /// The one mailbox the request names.
    pub to: String,
    /// Whether that mailbox is the message's `Return-Path`.
    pub same_as_sender: bool,
    /// Whether the message is list or bulk mail.
    pub from_list: bool,
}

/// Why a message carries no request that can be answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoRequest {
    /// No `Disposition-Notification-To` naming exactly one acceptable mailbox.
    Absent,
    /// The message is itself a report (`multipart/report`).
    Report,
    /// The message's `Return-Path` is the null path.
    NullReturnPath,
}

impl NoRequest {
    fn description(self) -> &'static str {
        match self {
            Self::Absent => "the message does not request a read receipt",
            Self::Report => "the message is itself a report; a report is not answered",
            Self::NullReturnPath => {
                "the message has a null return path; a notification is not answered"
            }
        }
    }
}

/// Read the receipt request of a stored message.
///
/// The address comes from `mw-mime`'s reader, which yields one only for a
/// single header naming a single mailbox, and is checked again with
/// `mw_smtp::validate_mailbox` because it becomes an SMTP recipient.
pub(crate) fn receipt_request(raw: &[u8]) -> std::result::Result<ReceiptRequest, NoRequest> {
    let receipt = mw_mime::parse(raw)
        .map(|p| p.receipt)
        .map_err(|_| NoRequest::Absent)?;
    let to = receipt
        .disposition_notification_to
        .filter(|addr| mw_smtp::validate_mailbox(addr).is_ok())
        .ok_or(NoRequest::Absent)?;
    if is_report(raw) {
        return Err(NoRequest::Report);
    }
    if receipt.return_path.as_deref() == Some("") {
        return Err(NoRequest::NullReturnPath);
    }
    Ok(ReceiptRequest {
        same_as_sender: receipt
            .return_path
            .as_deref()
            .is_some_and(|path| same_mailbox(path, &to)),
        from_list: receipt.from_list,
        to,
    })
}

/// Whether the message's top-level type is `multipart/report` (a delivery
/// status or disposition notification, RFC 6522).
fn is_report(raw: &[u8]) -> bool {
    MessageParser::default().parse(raw).is_some_and(|m| {
        m.root_part().content_type().is_some_and(|ct| {
            ct.ctype().eq_ignore_ascii_case("multipart")
                && ct
                    .subtype()
                    .is_some_and(|s| s.eq_ignore_ascii_case("report"))
        })
    })
}

/// Whether two addresses name the same mailbox: the local parts are equal and
/// the domains are equal without regard to ASCII case.
fn same_mailbox(a: &str, b: &str) -> bool {
    match (a.rsplit_once('@'), b.rsplit_once('@')) {
        (Some((la, da)), Some((lb, db))) => la == lb && da.eq_ignore_ascii_case(db),
        _ => false,
    }
}

/// The `mailwomanMdn` and `mailwomanMdnReport` values of `Email/get` for one
/// message, from its raw bytes.
///
/// The first is `None` unless the message carries a request this engine would
/// answer ([`receipt_request`]); the caller adds `sent` to it
/// ([`Engine::mdn_sent`]). The second is `null` unless the message is a
/// disposition notification `mw_mime::parse_mdn` reads. Every string in the
/// report was written by whoever sent it.
pub(crate) fn email_properties(raw: &[u8]) -> (Option<Value>, Value) {
    let request = receipt_request(raw).ok().map(|r| {
        json!({
            "requestedBy": r.to,
            "sameAsSender": r.same_as_sender,
            "fromList": r.from_list,
        })
    });
    let report = match mw_mime::parse_mdn(raw) {
        Some(r) => json!({
            "originalMessageId": r.original_message_id,
            "finalRecipient": r.final_recipient,
            "disposition": r.disposition.as_str(),
        }),
        None => Value::Null,
    };
    (request, report)
}

/// Whether `flags_json` holds the `$mdnsent` keyword (in any case: IMAP
/// servers spell it `$MDNSent`).
pub(crate) fn has_sent_keyword(flags_json: &str) -> bool {
    flags_from_json(flags_json)
        .iter()
        .any(|f| matches!(f, Flag::Keyword(k) if k.eq_ignore_ascii_case(MDN_SENT_KEYWORD)))
}

fn ledger_key(account_id: &str, email_id: &str) -> String {
    format!("mdn_sent:{account_id}:{email_id}")
}

/// Email ids with an `MDN/send` in progress in this process.
fn sends_in_flight() -> &'static Mutex<HashSet<String>> {
    static IN_FLIGHT: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    IN_FLIGHT.get_or_init(Default::default)
}

/// Holds an email id in [`sends_in_flight`] until dropped.
struct InFlight(String);

impl InFlight {
    fn claim(email_id: &str) -> Option<Self> {
        sends_in_flight()
            .lock()
            .expect("mdn in-flight lock")
            .insert(email_id.to_string())
            .then(|| Self(email_id.to_string()))
    }
}

impl Drop for InFlight {
    fn drop(&mut self) {
        sends_in_flight()
            .lock()
            .expect("mdn in-flight lock")
            .remove(&self.0);
    }
}

fn refusal(kind: &str, description: impl Into<String>) -> Value {
    json!({ "type": kind, "description": description.into() })
}

impl Engine {
    /// The values the read-receipt policy can take. `ask` is the default.
    pub const MDN_POLICIES: [&'static str; 3] = ["ask", "never", "always"];

    /// The `settings` key holding one account's read-receipt policy.
    pub fn mdn_policy_key(account_id: &str) -> String {
        format!("mdn_policy:{account_id}")
    }

    /// The account's read-receipt policy: one of [`Engine::MDN_POLICIES`].
    /// Nothing stored, or a stored value that is none of them, reads as `ask`.
    pub async fn mdn_policy(&self, account_id: &str) -> Result<&'static str> {
        let stored = self
            .store()
            .get_setting(&Self::mdn_policy_key(account_id))
            .await?;
        Ok(Self::MDN_POLICIES
            .into_iter()
            .find(|p| stored.as_deref() == Some(p))
            .unwrap_or("ask"))
    }

    /// Whether a receipt has been sent, or claimed, for a message: the
    /// `$mdnsent` keyword, or a ledger entry that was not released. The ledger
    /// is what survives a flag sync or a keyword replacement that drops the
    /// keyword.
    pub(crate) async fn mdn_sent(
        &self,
        account_id: &str,
        email_id: &str,
        flags_json: &str,
    ) -> Result<bool> {
        if has_sent_keyword(flags_json) {
            return Ok(true);
        }
        Ok(self
            .store()
            .get_setting(&ledger_key(account_id, email_id))
            .await?
            .is_some_and(|v| v != RELEASED))
    }

    /// `MDN/send { emailId, automatic? }`: send the read receipt a stored
    /// message asks for. The module documentation lists the rules.
    ///
    /// Answers `{ accountId, emailId, sent: true, to, automatic }`, or an
    /// error object whose `type` is one of `invalidArguments`, `notFound`,
    /// `mdnNotRequested`, `mdnNotAllowed`, `mdnAlreadySent`,
    /// `mdnAutomaticRefused`, `mdnNoRecipientAddress`, `mdnNotSent`,
    /// `accountNotFound`, `serverFail`.
    pub(crate) async fn mdn_send(&self, account_id: &str, args: &Value) -> Value {
        let Some(email_id) = args.get("emailId").and_then(Value::as_str) else {
            return refusal("invalidArguments", "emailId must be the id of an Email");
        };
        let automatic = match args.get("automatic") {
            None | Some(Value::Null) => false,
            Some(Value::Bool(b)) => *b,
            Some(_) => return refusal("invalidArguments", "automatic must be true or false"),
        };
        let Some(rt) = self.runtime(account_id) else {
            return refusal("accountNotFound", "account is not connected in engine mode");
        };

        let msg = match self.store().get_message(email_id).await {
            Ok(m) if m.account_id == account_id => m,
            Ok(_) | Err(mw_store::StoreError::NotFound) => {
                return refusal("notFound", "no such Email in this account");
            }
            Err(e) => return server_fail(e),
        };
        let raw = match self.fetch_blob(account_id, email_id).await {
            Ok(Some(blob)) => blob.bytes,
            Ok(None) => return refusal("notFound", "the Email has no stored message"),
            Err(e) => return server_fail(e),
        };
        let request = match receipt_request(&raw) {
            Ok(r) => r,
            Err(why) => return refusal("mdnNotRequested", why.description()),
        };

        let role = match self.store().get_mailbox(&msg.mailbox_id).await {
            Ok(mb) => mb.role,
            Err(e) => return server_fail(e),
        };
        let is_draft = flags_from_json(&msg.flags_json).contains(&Flag::Draft);
        if is_draft
            || role
                .as_deref()
                .is_some_and(|r| NO_RECEIPT_ROLES.contains(&r))
        {
            return refusal(
                "mdnNotAllowed",
                "no receipt is sent for a draft or for a message in Drafts, Sent, Junk or Trash",
            );
        }

        match self.mdn_sent(account_id, email_id, &msg.flags_json).await {
            Ok(false) => {}
            Ok(true) => return already_sent(),
            Err(e) => return server_fail(e),
        }

        if automatic {
            let policy = match self.mdn_policy(account_id).await {
                Ok(p) => p,
                Err(e) => return server_fail(e),
            };
            let why = if policy != "always" {
                Some("the account's read-receipt policy is not \"always\"")
            } else if !request.same_as_sender {
                Some("the request address is not the message's return path")
            } else if request.from_list {
                Some("the message is list or bulk mail")
            } else {
                None
            };
            if let Some(why) = why {
                return refusal(
                    "mdnAutomaticRefused",
                    format!("not sent without the user's confirmation: {why}"),
                );
            }
        }

        let parsed = mw_mime::parse(&raw).map(|p| p.email).unwrap_or_default();
        let Some(own_address) = self
            .addressed_own_address(account_id, &rt.identity, &parsed)
            .await
        else {
            return refusal(
                "mdnNoRecipientAddress",
                "the message's To and Cc name no address of this account, so there is no \
                 address a receipt could be sent from",
            );
        };

        // One sender per message: in this process by the in-flight set, across
        // processes and restarts by the ledger row.
        let Some(_in_flight) = InFlight::claim(email_id) else {
            return already_sent();
        };
        let ledger = ledger_key(account_id, email_id);
        match self.claim_ledger(&ledger).await {
            Ok(true) => {}
            Ok(false) => return already_sent(),
            Err(e) => return server_fail(e),
        }
        // `$mdnsent` before the submitter is called. A server that refuses the
        // keyword does not stop the receipt: the ledger row already records it.
        if let Err(why) = self
            .set_sent_keyword(account_id, email_id, json!(true))
            .await
        {
            tracing::warn!("could not set {MDN_SENT_KEYWORD} on {email_id}: {why}");
        }

        let outcome = self
            .transmit_mdn(&rt, &request, &own_address, &msg, &parsed, automatic)
            .await;
        match outcome {
            Ok(()) => {
                let stamp = format!("sent:{}", chrono::Utc::now().to_rfc3339());
                if let Err(e) = self.store().set_setting(&ledger, &stamp).await {
                    // The claim stays, which reads as sent.
                    tracing::warn!("could not stamp the receipt ledger for {email_id}: {e}");
                }
                json!({
                    "accountId": account_id,
                    "emailId": email_id,
                    "sent": true,
                    "to": request.to,
                    "automatic": automatic,
                })
            }
            Err(e) => {
                if let Err(why) = self
                    .set_sent_keyword(account_id, email_id, Value::Null)
                    .await
                {
                    tracing::warn!("could not clear {MDN_SENT_KEYWORD} on {email_id}: {why}");
                }
                if let Err(e) = self.store().set_setting(&ledger, RELEASED).await {
                    tracing::warn!("could not release the receipt claim for {email_id}: {e}");
                }
                refusal("mdnNotSent", format!("the receipt was not sent: {e}"))
            }
        }
    }

    /// Take the ledger row for one message: written only when the row is
    /// absent or holds a released claim, each in one statement.
    async fn claim_ledger(&self, key: &str) -> Result<bool> {
        let claim = format!("claimed:{}", chrono::Utc::now().to_rfc3339());
        if self
            .store()
            .compare_and_set_setting(key, None, &claim)
            .await?
        {
            return Ok(true);
        }
        Ok(self
            .store()
            .compare_and_set_setting(key, Some(RELEASED), &claim)
            .await?)
    }

    /// Set (`true`) or clear (`null`) `$mdnsent` through `Email/set`, so the
    /// flag reaches the store, the upstream server, the search index and the
    /// change log the way any keyword change does.
    async fn set_sent_keyword(
        &self,
        account_id: &str,
        email_id: &str,
        value: Value,
    ) -> std::result::Result<(), String> {
        let patch = json!({ format!("keywords/{MDN_SENT_KEYWORD}"): value });
        let request = json!({
            "methodCalls": [["Email/set", { "update": { email_id: patch } }, "mdn"]]
        });
        // `handle_jmap` is what dispatched this method; the nested call is boxed.
        let response = Box::pin(self.handle_jmap(account_id, &request)).await;
        let result = &response["methodResponses"][0][1];
        if result["updated"]
            .as_object()
            .is_some_and(|u| u.contains_key(email_id))
        {
            Ok(())
        } else {
            Err(result.to_string())
        }
    }

    /// The address of this account the original was sent to: the first of the
    /// connected identity and the stored sending identities that appears in the
    /// original's `To` or `Cc` (compared without regard to ASCII case), is a
    /// mailbox, and is ASCII (`mw-mime` writes `Final-Recipient` as `rfc822`).
    /// The value returned is the account's own spelling, not the message's.
    async fn addressed_own_address(
        &self,
        account_id: &str,
        identity: &str,
        original: &mw_jmap::Email,
    ) -> Option<String> {
        let mut own = vec![identity.to_string()];
        match self.store().list_identities(account_id).await {
            Ok(rows) => own.extend(rows.into_iter().map(|r| r.email)),
            Err(e) => tracing::warn!("reading identities for a receipt failed: {e}"),
        }
        let addressed: Vec<&str> = [&original.to, &original.cc]
            .into_iter()
            .flatten()
            .flatten()
            .map(|a| a.email.as_str())
            .collect();
        own.into_iter().find(|mine| {
            mine.is_ascii()
                && mw_smtp::validate_mailbox(mine).is_ok()
                && addressed.iter().any(|a| a.eq_ignore_ascii_case(mine))
        })
    }

    /// Build the report and hand it to the account's submitter with the null
    /// reverse path and the request address as the only recipient.
    async fn transmit_mdn(
        &self,
        rt: &crate::account::AccountRuntime,
        request: &ReceiptRequest,
        own_address: &str,
        msg: &mw_store::Message,
        original: &mw_jmap::Email,
        automatic: bool,
    ) -> Result<()> {
        let report = mw_mime::build_mdn(&MdnInput {
            from: EmailAddress {
                name: None,
                email: own_address.to_string(),
            },
            to: request.to.clone(),
            final_recipient: own_address.to_string(),
            original_recipient: None,
            original_message_id: msg
                .message_id
                .as_deref()
                .map(|id| id.trim().trim_start_matches('<').trim_end_matches('>'))
                .filter(|id| writable_message_id(id))
                .map(String::from),
            original_subject: original.subject.clone(),
            // The message was displayed because the user opened it; what can be
            // automatic is the sending of the report.
            action_mode: MdnActionMode::Manual,
            sending_mode: if automatic {
                MdnSendingMode::Automatic
            } else {
                MdnSendingMode::Manual
            },
            disposition: MdnDisposition::Displayed,
            message_id: Some(crate::jmap::gen_message_id()),
        })
        .map_err(|e| EngineError::Protocol(e.to_string()))?;

        let outgoing = mw_smtp::Outgoing {
            mail_from: String::new(),
            rcpt_to: vec![request.to.clone()],
            raw: report,
        };
        outgoing
            .validate()
            .map_err(|e| EngineError::Protocol(e.to_string()))?;
        let result = rt.submitter.submit(outgoing).await?;
        if result.accepted.is_empty() {
            return Err(EngineError::Protocol(format!(
                "the recipient was rejected: {:?}",
                result.rejected
            )));
        }
        Ok(())
    }
}

fn already_sent() -> Value {
    refusal(
        "mdnAlreadySent",
        "a receipt has already been sent for this message",
    )
}

/// Whether `id` (without angle brackets) is one `mw_mime::build_mdn` writes as
/// `Original-Message-ID`; any other id is left out of the report rather than
/// failing it.
fn writable_message_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 800
        && id
            .bytes()
            .all(|b| b.is_ascii_graphic() && b != b'<' && b != b'>')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mailbox_comparison_ignores_case_in_the_domain_only() {
        assert!(same_mailbox("ann@Example.ORG", "ann@example.org"));
        assert!(!same_mailbox("Ann@example.org", "ann@example.org"));
        assert!(!same_mailbox("ann@example.org", "ann@example.net"));
        assert!(!same_mailbox("", "ann@example.org"));
        assert!(!same_mailbox("ann", "ann"));
    }

    #[test]
    fn the_sent_keyword_is_read_in_any_case() {
        assert!(has_sent_keyword(r#"["Seen",{"Keyword":"$MDNSent"}]"#));
        assert!(has_sent_keyword(r#"[{"Keyword":"$mdnsent"}]"#));
        assert!(!has_sent_keyword(r#"["Seen",{"Keyword":"$forwarded"}]"#));
        assert!(!has_sent_keyword("not json"));
    }
}
