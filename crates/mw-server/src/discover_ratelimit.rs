//! Rate limit for `POST /api/discover` (t22-e9).
//!
//! # Why this endpoint and not others
//! `/api/discover` takes an email address from the request body, splits it, and
//! fetches `https://autoconfig.{domain}/…`, `https://{domain}/.well-known/…` and
//! `https://autodiscover.{domain}/…`. It is **unauthenticated** (the handler's own
//! comment: "Pre-login, so unauthenticated") and CSRF-exempt. The SSRF address gate
//! bounds *where* those fetches may go; it does nothing about *how often* an
//! anonymous caller can make us go there. Without a bound the endpoint is a probe
//! amplifier: free outbound requests from our address, with timing observable.
//!
//! # The key
//! [`crate::scope_mw::proxy::client_ip`] — the peer address as the floor, refined by
//! a forwarded header **only** when `MW_FORWARDED_MODE` selects one *and* the peer
//! is inside `MW_TRUSTED_PROXIES`. Reading `X-Forwarded-For` directly would make the
//! limit bypassable by rotating a header, which is bug t20 B1 in a new place;
//! `proxy.rs`'s own `client_ip_requires_connect_info` already pins that a header
//! with no `ConnectInfo` yields `None` whatever it says.
//!
//! IPv6 is keyed by **/64 prefix**, not by address: a single host is routinely
//! handed a whole /64, so per-address keying lets one machine mint 2⁶⁴ identities
//! and evade the limit entirely. IPv4-mapped forms (`::ffff:a.b.c.d`) are unwrapped
//! FIRST — keyed as v6 they would all collapse into the single prefix `0:0:0:0::/64`
//! and share one bucket, so any one client could exhaust the budget for every
//! IPv4-mapped client at once.
//!
//! # FAIL-OPEN, deliberately, and guarded
//! No `ConnectInfo` ⇒ no client IP ⇒ **not counted**. That matches the pre-auth admin
//! login ban (`admin.rs`'s `without_a_peer_address_nothing_is_counted_or_banned`)
//! and is safe only because every production mount installs `ConnectInfo`, making
//! `None` reachable in an in-process test transport and nowhere else. That is a
//! claim about wiring which an unrelated refactor could silently falsify, so it is
//! pinned by `tests/t22_connect_info_guard.rs` rather than assumed.
//!
//! # The budget, and the trade nobody sees later
//! **Burst 20, refill 0.5/s (30/min sustained)** per key, and the number is squeezed
//! from BOTH sides — whichever we pick, one of two failure modes is the one we
//! accepted:
//!
//!   * **Too generous** and the amplifier stays useful.
//!   * **Too tight and an office breaks.** A shared corporate NAT egress is ONE key
//!     for everyone behind it. At the burst-5 figure first proposed, five colleagues
//!     setting up accounts on a Monday morning would lock out the sixth — not an
//!     edge case, that is onboarding, and it presents as "the product is broken"
//!     with nobody suspecting a rate limiter.
//!
//! 20/30-per-minute leaves a NAT'd office workable while staying ~5× tighter than
//! the image proxy's 120 burst / 1-per-second (which is sized for a reader loading a
//! mailbox of images — a different shape of traffic entirely). The abuse case this
//! must stop is thousands of probes, orders of magnitude above either number. **A
//! future tuner sees only the abuse side unless the other is written down, which is
//! how a limiter gets tightened into an outage.**
//!
//! # The global ceiling
//! Per-key limiting cannot bound a **distributed** caller: 30/min each across ten
//! thousand addresses is still 300 000/min of outbound discovery. Per-key buckets are
//! simply the wrong instrument for that, and the eviction cap does not help — it
//! bounds *memory*, not *rate*. So there is also an aggregate ceiling across all
//! keys, which is the only thing that bounds the total regardless of how many
//! identities exist. It is set where no honest deployment reaches it: a deployment
//! onboarding 300 accounts in one minute is implausible, and an attacker with
//! unlimited addresses is still held to that.
//!
//! Per-key is charged FIRST, so a single abusive key is refused by its own bucket
//! without spending the shared budget everyone else depends on.

use std::net::IpAddr;
use std::sync::{Mutex, OnceLock};

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::scope_mw::proxy::client_ip;
use crate::scope_mw::rate_limit::KeyedRateLimiter;

/// Most discovery attempts one source may spend at once.
const BURST: f64 = 20.0;
/// Sustained refill — 0.5/s is 30/min.
const REFILL_PER_SEC: f64 = 0.5;
/// Retained source keys. Bounded because the key is attacker-chosen; see
/// [`crate::scope_mw::rate_limit`].
const KEY_CAPACITY: usize = 10_000;

