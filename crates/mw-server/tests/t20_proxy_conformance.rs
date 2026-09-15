//! t20-e11 — reverse-proxy conformance, driven **through** a booted proxy.
//!
//! The rule this tag set itself is that nothing may be claimed on the strength of
//! a shipped config file. This binary is the artifact that makes the claim: it
//! talks to the published port of one proxy cell from
//! `docker-compose.proxy.yml` and asserts, end to end, the behaviours a reverse
//! proxy is able to break.
//!
//! # Running it
//!
//! ```text
//! sh docs/deploy/proxy/tls/gen-certs.sh
//! docker compose -f docker-compose.proxy.yml --profile nginx up -d --wait
//! MW_T20_PROXY=1 MW_T20_PROXY_KIND=nginx MW_T20_PROXY_BASE=http://127.0.0.1:8601 \
//!   cargo test -p mw-server --test t20_proxy_conformance -- --nocapture --test-threads=1
//! docker compose -f docker-compose.proxy.yml --profile nginx down -v
//! ```
//!
//! `.github/workflows/proxy-conformance.yml` (t20-e14) runs exactly that across the
//! six tier-1 cells. **With `MW_T20_PROXY` unset every test loud-skips**, so the
//! binary is harmless in the ordinary workspace gate — and says so on stderr rather
//! than reporting a silent pass.
//!
//! # Environment contract (t20-e14, `678d3d1`)
//!
//! | var | meaning |
//! |---|---|
//! | `MW_T20_PROXY` | `1` — master gate |
//! | `MW_T20_PROXY_KIND` | profile name verbatim (`nginx`, `apache`, …) |
//! | `MW_T20_PROXY_BASE` | `http://127.0.0.1:<port>`; **empty** for `haproxy-l4` |
//! | `MW_T20_PROXY_TLS_BASE` | `https://127.0.0.1:<port>` |
//! | `MW_T20_PROXY_CA` | path to the self-signed leaf the cells present |
//! | `MW_T20_DIRECT_BASE` | the same app process with no proxy — the control |
//! | `MW_T20_EXPECT_CLIENT_IP` | the address the app must attribute the request to |
//! | `MW_T20_JMAP_URL` / `MW_T20_USER` / `MW_T20_PASS` | upstream login |
//! | `MW_T20_PROXY_PREFIX` | sub-path base; unset in every cell today |
//!
//! An **empty** value is treated exactly like an absent one. GitHub Actions cannot
//! omit a key from a step's `env:` map, so `MW_T20_PROXY_BASE` arrives as `""` on
//! the `haproxy-l4` leg; a suite that checked only `is_ok()` would try to reach
//! `http://127.0.0.1:`.
//!
//! # How the attributed address is read out
//!
//! Through the OAuth scoped-key IP allowlist, the production consumer of
//! `scope_mw::client_ip`. A key allowlisted to `X` answers `200` iff the app
//! resolved the client to `X` and `403` otherwise — an exact read-out, and a
//! **positive** one, so a cell cannot pass by refusing everything.
//!
//! # What a first run should suspect first
//!
//! `docker-compose.proxy.yml`'s trust-model walkthrough (the `172.28.x` addressing,
//! and `MW_T20_EXPECT_CLIENT_IP=172.28.0.1`) is marked in that file as a
//! **prediction**: it was derived from the pinned IPAM config on a host with no
//! Docker daemon, and no cell had ever been booted when it was written. If a cell
//! attributes a request to something other than the expected address, check the
//! runner's NAT behaviour and that comment **before** changing any proxy config or
//! the trust model.

mod common;

use std::time::{Duration, Instant};

use futures_util::StreamExt;
use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// Gates
// ---------------------------------------------------------------------------

