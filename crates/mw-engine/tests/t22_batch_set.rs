//! 26.20 `t22-e3s` — `Email/set` over a multi-id `update` must cost **one**
//! search-index commit and **one** state increment however many ids it touches,
//! and its SQL cost must not depend on the id count at all.
//!
//! It still writes a change row **per id** — all sharing that one state — because
//! one state bump is what makes the batch cheap while a row per id is what keeps
//! `Email/changes` truthful. Collapsing to a single row satisfies the first and
//! silently breaks the second.
//!
//! ## Why the instruments here are counts and not timings
//!
//! The measured cost of a large `Email/set` was a full Tantivy `commit()` +
//! `reader.reload()` **per message** inside `reindex_message` — 9.5 s on SQLite
//! and 26.5 s on Postgres for 500 ids. A fix that batches the transaction and
//! the state bump but leaves the per-id re-index in place moves 9.5 s to ~9.4 s
//! and passes any timing assertion. So no assertion in this file is a timing.
//!
//! Three ways a naive version of these assertions passes falsely, each closed
//! here deliberately:
//!
//! 1. **Counting the public batch method instead of the commit.** A counter on
//!    `Index::upsert_batch` reads `1` for a caller that hands over 500 documents
//!    even if the implementation still commits per document. The counter this
//!    file reads ([`mw_search::commit_count`]) is incremented on the single
//!    `IndexWriter::commit` call site in `mw-search`, so it reports what the
//!    index did rather than how the caller was written.
//! 2. **Deleting the re-index.** `commits == 1` is also satisfied by `commits ==
//!    0`, and outright by dropping the re-index from the set path — which is the
//!    fastest way to pass a count-only assertion and it silently breaks search.
//!    Every commit-count assertion here is therefore paired with an index-backed
//!    search whose result depends on the *updated* state
//!    (`batched_set_keeps_the_index_truthful`).
//! 3. **Bounds instead of constants.** `commits(500) == commits(5) == 1` is
//!    asserted as an equality at two different loads, so a per-id path cannot
//!    hide inside a generous bound. The SQL assertion is the same shape —
//!    `stmts(500) == stmts(5)` — plus the removed statements named individually
//!    and required to be zero. It was a per-id *average* first, and that was
//!    wrong: an average moves whenever any peer changes store internals, and it
//!    did, twice.
//!
//! Recorded against `master` (`57046c7`) before the fix, with only the counter
//! added: **500 commits for 500 ids** (one per id, at every id count tried —
//! 5, 10, 25, 70, 500) and **4 514 SQL statements for 500 ids = 9.03/id**, with
//! state advancing by 500 and `Email/changes` reporting all 500 across 500
//! states. Six of the eight tests then present failed on that run; the two that
//! passed are the calibration control and the all-ids-fail edge.
//!
//! Now: **1 commit · 11 statements at any id count · state +1 · all 500 ids
//! reported at that one state.**

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, Flag, MailboxDelta, MailboxRole, MessageRef,
    MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor, WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

const UIDVALIDITY: u32 = 100;

// ---------------------------------------------------------------------------
// The SQL-statement counter
// ---------------------------------------------------------------------------
//
// `sqlx` emits one `tracing` event per executed statement on the target
// `sqlx::query`. Counting those events is load-independent: it does not move
// with machine load, unlike every wall-clock number. The subscriber is
// hand-rolled over `tracing` itself rather than pulled from `tracing-subscriber`
// so this test adds no dependency.
//
// It is installed globally and once, because `sqlx`'s SQLite driver may emit
// from a connection worker thread, which a thread-local default dispatcher would
// miss. Tests snapshot the counter around the call under test rather than
// resetting it, so ordering between tests does not matter.

static SQL_STATEMENTS: AtomicU64 = AtomicU64::new(0);
static SQL_TEXT: Mutex<Vec<String>> = Mutex::new(Vec::new());
static SUBSCRIBER: OnceLock<()> = OnceLock::new();

struct SqlCounter;

/// Pulls `sqlx`'s `db.statement` field out of the event so a specific statement
/// can be asserted present or absent, not merely counted.
struct StatementVisitor;

