//! t10-e14 backend live-E2E — spam classify through the REAL jail (+ masked-email
//! lifecycle). SPEC §10.8 / §28.4.
//!
//! Two layers, both driving the REAL digest-shipped `spam-{rspamd,spamassassin}.wasm`
//! components through the REAL wasmtime jail + cap gate:
//!
//!  * **Fail-soft legs (default `cargo test` gate, no services).** A denied host and an
//!    unreachable daemon MUST resolve to an `unknown` verdict — never `spam`, never a
//!    hard block. This is the fail-soft contract the delivery pipeline relies on, proven
//!    against the real component (deny-by-default at the `net_allowlist`, transport error
//!    from a failing host fetcher).
//!
//!  * **Live daemon leg (gated `MW_SPAM_LIVE=1`).** A GTUBE spam fixture and a ham fixture
//!    are classified against a REAL rspamd scan worker (`:11333/checkv2`, native HTTP) and
//!    a REAL SpamAssassin `spamd` behind the SPAMC→HTTP relay. GTUBE ⇒ `spam`, ham ⇒
//!    `ham`. Loud skip (never silent) when the stack is not up.
//!
//!    docker compose -f docker-compose.ci.yml up -d --wait rspamd spamd spamassassin
//!    MW_SPAM_LIVE=1 cargo test -p mw-server --test t10_spam_masked -- --nocapture
//!
//! The host fetcher rewrites the guest's compiled service hostnames (`rspamd`,
//! `spamassassin`) to the published localhost ports so the on-host test reaches the
//! compose services while the guest URL + `net_allowlist` stay exactly as production.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use mw_plugin::{
    Capability, Grant, HostServices, HttpFetcher, HttpReq, HttpResp, PluginHandle, PluginHost,
    PluginManifest, TrustRoot,
};

// ── Fixtures ────────────────────────────────────────────────────────────────────

/// The GTUBE test string (Generic Test for Unsolicited Bulk Email) — every spam engine
/// flags it deterministically, so the live leg needs no rule training.
const GTUBE: &str = "XJS*C4JDBQADN1.NSBN3*2IDNEN*GTUBE-STANDARD-ANTI-UBE-TEST-EMAIL*C.34X";

fn gtube_message() -> Vec<u8> {
    format!(
        "From: sender@spam.example\r\n\
         To: victim@vogue-homes.com\r\n\
         Subject: you won\r\n\
         Message-ID: <gtube@spam.example>\r\n\
         \r\n\
         {GTUBE}\r\n"
    )
    .into_bytes()
}

fn ham_message() -> Vec<u8> {
    b"From: colleague@vogue-homes.com\r\n\
      To: me@vogue-homes.com\r\n\
      Subject: lunch tomorrow?\r\n\
      Message-ID: <ham1@vogue-homes.com>\r\n\
      \r\n\
      Are we still on for lunch at noon? Thanks.\r\n"
        .to_vec()
}

