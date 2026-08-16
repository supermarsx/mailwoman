//! 26.20 `t22-e13` — `ContactCard/merge` and the request the web client has
//! actually been sending since it shipped.
//!
//! ## What was broken
//!
//! The client emits `{accountId, keepId, mergeIds}`
//! (`apps/web/src/modules/contacts/api.ts`). The engine read `args["ids"]`,
//! defaulted it to empty, and rejected anything shorter than two — so **every
//! merge the product could issue failed**, with a `serverFail` the UI swallowed.
//! Two further divergences sat behind that one, and a fix that only renamed the
//! argument would have shipped both:
//!
//! * **Survivor.** The engine minted a *new* card and tombstoned every source,
//!   `keepId` included. The client names a card to keep and patches the response
//!   into that id's slot. Neither `keepId` nor `ids[0]` survived.
//! * **Response.** The engine answered `{merged: <id string>, tombstoned: [..]}`;
//!   the client is typed for `{merged: <card object>, destroyed: [..]}` and
//!   spreads `merged` into its store. A request-parsing-only fix passes a
//!   server-side test and still leaves a string where a card belongs.
//!
//! ## Why these assertions are on the server's response
//!
//! Asserting the *client's* request shape proves nothing here — the client was
//! already correct, and was already sending exactly this. The bug was only ever
//! visible from the far side, so every assertion below reads what the engine
//! returned and what the store holds afterwards.
//!
//! **`the_survivor_helper_can_tell_two_survivors_apart` is the negative
//! control.** `survivor_id` is the helper the main assertion leans on; a helper
//! that quietly returned the id it was handed, or read a missing field as a
//! match, would make every other test in this file vacuous. That test runs the
//! same helper over the legacy form — which deliberately does *not* keep its
//! first source — and requires it to report a different id. It fails if the
//! helper stops being able to distinguish.
//!
//! ## Recorded against `master` (`224fd1a`) before the fix
//!
//! `3 failed; 2 passed`. The two `keepId` tests failed on the real bug — the
//! engine never reached its merge logic because `ids` was absent:
//!
//! ```text
//! merge_with_keep_id_keeps_that_card_and_destroys_only_the_others
//!   panicked: merge returned an error: {"description":"ContactCard/merge
//!   requires at least two ids","type":"serverFail"}
//! the_survivor_helper_can_tell_two_survivors_apart
//!   panicked: keepId merge returned an error: (same serverFail)
//! legacy_ids_form_still_mints_a_new_card_and_tombstones_every_source
//!   panicked: `destroyed` missing from {"accountId":"…","merged":
//!   "contact-18cc5ffc013ba0006","tombstoned":["contact-…c2","contact-…44"]}
//! ```
//!
//! **The legacy test failed on `master` only on the additively-added
//! `destroyed` field.** Its behavioural assertions — a new id, both sources
//! tombstoned, the union of both addresses, `tombstoned` listing both — all held
//! on `master` and all still hold. That is what makes it a control on the old
//! shape rather than a second test of the new one.
//!
//! **The two refusal tests passed on `master`, and not for a good reason:** on
//! `master` *every* `keepId` request was refused, so "a refused merge destroys
//! nothing" was trivially true. They are not evidence of the fix; they are
//! guards on the paths the fix introduces (self-tombstoning via `mergeIds`
//! containing `keepId`, and a half-applied merge when one id is unknown), and
//! they would have passed on `master` no matter what. Said plainly here because
//! a green count is not the same as a test that could have gone red.

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

/// A backend that serves no mail — PIM methods never call it, but a registered
/// runtime is required for `handle_jmap` to dispatch.
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
struct NoopSubmitter {
    sent: Mutex<Vec<Outgoing>>,
}

#[async_trait]
impl MailSubmitter for NoopSubmitter {
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
    let runtime = AccountRuntime::new(
        Arc::new(NoopBackend) as Arc<dyn AccountBackend>,
        Arc::new(NoopSubmitter::default()) as Arc<dyn MailSubmitter>,
        "me@example.org",
    );
    engine.register_backend(account_id.clone(), runtime);
    Harness { engine, account_id }
}

impl Harness {
    /// Invoke one JMAP method and return its response arguments object.
    async fn call(&self, method: &str, args: Value) -> Value {
        let req = json!({ "methodCalls": [[method, args, "c0"]] });
        let resp = self.engine.handle_jmap(&self.account_id, &req).await;
        resp["methodResponses"][0][1].clone()
    }

    /// Create a card and return its server-assigned id.
    async fn card(&self, full: &str, email: &str) -> String {
        let set = self
            .call(
                "ContactCard/set",
                json!({ "create": { "c": {
                    "name": { "full": full },
                    "emails": [{ "context": "work", "value": email, "pref": 1 }],
                }}}),
            )
            .await;
        set["created"]["c"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("create rejected: {set}"))
            .to_string()
    }

