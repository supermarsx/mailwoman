//! t35: lag detection, resync from current state, duplicate suppression,
//! shutdown close and the switches. The integration legs with a real store and
//! real links are in `tests/t32_h13_realtime_scope.rs` and
//! `tests/t46_linked_live.rs`.
//!
//! How a lag is produced deterministically: `#[tokio::test]` runs on one
//! thread, and the server's stream task shares it. Sending more changes than
//! the channel holds without an `.await` in between means the stream cannot
//! take any of them until the sends are done, so its receiver must report
//! `Lagged`.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::Router;
use axum::routing::get;
use futures_util::StreamExt;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message as Frame;

use super::*;

const BEAT: Duration = Duration::from_secs(3600);
const CAPACITY: usize = 64;

fn handle(lag_resync: bool) -> PushHandle {
    PushHandle::with_config(RealtimeConfig {
        fanout_capacity: CAPACITY,
        lag_resync,
        ..RealtimeConfig::default()
    })
}

fn guard_holding() -> StreamGuard {
    StreamGuard {
        period: Duration::from_secs(3600),
        check: Arc::new(|| Box::pin(async { true })),
    }
}

fn change(account: &str, email: &str) -> StateChange {
    StateChange {
        account_id: account.into(),
        email: email.into(),
        mailbox: "1".into(),
        submission: "1".into(),
        thread: email.into(),
        crypto_key: "0".into(),
        mail_rule: "0".into(),
    }
}

/// A state source that answers `change(account, "current-<account>")` and
/// records which accounts it was asked for.
#[derive(Clone, Default)]
struct Source {
    asked: Arc<Mutex<Vec<String>>>,
    fail: bool,
}

impl Source {
    fn failing() -> Self {
        Self {
            fail: true,
            ..Self::default()
        }
    }
    fn reader(&self) -> ReadFn {
        let me = self.clone();
        Arc::new(move |account: String| {
            let me = me.clone();
            Box::pin(async move {
                me.asked.lock().unwrap().push(account.clone());
                if me.fail {
                    return Err("injected read failure".into());
                }
                Ok(change(&account, &format!("current-{account}")))
            })
        })
    }
    fn asked(&self) -> Vec<String> {
        self.asked.lock().unwrap().clone()
    }
}

fn scoped(push: &PushHandle, account: &str, source: &Source) -> ScopedChanges {
    ScopedChanges::new(push.subscribe(), vec![account.into()]).attach(push, Some(source.reader()))
}

async fn spawn(app: Router) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    addr
}

async fn spawn_ws(push: PushHandle, account: &'static str, source: Source) -> SocketAddr {
    spawn(
        Router::new().route(
            "/jmap/ws",
            get(move |ws: WebSocketUpgrade| {
                let (push, source) = (push.clone(), source.clone());
                async move {
                    negotiated_upgrade(ws, scoped(&push, account, &source), BEAT, guard_holding())
                }
            }),
        ),
    )
    .await
}

async fn spawn_sse(push: PushHandle, account: &'static str, source: Source) -> SocketAddr {
    spawn(Router::new().route(
        "/jmap/eventsource",
        get(move || {
            let (push, source) = (push.clone(), source.clone());
            async move { sse_response(scoped(&push, account, &source), BEAT, guard_holding()) }
        }),
    ))
    .await
}

type Ws = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>;

async fn open_ws(addr: SocketAddr) -> Ws {
    tokio_tungstenite::connect_async(format!("ws://{addr}/jmap/ws"))
        .await
        .unwrap()
        .0
}

/// What a WS client saw: text frames, then the close code if the server
/// closed within `within`.
async fn ws_read(ws: &mut Ws, within: Duration, until: &str) -> (Vec<String>, Option<u16>) {
    let deadline = tokio::time::Instant::now() + within;
    let mut texts = Vec::new();
    loop {
        match tokio::time::timeout_at(deadline, ws.next()).await {
            Err(_) => return (texts, None),
            Ok(Some(Ok(Frame::Text(t)))) => {
                let done = !until.is_empty() && t.as_str().contains(until);
                texts.push(t.as_str().to_string());
                if done {
                    return (texts, None);
                }
            }
            Ok(Some(Ok(Frame::Close(frame)))) => {
                return (texts, frame.map(|f| u16::from(f.code)));
            }
            Ok(Some(Ok(_))) => {}
            Ok(Some(Err(_))) | Ok(None) => return (texts, Some(0)),
        }
    }
}