/// Read an environment variable, treating empty exactly like absent.
fn var(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn skip(reason: &str) {
    common::gate::skip(format_args!("[t20-e11] {reason}"));
}

/// One booted proxy cell.
struct Cell {
    kind: String,
    http_base: Option<String>,
    tls_base: Option<String>,
    ca_path: Option<String>,
    direct_base: Option<String>,
    expect_client_ip: Option<String>,
    jmap_url: String,
    user: String,
    pass: String,
    /// `MW_T20_PROXY_PREFIX`, canonicalised to `""` or `/mail` (no trailing slash).
    prefix: String,
}

impl Cell {
    /// The cell under test, or `None` with a loud reason on stderr.
    fn from_env() -> Option<Self> {
        if var("MW_T20_PROXY").as_deref() != Some("1") {
            skip(
                "MW_T20_PROXY!=1 — no proxy cell is booted, so nothing here ran. \
                 Bring one up with `docker compose -f docker-compose.proxy.yml \
                 --profile <cell> up -d --wait` and see the module header.",
            );
            return None;
        }
        let http_base = var("MW_T20_PROXY_BASE");
        let tls_base = var("MW_T20_PROXY_TLS_BASE");
        if http_base.is_none() && tls_base.is_none() {
            skip("MW_T20_PROXY=1 but neither MW_T20_PROXY_BASE nor MW_T20_PROXY_TLS_BASE is set.");
            return None;
        }
        let prefix = var("MW_T20_PROXY_PREFIX")
            .map(|p| format!("/{}", p.trim_matches('/')))
            .filter(|p| p != "/")
            .unwrap_or_default();
        Some(Self {
            kind: var("MW_T20_PROXY_KIND").unwrap_or_else(|| "unknown".into()),
            http_base,
            tls_base,
            ca_path: var("MW_T20_PROXY_CA"),
            direct_base: var("MW_T20_DIRECT_BASE"),
            expect_client_ip: var("MW_T20_EXPECT_CLIENT_IP"),
            jmap_url: var("MW_T20_JMAP_URL")
                .unwrap_or_else(|| "http://mock:8181/.well-known/jmap".into()),
            user: var("MW_T20_USER").unwrap_or_else(|| "testuser@example.org".into()),
            pass: var("MW_T20_PASS").unwrap_or_else(|| "testpass".into()),
            prefix,
        })
    }

    /// The base every non-TLS-specific test drives. `haproxy-l4` publishes no
    /// plaintext listener — it is TLS passthrough by construction — so that cell
    /// runs the whole suite over https.
    fn base(&self) -> &str {
        self.http_base
            .as_deref()
            .or(self.tls_base.as_deref())
            .expect("from_env guarantees one of the two")
    }

    /// Does this cell's proxy **append** to `X-Forwarded-For` instead of
    /// overwriting it? Only Envoy does, among the shipped configs; the override
    /// exists so an appending cell added later needs no code change here.
    fn appends_xff(&self) -> bool {
        self.kind == "envoy" || var("MW_T20_XFF_APPENDS").as_deref() == Some("1")
    }

    /// A client that trusts the cell's self-signed leaf.
    fn client(&self) -> reqwest::Client {
        let mut b = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(Duration::from_secs(120));
        match self.ca_path.as_deref().map(std::fs::read) {
            Some(Ok(pem)) => match reqwest::Certificate::from_pem(&pem) {
                Ok(cert) => b = b.add_root_certificate(cert),
                Err(e) => {
                    eprintln!("[t20-e11] MW_T20_PROXY_CA is not a PEM certificate ({e})");
                }
            },
            Some(Err(e)) => eprintln!("[t20-e11] MW_T20_PROXY_CA unreadable ({e})"),
            None => {}
        }
        // The cells present a leaf for the compose service names and 127.0.0.1;
        // the suite addresses them through a published host port, so hostname
        // verification is not the property under test here.
        b.danger_accept_invalid_certs(true).build().unwrap()
    }

    /// Log in through this cell and return a driveable session.
    async fn login(&self, base: &str) -> Ctx {
        let client = self.client();
        let resp = client
            .post(format!("{base}{}/api/login", self.prefix))
            .json(&json!({
                "jmapUrl": self.jmap_url,
                "username": self.user,
                "password": self.pass,
            }))
            .send()
            .await
            .unwrap_or_else(|e| panic!("[{}] login request failed at {base}: {e}", self.kind));
        let status = resp.status();
        let cookie_header = resp
            .headers()
            .get_all(reqwest::header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .filter_map(|v| v.split(';').next())
            .collect::<Vec<_>>()
            .join("; ");
        let secure_session = resp
            .headers()
            .get_all(reqwest::header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .any(|v| v.starts_with("mw_session=") && v.to_ascii_lowercase().contains("secure"));
        let hsts = resp
            .headers()
            .get("strict-transport-security")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        assert_eq!(
            status, 200,
            "[{}] login through {base} must succeed — if this fails the cell is \
             not wired to the mock, and every other assertion is meaningless",
            self.kind
        );
        let body: Value = resp.json().await.unwrap();
        Ctx {
            base: base.to_string(),
            prefix: self.prefix.clone(),
            kind: self.kind.clone(),
            account_id: body["accountId"].as_str().unwrap().to_string(),
            client,
            cookie_header,
            secure_session,
            hsts,
        }
    }
}

/// A logged-in session against one base URL.
struct Ctx {
    base: String,
    prefix: String,
    kind: String,
    account_id: String,
    client: reqwest::Client,
    cookie_header: String,
    /// Did the login's `mw_session` cookie carry `Secure`?
    secure_session: bool,
    /// The `Strict-Transport-Security` on the login response, if any.
    hsts: Option<String>,
}

impl Ctx {
    fn url(&self, path: &str) -> String {
        format!("{}{}{}", self.base, self.prefix, path)
    }

    /// Mint a read-only mail key restricted to `allowlist`.
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
            .post(self.url("/api/keys"))
            .json(&json!({ "label": "t20-e11", "accountId": self.account_id, "scope": scope }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        minted["displayToken"]
            .as_str()
            .unwrap_or_else(|| panic!("[{}] key mint returned no token: {minted}", self.kind))
            .to_string()
    }

    /// `200` iff the app resolved this request's client inside `key`'s allowlist.
    async fn probe(&self, key: &str, headers: &[(&str, &str)]) -> u16 {
        let mut req = self
            .client
            .get(self.url("/api/v1/messages"))
            .header("x-api-key", key);
        for (k, v) in headers {
            req = req.header(*k, *v);
        }
        req.send().await.unwrap().status().as_u16()
    }

    /// Assert the app attributed the request to `expected` and to nothing else.
    async fn assert_attributed_to(&self, expected: &str, decoy: &str, headers: &[(&str, &str)]) {
        let allowed = self.key_for(&[expected]).await;
        assert_eq!(
            self.probe(&allowed, headers).await,
            200,
            "[{}] expected the client to be attributed to {expected} (headers: {headers:?}). \
             If this is the first run of a new cell, suspect the compose \
             trust-model comment and the runner's NAT before the trust model.",
            self.kind
        );
        let refused = self.key_for(&[decoy]).await;
        assert_eq!(
            self.probe(&refused, headers).await,
            403,
            "[{}] the client must NOT be attributed to {decoy} (headers: {headers:?})",
            self.kind
        );
    }
}

/// `/32` (or `/128`) form of a bare address, for an allowlist entry.
fn host_route(ip: &str) -> String {
    if ip.contains(':') {
        format!("{ip}/128")
    } else {
        format!("{ip}/32")
    }
}

/// A decoy address guaranteed not to be the real client: TEST-NET-3, which no
/// compose network or runner uses.
const DECOY: &str = "203.0.113.9/32";

// ===========================================================================
// 1. Forwarded client IP
// ===========================================================================

/// The request is attributed to the **test client**, not to the proxy.
///
/// This is the positive case for the whole trust model: the app is behind a proxy
/// it trusts, so it reads `X-Forwarded-For` — and the address it lands on must be
/// the one on the far side of the proxy.
#[tokio::test]
async fn the_proxy_attributes_the_request_to_the_real_client() {
    let Some(cell) = Cell::from_env() else { return };
    let Some(expected) = cell.expect_client_ip.clone() else {
        skip("MW_T20_EXPECT_CLIENT_IP unset — cannot assert which address the app resolved.");
        return;
    };
    let ctx = cell.login(cell.base()).await;
    ctx.assert_attributed_to(&host_route(&expected), DECOY, &[])
        .await;
}

/// A client-supplied `X-Forwarded-For` does not change the attribution — either
/// because the proxy overwrote it (every tier-1 config does) or because the
/// right-to-left walk stopped before reaching it.
#[tokio::test]
async fn a_client_supplied_forwarded_header_is_not_honoured() {
    let Some(cell) = Cell::from_env() else { return };
    let Some(expected) = cell.expect_client_ip.clone() else {
        skip("MW_T20_EXPECT_CLIENT_IP unset — see the sibling test.");
        return;
    };
    let ctx = cell.login(cell.base()).await;
    let forged = [("x-forwarded-for", "203.0.113.9")];
    ctx.assert_attributed_to(&host_route(&expected), DECOY, &forged)
        .await;
}

/// The genuine **two-hop** case: a proxy that *appends* hands the app
/// `X-Forwarded-For: <forged>, <real client>`, and the right-to-left walk must stop
/// at the real client.
///
/// This is deliberately separate from the test above, because through an
/// overwriting proxy the forged hop never reaches the app at all — that assertion
/// tests the proxy's config, this one tests the app's walk.
///
/// **Coverage gap, stated plainly:** among the eight shipped configs only Envoy
/// appends (`use_remote_address: true`, `xff_num_trusted_hops: 0`), and Envoy is
/// tier 2 and out of the CI matrix. Every tier-1 cell overwrites, so on a normal CI
/// run this leg skips and the multi-hop walk is covered only by the in-process
/// `t20_forwarded_spoof.rs`. Closing it in CI means adding an appending location to
/// a tier-1 config (`$proxy_add_x_forwarded_for` on nginx) — those files are
/// `t20-e7`'s locks, not this lane's, so it is reported rather than taken. Run
/// `--profile envoy` (port 8606) to exercise it today.
#[tokio::test]
async fn an_appending_proxy_still_resolves_the_real_client() {
    let Some(cell) = Cell::from_env() else { return };
    if !cell.appends_xff() {
        skip(&format!(
            "cell `{}` OVERWRITES X-Forwarded-For, so the app never sees two hops \
             and the right-to-left walk is not exercised here. Only the `envoy` \
             profile appends (docker compose -f docker-compose.proxy.yml \
             --profile envoy up -d --wait, port 8606), or set MW_T20_XFF_APPENDS=1 \
             for a cell whose config appends. The walk itself is covered in-process \
             by t20_forwarded_spoof.rs::the_walk_stops_at_the_first_untrusted_hop.",
            cell.kind
        ));
        return;
    }
    let Some(expected) = cell.expect_client_ip.clone() else {
        skip("MW_T20_EXPECT_CLIENT_IP unset — see the sibling test.");
        return;
    };
    let ctx = cell.login(cell.base()).await;

    // The proxy appends its peer, so the app sees `203.0.113.9, <real client>`.
    // The walk must stop at the rightmost untrusted hop and never reach the forged
    // one, however many the client prepends.
    for forged in ["203.0.113.9", "10.0.0.1, 203.0.113.9, 8.8.8.8"] {
        ctx.assert_attributed_to(
            &host_route(&expected),
            DECOY,
            &[("x-forwarded-for", forged)],
        )
        .await;
    }
}

/// The control: the same app process reached **directly**, with no proxy in front.
/// The connecting peer is not in `MW_TRUSTED_PROXIES`, so its `X-Forwarded-For` is
/// its own claim about itself and is ignored.
#[tokio::test]
async fn the_direct_baseline_ignores_a_forwarded_header() {
    let Some(cell) = Cell::from_env() else { return };
    let Some(direct) = cell.direct_base.clone() else {
        skip("MW_T20_DIRECT_BASE unset — the no-proxy control case did not run.");
        return;
    };
    let ctx = cell.login(&direct).await;
    let forged = [("x-forwarded-for", "203.0.113.9")];
    let refused = ctx.key_for(&[DECOY]).await;
    assert_eq!(
        ctx.probe(&refused, &forged).await,
        403,
        "[{}] a direct (untrusted) client must not be able to name its own address",
        cell.kind
    );
}

// ===========================================================================
// 2. Streaming
// ===========================================================================

/// `/jmap/eventsource` reaches the client as a stream, not as a buffered blob.
///
/// **The load-bearing assertion is the body frame, not any header.** The stream is
/// otherwise idle, so the frame that arrives is the 30 s keepalive
/// (`push::HEARTBEAT`) — precisely what a buffering proxy holds back and what an
/// idle read timeout reaps. If that byte reaches the client, the stream was not
/// buffered, whatever the headers say.
///
/// ## Why the header is no longer asserted to be present (t20-e-e2e, D5)
///
/// The first version of this test required `X-Accel-Buffering: no` on the response
/// **through the proxy**, and it failed on `nginx` alone, with
/// `"X-Accel-Buffering was stripped by the proxy"`. That was backwards: `X-Accel-*`
/// are *upstream control* headers, and nginx **consumes** them rather than
/// forwarding them. Their absence through nginx is proof the mechanism worked. The
/// other cells passed only because they ignore the header and pass it through
/// verbatim — so the assertion went green on every proxy that disregards it and red
/// on the one that respects it, and its message named the wrong component. That is
/// a false *failure* pointed at an innocent cell, the mirror image of the
/// false-pass problem this suite exists to avoid.
///
/// What is asserted now:
///   * through the proxy — a body frame inside a plausible read timeout (the real
///     question), and that `X-Accel-Buffering` is either `no` (a proxy that ignores
///     it, forwarding verbatim) or **absent** (a proxy that honoured and consumed
///     it). Any other value would mean an intermediary rewrote it;
///   * against `MW_T20_DIRECT_BASE` — the same app process with nothing in front —
///     that the app **does** emit `X-Accel-Buffering: no`. That is where the
///     question "does the app send the header" has an unambiguous answer, and
///     asking it there is what keeps the relaxed through-proxy check honest.
///
/// The header checks also run **after** the body-frame assertion. In the original
/// ordering the header panic fired first, so on `nginx` the timing leg — the one
/// that matters — never ran at all.
#[tokio::test]
async fn the_event_source_stream_is_not_buffered() {
    let Some(cell) = Cell::from_env() else { return };
    let ctx = cell.login(cell.base()).await;

    let started = Instant::now();
    let resp = ctx
        .client
        .get(ctx.url("/jmap/eventsource"))
        .send()
        .await
        .unwrap();
    let head_took = started.elapsed();
    assert_eq!(resp.status(), 200, "[{}] eventsource is gated", cell.kind);
    assert!(
        head_took < Duration::from_secs(2),
        "[{}] the SSE response head took {head_took:?} — a proxy is buffering it",
        cell.kind
    );

    // Capture the headers before the body is consumed, so a header complaint can
    // never short-circuit the assertion that actually answers the question.
    let content_type = header_of(&resp, "content-type");
    let accel = header_of(&resp, "x-accel-buffering");
    let cache = header_of(&resp, "cache-control");

    assert!(
        content_type.starts_with("text/event-stream"),
        "[{}] content-type was {content_type}",
        cell.kind
    );

    // ── The property under test: a body byte reaches the client ───────────────
    // The server's keepalive is 30 s. Budget generously: the claim is "well inside
    // a 60 s read timeout", not "at exactly 30 s".
    let budget = Duration::from_secs(50);
    let waited = Instant::now();
    let mut stream = resp.bytes_stream();
    let frame = tokio::time::timeout(budget, stream.next())
        .await
        .unwrap_or_else(|_| {
            panic!(
                "[{}] no SSE body frame arrived within {budget:?}. Either the proxy \
                 is buffering the stream or it reaped the idle connection — both are \
                 the failure this cell exists to catch.",
                cell.kind
            )
        })
        .expect("the stream stays open")
        .expect("no transport error");
    let text = String::from_utf8_lossy(&frame).to_string();
    assert!(
        text.contains("data:"),
        "[{}] first SSE frame was not a data frame: {text:?}",
        cell.kind
    );
    eprintln!(
        "[t20-e11] {}: SSE head in {head_took:?}, first body frame in {:?}",
        cell.kind,
        waited.elapsed()
    );

    // ── Header hygiene, now that the stream itself has been proven ────────────
    assert!(
        accel.is_empty() || accel == "no",
        "[{}] X-Accel-Buffering came back as {accel:?}. The app sends `no`; a proxy \
         may forward that verbatim or consume it (nginx does, which is what \
         honouring it means) — but rewriting it to anything else means an \
         intermediary is asking for the stream to be buffered.",
        cell.kind
    );
    assert!(
        cache.contains("no-transform"),
        "[{}] cache-control was {cache} — a compressing intermediary may hold frames",
        cell.kind
    );

    // ── Does the APP emit the header at all? Asked where the answer is plain ──
    let Some(direct) = cell.direct_base.clone() else {
        skip(&format!(
            "cell `{}`: MW_T20_DIRECT_BASE unset, so `X-Accel-Buffering: no` was not \
             confirmed at the app itself. Through a proxy its absence is legitimate \
             (nginx consumes it), so without the direct leg nothing here would catch \
             the app dropping the header entirely.",
            cell.kind
        ));
        return;
    };
    let direct_ctx = cell.login(&direct).await;
    let resp = direct_ctx
        .client
        .get(format!("{direct}{}/jmap/eventsource", cell.prefix))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "[{}] eventsource on the direct baseline",
        cell.kind
    );
    assert_eq!(
        header_of(&resp, "x-accel-buffering"),
        "no",
        "[{}] the app itself must emit X-Accel-Buffering: no — with no proxy in \
         front there is nothing that could have consumed it, so this is the app's \
         own behaviour and not a cell's",
        cell.kind
    );
    assert!(
        header_of(&resp, "cache-control").contains("no-transform"),
        "[{}] the app itself must emit Cache-Control: … no-transform",
        cell.kind
    );
}

/// One response header as a string, empty when absent or non-ASCII.
fn header_of(resp: &reqwest::Response, name: &str) -> String {
    resp.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string()
}

/// `POST /api/assist/invoke` is reachable through the proxy and answers with the
/// **app's** JSON, not the proxy's error page.
///
/// The streaming half of this route cannot be asserted here: no cell configures an
/// Assist gateway, so `invoke` returns `AssistError::Disabled` (`404`) before it
/// ever opens an SSE stream. What this does prove is that a `POST` whose response
/// would be `text/event-stream` reaches the app at all — it matches no rule keyed
/// on method or on an `/eventsource` path, which is exactly why proxy configs miss
/// it. The buffering exemption for it is asserted by config review (t20-e7) and by
/// the sibling `/jmap/eventsource` test, not by a live stream.
#[tokio::test]
async fn the_assist_invoke_route_reaches_the_app_not_the_proxys_error_page() {
    let Some(cell) = Cell::from_env() else { return };
    let ctx = cell.login(cell.base()).await;

    let resp = ctx
        .client
        .post(ctx.url("/api/assist/invoke"))
        .json(&json!({
            "capability": "summarize",
            "input": { "prompt": "t20 conformance" },
        }))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    // An HTML body is the tell: it is either the fronting proxy's error page or the
    // SPA fallback, and both mean the POST never reached the handler.
    assert!(
        !content_type.starts_with("text/html"),
        "[{}] /api/assist/invoke answered {status} {content_type} — an HTML body \
         here means the proxy (or the SPA fallback) answered, not the app",
        cell.kind
    );
    assert!(
        content_type.starts_with("application/json"),
        "[{}] /api/assist/invoke answered {status} {content_type}; the app answers \
         this route in JSON (404 `assist disabled` when no gateway is configured). \
         A `text/plain` 422 means the request body no longer matches `InvokeReq` — \
         fix the body here, it is not a proxy fault",
        cell.kind
    );
    skip(&format!(
        "cell `{}`: the /api/assist/invoke SSE STREAM was not driven — no cell \
         configures an Assist gateway, so the route answers {status} before \
         streaming. Reachability and the JSON content-type ARE asserted.",
        cell.kind
    ));
}

/// The WebSocket upgrade survives the proxy, the RFC 8887 `jmap` subprotocol is
/// echoed, and a ping/pong round trip completes — over **plaintext or TLS**.
///
/// ## Why this connects the socket by hand (t20-e7's finding)
///
/// `tokio_tungstenite::connect_async` carries no TLS connector in this tree, so it
/// refuses a `wss://` URL outright with `URL scheme not supported`. `haproxy-l4` is
/// TLS-only by construction, so an earlier version of this test skipped there and
/// the matrix read as **"PROXY protocol breaks WebSockets"** — which it does not.
/// t20-e7 showed the failure was the URL scheme and not the cell, by reproducing it
/// against nginx's TLS port while nginx over plaintext passed.
///
/// Adding `tokio-tungstenite`'s TLS feature would have been the obvious fix; it is
/// unnecessary. `rustls`, `tokio-rustls` and `base64` are already direct
/// dependencies of `mw-server` and so are available to its test targets, so the
/// socket is established here — TCP, optionally wrapped in TLS — and handed to
/// [`tokio_tungstenite::client_async`], which takes any duplex stream. **Net-zero
/// new dependencies and no feature change**, and the `jmap` subprotocol echo is now
/// asserted on the only cell that exercises PROXY protocol, where it was previously
/// unproven by anything but a one-off manual check.
#[tokio::test]
async fn the_websocket_upgrades_and_echoes_the_jmap_subprotocol() {
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;

    let Some(cell) = Cell::from_env() else { return };
    let base = cell.base().to_string();
    let ctx = cell.login(&base).await;

    let Some((tls, host, port)) = split_base(&base) else {
        skip(&format!(
            "cell `{}`: could not parse a host:port out of {base:?}, so the \
             WebSocket leg did not run.",
            cell.kind
        ));
        return;
    };
    let scheme = if tls { "wss" } else { "ws" };
    let ws_url = format!("{scheme}://{host}:{port}{}/jmap/ws", cell.prefix);
    let mut req = ws_url.into_client_request().unwrap();
    req.headers_mut()
        .insert("cookie", ctx.cookie_header.parse().unwrap());
    req.headers_mut()
        .insert("sec-websocket-protocol", "jmap".parse().unwrap());

    let stream = match ws_stream(&cell, tls, &host, port).await {
        Ok(s) => s,
        Err(e) => {
            // Never a bare failure: a transport we could not establish is reported
            // as a limitation of this client, not as a verdict on the cell.
            skip(&format!(
                "cell `{}`: could not open a {} socket to {host}:{port} for the \
                 WebSocket leg ({e}). This is the SUITE's transport, not the cell — \
                 do not read it as a proxy fault. For a TLS cell, check that \
                 MW_T20_PROXY_CA points at docs/deploy/proxy/tls/certs/server.crt.",
                cell.kind,
                if tls { "TLS" } else { "TCP" },
            ));
            return;
        }
    };

    // tungstenite enforces RFC 6455 §4.1: a client that offered a subprotocol and
    // is answered with none fails the connection. Reaching `expect` at all is the
    // negotiation assertion; the header check pins which token was chosen.
    let (mut ws, resp) = tokio_tungstenite::client_async(req, stream)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "[{}] WebSocket upgrade through the proxy failed ({scheme}://): {e}",
                cell.kind
            )
        });
    assert_eq!(
        resp.headers()
            .get("sec-websocket-protocol")
            .map(|v| v.as_bytes()),
        Some(&b"jmap"[..]),
        "[{}] the jmap subprotocol was not echoed through the proxy",
        cell.kind
    );

    // Client ping → server pong (axum answers pings inside `ws_loop`). A proxy that
    // does not pass control frames through fails here.
    use futures_util::SinkExt;
    ws.send(Message::Ping(b"t20".to_vec().into()))
        .await
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        assert!(
            Instant::now() < deadline,
            "[{}] no pong came back through the proxy",
            cell.kind
        );
        let msg = tokio::time::timeout(Duration::from_secs(20), ws.next())
            .await
            .expect("a frame arrives")
            .expect("the stream stays open")
            .expect("no ws error");
        match msg {
            Message::Pong(p) => {
                assert_eq!(p.as_ref(), b"t20", "[{}] pong payload", cell.kind);
                break;
            }
            // The server also emits keepalives; ignore them and keep waiting.
            Message::Ping(_) | Message::Text(_) => continue,
            other => panic!("[{}] unexpected frame: {other:?}", cell.kind),
        }
    }
}

