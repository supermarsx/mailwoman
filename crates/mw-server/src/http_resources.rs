//! What the process has (CPUs, memory, open files) and the HTTP limits derived from
//! it when the operator sets none. The formulas live here as pure functions over a
//! [`Resources`] value so they can be tested with any reading; [`Resources::probe`]
//! is the only part that looks at the running system.
//!
//! Formulas (C = CPUs, M = memory bytes, F = open-file limit, P = database pool
//! maximum, H = TLS + PROXY handshake slots):
//!
//! * reserve R = 512 + P + H descriptors (logs, database files, index segments,
//!   static files, the pool, sockets still in a handshake);
//! * connections = max(1024, min((F − R) / 2, M / 1 MiB, 65536)). Half of the
//!   descriptors left after the reserve go to client connections; the other half
//!   stays for upstream IMAP/SMTP connections (one or more per active proxy
//!   session), outbound HTTP and anything else a handler opens;
//! * requests = clamp(max(64·C, 16·P), 256, min(16384, M / 4 MiB));
//! * HTTP/2 streams per connection = clamp(requests / 8, 32, 100): one connection
//!   can hold at most an eighth of the request pool, as 32 of 256 did before;
//! * queue = requests.
//!
//! A reading that is missing drops its term; with no reading at all every value is
//! the fixed default (1024, 256, 32), which is also every floor.

/// Fixed defaults, which are also the floors of the derived values.
pub const CONNECTIONS_FLOOR: usize = 1024;
pub const REQUESTS_FLOOR: usize = 256;
pub const STREAMS_FLOOR: u32 = 32;
/// Hard ceilings of the derived values; also the top of the explicit ranges.
pub const CONNECTIONS_CEILING: usize = 65536;
pub const REQUESTS_CEILING: usize = 16384;
pub const STREAMS_AUTO_CEILING: u32 = 100;
/// Descriptors kept for everything that is not a client connection or the pool.
pub const FIXED_FD_RESERVE: u64 = 512;
/// Memory budgeted per connection and per request in progress.
pub const BYTES_PER_CONNECTION: u64 = 1024 * 1024;
pub const BYTES_PER_REQUEST: u64 = 4 * 1024 * 1024;
/// The soft open-file limit is raised towards the hard limit, but not past this.
pub const OPEN_FILES_RAISE_CAP: u64 = 1_048_576;

/// What the process has. `None` = not readable on this platform or right now.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Resources {
    /// CPUs this process may use (cgroup quota and affinity applied on Linux).
    pub cpus: Option<u32>,
    /// Memory limit in bytes: the lower of the cgroup limit and physical memory.
    pub memory_bytes: Option<u64>,
    /// Soft open-file limit (RLIMIT_NOFILE), after any raise.
    pub open_files: Option<u64>,
}

/// The derived values and how they came about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Auto {
    pub connections: usize,
    pub requests: usize,
    pub http2_streams: u32,
    /// R, when the open-file limit is known.
    pub fd_reserve: Option<u64>,
    /// The open-file limit leaves room for fewer connections than the floor.
    pub fd_short: bool,
}

/// Derive the limits from `resources`, a database pool of `db_pool` connections and
/// `handshakes` TLS/PROXY handshake slots.
pub fn derive(resources: &Resources, db_pool: u32, handshakes: u64) -> Auto {
    let reserve = FIXED_FD_RESERVE
        .saturating_add(u64::from(db_pool))
        .saturating_add(handshakes);
    let fd_term = resources
        .open_files
        .map(|files| files.saturating_sub(reserve) / 2);
    let memory_connections = resources.memory_bytes.map(|m| m / BYTES_PER_CONNECTION);
    let mut connections = CONNECTIONS_FLOOR as u64;
    if fd_term.is_some() || memory_connections.is_some() {
        let bound = [fd_term, memory_connections]
            .into_iter()
            .flatten()
            .fold(CONNECTIONS_CEILING as u64, u64::min);
        connections = connections.max(bound);
    }
    let fd_short = fd_term.is_some_and(|room| room < CONNECTIONS_FLOOR as u64);

    let cpus = u64::from(resources.cpus.unwrap_or(0));
    let wanted = (64 * cpus).max(16 * u64::from(db_pool));
    let ceiling = resources
        .memory_bytes
        .map(|m| m / BYTES_PER_REQUEST)
        .unwrap_or(u64::MAX)
        .min(REQUESTS_CEILING as u64);
    let requests = wanted.min(ceiling).max(REQUESTS_FLOOR as u64);

    Auto {
        connections: connections as usize,
        requests: requests as usize,
        http2_streams: streams_for(requests as usize),
        fd_reserve: resources.open_files.map(|_| reserve),
        fd_short,
    }
}

/// HTTP/2 streams per connection for a request limit: an eighth of it, 32..=100.
pub fn streams_for(requests: usize) -> u32 {
    ((requests / 8) as u32).clamp(STREAMS_FLOOR, STREAMS_AUTO_CEILING)
}

