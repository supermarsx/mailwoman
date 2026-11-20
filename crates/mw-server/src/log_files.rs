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
    sync::{
        Arc, Mutex, OnceLock, RwLock, Weak,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        mpsc::{self, SyncSender, TrySendError},
    },
    time::{Duration, Instant},
};

// Each append is independently authenticated; rotation may rename/compress the
// ciphertext without decrypting it. stdout remains the platform's responsibility.
const SEALED_LOG: &[u8] = b"MWLOG1:";
fn key_cell()->&'static RwLock<Option<mw_store::ServerKey>> {
    static K:OnceLock<RwLock<Option<mw_store::ServerKey>>>=OnceLock::new();
    K.get_or_init(||RwLock::new(None))
}
pub fn install_file_key(key: mw_store::ServerKey) {
    let _configuration = configuration_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    *key_cell().write().unwrap_or_else(|e| e.into_inner()) = Some(key.clone());
    if let Some(worker) = OUTPUT.get() {
        let mut state = worker.state.lock().unwrap_or_else(|e| e.into_inner());
        state.key = Some(key);
        state.key_epoch = state.key_epoch.wrapping_add(1);
    }
}
fn file_key()->Option<mw_store::ServerKey>{key_cell().read().unwrap_or_else(|e|e.into_inner()).clone()}
fn crypto_error(error:impl std::fmt::Display)->io::Error{io::Error::new(io::ErrorKind::InvalidData,error.to_string())}
fn encrypt_log(bytes:&[u8],key:&mw_store::ServerKey)->io::Result<Vec<u8>>{
    use base64::Engine as _;
    let sealed=key.seal_bound(bytes,"application-log-record-v1",&[]).map_err(crypto_error)?;
    let mut out=SEALED_LOG.to_vec();out.extend_from_slice(base64::engine::general_purpose::STANDARD.encode(sealed).as_bytes());out.push(b'\n');Ok(out)
}
pub(crate) fn decode_log(bytes:&[u8])->io::Result<Vec<u8>>{
    if !bytes.starts_with(SEALED_LOG){return Ok(bytes.to_vec());}
    let key=file_key().ok_or_else(||crypto_error("log encryption key unavailable"))?;
    decode_log_with_key(bytes,&key)
}
fn decode_log_with_key(bytes:&[u8],key:&mw_store::ServerKey)->io::Result<Vec<u8>>{
    use base64::Engine as _;
    let mut out=Vec::new();
    for record in bytes.split(|b|*b==b'\n').filter(|r|!r.is_empty()){
        let encoded=record.strip_prefix(SEALED_LOG).ok_or_else(||crypto_error("invalid encrypted log record"))?;
        let sealed=base64::engine::general_purpose::STANDARD.decode(encoded).map_err(crypto_error)?;
        out.extend_from_slice(&key.open_bound(&sealed,"application-log-record-v1",&[]).map_err(crypto_error)?);
    }
    Ok(out)
}
pub(crate) fn encode_log(bytes:&[u8])->io::Result<Vec<u8>>{
    match file_key(){Some(key)=>encrypt_log(bytes,&key),None=>Ok(bytes.to_vec())}
}
fn migrate_log(path:&Path,key:&mw_store::ServerKey)->io::Result<()> {
    use std::io::Read;
    if !fs::symlink_metadata(path)?.file_type().is_file(){return Err(crypto_error("log migration requires a regular file"));}
    let raw=fs::read(path)?;let gzip=path.extension().is_some_and(|ext|ext=="gz");
    let plain=if gzip{let mut bytes=Vec::new();flate2::read::MultiGzDecoder::new(&raw[..]).read_to_end(&mut bytes)?;bytes}else{raw};
    if plain.is_empty(){return Ok(());}
    if plain.starts_with(SEALED_LOG){decode_log_with_key(&plain,key)?;return Ok(());}
    let sealed=encrypt_log(&plain,key)?;
    let tmp=path.with_extension(format!("encrypt-{}.tmp",uuid::Uuid::new_v4()));
    let mut file=OpenOptions::new().create_new(true).write(true).open(&tmp)?;
    if gzip{let mut enc=flate2::write::GzEncoder::new(&mut file,flate2::Compression::default());enc.write_all(&sealed)?;enc.finish()?;}else{file.write_all(&sealed)?;}
    file.sync_all()?;drop(file);fs::rename(tmp,path)
}

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

