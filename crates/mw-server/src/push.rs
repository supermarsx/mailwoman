//! Realtime JMAP push (plan §2.2): `/jmap/ws` (WebSocket, RFC 8887) and
//! `/jmap/eventsource` (SSE fallback), both authenticated by the same
//! `mw_session` cookie and both streaming the identical [`StateChange`] wire
//! object produced by [`mw_engine::StateChange::to_wire`].
//!
//! ## Where the frames come from
//! `mw-server` never invents state — it drains a [`broadcast`] channel. In
//! engine mode `build_app` bridges `Engine::subscribe()` (fed by e9's
//! `start_watch` loop) into that channel; tests inject synthetic changes via
//! [`PushHandle`]. Either way the socket loop below is identical, so the wire
//! contract is proven without a live engine.
//!
//! ## Getting through a reverse proxy
//! Both endpoints are long-lived streams, which is exactly the shape a stock
//! proxy default breaks. The SSE response therefore carries
//! `X-Accel-Buffering: no` (nginx reads it and disables response buffering for
//! that one response) and `Cache-Control: no-cache, no-transform` (`no-transform`
//! tells any intermediary not to recompress or otherwise rewrite the body — a
//! compressing proxy holds SSE frames until its buffer fills). The `/jmap/ws`
//! upgrade negotiates the RFC 8887 `jmap` subprotocol when the client offers it.
//!
//! ## Keepalives a client can actually see
//! Both legs emit their idle keepalive as an ordinary `{"@type":"ping"}` message
//! — a `data:` frame on SSE, a text frame on WS — because neither an SSE comment
//! line nor a protocol-level WebSocket Ping reaches browser JavaScript: the DOM
//! surfaces comments and Pings nowhere. The web client's own liveness timer only
//! advances on an observed message, so an invisible keepalive leaves an idle but
//! perfectly healthy session indistinguishable from a dead one. The WS leg still
//! sends the protocol Ping as well, for proxies and native clients that watch it.
//!
//! ## Whose changes a stream carries, and for how long (D-27)
//! The broadcast is process-wide: it carries every account's changes. Each stream
//! filters it at its own subscription ([`ScopedChanges`]) down to the accounts
//! [`stream_accounts`] names for the session that opened it — today the
//! session's own account — so one user never sees another account's id or state
//! tokens. The client has no say in that list.
//!
//! The session is checked once at the upgrade by `authed`, and then again every
//! [`recheck_period`] (`MW_REALTIME_RECHECK_SECS`, default 30 s) by
//! [`still_valid`]: the bearer token's native marker, the session row, its
//! idle/absolute timeouts and the account's flags. A revoked or signed-out
//! session, a revoked bearer token, an expired session, a disabled account, an
//! account held for a password change, or a store that cannot be read ends the
//! stream — a WS close with code 1008, or the end of the SSE body. The re-check
//! does not count as activity, so an open stream does not keep an idle session
//! alive. Between the session ending and the next re-check (at most one period)
//! the stream can still carry that account's own changes; nothing is written to
//! it after a failed re-check.

use std::convert::Infallible;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade, close_code};
use axum::http::HeaderMap;
use axum::http::header::{CACHE_CONTROL, HeaderName, HeaderValue};
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use futures_util::Stream;
use futures_util::future::BoxFuture;
use tokio::sync::broadcast;
use tokio::time::{Interval, MissedTickBehavior};

use mw_engine::StateChange;
use mw_store::{Session, StoreError};

use crate::{AppState, account_gate, authed, push_relay};

/// How often an idle connection is nudged so proxies do not reap it (§2.2).
const HEARTBEAT: Duration = Duration::from_secs(30);

/// The environment variable that sets how often an open stream re-checks its
/// session, in whole seconds (D-27).
const RECHECK_ENV: &str = "MW_REALTIME_RECHECK_SECS";

/// [`RECHECK_ENV`]'s default, in seconds.
const RECHECK_DEFAULT_SECS: u64 = 30;

/// The reason sent with the WS close frame when a re-check ends a stream.
const SESSION_ENDED: &str = "session ended";

/// The RFC 8887 WebSocket subprotocol token. Echoed back only when the client
/// offered it; see [`negotiated_upgrade`].
const JMAP_SUBPROTOCOL: &str = "jmap";