/// The database pool maximum the store will use: `MW_DB_POOL`'s explicit maximum
/// for the backend, else the store's own default (`mw_store::pool_policy`:
/// PostgreSQL 4 per CPU, 4..=64; SQLite 2 per CPU, 2..=8). A maximum set through
/// the stored `database.pool` setting is not visible before the database opens.
pub fn db_pool_estimate(db_path: Option<&str>, pool_json: Option<&str>, cpus: u32) -> u32 {
    let postgres =
        db_path.is_some_and(|p| p.starts_with("postgres://") || p.starts_with("postgresql://"));
    let field = if postgres {
        "postgresMaxConnections"
    } else {
        "sqliteMaxConnections"
    };
    let pinned = pool_json
        .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
        .and_then(|value| value.get(field).and_then(serde_json::Value::as_u64))
        .map(|n| n.min(u64::from(u32::MAX)) as u32);
    let cpus = cpus.max(1);
    pinned.unwrap_or(if postgres {
        cpus.saturating_mul(4).clamp(4, 64)
    } else {
        cpus.saturating_mul(2).clamp(2, 8)
    })
}

/// The adaptive limiter's floor: max(256, 16·C), never above `ceiling`.
pub fn adaptive_floor(cpus: Option<u32>, ceiling: usize) -> usize {
    (16 * cpus.unwrap_or(0) as usize)
        .max(REQUESTS_FLOOR)
        .min(ceiling)
        .max(1)
}

impl Resources {
    /// Read the running system. With `raise_open_files`, the soft open-file limit
    /// is first raised to min(hard limit, [`OPEN_FILES_RAISE_CAP`]) where the
    /// platform allows it.
    pub fn probe(raise_open_files: bool) -> Self {
        Self {
            cpus: std::thread::available_parallelism()
                .ok()
                .map(|n| n.get().min(u32::MAX as usize) as u32),
            memory_bytes: memory_limit(),
            open_files: open_files(raise_open_files),
        }
    }
}

/// A cgroup memory limit file's content: v2 `max` or a v1 value near `u64::MAX`
/// mean no limit.
pub fn parse_cgroup_memory(text: &str) -> Option<u64> {
    let text = text.trim();
    if text == "max" {
        return None;
    }
    text.parse::<u64>().ok().filter(|&n| n > 0 && n < (1 << 60))
}

/// `MemTotal` from `/proc/meminfo`, in bytes.
pub fn parse_meminfo(text: &str) -> Option<u64> {
    text.lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))
        .and_then(|rest| {
            rest.trim()
                .trim_end_matches("kB")
                .trim()
                .parse::<u64>()
                .ok()
        })
        .map(|kib| kib.saturating_mul(1024))
        .filter(|&n| n > 0)
}

/// The cgroup files that may hold this process's memory limit, from
/// `/proc/self/cgroup`: its own group first, then the root of the mount.
pub fn cgroup_memory_files(self_cgroup: &str) -> Vec<String> {
    let mut files = Vec::new();
    for line in self_cgroup.lines() {
        let mut parts = line.splitn(3, ':');
        let (Some(_), Some(controllers), Some(path)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        let path = path.trim_end_matches('/');
        if controllers.is_empty() {
            files.push(format!("/sys/fs/cgroup{path}/memory.max"));
        } else if controllers.split(',').any(|c| c == "memory") {
            files.push(format!("/sys/fs/cgroup/memory{path}/memory.limit_in_bytes"));
        }
    }
    files.push("/sys/fs/cgroup/memory.max".into());
    files.push("/sys/fs/cgroup/memory/memory.limit_in_bytes".into());
    files.dedup();
    files
}

/// The memory limit from the file contents: the lowest cgroup limit, else physical
/// memory, whichever is lower. Unreadable files are `None` and ignored.
pub fn memory_from(meminfo: Option<&str>, cgroup_files: &[Option<String>]) -> Option<u64> {
    let physical = meminfo.and_then(parse_meminfo);
    let cgroup = cgroup_files
        .iter()
        .filter_map(|text| text.as_deref().and_then(parse_cgroup_memory))
        .min();
    [physical, cgroup].into_iter().flatten().min()
}

#[cfg(target_os = "linux")]
fn memory_limit() -> Option<u64> {
    let read = |path: &str| std::fs::read_to_string(path).ok();
    let files: Vec<Option<String>> = read("/proc/self/cgroup")
        .map(|text| cgroup_memory_files(&text))
        .unwrap_or_default()
        .iter()
        .map(|file| read(file))
        .collect();
    memory_from(read("/proc/meminfo").as_deref(), &files)
}
#[cfg(not(target_os = "linux"))]
fn memory_limit() -> Option<u64> {
    None
}

#[cfg(unix)]
#[allow(clippy::unnecessary_cast, clippy::useless_conversion)]
fn open_files(raise: bool) -> Option<u64> {
    let mut limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: getrlimit writes one rlimit into the struct we own.
    if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit) } != 0 {
        return None;
    }
    let current = limit.rlim_cur as u64;
    let hard = limit.rlim_max as u64;
    if raise && current < hard.min(OPEN_FILES_RAISE_CAP) {
        // macOS refuses values above OPEN_MAX (10240) even when the hard limit is
        // unlimited, so try that if the first raise is refused.
        for target in [hard.min(OPEN_FILES_RAISE_CAP), hard.min(10240)] {
            if target <= current {
                continue;
            }
            let raised = libc::rlimit {
                rlim_cur: target as libc::rlim_t,
                rlim_max: limit.rlim_max,
            };
            // SAFETY: setrlimit reads one rlimit we own; failure leaves the limit as is.
            if unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &raised) } == 0 {
                return Some(target);
            }
        }
    }
    Some(current)
}
#[cfg(not(unix))]
fn open_files(_raise: bool) -> Option<u64> {
    // Windows sockets are not counted against a per-process descriptor limit like
    // RLIMIT_NOFILE, so there is no reading and no fd term.
    None
}

#[cfg(test)]
#[path = "http_resources_tests.rs"]
mod tests;