/// Migrate existing app-owned log files under the writer lock. Old files stay
/// in place if encryption fails; no plaintext migration file is created.
pub(crate) fn encrypt_legacy_logs() -> io::Result<usize> {
    let _maintenance = maintenance_lock().try_lock().map_err(|_| busy_error())?;
    let (reply, receive) = mpsc::sync_channel(1);
    submit_control(|snapshot| OutputCommand::Migrate { snapshot, reply })?;
    receive
        .recv_timeout(CONTROL_TIMEOUT)
        .map_err(|_| control_timeout())?
}
fn encrypt_legacy_logs_direct(policy: &Policy, key: &mw_store::ServerKey) -> io::Result<usize> {
    let mut sink = sink_cell().lock().unwrap_or_else(|e| e.into_inner());
    *sink = None;
    let mut count = 0;
    if !policy.directory.exists() {
        return Ok(0);
    }
    for entry in fs::read_dir(&policy.directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == CURRENT
            || (name.starts_with("mailwoman-")
                && (name.ends_with(".log") || name.ends_with(".log.gz")))
        {
            migrate_log(&entry.path(), &key)?;
            count += 1;
        }
    }
    Ok(count)
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
    !dir.trim().is_empty() && p.components().all(|c| !matches!(c, Component::ParentDir))
}

/// The level directives for `level` and the component entries, or `None` for the
/// defaults (so the start-up directives stay in force). Entries that do not parse
/// as `component=level` are dropped.
pub fn directives(level: &str, components: &[String]) -> String {
    let valid = |l: &str| matches!(l, "error" | "warn" | "info" | "debug" | "trace");
    let mut out = vec![if valid(level) {
        level.to_string()
    } else {
        "info".into()
    }];
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
    key: Option<mw_store::ServerKey>,
    key_epoch: u64,
}

impl FileSink {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            file: None,
            size: 0,
            opened_at: 0,
            key: None,
            key_epoch: 0,
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
        if let Some(key)=&self.key {if path.exists(){migrate_log(&path,key)?;}}
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

    /// Close the current file and move it aside. Production calls run on the
    /// one output worker, including compression and retention cleanup.
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
        let work = move || -> io::Result<()> {
            if policy.compress {
                gzip_file(&target)?;
            }
            enforce_limits(&dir, &policy, now, 0);
            Ok(())
        };
        work()?;
        Ok(())
    }

    /// Append `bytes`, rotating first when the size or age limit is reached.
    pub fn write(&mut self, bytes: &[u8], now: i64, p: &Policy) -> io::Result<()> {
        let encrypted;
        let bytes=if let Some(key)=&self.key{encrypted=encrypt_log(bytes,key)?;&encrypted[..]}else{bytes};
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

/// Drain earlier file events and pause file writes for an external rewrite.
/// Keep the returned guard alive only around synchronous filesystem work.
pub fn release_current() -> io::Result<FilePauseGuard> {
    let maintenance = maintenance_lock().try_lock().map_err(|_| busy_error())?;
    let lease = Arc::new(());
    let (reply, receive) = mpsc::sync_channel(1);
    submit_control(|_| OutputCommand::Pause {
        lease: lease.clone(),
        reply,
    })?;
    receive
        .recv_timeout(CONTROL_TIMEOUT)
        .map_err(|_| control_timeout())??;
    Ok(FilePauseGuard {
        _lease: lease,
        _maintenance: maintenance,
    })
}

pub struct FilePauseGuard {
    _lease: Arc<()>,
    _maintenance: std::sync::MutexGuard<'static, ()>,
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
    let _configuration = configuration_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut cur = policy_cell().write().unwrap_or_else(|e| e.into_inner());
    *cur = p.clone();
    if let Some(worker) = OUTPUT.get() {
        worker
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .policy = Arc::new(p);
    }
}

/// Apply the configured age/count/size limits immediately, with optional rotation.
/// No caller-supplied path and no bypass of the configured retention policy.
pub fn maintain_now(rotate: bool) -> io::Result<serde_json::Value> {
    let _maintenance = maintenance_lock().try_lock().map_err(|_| busy_error())?;
    let (reply, receive) = mpsc::sync_channel(1);
    submit_control(|snapshot| OutputCommand::Maintain {
        rotate,
        snapshot,
        reply,
    })?;
    receive
        .recv_timeout(CONTROL_TIMEOUT)
        .map_err(|_| control_timeout())?
}
fn maintain_direct(rotate: bool, snapshot: &OutputSnapshot) -> io::Result<serde_json::Value> {
    let p = &snapshot.policy;
    let now = chrono::Utc::now().timestamp();
    let removed = {
        let mut cell = sink_cell().lock().unwrap_or_else(|e| e.into_inner());
        if rotate && p.file_enabled {
            let key = snapshot
                .key
                .clone()
                .ok_or_else(|| crypto_error("server key unavailable for log maintenance"))?;
            let sink = prepare_sink(&mut cell, snapshot, key);
            sink.rotate(now, &p)?;
        }
        let current = fs::metadata(p.directory.join(CURRENT))
            .map(|m| m.len())
            .unwrap_or(0);
        enforce_limits(&p.directory, &p, now, current).len()
    };
    Ok(
        serde_json::json!({"removedFiles":removed,"rotationRequested":rotate&&p.file_enabled,"compressionMayBeRunning":false,"status":status()}),
    )
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
        "encryptedWrites":file_key().is_some(),
        "directory": p.directory.to_string_lossy(),
        "currentBytes": live,
        "rotatedFiles": rotated.len(),
        "rotatedBytes": rotated.iter().map(|f| f.bytes).sum::<u64>(),
        "oldestStamp": rotated.first().map(|f| f.stamp),
        "newestStamp": rotated.last().map(|f| f.stamp),
        "outputWorker":output_status(),
    })
}

