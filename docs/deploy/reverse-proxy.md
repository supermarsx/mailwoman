# Running Mailwoman behind a reverse proxy

This page explains how to put Mailwoman behind nginx, Apache httpd, Caddy, HAProxy,
Traefik, Envoy or IIS: which environment variables the server needs, what the proxy
must send and allow, and how to check the result. Example configurations for each
proxy are in [`proxy/`](proxy/README.md).

## Requirements

The proxy in front of Mailwoman must:

- pass the browser's `Host` header through unchanged, including any port;
- overwrite `X-Forwarded-For` with the client address and set `X-Forwarded-Proto`;
- pass WebSocket upgrades on `/jmap/ws` (and preferably on `/jmap/actions`; without
  it the web client sends those requests over ordinary HTTP);
- not buffer or compress the event streams `/jmap/eventsource` and `/api/assist/invoke`;
- keep idle streams open for longer than the server's 30-second heartbeat;
- accept request bodies larger than Mailwoman's own limits (50 MB for an attachment
  upload, 64 MiB for a mailbox import), so that Mailwoman, not the proxy, answers an
  oversized upload.

The server must be told which addresses the proxy connects from. Until you do,
Mailwoman ignores every forwarded header and treats the proxy as the client.

## Set up Mailwoman behind a proxy

1. Start from the example configuration for your proxy in [`proxy/`](proxy/README.md).
   Change the upstream address, the server name and the certificate paths.
2. Find the address the proxy connects to Mailwoman **from**. In Docker or
   Kubernetes this is the proxy container's or pod's address, not a service address.
3. Set these variables in the server's environment:

   ```sh
   MW_TRUSTED_PROXIES=10.0.0.5        # the proxy's source address or network
   MW_FORWARDED_MODE=xff              # the proxy sends X-Forwarded-For
   MW_PUBLIC_URL=https://mail.example.org
   ```

4. Restart the server. These variables are read at start-up.
5. Sign in through the proxy and open **Diagnostics → Deployment security
   assessment** in the admin console. **HTTPS request** and **Secure cookies** should
   report the HTTPS your browser used. See [Deployment security
   assessment](../admin/deployment-security.md).

If you install with `deploy.sh`, it writes `MW_TRUSTED_PROXIES` and
`MW_FORWARDED_MODE=xff` for you with `--domain` (its own Caddy) and with
`--proxy-network` (an existing proxy on a Docker network), unless `app.env` already
sets them. See [deploy.sh](script.md).

## How the client address is decided

Mailwoman uses the client address for source bans, sign-in rate limits and
geography, API-key IP allowlists, the health-endpoint rule and the addresses it logs.
It decides that address as follows:

- With `MW_FORWARDED_MODE` unset or `off`, the address of the connection is the
  client address. Forwarded headers are ignored.
- With `MW_FORWARDED_MODE=xff` or `forwarded`, the header is read **only** when the
  connection comes from an address in `MW_TRUSTED_PROXIES`. Both variables are
  needed; either one alone changes nothing, and the server logs a warning when a mode
  is set with no usable proxy list.
- Exactly one header is read: `X-Forwarded-For` for `xff`, the RFC 7239 `Forwarded`
  header (its `for=` parameter) for `forwarded`. A client cannot switch to the other
  header.
- The header is read from right to left. Entries that are themselves trusted
  proxies are skipped, and the first entry that is not a trusted proxy is the client.
  Anything to its left is ignored. An entry that is not an address (`unknown`, an
  obfuscated identifier) stops the walk at the last trusted proxy.

`MW_TRUSTED_PROXIES` takes a comma- or space-separated list of addresses and CIDR
networks, IPv4 or IPv6. A malformed entry is dropped; it never widens the list. An
IPv4 client seen by a dual-stack listener as `::ffff:a.b.c.d` matches an IPv4 entry.

::: warning
List only addresses you control. A host inside `MW_TRUSTED_PROXIES` can claim any
client address it likes.
:::

### Single proxy or a chain

The example configurations overwrite `X-Forwarded-For` with the address the proxy
saw, so Mailwoman receives one entry the client could not influence. If you run a
chain (for example a CDN in front of nginx), switch the proxy to appending (each
example file shows how) and list **every** hop in `MW_TRUSTED_PROXIES`. A hop missing
from the list makes the walk stop early, at that hop's address.

## How the scheme and host are decided

