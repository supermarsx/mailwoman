//! Server log destinations the server controls: stdout always, plus optional
//! size- and age-rotated files, with redaction of addresses and secrets and a
//! level per component.
//!
//! The policy is the `logging.*` settings. They are read after the store opens,
//! so until then the defaults apply (stdout only, `MW_LOG` level, secrets cut).
//! Under a container the stdout stream is what the platform collects; rotation
//! of that stream belongs to the platform (for example the Docker `json-file`
//! driver's `max-size` and `max-file`), and this module then still provides the
//! redaction and level controls.
//!
//! Rotated files are named `mailwoman-<unix seconds>-<n>.log` (`.gz` when
//! compressed); the current file is `mailwoman.log`. File age comes from the
//! name, so no clock or file-system time is needed to decide what to delete.

use regex::Regex;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    net::{Ipv4Addr, Ipv6Addr},
    path::{Component, Path, PathBuf},
    sync::{Mutex, OnceLock, RwLock},
};

/// How an address appears in a log line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Redaction {
    Full,
    Truncated,
    Hashed,
    None,
}

impl Redaction {
    pub fn parse(s: &str) -> Self {
        match s {
            "truncated" => Self::Truncated,
            "hashed" => Self::Hashed,
            "none" => Self::None,
            _ => Self::Full,
        }
    }
}

/// The effective `logging.*` settings.
#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    pub file_enabled: bool,
    pub directory: PathBuf,
    pub max_size_bytes: u64,
    /// 0 = rotate by size only.
    pub rotate_secs: i64,
    pub max_age_days: i64,
    pub max_files: usize,
    pub max_total_bytes: u64,
    pub compress: bool,
    pub level: String,
    /// `component=level` entries.
    pub components: Vec<String>,
    pub addresses: Redaction,
    pub ips: Redaction,
    pub tokens: bool,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            file_enabled: false,
            directory: PathBuf::from("logs"),
            max_size_bytes: 50 << 20,
            rotate_secs: 24 * 3600,
            max_age_days: 14,
            max_files: 14,
            max_total_bytes: 500 << 20,
            compress: true,
            level: "info".into(),
            components: Vec::new(),
            addresses: Redaction::Full,
            ips: Redaction::Full,
            tokens: true,
        }
    }
}

/// A directory the server may write logs to: no parent-directory segments.
pub fn directory_is_valid(dir: &str) -> bool {
    let p = Path::new(dir);
    !dir.trim().is_empty()
        && p.components()
            .all(|c| !matches!(c, Component::ParentDir))
}

/// The level directives for `level` and the component entries, or `None` for the
/// defaults (so the start-up directives stay in force). Entries that do not parse
/// as `component=level` are dropped.
pub fn directives(level: &str, components: &[String]) -> String {
    let valid = |l: &str| matches!(l, "error" | "warn" | "info" | "debug" | "trace");
    let mut out = vec![if valid(level) { level.to_string() } else { "info".into() }];
    for c in components {
        if let Some((name, lvl)) = c.split_once('=') {
            let name = name.trim();
            let lvl = lvl.trim().to_ascii_lowercase();
            if !name.is_empty()
                && name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b':' | b'-'))
                && valid(&lvl)
            {
                out.push(format!("{name}={lvl}"));
            }
        }
    }
    out.join(",")
}

// ─── redaction ─────────────────────────────────────────────────────────────────

fn email_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}").unwrap())
}
fn v4_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap())
}
fn v6_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?:[0-9A-Fa-f]{0,4}:){2,7}[0-9A-Fa-f]{0,4}").unwrap())
}
fn ansi_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new("\u{1b}\\[[0-9;]*m").unwrap())
}

fn salt() -> &'static str {
    static SALT: OnceLock<String> = OnceLock::new();
    SALT.get_or_init(|| uuid::Uuid::new_v4().to_string())
}

fn short_hash(input: &str) -> String {
    let d = Sha256::digest(format!("{}|{input}", salt()).as_bytes());
    d.iter().take(6).map(|b| format!("{b:02x}")).collect()
}