impl tracing::field::Visit for StatementVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "db.statement" {
            SQL_TEXT.lock().unwrap().push(format!("{value:?}"));
        }
    }
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "db.statement" {
            SQL_TEXT.lock().unwrap().push(value.to_string());
        }
    }
}

impl tracing::Subscriber for SqlCounter {
    fn enabled(&self, meta: &tracing::Metadata<'_>) -> bool {
        meta.target().starts_with("sqlx::query")
    }
    fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::Id {
        tracing::Id::from_u64(1)
    }
    fn record(&self, _: &tracing::Id, _: &tracing::span::Record<'_>) {}
    fn record_follows_from(&self, _: &tracing::Id, _: &tracing::Id) {}
    fn event(&self, event: &tracing::Event<'_>) {
        if event.metadata().target().starts_with("sqlx::query") {
            SQL_STATEMENTS.fetch_add(1, Ordering::Relaxed);
            event.record(&mut StatementVisitor);
        }
    }
    fn enter(&self, _: &tracing::Id) {}
    fn exit(&self, _: &tracing::Id) {}
}

fn install_sql_counter() {
    SUBSCRIBER.get_or_init(|| {
        // A failure here means something else in the process already installed a
        // global subscriber; the counter would then read zero and every
        // statement-count assertion would pass vacuously, so refuse loudly.
        tracing::subscriber::set_global_default(SqlCounter)
            .expect("no other global tracing subscriber in this test binary");
    });
}

fn sql_count() -> u64 {
    SQL_STATEMENTS.load(Ordering::Relaxed)
}

/// The statements recorded since the last [`take_sql_text`], collapsed to a
/// `(first two words, count)` histogram — enough to attribute a per-id cost to a
/// call site without dumping thousands of lines.
fn take_sql_text() -> Vec<(String, usize)> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    for s in SQL_TEXT.lock().unwrap().drain(..) {
        let key: String = s.split_whitespace().take(6).collect::<Vec<_>>().join(" ");
        *seen.entry(key).or_default() += 1;
    }
    let mut v: Vec<(String, usize)> = seen.into_iter().collect();
    // Descending by count. Presentational only: every consumer is `count_of`,
    // which filters by SQL prefix and sums, so no assertion can depend on this
    // order — it decides what a failure message shows first.
    v.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    v
}

// ---------------------------------------------------------------------------
// Backend
// ---------------------------------------------------------------------------

type ScriptMsg = (u32, Vec<u8>, Vec<Flag>, String);

struct FakeBackend {
    messages: Mutex<HashMap<String, Vec<ScriptMsg>>>,
}

impl FakeBackend {
    fn with_inbox(n: usize) -> Self {
        let msgs: Vec<ScriptMsg> = (0..n)
            .map(|i| {
                (
                    i as u32 + 1,
                    seeded_msg(i),
                    Vec::new(),
                    format!("2026-07-01T09:{:02}:00Z", i % 60),
                )
            })
            .collect();
        let mut messages = HashMap::new();
        messages.insert("INBOX".to_string(), msgs);
        messages.insert("Archive".to_string(), Vec::new());
        Self {
            messages: Mutex::new(messages),
        }
    }
}

fn seeded_msg(i: usize) -> Vec<u8> {
    format!(
        "Message-ID: <seed-{i}@x>\r\n\
         From: alice@example.org\r\n\
         To: me@example.org\r\n\
         Subject: Seeded {i}\r\n\
         Date: Wed, 01 Jul 2026 09:00:00 +0000\r\n\
         \r\n\
         body of seeded message {i}\r\n"
    )
    .into_bytes()
}