Mailwoman needs to know whether the browser used HTTPS (for the `Secure` cookie flag
and the `Strict-Transport-Security` header) and which host it addressed (for the
passkey origin). For the scheme, in order:

1. `MW_PUBLIC_URL`, when it is set: `https://…` means HTTPS, `http://…` means not.
2. `X-Forwarded-Proto` from a connection inside `MW_TRUSTED_PROXIES`. If the header
   has several values, the last one (written by the nearest proxy) is used.
3. Whether Mailwoman terminates TLS itself (`MW_ACME`, or both `MW_TLS_CERT` and
   `MW_TLS_KEY`, set in the environment).

For the host: the host in `MW_PUBLIC_URL`, else `X-Forwarded-Host` from a trusted
proxy, else the `Host` header.

Trust for `X-Forwarded-Proto` and `X-Forwarded-Host` depends on `MW_TRUSTED_PROXIES`
alone, not on `MW_FORWARDED_MODE`. Setting the proxy list with
`MW_FORWARDED_MODE=off` therefore trusts the proxy for scheme and host while the
client address stays the connection address.

### Cookies and HSTS

- The session cookie carries `Secure` when the effective scheme is HTTPS, when
  `--cookie-secure` is passed, or when `MW_PUBLIC_URL` starts with `https://`.
- `MW_COOKIE_SECURE=true` or `false` overrides all of these in both directions.
- `Strict-Transport-Security` is sent only on responses to HTTPS requests, by default
  `max-age=31536000`. Do not add a second one in the proxy; the example
  configurations leave it to Mailwoman.

### Passkeys and the public URL

The passkey (WebAuthn) origin is `MW_WEBAUTHN_ORIGIN` when set, otherwise the
effective scheme and host described above. A passkey only works for the origin it
was registered under.

::: warning
Changing the scheme or host the server sees (by setting or changing `MW_PUBLIC_URL`,
moving to another proxy, or changing `X-Forwarded-Host`) changes the passkey origin.
Passkeys registered under the old origin stop working, and users must sign in with
another factor and register them again. Make the change in a planned window and tell
users first.
:::

## Headers the proxy sends

| Header | What the proxy does | Why |
|---|---|---|
| `Host` | Pass the browser's value unchanged, with the port | Mailwoman compares `Origin` against `Host` on every state-changing request; a rewritten `Host` makes sign-in fail with `403`. |
| `X-Forwarded-For` | Set to the client address (overwrite) | Client address, when `MW_FORWARDED_MODE=xff`. |
| `X-Forwarded-Proto` | Set to `https` or `http` | Effective scheme: `Secure` cookies, HSTS, passkey origin. |
| `X-Forwarded-Host` | Set to the browser's `Host` | Host for the passkey origin when `MW_PUBLIC_URL` is unset. |
| `Forwarded` | Remove the client's copy, unless you use `MW_FORWARDED_MODE=forwarded` | Only one forwarded header should reach the server. |
| `X-CSRF-Token`, cookies | Pass through unchanged | `deploy.sh` turns on strict CSRF (`MW_CSRF_STRICT=true`); a write without the token is refused with `403`. |

## Long-lived connections

| Path | Kind | What the proxy needs |
|---|---|---|
| `/jmap/ws` | WebSocket, push notifications | Upgrade support; idle timeout well above 30 s |
| `/jmap/eventsource` | Server-sent events, push fallback | No buffering, no compression; idle timeout well above 30 s |
| `/api/assist/invoke` | `POST` answered with server-sent events (AI assistant) | No buffering, no compression; long read timeout |
| `/jmap/actions` and `<console path>/actions` | WebSocket that carries web-client and admin-console requests | Upgrade support. Without it the client sends the same requests over ordinary HTTP. |

The server sends a heartbeat on idle push streams every 30 seconds. The SSE responses
carry `X-Accel-Buffering: no` (which nginx honours) and `Cache-Control: no-cache,
no-transform`. Other proxies need buffering turned off in their configuration, as the
examples do. More detail: [Realtime push](websocket.md).

## Request body size

Set the proxy's body limit above Mailwoman's limits (50 MB for `/jmap/upload`,
64 MiB for `/api/import/*`). The example configurations use 100 MB. If the proxy's
limit is lower, an oversized upload gets the proxy's HTML error page instead of
Mailwoman's JSON `413`, and the web client cannot explain it.