fn reduce_email(addr: &str, mode: Redaction) -> String {
    let Some((local, domain)) = addr.split_once('@') else {
        return addr.to_string();
    };
    match mode {
        Redaction::Full => addr.to_string(),
        Redaction::Truncated => {
            let first: String = local.chars().take(1).collect();
            format!("{first}***@{domain}")
        }
        Redaction::Hashed => format!("h:{}@{domain}", short_hash(&local.to_ascii_lowercase())),
        Redaction::None => "[address]".into(),
    }
}

fn reduce_ip(ip: &str, mode: Redaction) -> String {
    match mode {
        Redaction::Full => ip.to_string(),
        Redaction::Hashed => format!("h:{}", short_hash(ip)),
        Redaction::None => "[ip]".into(),
        Redaction::Truncated => {
            if let Ok(a) = ip.parse::<Ipv4Addr>() {
                let o = a.octets();
                format!("{}.{}.{}.0", o[0], o[1], o[2])
            } else if let Ok(a) = ip.parse::<Ipv6Addr>() {
                let s = a.segments();
                format!("{:x}:{:x}:{:x}::", s[0], s[1], s[2])
            } else {
                ip.to_string()
            }
        }
    }
}

/// `line` with the policy's redactions applied.
pub fn redact(line: &str, p: &Policy) -> String {
    let mut s = line.to_string();
    if p.tokens {
        s = crate::application_logs::redact_text(&s);
    }
    if p.addresses != Redaction::Full {
        s = email_re()
            .replace_all(&s, |c: &regex::Captures| reduce_email(&c[0], p.addresses))
            .into_owned();
    }
    if p.ips != Redaction::Full {
        s = v4_re()
            .replace_all(&s, |c: &regex::Captures| {
                if c[0].parse::<Ipv4Addr>().is_ok() {
                    reduce_ip(&c[0], p.ips)
                } else {
                    c[0].to_string()
                }
            })
            .into_owned();
        s = v6_re()
            .replace_all(&s, |c: &regex::Captures| {
                if c[0].parse::<Ipv6Addr>().is_ok() {
                    reduce_ip(&c[0], p.ips)
                } else {
                    c[0].to_string()
                }
            })
            .into_owned();
    }
    s
}

// ─── rotating file ─────────────────────────────────────────────────────────────

const CURRENT: &str = "mailwoman.log";

/// One rotated file found in the directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rotated {
    pub name: String,
    pub stamp: i64,
    pub bytes: u64,
}

fn parse_rotated(name: &str) -> Option<i64> {
    let rest = name.strip_prefix("mailwoman-")?;
    let rest = rest
        .strip_suffix(".log.gz")
        .or_else(|| rest.strip_suffix(".log"))?;
    rest.split('-').next()?.parse().ok()
}

/// The rotated files in `dir`, oldest first.
pub fn list_rotated(dir: &Path) -> Vec<Rotated> {
    let mut v: Vec<Rotated> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let stamp = parse_rotated(&name)?;
            let bytes = e.metadata().ok()?.len();
            Some(Rotated { name, stamp, bytes })
        })
        .collect();
    v.sort_by(|a, b| (a.stamp, &a.name).cmp(&(b.stamp, &b.name)));
    v
}

/// Delete rotated files past the age, count and total-size limits, oldest first.
/// `current_bytes` is the live file, which counts toward the total but is never
/// deleted. Returns the names removed.
pub fn enforce_limits(dir: &Path, p: &Policy, now: i64, current_bytes: u64) -> Vec<String> {
    let mut files = list_rotated(dir);
    let mut removed = Vec::new();
    let mut drop = |files: &mut Vec<Rotated>, i: usize, removed: &mut Vec<String>| {
        let f = files.remove(i);
        if fs::remove_file(dir.join(&f.name)).is_ok() {
            removed.push(f.name);
        }
    };
    let cutoff = now - p.max_age_days * 86_400;
    while files.first().is_some_and(|f| f.stamp < cutoff) {
        drop(&mut files, 0, &mut removed);
    }
    while files.len() > p.max_files {
        drop(&mut files, 0, &mut removed);
    }
    while !files.is_empty()
        && files.iter().map(|f| f.bytes).sum::<u64>() + current_bytes > p.max_total_bytes
    {
        drop(&mut files, 0, &mut removed);
    }
    removed
}