const MAX_EVENT_BYTES: usize = 16 * 1024;
const MAX_REDACTED_BYTES: usize = 128 * 1024;
const OUTPUT_QUEUE_CAPACITY: usize = 1024;
const CONTROL_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Default)]
struct OutputCounters {
    accepted: AtomicU64,
    processed: AtomicU64,
    queue_full: AtomicU64,
    oversized: AtomicU64,
    unavailable: AtomicU64,
    contended: AtomicU64,
    stdout_errors: AtomicU64,
    file_errors: AtomicU64,
    missing_key: AtomicU64,
    paused_files: AtomicU64,
    redacted_oversized: AtomicU64,
    worker_panics: AtomicU64,
    shutdown_timeouts: AtomicU64,
    queued: AtomicUsize,
    processing: AtomicBool,
}
fn counters() -> &'static Arc<OutputCounters> {
    static C: OnceLock<Arc<OutputCounters>> = OnceLock::new();
    C.get_or_init(|| Arc::new(OutputCounters::default()))
}
fn configuration_lock() -> &'static Mutex<()> {
    static L: Mutex<()> = Mutex::new(());
    &L
}
fn maintenance_lock() -> &'static Mutex<()> {
    static L: Mutex<()> = Mutex::new(());
    &L
}
fn busy_error() -> io::Error {
    io::Error::new(io::ErrorKind::WouldBlock, "log output maintenance is busy")
}
fn control_timeout() -> io::Error {
    io::Error::new(
        io::ErrorKind::TimedOut,
        "log output did not acknowledge within the bounded wait",
    )
}

#[derive(Clone)]
struct OutputSnapshot {
    policy: Arc<Policy>,
    key: Option<mw_store::ServerKey>,
    key_epoch: u64,
}
enum OutputCommand {
    Record {
        bytes: Vec<u8>,
        snapshot: OutputSnapshot,
    },
    Pause {
        lease: Arc<()>,
        reply: SyncSender<io::Result<()>>,
    },
    Maintain {
        rotate: bool,
        snapshot: OutputSnapshot,
        reply: SyncSender<io::Result<serde_json::Value>>,
    },
    Migrate {
        snapshot: OutputSnapshot,
        reply: SyncSender<io::Result<usize>>,
    },
}
struct OutputState {
    sender: Option<SyncSender<OutputCommand>>,
    policy: Arc<Policy>,
    key: Option<mw_store::ServerKey>,
    key_epoch: u64,
}
impl OutputState {
    fn snapshot(&self) -> OutputSnapshot {
        OutputSnapshot {
            policy: self.policy.clone(),
            key: self.key.clone(),
            key_epoch: self.key_epoch,
        }
    }
}
struct OutputWorker {
    state: Mutex<OutputState>,
    done: Arc<AtomicBool>,
    join: Mutex<Option<std::thread::JoinHandle<()>>>,
}
static OUTPUT: OnceLock<OutputWorker> = OnceLock::new();

/// Start before installing the tracing subscriber. Failure is returned to main;
/// events never fall back to an unredacted synchronous destination.
pub fn start_output_worker() -> io::Result<()> {
    let _configuration = configuration_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(worker) = OUTPUT.get() {
        return if !worker.done.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(io::Error::other("log output worker has stopped"))
        };
    }
    let (sender, receiver) = mpsc::sync_channel(OUTPUT_QUEUE_CAPACITY);
    let done = Arc::new(AtomicBool::new(false));
    let completed = done.clone();
    let stats = counters().clone();
    let join = std::thread::Builder::new()
        .name("mw-log-output".into())
        .spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                output_loop(receiver, &stats)
            }));
            if result.is_err() {
                stats.worker_panics.fetch_add(1, Ordering::Relaxed);
            }
            completed.store(true, Ordering::Release);
        })?;
    let worker = OutputWorker {
        state: Mutex::new(OutputState {
            sender: Some(sender),
            policy: Arc::new(policy()),
            key: file_key(),
            key_epoch: 0,
        }),
        done,
        join: Mutex::new(Some(join)),
    };
    OUTPUT
        .set(worker)
        .map_err(|_| io::Error::other("log output worker already installed"))
}

