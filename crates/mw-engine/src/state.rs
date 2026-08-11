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

    async fn crypto_type_num(&self, account_id: &str, kind: ChangeType) -> u64 {
        self.store()
            .current_crypto_state(account_id, kind.as_str())
            .await
            .unwrap_or(0)
    }

    async fn type_num(&self, account_id: &str, kind: ChangeType) -> u64 {
        self.store()
            .current_state(account_id, kind.as_str())
            .await
            .unwrap_or(0)
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
    /// datatype since `since_state` (frozen §2.1). `has_more_changes` is always
    /// false — the whole tail is returned.
    pub(crate) async fn build_changes(
        &self,
        account_id: &str,
        kind: ChangeType,
        since_state: &str,
    ) -> Result<Changes> {
        let since: u64 = since_state.parse().unwrap_or(0);
        let current = self
            .store()
            .current_state(account_id, kind.as_str())
            .await?;
        let rows = self
            .store()
            .changes_since(account_id, kind.as_str(), since)
            .await?;

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

        Ok(Changes {
            old_state: since.to_string(),
            new_state: current.to_string(),
            created,
            updated,
            destroyed,
            has_more_changes: false,
        })
    }

    /// Fan a [`StateChange`] out to every subscribed WS/SSE session (plan §1.2,
    /// §2.2). A no-op when no session is listening.
    pub(crate) async fn broadcast_state(&self, account_id: &str) {
        let email = self
            .type_num(account_id, ChangeType::Email)
            .await
            .to_string();
        let mailbox = self
            .type_num(account_id, ChangeType::Mailbox)
            .await
            .to_string();
        let submission = self
            .type_num(account_id, ChangeType::EmailSubmission)
            .await
            .to_string();
        // V4 crypto/security counters, sourced from the `crypto_changes` log so a
        // CryptoKey/MailRule change reaches connected sessions (plan §2.2).
        let crypto_key = self
            .crypto_type_num(account_id, ChangeType::CryptoKey)
            .await
            .to_string();
        let mail_rule = self
            .crypto_type_num(account_id, ChangeType::MailRule)
            .await
            .to_string();
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
mod session_state_tests {
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
    static SEEN: OnceLock<Mutex<Vec<(ThreadId, String)>>> = OnceLock::new();
    static MEASURING: AtomicBool = AtomicBool::new(false);
    /// Serializes measurements so two concurrent `#[tokio::test]`s cannot clear
    /// each other's recording. Statements from *unmeasured* concurrent tests are
    /// filtered out by thread instead.
    static MEASURE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

    fn seen() -> &'static Mutex<Vec<(ThreadId, String)>> {
        SEEN.get_or_init(Mutex::default)
    }

    /// The SQL statements one measurement observed.
    struct Stmts(Vec<String>);

    impl Stmts {
        fn len(&self) -> usize {
            self.0.len()
        }

        /// How many recorded statements mention `table`. Used to prove the fold
        /// is *one statement per group*, not three statements against one table.
        fn against(&self, table: &str) -> usize {
            self.0.iter().filter(|s| s.contains(table)).count()
        }
    }

    impl std::fmt::Debug for Stmts {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_list().entries(self.0.iter()).finish()
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
    }

    impl SqlVisitor {
        fn text(self) -> String {
            if self.statement.trim().is_empty() {
                self.summary
            } else {
                self.statement
            }
        }
    }

    impl Visit for SqlVisitor {
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
                .push((std::thread::current().id(), v.text()));
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
    async fn counted<T>(
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
    async fn db_threads(store: &Store) -> HashSet<ThreadId> {
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
                .filter(|s| !s.contains("pim_changes") && !s.contains("crypto_changes"))
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

    // ── live Postgres ───────────────────────────────────────────────────────
    // The divergence this lane exists for only appears on PG: the 12 sequential
    // round trips are ~2 ms on SQLite and ~45 ms on PG. Skipped LOUDLY (the test
    // name says live, and the skip prints) when no DSN is configured.

    fn pg_dsn() -> Option<String> {
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
    /// So on the engine path a `Mailbox/get` request is **1 + 3 = 4** statements
    /// after this lane and **1 + 12 = 13** before it. That does not reproduce the
    /// verifier's 15; the balance is per-request work above the engine, which
    /// this lane neither owns nor measures. `mailbox_get` is private to
    /// `mod jmap` and `handle_jmap` dispatch needs a connected `AccountRuntime`,
    /// so driving the method end to end needs a backend harness that lives in
    /// `tests/`, not here — deliberately not built, and flagged rather than
    /// papered over with arithmetic presented as a measurement.
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