fn gzip_file(path: &Path) -> io::Result<PathBuf> {
    let mut gz = path.as_os_str().to_owned();
    gz.push(".gz");
    let gz = PathBuf::from(gz);
    let mut input = File::open(path)?;
    let mut enc = flate2::write::GzEncoder::new(File::create(&gz)?, flate2::Compression::default());
    io::copy(&mut input, &mut enc)?;
    enc.finish()?;
    fs::remove_file(path)?;
    Ok(gz)
}

/// The current log file and its rotation state.
pub struct FileSink {
    dir: PathBuf,
    file: Option<File>,
    size: u64,
    opened_at: i64,
}

impl FileSink {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            file: None,
            size: 0,
            opened_at: 0,
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    fn open(&mut self, now: i64) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let path = self.dir.join(CURRENT);
        let f = OpenOptions::new().create(true).append(true).open(&path)?;
        let meta = f.metadata()?;
        self.size = meta.len();
        // An existing file keeps its own age (creation, else last write).
        self.opened_at = if self.size == 0 {
            now
        } else {
            meta.created()
                .or_else(|_| meta.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(now)
        };
        self.file = Some(f);
        Ok(())
    }

    /// Close the current file and move it aside; compression runs on a thread so
    /// a large file does not stall logging.
    pub fn rotate(&mut self, now: i64, p: &Policy) -> io::Result<()> {
        self.file = None;
        let cur = self.dir.join(CURRENT);
        if fs::metadata(&cur).map(|m| m.len()).unwrap_or(0) == 0 {
            return Ok(());
        }
        let mut n = 0;
        let target = loop {
            let t = self.dir.join(format!("mailwoman-{now:010}-{n}.log"));
            let gz = self.dir.join(format!("mailwoman-{now:010}-{n}.log.gz"));
            if !t.exists() && !gz.exists() {
                break t;
            }
            n += 1;
        };
        fs::rename(&cur, &target)?;
        self.size = 0;
        let (dir, policy) = (self.dir.clone(), p.clone());
        let work = move || {
            if policy.compress {
                let _ = gzip_file(&target);
            }
            enforce_limits(&dir, &policy, now, 0);
        };
        if cfg!(test) {
            work();
        } else {
            std::thread::spawn(work);
        }
        Ok(())
    }

    /// Append `bytes`, rotating first when the size or age limit is reached.
    pub fn write(&mut self, bytes: &[u8], now: i64, p: &Policy) -> io::Result<()> {
        if self.file.is_none() {
            self.open(now)?;
        }
        let too_big = self.size > 0 && self.size + bytes.len() as u64 > p.max_size_bytes;
        let too_old = self.size > 0 && p.rotate_secs > 0 && now - self.opened_at >= p.rotate_secs;
        if too_big || too_old {
            self.rotate(now, p)?;
            self.open(now)?;
        }
        if let Some(f) = self.file.as_mut() {
            f.write_all(bytes)?;
            self.size += bytes.len() as u64;
        }
        Ok(())
    }
}

// ─── the live writer ───────────────────────────────────────────────────────────

fn policy_cell() -> &'static RwLock<Policy> {
    static P: OnceLock<RwLock<Policy>> = OnceLock::new();
    P.get_or_init(|| RwLock::new(Policy::default()))
}
fn sink_cell() -> &'static Mutex<Option<FileSink>> {
    static S: OnceLock<Mutex<Option<FileSink>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(None))
}

