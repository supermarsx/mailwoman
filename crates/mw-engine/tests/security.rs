//! Engine integration tests for the V4 crypto/security surface (plan §3 e6
//! acceptance): keyring round-trip (opaque backup — never plaintext private),
//! sender-control block → a real MailRule/Sieve rule, DLP block on send (redacted
//! audit), PQC store-key wrap round-trip, and the mock-vs-engine golden-shape
//! parity gate (a divergence here is a live crash later — the V2/V3 lesson).
//!
//! Everything is driven through `Engine::handle_jmap` — the exact envelope the
//! web client speaks — over a no-op mail backend.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MessageRef,
    MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor, WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

// ── harness ──────────────────────────────────────────────────────────────────

struct NoopBackend;

#[async_trait]
impl AccountBackend for NoopBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(Vec::new())
    }
    async fn sync_mailbox(&self, _m: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
        Ok(MailboxDelta {
            added: Vec::new(),
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: c.clone(),
        })
    }
    async fn fetch_raw(&self, _refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        Ok(Vec::new())
    }
    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _d: &[Flag]) -> Result<()> {
        Ok(())
    }
    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("noop".into()))
    }
    async fn append(&self, _m: &RawMailboxRef, _raw: &[u8], _f: &[Flag]) -> Result<MessageRef> {
        Err(EngineError::Unsupported("noop".into()))
    }
    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported("noop".into()))
    }
}

#[derive(Default)]
struct CapturingSubmitter {
    sent: Mutex<Vec<Outgoing>>,
}

#[async_trait]
impl MailSubmitter for CapturingSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        let accepted = msg.rcpt_to.clone();
        self.sent.lock().unwrap().push(msg);
        Ok(SubmissionResult {
            accepted,
            rejected: Vec::new(),
        })
    }
}

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
}

async fn setup() -> Harness {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: "me@example.org",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "me@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    let engine = Arc::new(Engine::new(store));
    let submitter = Arc::new(CapturingSubmitter::default());
    let runtime = AccountRuntime::new(
        Arc::new(NoopBackend) as Arc<dyn AccountBackend>,
        submitter as Arc<dyn MailSubmitter>,
        "me@example.org",
    );
    engine.register_backend(account_id.clone(), runtime);
    Harness { engine, account_id }
}

impl Harness {
    async fn call(&self, method: &str, args: Value) -> Value {
        let req = json!({ "methodCalls": [[method, args, "c0"]] });
        let resp = self.engine.handle_jmap(&self.account_id, &req).await;
        resp["methodResponses"][0][1].clone()
    }

    async fn call_all(&self, calls: Value) -> Value {
        self.engine
            .handle_jmap(&self.account_id, &json!({ "methodCalls": calls }))
            .await
    }
}

// ── keyring ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn keyring_round_trip_keeps_backup_opaque_never_plaintext_private() {
    let h = setup().await;
    let set = h
        .call(
            "CryptoKey/set",
            json!({ "create": { "k1": {
                "id": "",
                "kind": "pgp",
                "isOwn": true,
                "addresses": ["me@example.org"],
                "fingerprint": "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234",
                "keyId": "ABCD1234ABCD1234",
                "algorithm": "ed25519",
                "createdAt": "2026-07-13T09:00:00Z",
                "expiresAt": null,
                "publicKeyArmored": "-----BEGIN PGP PUBLIC KEY BLOCK-----\nX\n-----END PGP PUBLIC KEY BLOCK-----",
                "certPem": null,
                "trust": "unverified",
                "autocrypt": true,
                "source": "generated",
                "hasPrivate": true,
                "encryptedPrivateBackup": "OPAQUE-CLIENT-ENCRYPTED-BLOB",
                "verifiedAt": null,
                "keyHistory": []
            } } }),
        )
        .await;
    let id = set["created"]["k1"]["id"].as_str().unwrap().to_string();

    let got = h.call("CryptoKey/get", json!({ "ids": [id] })).await;
    let key = &got["list"][0];
    assert_eq!(key["kind"], "pgp");
    assert_eq!(key["isOwn"], true);
    assert_eq!(key["hasPrivate"], true);
    // The opaque client-encrypted backup round-trips verbatim.
    assert_eq!(
        key["encryptedPrivateBackup"],
        "OPAQUE-CLIENT-ENCRYPTED-BLOB"
    );
    // TOFU key-history seeded from the fingerprint.
    assert_eq!(
        key["keyHistory"][0]["fingerprint"],
        "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234"
    );
    // There is NO plaintext-private field anywhere in the persisted/emitted key
    // (the server can never hold plaintext private material — plan §1.2/risk #4).
    let serialized = key.to_string().to_lowercase();
    assert!(!serialized.contains("privatekey\":\"-----begin"));
    assert!(!serialized.contains("plaintextprivate"));

    // setTrust → verified stamps verifiedAt.
    h.call(
        "CryptoKey/setTrust",
        json!({ "id": id, "trust": "verified" }),
    )
    .await;
    let got2 = h.call("CryptoKey/get", json!({ "ids": [id] })).await;
    assert_eq!(got2["list"][0]["trust"], "verified");
    assert!(got2["list"][0]["verifiedAt"].is_string());
}

