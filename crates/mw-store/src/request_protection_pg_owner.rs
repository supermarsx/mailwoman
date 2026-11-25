//! The request-protection witness on PostgreSQL (t50hp-e8). The SQLite witness
//! (`request_protection_owner.rs`) uses lock files; here the same protocol runs
//! on session-level advisory locks held on one dedicated connection per
//! participant, outside the pool.
//!
//! Keys (reserved class, next to the change-log locks 719304-719306; see
//! docs/deploy/postgres.md):
//! - participant lock: `hashtextextended(<scope>, 719307)`, SHARED for the life
//!   of every participant's witness session;
//! - admission lock: `hashtextextended(<scope>, 719308)`, transaction-level,
//!   EXCLUSIVE during a join or a check.
//!
//! `<scope>` is `current_database() || ':' || current_schema() || ':' || <id>`,
//! where `<id>` is a value derived from this installation's server key (every
//! process of one installation holds the same key; two installations, or a test
//! and a production schema, in one cluster never share a key).
//!
//! State: the settings row `request-protection:pg-witness`, one line in the
//! same format as the SQLite state file (`v2 <joins> <acknowledged> <owner>
//! <window_ms>`), read and written under the admission lock.
//!
//! Check: under the admission lock, `pg_try_advisory_lock(participant)` on the
//! witness session succeeds only when no other session holds the participant
//! lock (a session's own locks never conflict with its own requests); it is
//! released at once. Ownership also requires `pg_is_in_recovery() = false` and
//! a session whose identity was stable when it was opened (a transaction-mode
//! pooler would move statements between server sessions and make advisory
//! locks meaningless: such a session never owns). Every statement on the
//! witness session runs under [`STATEMENT_TIMEOUT`] on the server and the same
//! client-side timeout, so a dead or partitioned connection makes the check
//! fail (`Lost`); ownership then lapses at the deadline of the previous check,
//! which is counted from before that check (R3). A participant does not report
//! `Alone` until [`GRACE`] after it joined AND after the last check that saw
//! another participant: every other participant notices the loss of its own
//! witness session within [`LIVENESS`] + [`STATEMENT_TIMEOUT`], then bumps the
//! writers marker at once and marks its writes from then on, so neither after
//! a failover (all sessions lost, advisory locks gone) nor after one other
//! session dies does an owner trust its cache while that writer is unmarked
//! (e15a C1). The session's identity (backend pid and a session setting) is
//! checked again in every check; once it changes the participant never owns
//! (a transaction-mode pooler, e15a C2).
use super::owner::{Checked, Ownership, StateLine, join_wait};
use crate::StoreError;
use sqlx::{ConnectOptions, Connection, Executor, Row};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Server-side and client-side limit of every witness statement.
pub(crate) const STATEMENT_TIMEOUT: Duration = Duration::from_secs(2);
/// How often a participant's own witness session is probed.
pub(crate) const LIVENESS: Duration = Duration::from_secs(1);
/// A participant does not own before this long after its join or after a
/// check that saw another participant: [`LIVENESS`] + [`STATEMENT_TIMEOUT`]
/// (how long another participant may take to notice its own session died)
/// plus a 2 s margin.
pub(crate) const GRACE: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(25);
const STATE_KEY: &str = "request-protection:pg-witness";
const SCOPE: &str = "current_database()||':'||current_schema()||':'||$1";

fn db(error: impl std::fmt::Display) -> StoreError {
    StoreError::Corrupt(format!("request protection witness (postgres): {error}"))
}
async fn timed<T>(
    future: impl std::future::Future<Output = Result<T, sqlx::Error>>,
) -> Result<T, StoreError> {
    match tokio::time::timeout(STATEMENT_TIMEOUT, future).await {
        Ok(result) => result.map_err(db),
        Err(_) => Err(db("statement did not answer in time")),
    }
}

/// The witness session of one participant.
pub(crate) struct PgWitness {
    connection: tokio::sync::Mutex<Option<sqlx::PgConnection>>,
    id: String,
    /// The session has kept its identity in every statement so far (sticky false).
    stable: AtomicBool,
    pid: i32,
    token: String,
    seen_joins: Mutex<Option<u64>>,
    joined_at: Instant,
    /// The last check that found another participant.
    last_other: Mutex<Option<Instant>>,
    alive: AtomicBool,
}
fn warn_unstable() {
    static WARNED: AtomicBool = AtomicBool::new(false);
    if !WARNED.swap(true, Ordering::Relaxed) {
        eprintln!("[mw-store] request protection witness: the PostgreSQL session is not stable (a transaction pooler?); write-behind accounting will not own. Use a direct connection or a session-mode pooler.");
    }
}
impl std::fmt::Debug for PgWitness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PgWitness").field("stable", &self.stable.load(Ordering::Relaxed)).finish_non_exhaustive()
    }
}

