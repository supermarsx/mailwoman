//! HTTP/2 cleartext (prior knowledge) client with the connection models of
//! http-h2-client.mjs: `browser-like` (a pool per simulated client) and
//! `proxy-like` (a few shared connections per target). When every connection is
//! at its stream limit the request waits in the generator (FIFO) and the wait is
//! scheduling lag, as in Node. TLS is not implemented here.
use bytes::Bytes;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::net::TcpStream;
use tokio::sync::oneshot;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct H2Config {
    pub mode: String,
    pub preset: String,
    pub per_client: bool,
    pub connections: usize,
    pub max_streams: usize,
}
impl H2Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.mode != "h2c" {
            return Err("The native generator supports HTTP/2 mode h2c only".into());
        }
        if !(1..=64).contains(&self.connections) || !(1..=1000).contains(&self.max_streams) {
            return Err("Invalid HTTP/2 connection model".into());
        }
        Ok(())
    }
}

struct Conn {
    send: Option<h2::client::SendRequest<Bytes>>,
    active: usize,
    total: u64,
    limit: usize,
    dead: bool,
}

#[derive(Default)]
struct PoolState {
    conns: Vec<Conn>,
    waiters: VecDeque<oneshot::Sender<usize>>,
}

struct Pool {
    addr: SocketAddr,
    state: Mutex<PoolState>,
    connecting: tokio::sync::Mutex<()>,
}

#[derive(Default)]
struct Stats {
    opened: AtomicU64,
    closed: AtomicU64,
    goaway: AtomicU64,
    reconnects: AtomicU64,
    connect_errors: AtomicU64,
    refused: AtomicU64,
    local_cancels: AtomicU64,
    streams: AtomicU64,
    peak: AtomicUsize,
    inflight: AtomicUsize,
    queued: AtomicU64,
    queue_wait_us: AtomicU64,
    queue_wait_max_us: AtomicU64,
    server_max: AtomicUsize,
    rst: Mutex<HashMap<u32, u64>>,
}

pub struct Client {
    cfg: H2Config,
    pools: Mutex<HashMap<(SocketAddr, usize), Arc<Pool>>>,
    stats: Stats,
    reconnects_left: AtomicU64,
}

pub struct Slot {
    pool: Arc<Pool>,
    index: usize,
    send: h2::client::SendRequest<Bytes>,
    pub queued_ms: f64,
}

pub struct Response {
    pub status: u16,
    pub encoding: Option<String>,
    pub stream: h2::RecvStream,
}

impl Client {
    pub fn new(cfg: H2Config) -> Client {
        Client {
            cfg,
            pools: Mutex::new(HashMap::new()),
            stats: Stats {
                server_max: AtomicUsize::new(usize::MAX),
                ..Default::default()
            },
            reconnects_left: AtomicU64::new(1000),
        }
    }

    fn pool(&self, addr: SocketAddr, client: usize) -> Arc<Pool> {
        let key = (addr, if self.cfg.per_client { client } else { 0 });
        let mut pools = self.pools.lock().unwrap();
        pools
            .entry(key)
            .or_insert_with(|| {
                Arc::new(Pool {
                    addr,
                    state: Mutex::new(PoolState::default()),
                    connecting: tokio::sync::Mutex::new(()),
                })
            })
            .clone()
    }

    async fn open(&self, addr: SocketAddr) -> Option<(h2::client::SendRequest<Bytes>, usize)> {
        self.stats.opened.fetch_add(1, Ordering::Relaxed);
        let tcp = match TcpStream::connect(addr).await {
            Ok(tcp) => tcp,
            Err(_) => {
                self.stats.connect_errors.fetch_add(1, Ordering::Relaxed);
                self.stats.closed.fetch_add(1, Ordering::Relaxed);
                return None;
            }
        };
        let _ = tcp.set_nodelay(true);
        let (send, connection) = match h2::client::Builder::new()
            .enable_push(false)
            .handshake::<_, Bytes>(tcp)
            .await
        {
            Ok(pair) => pair,
            Err(_) => {
                self.stats.connect_errors.fetch_add(1, Ordering::Relaxed);
                self.stats.closed.fetch_add(1, Ordering::Relaxed);
                return None;
            }
        };
        tokio::spawn(async move {
            let _ = connection.await;
        });
        // Ready = the server's SETTINGS are known, so its stream limit holds from the first stream.
        let send = match send.ready().await {
            Ok(send) => send,
            Err(_) => {
                self.stats.connect_errors.fetch_add(1, Ordering::Relaxed);
                self.stats.closed.fetch_add(1, Ordering::Relaxed);
                return None;
            }
        };
        let server = send.current_max_send_streams();
        self.stats.server_max.fetch_min(server, Ordering::Relaxed);
        Some((send.clone(), self.cfg.max_streams.min(server.max(1))))
    }