fn submit_control(make: impl FnOnce(OutputSnapshot) -> OutputCommand) -> io::Result<()> {
    let worker = OUTPUT.get().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotConnected,
            "log output worker is not running",
        )
    })?;
    let state = worker.state.lock().unwrap_or_else(|e| e.into_inner());
    let sender = state.sender.as_ref().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotConnected, "log output is shutting down")
    })?;
    sender
        .try_send(make(state.snapshot()))
        .map_err(|error| match error {
            TrySendError::Full(_) => busy_error(),
            TrySendError::Disconnected(_) => io::Error::new(
                io::ErrorKind::BrokenPipe,
                "log output worker is unavailable",
            ),
        })
}

fn prepare_sink<'a>(
    cell: &'a mut Option<FileSink>,
    snapshot: &OutputSnapshot,
    key: mw_store::ServerKey,
) -> &'a mut FileSink {
    if cell.as_ref().is_some_and(|sink| {
        sink.dir != snapshot.policy.directory || sink.key_epoch != snapshot.key_epoch
    }) {
        *cell = None;
    }
    cell.get_or_insert_with(|| {
        let mut sink = FileSink::new(snapshot.policy.directory.clone());
        sink.key = Some(key);
        sink.key_epoch = snapshot.key_epoch;
        sink
    })
}
fn flush_output() -> io::Result<()> {
    io::stdout().flush()?;
    if let Some(sink) = sink_cell()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_mut()
        && let Some(file) = sink.file.as_mut()
    {
        file.flush()?;
    }
    Ok(())
}
fn output_loop(receiver: mpsc::Receiver<OutputCommand>, stats: &OutputCounters) {
    let mut pause: Weak<()> = Weak::new();
    while let Ok(command) = receiver.recv() {
        match command {
            OutputCommand::Record { bytes, snapshot } => {
                stats.queued.fetch_sub(1, Ordering::Relaxed);
                stats.processing.store(true, Ordering::Release);
                output_record(&bytes, &snapshot, pause.upgrade().is_some(), stats);
                stats.processed.fetch_add(1, Ordering::Relaxed);
                stats.processing.store(false, Ordering::Release);
            }
            OutputCommand::Pause { lease, reply } => {
                let result = flush_output().map(|()| {
                    *sink_cell().lock().unwrap_or_else(|e| e.into_inner()) = None;
                    pause = Arc::downgrade(&lease);
                });
                let _ = reply.send(result);
                // Only the caller's returned guard keeps the pause alive. A
                // timed-out/cancelled caller cannot leave a permanent pause.
            }
            OutputCommand::Maintain {
                rotate,
                snapshot,
                reply,
            } => {
                let result = if pause.upgrade().is_some() {
                    Err(busy_error())
                } else {
                    maintain_direct(rotate, &snapshot)
                };
                let _ = reply.send(result);
            }
            OutputCommand::Migrate { snapshot, reply } => {
                let result = if pause.upgrade().is_some() {
                    Err(busy_error())
                } else {
                    match snapshot.key.as_ref() {
                        Some(key) => encrypt_legacy_logs_direct(&snapshot.policy, key),
                        None => Err(crypto_error("server key unavailable")),
                    }
                };
                let _ = reply.send(result);
            }
        }
    }
    if flush_output().is_err() {
        stats.stdout_errors.fetch_add(1, Ordering::Relaxed);
    }
    *sink_cell().lock().unwrap_or_else(|e| e.into_inner()) = None;
}
fn output_record(bytes: &[u8], snapshot: &OutputSnapshot, paused: bool, stats: &OutputCounters) {
    let mut cell = sink_cell().lock().unwrap_or_else(|e| e.into_inner());
    output_record_to(bytes, snapshot, paused, stats, &mut io::stdout(), &mut cell);
}
fn output_record_to(
    bytes: &[u8],
    snapshot: &OutputSnapshot,
    paused: bool,
    stats: &OutputCounters,
    stdout: &mut dyn Write,
    cell: &mut Option<FileSink>,
) {
    let line = redact(&String::from_utf8_lossy(bytes), &snapshot.policy);
    if line.len() > MAX_REDACTED_BYTES {
        stats.redacted_oversized.fetch_add(1, Ordering::Relaxed);
        return;
    }
    if stdout.write_all(line.as_bytes()).is_err() {
        stats.stdout_errors.fetch_add(1, Ordering::Relaxed);
    }
    if snapshot.policy.file_enabled {
        if paused {
            stats.paused_files.fetch_add(1, Ordering::Relaxed);
            return;
        }
        let Some(key) = snapshot.key.clone() else {
            stats.missing_key.fetch_add(1, Ordering::Relaxed);
            return;
        };
        let plain = ansi_re().replace_all(&line, "");
        if prepare_sink(cell, snapshot, key)
            .write(
                plain.as_bytes(),
                chrono::Utc::now().timestamp(),
                &snapshot.policy,
            )
            .is_err()
        {
            stats.file_errors.fetch_add(1, Ordering::Relaxed);
            *cell = None;
        }
    } else {
        *cell = None;
    }
}

