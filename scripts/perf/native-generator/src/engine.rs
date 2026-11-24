//! One measured stage, with the semantics of `measure()` in http-load.mjs:
//! open-loop slots at a fixed arrival rate (late=drop or late=dispatch) or a
//! closed loop of `concurrency` senders; per-principal FIFO serialisation;
//! a mutation whose outcome is unknown taints the stage and stops it; every
//! counter the Node report is built from. The Node adapter turns the raw output
//! into the stage object with Node's own report functions.
use crate::http1::{self, Limits};
use crate::http2::{self, H2Config};
use crate::measure::{BytesProfile, Hdr, Meas, Reasons503, Status, Timings, classify_503};
use crate::plan::{PlanFile, Principal};
use crate::templates::{self, Spec};
use crate::validate::{Validator, Verdict};
use bytes::Bytes;
use serde::Deserialize;
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

pub const MAX_OUTSTANDING: usize = 50000;
pub const MAX_STAGE_SECONDS: f64 = 3600.0;
pub const MAX_OFFERED_RPS: f64 = 250000.0;
pub const SCHEDULING_LAG_LIMIT_MS: f64 = 1.0;
const MAX_THREADS: usize = 64;

fn default_timeout() -> f64 {
    10000.0
}
fn default_late() -> String {
    "drop".into()
}
fn default_runtime() -> String {
    "multi".into()
}
fn yes() -> bool {
    true
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlainRequest {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub body: Option<String>,
    /// none | json (JSON must parse) | ok-true (`body.ok`)
    #[serde(default)]
    pub validate: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DraftEntry {
    pub principal_index: usize,
    pub id: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StageSpec {
    pub base: String,
    #[serde(default)]
    pub bases: Option<Vec<String>>,
    #[serde(default)]
    pub allowed_targets: Vec<String>,
    /// group name of the plan; ignored with `plain`
    #[serde(default)]
    pub endpoint: String,
    pub concurrency: usize,
    pub seconds: f64,
    #[serde(default = "default_timeout")]
    pub timeout_ms: f64,
    #[serde(default)]
    pub arrival_rate: Option<f64>,
    #[serde(default = "default_late")]
    pub late_policy: String,
    #[serde(default)]
    pub common_start_at_ms: Option<f64>,
    #[serde(default)]
    pub h2: Option<H2Config>,
    #[serde(default)]
    pub draft_pool: Vec<DraftEntry>,
    #[serde(default)]
    pub draft_pool_out: Option<String>,
    #[serde(default)]
    pub threads: Option<usize>,
    /// multi (one tokio multi-thread runtime) | per-core (one current-thread runtime per thread, principals affine)
    #[serde(default = "default_runtime")]
    pub runtime: String,
    #[serde(default = "yes")]
    pub rate_guard: bool,
    /// calibration without a plan: fixed requests and stage headers
    #[serde(default)]
    pub plain: Option<Vec<PlainRequest>>,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    /// scheduler wake-up period in microseconds (50-1000)
    #[serde(default = "default_tick")]
    pub tick_micros: u64,
    /// HTTP/1.1 keep-alive connections opened before the window (Node's pool is warm from earlier stages; a native stage is a fresh process)
    #[serde(default)]
    pub prewarm_connections: Option<usize>,
}

// ---- targets (http-targets.mjs rules) ----
fn classify_host(host: &str) -> &'static str {
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    if let Ok(v4) = bare.parse::<Ipv4Addr>() {
        if bare == "127.0.0.1" {
            return "loopback";
        }
        let [a, b, _, _] = v4.octets();
        if a == 10 || (a == 172 && (16..=31).contains(&b)) || (a == 192 && b == 168) {
            return "private";
        }
        return "refused";
    }
    if let Ok(v6) = bare.parse::<std::net::Ipv6Addr>() {
        if !bare.contains('.') && (v6.segments()[0] & 0xfe00) == 0xfc00 {
            return "private";
        }
    }
    "refused"
}
/// Plain http origin with a literal address and explicit port; loopback, or private and allow-listed.
fn parse_target(text: &str, allowed: &[String]) -> Result<(SocketAddr, String), String> {
    let rest = text
        .strip_prefix("http://")
        .ok_or("Invalid target origin")?;
    let rest = rest.strip_suffix('/').unwrap_or(rest);
    if rest.contains('/') || rest.contains('@') || rest.contains('?') || rest.contains('#') {
        return Err("Invalid target origin".into());
    }
    let addr: SocketAddr = rest.parse().map_err(|_| "Invalid target origin")?;
    if addr.port() == 0 {
        return Err("Invalid target origin".into());
    }
    let host = match addr {
        SocketAddr::V4(a) => a.ip().to_string(),
        SocketAddr::V6(a) => format!("[{}]", a.ip()),
    };
    match classify_host(&host) {
        "loopback" => {}
        "private" => {
            let origin = format!("http://{rest}");
            if !allowed.iter().any(|a| {
                a.trim_end_matches('/') == origin || format!("http://{}", a.trim()) == origin
            }) {
                return Err("Private target is not in the explicit allow-list".into());
            }
        }
        _ => return Err("Target address is not a literal loopback or private rig address".into()),
    }
    Ok((addr, rest.to_owned()))
}

// ---- prebuilt requests ----
pub struct Prebuilt {
    pub spec: Spec,
    /// "<METHOD> <path> HTTP/1.1\r\nhost: "
    head: Vec<u8>,
    /// after the host value up to the forwarded address value (or everything when there is none)
    pre: Vec<u8>,
    /// after the forwarded address value
    post: Vec<u8>,
    forwarded: bool,
    h2_headers: Vec<(http::HeaderName, http::HeaderValue)>,
    h2_forwarded_at: Option<usize>,
    body: Option<Bytes>,
}

const FETCH_DEFAULTS: [(&str, &str); 5] = [
    ("accept", "*/*"),
    ("accept-language", "*"),
    ("sec-fetch-mode", "cors"),
    ("user-agent", "node"),
    ("accept-encoding", "gzip, deflate"),
];
const H2_FORBIDDEN: [&str; 6] = [
    "host",
    "connection",
    "keep-alive",
    "transfer-encoding",
    "upgrade",
    "proxy-connection",
];

/// Header order of `fetch(url,{headers:{...measureHeaders,...spec.headers}})`.
fn merged_headers(
    first: &[(String, String)],
    client: &[(String, String)],
    spec: &[(String, String)],
) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let set = |k: &str, v: &str, out: &mut Vec<(String, String)>| {
        if let Some(entry) = out.iter_mut().find(|(n, _)| n == k) {
            entry.1 = v.to_owned();
        } else {
            out.push((k.to_owned(), v.to_owned()));
        }
    };
    for (k, v) in first {
        set(k, v, &mut out);
    }
    for (k, v) in client.iter().chain(spec) {
        set(k, v, &mut out);
    }
    out
}

fn prebuild(
    spec: Spec,
    first_headers: &[(String, String)],
    client_headers: &[(String, String)],
    forwarded: bool,
) -> Prebuilt {
    let headers = merged_headers(first_headers, client_headers, &spec.headers);
    let mut head = Vec::new();
    head.extend_from_slice(format!("{} {} HTTP/1.1\r\nhost: ", spec.method, spec.path).as_bytes());
    let mut pre = b"\r\nconnection: keep-alive\r\n".to_vec();
    let mut post = Vec::new();
    let mut seen_forwarded = false;
    for (k, v) in &headers {
        let target = if seen_forwarded { &mut post } else { &mut pre };
        if k == "x-forwarded-for" && forwarded {
            target.extend_from_slice(b"x-forwarded-for: ");
            seen_forwarded = true;
            post.extend_from_slice(b"\r\n");
            continue;
        }
        target.extend_from_slice(format!("{k}: {v}\r\n").as_bytes());
    }
    let tail = if seen_forwarded { &mut post } else { &mut pre };
    for (k, v) in FETCH_DEFAULTS {
        if !headers.iter().any(|(n, _)| n == k) {
            tail.extend_from_slice(format!("{k}: {v}\r\n").as_bytes());
        }
    }
    match (&spec.body, spec.method) {
        (Some(body), _) => tail.extend_from_slice(
            format!("content-length: {}\r\n\r\n{}", body.len(), body).as_bytes(),
        ),
        (None, "POST") => tail.extend_from_slice(b"content-length: 0\r\n\r\n"),
        (None, _) => tail.extend_from_slice(b"\r\n"),
    }
    let mut h2_headers = Vec::new();
    let mut h2_forwarded_at = None;
    for (k, v) in &headers {
        if H2_FORBIDDEN.contains(&k.as_str()) {
            continue;
        }
        if k == "x-forwarded-for" && forwarded {
            h2_forwarded_at = Some(h2_headers.len());
        }
        h2_headers.push((
            http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
            http::HeaderValue::from_str(v).unwrap_or(http::HeaderValue::from_static("")),
        ));
    }
    if !headers.iter().any(|(n, _)| n == "accept-encoding") {
        h2_headers.push((
            http::header::ACCEPT_ENCODING,
            http::HeaderValue::from_static("gzip, deflate"),
        ));
    }
    let body = spec.body.clone().map(Bytes::from);
    Prebuilt {
        spec,
        head,
        pre,
        post,
        forwarded: seen_forwarded,
        h2_headers,
        h2_forwarded_at,
        body,
    }
}

impl Prebuilt {
    fn h1_bytes(&self, out: &mut Vec<u8>, host: &str, address: &str) {
        out.clear();
        out.extend_from_slice(&self.head);
        out.extend_from_slice(host.as_bytes());
        out.extend_from_slice(&self.pre);
        if self.forwarded {
            out.extend_from_slice(address.as_bytes());
            out.extend_from_slice(&self.post);
        }
    }
    /// The full HTTP/1.1 request as fetch sends it (for the template comparison).
    pub fn h1_text(&self, host: &str, address: &str) -> Vec<u8> {
        let mut out = Vec::new();
        self.h1_bytes(&mut out, host, address);
        out
    }
}

// ---- per-thread counters ----
#[derive(Default)]
struct Participants {
    attempted: HashMap<usize, u64>,
    accepted: HashMap<usize, u64>,
    within: HashSet<usize>,
}
impl Participants {
    fn merge(&mut self, o: &Participants) {
        for (k, v) in &o.attempted {
            *self.attempted.entry(*k).or_default() += v;
        }
        for (k, v) in &o.accepted {
            *self.accepted.entry(*k).or_default() += v;
        }
        self.within.extend(&o.within);
    }
}
fn distribution(counts: &HashMap<usize, u64>) -> Value {
    let total: u64 = counts.values().sum();
    let n = counts.len();
    json!({"participants": n, "requests": total, "min": counts.values().min().copied().unwrap_or(0), "max": counts.values().max().copied().unwrap_or(0), "mean": if n > 0 { total as f64 / n as f64 } else { 0.0 }})
}

#[derive(Default)]
struct Shard {
    aggregate: Meas,
    by: Vec<Meas>,
    arrival: Meas,
    scheduling: Meas,
    profile: BytesProfile,
    e503: Reasons503,
    e503_by: Vec<Reasons503>,
    lat: Timings,
    lat_by: Vec<Timings>,
    principals: Participants,
    clients: Participants,
    per_second: Vec<u64>,
    accepted_within: u64,
    hdr: Hdr,
    hdr_by: Vec<Hdr>,
    lag_hdr: Hdr,
    validation_hdr: Hdr,
    /// largest scheduling lag (ms) per second of the window, by planned slot time
    lag_max_by_second: Vec<f64>,
}
impl Shard {
    fn new(names: usize, seconds: usize) -> Shard {
        Shard {
            by: vec![Meas::default(); names],
            e503_by: vec![Reasons503::default(); names],
            lat_by: vec![Timings::default(); names],
            hdr_by: vec![Hdr::default(); names],
            per_second: vec![0; seconds],
            lag_max_by_second: vec![0.0; seconds],
            ..Default::default()
        }
    }
    fn merge(&mut self, o: &Shard) {
        self.aggregate.merge(&o.aggregate);
        for (a, b) in self.by.iter_mut().zip(&o.by) {
            a.merge(b);
        }
        self.arrival.merge(&o.arrival);
        self.scheduling.merge(&o.scheduling);
        self.profile.merge(&o.profile);
        self.e503.merge(&o.e503);
        for (a, b) in self.e503_by.iter_mut().zip(&o.e503_by) {
            a.merge(b);
        }
        self.lat.merge(&o.lat);
        for (a, b) in self.lat_by.iter_mut().zip(&o.lat_by) {
            a.merge(b);
        }
        self.principals.merge(&o.principals);
        self.clients.merge(&o.clients);
        for (a, b) in self.per_second.iter_mut().zip(&o.per_second) {
            *a += b;
        }
        self.accepted_within += o.accepted_within;
        self.hdr.merge(&o.hdr);
        for (a, b) in self.hdr_by.iter_mut().zip(&o.hdr_by) {
            a.merge(b);
        }
        self.lag_hdr.merge(&o.lag_hdr);
        self.validation_hdr.merge(&o.validation_hdr);
        for (a, b) in self.lag_max_by_second.iter_mut().zip(&o.lag_max_by_second) {
            *a = a.max(*b);
        }
    }
}
static DIMS: OnceLock<(usize, usize)> = OnceLock::new();
static SHARDS: Mutex<Vec<Arc<Mutex<Shard>>>> = Mutex::new(Vec::new());
thread_local! {
    static SHARD: RefCell<Option<Arc<Mutex<Shard>>>> = const { RefCell::new(None) };
}
fn with_shard<T>(f: impl FnOnce(&mut Shard) -> T) -> T {
    SHARD.with(|cell| {
        let mut slot = cell.borrow_mut();
        let shard = slot.get_or_insert_with(|| {
            let (names, seconds) = *DIMS.get().unwrap();
            let shard = Arc::new(Mutex::new(Shard::new(names, seconds)));
            SHARDS.lock().unwrap().push(shard.clone());
            shard
        });
        let mut guard = shard.lock().unwrap();
        f(&mut guard)
    })
}

// ---- stage context ----
struct SlotDef {
    unique: usize,
    name: usize,
    serialize: bool,
    stateful: bool,
    discard: bool,
}

enum Transport {
    H1(Vec<Vec<http1::Pool>>), // [runtime][target]
    H2(Arc<http2::Client>),
}

struct Ctx {
    start: Instant,
    deadline: Instant,
    late_dispatch: bool,
    timeout: Duration,
    seconds: f64,
    slots: Vec<SlotDef>,
    principals: Vec<Principal>,
    first_headers: Vec<(String, String)>,
    simulated: bool,
    prebuilt: Vec<Vec<Arc<Prebuilt>>>,
    locks: Vec<tokio::sync::Mutex<()>>,
    rotation: Vec<AtomicU64>,
    pool: Mutex<VecDeque<(usize, String)>>,
    tainted: AtomicBool,
    tainted_principals: Mutex<HashSet<usize>>,
    unknown_mutation_outcomes: AtomicU64,
    blocked_after_taint: AtomicU64,
    http_dispatch_attempts: AtomicU64,
    late_slots_dispatched: AtomicU64,
    outstanding: AtomicUsize,
    peak_outstanding: AtomicUsize,
    finished: tokio::sync::Notify,
    sequence: AtomicU64,
    transport: Transport,
    targets: Vec<(SocketAddr, String)>,
    runtimes: usize,
    per_core: bool,
    sends: AtomicU64,
}

struct Materialized {
    sequence: u64,
    principal: usize,
    slot: usize,
    prebuilt: Arc<Prebuilt>,
}

impl Ctx {
    fn materialize(&self, sequence: u64) -> Materialized {
        let count = self.slots.len() as u64;
        let slot = (sequence % count) as usize;
        let def = &self.slots[slot];
        let mut principal =
            templates::principal_index(sequence, count, self.principals.len() as u64) as usize;
        let mut prebuilt = self.prebuilt[principal][def.unique].clone();
        if def.discard {
            if let Some((owner, id)) = self.pool.lock().unwrap().pop_front() {
                principal = owner;
                let p = &self.principals[owner];
                prebuilt = Arc::new(prebuild(
                    templates::draft_discard(p, &id),
                    &self.first_headers,
                    &p.headers,
                    self.simulated,
                ));
            }
        }
        Materialized {
            sequence,
            principal,
            slot,
            prebuilt,
        }
    }
    fn runtime_of(&self, principal: usize) -> usize {
        if self.per_core {
            principal % self.runtimes
        } else {
            0
        }
    }
    fn ms(&self, at: Instant) -> f64 {
        at.saturating_duration_since(self.start).as_secs_f64() * 1000.0
    }
}

fn bounded(index: usize) -> bool {
    index < 10000
}

/// `send()` of measure(): one request, from principal queue to accounting.
async fn send(ctx: Arc<Ctx>, m: Materialized, planned: Instant) {
    let began = Instant::now();
    let def = &ctx.slots[m.slot];
    let p = &ctx.principals[m.principal];
    let turn = if p.addresses.len() > 1 {
        ctx.rotation[m.principal].fetch_add(1, Ordering::Relaxed) as usize % p.addresses.len()
    } else {
        0
    };
    let client_index = p.client_indexes.get(turn).copied();
    let address = p.addresses.get(turn).map(String::as_str).unwrap_or("");
    with_shard(|s| {
        if bounded(m.principal) {
            *s.principals.attempted.entry(m.principal).or_default() += 1;
        }
        if let Some(c) = client_index {
            *s.clients.attempted.entry(c).or_default() += 1;
        }
    });
    ctx.sends.fetch_add(1, Ordering::Relaxed);
    let lag_base = began.saturating_duration_since(planned).as_secs_f64() * 1000.0;
    let h2 = matches!(ctx.transport, Transport::H2(_));
    let mut lag_recorded = !h2;
    if ctx.late_dispatch && lag_base > 100.0 {
        ctx.late_slots_dispatched.fetch_add(1, Ordering::Relaxed);
    }
    let deadline = began + ctx.timeout;
    let mut status = Status::Network;
    let mut bytes: u64 = 0;
    let mut ok = false;
    let mut timed_out = false;
    let mut dispatched = false;
    let mut dispatch_at: Option<Instant> = None;
    let mut queue_ms = 0.0;
    let mut validation_ms: Option<f64> = None;
    let mut reason_503 = 4usize;
    let mut encoding: Option<String> = None;
    let mut wire: Option<u64> = None;
    let mut created: Option<String> = None;
    let spec = &m.prebuilt.spec;
    let target = &ctx.targets[(m.sequence % ctx.targets.len() as u64) as usize];
    let mut lag_extra: f64 = 0.0;
    // principal queue (FIFO); the timeout runs from `began` and is checked once the turn comes
    let _guard = if def.serialize {
        let queued_at = Instant::now();
        let guard = ctx.locks[m.principal].lock().await;
        queue_ms = queued_at.elapsed().as_secs_f64() * 1000.0;
        Some(guard)
    } else {
        None
    };
    'request: {
        if def.serialize {
            if ctx.tainted.load(Ordering::Relaxed) {
                status = Status::Aborted;
                ctx.blocked_after_taint.fetch_add(1, Ordering::Relaxed);
                break 'request;
            }
            if Instant::now() >= deadline {
                timed_out = true;
                break 'request;
            }
        }
        if ctx.tainted.load(Ordering::Relaxed) {
            status = Status::Aborted;
            ctx.blocked_after_taint.fetch_add(1, Ordering::Relaxed);
            break 'request;
        }
        let keep = !matches!(spec.validator, Validator::None);
        let limits = Limits {
            total: 8 * 1024 * 1024,
            keep,
            keep_limit: if spec.validator.wants_bytes() {
                1024 * 1024
            } else {
                65536
            },
        };
        let body_503: Option<Vec<u8>>;
        let body: Vec<u8>;
        match &ctx.transport {
            Transport::H1(pools) => {
                let pool = &pools[ctx.runtime_of(m.principal)]
                    [(m.sequence % ctx.targets.len() as u64) as usize];
                dispatched = true;
                ctx.http_dispatch_attempts.fetch_add(1, Ordering::Relaxed);
                dispatch_at = Some(Instant::now());
                if let Some(b) = &m.prebuilt.body {
                    with_shard(|s| s.profile.request_body_bytes += b.len() as u64);
                }
                let exchange = async {
                    let mut conn = pool.take().await.map_err(|_| http1::Failure::Network)?;
                    m.prebuilt.h1_bytes(&mut conn.write, &target.1, address);
                    let (response, reusable) =
                        conn.exchange(spec.method == "HEAD", &limits).await?;
                    if reusable {
                        pool.give(conn);
                    }
                    Ok::<_, http1::Failure>(response)
                };
                match tokio::time::timeout_at(deadline.into(), exchange).await {
                    Err(_) => {
                        timed_out = true;
                        break 'request;
                    }
                    Ok(Err(http1::Failure::Network)) => break 'request,
                    Ok(Err(http1::Failure::Body {
                        status: code,
                        bytes: n,
                    })) => {
                        status = Status::Http(code);
                        bytes = n;
                        break 'request;
                    }
                    Ok(Ok(response)) => {
                        status = Status::Http(response.status);
                        bytes = response.bytes;
                        let coded = response
                            .encoding
                            .as_deref()
                            .is_some_and(|e| !e.is_empty() && e != "identity");
                        wire = if coded {
                            response.content_length
                        } else {
                            Some(response.bytes)
                        };
                        encoding = response.encoding;
                        body = response.body;
                        body_503 = response.body_503;
                    }
                }
            }
            Transport::H2(client) => {
                let slot = client
                    .acquire(target.0, client_index.unwrap_or(0), deadline.into())
                    .await;
                let Ok(mut slot) = slot else {
                    // a full stream limit waited past the timeout, or no connection
                    timed_out = Instant::now() >= deadline;
                    break 'request;
                };
                lag_extra = slot.queued_ms;
                lag_recorded = true;
                with_shard(|s| {
                    s.scheduling.sample(lag_base + lag_extra);
                    s.lag_hdr.record((lag_base + lag_extra) * 1000.0);
                });
                dispatched = true;
                ctx.http_dispatch_attempts.fetch_add(1, Ordering::Relaxed);
                dispatch_at = Some(Instant::now());
                if let Some(b) = &m.prebuilt.body {
                    with_shard(|s| s.profile.request_body_bytes += b.len() as u64);
                }
                let mut builder = http::Request::builder()
                    .method(spec.method)
                    .uri(format!("http://{}{}", target.1, spec.path));
                let headers = builder.headers_mut().unwrap();
                for (i, (k, v)) in m.prebuilt.h2_headers.iter().enumerate() {
                    if Some(i) == m.prebuilt.h2_forwarded_at {
                        headers.append(k.clone(), http::HeaderValue::from_str(address).unwrap());
                    } else {
                        headers.append(k.clone(), v.clone());
                    }
                }
                let request = builder.body(()).unwrap();
                let outcome = tokio::time::timeout_at(deadline.into(), async {
                    let response = client
                        .request(&mut slot, request, m.prebuilt.body.clone())
                        .await
                        .map_err(|broken| (broken, None))?;
                    let code = response.status;
                    let enc = response.encoding.clone();
                    let mut stream = response.stream;
                    let coded = enc
                        .as_deref()
                        .is_some_and(|e| !e.is_empty() && e != "identity");
                    let mut raw: Vec<u8> = Vec::new();
                    let mut wire_n = 0u64;
                    let mut n = 0u64;
                    let mut kept: Vec<u8> = Vec::new();
                    let mut e503: Option<Vec<u8>> =
                        if code == 503 { Some(Vec::new()) } else { None };
                    let push = |part: &[u8],
                                n: &mut u64,
                                kept: &mut Vec<u8>,
                                e503: &mut Option<Vec<u8>>|
                     -> bool {
                        *n += part.len() as u64;
                        if let Some(b) = e503 {
                            if *n <= 4096 {
                                b.extend_from_slice(part)
                            } else {
                                *e503 = None
                            }
                        }
                        if *n > limits.total as u64 {
                            return false;
                        }
                        if limits.keep {
                            if *n > limits.keep_limit as u64 {
                                return false;
                            }
                            kept.extend_from_slice(part);
                        }
                        true
                    };
                    while let Some(chunk) = http2::next_chunk(&mut stream).await {
                        let Ok(chunk) = chunk else {
                            return Err((false, Some((code, n))));
                        };
                        wire_n += chunk.len() as u64;
                        if coded {
                            raw.extend_from_slice(&chunk);
                            if raw.len() > limits.total {
                                client.local_cancel();
                                return Err((false, Some((code, n))));
                            }
                        } else if !push(&chunk, &mut n, &mut kept, &mut e503) {
                            client.local_cancel();
                            return Err((false, Some((code, n))));
                        }
                    }
                    if coded {
                        let decoded = http1::decode(enc.as_deref().unwrap_or(""), &raw)
                            .ok_or((false, Some((code, 0))))?;
                        for part in decoded.chunks(16 * 1024) {
                            if !push(part, &mut n, &mut kept, &mut e503) {
                                return Err((false, Some((code, n))));
                            }
                        }
                    }
                    Ok((code, enc, n, wire_n, kept, e503))
                })
                .await;
                match outcome {
                    Err(_) => {
                        client.local_cancel();
                        client.release(&slot, false);
                        timed_out = true;
                        break 'request;
                    }
                    Ok(Err((broken, partial))) => {
                        client.release(&slot, broken);
                        if let Some((code, n)) = partial {
                            status = Status::Http(code);
                            bytes = n;
                        }
                        break 'request;
                    }
                    Ok(Ok((code, enc, n, wire_n, kept, e503))) => {
                        client.release(&slot, false);
                        status = Status::Http(code);
                        bytes = n;
                        wire = Some(wire_n);
                        encoding = enc;
                        body = kept;
                        body_503 = e503;
                    }
                }
            }
        }
        if status == Status::Http(503) {
            reason_503 = classify_503(body_503.as_deref());
        }
        let Status::Http(code) = status else {
            break 'request;
        };
        ok = code == 200;
        if ok && code != 429 && keep {
            let at = Instant::now();
            ok = if spec.validator.wants_bytes() {
                spec.validator.check_bytes(&body)
            } else {
                // JSON.parse(Buffer.concat(chunks).toString('utf8'))
                let text = String::from_utf8_lossy(&body);
                match crate::jv::parse(&text) {
                    Some(value) => match spec.validator.check(&value) {
                        Verdict::Reject => false,
                        Verdict::Accept => true,
                        Verdict::Created(id) => {
                            created = Some(id);
                            true
                        }
                    },
                    None => false,
                }
            };
            validation_ms = Some(at.elapsed().as_secs_f64() * 1000.0);
        }
    }
    // finally: a client abort does not cancel a server mutation; fence the queue first
    if def.stateful && dispatched && !ok {
        ctx.tainted.store(true, Ordering::Relaxed);
        ctx.unknown_mutation_outcomes
            .fetch_add(1, Ordering::Relaxed);
        if bounded(m.principal) {
            ctx.tainted_principals.lock().unwrap().insert(m.principal);
        }
    }
    if let Some(id) = created {
        // onCreated: the validator accepted the save
        ctx.pool.lock().unwrap().push_back((m.principal, id));
    }
    drop(_guard);
    let completed = Instant::now();
    let elapsed = if ctx.late_dispatch {
        completed.saturating_duration_since(planned)
    } else {
        completed.saturating_duration_since(began)
    }
    .as_secs_f64()
        * 1000.0;
    let in_window = ok && status.success() && completed <= ctx.deadline;
    let arrival_ms = completed.saturating_duration_since(planned).as_secs_f64() * 1000.0;
    let lag_total = lag_base + lag_extra;
    let planned_second = (ctx.ms(planned) / 1000.0).floor().max(0.0) as usize;
    with_shard(|s| {
        if let Some(slot) = s.lag_max_by_second.get_mut(planned_second) {
            *slot = slot.max(lag_total);
        }
        if !lag_recorded {
            s.scheduling.sample(lag_base);
            s.lag_hdr.record(lag_base * 1000.0);
        } else if !h2 {
            s.scheduling.sample(lag_base);
            s.lag_hdr.record(lag_base * 1000.0);
        }
        if status == Status::Http(503) {
            s.e503.add(reason_503);
            s.e503_by[def.name].add(reason_503);
        }
        for t in [&mut s.lat] {
            t.queue.sample(queue_ms);
            if let Some(at) = dispatch_at {
                t.http
                    .sample(completed.saturating_duration_since(at).as_secs_f64() * 1000.0);
            }
            if let Some(v) = validation_ms {
                t.validation.sample(v);
            }
        }
        {
            let t = &mut s.lat_by[def.name];
            t.queue.sample(queue_ms);
            if let Some(at) = dispatch_at {
                t.http
                    .sample(completed.saturating_duration_since(at).as_secs_f64() * 1000.0);
            }
            if let Some(v) = validation_ms {
                t.validation.sample(v);
                s.validation_hdr.record(v * 1000.0);
            }
        }
        if ok && status.success() {
            s.profile.add_size(bytes);
            s.profile.add_compression(encoding.as_deref(), wire, bytes);
            if bounded(m.principal) {
                *s.principals.accepted.entry(m.principal).or_default() += 1;
            }
            if let Some(c) = client_index {
                *s.clients.accepted.entry(c).or_default() += 1;
            }
            if in_window {
                s.accepted_within += 1;
                let second = (ctx.ms(completed) / 1000.0).floor().max(0.0) as usize;
                let last = s.per_second.len().saturating_sub(1);
                if !s.per_second.is_empty() {
                    s.per_second[second.min(last)] += 1;
                }
                if bounded(m.principal) {
                    s.principals.within.insert(m.principal);
                }
                if let Some(c) = client_index {
                    s.clients.within.insert(c);
                }
            }
        }
        s.aggregate
            .add(elapsed, status, bytes, ok, timed_out, in_window);
        s.arrival
            .add(arrival_ms, status, bytes, ok, timed_out, in_window);
        s.by[def.name].add(elapsed, status, bytes, ok, timed_out, in_window);
        s.hdr.record(elapsed * 1000.0);
        s.hdr_by[def.name].record(elapsed * 1000.0);
    });
    if ctx.outstanding.fetch_sub(1, Ordering::AcqRel) == 1 {
        ctx.finished.notify_waiters();
    }
}

