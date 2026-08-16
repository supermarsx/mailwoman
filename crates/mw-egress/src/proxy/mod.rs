//! Operator-configured **upstream egress proxy** transport (t22-e11, plan §4).
//!
//! # The bypass this exists to close
//! The crate's direct path guarantees: *we* resolve the name, *we* check every
//! resolved address against [`ip_allowed`], and *we* pin the connection to the one
//! address that passed. Rebinding afterwards changes nothing, because the socket
//! goes to an IP we already approved.
//!
//! Route that fetch through an HTTP `CONNECT` or SOCKS5 proxy **in the ordinary
//! way** and the proxy resolves the name. `CONNECT evil.example:443` and SOCKS5
//! `ATYP=0x03` both hand a *name* to a third party; the pin is not weakened, it is
//! **unused**. 26.19 demonstrated this rather than assuming it: with `HTTP_PROXY`
//! set, a client pinned to `pinned.invalid` — a name that cannot resolve in DNS
//! anywhere — still reached the proxy carrying that hostname, which means the third
//! party had to resolve it.
//!
//! # The design: resolve locally; never let the proxy resolve
//! 1. The origin hostname is resolved by [`validate_and_resolve`], which applies
//!    [`ip_allowed`] to **every** returned address and refuses if any is blocked.
//!    One approved [`SocketAddr`] survives.
//! 2. We dial the **proxy** and hand it that literal address:
//!    * HTTP `CONNECT` with an **IP-literal authority** ([`connect::authority`]).
//!      There is no name to resolve. A proxy that rejects IP-literal authorities is
//!      unusable, and that is an accepted, documented limitation.
//!    * SOCKS5 with `ATYP` ∈ {`0x01`, `0x04`} **only**. `ATYP=0x03` is not
//!      constructible: [`socks5::encode_connect_request`] takes a [`SocketAddr`],
//!      which cannot hold a name, and matches its two variants exhaustively.
//! 3. TLS is ours end to end **through** the tunnel ([`stream::wrap_tls`]):
//!    `ServerName::DnsName(origin_host)`, the same roots and the same provider as a
//!    direct fetch. The proxy sees ciphertext; a MITM proxy is a certificate
//!    failure, not a silent substitution.
//! 4. HTTP over the tunnel is ours ([`http::exchange`]): same headers, same
//!    `Accept-Encoding: identity`, same streaming size cap, redirects **disabled**
//!    so each `Location` re-enters step 1 from the top, re-resolving and
//!    re-validating the new hostname.
//!
//! # The residual, stated plainly
//! A deployment that configures an upstream proxy is **trusting that proxy with its
//! egress**. We can guarantee the proxy is never asked to resolve a name and never
//! sees plaintext for an `https` origin. We **cannot** guarantee it dials the
//! address we asked for: we ask for `203.0.113.7:443` and a hostile proxy may open a
//! socket to `169.254.169.254` instead and pipe those bytes back, and nothing
//! observable from our side distinguishes the two. For `https` origins the residual
//! is near-zero because a substituted peer cannot produce a valid certificate for
//! the origin's name — which is exactly why plaintext `http` through a proxy is
//! refused unless a route opts in ([`ProxyRoute::allow_plaintext`], plan OQ-3).
//!
//! # The deliberate asymmetry
//! The origin address must pass [`ip_allowed`]. The **proxy endpoint must not** —
//! "run a Squid or a Tor SOCKS5 on localhost" is the normal operator deployment, so
//! a loopback or RFC1918 proxy endpoint is permitted. That is safe **only** because
//! the endpoint comes from deployment-wide operator configuration and can never be
//! influenced by request data: there is no per-account and no per-request route
//! selection, no proxy field in any user-facing DTO, and no request header that can
//! select a route (plan §4.3, OQ-1).
//!
//! # Fail-closed
//! If a route is configured and the tunnel cannot be established, the fetch
//! **fails**. Nothing here falls back to a direct connection — an operator
//! configures an egress proxy for network policy, egress-IP control or reader
//! anonymity, and a silent direct fallback defeats all three at exactly the moment
//! the operator is least likely to notice (plan §4.4). Route selection and the
//! audit row belong to the caller; [`ProxyHop::traversed_proxy`] is set from this
//! transport's own progress so that row can be true.
//!
//! [`ip_allowed`]: crate::ip_allowed
//! [`validate_and_resolve`]: crate::validate_and_resolve

pub mod connect;
pub mod http;
pub mod socks5;
pub mod stream;

#[cfg(test)]
mod tests;

use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::net::TcpStream;

