# Outbound egress: what is guaranteed, and what is not

Mailwoman makes server-side outbound fetches — remote images through the
anonymizing proxy, webcal subscriptions, autoconfig lookups, key-server
lookups, webhooks, bridge OAuth. Every one of them is an SSRF surface, because
in most cases some part of the destination is influenced by a caller.

This page states what the egress policy guarantees, and — the more useful half —
what it does not.

Operator setup lives in [`../deploy/egress-proxy.md`](../deploy/egress-proxy.md).

---

## The policy

All gated fetches go through `mw-egress`, extracted in 26.20 from the image
proxy's implementation so that one policy backs every caller rather than each
caller growing its own.

* **Scheme allowlist** — `http`/`https` only.
* **DNS pinning.** The host is resolved once, the resulting address is checked,
  and the connection is made to **that address**. A name that resolves
  differently on a second lookup cannot move the target between the check and
  the connect (DNS rebinding).
* **Address refusal**, deny-by-default: loopback, private (RFC 1918), link-local,
  unique-local, and cloud metadata (`169.254.169.254`). IPv4-mapped IPv6 and the
  Teredo/6to4/ISATAP embeddings are decoded before the check, so
  `::ffff:127.0.0.1` and its relatives are refused as the addresses they encode.
* **Every redirect hop is re-validated** from step one. A permitted origin cannot
  redirect into a refused address.
* **Size and time caps**, and cookies/referrer stripped.
* **No ambient proxy.** Every `reqwest` client in production sets `.no_proxy()`,
  so an `HTTP_PROXY`/`ALL_PROXY` in the server's environment cannot silently
  become the destination — which would resolve the hostname itself and defeat the
  pin. This is enforced by a **structural test** that scans every crate source
  and fails with `file:line` for any client built without it, rather than by
  per-client assertions that cannot fail for the next client someone adds.

### Two policies, deliberately, because the input differs

`autoconfig` and the key-server lookups use the same code with **different
profiles**, and the difference is about who controls the input:

* **Key-server (VKS/WKD) fetches** carry an attacker-influenced key id and have
  no legitimate private-address case. Deny-private, always.
* **Autoconfig** resolves a domain taken from **the address the user typed**. On
  a self-hosted single-tenant deployment, `autoconfig.corp.example` on RFC 1918
  is legitimate; on a hosted multi-tenant one, the same path lets anyone who can
  type an address steer a server-side fetch. Same function, opposite risk,
  decided by deployment shape — so it is an **operator decision**, not a
  compile-time constant.

`MW_AUTOCONFIG_ALLOW_PRIVATE` is **off by default**: the unconfigured deployment
is the safe one. **Even with it on**, `169.254.0.0/16`, `fe80::/10` and the
Teredo/ISATAP decode paths stay refused — an "allow private for on-prem" switch
that also opened the metadata endpoint would convert a self-hoster convenience
into instance-credential theft.

### The proxy-mode JMAP upstream (26.20, t24 B2)

In proxy mode the upstream is chosen by the request **twice**. An anonymous
`POST /api/login` names the server, and that server's session document then names
the `apiUrl`, `downloadUrl` and `uploadUrl` that `/jmap/api`, `/jmap/download`,
`/jmap/upload` and the REST layer relay to, with the login's credential attached.
Until 26.20 `mw-jmap` built its own client, which checked no address and followed
redirects. A caller with a JMAP server of their own could therefore read back
internal responses through the download and API legs.

Every upstream request now goes through `mw_server::upstream_client`:

* **Same origin.** The target must share the origin of the session's server URL,
  so an upstream can only name URLs on itself. The session's three relayed URLs
  are checked at login, and each leg checks the URL it actually uses again,
  because a stored session outlives both a configuration change and the
  upstream's own session document.
* **Address policy.** When `MW_JMAP_UPSTREAMS` is unset, the strict policy above
  (`ip_allowed`) applies: public upstreams work and private, loopback and metadata
  addresses are refused before any connection is made. When it is set, only the
  listed origins are accepted, at any address. Naming an exact origin is the
  operator's decision, and it is the only way to use an internal JMAP server.
  Unlike `MW_AUTOCONFIG_ALLOW_PRIVATE`, it opens no *range*, so it keeps no
  metadata carve-out: a request cannot steer to an address the operator's listed
  name does not resolve to.
* **Pinned, no redirects, no ambient proxy.** The client is built with
  `harden_client` against the checked address. The session fetch follows up to
  `MAX_REDIRECTS` redirects **on the same origin only** (RFC 8620 §2.2 lets
  `/.well-known/jmap` redirect), checking and pinning each hop. The relayed legs
  follow none.