/// The scheduler wakes at most once per tick and dispatches every slot that came due
/// since, so a request costs one spawn, not one wake-up of the scheduler and a worker.
fn tick_sleep(duration: Duration, tick: Duration) {
    std::thread::sleep(duration.min(tick));
}
fn default_tick() -> u64 {
    400
}

struct Scheduled {
    offered: u64,
    dropped_capacity: u64,
    dropped_scheduler: u64,
    dropped_late: u64,
    aborted_unscheduled: u64,
}

pub fn run(plan: Option<PlanFile>, stage: StageSpec) -> Result<Value, String> {
    // ---- bounds (measure() and the campaign caps) ----
    if stage.concurrency < 1
        || stage.concurrency > MAX_OUTSTANDING
        || !stage.seconds.is_finite()
        || stage.seconds <= 0.0
        || stage.seconds > MAX_STAGE_SECONDS
        || !stage.timeout_ms.is_finite()
        || stage.timeout_ms < 1.0
        || stage.timeout_ms > 120000.0
    {
        return Err("Invalid bounded measurement parameters".into());
    }
    if !["drop", "dispatch"].contains(&stage.late_policy.as_str())
        || (stage.late_policy == "dispatch" && stage.arrival_rate.is_none())
    {
        return Err(
            "Late policy must be drop or dispatch, and dispatch needs an arrival rate".into(),
        );
    }
    if let Some(rate) = stage.arrival_rate {
        if !rate.is_finite() || rate <= 0.0 || rate > MAX_OFFERED_RPS {
            return Err("Invalid arrival rate".into());
        }
    }
    if let Some(h2) = &stage.h2 {
        h2.validate()?;
    }
    if !(50..=1000).contains(&stage.tick_micros) {
        return Err("tickMicros must be 50-1000".into());
    }
    let origins: Vec<String> = stage
        .bases
        .clone()
        .unwrap_or_else(|| vec![stage.base.clone()]);
    if origins.is_empty() || origins.len() > 64 {
        return Err("Invalid target rotation list".into());
    }
    let mut targets = Vec::new();
    for origin in &origins {
        let target = parse_target(origin, &stage.allowed_targets)?;
        if targets
            .iter()
            .any(|(a, _): &(SocketAddr, String)| *a == target.0)
        {
            return Err("Duplicate target".into());
        }
        targets.push(target);
    }
    let threads = stage.threads.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map_or(1, |n| n.get())
            .min(8)
    });
    if threads < 1 || threads > MAX_THREADS {
        return Err(format!("Generator threads must be 1-{MAX_THREADS}"));
    }
    let per_core = match stage.runtime.as_str() {
        "multi" => false,
        "per-core" => true,
        _ => return Err("runtime must be multi or per-core".into()),
    };

    // ---- templates ----
    let (principals, slots, names, prebuilt, first_headers, simulated, rate_check) =
        match (&plan, &stage.plain) {
            (_, Some(plain)) => {
                if plain.is_empty() || plain.len() > 100 {
                    return Err("Invalid plain request list".into());
                }
                let mut names: Vec<String> = Vec::new();
                let mut slots = Vec::new();
                let mut row = Vec::new();
                for (i, r) in plain.iter().enumerate() {
                    let name = match names.iter().position(|n| *n == r.name) {
                        Some(at) => at,
                        None => {
                            names.push(r.name.clone());
                            names.len() - 1
                        }
                    };
                    let validator = match r.validate.as_deref() {
                        None | Some("none") => Validator::None,
                        Some("json") => Validator::AnyJson,
                        Some("ok-true") => Validator::OkTrue,
                        Some(other) => return Err(format!("Unknown plain validator {other}")),
                    };
                    let method: &'static str = match r.method.as_deref().unwrap_or("GET") {
                        "GET" => "GET",
                        "POST" => "POST",
                        _ => return Err("plain method must be GET or POST".into()),
                    };
                    let spec = Spec {
                        name: r.name.clone(),
                        method,
                        path: r.path.clone(),
                        headers: r.headers.clone(),
                        body: r.body.clone(),
                        validator,
                    };
                    row.push(Arc::new(prebuild(spec, &stage.headers, &[], false)));
                    slots.push(SlotDef {
                        unique: i,
                        name,
                        serialize: false,
                        stateful: false,
                        discard: false,
                    });
                }
                let principal = Principal {
                    index: 0,
                    account_id: String::new(),
                    owner_account_id: None,
                    message_ids: Vec::new(),
                    inbox_id: None,
                    archive_id: None,
                    trash_id: None,
                    drafts_id: None,
                    attachment_blob_id: None,
                    body_sample_bytes: None,
                    attachment_sample_bytes: None,
                    planted_quarterly: None,
                    headers: Vec::new(),
                    addresses: Vec::new(),
                    client_indexes: Vec::new(),
                };
                (
                    vec![principal],
                    slots,
                    names,
                    vec![row],
                    Vec::new(),
                    false,
                    None,
                )
            }
            (Some(plan), None) => {
                if plan.schema != 1 {
                    return Err("Unsupported plan schema".into());
                }
                let principals = crate::plan::principals(plan)?;
                let group = plan
                    .groups
                    .get(&stage.endpoint)
                    .ok_or("Unknown endpoint group")?;
                if group.is_empty() || group.len() > 100 {
                    return Err("Invalid bounded measurement parameters".into());
                }
                let native = plan.engine_mode == "native";
                let mut unique: Vec<String> = Vec::new();
                let mut names: Vec<String> = Vec::new();
                let mut slots = Vec::new();
                for entry in group {
                    let u = match unique.iter().position(|n| *n == entry.name) {
                        Some(at) => at,
                        None => {
                            unique.push(entry.name.clone());
                            unique.len() - 1
                        }
                    };
                    let name = match names.iter().position(|n| *n == entry.name) {
                        Some(at) => at,
                        None => {
                            names.push(entry.name.clone());
                            names.len() - 1
                        }
                    };
                    slots.push(SlotDef {
                        unique: u,
                        name,
                        serialize: entry.serialize,
                        stateful: entry.stateful,
                        discard: entry.name == "draft-discard",
                    });
                }
                let simulated = plan.client_simulation.is_some();
                let mut first_headers = principals[0].headers.clone();
                if simulated {
                    first_headers
                        .push(("x-forwarded-for".into(), principals[0].addresses[0].clone()));
                }
                let mut prebuilt = Vec::with_capacity(principals.len());
                for p in &principals {
                    let mut client = p.headers.clone();
                    if simulated {
                        client.push(("x-forwarded-for".into(), p.addresses[0].clone()));
                    }
                    let mut row = Vec::with_capacity(unique.len());
                    for name in &unique {
                        let spec = templates::build(
                            name,
                            p,
                            &plan.constants,
                            plan.representative,
                            native,
                        )?
                        .ok_or_else(|| format!("Goal mix shape has no template: {name}"))?;
                        row.push(Arc::new(prebuild(spec, &first_headers, &client, simulated)));
                    }
                    prebuilt.push(row);
                }
                (
                    principals,
                    slots,
                    names,
                    prebuilt,
                    first_headers,
                    simulated,
                    Some(plan.rate_limits.clone()),
                )
            }
            (None, None) => return Err("A plan or plain requests are required".into()),
        };

    // ---- safety rail (plan N11): planned per-client and per-account rate under 80% of the limits ----
    let users = principals.len() as f64;
    let per_user_clients = principals
        .iter()
        .map(|p| p.addresses.len().max(1))
        .max()
        .unwrap_or(1) as f64;
    let mut guard = json!({"applied": false});
    if let (Some(limits), Some(rate)) = (&rate_check, stage.arrival_rate) {
        let client_limit = limits.client_requests / limits.rate_window_secs;
        let account_limit = limits.account_requests / limits.rate_window_secs;
        let per_account = rate / users;
        let per_client = per_account / per_user_clients;
        guard = json!({"applied": stage.rate_guard, "limitsSource": limits.source, "clientLimitPerSecond": client_limit, "accountLimitPerSecond": account_limit,
            "plannedPerClientPerSecond": per_client, "plannedPerAccountPerSecond": per_account, "clientsPerUser": per_user_clients, "refusalShare": 0.8});
        if stage.rate_guard
            && (per_client > 0.8 * client_limit || per_account > 0.8 * account_limit)
        {
            return Err(format!(
                "Refused: the planned rate is {per_client:.2} requests/s per client address and {per_account:.2} per account; the configured limits are {client_limit:.2} and {account_limit:.2} (refusal above 80%). Use more users or client addresses."
            ));
        }
    }

    let seconds_len = stage.seconds.ceil() as usize;
    DIMS.set((names.len(), seconds_len))
        .map_err(|_| "stage already ran in this process")?;

    // ---- runtimes ----
    let runtimes: Vec<tokio::runtime::Runtime> = if per_core {
        (0..threads)
            .map(|i| {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .thread_name(format!("mw-loadgen-{i}"))
                    .build()
                    .unwrap()
            })
            .collect()
    } else {
        vec![
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(threads)
                .enable_all()
                .thread_name("mw-loadgen")
                .build()
                .unwrap(),
        ]
    };
    let handles: Vec<tokio::runtime::Handle> =
        runtimes.iter().map(|r| r.handle().clone()).collect();
    // current-thread runtimes need a thread each that drives them
    let mut drivers = Vec::new();
    let runtimes = Arc::new(runtimes);
    if per_core {
        for i in 0..threads {
            let rts = runtimes.clone();
            let (tx, rx) = tokio::sync::oneshot::channel::<()>();
            drivers.push((
                std::thread::spawn(move || {
                    rts[i].block_on(async move {
                        let _ = rx.await;
                    });
                }),
                tx,
            ));
        }
    }
    let transport = match &stage.h2 {
        Some(cfg) => {
            let client = Arc::new(http2::Client::new(cfg.clone()));
            let addrs: Vec<SocketAddr> = targets.iter().map(|t| t.0).collect();
            let warm = client.clone();
            handles[0].block_on(async move { warm.warm(&addrs).await });
            Transport::H2(client)
        }
        None => Transport::H1(
            (0..runtimes.len())
                .map(|_| targets.iter().map(|t| http1::Pool::new(t.0)).collect())
                .collect(),
        ),
    };

    // ---- warm keep-alive connections (outside the window) ----
    let mut prewarmed = 0usize;
    if let Transport::H1(pools) = &transport {
        let wanted = stage
            .prewarm_connections
            .unwrap_or(stage.concurrency.min(512))
            .min(stage.concurrency)
            .min(MAX_OUTSTANDING);
        let per = wanted.div_ceil(pools.len().max(1));
        for (rt, row) in pools.iter().enumerate() {
            for pool in row {
                let addr = pool.addr;
                let opened: Vec<http1::Conn> = handles[rt].block_on(async move {
                    let mut tasks = Vec::new();
                    for _ in 0..per {
                        tasks.push(tokio::spawn(http1::Conn::connect(addr)));
                    }
                    let mut out = Vec::new();
                    for t in tasks {
                        if let Ok(Ok(c)) = t.await {
                            out.push(c);
                        }
                    }
                    out
                });
                prewarmed += opened.len();
                for c in opened {
                    pool.give(c);
                }
            }
        }
    }
    // ---- start ----
    let wall_now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
        * 1000.0;
    let mono_now = Instant::now();
    let start = match stage.common_start_at_ms {
        Some(at) => {
            if (at - wall_now).abs() > 60000.0 {
                return Err("Common fixture start must be within sixty seconds".into());
            }
            if at >= wall_now {
                mono_now + Duration::from_secs_f64((at - wall_now) / 1000.0)
            } else {
                mono_now
                    .checked_sub(Duration::from_secs_f64((wall_now - at) / 1000.0))
                    .unwrap_or(mono_now)
            }
        }
        None => mono_now,
    };
    let deadline = start + Duration::from_secs_f64(stage.seconds);
    let late_dispatch = stage.late_policy == "dispatch";
    let ctx = Arc::new(Ctx {
        start,
        deadline,
        late_dispatch,
        timeout: Duration::from_secs_f64(stage.timeout_ms / 1000.0),
        seconds: stage.seconds,
        locks: (0..principals.len())
            .map(|_| tokio::sync::Mutex::new(()))
            .collect(),
        rotation: (0..principals.len()).map(|_| AtomicU64::new(0)).collect(),
        slots,
        principals,
        first_headers,
        simulated,
        prebuilt,
        pool: Mutex::new(
            stage
                .draft_pool
                .iter()
                .map(|d| (d.principal_index, d.id.clone()))
                .collect(),
        ),
        tainted: AtomicBool::new(false),
        tainted_principals: Mutex::new(HashSet::new()),
        unknown_mutation_outcomes: AtomicU64::new(0),
        blocked_after_taint: AtomicU64::new(0),
        http_dispatch_attempts: AtomicU64::new(0),
        late_slots_dispatched: AtomicU64::new(0),
        outstanding: AtomicUsize::new(0),
        peak_outstanding: AtomicUsize::new(0),
        finished: tokio::sync::Notify::new(),
        sequence: AtomicU64::new(0),
        transport,
        targets,
        runtimes: handles.len(),
        per_core,
        sends: AtomicU64::new(0),
    });
    for p in &ctx.pool.lock().unwrap().iter().collect::<Vec<_>>() {
        if p.0 >= ctx.principals.len() {
            return Err("Draft pool entry names an unknown principal".into());
        }
    }
    while Instant::now() < start {
        std::thread::sleep((start - Instant::now()).min(Duration::from_millis(50)));
    }
    let actual_start = Instant::now();
    let cpu_start = crate::sys::cpu_micros();
    let rss_before = crate::sys::rss_bytes();

    let mut scheduled: Option<Scheduled> = None;
    match stage.arrival_rate {
        Some(rate) => {
            let interval = 1000.0 / rate;
            let total = (stage.seconds * rate).floor() as u64;
            let tick = Duration::from_micros(stage.tick_micros);
            let mut s = Scheduled {
                offered: 0,
                dropped_capacity: 0,
                dropped_scheduler: 0,
                dropped_late: 0,
                aborted_unscheduled: 0,
            };
            let mut slot: u64 = 0;
            let concurrency = stage.concurrency;
            while slot < total {
                if ctx.tainted.load(Ordering::Relaxed) {
                    s.aborted_unscheduled = total - slot;
                    break;
                }
                let now = Instant::now();
                let now_ms = ctx.ms(now);
                let due = slot as f64 * interval;
                if now_ms < due {
                    tick_sleep(Duration::from_secs_f64((due - now_ms) / 1000.0), tick);
                    continue;
                }
                if now >= deadline {
                    let missed = total - slot;
                    s.dropped_scheduler += missed;
                    s.offered += missed;
                    break;
                }
                let oldest = if late_dispatch {
                    slot
                } else {
                    total.min(slot.max(((now_ms - 100.0) / interval).ceil().max(0.0) as u64))
                };
                if oldest > slot {
                    s.dropped_late += oldest - slot;
                    s.dropped_scheduler += oldest - slot;
                    s.offered += oldest - slot;
                    slot = oldest;
                }
                let due_end = total.min((now_ms / interval).floor() as u64 + 1);
                let room =
                    concurrency.saturating_sub(ctx.outstanding.load(Ordering::Acquire)) as u64;
                let dispatch = due_end.saturating_sub(slot).min(room);
                let end = if late_dispatch {
                    slot + dispatch
                } else {
                    due_end.max(slot)
                };
                s.dropped_capacity += end - slot - dispatch;
                s.offered += end - slot;
                for i in 0..dispatch {
                    let planned =
                        start + Duration::from_secs_f64((slot + i) as f64 * interval / 1000.0);
                    let sequence = ctx.sequence.fetch_add(1, Ordering::Relaxed);
                    let m = ctx.materialize(sequence);
                    let now_out = ctx.outstanding.fetch_add(1, Ordering::AcqRel) + 1;
                    ctx.peak_outstanding.fetch_max(now_out, Ordering::Relaxed);
                    handles[ctx.runtime_of(m.principal)].spawn(send(ctx.clone(), m, planned));
                }
                slot = end;
                if slot < total {
                    tick_sleep(tick, tick);
                }
            }
            scheduled = Some(s);
            // await Promise.all(pending)
            handles[0].block_on(async {
                loop {
                    let notified = ctx.finished.notified();
                    if ctx.outstanding.load(Ordering::Acquire) == 0 {
                        break;
                    }
                    let _ = tokio::time::timeout(Duration::from_millis(5), notified).await;
                }
            });
            while !ctx.tainted.load(Ordering::Relaxed) && Instant::now() < deadline {
                std::thread::sleep((deadline - Instant::now()).min(Duration::from_millis(50)));
            }
        }
        None => {
            // closed loop: `concurrency` senders, each sending until the deadline
            let remaining = Arc::new(AtomicUsize::new(stage.concurrency));
            let done = Arc::new(tokio::sync::Notify::new());
            for worker in 0..stage.concurrency {
                let ctx = ctx.clone();
                let remaining = remaining.clone();
                let done = done.clone();
                handles[worker % handles.len()].spawn(async move {
                    while !ctx.tainted.load(Ordering::Relaxed) && Instant::now() < ctx.deadline {
                        let sequence = ctx.sequence.fetch_add(1, Ordering::Relaxed);
                        let m = ctx.materialize(sequence);
                        let now_out = ctx.outstanding.fetch_add(1, Ordering::AcqRel) + 1;
                        ctx.peak_outstanding.fetch_max(now_out, Ordering::Relaxed);
                        send(ctx.clone(), m, Instant::now()).await;
                    }
                    if remaining.fetch_sub(1, Ordering::AcqRel) == 1 {
                        done.notify_waiters();
                    }
                });
            }
            handles[0].block_on(async {
                loop {
                    let notified = done.notified();
                    if remaining.load(Ordering::Acquire) == 0 {
                        break;
                    }
                    let _ = tokio::time::timeout(Duration::from_millis(5), notified).await;
                }
            });
        }
    }
    let completed_at = Instant::now();
    let cpu_end = crate::sys::cpu_micros();
    let rss_after = crate::sys::rss_bytes();
    let elapsed = completed_at.saturating_duration_since(start).as_secs_f64();
    let cpu_elapsed = if stage.common_start_at_ms.is_none() {
        elapsed
    } else {
        completed_at
            .saturating_duration_since(actual_start)
            .as_secs_f64()
            .max(0.000001)
    };
    let h2_report = match &ctx.transport {
        Transport::H2(client) => Some(client.report()),
        Transport::H1(_) => None,
    };
    let h1_report = match &ctx.transport {
        Transport::H1(pools) => {
            let opened: u64 = pools
                .iter()
                .flatten()
                .map(|p| p.opened.load(Ordering::Relaxed))
                .sum();
            let reused: u64 = pools
                .iter()
                .flatten()
                .map(|p| p.reused.load(Ordering::Relaxed))
                .sum();
            Some(json!({"connectionsOpened": opened, "connectionReuses": reused}))
        }
        Transport::H2(_) => None,
    };
    for (handle, tx) in drivers {
        let _ = tx.send(());
        let _ = handle.join();
    }

    // ---- merge ----
    let mut total = Shard::new(names.len(), seconds_len);
    for shard in SHARDS.lock().unwrap().iter() {
        total.merge(&shard.lock().unwrap());
    }
    let user_cpu = cpu_end.0.saturating_sub(cpu_start.0);
    let system_cpu = cpu_end.1.saturating_sub(cpu_start.1);
    let cpu_us = (user_cpu + system_cpu) as f64;
    let requests = total.aggregate.count;
    if let Some(path) = &stage.draft_pool_out {
        let pool: Vec<Value> = ctx
            .pool
            .lock()
            .unwrap()
            .iter()
            .map(|(p, id)| json!({"principalIndex": p, "id": id}))
            .collect();
        crate::write_private(path, &serde_json::to_vec(&pool).unwrap())?;
    }
    let by_name = |f: &dyn Fn(usize) -> Value| -> serde_json::Map<String, Value> {
        names
            .iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), f(i)))
            .collect()
    };
    let lag_p99 = total.scheduling.percentile(0.99) as f64;
    let mut out = json!({
        "schema": 1,
        "generator": "native",
        "generatorVersion": env!("CARGO_PKG_VERSION"),
        "names": names,
        "raw": {
            "schema": 1,
            "aggregate": total.aggregate.snapshot(),
            "byRequest": by_name(&|i| total.by[i].snapshot()),
            "scheduling": total.scheduling.snapshot(),
            "arrivalLatency": total.arrival.snapshot(),
            "bytesProfile": total.profile.snapshot(),
            "http503Reasons": {"schema": 1, "aggregate": total.e503.snapshot(), "byRequest": by_name(&|i| total.e503_by[i].snapshot())},
            "latencyBreakdown": {"schema": 1, "aggregate": total.lat.snapshot(), "byRequest": by_name(&|i| total.lat_by[i].snapshot())},
        },
        "participantDistribution": {
            "principals": {"attempted": distribution(&total.principals.attempted), "accepted": distribution(&total.principals.accepted)},
            "simulatedClients": {"attempted": distribution(&total.clients.attempted), "accepted": distribution(&total.clients.accepted)},
        },
        "uniquePrincipalsAccepted": total.principals.accepted.len(),
        "uniquePrincipalsAttempted": total.principals.attempted.len(),
        "uniquePrincipalsAcceptedDuringScheduledWindow": total.principals.within.len(),
        "uniqueClientsAttempted": total.clients.attempted.len(),
        "uniqueClientsAccepted": total.clients.accepted.len(),
        "uniqueClientsAcceptedDuringScheduledWindow": total.clients.within.len(),
        "acceptedDuringScheduledWindow": total.accepted_within,
        "acceptedPerSecond": total.per_second,
        "mutationStateTainted": ctx.tainted.load(Ordering::Relaxed),
        "unknownMutationOutcomes": ctx.unknown_mutation_outcomes.load(Ordering::Relaxed),
        "taintedPrincipals": ctx.tainted_principals.lock().unwrap().len(),
        "httpDispatchAttempts": ctx.http_dispatch_attempts.load(Ordering::Relaxed),
        "blockedAfterMutationTaint": ctx.blocked_after_taint.load(Ordering::Relaxed),
        "abortedUnscheduled": scheduled.as_ref().map_or(0, |s| s.aborted_unscheduled),
        "peakOutstanding": ctx.peak_outstanding.load(Ordering::Relaxed),
        "lateSlotsDispatched": ctx.late_slots_dispatched.load(Ordering::Relaxed),
        "elapsedSeconds": elapsed,
        "cpuElapsedSeconds": cpu_elapsed,
        "cpu": {"userMs": user_cpu as f64 / 1000.0, "systemMs": system_cpu as f64 / 1000.0},
        "rssBeforeBytes": rss_before,
        "rssAfterBytes": rss_after,
        "protocol": if stage.h2.is_some() { "h2c" } else { "http/1.1" },
        "native": {
            "threads": threads,
            "runtime": if per_core { "per-core current-thread runtimes, principals affine to one runtime" } else { "tokio multi-thread runtime" },
            "http1Client": if stage.h2.is_none() { Value::from("raw keep-alive client (one connection per in-flight request, LIFO idle pool)") } else { Value::Null },
            "processors": crate::sys::processors(),
            "cpuMicrosPerRequest": if requests > 0 { cpu_us / requests as f64 } else { 0.0 },
            "cpuLogicalCores": cpu_us / (cpu_elapsed * 1e6),
            "requestsPerCpuSecond": if cpu_us > 0.0 { requests as f64 / (cpu_us / 1e6) } else { 0.0 },
            "latencyUs": total.hdr.report(),
            "latencyUsByRequest": by_name(&|i| total.hdr_by[i].report()),
            "schedulingLagUs": total.lag_hdr.report(),
            "validationUs": total.validation_hdr.report(),
            "schedulingLagLimitMs": SCHEDULING_LAG_LIMIT_MS,
            "schedulerTickMicros": stage.tick_micros,
            "prewarmedConnections": prewarmed,
            "schedulingLagMaxMsBySecond": total.lag_max_by_second,
            "rateGuard": guard,
            "h1": h1_report.unwrap_or(Value::Null),
        },
    });
    if let Some(h2) = h2_report {
        out["http2"] = h2;
    }
    if let Some(s) = scheduled {
        let o = out.as_object_mut().unwrap();
        o.insert("offered".into(), json!(s.offered));
        o.insert("droppedCapacity".into(), json!(s.dropped_capacity));
        o.insert("droppedScheduler".into(), json!(s.dropped_scheduler));
        o.insert("droppedLateSlots".into(), json!(s.dropped_late));
        o.insert(
            "generatorLimited".into(),
            json!(lag_p99 > SCHEDULING_LAG_LIMIT_MS),
        );
    }
    if let Some(at) = stage.common_start_at_ms {
        let wall = |i: Instant| {
            wall_now
                + (i.saturating_duration_since(mono_now).as_secs_f64()
                    - mono_now.saturating_duration_since(i).as_secs_f64())
                    * 1000.0
        };
        out["window"] = json!({"requestedStartAtMs": at, "scheduledStartAtMs": at, "scheduledEndAtMs": at + stage.seconds * 1000.0,
            "actualStartAtMs": wall(actual_start).round(), "completedAtMs": wall(completed_at).round(), "startLagMs": actual_start.saturating_duration_since(start).as_secs_f64() * 1000.0});
    }
    let _ = ctx.seconds;
    let _ = ctx.sends.load(Ordering::Relaxed);
    // runtimes are dropped here; keep-alive sockets close with them
    Ok(out)
}

