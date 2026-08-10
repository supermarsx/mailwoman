# Running Mailwoman behind a reverse proxy

This is the operator-facing document: **what is supported, what is merely
shipped, and what changes when you upgrade to 26.19.** The companion file
[`proxy/README.md`](proxy/README.md) describes the test harness and the design
of each configuration; this one tells you what to set and what has been proven.

`docs/deploy/proxy/README.md` has referred to this page since it was written.
It did not exist until 26.19.

---

## 1. Read this first if you are upgrading — one breaking change

**Mailwoman no longer trusts `X-Forwarded-For` from whoever connects.**

Before 26.19, any peer that could reach the application port could set
`X-Forwarded-For` and have it believed. That made the OAuth scoped-key IP
allowlist defeatable with a header, the per-key rate limit evadable by rotating
one, and every address in the audit log attacker-chosen. Forwarded headers are
now ignored by default.

**The consequence: a deployment that relied on the old implicit trust now sees
the proxy's own address as the client address** — in audit logs, in per-key IP
allowlists, in rate-limit buckets and in the login monitor's ban list. Nothing
fails loudly; the addresses are simply wrong until you configure it.

To restore correct client addresses you must set **both** of:

```sh
MW_TRUSTED_PROXIES=10.0.0.0/24      # the addresses your proxy connects FROM
MW_FORWARDED_MODE=xff               # which header it emits: xff | forwarded
```

Both are required, and each is inert without the other. Setting proxies without
choosing a mode changes nothing; choosing a mode without listing proxies changes
nothing. That is deliberate — neither half can start trusting a header on its
own.

`MW_TRUSTED_PROXIES` should list the addresses your proxy **connects from**,
which is not always the address it listens on. In Docker or Kubernetes this is
the pod/container address, not the service address. Get it from the app's own
logs on a real request rather than from the compose file.

---

## 2. Environment variables introduced or changed in 26.19

| Variable | Default | What it does |
|---|---|---|
| `MW_TRUSTED_PROXIES` | *(empty — trust nobody)* | Comma-separated CIDRs or bare addresses whose peers may assert a forwarded header, and which are skipped as intermediate hops when walking one. |
| `MW_FORWARDED_MODE` | `off` | Which forwarded header to read: `off`, `xff` (`X-Forwarded-For`), or `forwarded` (RFC 7239). **Exactly one is ever read**, so a client cannot pick whichever header the proxy does not set. An unrecognised value — including a typo — is `off`, never a guess. |
| `MW_PROXY_PROTOCOL` | `off` | PROXY protocol on the HTTP/HTTPS listener: `off`, `accept` (a trusted peer *may* send one), `require` (every connection must arrive from a trusted peer with a valid header; anything else is dropped). Use `require` once the port is behind an L4 balancer — a connection without a header there did not come through the balancer. |
| `MW_PUBLIC_URL` | *(unset)* | The canonical external base, e.g. `https://mail.example.com`. Highest-precedence source for the public scheme and host. **Read the migration note in §3 before setting it.** |
| `MW_BASE_PATH` | *(unset — root)* | Serve the application under a path prefix, e.g. `/mail`. See §6. |
| `MW_HEADER_AUTH_TRUSTED_IPS` | *(empty — deny)* | Which peers may assert `X-Remote-User` when `MW_HEADER_AUTH=1`. Fails closed: an unset or unparseable list authenticates **nobody** rather than everybody. See §5. |
| `MW_ASSIST_RATE_LIMIT_PER_MIN` | *(unset — unlimited)* | Per-account Assist budget in **outbound endpoint requests** per minute. `0` is an explicit hard stop (a kill switch that leaves the rest of the deployment running). Sizing matters: a cold-cache semantic search costs up to **33** requests (one for the query, up to 32 documents embedded on demand) and converges toward 1 as the cache fills, while a chat turn costs 1. `60` permits roughly two cold semantic searches a minute per account. |

Also relevant, and not new: `MW_COOKIE_SECURE` (absolute override in both
directions), `MW_HSTS` / `MW_HSTS_MAX_AGE` / `MW_HSTS_INCLUDE_SUBDOMAINS` /
`MW_HSTS_PRELOAD`.

