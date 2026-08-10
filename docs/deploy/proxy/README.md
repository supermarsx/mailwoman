# Reverse-proxy configurations and the conformance harness

This directory holds one configuration tree per proxy, plus the TLS material
generator they share. The container harness that boots them is
`docker-compose.proxy.yml` at the repository root.

The operator-facing document is `docs/deploy/reverse-proxy.md`. **This file
describes the harness**; that one describes what is supported.

## What "tier" means here, and what has actually been proven

Nothing in this directory is evidence that a proxy works. A shipped config file
is a claim, not a test — and this repository has a history of audits catching
exactly that pattern. The proof is a CI cell that boots the proxy and runs the
conformance suite through it.

| Cell | Image | Tier | In the CI matrix | Booted + suite run |
|---|---|---|---|---|
| `nginx` | `nginx:1.27-alpine` | 1 | yes | ✅ developer host — 11/12 |
| `apache` | `httpd:2.4-alpine` | 1 | yes | ✅ developer host — 10/12 |
| `caddy` | `caddy:2-alpine` | 1 | yes | ✅ developer host — 11/12 |
| `haproxy-l7` | `haproxy:3.0-alpine` | 1 | yes | ✅ developer host — 11/12 |
| `haproxy-l4` | `haproxy:3.0-alpine` | 1 | yes | ✅ developer host — 9/12 |
| `traefik` | `traefik:v3.3` | 1 | yes | ✅ developer host — 11/12 |
| `envoy` | `envoyproxy/envoy:v1.32` | 2 | no — on-demand profile only | once, off-matrix, for the appending-XFF walk only |
| `iis` | — | 2 | **cannot be** | ❌ never; see `iis/README.md` |

**Updated in 26.19.** This table used to say "intended" in both columns, which
was the honest word when the directory was written and had stopped being honest
once the cells were actually booted. Every tier-1 cell has now been started and
had the conformance suite run through it — on a **developer host with Docker,
not on a CI runner.** `.github/workflows/proxy-conformance.yml` is committed and
correctly shaped, and at the time of writing has not been observed to complete a
run, so **"passes in CI" is still not a thing anyone may say.**

Booting them found six real defects, two of which meant a cell could not start
at all (`apache` was missing a `LoadModule`; `haproxy-l4` inherited the image's
baked-in `HEALTHCHECK`, which spoke plain HTTP to a TLS-only listener). Both are
fixed here. Three more were application defects, not proxy ones, and are fixed in
the application. The remaining `apache` failure — the JSON `413` contract on
oversize uploads — was also fixed application-side, **after** the last cell run,
so it is closed in cause but not yet observed through httpd.

`docs/deploy/reverse-proxy.md` is the operator-facing page and carries the full
per-cell status, the caveats, and the list of claims that are still not earned.

## Fixed host ports

These are a contract. The CI workflow and the conformance suite address cells by
port; renumbering breaks both.

| Profile | HTTP | HTTPS |
|---|---|---|
| `nginx` | 8601 | 8641 |
| `apache` | 8602 | 8642 |
| `caddy` | 8603 | 8643 |
| `haproxy-l7` | 8604 | 8644 |
| `haproxy-l4` | — | 8645 |
| `traefik` | 8605 | 8646 |
| `envoy` | 8606 | 8647 |
| the app itself, no proxy | 8607 | — |

The app is published on 8607 in every profile, so a test can compare a
through-proxy result against the no-proxy baseline in the same run.

## Running a cell

```sh
# Once: generate the throwaway self-signed TLS pair the HTTPS listeners need.
sh docs/deploy/proxy/tls/gen-certs.sh

docker compose -f docker-compose.proxy.yml --profile nginx up -d --wait

MW_T20_PROXY=1 \
MW_T20_PROXY_BASE=http://127.0.0.1:8601 \
MW_T20_PROXY_KIND=nginx \
  cargo test -p mw-server --test t20_proxy_conformance -- --nocapture --test-threads=1

docker compose -f docker-compose.proxy.yml --profile nginx down -v
```

`down -v` rather than `down`: the app volume holds a seeded SQLite database and
leaving it behind makes the next cell start from another cell's state.

## What each cell is here to catch

Six proxies is not six copies of the same test. Each one has a stock default
that breaks something specific, and that is why it is in the matrix.

**nginx** is the baseline and the worst default set of the group. Three stock
values each break something: `proxy_buffering on` batches or swallows server-sent
events, `client_max_body_size 1m` rejects attachments long before the
application's own 50 MB limit, and HTTP/1.0 upstream means no WebSocket upgrade.
It is also the only proxy that honours `X-Accel-Buffering`, which is why the
application emits that header at all.

**Apache httpd** was named as supported in `SPEC.md` §18.1 for a long time while
the repository shipped nothing for it. Its failure modes are genuinely different
from nginx's: `ProxyPreserveHost` defaults to **Off**, which rewrites `Host` to
the backend name and silently changes the WebAuthn RP ID, so existing passkeys
stop validating. And its forwarded-header handling runs the opposite way round —
`mod_proxy` *appends* to whatever `X-Forwarded-For` the client sent, so the
correct pattern is to unset the client's and let `mod_proxy` add one hop, not to
set the header yourself.

