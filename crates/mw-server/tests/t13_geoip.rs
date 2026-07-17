//! t13-E10 — GeoIP/ASN Received-hop enrichment live-E2E (26.13 workstream 4).
//!
//! The "unit-green ≠ wired" gate for E5's BYO MaxMind-DB reader
//! (`crates/mw-engine/src/security/verdict.rs`). E5 proved the reader opens + caches
//! and that the no-DB path degrades to None; THIS leg proves the whole path is wired
//! end-to-end: with `MW_GEOIP_DB` pointed at a committed fixture `.mmdb`, a stored
//! message's Received chain surfaces ASN + country in the `SecurityVerdict` returned
//! by the JMAP `SecurityVerdict/get` method (the same surface the web reader hits).
//!
//! The message is ingested from the REAL Dovecot seed corpus (`05-geo.eml`), whose
//! Received chain carries two known GeoIP vectors:
//!   * `[1.128.0.0]`     → AS1221 (Telstra) in GeoLite2-ASN-Test.mmdb
//!   * `[89.160.20.128]` → country SE      in GeoIP2-Country-Test.mmdb
//!
//! The verdict is cached per (emailId, raw-hash), so each sub-test uses a FRESH
//! store (fresh emailId) and sets `MW_GEOIP_DB` BEFORE the first `SecurityVerdict/get`
//! for that message. Fixtures are the MaxMind `MaxMind-DB` test data (Apache-2.0 /
//! MIT — see fixtures/geoip/NOTICE).
//!
//! ## Running
//!   docker compose -f docker-compose.ci.yml up -d --wait dovecot-t13
//!   MW_T13_LIVE=1 cargo test -p mw-server --test t13_geoip -- --nocapture --test-threads=1

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::Engine;
use mw_engine::account::{AccountRuntime, MailSubmitter};
use mw_engine::backend::{AccountBackend, Result as EngineResult};
use mw_imap::transport::TlsMode;
use mw_imap::{Credentials as ImapCredentials, ImapBackend, ImapConfig};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials as StoreCreds, NewAccount, ServerKey, Store};

const IMAP_PLAINTEXT: u16 = 3143;
const USER: &str = "testuser";
const PASS: &str = "testpass";

fn live() -> bool {
    std::env::var("MW_T13_LIVE").ok().as_deref() == Some("1")
}
fn host() -> String {
    std::env::var("MW_T13_HOST").unwrap_or_else(|_| "127.0.0.1".into())
}
fn fixture(name: &str) -> String {
    format!("{}/../../fixtures/geoip/{name}", env!("CARGO_MANIFEST_DIR"))
}

struct NoSubmitter;
#[async_trait]
impl MailSubmitter for NoSubmitter {
    async fn submit(&self, msg: Outgoing) -> EngineResult<SubmissionResult> {
        Ok(SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

/// Resync the corpus, find the geo message's JMAP id, and return its SecurityVerdict
/// `received` hop array. `MW_GEOIP_DB` must already be set.
async fn geo_received_hops() -> Option<Vec<Value>> {
    let cfg = ImapConfig {
        host: host(),
        port: IMAP_PLAINTEXT,
        tls: TlsMode::Plaintext,
        credentials: ImapCredentials::Password {
            username: USER.into(),
            password: PASS.into(),
        },
        watch_mailbox: "INBOX".into(),
    };
    let backend = match ImapBackend::connect(cfg).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("\n[t13 GeoIP SKIP] dovecot-t13 unreachable ({e}).\n");
            return None;
        }
    };
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: &host(),
                port: IMAP_PLAINTEXT,
                tls: "plaintext",
                username: USER,
                sync_policy_json: "{}",
            },
            &StoreCreds {
                username: USER.into(),
                password: PASS.into(),
            },
        )
        .await
        .unwrap();
    let engine = Arc::new(Engine::new(store));
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            Arc::new(backend) as Arc<dyn AccountBackend>,
            Arc::new(NoSubmitter) as Arc<dyn MailSubmitter>,
            USER.to_string(),
        ),
    );
    engine.resync(&account_id).await.expect("live resync");

    // Resolve INBOX, then Email/query {inMailbox} (unfiltered query returns nothing).
    let inbox_id = engine
        .handle_jmap(
            &account_id,
            &json!({ "methodCalls": [["Mailbox/get", {}, "mb"]] }),
        )
        .await["methodResponses"][0][1]["list"]
        .as_array()
        .expect("mailbox list")
        .iter()
        .find(|m| m["role"] == "inbox")
        .and_then(|m| m["id"].as_str().map(String::from))
        .expect("INBOX after resync");
    let ids: Vec<String> = engine
        .handle_jmap(
            &account_id,
            &json!({ "methodCalls": [["Email/query", { "filter": { "inMailbox": inbox_id } }, "q"]] }),
        )
        .await["methodResponses"][0][1]["ids"]
        .as_array()
        .expect("ids")
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    // Email/get does not emit a JMAP messageId; identify the geo message by its
    // unique subject ("Login from new location").
    let got = engine
        .handle_jmap(
            &account_id,
            &json!({ "methodCalls": [["Email/get", { "ids": ids, "properties": ["subject"] }, "g"]] }),
        )
        .await;
    let geo_id = got["methodResponses"][0][1]["list"]
        .as_array()
        .expect("list")
        .iter()
        .find(|e| e["subject"].as_str() == Some("Login from new location"))
        .and_then(|e| e["id"].as_str().map(String::from))
        .expect("geo message ingested");

    let verdict = engine
        .handle_jmap(
            &account_id,
            &json!({ "methodCalls": [["SecurityVerdict/get", { "ids": [geo_id] }, "v"]] }),
        )
        .await;
    let hops = verdict["methodResponses"][0][1]["list"][0]["received"]
        .as_array()
        .cloned()
        .expect("verdict.received array");
    Some(hops)
}

#[tokio::test]
async fn geoip_asn_surfaces_in_received_hop() {
    if !live() {
        eprintln!("\n[t13 GeoIP SKIP] MW_T13_LIVE!=1 — live ingest not driven.\n");
        return;
    }
    unsafe {
        std::env::set_var("MW_GEOIP_DB", fixture("GeoLite2-ASN-Test.mmdb"));
    }
    let Some(hops) = geo_received_hops().await else {
        return;
    };
    let telstra = hops.iter().find(|h| h["asn"].as_i64() == Some(1221));
    assert!(
        telstra.is_some(),
        "a Received hop ([1.128.0.0]) must resolve to AS1221 via the ASN fixture; hops={hops:?}"
    );
    let org = telstra.unwrap()["asnOrg"].as_str().unwrap_or("");
    assert!(
        org.contains("Telstra"),
        "AS1221 org should name Telstra; got {org:?}"
    );
}

#[tokio::test]
async fn geoip_country_surfaces_in_received_hop() {
    if !live() {
        return;
    }
    unsafe {
        std::env::set_var("MW_GEOIP_DB", fixture("GeoIP2-Country-Test.mmdb"));
    }
    let Some(hops) = geo_received_hops().await else {
        return;
    };
    assert!(
        hops.iter().any(|h| h["country"].as_str() == Some("SE")),
        "a Received hop ([89.160.20.128]) must resolve to country SE via the Country fixture; hops={hops:?}"
    );
}