// ── sender controls → real MailRule/Sieve ────────────────────────────────────

#[tokio::test]
async fn block_sender_emits_a_real_mail_rule() {
    let h = setup().await;
    let resp = h
        .call(
            "SenderControl/set",
            json!({ "address": "spammer@bad.example", "action": "block" }),
        )
        .await;
    assert_eq!(resp["updated"], true);
    let rule_id = resp["mailRuleId"].as_str().expect("a real MailRule id");

    // The rule is a genuine From-is → Move Junk + Stop rule, visible over MailRule.
    let rules = h.call("MailRule/get", json!({})).await;
    let list = rules["list"].as_array().unwrap();
    let rule = list
        .iter()
        .find(|r| r["id"] == rule_id)
        .expect("block rule present");
    assert!(
        rule["conditions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["type"] == "from" && c["op"] == "is" && c["value"] == "spammer@bad.example")
    );
    let actions = rule["actions"].as_array().unwrap();
    assert!(
        actions
            .iter()
            .any(|a| a["type"] == "move" && a["value"] == "Junk")
    );
    assert!(actions.iter().any(|a| a["type"] == "stop"));
}

#[tokio::test]
async fn mail_rule_set_create_update_destroy() {
    let h = setup().await;
    let created = h
        .call(
            "MailRule/set",
            json!({ "create": { "r1": {
                "name": "Tag newsletters",
                "conditions": [{ "type": "from", "op": "contains", "value": "newsletter" }],
                "actions": [{ "type": "tag", "value": "$newsletter" }]
            } } }),
        )
        .await;
    let id = created["created"]["r1"]["id"].as_str().unwrap().to_string();
    let got = h.call("MailRule/get", json!({ "ids": [id] })).await;
    assert_eq!(got["list"][0]["name"], "Tag newsletters");

    h.call("MailRule/set", json!({ "destroy": [id] })).await;
    let after = h.call("MailRule/get", json!({})).await;
    assert!(after["list"].as_array().unwrap().is_empty());
}

// ── DLP block on send + redacted audit ───────────────────────────────────────

#[tokio::test]
async fn dlp_blocks_a_card_number_and_audits_redacted() {
    // Seed a config PAN block rule via MW_DLP_RULES (config-sourced, plan §1.8).
    let dir = std::env::temp_dir();
    let path = dir.join(format!("mw_dlp_rules_{}.json", std::process::id()));
    std::fs::write(
        &path,
        r#"[{"id":"rule-pan","name":"Block card numbers","enabled":true,"priority":10,
            "conditions":{"detectors":["pan"],"customRegex":null,"dictionaries":[],
              "attachmentTypes":[],"maxAttachmentSize":null,"recipientDomains":[],
              "recipientDomainMode":null,"classification":null},
            "action":"block","message":"Contains a card number."}]"#,
    )
    .unwrap();
    // SAFETY: single-threaded test set; only this test reads MW_DLP_RULES.
    unsafe {
        std::env::set_var("MW_DLP_RULES", &path);
    }

    let h = setup().await;
    let resp = h
        .call_all(json!([
            ["Email/set", { "create": { "draft": {
                "from": [{ "email": "me@example.org" }],
                "to": [{ "email": "friend@example.org" }],
                "subject": "Payment",
                "bodyValues": { "1": { "value": "Please charge 4111 1111 1111 1111 now." } },
                "textBody": [{ "partId": "1", "type": "text/plain" }]
            } } }, "c1"],
            ["EmailSubmission/set", { "create": { "sub1": { "emailId": "#draft" } } }, "c2"]
        ]))
        .await;

    let sub = &resp["methodResponses"][1][1];
    let not_created = &sub["notCreated"]["sub1"];
    assert_eq!(
        not_created["type"], "dlpBlocked",
        "send blocked by DLP: {sub}"
    );
    assert!(not_created["verdicts"][0]["blocked"].as_bool().unwrap());

    // A redacted audit row was written — matched detector + rule, NEVER content.
    let audit = h
        .engine
        .store()
        .list_dlp_audit(&h.account_id)
        .await
        .unwrap();
    assert_eq!(audit.len(), 1, "one audit row");
    let row = &audit[0];
    assert!(row.blocked);
    assert!(row.matched_detectors_json.contains("pan"));
    // The card number must appear NOWHERE in the audit row.
    let dumped = format!("{row:?}");
    assert!(!dumped.contains("4111"), "audit leaked content: {dumped}");

    unsafe {
        std::env::remove_var("MW_DLP_RULES");
    }
    let _ = std::fs::remove_file(&path);
}