async fn read_state(connection: &mut sqlx::PgConnection) -> Result<Option<StateLine>, StoreError> {
    let row = timed(sqlx::query("SELECT value FROM settings WHERE key=$1").bind(STATE_KEY).fetch_optional(&mut *connection)).await?;
    Ok(match row {
        None => Some(StateLine::default()),
        Some(row) => StateLine::parse(&row.try_get::<String, _>("value").map_err(db)?),
    })
}
async fn write_state(connection: &mut sqlx::PgConnection, state: StateLine) -> Result<(), StoreError> {
    timed(
        sqlx::query("INSERT INTO settings(key,value) VALUES($1,$2) ON CONFLICT(key) DO UPDATE SET value=excluded.value")
            .bind(STATE_KEY)
            .bind(state.render().trim_end().to_owned())
            .execute(&mut *connection),
    )
    .await?;
    Ok(())
}
/// Caller is inside a transaction holding the admission lock: is any other
/// session holding the participant lock?
async fn alone_now(connection: &mut sqlx::PgConnection, id: &str) -> Result<bool, StoreError> {
    let row = timed(
        sqlx::query(&format!("SELECT pg_try_advisory_lock(hashtextextended({SCOPE},719307)) AS alone"))
            .bind(id)
            .fetch_one(&mut *connection),
    )
    .await?;
    let alone: bool = row.try_get("alone").map_err(db)?;
    if alone {
        timed(sqlx::query(&format!("SELECT pg_advisory_unlock(hashtextextended({SCOPE},719307))")).bind(id).execute(&mut *connection)).await?;
    }
    Ok(alone)
}