async fn open_sse(addr: SocketAddr) -> TcpStream {
    use tokio::io::AsyncWriteExt;
    let mut sock = TcpStream::connect(addr).await.unwrap();
    sock.write_all(format!("GET /jmap/eventsource HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes())
        .await
        .unwrap();
    // The handler subscribes before it returns the head.
    sse_read(&mut sock, Duration::from_secs(5), "\r\n\r\n").await;
    sock
}

/// The SSE body read until `needle` (or the end of the body, or `within`).
/// Returns the text and whether the body ended.
async fn sse_read(sock: &mut TcpStream, within: Duration, needle: &str) -> (String, bool) {
    let deadline = tokio::time::Instant::now() + within;
    let mut body = String::new();
    let mut chunk = [0u8; 4096];
    loop {
        match tokio::time::timeout_at(deadline, sock.read(&mut chunk)).await {
            Err(_) => return (body, false),
            Ok(Ok(0)) | Ok(Err(_)) => return (body, true),
            Ok(Ok(n)) => {
                body.push_str(&String::from_utf8_lossy(&chunk[..n]));
                // The last chunk; its leading CRLF may have come with the head.
                if body.starts_with("0\r\n\r\n") || body.contains("\r\n0\r\n\r\n") {
                    return (body, true);
                }
                if !needle.is_empty() && body.contains(needle) {
                    return (body, false);
                }
            }
        }
    }
}

/// The lag: `acct1`'s own change first (it falls out of the channel), then more
/// changes for other accounts than the channel holds, with no `.await`.
fn overflow_with_own_change_lost(push: &PushHandle) {
    push.send(change("acct1", "dropped"));
    for i in 0..(CAPACITY * 2) {
        push.send(change(&format!("other-{i}"), "other"));
    }
}

fn snap(path: &[&str]) -> u64 {
    let mut v = metrics_snapshot();
    for key in path {
        v = v[*key].clone();
    }
    v.as_u64().unwrap_or(0)
}

// ── a lagged stream sends its accounts' current state ────────────────────────

#[tokio::test]
async fn a_lagged_ws_stream_sends_its_accounts_current_state() {
    let push = handle(true);
    let source = Source::default();
    let addr = spawn_ws(push.clone(), "acct1", source.clone()).await;
    let mut ws = open_ws(addr).await;
    let lagged_before = snap(&["lagged", "ws"]);

    overflow_with_own_change_lost(&push);

    let (texts, closed) = ws_read(&mut ws, Duration::from_secs(5), "current-acct1").await;
    assert_eq!(closed, None, "the stream stays open: {texts:?}");
    let last: serde_json::Value = serde_json::from_str(texts.last().unwrap()).unwrap();
    assert_eq!(last, change("acct1", "current-acct1").to_wire());
    let all = texts.join("\n");
    assert!(
        !all.contains("dropped"),
        "the lost frame is not replayed: {all}"
    );
    assert!(
        !all.contains("other-"),
        "no other account reaches the stream: {all}"
    );
    assert_eq!(
        source.asked(),
        vec!["acct1".to_string()],
        "only its own account is read"
    );
    assert!(
        snap(&["lagged", "ws"]) > lagged_before,
        "the lag is counted"
    );

    // Still live afterwards.
    push.send(change("acct1", "after"));
    let (texts, _) = ws_read(&mut ws, Duration::from_secs(5), "after").await;
    assert!(texts.last().unwrap().contains("after"));
}

#[tokio::test]
async fn a_lagged_sse_stream_sends_its_accounts_current_state() {
    let push = handle(true);
    let source = Source::default();
    let addr = spawn_sse(push.clone(), "acct1", source.clone()).await;
    let mut sock = open_sse(addr).await;
    let lagged_before = snap(&["lagged", "sse"]);

    overflow_with_own_change_lost(&push);

    let (body, ended) = sse_read(&mut sock, Duration::from_secs(5), "current-acct1").await;
    assert!(!ended, "the body goes on: {body}");
    assert!(body.contains("current-acct1"), "{body}");
    assert!(
        !body.contains("dropped") && !body.contains("other-"),
        "{body}"
    );
    assert_eq!(source.asked(), vec!["acct1".to_string()]);
    assert!(snap(&["lagged", "sse"]) > lagged_before);
}

/// `MW_REALTIME_LAG_RESYNC=off`: the earlier behaviour, the stream continues
/// past the gap without reading anything.
#[tokio::test]
async fn lag_resync_off_restores_continue() {
    for ws in [true, false] {
        let push = handle(false);
        let source = Source::default();
        if ws {
            let addr = spawn_ws(push.clone(), "acct1", source.clone()).await;
            let mut client = open_ws(addr).await;
            overflow_with_own_change_lost(&push);
            push.send(change("acct1", "after"));
            let (texts, closed) = ws_read(&mut client, Duration::from_secs(5), "after").await;
            assert_eq!(closed, None);
            assert!(!texts.join("\n").contains("current-"), "{texts:?}");
        } else {
            let addr = spawn_sse(push.clone(), "acct1", source.clone()).await;
            let mut sock = open_sse(addr).await;
            overflow_with_own_change_lost(&push);
            push.send(change("acct1", "after"));
            let (body, ended) = sse_read(&mut sock, Duration::from_secs(5), "after").await;
            assert!(
                !ended && body.contains("after") && !body.contains("current-"),
                "{body}"
            );
        }
        assert!(
            source.asked().is_empty(),
            "nothing is read with the switch off"
        );
    }
}

// ── a resync that cannot read closes the stream ──────────────────────────────

#[tokio::test]
async fn a_failed_resync_closes_ws_with_1011_and_ends_sse() {
    let failures_before = snap(&["resyncFailures"]);
    let push = handle(true);
    let addr = spawn_ws(push.clone(), "acct1", Source::failing()).await;
    let mut ws = open_ws(addr).await;
    overflow_with_own_change_lost(&push);
    let (texts, closed) = ws_read(&mut ws, Duration::from_secs(5), "").await;
    assert_eq!(closed, Some(1011), "{texts:?}");
    assert!(!texts.join("\n").contains("dropped"));

    let push = handle(true);
    let addr = spawn_sse(push.clone(), "acct1", Source::failing()).await;
    let mut sock = open_sse(addr).await;
    overflow_with_own_change_lost(&push);
    let (body, ended) = sse_read(&mut sock, Duration::from_secs(5), "").await;
    assert!(ended, "the SSE body ends: {body}");
    assert!(snap(&["resyncFailures"]) >= failures_before + 2);
}

/// A stream with no state source cannot catch up, so a gap closes it rather
/// than continuing as if nothing was lost.
#[tokio::test]
async fn a_stream_without_a_source_closes_on_a_gap() {
    let push = handle(true);
    let mut changes = ScopedChanges::new(push.subscribe(), vec!["acct1".into()]);
    overflow_with_own_change_lost(&push);
    let Err(broadcast::error::RecvError::Lagged(n)) = changes.recv().await else {
        panic!("the receiver lagged");
    };
    assert!(n > 0);
    assert!(
        !changes.lagged(Consumer::Ws, n).await,
        "close, do not continue"
    );
}

/// A source that answers for a different account than asked is refused: the
/// stream closes and nothing of the other account is written.
#[tokio::test]
async fn a_read_naming_another_account_is_refused() {
    let push = handle(true);
    let wrong: ReadFn = Arc::new(|_account| Box::pin(async { Ok(change("acct2", "foreign")) }));
    let mut changes =
        ScopedChanges::new(push.subscribe(), vec!["acct1".into()]).attach(&push, Some(wrong));
    assert!(!changes.catch_up(ResyncReason::Lag).await);
    assert!(changes.pending.is_empty(), "{:?}", changes.pending);
}

// ── resync_all: every stream resyncs ─────────────────────────────────────────

#[tokio::test]
async fn bridge_lag_makes_every_stream_resync() {
    let push = handle(true);
    let source = Source::default();
    let mut resync = push.subscribe_resync();
    let signals_before = snap(&["resyncSignals", "bridge"]);
    let a = spawn_ws(push.clone(), "acct1", source.clone()).await;
    let b = spawn_sse(push.clone(), "acct3", source.clone()).await;
    let mut ws = open_ws(a).await;
    let mut sse = open_sse(b).await;

    // The engine's channel is small; the bridge task cannot run during the
    // sends, so it lags.
    let (engine_tx, engine_rx) = broadcast::channel(4);
    tokio::spawn(bridge_engine(engine_rx, push.clone()));
    tokio::task::yield_now().await;
    for i in 0..16 {
        engine_tx
            .send(change("engine-only", &i.to_string()))
            .unwrap();
    }

    let (texts, _) = ws_read(&mut ws, Duration::from_secs(5), "current-acct1").await;
    assert!(texts.last().unwrap().contains("current-acct1"), "{texts:?}");
    let (body, _) = sse_read(&mut sse, Duration::from_secs(5), "current-acct3").await;
    assert!(
        body.contains("current-acct3") && !body.contains("acct1"),
        "{body}"
    );
    resync.changed().await.unwrap();
    assert_eq!(resync.borrow().reason, ResyncReason::Bridge);
    assert!(snap(&["resyncSignals", "bridge"]) > signals_before);
    assert!(snap(&["lagged", "bridge"]) > 0);
}

// ── duplicate suppression ────────────────────────────────────────────────────

#[tokio::test]
async fn identical_frames_are_sent_once_per_account() {
    let before = snap(&["duplicatesSuppressed"]);
    let push = handle(true);
    let source = Source::default();
    let mut changes = scoped(&push, "acct1", &source);
    changes.admit(Incoming::Own(change("acct1", "5"))).await;
    changes.admit(Incoming::Own(change("acct1", "5"))).await;
    changes.admit(Incoming::Own(change("acct1", "6"))).await;
    changes.admit(Incoming::Own(change("acct1", "5"))).await;
    let frames: Vec<String> = changes.pending.drain(..).collect();
    assert_eq!(frames.len(), 3, "only the repeat is dropped: {frames:?}");
    assert!(snap(&["duplicatesSuppressed"]) > before);

    // A resync that finds the state the stream last sent writes nothing.
    changes
        .admit(Incoming::Own(change("acct1", "current-acct1")))
        .await;
    changes.pending.clear();
    assert!(changes.catch_up(ResyncReason::Lag).await);
    assert!(changes.pending.is_empty(), "{:?}", changes.pending);
}

#[tokio::test]
async fn dedupe_off_sends_every_frame() {
    let push = PushHandle::with_config(RealtimeConfig {
        dedupe: false,
        ..RealtimeConfig::default()
    });
    let mut changes = scoped(&push, "acct1", &Source::default());
    changes.admit(Incoming::Own(change("acct1", "5"))).await;
    changes.admit(Incoming::Own(change("acct1", "5"))).await;
    assert_eq!(changes.pending.len(), 2);
}

// ── shutdown ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn begin_shutdown_closes_ws_with_1012_and_ends_sse() {
    let before = snap(&["shutdownCloses"]);
    let push = handle(true);
    let ws_addr = spawn_ws(push.clone(), "acct1", Source::default()).await;
    let sse_addr = spawn_sse(push.clone(), "acct1", Source::default()).await;
    let mut ws = open_ws(ws_addr).await;
    let mut sse = open_sse(sse_addr).await;
    assert!(!push.shutting_down());

    push.begin_shutdown();

    let (_, code) = ws_read(&mut ws, Duration::from_secs(5), "").await;
    assert_eq!(code, Some(1012));
    let (body, ended) = sse_read(&mut sse, Duration::from_secs(5), "").await;
    assert!(ended, "{body}");
    // A stream opened afterwards closes at once.
    let mut late = open_ws(ws_addr).await;
    let (_, code) = ws_read(&mut late, Duration::from_secs(5), "").await;
    assert_eq!(code, Some(1012));
    assert!(push.shutting_down());
    assert!(snap(&["shutdownCloses"]) >= before + 3);
}

// ── the single-flight read rule ──────────────────────────────────────────────

/// A read that started before a requester's gap never answers it; one that has
/// not started yet, or started after the gap, may.
#[tokio::test]
async fn a_shared_read_never_predates_the_requesters_gap() {
    let hub = Arc::new(Hub::new(4));
    let calls = Arc::new(AtomicUsize::new(0));
    let release = Arc::new(tokio::sync::Notify::new());
    let (started_tx, mut started_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    *hub.on_start.lock().unwrap() = Some(Arc::new(move |account: &str| {
        let _ = started_tx.send(account.to_string());
    }));
    let source: ReadFn = {
        let (calls, release) = (calls.clone(), release.clone());
        Arc::new(move |account: String| {
            let n = calls.fetch_add(1, Ordering::SeqCst) + 1;
            let release = release.clone();
            Box::pin(async move {
                release.notified().await;
                Ok(change(&account, &format!("read-{n}")))
            })
        })
    };
    let wait = Duration::from_secs(5);

    // A: gap g1, starts read 1, which blocks after its start position.
    let g1 = hub.gap();
    let a = tokio::spawn({
        let (hub, source) = (hub.clone(), source.clone());
        async move { hub.read(&source, "acct1", g1, wait).await }
    });
    assert_eq!(started_rx.recv().await.unwrap(), "acct1");
    // B: a gap detected after read 1 started must not be answered by it.
    let g2 = hub.gap();
    let b = tokio::spawn({
        let (hub, source) = (hub.clone(), source.clone());
        async move { hub.read(&source, "acct1", g2, wait).await }
    });
    assert_eq!(
        started_rx.recv().await.unwrap(),
        "acct1",
        "B started its own read"
    );
    // C: g1 predates read 2's start too, so C may join the newest read (2).
    let c = tokio::spawn({
        let (hub, source) = (hub.clone(), source.clone());
        async move { hub.read(&source, "acct1", g1, wait).await }
    });
    tokio::task::yield_now().await;
    assert_eq!(calls.load(Ordering::SeqCst), 2, "C shares; no third read");
    release.notify_waiters();
    let (a, b, c) = (
        a.await.unwrap().unwrap(),
        b.await.unwrap().unwrap(),
        c.await.unwrap().unwrap(),
    );
    assert_eq!(a.email, "read-1");
    assert_eq!(
        b.email, "read-2",
        "B is answered by the read that started after its gap"
    );
    assert_eq!(c.email, "read-2");
}

/// Reads not yet started (waiting for a permit) are shared by any requester:
/// they will start after every requester's gap.
#[tokio::test]
async fn a_queued_read_is_shared_and_concurrency_is_bounded() {
    let hub = Arc::new(Hub::new(1));
    let calls = Arc::new(AtomicUsize::new(0));
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    let source: ReadFn = {
        let (calls, release) = (calls.clone(), release.clone());
        Arc::new(move |account: String| {
            calls.fetch_add(1, Ordering::SeqCst);
            let release = release.clone();
            Box::pin(async move {
                release.acquire().await.unwrap().forget();
                Ok(change(&account, "x"))
            })
        })
    };
    let wait = Duration::from_secs(5);
    let mut tasks = Vec::new();
    for account in ["a", "b", "b", "b"] {
        let (hub, source) = (hub.clone(), source.clone());
        let gap = hub.gap();
        tasks.push(tokio::spawn(async move {
            hub.read(&source, account, gap, wait).await
        }));
    }
    for _ in 0..5 {
        tokio::task::yield_now().await;
    }
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "one permit: only a's read runs"
    );
    release.add_permits(2);
    for t in tasks {
        t.await.unwrap().unwrap();
    }
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "b's three requesters share one read"
    );
}

