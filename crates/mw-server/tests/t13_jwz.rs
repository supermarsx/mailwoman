//! t13-E10 — JWZ threading live-E2E (26.13 workstream 3).
//!
//! Two legs:
//!   1. **Canonical corpora (always-on, deterministic)** — parse the committed
//!      `fixtures/threads/*.eml` corpus and run the shipped `mw_engine::thread::thread`
//!      full JWZ over it, asserting the container forest groups messages correctly:
//!      linear nesting (References), In-Reply-To linking, subject-gather merge, and a
//!      missing-root empty-container group. (E4's 18 in-crate tests pin the algorithm;
//!      this pins it over a file corpus a reader can inspect.)
//!   2. **Live ingest (gated) on SQLite AND live Postgres** — resync the seeded
//!      threaded corpus from a REAL Dovecot into an `Engine`, then assert the stored
//!      `threadId`s: a reply ingested BEFORE its original converges with it; a
//!      truncated-References follow-up is repaired onto the same thread via the store
//!      sibling lookup; a standalone message is its own thread. This is the "wired"
//!      proof through the real store on both dialects (the E4 seam is per-message,
//!      not the set algorithm — the store path is what the live leg exercises).
//!
//! ## Running
//!   cargo test -p mw-server --test t13_jwz                       # leg 1 always runs
//!   docker compose -f docker-compose.ci.yml up -d --wait dovecot-t13
//!   MW_T13_LIVE=1 [MW_E14_PG_DSN=postgres://…] \
//!     cargo test -p mw-server --test t13_jwz -- --nocapture --test-threads=1

use std::collections::BTreeSet;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;

use mw_engine::Engine;
use mw_engine::account::{AccountRuntime, MailSubmitter};
use mw_engine::backend::{AccountBackend, Result as EngineResult};
use mw_engine::thread::{Message, ThreadNode, thread};
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

// ── Leg 1: canonical corpora from fixtures/threads/*.eml (deterministic) ──────────

/// Minimal header parse (Message-ID / In-Reply-To / References / Subject) — enough
/// to build a JWZ `Message` from a fixture `.eml`.
fn parse_fixture(raw: &str) -> Message {
    let mut m = Message::default();
    let angles = |s: &str| -> Vec<String> {
        s.split_whitespace()
            .filter_map(|t| t.trim().strip_prefix('<').and_then(|t| t.strip_suffix('>')))
            .map(|t| t.to_string())
            .collect()
    };
    for line in raw.lines() {
        if line.is_empty() {
            break; // end of headers
        }
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("message-id:") {
            m.message_id = angles(line).into_iter().next();
        } else if lower.starts_with("in-reply-to:") {
            m.in_reply_to = angles(line).into_iter().next();
        } else if lower.starts_with("references:") {
            m.references = angles(line);
        } else if lower.starts_with("subject:")
            && let Some((_, v)) = line.split_once(':')
        {
            m.subject = Some(v.trim().to_string());
        }
    }
    m
}

fn load_corpus() -> Vec<Message> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/threads");
    let mut msgs = Vec::new();
    for name in [
        "a-widget.eml",
        "b-widget-reply.eml",
        "c-widget-inreplyto.eml",
        "d-widget-subjectgather.eml",
        "o1-orphan.eml",
        "o2-orphan.eml",
    ] {
        let raw = std::fs::read_to_string(format!("{dir}/{name}"))
            .unwrap_or_else(|e| panic!("read fixture {name}: {e}"));
        msgs.push(parse_fixture(&raw));
    }
    msgs
}

/// Flatten a thread node into the set of concrete Message-IDs it contains.
fn ids_in(node: &ThreadNode, out: &mut BTreeSet<String>) {
    if let Some(id) = &node.message_id {
        out.insert(id.clone());
    }
    for c in &node.children {
        ids_in(c, out);
    }
}

#[test]
fn jwz_canonical_corpus_groups_threads() {
    let corpus = load_corpus();
    // Sanity: the fixtures parsed as expected.
    assert_eq!(
        corpus[0].message_id.as_deref(),
        Some("a@jwz.t13"),
        "fixture a parsed: {:?}",
        corpus[0]
    );
    assert_eq!(
        corpus[2].in_reply_to.as_deref(),
        Some("b@jwz.t13"),
        "fixture c In-Reply-To: {:?}",
        corpus[2]
    );
    assert_eq!(
        corpus[4].references,
        vec!["ghost-root@jwz.t13".to_string()],
        "fixture o1 refs: {:?}",
        corpus[4]
    );

    let forest = thread(&corpus);
    let root_sets: Vec<BTreeSet<String>> = forest
        .iter()
        .map(|r| {
            let mut s = BTreeSet::new();
            ids_in(r, &mut s);
            s
        })
        .collect();

    let widget = root_sets
        .iter()
        .find(|s| s.contains("a@jwz.t13"))
        .expect("a Widget thread exists");
    // Linear nesting (a>b>c) + In-Reply-To (c) + subject-gather (d) all in one thread.
    for id in ["a@jwz.t13", "b@jwz.t13", "c@jwz.t13", "d@jwz.t13"] {
        assert!(
            widget.contains(id),
            "{id} must be in the Widget thread; got {widget:?}"
        );
    }

    let orphan = root_sets
        .iter()
        .find(|s| s.contains("o1@jwz.t13"))
        .expect("an Orphan thread exists");
    assert!(
        orphan.contains("o2@jwz.t13"),
        "o1+o2 converge under the missing-root container: {orphan:?}"
    );
    assert!(
        !orphan.contains("a@jwz.t13") && !widget.contains("o1@jwz.t13"),
        "the Widget and Orphan threads are distinct"
    );
}

