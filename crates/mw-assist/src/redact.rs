//! Redaction — the safety-critical step (plan §14, §6 R4) that guarantees
//! E2EE-decrypted content and attachments **never** reach an adapter payload unless
//! explicitly granted, and that context outside the data-class ceiling is dropped.
//!
//! [`redact_chat`] is the ONLY path from mailbox context to an outbound
//! [`ChatPayload`]. It is `pub` so tests can assert the built payload directly (the
//! acceptance requires: "E2EE-decrypted content is never in an outbound payload by
//! default (asserted)").

use crate::adapters::ChatPayload;
use crate::{AssistCapability, AssistInput, ContentKind, ContextItem, DataScope};

/// What was dropped during redaction (for observability + tests). Carries **counts
/// only**, never content.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RedactionReport {
    /// Items dropped because they fell outside the data-class ceiling.
    pub dropped_scope: usize,
    /// E2EE-decrypted items dropped (not granted).
    pub dropped_e2ee: usize,
    /// Attachment items dropped (not granted).
    pub dropped_attachment: usize,
    /// Items kept and forwarded.
    pub kept: usize,
}

/// Build the outbound [`ChatPayload`] from `input`, applying — in order — the
/// data-class ceiling, then E2EE/attachment exclusion. The user's own `prompt` is
/// always forwarded (it is not mailbox content); only `context` items are subject
/// to redaction.
#[must_use]
pub fn redact_chat(input: &AssistInput, eff: &DataScope, cap: AssistCapability) -> ChatPayload {
    redact_chat_reported(input, eff, cap).0
}

/// [`redact_chat`] plus the [`RedactionReport`] (for tests / observability).
#[must_use]
pub fn redact_chat_reported(
    input: &AssistInput,
    eff: &DataScope,
    cap: AssistCapability,
) -> (ChatPayload, RedactionReport) {
    let mut report = RedactionReport::default();
    let mut kept: Vec<String> = Vec::new();

    for item in &input.context {
        if !within_ceiling(item, eff) {
            report.dropped_scope += 1;
            continue;
        }
        match item.kind {
            ContentKind::E2eeDecrypted if !eff.include_e2ee => {
                report.dropped_e2ee += 1;
                continue;
            }
            ContentKind::Attachment if !eff.include_attachments => {
                report.dropped_attachment += 1;
                continue;
            }
            _ => {}
        }
        report.kept += 1;
        kept.push(item.text.clone());
    }

    let prompt = if kept.is_empty() {
        input.prompt.clone()
    } else {
        format!("{}\n\n{}", input.prompt, kept.join("\n\n"))
    };

    (
        ChatPayload {
            system: Some(cap.system_prompt().to_string()),
            prompt,
        },
        report,
    )
}

/// Whether a context item is within the (already clamped) data-class ceiling.
/// Accounts are a strict allowlist (empty ⇒ none). Folders follow "empty ⇒ all".
fn within_ceiling(item: &ContextItem, eff: &DataScope) -> bool {
    if !eff.accounts.contains(&item.account) {
        return false;
    }
    if !eff.folders.is_empty() && !eff.folders.contains(&item.folder) {
        return false;
    }
    true
}
