//! t20-e11 — the trusted-proxy model, driven through a real server, no containers.
//!
//! This is the security half of the t20 verification lane. Everything here runs in
//! the normal `cargo test` gate: a real `axum::serve` on a real socket, driven with
//! a real HTTP client, with the forwarded-header posture set through the same
//! environment variables an operator would set.
//!
//! ## Why this file has to exist even though CI boots six proxies
//!
//! Every tier-1 proxy in `docker-compose.proxy.yml` **overwrites** `X-Forwarded-For`
//! with the peer address rather than appending to it (t20-e7 confirmed this against
//! all eight config trees). A container cell therefore hands the app a single-hop
//! header and never exercises the right-to-left walk at all. Envoy is the one
//! exception and it is deliberately out of the CI matrix. So the hop-skipping logic
//! — the part of `crates/mw-server/src/proxy.rs` that decides *which* hop is the
//! client — is tested here or nowhere.
//!
//! ## The trap this file is written to avoid
//!
//! `ConnectInfo` is absent from every pre-t20 test harness in this crate: they call
//! `axum::serve(listener, app)` with a plain `Router`. With no peer address
//! `proxy::client_ip` returns `None`, forwarded headers are never trusted, and
//! header auth refuses — all fail-closed. A suite written that way would assert
//! "the forged header was refused" and go green **with the whole feature deleted**.
//!
//! Both t20-e1 and t20-e3 flagged this explicitly. Two things answer it:
//!
//!   * the harness installs `into_make_service_with_connect_info::<SocketAddr>()`,
//!     exactly as `main.rs` does; and
//!   * [`the_harness_installs_connect_info`] is a **positive** control that fails
//!     if the wiring is absent, and pins its own teeth by showing the same request
//!     against a connect-info-less server is refused.
//!
//! Every trust assertion below has a matching positive, so "correctly refused" is
//! always distinguishable from "never wired".
//!
//! ## How the resolved address is observed
//!
//! Through the OAuth scoped-key IP allowlist (`mw_oauth::enforce::ip_allowed`, fed
//! by `scope_mw::client_ip`) — one of the three things B1 actually broke. A key
//! whose allowlist is `X/32` returns `200` iff the server resolved the client to
//! `X`, and `403` otherwise. That is an exact read-out of the resolved address, not
//! a proxy for one, and it is the production consumer rather than a test seam.
//!
//! Run: `cargo test -p mw-server --test t20_forwarded_spoof -- --test-threads=1`

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::OnceLock;
use std::time::Duration;

use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, MutexGuard};

use axum::Router;
use axum::serve::ListenerExt;
use mw_server::tls::proxy_protocol::{self, ProxyAcceptor};
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};

mod common;
use common::test_db;

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T20</div>";

// ---------------------------------------------------------------------------
// Environment posture
// ---------------------------------------------------------------------------

/// The forwarded/auth posture a test wants. Every field is applied on every call —
/// including the empty ones, which are *removed* — so no test can inherit a
/// neighbour's configuration. That matters because `proxy::resolve` reads the
/// environment per request and the environment is process-global.
#[derive(Default)]
struct Posture {
    /// `MW_TRUSTED_PROXIES`.
    trusted: &'static str,
    /// `MW_FORWARDED_MODE`: `off` (or empty) | `xff` | `forwarded`.
    mode: &'static str,
    /// `MW_PROXY_PROTOCOL`: empty (off) | `accept` | `require`.
    proxy_protocol: &'static str,
    /// `MW_HEADER_AUTH`: empty (off) | `1`.
    header_auth: &'static str,
    /// `MW_HEADER_AUTH_TRUSTED_IPS`.
    header_auth_ips: &'static str,
}

/// Serialises env mutation across the binary, so the suite is correct under any
/// `--test-threads`. A `tokio` mutex (not a `std` one) because it is held across
/// the awaits that drive the requests the posture applies to.
fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(Default::default)
}