// ── Leg 2: live ingest → stored threadId on SQLite AND live Postgres ──────────────

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

/// The JMAP mailbox id of the account's INBOX (role == "inbox").
async fn inbox_mailbox_id(engine: &Engine, account_id: &str) -> String {
    let mb = engine
        .handle_jmap(
            account_id,
            &json!({ "methodCalls": [["Mailbox/get", {}, "mb"]] }),
        )
        .await;
    mb["methodResponses"][0][1]["list"]
        .as_array()
        .expect("Mailbox/get list")
        .iter()
        .find(|m| m["role"] == "inbox")
        .and_then(|m| m["id"].as_str().map(String::from))
        .expect("an INBOX mailbox after resync")
}

/// Resync the seeded corpus from Dovecot into an Engine backed by `store`, and
/// return the ingested INBOX Email objects (each carries `threadId` + `preview`).
/// (Email/get does not emit a JMAP `messageId`, so the invariants identify messages
/// by their unique `preview` text.)
async fn ingest_emails(store: Store) -> Vec<serde_json::Value> {
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
    let backend = ImapBackend::connect(cfg)
        .await
        .expect("connect dovecot-t13");
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

    // Resolve the INBOX mailbox id, then Email/query {inMailbox} → ids (an unfiltered
    // Email/query returns nothing — the SQL fast path requires an inMailbox filter).
    let inbox_id = inbox_mailbox_id(&engine, &account_id).await;
    let q = engine
        .handle_jmap(
            &account_id,
            &json!({ "methodCalls": [["Email/query", { "filter": { "inMailbox": inbox_id } }, "q"]] }),
        )
        .await;
    let ids: Vec<String> = q["methodResponses"][0][1]["ids"]
        .as_array()
        .expect("Email/query ids")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert!(
        !ids.is_empty(),
        "resync ingested at least one INBOX message"
    );
    let g = engine
        .handle_jmap(
            &account_id,
            &json!({ "methodCalls": [["Email/get", { "ids": ids, "properties": ["threadId", "subject", "preview"] }, "g"]] }),
        )
        .await;
    g["methodResponses"][0][1]["list"]
        .as_array()
        .cloned()
        .expect("Email/get list")
}

/// The `threadId` of the ingested message whose `preview` contains `needle`.
fn tid_by_preview(emails: &[serde_json::Value], needle: &str, dialect: &str) -> String {
    emails
        .iter()
        .find(|e| e["preview"].as_str().unwrap_or("").contains(needle))
        .and_then(|e| e["threadId"].as_str().map(String::from))
        .unwrap_or_else(|| {
            panic!("[{dialect}] no ingested message with preview containing {needle:?}")
        })
}

/// Assert the JWZ convergence/repair invariants over the ingested Email objects.
fn assert_thread_invariants(emails: &[serde_json::Value], dialect: &str) {
    let origin = tid_by_preview(emails, "The original message, ingested AFTER", dialect);
    let reply = tid_by_preview(emails, "The reply arrives", dialect);
    let followup = tid_by_preview(emails, "A follow-up with a TRUNCATED", dialect);
    let standalone = tid_by_preview(emails, "A message with no References", dialect);

    // Reply ingested BEFORE its original converges with it.
    assert_eq!(
        reply, origin,
        "[{dialect}] reply must share the origin's thread (convergence)"
    );
    // Truncated-References follow-up is repaired onto the same thread.
    assert_eq!(
        followup, origin,
        "[{dialect}] truncated-References follow-up must be repaired onto the origin thread"
    );
    // A standalone message is its own thread.
    assert_ne!(
        standalone, origin,
        "[{dialect}] standalone message must be a distinct thread"
    );
}

#[tokio::test]
async fn jwz_live_ingest_convergence_sqlite() {
    if !live() {
        eprintln!(
            "\n[t13 JWZ SKIP] MW_T13_LIVE!=1 — live ingest not driven (leg 1 corpora still ran).\n"
        );
        return;
    }
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let emails = ingest_emails(store).await;
    assert_thread_invariants(&emails, "sqlite");
}

#[tokio::test]
async fn jwz_live_ingest_convergence_postgres() {
    if !live() {
        return;
    }
    let Ok(dsn) = std::env::var("MW_E14_PG_DSN") else {
        eprintln!("\n[t13 JWZ SKIP] MW_E14_PG_DSN unset — Postgres thread-id leg not driven.\n");
        return;
    };
    let store = Store::open(&dsn, ServerKey::generate())
        .await
        .expect("open live Postgres store");
    let emails = ingest_emails(store).await;
    assert_thread_invariants(&emails, "postgres");
}