/// Builds the templates of the first `count` principals for every shape of a group (`mw-loadgen templates`).
pub fn dump_templates(
    plan: &PlanFile,
    endpoint: &str,
    count: usize,
    host: &str,
) -> Result<Value, String> {
    let principals = crate::plan::principals(plan)?;
    let group = plan.groups.get(endpoint).ok_or("Unknown endpoint group")?;
    let simulated = plan.client_simulation.is_some();
    let mut first = principals[0].headers.clone();
    if simulated {
        first.push(("x-forwarded-for".into(), principals[0].addresses[0].clone()));
    }
    let mut out = Vec::new();
    for p in principals.iter().take(count) {
        let mut client = p.headers.clone();
        if simulated {
            client.push(("x-forwarded-for".into(), p.addresses[0].clone()));
        }
        let mut seen = HashSet::new();
        for entry in group {
            if !seen.insert(entry.name.clone()) {
                continue;
            }
            let spec = templates::build(
                &entry.name,
                p,
                &plan.constants,
                plan.representative,
                plan.engine_mode == "native",
            )?
            .ok_or_else(|| format!("no template: {}", entry.name))?;
            let pre = prebuild(spec, &first, &client, simulated);
            let text = String::from_utf8_lossy(
                &pre.h1_text(host, p.addresses.first().map(String::as_str).unwrap_or("")),
            )
            .into_owned();
            out.push(json!({"principalIndex": p.index, "name": entry.name, "serialize": entry.serialize, "stateful": entry.stateful, "http1": text}));
        }
        // the discard of a known id (dynamic template)
        if group.iter().any(|g| g.name == "draft-discard") {
            let pre = prebuild(
                templates::draft_discard(p, "DRAFT-ID"),
                &first,
                &client,
                simulated,
            );
            out.push(json!({"principalIndex": p.index, "name": "draft-discard(id)", "http1": String::from_utf8_lossy(&pre.h1_text(host, p.addresses.first().map(String::as_str).unwrap_or(""))).into_owned()}));
        }
    }
    Ok(Value::Array(out))
}

