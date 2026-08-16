//! Deny-by-default outbound egress: the SSRF address policy, one-shot DNS
//! resolution with a **pinned** connect, and the hardened per-hop fetch.
//!
//! # Why this is a crate (t22-e6)
//! This code was written for, and lived inside, `mw-server`'s image proxy
//! (`crate::image_proxy`, t16 26.16 DQ3, hardened again in 26.17/26.18/26.19). It
//! was `pub(crate)`, which is precisely why the *other* egress surfaces in this
//! workspace — autoconfig discovery, the VKS/WKD key lookups — could not reuse it:
//! they cannot depend on `mw-server`. Lifting it here is a **move, not a rewrite**;
//! the behaviour is unchanged and the tests that proved it moved with it.
//!
//! This crate depends on nothing else in the workspace, so any crate may use it.
//!
//! # The policy
//!   * scheme ∈ {`http`,`https`} only; URLs carrying credentials are refused;
//!   * DNS is resolved **once, by us**, and the fetch is PINNED to the resolved IP
//!     (reqwest `resolve`) so a name cannot rebind to a new address between our
//!     check and the connect (anti-DNS-rebinding). The client also sets `no_proxy`
//!     — an ambient `HTTP_PROXY`/`HTTPS_PROXY`/`ALL_PROXY` would otherwise hand the
//!     HOSTNAME to a proxy that resolves it itself, bypassing the pin
//!     ([`harden_client`]);
//!   * every resolved address is checked against [`ip_allowed`] — loopback,
//!     private, link-local (incl. the `169.254.169.254` cloud-metadata address),
//!     CGNAT, unique-local/link-local IPv6, multicast, unspecified and reserved
//!     ranges are REFUSED; IPv4-mapped IPv6 is unwrapped and re-checked, and the
//!     transitional NAT64/6to4/Teredo/ISATAP embeddings are decoded and re-checked;
//!   * redirects are NOT auto-followed — each hop's `Location` is re-parsed and
//!     re-validated through the same gate (a redirect to a private target is
//!     refused);
//!   * hard caps bound response size and the per-request timeout;
//!   * the request originates here with a normalized `User-Agent` and no forwarded
//!     `Cookie`/`Referer`/`Authorization`.
//!
//! # What this crate does NOT do
//! It does not bound *how often* a caller fetches, it does not cache, and it does
//! not re-encode what it returns. Those belong to the caller (the image proxy keeps
//! its own concurrency limiter, per-account token bucket, content-hash cache and
//! wasm media jail). A new caller that fetches attacker-influenceable URLs is
//! responsible for its own rate bound.
//!
//! # The two entry points that carry the gate
//! [`fetch_remote_accepting`] (and [`fetch_url_hardened`], its string-error
//! wrapper) run the full loop: validate → resolve → pin → fetch → re-validate every
//! redirect. The lower-level pieces ([`Target`], [`fetch_hop`], [`harden_client`])
//! are public because a transport that tunnels this fetch through an operator
//! proxy has to re-enter the loop itself — **but constructing a [`Target`] by hand
//! bypasses [`validate_and_resolve`], so any caller that does so owns the
//! [`ip_allowed`] check.**

/// Operator-configured upstream egress proxies (t22-e11). The transport that
/// tunnels the fetch above through an HTTP `CONNECT` or SOCKS5 proxy **without ever
/// handing the proxy a hostname to resolve** — see the module docs for the threat
/// model, the residual it cannot close, and the deliberate origin/proxy asymmetry.
pub mod proxy;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use reqwest::header;

// ── fetch caps ───────────────────────────────────────────────────────────────

/// Max bytes accepted from an upstream response (post-transfer-decoding). A hard
/// backstop against decompression/size bombs, enforced while streaming; a caller
/// that decodes what it fetched caps decode separately.
pub const MAX_IMAGE_BYTES: usize = 8 * 1024 * 1024;
/// Per-request upstream timeout (applies to each redirect hop independently).
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(10);
/// Maximum redirect hops followed (each re-validated); more → refuse.
pub const MAX_REDIRECTS: usize = 4;
/// Normalized outbound User-Agent; the reader's real UA is never forwarded.
pub const PROXY_UA: &str = "Mailwoman-Image-Proxy";

// ── SSRF refusal taxonomy ──────────────────────────────────────────────────────

/// Why a fetch was refused. A caller that serves HTTP maps these to a client error
/// or bad-gateway — a request never reveals internal reachability beyond a coarse
/// status + reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// Malformed URL / non-http(s) scheme / credentials in URL / missing host.
    BadRequest(&'static str),
    /// The (only, or every resolved) target address is in a blocked range — the
    /// SSRF gate. Deliberately coarse: it does not distinguish "private" from
    /// "does not resolve".
    Blocked,
    /// Upstream too slow (per-hop timeout).
    Timeout,
    /// Upstream transport failure / non-success status / too many redirects.
    ///
    /// **Known gap, deliberately not closed in this commit (t22-e9, plan OQ-none).**
    /// This collapses a non-2xx *status* together with a transport failure, and one
    /// caller needs them apart: `mw-crypto`'s VKS/WKD lookup renders **404 as "no
    /// key published for that lookup"** and anything else as "the keyserver
    /// failed" — so routing it through this gate as-is turns "this person has no
    /// published key" into "something is broken", a regression in text a user
    /// reads, caused by a security improvement.
    ///
    /// The fix is a `Refusal::Status(u16)` variant, which is written and tested but
    /// **held**: adding a variant to a `pub enum` breaks the one exhaustive match on
    /// this type (`mw-server::image_proxy::refusal_response`), which belongs to
    /// another lane. Sequencing that break is a coordination decision, not a
    /// unilateral one — see `.orchestration/logs/t22-e11.md`.
    Upstream,
    /// Upstream body exceeded [`MAX_IMAGE_BYTES`].
    TooLarge,
}

// ── IP egress policy (DQ3) ─────────────────────────────────────────────────────