#[tokio::test]
async fn a_read_that_outlasts_its_deadline_is_a_failure() {
    let hub = Hub::new(1);
    let source: ReadFn = Arc::new(|_| Box::pin(std::future::pending()));
    let gap = hub.gap();
    let out = hub
        .read(&source, "acct1", gap, Duration::from_millis(50))
        .await;
    assert!(out.is_err());
    assert!(hub.idle(), "nothing left behind");
}

// ── interest set ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn interest_counts_open_streams_per_account() {
    let push = handle(true);
    let source = Source::default();
    let a = scoped(&push, "acct1", &source);
    let b = scoped(&push, "acct1", &source);
    let c = scoped(&push, "acct2", &source);
    assert_eq!(push.interest_accounts(), vec!["acct1", "acct2"]);
    drop(a);
    assert_eq!(push.interest_accounts(), vec!["acct1", "acct2"]);
    drop(b);
    assert_eq!(push.interest_accounts(), vec!["acct2"]);
    drop(c);
    assert!(push.interest_accounts().is_empty());
}

// ── switches ─────────────────────────────────────────────────────────────────

#[test]
fn switches_default_and_refuse_invalid_values() {
    let parse = |pairs: &[(&str, &str)]| {
        let pairs: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        parse_config(move |name| {
            pairs
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.clone())
        })
    };
    assert_eq!(parse(&[]).unwrap(), RealtimeConfig::default());
    assert_eq!(
        RealtimeConfig::default().fanout_capacity,
        256,
        "today's capacity"
    );
    let set = parse(&[
        (FANOUT_ENV, "1024"),
        (LAG_RESYNC_ENV, "off"),
        (DEDUPE_ENV, " off "),
        (RESYNC_CONCURRENCY_ENV, "1"),
    ])
    .unwrap();
    assert_eq!(
        set,
        RealtimeConfig {
            fanout_capacity: 1024,
            lag_resync: false,
            dedupe: false,
            resync_concurrency: 1,
        }
    );
    for (name, bad) in [
        (FANOUT_ENV, "63"),
        (FANOUT_ENV, "65537"),
        (FANOUT_ENV, "lots"),
        (LAG_RESYNC_ENV, "yes"),
        (DEDUPE_ENV, "0"),
        (RESYNC_CONCURRENCY_ENV, "0"),
        (RESYNC_CONCURRENCY_ENV, "1025"),
    ] {
        let err = parse(&[(name, bad)]).unwrap_err();
        assert!(err.contains(name), "the error names the variable: {err}");
    }
}
