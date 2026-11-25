//! Cross-process witness for write-behind request-protection accounting (t50hp-e8).
//!
//! A process that caches subject rows (write-behind accounting) may trust its
//! cache without re-reading only while no other participant can write the
//! request-protection tables. Every writer of those tables is a *participant*:
//!
//! - each write-behind accounting instance holds its own participant;
//! - every other writer (the synchronous path, administrative clear from a
//!   synchronous process) joins once per process and database file, before its
//!   first write, through [`Store::request_protection_join`].
//!
//! Files beside the database (`<db>` is the logical SQLite path):
//! - `<db>.mw-request-protection.lock`: each participant holds a SHARED lock on
//!   its own handle for as long as it can write;
//! - `<db>.mw-request-protection-admission.lock`: held EXCLUSIVE only for the few
//!   system calls of a join or an ownership check, so the two never interleave;
//! - `<db>.mw-request-protection.state`: one line, `v2 <joins> <acknowledged>
//!   <owner> <window_ms>`, read and written only under the admission lock.
//!
//! `<db>` is the canonical path of the database file (symbolic links,
//! junctions, case and short names resolved), so every name of one file finds
//! the same witness. A file with more than one hard link is never owned (a
//! check reports `Shared`), because its other names would use other witnesses.
//!
//! Ownership check (`Participant::check`): under the admission lock the
//! participant releases its shared lock and tries an exclusive one on the same
//! handle. That succeeds only when no other handle, in this or another process,
//! holds the lock file: the caller is then the only participant ("alone"). The
//! shared lock is taken back before the admission lock is released. The check
//! also acknowledges every join recorded so far and reports whether a join
//! happened since the caller's previous check (a writer that joined and left).
//!
//! Join: under the admission lock, take the shared lock, count the join, and
//! when the state names an owner that has not acknowledged it, wait (polling,
//! never holding the admission lock while asleep) until the owner acknowledges
//! it, gives up ownership, or the wait for the owner's window passes. An owner
//! trusts its cache only until `started + window`, where `started` is taken
//! before its check takes the admission lock and `window` is written into the
//! state line by that check. A joiner enters after that check released the
//! lock, so `started + window` is at most `window` after the joiner's start; it
//! waits `window + max(window, 2 s)` ([`join_wait`]). A stall of the owner
//! anywhere after `started` only shortens its trust. When the wait ends without
//! an acknowledgement the participant is *unacknowledged* (it still holds its
//! lock, so the owner's next check sees it), and every write it makes also
//! bumps the database marker described in `request_protection.rs`, which
//! owners read on every check. A joiner that finds itself alone does not wait.
//!
//! Not covered: a writer that does not use this crate's request-protection
//! functions (direct SQL on the file, an older binary), and deleting these files
//! while processes run. A deleted or replaced lock file is detected by the next
//! check, which then reports `Lost`: the file's creator writes a random token
//! into it, each participant remembers the token, and a check compares it with
//! the file now at the path. (A file identity from metadata is not used: NTFS
//! keeps the creation time of a file re-created under the same name.)
use crate::StoreError;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Upper bound of a joiner's wait, whatever window a state line names.
pub(crate) const JOIN_WAIT_CAP: Duration = Duration::from_secs(30);
/// Margin added to an owner's window: at least the window, never under 2 s.
const MIN_MARGIN: Duration = Duration::from_secs(2);
/// The joiner's wait for an owner whose trust window is `window`.
pub fn join_wait(window: Duration) -> Duration {
    window + window.max(MIN_MARGIN)
}
/// The ownership window for a flush interval: two intervals. `Err` when the
/// joiner's wait for it would exceed [`JOIN_WAIT_CAP`], so start-up refuses it.
pub fn ownership_window(flush_every: Duration) -> Result<Duration, String> {
    let window = flush_every * 2;
    if join_wait(window) > JOIN_WAIT_CAP {
        return Err(format!(
            "a flush interval of {} ms gives an ownership window whose join wait exceeds {} s",
            flush_every.as_millis(),
            JOIN_WAIT_CAP.as_secs()
        ));
    }
    Ok(window)
}
/// Window assumed for a state line written before the window was recorded.
const LEGACY_WINDOW: Duration = Duration::from_secs(10);
/// Longest wait for the admission lock, which is only held for a few system calls.
const ADMISSION_WAIT: Duration = Duration::from_secs(2);
const POLL: Duration = Duration::from_millis(25);

