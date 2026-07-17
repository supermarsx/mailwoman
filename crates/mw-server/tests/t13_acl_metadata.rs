//! t13-E10 — IMAP ACL (RFC 4314) + METADATA (RFC 5464) live-E2E (26.13 workstream 2).
//!
//! The "unit-green ≠ wired" gate for E6 (mw-imap client commands), E7 (engine
//! MailboxRights/ServerMetadata seam) and E9 (the web revoke=null → DELETEACL
//! reconcile). E6/E7 proved the request framing + response parsing against in-crate
//! mock sockets; THIS leg round-trips every command against a REAL Dovecot 2.4.1
//! with the `acl` + `imap_acl` plugins and `imap_metadata` enabled — the true RFC
//! 4314/5464 enforcement point.
//!
//! Two surfaces are exercised:
//!   1. the E6 `mw_imap::session::Session` client methods directly (proves the raw
//!      response parsing — ACL lists, MYRIGHTS, and the METADATA literal form Dovecot
//!      actually returns), and
//!   2. the E7 engine `MailboxRights/*` + `ServerMetadata/*` JMAP methods against a
//!      live `ImapBackend`, incl. the E9 CRITICAL assertion: a revoke (rights=null)
//!      issues a real DELETEACL so the identifier is GONE from the ACL afterward —
//!      not left present with zero rights.
//!
//! ## Running (plaintext IMAP; no TLS needed for ACL/METADATA/ingest)
//!   docker compose -f docker-compose.ci.yml up -d --wait dovecot-t13
//!   MW_T13_LIVE=1 cargo test -p mw-server --test t13_acl_metadata -- --nocapture --test-threads=1

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::Engine;
use mw_engine::account::{AccountRuntime, MailSubmitter};
use mw_engine::backend::{AccountBackend, Result as EngineResult};
use mw_imap::session::{Credentials, Session};
use mw_imap::transport::TlsMode;
use mw_imap::{Credentials as ImapCredentials, ImapBackend, ImapConfig};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{
    AccountKind, Credentials as StoreCreds, MailboxUpsert, NewAccount, ServerKey, Store,
};

const IMAP_PLAINTEXT: u16 = 3143;
const USER: &str = "testuser";
const PASS: &str = "testpass";
const SHARED: &str = "Shared";

