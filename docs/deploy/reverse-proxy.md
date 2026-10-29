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
| `MW_ASSIST_RATE_LIMIT_PER_MIN` | *(unset — unlimited)* | Per-account AI assistant budget in **outbound endpoint requests** per minute. `0` is an explicit hard stop (a kill switch that leaves the rest of the deployment running). Sizing matters: a cold-cache semantic search costs up to **33** requests (one for the query, up to 32 documents embedded on demand) and converges toward 1 as the cache fills, while a chat turn costs 1. `60` permits roughly two cold semantic searches a minute per account. |

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
| **nginx** | ✅ | ✅ | ✅ developer host, server strict | 11 of 13 run and pass, 2 skip by design |
| **Caddy** | ✅ | ✅ | ✅ developer host, server strict | 11 of 13 run and pass, 2 skip by design |
| **HAProxy (L7)** | ✅ | ✅ | ✅ developer host, server strict | 11 of 13 run and pass, 2 skip by design |
| **Traefik** | ✅ | ✅ | ✅ developer host, server strict | 11 of 13 run and pass, 2 skip by design |
| **Apache httpd** | ✅ | ✅ | ✅ developer host, server strict | 11 of 13 run and pass, 2 skip by design — see §4.2 |
| **HAProxy (L4, PROXY protocol)** | ✅ | ✅ | ✅ developer host, server strict | 11 of 13 run and pass, 2 skip by design — see §4.3 |
| **Envoy** | ✅ | non-blocking cell, not in the tier-1 gate | developer host, server strict; CI run 38312068826 | 12 of 13 run and pass, the sub-path test skips — see §4.4 |
| **IIS** | ✅ | ❌ **cannot be** | ❌ **never** | see §4.4 |

### 4.1 What "in the CI matrix" does and does not mean

`.github/workflows/proxy-conformance.yml` boots each cell with the same
environment contract and ports as the manual runs (`up -d --wait`, then
`down -v`). The results in the table's last column come from a developer-host
run on 2026-10-23 of every cell with the server strict (`MW_CSRF_STRICT=true`,
as `deploy.sh` configures it), images built from `b889b9d`; the 13th test
checks that a write without `X-CSRF-Token` is refused. The workflow has also
completed: in run 38312068826 (commit `b889b9d`, before the cells ran strict,
12 tests) all seven cells — the six tier-1 cells and the non-blocking Envoy
cell — finished their suite with no failures. Its skip log names the sub-path test in every
cell and the appending-proxy test in every cell except Envoy. A test that
returns early without writing to the skip log looks like a pass in that output.

So: **say which run and which commit** when you say the matrix passes in CI,
not "the matrix passes in CI" as a standing fact; one green run says nothing
about the commits after it. The distinction is not pedantry — this repository
has already shipped a conformance
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

  The Apache cell has since run in CI: in run 38312068826 (commit `b889b9d`)
  `a_large_upload_succeeds_and_an_oversize_one_gets_the_apps_json_413` passed
  through httpd and is not in that run's skip log. That is one run on one
  commit. The strict developer-host run of 2026-10-23 passed it as well.

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

### 4.4 Envoy (a non-blocking cell) and IIS (shipped configuration only)

Neither is supported. Ship-and-document is not the same as tested, and the
difference is stated here rather than left to the reader.

- **Envoy** is not a tier-1 (gating) proxy, because it overlaps HAProxy L7 and
  Traefik. It is in the CI matrix as a **non-blocking** cell: a red Envoy cell
  does not fail the gate. It is there for one test. Envoy **appends** to
  `X-Forwarded-For` rather than overwriting, so it is the only shipped
  configuration that depends on the application's right-to-left hop walk being
  correct, and the only cell in which
  `an_appending_proxy_still_resolves_the_real_client` runs instead of skipping.
  The suite has passed against it on a developer host and in CI run
  38312068826 (11 of 12 tests run, the sub-path test skips), and in the
  strict developer-host run of 2026-10-23 (12 of 13 run, the same skip). That one
  property is earned. Nothing else about the Envoy configuration is.
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
- ❌ **Not** "the conformance matrix passes in CI" without the run and the
  commit — see §4.1.
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