use crate::{FETCH_TIMEOUT, Hop, MAX_REDIRECTS, Refusal, Target, ip_allowed, validate_and_resolve};

// ── route configuration (deployment-wide, operator-supplied) ───────────────────

/// Which tunnelling protocol a route speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyScheme {
    /// HTTP `CONNECT` (RFC 9110 §9.3.6) — e.g. Squid.
    HttpConnect,
    /// SOCKS5 (RFC 1928) — e.g. a Tor client's SOCKS port.
    Socks5,
}

impl ProxyScheme {
    /// The scheme token used in audit rows and operator-facing text.
    pub fn as_str(self) -> &'static str {
        match self {
            ProxyScheme::HttpConnect => "http-connect",
            // Deliberately NOT `socks5h`: the `h` form means "the proxy resolves
            // the name", which is the bypass. It is not merely unconfigured here,
            // it is unimplementable — nothing in `socks5` can send a name.
            ProxyScheme::Socks5 => "socks5",
        }
    }
}

/// Credentials presented **to the proxy**, never to an origin.
///
/// The [`fmt::Debug`] implementation is hand-written to redact the password: a
/// derived one would put the plaintext into every `tracing` event, panic message
/// and error body that formats a [`ProxyRoute`].
#[derive(Clone, PartialEq, Eq)]
pub struct ProxyAuth {
    /// Proxy username, stored and logged in the clear.
    pub username: String,
    /// Proxy password. Sealed at rest by the caller; never logged here.
    pub password: String,
}

impl fmt::Debug for ProxyAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProxyAuth")
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .finish()
    }
}

/// One deployment-wide egress route.
///
/// `host` may be a name or a literal and is **not** subject to [`ip_allowed`] — see
/// the module docs' asymmetry note. It is operator configuration; if it ever became
/// request-derived this type would be an SSRF primitive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyRoute {
    /// Stable identifier for audit rows and cache keys.
    pub id: String,
    /// Tunnelling protocol.
    pub scheme: ProxyScheme,
    /// Proxy hostname or IP literal (operator configuration).
    pub host: String,
    /// Proxy port.
    pub port: u16,
    /// Optional proxy credentials.
    pub auth: Option<ProxyAuth>,
    /// Permit plaintext `http` origins through this route. **Off by default**: for
    /// an `http` origin the tunnel carries cleartext the proxy can read and rewrite,
    /// and the certificate check that collapses the malicious-proxy residual does
    /// not exist (plan OQ-3).
    pub allow_plaintext: bool,
}

impl ProxyRoute {
    /// `scheme://host:port` — the only form that may be logged or audited. Carries
    /// no credential and no origin path.
    pub fn endpoint(&self) -> String {
        format!("{}://{}:{}", self.scheme.as_str(), self.host, self.port)
    }

    /// The credential pair, if the route has one.
    fn credentials(&self) -> Option<(&str, &str)> {
        self.auth
            .as_ref()
            .map(|a| (a.username.as_str(), a.password.as_str()))
    }
}

// ── refusal taxonomy ───────────────────────────────────────────────────────────

/// Why a proxied fetch was refused.
///
/// Finer-grained than [`Refusal`] because an operator debugging a route needs to
/// tell "my proxy is down" from "the origin's certificate is wrong" from "the
/// address policy refused the target" — three failures that a single `Upstream`
/// discriminant would flatten into one unactionable message. [`Refusal`] is still
/// what a caller serving HTTP maps to a status code (see the [`From`] impl).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyRefusal {
    /// The origin-address policy refused, exactly as it would on the direct path.
    Origin(Refusal),
    /// The route itself is not usable as configured.
    RouteInvalid(&'static str),
    /// The proxy could not be reached, or died mid-negotiation. **The fetch did not
    /// traverse the proxy.**
    ProxyUnreachable(&'static str),
    /// The proxy answered and refused the tunnel (`CONNECT` non-2xx, SOCKS5
    /// non-zero `REP`, auth failure).
    ProxyRejected(String),
    /// TLS to the **origin** failed inside the tunnel. A MITM proxy lands here.
    OriginTls(String),
    /// The HTTP exchange inside the tunnel failed.
    OriginHttp(String),
    /// A plaintext `http` origin was requested through a route without
    /// [`ProxyRoute::allow_plaintext`].
    PlaintextRefused,
}

impl From<ProxyRefusal> for Refusal {
    fn from(p: ProxyRefusal) -> Refusal {
        match p {
            ProxyRefusal::Origin(r) => r,
            // A misconfigured route or a refused plaintext origin is a request the
            // caller should not have made; everything else is an upstream failure.
            ProxyRefusal::RouteInvalid(_) => Refusal::BadRequest("egress route is not usable"),
            ProxyRefusal::PlaintextRefused => {
                Refusal::BadRequest("plaintext http is not permitted through this egress route")
            }
            ProxyRefusal::ProxyUnreachable(_)
            | ProxyRefusal::ProxyRejected(_)
            | ProxyRefusal::OriginTls(_)
            | ProxyRefusal::OriginHttp(_) => Refusal::Upstream,
        }
    }
}

impl fmt::Display for ProxyRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyRefusal::Origin(r) => write!(f, "origin policy refused: {r:?}"),
            ProxyRefusal::RouteInvalid(m) => write!(f, "egress route invalid: {m}"),
            ProxyRefusal::ProxyUnreachable(m) => write!(f, "egress proxy unreachable: {m}"),
            ProxyRefusal::ProxyRejected(m) => write!(f, "egress proxy refused the tunnel: {m}"),
            ProxyRefusal::OriginTls(m) => write!(f, "origin TLS failed through the tunnel: {m}"),
            ProxyRefusal::OriginHttp(m) => write!(f, "origin HTTP failed through the tunnel: {m}"),
            ProxyRefusal::PlaintextRefused => {
                write!(
                    f,
                    "plaintext http origins are not permitted through this route"
                )
            }
        }
    }
}

