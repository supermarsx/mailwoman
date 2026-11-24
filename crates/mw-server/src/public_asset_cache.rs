//! Public static representations only: no mail, API results, cookies or dynamic
//! analytics/OIDC documents. Cache budgets cover retained owned payload bytes,
//! not response lifetimes or total process RSS. All disk work stays off Tokio.
use axum::{
    body::{Body, Bytes},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, watch};
const WORKERS: usize = 2;
/// Unique misses that may hold a dispatcher task at once. A miss beyond this
/// waits for a slot (within the observer deadline) instead of failing.
const QUEUED: usize = 16;
/// One deadline covers admission and the load itself; 503 only when it passes.
const OBSERVE: Duration = Duration::from_secs(5);
const PREFIX: &str = "web.assets.cache.";
const MAX_DISCOVERY_BYTES: usize = 256 * 1024;
/// Bounds for the build enumeration used by the "app" preload.
const MAX_WALK_ENTRIES: usize = 50_000;
const MAX_WALK_DEPTH: usize = 8;
/// The `en` app translation pack (`assets/catalog.en.app-<hash>.json`). The
/// server has no default-locale setting, so `en`, the build's source locale,
/// is the one preloaded; other locales and the admin packs fill on first use.
const SOURCE_LOCALE_PACK: &str = "assets/catalog.en.app-";
fn is_source_locale_pack(path: &str) -> bool {
    path.starts_with(SOURCE_LOCALE_PACK)
        && path.ends_with(".json")
        && !path[SOURCE_LOCALE_PACK.len()..].contains('/')
        && crate::is_content_hashed(path)
}
// Discovery is optional. Never scan an unbounded deployed document on Tokio.
fn core_paths(shell: &[u8], base: &str) -> Vec<String> {
    let mut paths = vec!["sw.js".to_string(), "manifest.webmanifest".to_string()];
    if shell.len() > MAX_DISCOVERY_BYTES {
        return paths;
    }
    if let Ok(html) = std::str::from_utf8(shell) {
        for part in html.split(['"', '\'']) {
            let candidate = part
                .trim_start_matches(base)
                .trim_start_matches('/')
                .trim_start_matches("./");
            if candidate.starts_with("assets/")
                && (candidate.ends_with(".js") || candidate.ends_with(".css"))
                && candidate.len() <= 4096
                && !paths.iter().any(|p| p == candidate)
            {
                paths.push(candidate.into());
                if paths.len() >= 18 {
                    break;
                }
            }
        }
    }
    paths
}
/// Relative file paths of a deployed web directory. Symbolic links and other
/// reparse points (Windows junctions) are not followed or listed, names that
/// are not UTF-8 or that the request path guard would refuse are skipped, and
/// depth and entry count are bounded. Runs on a blocking thread.
fn walk(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = 0usize;
    let mut stack = vec![(root.to_path_buf(), String::new(), 0usize)];
    while let Some((dir, prefix, depth)) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries {
            seen += 1;
            if seen > MAX_WALK_ENTRIES {
                out.sort();
                return out;
            }
            let Ok(entry) = entry else { continue };
            let Ok(name) = entry.file_name().into_string() else {
                continue;
            };
            let relative = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            if relative.len() > 4096 || crate::error_pages::is_unsafe_asset_path(&relative) {
                continue;
            }
            let Ok(meta) = std::fs::symlink_metadata(entry.path()) else {
                continue;
            };
            if is_link(&meta) {
                continue;
            }
            if meta.is_dir() {
                if depth + 1 < MAX_WALK_DEPTH {
                    stack.push((entry.path(), relative, depth + 1));
                }
            } else if meta.is_file() {
                out.push(relative);
            }
        }
    }
    out.sort();
    out
}
fn is_link(meta: &std::fs::Metadata) -> bool {
    if meta.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return true;
        }
    }
    false
}
fn extension(path: &str) -> &str {
    path.rsplit('/')
        .next()
        .and_then(|name| name.rsplit_once('.'))
        .map_or("", |(_, ext)| ext)
}
/// The "app" preload set in load order: shell, worker, manifest, the files the
/// shell names, then every content-hashed script, stylesheet and WebAssembly
/// module, then the `en` app translation pack. Names the request path guard
/// would refuse are never planned.
fn app_paths(available: &[String], shell: Option<&[u8]>, base: &str) -> Vec<String> {
    let present: HashSet<&str> = available.iter().map(String::as_str).collect();
    let mut out: Vec<String> = Vec::new();
    let mut seen = HashSet::new();
    let mut push = |path: &str, out: &mut Vec<String>| {
        if present.contains(path)
            && !crate::error_pages::is_unsafe_asset_path(path)
            && seen.insert(path.to_string())
        {
            out.push(path.to_string());
        }
    };
    push("index.html", &mut out);
    for path in shell.map(|shell| core_paths(shell, base)).unwrap_or_else(|| {
        vec!["sw.js".to_string(), "manifest.webmanifest".to_string()]
    }) {
        push(&path, &mut out);
    }
    for path in available {
        if path.starts_with("assets/")
            && matches!(extension(path), "js" | "mjs" | "css" | "wasm")
            && crate::is_content_hashed(path)
        {
            push(path, &mut out);
        }
    }
    for path in available {
        if is_source_locale_pack(path) {
            push(path, &mut out);
        }
    }
    out
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Preload {
    Off,
    Shell,
    Core,
    App,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Config {
    pub enabled: bool,
    #[serde(rename = "budgetMiB")]
    pub budget_mib: u32,
    #[serde(rename = "maxFileMiB")]
    pub max_file_mib: u32,
    pub max_entries: usize,
    pub revalidate_seconds: u64,
    pub preload: Preload,
    pub compression_enabled: bool,
    pub brotli_enabled: bool,
    pub compression_min_bytes: usize,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            budget_mib: 128,
            max_file_mib: 8,
            max_entries: 4096,
            revalidate_seconds: 1,
            preload: Preload::App,
            compression_enabled: true,
            brotli_enabled: true,
            compression_min_bytes: 1024,
        }
    }
}
impl Config {
    fn budget(self) -> usize {
        self.budget_mib as usize * 1024 * 1024
    }
    fn file(self) -> usize {
        self.max_file_mib.min(self.budget_mib) as usize * 1024 * 1024
    }
    /// The per-file limit bounds owned retention. Embedded (borrowed) bytes cost
    /// no owned copy, so their encodings are prepared whatever their size.
    fn encodes(self, prepared: &Prepared) -> bool {
        prepared.charge == 0 || prepared.bytes.len() <= self.file()
    }
    fn from_values(values: &[crate::settings_registry::Effective]) -> Self {
        let mut out = Self::default();
        let v = |suffix: &str| {
            values
                .iter()
                .find(|v| v.key.strip_prefix(PREFIX) == Some(suffix))
                .map(|v| &v.value)
        };
        out.enabled = v("enabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(out.enabled);
        out.budget_mib = v("budgetMiB")
            .and_then(serde_json::Value::as_u64)
            .filter(|n| (8..=512).contains(n))
            .map_or(out.budget_mib, |n| n as u32);
        out.max_file_mib = v("maxFileMiB")
            .and_then(serde_json::Value::as_u64)
            .filter(|n| (1..=16).contains(n))
            .map_or(out.max_file_mib, |n| n as u32);
        out.max_entries = v("maxEntries")
            .and_then(serde_json::Value::as_u64)
            .filter(|n| (16..=4096).contains(n))
            .map_or(out.max_entries, |n| n as usize);
        out.revalidate_seconds = v("revalidateSeconds")
            .and_then(serde_json::Value::as_u64)
            .filter(|n| (1..=60).contains(n))
            .unwrap_or(out.revalidate_seconds);
        out.preload = match v("preload").and_then(serde_json::Value::as_str) {
            Some("off") => Preload::Off,
            Some("shell") => Preload::Shell,
            Some("core") => Preload::Core,
            Some("app") => Preload::App,
            _ => out.preload,
        };
        out.compression_enabled = v("compression.enabled").and_then(serde_json::Value::as_bool).unwrap_or(true);
        out.brotli_enabled = v("compression.brotli").and_then(serde_json::Value::as_bool).unwrap_or(true);
        out.compression_min_bytes = v("compression.minBytes").and_then(serde_json::Value::as_u64).filter(|n|(256..=65536).contains(n)).unwrap_or(1024) as usize;
        out
    }
}
#[derive(Clone)]
pub(crate) struct Prepared {
    pub bytes: Bytes,
    pub etag: HeaderValue,
    pub encodings: crate::public_asset_compression::Encodings,
    pub compressible: bool,
    charge: usize,
    immutable: bool,
}
impl Prepared {
    pub(crate) fn new(value: Cow<'static, [u8]>) -> Self {
        let immutable = matches!(&value, Cow::Borrowed(_));
        let charge = if matches!(value, Cow::Owned(_)) {
            value.len()
        } else {
            0
        };
        let bytes = match value {
            Cow::Borrowed(v) => Bytes::from_static(v),
            Cow::Owned(v) => Bytes::from(v),
        };
        let etag = HeaderValue::from_str(&crate::asset_etag(&bytes))
            .expect("content hash is an HTTP header");
        Self {
            bytes,
            etag,
            encodings: Default::default(),
            compressible: false,
            charge,
            immutable,
        }
    }
}
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) enum Source {
    Web,
    Pdf,
    Retained,
}
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) enum Variant {
    Raw,
    Public,
    Console,
}
#[derive(Clone, Hash, PartialEq, Eq)]
struct Key {
    source: Source,
    variant: Variant,
    path: String,
    base: Arc<str>,
    generation: u64,
}
/// Length and modification time of a deployed file, taken before it is read.
/// An expired entry whose file still has the same stamp is kept without a
/// re-read; a stamp without a modification time never matches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Stamp {
    len: u64,
    modified: SystemTime,
}
fn stamp(path: &Path) -> Option<Stamp> {
    let meta = std::fs::metadata(path).ok()?;
    Some(Stamp {
        len: meta.len(),
        modified: meta.modified().ok()?,
    })
    .filter(|_| meta.is_file())
}
struct Entry {
    value: Prepared,
    expires: Option<Instant>,
    stamp: Option<Stamp>,
    touched: u64,
}
struct Loaded {
    value: Prepared,
    stamp: Option<Stamp>,
    /// Content hash of the deployed shell's raw bytes (before prefix adaptation).
    shell_digest: Option<String>,
    reused: bool,
}
struct Flight {
    result: watch::Sender<Option<Result<Option<Prepared>, Busy>>>,
}
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Counters {
    hits: u64,
    misses: u64,
    loads: u64,
    coalesced: u64,
    evictions: u64,
    expired: u64,
    oversize: u64,
    busy: u64,
    load_failures: u64,
    stale_inserts: u64,
    clears: u64,
    preload_loaded: u64,
    preload_skipped: u64,
    generated_encodings: u64,
    validated_sidecars: u64,
    /// Expired deployed entries kept because length and mtime were unchanged.
    revalidated_unchanged: u64,
    /// Misses that waited for an admission slot instead of being refused.
    admission_waits: u64,
    /// Generation increments caused by a changed deployed shell.
    shell_changes: u64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum PreloadState {
    #[default]
    Idle,
    Running,
    Done,
    /// Stopped early: the planned set would exceed three quarters of the budget
    /// or entry limit; the rest fills on first use.
    Limited,
    /// A clear, configuration change or shell change started a new pass.
    Superseded,
}
#[derive(Clone, Copy, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreloadProgress {
    generation: u64,
    mode: Option<Preload>,
    state: PreloadState,
    planned: usize,
    ready: usize,
    absent: usize,
    skipped: usize,
    catalogs: usize,
}
struct Inner {
    config: Config,
    generation: u64,
    refresh_ticket: u64,
    entries: HashMap<Key, Entry>,
    pending: HashMap<Key, Arc<Flight>>,
    bytes: usize,
    clock: u64,
    running: usize,
    waiting: usize,
    shell_digest: Option<String>,
    progress: PreloadProgress,
    stats: Counters,
    configuration_stale: bool,
    last_config_at: Option<u64>,
}
pub(crate) struct Cache {
    inner: Mutex<Inner>,
    workers: Arc<Semaphore>,
    queue: Arc<Semaphore>,
    web_dir: Option<PathBuf>,
    base: Arc<str>,
    retained: Option<Arc<crate::static_resilience::Retained>>,
    preload: watch::Sender<u64>,
    #[cfg(test)]
    before_load: Mutex<Option<Arc<dyn Fn(&str) + Send + Sync>>>,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Busy;
impl Busy {
    pub(crate) fn response(self) -> Response {
        let mut r = (
            StatusCode::SERVICE_UNAVAILABLE,
            "Static asset loader is busy. Retry shortly.\n",
        )
            .into_response();
        let h = r.headers_mut();
        h.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        h.insert(header::RETRY_AFTER, HeaderValue::from_static("1"));
        h.insert(
            "x-mailwoman-asset-error",
            HeaderValue::from_static("assetLoadBusy"),
        );
        r
    }
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
/// Decrements the admission-wait count however the waiting future ends.
struct Waiting<'a>(&'a Cache);
impl Drop for Waiting<'_> {
    fn drop(&mut self) {
        let mut s = self.0.lock();
        s.waiting = s.waiting.saturating_sub(1);
    }
}
impl Cache {
    pub(crate) fn new(
        web_dir: Option<PathBuf>,
        base: Arc<str>,
        retained: Option<Arc<crate::static_resilience::Retained>>,
    ) -> Arc<Self> {
        let (preload, _) = watch::channel(0);
        Arc::new(Self {
            inner: Mutex::new(Inner {
                config: Config::default(),
                generation: 1,
                refresh_ticket: 0,
                entries: HashMap::new(),
                pending: HashMap::new(),
                bytes: 0,
                clock: 0,
                running: 0,
                waiting: 0,
                shell_digest: None,
                progress: PreloadProgress::default(),
                stats: Counters::default(),
                configuration_stale: false,
                last_config_at: None,
            }),
            workers: Arc::new(Semaphore::new(WORKERS)),
            queue: Arc::new(Semaphore::new(QUEUED)),
            web_dir,
            base,
            retained,
            preload,
            #[cfg(test)]
            before_load: Mutex::new(None),
        })
    }
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }
    fn begin_refresh(&self) -> u64 {
        let mut s = self.lock();
        s.refresh_ticket = s
            .refresh_ticket
            .checked_add(1)
            .expect("asset refresh ticket exhausted");
        s.refresh_ticket
    }
    // Completion order can differ from read-start order. An older periodic
    // snapshot must not replace an acknowledged newer administrative refresh,
    // nor may its later error mark that successful refresh stale.
    fn complete_refresh(&self, ticket: u64, result: Result<Config, ()>) -> bool {
        let (epoch, discarded) = {
            let mut s = self.lock();
            if ticket != s.refresh_ticket {
                return false;
            }
            let Ok(config) = result else {
                s.configuration_stale = true;
                return true;
            };
            s.configuration_stale = false;
            s.last_config_at = Some(now());
            let discarded = if s.config != config {
                s.config = config;
                s.generation = s
                    .generation
                    .checked_add(1)
                    .expect("asset cache generation exhausted");
                s.bytes = 0;
                Some(std::mem::take(&mut s.entries))
            } else {
                None
            };
            (s.generation, discarded)
        };
        drop(discarded);
        self.preload.send_replace(epoch);
        true
    }
    #[cfg(test)]
    fn apply(&self, config: Config) {
        let ticket = self.begin_refresh();
        assert!(self.complete_refresh(ticket, Ok(config)));
    }
    pub(crate) fn clear(&self) {
        let mut s = self.lock();
        s.generation = s
            .generation
            .checked_add(1)
            .expect("asset cache generation exhausted");
        s.entries.clear();
        s.bytes = 0;
        s.stats.clears = s.stats.clears.saturating_add(1);
        let epoch = s.generation;
        drop(s);
        self.preload.send_replace(epoch);
    }
    pub(crate) fn status(&self) -> serde_json::Value {
        let s = self.lock();
        serde_json::json!({"scope":"public-static-representations-only","config":s.config,"effectivePreload":if s.config.enabled { s.config.preload } else { Preload::Off },"preloadProgress":s.progress,"generation":s.generation,"residentOwnedBytes":s.bytes,"compressionEffectiveEnabled":s.config.enabled&&s.config.compression_enabled,"encodedResidentBytes":s.entries.values().map(|entry|entry.value.encodings.bytes()).sum::<usize>(),"ownedEncodedResidentBytes":s.entries.values().map(|entry|entry.value.encodings.charge()).sum::<usize>(),"entries":s.entries.len(),"effectiveMaxFileBytes":s.config.file(),"pendingUnique":s.pending.len(),"runningLoads":s.running,"queuedUnique":s.pending.len().saturating_sub(s.running),"admissionWaiting":s.waiting,"maxQueuedUnique":QUEUED,"maxRunningLoads":WORKERS,"configurationStale":s.configuration_stale,"lastConfigAt":s.last_config_at,"counters":s.stats,"hardRssLimit":false,"embeddedRawBytesDuplicated":false})
    }
    pub(crate) async fn refresh(&self, store: &mw_store::Store) {
        let ticket = self.begin_refresh();
        let result = crate::settings_registry::effective_all(
            store,
            &mw_store::settings_policy::SettingSubject::default(),
            |d| d.key.starts_with(PREFIX),
        )
        .await
        .map(|values| Config::from_values(&values))
        .map_err(|_| ());
        self.complete_refresh(ticket, result);
    }
    pub(crate) fn start(self: &Arc<Self>, store: mw_store::Store) {
        let weak = Arc::downgrade(self);
        let mut life = self.preload.subscribe();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            interval.tick().await;
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let Some(cache) = weak.upgrade() else { break };
                        cache.refresh(&store).await;
                    }
                    result = life.changed() => { if result.is_err() { break; } }
                }
            }
        });
        self.start_preloader();
    }
    fn start_preloader(self: &Arc<Self>) {
        let weak = Arc::downgrade(self);
        let mut changes = self.preload.subscribe();
        tokio::spawn(async move {
            // Startup is nonblocking. Only one preloader per cache, and each admission
            // requires an immediately available worker with no foreground work pending.
            tokio::task::yield_now().await;
            loop {
                changes.borrow_and_update();
                let Some(cache) = weak.upgrade() else { break };
                cache.preload_pass(&changes).await;
                drop(cache);
                if changes.changed().await.is_err() {
                    break;
                }
            }
        });
    }
    /// One preload pass for the current generation. Returns early when the
    /// generation changes; the caller then starts a new pass.
    async fn preload_pass(self: &Arc<Self>, changes: &watch::Receiver<u64>) {
        let (mode, epoch) = {
            let mut s = self.lock();
            let mode = if s.config.enabled {
                s.config.preload
            } else {
                Preload::Off
            };
            let epoch = s.generation;
            s.progress = PreloadProgress {
                generation: epoch,
                mode: Some(mode),
                state: if mode == Preload::Off {
                    PreloadState::Idle
                } else {
                    PreloadState::Running
                },
                ..Default::default()
            };
            (mode, epoch)
        };
        if mode == Preload::Off {
            return;
        }
        let superseded = |cache: &Cache| {
            cache.lock().generation != epoch || changes.has_changed().unwrap_or(true)
        };
        // "app" waits for the loaders to be idle; the older modes keep their
        // single best-effort attempt.
        if mode == Preload::App && !self.wait_idle(&superseded).await {
            return self.finish_pass(epoch, PreloadState::Superseded);
        }
        let shell = self
            .get_inner(Source::Web, Variant::Public, "index.html", true)
            .await
            .ok()
            .flatten();
        let paths = match mode {
            Preload::Off | Preload::Shell => Vec::new(),
            Preload::Core => {
                let Some(shell) = &shell else {
                    return self.finish_pass(epoch, PreloadState::Done);
                };
                if shell.bytes.len() > MAX_DISCOVERY_BYTES {
                    let mut state = self.lock();
                    state.stats.preload_skipped = state.stats.preload_skipped.saturating_add(1);
                }
                core_paths(&shell.bytes, &self.base)
            }
            Preload::App => {
                let available = match self.web_dir.clone() {
                    Some(root) => tokio::task::spawn_blocking(move || walk(&root))
                        .await
                        .unwrap_or_default(),
                    None => mw_web_assets::web_paths()
                        .map(Cow::into_owned)
                        .filter(|path| {
                            !crate::error_pages::is_unsafe_asset_path(path)
                        })
                        .collect(),
                };
                let mut plan = app_paths(
                    &available,
                    shell.as_ref().map(|shell| &shell.bytes[..]),
                    &self.base,
                );
                // The shell was loaded above; it stays first only for counting.
                plan.retain(|path| path != "index.html");
                let catalogs = plan.iter().filter(|path| is_source_locale_pack(path)).count();
                self.lock().progress.catalogs = catalogs;
                plan
            }
        };
        {
            let mut s = self.lock();
            if s.progress.generation == epoch {
                s.progress.planned = paths.len() + 1;
                if shell.is_some() {
                    s.progress.ready += 1;
                } else {
                    s.progress.absent += 1;
                }
            }
        }
        for path in paths {
            if !self.wait_idle(&superseded).await {
                return self.finish_pass(epoch, PreloadState::Superseded);
            }
            {
                let s = self.lock();
                if mode == Preload::App
                    && (s.bytes >= s.config.budget() / 4 * 3
                        || s.entries.len() >= s.config.max_entries / 4 * 3)
                {
                    drop(s);
                    return self.finish_pass(epoch, PreloadState::Limited);
                }
            }
            let mut outcome = Err(Busy);
            for _ in 0..3 {
                outcome = self
                    .get_inner(Source::Web, Variant::Public, &path, true)
                    .await;
                if outcome.is_ok() || mode != Preload::App || !self.wait_idle(&superseded).await {
                    break;
                }
            }
            {
                let mut s = self.lock();
                if s.progress.generation == epoch {
                    match outcome {
                        Ok(Some(_)) => s.progress.ready += 1,
                        Ok(None) => s.progress.absent += 1,
                        Err(_) => s.progress.skipped += 1,
                    }
                }
            }
            tokio::task::yield_now().await;
        }
        let state = if superseded(self.as_ref()) {
            PreloadState::Superseded
        } else {
            PreloadState::Done
        };
        self.finish_pass(epoch, state);
    }
    fn finish_pass(&self, epoch: u64, state: PreloadState) {
        let mut s = self.lock();
        if s.progress.generation == epoch {
            s.progress.state = state;
        }
    }
    /// Waits until no foreground miss is pending or waiting and a worker is free.
    async fn wait_idle(&self, superseded: &impl Fn(&Cache) -> bool) -> bool {
        loop {
            if superseded(self) {
                return false;
            }
            {
                let s = self.lock();
                if s.pending.is_empty() && s.waiting == 0 && self.workers.available_permits() > 0 {
                    return true;
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    pub(crate) async fn get(
        self: &Arc<Self>,
        source: Source,
        variant: Variant,
        path: &str,
    ) -> Result<Option<Prepared>, Busy> {
        self.get_inner(source, variant, path, false).await
    }
    async fn get_inner(
        self: &Arc<Self>,
        source: Source,
        variant: Variant,
        path: &str,
        preload: bool,
    ) -> Result<Option<Prepared>, Busy> {
        if path.len() > 4096
            || self.base.len() > 4096
            || crate::error_pages::is_unsafe_asset_path(path)
            || !std::path::Path::new(path)
                .components()
                .all(|part| matches!(part, std::path::Component::Normal(_)))
            || path.ends_with(".map") || path.ends_with(".map.gz") || path.ends_with(".map.br")
            || variant == Variant::Console && path != "index.html"
        {
            return Ok(None);
        }
        let deadline = tokio::time::Instant::now() + OBSERVE;
        let mut admission: Option<OwnedSemaphorePermit> = None;
        let mut previous: Option<(Prepared, Stamp)> = None;
        let mut receiver = loop {
            {
                let mut s = self.lock();
                let key = Key {
                    source,
                    variant,
                    path: path.into(),
                    base: self.base.clone(),
                    generation: s.generation,
                };
                s.clock = s.clock.saturating_add(1);
                let clock = s.clock;
                if s.config.enabled {
                    if let Some(entry) = s.entries.get_mut(&key) {
                        if entry
                            .expires
                            .map_or(true, |expires| expires > Instant::now())
                        {
                            entry.touched = clock;
                            let value = entry.value.clone();
                            s.stats.hits = s.stats.hits.saturating_add(1);
                            return Ok(Some(value));
                        }
                    }
                    if let Some(old) = s.entries.remove(&key) {
                        s.bytes = s.bytes.saturating_sub(old.value.charge);
                        s.stats.expired = s.stats.expired.saturating_add(1);
                        if let Some(stamp) = old.stamp {
                            previous = Some((old.value, stamp));
                        }
                    }
                }
                if let Some(flight) = s.pending.get(&key).cloned() {
                    s.stats.coalesced = s.stats.coalesced.saturating_add(1);
                    break flight.result.subscribe();
                }
                if preload {
                    if !s.pending.is_empty() || s.waiting > 0 {
                        s.stats.preload_skipped = s.stats.preload_skipped.saturating_add(1);
                        return Err(Busy);
                    }
                    match self.workers.clone().try_acquire_owned() {
                        Ok(worker) => {
                            break self.spawn_job(&mut s, key, Some(worker), None, true, previous.take());
                        }
                        Err(_) => {
                            s.stats.preload_skipped = s.stats.preload_skipped.saturating_add(1);
                            return Err(Busy);
                        }
                    }
                }
                if let Some(queued) = admission
                    .take()
                    .or_else(|| self.queue.clone().try_acquire_owned().ok())
                {
                    break self.spawn_job(&mut s, key, None, Some(queued), false, previous.take());
                }
                // All dispatcher slots are taken: wait for one instead of refusing.
                s.waiting += 1;
                s.stats.admission_waits = s.stats.admission_waits.saturating_add(1);
            }
            let waiting = Waiting(self);
            let permit = tokio::time::timeout_at(deadline, self.queue.clone().acquire_owned()).await;
            drop(waiting);
            match permit {
                Ok(Ok(permit)) => admission = Some(permit),
                _ => {
                    let mut state = self.lock();
                    state.stats.busy = state.stats.busy.saturating_add(1);
                    return Err(Busy);
                }
            }
        };
        // A slot taken while waiting is not needed to join another load.
        drop(admission);
        let observed = tokio::time::timeout_at(deadline, async {
            loop {
                if let Some(value) = receiver.borrow().clone() {
                    return value;
                }
                if receiver.changed().await.is_err() {
                    return Err(Busy);
                }
            }
        })
        .await;
        match observed {
            Ok(value) => value,
            Err(_) => {
                let mut state = self.lock();
                state.stats.busy = state.stats.busy.saturating_add(1);
                Err(Busy)
            }
        }
    }
    fn spawn_job(
        self: &Arc<Self>,
        s: &mut Inner,
        key: Key,
        worker: Option<OwnedSemaphorePermit>,
        queued: Option<OwnedSemaphorePermit>,
        preload: bool,
        previous: Option<(Prepared, Stamp)>,
    ) -> watch::Receiver<Option<Result<Option<Prepared>, Busy>>> {
        let (sender, receiver) = watch::channel(None);
        let flight = Arc::new(Flight { result: sender });
        s.pending.insert(key.clone(), flight.clone());
        s.stats.misses = s.stats.misses.saturating_add(1);
        if worker.is_some() {
            s.running += 1;
        }
        let mut job = Job {
            config: s.config,
            cache: self.clone(),
            key,
            flight,
            running: worker.is_some(),
            worker,
            queued,
            preload,
            previous,
            done: false,
        };
        tokio::spawn(async move {
            if job.worker.is_none() {
                let Ok(worker) = job.cache.workers.clone().acquire_owned().await else {
                    return;
                };
                job.worker = Some(worker);
                job.running = true;
                job.cache.lock().running += 1;
                job.queued.take();
            }
            // The blocking closure owns cleanup and permits. Cancelling the HTTP waiter
            // or aborting this async dispatcher cannot release a still-running worker.
            let _ = tokio::task::spawn_blocking(move || {
                let result =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| job.load()));
                match result {
                    Ok(value) => job.finish(value, false),
                    Err(_) => job.finish(None, true),
                }
            })
            .await;
        });
        receiver
    }
}
struct Job {
    config: Config,
    cache: Arc<Cache>,
    key: Key,
    flight: Arc<Flight>,
    worker: Option<OwnedSemaphorePermit>,
    queued: Option<OwnedSemaphorePermit>,
    running: bool,
    preload: bool,
    previous: Option<(Prepared, Stamp)>,
    done: bool,
}
impl Job {
    fn load(&self) -> Option<Loaded> {
        #[cfg(test)]
        {
            let hook = self.cache.before_load.lock().unwrap().clone();
            if let Some(hook) = hook {
                hook(&self.key.path);
            }
        }
        let deployed = match (self.key.source, &self.cache.web_dir) {
            (Source::Web, Some(root)) => Some(root.join(&self.key.path)),
            _ => None,
        };
        // Taken before the read: a write after this point changes the stamp.
        let file_stamp = deployed.as_deref().and_then(stamp);
        if let (Some((previous, old)), Some(current)) = (&self.previous, file_stamp)
            && *old == current
        {
            return Some(Loaded {
                value: previous.clone(),
                stamp: Some(current),
                shell_digest: None,
                reused: true,
            });
        }
        let raw: Cow<'static, [u8]> = match self.key.source {
            Source::Web => match &deployed {
                Some(file) => Cow::Owned(std::fs::read(file).ok()?),
                None => mw_web_assets::web(&self.key.path)?,
            },
            Source::Pdf => {
                crate::pdf_assets::load_public(self.cache.web_dir.as_deref(), &self.key.path)?
            }
            Source::Retained => Cow::Owned(self.cache.retained.as_ref()?.read(&self.key.path)?),
        };
        // A content-hashed name of this build cannot change content, so a
        // deployed copy is kept like embedded bytes; a new build has new names
        // and a new shell (see `shell_digest`).
        let hashed = self.key.source == Source::Web && crate::is_content_hashed(&self.key.path);
        // A PDF embedded fallback can become a permitted deployed override later.
        // Its bytes are static, but source selection must still be revalidated.
        let immutable = hashed
            || matches!(&raw, Cow::Borrowed(_))
                && !(self.key.source == Source::Pdf && self.cache.web_dir.is_some());
        let shell_digest = (deployed.is_some() && self.key.path == "index.html")
            .then(|| crate::asset_etag(&raw));
        let raw = if self.key.variant == Variant::Raw || self.key.source == Source::Pdf {
            raw
        } else if self.key.variant == Variant::Console {
            let bytes =
                crate::adapt_asset_to_base(&self.key.path, raw.into_owned(), &self.key.base);
            let html = String::from_utf8(bytes).ok()?;
            Cow::Owned(crate::mark_console_shell(&html, &self.key.base).into_bytes())
        } else if self.key.base.is_empty()
            || !(self.key.path == "index.html"
                || self.key.path.ends_with("/index.html")
                || self.key.path.ends_with(".css"))
        {
            raw
        } else {
            Cow::Owned(crate::adapt_asset_to_base(
                &self.key.path,
                raw.into_owned(),
                &self.key.base,
            ))
        };
        let mut prepared = Prepared::new(raw);
        prepared.immutable = immutable;
        prepared.compressible = self.key.variant == Variant::Public && crate::public_asset_compression::eligible(&self.key.path);
        if prepared.compressible && self.config.enabled && self.config.compression_enabled && prepared.bytes.len()>=self.config.compression_min_bytes && self.config.encodes(&prepared) {
            prepared.encodings = crate::public_asset_compression::prepare(&prepared.bytes,self.config.brotli_enabled,self.config.budget().saturating_sub(prepared.charge),|coding|{
                let sidecar=format!("{}{}",self.key.path,coding.suffix());
                match (&self.key.source,&self.cache.web_dir) {
                    (Source::Web|Source::Pdf,Some(root))=>{
                        use std::io::Read;
                        let path=root.join(sidecar);
                        if !std::fs::metadata(&path).ok()?.is_file(){return None;}
                        let file=std::fs::File::open(path).ok()?;
                        let max=prepared.bytes.len();
                        if file.metadata().ok()?.len()>max as u64{return None;}
                        let mut bytes=Vec::new();file.take(max as u64+1).read_to_end(&mut bytes).ok()?;
                        (bytes.len()<=max).then_some(Cow::Owned(bytes))
                    }
                    (Source::Web,None)=>mw_web_assets::web(&sidecar).filter(|bytes|bytes.len()<=prepared.bytes.len()),
                    (Source::Pdf,None)=>mw_web_assets::pdf(&sidecar).filter(|bytes|bytes.len()<=prepared.bytes.len()),
                    _=>None,
                }
            });
            prepared.charge += prepared.encodings.charge();
        }
        Some(Loaded {
            value: prepared,
            stamp: if immutable { None } else { file_stamp },
            shell_digest,
            reused: false,
        })
    }
    fn finish(&mut self, loaded: Option<Loaded>, failed: bool) {
        if self.done {
            return;
        }
        self.done = true;
        let mut s = self.cache.lock();
        s.pending.remove(&self.key);
        if self.running {
            s.running = s.running.saturating_sub(1);
        }
        s.stats.loads = s.stats.loads.saturating_add(1);
        if failed {
            s.stats.load_failures = s.stats.load_failures.saturating_add(1);
        }
        let mut discarded = None;
        let mut new_epoch = None;
        if let Some(loaded) = &loaded {
            let value = &loaded.value;
            if loaded.reused {
                s.stats.revalidated_unchanged = s.stats.revalidated_unchanged.saturating_add(1);
            } else {
                for encoded in [value.encodings.gzip.as_ref(),value.encodings.brotli.as_ref()].into_iter().flatten() {
                    if encoded.generated {s.stats.generated_encodings=s.stats.generated_encodings.saturating_add(1);}else{s.stats.validated_sidecars=s.stats.validated_sidecars.saturating_add(1);}
                }
            }
            // A changed deployed shell means a new build: drop the old set under a
            // new generation, as clear does. Only a load started in the current
            // generation may decide this, so a slow read of the previous shell
            // cannot flip it back.
            let mut key = self.key.clone();
            if let Some(digest) = &loaded.shell_digest
                && s.generation == key.generation
            {
                if s.shell_digest.as_ref().is_some_and(|old| old != digest) {
                    s.generation = s
                        .generation
                        .checked_add(1)
                        .expect("asset cache generation exhausted");
                    s.bytes = 0;
                    discarded = Some(std::mem::take(&mut s.entries));
                    s.stats.shell_changes = s.stats.shell_changes.saturating_add(1);
                    key.generation = s.generation;
                    new_epoch = Some(s.generation);
                }
                s.shell_digest = Some(digest.clone());
            }
            if s.generation != key.generation {
                s.stats.stale_inserts = s.stats.stale_inserts.saturating_add(1);
            } else if s.config.enabled
                && value.charge <= s.config.budget()
                && (value.charge == 0 || value.bytes.len() <= s.config.file())
            {
                while s.entries.len() >= s.config.max_entries
                    || s.bytes.saturating_add(value.charge) > s.config.budget()
                {
                    let Some(old) = s
                        .entries
                        .iter()
                        .min_by_key(|(_, e)| e.touched)
                        .map(|(k, _)| k.clone())
                    else {
                        break;
                    };
                    if let Some(e) = s.entries.remove(&old) {
                        s.bytes = s.bytes.saturating_sub(e.value.charge);
                        s.stats.evictions = s.stats.evictions.saturating_add(1);
                    }
                }
                s.clock = s.clock.saturating_add(1);
                let touched = s.clock;
                let expires = if value.immutable {
                    None
                } else {
                    Some(Instant::now() + Duration::from_secs(s.config.revalidate_seconds))
                };
                s.bytes += value.charge;
                s.entries.insert(
                    key,
                    Entry {
                        value: value.clone(),
                        expires,
                        stamp: loaded.stamp,
                        touched,
                    },
                );
                if self.preload {
                    s.stats.preload_loaded = s.stats.preload_loaded.saturating_add(1);
                }
            } else if value.charge > 0 && value.bytes.len() > s.config.file() {
                s.stats.oversize = s.stats.oversize.saturating_add(1);
            }
        }
        drop(s);
        drop(discarded);
        if let Some(epoch) = new_epoch {
            self.cache.preload.send_replace(epoch);
        }
        self.flight.result.send_replace(Some(if failed {
            Err(Busy)
        } else {
            Ok(loaded.map(|loaded| loaded.value))
        }));
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        if !self.done {
            self.finish(None, true);
        }
    }
}

#[cfg(test)]
#[path = "public_asset_cache_tests.rs"]
mod tests;