---

## 8. `deploy.sh` behind a proxy, and after a crash — what is tested

The proxy matrix in §4 runs the conformance suite against a compose file made
for it. `scripts/test-deploy-proxy.sh` instead installs the app with
`deploy.sh`, as an operator would, and checks it from the outside:

```sh
bash scripts/test-deploy-proxy.sh mailwoman:proxy-test mailwoman-mock:proxy-test
```

Build the two images first with `docker build --target runtime` and
`docker build --target mock`. Each case prints `PASS`, `FAIL` or
`SKIP: <reason>`; with `DEPLOY_TEST_REQUIRE=all` a skip makes the script exit
non-zero. `DEPLOY_PROXY_CASES` picks cases (`restart npm caddy`). Every
container, network and volume it creates carries a random project name and is
removed afterwards.

| Case | What it does | What it checks |
|---|---|---|
| 1 crash restart | `deploy.sh` on a localhost port; the server process is killed with SIGKILL from the host PID namespace (`docker run --privileged --pid=host … kill -9 <pid>`) | Docker records a `die` event with exit code 137 and restarts the container (`RestartCount` +1); health returns; an admin ban written before the kill is still listed |
| 2 Nginx Proxy Manager | `deploy.sh --public-url https://… --proxy-network <net>`; `jc21/nginx-proxy-manager:2.15.1` (pinned by digest) on that network, set up through its API: one proxy host forwarding to `http://<project>:8080` with WebSocket support, and an uploaded throwaway certificate | from a client on a second network: the certificate verifies against the throwaway CA and not without it; `/healthz`; the SPA; admin login, a write refused without `X-CSRF-Token` and accepted with it; `/jmap/ws` answers 101 with the `jmap` subprotocol for a mailbox session and 401 without one; the address the app logs for a failed admin login is the client's, not the proxy's |
| 3 Caddy with a local ACME server | `deploy.sh --domain` with `ACME_CA` / `ACME_CA_ROOT` pointing at Pebble 2.10.1 (the ACME test server), whose DNS server (`pebble-challtestsrv`) answers every name with Caddy's address, so Pebble validates through Caddy's ports 80 and 443. Needs host ports 80 and 443 free, else `SKIP` | Caddy obtains a certificate from Pebble; `deploy.sh`'s own check of `https://<domain>/` from the host passes (a curl config in `CURL_HOME` maps the domain to 127.0.0.1 and trusts Pebble's root); the served chain verifies against Pebble's issuing root, fetched from Pebble before the deploy, and not without it; its issuer is Pebble's intermediate; then the case 2 checks through Caddy |

Why case 1 does not use `docker kill`: on Docker 29.4.2 a `docker kill` is
recorded as a manual stop, and a container with `restart: unless-stopped`
(what `deploy.sh` writes) is then not restarted. The same script with
`docker kill` in place of the host-namespace kill fails, and so does a run with
the restart policy set to `no`; the forwarded-address check fails when the app
is told not to trust the proxy network (`MW_FORWARDED_MODE=off`), and the
WebSocket checks fail when the proxy host is created without WebSocket support.