When Mailwoman refuses an oversized upload it keeps reading (and discarding) the rest
of the body, up to a further 50 MB or 30 seconds, before it answers `413`. That lets a
proxy that stops relaying on a failed write, such as Apache `mod_proxy`, still deliver
the JSON answer.

## The health endpoint behind a proxy

`/healthz` answers only clients whose address is loopback, private, link-local or
unique-local; everyone else gets `404`, as for an unknown path. The address checked is
the one decided above.

::: warning
If `MW_TRUSTED_PROXIES` and `MW_FORWARDED_MODE` are not set, every request appears to
come from the proxy, which usually has a private address. `/healthz` is then
reachable from the internet. Configure the trusted proxy before you rely on this rule.
:::

To let an external monitor reach `/healthz` through the proxy, either set
`security.health.mode` to `allowlist` and add its network to
`security.health.networks`, or set `MW_HEALTH_TOKEN` and send the value in an
`X-Health-Token` header (or a `token` query parameter). See
[`security.health.mode`](../admin/settings.md#security.health.mode). The container's
own health check (`mailwoman healthcheck`) connects over loopback and sends
`MW_HEALTH_TOKEN` when it is set.

## Header authentication

With `MW_HEADER_AUTH=1` (or `true`), a proxy that has already authenticated the user
can name them in a header (`X-Remote-User` by default, `MW_HEADER_AUTH_HEADER` to
change it). Mailwoman reads the header when the sign-in request arrives and issues a
session without a password; a second factor, if the user has one, is still required.

- Only connections from `MW_HEADER_AUTH_TRUSTED_IPS` may use the header. An empty or
  unparseable list means nobody can, and the server logs a warning.
- `MW_HEADER_AUTH_TRUSTED_IPS` is a separate list from `MW_TRUSTED_PROXIES`, so a
  proxy trusted to report the client address is not automatically trusted to name
  the user.
- The proxy must remove any copy of the header the client sends. **None of the
  example configurations in `proxy/` do this**, because none of them authenticate
  users. Add it to your proxy configuration before you turn header authentication on.

::: danger
With `MW_PROXY_PROTOCOL` set to `accept` or `require`, the connection address is the
one the balancer declares in the PROXY header, and that is the address checked
against `MW_HEADER_AUTH_TRUSTED_IPS`. Any host inside `MW_TRUSTED_PROXIES` can then
declare an address from the header-auth list and sign in as anyone. If you use both
features, list single addresses in `MW_TRUSTED_PROXIES`, not networks.
:::

## PROXY protocol (layer-4 balancers)

When a TCP balancer (HAProxy in `mode tcp`, nginx `stream`, a cloud network load
balancer) passes connections through without terminating TLS, Mailwoman terminates
TLS itself ([ACME or its own certificate](acme.md)) and learns the client address
from a PROXY protocol header.

- `MW_PROXY_PROTOCOL=off` (the default): no header is read.
- `accept`: a peer in `MW_TRUSTED_PROXIES` may send a header; a connection without one
  is served as a direct client.
- `require`: every connection must come from a peer in `MW_TRUSTED_PROXIES` and start
  with a valid header; anything else is dropped. Use this when the port is reachable
  only through the balancer.

A header from a peer outside `MW_TRUSTED_PROXIES` always closes the connection.
Versions 1 and 2 are accepted. Version 2 TLVs, including the CRC32C checksum, are
skipped without being checked. An unrecognised value is treated as `off` and logged.
The balancer's own health checks must send the header too (HAProxy:
`check-send-proxy`). Header timeouts and limits are on [Connection
limits](connection-limits.md). The example is
[`proxy/haproxy-l4/haproxy.cfg`](proxy/README.md).

## Sub-path hosting

`MW_BASE_PATH=/mail` serves the application under `/mail`. The value must be a plain
path (letters, digits, `-`, `_`, `.`, `~` and `/` between segments); anything else is
ignored with a warning and the server stays at the root. `/mail` without a trailing
slash redirects to `/mail/`.

- The application stays reachable at the root as well. `/healthz` and
  `/.well-known/*` must be there for health checks, WKD and JMAP discovery. The prefix
  is routing, not a way to hide the application.
- The proxy may forward the prefix (`proxy_pass http://app;`) or strip it
  (`proxy_pass http://app/;`); both work.
