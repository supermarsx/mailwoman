//! RFC 5230/3834 automatic-response safety and persistent duplicate suppression.
use super::*;
use sha2::{Digest, Sha256};

pub(super) fn permitted(
    message: &Message,
    email: &mw_jmap::Email,
    identity: &str,
    vacation: &mw_sieve::local::Vacation,
) -> Option<String> {
    let sender = message.envelope_from.as_deref()?.trim();
    if mw_smtp::validate_mailbox(sender).is_err()
        || sender.eq_ignore_ascii_case(identity)
        || vacation
            .addresses
            .iter()
            .any(|a| a.eq_ignore_ascii_case(sender))
    {
        return None;
    }
    let local = sender.split('@').next()?.to_ascii_lowercase();
    if ["mailer-daemon", "postmaster", "noreply", "no-reply"].contains(&local.as_str())
        || local.starts_with("owner-")
        || local.ends_with("-request")
    {
        return None;
    }
    for (name, value) in &message.headers {
        let name = name.to_ascii_lowercase();
        if name.starts_with("list-")
            || matches!(name.as_str(), "mailing-list" | "x-mailwoman-loop")
            || name == "auto-submitted" && !value.trim().eq_ignore_ascii_case("no")
            || name == "precedence"
                && ["bulk", "junk", "list"].contains(&value.trim().to_ascii_lowercase().as_str())
            || name == "x-auto-response-suppress" && !value.trim().eq_ignore_ascii_case("none")
        {
            return None;
        }
    }
    let personal = email.to.iter().chain(email.cc.iter()).flatten().any(|a| {
        a.email.eq_ignore_ascii_case(identity)
            || vacation
                .addresses
                .iter()
                .any(|own| own.eq_ignore_ascii_case(&a.email))
    });
    personal.then(|| sender.to_string())
}

impl Engine {
    pub(super) async fn send_local_vacation(
        &self,
        account: &str,
        rt: &crate::account::AccountRuntime,
        message: &Message,
        email: &mw_jmap::Email,
        vacation: &mw_sieve::local::Vacation,
    ) -> Result<()> {
        let Some(sender) = permitted(message, email, &rt.identity, vacation) else {
            return Ok(());
        };
        let from = vacation.from.as_deref().unwrap_or(&rt.identity);
        // The engine has one verified submission identity. Do not let an imported
        // raw script turn the automatic responder into an identity-spoofing relay.
        if !from.eq_ignore_ascii_case(&rt.identity) {
            return Err(EngineError::Unsupported(
                "vacation From must match the account identity".into(),
            ));
        }
        let subject = vacation.subject.clone().unwrap_or_else(|| {
            format!(
                "Auto: {}",
                email.subject.as_deref().unwrap_or("Vacation reply")
            )
        });
        let parent_id = email.message_id.as_ref().and_then(|v| v.first()).cloned();
        let request = mw_mime::ComposeRequest {
            from: Some(mw_jmap::EmailAddress {
                name: None,
                email: from.into(),
            }),
            to: vec![mw_jmap::EmailAddress {
                name: None,
                email: sender.clone(),
            }],
            subject: Some(subject),
            text_body: Some(vacation.reason.clone()),
            in_reply_to: parent_id.clone(),
            references: parent_id.into_iter().collect(),
            headers: vec![
                ("Auto-Submitted".into(), "auto-replied".into()),
                ("X-Mailwoman-Loop".into(), "vacation".into()),
            ],
            ..Default::default()
        };
        let mut raw = mw_mime::build_with(&request, &Default::default())
            .map_err(|e| EngineError::Protocol(e.to_string()))?;
        if vacation.mime {
            let reason = vacation.reason.replace("\r\n", "\n").replace('\r', "\n");
            let (headers, body) = reason.split_once("\n\n").ok_or_else(|| {
                EngineError::Protocol("vacation MIME entity has no body separator".into())
            })?;
            let mut previous = false;
            for line in headers.lines() {
                if line.starts_with([' ', '\t']) && previous {
                    continue;
                }
                let Some((name, _)) = line.split_once(':') else {
                    return Err(EngineError::Protocol(
                        "invalid vacation MIME headers".into(),
                    ));
                };
                if !name.to_ascii_lowercase().starts_with("content-")
                    && !name.eq_ignore_ascii_case("mime-version")
                {
                    return Err(EngineError::Protocol(
                        "vacation MIME entity contains a message header".into(),
                    ));
                }
                previous = true;
            }
            let text = String::from_utf8(raw)
                .map_err(|_| EngineError::Protocol("invalid composed vacation".into()))?;
            let outer = text
                .split_once("\r\n\r\n")
                .map(|(h, _)| h)
                .ok_or_else(|| EngineError::Protocol("invalid composed vacation headers".into()))?;
            let mut kept = String::new();
            let mut skip = false;
            for line in outer.split("\r\n") {
                if !line.starts_with([' ', '\t']) {
                    skip = line.to_ascii_lowercase().starts_with("content-")
                        || line.to_ascii_lowercase().starts_with("mime-version:");
                }
                if !skip {
                    kept.push_str(line);
                    kept.push_str("\r\n");
                }
            }
            raw = format!(
                "{kept}MIME-Version: 1.0\r\n{}\r\n\r\n{}",
                headers.replace('\n', "\r\n"),
                body.replace('\n', "\r\n")
            )
            .into_bytes();
        }
        let mut digest = Sha256::new();
        digest.update(sender.to_ascii_lowercase());
        digest.update([0]);
        if let Some(handle) = &vacation.handle {
            digest.update(handle);
        } else {
            for field in [
                vacation.reason.as_str(),
                vacation.subject.as_deref().unwrap_or(""),
                from,
            ] {
                digest.update(field.as_bytes());
                digest.update([0]);
            }
            digest.update([vacation.mime as u8]);
        }
        let fingerprint: String = digest
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        if !self
            .store()
            .claim_sieve_vacation(
                account,
                &fingerprint,
                chrono::Utc::now().timestamp(),
                vacation.seconds,
            )
            .await?
        {
            return Ok(());
        }
        let outgoing = mw_smtp::Outgoing {
            mail_from: String::new(),
            rcpt_to: vec![sender],
            raw,
        };
        outgoing
            .validate()
            .map_err(|e| EngineError::Protocol(e.to_string()))?;
        let result = rt.submitter.submit(outgoing).await?;
        if result.accepted.is_empty() {
            return Err(EngineError::Protocol(
                "vacation response was refused".into(),
            ));
        }
        Ok(())
    }
}
