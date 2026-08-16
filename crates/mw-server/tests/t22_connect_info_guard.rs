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
//! Scope, stated rather than implied: this checks the **wiring in `main.rs`**, which
//! is the whole production serve surface for the app router (verified: two
//! `axum::serve` calls, plaintext/PROXY-protocol and TLS; no separate admin bind;
//! every other `axum::serve` in the workspace is inside a `#[cfg(test)]` module,
//! except `mw-mock-jmap`'s dev mock binary, which is not the product server). It
//! does not prove axum's own behaviour, and it is not a substitute for the runtime
//! assertion in `proxy.rs`.

use std::path::Path;

/// The serve calls that must each be paired with the connect-info make-service.
const SERVE: &str = "axum::serve(";
const CONNECT_INFO: &str = "into_make_service_with_connect_info";

/// Both production serve arms — plaintext (via `ProxyAcceptor`) and TLS. Asserted as
/// a floor so a scan that quietly stopped matching fails rather than passing
/// vacuously: "found no unguarded mounts" and "found no mounts" are not the same
/// result, and they look identical in a green test.
const EXPECTED_SERVE_SITES: usize = 2;

#[test]
fn every_production_mount_installs_connect_info() {
    let main_rs = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs");
    let src = std::fs::read_to_string(&main_rs).expect("mw-server/src/main.rs is readable");

    let mut sites = 0usize;
    let mut unguarded = Vec::new();

    for (at, _) in src.match_indices(SERVE) {
        sites += 1;
        // The make-service is the second argument of the same call, so it is a short
        // way past the opening paren. Bound the window to this call rather than
        // scanning to end-of-file, or the NEXT arm's connect-info would vouch for an
        // unguarded one.
        let window_end = src[at..]
            .find("\n        }")
            .map_or_else(|| src.len(), |e| at + e);
        if !src[at..window_end].contains(CONNECT_INFO) {
            let line = src[..at].matches('\n').count() + 1;
            unguarded.push(format!("main.rs:{line}"));
        }
    }

    assert_eq!(
        sites, EXPECTED_SERVE_SITES,
        "expected {EXPECTED_SERVE_SITES} production serve sites in main.rs, found {sites} — \
         either a mount was added or removed (update this floor deliberately), or this \
         scan has stopped matching and is no longer checking anything"
    );

    assert!(
        unguarded.is_empty(),
        "these production mounts do not install `ConnectInfo`, so `client_ip` returns \
         `None` for every request they serve and every source-address-keyed control \
         on them — the admin login ban, the /api/discover rate limit — silently \
         fails open. Add `.into_make_service_with_connect_info::<SocketAddr>()`:\n  {}",
        unguarded.join("\n  ")
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

    assert!(
        guarded.contains(SERVE) && guarded.contains(CONNECT_INFO),
        "the guarded shape must match both needles"
    );
    assert!(
        unguarded.contains(SERVE) && !unguarded.contains(CONNECT_INFO),
        "the unguarded shape must match the serve needle and NOT the connect-info one \
         — if this fails, the needles no longer discriminate and the scan above is \
         incapable of reporting a problem"
    );
}