/// Whether an address is a permitted egress target: only globally-routable unicast.
/// Deny-by-default — anything loopback/private/link-local/ULA/multicast/reserved/
/// unspecified (incl. the cloud-metadata `169.254.169.254`) is refused. IPv4-mapped
/// IPv6 is unwrapped so `::ffff:127.0.0.1` cannot smuggle a loopback target.
///
/// Public so a second egress surface (the ManageSieve caller) can reuse this
/// classification — while applying its own, deliberately narrower policy.
pub fn ip_allowed(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => ipv4_allowed(v4),
        IpAddr::V6(v6) => {
            // Handle the v6-native specials FIRST: `::1`/`::` fall inside the
            // IPv4-compatible `::/96` block, so unwrapping them via `to_ipv4()`
            // before this check would route loopback/unspecified through the
            // permissive IPv4 path.
            if v6.is_loopback() || v6.is_unspecified() || v6.is_multicast() {
                return false;
            }
            // Unwrap IPv4-mapped (`::ffff:a.b.c.d`) and the deprecated IPv4-compatible
            // (`::a.b.c.d`) embeddings and re-check as IPv4, so e.g.
            // `::ffff:127.0.0.1` / `::7f00:1` cannot smuggle a loopback target.
            if let Some(v4) = v6.to_ipv4() {
                return ipv4_allowed(&v4);
            }
            ipv6_allowed(v6)
        }
    }
}

/// The IPv4 half of [`ip_allowed`].
pub fn ipv4_allowed(ip: &Ipv4Addr) -> bool {
    if ip.is_loopback()          // 127.0.0.0/8
        || ip.is_private()       // 10/8, 172.16/12, 192.168/16
        || ip.is_link_local()    // 169.254/16 (incl. 169.254.169.254 metadata)
        || ip.is_unspecified()   // 0.0.0.0
        || ip.is_broadcast()     // 255.255.255.255
        || ip.is_multicast()     // 224/4
        || ip.is_documentation()
    // 192.0.2/24, 198.51.100/24, 203.0.113/24
    {
        return false;
    }
    let o = ip.octets();
    // CGNAT 100.64.0.0/10.
    if o[0] == 100 && (o[1] & 0xc0) == 0x40 {
        return false;
    }
    // Benchmarking 198.18.0.0/15.
    if o[0] == 198 && (o[1] & 0xfe) == 18 {
        return false;
    }
    // Reserved 240.0.0.0/4 (excludes the already-rejected 255.255.255.255).
    if o[0] >= 240 {
        return false;
    }
    // IETF protocol assignments 192.0.0.0/24 (incl. 192.0.0.0/29 etc).
    if o[0] == 192 && o[1] == 0 && o[2] == 0 {
        return false;
    }
    true
}

/// The IPv6 half of [`ip_allowed`] (IPv4-mapped/compat forms are unwrapped by
/// [`ip_allowed`] before this is reached).
pub fn ipv6_allowed(ip: &Ipv6Addr) -> bool {
    if ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() {
        return false;
    }
    let seg = ip.segments();
    // Unique-local fc00::/7.
    if (seg[0] & 0xfe00) == 0xfc00 {
        return false;
    }
    // Link-local fe80::/10.
    if (seg[0] & 0xffc0) == 0xfe80 {
        return false;
    }
    // Documentation 2001:db8::/32.
    if seg[0] == 0x2001 && seg[1] == 0x0db8 {
        return false;
    }
    // NAT64 / 6to4 / Teredo / ISATAP carry one (or, for Teredo, two) routable IPv4
    // address(es) embedded in the v6 address; decode EACH and apply the IPv4 egress
    // policy, so a private/metadata v4 cannot be smuggled past the v6 gate as e.g.
    // `64:ff9b::7f00:1` (127.0.0.1) or a Teredo-mapped `169.254.169.254`. Refuse if
    // ANY embedded v4 is disallowed (fail-safe). (IPv4-mapped/compat `::ffff:a.b.c.d`
    // / `::a.b.c.d` are already unwrapped upstream by `ip_allowed` via
    // `Ipv6Addr::to_ipv4`.)
    for v4 in embedded_ipv4s(ip) {
        if !ipv4_allowed(&v4) {
            return false;
        }
    }
    true
}

/// Decode EVERY routable IPv4 address embedded in a transitional IPv6 address and
/// return them for a caller to re-apply the IPv4 egress policy to EACH. Covers:
///   * **NAT64** well-known prefix `64:ff9b::/96` — v4 in the last 32 bits;
///   * **6to4** `2002::/16` — v4 in bits 16..48;
///   * **Teredo** `2001:0000::/32` — the Teredo *server* v4 (bits 32..64, plain) AND
///     the mapped *client* v4 (bits 96..128, obfuscated by XOR with `0xffffffff`);
///   * **ISATAP** interface-ID `::0:5efe:a.b.c.d` / `::200:5efe:a.b.c.d` — v4 in the
///     last 32 bits (under any routing prefix; the link-local `fe80::5efe:*` form is
///     already refused by the `fe80::/10` check before this is reached).
///
/// The forms have distinct prefixes, so at most one matches (early return); Teredo
/// contributes two addresses. Returns an empty `Vec` for a non-transitional address.
///
/// **NAT64 network-specific prefixes (RFC 6052 NSP) are deliberately NOT decoded:**
/// the prefix length (/32…/96) and the v4 byte positions are site configuration, so
/// without the deployment's NSP an address cannot be known to be NAT64 or where its
/// embedded v4 sits — genuinely undecidable here. The well-known prefix is covered;
/// an NSP deployment supplies its own egress ACL. Public so the Sieve caller can
/// unwrap the same forms under its narrower policy.
pub fn embedded_ipv4s(ip: &Ipv6Addr) -> Vec<Ipv4Addr> {
    let seg = ip.segments();
    // NAT64 well-known prefix 64:ff9b::/96 — v4 is the last 32 bits.
    if seg[0] == 0x0064
        && seg[1] == 0xff9b
        && seg[2] == 0
        && seg[3] == 0
        && seg[4] == 0
        && seg[5] == 0
    {
        return vec![v4_from_segments(seg[6], seg[7])];
    }
    // 6to4 2002::/16 — v4 is bits 16..48 (segments 1 and 2).
    if seg[0] == 0x2002 {
        return vec![v4_from_segments(seg[1], seg[2])];
    }
    // Teredo 2001:0000::/32 — segs[2..4] = Teredo server v4 (plain); segs[6..8] =
    // mapped client v4, obfuscated by XOR with 0xffffffff. Re-check both.
    if seg[0] == 0x2001 && seg[1] == 0x0000 {
        return vec![
            v4_from_segments(seg[2], seg[3]),
            v4_from_segments(seg[6] ^ 0xffff, seg[7] ^ 0xffff),
        ];
    }
    // ISATAP interface identifier `…:{0000,0200}:5efe:a.b.c.d` — segs[4] ∈
    // {0x0000,0x0200}, segs[5] == 0x5efe, v4 = the last 32 bits. Any routing prefix.
    if seg[5] == 0x5efe && (seg[4] == 0x0000 || seg[4] == 0x0200) {
        return vec![v4_from_segments(seg[6], seg[7])];
    }
    Vec::new()
}