### How the effective scheme is decided

In precedence order: `MW_PUBLIC_URL` (stated by you, so nothing can move it) →
the application's own TLS listener, if it is terminating TLS itself →
`X-Forwarded-Proto`, **only** from a peer inside `MW_TRUSTED_PROXIES`. A
forwarded header can only ever *raise* the answer to https, never lower it, and
`MW_COOKIE_SECURE` overrides the lot.

**`MW_TRUSTED_PROXIES` is load-bearing on its own for scheme and host.** That
trust gates on the proxy list alone, not on `MW_FORWARDED_MODE`. So
`MW_TRUSTED_PROXIES` set with `MW_FORWARDED_MODE=off` means "scheme and host
trusted, client IP still peer-only" — a combination that was inert before 26.19.
The worst a false assertion from such a peer produces is `Secure` cookies over
plaintext, or HSTS for a host with no TLS: both availability problems,
self-inflicted by a peer you explicitly trusted, and neither a confidentiality
nor an authentication widening.

---

## 3. Migration note — setting `MW_PUBLIC_URL` can invalidate existing passkeys

`MW_PUBLIC_URL=https://…` does two things at once:

1. It raises the cookie `Secure` floor, which is the point.
2. **It changes the WebAuthn origin scheme**, and therefore the origin every
   registered passkey was bound to.

A deployment that has been serving over plain http, or that was letting the
scheme be derived, and then sets an https `MW_PUBLIC_URL` **will find that
existing passkeys stop validating** — the assertion arrives bound to the old
origin and is rejected. Users fall back to password + TOTP or recovery codes and
must re-enrol their passkeys.

`MW_PUBLIC_URL` is new in 26.19, so this can only bite a deployment that adopts
it, never one that upgrades and leaves it alone. Set it during a window where
re-enrolment is acceptable, and tell your users first.

---

## 4. The proxy matrix — what has actually been proven

**Nothing in `docs/deploy/proxy/` is evidence that a proxy works.** A shipped
config file is a claim. The proof is booting the cell and running the
conformance suite through it.

| Proxy | Config shipped | In the CI matrix | Booted + suite run | Result |
|---|---|---|---|---|
| **nginx** | ✅ | ✅ | ✅ developer host | 11/12, sole failure since fixed app-side |
| **Caddy** | ✅ | ✅ | ✅ developer host | 11/12, sole failure since fixed app-side |
| **HAProxy (L7)** | ✅ | ✅ | ✅ developer host | 11/12, sole failure since fixed app-side |
| **Traefik** | ✅ | ✅ | ✅ developer host | 11/12, sole failure since fixed app-side |
| **Apache httpd** | ✅ | ✅ | ✅ developer host | **10/12** — see §4.2 |
| **HAProxy (L4, PROXY protocol)** | ✅ | ✅ | ✅ developer host | **9/12** — see §4.3 |
| **Envoy** | ✅ | ❌ | once, off-matrix | see §4.4 |
| **IIS** | ✅ | ❌ **cannot be** | ❌ **never** | see §4.4 |

### 4.1 What "in the CI matrix" does and does not mean

`.github/workflows/proxy-conformance.yml` is committed and correctly shaped —
same environment contract, same ports, `up -d --wait` then `down -v` as the
manual runs. **At the time of writing it has not been observed to complete a
run**, and every result in the table above comes from a developer host with
Docker, not from a CI runner.

So: **do not say the proxy conformance matrix passes in CI.** Say that six cells
were booted and measured, and that CI is configured to repeat it. The
distinction is not pedantry — this repository has already shipped a conformance
workflow (`t17-conformance.yml`) that never parsed, and therefore never ran, for
an entire release cycle without anyone noticing, because GitHub reports an
unparseable workflow as a file problem rather than a failing job.

### 4.2 Apache — supported, with one contract caveat

Apache is a tier-1 cell and its configuration works, but two things were broken
as shipped and are worth knowing about:

- **The cell could not start httpd at all.** `mailwoman.conf` used `SetEnv`
  (`mod_env`) while `httpd.conf` loaded `mod_setenvif`, a different module. Fixed;
  if you adapt these files, load `mod_env`.
