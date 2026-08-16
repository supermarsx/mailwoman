//! Real per-account state tokens + `*/changes` diffs + the realtime broadcast
//! (plan §1.2, §2.1/§2.2). Replaces the V1 `SESSION_STATE = "engine-0"`.
//!
//! State is sourced from the store `changes` log: each mutation appends a row
//! and bumps the `(account, type)` monotonic counter. `sessionState` is a
//! composite of the per-type counters, so it advances on any account change.

use std::collections::HashMap;

use crate::backend::Result;
use crate::change::{ChangeOp, ChangeType, Changes, StateChange};
use crate::engine::Engine;

/// The PIM datatypes whose counters are summed into `sessionState`.
///
/// `sessionState` sums *these*, not every row the `pim_changes` log happens to
/// hold, so adding a new PIM `ChangeType` does not silently change the composite
/// until it is listed here on purpose.
const PIM_STATE_TYPES: [ChangeType; 7] = [
    ChangeType::Calendar,
    ChangeType::CalendarEvent,
    ChangeType::Task,
    ChangeType::Note,
    ChangeType::AddressBook,
    ChangeType::ContactCard,
    ChangeType::ContactGroup,
];

/// The crypto/security datatypes whose counters are summed into `sessionState`
/// (plan §2.2), on the same closed-set rule as [`PIM_STATE_TYPES`].
const CRYPTO_STATE_TYPES: [ChangeType; 2] = [ChangeType::CryptoKey, ChangeType::MailRule];

/// One counter out of a batched read. A datatype the account has never touched
/// is **absent** from the map — `GROUP BY` emits no group for it — and reads as
/// `0`, which is what the per-counter `SELECT COALESCE(MAX(state), 0)` returned.
fn counter(counters: &HashMap<String, u64>, kind: ChangeType) -> u64 {
    counters.get(kind.as_str()).copied().unwrap_or(0)
}

/// The sum of `kinds`' counters, absent ones counting as `0`.
fn sum_counters(counters: &HashMap<String, u64>, kinds: &[ChangeType]) -> u64 {
    kinds.iter().map(|k| counter(counters, *k)).sum()
}

impl Engine {
    /// Append one change and return the new `(account, type)` state. Best-effort
    /// for callers that do not care about the exact number.
    pub(crate) async fn record_change(
        &self,
        account_id: &str,
        kind: ChangeType,
        stable_id: &str,
        op: ChangeOp,
    ) -> Result<u64> {
        Ok(self
            .store()
            .record_change(account_id, kind.as_str(), stable_id, op.as_str())
            .await?)
    }

    /// The current per-type state token as an opaque string.
    pub(crate) async fn type_state(&self, account_id: &str, kind: ChangeType) -> Result<String> {
        Ok(self
            .store()
            .current_state(account_id, kind.as_str())
            .await?
            .to_string())
    }

    /// The composite `sessionState`: it changes whenever any of the account's
    /// datatype states advance (RFC 8620 §2 — `sessionState`). The PIM datatype
    /// counters are folded in so a calendar/task/note/contact change also bumps
    /// `sessionState` (plan §1.8/§2.2).
    ///
    /// **One statement per change log, not one per counter** (26.20 t22-e0).
    /// This runs on *every* JMAP request (`handle_jmap`), before any method does
    /// any work. Read a counter at a time it was 12 sequential statements —
    /// invisible on SQLite, twelve round trips on Postgres.
    ///
    /// The three groups are read concurrently: they are independent statements
    /// against three different tables, so on Postgres the fixed cost is one
    /// round trip's latency rather than three.
    pub(crate) async fn session_state(&self, account_id: &str) -> String {
        let store = self.store();
        let (mail, pim, crypto) = tokio::join!(
            store.current_states(account_id),
            store.current_pim_states(account_id),
            store.current_crypto_states(account_id),
        );
        // A failed read degrades that group to zero, exactly as the per-counter
        // `unwrap_or(0)` it replaces did.
        let (mail, pim, crypto) = (
            mail.unwrap_or_default(),
            pim.unwrap_or_default(),
            crypto.unwrap_or_default(),
        );

        let e = counter(&mail, ChangeType::Email);
        let m = counter(&mail, ChangeType::Mailbox);
        let s = counter(&mail, ChangeType::EmailSubmission);
        let p = sum_counters(&pim, &PIM_STATE_TYPES);
        let c = sum_counters(&crypto, &CRYPTO_STATE_TYPES);
        format!("e{e}m{m}s{s}p{p}c{c}")
    }

    // ── PIM state tokens + `*/changes` (plan §1.8/§2.2) ─────────────────────
    // Sourced from the separate `pim_changes` log so PIM counters are disjoint
    // from the mail `changes` counters — a Calendar and an Email can share the
    // same numeric state without colliding.

    /// Append one PIM change and return the new `(account, type)` PIM state.
    pub(crate) async fn record_pim_change(
        &self,
        account_id: &str,
        kind: ChangeType,
        object_id: &str,
        op: ChangeOp,
    ) -> Result<u64> {
        Ok(self
            .store()
            .record_pim_change(account_id, kind.as_str(), object_id, op.as_str())
            .await?)
    }

    /// The current PIM per-type state token as an opaque string.
    pub(crate) async fn pim_type_state(
        &self,
        account_id: &str,
        kind: ChangeType,
    ) -> Result<String> {
        Ok(self
            .store()
            .current_pim_state(account_id, kind.as_str())
            .await?
            .to_string())
    }

    /// Build the `{oldState,newState,created,updated,destroyed}` diff for a PIM
    /// datatype since `since_state` (frozen §2.1), from the `pim_changes` log.
    pub(crate) async fn build_pim_changes(
        &self,
        account_id: &str,
        kind: ChangeType,
        since_state: &str,
    ) -> Result<Changes> {
        let since: u64 = since_state.parse().unwrap_or(0);
        let current = self
            .store()
            .current_pim_state(account_id, kind.as_str())
            .await?;
        let rows = self
            .store()
            .pim_changes_since(account_id, kind.as_str(), since)
            .await?;
        let (created, updated, destroyed) =
            fold_changes(rows.iter().map(|r| (r.object_id.as_str(), r.op.as_str())));
        Ok(Changes {
            old_state: since.to_string(),
            new_state: current.to_string(),
            created,
            updated,
            destroyed,
            has_more_changes: false,
        })
    }

    // ── Crypto/security state tokens + `*/changes` (plan §2.2) ──────────────
    // Sourced from the separate `crypto_changes` log (disjoint from the mail +
    // PIM counters) for `CryptoKey`/`MailRule`.

    /// Append one crypto/security change and return the new `(account, type)`
    /// crypto state.
    pub(crate) async fn record_crypto_change(
        &self,
        account_id: &str,
        kind: ChangeType,
        object_id: &str,
        op: ChangeOp,
    ) -> Result<u64> {
        Ok(self
            .store()
            .record_crypto_change(account_id, kind.as_str(), object_id, op.as_str())
            .await?)
    }