/// Take the env lock and install `p`. The returned guard must outlive every request
/// the test makes.
async fn posture(p: Posture) -> MutexGuard<'static, ()> {
    let guard = env_lock().lock().await;
    // SAFETY: the guard serialises every mutation and every read that depends on
    // one within this binary. Matches the house pattern (`t16_sandbox.rs`,
    // `t17_mcp_audience.rs`, `v6_mount.rs`).
    unsafe {
        set_or_clear("MW_TRUSTED_PROXIES", p.trusted);
        set_or_clear("MW_FORWARDED_MODE", p.mode);
        set_or_clear("MW_PROXY_PROTOCOL", p.proxy_protocol);
        set_or_clear("MW_HEADER_AUTH", p.header_auth);
        set_or_clear("MW_HEADER_AUTH_TRUSTED_IPS", p.header_auth_ips);
        // Never inherited from the ambient environment: these would silently change
        // what the scheme/cookie assertions below mean.
        for k in [
            "MW_PUBLIC_URL",
            "MW_COOKIE_SECURE",
            "MW_HSTS",
            "MW_HSTS_MAX_AGE",
            "MW_HEADER_AUTH_HEADER",
            "MW_HEADER_AUTH_PASSWORD",
            "MW_HEADER_AUTH_JMAP_URL",
        ] {
            std::env::remove_var(k);
        }
    }
    guard
}

/// # Safety
/// Caller holds [`env_lock`].
unsafe fn set_or_clear(key: &str, value: &str) {
    unsafe {
        if value.is_empty() {
            std::env::remove_var(key);
        } else {
            std::env::set_var(key, value);
        }
    }
}

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

async fn build() -> Router {
    let base = test_db::unique_dir("mw-t20-spoof");
    let web_dir = base.join("web");
    std::fs::create_dir_all(&web_dir).unwrap();
    std::fs::write(web_dir.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: base.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(web_dir),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let v6 = V6Config {
        admin_enabled: false,
        admin_username: None,
        admin_password: None,
        redis_url: None,
    };
    build_app_full(config, v6).await.unwrap().0
}

/// Serve `app` the way `main.rs` does — **with** connect info, so the peer address
/// reaches `proxy::peer_ip`.
async fn serve_with_connect_info(app: Router) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    addr
}

/// Serve `app` the way every pre-t20 harness in this crate does — **without**
/// connect info. Used only to prove the positive assertions have teeth.
async fn serve_without_connect_info(app: Router) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

/// Serve `app` behind a [`ProxyAcceptor`], the way `main.rs` does when
/// `MW_PROXY_PROTOCOL` is set. `tap_io` is the adapter that makes axum's blanket
/// `Connected` impl apply to a custom listener (t20-e1's TLS wrinkle, which t20-e2
/// hit verbatim); without it `ConnectInfo` would not be installed here either.
async fn serve_behind_proxy_protocol(app: Router) -> SocketAddr {
    let tcp = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let listener = ProxyAcceptor::new(tcp, proxy_protocol::Config::from_env()).unwrap();
    let addr = axum::serve::Listener::local_addr(&listener).unwrap();
    let listener = listener.tap_io(|_| {});
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    addr
}

/// A logged-in session against a booted server, able to mint scoped keys.
struct Session {
    base: String,
    client: reqwest::Client,
    account_id: String,
    /// Every cookie the login set, as one `Cookie:` header value — the raw-socket
    /// legs cannot use the cookie store.
    cookie_header: String,
}

