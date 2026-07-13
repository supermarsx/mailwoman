//! Frozen §2.1 crypto/security types for the engine surface. These are the
//! `mw-crypto` DTOs re-exported (single source of truth, plan §1.5) so the
//! engine, the mock (`mw-mock-jmap`), and the WASM boundary emit byte-identical
//! shapes — the V2/V3 parity gate. e6 constructs them from the store + `mw-crypto`
//! (native verify/harvest) + `mail-auth`; drift is a build failure.

pub use mw_crypto::types::{
    ArcVerdict, AttachmentRisk, AuthVerdict, CryptoKey, DkimVerdict, DlpConditions, DlpRule,
    DlpVerdict, DmarcVerdict, EncryptionInfo, KeyHistoryEntry, MailRule, MailRuleAction,
    MailRuleCondition, ReceivedHop, SecurityVerdict, SignatureVerdict, SpfVerdict,
};