* **No verbatim content headers.** A proxied download is served as an attachment
  with a type reduced to `type/subtype`, and never as an HTML, XML or script type.

**Why this is not the egress route.** A configured route may only be selected by
deployment configuration (`route_construction_sites.rs`); this upstream is
request-derived, so the policy and pinning primitives apply and
`fetch_remote_routed` does not. The same reason keeps it out of
`t22_every_fetch_takes_the_route.rs`, which enforces the route. The invariant B2
needs is enforced separately by `t24_proxy_upstream.rs`: a JMAP client may be
built only inside `upstream_client` in any crate, and `mw-jmap` may not build an
HTTP client of its own.

**Not covered.** Engine mode's IMAP/POP3/SMTP dial (t23-e2-02) is a separate
surface and is not changed by this. A proxied response body is still read into
memory whole, with no size cap beyond the 300-second request limit.

---

## The upstream proxy, and the one thing it cannot guarantee

26.20 adds the machinery for an operator to route outbound fetches through their
own HTTP `CONNECT` or SOCKS5 proxy. **Check `../deploy/egress-proxy.md` for what
is actually wired** — at the time of writing the transport, config and admin
surface had landed but no fetch path consumed a configured route.

Introducing a middlebox into a path whose whole point is address policy is a
security regression by construction unless the guarantees are re-established
through it, so the design is:

* **Mailwoman resolves the name and enforces its own address policy**, exactly as
  it does for a direct fetch. The proxy is handed an **IP literal**, never a
  hostname. The SOCKS5 encoder has no domain branch at all — `ATYP` is only ever
  `0x01` (IPv4) or `0x04` (IPv6) — so "the proxy is never asked to resolve a
  name" is a property of a function with no code path to violate it, not a
  convention.
* **TLS terminates at the origin, not the proxy.** The tunnel carries the
  origin's TLS with the origin's SNI, verified against the origin's certificate.
  A proxy presenting its own valid certificate is refused with a *certificate*
  error.
* **`http` origins are refused by default** through a route (`allow_plaintext`
  opts in, per route). An `https` origin is what stops a proxy reading what it
  carries.
* **Fail-closed.** If the tunnel cannot be established the fetch fails. There is
  no silent fall back to a direct connection, because a fallback would mean the
  operator's routing requirement quietly stops applying at the moment the proxy
  is unavailable.

### The residual, in plain words

> **We can guarantee the proxy is never asked to resolve a name, and never sees
> plaintext for an `https` origin. We cannot guarantee it dials the address we
> asked for.**

Read "never sees plaintext" as being about **content**, not about the identity of
the destination. **A proxy learns the origin hostname either way**: it is in the
`CONNECT` authority for an HTTP proxy, and the TLS **SNI is cleartext** in the
ClientHello, so an `https` origin's name is visible to anything on the path. What
the tunnel protects is the request and the response.

The narrower claim — that the proxy is never *asked to resolve* a name — is the
one that matters and is the one that holds: resolution and the address policy stay
on our side, so the proxy cannot choose which address the name maps to. It can see
where we are going; it cannot decide it.

A malicious or compromised proxy can connect somewhere other than the address it
was given. For an `https` origin this is bounded by certificate verification: the
wrong destination cannot present the origin's certificate, so the fetch fails
rather than succeeding against the wrong host. For a plaintext origin — which is
why it is off by default — there is no such bound.

This is irreducible for this design. Do not read the address policy as protecting
you *from your own proxy*; it protects the proxy's operator and Mailwoman from
the destinations a caller can name.

---

## Remote images: what the grant actually gates

Remote images are off by default and load only when a grant covers them. The
grant model is per **account** and per **message context** — `single` (a message
id), `per-sender`, `per-domain` (the *sender's* domain), `all`.

**The grant is not per-URL, and the image host has no relationship to the sender
domain.** So a session holding a covering grant for a message can cause the
proxy to fetch any *public* URL it names under that message's id. The address
policy still applies — private, loopback and metadata addresses are refused —
and the proxy is authenticated and rate-limited per account.

What this means in practice: the grant stops a *sender* from tracking a reader
who has not opted in. It does not turn the proxy into a per-URL allowlist, and
it is not a defence against the account holder, who is the party granting.

---

## Reporting

Security contact and disclosure policy: [`../../SECURITY.md`](../../SECURITY.md).