fn output_status() -> serde_json::Value {
    let stats = counters();
    let worker = OUTPUT.get();
    serde_json::json!({"started":worker.is_some(),"running":worker.is_some_and(|worker|!worker.done.load(Ordering::Acquire)),
        "queueCapacity":OUTPUT_QUEUE_CAPACITY,"maxEventBytes":MAX_EVENT_BYTES,"maxQueuedRawPayloadBytes":OUTPUT_QUEUE_CAPACITY*MAX_EVENT_BYTES,"queuedEvents":stats.queued.load(Ordering::Relaxed),"processingEvent":stats.processing.load(Ordering::Relaxed),
        "acceptedEvents":stats.accepted.load(Ordering::Relaxed),"processedEvents":stats.processed.load(Ordering::Relaxed),
        "queueFullDrops":stats.queue_full.load(Ordering::Relaxed),"oversizedEventDrops":stats.oversized.load(Ordering::Relaxed),
        "workerUnavailableDrops":stats.unavailable.load(Ordering::Relaxed),"submissionContentionDrops":stats.contended.load(Ordering::Relaxed),
        "stdoutWriteErrors":stats.stdout_errors.load(Ordering::Relaxed),"fileWriteErrors":stats.file_errors.load(Ordering::Relaxed),
        "missingFileKeyEvents":stats.missing_key.load(Ordering::Relaxed),"pausedFileEvents":stats.paused_files.load(Ordering::Relaxed),
        "oversizedRedactedEventDrops":stats.redacted_oversized.load(Ordering::Relaxed),"workerPanics":stats.worker_panics.load(Ordering::Relaxed),"shutdownTimeouts":stats.shutdown_timeouts.load(Ordering::Relaxed),
        "processedMeaning":"completed output attempts, not guaranteed delivery; errors and suppressed file events are counted separately"})
}

/// Queue depth plus the current output attempt, without filesystem work.
pub fn pending_output_events() -> usize {
    counters()
        .queued
        .load(Ordering::Relaxed)
        .saturating_add(usize::from(counters().processing.load(Ordering::Relaxed)))
}

