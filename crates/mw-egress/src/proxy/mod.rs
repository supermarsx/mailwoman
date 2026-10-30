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
//! influenced by request data: there is no per-request route selection, no proxy
//! field in any user-facing DTO, and no request header that can select a route
//! (plan §4.3, OQ-1). The only per-account input is the signed-in account's mail
//! domain, which picks among the operator's routes and cannot name a host
//! ([`crate::plan`], t32 e31b).
//!
//! # Route kinds (t32 e31b)
//! [`ProxyScheme`] has four kinds. `http-connect` and `socks5` are the design above.
//! `https-connect` is `http-connect` over TLS to the proxy, so the proxy credential
//! is not sent in the clear ([`stream::proxy_tls`]). `socks5h` is the one kind that
//! hands the proxy a name; it exists only behind a recorded admin acceptance and a
//! host-name denylist, and its rules are in [`remote_dns`].
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
pub mod remote_dns;
pub mod socks5;
pub mod stream;

#[cfg(test)]
mod tests;

use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::net::TcpStream;

use std::time::{Duration, Instant};

use crate::{
    FETCH_TIMEOUT, Hop, MAX_REDIRECTS, Outgoing, PROXY_UA, Refusal, Target, Timed, TimeoutPhase,
    ip_allowed,
};

/// A hop's outcome together with the media type its body declared.
type TypedHop = Result<(Hop, Option<String>), ProxyRefusal>;

// ── route configuration (deployment-wide, operator-supplied) ───────────────────

/// Which tunnelling protocol a route speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyScheme {
    /// HTTP `CONNECT` (RFC 9110 §9.3.6) — e.g. Squid.
    HttpConnect,
    /// HTTP `CONNECT` over TLS to the proxy, verified for the route's host against
    /// the same roots as an origin, so the `Proxy-Authorization` credential is
    /// never sent in the clear (t32 e31b).
    HttpsConnect,
    /// SOCKS5 (RFC 1928) — e.g. a Tor client's SOCKS port. The proxy is handed a
    /// literal address only.
    Socks5,
    /// SOCKS5 with **remote DNS**: the proxy is handed the origin name. Built only
    /// by `mw-server`'s one construction site, and only when an admin's acceptance
    /// of the risk is recorded on the route; see [`remote_dns`] for what still holds.
    Socks5h,
}