    /// Ids the account currently holds, via `ContactCard/get` with no `ids`.
    async fn all_ids(&self) -> Vec<String> {
        let got = self
            .call("ContactCard/get", json!({ "ids": Value::Null }))
            .await;
        got["list"]
            .as_array()
            .expect("list")
            .iter()
            .filter_map(|c| c["id"].as_str().map(String::from))
            .collect()
    }

    /// Whether `id` still resolves to a stored card.
    async fn exists(&self, id: &str) -> bool {
        let got = self.call("ContactCard/get", json!({ "ids": [id] })).await;
        got["list"].as_array().is_some_and(|l| !l.is_empty())
    }

    /// Every email value on a stored card, sorted.
    async fn emails(&self, id: &str) -> Vec<String> {
        let got = self.call("ContactCard/get", json!({ "ids": [id] })).await;
        let mut v: Vec<String> = got["list"][0]["emails"]
            .as_array()
            .unwrap_or_else(|| panic!("no emails on {id}: {got}"))
            .iter()
            .filter_map(|e| e["value"].as_str().map(String::from))
            .collect();
        v.sort();
        v
    }
}

// ── helpers the assertions lean on ───────────────────────────────────────────

/// Fail loudly on a `serverFail`, so a broken merge cannot be read as an empty
/// success by a later `.get(..)` that quietly returns `None`.
fn ok(resp: &Value, what: &str) -> Value {
    assert_ne!(
        resp["type"].as_str(),
        Some("serverFail"),
        "{what} returned an error: {resp}"
    );
    resp.clone()
}

/// The id of the card that survived a merge, whichever response shape carried
/// it: the new form answers with the survivor as a full card object, the legacy
/// form with a bare id string. Panics rather than defaulting — a helper that
/// answered `""` for a missing field would make every caller vacuously true.
fn survivor_id(resp: &Value) -> String {
    match &resp["merged"] {
        Value::String(s) => s.clone(),
        Value::Object(o) => o
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("merged card carries no id: {resp}"))
            .to_string(),
        other => panic!("merge response has no usable `merged`: {other} in {resp}"),
    }
}

fn ids(resp: &Value, field: &str) -> Vec<String> {
    resp[field]
        .as_array()
        .unwrap_or_else(|| panic!("`{field}` missing from {resp}"))
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect()
}

// ── the contract the client speaks ───────────────────────────────────────────

#[tokio::test]
async fn merge_with_keep_id_keeps_that_card_and_destroys_only_the_others() {
    let h = setup().await;
    let keep = h.card("Ada Lovelace", "ada@example.org").await;
    let dup1 = h.card("Ada Lovelace", "ada@home.test").await;
    let dup2 = h.card("A. Lovelace", "ada@work.test").await;

    // Exactly the request `contactMerge()` in the web client builds.
    let resp = ok(
        &h.call(
            "ContactCard/merge",
            json!({ "keepId": keep, "mergeIds": [dup1, dup2] }),
        )
        .await,
        "merge",
    );

    // The survivor is the card the caller named — not a freshly minted id, and
    // not `mergeIds[0]`.
    assert_eq!(
        survivor_id(&resp),
        keep,
        "the survivor must be keepId: {resp}"
    );
    assert_eq!(resp["keptId"].as_str(), Some(keep.as_str()));

    // The server's own account of what it destroyed.
    let mut destroyed = ids(&resp, "destroyed");
    destroyed.sort();
    let mut expected = vec![dup1.clone(), dup2.clone()];
    expected.sort();
    assert_eq!(destroyed, expected, "destroyed must be exactly mergeIds");
    assert!(
        !destroyed.contains(&keep),
        "the kept card must never appear in `destroyed`"
    );

    // And what the store actually holds: the survivor and nothing else. This is
    // what separates "kept keepId" from "kept keepId *and* also minted a new
    // card" — a response-only assertion cannot see the difference.
    assert!(h.exists(&keep).await, "the kept card must still be stored");
    assert!(!h.exists(&dup1).await, "dup1 must be tombstoned");
    assert!(!h.exists(&dup2).await, "dup2 must be tombstoned");
    assert_eq!(
        h.all_ids().await,
        vec![keep.clone()],
        "exactly one card must remain, and it must be the kept one"
    );

    // The merge did its actual job: the survivor carries every source's email.
    assert_eq!(
        h.emails(&keep).await,
        vec![
            "ada@example.org".to_string(),
            "ada@home.test".to_string(),
            "ada@work.test".to_string()
        ],
        "the survivor must union all three addresses"
    );

    // The response carries the survivor as a card, not an id — the client
    // patches this object straight into its store.
    assert!(
        resp["merged"].is_object(),
        "`merged` must be the card itself: {resp}"
    );
    assert_eq!(
        resp["merged"]["name"]["full"].as_str(),
        Some("Ada Lovelace")
    );
    assert_eq!(
        resp["merged"]["emails"].as_array().map(Vec::len),
        Some(3),
        "the returned card must be the merged one, not the pre-merge card"
    );
}