    async fn fill(&self, pool: &Pool) {
        let _guard = pool.connecting.lock().await;
        loop {
            let (_live, missing) = {
                let state = pool.state.lock().unwrap();
                let live = state.conns.iter().filter(|c| !c.dead).count();
                (live, live < self.cfg.connections)
            };
            if !missing {
                return;
            }
            let replacing = pool.state.lock().unwrap().conns.iter().any(|c| c.dead);
            if replacing {
                if self.reconnects_left.load(Ordering::Relaxed) == 0 {
                    return;
                }
                self.reconnects_left.fetch_sub(1, Ordering::Relaxed);
                self.stats.reconnects.fetch_add(1, Ordering::Relaxed);
            }
            let Some((send, limit)) = self.open(pool.addr).await else {
                return;
            };
            let mut state = pool.state.lock().unwrap();
            if let Some(dead) = state.conns.iter().position(|c| c.dead && c.active == 0) {
                let old = std::mem::replace(
                    &mut state.conns[dead],
                    Conn {
                        send: Some(send),
                        active: 0,
                        total: 0,
                        limit,
                        dead: false,
                    },
                );
                drop(old);
            } else {
                state.conns.push(Conn {
                    send: Some(send),
                    active: 0,
                    total: 0,
                    limit,
                    dead: false,
                });
            }
            Self::wake(&mut state, &self.stats);
        }
    }

    fn pick(state: &PoolState) -> Option<usize> {
        let mut best: Option<usize> = None;
        for (i, c) in state.conns.iter().enumerate() {
            if !c.dead
                && c.active < c.limit
                && best.is_none_or(|b| c.active < state.conns[b].active)
            {
                best = Some(i);
            }
        }
        best
    }
    fn reserve(state: &mut PoolState, index: usize, stats: &Stats) {
        let c = &mut state.conns[index];
        c.active += 1;
        c.total += 1;
        stats.streams.fetch_add(1, Ordering::Relaxed);
        let now = stats.inflight.fetch_add(1, Ordering::Relaxed) + 1;
        stats.peak.fetch_max(now, Ordering::Relaxed);
    }
    fn wake(state: &mut PoolState, stats: &Stats) {
        while !state.waiters.is_empty() {
            let Some(index) = Self::pick(state) else {
                return;
            };
            let waiter = state.waiters.pop_front().unwrap();
            Self::reserve(state, index, stats);
            if waiter.send(index).is_err() {
                // the waiter gave up (timeout): undo
                state.conns[index].active -= 1;
                stats.inflight.fetch_sub(1, Ordering::Relaxed);
            }
        }
    }

    /// Opens the shared connections before the measured window (proxy-like).
    pub async fn warm(&self, addrs: &[SocketAddr]) {
        if self.cfg.per_client {
            return;
        }
        for addr in addrs {
            let pool = self.pool(*addr, 0);
            self.fill(&pool).await;
        }
    }

    pub async fn acquire(
        &self,
        addr: SocketAddr,
        client: usize,
        deadline: tokio::time::Instant,
    ) -> Result<Slot, ()> {
        let pool = self.pool(addr, client);
        let began = Instant::now();
        let needs_fill = {
            let state = pool.state.lock().unwrap();
            state.conns.iter().filter(|c| !c.dead).count() < self.cfg.connections
        };
        if needs_fill {
            self.fill(&pool).await;
        }
        let receiver = {
            let mut state = pool.state.lock().unwrap();
            if state.conns.iter().all(|c| c.dead) {
                return Err(());
            }
            match Self::pick(&state) {
                Some(index) => {
                    Self::reserve(&mut state, index, &self.stats);
                    let send = state.conns[index].send.clone().unwrap();
                    return Ok(Slot {
                        pool: pool.clone(),
                        index,
                        send,
                        queued_ms: 0.0,
                    });
                }
                None => {
                    let (tx, rx) = oneshot::channel();
                    state.waiters.push_back(tx);
                    rx
                }
            }
        };
        let index = match tokio::time::timeout_at(deadline, receiver).await {
            Ok(Ok(index)) => index,
            _ => return Err(()),
        };
        let queued = began.elapsed();
        let us = queued.as_micros() as u64;
        self.stats.queued.fetch_add(1, Ordering::Relaxed);
        self.stats.queue_wait_us.fetch_add(us, Ordering::Relaxed);
        self.stats
            .queue_wait_max_us
            .fetch_max(us, Ordering::Relaxed);
        let send = pool.state.lock().unwrap().conns[index]
            .send
            .clone()
            .unwrap();
        Ok(Slot {
            pool,
            index,
            send,
            queued_ms: queued.as_secs_f64() * 1000.0,
        })
    }