/// The policy in force.
pub fn policy() -> Policy {
    policy_cell()
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

/// Replace the policy. A changed directory or a switched-off file closes the
/// current sink.
pub fn set_policy(p: Policy) {
    let mut cur = policy_cell().write().unwrap_or_else(|e| e.into_inner());
    if cur.directory != p.directory || !p.file_enabled {
        *sink_cell().lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
    *cur = p;
}

/// Bytes in the live file and the rotated files, for the console.
pub fn status() -> serde_json::Value {
    let p = policy();
    let rotated = list_rotated(&p.directory);
    let live = fs::metadata(p.directory.join(CURRENT))
        .map(|m| m.len())
        .unwrap_or(0);
    serde_json::json!({
        "fileEnabled": p.file_enabled,
        "directory": p.directory.to_string_lossy(),
        "currentBytes": live,
        "rotatedFiles": rotated.len(),
        "rotatedBytes": rotated.iter().map(|f| f.bytes).sum::<u64>(),
        "oldestStamp": rotated.first().map(|f| f.stamp),
        "newestStamp": rotated.last().map(|f| f.stamp),
    })
}

/// A `tracing_subscriber` writer factory: each event is buffered, redacted and
/// written to stdout and, when enabled, to the rotating file.
#[derive(Clone, Copy, Default)]
pub struct LogWriter;

pub struct EventBuffer(Vec<u8>);

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for LogWriter {
    type Writer = EventBuffer;
    fn make_writer(&'a self) -> EventBuffer {
        EventBuffer(Vec::with_capacity(256))
    }
}

impl Write for EventBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for EventBuffer {
    fn drop(&mut self) {
        if self.0.is_empty() {
            return;
        }
        let p = policy();
        let text = String::from_utf8_lossy(&self.0);
        let line = redact(&text, &p);
        let _ = io::stdout().write_all(line.as_bytes());
        if p.file_enabled {
            let plain = ansi_re().replace_all(&line, "");
            let now = chrono::Utc::now().timestamp();
            let mut cell = sink_cell().lock().unwrap_or_else(|e| e.into_inner());
            let sink = cell.get_or_insert_with(|| FileSink::new(p.directory.clone()));
            // A file that cannot be written must not stop the server logging.
            let _ = sink.write(plain.as_bytes(), now, &p);
        }
    }
}

// ─── level reload ──────────────────────────────────────────────────────────────

struct Levels {
    handle: crate::observability::ReloadHandle,
    initial: String,
    applied: Mutex<Option<String>>,
}

static LEVELS: OnceLock<Levels> = OnceLock::new();

/// Remember the reload handle and the start-up directives (env or `MW_LOG`).
pub fn install_level_handle(handle: crate::observability::ReloadHandle, initial: &str) {
    let _ = LEVELS.set(Levels {
        handle,
        initial: initial.to_string(),
        applied: Mutex::new(None),
    });
}

/// Apply the settings' level. The default level with no component entries leaves
/// the start-up directives in force; going back to it restores them.
pub fn apply_levels(level: &str, components: &[String]) -> Result<(), String> {
    let Some(l) = LEVELS.get() else {
        return Ok(());
    };
    let wanted = if level == "info" && components.is_empty() {
        l.initial.clone()
    } else {
        directives(level, components)
    };
    let mut applied = l.applied.lock().unwrap_or_else(|e| e.into_inner());
    if applied.as_deref() == Some(wanted.as_str()) {
        return Ok(());
    }
    if applied.is_none() && wanted == l.initial {
        *applied = Some(wanted);
        return Ok(());
    }
    l.handle.reload(&wanted)?;
    *applied = Some(wanted);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("mw-logfiles-{tag}-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn small() -> Policy {
        Policy {
            max_size_bytes: 100,
            rotate_secs: 0,
            compress: false,
            ..Policy::default()
        }
    }

    #[test]
    fn size_rotation_moves_the_file_aside() {
        let dir = tmp("size");
        let p = small();
        let mut sink = FileSink::new(dir.clone());
        for i in 0..4 {
            sink.write(&[b'x'; 60], 1_000 + i, &p).unwrap();
        }
        let rotated = list_rotated(&dir);
        assert_eq!(rotated.len(), 3, "{rotated:?}");
        assert!(dir.join(CURRENT).exists());
        assert!(sink.size() <= 100);
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn age_rotation_needs_rotate_secs_and_content() {
        let dir = tmp("age");
        let p = Policy {
            rotate_secs: 3600,
            ..small()
        };
        let mut sink = FileSink::new(dir.clone());
        sink.write(b"a\n", 1_000, &p).unwrap();
        sink.write(b"b\n", 1_000 + 3_599, &p).unwrap();
        assert!(list_rotated(&dir).is_empty());
        sink.write(b"c\n", 1_000 + 3_600, &p).unwrap();
        assert_eq!(list_rotated(&dir).len(), 1);
        let off = Policy {
            rotate_secs: 0,
            ..small()
        };
        sink.write(b"d\n", 1_000_000, &off).unwrap();
        assert_eq!(list_rotated(&dir).len(), 1, "0 means size only");
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn rotated_files_are_gzipped_when_asked() {
        let dir = tmp("gz");
        let p = Policy {
            compress: true,
            ..small()
        };
        let mut sink = FileSink::new(dir.clone());
        sink.write(&[b'a'; 90], 5_000, &p).unwrap();
        sink.write(&[b'b'; 90], 5_001, &p).unwrap();
        let rotated = list_rotated(&dir);
        assert_eq!(rotated.len(), 1);
        assert!(rotated[0].name.ends_with(".log.gz"));
        let mut out = Vec::new();
        io::Read::read_to_end(
            &mut flate2::read::GzDecoder::new(File::open(dir.join(&rotated[0].name)).unwrap()),
            &mut out,
        )
        .unwrap();
        assert_eq!(out, vec![b'a'; 90]);
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn limits_remove_oldest_first_by_age_count_and_total() {
        let dir = tmp("limits");
        for (stamp, size) in [(100, 10), (200, 10), (300, 10), (400, 10)] {
            fs::write(dir.join(format!("mailwoman-{stamp:010}-0.log")), vec![0u8; size]).unwrap();
        }
        fs::write(dir.join("unrelated.txt"), b"keep").unwrap();
        let day = 86_400;
        let p = Policy {
            max_age_days: 1,
            max_files: 10,
            max_total_bytes: 1 << 20,
            ..Policy::default()
        };
        // Files stamped 100 and 200 are older than one day at now = 300 + day.
        let gone = enforce_limits(&dir, &p, 250 + day, 0);
        assert_eq!(gone.len(), 2, "{gone:?}");
        let p = Policy { max_files: 1, ..p };
        enforce_limits(&dir, &p, 250 + day, 0);
        assert_eq!(list_rotated(&dir).len(), 1);
        assert_eq!(list_rotated(&dir)[0].stamp, 400);
        let p = Policy {
            max_files: 10,
            max_total_bytes: 5,
            ..p
        };
        enforce_limits(&dir, &p, 250 + day, 0);
        assert!(list_rotated(&dir).is_empty());
        assert!(dir.join("unrelated.txt").exists());
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn redaction_modes_for_addresses_and_ips() {
        let line = "login ok for jane.doe@example.org from 203.0.113.77 and 2001:db8:abcd:1234::9 at 12:34:56";
        let mut p = Policy {
            tokens: false,
            ..Policy::default()
        };
        assert_eq!(redact(line, &p), line);
        p.addresses = Redaction::Truncated;
        p.ips = Redaction::Truncated;
        assert_eq!(
            redact(line, &p),
            "login ok for j***@example.org from 203.0.113.0 and 2001:db8:abcd:: at 12:34:56"
        );
        p.addresses = Redaction::None;
        p.ips = Redaction::None;
        assert_eq!(
            redact(line, &p),
            "login ok for [address] from [ip] and [ip] at 12:34:56"
        );
        p.addresses = Redaction::Hashed;
        p.ips = Redaction::Hashed;
        let out = redact(line, &p);
        assert!(!out.contains("jane.doe") && !out.contains("203.0.113.77"));
        assert!(out.contains("@example.org") && out.contains("h:"));
        // Same input, same hash; different input, different hash.
        assert_eq!(out, redact(line, &p));
        assert_ne!(reduce_ip("203.0.113.77", Redaction::Hashed), reduce_ip("203.0.113.78", Redaction::Hashed));
        // A version string is not an address.
        p.ips = Redaction::None;
        assert_eq!(redact("build 999.1.2.3 ok", &p), "build 999.1.2.3 ok");
    }

    #[test]
    fn tokens_are_cut_when_enabled() {
        let p = Policy::default();
        let out = redact("authorization: Bearer abcdefghijklmnop1234567890", &p);
        assert!(!out.contains("abcdefghijklmnop1234567890"), "{out}");
        let off = Policy {
            tokens: false,
            ..Policy::default()
        };
        assert!(redact("authorization: Bearer abcdefghijklmnop1234567890", &off).contains("abcdef"));
    }

    #[test]
    fn directives_and_directory_validation() {
        assert_eq!(directives("warn", &[]), "warn");
        assert_eq!(
            directives("info", &["mw_imap=debug".into(), "bad".into(), "x=loud".into(), "a b=warn".into()]),
            "info,mw_imap=debug"
        );
        assert_eq!(directives("shout", &[]), "info");
        assert!(directory_is_valid("logs") && directory_is_valid("/var/log/mailwoman"));
        assert!(!directory_is_valid("../etc") && !directory_is_valid("a/../../b") && !directory_is_valid(" "));
    }
}