    /// The current crypto/security per-type state token as an opaque string.
    pub(crate) async fn crypto_type_state(
        &self,
        account_id: &str,
        kind: ChangeType,
    ) -> Result<String> {
        Ok(self
            .store()
            .current_crypto_state(account_id, kind.as_str())
            .await?
            .to_string())
    }

    /// Build the `{oldState,newState,created,updated,destroyed}` diff for a
    /// crypto/security datatype since `since_state`, from the `crypto_changes` log.
    pub(crate) async fn build_crypto_changes(
        &self,
        account_id: &str,
        kind: ChangeType,
        since_state: &str,
    ) -> Result<Changes> {
        let since: u64 = since_state.parse().unwrap_or(0);
        let current = self
            .store()
            .current_crypto_state(account_id, kind.as_str())
            .await?;
        let rows = self
            .store()
            .crypto_changes_since(account_id, kind.as_str(), since)
            .await?;
        let (created, updated, destroyed) =
            fold_changes(rows.iter().map(|r| (r.object_id.as_str(), r.op.as_str())));
        Ok(Changes {
            old_state: since.to_string(),
            new_state: current.to_string(),
            created,
            updated,
            destroyed,
            has_more_changes: false,
        })
    }

    /// Build the `{oldState,newState,created,updated,destroyed}` diff for a
    /// datatype since `since_state` (frozen §2.1), returning the **whole** tail
    /// with `has_more_changes: false`.
    ///
    /// The `changes` table is append-only — nothing in `crates/` deletes from it
    /// — so "the whole tail" grows for the life of a deployment. Anything
    /// answering a client should pass a cap via [`Engine::build_changes_limited`]
    /// instead (26.20 t22-e2, finding V6).
    pub(crate) async fn build_changes(
        &self,
        account_id: &str,
        kind: ChangeType,
        since_state: &str,
    ) -> Result<Changes> {
        Ok(self
            .build_changes_limited(account_id, kind, since_state, None)
            .await?
            .expect("the unbounded form never refuses: it has no cap to fail"))
    }

    /// [`Engine::build_changes`] with JMAP's `maxChanges` reaching **SQL**
    /// (26.20 t22-e2, finding V6).
    ///
    /// `max: None` is the unbounded form. `max: Some(n)` returns at most `n`
    /// change rows and sets `has_more_changes` when the tail continued past
    /// them. `Ok(None)` means the cap **cannot** be honoured without losing
    /// changes — see the state-boundary rule below — and is the caller's cue to
    /// answer `cannotCalculateChanges`.
    ///
    /// # `new_state` is a resume point, and it must land on a state BOUNDARY
    ///
    /// Two things have to be right here, and only the first is obvious.
    ///
    /// **It is the last row returned, not the current state.** Reporting the
    /// current state alongside a partial list tells the client "you are now up
    /// to date" about changes it was never sent, and it will not ask again.
    ///
    /// **Rows do not carry distinct states.** `Store::record_changes` writes
    /// **N rows at ONE state** — that is the whole point of it, and it is what a
    /// batched `Email/set` over 500 ids produces. So "the last row returned" can
    /// sit in the *middle* of a batch, and a client resuming at `state > that`
    /// would silently skip the rest of it. The cap is therefore pulled back to
    /// the last **complete** state: every row sharing the highest returned state
    /// is dropped, and `new_state` becomes the state below it, so the next call
    /// re-reads that batch whole.
    ///
    /// **When there is no such boundary** — the entire capped page is one state
    /// and there is more of it — pulling back would return nothing and advance
    /// nothing, so the client would loop forever. There is no partial answer
    /// that is both within the cap and lossless, which is exactly the condition
    /// RFC 8620 §5.2 defines `cannotCalculateChanges` for. Hence `Ok(None)`,
    /// rather than a page that quietly drops 450 of 500 ids.
    ///
    /// **This is conservative by exactly one state, deliberately.**
    /// [`Store::changes_since_limited`] reports *that* the tail continued, not
    /// the state it continued at, so a page whose last state happens to be
    /// complete is indistinguishable from one cut in half — and the highest
    /// state is dropped either way. A client asking for 20 singly-recorded
    /// changes therefore gets 19. That is legal (a server may always return
    /// fewer) and costs one re-read; the opposite error, keeping a state that
    /// turns out to be incomplete, strands the rest of that batch permanently.
    /// Distinguishing the two would mean returning the lookahead row's state
    /// from the store, which is a change to a method this lane does not own.
    ///
    /// The fold below can still collapse the returned rows to fewer entries — a
    /// created-then-destroyed pair inside the window cancels — so
    /// `created.len() + updated.len() + destroyed.len()` is not the cap and is
    /// not what `has_more_changes` describes. The cap is on **rows read**, which
    /// is what bounds the work and the response.
    pub(crate) async fn build_changes_limited(
        &self,
        account_id: &str,
        kind: ChangeType,
        since_state: &str,
        max: Option<i64>,
    ) -> Result<Option<Changes>> {
        let since: u64 = since_state.parse().unwrap_or(0);
        let current = self
            .store()
            .current_state(account_id, kind.as_str())
            .await?;
        let (mut rows, has_more) = match max {
            Some(n) => {
                self.store()
                    .changes_since_limited(account_id, kind.as_str(), since, n)
                    .await?
            }
            None => (
                self.store()
                    .changes_since(account_id, kind.as_str(), since)
                    .await?,
                false,
            ),
        };

        // The resume point, computed before the fold consumes `rows`: it is a
        // property of the rows read, not of the ids they folded into.
        let mut resume_state = since;
        if has_more {
            // Pull back to the last COMPLETE state. `record_changes` writes N
            // rows at one state, so the row the cap landed on is very likely
            // mid-batch, and `state > it` would skip the batch's remainder.
            let highest = rows.last().map(|r| r.state).unwrap_or(since);
            let complete = rows.iter().filter(|r| r.state < highest).count();
            if complete == 0 {
                // The whole capped page is one state with more of it beyond:
                // no answer exists that is both within the cap and lossless.
                return Ok(None);
            }
            rows.truncate(complete);
            resume_state = rows.last().map(|r| r.state).unwrap_or(since);
        }

        // Fold to the latest op per id; "created then destroyed in-window" cancels.
        let mut order: Vec<String> = Vec::new();
        let mut folded: HashMap<String, (bool, &'static str)> = HashMap::new();
        for r in &rows {
            let entry = folded.entry(r.stable_id.clone()).or_insert_with(|| {
                order.push(r.stable_id.clone());
                (false, "updated")
            });
            if r.op == "created" {
                entry.0 = true;
            }
            entry.1 = match r.op.as_str() {
                "created" => "created",
                "destroyed" => "destroyed",
                _ => "updated",
            };
        }

        let (mut created, mut updated, mut destroyed) = (Vec::new(), Vec::new(), Vec::new());
        for id in order {
            let (created_in_window, last) = folded[&id];
            match last {
                "destroyed" => {
                    if !created_in_window {
                        destroyed.push(id);
                    }
                }
                "created" => created.push(id),
                _ => updated.push(id),
            }
        }

        Ok(Some(Changes {
            old_state: since.to_string(),
            new_state: if has_more {
                resume_state.to_string()
            } else {
                current.to_string()
            },
            created,
            updated,
            destroyed,
            has_more_changes: has_more,
        }))
    }

    /// Fan a [`StateChange`] out to every subscribed WS/SSE session (plan §1.2,
    /// §2.2). A no-op when no session is listening.
    pub(crate) async fn broadcast_state(&self, account_id: &str) {
        // **One statement per change log, not one per counter** (26.20 t22-e-perf).
        // Measured: five sequential counter SELECTs, now two.
        //
        // This is `sessionState`'s 12 → 3 one layer over, and the cost lands
        // somewhere different: `session_state` runs per *request*, this runs on
        // every **broadcasting mutation** — so a client that writes pays it, and a
        // bulk operation pays it once per resync rather than once per read.
        //
        // The two groups are read concurrently: independent statements against two
        // tables, so on Postgres the fixed cost is one round trip's latency rather
        // than two. The PIM log is deliberately not read — no PIM counter appears
        // in a `StateChange`, and a fan-out must not pay for a value it does not
        // send.
        //
        // A failed read degrades that group to zeros, exactly as the per-counter
        // `unwrap_or(0)` it replaces did.
        let store = self.store();
        let (mail, crypto) = tokio::join!(
            store.current_states(account_id),
            store.current_crypto_states(account_id),
        );
        let (mail, crypto) = (mail.unwrap_or_default(), crypto.unwrap_or_default());

        let email = counter(&mail, ChangeType::Email).to_string();
        let mailbox = counter(&mail, ChangeType::Mailbox).to_string();
        let submission = counter(&mail, ChangeType::EmailSubmission).to_string();
        // V4 crypto/security counters, sourced from the `crypto_changes` log so a
        // CryptoKey/MailRule change reaches connected sessions (plan §2.2).
        let crypto_key = counter(&crypto, ChangeType::CryptoKey).to_string();
        let mail_rule = counter(&crypto, ChangeType::MailRule).to_string();
        let sc = StateChange {
            account_id: account_id.to_string(),
            thread: email.clone(),
            email,
            mailbox,
            submission,
            crypto_key,
            mail_rule,
        };
        // Ignore the "no receivers" error — sessions attach lazily.
        let _ = self.changes_tx().send(sc);
    }
}

/// Fold an ordered `(object_id, op)` change stream to the JMAP
/// `created`/`updated`/`destroyed` id sets (frozen §2.1): the latest op per id
/// wins, and a "created then destroyed in-window" cancels out. Shared by the
/// mail and PIM `*/changes` builders.
pub(crate) fn fold_changes<'a>(
    rows: impl Iterator<Item = (&'a str, &'a str)>,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut order: Vec<String> = Vec::new();
    let mut folded: HashMap<String, (bool, &'static str)> = HashMap::new();
    for (id, op) in rows {
        let entry = folded.entry(id.to_string()).or_insert_with(|| {
            order.push(id.to_string());
            (false, "updated")
        });
        if op == "created" {
            entry.0 = true;
        }
        entry.1 = match op {
            "created" => "created",
            "destroyed" => "destroyed",
            _ => "updated",
        };
    }
    let (mut created, mut updated, mut destroyed) = (Vec::new(), Vec::new(), Vec::new());
    for id in order {
        let (created_in_window, last) = folded[&id];
        match last {
            "destroyed" => {
                if !created_in_window {
                    destroyed.push(id);
                }
            }
            "created" => created.push(id),
            _ => updated.push(id),
        }
    }
    (created, updated, destroyed)
}

#[cfg(test)]
pub(crate) mod session_state_tests {
    //! `sessionState` cost + value tests (t22-e0).
    //!
    //! The instrument is a **statement count**, not elapsed time. On SQLite the
    //! 12 counter SELECTs cost ~2 ms and a timing assertion would read as noise;
    //! on Postgres the same 12 are ~45 ms of round trip on **every** request.
    //!
    //! **The layer counted is sqlx's driver layer**: `sqlx_core::logger::
    //! QueryLogger::finish` emits exactly one `sqlx::query` tracing event per
    //! statement it actually executed, after execution. It is not a count of
    //! `Sql::execute` call sites in `mw-store`, and not a count of store-method
    //! invocations — a "batch" method that loops internally would still be
    //! counted once per statement it issues. On Postgres one such event is one
    //! extended-query round trip, which is the cost this lane exists to remove.
    //! [`stmt_counter_negative_control`] calibrates the counter before any other
    //! number here is believed.

