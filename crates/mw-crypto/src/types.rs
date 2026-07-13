//! Frozen §2.1 crypto/security DTOs (Mailwoman-native, camelCase). These are the
//! contract every parallel V4 builder and the web client agree on — field names
//! match plan §2.1 EXACTLY and mirror `apps/web/src/api/{crypto,security}-types.ts`.
//! `mw-engine`'s `security/types.rs` re-exports these so the engine, the mock, and
//! the WASM boundary emit byte-identical shapes (the V2/V3 parity gate, plan §1.5).
//!
//! DTOs only: `Serialize`/`Deserialize` shapes with no behaviour. e1 (crypto) and
//! e6 (engine) construct them; the enum-like string fields carry the frozen token
//! sets documented on each field (the engine owns the values — the store/DTOs
//! never interpret them, matching the V3 `pim/types.rs` convention).

use serde::{Deserialize, Serialize};

/// An RFC3339 UTC timestamp (e.g. `"2026-07-12T09:00:00Z"`).
pub type UtcDate = String;

// ── Keyring (§2.1 `CryptoKey`) ───────────────────────────────────────────────

/// One TOFU key-history entry (a fingerprint first seen at a time).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyHistoryEntry {
    pub fingerprint: String,
    pub seen_at: UtcDate,
}

/// A PGP or S/MIME key/cert (§2.1). Own keys carry an opaque
/// `encryptedPrivateBackup` (client-encrypted — the server NEVER decrypts it);
/// harvested/contact keys are public-only. `kind`/`trust`/`source` are frozen
/// token sets (see field docs). Per-contact association reuses the V3
/// `ContactCard.pgpKey`/`smimeCert` fields (now populated, not placeholders).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CryptoKey {
    pub id: String,
    /// `"pgp"` | `"smime"`.
    pub kind: String,
    pub is_own: bool,
    pub addresses: Vec<String>,
    pub fingerprint: String,
    pub key_id: String,
    pub algorithm: String,
    pub created_at: UtcDate,
    pub expires_at: Option<UtcDate>,
    /// Armored public key (pgp) — `null` for S/MIME.
    pub public_key_armored: Option<String>,
    /// PEM certificate (smime) — `null` for PGP.
    pub cert_pem: Option<String>,
    /// `"unverified"` | `"tofu"` | `"verified"` | `"revoked"`.
    pub trust: String,
    pub autocrypt: bool,
    /// `"generated"` | `"imported"` | `"pkcs12"` | `"wkd"` | `"vks"` |
    /// `"harvested"` | `"autocrypt-header"`.
    pub source: String,
    pub has_private: bool,
    /// Opaque client-encrypted private backup — the server never decrypts it.
    pub encrypted_private_backup: Option<String>,
    pub verified_at: Option<UtcDate>,
    pub key_history: Vec<KeyHistoryEntry>,
}

// ── Security verdict (§2.1 `SecurityVerdict`) ────────────────────────────────

/// DKIM auth result (`result` is the shared enum:
/// `"pass"|"fail"|"none"|"neutral"|"temperror"|"permerror"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DkimVerdict {
    pub result: String,
    pub domain: Option<String>,
    pub selector: Option<String>,
}

/// SPF auth result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpfVerdict {
    pub result: String,
    pub domain: Option<String>,
}

/// DMARC auth result + published policy + alignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DmarcVerdict {
    pub result: String,
    /// `"none"` | `"quarantine"` | `"reject"` | `null`.
    pub policy: Option<String>,
    pub aligned: bool,
}

/// ARC chain result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArcVerdict {
    pub result: String,
    pub chain_length: i64,
}

/// The email-authentication block (DKIM/SPF/DMARC/ARC), computed server-side by
/// `mail-auth` (all public — no secret material).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthVerdict {
    pub dkim: DkimVerdict,
    pub spf: SpfVerdict,
    pub dmarc: DmarcVerdict,
    pub arc: ArcVerdict,
}

/// One `Received`-chain hop (parsed from the header; ASN/country are optional
/// GeoIP enrichment, plan §1.12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceivedHop {
    pub index: i64,
    pub by_host: Option<String>,
    pub from_host: Option<String>,
    pub protocol: Option<String>,
    pub timestamp: Option<UtcDate>,
    pub delay_ms: Option<i64>,
    pub asn: Option<i64>,
    pub asn_org: Option<String>,
    pub country: Option<String>,
}

