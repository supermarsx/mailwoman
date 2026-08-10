//! The frozen §2.1 DTO wire shapes.
//!
//! `types.rs` is documented as "frozen field-for-field — parity-critical": the
//! engine, the mock and the WASM boundary must all emit byte-identical JSON, and
//! `apps/web/src/api/{crypto,security}-types.ts` is written against it. Nothing
//! was asserting that. The file carried no coverage at all in the 26.19 baseline
//! because its `serde` impls are generic and were never instantiated by a test.
//!
//! These tests pin the *names on the wire*, which is the thing a rename or a
//! dropped `#[serde(rename_all)]` would break silently — the Rust field names
//! would still compile everywhere and only the browser would notice.

use mw_crypto::{
    ArcVerdict, AttachmentRisk, AuthVerdict, CryptoKey, DkimVerdict, DlpConditions, DlpRule,
    DlpVerdict, DmarcVerdict, EncryptionInfo, KeyHistoryEntry, MailRule, MailRuleAction,
    MailRuleCondition, ReceivedHop, SecurityVerdict, SignatureVerdict, SpfVerdict,
};

fn json<T: serde::Serialize>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).expect("serialize")
}

/// Every key of `value` (one level deep), sorted — the wire contract.
fn keys(value: &serde_json::Value) -> Vec<String> {
    let mut k: Vec<String> = value
        .as_object()
        .expect("an object")
        .keys()
        .cloned()
        .collect();
    k.sort();
    k
}

fn sample_key() -> CryptoKey {
    CryptoKey {
        id: "k1".into(),
        kind: "pgp".into(),
        is_own: true,
        addresses: vec!["a@example.test".into()],
        fingerprint: "AA".repeat(32),
        key_id: "0011223344556677".into(),
        algorithm: "ed25519".into(),
        created_at: "2026-01-01T00:00:00Z".into(),
        expires_at: None,
        public_key_armored: Some("-----BEGIN PGP PUBLIC KEY BLOCK-----".into()),
        cert_pem: None,
        trust: "tofu".into(),
        autocrypt: true,
        source: "imported".into(),
        has_private: true,
        encrypted_private_backup: Some("opaque".into()),
        verified_at: None,
        key_history: vec![KeyHistoryEntry {
            fingerprint: "BB".repeat(32),
            seen_at: "2026-01-02T00:00:00Z".into(),
        }],
    }
}

/// `CryptoKey` serialises in camelCase with every documented field present, and
/// `null` for the absent optional ones (rather than being omitted — the web
/// client reads them positionally).
#[test]
fn crypto_key_wire_shape_is_camel_case_and_complete() {
    let v = json(&sample_key());
    assert_eq!(
        keys(&v),
        [
            "addresses",
            "algorithm",
            "autocrypt",
            "certPem",
            "createdAt",
            "encryptedPrivateBackup",
            "expiresAt",
            "fingerprint",
            "hasPrivate",
            "id",
            "isOwn",
            "keyHistory",
            "keyId",
            "kind",
            "publicKeyArmored",
            "source",
            "trust",
            "verifiedAt",
        ]
    );
    assert!(v["expiresAt"].is_null());
    assert!(v["certPem"].is_null());
    assert_eq!(keys(&v["keyHistory"][0]), ["fingerprint", "seenAt"]);
}

/// A create arrives without an `id` — the client omits it and the engine mints
/// one — so `id` must default rather than fail to deserialize (V4 live-E2E gap
/// #2, recorded on the field itself).
#[test]
fn crypto_key_deserializes_without_an_id() {
    let mut v = json(&sample_key());
    v.as_object_mut().unwrap().remove("id");
    let back: CryptoKey = serde_json::from_value(v).expect("id defaults");
    assert_eq!(back.id, "");
    assert_eq!(back.kind, "pgp");
}

/// A full round-trip through JSON is lossless for every DTO the engine returns.
#[test]
fn crypto_key_round_trips_through_json() {
    let original = sample_key();
    let back: CryptoKey = serde_json::from_value(json(&original)).expect("round trip");
    assert_eq!(back, original);
}