impl ProxyScheme {
    /// The scheme token used in audit rows and operator-facing text.
    pub fn as_str(self) -> &'static str {
        match self {
            ProxyScheme::HttpConnect => "http-connect",
            ProxyScheme::HttpsConnect => "https-connect",
            ProxyScheme::Socks5 => "socks5",
            // The `h` form means "the proxy resolves the name". It is a separate
            // kind with its own encoder (`remote_dns`), never what `socks5` does.
            ProxyScheme::Socks5h => "socks5h",
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
    /// The proxy answered and refused the tunnel: a `CONNECT` non-2xx other than
    /// `407`, or a SOCKS5 non-zero `REP`. **Authentication failure is
    /// [`ProxyRefusal::ProxyAuthRejected`], not this** — see there for why.
    ProxyRejected(String),
    /// The proxy rejected **our credentials** (HTTP `407 Proxy Authentication
    /// Required`, or an RFC 1929 sub-negotiation failure), or demanded
    /// authentication this route cannot supply.
    ///
    /// Split out of [`ProxyRefusal::ProxyRejected`] because "your proxy password is
    /// wrong" is the single most actionable thing an operator's *test this route*
    /// button can report, and it is the one failure they can fix themselves. Bundled
    /// with every other refusal it is recoverable only by parsing a message string —
    /// which means re-deriving, in `mw-server` and away from the protocol it
    /// describes, a distinction this code already had and threw away (t22-e12).
    ///
    /// Deliberately **not** included: SOCKS5 `REP=0x02` ("connection not allowed by
    /// ruleset") and HTTP `403`. Those are the proxy refusing the *destination*,
    /// which is authorization of the target rather than authentication of us — an
    /// operator who is told "check your password" when the real answer is "your
    /// proxy will not reach that host" has been sent to the wrong place.
    ///
    /// The payload is `&'static str`: it never carries the credential, and never
    /// carries third-party text from the proxy.
    ProxyAuthRejected(&'static str),
    /// TLS to the **origin** failed inside the tunnel. A MITM proxy lands here.
    OriginTls(String),
    /// TLS to the **proxy** failed (an `https-connect` route): its certificate is
    /// not valid for the route's host, or it does not speak TLS. Nothing was sent
    /// to the proxy, the credential included.
    ProxyTls(String),
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
            | ProxyRefusal::ProxyAuthRejected(_)
            | ProxyRefusal::OriginTls(_)
            | ProxyRefusal::ProxyTls(_)
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
            ProxyRefusal::ProxyAuthRejected(m) => {
                write!(f, "egress proxy rejected the route credentials: {m}")
            }
            ProxyRefusal::OriginTls(m) => write!(f, "origin TLS failed through the tunnel: {m}"),
            ProxyRefusal::ProxyTls(m) => write!(f, "TLS to the egress proxy failed: {m}"),
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
    /// The media type a body declared (`crate::declared_media_type`); `None` for a
    /// redirect, a refusal or a body without one.
    pub content_type: Option<String>,
    /// The hop's body or redirect, or why it was refused.
    pub outcome: Result<Hop, ProxyRefusal>,
}

/// A complete proxied fetch, redirects re-validated, with the same truthful
/// `traversed_proxy` signal (true if **any** hop traversed the proxy).
#[derive(Debug)]
pub struct ProxyFetch {
    /// Whether any hop's connection actually went through the proxy.
    pub traversed_proxy: bool,
    /// The media type the final body declared; `None` on a refusal or when absent.
    pub content_type: Option<String>,
    /// The fetched, size-capped body, or why the fetch was refused.
    pub outcome: Result<Vec<u8>, ProxyRefusal>,
}

// ── the transport ──────────────────────────────────────────────────────────────

/// Where the proxy is asked to connect: a literal address (every kind), or a
/// checked name (`socks5h` only).
enum TunnelDest<'a> {
    Addr(SocketAddr),
    Remote(&'a remote_dns::RemoteDest),
}

/// The TLS connector used for TLS to the proxy. Production uses
/// [`stream::connector`]; this crate's tests substitute one that trusts their
/// stand-in proxy's certificate.
struct Dialer {
    proxy_tls: tokio_rustls::TlsConnector,
}

impl Dialer {
    fn production() -> Dialer {
        Dialer {
            proxy_tls: stream::connector(),
        }
    }
}

/// Dial the route's proxy and negotiate a tunnel to `dst`.
///
/// This is transport, not policy: a [`TunnelDest::Addr`] has *already* been through
/// [`ip_allowed`] ([`tunnel_send_hop`] checks it first), and a
/// [`TunnelDest::Remote`] through [`remote_dns::remote_destination`]. The proxy
/// endpoint itself is deliberately **not** checked — see the module docs.
async fn open_tunnel(
    dialer: &Dialer,
    route: &ProxyRoute,
    dst: TunnelDest<'_>,
) -> Result<stream::ProxyConn, ProxyRefusal> {
    // The proxy's own hostname is resolved by us as well, so a route naming a host
    // gets one address rather than reconnect-per-attempt roulette.
    let proxy_addr = tokio::net::lookup_host((route.host.as_str(), route.port))
        .await
        .map_err(|_| ProxyRefusal::ProxyUnreachable("egress proxy host does not resolve"))?
        .next()
        .ok_or(ProxyRefusal::ProxyUnreachable(
            "egress proxy host resolved to no address",
        ))?;

    let tcp = TcpStream::connect(proxy_addr)
        .await
        .map_err(|_| ProxyRefusal::ProxyUnreachable("could not connect to the egress proxy"))?;
    // TLS to the proxy comes first, so `CONNECT` and its credential are never
    // written to a plain socket on an `https-connect` route.
    let mut conn = match route.scheme {
        ProxyScheme::HttpsConnect => {
            stream::proxy_tls(dialer.proxy_tls.clone(), tcp, &route.host).await?
        }
        _ => stream::ProxyConn::Tcp(tcp),
    };

    let creds = route.credentials();
    match (route.scheme, dst) {
        (ProxyScheme::HttpConnect | ProxyScheme::HttpsConnect, TunnelDest::Addr(a)) => {
            connect::handshake(&mut conn, a, creds).await?
        }
        (ProxyScheme::Socks5, TunnelDest::Addr(a)) => {
            socks5::handshake(&mut conn, a, creds).await?
        }
        (ProxyScheme::Socks5h, TunnelDest::Remote(d)) => {
            remote_dns::handshake(&mut conn, d, creds).await?
        }
        // A name for a kind that resolves locally, or a resolved address for the
        // remote-DNS kind: the dispatch below never builds either, and refusing is
        // the answer if it ever did.
        _ => {
            return Err(ProxyRefusal::RouteInvalid(
                "the destination does not match the route kind",
            ));
        }
    }
    Ok(conn)
}

/// Internal transport seam. Only the plan's guarded mail connector may call
/// this after selecting the scoped route and validating every resolved IP.
pub(crate) async fn open_pinned_stream(
    route: &ProxyRoute,
    destination: SocketAddr,
) -> Result<stream::ProxyConn, ProxyRefusal> {
    if route.scheme == ProxyScheme::Socks5h {
        return Err(ProxyRefusal::RouteInvalid(
            "remote DNS is unavailable for pinned mail transport",
        ));
    }
    open_tunnel(&Dialer::production(), route, TunnelDest::Addr(destination)).await
}

/// One tunnelled hop against an **already validated** target, with no address check
/// of its own. Private: the only way in from outside this module is
/// [`tunnel_fetch_hop`], which performs the check.
///
/// `traversed` is flipped the moment the proxy accepts the tunnel, so the caller
/// can report the truth even when the hop is later cancelled by the timeout.
///
/// Only this crate's transport tests call it; production goes through
/// [`tunnel_send_hop`].
#[cfg(test)]
async fn hop_over_tunnel(
    target: &Target,
    route: &ProxyRoute,
    out: &Outgoing<'_>,
    traversed: &Arc<AtomicBool>,
) -> Result<Hop, ProxyRefusal> {
    hop_with(
        &Dialer::production(),
        route,
        TunnelDest::Addr(target.addr),
        &target.url,
        &target.host,
        out,
        traversed,
        None,
    )
    .await
    .map(|(hop, _)| hop)
}

/// The hop body shared by every kind: the plaintext rule, the tunnel, origin TLS
/// against `origin_host` (the URL's host, never the proxy's and never the literal
/// handed to it), and the exchange.
#[allow(clippy::too_many_arguments)]
async fn hop_with(
    dialer: &Dialer,
    route: &ProxyRoute,
    dst: TunnelDest<'_>,
    url: &reqwest::Url,
    origin_host: &str,
    out: &Outgoing<'_>,
    traversed: &Arc<AtomicBool>,
    timed: Option<Timed<'_>>,
) -> TypedHop {
    if let Some(api) = out.authorized_api {
        api.validate_target(url).map_err(ProxyRefusal::Origin)?;
    }
    let is_https = url.scheme() == "https";
    if !is_https && !route.allow_plaintext {
        return Err(ProxyRefusal::PlaintextRefused);
    }

    // Under t56 bounds the connect clock covers dialling the proxy, its CONNECT or
    // SOCKS handshake and the origin TLS handshake inside the tunnel: everything
    // before the request is written, as reqwest's connect timeout does directly.
    let connect_deadline = timed.map(|t| (t, tokio::time::Instant::now() + t.limits.connect));
    let conn = match connect_deadline {
        None => open_tunnel(dialer, route, dst).await?,
        Some((t, at)) => match tokio::time::timeout_at(at, open_tunnel(dialer, route, dst)).await {
            Ok(conn) => conn?,
            Err(_) => return Err(ProxyRefusal::Origin(t.fired(TimeoutPhase::Connect))),
        },
    };
    // From here the bytes have gone through the proxy. Set this BEFORE the TLS
    // handshake, so a certificate failure still reports a traversed tunnel.
    traversed.store(true, Ordering::SeqCst);

    let wrapping = async {
        Ok::<_, ProxyRefusal>(if is_https {
            if let Some(dav) = out.authorized_dav {
                dav.validate().map_err(ProxyRefusal::Origin)?;
                stream::wrap_tls_with_roots(conn, origin_host, dav.additional_roots).await?
            } else if let Some(plugin) = out.authorized_plugin {
                plugin.validate().map_err(ProxyRefusal::Origin)?;
                stream::wrap_tls_with_roots(conn, origin_host, plugin.additional_roots).await?
            } else if let Some(download) = out.download.filter(|d| !d.additional_roots.is_empty()) {
                download.validate().map_err(ProxyRefusal::Origin)?;
                stream::wrap_tls_with_roots(conn, origin_host, download.additional_roots).await?
            } else {
                stream::wrap_tls(conn, origin_host).await?
            }
        } else {
            stream::TunnelStream::Plain(conn)
        })
    };
    let pipe = match connect_deadline {
        None => wrapping.await?,
        Some((t, at)) => match tokio::time::timeout_at(at, wrapping).await {
            Ok(pipe) => pipe?,
            Err(_) => return Err(ProxyRefusal::Origin(t.fired(TimeoutPhase::Connect))),
        },
    };
    http::exchange_typed(pipe, url, out, timed).await
}

/// The bound on one tunnelled hop: the legacy [`FETCH_TIMEOUT`], or under t56
/// bounds whatever is left of the fetch's total deadline.
fn hop_budget(timed: Option<(Timed<'_>, Instant)>) -> Duration {
    match timed {
        None => FETCH_TIMEOUT,
        Some((_, deadline)) => deadline
            .saturating_duration_since(Instant::now())
            .max(Duration::from_millis(1)),
    }
}

/// The refusal for an elapsed hop budget: the total deadline under t56 bounds.
fn budget_elapsed(timed: Option<(Timed<'_>, Instant)>) -> ProxyRefusal {
    ProxyRefusal::Origin(match timed {
        None => Refusal::Timeout,
        Some((t, _)) => t.fired(TimeoutPhase::Total),
    })
}

/// A [`ProxyHop`] from a typed outcome.
fn proxy_hop(traversed: &Arc<AtomicBool>, outcome: TypedHop) -> ProxyHop {
    match outcome {
        Ok((hop, content_type)) => ProxyHop {
            traversed_proxy: traversed.load(Ordering::SeqCst),
            content_type,
            outcome: Ok(hop),
        },
        Err(e) => ProxyHop {
            traversed_proxy: traversed.load(Ordering::SeqCst),
            content_type: None,
            outcome: Err(e),
        },
    }
}

/// One hop on a `socks5h` route: the destination is checked by
/// [`remote_dns::remote_destination`] (a literal through [`ip_allowed`], a name
/// through the denylist) instead of being resolved here.
async fn remote_dns_hop(
    dialer: &Dialer,
    url: &reqwest::Url,
    route: &ProxyRoute,
    out: &Outgoing<'_>,
    timed: Option<(Timed<'_>, Instant)>,
) -> ProxyHop {
    let (dest, host) = match remote_dns::remote_destination(url) {
        Ok(d) => d,
        Err(r) => {
            return ProxyHop {
                traversed_proxy: false,
                content_type: None,
                outcome: Err(ProxyRefusal::Origin(r)),
            };
        }
    };
    let traversed = Arc::new(AtomicBool::new(false));
    let outcome = match tokio::time::timeout(
        hop_budget(timed),
        hop_with(
            dialer,
            route,
            TunnelDest::Remote(&dest),
            url,
            &host,
            out,
            &traversed,
            timed.map(|(t, _)| t),
        ),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => Err(budget_elapsed(timed)),
    };
    proxy_hop(&traversed, outcome)
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
    tunnel_send_hop(
        target,
        route,
        &Outgoing {
            response_metadata: None,
            authorized_plugin: None,
            authorized_api: None,
            authorized_dav: None,
            web_push: None,
            accept,
            user_agent: PROXY_UA,
            body: None,
            response_limit: crate::MAX_IMAGE_BYTES,
            download: None,
        },
    )
    .await
}

/// [`tunnel_fetch_hop`] for any [`Outgoing`] (the caller's `User-Agent`, and a
/// `POST` body when there is one). The address and scheme checks below are the
/// same, unconditional, and still run before the proxy is dialled.
pub async fn tunnel_send_hop(target: &Target, route: &ProxyRoute, out: &Outgoing<'_>) -> ProxyHop {
    tunnel_send_hop_with(&Dialer::production(), target, route, out).await
}

/// [`tunnel_send_hop`] with a given [`Dialer`]; the checks are the same.
async fn tunnel_send_hop_with(
    dialer: &Dialer,
    target: &Target,
    route: &ProxyRoute,
    out: &Outgoing<'_>,
) -> ProxyHop {
    tunnel_send_hop_gated(dialer, target, route, out, &|_, _, ip| ip_allowed(ip), None).await
}

async fn tunnel_send_hop_gated(
    dialer: &Dialer,
    target: &Target,
    route: &ProxyRoute,
    out: &Outgoing<'_>,
    gate: &crate::AddressGate<'_>,
    timed: Option<(Timed<'_>, Instant)>,
) -> ProxyHop {
    if !gate(&target.host, target.addr.port(), &target.addr.ip()) {
        return ProxyHop {
            traversed_proxy: false,
            content_type: None,
            outcome: Err(ProxyRefusal::Origin(Refusal::Blocked)),
        };
    }
    // The scheme gate is `validate_and_resolve`'s, and a hand-built `Target` has not
    // been through it either. Re-applied here for the same reason as the address
    // check: this function's contract must not depend on how its argument was made.
    if !matches!(target.url.scheme(), "http" | "https") {
        return ProxyHop {
            traversed_proxy: false,
            content_type: None,
            outcome: Err(ProxyRefusal::Origin(Refusal::BadRequest(
                "only http/https URLs are proxied",
            ))),
        };
    }

    if route.scheme == ProxyScheme::Socks5h {
        // A remote-DNS route is driven by name ([`send_via_proxy`]); handing it a
        // locally resolved target would be a third behaviour nobody configured.
        return ProxyHop {
            traversed_proxy: false,
            content_type: None,
            outcome: Err(ProxyRefusal::RouteInvalid(
                "a remote-DNS route is used by name, not with a resolved target",
            )),
        };
    }

    let traversed = Arc::new(AtomicBool::new(false));
    let outcome = match tokio::time::timeout(
        hop_budget(timed),
        hop_with(
            dialer,
            route,
            TunnelDest::Addr(target.addr),
            &target.url,
            &target.host,
            out,
            &traversed,
            timed.map(|(t, _)| t),
        ),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => Err(budget_elapsed(timed)),
    };
    proxy_hop(&traversed, outcome)
}

/// Fetch `start` through the route's proxy, re-validating every redirect.
///
/// Each hop — the first and every `Location` — goes through
/// [`validate_and_resolve`] and then [`tunnel_fetch_hop`], so a redirect to a
/// private or metadata address is refused exactly like a direct request to one, and
/// a redirect to a different host tears the tunnel down and rebuilds it (which is
/// also why a proxy credential can never travel with a redirected request).
pub async fn fetch_via_proxy(start: reqwest::Url, route: &ProxyRoute, accept: &str) -> ProxyFetch {
    send_via_proxy(
        start,
        route,
        &Outgoing {
            response_metadata: None,
            authorized_plugin: None,
            authorized_api: None,
            authorized_dav: None,
            web_push: None,
            accept,
            user_agent: PROXY_UA,
            body: None,
            response_limit: crate::MAX_IMAGE_BYTES,
            download: None,
        },
    )
    .await
}

/// [`fetch_via_proxy`] for any [`Outgoing`].
///
/// A `GET` follows redirects exactly as [`fetch_via_proxy`] does. A `POST` follows
/// **none**: a redirect answer is [`Refusal::Upstream`], because re-sending a body
/// to a `Location` the origin chose is a decision this transport does not make for
/// its caller (and a `303` would silently turn the request into a `GET`).
pub async fn send_via_proxy(
    start: reqwest::Url,
    route: &ProxyRoute,
    out: &Outgoing<'_>,
) -> ProxyFetch {
    send_via_proxy_with(&Dialer::production(), start, route, out).await
}

/// [`send_via_proxy`] with a given [`Dialer`]. Each hop of a `socks5h` route is
/// checked by name ([`remote_dns_hop`]); every other kind resolves locally through
/// [`validate_and_resolve`] and the address check in [`tunnel_send_hop`].
async fn send_via_proxy_with(
    dialer: &Dialer,
    start: reqwest::Url,
    route: &ProxyRoute,
    out: &Outgoing<'_>,
) -> ProxyFetch {
    send_via_proxy_gated_with(dialer, start, route, out, &|_, _, ip| ip_allowed(ip), None).await
}

/// Internal policy-aware entrypoint: callers supply only the current plan's address gate.
pub(crate) async fn send_via_proxy_gated(
    start: reqwest::Url,
    route: &ProxyRoute,
    out: &Outgoing<'_>,
    gate: &crate::AddressGate<'_>,
) -> ProxyFetch {
    send_via_proxy_gated_with(&Dialer::production(), start, route, out, gate, None).await
}

/// [`send_via_proxy`] (or, with `gate`, [`send_via_proxy_gated`]) under t56
/// [`crate::FetchTimeouts`]: a connect clock over the tunnel and origin TLS, the
/// headers and idle clocks inside the exchange, the caller's redirect cap, and a
/// total deadline over every hop. Each hop is still resolved and gated exactly as
/// without bounds, and `traversed_proxy` stays a measurement: a hop cut off by the
/// deadline reports whether its tunnel had been accepted.
pub(crate) async fn send_via_proxy_timed(
    start: reqwest::Url,
    route: &ProxyRoute,
    out: &Outgoing<'_>,
    gate: Option<&crate::AddressGate<'_>>,
    timed: Timed<'_>,
) -> ProxyFetch {
    let strict = |_: &str, _: u16, ip: &std::net::IpAddr| ip_allowed(ip);
    send_via_proxy_gated_with(
        &Dialer::production(),
        start,
        route,
        out,
        gate.unwrap_or(&strict),
        Some(timed),
    )
    .await
}

async fn send_via_proxy_gated_with(
    dialer: &Dialer,
    start: reqwest::Url,
    route: &ProxyRoute,
    out: &Outgoing<'_>,
    gate: &crate::AddressGate<'_>,
    timed: Option<Timed<'_>>,
) -> ProxyFetch {
    let timed = timed.map(|t| (t, Instant::now() + t.limits.total));
    if out.authorized_plugin.is_some() && route.scheme == ProxyScheme::Socks5h {
        return ProxyFetch {
            traversed_proxy: false,
            content_type: None,
            outcome: Err(ProxyRefusal::Origin(Refusal::BadRequest(
                "plugin requests require local DNS verification",
            ))),
        };
    }
    if let Some(api) = out.authorized_api {
        let refused = api.validate_target(&start).err().or_else(|| {
            (route.scheme == ProxyScheme::Socks5h).then_some(Refusal::BadRequest(
                "package API requires local DNS verification",
            ))
        });
        if let Some(error) = refused {
            return ProxyFetch {
                traversed_proxy: false,
                content_type: None,
                outcome: Err(ProxyRefusal::Origin(error)),
            };
        }
    }
    let first = start.clone();
    let mut url = start;
    let mut traversed_proxy = false;
    let redirects = timed.map_or(MAX_REDIRECTS, |(t, _)| t.limits.max_redirects);
    for _ in 0..=redirects {
        let hop = if route.scheme == ProxyScheme::Socks5h {
            remote_dns_hop(dialer, &url, route, out, timed).await
        } else {
            let resolved = if out.authorized_plugin.is_some() {
                crate::resolve_gated_all(url.clone(), gate).await
            } else {
                crate::resolve_gated(url.clone(), gate).await
            };
            let target = match resolved {
                Ok(t) => t,
                Err(r) => {
                    return ProxyFetch {
                        traversed_proxy,
                        content_type: None,
                        outcome: Err(ProxyRefusal::Origin(r)),
                    };
                }
            };
            tunnel_send_hop_gated(dialer, &target, route, out, gate, timed).await
        };
        traversed_proxy |= hop.traversed_proxy;
        match hop.outcome {
            Ok(Hop::Body(bytes)) => {
                return ProxyFetch {
                    traversed_proxy,
                    content_type: hop.content_type,
                    outcome: Ok(bytes),
                };
            }
            Ok(Hop::Redirect(_))
                if out.body.is_some()
                    || out.authorized_dav.is_some()
                    || out.authorized_api.is_some() =>
            {
                return ProxyFetch {
                    traversed_proxy,
                    content_type: None,
                    outcome: Err(ProxyRefusal::Origin(Refusal::Upstream)),
                };
            }
            Ok(Hop::Redirect(location)) => match url.join(&location) {
                Ok(next) if !out.redirect_permitted(&first, &next) => {
                    return ProxyFetch {
                        traversed_proxy,
                        content_type: None,
                        outcome: Err(ProxyRefusal::Origin(Refusal::BadRequest(
                            "redirect to an unlisted host refused",
                        ))),
                    };
                }
                Ok(next) => url = next,
                Err(_) => {
                    return ProxyFetch {
                        traversed_proxy,
                        content_type: None,
                        outcome: Err(ProxyRefusal::Origin(Refusal::Upstream)),
                    };
                }
            },
            Err(e) => {
                return ProxyFetch {
                    traversed_proxy,
                    content_type: None,
                    outcome: Err(e),
                };
            }
        }
    }
    ProxyFetch {
        traversed_proxy,
        content_type: None,
        outcome: Err(ProxyRefusal::Origin(Refusal::Upstream)),
    }
}