/// `(is_tls, host, port)` from `http(s)://host:port`.
fn split_base(base: &str) -> Option<(bool, String, u16)> {
    let (tls, rest) = match base.split_once("://") {
        Some(("https", rest)) => (true, rest),
        Some(("http", rest)) => (false, rest),
        _ => return None,
    };
    let authority = rest.split(['/', '?']).next().unwrap_or(rest);
    let (host, port) = authority.rsplit_once(':')?;
    Some((tls, host.to_string(), port.parse().ok()?))
}

/// Anything the WebSocket client can be driven over. `client_async` takes any
/// duplex stream, which is what lets the plaintext and TLS cases share one path.
trait DuplexStream: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send {}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send> DuplexStream for T {}

/// Open the socket the WebSocket handshake will run over: TCP, wrapped in TLS when
/// the cell fronts with it.
async fn ws_stream(
    cell: &Cell,
    tls: bool,
    host: &str,
    port: u16,
) -> Result<Box<dyn DuplexStream>, String> {
    let tcp = tokio::net::TcpStream::connect((host, port))
        .await
        .map_err(|e| format!("tcp connect: {e}"))?;
    if !tls {
        return Ok(Box::new(tcp));
    }

    let mut roots = rustls::RootCertStore::empty();
    let ca = cell
        .ca_path
        .as_deref()
        .ok_or_else(|| "MW_T20_PROXY_CA is unset, so the cell's leaf is untrusted".to_string())?;
    let pem = std::fs::read_to_string(ca).map_err(|e| format!("read {ca}: {e}"))?;
    let mut added = 0usize;
    for der in pem_blocks(&pem, "CERTIFICATE") {
        if roots.add(der.into()).is_ok() {
            added += 1;
        }
    }
    if added == 0 {
        return Err(format!("no usable CERTIFICATE block in {ca}"));
    }

    // Build against the ring provider explicitly rather than relying on a
    // process-wide default having been installed — nothing in a test binary
    // guarantees that, and the failure would be a panic deep inside rustls.
    let mut config = rustls::ClientConfig::builder_with_provider(
        rustls::crypto::ring::default_provider().into(),
    )
    .with_safe_default_protocol_versions()
    .map_err(|e| format!("rustls protocol versions: {e}"))?
    .with_root_certificates(roots)
    .with_no_client_auth();
    // WebSocket is an HTTP/1.1 upgrade, so ask for it by name: an ALPN-selected h2
    // would make the handshake fail for a reason that has nothing to do with the
    // proxy under test.
    config.alpn_protocols = vec![b"http/1.1".to_vec()];

    let server_name = rustls_pki_types::ServerName::try_from(host.to_string())
        .map_err(|e| format!("server name {host}: {e}"))?;
    let stream = tokio_rustls::TlsConnector::from(std::sync::Arc::new(config))
        .connect(server_name, tcp)
        .await
        .map_err(|e| format!("tls handshake: {e}"))?;
    Ok(Box::new(stream))
}

