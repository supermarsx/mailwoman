//! t22-e8 — **no HTTP client in this workspace may inherit an ambient proxy.**
//!
//! 26.19 proved the bypass rather than suspecting it: `reqwest::Client::builder()`
//! turns on environment proxy detection by default, and a proxied request is sent
//! to the proxy **by name** — a plain `GET http://host/…` line, or `CONNECT
//! host:443`. The proxy therefore performs its own DNS resolution, so any
//! `.resolve()` pin the caller set is never consulted. The control that settled it
//! pinned a client to `pinned.invalid` (RFC 6761 — unresolvable in DNS anywhere)
//! and still reached a proxy carrying that hostname, which is only possible if the
//! third party resolved it. `fe09384` closed this for the image proxy;
//! `crates/mw-egress` carries it for the gated fetch path. Every *other* client in
//! the tree was still open, and each one carries something worth protecting: API
//! keys, account passwords, bearer tokens, OAuth codes, HMAC-signed payloads.
//!
//! Two tests, doing two different jobs:
//!
//!   1. [`a_client_without_no_proxy_reaches_the_proxy`] — the behaviour, with a
//!      control. Two builders identical except for one call; the one without
//!      `.no_proxy()` is *shown* handing the request to a proxy.
//!   2. [`every_client_constructed_in_crate_sources_sets_no_proxy`] — the
//!      structural rule, over the whole tree. A per-call-site test is easy to get
//!      90% right, and the client it misses is invisible; this one fails for the
//!      *next* client somebody adds, which is the one nobody will think to check.
//!
//! [`the_structural_check_flags_a_client_built_without_no_proxy`] proves the
//! scanner in (2) actually detects — a lint that silently stopped matching would
//! pass this file forever.

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// ── 1. the behaviour, with a control ──────────────────────────────────────────

async fn spawn_origin(body: &'static [u8]) -> SocketAddr {
    let app = axum::Router::new().route(
        "/",
        axum::routing::get(move || async move { axum::body::Bytes::from_static(body) }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

/// A stand-in proxy that counts connections and hangs up. Anything that reaches
/// it was handed the hostname to resolve.
async fn spawn_counting_proxy() -> (SocketAddr, Arc<AtomicUsize>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&hits);
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            counter.fetch_add(1, Ordering::SeqCst);
            drop(stream);
        }
    });
    (addr, hits)
}

#[tokio::test]
async fn a_client_without_no_proxy_reaches_the_proxy() {
    // The two builders below differ by EXACTLY one call. `pinned.invalid` cannot
    // resolve in DNS anywhere, so a successful body proves the `.resolve()` pin was
    // honoured, and a proxy hit proves the name was handed to a third party.
    //
    // The proxy is set explicitly rather than through `HTTP_PROXY` because
    // `std::env::set_var` is `unsafe` in this edition and would leak across every
    // other test in the binary. That costs nothing: `ClientBuilder::no_proxy` is
    // one function — it does `self.config.proxies.clear(); self.config
    // .auto_sys_proxy = false;` (reqwest 0.12.28, `async_impl/client.rs:1427`) —
    // so the explicit half exercised here and the environment half are closed by
    // the same call. 26.19 proved the environment half live.
    let origin = spawn_origin(b"from-the-pinned-origin").await;
    let (proxy_addr, hits) = spawn_counting_proxy().await;
    let proxy = || reqwest::Proxy::all(format!("http://{proxy_addr}")).unwrap();

    // CONTROL — no `.no_proxy()`. This is what every client in this workspace
    // looked like before this change.
    let unhardened = reqwest::Client::builder()
        .proxy(proxy())
        .resolve("pinned.invalid", origin)
        .build()
        .unwrap();
    let result = unhardened.get("http://pinned.invalid/").send().await;
    assert!(
        result.is_err(),
        "the control must NOT reach the origin — it went to the proxy, which hung up"
    );
    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "the control must hand the request to the proxy: that is the bypass"
    );

    // THE SAME BUILDER, plus one call.
    let hardened = reqwest::Client::builder()
        .proxy(proxy())
        .resolve("pinned.invalid", origin)
        .no_proxy()
        .build()
        .unwrap();
    let resp = hardened
        .get("http://pinned.invalid/")
        .send()
        .await
        .expect("the pinned address must be contacted, not the proxy");
    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(
        resp.bytes().await.unwrap().as_ref(),
        b"from-the-pinned-origin"
    );
    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "still 1 — the hardened client added no proxy connection of its own"
    );
}