**Caddy** gets the most right by default: HTTP/2, WebSocket upgrades with no
directive, no response buffering, no body limit. It is the zero-config path most
people try first, and it is in the matrix to prove that path stays working.

**haproxy-l7** has its own timeout model. `timeout tunnel` governs an upgraded
connection and neither `timeout client` nor `timeout server` applies once the
WebSocket is established — leave it at the default and push sockets get cut on a
schedule that has nothing to do with the heartbeat.

**haproxy-l4** is the distinctive one: `mode tcp` with `send-proxy-v2` against
the application's **own TLS listener**. No HTTP parsing, no forwarded headers at
all — the client address arrives out of band in a PROXY protocol v2 header. This
shape exists because the application ships a built-in ACME client, and for
TLS-ALPN-01 to work the application has to be the TLS endpoint. It is also what
an AWS NLB or an nginx `stream` block gives you. This is the only cell that
exercises PROXY protocol.

**traefik** is the closest analogue to the Kubernetes ingress story the Helm
chart needs, and it splits configuration into static and dynamic halves in a way
that makes it easy to put a timeout in the file where it is ignored.

**envoy** overlaps haproxy-l7 and traefik closely enough that it was cut from the
default matrix, but it has one genuinely distinct behaviour worth reading:
unlike every other cell it *appends* to `X-Forwarded-For` rather than
overwriting, which means it is the one configuration that depends on the
application's right-to-left walk being correct. See the comments in
`envoy/envoy.yaml`.

**iis** cannot be tested in CI at all, and has never been booted by anyone. See
`iis/README.md`, which is explicit about what has and has not been verified.

## The trust model these configs feed

The application ignores forwarded headers unless `MW_TRUSTED_PROXIES` names the
peer. That is a deliberate change of behaviour — trusting `X-Forwarded-For` from
anyone means the OAuth scoped-key IP allowlist can be defeated with a header,
the per-key rate limit can be evaded by rotating one, and every audit-log IP is
attacker-chosen. Ignoring the headers by default is the fix.

So every configuration here does two things, and both are necessary:

- **Overwrite `X-Forwarded-For` rather than append it** (Envoy excepted, above),
  so the application receives exactly one hop that the client could not
  influence. Each file carries a comment showing the append form to use instead
  in a genuine proxy *chain*, and the condition under which that is safe: every
  upstream hop must be listed in `MW_TRUSTED_PROXIES`. A hop listed by mistake
  can claim any client address it likes.
- **Strip the RFC 7239 `Forwarded` header** the client may have sent, since none
  of these configurations speak it.

They also preserve `Host` including its port. The application derives the OAuth
DCR issuer URL and the WebAuthn RP ID from `Host` and compares `Origin` against
it for CSRF, so a proxy that rewrites or truncates it breaks login in ways that
are hard to trace back to the proxy.

### Addresses inside the harness

`docker-compose.proxy.yml` pins the network so the allowlist is deterministic
and, critically, **narrower than the network**:

```
172.28.10.0/24   proxy containers   <- the only trusted range
172.28.20.0/24   app containers
172.28.0.1       bridge gateway     <- host traffic appears from here; untrusted
```

A test running on the host reaches the proxy as `172.28.0.1`. That address is
deliberately outside `MW_TRUSTED_PROXIES`, so the right-to-left walk stops there
and attributes the request to it. **The "true client IP" a test should expect is
the gateway address, not the host's LAN address** — Docker NATs host-originated
traffic and there is no way to see the host's real address from inside the
network. This does not weaken what is being proven: the gateway address is
distinguishable from every proxy address, which is the whole question.

## Body limits are set *above* the application's, on purpose

Every cell allows 100 MB where the application's upload limit is 50 MB. That
looks backwards and is not. If the proxy's limit sat at or below the
application's, an oversized upload would be rejected by the proxy and the client
would get an HTML error page from nginx or IIS instead of the application's JSON
error. Letting the request through so the application is the one that says no is
what keeps the error contract intact.

## Files

```
nginx/       nginx.conf (container-complete) + mailwoman.conf (drop-in vhost)
apache/      httpd.conf (container-complete) + mailwoman.conf (drop-in vhosts)
caddy/       Caddyfile
haproxy-l7/  haproxy.cfg — mode http, TLS terminated here
haproxy-l4/  haproxy.cfg — mode tcp, send-proxy-v2, TLS passthrough
traefik/     traefik.yml (static) + dynamic.yml (routers/service/TLS)
envoy/       envoy.yaml — tier 2
iis/         web.config + README.md runbook — tier 2, unverified
tls/         gen-certs.sh; certs/ is generated and gitignored
```

The TLS material is a throwaway self-signed leaf for the harness only. Real
deployments use a CA-issued pair, the proxy's own ACME, or the application's
built-in ACME behind an L4 balancer.