// ── outcomes that carry what actually happened ─────────────────────────────────

/// One tunnelled hop's result, plus whether the connection **actually** traversed
/// the proxy.
///
/// `traversed_proxy` is set by the transport from its own progress — it is `true`
/// only once the proxy has accepted the tunnel — and never from the configuration
/// that was *intended*. A caller writing an audit row must take it from here; a row
/// that says `proxied: true` because a route was configured is the failure shape
/// 26.19 shipped and 26.20 is fixing (plan §4.3).
#[derive(Debug)]
pub struct ProxyHop {
    /// Whether the proxy accepted the tunnel and bytes flowed through it.
    pub traversed_proxy: bool,
    /// The hop's body or redirect, or why it was refused.
    pub outcome: Result<Hop, ProxyRefusal>,
}

/// A complete proxied fetch, redirects re-validated, with the same truthful
/// `traversed_proxy` signal (true if **any** hop traversed the proxy).
#[derive(Debug)]
pub struct ProxyFetch {
    /// Whether any hop's connection actually went through the proxy.
    pub traversed_proxy: bool,
    /// The fetched, size-capped body, or why the fetch was refused.
    pub outcome: Result<Vec<u8>, ProxyRefusal>,
}

// ── the transport ──────────────────────────────────────────────────────────────

/// Dial the route's proxy and negotiate a tunnel to the literal address `dst`.
///
/// This is transport, not policy: `dst` has *already* been through
/// [`ip_allowed`] by the caller ([`tunnel_fetch_hop`] is the only in-crate caller
/// and checks it first). The proxy endpoint itself is deliberately **not** checked
/// — see the module docs.
async fn open_tunnel(route: &ProxyRoute, dst: SocketAddr) -> Result<TcpStream, ProxyRefusal> {
    // The proxy's own hostname is resolved by us as well, so a route naming a host
    // gets one address rather than reconnect-per-attempt roulette.
    let proxy_addr = tokio::net::lookup_host((route.host.as_str(), route.port))
        .await
        .map_err(|_| ProxyRefusal::ProxyUnreachable("egress proxy host does not resolve"))?
        .next()
        .ok_or(ProxyRefusal::ProxyUnreachable(
            "egress proxy host resolved to no address",
        ))?;

    let mut stream = TcpStream::connect(proxy_addr)
        .await
        .map_err(|_| ProxyRefusal::ProxyUnreachable("could not connect to the egress proxy"))?;

    match route.scheme {
        ProxyScheme::HttpConnect => {
            connect::handshake(&mut stream, dst, route.credentials()).await?
        }
        ProxyScheme::Socks5 => socks5::handshake(&mut stream, dst, route.credentials()).await?,
    }
    Ok(stream)
}