// ── 2. the structural rule ────────────────────────────────────────────────────

/// One client construction that does not refuse ambient proxies.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Offence {
    line: usize,
    shape: &'static str,
}

/// Every way this workspace can end up with a client that reads the environment's
/// proxy variables.
///
/// `reqwest::Client::new()` matters as much as `Client::builder()` and is easier to
/// miss: it is defined as `builder().build().unwrap()`, so it carries the same
/// `auto_sys_proxy = true` while being invisible to a `builder()` grep. The plan
/// this lane started from searched only for `builder()` and consequently missed
/// most of the tree. `reqwest::get()` is the same trap one level up — it builds a
/// default client per call.
fn offences(source: &str) -> Vec<Offence> {
    let src = strip_line_comments(source);
    let line_of = |byte: usize| src[..byte].matches('\n').count() + 1;
    let mut out = Vec::new();

    // No builder exists at these call sites, so there is nowhere to put the call:
    // they are offences wherever they appear.
    for (shape, needle) in [
        ("reqwest::Client::new()", "reqwest::Client::new("),
        ("reqwest::get()", "reqwest::get("),
    ] {
        for (at, _) in src.match_indices(needle) {
            out.push(Offence {
                line: line_of(at),
                shape,
            });
        }
    }

    // A builder chain is fine as long as `.no_proxy()` is somewhere in it. The
    // chain always terminates in `.build()`, which bounds the window to search.
    for shape in ["Client::builder()", "ClientBuilder::new()"] {
        for (at, _) in src.match_indices(shape) {
            let rest = &src[at..];
            let end = rest
                .find(".build()")
                .map_or_else(|| rest.len().min(600), |e| e + ".build()".len());
            if rest[..end].contains(".no_proxy()") {
                continue;
            }
            // The sanctioned exception: a builder handed straight to
            // `mw_egress::harden_client`, which sets `.no_proxy()` itself and is
            // proven to do so by `policy_through_crate.rs`.
            if src[at.saturating_sub(80)..at].contains("harden_client(") {
                continue;
            }
            out.push(Offence {
                line: line_of(at),
                shape,
            });
        }
    }

    out.sort();
    out
}

/// Drop `//` comments so prose about `Client::builder()` is not read as code. A
/// `//` preceded by `:` is left alone — that is a URL inside a string literal, not
/// a comment.
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|line| match line.find("//") {
            Some(i) if !line[..i].ends_with(':') => &line[..i],
            _ => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/mw-egress sits two levels below the workspace root")
        .to_path_buf()
}