// ── the on-premises profile (t22-e11 for t22-e9) ───────────────────────────────

/// The **permissive** egress profile: private ranges are reachable, but loopback,
/// link-local (including the cloud-metadata address) and every transitional
/// embedding of those remain refused.
///
/// # Why this lives here rather than at each call site
/// A self-hosted deployment legitimately has `autoconfig.corp.internal` or its own
/// ManageSieve server on RFC1918, so those callers need an opt-in that
/// [`ip_allowed`] cannot express. The tempting shape is for each caller to author
/// its own predicate — and that is exactly the failure mode 26.18 spent a tag
/// closing. `mw-server::sieve_sync::sieve_egress_permitted` is already a hand-rolled
/// instance of this policy; an autoconfig copy would be the **third** place someone
/// has to remember when a new transitional embedding is added to [`embedded_ipv4s`].
///
/// Owning it in the crate that owns the decode makes the carve-out an **invariant**
/// rather than a promise repeated at every call site: `169.254.0.0/16`, `fe80::/10` and
/// the NAT64/6to4/Teredo/ISATAP decode paths stay denied **under the opt-in too**,
/// and they stay denied by construction because there is one implementation.
///
/// Callers pass the *name*: `validate_and_resolve_with(url, on_prem_allowed)`.
/// (Collapsing `sieve_egress_permitted` onto this is a 26.21 item, not t22's.)
pub fn on_prem_allowed(ip: &IpAddr) -> bool {
    // Fast path: anything the strict policy already permits is public unicast, which
    // is a subset of what this profile permits.
    if ip_allowed(ip) {
        return true;
    }
    // Otherwise the address is in some range the strict policy blocks (private,
    // loopback, link-local, CGNAT, …). Refuse ONLY loopback and link-local —
    // including `169.254.169.254` — and let the rest through.
    match ip {
        IpAddr::V4(v4) => on_prem_v4_allowed(v4),
        IpAddr::V6(v6) => {
            if v6.is_loopback() || v6.is_unspecified() || v6.is_multicast() {
                return false;
            }
            // Link-local fe80::/10 stays denied under the opt-in.
            if (v6.segments()[0] & 0xffc0) == 0xfe80 {
                return false;
            }
            // Unwrap IPv4-mapped/compat and apply the same narrow rule, so
            // `::ffff:169.254.169.254` cannot smuggle metadata past the opt-in.
            if let Some(v4) = v6.to_ipv4() {
                return on_prem_v4_allowed(&v4);
            }
            // Every transitional embedding is decoded by the SAME function the
            // strict policy uses, and ANY embedded loopback/link-local v4 refuses.
            // This is the clause that must not be re-authored per caller.
            for v4 in embedded_ipv4s(v6) {
                if !on_prem_v4_allowed(&v4) {
                    return false;
                }
            }
            true
        }
    }
}

/// The IPv4 rule shared by every arm of [`on_prem_allowed`], including the decoded
/// embeddings — one place, so the carve-out cannot drift between them.
fn on_prem_v4_allowed(v4: &Ipv4Addr) -> bool {
    !(v4.is_loopback() || v4.is_link_local() || v4.is_unspecified() || v4.is_multicast())
}

/// Reassemble an IPv4 address from the two 16-bit v6 segments that carry it.
fn v4_from_segments(hi: u16, lo: u16) -> Ipv4Addr {
    Ipv4Addr::new(
        (hi >> 8) as u8,
        (hi & 0xff) as u8,
        (lo >> 8) as u8,
        (lo & 0xff) as u8,
    )
}

// ── validate + resolve (the SSRF gate) ─────────────────────────────────────────

/// A validated, IP-pinned fetch target.
///
/// The fields are public so a tunnelling transport can re-enter the fetch loop with
/// a target it built itself. **That bypasses [`validate_and_resolve`]**: a
/// hand-built `Target` has passed no gate, and whoever builds one owns the
/// [`ip_allowed`] check on [`Target::addr`]. The gated way to obtain one is
/// [`validate_and_resolve`].
#[derive(Debug)]
pub struct Target {
    /// The full URL to fetch (host unchanged so TLS SNI + Host match).
    pub url: reqwest::Url,
    /// The hostname (for the reqwest `resolve` pin).
    pub host: String,
    /// The single resolved, allowed socket address the fetch is pinned to.
    pub addr: SocketAddr,
}

/// Parse + validate a URL and resolve it to ONE allowed, pinned address. Refuses a
/// non-http(s) scheme, a URL with embedded credentials, a missing host, and any
/// target that resolves only to blocked ranges. DNS is resolved here exactly once;
/// the returned [`Target::addr`] is what the fetch connects to (anti-rebinding).
pub async fn validate_and_resolve(url: reqwest::Url) -> Result<Target, Refusal> {
    validate_and_resolve_with(url, ip_allowed).await
}