/// One tunnelled hop against an **already validated** target, with no address check
/// of its own. Private: the only way in from outside this module is
/// [`tunnel_fetch_hop`], which performs the check.
///
/// `traversed` is flipped the moment the proxy accepts the tunnel, so the caller
/// can report the truth even when the hop is later cancelled by the timeout.
async fn hop_over_tunnel(
    target: &Target,
    route: &ProxyRoute,
    accept: &str,
    traversed: &Arc<AtomicBool>,
) -> Result<Hop, ProxyRefusal> {
    let is_https = target.url.scheme() == "https";
    if !is_https && !route.allow_plaintext {
        return Err(ProxyRefusal::PlaintextRefused);
    }

    let tcp = open_tunnel(route, target.addr).await?;
    // From here the bytes have gone through the proxy. Set this BEFORE the TLS
    // handshake, so a certificate failure still reports a traversed tunnel.
    traversed.store(true, Ordering::SeqCst);

    let pipe = if is_https {
        stream::wrap_tls(tcp, &target.host).await?
    } else {
        stream::TunnelStream::Plain(tcp)
    };
    http::exchange(pipe, &target.url, accept).await
}

/// Fetch **one hop** through the route's proxy, refusing unless the target's
/// address passes [`ip_allowed`].
///
/// # The explicit check this function exists to carry
/// [`Target`]'s fields are `pub` so a tunnelling transport can build one, and its
/// doc comment states that **whoever constructs one owns the [`ip_allowed`] check
/// on [`Target::addr`]** — a hand-built `Target` has passed no gate. This function
/// is where that ownership is discharged for the proxy path: the check below is
/// unconditional, is not behind a flag or a feature, and runs **before the proxy is
/// dialled**, so a blocked target never causes a socket to be opened at all.
///
/// It is deliberately stricter than the direct path's [`crate::fetch_hop`], which
/// applies no address policy of its own. Repeating the check after
/// [`validate_and_resolve`] has already run costs one match on an enum and closes
/// the case where a future caller assembles a `Target` some other way.
pub async fn tunnel_fetch_hop(target: &Target, route: &ProxyRoute, accept: &str) -> ProxyHop {
    // ── the ip_allowed check on a possibly hand-built Target ──────────────────
    if !ip_allowed(&target.addr.ip()) {
        return ProxyHop {
            traversed_proxy: false,
            outcome: Err(ProxyRefusal::Origin(Refusal::Blocked)),
        };
    }
    // The scheme gate is `validate_and_resolve`'s, and a hand-built `Target` has not
    // been through it either. Re-applied here for the same reason as the address
    // check: this function's contract must not depend on how its argument was made.
    if !matches!(target.url.scheme(), "http" | "https") {
        return ProxyHop {
            traversed_proxy: false,
            outcome: Err(ProxyRefusal::Origin(Refusal::BadRequest(
                "only http/https URLs are proxied",
            ))),
        };
    }

    let traversed = Arc::new(AtomicBool::new(false));
    let outcome = match tokio::time::timeout(
        FETCH_TIMEOUT,
        hop_over_tunnel(target, route, accept, &traversed),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => Err(ProxyRefusal::Origin(Refusal::Timeout)),
    };
    ProxyHop {
        traversed_proxy: traversed.load(Ordering::SeqCst),
        outcome,
    }
}

/// Fetch `start` through the route's proxy, re-validating every redirect.
///
/// Each hop — the first and every `Location` — goes through
/// [`validate_and_resolve`] and then [`tunnel_fetch_hop`], so a redirect to a
/// private or metadata address is refused exactly like a direct request to one, and
/// a redirect to a different host tears the tunnel down and rebuilds it (which is
/// also why a proxy credential can never travel with a redirected request).
pub async fn fetch_via_proxy(start: reqwest::Url, route: &ProxyRoute, accept: &str) -> ProxyFetch {
    let mut url = start;
    let mut traversed_proxy = false;
    for _ in 0..=MAX_REDIRECTS {
        let target = match validate_and_resolve(url.clone()).await {
            Ok(t) => t,
            Err(r) => {
                return ProxyFetch {
                    traversed_proxy,
                    outcome: Err(ProxyRefusal::Origin(r)),
                };
            }
        };
        let hop = tunnel_fetch_hop(&target, route, accept).await;
        traversed_proxy |= hop.traversed_proxy;
        match hop.outcome {
            Ok(Hop::Body(bytes)) => {
                return ProxyFetch {
                    traversed_proxy,
                    outcome: Ok(bytes),
                };
            }
            Ok(Hop::Redirect(location)) => match url.join(&location) {
                Ok(next) => url = next,
                Err(_) => {
                    return ProxyFetch {
                        traversed_proxy,
                        outcome: Err(ProxyRefusal::Origin(Refusal::Upstream)),
                    };
                }
            },
            Err(e) => {
                return ProxyFetch {
                    traversed_proxy,
                    outcome: Err(e),
                };
            }
        }
    }
    ProxyFetch {
        traversed_proxy,
        outcome: Err(ProxyRefusal::Origin(Refusal::Upstream)),
    }
}