- **The JSON error contract on oversize uploads.** Mailwoman answers an oversize
  upload with a JSON `413`. Behind `mod_proxy` this arrived as httpd's own
  `502 text/html`, because the application answered and closed while the client
  was still writing, and `mod_proxy` abandons an exchange whose write failed
  **without reading the response already waiting on the socket**. Five mod_proxy
  configurations were measured and none of them changed it; there is no directive
  meaning "on write failure, read the response anyway".

  **This is fixed in the application, not in the configuration**: the request
  body is now drained (bounded by both bytes and time) before the `413` is
  written, so the client can finish writing. The fix is proven at the socket
  level — a raw client writing a 75 MB body now completes and receives the JSON
  `413`, where before it aborted mid-write.

  **Honest limit: the Apache cell has not been re-booted since that fix.** The
  cause is closed and the mechanism is understood, so the contract is expected to
  hold on Apache now. It has not been *observed* to hold through httpd. Until a
  run confirms it, state the JSON error contract as proven on the other five
  cells and expected on Apache.

### 4.3 HAProxy L4 — the PROXY-protocol cell

This is `mode tcp` with `send-proxy-v2` against the application's **own TLS
listener**: no HTTP parsing, no forwarded headers, the client address arriving
out of band. It exists because the built-in ACME client means the application can
be the TLS endpoint, which is also what an AWS NLB or an nginx `stream` block
gives you.

**PROXY protocol v2 has been verified against a real `send-proxy-v2` sender**,
which is what closes the loop on a parser whose test vectors were otherwise
hand-built to the specification. Two caveats: the `PP2_TYPE_CRC32C` TLV is
consumed but **not checked**, so do not describe the implementation as
conformant; and the cell could not boot as shipped (omitting `healthcheck:` does
not suppress the image's baked-in `HEALTHCHECK`, which spoke plain HTTP to a
TLS-only listener — `healthcheck: disable: true` is required).

### 4.4 Envoy and IIS — shipped configuration only

Neither is supported. Ship-and-document is not the same as tested, and the
difference is stated here rather than left to the reader.

- **Envoy** is out of the CI matrix because it overlaps HAProxy L7 and Traefik.
  It has one genuinely distinct behaviour: it **appends** to `X-Forwarded-For`
  rather than overwriting, so it is the only configuration that depends on the
  application's right-to-left hop walk being correct. It has been booted **once**,
  on a developer host, specifically to prove that walk against a real appending
  proxy — that one property is earned. Nothing else about the Envoy configuration
  is.
- **IIS** cannot be tested in CI at all and **has never been booted by anyone**.
  The `web.config` and its runbook are a starting point written from the
  documentation, not a verified recipe. See
  [`proxy/iis/README.md`](proxy/iis/README.md), which is explicit about what has
  and has not been checked.

Do not write "supports Envoy" or "supports IIS" anywhere.

### 4.5 Claims that are not earned

For anyone writing release notes or marketing copy from this page:

- ❌ **Not** "RFC 7239 supported". `Forwarded` is read for the **client IP
  only**; its `proto=` parameter is ignored, so a strict-7239 proxy that emits
  `Forwarded` and no `X-Forwarded-Proto` leaves the effective scheme at the
  listener's. Every proxy in our matrix emits `X-Forwarded-Proto`, so this is not
  reachable here, and the failure direction is safe — it degrades to the
  pre-26.19 posture and never grants trust.
- ❌ **Not** "PROXY protocol v2 verified/conformant" — see the CRC32C note above.
- ❌ **Not** "the conformance matrix passes in CI" — see §4.1.
- ❌ **Not** "`MW_BASE_PATH` isolates the app to a sub-path" — see §6.

---

## 5. Header authentication and the proxy trust list are not independent

`MW_HEADER_AUTH=1` plus `MW_HEADER_AUTH_TRUSTED_IPS` lets a listed peer assert
`X-Remote-User` and be issued that user's session with no credential. The
allowlist is a **separate list** from `MW_TRUSTED_PROXIES` on purpose: a proxy
trusted to report *where a client is* is not thereby trusted to declare *who
they are*.