#[async_trait]
impl AccountBackend for FakeBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps {
            uidplus: true,
            r#move: true,
            special_use: true,
            ..BackendCaps::default()
        })
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        let st = self.messages.lock().unwrap();
        Ok([
            ("INBOX", MailboxRole::Inbox),
            ("Archive", MailboxRole::Archive),
        ]
        .into_iter()
        .map(|(name, role)| {
            let total = st.get(name).map(|m| m.len()).unwrap_or(0) as u32;
            RawMailbox {
                mailbox_ref: RawMailboxRef {
                    name: name.to_string(),
                    uidvalidity: UIDVALIDITY,
                },
                role,
                parent: None,
                uidnext: total + 1,
                highestmodseq: 0,
                total,
                unread: total,
            }
        })
        .collect())
    }

    async fn sync_mailbox(
        &self,
        mbox: &RawMailboxRef,
        cursor: &SyncCursor,
    ) -> Result<MailboxDelta> {
        let uidnext_from = match cursor {
            SyncCursor::UidWindow { uidnext, .. } => *uidnext,
            _ => 1,
        };
        let st = self.messages.lock().unwrap();
        let msgs = st.get(&mbox.name).cloned().unwrap_or_default();
        let added: Vec<MessageRef> = msgs
            .iter()
            .filter(|(uid, ..)| *uid >= uidnext_from)
            .map(|(uid, ..)| MessageRef::Imap {
                mailbox: mbox.clone(),
                uidvalidity: UIDVALIDITY,
                uid: *uid,
            })
            .collect();
        let max_uid = msgs.iter().map(|(u, ..)| *u).max().unwrap_or(0);
        Ok(MailboxDelta {
            added,
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: UIDVALIDITY,
                uidnext: max_uid + 1,
            },
        })
    }

    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        let st = self.messages.lock().unwrap();
        let mut out = Vec::new();
        for r in refs {
            let MessageRef::Imap { mailbox, uid, .. } = r else {
                continue;
            };
            if let Some(msgs) = st.get(&mailbox.name)
                && let Some((_, raw, flags, internaldate)) = msgs.iter().find(|(u, ..)| u == uid)
            {
                out.push(RawMessage {
                    message_ref: r.clone(),
                    raw: raw.clone(),
                    flags: flags.clone(),
                    internaldate: Some(internaldate.clone()),
                });
            }
        }
        Ok(out)
    }

    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _rm: &[Flag]) -> Result<()> {
        Ok(())
    }

    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Ok(MoveOutcome::Uidplus {
            uidvalidity: UIDVALIDITY,
            uids: vec![9001],
        })
    }

    async fn append(&self, mbox: &RawMailboxRef, raw: &[u8], flags: &[Flag]) -> Result<MessageRef> {
        let mut st = self.messages.lock().unwrap();
        let entry = st.entry(mbox.name.clone()).or_default();
        let uid = entry.iter().map(|(u, ..)| *u).max().unwrap_or(0) + 1;
        entry.push((
            uid,
            raw.to_vec(),
            flags.to_vec(),
            "2026-07-10T09:00:00Z".into(),
        ));
        Ok(MessageRef::Imap {
            mailbox: mbox.clone(),
            uidvalidity: UIDVALIDITY,
            uid,
        })
    }

    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

struct FakeSubmitter {
    calls: AtomicUsize,
}

#[async_trait]
impl MailSubmitter for FakeSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
}

async fn setup(seed: usize) -> Harness {
    install_sql_counter();
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
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            Arc::new(FakeBackend::with_inbox(seed)) as Arc<dyn AccountBackend>,
            Arc::new(FakeSubmitter {
                calls: AtomicUsize::new(0),
            }) as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    let h = Harness { engine, account_id };
    h.engine.resync(&h.account_id).await.unwrap();
    h
}

async fn jmap(h: &Harness, calls: Value) -> Value {
    h.engine
        .handle_jmap(&h.account_id, &json!({ "methodCalls": calls }))
        .await
}

fn result<'a>(resp: &'a Value, call_id: &str) -> &'a Value {
    resp["methodResponses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r[2] == call_id)
        .map(|r| &r[1])
        .unwrap_or(&Value::Null)
}