/// Validates recorded responses (`mw-loadgen validate`): one verdict per corpus entry.
pub fn validate_corpus(plan: &PlanFile, entries: &[Value]) -> Result<Value, String> {
    let principals = crate::plan::principals(plan)?;
    let native = plan.engine_mode == "native";
    let mut verdicts = Vec::new();
    for entry in entries {
        let index = entry["principalIndex"]
            .as_u64()
            .ok_or("corpus entry needs principalIndex")? as usize;
        let name = entry["name"].as_str().ok_or("corpus entry needs name")?;
        let p = principals
            .get(index)
            .ok_or("corpus principal out of range")?;
        let spec = match entry.get("draftId").and_then(Value::as_str) {
            Some(id) => templates::draft_discard(p, id),
            None => templates::build(name, p, &plan.constants, plan.representative, native)?
                .ok_or("unknown shape")?,
        };
        let status = entry["status"].as_u64().unwrap_or(200) as u16;
        let body: Vec<u8> = match (
            entry.get("bodyBase64").and_then(Value::as_str),
            entry.get("body").and_then(Value::as_str),
        ) {
            (Some(b64), _) => crate::base64_decode(b64).ok_or("bad base64")?,
            (None, Some(text)) => text.as_bytes().to_vec(),
            _ => Vec::new(),
        };
        let mut created = Value::Null;
        let ok = status == 200
            && if spec.validator.wants_bytes() {
                spec.validator.check_bytes(&body)
            } else {
                match crate::jv::parse(&String::from_utf8_lossy(&body)) {
                    Some(value) => match spec.validator.check(&value) {
                        Verdict::Reject => false,
                        Verdict::Accept => true,
                        Verdict::Created(id) => {
                            created = json!(id);
                            true
                        }
                    },
                    None => false,
                }
            };
        verdicts.push(json!({"ok": ok, "created": created}));
    }
    Ok(Value::Array(verdicts))
}

