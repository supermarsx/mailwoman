//! Live egress-transport tests against a REAL forward proxy (26.20 t22-e-e2e).
//!
//! `t22-e11`'s `crates/mw-egress/tests/proxy_transport.rs` drives the transport
//! against **in-process `TcpListener` stand-ins** — a recording `CONNECT` proxy and
//! a recording SOCKS5 server. That is the *stronger* instrument for the properties
//! it asserts, because it inspects the bytes the far side actually received. What
//! it cannot show is how a real proxy behaves, and `t22-e11` recorded a prediction
//! about exactly that: **stock Squid restricts `CONNECT` to `SSL_ports` (443, 563),
//! so a plaintext route through an unmodified Squid is refused by SQUID, not by
//! us.** Until this file that prediction had nothing behind it.
//!
//! Measured before it was written, against `ubuntu/squid:latest` with its shipped
//! config (`acl SSL_ports port 443` / `http_access deny CONNECT !SSL_ports`):
//!
//! ```text
//! CONNECT example.com:443 -> HTTP/1.1 200 Connection established
//! CONNECT example.com:80  -> HTTP/1.1 403 Forbidden
//! ```
//!
//! So the prediction **holds**, and the interesting assertion is not merely "the
//! plaintext route fails" — it is *which refusal it produces*. `t22-e11` split
//! `ProxyAuthRejected` out of `ProxyRejected` deliberately, and documented that an
//! HTTP `403` must stay in `ProxyRejected` because it is the proxy refusing the
//! **destination** (authorization of the target) rather than rejecting **us**
//! (authentication). An operator told "check your proxy password" when the real
//! answer is "your proxy will not reach that host" has been sent to the wrong
//! place. This file is the first thing to check that split against a proxy that
//! was not written by us.
//!
//! ## Running it
//!
//! ```text
//! docker run -d --name mw-squid-e2e -p 3128:3128 ubuntu/squid:latest
//! MW_T22_SQUID=127.0.0.1:3128 cargo test -p mw-server --test t22_egress_live -- --nocapture
//! ```
//!
//! **Requires outbound network**: Squid resolves and dials the origin itself. The
//! tests SKIP LOUDLY without `MW_T22_SQUID` — they print why and pass, because a
//! machine with no Docker must not fail the suite. A skip is never reported as a
//! verification; read the printed line, not the green tick.

mod common;

use mw_egress::proxy::{ProxyRefusal, ProxyRoute, ProxyScheme, fetch_via_proxy};

/// `host:port` of a live forward proxy, or `None` to skip.
fn squid() -> Option<(String, u16)> {
    let raw = std::env::var("MW_T22_SQUID").ok()?;
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let (h, p) = raw.rsplit_once(':')?;
    Some((h.to_string(), p.parse().ok()?))
}

fn route(host: String, port: u16, allow_plaintext: bool) -> ProxyRoute {
    ProxyRoute {
        id: "e2e-squid".to_string(),
        scheme: ProxyScheme::HttpConnect,
        host,
        port,
        auth: None,
        allow_plaintext,
    }
}

/// Print the reason and return, so a skip is visible in the output rather than
/// indistinguishable from a pass.
macro_rules! skip_unless_squid {
    () => {
        match squid() {
            Some(v) => v,
            None => {
                common::gate::skip(
                    "[t22-e2e] set MW_T22_SQUID=host:port to a live forward proxy \
                     (docker run -d -p 3128:3128 ubuntu/squid:latest). Nothing was verified.",
                );
                return;
            }
        }
    };
}

#[tokio::test]
async fn https_origin_tunnels_through_a_real_squid() {
    let (h, p) = skip_unless_squid!();
    let r = route(h, p, false);

    let fetched = fetch_via_proxy(
        "https://example.com/".parse().expect("url"),
        &r,
        "text/html",
    )
    .await;

    // The control for the refusal test below: without it, "the plaintext route is
    // refused" also holds for a rig where NOTHING gets through the proxy — a
    // misconfigured container, a blocked port, no egress at all.
    assert!(
        fetched.traversed_proxy,
        "https origin did not traverse the proxy: {:?}",
        fetched.outcome
    );
    assert!(
        fetched.outcome.is_ok(),
        "https origin through Squid should succeed, got {:?}",
        fetched.outcome
    );
}

#[tokio::test]
async fn plaintext_origin_is_refused_by_squid_and_lands_in_proxy_rejected() {
    let (h, p) = skip_unless_squid!();
    // `allow_plaintext: true` — so OUR policy permits it and anything that refuses
    // is the proxy, which is the whole point. With it false the refusal would be
    // ours and the test would prove nothing about Squid.
    let r = route(h, p, true);

    let fetched =
        fetch_via_proxy("http://example.com/".parse().expect("url"), &r, "text/html").await;

    match fetched.outcome {
        Err(ProxyRefusal::ProxyRejected(detail)) => {
            // Squid answers `403 Forbidden` to CONNECT on a non-`SSL_ports` port.
            assert!(
                detail.contains("403"),
                "expected Squid's 403 in the refusal detail, got {detail:?}"
            );
        }
        // THE discrimination this file exists for. A 403 is the proxy refusing the
        // DESTINATION; routing it to ProxyAuthRejected would tell an operator to
        // check a password that is not the problem, and `t22-e11` documented that
        // split as deliberate. Until now nothing checked it against a real proxy.
        Err(ProxyRefusal::ProxyAuthRejected(what)) => panic!(
            "Squid's destination refusal was misclassified as an AUTH failure ({what}) — \
             an operator would be sent to fix a credential that is not the problem"
        ),
        other => panic!(
            "expected ProxyRejected carrying Squid's 403, got {other:?} — if this is Ok, the \
             proxy's SSL_ports ACL has been widened and the test is no longer measuring it"
        ),
    }
}