/// DER blocks for one PEM tag. A dependency-free reader, mirroring the one in
/// `crates/mw-server/src/tls.rs` (which is private to that module).
fn pem_blocks(pem: &str, tag: &str) -> Vec<Vec<u8>> {
    use base64::Engine;
    let begin = format!("-----BEGIN {tag}-----");
    let end = format!("-----END {tag}-----");
    let mut out = Vec::new();
    let mut rest = pem;
    while let Some(start) = rest.find(&begin) {
        let after = &rest[start + begin.len()..];
        let Some(stop) = after.find(&end) else { break };
        let body: String = after[..stop].split_whitespace().collect();
        if let Ok(der) = base64::engine::general_purpose::STANDARD.decode(&body) {
            out.push(der);
        }
        rest = &after[stop + end.len()..];
    }
    out
}

// ===========================================================================
// 3. Body limits and ranges
// ===========================================================================

/// A 20 MB upload succeeds through the proxy — the proxy's body limit is raised
/// above the app's advertised `maxSizeUpload` — and a 60 MB one is refused by the
/// **app**, with its stable JSON `413`, not by the proxy with an HTML error page.
///
/// The distinction is the whole point: nginx's stock `client_max_body_size 1m` and
/// most ingress controllers' 1 MB default reject an attachment long before the app
/// does, and the client then sees HTML where it expected JSON.
#[tokio::test]
async fn a_large_upload_succeeds_and_an_oversize_one_gets_the_apps_json_413() {
    let Some(cell) = Cell::from_env() else { return };
    let ctx = cell.login(cell.base()).await;
    let upload = ctx.url(&format!("/jmap/upload/{}", ctx.account_id));

    let ok = ctx
        .client
        .post(&upload)
        .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
        .body(vec![b'a'; 20 * 1024 * 1024])
        .send()
        .await
        .unwrap();
    assert_eq!(
        ok.status(),
        200,
        "[{}] a 20 MB upload must pass the proxy's body limit",
        cell.kind
    );
    let body: Value = ok.json().await.unwrap();
    assert!(
        body["blobId"].is_string(),
        "[{}] upload response was {body}",
        cell.kind
    );

    // The app answers `413` and closes WITHOUT draining the remaining body. A
    // client still writing when that happens can lose the response to a write
    // abort instead of reading it — observed once here on `haproxy-l4`, where L4
    // passthrough means the client is talking straight to the app with no proxy
    // buffering the request. `curl`, which sends `Expect: 100-continue`, always
    // sees the `413`; `hyper` does not send it and occasionally does not.
    //
    // One retry, then a diagnosis rather than a bare unwrap. A transport abort
    // here is NOT a verdict on the proxy, and a red that reads like one is how
    // "PROXY protocol breaks large uploads" gets into a matrix.
    let mut attempt = Err(None);
    for _ in 0..2 {
        match ctx
            .client
            .post(&upload)
            .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
            .body(vec![b'a'; 60 * 1024 * 1024])
            .send()
            .await
        {
            Ok(r) => {
                attempt = Ok(r);
                break;
            }
            Err(e) => attempt = Err(Some(e.to_string())),
        }
    }
    let too_big = attempt.unwrap_or_else(|e| {
        panic!(
            "[{}] the oversize upload never produced a response, twice: {}. The app \
             answers 413 and closes before draining the body, so a client that is \
             still writing can lose the response — this is an APP behaviour on an \
             early refusal, not a fault of this proxy. `curl` (which sends \
             `Expect: 100-continue`) does observe the 413 on this cell.",
            cell.kind,
            e.unwrap_or_else(|| "<no error captured>".into()),
        )
    });
    assert_eq!(
        too_big.status(),
        413,
        "[{}] a 60 MB upload must be refused",
        cell.kind
    );
    let content_type = too_big
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        content_type.starts_with("application/json"),
        "[{}] the 413 was {content_type} — that is the PROXY's refusal, not the \
         app's. Raise the cell's body limit above the app's maxSizeUpload.",
        cell.kind
    );
    let body: Value = too_big.json().await.unwrap();
    assert_eq!(body["error"], "payload too large", "[{}]", cell.kind);
    assert_eq!(body["limit"], "maxSizeUpload", "[{}]", cell.kind);
    assert!(body["maxBytes"].is_number(), "[{}]", cell.kind);
}