/// nginx's per-response opt-out from `proxy_buffering on`, its stock default.
/// Other proxies ignore an unknown header, so emitting it unconditionally is
/// safe; the non-nginx recipes in `docs/deploy/proxy/` disable buffering in
/// their own config instead.
const X_ACCEL_BUFFERING: HeaderName = HeaderName::from_static("x-accel-buffering");

/// The idle keepalive payload, identical on both transports. Deliberately the
/// same object the web client already sends on the WS leg, and deliberately not
/// a `StateChange`: a client that does not know the type ignores it, while
/// having *received* it is the whole point.
const PING_FRAME: &str = r#"{"@type":"ping"}"#;

/// A keepalive ticker whose first tick is one full interval away. Ticking at t=0
/// would tell a client nothing it does not already know (it just connected) and
/// would put a keepalive ahead of the first real frame.
fn idle_beat(period: Duration) -> Interval {
    let mut beat = tokio::time::interval_at(tokio::time::Instant::now() + period, period);
    beat.set_missed_tick_behavior(MissedTickBehavior::Skip);
    beat
}

/// A cloneable sender end of the realtime push channel. `mw-server` holds one in
/// [`AppState`]; the engine-bridge and tests both feed it.
///
/// Two independent broadcast channels ride behind one handle: the `ws` channel
/// feeds the realtime `/jmap/ws` + `/jmap/eventsource` sessions, and the `relay`
/// channel feeds the V5 push dispatcher (plan §2.3 — the "second consumer" of the
/// engine `StateChange` broadcast that sends opaque WebPush/UnifiedPush wakes).
/// Keeping them separate means the dispatcher's always-on receiver does NOT
/// inflate the WS/SSE subscriber count [`send`](Self::send) reports — the
/// realtime wire behaviour (and its tests) is byte-identical.
#[derive(Clone)]
pub struct PushHandle {
    ws: broadcast::Sender<StateChange>,
    relay: broadcast::Sender<StateChange>,
}

impl PushHandle {
    /// Create a fresh push channel with a bounded backlog (slow WS/SSE clients
    /// lag rather than stall the engine).
    pub fn new() -> Self {
        let (ws, _rx) = broadcast::channel(256);
        let (relay, _rx) = broadcast::channel(256);
        Self { ws, relay }
    }

    /// Publish a [`StateChange`] to every connected session. Returns the number
    /// of live WS/SSE receivers (0 when nobody is listening — not an error). The
    /// change is also fanned out to the push-relay dispatcher (a separate channel,
    /// so it never changes the reported WS/SSE count).
    pub fn send(&self, change: StateChange) -> usize {
        // Fan out to the opaque-wake dispatcher (plan §2.3). Ignore its receiver
        // count / absence — it is an independent consumer.
        let _ = self.relay.send(change.clone());
        self.ws.send(change).unwrap_or(0)
    }

    /// A new receiver for one WS/SSE session.
    pub fn subscribe(&self) -> broadcast::Receiver<StateChange> {
        self.ws.subscribe()
    }

    /// A new receiver for the push-relay dispatcher (plan §2.3). Distinct from
    /// [`subscribe`](Self::subscribe) so the dispatcher does not count as a
    /// realtime WS/SSE subscriber.
    pub fn subscribe_relay(&self) -> broadcast::Receiver<StateChange> {
        self.relay.subscribe()
    }
}

impl Default for PushHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// The account ids a session's realtime stream may carry changes for. Today that
/// is the session's own account; accounts a session is later authorised for
/// beyond its own join here, so every stream is filtered through this one
/// function. An empty account id names no account (a proxy-mode upstream that
/// reports none) and such a session sees no changes at all.
pub(crate) fn stream_accounts(session: &Session) -> Vec<String> {
    if session.account_id.is_empty() {
        Vec::new()
    } else {
        vec![session.account_id.clone()]
    }
}

/// One stream's subscription to the process-wide broadcast, narrowed to the
/// changes for `accounts`. Changes for any other account are dropped here and
/// never reach the socket.
pub(crate) struct ScopedChanges {
    rx: broadcast::Receiver<StateChange>,
    accounts: Vec<String>,
}

impl ScopedChanges {
    pub(crate) fn new(rx: broadcast::Receiver<StateChange>, accounts: Vec<String>) -> Self {
        Self { rx, accounts }
    }