// ── PQC store-key wrap ───────────────────────────────────────────────────────

#[tokio::test]
async fn pqc_wrapped_store_key_round_trips() {
    let h = setup().await;
    let seal_key = b"this-is-a-32-byte-seal-key-000000".to_vec();
    let recipient = h.engine.pqc_wrap_store_seal(&seal_key).await.unwrap();
    let unwrapped = h
        .engine
        .pqc_unwrap_store_seal(&recipient.secret)
        .await
        .unwrap();
    assert_eq!(unwrapped, seal_key, "PQC-wrapped seal key round-trips");

    // The persisted material carries the crypto-agility suite tag.
    let row = h
        .engine
        .store()
        .get_store_key_material()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.suite, mw_crypto::STORE_KEY_WRAP_SUITE);
}

// ── mock ↔ engine golden shape (the parity gate) ─────────────────────────────

/// Whether two JSON values have a compatible FIELD structure (names + nesting):
/// objects must have the same key set (recursively); arrays compare their first
/// element's shape, but an empty array on either side is a wildcard (its element
/// shape is unknowable); scalars/nulls are leaves and always compatible. Captures
/// the drift risk (field-name divergence) while ignoring value types + nulls.
fn shapes_compatible(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Object(ma), Value::Object(mb)) => {
            ma.len() == mb.len()
                && ma
                    .iter()
                    .all(|(k, va)| mb.get(k).is_some_and(|vb| shapes_compatible(va, vb)))
        }
        (Value::Array(aa), Value::Array(ab)) => match (aa.first(), ab.first()) {
            (Some(x), Some(y)) => shapes_compatible(x, y),
            _ => true, // one side empty → element shape unknowable → compatible
        },
        (Value::Object(_), _) | (_, Value::Object(_)) => false,
        (Value::Array(_), _) | (_, Value::Array(_)) => false,
        _ => true, // both scalars/nulls
    }
}

#[tokio::test]
async fn crypto_key_shape_matches_mock() {
    let h = setup().await;
    h.call(
        "CryptoKey/set",
        json!({ "create": { "k1": {
            "id": "", "kind": "pgp", "isOwn": true, "addresses": ["me@example.org"],
            "fingerprint": "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234", "keyId": "ABCD1234ABCD1234",
            "algorithm": "ed25519", "createdAt": "2026-07-13T09:00:00Z", "expiresAt": null,
            "publicKeyArmored": "-----BEGIN PGP PUBLIC KEY BLOCK-----\nX\n-----END PGP PUBLIC KEY BLOCK-----",
            "certPem": null, "trust": "verified", "autocrypt": true, "source": "generated",
            "hasPrivate": true, "encryptedPrivateBackup": null, "verifiedAt": "2026-07-13T09:00:00Z",
            "keyHistory": [{ "fingerprint": "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234", "seenAt": "2026-07-13T09:00:00Z" }]
        } } }),
    )
    .await;
    let engine_key = h.call("CryptoKey/get", json!({})).await["list"][0].clone();
    let mock = mw_mock_jmap::security_case("CryptoKey/get", &json!({})).unwrap();
    let mock_key = mock["list"][0].clone();
    assert!(
        shapes_compatible(&engine_key, &mock_key),
        "CryptoKey shape drift: engine {engine_key} vs mock {mock_key}"
    );
}