/// `Accept-Ranges` is advertised and a single `Range` is answered with `206` and
/// the exact slice, through the proxy.
#[tokio::test]
async fn a_range_request_returns_206_through_the_proxy() {
    let Some(cell) = Cell::from_env() else { return };
    let ctx = cell.login(cell.base()).await;
    let download = ctx.url(&format!(
        "/jmap/download/{}/t20-blob/t20.txt",
        ctx.account_id
    ));

    let whole = ctx.client.get(&download).send().await.unwrap();
    assert_eq!(whole.status(), 200, "[{}] download is wired", cell.kind);
    assert_eq!(
        whole
            .headers()
            .get("accept-ranges")
            .and_then(|v| v.to_str().ok()),
        Some("bytes"),
        "[{}] Accept-Ranges did not survive the proxy",
        cell.kind
    );
    let full = whole.text().await.unwrap();
    assert!(full.len() > 8, "[{}] blob body was {full:?}", cell.kind);

    let part = ctx
        .client
        .get(&download)
        .header(reqwest::header::RANGE, "bytes=2-5")
        .send()
        .await
        .unwrap();
    assert_eq!(
        part.status(),
        206,
        "[{}] a single Range must be answered with 206 — a proxy that strips the \
         Range header or normalises the status shows up here",
        cell.kind
    );
    let content_range = part
        .headers()
        .get("content-range")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert_eq!(
        content_range,
        format!("bytes 2-5/{}", full.len()),
        "[{}]",
        cell.kind
    );
    assert_eq!(
        part.text().await.unwrap(),
        full[2..=5],
        "[{}] the 206 body must be the exact slice",
        cell.kind
    );
}

