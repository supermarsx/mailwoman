//! Bounded, account-owned evidence for automatic read receipts.
use super::*;
use mw_store::receipts::{SenderDecision, normalize_sender};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Action {
    None,
    Ask,
    Automatic,
}

fn choose(
    mode: &str,
    exception: Option<SenderDecision>,
    contact: bool,
    organisation: bool,
) -> Action {
    if mode == "never" || exception == Some(SenderDecision::Never) {
        return Action::None;
    }
    if exception == Some(SenderDecision::Always) {
        return Action::Automatic;
    }
    match mode {
        "always" => Action::Automatic,
        "ask" => Action::Ask,
        "contactsOnly" if contact => Action::Automatic,
        "orgOnly" | "orgAlwaysAskOthers" if organisation => Action::Automatic,
        "orgAlwaysAskOthers" => Action::Ask,
        _ => Action::None,
    }
}

impl Engine {
    pub(super) async fn receipt_policy_action(
        &self,
        account: &str,
        request: &ReceiptRequest,
    ) -> Result<Action> {
        let mode = self.mdn_policy(account).await?;
        // Held administrator policies ignore personal exceptions entirely.
        let exception = if self.held_mdn_policy(account).await?.is_none() {
            self.store()
                .receipt_sender_decision(account, &request.to)
                .await?
        } else {
            None
        };
        let contact = if mode == "contactsOnly" {
            let canonical = normalize_sender(&request.to)?;
            let rows = self
                .store()
                .receipt_contact_cards(account, &request.to)
                .await?;
            rows.into_iter().any(|raw| {
                raw.len() <= 64 * 1024
                    && mw_ics::parse_vcard(raw.as_bytes()).is_ok_and(|cards| {
                        cards.iter().any(|card| {
                            card.json["emails"].as_array().is_some_and(|emails| {
                                emails
                                    .iter()
                                    .filter_map(|email| email["value"].as_str())
                                    .any(|email| {
                                        normalize_sender(email)
                                            .is_ok_and(|value| value == canonical)
                                    })
                            })
                        })
                    })
            })
        } else {
            false
        };
        let organisation = if matches!(mode, "orgOnly" | "orgAlwaysAskOthers") {
            let username = self.store().get_account(account).await?.username;
            if let (Some((_, own)), Some((_, sender))) =
                (username.rsplit_once('@'), request.to.rsplit_once('@'))
            {
                let own = mw_store::settings_policy::normalize_policy_domain(own)?;
                let sender = mw_store::settings_policy::normalize_policy_domain(sender)?;
                let own = self.store().domain_alias_target(&own).await?.unwrap_or(own);
                let sender = self
                    .store()
                    .domain_alias_target(&sender)
                    .await?
                    .unwrap_or(sender);
                own == sender
            } else {
                false
            }
        } else {
            false
        };
        let action = choose(mode, exception, contact, organisation);
        Ok(
            if action == Action::Automatic && (!request.same_as_sender || request.from_list) {
                Action::Ask
            } else {
                action
            },
        )
    }

    /// Read-only eligibility; the actual send repeats authority and once-only checks.
    /// Missing, foreign, uncached or oversized messages reveal no receipt metadata.
    pub async fn receipt_response_decision(
        &self,
        account: &str,
        email_id: &str,
    ) -> Result<Option<Value>> {
        let msg = match self.store().get_message(email_id).await {
            Ok(msg) if msg.account_id == account => msg,
            Ok(_) | Err(mw_store::StoreError::NotFound) => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let Some(raw) = self
            .stored_raw_limited(account, email_id, 8 * 1024 * 1024)
            .await?
        else {
            return Ok(None);
        };
        let Some(rt) = self.runtime(account) else {
            return Ok(None);
        };
        let request = match receipt_request(&raw) {
            Ok(request) => request,
            Err(_) => return Ok(None),
        };
        let handled = self.mdn_sent(account, email_id, &msg.flags_json).await?;
        let mailbox = self.store().get_mailbox(&msg.mailbox_id).await?;
        let parsed = mw_mime::parse(&raw)
            .map(|message| message.email)
            .unwrap_or_default();
        let forbidden = handled
            || flags_from_json(&msg.flags_json).contains(&Flag::Draft)
            || mailbox
                .role
                .as_deref()
                .is_some_and(|role| NO_RECEIPT_ROLES.contains(&role))
            || !self.mdn_sending_allowed(account).await?
            || self
                .addressed_own_address(account, &rt.identity, &parsed)
                .await
                .is_none();
        let action = if forbidden {
            Action::None
        } else {
            self.receipt_policy_action(account, &request).await?
        };
        Ok(Some(
            json!({"accountId": account, "emailId": email_id, "requestedBy": request.to,
            "handled": handled, "action": match action { Action::None => "none", Action::Ask => "ask", Action::Automatic => "automatic" }}),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn policies_and_exceptions_have_explicit_precedence() {
        assert_eq!(
            choose("never", Some(SenderDecision::Always), true, true),
            Action::None
        );
        assert_eq!(
            choose("always", Some(SenderDecision::Never), true, true),
            Action::None
        );
        assert_eq!(
            choose("ask", Some(SenderDecision::Always), false, false),
            Action::Automatic
        );
        for (mode, contact, org, expected) in [
            ("ask", false, false, Action::Ask),
            ("always", false, false, Action::Automatic),
            ("contactsOnly", false, true, Action::None),
            ("contactsOnly", true, false, Action::Automatic),
            ("orgOnly", true, false, Action::None),
            ("orgOnly", false, true, Action::Automatic),
            ("orgAlwaysAskOthers", false, false, Action::Ask),
            ("orgAlwaysAskOthers", false, true, Action::Automatic),
            ("invalid", true, true, Action::None),
        ] {
            assert_eq!(choose(mode, None, contact, org), expected);
        }
    }
}