    /// The next change for one of this stream's accounts. Cancel-safe, as
    /// [`broadcast::Receiver::recv`] is: a change for another account is dropped
    /// whether or not the caller is still waiting.
    async fn recv(&mut self) -> Result<StateChange, broadcast::error::RecvError> {
        loop {
            let change = self.rx.recv().await?;
            if self.accounts.contains(&change.account_id) {
                return Ok(change);
            }
        }
    }
}

/// How often an open stream re-checks its session: [`RECHECK_ENV`] read once
/// per process, at least one second, [`RECHECK_DEFAULT_SECS`] when unset or not
/// a whole number.
fn recheck_period() -> Duration {
    static PERIOD: OnceLock<Duration> = OnceLock::new();
    *PERIOD.get_or_init(|| parse_recheck(std::env::var(RECHECK_ENV).ok().as_deref()))
}

fn parse_recheck(raw: Option<&str>) -> Duration {
    let secs = match raw.map(str::trim) {
        None | Some("") => RECHECK_DEFAULT_SECS,
        Some(v) => v.parse::<u64>().map(|n| n.max(1)).unwrap_or_else(|_| {
            tracing::warn!(
                "{RECHECK_ENV}={v:?} is not a whole number of seconds; using {RECHECK_DEFAULT_SECS}"
            );
            RECHECK_DEFAULT_SECS
        }),
    };
    Duration::from_secs(secs)
}

/// Decides, every `period`, whether an open stream may stay open.
#[derive(Clone)]
struct StreamGuard {
    period: Duration,
    check: Arc<dyn Fn() -> BoxFuture<'static, bool> + Send + Sync>,
}

impl StreamGuard {
    /// The guard for a stream opened by `session`; `bearer` says whether it was
    /// presented as a native bearer token rather than a cookie.
    fn for_session(state: AppState, session: Session, bearer: bool) -> Self {
        Self {
            period: recheck_period(),
            check: Arc::new(move || {
                let (state, session) = (state.clone(), session.clone());
                Box::pin(async move { still_valid(&state, &session, bearer).await })
            }),
        }
    }

    async fn holds(&self) -> bool {
        (self.check)().await
    }
}

/// Is `session` still good for a realtime stream? Re-reads what `authed` read at
/// the upgrade, without recording activity. Any read that fails says no (fail
/// closed).
async fn still_valid(state: &AppState, session: &Session, bearer: bool) -> bool {
    match recheck(state, session, bearer).await {
        Ok(()) => true,
        Err(why) => {
            tracing::info!("realtime stream for {} ended: {why}", session.username);
            false
        }
    }
}

async fn recheck(state: &AppState, session: &Session, bearer: bool) -> Result<(), &'static str> {
    const UNREADABLE: &str = "the store could not be read";
    if bearer {
        match state
            .store
            .get_native_session(&push_relay::hash_token(&session.id))
            .await
        {
            Ok(Some(_)) => {}
            Ok(None) => return Err("bearer token revoked"),
            Err(_) => return Err(UNREADABLE),
        }
    }
    let current = match state.store.get_session(&session.id).await {
        Ok(current) => current,
        Err(StoreError::NotFound) => return Err("session revoked"),
        Err(_) => return Err(UNREADABLE),
    };
    if current.account_id != session.account_id {
        return Err("session changed account");
    }
    // Not `SessionGuard::check`: that records activity, and an open stream is
    // not activity. The next request with the session deletes it, as `authed`
    // does for any expired session.
    if state
        .sessions
        .expired(&session.id, &state.hardening)
        .is_some()
    {
        return Err("session expired");
    }
    // Request telemetry and a process-local clock cannot extend the persisted
    // foreground-activity deadline, including after restarts or policy changes.
    crate::security_center::enforce_session(state, &current)
        .await
        .map_err(|_| "session expired or policy unavailable")?;
    let gate = account_gate::for_session(&state.store, &current)
        .await
        .map_err(|_| UNREADABLE)?;
    if gate.disabled {
        return Err("account disabled");
    }
    if gate.password_change_required {
        return Err("password change required");
    }
    Ok(())
}