#[derive(Clone, Debug)]
struct Paths {
    lock: PathBuf,
    admission: PathBuf,
    state: PathBuf,
}
impl Paths {
    fn new(logical: &Path) -> Self {
        let side = |suffix: &str| {
            let mut name = logical.as_os_str().to_owned();
            name.push(suffix);
            PathBuf::from(name)
        };
        Self {
            lock: side(".mw-request-protection.lock"),
            admission: side(".mw-request-protection-admission.lock"),
            state: side(".mw-request-protection.state"),
        }
    }
}

/// What the database file is, for the witness.
#[derive(Clone, Debug)]
pub(crate) enum Location {
    /// A SQLite file other processes can open, by its canonical path.
    File(PathBuf),
    /// A SQLite file whose canonical path could not be resolved: never owned,
    /// and its writers cannot join.
    Unresolved,
    /// An in-memory SQLite database: private to its pool, so no other process
    /// can write it.
    Private,
    /// No witness and never owned.
    Shared,
    /// PostgreSQL: session advisory locks; `id` is derived from the server key
    /// (see `request_protection_pg_owner.rs`).
    Postgres { pool: sqlx::PgPool, id: String },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct StateLine {
    pub(crate) joins: u64,
    pub(crate) acknowledged: u64,
    pub(crate) owner: bool,
    /// The owner's trust window in milliseconds (0: none recorded).
    pub(crate) window_ms: u64,
}
impl StateLine {
    pub(crate) fn parse(raw: &str) -> Option<Self> {
        let mut parts = raw.split_ascii_whitespace();
        let version = parts.next()?;
        if version != "v1" && version != "v2" {
            return None;
        }
        let joins = parts.next()?.parse().ok()?;
        let acknowledged = parts.next()?.parse().ok()?;
        let owner = match parts.next()? {
            "1" => true,
            "0" => false,
            _ => return None,
        };
        let window_ms = if version == "v2" { parts.next()?.parse().ok()? } else { 0 };
        parts.next().is_none().then_some(Self { joins, acknowledged, owner, window_ms })
    }
    pub(crate) fn render(self) -> String {
        format!("v2 {} {} {} {}\n", self.joins, self.acknowledged, u8::from(self.owner), self.window_ms)
    }
    pub(crate) fn window(self) -> Duration {
        if self.window_ms == 0 { LEGACY_WINDOW } else { Duration::from_millis(self.window_ms) }
    }
}

fn io(error: std::io::Error) -> StoreError {
    StoreError::Corrupt(format!("request protection witness: {error}"))
}
/// The message of the one transient join failure (the admission lock stayed busy).
pub(crate) const ADMISSION_BUSY: &str = "request protection witness: admission lock busy";
/// A join failure worth retrying soon (the admission lock was busy).
pub fn transient(error: &StoreError) -> bool {
    matches!(error, StoreError::Invalid(message) if *message == ADMISSION_BUSY)
}
fn open_rw(path: &Path) -> Result<File, StoreError> {
    OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path).map_err(io)
}
/// An unreadable or malformed state line reads as `None`: callers treat it as a
/// change (an owner invalidates; a joiner rewrites it).
fn read_state(path: &Path) -> Result<Option<StateLine>, StoreError> {
    let mut raw = String::new();
    match File::open(path) {
        Ok(file) => {
            file.take(256).read_to_string(&mut raw).map_err(io)?;
            Ok(StateLine::parse(&raw))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Some(StateLine::default())),
        Err(error) => Err(io(error)),
    }
}
fn write_state(path: &Path, state: StateLine) -> Result<(), StoreError> {
    let mut file =
        OpenOptions::new().write(true).create(true).truncate(true).open(path).map_err(io)?;
    file.write_all(state.render().as_bytes()).map_err(io)
}