impl PgWitness {
    /// Open the witness session, prove it is a stable server session, take the
    /// shared participant lock and count the join (see module docs).
    pub(crate) async fn join(pool: &sqlx::PgPool, id: &str) -> Result<(Self, bool), StoreError> {
        let options = (*pool.connect_options()).clone().application_name("mailwoman-protection-witness");
        let mut connection = timed(options.connect()).await?;
        timed(connection.execute(sqlx::raw_sql(
            "SET statement_timeout='2s'; SET lock_timeout='2s'; SET idle_in_transaction_session_timeout='10s'",
        )))
        .await?;
        // A transaction-mode pooler hands consecutive statements to different
        // server sessions: the backend and a session setting must survive.
        let token = format!("{:032x}", rand::random::<u128>());
        let first = timed(sqlx::query("SELECT pg_backend_pid() AS pid, set_config('mw.protection_witness',$1,false) AS token").bind(&token).fetch_one(&mut connection)).await?;
        let _ = timed(sqlx::query("SELECT 1").execute(&mut connection)).await?;
        let second = timed(sqlx::query("SELECT pg_backend_pid() AS pid, current_setting('mw.protection_witness',true) AS token").fetch_one(&mut connection)).await?;
        let pid = first.try_get::<i32, _>("pid").map_err(db)?;
        let stable = pid == second.try_get::<i32, _>("pid").map_err(db)?
            && second.try_get::<Option<String>, _>("token").map_err(db)?.as_deref() == Some(token.as_str());
        if !stable {
            warn_unstable();
        }
        // Exclusive holds of the participant lock are momentary (a check under
        // the admission lock): retry the shared lock briefly.
        let until = Instant::now() + STATEMENT_TIMEOUT;
        loop {
            let row = timed(sqlx::query(&format!("SELECT pg_try_advisory_lock_shared(hashtextextended({SCOPE},719307)) AS held")).bind(id).fetch_one(&mut connection)).await?;
            if row.try_get::<bool, _>("held").map_err(db)? {
                break;
            }
            if Instant::now() >= until {
                return Err(StoreError::Invalid(super::owner::ADMISSION_BUSY));
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        let mut tx = timed(connection.begin()).await?;
        timed(sqlx::query(&format!("SELECT pg_advisory_xact_lock(hashtextextended({SCOPE},719308))")).bind(id).execute(&mut *tx)).await?;
        let mut state = read_state(&mut tx).await?.unwrap_or_default();
        state.joins = state.joins.saturating_add(1);
        let mine = state.joins;
        if alone_now(&mut tx, id).await? {
            state.owner = false;
            state.acknowledged = mine;
        }
        let wait = state.owner && state.acknowledged < mine;
        let window = state.window();
        write_state(&mut tx, state).await?;
        timed(tx.commit()).await?;
        let mut acknowledged = !wait;
        if wait {
            let until = tokio::time::Instant::now() + join_wait(window).min(super::owner::JOIN_WAIT_CAP);
            while tokio::time::Instant::now() < until {
                tokio::time::sleep(POLL).await;
                let current = read_state(&mut connection).await?;
                if current.is_none_or(|current| !current.owner || current.acknowledged >= mine) {
                    acknowledged = true;
                    break;
                }
            }
        }
        let witness = Self {
            connection: tokio::sync::Mutex::new(Some(connection)),
            id: id.to_owned(),
            stable: AtomicBool::new(stable),
            pid,
            token,
            seen_joins: Mutex::new(Some(mine)),
            joined_at: Instant::now(),
            last_other: Mutex::new(None),
            alive: AtomicBool::new(true),
        };
        Ok((witness, acknowledged))
    }

    /// One ownership check (see module docs).
    pub(crate) async fn check(&self, window: Duration) -> Checked {
        let started = Instant::now();
        let ownership = match self.check_at(window).await {
            Ok(ownership) => ownership,
            Err(_) => {
                self.alive.store(false, Ordering::Release);
                *self.connection.lock().await = None;
                Ownership::Lost
            }
        };
        Checked { ownership, started }
    }
    async fn check_at(&self, window: Duration) -> Result<Ownership, StoreError> {
        let mut slot = self.connection.lock().await;
        let Some(connection) = slot.as_mut() else { return Ok(Ownership::Lost) };
        let mut tx = timed(connection.begin()).await?;
        let admitted = timed(sqlx::query(&format!("SELECT pg_try_advisory_xact_lock(hashtextextended({SCOPE},719308)) AS admitted, pg_is_in_recovery() AS standby, pg_backend_pid() AS pid, current_setting('mw.protection_witness',true) AS token")).bind(&self.id).fetch_one(&mut *tx)).await?;
        // Every check: still the session that holds this participant's lock?
        let same = admitted.try_get::<i32, _>("pid").map_err(db)? == self.pid
            && admitted.try_get::<Option<String>, _>("token").map_err(db)?.as_deref() == Some(self.token.as_str());
        if !same && self.stable.swap(false, Ordering::AcqRel) {
            warn_unstable();
        }
        if !admitted.try_get::<bool, _>("admitted").map_err(db)? {
            timed(tx.rollback()).await?;
            return Ok(Ownership::Busy);
        }
        let standby: bool = admitted.try_get("standby").map_err(db)?;
        let alone = alone_now(&mut tx, &self.id).await?;
        let state = read_state(&mut tx).await?;
        let quiet = {
            let mut last_other = self.last_other.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if !alone {
                *last_other = Some(Instant::now());
            }
            last_other.is_none_or(|at| at.elapsed() >= GRACE)
        };
        let may_own = alone
            && quiet
            && !standby
            && self.stable.load(Ordering::Acquire)
            && self.joined_at.elapsed() >= GRACE;
        let seen_joins = *self.seen_joins.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let changed = state.is_none_or(|state| Some(state.joins) != seen_joins);
        let mut next = state.unwrap_or_default();
        next.acknowledged = next.joins;
        next.owner = may_own;
        next.window_ms = if may_own { u64::try_from(window.as_millis()).unwrap_or(u64::MAX) } else { 0 };
        if state != Some(next) && !standby {
            write_state(&mut tx, next).await?;
        }
        timed(tx.commit()).await?;
        *self.seen_joins.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(next.joins);
        Ok(if may_own { Ownership::Alone { changed } } else { Ownership::Shared })
    }

    /// Probe the witness session (`SELECT 1` under the timeout). A dead
    /// session marks this participant not alive.
    pub(crate) async fn probe(&self) -> bool {
        let mut slot = self.connection.lock().await;
        let ok = match slot.as_mut() {
            Some(connection) => timed(sqlx::query("SELECT 1").execute(&mut *connection)).await.is_ok(),
            None => false,
        };
        if !ok {
            *slot = None;
            self.alive.store(false, Ordering::Release);
        }
        ok
    }
    pub(crate) fn alive(&self) -> bool {
        self.alive.load(Ordering::Acquire)
    }
    pub(crate) async fn backend_pid(&self) -> Option<i32> {
        self.connection.lock().await.as_ref().map(|_| self.pid)
    }
    pub(crate) fn stable(&self) -> bool {
        self.stable.load(Ordering::Acquire)
    }
    /// Test hook: make the next check see another session's identity.
    #[cfg(test)]
    pub(crate) fn pretend_other_session(&mut self) {
        self.pid = -1;
    }
}
