//! Row ↔ DTO conversions for the crypto/security surface (plan §2.1/§2.4). Keeps
//! the store seam (opaque primitives) and the frozen §2.1 camelCase DTOs in one
//! place so the keyring, the mail-rule surface, and the mock-parity golden test
//! all project identically (the V2/V3 parity discipline).

use mw_sieve::{Action, Condition, MatchOp, Rule, StringTest};
use mw_store::CryptoKeyRow;

use super::types::{CryptoKey, KeyHistoryEntry, MailRule, MailRuleAction, MailRuleCondition};

// ── CryptoKey ────────────────────────────────────────────────────────────────

/// Project a stored [`CryptoKeyRow`] to the frozen §2.1 [`CryptoKey`] DTO.
/// `hasPrivate` is derived from ownership (the private material itself lives only
/// in the client vault — plan §1.2); `encryptedPrivateBackup` is the opaque
/// client blob decoded as UTF-8 text (it is base64/armored, never binary secret).
pub(crate) fn key_row_to_dto(row: &CryptoKeyRow) -> CryptoKey {
    CryptoKey {
        id: row.id.clone(),
        kind: row.kind.clone(),
        is_own: row.is_own,
        addresses: serde_json::from_str(&row.addresses_json).unwrap_or_default(),
        fingerprint: row.fingerprint.clone(),
        key_id: row.key_id.clone(),
        algorithm: row.algorithm.clone(),
        created_at: row.created_at.clone(),
        expires_at: row.expires_at.clone(),
        public_key_armored: row.public_key.clone(),
        cert_pem: row.cert_pem.clone(),
        trust: row.trust.clone(),
        autocrypt: row.autocrypt,
        source: row.source.clone(),
        has_private: row.is_own,
        encrypted_private_backup: row
            .encrypted_private_backup
            .as_ref()
            .map(|b| String::from_utf8_lossy(b).into_owned()),
        verified_at: row.verified_at.clone(),
        key_history: serde_json::from_str(&row.key_history_json).unwrap_or_default(),
    }
}

/// Build a [`CryptoKeyRow`] from an uploaded/looked-up [`CryptoKey`] DTO. The
/// opaque `encryptedPrivateBackup` string is stored verbatim as bytes — NEVER
/// decrypted (plan §1.2 / risk #4).
pub(crate) fn key_dto_to_row(account_id: &str, dto: &CryptoKey) -> CryptoKeyRow {
    CryptoKeyRow {
        id: dto.id.clone(),
        account_id: account_id.to_string(),
        kind: dto.kind.clone(),
        is_own: dto.is_own,
        addresses_json: serde_json::to_string(&dto.addresses).unwrap_or_else(|_| "[]".into()),
        fingerprint: dto.fingerprint.clone(),
        key_id: dto.key_id.clone(),
        algorithm: dto.algorithm.clone(),
        created_at: dto.created_at.clone(),
        expires_at: dto.expires_at.clone(),
        public_key: dto.public_key_armored.clone(),
        cert_pem: dto.cert_pem.clone(),
        trust: dto.trust.clone(),
        autocrypt: dto.autocrypt,
        source: dto.source.clone(),
        encrypted_private_backup: dto
            .encrypted_private_backup
            .as_ref()
            .map(|s| s.clone().into_bytes()),
        verified_at: dto.verified_at.clone(),
        key_history_json: serde_json::to_string(&dto.key_history).unwrap_or_else(|_| "[]".into()),
    }
}

/// A one-entry key history for a freshly-seen fingerprint (TOFU first-seen).
pub(crate) fn initial_history(fingerprint: &str, seen_at: &str) -> Vec<KeyHistoryEntry> {
    vec![KeyHistoryEntry {
        fingerprint: fingerprint.to_string(),
        seen_at: seen_at.to_string(),
    }]
}

// ── MailRule ↔ mw_sieve::Rule ────────────────────────────────────────────────

fn matchop_to_str(op: MatchOp) -> &'static str {
    match op {
        MatchOp::Is => "is",
        MatchOp::Contains | MatchOp::Matches => "contains",
    }
}

fn str_to_matchop(op: &str) -> MatchOp {
    match op {
        "is" => MatchOp::Is,
        _ => MatchOp::Contains,
    }
}

/// Project a stored [`Rule`] to the frozen §2.1 [`MailRule`] DTO. Only the
/// from/to/subject conditions + move/tag/stop/archive actions have a MailRule
/// surface; other Sieve constructs are omitted (the block/silence/ignore surface
/// only needs these — plan §1.9). `runsAt` is `"engine"` (the always-green path).
pub(crate) fn rule_to_mail_rule(rule: &Rule) -> MailRule {
    let conditions = rule
        .conditions
        .iter()
        .filter_map(|c| match c {
            Condition::From(t) => Some(("from", t)),
            Condition::To(t) => Some(("to", t)),
            Condition::Subject(t) => Some(("subject", t)),
            _ => None,
        })
        .map(|(kind, t)| MailRuleCondition {
            kind: kind.to_string(),
            op: matchop_to_str(t.op).to_string(),
            value: t.value.clone(),
        })
        .collect();
    let actions = rule
        .actions
        .iter()
        .filter_map(|a| match a {
            Action::Move { mailbox } if mailbox.eq_ignore_ascii_case("archive") => {
                Some(MailRuleAction {
                    kind: "archive".into(),
                    value: None,
                })
            }
            Action::Move { mailbox } => Some(MailRuleAction {
                kind: "move".into(),
                value: Some(mailbox.clone()),
            }),
            Action::Tag { keyword } | Action::Mark { keyword } => Some(MailRuleAction {
                kind: "tag".into(),
                value: Some(keyword.clone()),
            }),
            Action::Stop => Some(MailRuleAction {
                kind: "stop".into(),
                value: None,
            }),
            _ => None,
        })
        .collect();
    MailRule {
        id: rule.id.clone(),
        name: rule.name.clone(),
        match_all: rule.match_all,
        conditions,
        actions,
        enabled: rule.enabled,
        runs_at: "engine".into(),
    }
}

/// Build a [`Rule`] from a [`MailRule`] DTO (the reverse of [`rule_to_mail_rule`]).
/// `suppressNotify` has no Sieve action (silence is an engine-side flag, plan
/// §1.9) and is dropped from the generated rule.
pub(crate) fn mail_rule_to_rule(mr: &MailRule) -> Rule {
    let conditions = mr
        .conditions
        .iter()
        .filter_map(|c| {
            let test = StringTest {
                op: str_to_matchop(&c.op),
                value: c.value.clone(),
            };
            match c.kind.as_str() {
                "from" => Some(Condition::From(test)),
                "to" => Some(Condition::To(test)),
                "subject" => Some(Condition::Subject(test)),
                _ => None,
            }
        })
        .collect();
    let actions = mr
        .actions
        .iter()
        .filter_map(|a| match a.kind.as_str() {
            "move" => Some(Action::Move {
                mailbox: a.value.clone().unwrap_or_default(),
            }),
            "archive" => Some(Action::Move {
                mailbox: "Archive".into(),
            }),
            "tag" => Some(Action::Tag {
                keyword: a.value.clone().unwrap_or_default(),
            }),
            "stop" => Some(Action::Stop),
            _ => None,
        })
        .collect();
    Rule {
        id: mr.id.clone(),
        name: mr.name.clone(),
        match_all: mr.match_all,
        conditions,
        actions,
        enabled: mr.enabled,
    }
}