// ===========================================================================
// 4. Static surface
// ===========================================================================

/// `/` serves the SPA shell as HTML, a content-hashed asset is served with the
/// immutable cache lifetime and an `ETag`, and a conditional re-request is `304`.
#[tokio::test]
async fn the_shell_and_hashed_assets_carry_the_right_cache_headers() {
    let Some(cell) = Cell::from_env() else { return };
    let client = cell.client();

    let shell = client
        .get(format!("{}{}/", cell.base(), cell.prefix))
        .send()
        .await
        .unwrap();
    assert_eq!(shell.status(), 200, "[{}] the shell is served", cell.kind);
    let shell_type = shell
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        shell_type.starts_with("text/html"),
        "[{}] shell content-type was {shell_type}",
        cell.kind
    );
    let shell_cache = shell
        .headers()
        .get(reqwest::header::CACHE_CONTROL)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        shell_cache.contains("no-cache"),
        "[{}] index.html names hashed assets, so it must not be cached \
         immutably; cache-control was {shell_cache:?}",
        cell.kind
    );
    let html = shell.text().await.unwrap();

    let Some(asset) = first_hashed_asset(&html) else {
        skip(&format!(
            "cell `{}`: no content-hashed `assets/…` reference found in the served \
             shell, so the immutable-cache and 304 legs did not run. The image may \
             have been built without the SPA bundle.",
            cell.kind
        ));
        return;
    };
    let asset_url = format!("{}{}/{}", cell.base(), cell.prefix, asset.trim_matches('/'));

    let first = client.get(&asset_url).send().await.unwrap();
    assert_eq!(
        first.status(),
        200,
        "[{}] hashed asset {asset_url} is served",
        cell.kind
    );
    let cache = header_of(&first, "cache-control");

    // Attribute the answer before complaining about it. The same asset fetched
    // from the app with nothing in front settles whether a proxy rewrote the
    // header or the app never sent the right one — the D5 lesson applied here:
    // a failure message that names the wrong component costs a diagnosis cycle.
    let attribution = match cell.direct_base.as_deref() {
        Some(direct) => {
            let d = client
                .get(format!(
                    "{direct}{}/{}",
                    cell.prefix,
                    asset.trim_matches('/')
                ))
                .send()
                .await
                .unwrap();
            let direct_cache = header_of(&d, "cache-control");
            if direct_cache == cache {
                format!(
                    "the app itself serves the same value with NO proxy in front \
                     ({direct} → {direct_cache:?}), so this is an APP defect, not \
                     this cell's — see t20-e-e2e D3: `is_content_hashed` requires \
                     the hash suffix to contain an ASCII digit, and a digit-free \
                     Vite hash (~1 build in 4) therefore falls through to `no-cache`"
                )
            } else {
                format!(
                    "the app serves {direct_cache:?} directly ({direct}) but this \
                     cell delivers {cache:?} — the PROXY rewrote it"
                )
            }
        }
        None => "MW_T20_DIRECT_BASE unset, so app-vs-proxy cannot be attributed here".to_string(),
    };
    assert!(
        cache.contains("immutable") && cache.contains("max-age="),
        "[{}] a content-hashed asset ({asset}) must be cacheable forever; \
         cache-control was {cache:?}. {attribution}",
        cell.kind
    );
    let etag = first
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .unwrap_or_else(|| panic!("[{}] no ETag on {asset_url}", cell.kind));

    let revalidated = client
        .get(&asset_url)
        .header(reqwest::header::IF_NONE_MATCH, &etag)
        .send()
        .await
        .unwrap();
    assert_eq!(
        revalidated.status(),
        304,
        "[{}] a conditional re-request must be answered 304",
        cell.kind
    );
}