**Recorded run:** 2026-10-23, `deploy.sh` and the script at commit
`d1a69ef`, image `mailwoman:t31-w0` built from commit `b889b9d`, Docker 29.4.2
with Compose 5.1.3 (Docker Desktop, WSL2 kernel 6.6.114.1, x86_64), Caddy
`caddy:2-alpine` (image ID `sha256:eb4d35b7…`, 2.11.7), with
`DEPLOY_TEST_REQUIRE=all`: case 1 PASS (healthy again 10 s after the kill),
case 2 PASS, case 3 PASS (certificate issued by Pebble's intermediate); exit 0.

Case 3 found a defect on its first run: before `d1a69ef`, Compose (5.1.3, and
2.38.2) refused to start the Caddy service that `deploy.sh --domain` wrote —
`cannot create config … in read-only service caddy: file is the sole supported
option` — because the Caddyfile was an inline `configs: content:` on a
`read_only: true` container, so no Caddy installation made by `deploy.sh`
started. Since `d1a69ef` the Caddyfile and the ACME root are environment values
that the entrypoint writes to the container's `/tmp`. Case 3 also fails when
Pebble cannot validate through Caddy (Pebble set to its default validation
ports): no certificate is issued and `deploy.sh`'s own host check fails.

**Not proven by this script:** issuance by the public Let's Encrypt endpoint
for a real domain; DNS and inbound reachability on ports 80 and 443;
certificate renewal over time; Nginx Proxy Manager's own Let's Encrypt flow and
any setting made by hand in its UI; behaviour on Docker engines other than the
one used for the recorded run, and Caddy images other than the `caddy:2-alpine`
that `deploy.sh` pulled for it. Case 1 kills the process once; it does not show
what happens to a write in progress at the moment of the kill.

---

## 9. HTTP/3 (QUIC) — terminated at the proxy

Mailwoman has no QUIC listener and does not speak HTTP/3 itself. HTTP/3 is
terminated by the proxy in front of it; the proxy talks HTTP/1.1 to the app and
forwards the client's address, scheme and host in `X-Forwarded-For`,
`X-Forwarded-Proto` and `Host`/`X-Forwarded-Host`, exactly as it does for an
HTTP/2 client. Nothing changes on the application side: the same
`MW_TRUSTED_PROXIES` / `MW_FORWARDED_MODE` / `MW_PUBLIC_URL` settings (§2)
decide what is trusted, whichever protocol the browser used.

### 9.1 Which shipped configurations enable it

| Proxy | HTTP/3 in the shipped config | Checked with `scripts/proxy-h3-check.sh` |
|---|---|---|
| **Caddy** (`docs/deploy/proxy/caddy/Caddyfile`) | on by default for every TLS site; nothing to add | yes, 2026-10-29, developer host |
| **`deploy.sh --domain`** (managed Caddy) | on by default (`CADDY_PROTOCOLS='h1 h2 h3'`), publishes `443/udp`; see [script.md](script.md) | rendered manifest only (`scripts/test-deploy-http-protocols.sh`) |
| **nginx** (`docs/deploy/proxy/nginx/mailwoman.conf`, `docs/deploy/nginx.conf`) | `listen … quic reuseport;` + `http3 on;` + `add_header Alt-Svc 'h3=":443"; ma=86400' always;` — needs nginx ≥ 1.25 built with `--with-http_v3_module` (the official images are) | yes (conformance vhost), 2026-10-29, developer host; `docs/deploy/nginx.conf` passes `nginx -t` |
| **Traefik** v3 (`docs/deploy/proxy/traefik/traefik.yml`) | `http3: {}` on the `websecure` entry point | yes, 2026-10-29, developer host |
| HAProxy, Apache httpd, Envoy, IIS | not configured in the shipped files | no |

**nginx: `Host` over HTTP/3.** An HTTP/3 request has no `Host` header, only
`:authority`, and nginx does not create one, so `$http_host` is empty over
QUIC. A vhost that sends `proxy_set_header Host $http_host;` then sends no
`Host` to the app, the app's Origin check has nothing to compare with, and
every login over HTTP/3 answers **403** while HTTP/2 works. The check script
found exactly that in the conformance vhost. It now sends `$mw_host`, a `map`
that uses `$http_host` when present and otherwise `$host` plus the port when it
is not 443. `docs/deploy/nginx.conf` uses `$host` on port 443, which is
correct for HTTP/3 as it is.

### 9.2 Firewall and port publishing

QUIC runs over **UDP 443**. Open it next to TCP 80/443 in the host firewall
(`ufw allow 443/udp`, `firewall-cmd --add-port=443/udp`), in any cloud security
group, and on any load balancer in front (an L4 balancer must forward UDP, not
only TCP). In a container, publish the port for both protocols:
`"443:443/tcp"` and `"443:443/udp"`. `docker-compose.proxy.yml` publishes the
nginx, caddy and traefik HTTPS ports over UDP for the check below.

### 9.3 Fallback

Browsers do not start on HTTP/3. The first connection is TCP (HTTP/2 or 1.1);
the response carries `Alt-Svc: h3=":443"`, and the browser tries QUIC on later
requests. If UDP 443 is blocked or the QUIC handshake fails, the browser keeps
using the TCP connection; the user sees no error. WebSocket connections
(`/jmap/ws`) are opened over TCP, so the proxy's TCP listener and its WebSocket
settings stay required with HTTP/3 enabled. To switch HTTP/3 off, drop the
`quic` listeners and `Alt-Svc` (nginx), remove `http3` (Traefik), or set
`protocols h1 h2` (Caddy; `CADDY_PROTOCOLS='h1 h2'` for `deploy.sh`).

### 9.4 How to check a deployment

```sh
# The TCP response must advertise HTTP/3:
curl -sI --http2 https://mail.example.org/healthz | grep -i '^alt-svc'
# HTTP/3 only, no TCP fallback. Needs a curl built with HTTP/3
# (`curl -V` lists HTTP3 under Features); prints "3" when QUIC worked:
curl -s -o /dev/null -w '%{http_version}\n' --http3-only https://mail.example.org/healthz
```

In the browser, open the developer tools' Network panel, enable the
**Protocol** column and reload twice: after the first load the requests show
`h3`. If they stay on `h2`, UDP 443 is not reaching the proxy.

### 9.5 What is tested

`scripts/proxy-h3-check.sh <caddy|nginx|traefik> [--published PORT]` runs
against a booted cell of `docker-compose.proxy.yml`. A curl build with HTTP/3
(quiche; image pinned in the script) joins the cell's network at 172.28.30.5,
outside `MW_TRUSTED_PROXIES`, and:

1. requests `/healthz` over TCP, asserting HTTP/2 and an `Alt-Svc: h3=` header;
2. requests `/healthz` with `--http3-only`, asserting HTTP/3;
3. over HTTP/2 **and** over HTTP/3, asserts what the app saw: a login whose
   `Origin` is the proxy's address is accepted and one with a foreign `Origin`
   is refused (the app compares `Origin` with `Host`); the session cookie
   carries `Secure` and the response carries HSTS (from a trusted
   `X-Forwarded-Proto: https`); an API key allowlisted to 172.28.30.5 answers
   200 and one allowlisted to 203.0.113.9 answers 403 (from `X-Forwarded-For`);
4. with `--published`, requests HTTP/3 from the host to the cell's published
   UDP port.

**Recorded run:** 2026-10-29, Docker 29.4.2 with Compose 5.1.3 (Docker
Desktop, Windows 11), app image `mailwoman:t33e15c`, `caddy:2-alpine` (2.11.7),
`nginx:1.27-alpine` (1.27.5), `traefik:v3.3` (3.3.7): all four steps pass for
caddy, nginx and traefik, including the published UDP port. Before the
`$mw_host` change, the nginx cell failed step 3 over HTTP/3 only (login 403).

The `h3` job in `.github/workflows/proxy-conformance.yml` runs the script for
the three cells. It is **non-blocking** and not part of the tier-1 gate: no
hosted runner has run it yet, and UDP between the runner and its containers is
unverified there. Not proven: HTTP/3 from a real browser (the steps above use
curl), 0-RTT, connection migration, and any proxy not listed in §9.1.