/// The §7.3 security verdict is the parity-critical one: the reader renders it
/// field by field.
#[test]
fn security_verdict_wire_shape() {
    let v = json(&SecurityVerdict {
        email_id: "e1".into(),
        auth: AuthVerdict {
            dkim: DkimVerdict {
                result: "pass".into(),
                domain: Some("example.test".into()),
                selector: Some("s1".into()),
            },
            spf: SpfVerdict {
                result: "fail".into(),
                domain: None,
            },
            dmarc: DmarcVerdict {
                result: "pass".into(),
                policy: Some("reject".into()),
                aligned: true,
            },
            arc: ArcVerdict {
                result: "none".into(),
                chain_length: 0,
            },
        },
        plain_language: "Looks genuine.".into(),
        received: vec![ReceivedHop {
            index: 0,
            by_host: Some("mx.example.test".into()),
            from_host: None,
            protocol: Some("ESMTPS".into()),
            timestamp: Some("2026-01-01T00:00:00Z".into()),
            delay_ms: Some(120),
            asn: Some(64_500),
            asn_org: Some("Example AS".into()),
            country: Some("NL".into()),
        }],
        signature: Some(SignatureVerdict {
            kind: "smime".into(),
            status: "verified".into(),
            signer_key_id: Some("0011".into()),
            algorithm: Some("rsa-2048".into()),
            key_created_at: None,
            key_expires_at: None,
            chain_status: Some("trusted".into()),
            revocation_status: Some("good".into()),
            key_changed: false,
        }),
        encryption: EncryptionInfo {
            kind: "none".into(),
            is_encrypted: false,
            decrypts_client_side: false,
        },
        attachments: vec![AttachmentRisk {
            name: "invoice.pdf.exe".into(),
            declared_type: Some("application/pdf".into()),
            detected_type: Some("application/x-dosexec".into()),
            mismatch: true,
            risk: "double-extension".into(),
        }],
        anomalies: vec!["replyToMismatch".into()],
    });

    assert_eq!(
        keys(&v),
        [
            "anomalies",
            "attachments",
            "auth",
            "emailId",
            "encryption",
            "plainLanguage",
            "received",
            "signature",
        ]
    );
    assert_eq!(keys(&v["auth"]), ["arc", "dkim", "dmarc", "spf"]);
    assert_eq!(keys(&v["auth"]["dkim"]), ["domain", "result", "selector"]);
    assert_eq!(keys(&v["auth"]["dmarc"]), ["aligned", "policy", "result"]);
    assert_eq!(keys(&v["auth"]["arc"]), ["chainLength", "result"]);
    assert_eq!(
        keys(&v["received"][0]),
        [
            "asn",
            "asnOrg",
            "byHost",
            "country",
            "delayMs",
            "fromHost",
            "index",
            "protocol",
            "timestamp",
        ]
    );
    assert_eq!(
        keys(&v["signature"]),
        [
            "algorithm",
            "chainStatus",
            "keyChanged",
            "keyCreatedAt",
            "keyExpiresAt",
            "kind",
            "revocationStatus",
            "signerKeyId",
            "status",
        ]
    );
    assert_eq!(
        keys(&v["encryption"]),
        ["decryptsClientSide", "isEncrypted", "kind"]
    );
    assert_eq!(
        keys(&v["attachments"][0]),
        ["declaredType", "detectedType", "mismatch", "name", "risk"]
    );
    // A signature-less message serialises `signature` as null, not as a missing key.
    let mut unsigned = v.clone();
    unsigned["signature"] = serde_json::Value::Null;
    let back: SecurityVerdict = serde_json::from_value(unsigned).expect("null signature");
    assert!(back.signature.is_none());
}

/// DLP rules and verdicts: `type` is a reserved word in Rust, so the two
/// condition/action structs carry an explicit `#[serde(rename = "type")]` that a
/// refactor could drop without any Rust-side breakage.
#[test]
fn dlp_and_mail_rule_wire_shapes() {
    let dlp = json(&DlpRule {
        id: "r1".into(),
        name: "PAN".into(),
        enabled: true,
        priority: 10,
        conditions: DlpConditions {
            detectors: vec!["pan".into()],
            custom_regex: None,
            dictionaries: vec![],
            attachment_types: vec!["application/pdf".into()],
            max_attachment_size: Some(1024),
            recipient_domains: vec!["example.test".into()],
            recipient_domain_mode: Some("notIn".into()),
            classification: None,
        },
        action: "block".into(),
        message: "blocked".into(),
    });
    assert_eq!(
        keys(&dlp),
        [
            "action",
            "conditions",
            "enabled",
            "id",
            "message",
            "name",
            "priority"
        ]
    );
    assert_eq!(
        keys(&dlp["conditions"]),
        [
            "attachmentTypes",
            "classification",
            "customRegex",
            "detectors",
            "dictionaries",
            "maxAttachmentSize",
            "recipientDomainMode",
            "recipientDomains",
        ]
    );

    let verdict = json(&DlpVerdict {
        rule_id: "r1".into(),
        rule_name: "PAN".into(),
        action: "block".into(),
        matched_detectors: vec!["pan".into()],
        excerpt_redacted: "•••• 1234".into(),
        blocked: true,
    });
    assert_eq!(
        keys(&verdict),
        [
            "action",
            "blocked",
            "excerptRedacted",
            "matchedDetectors",
            "ruleId",
            "ruleName",
        ]
    );

    let rule = json(&MailRule {
        id: "m1".into(),
        name: "silence".into(),
        match_all: true,
        conditions: vec![MailRuleCondition {
            kind: "from".into(),
            op: "contains".into(),
            value: "noreply@".into(),
        }],
        actions: vec![MailRuleAction {
            kind: "suppressNotify".into(),
            value: None,
        }],
        enabled: true,
        runs_at: "engine".into(),
    });
    assert_eq!(
        keys(&rule),
        [
            "actions",
            "conditions",
            "enabled",
            "id",
            "matchAll",
            "name",
            "runsAt"
        ]
    );
    // `type`, not `kind` — this is the rename that a refactor would silently drop.
    assert_eq!(keys(&rule["conditions"][0]), ["op", "type", "value"]);
    assert_eq!(rule["conditions"][0]["type"], "from");
    assert_eq!(keys(&rule["actions"][0]), ["type", "value"]);
    assert_eq!(rule["actions"][0]["type"], "suppressNotify");
}