async fn mailbox_with_role(h: &Harness, role: &str) -> String {
    let mb = jmap(h, json!([["Mailbox/get", {}, "mb"]])).await;
    result(&mb, "mb")["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == role)
        .unwrap_or_else(|| panic!("mailbox with role {role}"))["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn inbox(h: &Harness) -> String {
    let mb = jmap(h, json!([["Mailbox/get", {}, "mb"]])).await;
    result(&mb, "mb")["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "inbox")
        .expect("inbox")["id"]
        .as_str()
        .unwrap()
        .to_string()
}

/// Every id in the inbox, oldest first, via the store path (no text filter).
async fn all_ids(h: &Harness, inbox: &str) -> Vec<String> {
    let q = jmap(
        h,
        json!([[
            "Email/query",
            { "filter": { "inMailbox": inbox }, "limit": 100000 },
            "q"
        ]]),
    )
    .await;
    result(&q, "q")["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

/// Ids matching an **index-backed** operator query. Routed through `filter.text`
/// so the whole answer comes from Tantivy — if the re-index is skipped this
/// returns the pre-update answer, which is exactly what makes it a control on
/// "delete the re-index and the commit count still passes".
async fn search_ids(h: &Harness, inbox: &str, text: &str) -> Vec<String> {
    let q = jmap(
        h,
        json!([[
            "Email/query",
            { "filter": { "inMailbox": inbox, "text": text }, "limit": 100000 },
            "q"
        ]]),
    )
    .await;
    result(&q, "q")["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

fn seen_patch(ids: &[String]) -> Value {
    let mut update = serde_json::Map::new();
    for id in ids {
        update.insert(id.clone(), json!({ "keywords": { "$seen": true } }));
    }
    Value::Object(update)
}

/// One `Email/set` over `ids`, returning `(response, commits, sql statements)`
/// for exactly that call.
async fn measured_set(h: &Harness, ids: &[String]) -> (Value, u64, u64) {
    let commits_before = mw_search::commit_count();
    let sql_before = sql_count();
    let resp = jmap(
        h,
        json!([["Email/set", { "update": seen_patch(ids) }, "s"]]),
    )
    .await;
    (
        result(&resp, "s").clone(),
        mw_search::commit_count() - commits_before,
        sql_count() - sql_before,
    )
}

fn state_of(set: &Value, key: &str) -> u64 {
    set[key]
        .as_str()
        .expect(key)
        .parse()
        .expect("numeric state")
}

// ---------------------------------------------------------------------------
// Calibration — a counter nobody has checked is not an instrument
// ---------------------------------------------------------------------------

/// Negative control for both counters, so no number below rests on an untested
/// instrument. A single `Index::upsert` is one commit; a single trivial store
/// read is one SQL statement.
#[tokio::test]
async fn counters_are_calibrated_against_a_known_single_operation() {
    install_sql_counter();

    let idx = mw_search::Index::open_in_ram().unwrap();
    let before = mw_search::commit_count();
    idx.upsert(&mw_search::IndexDoc {
        stable_id: "cal-1".into(),
        subject: "calibration".into(),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        mw_search::commit_count() - before,
        1,
        "one upsert must be exactly one index commit"
    );

    // …and 500 documents through the batch method is still exactly one, which is
    // the capability the set path is required to use.
    let docs: Vec<mw_search::IndexDoc> = (0..500)
        .map(|i| mw_search::IndexDoc {
            stable_id: format!("cal-batch-{i}"),
            subject: "calibration".into(),
            ..Default::default()
        })
        .collect();
    let before = mw_search::commit_count();
    idx.upsert_batch(&docs).unwrap();
    assert_eq!(
        mw_search::commit_count() - before,
        1,
        "upsert_batch commits once regardless of document count"
    );

    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let before = sql_count();
    let _ = store
        .current_state("no-such-account", "Email")
        .await
        .unwrap();
    assert_eq!(
        sql_count() - before,
        1,
        "one single-statement store call must count as exactly one SQL statement — \
         if this is 0 the tracing subscriber is not seeing sqlx and every \
         statement-count assertion below is vacuous"
    );
}

// ---------------------------------------------------------------------------
// The acceptance assertions
// ---------------------------------------------------------------------------

/// The lane's headline: the index commits **once** for the batch, at both loads.
/// On `master` this reads 500 for the 500-id call and 5 for the 5-id call.
#[tokio::test]
async fn multi_id_email_set_commits_the_index_exactly_once() {
    let h = setup(520).await;
    let inbox = inbox(&h).await;
    let ids = all_ids(&h, &inbox).await;
    assert_eq!(ids.len(), 520, "seed");

    let (_, commits_5, _) = measured_set(&h, &ids[..5]).await;
    let (_, commits_500, _) = measured_set(&h, &ids[20..520]).await;

    assert_eq!(commits_5, 1, "5 ids must cost one index commit");
    assert_eq!(commits_500, 1, "500 ids must cost one index commit");
    assert_eq!(
        commits_500, commits_5,
        "the commit count must be independent of the id count"
    );
}

/// One transaction's worth of bookkeeping: one state increment and one change
/// row for the whole batch, not one per id.
#[tokio::test]
async fn multi_id_email_set_bumps_state_once_and_reports_every_changed_id() {
    let h = setup(500).await;
    let inbox = inbox(&h).await;
    let ids = all_ids(&h, &inbox).await;

    let changes_before = jmap(&h, json!([["Email/changes", { "sinceState": "0" }, "c"]])).await;
    let since = result(&changes_before, "c")["newState"]
        .as_str()
        .unwrap()
        .to_string();

    let (set, _, _) = measured_set(&h, &ids).await;

    let old = state_of(&set, "oldState");
    let new = state_of(&set, "newState");
    assert_eq!(
        new - old,
        1,
        "state must advance by one for the batch, not by the id count ({} ids)",
        ids.len()
    );

    // …and every id the batch touched is reported as changed, all sharing that
    // one state.
    //
    // The two assertions together are the point. One state bump is what makes
    // the batch cheap; a row per id is what keeps `Email/changes` truthful.
    // Satisfying only the first — a single row naming a single id, which is what
    // this batch wrote until `Store::record_changes` existed — leaves every other
    // device and push subscriber showing stale flags for the remaining N-1 until
    // something else happens to touch them.
    let after = jmap(&h, json!([["Email/changes", { "sinceState": since }, "c"]])).await;
    let mut updated: Vec<String> = result(&after, "c")["updated"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let mut expected = ids.clone();
    updated.sort();
    expected.sort();
    assert_eq!(
        updated, expected,
        "Email/changes must report every id the batch touched, not just one"
    );
}

/// The store round trips the batch removed, asserted **by name and by exact
/// count** — and the property those add up to: **the statement count of a
/// multi-id `Email/set` no longer depends on the id count at all.**
///
/// A per-id average is a weak instrument: it is satisfied by a path that still
/// does something once per id, just fewer somethings, and it moves whenever a
/// peer adds or removes legitimate per-id store work anywhere under the call.
/// That is not hypothetical — it happened to this very assertion, twice, before
/// it was rewritten this way. So nothing here is a ratio.
///
/// Three measurements on this harness, 500 ids, same fixture:
///
/// | | statements | per id |
/// |---|---|---|
/// | master `57046c7` | 4 514 | 9.03 |
/// | batched index commit (`fef455a`) | 1 016 → 2 007 once `t22-e1` landed V8's counter | 2.03 → 4.01 |
/// | adopting `t22-e1`'s `get_messages` + `set_flags_batch` | **11** | **0.022** |
///
/// The middle row is why the plan's `≤ 3/id` target was briefly unreachable and
/// why an average was the wrong thing to assert: two of those four statements
/// were a *peer's* correct new work (`Store::set_flags` re-reading to diff
/// `$seen`, and V8's per-message `unread` counter), landing inside a method this
/// lane does not own. Adopting the batch forms removed the question rather than
/// arguing about the number — the eleven statements are now a fixed cost, so
/// there is no per-id budget left to breach.
#[tokio::test]
async fn multi_id_email_set_costs_the_same_statements_at_any_id_count() {
    let h = setup(520).await;
    let inbox = inbox(&h).await;
    let ids = all_ids(&h, &inbox).await;

    // Five ids, then a hundred times as many, through the identical path.
    let _ = take_sql_text();
    let (_, _, stmts_5) = measured_set(&h, &ids[..5]).await;
    let _ = take_sql_text();
    let (_, _, stmts_500) = measured_set(&h, &ids[20..520]).await;
    let histogram = take_sql_text();

    let count_of = |prefix: &str| -> usize {
        histogram
            .iter()
            .filter(|(sql, _)| sql.starts_with(prefix))
            .map(|(_, n)| *n)
            .sum()
    };

    // THE assertion. Not a bound, not an average — an equality across a 100x
    // load difference. A path that does anything once per id fails it.
    assert_eq!(
        stmts_500, stmts_5,
        "the SQL cost of Email/set must not depend on the id count: {stmts_5}          statements for 5 ids vs {stmts_500} for 500. Breakdown of the 500-id          call: {histogram:#?}"
    );

    // The per-id statements this lane and `t22-e1`'s batch forms removed. Each
    // was 500 on master or at `fef455a`; each must now be zero.
    for (sql, what) in [
        ("SELECT envelope_json FROM messages", "get_envelope"),
        ("SELECT sealed_bytes FROM bodies", "get_body"),
        (
            "SELECT pinned, snoozed_until, follow_up_at",
            "get_message_meta",
        ),
        (
            "SELECT mailbox_id, uidvalidity, uid FROM messages",
            "message_location",
        ),
        ("UPDATE messages SET flags_json = ?2", "per-id set_flags"),
    ] {
        assert_eq!(
            count_of(sql),
            0,
            "{what} must not be issued at all for a batched set.              Breakdown: {histogram:#?}"
        );
    }

    // …and the batch forms that replaced them, each exactly once.
    for (sql, what) in [
        ("SELECT stable_id, account_id", "the batched message read"),
        (
            "UPDATE messages SET flags_json = j.f",
            "the batched flag write",
        ),
        ("INSERT INTO changes", "the change-log write"),
        ("SELECT id, account_id, name, role", "the mailbox row read"),
    ] {
        assert_eq!(
            count_of(sql),
            1,
            "{what} happens once for the batch, at any id count.              Breakdown: {histogram:#?}"
        );
    }

    // A hard constant, independent of n. Master needed 4 514 for this call.
    assert!(
        stmts_500 <= 16,
        "{stmts_500} statements for a 500-id set, over the fixed ceiling of 16.          Breakdown: {histogram:#?}"
    );
}

/// The pairing that stops "delete the re-index" from passing every count above.
///
/// `is:read` / `is:unread` are answered entirely from the Tantivy index (they
/// parse to `Keyword`/`NotKeyword` clauses over the indexed `keywords` field),
/// so the post-set answer is only correct if the batched re-index actually wrote
/// the updated keyword set. Removing `reindex_messages` from the set path leaves
/// `is:read` empty and `is:unread` at the full seed count — this test fails, the
/// commit-count tests do not.
#[tokio::test]
async fn batched_set_keeps_the_index_truthful() {
    let h = setup(120).await;
    let inbox = inbox(&h).await;
    let ids = all_ids(&h, &inbox).await;

    // Nothing is read yet, and the index agrees.
    assert_eq!(search_ids(&h, &inbox, "is:read").await.len(), 0);
    assert_eq!(search_ids(&h, &inbox, "is:unread").await.len(), 120);

    let marked: Vec<String> = ids[..70].to_vec();
    let (set, commits, _) = measured_set(&h, &marked).await;
    assert_eq!(commits, 1);
    assert_eq!(set["updated"].as_object().unwrap().len(), 70);

    let mut read = search_ids(&h, &inbox, "is:read").await;
    let mut expected = marked.clone();
    read.sort();
    expected.sort();
    assert_eq!(
        read, expected,
        "an index-backed search for the updated state must return exactly the \
         batched ids — if this is empty the re-index was dropped, not batched"
    );
    assert_eq!(
        search_ids(&h, &inbox, "is:unread").await.len(),
        50,
        "the untouched ids must still be unread in the index"
    );
}

/// Engine-local metadata (`pinned`) rides the same batch and is equally visible
/// to the index afterwards — the batched re-index must not be keyword-only.
#[tokio::test]
async fn batched_set_reindexes_engine_local_metadata_too() {
    let h = setup(60).await;
    let inbox = inbox(&h).await;
    let ids = all_ids(&h, &inbox).await;

    let mut update = serde_json::Map::new();
    for id in &ids[..25] {
        update.insert(id.clone(), json!({ "pinned": true }));
    }
    let before = mw_search::commit_count();
    let resp = jmap(&h, json!([["Email/set", { "update": update }, "s"]])).await;
    assert_eq!(mw_search::commit_count() - before, 1);
    assert_eq!(result(&resp, "s")["updated"].as_object().unwrap().len(), 25);

    let pinned = search_ids(&h, &inbox, "is:pinned").await;
    assert_eq!(
        pinned.len(),
        25,
        "pinned must be visible to the index after a batched set"
    );
}

/// A failing id inside the batch does not reach the index, does not contribute a
/// change row, and does not take the succeeding ids down with it.
///
/// Note the direction: JMAP `Email/set` is explicitly per-id (RFC 8620 §5.3 —
/// failures go to `notUpdated` and the rest still apply), so "roll the batch
/// back" cannot mean discarding the successes. What must hold, and what is
/// asserted, is that **the index never publishes a state the store rejected**:
/// the failed id is absent from the batch, so a search cannot see an update that
/// did not happen.
#[tokio::test]
async fn a_failing_id_mid_batch_never_reaches_the_index() {
    let h = setup(40).await;
    let inbox = inbox(&h).await;
    let ids = all_ids(&h, &inbox).await;

    let mut batch: Vec<String> = ids[..10].to_vec();
    // A stable id the store has never heard of, in the middle of the batch.
    batch.insert(5, "not-a-real-stable-id".to_string());

    let (set, commits, _) = measured_set(&h, &batch).await;

    assert_eq!(commits, 1, "still exactly one commit");
    assert_eq!(set["updated"].as_object().unwrap().len(), 10);
    assert_eq!(
        set["notUpdated"]
            .as_object()
            .expect("the unknown id is reported, not swallowed")
            .len(),
        1
    );

    let mut read = search_ids(&h, &inbox, "is:read").await;
    let mut expected = ids[..10].to_vec();
    read.sort();
    expected.sort();
    assert_eq!(
        read, expected,
        "the index holds the ten that succeeded and nothing else"
    );
    assert!(
        !read.iter().any(|id| id == "not-a-real-stable-id"),
        "a rejected id must never appear in the index"
    );
}

/// A batch in which **every** id fails must commit the index zero times and must
/// not advance state — the empty-batch edge of the `== 1` assertions above,
/// which an off-by-one fix would otherwise satisfy.
#[tokio::test]
async fn an_all_failing_batch_commits_nothing_and_does_not_advance_state() {
    let h = setup(10).await;
    let bogus: Vec<String> = (0..5).map(|i| format!("nope-{i}")).collect();

    let (set, commits, _) = measured_set(&h, &bogus).await;

    assert_eq!(commits, 0, "no successful id means no index commit");
    assert_eq!(set["updated"].as_object().unwrap().len(), 0);
    assert_eq!(set["notUpdated"].as_object().unwrap().len(), 5);
    assert_eq!(
        state_of(&set, "newState"),
        state_of(&set, "oldState"),
        "a batch that changed nothing must not advance state"
    );
}

/// A patch that sets keywords **and** moves must still write its flags to the
/// store *before* the move runs.
///
/// This is the one branch the batched flag write could have broken silently.
/// `move_email` re-keys the index entry from what the store holds at that
/// moment, so deferring such an id's flag write into the post-loop batch would
/// relocate a document carrying the *old* keywords — and every count assertion
/// in this file would still pass. `update_email` therefore writes moving ids
/// through directly and only batches the rest; this test is what holds that
/// rule in place.
#[tokio::test]
async fn a_patch_that_moves_and_sets_flags_keeps_both() {
    let h = setup(6).await;
    let inbox = inbox(&h).await;
    let archive = mailbox_with_role(&h, "archive").await;
    let ids = all_ids(&h, &inbox).await;
    let mover = ids[0].clone();

    // One id moves and is marked read in the same patch; the rest only get the
    // keyword, so both paths run in a single Email/set.
    let mut update = serde_json::Map::new();
    update.insert(
        mover.clone(),
        json!({ "keywords": { "$seen": true }, "mailboxIds": { archive.clone(): true } }),
    );
    for id in &ids[1..] {
        update.insert(id.clone(), json!({ "keywords": { "$seen": true } }));
    }
    let resp = jmap(&h, json!([["Email/set", { "update": update }, "s"]])).await;
    assert_eq!(result(&resp, "s")["updated"].as_object().unwrap().len(), 6);

    // The move landed…
    let moved = search_ids(&h, &archive, "is:read").await;
    assert_eq!(
        moved,
        vec![mover.clone()],
        "the moved id must be in Archive AND carry its new keyword — if the flag \
         write had been deferred past the move, the relocated index document \
         would still say unread"
    );
    // …and the five batched ids kept theirs, in the mailbox they never left.
    assert_eq!(search_ids(&h, &inbox, "is:read").await.len(), 5);
    assert_eq!(search_ids(&h, &inbox, "is:unread").await.len(), 0);
}