fn dist_wasm(id: &str) -> Vec<u8> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../plugins/dist")
        .join(format!("{id}.wasm"));
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn verdict_of(json: &str) -> String {
    serde_json::from_str::<Value>(json)
        .ok()
        .and_then(|v| v["verdict"].as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".into())
}

/// Load a spam-action component into a real jail with the given services + net allowlist.
fn load_spam(services: HostServices, id: &str, net_allowlist: Vec<String>) -> PluginHandle {
    let host = PluginHost::try_new(services, TrustRoot::empty()).expect("host");
    let manifest = PluginManifest {
        id: id.into(),
        name: id.into(),
        version: "26.10.0".into(),
        signature: None,
        capabilities: vec![Capability::SpamAction, Capability::Net],
        net_allowlist,
        limits: mw_plugin::PluginLimits::default(),
    };
    let grant = Grant {
        plugin_id: id.into(),
        capabilities: vec![Capability::SpamAction, Capability::Net],
        granted_by: "e14".into(),
        allow_unsigned: true,
    };
    host.load(&dist_wasm(id), &manifest, &grant)
        .expect("load spam component")
}

// ── A host fetcher that always fails (simulates a killed/unreachable daemon) ──────
struct DeadFetcher;
#[async_trait]
impl HttpFetcher for DeadFetcher {
    async fn fetch(&self, _req: HttpReq) -> Result<HttpResp, String> {
        Err("connection refused (daemon down)".into())
    }
}

// ── 1. Deny-by-default: a host outside the net_allowlist ⇒ fail-soft unknown ──────
#[tokio::test]
async fn spam_classify_denied_host_is_failsoft_unknown() {
    for id in ["spam-rspamd", "spam-spamassassin"] {
        // Empty allowlist ⇒ the guest's http_fetch is denied at the gate BEFORE any
        // network. The guest maps CapabilityDenied to an explicit unknown verdict.
        let handle = load_spam(HostServices::default(), id, vec![]);
        let out = handle
            .call_spam_action(gtube_message())
            .await
            .expect("classify is Ok even when denied (fail-soft, never Err)");
        assert_eq!(
            verdict_of(&out),
            "unknown",
            "{id}: a denied daemon host ⇒ unknown, NEVER a hard block (verdict={out})"
        );
        assert_ne!(verdict_of(&out), "spam", "{id}: fail-soft is never spam");
    }
}

// ── 2. Daemon unreachable (transport error) ⇒ fail-soft unknown, message delivered ─
#[tokio::test]
async fn spam_classify_daemon_down_is_failsoft_unknown() {
    for (id, allow) in [
        ("spam-rspamd", "rspamd"),
        ("spam-spamassassin", "spamassassin"),
    ] {
        let services = HostServices {
            http: Arc::new(DeadFetcher),
            ..HostServices::default()
        };
        let handle = load_spam(services, id, vec![allow.into()]);
        // Even a GTUBE message: with the daemon down we CANNOT classify, so the verdict
        // is unknown and the delivery pipeline treats it as deliver-normally.
        let out = handle.call_spam_action(gtube_message()).await.expect("Ok");
        assert_eq!(
            verdict_of(&out),
            "unknown",
            "{id}: an unreachable daemon ⇒ unknown (fail-soft), never a dropped/blocked \
             message (verdict={out})"
        );
    }
}

// ── 3. LIVE: real rspamd + real SpamAssassin classify GTUBE=spam, ham=ham ─────────

fn spam_live() -> bool {
    std::env::var("MW_SPAM_LIVE").ok().as_deref() == Some("1")
}

fn rspamd_port() -> String {
    std::env::var("MW_RSPAMD_PORT").unwrap_or_else(|_| "11333".into())
}
fn relay_port() -> String {
    std::env::var("MW_SPAMC_RELAY_PORT").unwrap_or_else(|_| "7830".into())
}

/// A real reqwest fetcher that rewrites the guest's compiled service hostnames to the
/// published localhost ports (the guest URL + net_allowlist stay production-shaped).
struct LiveRewriteFetcher {
    client: reqwest::Client,
}

#[async_trait]
impl HttpFetcher for LiveRewriteFetcher {
    async fn fetch(&self, req: HttpReq) -> Result<HttpResp, String> {
        let url = req
            .url
            .replacen("//rspamd:11333", &format!("//127.0.0.1:{}", rspamd_port()), 1)
            .replacen(
                "//spamassassin:783",
                &format!("//127.0.0.1:{}", relay_port()),
                1,
            );
        let method = reqwest::Method::from_bytes(req.method.as_bytes()).map_err(|e| e.to_string())?;
        let mut rb = self.client.request(method, &url);
        for (k, v) in &req.headers {
            rb = rb.header(k.as_str(), v.as_str());
        }
        if let Some(body) = req.body {
            rb = rb.body(body);
        }
        let resp = rb.send().await.map_err(|e| e.to_string())?;
        let status = resp.status().as_u16();
        let body = resp.bytes().await.map_err(|e| e.to_string())?.to_vec();
        Ok(HttpResp {
            status,
            headers: vec![],
            body,
        })
    }
}

async fn classify_live(id: &str, allow: &str, msg: Vec<u8>) -> String {
    let services = HostServices {
        http: Arc::new(LiveRewriteFetcher {
            client: reqwest::Client::new(),
        }),
        ..HostServices::default()
    };
    let handle = load_spam(services, id, vec![allow.into()]);
    let out = handle.call_spam_action(msg).await.expect("classify Ok");
    verdict_of(&out)
}

#[tokio::test]
async fn spam_live_gtube_is_spam_ham_is_ham() {
    if !spam_live() {
        eprintln!(
            "\n[t10-e14 SPAM SKIP] MW_SPAM_LIVE!=1 — real rspamd/SpamAssassin not driven. \
             Bring up: docker compose -f docker-compose.ci.yml up -d --wait rspamd spamd \
             spamassassin ; then MW_SPAM_LIVE=1 cargo test -p mw-server --test t10_spam_masked.\n"
        );
        return;
    }

    for (id, allow) in [
        ("spam-rspamd", "rspamd"),
        ("spam-spamassassin", "spamassassin"),
    ] {
        let spam = classify_live(id, allow, gtube_message()).await;
        assert_eq!(spam, "spam", "{id}: GTUBE must classify as spam (got {spam})");

        let ham = classify_live(id, allow, ham_message()).await;
        assert_eq!(ham, "ham", "{id}: a benign message must classify as ham (got {ham})");
    }
}

// ── 4. Masked-email alias lifecycle (§28.4) round-trips through the store repo ────
#[tokio::test]
async fn masked_email_lifecycle_round_trips() {
    use mw_store::{MaskedEmailRow, ServerKey, Store};

    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account = "acct-e14";
    let row = MaskedEmailRow {
        id: "mask-1".into(),
        account_id: account.into(),
        alias_addr: "a1b2c3@masked.vogue-homes.com".into(),
        target_desc: "newsletter signup".into(),
        state: "enabled".into(),
        created_at: "2026-07-14T00:00:00Z".into(),
        last_used_at: None,
    };
    store.put_masked_email(&row).await.unwrap();

    // Listed for its own account.
    let listed = store.list_masked_email(account).await.unwrap();
    assert_eq!(listed.len(), 1, "the alias is listed for its account");
    assert_eq!(listed[0].alias_addr, row.alias_addr);

    // Disable (state toggle) round-trips.
    store
        .set_masked_email_state("mask-1", "disabled")
        .await
        .unwrap();
    assert_eq!(
        store.get_masked_email("mask-1").await.unwrap().unwrap().state,
        "disabled"
    );

    // Soft-delete → 'deleted' state. The store RETAINS the row (audit/history); the
    // non-deleted VIEW is the route layer's `state != 'deleted'` filter (masked.rs
    // list_active) — asserted here by replicating that filter over the repo result.
    store
        .set_masked_email_state("mask-1", "deleted")
        .await
        .unwrap();
    let all = store.list_masked_email(account).await.unwrap();
    assert_eq!(
        all.iter().find(|r| r.id == "mask-1").map(|r| r.state.as_str()),
        Some("deleted"),
        "the store soft-deletes (retains the row with state='deleted')"
    );
    let active: Vec<_> = all.iter().filter(|r| r.state != "deleted").collect();
    assert!(
        active.is_empty(),
        "the route's non-deleted view (state != 'deleted') is empty after delete"
    );
}