/// The scoped subscription and the guard for a stream `session` is opening.
fn open_stream(
    state: &AppState,
    headers: &HeaderMap,
    session: Session,
) -> (ScopedChanges, StreamGuard) {
    // The same test `authed_with_gate` uses to pick the bearer path.
    let bearer = push_relay::bearer_token(headers).is_some();
    let changes = ScopedChanges::new(state.push.subscribe(), stream_accounts(&session));
    (
        changes,
        StreamGuard::for_session(state.clone(), session, bearer),
    )
}

/// Forward every `Engine` broadcast into the server push channel for the life of
/// the process. Spawned once by `build_app` in engine mode.
pub(crate) async fn bridge_engine(mut src: broadcast::Receiver<StateChange>, out: PushHandle) {
    loop {
        match src.recv().await {
            Ok(change) => {
                out.send(change);
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("push bridge lagged {n} engine changes");
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

// ---------------------------------------------------------------------------
// WebSocket (RFC 8887)
// ---------------------------------------------------------------------------

/// `GET /jmap/ws` — authenticate via cookie or native bearer *before* upgrading,
/// then stream the session's own `StateChange` frames until the session ends.
/// An unauthenticated request never upgrades (401).
pub(crate) async fn jmap_ws(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    let session = match authed(&state, &headers).await {
        Ok(session) => session,
        Err(resp) => return resp,
    };
    let (changes, guard) = open_stream(&state, &headers, session);
    negotiated_upgrade(ws, changes, HEARTBEAT, guard)
}

/// Complete the upgrade, offering the RFC 8887 `jmap` subprotocol.
///
/// [`WebSocketUpgrade::protocols`] selects from what the *client* offered, so
/// the echo happens only for a client that asked for `jmap`. A client that
/// offers nothing, or offers only tokens we do not speak, still upgrades with no
/// `Sec-WebSocket-Protocol` in the response — which is what every browser and
/// the pre-existing in-tree clients expect. Making the subprotocol mandatory
/// would break all of them.
fn negotiated_upgrade(
    ws: WebSocketUpgrade,
    changes: ScopedChanges,
    heartbeat: Duration,
    guard: StreamGuard,
) -> Response {
    ws.protocols([JMAP_SUBPROTOCOL])
        .on_upgrade(move |socket| ws_loop(socket, changes, heartbeat, guard))
}

/// Pump the stream's changes to the socket, answer nothing but pings/closes from
/// the client, and keep alive every `heartbeat`. Exits cleanly on any I/O error,
/// and with close code 1008 when a re-check finds the session no longer valid.
async fn ws_loop(
    mut socket: WebSocket,
    mut changes: ScopedChanges,
    heartbeat: Duration,
    guard: StreamGuard,
) {
    let mut beat = idle_beat(heartbeat);
    let mut recheck = idle_beat(guard.period);
    loop {
        tokio::select! {
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Close(_))) | None => break,
                Some(Ok(_)) => {}            // pong/ping/text/binary: ignore (axum auto-pongs)
                Some(Err(_)) => break,
            },
            change = changes.recv() => match change {
                Ok(sc) => {
                    let frame = sc.to_wire().to_string();
                    if socket.send(Message::Text(frame.into())).await.is_err() {
                        break;
                    }
                    beat.reset();     // a real frame is itself proof of life
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            },
            _ = beat.tick() => {
                // Protocol Ping first (proxies and native clients watch for it),
                // then the text frame that is the only half browser JS can see.
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
                    break;
                }
                if socket.send(Message::Text(PING_FRAME.into())).await.is_err() {
                    break;
                }
            }
            _ = recheck.tick() => {
                if !guard.holds().await {
                    let frame = CloseFrame {
                        code: close_code::POLICY,
                        reason: SESSION_ENDED.into(),
                    };
                    let _ = socket.send(Message::Close(Some(frame))).await;
                    return;
                }
            }
        }
    }
    let _ = socket.send(Message::Close(None)).await;
}

// ---------------------------------------------------------------------------
// EventSource / SSE fallback
// ---------------------------------------------------------------------------

/// `GET /jmap/eventsource` — the SSE fallback. Same auth, same scope and the
/// same re-check as the WS leg, the session's `StateChange` JSON emitted as
/// `data:` frames, plus a `data:` keepalive every [`HEARTBEAT`].
pub(crate) async fn jmap_eventsource(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    let session = match authed(&state, &headers).await {
        Ok(session) => session,
        Err(resp) => return resp,
    };
    let (changes, guard) = open_stream(&state, &headers, session);
    sse_response(changes, HEARTBEAT, guard)
}

/// Build the SSE response, overriding the two headers a stock reverse proxy
/// otherwise gets wrong for a long-lived stream.
///
/// `axum` already sets `Cache-Control: no-cache`; the `insert` widens it to
/// `no-cache, no-transform` so an intermediary is also told not to recompress
/// the body, which is the other common way SSE frames get held back.
fn sse_response(changes: ScopedChanges, heartbeat: Duration, guard: StreamGuard) -> Response {
    let mut resp = Sse::new(change_stream(changes, heartbeat, guard)).into_response();
    let headers = resp.headers_mut();
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-transform"),
    );
    headers.insert(X_ACCEL_BUFFERING, HeaderValue::from_static("no"));
    resp
}