/// [`validate_and_resolve`] under a caller-chosen address policy.
///
/// The scheme, credential, host and port checks are **identical and not
/// parameterised** — only the address predicate varies, so an opt-in cannot
/// accidentally widen anything but the address range. Pass [`ip_allowed`] for the
/// strict profile or [`on_prem_allowed`] for the deployment opt-in; authoring a
/// predicate inline is possible but is the thing [`on_prem_allowed`]'s doc comment
/// argues against.
///
/// The fail-safe direction is preserved verbatim: DNS is resolved exactly once here
/// and the returned [`Target::addr`] is what the fetch connects to.
pub async fn validate_and_resolve_with(
    url: reqwest::Url,
    policy: fn(&IpAddr) -> bool,
) -> Result<Target, Refusal> {
    match url.scheme() {
        "http" | "https" => {}
        _ => return Err(Refusal::BadRequest("only http/https URLs are proxied")),
    }
    // No credentials (DQ3).
    if !url.username().is_empty() || url.password().is_some() {
        return Err(Refusal::BadRequest("credentials in URL are not allowed"));
    }
    let host = url
        .host_str()
        .filter(|h| !h.is_empty())
        .ok_or(Refusal::BadRequest("URL has no host"))?
        .to_string();
    let port = url
        .port_or_known_default()
        .ok_or(Refusal::BadRequest("URL has no port"))?;

    // Resolve ONCE. `lookup_host` parses an IP literal directly (so a literal
    // loopback/metadata host is caught here too). Pin to the first allowed address;
    // if none is allowed, refuse (a rebinding answer of [public, private] never
    // reaches the private one because we pin to the allowed address).
    let resolved = tokio::net::lookup_host((host.as_str(), port))
        .await
        .map_err(|_| Refusal::Blocked)?;
    let addr = resolved
        .into_iter()
        .find(|a| policy(&a.ip()))
        .ok_or(Refusal::Blocked)?;

    Ok(Target { url, host, addr })
}

// ── the pinned single-hop fetch ────────────────────────────────────────────────

/// One hop's result: either a validated body, or a redirect to re-validate.
#[derive(Debug)]
pub enum Hop {
    /// The upstream body, already size-capped.
    Body(Vec<u8>),
    /// The raw `Location` header of a redirect, for the caller to re-validate.
    Redirect(String),
}

/// Apply the pinned-fetch hardening to a client builder: no auto-redirects, the
/// per-hop timeout, the `host → addr` pin, and **no proxy**.
///
/// `.no_proxy()` is what keeps the pin real. `reqwest::Client::builder()` reads
/// `HTTP_PROXY`/`HTTPS_PROXY`/`ALL_PROXY` from the process environment by default,
/// and a proxied request is sent to the proxy **by name** (a plain `GET
/// http://host/…` line, or `CONNECT host:443`) — so the proxy performs its own DNS
/// resolution and the `.resolve()` pin above is never consulted. A name that
/// answers with an allowed address at gate time could then be re-resolved by the
/// third party to an internal one, which is precisely the rebinding case the pin
/// exists to close. Refusing every proxy restores it.
///
/// The trade is deliberate and fail-safe: a deployment behind a mandatory egress
/// proxy loses remote images and webcal/ICS import — a visible failure — rather
/// than silently losing the pin. A configured, gate-aware egress proxy is a
/// separate design question and is not answered here.
///
/// Split out of [`fetch_hop`] so a test can hand in a builder that already carries
/// a proxy and assert the hardening still wins (`no_proxy` clears explicitly-set
/// proxies as well as the environment reader).
pub fn harden_client(
    base: reqwest::ClientBuilder,
    host: &str,
    addr: SocketAddr,
) -> reqwest::ClientBuilder {
    base.redirect(reqwest::redirect::Policy::none())
        .timeout(FETCH_TIMEOUT)
        .resolve(host, addr)
        .no_proxy()
}

/// Fetch ONE hop from a pinned target with redirects disabled + size/timeout caps.
/// The reqwest client pins `host → addr`, so even though the URL still names `host`
/// (for TLS/SNI/Host correctness) the connection goes only to the address we
/// validated. No cookie store; no forwarded headers. `accept` is the `Accept`
/// header (the image proxy asks for `image/*`; other reusers pass their own).
pub async fn fetch_hop(target: &Target, accept: &str) -> Result<Hop, Refusal> {
    fetch_hop_as(target, accept, PROXY_UA).await
}

/// [`fetch_hop`] with a caller-chosen `User-Agent`.
///
/// The UA is per call because it is not cosmetic: mail providers key off it on their
/// autodiscovery endpoints, so announcing `Mailwoman-Image-Proxy` to an autoconfig
/// endpoint is wrong in a way that produces support reports nobody can reproduce
/// (t22-e9). It is still a **normalized, caller-declared constant** — the reader's
/// real UA is never forwarded, and no request data reaches this value.
pub async fn fetch_hop_as(target: &Target, accept: &str, user_agent: &str) -> Result<Hop, Refusal> {
    let client = harden_client(reqwest::Client::builder(), &target.host, target.addr)
        .build()
        .map_err(|_| Refusal::Upstream)?;

    let resp = client
        .get(target.url.clone())
        .header(header::USER_AGENT, user_agent)
        .header(header::ACCEPT, accept)
        // Ask for no transfer compression — one less decompression-bomb surface.
        .header(header::ACCEPT_ENCODING, "identity")
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                Refusal::Timeout
            } else {
                Refusal::Upstream
            }
        })?;

    let status = resp.status();
    if status.is_redirection() {
        let loc = resp
            .headers()
            .get(header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(Refusal::Upstream)?
            .to_string();
        return Ok(Hop::Redirect(loc));
    }
    if !status.is_success() {
        return Err(Refusal::Upstream);
    }
    // Early size refusal from Content-Length when present.
    if let Some(len) = resp.content_length()
        && len as usize > MAX_IMAGE_BYTES
    {
        return Err(Refusal::TooLarge);
    }
    Ok(Hop::Body(read_capped(resp).await?))
}