    use std::collections::HashSet;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Mutex, OnceLock};
    use std::thread::ThreadId;

    use crate::backend::AccountBackend;
    use mw_store::{AccountKind, Credentials, MailboxUpsert, NewAccount, ServerKey, Store};
    use tracing::field::{Field, Visit};
    use tracing::level_filters::LevelFilter;
    use tracing::span;
    use tracing::subscriber::Interest;
    use tracing::{Event, Metadata, Subscriber};

    use super::*;

    /// The three mail counters folded into `sessionState`.
    const MAIL: [ChangeType; 3] = [
        ChangeType::Email,
        ChangeType::Mailbox,
        ChangeType::EmailSubmission,
    ];
    /// The seven PIM counters folded into `sessionState`.
    const PIM: [ChangeType; 7] = [
        ChangeType::Calendar,
        ChangeType::CalendarEvent,
        ChangeType::Task,
        ChangeType::Note,
        ChangeType::AddressBook,
        ChangeType::ContactCard,
        ChangeType::ContactGroup,
    ];
    /// The two crypto/security counters folded into `sessionState`.
    const CRYPTO: [ChangeType; 2] = [ChangeType::CryptoKey, ChangeType::MailRule];

    // ── the statement counter ───────────────────────────────────────────────

    /// Every SQL statement observed while [`MEASURING`] is set, tagged with the
    /// thread that executed it.
    ///
    /// It cannot be a thread-local: **sqlx-sqlite runs every statement on a
    /// per-connection worker thread** (`sqlx_sqlite::connection::worker`), so
    /// the `sqlx::query` event is emitted from a thread the test never touches.
    /// A thread-local recorder counts zero for SQLite while happily counting a
    /// hand-written probe from the test thread — the second thing the negative
    /// control caught. sqlx-postgres has no worker thread and emits from the
    /// polling thread, so the emitting thread differs per backend and is
    /// discovered empirically by [`db_threads`] rather than assumed.
    static SEEN: OnceLock<Mutex<Vec<(ThreadId, Stmt)>>> = OnceLock::new();
    static MEASURING: AtomicBool = AtomicBool::new(false);
    /// Serializes measurements so two concurrent `#[tokio::test]`s cannot clear
    /// each other's recording. Statements from *unmeasured* concurrent tests are
    /// filtered out by thread instead.
    static MEASURE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