- None of the example configurations in `proxy/` uses a prefix. Their WebSocket and
  stream locations match `/jmap/ws`, `/jmap/eventsource` and `/api/assist/invoke` at
  the root, so add the prefix to each of those matches, and check that push works.

## HTTP/3

Mailwoman does not speak HTTP/3. A proxy can accept HTTP/3 from browsers and talk
HTTP/1.1 to Mailwoman; the forwarded headers work the same way.

| Configuration | HTTP/3 |
|---|---|
| Caddy ([`proxy/caddy/Caddyfile`](proxy/README.md)) | On by default for every TLS site |
| `deploy.sh --domain` (its own Caddy) | On by default; `--http-protocols 'h1 h2'` turns it off. Publishes `443/udp`. See [deploy.sh](script.md). |
| nginx ([`proxy/nginx/mailwoman.conf`](proxy/README.md), [`nginx.conf`](https://github.com/supermarsx/mailwoman/blob/master/docs/deploy/nginx.conf)) | `listen … quic reuseport;`, `http3 on;` and an `Alt-Svc` header. Needs nginx built with `--with-http_v3_module`. |
| Traefik ([`proxy/traefik/traefik.yml`](proxy/README.md)) | `http3: {}` on the `websecure` entry point |
| HAProxy, Apache httpd, Envoy, IIS examples | Not configured |

HTTP/3 runs over **UDP 443**. Open it in the host firewall (for example
`ufw allow 443/udp`), in cloud security groups and on any balancer in front, and
publish it in containers (`"443:443/udp"` next to `"443:443/tcp"`). Browsers start on
TCP, read `Alt-Svc`, and move to HTTP/3 on later requests; if UDP is blocked they
stay on TCP without an error. WebSocket connections use TCP, so the TCP listener and
its WebSocket settings remain necessary.

nginx does not create a `Host` header for HTTP/3 requests, so `$http_host` is empty
there. The example vhost sends `$mw_host` (a `map` that falls back to `$host` plus the
port) so that sign-in over HTTP/3 keeps a `Host` to compare `Origin` against.

To check a deployment:

```sh
# The TCP response must advertise HTTP/3:
curl -sI --http2 https://mail.example.org/healthz | grep -i '^alt-svc'
# HTTP/3 only; needs a curl with HTTP3 in `curl -V`. Prints 3 on success:
curl -s -o /dev/null -w '%{http_version}\n' --http3-only https://mail.example.org/healthz
```

Both requests reach `/healthz`, so run them from a network the health rule admits, or
add `-H "X-Health-Token: …"`.

## Tested configurations

The `proxy-conformance` workflow (`.github/workflows/proxy-conformance.yml`) boots each
example configuration from `docker-compose.proxy.yml` in front of a Mailwoman server
with strict CSRF and runs the test suite `crates/mw-server/tests/t20_proxy_conformance.rs`
through it.

| Proxy | Example | In the workflow |
|---|---|---|
| nginx | `proxy/nginx/` | Yes, blocking |
| Apache httpd | `proxy/apache/` | Yes, blocking |
| Caddy | `proxy/caddy/` | Yes, blocking |
| HAProxy, HTTP mode | `proxy/haproxy-l7/` | Yes, blocking |
| Traefik | `proxy/traefik/` | Yes, blocking |
| HAProxy, TCP mode with PROXY protocol | `proxy/haproxy-l4/` | Yes, non-blocking |
| Envoy | `proxy/envoy/` | Yes, non-blocking |
| IIS with ARR | `proxy/iis/` | No. Documented, not CI-tested. See [IIS](proxy/iis/README.md). |

A non-blocking cell runs the suite, but its failure does not fail the workflow. The
suite checks the client address, rejection of a client-supplied forwarded header,
SSE streaming, the WebSocket upgrade and `jmap` subprotocol, large and oversized
uploads, range requests, cache headers, `Secure` cookies and HSTS behind TLS, and
strict CSRF. Two tests do not run in every cell: the appending-proxy test runs only
in the Envoy cell (the only example that appends to `X-Forwarded-For`), and the
sub-path test runs in no cell, because no example uses a prefix. The workflow's
non-blocking `h3` job runs `scripts/proxy-h3-check.sh` against the Caddy, nginx and
Traefik cells.

Separately, the `ci.yml` workflow runs `scripts/test-deploy-proxy.sh`, which installs
Mailwoman with `deploy.sh` behind Nginx Proxy Manager and behind its own Caddy
(issuing a certificate from a local ACME test server), and checks sign-in, strict
CSRF, the `/jmap/ws` upgrade and the logged client address through each.

## Options

| Variable | What it does | Default |
|---|---|---|
| `MW_TRUSTED_PROXIES` | Addresses and CIDR networks whose forwarded headers and PROXY headers are believed | empty (nobody) |
| `MW_FORWARDED_MODE` | Which header carries the client address: `off`, `xff` (`X-Forwarded-For`), `forwarded` (RFC 7239). Any other value is `off`. | `off` |
| `MW_PROXY_PROTOCOL` | PROXY protocol on the listener: `off`, `accept`, `require` | `off` |
| `MW_PUBLIC_URL` | The public base URL, for example `https://mail.example.org`. Sets the scheme and host; a path in it is ignored (use `MW_BASE_PATH`). | unset |
| `MW_BASE_PATH` | Serve under a path prefix such as `/mail` | unset (root) |
| `MW_COOKIE_SECURE` | `true` or `false`: forces the `Secure` cookie flag either way | unset (derived) |
| `MW_HSTS` | `off`, `0` or `false` stops the `Strict-Transport-Security` header | on |
| `MW_HSTS_MAX_AGE` | HSTS `max-age` in seconds | `31536000` |
| `MW_HSTS_INCLUDE_SUBDOMAINS` | `1` or `true` adds `includeSubDomains` | off |
| `MW_HSTS_PRELOAD` | `1` or `true` adds `preload` | off |
| `MW_HEALTH_TOKEN` | Token that admits a `/healthz` request from any address | unset |
| `MW_HEADER_AUTH` | `1` or `true` turns on header authentication | off |
| `MW_HEADER_AUTH_HEADER` | Header that names the user | `X-Remote-User` |
| `MW_HEADER_AUTH_TRUSTED_IPS` | Addresses allowed to use the header | empty (nobody) |
| `MW_HEADER_AUTH_PASSWORD` | Password Mailwoman uses for the mail backend for header-authenticated users | empty |
| `MW_HEADER_AUTH_JMAP_URL` | JMAP server for header-authenticated users in proxy mode | the configured upstream |
| `MW_WEBAUTHN_ORIGIN`, `MW_WEBAUTHN_RP_ID` | Fix the passkey origin and relying-party ID instead of deriving them | unset |

All of them are read at start-up; restart the server after a change.

## Troubleshooting

**Logs, bans and API-key allowlists show the proxy's address for every request.**
`MW_TRUSTED_PROXIES` or `MW_FORWARDED_MODE` is missing, or the proxy connects from an
address the list does not contain. Set both, using the address the connection
actually comes from.

**Sign-in fails with `403` through the proxy but works directly.** The proxy changes
`Host` (Apache without `ProxyPreserveHost On`, IIS without `preserveHostHeader`,
nginx with `proxy_set_header Host $host` on a non-default port), so it does not
match the browser's `Origin`. Pass `Host` through unchanged. Over HTTP/3 with
nginx, use the `$mw_host` map from the example.

**Session cookie has no `Secure` flag behind a TLS proxy.** The proxy is not in
`MW_TRUSTED_PROXIES`, or it does not send `X-Forwarded-Proto: https`. Fix either, or
set `MW_PUBLIC_URL=https://…`.

**Push arrives late or never; the client falls back to polling.** The proxy drops the
WebSocket upgrade on `/jmap/ws` or buffers `/jmap/eventsource`. See
[Realtime push](websocket.md#troubleshooting).

**An oversized upload shows an HTML error page instead of a message.** The proxy's
body limit is below Mailwoman's. Raise it above 64 MiB.

**`/healthz` answers from the internet.** Forwarded headers are not trusted, so every
request seems to come from the proxy's private address. Configure the trusted proxy.

**Every connection is closed immediately after enabling `MW_PROXY_PROTOCOL=require`.**
The balancer's address is not in `MW_TRUSTED_PROXIES`, or the balancer (or its
health check) does not send a PROXY header.

## Related

- [Example proxy configurations](proxy/README.md)
- [Realtime push behind a proxy](websocket.md)
- [TLS: ACME and external certificates](acme.md)
- [Connection limits](connection-limits.md)
- [Deployment security assessment](../admin/deployment-security.md)