/// Times JSON parsing and validation per corpus entry (`mw-loadgen bench-validate`); diagnostics only.
pub fn bench_validate(
    plan: &PlanFile,
    entries: &[Value],
    iterations: usize,
) -> Result<Value, String> {
    let principals = crate::plan::principals(plan)?;
    let mut rows = Vec::new();
    for entry in entries {
        let index = entry["principalIndex"].as_u64().unwrap_or(0) as usize;
        let name = entry["name"].as_str().unwrap_or("");
        let Some(p) = principals.get(index) else {
            continue;
        };
        let spec = match entry.get("draftId").and_then(Value::as_str) {
            Some(id) => templates::draft_discard(p, id),
            None => match templates::build(name, p, &plan.constants, plan.representative, true)? {
                Some(s) => s,
                None => continue,
            },
        };
        if spec.validator.wants_bytes() {
            continue;
        }
        let body =
            crate::base64_decode(entry["bodyBase64"].as_str().unwrap_or("")).unwrap_or_default();
        let t = Instant::now();
        let mut ok = 0;
        for _ in 0..iterations {
            let text = String::from_utf8_lossy(&body);
            let v = crate::jv::parse(&text).unwrap();
            if !matches!(spec.validator.check(&v), Verdict::Reject) {
                ok += 1;
            }
        }
        let total = t.elapsed().as_secs_f64() * 1e6 / iterations as f64;
        let text = String::from_utf8_lossy(&body);
        let t = Instant::now();
        for _ in 0..iterations {
            let _v = crate::jv::parse(&text).unwrap();
        }
        let parse = t.elapsed().as_secs_f64() * 1e6 / iterations as f64;
        rows.push(json!({"name": name, "bytes": body.len(), "parseUs": parse, "parseAndCheckUs": total, "accepted": ok == iterations}));
    }
    Ok(Value::Array(rows))
}