fn rust_sources_under(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_sources_under(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Every `crates/*/src/**/*.rs` in the workspace, except `mw-egress`'s own.
///
/// **Why `mw-egress` is excluded and nothing else is.** This crate *is* the egress
/// policy. `harden_client` sets `.no_proxy()` for the gated fetch path, and
/// `crates/mw-egress/src/proxy/**` is the deliberately configured upstream-proxy
/// route — the one place in the tree where a proxy is a decision rather than an
/// accident. Both are covered by this crate's own tests. A blanket rule applied
/// here would forbid the sanctioned path from existing.
///
/// **Why `tests/` directories are out of scope.** Integration tests bind loopback
/// origins in-process; a proxy there is a test-hygiene question, not an egress
/// surface — and `policy_through_crate.rs` must be free to set one on purpose.
/// In-crate `#[cfg(test)]` modules ARE in scope, deliberately: exempting them would
/// mean parsing module boundaries, and a rule with no exemptions cannot be gamed by
/// putting a client somewhere the scanner decided not to look.
fn scanned_sources() -> Vec<PathBuf> {
    let root = workspace_root();
    let mut out = Vec::new();
    for entry in std::fs::read_dir(root.join("crates"))
        .expect("crates/ is readable")
        .flatten()
    {
        if entry.file_name() == "mw-egress" {
            continue;
        }
        rust_sources_under(&entry.path().join("src"), &mut out);
    }
    out.sort();
    out
}

/// Client constructions the scanner must keep finding. Well below the ~37 present
/// when this landed, but high enough that a scanner which quietly stopped matching
/// — a rename upstream, a bad edit here — fails instead of passing vacuously.
const MINIMUM_SITES_SCANNED: usize = 30;

#[test]
fn every_client_constructed_in_crate_sources_sets_no_proxy() {
    let mut found = Vec::new();
    let mut sites = 0usize;

    for path in scanned_sources() {
        let src = std::fs::read_to_string(&path).expect("source is readable");
        let stripped = strip_line_comments(&src);
        sites += stripped.matches("reqwest::Client::new(").count()
            + stripped.matches("reqwest::get(").count()
            + stripped.matches("Client::builder()").count()
            + stripped.matches("ClientBuilder::new()").count();
        for offence in offences(&src) {
            found.push(format!(
                "{}:{} — {} without .no_proxy()",
                path.display(),
                offence.line,
                offence.shape
            ));
        }
    }

    // Asserted BEFORE the emptiness of `found`, so a run that scanned nothing
    // cannot pass as a run that found nothing wrong.
    assert!(
        sites >= MINIMUM_SITES_SCANNED,
        "only {sites} client constructions scanned (expected >= {MINIMUM_SITES_SCANNED}) — \
         the scanner has stopped matching and is no longer checking anything"
    );

    assert!(
        found.is_empty(),
        "these HTTP clients inherit the environment's proxy settings, so a third \
         party resolves their hostnames and sees their traffic. Add `.no_proxy()` \
         to the builder (`reqwest::Client::new()` has no builder — write \
         `reqwest::Client::builder().no_proxy().build()`, which panics on failure \
         exactly as `new()` does). If the client genuinely needs a proxy, it \
         belongs behind `mw-egress`'s configured upstream-proxy path, not on an \
         ambient environment variable:\n  {}",
        found.join("\n  ")
    );
}

#[test]
fn the_structural_check_flags_a_client_built_without_no_proxy() {
    // Demonstrating that the check detects, rather than asserting it. Each shape
    // below is one this workspace actually contained before this change.
    let bad = r#"
        let a = reqwest::Client::new();
        let b = reqwest::Client::builder().build().unwrap();
        let c = reqwest::Client::builder()
            .user_agent("x")
            .timeout(t)
            .build()?;
        let d = openidconnect::reqwest::ClientBuilder::new().build()?;
        let e = reqwest::get(&url).await?;
    "#;
    let flagged = offences(bad);
    assert_eq!(
        flagged.len(),
        5,
        "every unhardened shape must be caught, got {flagged:?}"
    );
    assert_eq!(
        flagged.iter().map(|o| o.shape).collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "reqwest::Client::new()",
            "reqwest::get()",
            "Client::builder()",
            "ClientBuilder::new()",
        ])
    );

    // And that it does not fire on the hardened forms — a check that flags
    // everything is no more useful than one that flags nothing.
    let good = r#"
        let a = reqwest::Client::builder().no_proxy().build().unwrap();
        let b = reqwest::Client::builder()
            .user_agent("x")
            .no_proxy()
            .build()?;
        let c = harden_client(reqwest::Client::builder(), &target.host, target.addr)
            .build()?;
        // prose mentioning reqwest::Client::new() and Client::builder() is not code
    "#;
    assert_eq!(
        offences(good),
        vec![],
        "hardened clients must not be flagged"
    );

    // The `.build()` of the NEXT chain must not satisfy the previous one.
    let sequential = r#"
        let a = reqwest::Client::builder().build()?;
        let b = reqwest::Client::builder().no_proxy().build()?;
    "#;
    assert_eq!(
        offences(sequential).len(),
        1,
        "the window must stop at its own .build()"
    );
}