#[tokio::test]
async fn security_verdict_and_dlp_and_mail_rule_shapes_match_mock() {
    // The canonical DTOs the engine serializes (mw_crypto::types) must shape-match
    // the mock's frozen fixtures field-for-field. The engine emits these exact
    // structs (serde), so DTO≡mock ⟹ engine≡mock.
    use mw_crypto::types::{
        ArcVerdict, AttachmentRisk, AuthVerdict, DkimVerdict, DlpVerdict, DmarcVerdict,
        EncryptionInfo, ReceivedHop, SecurityVerdict, SignatureVerdict,
    };

    let verdict = SecurityVerdict {
        email_id: "e1".into(),
        auth: AuthVerdict {
            dkim: DkimVerdict {
                result: "pass".into(),
                domain: Some("x".into()),
                selector: Some("s".into()),
            },
            spf: SpfVerdictLocal(),
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
        plain_language: "ok".into(),
        received: vec![ReceivedHop {
            index: 0,
            by_host: Some("h".into()),
            from_host: Some("f".into()),
            protocol: Some("ESMTP".into()),
            timestamp: Some("2026-07-13T09:00:00Z".into()),
            delay_ms: Some(1),
            asn: Some(1),
            asn_org: Some("o".into()),
            country: Some("PT".into()),
        }],
        signature: Some(SignatureVerdict {
            kind: "pgp".into(),
            status: "verified".into(),
            signer_key_id: Some("k".into()),
            algorithm: Some("ed25519".into()),
            key_created_at: Some("2026-07-13T09:00:00Z".into()),
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
            name: "a".into(),
            declared_type: Some("t".into()),
            detected_type: Some("t".into()),
            mismatch: false,
            risk: "none".into(),
        }],
        anomalies: vec!["replyToMismatch".into()],
    };

    let engine_verdict = serde_json::to_value(&verdict).unwrap();
    let mock_verdict = mw_mock_jmap::security_case(
        "SecurityVerdict/get",
        &json!({ "ids": ["e1"] }),
    )
    .unwrap()["list"][0]
        .clone();
    assert!(
        shapes_compatible(&engine_verdict, &mock_verdict),
        "SecurityVerdict shape drift: engine {engine_verdict} vs mock {mock_verdict}"
    );

    // DlpVerdict: no mock fixture (scan returns []); assert the frozen §2.1 keys.
    let dv = DlpVerdict {
        rule_id: "r".into(),
        rule_name: "n".into(),
        action: "block".into(),
        matched_detectors: vec!["pan".into()],
        excerpt_redacted: "••••".into(),
        blocked: true,
    };
    let dvj = serde_json::to_value(&dv).unwrap();
    let mut keys: Vec<&String> = dvj.as_object().unwrap().keys().collect();
    keys.sort();
    assert_eq!(
        keys,
        vec![
            "action",
            "blocked",
            "excerptRedacted",
            "matchedDetectors",
            "ruleId",
            "ruleName"
        ]
    );

    // MailRule: engine DTO shape-matches the mock's fixture.
    let mock_rule =
        mw_mock_jmap::security_case("MailRule/get", &json!({})).unwrap()["list"][0].clone();
    let engine_rule = serde_json::to_value(mw_crypto::types::MailRule {
        id: "mr-1".into(),
        name: "Block spammer".into(),
        match_all: false,
        conditions: vec![mw_crypto::types::MailRuleCondition {
            kind: "from".into(),
            op: "is".into(),
            value: "spammer@bad.example".into(),
        }],
        actions: vec![mw_crypto::types::MailRuleAction {
            kind: "move".into(),
            value: Some("Junk".into()),
        }],
        enabled: true,
        runs_at: "engine".into(),
    })
    .unwrap();
    assert!(
        shapes_compatible(&engine_rule, &mock_rule),
        "MailRule shape drift"
    );
}

/// Local helper so the big struct literal above stays readable.
#[allow(non_snake_case)]
fn SpfVerdictLocal() -> mw_crypto::types::SpfVerdict {
    mw_crypto::types::SpfVerdict {
        result: "pass".into(),
        domain: Some("x".into()),
    }
}