#[tokio::test]
async fn legacy_ids_form_still_mints_a_new_card_and_tombstones_every_source() {
    let h = setup().await;
    let a = h.card("Grace Hopper", "grace@example.org").await;
    let b = h.card("Grace Hopper", "grace@navy.test").await;

    let resp = ok(
        &h.call("ContactCard/merge", json!({ "ids": [a, b] })).await,
        "legacy merge",
    );

    // Unchanged from before this lane: a new id, both sources gone, and the two
    // pre-existing response fields still populated as they were.
    let new_id = survivor_id(&resp);
    assert!(resp["merged"].is_string(), "legacy `merged` stays an id");
    assert_ne!(new_id, a);
    assert_ne!(new_id, b);
    assert_eq!(ids(&resp, "tombstoned"), vec![a.clone(), b.clone()]);
    assert_eq!(
        ids(&resp, "destroyed"),
        vec![a.clone(), b.clone()],
        "`destroyed` is reported by both shapes"
    );

    assert!(!h.exists(&a).await);
    assert!(!h.exists(&b).await);
    assert_eq!(h.all_ids().await, vec![new_id.clone()]);
    assert_eq!(
        h.emails(&new_id).await,
        vec![
            "grace@example.org".to_string(),
            "grace@navy.test".to_string()
        ],
    );
}

/// NEGATIVE CONTROL for `survivor_id`. Every assertion above rests on this
/// helper reporting the id the engine really kept. Run it over the two shapes
/// in one test and require the answers to differ in exactly the way the two
/// designs differ: the new form's survivor **is** an input id, the legacy
/// form's survivor **is not** any input id. A helper that echoed its input, or
/// that read a missing field as a match, cannot satisfy both halves.
#[tokio::test]
async fn the_survivor_helper_can_tell_two_survivors_apart() {
    let h = setup().await;

    let keep = h.card("Kept", "kept@example.org").await;
    let gone = h.card("Gone", "gone@example.org").await;
    let kept_resp = h
        .call(
            "ContactCard/merge",
            json!({ "keepId": keep, "mergeIds": [gone] }),
        )
        .await;
    let kept_survivor = survivor_id(&ok(&kept_resp, "keepId merge"));

    let x = h.card("X", "x@example.org").await;
    let y = h.card("Y", "y@example.org").await;
    let legacy_resp = h.call("ContactCard/merge", json!({ "ids": [x, y] })).await;
    let legacy_survivor = survivor_id(&ok(&legacy_resp, "legacy merge"));

    assert_eq!(
        kept_survivor, keep,
        "keepId form: survivor is the named input"
    );
    assert!(
        legacy_survivor != x && legacy_survivor != y,
        "legacy form: survivor is a new card, not an input — if this reads as a \
         match, `survivor_id` is not reading a real value"
    );
    assert_ne!(
        kept_survivor, legacy_survivor,
        "the helper must distinguish the two survivors"
    );
}

// ── refusals: a merge that cannot proceed must not destroy anything ──────────

#[tokio::test]
async fn merge_refuses_without_a_second_card_and_keeps_the_first() {
    let h = setup().await;
    let only = h.card("Solo", "solo@example.org").await;

    for args in [
        json!({ "keepId": only, "mergeIds": [] }),
        // `keepId` listed among the ids to merge away is not a licence to
        // tombstone the survivor.
        json!({ "keepId": only, "mergeIds": [only] }),
        json!({ "keepId": only }),
    ] {
        let resp = h.call("ContactCard/merge", args.clone()).await;
        assert_eq!(
            resp["type"].as_str(),
            Some("serverFail"),
            "{args} should have been refused, got {resp}"
        );
        assert!(
            h.exists(&only).await,
            "a refused merge must leave the card alone ({args})"
        );
    }
    assert_eq!(h.all_ids().await, vec![only]);
}

#[tokio::test]
async fn merge_with_an_unknown_id_destroys_nothing() {
    let h = setup().await;
    let keep = h.card("Ada", "ada@example.org").await;
    let dup = h.card("Ada", "ada@home.test").await;

    let resp = h
        .call(
            "ContactCard/merge",
            json!({ "keepId": keep, "mergeIds": [dup, "contact_does_not_exist"] }),
        )
        .await;
    assert_eq!(resp["type"].as_str(), Some("serverFail"), "{resp}");

    // Every source is resolved before anything is written, so a bad id in the
    // list cannot leave a half-applied merge behind.
    assert!(h.exists(&keep).await, "keep survived the refusal");
    assert!(
        h.exists(&dup).await,
        "dup was not tombstoned by a failed merge"
    );
    assert_eq!(h.emails(&keep).await, vec!["ada@example.org".to_string()]);
}