/// Aggregate burst across every source.
const GLOBAL_BURST: f64 = 100.0;
/// Aggregate sustained rate — 5/s is 300/min.
const GLOBAL_REFILL_PER_SEC: f64 = 5.0;

fn per_source() -> &'static Mutex<KeyedRateLimiter> {
    static RL: OnceLock<Mutex<KeyedRateLimiter>> = OnceLock::new();
    RL.get_or_init(|| Mutex::new(KeyedRateLimiter::new(BURST, REFILL_PER_SEC, KEY_CAPACITY)))
}

fn global() -> &'static Mutex<KeyedRateLimiter> {
    static RL: OnceLock<Mutex<KeyedRateLimiter>> = OnceLock::new();
    RL.get_or_init(|| {
        Mutex::new(KeyedRateLimiter::new(
            GLOBAL_BURST,
            GLOBAL_REFILL_PER_SEC,
            1,
        ))
    })
}

/// The single key the aggregate bucket uses. Not an address, so it cannot collide
/// with a source key.
const GLOBAL_KEY: &str = "*aggregate*";

/// The rate-limit key for a source address: the exact address for IPv4, the /64
/// prefix for IPv6, with IPv4-mapped forms unwrapped to IPv4 first.
fn rate_key(ip: IpAddr) -> String {
    match ip {
        IpAddr::V4(v4) => v4.to_string(),
        IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            // `::ffff:a.b.c.d` is an IPv4 client. Keyed as v6 every such address
            // would share the prefix `0:0:0:0::/64` and so share one bucket.
            Some(v4) => v4.to_string(),
            None => {
                let s = v6.segments();
                format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
            }
        },
    }
}

/// `route_layer` for `/api/discover`. Refuses with `429` once a source, or the
/// deployment as a whole, exceeds its budget.
pub(crate) async fn guard(req: Request, next: Next) -> Response {
    let headers = req.headers().clone();
    let Some(ip) = client_ip(&headers, req.extensions()) else {
        // Fail-open — see the module docs. Pinned by `t22_connect_info_guard.rs`.
        return next.run(req).await;
    };

    // Per-source first: an abusive key is stopped by its own bucket rather than
    // draining the aggregate budget that every other source shares.
    if !per_source()
        .lock()
        .expect("discover rate-limit lock")
        .check(&rate_key(ip))
    {
        return refused();
    }
    if !global()
        .lock()
        .expect("discover rate-limit lock")
        .check(GLOBAL_KEY)
    {
        return refused();
    }
    next.run(req).await
}

/// `429`, saying nothing about which of the two budgets was spent — a caller
/// learning it had tripped the *aggregate* limit would learn about other traffic.
fn refused() -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        axum::Json(serde_json::json!({ "error": "too many discovery requests" })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipv6_is_keyed_by_prefix_so_one_host_cannot_mint_identities() {
        let a: IpAddr = "2001:db8:1:2:aaaa:bbbb:cccc:dddd".parse().unwrap();
        let b: IpAddr = "2001:db8:1:2:0:0:0:1".parse().unwrap();
        assert_eq!(
            rate_key(a),
            rate_key(b),
            "two addresses in one /64 are one host and must share a bucket"
        );

        let other: IpAddr = "2001:db8:1:3::1".parse().unwrap();
        assert_ne!(
            rate_key(a),
            rate_key(other),
            "a different /64 is a different subscriber and must not be lumped in"
        );
    }

    #[test]
    fn ipv4_mapped_addresses_do_not_collapse_into_one_bucket() {
        let a: IpAddr = "::ffff:192.0.2.1".parse().unwrap();
        let b: IpAddr = "::ffff:198.51.100.9".parse().unwrap();
        assert_ne!(
            rate_key(a),
            rate_key(b),
            "keyed as v6 these share the prefix 0:0:0:0::/64 — one client would then \
             exhaust the budget for every IPv4-mapped client at once"
        );
        assert_eq!(rate_key(a), "192.0.2.1", "mapped form must unwrap to IPv4");
    }

    #[test]
    fn ipv4_is_keyed_exactly() {
        let a: IpAddr = "198.51.100.7".parse().unwrap();
        let b: IpAddr = "198.51.100.8".parse().unwrap();
        assert_eq!(rate_key(a), "198.51.100.7");
        assert_ne!(rate_key(a), rate_key(b));
    }

    #[test]
    fn the_aggregate_key_cannot_collide_with_a_source_key() {
        // Every source key is an IPv4 address or an IPv6 /64; neither can produce
        // this string, so the aggregate bucket cannot be spent by a crafted source.
        for ip in [
            "0.0.0.0",
            "255.255.255.255",
            "::",
            "::ffff:0.0.0.0",
            "ffff:ffff:ffff:ffff::1",
        ] {
            assert_ne!(rate_key(ip.parse().unwrap()), GLOBAL_KEY);
        }
    }
}