/// The admission lock, held for the lifetime of the value.
struct Admission(#[allow(dead_code)] File);
impl Admission {
    fn try_take(paths: &Paths) -> Result<Option<Self>, StoreError> {
        let file = open_rw(&paths.admission)?;
        match file.try_lock() {
            Ok(()) => Ok(Some(Self(file))),
            Err(std::fs::TryLockError::WouldBlock) => Ok(None),
            Err(std::fs::TryLockError::Error(error)) => Err(io(error)),
        }
    }
    async fn take(paths: &Paths) -> Result<Self, StoreError> {
        let until = tokio::time::Instant::now() + ADMISSION_WAIT;
        loop {
            if let Some(admission) = Self::try_take(paths)? {
                return Ok(admission);
            }
            if tokio::time::Instant::now() >= until {
                return Err(StoreError::Invalid(ADMISSION_BUSY));
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    }
}

/// The token of the lock file now at `path` (`None`: missing or unreadable).
fn token_at(path: &Path) -> Option<String> {
    let mut raw = String::new();
    File::open(path).ok()?.take(64).read_to_string(&mut raw).ok()?;
    let token = raw.trim();
    (token.len() == 32 && token.bytes().all(|byte| byte.is_ascii_hexdigit())).then(|| token.to_owned())
}
/// Caller holds the admission lock and no participant holds `file` yet when it
/// is empty: write a new token into a new (empty) lock file.
fn ensure_token(path: &Path, file: &mut File) -> Result<String, StoreError> {
    if let Some(token) = token_at(path) {
        return Ok(token);
    }
    let token = format!("{:032x}", rand::random::<u128>());
    file.set_len(0).map_err(io)?;
    file.write_all(token.as_bytes()).map_err(io)?;
    file.sync_all().map_err(io)?;
    Ok(token)
}

/// One ownership check: the result and the instant taken before the check
/// took the admission lock, from which the owner's trust window is counted.
#[derive(Clone, Copy, Debug)]
pub struct Checked {
    pub ownership: Ownership,
    pub started: std::time::Instant,
}

/// Result of one ownership check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ownership {
    /// No other participant exists. `changed`: a participant joined since this
    /// one's previous check (it may have written and left), or the state line was
    /// unreadable; cached subjects must be re-read once.
    Alone { changed: bool },
    /// Another participant holds the lock file, the database file has more than
    /// one hard link, or ownership is impossible for this location.
    Shared,
    /// The admission lock was busy (a join or another check in progress).
    Busy,
    /// The lock file was removed or replaced, or a lock call failed; this
    /// participant must join again before it writes.
    Lost,
}

#[derive(Debug)]
enum Kind {
    File {
        paths: Paths,
        database: PathBuf,
        file: File,
        token: String,
        /// Joins seen by this participant's previous check; one check at a time.
        seen_joins: std::sync::Mutex<Option<u64>>,
    },
    Private,
    Shared,
    Postgres(super::pg_owner::PgWitness),
}

/// A writer of the request-protection tables (see module docs). Dropping it
/// releases its lock.
#[derive(Debug)]
pub struct ProtectionParticipant {
    kind: Kind,
    /// No owner was waiting on this join, or the owner acknowledged it in time.
    acknowledged: bool,
}

impl ProtectionParticipant {
    pub(crate) async fn join(location: &Location) -> Result<Self, StoreError> {
        let (paths, database) = match location {
            Location::Private => return Ok(Self { kind: Kind::Private, acknowledged: true }),
            Location::Shared => return Ok(Self { kind: Kind::Shared, acknowledged: true }),
            Location::Unresolved => {
                return Err(StoreError::Invalid("request protection witness: database path unresolved"));
            }
            Location::File(canonical) => (Paths::new(canonical), canonical.clone()),
            Location::Postgres { pool, id } => {
                let (witness, acknowledged) = super::pg_owner::PgWitness::join(pool, id).await?;
                return Ok(Self { kind: Kind::Postgres(witness), acknowledged });
            }
        };
        let admission = Admission::take(&paths).await?;
        let mut file = open_rw(&paths.lock)?;
        let token = ensure_token(&paths.lock, &mut file)?;
        // Exclusive locks on this file are only taken under the admission lock,
        // which this joiner holds, so the shared lock is free to take.
        file.try_lock_shared().map_err(|error| match error {
            std::fs::TryLockError::WouldBlock => {
                StoreError::Invalid("request protection witness: lock file held exclusively")
            }
            std::fs::TryLockError::Error(error) => io(error),
        })?;
        let mut state = read_state(&paths.state)?.unwrap_or_default();
        state.joins = state.joins.saturating_add(1);
        let mine = state.joins;
        let alone = Self::alone_now(&file)?;
        if alone {
            // No other participant: any owner recorded in the file has exited.
            state.owner = false;
            state.acknowledged = mine;
        }
        let wait = state.owner && state.acknowledged < mine;
        let window = state.window();
        write_state(&paths.state, state)?;
        drop(admission);
        let mut acknowledged = !wait;
        if wait {
            let until = tokio::time::Instant::now() + join_wait(window).min(JOIN_WAIT_CAP);
            while tokio::time::Instant::now() < until {
                tokio::time::sleep(POLL).await;
                let Some(admission) = Admission::try_take(&paths)? else { continue };
                let current = read_state(&paths.state)?;
                drop(admission);
                if current.is_none_or(|current| !current.owner || current.acknowledged >= mine) {
                    acknowledged = true;
                    break;
                }
            }
        }
        Ok(Self {
            kind: Kind::File { paths, database, file, token, seen_joins: std::sync::Mutex::new(Some(mine)) },
            acknowledged,
        })
    }

    /// Caller holds the admission lock and a shared lock on `file`: release it,
    /// try an exclusive lock, and take the shared lock back.
    fn alone_now(file: &File) -> Result<bool, StoreError> {
        file.unlock().map_err(io)?;
        let alone = match file.try_lock() {
            Ok(()) => {
                file.unlock().map_err(io)?;
                true
            }
            Err(std::fs::TryLockError::WouldBlock) => false,
            Err(std::fs::TryLockError::Error(error)) => {
                let _ = file.try_lock_shared();
                return Err(io(error));
            }
        };
        file.try_lock_shared().map_err(|error| match error {
            std::fs::TryLockError::WouldBlock => {
                StoreError::Invalid("request protection witness: shared lock not restored")
            }
            std::fs::TryLockError::Error(error) => io(error),
        })?;
        Ok(alone)
    }

    /// One ownership check (see module docs) for an owner whose trust window is
    /// `window`. SQLite: no SQL, a few file system calls. PostgreSQL: one short
    /// transaction on the witness session.
    pub async fn check(&self, window: Duration) -> Checked {
        if let Kind::Postgres(witness) = &self.kind {
            return witness.check(window).await;
        }
        let started = std::time::Instant::now();
        let ownership = self.check_at(window);
        Checked { ownership, started }
    }
    fn check_at(&self, window: Duration) -> Ownership {
        let (paths, database, file, token, seen_joins) = match &self.kind {
            Kind::Private => return Ownership::Alone { changed: false },
            Kind::Shared | Kind::Postgres(_) => return Ownership::Shared,
            Kind::File { paths, database, file, token, seen_joins } => (paths, database, file, token, seen_joins),
        };
        let mut seen_joins = seen_joins.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let admission = match Admission::try_take(paths) {
            Ok(Some(admission)) => admission,
            Ok(None) => return Ownership::Busy,
            Err(_) => return Ownership::Lost,
        };
        if token_at(&paths.lock).as_deref() != Some(token.as_str()) {
            return Ownership::Lost;
        }
        let alone = match Self::alone_now(file) {
            Ok(alone) => alone,
            Err(_) => return Ownership::Lost,
        };
        let state = match read_state(&paths.state) {
            Ok(state) => state,
            Err(_) => return Ownership::Lost,
        };
        // Another name (hard link) of the database file would use another witness.
        // No file at the name: a generation database, opened through its manifest
        // by every process, so the name itself is the identity.
        let single = match mw_platform_fs::file_identity(database) {
            Ok(identity) => identity.links == 1,
            Err(error) => error.kind() == std::io::ErrorKind::NotFound && !database.exists(),
        };
        let alone = alone && single;
        let changed = state.is_none_or(|state| Some(state.joins) != *seen_joins);
        let mut next = state.unwrap_or_default();
        next.acknowledged = next.joins;
        next.owner = alone;
        next.window_ms = if alone { u64::try_from(window.as_millis()).unwrap_or(u64::MAX) } else { 0 };
        if state != Some(next) && write_state(&paths.state, next).is_err() {
            return Ownership::Lost;
        }
        *seen_joins = Some(next.joins);
        drop(admission);
        if alone { Ownership::Alone { changed } } else { Ownership::Shared }
    }

    /// Whether ownership is possible at all (a SQLite file or a private database).
    pub fn can_own(&self) -> bool {
        match &self.kind {
            Kind::Shared => false,
            Kind::Postgres(witness) => witness.stable(),
            _ => true,
        }
    }
    /// The server process id of the PostgreSQL witness session (`None` otherwise).
    pub async fn witness_backend_pid(&self) -> Option<i32> {
        match &self.kind {
            Kind::Postgres(witness) => witness.backend_pid().await,
            _ => None,
        }
    }
    /// The witness session still answers (PostgreSQL; always true otherwise).
    pub fn alive(&self) -> bool {
        match &self.kind {
            Kind::Postgres(witness) => witness.alive(),
            _ => true,
        }
    }
    /// Probe the witness session now (PostgreSQL liveness; see `pg_owner`).
    pub async fn probe(&self) -> bool {
        match &self.kind {
            Kind::Postgres(witness) => witness.probe().await,
            _ => true,
        }
    }
    /// See the `acknowledged` field.
    pub fn acknowledged(&self) -> bool {
        self.acknowledged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: Duration = Duration::from_millis(500);
    fn location(dir: &tempfile::TempDir) -> Location {
        let database = dir.path().join("witness.db");
        if !database.exists() {
            std::fs::write(&database, b"").unwrap();
        }
        Location::File(std::fs::canonicalize(database).unwrap())
    }

    #[tokio::test]
    async fn a_lone_participant_owns_and_a_second_one_ends_ownership() {
        let dir = tempfile::tempdir().unwrap();
        let first = ProtectionParticipant::join(&location(&dir)).await.unwrap();
        assert_eq!(first.check(W).await.ownership, Ownership::Alone { changed: false });
        assert_eq!(first.check(W).await.ownership, Ownership::Alone { changed: false });
        // A second participant joins: the owner has acknowledged nothing yet, so
        // the joiner waits for the owner's next check.
        let started = std::time::Instant::now();
        let joining = tokio::spawn({
            let location = location(&dir);
            async move { ProtectionParticipant::join(&location).await.unwrap() }
        });
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(!joining.is_finished(), "the joiner waits for the owner's acknowledgement");
        assert_eq!(first.check(W).await.ownership, Ownership::Shared);
        let second = joining.await.unwrap();
        assert!(started.elapsed() < join_wait(W));
        assert!(second.acknowledged());
        assert_eq!(second.check(W).await.ownership, Ownership::Shared);
        drop(second);
        // Alone again. The join was already seen by the previous check, so the
        // caller (which stopped trusting its cache at `Shared`) is not told twice.
        assert_eq!(first.check(W).await.ownership, Ownership::Alone { changed: false });
    }

    #[tokio::test]
    async fn a_writer_that_joined_and_left_between_two_checks_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let first = ProtectionParticipant::join(&location(&dir)).await.unwrap();
        // No check yet, so no owner is recorded and the next joiner does not wait.
        let transient = ProtectionParticipant::join(&location(&dir)).await.unwrap();
        drop(transient);
        assert_eq!(first.check(W).await.ownership, Ownership::Alone { changed: true });
        assert_eq!(first.check(W).await.ownership, Ownership::Alone { changed: false });
        // An unreadable state line also counts as a change.
        std::fs::write(Paths::new(&std::fs::canonicalize(dir.path().join("witness.db")).unwrap()).state, b"garbage").unwrap();
        assert_eq!(first.check(W).await.ownership, Ownership::Alone { changed: true });
        assert_eq!(first.check(W).await.ownership, Ownership::Alone { changed: false });
    }

    #[tokio::test]
    async fn a_joiner_that_finds_itself_alone_does_not_wait_for_a_departed_owner() {
        let dir = tempfile::tempdir().unwrap();
        let owner = ProtectionParticipant::join(&location(&dir)).await.unwrap();
        assert_eq!(owner.check(W).await.ownership, Ownership::Alone { changed: false });
        drop(owner); // exits without clearing the owner flag
        let started = std::time::Instant::now();
        let next = ProtectionParticipant::join(&location(&dir)).await.unwrap();
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_eq!(next.check(W).await.ownership, Ownership::Alone { changed: false });
    }

    #[tokio::test]
    async fn a_replaced_lock_file_is_reported_lost() {
        let dir = tempfile::tempdir().unwrap();
        let owner = ProtectionParticipant::join(&location(&dir)).await.unwrap();
        assert_eq!(owner.check(W).await.ownership, Ownership::Alone { changed: false });
        let lock = Paths::new(&std::fs::canonicalize(dir.path().join("witness.db")).unwrap()).lock;
        if std::fs::remove_file(&lock).is_ok() && std::fs::write(&lock, b"").is_ok() {
            assert_eq!(owner.check(W).await.ownership, Ownership::Lost);
            // A new participant on the new file is alone and owns.
            let next = ProtectionParticipant::join(&location(&dir)).await.unwrap();
            assert!(matches!(next.check(W).await.ownership, Ownership::Alone { .. }));
        } else {
            eprintln!("[witness] a held lock file cannot be replaced on this platform");
        }
    }

    #[tokio::test]
    async fn private_and_shared_locations() {
        let private = ProtectionParticipant::join(&Location::Private).await.unwrap();
        assert_eq!(private.check(W).await.ownership, Ownership::Alone { changed: false });
        let shared = ProtectionParticipant::join(&Location::Shared).await.unwrap();
        assert_eq!(shared.check(W).await.ownership, Ownership::Shared);
        assert!(!shared.can_own() && private.can_own());
    }

    #[test]
    fn state_lines_round_trip_and_reject_garbage() {
        let line = StateLine { joins: 7, acknowledged: 5, owner: true, window_ms: 500 };
        assert_eq!(StateLine::parse(&line.render()), Some(line));
        let legacy = StateLine::parse("v1 3 2 1").unwrap();
        assert_eq!((legacy.joins, legacy.window()), (3, LEGACY_WINDOW));
        for raw in ["", "v3 1 1 1 1", "v2 1 1 1", "v1 1 1", "v1 1 1 2", "v1 x 1 1", "v1 1 1 1 extra", "v2 1 1 1 x"] {
            assert_eq!(StateLine::parse(raw), None, "{raw}");
        }
    }

    #[tokio::test]
    async fn a_hard_linked_database_is_never_owned() {
        let dir = tempfile::tempdir().unwrap();
        let owner = ProtectionParticipant::join(&location(&dir)).await.unwrap();
        assert_eq!(owner.check(W).await.ownership, Ownership::Alone { changed: false });
        std::fs::hard_link(dir.path().join("witness.db"), dir.path().join("second-name.db")).unwrap();
        assert_eq!(owner.check(W).await.ownership, Ownership::Shared, "another name could use another witness");
        std::fs::remove_file(dir.path().join("second-name.db")).unwrap();
        assert!(matches!(owner.check(W).await.ownership, Ownership::Alone { .. }));
    }

    #[tokio::test]
    async fn an_unresolved_database_path_cannot_join() {
        assert!(ProtectionParticipant::join(&Location::Unresolved).await.is_err());
    }

    #[tokio::test]
    async fn a_joiner_that_is_not_acknowledged_in_time_is_marked_unacknowledged() {
        let dir = tempfile::tempdir().unwrap();
        let owner = ProtectionParticipant::join(&location(&dir)).await.unwrap();
        // A short window: the joiner waits window + 2 s, then gives up.
        let window = Duration::from_millis(100);
        assert!(matches!(owner.check(window).await.ownership, Ownership::Alone { .. }));
        let started = std::time::Instant::now();
        let joiner = ProtectionParticipant::join(&location(&dir)).await.unwrap();
        let waited = started.elapsed();
        assert!(!joiner.acknowledged(), "the owner never checked");
        assert!(waited >= join_wait(window) && waited < join_wait(window) + Duration::from_secs(2), "{waited:?}");
        assert_eq!(owner.check(window).await.ownership, Ownership::Shared);
    }

    #[test]
    fn the_window_and_the_join_wait_follow_the_flush_interval() {
        assert_eq!(ownership_window(Duration::from_millis(250)).unwrap(), Duration::from_millis(500));
        assert_eq!(join_wait(Duration::from_millis(500)), Duration::from_millis(2500));
        assert_eq!(join_wait(Duration::from_secs(10)), Duration::from_secs(20));
        // The largest configurable interval (5 s) is accepted; longer ones are refused.
        assert!(ownership_window(Duration::from_secs(5)).is_ok());
        assert!(ownership_window(Duration::from_secs(8)).is_err());
    }
}