/// The 3-state signature/cert verdict (`status` is the FROZEN UI contract:
/// `"verified"|"unverified-key"|"invalid"|"none"`). Reused as the WASM
/// verify/decrypt/sign return (§2.3 `SignatureVerdict`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureVerdict {
    /// `"pgp"` | `"smime"`.
    pub kind: String,
    /// `"verified"` | `"unverified-key"` | `"invalid"` | `"none"`.
    pub status: String,
    pub signer_key_id: Option<String>,
    pub algorithm: Option<String>,
    pub key_created_at: Option<UtcDate>,
    pub key_expires_at: Option<UtcDate>,
    /// `"trusted"` | `"untrusted"` | `"expired"` | `"unknown"` | `null`.
    pub chain_status: Option<String>,
    /// `"good"` | `"revoked"` | `"unknown"` | `null`.
    pub revocation_status: Option<String>,
    pub key_changed: bool,
}

/// Whether/how the message is encrypted. `decryptsClientSide` flags the client
/// WASM decrypt path (the only server-side field that is not purely public).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionInfo {
    /// `"pgp"` | `"smime"` | `"none"`.
    pub kind: String,
    pub is_encrypted: bool,
    pub decrypts_client_side: bool,
}

/// Per-attachment risk analysis (ext-vs-magic mismatch + a risk token).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentRisk {
    pub name: String,
    pub declared_type: Option<String>,
    pub detected_type: Option<String>,
    pub mismatch: bool,
    /// `"none"` | `"macro"` | `"executable"` | `"encrypted-archive"` |
    /// `"double-extension"`.
    pub risk: String,
}

/// The §7.3 security verdict (frozen field-for-field — parity-critical). Computed
/// server-side (all public) except `encryption.decryptsClientSide`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityVerdict {
    pub email_id: String,
    pub auth: AuthVerdict,
    pub plain_language: String,
    pub received: Vec<ReceivedHop>,
    pub signature: Option<SignatureVerdict>,
    pub encryption: EncryptionInfo,
    pub attachments: Vec<AttachmentRisk>,
    /// Enum tokens: `"replyToMismatch"` | `"envelopeFromDivergence"` |
    /// `"messageIdDomainAnomaly"` | `"dateSkew"` | `"punycodeSender"`.
    pub anomalies: Vec<String>,
}

// ── DLP (§2.1 `DlpRule` / `DlpVerdict`) ──────────────────────────────────────

/// The match conditions of a DLP rule (config-sourced).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DlpConditions {
    /// Built-in detectors: `"pan"|"iban"|"national-id"|"ssn"|"custom-regex"`.
    pub detectors: Vec<String>,
    pub custom_regex: Option<String>,
    pub dictionaries: Vec<String>,
    pub attachment_types: Vec<String>,
    pub max_attachment_size: Option<i64>,
    pub recipient_domains: Vec<String>,
    /// `"in"` | `"notIn"` | `null`.
    pub recipient_domain_mode: Option<String>,
    pub classification: Option<String>,
}

/// A DLP rule (config/env-sourced in V4 — the admin panel is V6, plan §1.8).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DlpRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub priority: i64,
    pub conditions: DlpConditions,
    /// `"warn"` | `"block"` | `"require-encryption"` | `"notify-admin"`.
    pub action: String,
    pub message: String,
}

/// One rule's evaluation against an outbound draft (redacted — never content).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DlpVerdict {
    pub rule_id: String,
    pub rule_name: String,
    pub action: String,
    pub matched_detectors: Vec<String>,
    pub excerpt_redacted: String,
    pub blocked: bool,
}

// ── Mail rules (§2.1 `MailRule`) — the block/silence/ignore surface ──────────

/// One condition of a mail rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MailRuleCondition {
    /// `"from"` | `"to"` | `"subject"` | `"thread"`.
    #[serde(rename = "type")]
    pub kind: String,
    /// `"is"` | `"contains"`.
    pub op: String,
    pub value: String,
}

/// One action of a mail rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MailRuleAction {
    /// `"move"` | `"tag"` | `"stop"` | `"suppressNotify"` | `"archive"`.
    #[serde(rename = "type")]
    pub kind: String,
    pub value: Option<String>,
}

/// A mail rule (block/silence/ignore materialize here; also exposes the Sieve
/// round-trip). `runsAt`: `"server-sieve"` | `"engine"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MailRule {
    pub id: String,
    pub name: String,
    pub match_all: bool,
    pub conditions: Vec<MailRuleCondition>,
    pub actions: Vec<MailRuleAction>,
    pub enabled: bool,
    pub runs_at: String,
}
