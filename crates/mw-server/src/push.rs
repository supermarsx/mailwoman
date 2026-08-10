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

use std::convert::Infallible;
use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::HeaderMap;
use axum::http::header::{CACHE_CONTROL, HeaderName, HeaderValue};
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use futures_util::Stream;
use tokio::sync::broadcast;
use tokio::time::{Interval, MissedTickBehavior};

use mw_engine::StateChange;

use crate::{AppState, authed};

/// How often an idle connection is nudged so proxies do not reap it (§2.2).
const HEARTBEAT: Duration = Duration::from_secs(30);

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

/// `GET /jmap/ws` — authenticate via cookie *before* upgrading, then stream
/// `StateChange` frames. An unauthenticated request never upgrades (401).
pub(crate) async fn jmap_ws(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    if let Err(resp) = authed(&state, &headers).await {
        return resp;
    }
    negotiated_upgrade(ws, state.push.subscribe(), HEARTBEAT)
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
    rx: broadcast::Receiver<StateChange>,
    heartbeat: Duration,
) -> Response {
    ws.protocols([JMAP_SUBPROTOCOL])
        .on_upgrade(move |socket| ws_loop(socket, rx, heartbeat))
}

/// Pump broadcast frames to the socket, answer nothing but pings/closes from the
/// client, and keep alive every `heartbeat`. Exits cleanly on any I/O error.
async fn ws_loop(
    mut socket: WebSocket,
    mut rx: broadcast::Receiver<StateChange>,
    heartbeat: Duration,
) {
    let mut beat = idle_beat(heartbeat);
    loop {
        tokio::select! {
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Close(_))) | None => break,
                Some(Ok(_)) => {}            // pong/ping/text/binary: ignore (axum auto-pongs)
                Some(Err(_)) => break,
            },
            change = rx.recv() => match change {
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
        }
    }
    let _ = socket.send(Message::Close(None)).await;
}

// ---------------------------------------------------------------------------
// EventSource / SSE fallback
// ---------------------------------------------------------------------------

/// `GET /jmap/eventsource` — the SSE fallback. Same cookie auth, same
/// `StateChange` JSON emitted as `data:` frames, plus a `data:` keepalive every
/// [`HEARTBEAT`].
pub(crate) async fn jmap_eventsource(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if let Err(resp) = authed(&state, &headers).await {
        return resp;
    }
    sse_response(state.push.subscribe(), HEARTBEAT)
}

/// Build the SSE response, overriding the two headers a stock reverse proxy
/// otherwise gets wrong for a long-lived stream.
///
/// `axum` already sets `Cache-Control: no-cache`; the `insert` widens it to
/// `no-cache, no-transform` so an intermediary is also told not to recompress
/// the body, which is the other common way SSE frames get held back.
fn sse_response(rx: broadcast::Receiver<StateChange>, heartbeat: Duration) -> Response {
    let mut resp = Sse::new(change_stream(rx, heartbeat)).into_response();
    let headers = resp.headers_mut();
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-transform"),
    );
    headers.insert(X_ACCEL_BUFFERING, HeaderValue::from_static("no"));
    resp
}

/// Turn a broadcast receiver into an infinite SSE `Event` stream, skipping lag
/// gaps and ending when the channel closes.
///
/// The keepalive is emitted here rather than through [`axum::response::sse::KeepAlive`]
/// because that helper writes an SSE *comment* (`:keep-alive`), which the
/// `EventSource` API drops on the floor — see the module header. A `data:` frame
/// costs the same bytes and is observable.
fn change_stream(
    rx: broadcast::Receiver<StateChange>,
    heartbeat: Duration,
) -> impl Stream<Item = Result<Event, Infallible>> + Send {
    futures_util::stream::unfold(
        (rx, idle_beat(heartbeat)),
        |(mut rx, mut beat)| async move {
            loop {
                tokio::select! {
                    // `broadcast::Receiver::recv` is cancel-safe, so losing this
                    // branch of the select drops no change.
                    change = rx.recv() => match change {
                        Ok(sc) => {
                            beat.reset();     // a real frame is itself proof of life
                            let ev = Event::default().data(sc.to_wire().to_string());
                            return Some((Ok(ev), (rx, beat)));
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(broadcast::error::RecvError::Closed) => return None,
                    },
                    _ = beat.tick() => {
                        return Some((Ok(Event::default().data(PING_FRAME)), (rx, beat)));
                    }
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

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

    async fn spawn_sse(push: PushHandle) -> SocketAddr {
        let app = Router::new().route(
            "/jmap/eventsource",
            get(move || {
                let push = push.clone();
                async move { sse_response(push.subscribe(), TEST_BEAT) }
            }),
        );
        spawn(app).await
    }

    async fn spawn_ws(push: PushHandle) -> SocketAddr {
        let app = Router::new().route(
            "/jmap/ws",
            get(move |ws: WebSocketUpgrade| {
                let push = push.clone();
                async move { negotiated_upgrade(ws, push.subscribe(), TEST_BEAT) }
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
        StateChange {
            account_id: "acct1".into(),
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