impl Session {
    async fn open(addr: SocketAddr, mock: &str) -> Self {
        let base = format!("http://{addr}");
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();
        let resp = client
            .post(format!("{base}/api/login"))
            .json(&json!({
                "jmapUrl": mock,
                "username": mw_mock_jmap::USER,
                "password": mw_mock_jmap::PASS,
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "login through the harness must succeed");
        let cookie_header = resp
            .headers()
            .get_all(reqwest::header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .filter_map(|v| v.split(';').next())
            .collect::<Vec<_>>()
            .join("; ");
        let body: Value = resp.json().await.unwrap();
        let account_id = body["accountId"].as_str().unwrap().to_string();
        Self {
            base,
            client,
            account_id,
            cookie_header,
        }
    }

    /// Mint a read-only mail key restricted to `allowlist`, and return its token.
    async fn key_for(&self, allowlist: &[&str]) -> String {
        let scope = json!({
            "read": true, "send": false, "delete": false,
            "accounts": { "subset": [self.account_id] }, "folders": "all",
            "mail": true, "pim": false,
            "ip_allowlist": allowlist,
            "expires_at": null, "rate_limit": null,
            "mcp_tools": [], "unattended_send": false,
        });
        let minted: Value = self
            .client
            .post(format!("{}/api/keys", self.base))
            .json(&json!({ "label": "t20-e11", "accountId": self.account_id, "scope": scope }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        minted["displayToken"]
            .as_str()
            .expect("a minted key shows its token once")
            .to_string()
    }

    /// `GET /api/v1/messages` with `key`, carrying `headers` verbatim. `200` means
    /// the server resolved the client inside the key's allowlist; `403` means it did
    /// not.
    async fn probe(&self, key: &str, headers: &[(&str, &str)]) -> u16 {
        let mut req = self
            .client
            .get(format!("{}/api/v1/messages", self.base))
            .header("x-api-key", key);
        for (k, v) in headers {
            req = req.header(*k, *v);
        }
        req.send().await.unwrap().status().as_u16()
    }

    /// Assert the server resolved the client to `expected` and to nothing else, by
    /// admitting a key allowlisted to it and refusing one allowlisted elsewhere.
    async fn assert_resolves_to(&self, expected: &str, decoy: &str, headers: &[(&str, &str)]) {
        let allowed = self.key_for(&[expected]).await;
        assert_eq!(
            self.probe(&allowed, headers).await,
            200,
            "expected the client to resolve to {expected} (headers: {headers:?})"
        );
        let refused = self.key_for(&[decoy]).await;
        assert_eq!(
            self.probe(&refused, headers).await,
            403,
            "the client must NOT resolve to {decoy} (headers: {headers:?})"
        );
    }
}

// ---------------------------------------------------------------------------
// Raw-socket helpers (PROXY protocol)
// ---------------------------------------------------------------------------

/// Write `prefix` then `request` to `addr` and read the whole reply.
///
/// `None` means the connection produced **no bytes at all** — which is precisely
/// how `proxy_protocol::negotiate` refuses an untrusted sender: it drops the
/// `TcpStream` rather than answering. The request always asks for `Connection:
/// close`, so a served request terminates the read at EOF.
async fn raw_exchange(addr: SocketAddr, prefix: &[u8], request: &str) -> Option<String> {
    let mut sock = TcpStream::connect(addr).await.ok()?;
    sock.write_all(prefix).await.ok()?;
    sock.write_all(request.as_bytes()).await.ok()?;
    let mut buf = Vec::new();
    // A refused connection closes immediately; a served one closes after the body.
    // The timeout only fires if the server neither answers nor closes, which is
    // itself a failure — report it as "no response".
    let _ = tokio::time::timeout(Duration::from_secs(10), sock.read_to_end(&mut buf)).await;
    if buf.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(&buf).into_owned())
    }
}

/// A PROXY protocol v1 line (the ASCII form).
fn proxy_v1(src: SocketAddrV4, dst: SocketAddrV4) -> Vec<u8> {
    format!(
        "PROXY TCP4 {} {} {} {}\r\n",
        src.ip(),
        dst.ip(),
        src.port(),
        dst.port()
    )
    .into_bytes()
}

/// A PROXY protocol v2 frame for an IPv4 stream connection — the shape HAProxy's
/// `send-proxy-v2` emits.
fn proxy_v2(src: SocketAddrV4, dst: SocketAddrV4) -> Vec<u8> {
    let mut b = vec![
        0x0D, 0x0A, 0x0D, 0x0A, 0x00, 0x0D, 0x0A, 0x51, 0x55, 0x49, 0x54, 0x0A,
    ];
    b.push(0x21); // version 2, command PROXY
    b.push(0x11); // AF_INET, SOCK_STREAM
    b.extend_from_slice(&12u16.to_be_bytes()); // address block length
    b.extend_from_slice(&src.ip().octets());
    b.extend_from_slice(&dst.ip().octets());
    b.extend_from_slice(&src.port().to_be_bytes());
    b.extend_from_slice(&dst.port().to_be_bytes());
    b
}

fn v4(addr: &str, port: u16) -> SocketAddrV4 {
    SocketAddrV4::new(addr.parse::<Ipv4Addr>().unwrap(), port)
}

/// A complete HTTP/1.1 request head, with `Connection: close` so the reply is
/// terminated by EOF.
fn http_get(addr: SocketAddr, path: &str, headers: &[(&str, &str)]) -> String {
    let extra: String = headers
        .iter()
        .map(|(k, v)| format!("{k}: {v}\r\n"))
        .collect();
    format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n{extra}\r\n")
}

/// The status line's numeric code.
fn status_of(response: &str) -> u16 {
    response
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

// ===========================================================================
// 1. The control: is the model wired at all?
// ===========================================================================

/// **The positive control this whole file rests on.** With connect info installed
/// the peer address is a real address, so a key allowlisted to loopback is admitted
/// and one allowlisted elsewhere is refused.
///
/// The second half is what gives that teeth: the identical key against a server
/// served the pre-t20 way (`axum::serve(listener, app)`) is **refused**, because
/// `client_ip` is `None` and `ip_allowed` denies on an unknown source. So a green
/// first half genuinely means "the wiring is present", not "everything is refused".
#[tokio::test]
async fn the_harness_installs_connect_info() {
    let _env = posture(Posture {
        trusted: "127.0.0.0/8, ::1/128",
        mode: "xff",
        ..Default::default()
    })
    .await;
    let mock = spawn_mock().await;

    let wired = Session::open(serve_with_connect_info(build().await).await, &mock).await;
    let loopback = wired.key_for(&["127.0.0.0/8", "::1/128"]).await;
    assert_eq!(
        wired.probe(&loopback, &[]).await,
        200,
        "with ConnectInfo the peer address is the client IP"
    );
    let elsewhere = wired.key_for(&["203.0.113.0/24"]).await;
    assert_eq!(
        wired.probe(&elsewhere, &[]).await,
        403,
        "the allowlist is genuinely enforced, not a no-op"
    );

    // The teeth. Same posture, same key shape, no ConnectInfo.
    let blind = Session::open(serve_without_connect_info(build().await).await, &mock).await;
    let loopback = blind.key_for(&["127.0.0.0/8", "::1/128"]).await;
    assert_eq!(
        blind.probe(&loopback, &[]).await,
        403,
        "without ConnectInfo there is no source IP, so every allowlist denies — \
         which is why a suite that only asserts refusals proves nothing"
    );
}

// ===========================================================================
// 2. X-Forwarded-For trust
// ===========================================================================

/// An untrusted peer's forwarded header is its own claim about itself, and is
/// ignored entirely. The peer address wins in both directions.
#[tokio::test]
async fn an_untrusted_peer_cannot_name_its_own_address() {
    let _env = posture(Posture {
        // Loopback — where the test client connects from — is deliberately NOT here.
        trusted: "10.0.0.0/8",
        mode: "xff",
        ..Default::default()
    })
    .await;
    let mock = spawn_mock().await;
    let s = Session::open(serve_with_connect_info(build().await).await, &mock).await;

    s.assert_resolves_to(
        "127.0.0.0/8",
        "8.8.8.8/32",
        &[("x-forwarded-for", "8.8.8.8")],
    )
    .await;

    // Not even by naming an address inside the trusted range: trust is decided by
    // who is connecting, never by what the connection claims.
    s.assert_resolves_to(
        "127.0.0.0/8",
        "10.0.0.0/8",
        &[("x-forwarded-for", "10.1.2.3")],
    )
    .await;
}

/// **Positive:** a peer inside `MW_TRUSTED_PROXIES` is believed, and the address it
/// reports *replaces* the peer address rather than sitting alongside it.
#[tokio::test]
async fn a_trusted_proxys_forwarded_header_is_honoured() {
    let _env = posture(Posture {
        trusted: "127.0.0.0/8, ::1/128",
        mode: "xff",
        ..Default::default()
    })
    .await;
    let mock = spawn_mock().await;
    let s = Session::open(serve_with_connect_info(build().await).await, &mock).await;

    // The decoy is loopback itself: if the header were merely *added* to the peer
    // address rather than replacing it, this second leg would pass and the walk
    // would be untested.
    s.assert_resolves_to(
        "203.0.113.9/32",
        "127.0.0.0/8",
        &[("x-forwarded-for", "203.0.113.9")],
    )
    .await;
}

/// The right-to-left walk: skip hops that are themselves trusted proxies, stop at
/// the first that is not, and never reach anything to its left.
///
/// This is the case **no CI cell can reach** — every tier-1 proxy overwrites
/// `X-Forwarded-For`, so a container run only ever delivers one hop.
#[tokio::test]
async fn the_walk_stops_at_the_first_untrusted_hop() {
    let _env = posture(Posture {
        // Loopback is the connecting proxy; 10.0.0.0/8 is an inner tier.
        trusted: "127.0.0.0/8, ::1/128, 10.0.0.0/8",
        mode: "xff",
        ..Default::default()
    })
    .await;
    let mock = spawn_mock().await;
    let s = Session::open(serve_with_connect_info(build().await).await, &mock).await;

    // `1.1.1.1` is what the client wrote; `203.0.113.9` is what the outermost
    // trusted proxy actually saw; `10.0.0.2` is an inner hop that gets skipped.
    let chain = [("x-forwarded-for", "1.1.1.1, 203.0.113.9, 10.0.0.2")];
    s.assert_resolves_to("203.0.113.9/32", "1.1.1.1/32", &chain)
        .await;

    // A skipped trusted hop must not become the answer either.
    let inner = s.key_for(&["10.0.0.0/8"]).await;
    assert_eq!(
        s.probe(&inner, &chain).await,
        403,
        "a trusted intermediate hop is skipped, not selected"
    );

    // Longer forged prefix, same answer — the client cannot promote its choice by
    // adding hops.
    s.assert_resolves_to(
        "203.0.113.9/32",
        "8.8.8.8/32",
        &[("x-forwarded-for", "8.8.8.8, 1.1.1.1, 203.0.113.9")],
    )
    .await;

    // An unusable hop stops the walk at the last *verified* address rather than
    // being skipped over into client-written territory.
    s.assert_resolves_to(
        "10.0.0.2/32",
        "203.0.113.9/32",
        &[("x-forwarded-for", "203.0.113.9, unknown, 10.0.0.2")],
    )
    .await;
}

/// `MW_TRUSTED_PROXIES` alone is not consent: without a mode the header is not read.
/// (The inverse — a mode with no usable network — is covered by `proxy.rs`'s unit
/// tests and warns at startup.)
#[tokio::test]
async fn a_trusted_proxy_without_a_mode_changes_nothing() {
    let _env = posture(Posture {
        trusted: "127.0.0.0/8, ::1/128",
        mode: "", // MW_FORWARDED_MODE absent — the default posture
        ..Default::default()
    })
    .await;
    let mock = spawn_mock().await;
    let s = Session::open(serve_with_connect_info(build().await).await, &mock).await;

    s.assert_resolves_to(
        "127.0.0.0/8",
        "203.0.113.9/32",
        &[("x-forwarded-for", "203.0.113.9")],
    )
    .await;
}

/// Exactly one header is read per mode, so a client behind a proxy that rewrites
/// one of them cannot switch to the other.
#[tokio::test]
async fn a_mode_reads_its_own_header_and_no_other() {
    let _env = posture(Posture {
        trusted: "127.0.0.0/8, ::1/128",
        mode: "forwarded",
        ..Default::default()
    })
    .await;
    let mock = spawn_mock().await;
    let s = Session::open(serve_with_connect_info(build().await).await, &mock).await;

    // RFC 7239 is read in this mode …
    s.assert_resolves_to(
        "203.0.113.9/32",
        "127.0.0.0/8",
        &[("forwarded", "for=203.0.113.9;proto=https")],
    )
    .await;

    // … and X-Forwarded-For is not, even from the same trusted peer.
    s.assert_resolves_to(
        "127.0.0.0/8",
        "8.8.8.8/32",
        &[("x-forwarded-for", "8.8.8.8")],
    )
    .await;
}

// ===========================================================================
// 3. Header authentication (B2)
// ===========================================================================

/// `MW_HEADER_AUTH` mints a session from an asserted identity with no password, so
/// the gate is on **who is connecting** — the socket peer — and fails closed on
/// every uncertainty.
///
/// The attack in the third leg is the one t20-e1 corrected itself on mid-wave: if
/// the gate consulted `client_ip` instead of `peer_ip`, anyone behind a trusted
/// forwarding tier could send `X-Forwarded-For: <allowlisted>` and authenticate as
/// any user.
#[tokio::test]
async fn header_auth_is_restricted_to_the_connecting_peer() {
    let mock = spawn_mock().await;

    // ── refuse: the peer is outside the allowlist ────────────────────────────
    {
        let _env = posture(Posture {
            header_auth: "1",
            header_auth_ips: "10.0.0.0/8",
            ..Default::default()
        })
        .await;
        let addr = serve_with_connect_info(build().await).await;
        assert_eq!(
            header_auth_login(addr, &mock, &[]).await,
            401,
            "a peer outside MW_HEADER_AUTH_TRUSTED_IPS may not assert an identity"
        );
    }

    // ── refuse: the feature is on but no allowlist is configured ─────────────
    {
        let _env = posture(Posture {
            header_auth: "1",
            header_auth_ips: "",
            ..Default::default()
        })
        .await;
        let addr = serve_with_connect_info(build().await).await;
        assert_eq!(
            header_auth_login(addr, &mock, &[]).await,
            401,
            "an unset allowlist must authenticate nobody, not everybody"
        );
    }

    // ── refuse: a forwarded header cannot stand in for the peer ──────────────
    {
        let _env = posture(Posture {
            trusted: "127.0.0.0/8, ::1/128",
            mode: "xff",
            header_auth: "1",
            header_auth_ips: "10.0.0.0/8",
            ..Default::default()
        })
        .await;
        let addr = serve_with_connect_info(build().await).await;
        assert_eq!(
            header_auth_login(addr, &mock, &[("x-forwarded-for", "10.1.2.3")]).await,
            401,
            "identity assertion is gated on the hop that made it, not on a header \
             that hop forwarded"
        );
    }

    // ── refuse: no peer address at all ───────────────────────────────────────
    {
        let _env = posture(Posture {
            header_auth: "1",
            header_auth_ips: "127.0.0.0/8, ::1/128",
            ..Default::default()
        })
        .await;
        let addr = serve_without_connect_info(build().await).await;
        assert_eq!(
            header_auth_login(addr, &mock, &[]).await,
            401,
            "with no ConnectInfo there is nothing to check the assertion against"
        );
    }

    // ── POSITIVE: an allowlisted peer mints the asserted user's session ───────
    {
        let _env = posture(Posture {
            header_auth: "1",
            header_auth_ips: "127.0.0.0/8, ::1/128",
            ..Default::default()
        })
        .await;
        let addr = serve_with_connect_info(build().await).await;
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();
        let resp = client
            .post(format!("http://{addr}/api/login"))
            .header("x-remote-user", "asserted@example.org")
            .json(&json!({ "jmapUrl": mock, "username": "", "password": "" }))
            .send()
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            200,
            "an allowlisted peer's assertion is honoured — without this the four \
             refusals above would pass with the feature deleted"
        );
        let body: Value = resp.json().await.unwrap();
        assert_eq!(
            body["accountId"], "asserted@example.org",
            "the session is minted for the ASSERTED identity"
        );
    }
}

/// `POST /api/login` carrying an `X-Remote-User` assertion; returns the status.
async fn header_auth_login(addr: SocketAddr, mock: &str, headers: &[(&str, &str)]) -> u16 {
    let client = reqwest::Client::new();
    let mut req = client
        .post(format!("http://{addr}/api/login"))
        .header("x-remote-user", "asserted@example.org")
        .json(&json!({ "jmapUrl": mock, "username": "", "password": "" }));
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    req.send().await.unwrap().status().as_u16()
}

// ===========================================================================
// 4. PROXY protocol (t20-e2)
// ===========================================================================

/// A PROXY header from a peer outside `MW_TRUSTED_PROXIES` is not ignored — the
/// connection is **closed**, in both wire versions, and whether or not the header
/// names an address inside the trusted range.
///
/// The assertion is "no bytes came back", which is stronger and more honest than
/// "the address was not used": a server that answered would have had to decide what
/// to do with the claim.
#[tokio::test]
async fn a_forged_proxy_protocol_header_gets_no_response() {
    let _env = posture(Posture {
        // Loopback, where this test connects from, is NOT trusted.
        trusted: "10.0.0.0/8",
        proxy_protocol: "accept",
        ..Default::default()
    })
    .await;
    let addr = serve_behind_proxy_protocol(build().await).await;
    let dst = v4("127.0.0.1", addr.port());
    let req = http_get(addr, "/healthz", &[]);

    for (label, header) in [
        ("v1", proxy_v1(v4("198.51.100.7", 51234), dst)),
        ("v2", proxy_v2(v4("198.51.100.7", 51234), dst)),
        // …including one that names an address inside the trusted range.
        (
            "v2 inside the trusted range",
            proxy_v2(v4("10.1.2.3", 51234), dst),
        ),
    ] {
        assert!(
            raw_exchange(addr, &header, &req).await.is_none(),
            "{label}: a forged PROXY header must close the connection, not be answered"
        );
    }

    // The control: the same server, same posture, no PROXY header — an ordinary
    // direct client under `accept` is served normally.
    let reply = raw_exchange(addr, b"", &req)
        .await
        .expect("accept mode serves a headerless client");
    assert_eq!(status_of(&reply), 200, "reply was: {reply}");
}

/// **Positive:** a header from a *trusted* sender is believed, and the address it
/// carries reaches the scoped-key IP allowlist — not merely `ConnectInfo`.
///
/// t20-e2's own end-to-end leg stops at `ConnectInfo`; this closes the remaining
/// half of the claim.
#[tokio::test]
async fn a_believed_proxy_protocol_address_reaches_the_ip_allowlist() {
    let _env = posture(Posture {
        trusted: "127.0.0.0/8, ::1/128",
        proxy_protocol: "accept",
        ..Default::default()
    })
    .await;
    let mock = spawn_mock().await;
    let app = build().await;

    // Two listeners over ONE app: a plain one to log in and mint keys with a normal
    // client, and the PROXY-protocol one the raw legs drive. Same store, same
    // session — so the cookie minted on the first is valid on the second.
    let plain = serve_with_connect_info(app.clone()).await;
    let pp = serve_behind_proxy_protocol(app).await;
    let s = Session::open(plain, &mock).await;

    let allowed = s.key_for(&["198.51.100.0/24"]).await;
    let refused = s.key_for(&["127.0.0.0/8"]).await;
    let dst = v4("127.0.0.1", pp.port());
    let header = proxy_v2(v4("198.51.100.7", 51234), dst);

    let probe = |key: String, prefix: Vec<u8>| {
        let cookie = s.cookie_header.clone();
        async move {
            let req = http_get(
                pp,
                "/api/v1/messages",
                &[("x-api-key", &key), ("cookie", &cookie)],
            );
            let reply = raw_exchange(pp, &prefix, &req)
                .await
                .expect("a believed sender is served");
            status_of(&reply)
        }
    };

    assert_eq!(
        probe(allowed.clone(), header.clone()).await,
        200,
        "the address the balancer reported is what the allowlist sees"
    );
    assert_eq!(
        probe(refused.clone(), header).await,
        403,
        "…and the socket peer is no longer the answer"
    );

    // Control: without the header the same connection is attributed to loopback.
    assert_eq!(
        probe(refused, Vec::new()).await,
        200,
        "a headerless connection under `accept` is attributed to its real peer"
    );
}

/// Under `require`, a connection with no header is dropped even from a trusted
/// peer: behind an L4 balancer, a headerless connection did not come through it.
#[tokio::test]
async fn require_mode_drops_a_headerless_connection() {
    let _env = posture(Posture {
        trusted: "127.0.0.0/8, ::1/128",
        proxy_protocol: "require",
        ..Default::default()
    })
    .await;
    let addr = serve_behind_proxy_protocol(build().await).await;
    let req = http_get(addr, "/healthz", &[]);

    assert!(
        raw_exchange(addr, b"", &req).await.is_none(),
        "`require` must drop a connection that carries no PROXY header"
    );

    // …and the same connection WITH a valid header from the same trusted peer is
    // served, so the refusal above is about the header and not about the posture
    // rejecting everything.
    let header = proxy_v2(v4("198.51.100.7", 51234), v4("127.0.0.1", addr.port()));
    let reply = raw_exchange(addr, &header, &req)
        .await
        .expect("`require` serves a trusted sender that sends a header");
    assert_eq!(status_of(&reply), 200, "reply was: {reply}");
}

/// With `MW_PROXY_PROTOCOL` unset the connection path is unchanged: no header is
/// read, and one that is sent is not consumed — so it lands in the HTTP parser and
/// the request is not served as if the claim had been honoured.
#[tokio::test]
async fn the_default_posture_reads_no_proxy_header() {
    let _env = posture(Posture {
        trusted: "127.0.0.0/8, ::1/128",
        proxy_protocol: "", // absent — the default
        ..Default::default()
    })
    .await;
    let addr = serve_behind_proxy_protocol(build().await).await;
    let req = http_get(addr, "/healthz", &[]);

    let reply = raw_exchange(addr, b"", &req)
        .await
        .expect("an ordinary client is served");
    assert_eq!(status_of(&reply), 200, "reply was: {reply}");

    // A header sent to an `off` listener is just bytes in front of a request. It
    // must NOT be honoured; whether hyper answers 400 or drops the connection is
    // its business, but a 200 would mean the header had been consumed and believed.
    let header = proxy_v2(v4("198.51.100.7", 51234), v4("127.0.0.1", addr.port()));
    let status = raw_exchange(addr, &header, &req)
        .await
        .map(|r| status_of(&r))
        .unwrap_or(0);
    assert_ne!(
        status, 200,
        "with MW_PROXY_PROTOCOL unset a PROXY header must not be consumed"
    );
}

// ===========================================================================
// 5. Effective scheme — characterization of follow-up F1
// ===========================================================================

/// The effective public scheme comes from `X-Forwarded-Proto` (when the peer is a
/// trusted proxy), and drives HSTS and the cookie `Secure` attribute.
///
/// **This test also pins t20-e3's open follow-up F1 as it stands today.**
/// `ExternalBase::is_https` reads only `X-Forwarded-Proto`; RFC 7239 carries the
/// scheme in its own `proto=` parameter, so a strict-7239 proxy that emits
/// `Forwarded` and no `X-Forwarded-Proto` leaves the scheme falling back to the
/// listener's — degrading to `http`, so no `Secure` and no HSTS behind a real TLS
/// terminator. The direction of failure is safe and no tier-1 proxy hits it, but it
/// is a real gap.
///
/// The `proto=` leg is asserted **as it behaves now, deliberately**. When someone
/// implements F1 this test goes red at a line that says exactly what changed and
/// why, which is the point: an undocumented silent flip is how a "safe degradation"
/// becomes a shipped claim nobody verified. Fixing F1 means inverting that one
/// assertion and deleting this paragraph.
#[tokio::test]
async fn the_effective_scheme_comes_from_x_forwarded_proto_only() {
    let _env = posture(Posture {
        trusted: "127.0.0.0/8, ::1/128",
        mode: "forwarded",
        ..Default::default()
    })
    .await;
    let addr = serve_with_connect_info(build().await).await;

    // Plaintext listener, no assertion: no HSTS.
    assert!(hsts(addr, &[]).await.is_none(), "no HSTS over plain http");

    // POSITIVE: a trusted proxy's X-Forwarded-Proto makes the request effectively
    // https, and HSTS is emitted.
    let value = hsts(addr, &[("x-forwarded-proto", "https")])
        .await
        .expect("a trusted X-Forwarded-Proto makes the request effectively https");
    assert!(value.contains("max-age="), "HSTS was: {value}");

    // F1, asserted as it is: RFC 7239 `proto=https` from the same trusted peer does
    // NOT raise the scheme. Invert this when F1 is implemented.
    assert!(
        hsts(addr, &[("forwarded", "for=127.0.0.1;proto=https")])
            .await
            .is_none(),
        "t20-e3 follow-up F1: RFC 7239 `proto=` is not read for the effective \
         scheme. If this line now fails, F1 has been implemented — invert the \
         assertion and update docs/deploy, which must not claim RFC 7239 scheme \
         support until then."
    );

    // An UNTRUSTED peer's X-Forwarded-Proto must never raise the scheme; otherwise
    // any client could force HSTS onto a host that has no TLS.
    //
    // NOTE, and it caught this test out first time round: unlike the client-IP
    // model, which reads `MW_TRUSTED_PROXIES` per request, `ExternalBase::from_env`
    // reads it ONCE at `build_app` time. Changing the variable under a running
    // server does nothing to the scheme decision, so the untrusted posture needs a
    // freshly built app. Both are boot-time configuration in production, so this is
    // a property of the test harness rather than a defect — but a test that reuses
    // the server here asserts nothing.
    drop(_env);
    let _env = posture(Posture {
        trusted: "10.0.0.0/8",
        mode: "xff",
        ..Default::default()
    })
    .await;
    let untrusting = serve_with_connect_info(build().await).await;
    assert!(
        hsts(untrusting, &[("x-forwarded-proto", "https")])
            .await
            .is_none(),
        "an untrusted peer cannot assert the scheme"
    );
}

/// The `Strict-Transport-Security` value on `GET /healthz`, if any.
async fn hsts(addr: SocketAddr, headers: &[(&str, &str)]) -> Option<String> {
    let mut req = reqwest::Client::new().get(format!("http://{addr}/healthz"));
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    req.send()
        .await
        .unwrap()
        .headers()
        .get("strict-transport-security")
        .map(|v| v.to_str().unwrap().to_string())
}