**That separation holds only while `MW_PROXY_PROTOCOL=off`.**

With `accept` or `require`, the peer address is itself a value the connecting
proxy declares in the PROXY header, and that declared address is what the
header-auth gate checks. So any member of `MW_TRUSTED_PROXIES` can name a peer
address inside `MW_HEADER_AUTH_TRUSTED_IPS` and then assert an identity for a
password-less session.

This grants nothing new when `MW_TRUSTED_PROXIES` names your balancer's own
addresses — that membership is already total authority over the client-IP model.
**It matters when the list is written as a subnet** (a pod range, a VPC CIDR)
covering hosts other than the balancer, because every host in that range can then
assert any identity.

> **If you enable both header auth and PROXY protocol, list individual addresses
> in `MW_TRUSTED_PROXIES`, not ranges.**

Header auth remains safe only behind a proxy that **strips** any client-supplied
copy of the header. Every configuration under `docs/deploy/proxy/` does this.

---

## 6. Sub-path hosting (`MW_BASE_PATH`)

`MW_BASE_PATH=/mail` mounts the whole application under a prefix. **New in
26.19 — this variable was documented before any code read it**, so setting it
previously had no effect at all.

Two things to understand before using it.

**The application also stays mounted at the origin root, deliberately.** The
prefix is *routing*, not an isolation boundary. `/healthz` must stay reachable
for a container health check and `/.well-known/*` must stay reachable for WKD and
JMAP autodiscovery, whatever prefix the UI lives under — both live at the origin
root by definition. Do not set `MW_BASE_PATH` expecting it to hide the
application; it does not, and it is not trying to.

**No shipped proxy configuration includes a sub-path recipe.** Sub-path hosting
is proven for `/mail/api/*` through a real nginx — API routes beneath the prefix
return JSON, which is the failure mode that matters, because the intuitive
implementation makes every `/mail/api/*` call return `200 text/html` while
looking entirely healthy on a status-only check. But the shipped nginx
configuration's `location` blocks are prefix-blind, so **WebSocket push breaks
under a prefix** with that file as written. If you need sub-path hosting today,
expect to adapt the location blocks yourself, and test push specifically.

Both reverse-proxy idioms work at the application end — `proxy_pass http://app;`
(prefix preserved) and `proxy_pass http://app/;` (prefix already stripped).

---

## 7. Why each proxy is in the matrix

Six proxies is not six copies of one test; each has a stock default that breaks
something specific. The reasoning, the fixed host-port contract, the harness
addressing and the per-file layout are all in
[`proxy/README.md`](proxy/README.md). The short version:

- **nginx** — worst default set of the group: `proxy_buffering on` swallows
  server-sent events, `client_max_body_size 1m` rejects attachments, HTTP/1.0
  upstream means no WebSocket upgrade. It is also the only proxy that honours
  `X-Accel-Buffering`, which is why the application emits that header at all.
- **Apache httpd** — `ProxyPreserveHost` defaults to **Off**, which rewrites
  `Host` and therefore silently changes the WebAuthn RP ID, so existing passkeys
  stop validating. And `mod_proxy` *appends* to `X-Forwarded-For`, so the correct
  pattern is to unset the client's and let `mod_proxy` add one hop.
- **Caddy** — gets the most right by default; it is the zero-config path most
  people try first and it is in the matrix to keep that path working.
- **HAProxy L7** — `timeout tunnel` governs an upgraded connection and neither
  `timeout client` nor `timeout server` applies once the WebSocket is up.
- **HAProxy L4** — the only PROXY-protocol cell; see §4.3.
- **Traefik** — closest analogue to the Kubernetes ingress story the Helm chart
  needs, and it splits static from dynamic configuration in a way that makes it
  easy to put a timeout in the file where it is ignored.

**Body limits are set above the application's, on purpose.** Every cell allows
100 MB where the application's upload limit is 50 MB. If the proxy's limit sat at
or below the application's, an oversized upload would be rejected by the proxy and
the user would get nginx's or IIS's HTML error page instead of Mailwoman's JSON
error. Letting the request through so the application is the one that says no is
what keeps the error contract intact.