/// The first `assets/…` reference in the served shell, if it looks content-hashed.
///
/// The shipped shell emits **relative** hrefs (`./assets/index-<hash>.js`, from
/// t20-e12's `base: './'`), so the match is on the `assets/` segment and the caller
/// re-roots it under the deploy prefix — which is where a browser would resolve it
/// from, the shell being served at `{prefix}/`.
fn first_hashed_asset(html: &str) -> Option<String> {
    for (at, _) in html.match_indices("assets/") {
        let candidate: String = html[at + "assets/".len()..]
            .chars()
            .take_while(|c| !matches!(c, '"' | '\'' | '`' | ' ' | '>' | ')'))
            .collect();
        // Vite emits `name-<hash>.ext`; the hash is what makes an immutable
        // lifetime safe, so only such a name is asserted on.
        if candidate.contains('-') && (candidate.ends_with(".js") || candidate.ends_with(".css")) {
            return Some(format!(
                "assets/{}",
                candidate.trim_start_matches("assets/")
            ));
        }
    }
    None
}

// ===========================================================================
// 5. TLS posture
// ===========================================================================

/// Behind a TLS-terminating proxy the session cookie carries `Secure` and the
/// response carries HSTS — both derived from the effective scheme, which the app
/// learns from a trusted `X-Forwarded-Proto` (or from `MW_PUBLIC_URL`).
#[tokio::test]
async fn a_tls_fronted_cell_sets_secure_cookies_and_hsts() {
    let Some(cell) = Cell::from_env() else { return };
    let Some(tls_base) = cell.tls_base.clone() else {
        skip("MW_T20_PROXY_TLS_BASE unset — the TLS posture legs did not run.");
        return;
    };
    let ctx = cell.login(&tls_base).await;
    assert!(
        ctx.secure_session,
        "[{}] the session cookie must carry Secure when the client reached us over \
         https — check that the cell sets X-Forwarded-Proto and that its address is \
         inside MW_TRUSTED_PROXIES",
        cell.kind
    );
    let hsts = ctx
        .hsts
        .clone()
        .unwrap_or_else(|| panic!("[{}] no Strict-Transport-Security over https", cell.kind));
    assert!(
        hsts.contains("max-age="),
        "[{}] HSTS was {hsts:?}",
        cell.kind
    );
}