/// Turn the stream's changes into an SSE `Event` stream, skipping lag gaps and
/// ending when the channel closes or a re-check finds the session no longer
/// valid.
///
/// The keepalive is emitted here rather than through [`axum::response::sse::KeepAlive`]
/// because that helper writes an SSE *comment* (`:keep-alive`), which the
/// `EventSource` API drops on the floor — see the module header. A `data:` frame
/// costs the same bytes and is observable.
fn change_stream(
    changes: ScopedChanges,
    heartbeat: Duration,
    guard: StreamGuard,
) -> impl Stream<Item = Result<Event, Infallible>> + Send {
    let recheck = idle_beat(guard.period);
    futures_util::stream::unfold(
        (changes, idle_beat(heartbeat), recheck, guard),
        |(mut changes, mut beat, mut recheck, guard)| async move {
            loop {
                tokio::select! {
                    // `ScopedChanges::recv` is cancel-safe, so losing this branch
                    // of the select drops no change for this stream.
                    change = changes.recv() => match change {
                        Ok(sc) => {
                            beat.reset();     // a real frame is itself proof of life
                            let ev = Event::default().data(sc.to_wire().to_string());
                            return Some((Ok(ev), (changes, beat, recheck, guard)));
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(broadcast::error::RecvError::Closed) => return None,
                    },
                    _ = beat.tick() => {
                        let ev = Event::default().data(PING_FRAME);
                        return Some((Ok(ev), (changes, beat, recheck, guard)));
                    }
                    _ = recheck.tick() => {
                        if !guard.holds().await {
                            return None;
                        }
                    }
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use axum::Router;
    use axum::routing::get;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    use super::*;

    /// A keepalive period short enough to observe inside a test. The production
    /// value is [`HEARTBEAT`]; both endpoints take it as a parameter precisely so
    /// the idle behaviour is testable without a 30 s wait.
    const TEST_BEAT: Duration = Duration::from_millis(60);

    /// Serve `app` on an ephemeral loopback port. These routers deliberately omit
    /// `AppState` and the `authed` guard: what is under test is the wire
    /// behaviour of the two response builders, which the real handlers call
    /// unchanged once authentication has passed.
    async fn spawn(app: Router) -> SocketAddr {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        addr
    }

    /// A guard that always holds. The re-check is the integration tests' subject
    /// (`tests/t32_h13_realtime_scope.rs`); here it must not interfere.
    fn guard_holding() -> StreamGuard {
        StreamGuard {
            period: Duration::from_secs(3600),
            check: Arc::new(|| Box::pin(async { true })),
        }
    }

    /// A guard that holds `n` times and then fails, re-checking every 50 ms.
    fn guard_failing_after(n: usize) -> StreamGuard {
        let left = Arc::new(AtomicUsize::new(n));
        StreamGuard {
            period: Duration::from_millis(50),
            check: Arc::new(move || {
                let left = left.clone();
                Box::pin(async move {
                    left.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| v.checked_sub(1))
                        .is_ok()
                })
            }),
        }
    }

    /// The stream the test endpoints serve: scoped to [`change`]'s account.
    fn scoped(push: &PushHandle) -> ScopedChanges {
        ScopedChanges::new(push.subscribe(), vec!["acct1".into()])
    }

    async fn spawn_sse(push: PushHandle) -> SocketAddr {
        spawn_sse_guarded(push, guard_holding()).await
    }

    async fn spawn_sse_guarded(push: PushHandle, guard: StreamGuard) -> SocketAddr {
        let app = Router::new().route(
            "/jmap/eventsource",
            get(move || {
                let (push, guard) = (push.clone(), guard.clone());
                async move { sse_response(scoped(&push), TEST_BEAT, guard) }
            }),
        );
        spawn(app).await
    }

    async fn spawn_ws(push: PushHandle) -> SocketAddr {
        spawn_ws_guarded(push, guard_holding()).await
    }

    async fn spawn_ws_guarded(push: PushHandle, guard: StreamGuard) -> SocketAddr {
        let app = Router::new().route(
            "/jmap/ws",
            get(move |ws: WebSocketUpgrade| {
                let (push, guard) = (push.clone(), guard.clone());
                async move { negotiated_upgrade(ws, scoped(&push), TEST_BEAT, guard) }
            }),
        );
        spawn(app).await
    }

    /// Send one raw HTTP/1.1 request head and return the (lowercased) response
    /// head plus the still-open socket. Raw bytes rather than a client library so
    /// the assertions are about what actually goes on the wire.
    async fn raw_request(addr: SocketAddr, head: &str) -> (String, TcpStream) {
        let mut sock = TcpStream::connect(addr).await.unwrap();
        sock.write_all(head.as_bytes()).await.unwrap();
        let mut buf = Vec::new();
        let mut byte = [0u8; 1];
        while !buf.ends_with(b"\r\n\r\n") {
            let n = tokio::time::timeout(Duration::from_secs(5), sock.read(&mut byte))
                .await
                .expect("response head arrives")
                .unwrap();
            assert_eq!(n, 1, "connection closed mid response head");
            buf.push(byte[0]);
        }
        (String::from_utf8(buf).unwrap().to_ascii_lowercase(), sock)
    }

    /// A well-formed RFC 6455 upgrade head, optionally offering subprotocols.
    fn upgrade_head(addr: SocketAddr, protocols: Option<&str>) -> String {
        let offer = protocols
            .map(|p| format!("Sec-WebSocket-Protocol: {p}\r\n"))
            .unwrap_or_default();
        format!(
            "GET /jmap/ws HTTP/1.1\r\n\
             Host: {addr}\r\n\
             Connection: Upgrade\r\n\
             Upgrade: websocket\r\n\
             Sec-WebSocket-Version: 13\r\n\
             Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
             {offer}\r\n"
        )
    }

    /// Read from `sock` until `needle` shows up in the accumulated body.
    async fn read_until(sock: &mut TcpStream, needle: &str) -> String {
        let mut body = String::new();
        let mut chunk = [0u8; 2048];
        let outcome = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let n = sock.read(&mut chunk).await.unwrap();
                assert!(n > 0, "stream closed before {needle:?} arrived");
                body.push_str(&String::from_utf8_lossy(&chunk[..n]));
                if body.contains(needle) {
                    return;
                }
            }
        })
        .await;
        assert!(outcome.is_ok(), "timed out waiting for {needle:?}");
        body
    }

    fn change() -> StateChange {
        change_for("acct1")
    }

    fn change_for(account_id: &str) -> StateChange {
        StateChange {
            account_id: account_id.into(),
            email: "7".into(),
            mailbox: "3".into(),
            submission: "1".into(),
            thread: "7".into(),
            crypto_key: "2".into(),
            mail_rule: "1".into(),
        }
    }

    #[tokio::test]
    async fn push_handle_delivers_to_subscribers() {
        let h = PushHandle::new();
        let mut a = h.subscribe();
        let mut b = h.subscribe();
        assert_eq!(h.send(change()), 2);
        assert_eq!(a.recv().await.unwrap(), change());
        assert_eq!(b.recv().await.unwrap(), change());
    }

    #[tokio::test]
    async fn send_with_no_subscribers_is_not_an_error() {
        let h = PushHandle::new();
        assert_eq!(h.send(change()), 0);
    }

    #[tokio::test]
    async fn scoped_changes_drop_other_accounts() {
        let h = PushHandle::new();
        let mut mine = ScopedChanges::new(h.subscribe(), vec!["acct1".into()]);
        h.send(change_for("acct2"));
        h.send(change_for(""));
        h.send(change_for("acct1"));
        assert_eq!(mine.recv().await.unwrap(), change_for("acct1"));
    }

    #[test]
    fn an_empty_account_id_is_scoped_to_nothing() {
        let session = |account_id: &str| Session {
            id: "s".into(),
            account_id: account_id.into(),
            username: "u".into(),
            jmap_url: String::new(),
            api_url: String::new(),
            credentials: mw_store::Credentials {
                username: "u".into(),
                password: "p".into(),
            },
        };
        assert!(stream_accounts(&session("")).is_empty());
        assert_eq!(
            stream_accounts(&session("acct1")),
            vec!["acct1".to_string()]
        );
    }

    #[test]
    fn recheck_period_is_whole_seconds_with_a_floor_of_one() {
        let secs = |raw| parse_recheck(raw).as_secs();
        assert_eq!(secs(None), RECHECK_DEFAULT_SECS);
        assert_eq!(secs(Some("")), RECHECK_DEFAULT_SECS);
        assert_eq!(secs(Some("5")), 5);
        assert_eq!(secs(Some(" 7 ")), 7);
        assert_eq!(secs(Some("0")), 1);
        assert_eq!(secs(Some("-3")), RECHECK_DEFAULT_SECS);
        assert_eq!(secs(Some("1.5")), RECHECK_DEFAULT_SECS);
    }

    #[tokio::test]
    async fn a_failed_recheck_ends_the_sse_body() {
        let addr = spawn_sse_guarded(PushHandle::new(), guard_failing_after(1)).await;
        let (_head, mut sock) = raw_request(
            addr,
            &format!("GET /jmap/eventsource HTTP/1.1\r\nHost: {addr}\r\n\r\n"),
        )
        .await;
        // A chunked body ends with its zero-length last chunk.
        read_until(&mut sock, "\r\n0\r\n\r\n").await;
    }

    #[tokio::test]
    async fn a_failed_recheck_closes_the_ws_with_policy_violation() {
        use futures_util::StreamExt;
        use tokio_tungstenite::tungstenite::Message as Frame;

        let addr = spawn_ws_guarded(PushHandle::new(), guard_failing_after(1)).await;
        let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/jmap/ws"))
            .await
            .unwrap();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            let msg = tokio::time::timeout_at(deadline, ws.next())
                .await
                .expect("the close arrives");
            match msg {
                Some(Ok(Frame::Close(Some(frame)))) => {
                    assert_eq!(u16::from(frame.code), 1008);
                    assert_eq!(frame.reason.as_str(), SESSION_ENDED);
                    return;
                }
                Some(Ok(Frame::Ping(_) | Frame::Text(_))) => continue,
                other => panic!("expected a policy close, got {other:?}"),
            }
        }
    }

    #[tokio::test]
    async fn bridge_forwards_engine_changes() {
        let (engine_tx, engine_rx) = broadcast::channel(8);
        let out = PushHandle::new();
        let mut sink = out.subscribe();
        tokio::spawn(bridge_engine(engine_rx, out.clone()));
        engine_tx.send(change()).unwrap();
        assert_eq!(sink.recv().await.unwrap(), change());
    }

    // -----------------------------------------------------------------------
    // SSE: proxy headers + observable keepalive
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn sse_carries_the_proxy_buffering_headers() {
        let addr = spawn_sse(PushHandle::new()).await;
        let (head, _sock) = raw_request(
            addr,
            &format!("GET /jmap/eventsource HTTP/1.1\r\nHost: {addr}\r\n\r\n"),
        )
        .await;

        assert!(head.starts_with("http/1.1 200"), "head was: {head}");
        assert!(
            head.contains("content-type: text/event-stream"),
            "head was: {head}"
        );
        // Nginx's stock `proxy_buffering on` batches SSE frames without this.
        assert!(head.contains("x-accel-buffering: no"), "head was: {head}");
        // `no-transform` on top of axum's `no-cache`: a recompressing
        // intermediary holds frames until its buffer fills.
        assert!(
            head.contains("cache-control: no-cache, no-transform"),
            "head was: {head}"
        );
        assert!(
            !head.contains("cache-control: no-cache\r\n"),
            "the narrower axum default must have been replaced, not appended: {head}"
        );
    }

    #[tokio::test]
    async fn sse_idle_keepalive_is_a_data_frame_not_a_comment() {
        let addr = spawn_sse(PushHandle::new()).await;
        let (_head, mut sock) = raw_request(
            addr,
            &format!("GET /jmap/eventsource HTTP/1.1\r\nHost: {addr}\r\n\r\n"),
        )
        .await;

        // Nothing is broadcast; the only traffic can be the idle keepalive.
        let body = read_until(&mut sock, PING_FRAME).await;
        assert!(
            body.lines().any(|l| l == format!("data: {PING_FRAME}")),
            "keepalive must be a `data:` line an EventSource can see, got: {body:?}"
        );
        assert!(
            !body.contains(":keep-alive"),
            "the invisible comment keepalive should be gone, got: {body:?}"
        );
    }

    #[tokio::test]
    async fn sse_still_streams_state_changes() {
        let push = PushHandle::new();
        let addr = spawn_sse(push.clone()).await;
        let (_head, mut sock) = raw_request(
            addr,
            &format!("GET /jmap/eventsource HTTP/1.1\r\nHost: {addr}\r\n\r\n"),
        )
        .await;

        // The handler subscribes before it returns the head, so the subscriber
        // is live by now.
        assert_eq!(push.send(change()), 1);
        let body = read_until(&mut sock, "StateChange").await;
        assert!(
            body.lines()
                .any(|l| l.starts_with("data: {") && l.contains("StateChange")),
            "change must still arrive as a data: frame, got: {body:?}"
        );
    }

    // -----------------------------------------------------------------------
    // WebSocket: RFC 8887 subprotocol negotiation + observable keepalive
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn ws_echoes_the_jmap_subprotocol_and_pings_observably() {
        use futures_util::StreamExt;
        use tokio_tungstenite::tungstenite::Message as Frame;
        use tokio_tungstenite::tungstenite::client::IntoClientRequest;

        let addr = spawn_ws(PushHandle::new()).await;
        let mut req = format!("ws://{addr}/jmap/ws")
            .into_client_request()
            .unwrap();
        req.headers_mut()
            .insert("sec-websocket-protocol", JMAP_SUBPROTOCOL.parse().unwrap());

        // tungstenite enforces RFC 6455 §4.1: a client that offered a
        // subprotocol and gets none back fails the connection. So reaching
        // `unwrap` at all is the negotiation assertion; the header check below
        // pins which token was chosen.
        let (mut ws, resp) = tokio_tungstenite::connect_async(req).await.unwrap();
        assert_eq!(
            resp.headers().get("sec-websocket-protocol").unwrap(),
            JMAP_SUBPROTOCOL
        );

        let mut saw_protocol_ping = false;
        let mut saw_text_ping = false;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        while !(saw_protocol_ping && saw_text_ping) {
            let msg = tokio::time::timeout_at(deadline, ws.next())
                .await
                .expect("keepalives arrive")
                .expect("stream stays open")
                .expect("no ws error");
            match msg {
                Frame::Ping(_) => saw_protocol_ping = true,
                Frame::Text(t) => {
                    assert_eq!(t.as_str(), PING_FRAME, "unexpected text frame");
                    saw_text_ping = true;
                }
                other => panic!("unexpected frame: {other:?}"),
            }
        }
    }

    #[tokio::test]
    async fn ws_upgrade_without_a_subprotocol_offer_is_accepted() {
        let addr = spawn_ws(PushHandle::new()).await;
        let (head, _sock) = raw_request(addr, &upgrade_head(addr, None)).await;

        assert!(head.starts_with("http/1.1 101"), "head was: {head}");
        // Echoing a protocol the client never offered would make a browser fail
        // the connection.
        assert!(!head.contains("sec-websocket-protocol"), "head was: {head}");
    }

    #[tokio::test]
    async fn ws_upgrade_offering_only_an_unknown_subprotocol_still_connects() {
        let addr = spawn_ws(PushHandle::new()).await;
        let (head, _sock) = raw_request(addr, &upgrade_head(addr, Some("soap, wamp"))).await;

        assert!(head.starts_with("http/1.1 101"), "head was: {head}");
        assert!(
            !head.contains("sec-websocket-protocol"),
            "we must not claim a protocol we do not speak: {head}"
        );
    }
}
