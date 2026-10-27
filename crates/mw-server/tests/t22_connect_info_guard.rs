//! t22-e9 — **every production mount must install `ConnectInfo`.**
//!
//! `scope_mw::proxy::client_ip` returns `Option<IpAddr>` and yields `None` when the
//! serve path installed no `ConnectInfo` — the peer address is the floor of the
//! trusted-proxy model, and without it a forwarded header alone never produces a
//! client IP (`proxy.rs`'s own `client_ip_requires_connect_info` pins that).
//!
//! Several source-address-keyed controls are built on that seam and **fail open**
//! when it yields `None`: the pre-auth admin login ban (`admin.rs`, whose
//! `without_a_peer_address_nothing_is_counted_or_banned` documents the choice) and
//! the `/api/discover` rate limit. Those fail-opens are only defensible because
//! every production mount installs `ConnectInfo`, so `None` is reachable in an
//! in-process test transport and nowhere else.
//!
//! That is a claim about wiring, not about logic, and it is the kind of claim an
//! unrelated refactor can silently falsify — someone adds a listener, forgets
//! `into_make_service_with_connect_info`, and every source-IP check on that mount
//! degrades to "unknown" with nothing to notice. This test turns the claim into
//! something that fails loudly.
//!
//! Scope, stated rather than implied: this checks the **wiring in `main.rs`** and the
//! one serve loop it calls. Since 9ae23aee ("bound HTTP admission and enable HTTP2
//! serving") `main.rs` no longer calls `axum::serve` itself: both production arms
//! (plaintext/PROXY-protocol and TLS) call `mw_server::inbound_http::serve`, a
//! hand-rolled hyper loop that inserts `ConnectInfo(peer)` into each request's
//! extensions before handing it to the router. So the check has two parts:
//!
//! 1. `main.rs` has exactly [`EXPECTED_SERVE_SITES`] production serve sites, and each
//!    one is either `inbound_http::serve(` or an `axum::serve(` paired with
//!    `into_make_service_with_connect_info`;
//! 2. `inbound_http::serve` takes `peer` from `listener.accept()`, inserts
//!    `ConnectInfo(peer)` into the request parts, and its single router call is made
//!    with those same parts, after the insert.
//!
//! No separate admin bind exists; every other `axum::serve` in the workspace is inside
//! a `#[cfg(test)]` module or a test file, except `mw-mock-jmap`'s dev mock binary,
//! which is not the product server. This does not prove hyper's or axum's own
//! behaviour, and it is not a substitute for the runtime assertion in `proxy.rs`.

use std::path::Path;

/// A direct axum serve call, which must be paired with the connect-info make-service.
const AXUM_SERVE: &str = "axum::serve(";
const CONNECT_INFO: &str = "into_make_service_with_connect_info";
/// The bounded serve loop both production arms use since 9ae23aee.
const INBOUND_SERVE: &str = "inbound_http::serve(";

/// Both production serve arms — plaintext (via `ProxyAcceptor`) and TLS. Asserted as
/// a floor so a scan that quietly stopped matching fails rather than passing
/// vacuously: "found no unguarded mounts" and "found no mounts" are not the same
/// result, and they look identical in a green test.
const EXPECTED_SERVE_SITES: usize = 2;

/// Needles inside `inbound_http::serve`.
const SERVE_FN: &str = "pub async fn serve<";
const ACCEPT: &str = "connection=listener.accept()";
const PEER_BINDING: &str = "let (io, peer) =";
const INSERT: &str = "parts.extensions.insert(ConnectInfo(peer));";
const SPLIT: &str = "let (mut parts, body) = request.into_parts();";
const ROUTER_CALL: &str = ".call(";
const REBUILD: &str = "Request::from_parts(parts,";

fn read(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("{} is readable: {e}", p.display()))
        // A Windows checkout under autocrlf has CRLF; the needles are written with LF.
        .replace("\r\n", "\n")
}

fn line_of(src: &str, at: usize) -> usize {
    src[..at].matches('\n').count() + 1
}

/// Part 1: the production serve sites in `main.rs`. Returns (sites, problems).
fn check_main(src: &str) -> (usize, Vec<String>) {
    let mut sites = 0usize;
    let mut problems = Vec::new();
    for (at, _) in src.match_indices(AXUM_SERVE) {
        sites += 1;
        // The make-service is the second argument of the same call, so it is a short
        // way past the opening paren. Bound the window to this call rather than
        // scanning to end-of-file, or the NEXT arm's connect-info would vouch for an
        // unguarded one.
        let window_end = src[at..]
            .find("\n        }")
            .map_or_else(|| src.len(), |e| at + e);
        if !src[at..window_end].contains(CONNECT_INFO) {
            problems.push(format!(
                "main.rs:{}: axum::serve without {CONNECT_INFO}",
                line_of(src, at)
            ));
        }
    }
    sites += src.matches(INBOUND_SERVE).count();
    (sites, problems)
}

