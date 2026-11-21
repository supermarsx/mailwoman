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
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, watch};
const WORKERS: usize = 2;
const QUEUED: usize = 16;
const PREFIX: &str = "web.assets.cache.";
const MAX_DISCOVERY_BYTES: usize = 256 * 1024;
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Preload {
    Off,
    Shell,
    Core,
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
            budget_mib: 64,
            max_file_mib: 4,
            max_entries: 1024,
            revalidate_seconds: 1,
            preload: Preload::Shell,
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
            .unwrap_or(64) as u32;
        out.max_file_mib = v("maxFileMiB")
            .and_then(serde_json::Value::as_u64)
            .filter(|n| (1..=16).contains(n))
            .unwrap_or(4) as u32;
        out.max_entries = v("maxEntries")
            .and_then(serde_json::Value::as_u64)
            .filter(|n| (16..=4096).contains(n))
            .unwrap_or(1024) as usize;
        out.revalidate_seconds = v("revalidateSeconds")
            .and_then(serde_json::Value::as_u64)
            .filter(|n| (1..=60).contains(n))
            .unwrap_or(1);
        out.preload = match v("preload").and_then(serde_json::Value::as_str) {
            Some("off") => Preload::Off,
            Some("core") => Preload::Core,
            _ => Preload::Shell,
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
struct Entry {
    value: Prepared,
    expires: Option<Instant>,
    touched: u64,
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
        serde_json::json!({"scope":"public-static-representations-only","config":s.config,"effectivePreload":if s.config.enabled { s.config.preload } else { Preload::Off },"generation":s.generation,"residentOwnedBytes":s.bytes,"compressionEffectiveEnabled":s.config.enabled&&s.config.compression_enabled,"encodedResidentBytes":s.entries.values().map(|entry|entry.value.encodings.bytes()).sum::<usize>(),"ownedEncodedResidentBytes":s.entries.values().map(|entry|entry.value.encodings.charge()).sum::<usize>(),"entries":s.entries.len(),"effectiveMaxFileBytes":s.config.file(),"pendingUnique":s.pending.len(),"runningLoads":s.running,"queuedUnique":s.pending.len().saturating_sub(s.running),"maxQueuedUnique":QUEUED,"maxRunningLoads":WORKERS,"configurationStale":s.configuration_stale,"lastConfigAt":s.last_config_at,"counters":s.stats,"hardRssLimit":false,"embeddedRawBytesDuplicated":false})
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
        let weak = Arc::downgrade(self);
        let mut changes = self.preload.subscribe();
        tokio::spawn(async move {
            // Startup is nonblocking. Only one preloader per cache, and each admission
            // requires an immediately available worker with no foreground work pending.
            tokio::task::yield_now().await;
            loop {
                let Some(cache) = weak.upgrade() else { break };
                let (mode, epoch) = {
                    let s = cache.lock();
                    (
                        if s.config.enabled {
                            s.config.preload
                        } else {
                            Preload::Off
                        },
                        s.generation,
                    )
                };
                if mode != Preload::Off {
                    let shell = cache
                        .get_inner(Source::Web, Variant::Public, "index.html", true)
                        .await
                        .ok()
                        .flatten();
                    if mode == Preload::Core
                        && let Some(shell) = shell
                    {
                        if shell.bytes.len() > MAX_DISCOVERY_BYTES {
                            let mut state = cache.lock();
                            state.stats.preload_skipped =
                                state.stats.preload_skipped.saturating_add(1);
                        }
                        let paths = core_paths(&shell.bytes, &cache.base);
                        for path in paths {
                            if cache.lock().generation != epoch
                                || changes.has_changed().unwrap_or(true)
                            {
                                break;
                            }
                            let _ = cache
                                .get_inner(Source::Web, Variant::Public, &path, true)
                                .await;
                            tokio::task::yield_now().await;
                        }
                    }
                }
                drop(cache);
                if changes.changed().await.is_err() {
                    break;
                }
                changes.borrow_and_update();
            }
        });
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
        let mut receiver = {
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
                }
            }
            let receiver = if let Some(flight) = s.pending.get(&key).cloned() {
                s.stats.coalesced = s.stats.coalesced.saturating_add(1);
                flight.result.subscribe()
            } else {
                let (worker, queued) = if preload {
                    if !s.pending.is_empty() {
                        s.stats.preload_skipped = s.stats.preload_skipped.saturating_add(1);
                        return Err(Busy);
                    }
                    match self.workers.clone().try_acquire_owned() {
                        Ok(w) => (Some(w), None),
                        Err(_) => {
                            s.stats.preload_skipped = s.stats.preload_skipped.saturating_add(1);
                            return Err(Busy);
                        }
                    }
                } else {
                    match self.queue.clone().try_acquire_owned() {
                        Ok(q) => (None, Some(q)),
                        Err(_) => {
                            s.stats.busy = s.stats.busy.saturating_add(1);
                            return Err(Busy);
                        }
                    }
                };
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
                    worker,
                    queued,
                    running: preload,
                    preload,
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
            };
            receiver
        };
        let observed = tokio::time::timeout(Duration::from_secs(5), async {
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
    done: bool,
}
impl Job {
    fn load(&self) -> Option<Prepared> {
        #[cfg(test)]
        {
            let hook = self.cache.before_load.lock().unwrap().clone();
            if let Some(hook) = hook {
                hook(&self.key.path);
            }
        }
        let raw: Cow<'static, [u8]> = match self.key.source {
            Source::Web => match &self.cache.web_dir {
                Some(root) => Cow::Owned(std::fs::read(root.join(&self.key.path)).ok()?),
                None => mw_web_assets::web(&self.key.path)?,
            },
            Source::Pdf => {
                crate::pdf_assets::load_public(self.cache.web_dir.as_deref(), &self.key.path)?
            }
            Source::Retained => Cow::Owned(self.cache.retained.as_ref()?.read(&self.key.path)?),
        };
        // A PDF embedded fallback can become a permitted deployed override later.
        // Its bytes are static, but source selection must still be revalidated.
        let immutable = matches!(&raw, Cow::Borrowed(_))
            && !(self.key.source == Source::Pdf && self.cache.web_dir.is_some());
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
        if prepared.compressible && self.config.enabled && self.config.compression_enabled && prepared.bytes.len()>=self.config.compression_min_bytes && prepared.bytes.len()<=self.config.file() {
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
        Some(prepared)
    }
    fn finish(&mut self, value: Option<Prepared>, failed: bool) {
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
        if let Some(value) = &value {
            for encoded in [value.encodings.gzip.as_ref(),value.encodings.brotli.as_ref()].into_iter().flatten() {
                if encoded.generated {s.stats.generated_encodings=s.stats.generated_encodings.saturating_add(1);}else{s.stats.validated_sidecars=s.stats.validated_sidecars.saturating_add(1);}
            }
            if s.generation != self.key.generation {
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
                    self.key.clone(),
                    Entry {
                        value: value.clone(),
                        expires,
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
        self.flight
            .result
            .send_replace(Some(if failed { Err(Busy) } else { Ok(value) }));
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