    fn seen() -> &'static Mutex<Vec<(ThreadId, Stmt)>> {
        SEEN.get_or_init(Mutex::default)
    }

    /// One observed statement: its SQL, and **how many rows it returned**.
    ///
    /// `rows` comes from sqlx's own `rows_returned` field
    /// (`sqlx_core::logger::QueryLogger::finish`) and is the instrument for a
    /// question the SQL text cannot answer, because the text carries
    /// placeholders rather than bound values: *did the `LIMIT` reach the
    /// database, or did Rust slice afterwards?* Both shapes issue one statement
    /// against one table; only one of them reads 50 rows out of 20 000
    /// (26.20 t22-e2).
    pub(crate) struct Stmt {
        pub(crate) sql: String,
        pub(crate) rows: u64,
    }

    /// The SQL statements one measurement observed.
    pub(crate) struct Stmts(pub(crate) Vec<Stmt>);

    impl Stmts {
        pub(crate) fn len(&self) -> usize {
            self.0.len()
        }

        /// How many recorded statements mention `table`. Used to prove the fold
        /// is *one statement per group*, not three statements against one table.
        pub(crate) fn against(&self, table: &str) -> usize {
            self.0.iter().filter(|s| s.sql.contains(table)).count()
        }

        /// Every statement whose SQL mentions `needle`.
        pub(crate) fn matching(&self, needle: &str) -> Vec<&Stmt> {
            self.0.iter().filter(|s| s.sql.contains(needle)).collect()
        }
    }

    impl std::fmt::Debug for Stmts {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_list().entries(self.0.iter()).finish()
        }
    }

    impl std::fmt::Debug for Stmt {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "[rows={}] {}", self.rows, self.sql)
        }
    }

    /// A `tracing` subscriber that records one entry per executed SQL statement
    /// into the calling thread's recorder.
    ///
    /// Hand-rolled over the `tracing` facade `mw-engine` already links rather
    /// than pulling `tracing-subscriber` in as a dev-dependency (net-zero deps).
    ///
    /// It must be the **global** default, not a thread-local one: `set_default`
    /// does not rebuild tracing's interest cache or raise `LevelFilter::current`
    /// (`tracing_core::dispatcher::State::set_default`), so sqlx's
    /// `tracing::enabled!(target: "sqlx::query", DEBUG)` guard short-circuits on
    /// the level check and never reaches `Subscriber::enabled`. A scoped
    /// subscriber therefore silently counts zero — which is precisely what
    /// [`stmt_counter_negative_control`] exists to catch, and did.
    struct StmtSubscriber;

    fn install() {
        static ONCE: OnceLock<()> = OnceLock::new();
        ONCE.get_or_init(|| {
            // Ignore an already-set global: another test may have installed one,
            // in which case our counts would be zero and the negative control
            // fails loudly rather than this lane reporting a fake improvement.
            let _ = tracing::subscriber::set_global_default(StmtSubscriber);
        });
    }

    /// Pulls the SQL text out of sqlx's `sqlx::query` event. sqlx puts the full
    /// statement in `db.statement` whenever it differs from the short `summary`.
    #[derive(Default)]
    struct SqlVisitor {
        summary: String,
        statement: String,
        rows: u64,
    }

    impl SqlVisitor {
        fn stmt(self) -> Stmt {
            let sql = if self.statement.trim().is_empty() {
                self.summary
            } else {
                self.statement
            };
            Stmt {
                sql,
                rows: self.rows,
            }
        }
    }

    impl Visit for SqlVisitor {
        /// sqlx emits `rows_returned`/`rows_affected` as `u64` fields.
        fn record_u64(&mut self, field: &Field, value: u64) {
            if field.name() == "rows_returned" {
                self.rows = value;
            }
        }

        fn record_str(&mut self, field: &Field, value: &str) {
            match field.name() {
                "summary" => self.summary = value.to_string(),
                "db.statement" => self.statement = value.to_string(),
                _ => {}
            }
        }

        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            match field.name() {
                "summary" if self.summary.is_empty() => self.summary = format!("{value:?}"),
                "db.statement" if self.statement.is_empty() => {
                    self.statement = format!("{value:?}")
                }
                _ => {}
            }
        }
    }

    impl Subscriber for StmtSubscriber {
        fn register_callsite(&self, meta: &'static Metadata<'static>) -> Interest {
            if meta.target() == "sqlx::query" {
                Interest::always()
            } else {
                Interest::never()
            }
        }

        fn enabled(&self, meta: &Metadata<'_>) -> bool {
            meta.target() == "sqlx::query"
        }

        fn max_level_hint(&self) -> Option<LevelFilter> {
            Some(LevelFilter::TRACE)
        }

        fn new_span(&self, _: &span::Attributes<'_>) -> span::Id {
            span::Id::from_u64(1)
        }

        fn record(&self, _: &span::Id, _: &span::Record<'_>) {}

        fn record_follows_from(&self, _: &span::Id, _: &span::Id) {}

        fn event(&self, event: &Event<'_>) {
            if event.metadata().target() != "sqlx::query" {
                return;
            }
            if !MEASURING.load(Ordering::Relaxed) {
                return;
            }
            let mut v = SqlVisitor::default();
            event.record(&mut v);
            seen()
                .lock()
                .unwrap()
                .push((std::thread::current().id(), v.stmt()));
        }

        fn enter(&self, _: &span::Id) {}

        fn exit(&self, _: &span::Id) {}
    }

    /// Run `fut` and return what it produced alongside every SQL statement
    /// `threads` executed while it ran.
    ///
    /// Restricting to `threads` — the threads that actually run this store's
    /// statements — is what keeps the count exact when the suite runs with more
    /// than one test thread: a concurrent, unmeasured test's SQL lands in
    /// [`SEEN`] but is discarded here.
    pub(crate) async fn counted<T>(
        threads: &HashSet<ThreadId>,
        fut: impl std::future::Future<Output = T>,
    ) -> (T, Stmts) {
        install();
        let _lock = MEASURE_LOCK
            .get_or_init(tokio::sync::Mutex::default)
            .lock()
            .await;
        seen().lock().unwrap().clear();
        MEASURING.store(true, Ordering::Relaxed);
        let out = fut.await;
        MEASURING.store(false, Ordering::Relaxed);
        let sql = std::mem::take(&mut *seen().lock().unwrap())
            .into_iter()
            .filter(|(t, _)| threads.contains(t))
            .map(|(_, s)| s)
            .collect();
        (out, Stmts(sql))
    }

    /// The threads that execute `store`'s statements, discovered by running one
    /// statement per pooled connection and seeing where the event comes from.
    /// Empirical rather than assumed: SQLite answers with worker threads and
    /// Postgres answers with the polling thread.
    pub(crate) async fn db_threads(store: &Store) -> HashSet<ThreadId> {
        install();
        seen().lock().unwrap().clear();
        MEASURING.store(true, Ordering::Relaxed);
        for _ in 0..8 {
            let _ = store.current_state("thread-probe", "Email").await;
            let _ = store.current_pim_state("thread-probe", "Note").await;
            let _ = store
                .current_crypto_state("thread-probe", "CryptoKey")
                .await;
        }
        MEASURING.store(false, Ordering::Relaxed);
        let me = std::thread::current().id();
        let found: HashSet<ThreadId> = std::mem::take(&mut *seen().lock().unwrap())
            .into_iter()
            .map(|(t, _)| t)
            .collect();
        assert!(
            !found.is_empty(),
            "no `sqlx::query` event reached the subscriber — the statement counter \
             is not measuring anything (test thread {me:?})"
        );
        found
    }

    // ── fixtures ────────────────────────────────────────────────────────────

    /// An engine plus the threads that execute its SQL — everything a
    /// measurement needs.
    struct Probe {
        engine: Engine,
        threads: HashSet<ThreadId>,
    }

    impl Probe {
        async fn sqlite() -> Self {
            let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
            let threads = db_threads(&store).await;
            Self {
                engine: Engine::new(store),
                threads,
            }
        }

        async fn pg() -> Option<Self> {
            let dsn = pg_dsn()?;
            let store = Store::open(&dsn, ServerKey::generate()).await.ok()?;
            let threads = db_threads(&store).await;
            Some(Self {
                engine: Engine::new(store),
                threads,
            })
        }

        fn store(&self) -> &Store {
            self.engine.store()
        }

        async fn count<T>(&self, fut: impl std::future::Future<Output = T>) -> (T, Stmts) {
            counted(&self.threads, fut).await
        }

        async fn account(&self, username: &str) -> String {
            self.store()
                .create_account(
                    &NewAccount {
                        kind: AccountKind::Imap,
                        host: "h",
                        port: 993,
                        tls: "implicit",
                        username,
                        sync_policy_json: "{}",
                    },
                    &Credentials {
                        username: username.into(),
                        password: "p".into(),
                    },
                )
                .await
                .unwrap()
        }

        /// Populate every counter the fold reads, at distinct values, so a fold
        /// that mixed two groups up would produce a different string.
        async fn seed_all(&self, account_id: &str) {
            for (n, kind) in MAIL.iter().enumerate() {
                for i in 0..=n {
                    self.engine
                        .record_change(account_id, *kind, &format!("m{n}-{i}"), ChangeOp::Created)
                        .await
                        .unwrap();
                }
            }
            for (n, kind) in PIM.iter().enumerate() {
                for i in 0..=n {
                    self.engine
                        .record_pim_change(
                            account_id,
                            *kind,
                            &format!("p{n}-{i}"),
                            ChangeOp::Created,
                        )
                        .await
                        .unwrap();
                }
            }
            for (n, kind) in CRYPTO.iter().enumerate() {
                for i in 0..=n {
                    self.engine
                        .record_crypto_change(
                            account_id,
                            *kind,
                            &format!("c{n}-{i}"),
                            ChangeOp::Created,
                        )
                        .await
                        .unwrap();
                }
            }
        }

        /// The **pre-fix** `sessionState`, reproduced verbatim as 12
        /// single-counter SELECTs against the store. Every equality assertion
        /// below compares the folded result to *this*, not to a hand-written
        /// expected string — a hand-written expectation would encode my
        /// assumption about the fold rather than `master`'s actual behaviour.
        async fn session_state_unfolded(&self, account_id: &str) -> String {
            let store = self.store();
            let mut mail = [0u64; 3];
            for (slot, kind) in mail.iter_mut().zip(MAIL) {
                *slot = store
                    .current_state(account_id, kind.as_str())
                    .await
                    .unwrap_or(0);
            }
            let mut p = 0u64;
            for kind in PIM {
                p += store
                    .current_pim_state(account_id, kind.as_str())
                    .await
                    .unwrap_or(0);
            }
            let mut c = 0u64;
            for kind in CRYPTO {
                c += store
                    .current_crypto_state(account_id, kind.as_str())
                    .await
                    .unwrap_or(0);
            }
            let (e, m, s) = (mail[0], mail[1], mail[2]);
            format!("e{e}m{m}s{s}p{p}c{c}")
        }

        /// Assert the fold on one prepared account: identical value, 12
        /// statements before, 3 after, and exactly one against each change log.
        async fn assert_folded(&self, account_id: &str, what: &str) {
            let (before, unfolded) = self.count(self.session_state_unfolded(account_id)).await;
            assert_eq!(
                unfolded.len(),
                12,
                "{what}: the pre-fix path is 12 counter SELECTs; got {unfolded:#?}"
            );

            let (after, folded) = self.count(self.engine.session_state(account_id)).await;

            assert_eq!(
                after, before,
                "{what}: folded sessionState must be byte-identical to the 12-SELECT value"
            );
            assert_eq!(
                folded.len(),
                3,
                "{what}: one statement per group (mail/PIM/crypto); got {folded:#?}"
            );
            assert_eq!(
                folded.against("crypto_changes"),
                1,
                "{what}: one crypto statement; got {folded:#?}"
            );
            assert_eq!(
                folded.against("pim_changes"),
                1,
                "{what}: one PIM statement; got {folded:#?}"
            );
            // `changes` is a substring of both other table names, so the mail
            // statement is the one that mentions neither of them.
            let mail_stmts = folded
                .0
                .iter()
                .filter(|s| !s.sql.contains("pim_changes") && !s.sql.contains("crypto_changes"))
                .count();
            assert_eq!(mail_stmts, 1, "{what}: one mail statement; got {folded:#?}");
        }

        /// The end-to-end instrument, through the public entry point: an empty
        /// `methodCalls` request does no method work, so every statement it
        /// issues is the fixed per-request `sessionState` cost (jmap.rs:139).
        async fn assert_per_request_overhead(&self, account_id: &str, what: &str) {
            let (resp, stmts) = self
                .count(
                    self.engine
                        .handle_jmap(account_id, &serde_json::json!({ "methodCalls": [] })),
                )
                .await;

            assert_eq!(
                resp["sessionState"],
                serde_json::Value::String(self.session_state_unfolded(account_id).await),
                "{what}: the request envelope carries the folded value"
            );
            assert_eq!(
                stmts.len(),
                3,
                "{what}: fixed per-request overhead (was 12); got {stmts:#?}"
            );
        }
    }

    // ── the calibration ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn stmt_counter_negative_control() {
        let p = Probe::sqlite().await;

        let (n, one) = p.count(p.store().current_state("nobody", "Email")).await;
        assert_eq!(n.unwrap(), 0);
        assert_eq!(
            one.len(),
            1,
            "one `current_state` must read as exactly 1 statement, else no other \
             number in this module is a measurement; got {one:#?}"
        );

        let ((), two) = p
            .count(async {
                let _ = p.store().current_state("nobody", "Email").await;
                let _ = p.store().current_pim_state("nobody", "Note").await;
            })
            .await;
        assert_eq!(two.len(), 2, "the counter tracks additional statements");
        assert_eq!(
            two.against("pim_changes"),
            1,
            "the counter records which table each statement hit; got {two:#?}"
        );
    }

    // ── SQLite ──────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn session_state_is_one_statement_per_group_all_counters_populated() {
        let p = Probe::sqlite().await;
        let acct = p.account("full").await;
        p.seed_all(&acct).await;

        // Guard the fixture itself: this case must NOT be all-zero, or it would
        // prove nothing about the values.
        assert_ne!(
            p.engine.session_state(&acct).await,
            "e0m0s0p0c0",
            "fixture must actually populate the counters"
        );
        p.assert_folded(&acct, "all counters populated").await;
    }

    #[tokio::test]
    async fn session_state_is_one_statement_per_group_with_every_group_empty() {
        // No account row at all: all 12 counters are ABSENT, not zero-valued.
        // A `GROUP BY` fold returns no rows here, so this is the case that
        // catches a fold which reads a missing row as anything but 0.
        let p = Probe::sqlite().await;
        assert_eq!(p.engine.session_state("never-seen").await, "e0m0s0p0c0");
        p.assert_folded("never-seen", "every group empty").await;
    }

    #[tokio::test]
    async fn session_state_is_one_statement_per_group_with_only_one_group_populated() {
        // The failure the brief names: a `GROUP BY` silently dropping the rows
        // it has no data for. Only ONE PIM type is populated, so the mail group
        // and the crypto group return zero rows, and six of the seven PIM types
        // are absent from the one group that does return rows.
        let p = Probe::sqlite().await;
        let acct = p.account("pim-only").await;
        for id in ["note-1", "note-2"] {
            p.engine
                .record_pim_change(&acct, ChangeType::Note, id, ChangeOp::Created)
                .await
                .unwrap();
        }

        assert_eq!(
            p.engine.session_state(&acct).await,
            "e0m0s0p2c0",
            "empty mail + crypto groups must read 0, not vanish"
        );
        p.assert_folded(&acct, "only the PIM group populated").await;
    }

    #[tokio::test]
    async fn session_state_is_one_statement_per_group_with_only_crypto_populated() {
        // The mirror case: the two-member group is the only populated one, so a
        // fold that hard-codes "the mail query always returns three rows" fails.
        let p = Probe::sqlite().await;
        let acct = p.account("crypto-only").await;
        p.engine
            .record_crypto_change(&acct, ChangeType::MailRule, "rule-1", ChangeOp::Created)
            .await
            .unwrap();

        assert_eq!(p.engine.session_state(&acct).await, "e0m0s0p0c1");
        p.assert_folded(&acct, "only the crypto group populated")
            .await;
    }

    #[tokio::test]
    async fn per_request_session_state_overhead_is_three_statements() {
        let p = Probe::sqlite().await;
        let acct = p.account("per-request").await;
        p.seed_all(&acct).await;
        p.assert_per_request_overhead(&acct, "SQLite").await;
    }

    // ── the connected-account harness (t22-e-perf item 1) ──────────────────
    //
    // `handle_jmap` refuses every method with `accountNotFound` unless the
    // account has a registered `AccountRuntime`, so measuring a whole request
    // needs a backend. `tests/`'s `FakeBackend` is duplicated across five files
    // and cannot be reached from a unit test anyway (the statement counter is
    // `#[cfg(test)]` inside this crate; an integration test links only the public
    // API). So this is a sixth copy by deliberate choice — a copy costs ~60 lines
    // once, refactoring five files mid-wave costs five lanes a rebase.
    //
    // **Every method panics**, which is the point rather than laziness:
    // `Mailbox/get` is served entirely from the store, so a fake that cannot be
    // called turns "this request touches no backend" into an assertion instead of
    // a claim. If a future `Mailbox/get` grows a backend read, this test fails
    // loudly with the method name.

    struct NeverCalledBackend;

    #[async_trait::async_trait]
    impl AccountBackend for NeverCalledBackend {
        async fn capabilities(&self) -> crate::backend::Result<crate::backend::BackendCaps> {
            unreachable!("Mailbox/get must not ask the backend for capabilities")
        }
        async fn list_mailboxes(&self) -> crate::backend::Result<Vec<crate::backend::RawMailbox>> {
            unreachable!("Mailbox/get must be served from the store, not the backend")
        }
        async fn sync_mailbox(
            &self,
            _mbox: &crate::backend::RawMailboxRef,
            _cursor: &crate::backend::SyncCursor,
        ) -> crate::backend::Result<crate::backend::MailboxDelta> {
            unreachable!("Mailbox/get must not sync")
        }
        async fn fetch_raw(
            &self,
            _refs: &[crate::backend::MessageRef],
        ) -> crate::backend::Result<Vec<crate::backend::RawMessage>> {
            unreachable!("Mailbox/get must not fetch bodies")
        }
        async fn store_flags(
            &self,
            _refs: &[crate::backend::MessageRef],
            _add: &[crate::backend::Flag],
            _remove: &[crate::backend::Flag],
        ) -> crate::backend::Result<()> {
            unreachable!("Mailbox/get must not write flags")
        }
        async fn move_messages(
            &self,
            _refs: &[crate::backend::MessageRef],
            _to: &crate::backend::RawMailboxRef,
        ) -> crate::backend::Result<crate::backend::MoveOutcome> {
            unreachable!("Mailbox/get must not move messages")
        }
        async fn append(
            &self,
            _mbox: &crate::backend::RawMailboxRef,
            _raw: &[u8],
            _flags: &[crate::backend::Flag],
        ) -> crate::backend::Result<crate::backend::MessageRef> {
            unreachable!("Mailbox/get must not append")
        }
        async fn watch(
            &self,
            _sink: crate::backend::ChangeSink,
        ) -> crate::backend::Result<crate::backend::WatchHandle> {
            unreachable!("Mailbox/get must not open a watch")
        }
    }

    struct NeverCalledSubmitter;

    #[async_trait::async_trait]
    impl crate::MailSubmitter for NeverCalledSubmitter {
        async fn submit(
            &self,
            _msg: mw_smtp::Outgoing,
        ) -> crate::backend::Result<mw_smtp::SubmissionResult> {
            unreachable!("Mailbox/get must not submit mail")
        }
    }

    impl Probe {
        /// Register a runtime so `handle_jmap` will dispatch, without giving it a
        /// backend that can answer anything.
        fn connect(&self, account_id: &str) {
            self.engine.register_backend(
                account_id.to_string(),
                crate::account::AccountRuntime::new(
                    std::sync::Arc::new(NeverCalledBackend) as std::sync::Arc<dyn AccountBackend>,
                    std::sync::Arc::new(NeverCalledSubmitter)
                        as std::sync::Arc<dyn crate::MailSubmitter>,
                    "me@example.org",
                ),
            );
        }
    }

    /// **t22-e-perf item 1: what a whole `Mailbox/get` REQUEST costs**, driven
    /// through `handle_jmap` rather than through one store method.
    ///
    /// This settles the residual `t22-e0` left open, and it settles it in a
    /// direction that lane could not see from where it was standing.
    ///
    /// e0 published *"engine path 1 + 3 = 4 after, 1 + 12 = 13 before; the
    /// balance to the verifier's 15 is per-request work above the engine"*, and
    /// was right to refuse to invent the balance. But the `1` in that sum came
    /// from `mailbox_get_reads_the_store_once_whatever_the_mailbox_count`, which
    /// measures **`Store::list_mailboxes` directly** — a true claim about the
    /// mailbox list, and not a measurement of `Engine::mailbox_get` at all. That
    /// the method reads the store exactly once was read off the code, not
    /// counted.
    ///
    /// Counted, `mailbox_get` reads the store **three** times: `list_mailboxes`,
    /// `list_saved_searches` (saved searches surface as virtual folders, §2.1),
    /// and `type_state` for the `Mailbox` counter. So the request is
    /// **3 + 3 = 6** now and **3 + 12 = 15** before e0 — which reproduces the
    /// verifier's 15 exactly, with **nothing above the engine at all**.
    ///
    /// The residual was never per-request work in `mw-server`. It was two engine
    /// statements that a store-level measurement could not attribute, which is
    /// why this had to be driven end to end to be answered.
    #[tokio::test]
    async fn a_whole_mailbox_get_request_is_six_statements() {
        let p = Probe::sqlite().await;
        let acct = p.account("mbox-get-request").await;
        for name in ["INBOX", "Sent", "Drafts"] {
            p.store()
                .upsert_mailbox(&MailboxUpsert {
                    account_id: &acct,
                    name,
                    role: None,
                    uidvalidity: 100,
                    uidnext: 1,
                    highestmodseq: 0,
                    total: 0,
                    unread: 0,
                    parent_id: None,
                })
                .await
                .unwrap();
        }

        p.connect(&acct);
        let req = serde_json::json!({
            "methodCalls": [["Mailbox/get", { "accountId": acct }, "c0"]]
        });
        let (resp, stmts) = p.count(p.engine.handle_jmap(&acct, &req)).await;

        // The request actually did its work — without this the count below is a
        // count of a failure.
        let list = resp["methodResponses"][0][1]["list"]
            .as_array()
            .expect("Mailbox/get returned a list");
        assert_eq!(list.len(), 3, "three mailboxes: {resp}");

        // The three that are `sessionState`, folded by t22-e0.
        let session_state = stmts.against("changes") - stmts.against("mailboxes");
        assert_eq!(
            stmts.len(),
            6,
            "a whole Mailbox/get request: 3 sessionState + 3 method statements. \
             e0 published 4 for this, from a store-level measurement that could \
             not see `list_saved_searches` or `type_state`. Got {stmts:#?}"
        );
        assert!(
            session_state >= 3,
            "three change-log statements are sessionState's; got {stmts:#?}"
        );

        // Named individually, so a regression says WHICH read came back rather
        // than only that the total moved.
        assert_eq!(
            stmts.against("FROM mailboxes"),
            1,
            "one mailbox list; got {stmts:#?}"
        );
        assert_eq!(
            stmts.against("saved_searches"),
            1,
            "saved searches surface as virtual folders, so the method reads them \
             unconditionally — this is one of the two statements e0's 4 omits; \
             got {stmts:#?}"
        );

        // And the property e0 DID establish, which stands: none of this scales
        // with the mailbox count.
        for name in ["Archive", "Spam", "Trash", "Junk"] {
            p.store()
                .upsert_mailbox(&MailboxUpsert {
                    account_id: &acct,
                    name,
                    role: None,
                    uidvalidity: 100,
                    uidnext: 1,
                    highestmodseq: 0,
                    total: 0,
                    unread: 0,
                    parent_id: None,
                })
                .await
                .unwrap();
        }
        let (resp7, stmts7) = p.count(p.engine.handle_jmap(&acct, &req)).await;
        assert_eq!(
            resp7["methodResponses"][0][1]["list"]
                .as_array()
                .unwrap()
                .len(),
            7
        );
        assert_eq!(
            stmts7.len(),
            stmts.len(),
            "seven mailboxes must cost what three did; got {stmts7:#?}"
        );
    }

    // ── broadcast_state (t22-e-perf item 2) ────────────────────────────────

    impl Probe {
        /// **The pre-fix `broadcast_state` reads, reproduced verbatim** as five
        /// single-counter round trips — `session_state_unfolded`'s pattern, and
        /// for the same reason: the equality below compares the folded result to
        /// *this*, measured in the same run, rather than to a hand-written
        /// expectation that would encode my assumption about the fold.
        ///
        /// `thread` is `email` in both, and is not a sixth read.
        ///
        /// It reads the store directly because the `Engine::type_num` /
        /// `crypto_type_num` helpers it used to call had no caller left once
        /// `broadcast_state` was folded, and dead production code that reads like
        /// a live second path is worse than an oracle that states what it is
        /// (same move as `t22-e3g`'s `build_email_per_id`).
        async fn broadcast_unfolded(&self, account_id: &str) -> StateChange {
            let one = async |kind: ChangeType| {
                self.store()
                    .current_state(account_id, kind.as_str())
                    .await
                    .unwrap_or(0)
                    .to_string()
            };
            let one_crypto = async |kind: ChangeType| {
                self.store()
                    .current_crypto_state(account_id, kind.as_str())
                    .await
                    .unwrap_or(0)
                    .to_string()
            };
            let email = one(ChangeType::Email).await;
            let mailbox = one(ChangeType::Mailbox).await;
            let submission = one(ChangeType::EmailSubmission).await;
            let crypto_key = one_crypto(ChangeType::CryptoKey).await;
            let mail_rule = one_crypto(ChangeType::MailRule).await;
            StateChange {
                account_id: account_id.to_string(),
                thread: email.clone(),
                email,
                mailbox,
                submission,
                crypto_key,
                mail_rule,
            }
        }

        /// Assert the fold on one prepared account: identical `StateChange`,
        /// 5 statements before, 2 after, and one against each change log.
        async fn assert_broadcast_folded(&self, account_id: &str, what: &str) {
            let (unfolded, before) = self.count(self.broadcast_unfolded(account_id)).await;
            assert_eq!(
                before.len(),
                5,
                "{what}: the pre-fix fan-out is 5 sequential counter SELECTs; got {before:#?}"
            );

            let mut rx = self.engine.subscribe();
            let (_, after) = self.count(self.engine.broadcast_state(account_id)).await;
            let sent = rx
                .try_recv()
                .expect("broadcast_state must publish a StateChange");

            assert_eq!(
                sent, unfolded,
                "{what}: the folded fan-out must be identical to the 5-read value"
            );
            assert_eq!(
                after.len(),
                2,
                "{what}: one statement per change log (mail/crypto); got {after:#?}"
            );
            assert_eq!(
                after.against("crypto_changes"),
                1,
                "{what}: one crypto statement; got {after:#?}"
            );
            let mail_stmts = after
                .0
                .iter()
                .filter(|s| !s.sql.contains("pim_changes") && !s.sql.contains("crypto_changes"))
                .count();
            assert_eq!(mail_stmts, 1, "{what}: one mail statement; got {after:#?}");
            assert_eq!(
                after.against("pim_changes"),
                0,
                "{what}: the fan-out carries no PIM counter, so it must not read \
                 that log at all; got {after:#?}"
            );
        }
    }

    /// t22-e-perf item 2. `broadcast_state` runs on **every broadcasting
    /// mutation**, so its cost is paid per write rather than per request — the
    /// same shape as `sessionState`'s 12 → 3, one layer over.
    ///
    /// The instrument is the statement count, calibrated by
    /// [`stmt_counter_negative_control`] and re-calibrated here by the
    /// `before.len() == 5` leg: a counter that could not tell 5 sequential reads
    /// from 2 batched ones would make the `after` number meaningless.
    ///
    /// Every group is asserted **populated and empty**, because a fold that
    /// mixed two groups up produces the same answer when both are zero.
    #[tokio::test]
    async fn broadcast_state_is_one_statement_per_change_log() {
        let p = Probe::sqlite().await;

        let full = p.account("bcast-full").await;
        p.seed_all(&full).await;
        p.assert_broadcast_folded(&full, "all counters populated")
            .await;

        p.assert_broadcast_folded("bcast-never-seen", "every group empty")
            .await;

        // Only the crypto group populated: the mail counters must still read 0
        // rather than inheriting a crypto value.
        let crypto_only = p.account("bcast-crypto").await;
        p.engine
            .record_crypto_change(&crypto_only, ChangeType::CryptoKey, "k1", ChangeOp::Created)
            .await
            .unwrap();
        p.assert_broadcast_folded(&crypto_only, "only the crypto group populated")
            .await;

        // Only the mail group populated, and specifically ONE of its three
        // counters — the case that catches a fold reading the wrong key.
        let mail_only = p.account("bcast-mail").await;
        p.engine
            .record_change(&mail_only, ChangeType::Mailbox, "mb1", ChangeOp::Created)
            .await
            .unwrap();
        p.assert_broadcast_folded(&mail_only, "only Mailbox populated")
            .await;
    }

    // ── live Postgres ───────────────────────────────────────────────────────
    // The divergence this lane exists for only appears on PG: the 12 sequential
    // round trips are ~2 ms on SQLite and ~45 ms on PG. Skipped LOUDLY (the test
    // name says live, and the skip prints) when no DSN is configured.

    pub(crate) fn pg_dsn() -> Option<String> {
        std::env::var("MW_E14_PG_DSN")
            .or_else(|_| std::env::var("DATABASE_URL_PG"))
            .ok()
            .filter(|s| !s.trim().is_empty())
    }

    /// A run-unique username so repeated live runs never collide on the shared
    /// database (this lane never drops a schema out from under a peer).
    fn unique(tag: &str) -> String {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("t22e0-{tag}-{n}")
    }

    #[tokio::test]
    async fn live_pg_session_state_is_one_statement_per_group() {
        let Some(p) = Probe::pg().await else {
            eprintln!("SKIP live_pg_session_state_is_one_statement_per_group: no MW_E14_PG_DSN");
            return;
        };

        let full = p.account(&unique("full")).await;
        p.seed_all(&full).await;
        p.assert_folded(&full, "live PG, all counters populated")
            .await;

        p.assert_folded(&unique("never-seen"), "live PG, every group empty")
            .await;

        let partial = p.account(&unique("pim")).await;
        p.engine
            .record_pim_change(&partial, ChangeType::Note, "note-1", ChangeOp::Created)
            .await
            .unwrap();
        assert_eq!(p.engine.session_state(&partial).await, "e0m0s0p1c0");
        p.assert_folded(&partial, "live PG, only the PIM group populated")
            .await;
    }

    /// t22-e-perf item 2, on the backend the fold exists for.
    ///
    /// The statement *count* is dialect-independent — both backends issue the
    /// same five, then the same two. What is not dialect-independent is what
    /// they cost: five sequential counter SELECTs are ~1 ms on SQLite and five
    /// network round trips on Postgres, on **every broadcasting mutation**. This
    /// leg also exercises the `GROUP BY` batched readers against real Postgres,
    /// which the SQLite leg cannot.
    #[tokio::test]
    async fn live_pg_broadcast_state_is_one_statement_per_change_log() {
        let Some(p) = Probe::pg().await else {
            eprintln!(
                "SKIP live_pg_broadcast_state_is_one_statement_per_change_log: no MW_E14_PG_DSN"
            );
            return;
        };
        let full = p.account(&unique("bcast")).await;
        p.seed_all(&full).await;
        p.assert_broadcast_folded(&full, "live PG, all counters populated")
            .await;
        p.assert_broadcast_folded(&unique("bcast-empty"), "live PG, every group empty")
            .await;
    }

    #[tokio::test]
    async fn live_pg_per_request_session_state_overhead_is_three_statements() {
        let Some(p) = Probe::pg().await else {
            eprintln!(
                "SKIP live_pg_per_request_session_state_overhead_is_three_statements: \
                 no MW_E14_PG_DSN"
            );
            return;
        };
        let acct = p.account(&unique("req")).await;
        p.seed_all(&acct).await;
        p.assert_per_request_overhead(&acct, "live PG").await;
    }

    /// What a real `Mailbox/get` costs, to the extent this lane can measure it.
    ///
    /// The plan's headline for this row is "`Mailbox/get` = 15 statements, of
    /// which 12 are `sessionState`". The 12 and their removal are measured
    /// directly above, on both backends. This test pins the **other** half of
    /// the engine-side composition: `Engine::mailbox_get` reads the store
    /// exactly once (`Store::list_mailboxes`) and then builds JSON from the rows
    /// it already has — no per-mailbox follow-up read.
    ///
    /// **SUPERSEDED, and correctly refused rather than guessed** (26.20
    /// t22-e-perf). This lane inferred from the code that `mailbox_get` reads the
    /// store once and published **1 + 3 = 4** after / **1 + 12 = 13** before,
    /// noting it did not reproduce the verifier's 15 and that the balance must be
    /// per-request work above the engine — refusing to invent it, which was the
    /// right call.
    ///
    /// Driven end to end through `handle_jmap` with a connected runtime, the real
    /// figure is **3 + 3 = 6** after and **3 + 12 = 15** before, which reproduces
    /// the verifier's 15 exactly and leaves **nothing above the engine**.
    /// `mailbox_get` makes three store reads, not one: `list_mailboxes`,
    /// `list_saved_searches` and `type_state`. The test below is a true claim
    /// about `Store::list_mailboxes`, and its scaling property stands — it simply
    /// never measured `Engine::mailbox_get`. See
    /// [`a_whole_mailbox_get_request_is_six_statements`].
    #[tokio::test]
    async fn mailbox_get_reads_the_store_once_whatever_the_mailbox_count() {
        let p = Probe::sqlite().await;
        let acct = p.account("mailbox-get").await;

        for (n, name) in ["INBOX", "Sent", "Drafts", "Archive", "Spam"]
            .into_iter()
            .enumerate()
        {
            p.store()
                .upsert_mailbox(&MailboxUpsert {
                    account_id: &acct,
                    name,
                    role: None,
                    uidvalidity: 100,
                    uidnext: 1,
                    highestmodseq: 0,
                    total: n as u32,
                    unread: 0,
                    parent_id: None,
                })
                .await
                .unwrap();
        }

        let (mailboxes, stmts) = p.count(p.store().list_mailboxes(&acct)).await;
        assert_eq!(
            mailboxes.unwrap().len(),
            5,
            "the fixture has five mailboxes"
        );
        assert_eq!(
            stmts.len(),
            1,
            "the mailbox list is one statement, not one per mailbox; got {stmts:#?}"
        );
    }
}
