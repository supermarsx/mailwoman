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

use std::convert::Infallible;
use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use futures_util::Stream;
use tokio::sync::broadcast;

use mw_engine::StateChange;

use crate::{AppState, authed};

/// How often an idle connection is nudged so proxies do not reap it (§2.2).
const HEARTBEAT: Duration = Duration::from_secs(30);

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
    let rx = state.push.subscribe();
    ws.on_upgrade(move |socket| ws_loop(socket, rx))
}

/// Pump broadcast frames to the socket, answer nothing but pings/closes from the
/// client, and heartbeat every [`HEARTBEAT`]. Exits cleanly on any I/O error.
async fn ws_loop(mut socket: WebSocket, mut rx: broadcast::Receiver<StateChange>) {
    let mut beat = tokio::time::interval(HEARTBEAT);
    beat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
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
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            },
            _ = beat.tick() => {
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
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
/// `StateChange` JSON emitted as `data:` frames; keep-alive comment lines every
/// [`HEARTBEAT`].
pub(crate) async fn jmap_eventsource(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if let Err(resp) = authed(&state, &headers).await {
        return resp;
    }
    let stream = change_stream(state.push.subscribe());
    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(HEARTBEAT).text("keep-alive"))
        .into_response()
}

/// Turn a broadcast receiver into an infinite SSE `Event` stream, skipping lag
/// gaps and ending when the channel closes.
fn change_stream(
    rx: broadcast::Receiver<StateChange>,
) -> impl Stream<Item = Result<Event, Infallible>> + Send {
    futures_util::stream::unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(sc) => {
                    let ev = Event::default().data(sc.to_wire().to_string());
                    return Some((Ok(ev), rx));
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