/// Stop admission and drain in FIFO order. Waiting is asynchronous and bounded;
/// a worker blocked by platform stdout is detached, never joined indefinitely.
pub async fn shutdown_output_worker() -> bool {
    let Some(worker) = OUTPUT.get() else {
        return true;
    };
    stop_output_worker(worker, counters(), CONTROL_TIMEOUT).await
}
async fn stop_output_worker(
    worker: &OutputWorker,
    stats: &OutputCounters,
    timeout: Duration,
) -> bool {
    worker
        .state
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .sender
        .take();
    let deadline = Instant::now() + timeout;
    loop {
        let finished = worker
            .join
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .map_or_else(
                || worker.done.load(Ordering::Acquire),
                std::thread::JoinHandle::is_finished,
            );
        if finished {
            if let Some(join) = worker.join.lock().unwrap_or_else(|e| e.into_inner()).take() {
                if join.join().is_err() {
                    return false;
                }
            }
            return stats.queued.load(Ordering::Relaxed) == 0
                && !stats.processing.load(Ordering::Relaxed)
                && stats.worker_panics.load(Ordering::Relaxed) == 0;
        }
        if Instant::now() >= deadline {
            stats.shutdown_timeouts.fetch_add(1, Ordering::Relaxed);
            worker.join.lock().unwrap_or_else(|e| e.into_inner()).take();
            return false;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

fn enqueue_record(worker: &OutputWorker, stats: &OutputCounters, bytes: Vec<u8>) {
    let Ok(state) = worker.state.try_lock() else {
        stats.contended.fetch_add(1, Ordering::Relaxed);
        return;
    };
    let Some(sender) = state.sender.as_ref() else {
        stats.unavailable.fetch_add(1, Ordering::Relaxed);
        return;
    };
    let command = OutputCommand::Record {
        bytes,
        snapshot: state.snapshot(),
    };
    stats.queued.fetch_add(1, Ordering::Relaxed);
    match sender.try_send(command) {
        Ok(()) => {
            stats.accepted.fetch_add(1, Ordering::Relaxed);
        }
        Err(error) => {
            stats.queued.fetch_sub(1, Ordering::Relaxed);
            match error {
                TrySendError::Full(_) => {
                    stats.queue_full.fetch_add(1, Ordering::Relaxed);
                }
                TrySendError::Disconnected(_) => {
                    stats.unavailable.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }
}

/// A tracing formatter writes only into a bounded event buffer on the caller.
/// Redaction, encryption, compression and destination I/O belong to the worker.
#[derive(Clone, Copy, Default)]
pub struct LogWriter;

pub struct EventBuffer {
    bytes: Vec<u8>,
    oversized: bool,
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for LogWriter {
    type Writer = EventBuffer;
    fn make_writer(&'a self) -> EventBuffer {
        EventBuffer {
            bytes: Vec::with_capacity(256),
            oversized: false,
        }
    }
}

impl Write for EventBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if !self.oversized {
            if self
                .bytes
                .len()
                .checked_add(buf.len())
                .is_none_or(|size| size > MAX_EVENT_BYTES)
            {
                self.oversized = true;
                self.bytes.clear();
            } else {
                self.bytes.extend_from_slice(buf);
            }
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for EventBuffer {
    fn drop(&mut self) {
        let stats = counters();
        if self.oversized {
            stats.oversized.fetch_add(1, Ordering::Relaxed);
            return;
        }
        if self.bytes.is_empty() {
            return;
        }
        let Some(worker) = OUTPUT.get() else {
            stats.unavailable.fetch_add(1, Ordering::Relaxed);
            return;
        };
        enqueue_record(worker, stats, std::mem::take(&mut self.bytes));
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

    fn output_fixture(
        capacity: usize,
        snapshot: OutputSnapshot,
    ) -> (OutputWorker, mpsc::Receiver<OutputCommand>) {
        let (sender, receiver) = mpsc::sync_channel(capacity);
        (
            OutputWorker {
                state: Mutex::new(OutputState {
                    sender: Some(sender),
                    policy: snapshot.policy,
                    key: snapshot.key,
                    key_epoch: snapshot.key_epoch,
                }),
                done: Arc::new(AtomicBool::new(false)),
                join: Mutex::new(None),
            },
            receiver,
        )
    }
    fn output_snapshot(directory: PathBuf, key: mw_store::ServerKey) -> OutputSnapshot {
        OutputSnapshot {
            policy: Arc::new(Policy {
                file_enabled: true,
                directory,
                addresses: Redaction::None,
                ips: Redaction::None,
                compress: false,
                ..Default::default()
            }),
            key: Some(key),
            key_epoch: 1,
        }
    }
    fn queue_control(worker: &OutputWorker, command: OutputCommand) {
        worker
            .state
            .lock()
            .unwrap()
            .sender
            .as_ref()
            .unwrap()
            .try_send(command)
            .unwrap_or_else(|_| panic!("control fixture queue full"));
    }

    #[test]
    fn output_event_cap_discards_whole_event_instead_of_sensitive_truncation() {
        let before = counters().oversized.load(Ordering::Relaxed);
        let mut event = EventBuffer {
            bytes: Vec::new(),
            oversized: false,
        };
        event.write_all(&vec![b'x'; MAX_EVENT_BYTES]).unwrap();
        event
            .write_all(b" authorization: Bearer do-not-keep-a-partial-secret")
            .unwrap();
        assert!(event.oversized);
        assert!(event.bytes.is_empty());
        event.write_all(b"must still be discarded").unwrap();
        assert!(event.bytes.is_empty());
        drop(event);
        assert_eq!(counters().oversized.load(Ordering::Relaxed), before + 1);
    }

    #[test]
    fn output_queue_is_bounded_nonblocking_and_fail_closed_when_full_or_down() {
        let stats = OutputCounters::default();
        let (worker, receiver) = output_fixture(
            1,
            output_snapshot(PathBuf::from("unused"), mw_store::ServerKey::generate()),
        );
        enqueue_record(&worker, &stats, b"first".to_vec());
        enqueue_record(
            &worker,
            &stats,
            b"full must not fall back to stdout".to_vec(),
        );
        assert_eq!(stats.accepted.load(Ordering::Relaxed), 1);
        assert_eq!(stats.queue_full.load(Ordering::Relaxed), 1);
        assert_eq!(stats.queued.load(Ordering::Relaxed), 1);
        {
            let _held = worker.state.lock().unwrap();
            enqueue_record(&worker, &stats, b"contended".to_vec());
        }
        assert_eq!(stats.contended.load(Ordering::Relaxed), 1);
        drop(receiver);
        enqueue_record(&worker, &stats, b"disconnected".to_vec());
        assert_eq!(stats.unavailable.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn output_queue_binds_redaction_policy_and_encryption_key_at_submission() {
        let a = tmp("output-a");
        let b = tmp("output-b");
        let key_a = mw_store::ServerKey::generate();
        let key_b = mw_store::ServerKey::generate();
        let snapshot = output_snapshot(a.clone(), key_a.clone());
        let (worker, receiver) = output_fixture(2, snapshot);
        let stats = OutputCounters::default();
        let raw = b"owner@example.org authorization: Bearer abcdefghijklmnop1234567890\n";
        enqueue_record(&worker, &stats, raw.to_vec());
        {
            let mut state = worker.state.lock().unwrap();
            let mut next = (*state.policy).clone();
            next.addresses = Redaction::Full;
            next.directory = b.clone();
            state.policy = Arc::new(next);
            state.key = Some(key_b.clone());
            state.key_epoch = 2;
        }
        enqueue_record(&worker, &stats, raw.to_vec());
        let mut sink = None;
        let mut stdout = Vec::new();
        for _ in 0..2 {
            let OutputCommand::Record { bytes, snapshot } = receiver.recv().unwrap() else {
                panic!("record expected")
            };
            output_record_to(&bytes, &snapshot, false, &stats, &mut stdout, &mut sink);
        }
        drop(sink);
        let bytes_a = fs::read(a.join(CURRENT)).unwrap();
        let bytes_b = fs::read(b.join(CURRENT)).unwrap();
        let plain_a = String::from_utf8(decode_log_with_key(&bytes_a, &key_a).unwrap()).unwrap();
        let plain_b = String::from_utf8(decode_log_with_key(&bytes_b, &key_b).unwrap()).unwrap();
        assert!(plain_a.contains("[address]"));
        assert!(!plain_a.contains("owner@example.org"));
        assert!(plain_b.contains("owner@example.org"));
        for text in [&plain_a, &plain_b] {
            assert!(!text.contains("abcdefghijklmnop1234567890"));
        }
        assert!(decode_log_with_key(&bytes_a, &key_b).is_err());
        assert!(decode_log_with_key(&bytes_b, &key_a).is_err());
        fs::remove_dir_all(a).unwrap();
        fs::remove_dir_all(b).unwrap();
    }

    #[test]
    fn output_key_epoch_cannot_append_mixed_keys_and_write_errors_are_counted() {
        let dir = tmp("output-key-epoch");
        let key = mw_store::ServerKey::generate();
        let first = output_snapshot(dir.clone(), key.clone());
        let mut cell = None;
        let mut stdout = Vec::new();
        let stats = OutputCounters::default();
        output_record_to(
            b"first record\n",
            &first,
            false,
            &stats,
            &mut stdout,
            &mut cell,
        );
        let before = fs::read(dir.join(CURRENT)).unwrap();
        let mut second = output_snapshot(dir.clone(), mw_store::ServerKey::generate());
        second.key_epoch = 2;
        output_record_to(
            b"must not append under a different key\n",
            &second,
            false,
            &stats,
            &mut stdout,
            &mut cell,
        );
        assert_eq!(stats.file_errors.load(Ordering::Relaxed), 1);
        assert_eq!(fs::read(dir.join(CURRENT)).unwrap(), before);
        assert_eq!(
            decode_log_with_key(&before, &key).unwrap(),
            b"first record\n"
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn output_pause_fences_files_and_expired_pause_does_not_stick() {
        let dir = tmp("output-pause");
        let key = mw_store::ServerKey::generate();
        let snapshot = output_snapshot(dir.clone(), key.clone());
        let (worker, receiver) = output_fixture(8, snapshot.clone());
        let stats = Arc::new(OutputCounters::default());
        let thread_stats = stats.clone();
        let done = worker.done.clone();
        *worker.join.lock().unwrap() = Some(std::thread::spawn(move || {
            output_loop(receiver, &thread_stats);
            done.store(true, Ordering::Release);
        }));
        enqueue_record(&worker, &stats, b"before pause\n".to_vec());
        let lease = Arc::new(());
        let (reply, receive) = mpsc::sync_channel(1);
        queue_control(
            &worker,
            OutputCommand::Pause {
                lease: lease.clone(),
                reply,
            },
        );
        receive
            .recv_timeout(Duration::from_secs(1))
            .unwrap()
            .unwrap();
        let before = fs::read(dir.join(CURRENT)).unwrap();
        enqueue_record(&worker, &stats, b"during pause\n".to_vec());
        let (reply, receive) = mpsc::sync_channel(1);
        queue_control(
            &worker,
            OutputCommand::Maintain {
                rotate: false,
                snapshot: snapshot.clone(),
                reply,
            },
        );
        assert!(
            receive
                .recv_timeout(Duration::from_secs(1))
                .unwrap()
                .is_err()
        );
        assert_eq!(fs::read(dir.join(CURRENT)).unwrap(), before);
        assert_eq!(stats.paused_files.load(Ordering::Relaxed), 1);
        drop(lease);
        enqueue_record(&worker, &stats, b"after pause\n".to_vec());
        assert!(stop_output_worker(&worker, &stats, Duration::from_secs(1)).await);
        let plain = decode_log_with_key(&fs::read(dir.join(CURRENT)).unwrap(), &key).unwrap();
        assert_eq!(plain, b"before pause\nafter pause\n");
        fs::remove_dir_all(dir).unwrap();

        let dir = tmp("output-expired-pause");
        let snapshot = output_snapshot(dir.clone(), key.clone());
        let (worker, receiver) = output_fixture(4, snapshot);
        let stats = Arc::new(OutputCounters::default());
        let lease = Arc::new(());
        let (reply, receive) = mpsc::sync_channel(1);
        queue_control(
            &worker,
            OutputCommand::Pause {
                lease: lease.clone(),
                reply,
            },
        );
        drop(receive);
        drop(lease);
        enqueue_record(
            &worker,
            &stats,
            b"cancelled pause must not suppress this\n".to_vec(),
        );
        let thread_stats = stats.clone();
        let done = worker.done.clone();
        *worker.join.lock().unwrap() = Some(std::thread::spawn(move || {
            output_loop(receiver, &thread_stats);
            done.store(true, Ordering::Release);
        }));
        assert!(stop_output_worker(&worker, &stats, Duration::from_secs(1)).await);
        assert_eq!(stats.paused_files.load(Ordering::Relaxed), 0);
        assert!(fs::read(dir.join(CURRENT)).unwrap().starts_with(SEALED_LOG));
        fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn output_shutdown_is_async_bounded_and_does_not_join_a_blocked_sink() {
        let snapshot = output_snapshot(PathBuf::from("unused"), mw_store::ServerKey::generate());
        let (worker, receiver) = output_fixture(1, snapshot);
        let stats = OutputCounters::default();
        let (release, blocked) = mpsc::sync_channel(1);
        let done = worker.done.clone();
        *worker.join.lock().unwrap() = Some(std::thread::spawn(move || {
            let _ = blocked.recv();
            drop(receiver);
            done.store(true, Ordering::Release);
        }));
        let heartbeat = Arc::new(AtomicBool::new(false));
        let observed = heartbeat.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(2)).await;
            observed.store(true, Ordering::Release);
        });
        assert!(!stop_output_worker(&worker, &stats, Duration::from_millis(25)).await);
        assert!(
            heartbeat.load(Ordering::Acquire),
            "shutdown must let the runtime keep polling"
        );
        assert!(
            !stop_output_worker(&worker, &stats, Duration::from_millis(1)).await,
            "a detached live worker is not a successful drain"
        );
        assert_eq!(stats.shutdown_timeouts.load(Ordering::Relaxed), 2);
        release.send(()).unwrap();
        assert!(stop_output_worker(&worker, &stats, Duration::from_secs(1)).await);
    }

    #[test]
    fn encrypted_log_records_migrate_rotate_and_authenticate(){
        let dir=tmp("encrypted");let key=mw_store::ServerKey::generate();let policy=small();
        fs::write(dir.join(CURRENT),"legacy secret@example.org\n").unwrap();
        let mut sink=FileSink::new(dir.clone());sink.key=Some(key.clone());sink.write(b"new secret@example.org\n",1000,&policy).unwrap();drop(sink);
        let raw=fs::read(dir.join(CURRENT)).unwrap();assert!(raw.starts_with(SEALED_LOG));assert!(!String::from_utf8_lossy(&raw).contains("secret@example.org"));
        let plain=decode_log_with_key(&raw,&key).unwrap();assert!(String::from_utf8_lossy(&plain).contains("secret@example.org"));
        assert!(decode_log_with_key(&raw,&mw_store::ServerKey::generate()).is_err());
        let mut bad=raw.clone();bad[12]^=1;assert!(decode_log_with_key(&bad,&key).is_err());
        assert!(dir.starts_with(std::env::temp_dir()));fs::remove_dir_all(dir).unwrap();
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