fn live() -> bool {
    std::env::var("MW_T13_LIVE").ok().as_deref() == Some("1")
}
fn host() -> String {
    std::env::var("MW_T13_HOST").unwrap_or_else(|_| "127.0.0.1".into())
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

async fn login_session(scenario: &str) -> Option<Session> {
    match Session::connect(&host(), IMAP_PLAINTEXT, TlsMode::Plaintext).await {
        Ok(mut s) => {
            s.probe_capabilities().await.ok()?;
            s.login(&Credentials::Password {
                username: USER.into(),
                password: PASS.into(),
            })
            .await
            .expect("SCRAM login to dovecot-t13");
            Some(s)
        }
        Err(e) => {
            eprintln!(
                "\n[t13 ACL/METADATA SKIP] {scenario}: dovecot-t13 unreachable at {}:{IMAP_PLAINTEXT} \
                 ({e}). Bring it up: docker compose -f docker-compose.ci.yml up -d --wait dovecot-t13.\n",
                host()
            );
            None
        }
    }
}

// ── 1. E6 session client: ACL round-trip incl. DELETEACL semantics ───────────────
#[tokio::test]
async fn session_acl_roundtrip_and_deleteacl_is_gone() {
    if !live() {
        eprintln!("\n[t13 ACL/METADATA SKIP] MW_T13_LIVE!=1 — real Dovecot not driven.\n");
        return;
    }
    let Some(mut s) = login_session("session_acl_roundtrip").await else {
        return;
    };

    // The owner holds the `a` (administer) right on its own mailbox.
    let rights = s.my_rights(SHARED).await.expect("MYRIGHTS Shared");
    assert!(
        rights.contains('a'),
        "owner must hold `a` on {SHARED}: {rights:?}"
    );

    // Grant a named identifier (Dovecot disallows `anyone`); GETACL reflects it.
    s.set_acl(SHARED, "alice", "lr")
        .await
        .expect("SETACL Shared alice lr");
    let acl = s.get_acl(SHARED).await.expect("GETACL after grant");
    let alice = acl.iter().find(|e| e.identifier == "alice");
    assert!(
        matches!(alice, Some(e) if e.rights.contains('l') && e.rights.contains('r')),
        "granted alice should appear with lr rights: {acl:?}"
    );

    // Revoke via DELETEACL — the E9 CRITICAL semantics: alice must be GONE entirely,
    // NOT present with an empty/zero-rights string.
    s.delete_acl(SHARED, "alice")
        .await
        .expect("DELETEACL Shared alice");
    let acl_after = s.get_acl(SHARED).await.expect("GETACL after delete");
    assert!(
        !acl_after.iter().any(|e| e.identifier == "alice"),
        "after DELETEACL, alice must be GONE from the ACL (not zero-rights): {acl_after:?}"
    );

    // LISTRIGHTS returns the rights tokens for an identifier.
    let lr = s
        .list_rights(SHARED, "alice")
        .await
        .expect("LISTRIGHTS Shared alice");
    assert!(
        !lr.is_empty(),
        "LISTRIGHTS should return the rights token list: {lr:?}"
    );

    let _ = s.logout().await;
}

// ── 2. E6 session client: METADATA round-trip ────────────────────────────────────
//
// ✅ FIXED (t13-26.13 fast-follow): Dovecot returns METADATA values as IMAP
// synchronizing literals (`* METADATA INBOX (/private/comment {9}\r\nhello t13)`),
// per RFC 5464. `crates/mw-imap/src/session.rs` now reassembles the raw reply with
// its inter-line CRLFs restored and its tokenizer is literal-aware, so `get_metadata`
// returns `"hello t13"` (not the `{9}` length marker). This round-trip asserts the
// CORRECT behavior against the real server; the former green defect-guard
// (`session_metadata_get_literal_value_bug_present`) was deleted with the fix.
#[tokio::test]
async fn session_metadata_roundtrip_mailbox_and_server_level() {
    if !live() {
        return;
    }
    let Some(mut s) = login_session("session_metadata_roundtrip").await else {
        return;
    };

    // Mailbox-level: SET then GET must return the exact value.
    s.set_metadata("INBOX", "/private/comment", Some("hello t13"))
        .await
        .expect("SETMETADATA INBOX /private/comment");
    let md = s
        .get_metadata("INBOX", &["/private/comment".to_string()])
        .await
        .expect("GETMETADATA INBOX /private/comment");
    let entry = md.iter().find(|e| e.entry == "/private/comment");
    assert_eq!(
        entry.and_then(|e| e.value.as_deref()),
        Some("hello t13"),
        "GETMETADATA must round-trip the value Dovecot returns as a literal: {md:?}"
    );

    // Server-level scope (empty mailbox name, RFC 5464).
    s.set_metadata("", "/private/vendor/mw", Some("srv-value"))
        .await
        .expect("SETMETADATA server-level");
    let smd = s
        .get_metadata("", &["/private/vendor/mw".to_string()])
        .await
        .expect("GETMETADATA server-level");
    assert_eq!(
        smd.iter()
            .find(|e| e.entry == "/private/vendor/mw")
            .and_then(|e| e.value.as_deref()),
        Some("srv-value"),
        "server-level GETMETADATA must round-trip: {smd:?}"
    );

    // NIL removal: SET value=None then GET yields value None.
    s.set_metadata("INBOX", "/private/comment", None)
        .await
        .expect("SETMETADATA NIL (remove)");
    let removed = s
        .get_metadata("INBOX", &["/private/comment".to_string()])
        .await
        .expect("GETMETADATA after remove");
    assert!(
        removed
            .iter()
            .find(|e| e.entry == "/private/comment")
            .and_then(|e| e.value.as_deref())
            .is_none(),
        "after NIL SETMETADATA the entry value must be None: {removed:?}"
    );

    let _ = s.logout().await;
}

// ── 3. E7 engine seam: MailboxRights/* + ServerMetadata/* via a live backend ──────
async fn engine_with_live_backend() -> Option<(Arc<Engine>, String, String)> {
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
            eprintln!("\n[t13 ACL/METADATA SKIP] engine backend connect failed: {e}\n");
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
    // The engine resolves a JMAP mailboxId → RawMailboxRef via the store; upsert the
    // Shared mailbox so MailboxRights/* targets "Shared" on the backend.
    let mailbox_id = store
        .upsert_mailbox(&MailboxUpsert {
            account_id: &account_id,
            name: SHARED,
            role: None,
            uidvalidity: 1,
            uidnext: 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        })
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
    Some((engine, account_id, mailbox_id))
}

async fn jmap(engine: &Engine, account_id: &str, method: &str, args: Value) -> Value {
    let req = json!({ "methodCalls": [[method, args, "c0"]] });
    let resp = engine.handle_jmap(account_id, &req).await;
    resp["methodResponses"][0][1].clone()
}

#[tokio::test]
async fn engine_mailbox_rights_grant_then_revoke_deleteacl() {
    if !live() {
        return;
    }
    let Some((engine, account_id, mailbox_id)) = engine_with_live_backend().await else {
        return;
    };

    // MailboxRights/get surfaces myRights (must include `a`) + the ACL list.
    let got = jmap(
        &engine,
        &account_id,
        "MailboxRights/get",
        json!({ "mailboxId": mailbox_id }),
    )
    .await;
    assert!(
        got["myRights"].as_str().unwrap_or("").contains('a'),
        "engine MailboxRights/get.myRights must include `a`: {got}"
    );

    // Grant bob via MailboxRights/set (rights present ⇒ SETACL).
    let set = jmap(
        &engine,
        &account_id,
        "MailboxRights/set",
        json!({ "mailboxId": mailbox_id, "identifier": "bob", "rights": "lrs" }),
    )
    .await;
    assert!(
        set.get("updated").and_then(|u| u.get("bob")).is_some(),
        "grant should be in `updated`: {set}"
    );
    let after_grant = jmap(
        &engine,
        &account_id,
        "MailboxRights/get",
        json!({ "mailboxId": mailbox_id }),
    )
    .await;
    assert!(
        after_grant["acl"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["identifier"] == "bob"),
        "bob must appear in the ACL after grant: {after_grant}"
    );

    // Revoke via rights=null ⇒ DELETEACL (E9). bob must be GONE, not zero-rights.
    let revoke = jmap(
        &engine,
        &account_id,
        "MailboxRights/set",
        json!({ "mailboxId": mailbox_id, "identifier": "bob", "rights": Value::Null }),
    )
    .await;
    assert!(
        revoke.get("updated").and_then(|u| u.get("bob")).is_some(),
        "revoke should be in `updated`: {revoke}"
    );
    let after_revoke = jmap(
        &engine,
        &account_id,
        "MailboxRights/get",
        json!({ "mailboxId": mailbox_id }),
    )
    .await;
    assert!(
        !after_revoke["acl"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["identifier"] == "bob"),
        "after revoke (DELETEACL), bob must be GONE from the engine's ACL view: {after_revoke}"
    );
}

// Same METADATA-literal path, surfaced through the E7 engine seam: the
// ServerMetadata/get response now reassembles the `{n}` literal (fixed in
// `mw-imap` `parse_metadata`), so this round-trip asserts the correct value. The
// former green defect-guard (`engine_server_metadata_get_literal_value_bug_present`)
// was deleted with the fix.
#[tokio::test]
async fn engine_server_metadata_set_get_and_remove() {
    if !live() {
        return;
    }
    let Some((engine, account_id, _mailbox_id)) = engine_with_live_backend().await else {
        return;
    };

    let set = jmap(
        &engine,
        &account_id,
        "ServerMetadata/set",
        json!({ "mailboxId": Value::Null, "entry": "/private/vendor/mw-engine", "value": "abc" }),
    )
    .await;
    assert!(
        set.get("updated")
            .and_then(|u| u.get("/private/vendor/mw-engine"))
            .is_some(),
        "server metadata set should be in `updated`: {set}"
    );
    let got = jmap(
        &engine,
        &account_id,
        "ServerMetadata/get",
        json!({ "mailboxId": Value::Null, "entries": ["/private/vendor/mw-engine"] }),
    )
    .await;
    let list = got["list"].as_array().unwrap();
    assert_eq!(
        list.iter()
            .find(|e| e["entry"] == "/private/vendor/mw-engine")
            .and_then(|e| e["value"].as_str()),
        Some("abc"),
        "server-level ServerMetadata/get must round-trip the value: {got}"
    );
}