/// Part 2: `inbound_http::serve` installs `ConnectInfo(peer)` on every request it
/// hands to the router.
fn check_inbound(src: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let Some(start) = src.find(SERVE_FN) else {
        return vec![format!("inbound_http.rs: `{SERVE_FN}` not found")];
    };
    // The function ends at the first column-0 closing brace after its signature.
    let end = src[start..]
        .find("\n}\n")
        .map_or_else(|| src.len(), |e| start + e);
    let body = &src[start..end];

    for needle in [ACCEPT, PEER_BINDING, SPLIT] {
        if !body.contains(needle) {
            problems.push(format!("inbound_http::serve: `{needle}` not found"));
        }
    }
    let inserts: Vec<usize> = body.match_indices(INSERT).map(|(i, _)| i).collect();
    let calls: Vec<usize> = body.match_indices(ROUTER_CALL).map(|(i, _)| i).collect();
    if inserts.len() != 1 {
        problems.push(format!(
            "inbound_http::serve: expected exactly one `{INSERT}`, found {}",
            inserts.len()
        ));
    }
    if calls.len() != 1 {
        problems.push(format!(
            "inbound_http::serve: expected exactly one router `{ROUTER_CALL}` (every request \
             reaches the app through it), found {} — a second call site could hand the \
             router a request without ConnectInfo",
            calls.len()
        ));
    }
    if let (Some(&ins), Some(&call)) = (inserts.first(), calls.first()) {
        if let Some(split) = body.find(SPLIT)
            && !(split < ins && ins < call)
        {
            problems.push(
                "inbound_http::serve: ConnectInfo must be inserted after the request is \
                 split and before the router call"
                    .into(),
            );
        }
        // The router must be called with the same `parts` that received the insert.
        let after_call = &body[call..];
        let stmt_end = after_call.find(".await").unwrap_or(after_call.len());
        if !after_call[..stmt_end].contains(REBUILD) {
            problems.push(format!(
                "inbound_http::serve: the router call is not made with `{REBUILD}` — the \
                 request it receives may not carry ConnectInfo"
            ));
        }
    }
    problems
}

#[test]
fn every_production_mount_installs_connect_info() {
    let main_rs = read("src/main.rs");
    let inbound_rs = read("src/inbound_http.rs");

    let (sites, mut problems) = check_main(&main_rs);
    assert_eq!(
        sites, EXPECTED_SERVE_SITES,
        "expected {EXPECTED_SERVE_SITES} production serve sites in main.rs, found {sites} — \
         either a mount was added or removed (update this floor deliberately), or this \
         scan has stopped matching and is no longer checking anything"
    );
    if main_rs.contains(INBOUND_SERVE) {
        problems.extend(check_inbound(&inbound_rs));
    }

    assert!(
        problems.is_empty(),
        "these production mounts do not install `ConnectInfo`, so `client_ip` returns \
         `None` for every request they serve and every source-address-keyed control \
         on them — the admin login ban, the /api/discover rate limit — silently \
         fails open:\n  {}",
        problems.join("\n  ")
    );
}

#[test]
fn the_guard_detects_a_mount_that_is_missing_connect_info() {
    // Demonstrating that the check can fail, rather than asserting that it does.
    // A scan whose needle stopped matching would report "no unguarded mounts"
    // forever, and would look exactly like this test passing.
    let guarded = "\
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(shutdown_signal())
            .await?;
        }";
    let unguarded = "\
            axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
        }";

    assert_eq!(check_main(guarded), (1, vec![]));
    let (sites, problems) = check_main(unguarded);
    assert_eq!(sites, 1);
    assert_eq!(
        problems.len(),
        1,
        "the unguarded shape must be reported — if this fails, the needles no longer \
         discriminate and the scan above is incapable of reporting a problem"
    );
}

#[test]
fn the_guard_detects_seeded_breaks_in_the_real_serve_loop() {
    // Seeded breaks against the real source: each mutation must be reported, and the
    // unmutated source must not be. This is what keeps part 2 from passing vacuously.
    let real = read("src/inbound_http.rs");
    assert!(
        check_inbound(&real).is_empty(),
        "real source must pass: {:?}",
        check_inbound(&real)
    );

    let dropped_insert = real.replacen(INSERT, "", 1);
    assert!(
        !check_inbound(&dropped_insert).is_empty(),
        "removing the ConnectInfo insert must be reported"
    );

    let wrong_value = real.replacen(INSERT, "parts.extensions.insert(ConnectInfo(fake));", 1);
    assert!(
        !check_inbound(&wrong_value).is_empty(),
        "inserting something other than the accepted peer must be reported"
    );

    let bypass = real.replacen(
        SPLIT,
        "let _ = TowerToHyperService::new(app.clone()).call(request);\n                    \
         let (mut parts, body) = request.into_parts();",
        1,
    );
    assert!(
        !check_inbound(&bypass).is_empty(),
        "a second router call that skips the insert must be reported"
    );

    let renamed = real.replacen(SERVE_FN, "pub async fn serve_inner<", 1);
    assert!(
        !check_inbound(&renamed).is_empty(),
        "a scan that no longer finds the serve loop must fail, not pass"
    );
}