/// Stream a response body, refusing once it exceeds [`MAX_IMAGE_BYTES`]. Uses
/// `chunk()` (no `stream` feature needed) so the cap applies to decoded bytes.
async fn read_capped(mut resp: reqwest::Response) -> Result<Vec<u8>, Refusal> {
    let mut buf = Vec::new();
    while let Some(chunk) = resp.chunk().await.map_err(|_| Refusal::Upstream)? {
        if buf.len() + chunk.len() > MAX_IMAGE_BYTES {
            return Err(Refusal::TooLarge);
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}

/// Fetch a remote image, following (and re-validating) redirects. Every hop —
/// including each redirect target — goes through [`validate_and_resolve`], so a
/// redirect to a private/metadata address is refused exactly like a direct one.
pub async fn fetch_remote(start: reqwest::Url) -> Result<Vec<u8>, Refusal> {
    fetch_remote_accepting(start, "image/*").await
}

/// [`fetch_remote`] with a caller-chosen `Accept` header. The SSRF gate
/// ([`validate_and_resolve`] per hop + redirect re-validation + the size/timeout
/// caps) is identical; only the advertised content preference differs.
pub async fn fetch_remote_accepting(start: reqwest::Url, accept: &str) -> Result<Vec<u8>, Refusal> {
    fetch_remote_with(start, accept, PROXY_UA, ip_allowed).await
}

/// [`fetch_remote_accepting`] under a caller-chosen `User-Agent` and address policy.
///
/// **The policy applies to every hop**, including each redirect target — an opt-in
/// does not become a one-hop exemption that a `Location` can escape, and a strict
/// caller cannot be redirected into a permissive resolution. Everything else (single
/// resolution, pinned connect, redirects disabled and re-validated, size and timeout
/// caps) is the same code as the strict path, because it *is* the strict path.
pub async fn fetch_remote_with(
    start: reqwest::Url,
    accept: &str,
    user_agent: &str,
    policy: fn(&IpAddr) -> bool,
) -> Result<Vec<u8>, Refusal> {
    let mut url = start;
    for _ in 0..=MAX_REDIRECTS {
        let target = validate_and_resolve_with(url.clone(), policy).await?;
        match fetch_hop_as(&target, accept, user_agent).await? {
            Hop::Body(bytes) => return Ok(bytes),
            Hop::Redirect(loc) => {
                // Resolve the Location against the current URL (handles relative
                // redirects) and loop — the new URL is re-validated next iteration.
                url = url.join(&loc).map_err(|_| Refusal::Upstream)?;
            }
        }
    }
    Err(Refusal::Upstream)
}

/// Reuse hook: fetch `url_str` through the exact same SSRF-hardened path the image
/// proxy uses — scheme/credential checks, DNS-pin, per-hop re-validation, and the
/// size/timeout caps — with a caller-chosen `Accept`. This exists so a second
/// attacker-influenceable fetch surface (a `webcal://` subscription URL) does NOT
/// hand-roll its own, weaker fetcher. There is no concurrency limiter here; a
/// caller bounds its own call rate.
pub async fn fetch_url_hardened(url_str: &str, accept: &str) -> Result<Vec<u8>, String> {
    fetch_url_hardened_with(url_str, accept, PROXY_UA, ip_allowed)
        .await
        .map_err(|r| match r {
            Refusal::BadRequest(m) => m.to_string(),
            Refusal::Blocked => "target address is not permitted".to_string(),
            Refusal::Timeout => "upstream timed out".to_string(),
            Refusal::Upstream => "upstream fetch failed".to_string(),
            Refusal::TooLarge => "upstream response too large".to_string(),
        })
}

/// [`fetch_url_hardened`] with a caller-chosen `User-Agent` and address policy,
/// returning the structured [`Refusal`] rather than a flattened string.
///
/// A caller that renders a *different* message per status — `mw-crypto`'s VKS/WKD
/// lookup turning `404` into "no key published for that lookup" — needs the
/// discriminant, not prose. The string-returning [`fetch_url_hardened`] stays for
/// callers that do not.
pub async fn fetch_url_hardened_with(
    url_str: &str,
    accept: &str,
    user_agent: &str,
    policy: fn(&IpAddr) -> bool,
) -> Result<Vec<u8>, Refusal> {
    let url = reqwest::Url::parse(url_str).map_err(|_| Refusal::BadRequest("malformed URL"))?;
    fetch_remote_with(url, accept, user_agent, policy).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::http::StatusCode;
    use axum::response::Response;
    use std::net::SocketAddr;

    // ── SSRF IP policy (the security core; pure, no network) ──────────────────

    #[test]
    fn blocks_loopback_private_linklocal_and_metadata() {
        for s in [
            "127.0.0.1",
            "127.5.6.7",
            "10.0.0.1",
            "172.16.0.1",
            "172.31.255.255",
            "192.168.1.1",
            "169.254.1.1",
            "169.254.169.254", // cloud metadata
            "0.0.0.0",
            "255.255.255.255",
            "224.0.0.1",  // multicast
            "100.64.0.1", // CGNAT
            "198.18.0.1", // benchmarking
            "240.0.0.1",  // reserved
            "192.0.2.1",  // documentation
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(!ip_allowed(&ip), "{s} must be blocked");
        }
    }

    #[test]
    fn blocks_ipv6_loopback_ula_linklocal_and_mapped() {
        for s in [
            "::1",                    // loopback
            "::",                     // unspecified
            "fc00::1",                // ULA
            "fd12:3456::1",           // ULA
            "fe80::1",                // link-local
            "ff02::1",                // multicast
            "::ffff:127.0.0.1",       // IPv4-mapped loopback (must unwrap + block)
            "::ffff:169.254.169.254", // IPv4-mapped metadata
            "2001:db8::1",            // documentation
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(!ip_allowed(&ip), "{s} must be blocked");
        }
    }

    #[test]
    fn blocks_nat64_and_6to4_embedded_private_ipv4() {
        // L3 (t17-e6): a private/metadata IPv4 smuggled inside a NAT64 (64:ff9b::/96)
        // or 6to4 (2002::/16) IPv6 address must be decoded and refused.
        for s in [
            "64:ff9b::7f00:1",    // NAT64 → 127.0.0.1 loopback
            "64:ff9b::a9fe:a9fe", // NAT64 → 169.254.169.254 metadata
            "64:ff9b::c0a8:101",  // NAT64 → 192.168.1.1 private
            "64:ff9b::a00:1",     // NAT64 → 10.0.0.1 private
            "2002:7f00:1::",      // 6to4  → 127.0.0.1 loopback
            "2002:a9fe:a9fe::",   // 6to4  → 169.254.169.254 metadata
            "2002:c0a8:101::",    // 6to4  → 192.168.1.1 private
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(
                !ip_allowed(&ip),
                "{s} must be blocked (embedded private v4)"
            );
        }
    }

    #[test]
    fn blocks_teredo_and_isatap_embedded_private_ipv4() {
        // R4 (t18): Teredo (2001:0000::/32) embeds a plain server v4 and an
        // XOR-obfuscated client v4; ISATAP embeds a v4 in the last 32 bits. A
        // private/metadata/loopback v4 in ANY embedded position must be refused.
        for s in [
            // Teredo, public server, client v4 = 127.0.0.1 (obfuscated 0x80fffffe).
            "2001:0:4136:e378:8000:ffff:80ff:fffe",
            // Teredo, public server, client v4 = 169.254.169.254 (0x56015601).
            "2001:0:4136:e378:8000:ffff:5601:5601",
            // Teredo, PRIVATE server v4 = 10.0.0.1, public client (8.8.8.8 → 0xf7f7f7f7).
            "2001:0:a00:1:8000:ffff:f7f7:f7f7",
            // ISATAP global-prefix IID wrapping 127.0.0.1 (seg[4]=0x0000).
            "2001:470::5efe:7f00:1",
            // ISATAP with the 0x0200 flag word wrapping 192.168.1.1.
            "2001:470:0:0:200:5efe:c0a8:101",
            // ISATAP wrapping the metadata address.
            "2001:470::5efe:a9fe:a9fe",
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(
                !ip_allowed(&ip),
                "{s} must be blocked (embedded private/metadata v4)"
            );
        }
    }

    #[test]
    fn allows_teredo_and_isatap_embedded_public_ipv4() {
        // Both embedded v4s public → allowed; the decode re-checks, it does not
        // blanket-refuse the transitional prefix.
        for s in [
            // Teredo: public server 65.54.227.120 + public client 8.8.8.8.
            "2001:0:4136:e378:8000:ffff:f7f7:f7f7",
            // ISATAP global-prefix IID wrapping 8.8.8.8.
            "2001:470::5efe:808:808",
            "2001:470:0:0:200:5efe:808:808",
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(
                ip_allowed(&ip),
                "{s} must be allowed (both embedded v4s public)"
            );
        }
    }

    #[test]
    fn allows_nat64_and_6to4_embedded_public_ipv4() {
        // A NAT64/6to4 address whose embedded IPv4 is public unicast stays allowed —
        // the decode re-checks the v4, it does not blanket-refuse the prefix.
        for s in [
            "64:ff9b::808:808", // NAT64 → 8.8.8.8
            "64:ff9b::101:101", // NAT64 → 1.1.1.1
            "2002:808:808::",   // 6to4  → 8.8.8.8
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(ip_allowed(&ip), "{s} must be allowed (embedded public v4)");
        }
    }

    #[test]
    fn allows_public_unicast() {
        for s in ["1.1.1.1", "8.8.8.8", "93.184.216.34", "2606:2800:220:1::1"] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(ip_allowed(&ip), "{s} must be allowed");
        }
    }

    // ── the on-premises opt-in profile (t22-e11 for t22-e9) ───────────────────

    #[test]
    fn on_prem_reaches_private_ranges_that_strict_refuses() {
        // The whole point of the opt-in: a self-hosted `autoconfig.corp.internal`
        // or ManageSieve server on RFC1918 must be reachable. This is also the
        // NEGATIVE CONTROL for the refusal tests below — without it, a profile that
        // simply refused everything would pass them all.
        for s in [
            "10.0.0.1",
            "172.16.0.1",
            "192.168.1.1",
            "100.64.0.1",   // CGNAT
            "198.18.0.1",   // benchmarking
            "fc00::1",      // ULA
            "fd12:3456::1", // ULA
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(!ip_allowed(&ip), "{s} must be refused by the STRICT policy");
            assert!(
                on_prem_allowed(&ip),
                "{s} must be reachable under the opt-in"
            );
        }
    }

    #[test]
    fn on_prem_still_refuses_loopback_and_link_local() {
        // The carve-out that must survive the opt-in.
        for s in [
            "127.0.0.1",
            "127.5.6.7",
            "0.0.0.0",
            "169.254.1.1",
            "169.254.169.254", // cloud metadata
            "224.0.0.1",       // multicast
            "::1",
            "::",
            "fe80::1", // link-local
            "ff02::1", // multicast
            "::ffff:127.0.0.1",
            "::ffff:169.254.169.254", // IPv4-mapped metadata
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(
                !on_prem_allowed(&ip),
                "{s} must stay refused even under the on-prem opt-in"
            );
        }
    }

    #[test]
    fn on_prem_refuses_metadata_smuggled_through_every_transitional_embedding() {
        // This is the clause that would rot if each caller authored its own profile:
        // the opt-in widens the PRIVATE ranges, and a loopback/metadata address
        // smuggled inside NAT64/6to4/Teredo/ISATAP must still be refused. It holds
        // here because `on_prem_allowed` decodes with the same `embedded_ipv4s` the
        // strict policy uses and applies one shared IPv4 rule to every arm.
        for s in [
            "64:ff9b::7f00:1",                      // NAT64  → 127.0.0.1
            "64:ff9b::a9fe:a9fe",                   // NAT64  → 169.254.169.254
            "2002:7f00:1::",                        // 6to4   → 127.0.0.1
            "2002:a9fe:a9fe::",                     // 6to4   → 169.254.169.254
            "2001:0:4136:e378:8000:ffff:80ff:fffe", // Teredo → client 127.0.0.1
            "2001:0:4136:e378:8000:ffff:5601:5601", // Teredo → client 169.254.169.254
            "2001:470::5efe:7f00:1",                // ISATAP → 127.0.0.1
            "2001:470::5efe:a9fe:a9fe",             // ISATAP → 169.254.169.254
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(
                !on_prem_allowed(&ip),
                "{s} smuggles loopback/metadata past the opt-in"
            );
        }
        // Negative control: the SAME embeddings carrying a PRIVATE (not
        // loopback/link-local) v4 are permitted under the opt-in — so the test above
        // is measuring the loopback/link-local rule, not a blanket refusal of the
        // transitional prefixes.
        for s in [
            "64:ff9b::a00:1",  // NAT64 → 10.0.0.1
            "2002:c0a8:101::", // 6to4  → 192.168.1.1
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(!ip_allowed(&ip), "{s} must be refused by the STRICT policy");
            assert!(
                on_prem_allowed(&ip),
                "{s} carries a private, non-loopback v4 and must be reachable"
            );
        }
    }

    #[test]
    fn on_prem_permits_everything_strict_permits() {
        // The opt-in is a strict SUPERSET, so it can never refuse something the
        // strict profile allows — otherwise opting in would silently break a
        // public fetch.
        for s in [
            "1.1.1.1",
            "8.8.8.8",
            "93.184.216.34",
            "2606:2800:220:1::1",
            "64:ff9b::808:808",
            "2001:470::5efe:808:808",
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(ip_allowed(&ip));
            assert!(
                on_prem_allowed(&ip),
                "{s} must remain allowed under the opt-in"
            );
        }
    }

    // ── the policy-parameterised gate ─────────────────────────────────────────

    #[tokio::test]
    async fn the_policy_parameter_changes_the_answer_for_a_private_literal() {
        // Same URL, two profiles, two answers — which is what proves the parameter
        // is actually consulted rather than decorative.
        let url = reqwest::Url::parse("http://10.0.0.5/mail/config-v1.1.xml").unwrap();
        assert_eq!(
            validate_and_resolve_with(url.clone(), ip_allowed)
                .await
                .unwrap_err(),
            Refusal::Blocked
        );
        let target = validate_and_resolve_with(url, on_prem_allowed)
            .await
            .expect("the on-prem profile must reach RFC1918");
        assert_eq!(target.addr.ip().to_string(), "10.0.0.5");
    }

    #[tokio::test]
    async fn the_policy_parameter_cannot_widen_anything_but_the_address_range() {
        // Scheme, credentials and host are checked in the shared body and are NOT
        // parameterised, so the permissive profile refuses them exactly as the
        // strict one does. An opt-in that also relaxed these would be a much bigger
        // grant than the one that was asked for.
        for u in [
            "file:///etc/passwd",
            "ftp://10.0.0.5/x",
            "gopher://10.0.0.5/1",
        ] {
            let url = reqwest::Url::parse(u).unwrap();
            let err = validate_and_resolve_with(url, on_prem_allowed)
                .await
                .unwrap_err();
            assert!(matches!(err, Refusal::BadRequest(_)), "{u} → {err:?}");
        }
        let creds = reqwest::Url::parse("http://user:pw@10.0.0.5/x").unwrap();
        let err = validate_and_resolve_with(creds, on_prem_allowed)
            .await
            .unwrap_err();
        assert!(matches!(err, Refusal::BadRequest(_)), "{err:?}");
        // And metadata stays refused under the opt-in, end to end through the gate.
        let meta = reqwest::Url::parse("http://169.254.169.254/latest/meta-data/").unwrap();
        assert_eq!(
            validate_and_resolve_with(meta, on_prem_allowed)
                .await
                .unwrap_err(),
            Refusal::Blocked
        );
    }

    // ── URL/scheme gate ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn refuses_non_http_schemes() {
        for u in ["file:///etc/passwd", "ftp://example.com/x", "gopher://x/1"] {
            let url = reqwest::Url::parse(u).unwrap();
            let err = validate_and_resolve(url).await.unwrap_err();
            assert!(matches!(err, Refusal::BadRequest(_)), "{u} → {err:?}");
        }
    }

    #[tokio::test]
    async fn refuses_credentials_in_url() {
        let url = reqwest::Url::parse("http://user:pw@example.com/x.png").unwrap();
        let err = validate_and_resolve(url).await.unwrap_err();
        assert!(matches!(err, Refusal::BadRequest(_)), "{err:?}");
    }

    #[tokio::test]
    async fn gate_blocks_literal_loopback_and_metadata_hosts() {
        // A literal private/metadata host is resolved by lookup_host to itself and
        // refused by the IP gate — no DNS needed. This is the end-to-end SSRF refusal.
        for u in [
            "http://127.0.0.1/x.png",
            "http://169.254.169.254/latest/meta-data/",
            "http://[::1]/x.png",
            "http://10.0.0.5/x.png",
        ] {
            let url = reqwest::Url::parse(u).unwrap();
            let err = validate_and_resolve(url).await.unwrap_err();
            assert_eq!(err, Refusal::Blocked, "{u} must be blocked");
        }
    }

    // ── fetch mechanics (size cap / timeout) against a LOCAL origin ────────────
    //
    // These call the low-level `fetch_hop` with a pinned loopback address DIRECTLY,
    // deliberately bypassing the SSRF gate (which — correctly — would refuse the
    // 127.0.0.1 test server). They exercise the streaming size cap + timeout, not
    // the gate (which the pure tests above cover).

    async fn spawn_origin(
        body: Vec<u8>,
        delay: Option<Duration>,
        status: StatusCode,
        location: Option<String>,
    ) -> SocketAddr {
        use axum::routing::get as aget;
        let handler = move || async move {
            if let Some(d) = delay {
                tokio::time::sleep(d).await;
            }
            let mut resp = Response::new(axum::body::Body::from(body));
            *resp.status_mut() = status;
            if let Some(loc) = location {
                resp.headers_mut()
                    .insert(axum::http::header::LOCATION, loc.parse().unwrap());
            }
            resp
        };
        let app: Router = Router::new().route("/img", aget(handler));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        addr
    }

    fn target_for(addr: SocketAddr) -> Target {
        Target {
            url: reqwest::Url::parse(&format!("http://{addr}/img")).unwrap(),
            host: addr.ip().to_string(),
            addr,
        }
    }

    #[tokio::test]
    async fn fetch_hop_enforces_size_cap() {
        let big = vec![0u8; MAX_IMAGE_BYTES + 1];
        let addr = spawn_origin(big, None, StatusCode::OK, None).await;
        let err = fetch_hop(&target_for(addr), "image/*").await.unwrap_err();
        assert_eq!(err, Refusal::TooLarge);
    }

    #[tokio::test]
    async fn fetch_hop_returns_small_body() {
        let addr = spawn_origin(b"hello".to_vec(), None, StatusCode::OK, None).await;
        match fetch_hop(&target_for(addr), "image/*").await.unwrap() {
            Hop::Body(b) => assert_eq!(b, b"hello"),
            Hop::Redirect(_) => panic!("unexpected redirect"),
        }
    }

    #[tokio::test]
    async fn fetch_hop_surfaces_redirect_location() {
        let addr = spawn_origin(
            Vec::new(),
            None,
            StatusCode::FOUND,
            Some("http://127.0.0.1/next".into()),
        )
        .await;
        match fetch_hop(&target_for(addr), "image/*").await.unwrap() {
            Hop::Redirect(loc) => assert_eq!(loc, "http://127.0.0.1/next"),
            Hop::Body(_) => panic!("expected redirect"),
        }
    }

    #[tokio::test]
    async fn the_user_agent_is_per_call_and_the_readers_own_is_never_forwarded() {
        use axum::routing::get as aget;
        use std::sync::Arc;
        use std::sync::Mutex;

        // An origin that reports back exactly what it received, so this asserts on
        // the header a server SAW rather than on the value we passed in.
        let seen: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&seen);
        let handler = move |headers: axum::http::HeaderMap| {
            let sink = Arc::clone(&sink);
            async move {
                let ua = headers
                    .get(axum::http::header::USER_AGENT)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("<none>")
                    .to_string();
                sink.lock().unwrap().push(ua);
                "ok"
            }
        };
        let app: Router = Router::new().route("/img", aget(handler));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        // The default path is unchanged: still the normalized image-proxy UA.
        fetch_hop(&target_for(addr), "image/*").await.unwrap();
        // A caller that needs its own identity gets it (mail providers key off the
        // UA on autodiscovery endpoints, so announcing an image proxy there is
        // wrong).
        fetch_hop_as(&target_for(addr), "application/xml", "mailwoman-autoconfig")
            .await
            .unwrap();

        let seen = seen.lock().unwrap().clone();
        assert_eq!(
            seen,
            vec![PROXY_UA.to_string(), "mailwoman-autoconfig".to_string()]
        );
    }

    #[tokio::test]
    async fn redirect_to_private_target_is_refused_by_the_loop() {
        // A public-looking start that 302s to a private host: the redirect is
        // re-validated and refused. We drive this at the loop level by joining +
        // re-validating the Location, matching `fetch_remote`'s hop check.
        let loc = "http://169.254.169.254/latest/";
        let joined = reqwest::Url::parse("http://cdn.example/x")
            .unwrap()
            .join(loc)
            .unwrap();
        let err = validate_and_resolve(joined).await.unwrap_err();
        assert_eq!(err, Refusal::Blocked);
    }

    // ── the connect pin survives proxy configuration ──────────────────────────

    #[tokio::test]
    async fn hardened_client_ignores_a_configured_proxy_and_honours_the_pin() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        // The pin's whole point is that the connection lands on the address WE
        // resolved. A proxy defeats it: the request is sent to the proxy carrying
        // the HOSTNAME, so the proxy resolves the name again and can be steered
        // somewhere we never validated. `harden_client` sets `no_proxy`, so no
        // proxy configuration — ambient or explicit — can take the request.
        //
        // The host is `pinned.invalid`: RFC 6761 reserves `.invalid` and it cannot
        // resolve in DNS anywhere. So this asserts more than "the fetch worked" —
        // only the pin can produce a connection to the origin at all.
        //
        // Driven through the BUILDER, not `HTTP_PROXY`: env vars are process-global
        // (and `set_var` is `unsafe` in this edition), so setting one here would
        // leak into every other client this binary builds, and the workspace's
        // `--test-threads=1` is the only thing that would stand between that and a
        // cross-test flake. `ClientBuilder::no_proxy` clears explicitly-set proxies
        // AND the environment reader (`auto_sys_proxy`), so an explicit proxy
        // exercises the same one call that closes the env-var path.
        let origin = spawn_origin(
            b"from-the-pinned-origin".to_vec(),
            None,
            StatusCode::OK,
            None,
        )
        .await;

        // A stand-in proxy that only counts connections and hangs up. If the
        // request goes here instead of to the pin, `hits` is non-zero.
        let observer = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let proxy_addr = observer.local_addr().unwrap();
        let hits = Arc::new(AtomicUsize::new(0));
        {
            let hits = Arc::clone(&hits);
            tokio::spawn(async move {
                while let Ok((stream, _)) = observer.accept().await {
                    hits.fetch_add(1, Ordering::SeqCst);
                    drop(stream);
                }
            });
        }

        let client = harden_client(
            reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(format!("http://{proxy_addr}")).unwrap()),
            "pinned.invalid",
            origin,
        )
        .build()
        .unwrap();

        let resp = client
            .get("http://pinned.invalid/img")
            .send()
            .await
            .expect("the pinned address must be contacted, not the proxy");
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.bytes().await.unwrap().as_ref(),
            b"from-the-pinned-origin"
        );
        assert_eq!(
            hits.load(Ordering::SeqCst),
            0,
            "the proxy must never receive the request — it would resolve the name itself"
        );
    }
}