// ===========================================================================
// 6. Sub-path hosting
// ===========================================================================

/// Under `MW_BASE_PATH`, API routes beneath the prefix return **JSON**, not the SPA
/// shell with a `200`.
///
/// That precise defect — a prefix-stripping middleware making every `/mail/api/*`
/// call answer `200 text/html` because the rewritten URI is only ever seen by the
/// fallback — was found and fixed in wave 3 (`t20-e13`, `694f4ff`). It is asserted
/// here so it cannot come back: a status-only check calls it green.
#[tokio::test]
async fn sub_path_api_routes_return_json_not_the_spa_shell() {
    let Some(cell) = Cell::from_env() else { return };
    if cell.prefix.is_empty() {
        skip(
            "MW_T20_PROXY_PREFIX unset — no cell in docker-compose.proxy.yml ships a \
             sub-path configuration (t20-e7 deliberately shipped none rather than a \
             recipe that could not work before t20-e12/e13 landed). Sub-path hosting \
             is therefore NOT proven through a real proxy by this suite; t20-e13 \
             proved the server half against the real binary by crawl. Nothing may \
             claim proxy-fronted sub-path support on the strength of this run.",
        );
        return;
    }
    let client = cell.client();
    let base = cell.base();

    // Unauthenticated API routes under the prefix: JSON 401, never the shell.
    for path in ["/api/me", "/jmap/session", "/api/account/signatures"] {
        let resp = client
            .get(format!("{base}{}{path}", cell.prefix))
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();
        assert!(
            content_type.starts_with("application/json"),
            "[{}] {}{path} answered {status} {content_type} — an HTML body under a \
             200/401 here is the SPA fallback swallowing an API route, the exact \
             defect fixed in 694f4ff",
            cell.kind,
            cell.prefix
        );
        assert_ne!(
            status, 200,
            "[{}] {}{path} must not be 200 unauthenticated",
            cell.kind, cell.prefix
        );
    }

    // The bare prefix redirects to its slashed form, and the slashed form is the
    // shell.
    let bare = cell
        .client()
        .get(format!("{base}{}", cell.prefix))
        .send()
        .await
        .unwrap();
    assert_eq!(
        bare.status(),
        200,
        "[{}] {} must resolve (via 308) to the shell",
        cell.kind,
        cell.prefix
    );
    let content_type = bare
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        content_type.starts_with("text/html"),
        "[{}] the shell under {} was {content_type}",
        cell.kind,
        cell.prefix
    );

    // And the full suite's own client works end to end under the prefix.
    let ctx = cell.login(base).await;
    if let Some(expected) = cell.expect_client_ip.clone() {
        ctx.assert_attributed_to(&host_route(&expected), DECOY, &[])
            .await;
    }
}