    pub fn release(&self, slot: &Slot, broken: bool) {
        let mut state = slot.pool.state.lock().unwrap();
        let c = &mut state.conns[slot.index];
        c.active -= 1;
        if broken && !c.dead {
            c.dead = true;
            self.stats.closed.fetch_add(1, Ordering::Relaxed);
        }
        self.stats.inflight.fetch_sub(1, Ordering::Relaxed);
        Self::wake(&mut state, &self.stats);
    }

    /// Sends one request; Err(broken) when no response head arrived.
    pub async fn request(
        &self,
        slot: &mut Slot,
        request: http::Request<()>,
        body: Option<Bytes>,
    ) -> Result<Response, bool> {
        let mut send = match slot.send.clone().ready().await {
            Ok(send) => send,
            Err(error) => return Err(self.note_error(&error)),
        };
        let (future, mut stream) = match send.send_request(request, body.is_none()) {
            Ok(pair) => pair,
            Err(error) => return Err(self.note_error(&error)),
        };
        if let Some(body) = body {
            if stream.send_data(body, true).is_err() {
                return Err(true);
            }
        }
        match future.await {
            Ok(response) => {
                let status = response.status().as_u16();
                let encoding = response
                    .headers()
                    .get("content-encoding")
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_owned);
                Ok(Response {
                    status,
                    encoding,
                    stream: response.into_body(),
                })
            }
            Err(error) => Err(self.note_error(&error)),
        }
    }

    /// Records RST_STREAM / GOAWAY; true when the connection is no longer usable.
    pub fn note_error(&self, error: &h2::Error) -> bool {
        if error.is_go_away() {
            self.stats.goaway.fetch_add(1, Ordering::Relaxed);
            return true;
        }
        if error.is_reset() {
            if let Some(reason) = error.reason() {
                let code: u32 = reason.into();
                *self.stats.rst.lock().unwrap().entry(code).or_default() += 1;
                if reason == h2::Reason::REFUSED_STREAM {
                    self.stats.refused.fetch_add(1, Ordering::Relaxed);
                }
            }
            return false;
        }
        error.is_io() || error.reason().is_none()
    }
    pub fn local_cancel(&self) {
        self.stats.local_cancels.fetch_add(1, Ordering::Relaxed);
    }

    /// Same fields as the Node h2 client report.
    pub fn report(&self) -> Value {
        let pools = self.pools.lock().unwrap();
        let mut totals = Vec::new();
        for pool in pools.values() {
            for c in &pool.state.lock().unwrap().conns {
                totals.push(c.total);
            }
        }
        let rst = self.stats.rst.lock().unwrap();
        let rst_obj: serde_json::Map<String, Value> =
            rst.iter().map(|(k, v)| (k.to_string(), json!(v))).collect();
        let server = self.stats.server_max.load(Ordering::Relaxed);
        let s = &self.stats;
        json!({"schema": 1, "protocol": "h2c", "transport": "h2c", "preset": self.cfg.preset, "perClientConnections": self.cfg.per_client, "connectionsPerPool": self.cfg.connections, "pools": pools.len(),
            "maxStreamsConfigured": self.cfg.max_streams, "serverMaxConcurrentStreams": if server == usize::MAX { Value::Null } else { json!(server) },
            "connectionsOpened": s.opened.load(Ordering::Relaxed), "connectionsClosed": s.closed.load(Ordering::Relaxed), "reconnects": s.reconnects.load(Ordering::Relaxed),
            "connectErrors": s.connect_errors.load(Ordering::Relaxed), "alpnFailures": 0, "goaway": s.goaway.load(Ordering::Relaxed),
            "rstStream": rst_obj, "rstStreamTotal": rst.values().sum::<u64>(), "refusedStreams": s.refused.load(Ordering::Relaxed), "localCancels": s.local_cancels.load(Ordering::Relaxed),
            "streams": s.streams.load(Ordering::Relaxed), "peakStreamsInFlight": s.peak.load(Ordering::Relaxed), "queuedStreams": s.queued.load(Ordering::Relaxed),
            "queueWaitMs": {"total": s.queue_wait_us.load(Ordering::Relaxed) as f64 / 1000.0, "max": s.queue_wait_max_us.load(Ordering::Relaxed) as f64 / 1000.0},
            "streamsPerConnection": {"connections": totals.len(), "min": totals.iter().min().copied().unwrap_or(0), "max": totals.iter().max().copied().unwrap_or(0), "total": totals.iter().sum::<u64>()},
            "flowControlStalls": "not exposed by the h2 crate"})
    }
}

/// Reads a response body chunk by chunk, releasing flow-control capacity.
pub async fn next_chunk(stream: &mut h2::RecvStream) -> Option<Result<Bytes, h2::Error>> {
    let chunk = stream.data().await?;
    if let Ok(bytes) = &chunk {
        let _ = stream.flow_control().release_capacity(bytes.len());
    }
    Some(chunk)
}
