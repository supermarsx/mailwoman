# Connection and HTTP request limits

Mailwoman's built-in listener has limits on TLS handshakes, PROXY protocol headers,
connections, requests and request headers. This page lists them, their defaults and
accepted ranges, how the connection, request and HTTP/2 stream limits are derived
when you do not set them, and what happens when a limit is reached. The limits apply
only to Mailwoman's own listener; a reverse proxy in front has its own.

## Before you start

- Every variable on this page is read once at start-up. Restart the server after a
  change. The connection, request, HTTP/2 stream, wait and adaptive values can also
  be set in the admin console (see [In the admin console](#in-the-admin-console));
  a set variable wins over the console value.
- An unset variable uses its default. An empty, non-numeric or out-of-range value
  stops the server from starting, with an error naming the variable and its range.
- To see the values in force, open the admin console. The **Effective
  configuration** page lists each variable. Under **Diagnostics → Deployment security
  assessment**, the checks **Inbound connection negotiation limits** and **HTTP
  admission and request limits** summarise them. See [Deployment security
  assessment](../admin/deployment-security.md).

![The HTTP admission and request limits and Inbound connection negotiation limits rows of the Deployment security assessment, listing the default limits.](/screenshots/deploy/connection-limits-assessment.webp){.mw-crop}

## TLS handshakes and PROXY headers

| Variable | What it limits | Default | Range |
|---|---|---:|---:|
| `MW_TLS_HANDSHAKE_TIMEOUT_MS` | Time allowed for one TLS handshake (own certificate or ACME) | 10000 | 1000–60000 |
| `MW_TLS_MAX_HANDSHAKES` | TLS handshakes in progress at once | 128 | 1–1024 |
| `MW_PROXY_HEADER_TIMEOUT_MS` | Time allowed for a connection to deliver its PROXY protocol header | 5000 | 1000–30000 |
| `MW_PROXY_MAX_HANDSHAKES` | PROXY headers being read at once | 128 | 1–1024 |

- A handshake that does not finish before its deadline is dropped and frees its slot.
- When every slot is in use, the listener stops accepting new connections until a
  slot frees. Connections that arrive meanwhile wait in the operating system's
  backlog.
- The PROXY limits apply only when `MW_PROXY_PROTOCOL` is `accept` or `require`. The
  PROXY header is read, and its sender checked against `MW_TRUSTED_PROXIES`, before
  the TLS handshake or the first HTTP request. See [PROXY
  protocol](reverse-proxy.md#proxy-protocol-layer-4-balancers).

## HTTP connections and requests

These apply to HTTP/1.1 and HTTP/2, with or without TLS.

| Variable | What it limits | Default | Range |
|---|---|---:|---:|
| `MW_HTTP_MAX_CONNECTIONS` | Open connections, including upgraded WebSockets | derived, at least 1024 | 16–65536 |
| `MW_HTTP_MAX_REQUESTS` | Requests in progress at once, each counted until its response body is sent | derived, at least 256 | 1–16384 |
| `MW_HTTP_MAX_HEADER_BYTES` | Total size of a request's headers | 32768 | 8192–65536 |
| `MW_HTTP_MAX_HEADERS` | Number of request headers | 100 | 16–256 |
| `MW_HTTP_HEADER_TIMEOUT_MS` | Time to open the connection and send each HTTP/1 request's headers | 10000 | 1000–60000 |
| `MW_HTTP_BODY_TIMEOUT_MS` | Time to receive a request body | 60000 | 1000–300000 |
| `MW_HTTP2_MAX_STREAMS` | Concurrent HTTP/2 streams on one connection | derived, 32–100 | 1–128 |
| `MW_HTTP_SHUTDOWN_TIMEOUT_MS` | Time requests in progress get to finish when the server stops | 30000 | 1000–120000 |
| `MW_HTTP_REQUEST_QUEUE_MS` | How long a request at the request limit waits for a slot; `0` refuses at once | 200 | 0–2000 |
| `MW_HTTP_REQUEST_QUEUE_MAX` | Requests that may wait for a slot at once | the request limit | 1–65536 |
| `MW_HTTP_ADAPTIVE_CONCURRENCY` | `on` lowers the request limit while requests slow down (see below) | `off` | `off`, `on` |
| `MW_HTTP_RAISE_OPEN_FILES` | `off` keeps the soft open-file limit as it is at start; `on` raises it to the hard limit, at most 1,048,576, when `MW_HTTP_MAX_CONNECTIONS` is unset. The hard limit is never changed | `on` | `on`, `off` |

### Derived limits

When `MW_HTTP_MAX_CONNECTIONS`, `MW_HTTP_MAX_REQUESTS` or `MW_HTTP2_MAX_STREAMS` is
unset, the server derives it at start-up from what the process may use. A derived
value is never lower than the fixed value of earlier releases (1024, 256, 32).

| Reading | Source |
|---|---|
| CPUs (C) | CPUs the process may run on; on Linux this applies the container's CPU quota |
| Memory (M) | Linux only: the lower of the cgroup memory limit and physical memory |
| Open files (F) | Unix only: the soft `RLIMIT_NOFILE`. When `MW_HTTP_MAX_CONNECTIONS` is unset and `MW_HTTP_RAISE_OPEN_FILES` is not `off`, the server first raises the soft limit to the hard limit, at most 1,048,576, and logs the old and new values |
| Database pool (P) | The `MW_DB_POOL` maximum, else the store's default (PostgreSQL 4 per CPU, 4–64; SQLite 2 per CPU, 2–8). A maximum set only in the stored `database.pool` setting is not seen |
| Handshake slots (H) | `MW_TLS_MAX_HANDSHAKES` + `MW_PROXY_MAX_HANDSHAKES` |

- **Connections** = max(1024, min((F − R) / 2, M / 1 MiB, 65536)), with a
  descriptor reserve R = 512 + P + H. Half of the descriptors left after the reserve
  go to client connections; the other half stay for upstream IMAP and SMTP
  connections, outbound HTTP and files.
- **Requests** = max(64 × C, 16 × P), at least 256, at most the lower of 16384 and
  M / 4 MiB.
- **HTTP/2 streams per connection** = an eighth of the request limit, from 32 to 100.
  One connection can then hold at most an eighth of the request slots.
- **Queue size** = the request limit.

A reading the platform does not provide (for example memory and open files on
Windows) is left out of the formula; with no readings the fixed values apply. If the
open-file limit is too low for 1024 connections, the server keeps 1024 and logs a
warning; raise the limit (`ulimit -n`, `LimitNOFILE=` in systemd) or set
`MW_HTTP_MAX_CONNECTIONS` lower.

At start-up the server logs one line, `HTTP limits: …`, with each effective value,
whether it was set, derived or fixed, each reading used (or `unknown (term dropped)`)
and what happened to the open-file limit. When the floor decided a derived value
because descriptors or memory were too small, a warning is logged too. The **HTTP admission and
request limits** check in the admin console shows the same values and the admission
counters.

To keep the values of earlier releases, set them explicitly:

```sh
MW_HTTP_MAX_CONNECTIONS=1024
MW_HTTP_MAX_REQUESTS=256
MW_HTTP2_MAX_STREAMS=32
MW_HTTP_REQUEST_QUEUE_MS=0
```

What happens at each limit:

- **Too many connections.** A connection beyond `MW_HTTP_MAX_CONNECTIONS` is closed
  before any request on it is read. A WebSocket counts until its connection closes.
- **Too many requests.** A request beyond `MW_HTTP_MAX_REQUESTS` waits, in order of
  arrival, for up to `MW_HTTP_REQUEST_QUEUE_MS` for a slot. It is answered `503` with
  `Retry-After: 1` when the wait ends without a slot, or at once when
  `MW_HTTP_REQUEST_QUEUE_MAX` requests are already waiting. A client that disconnects
  while waiting gives up its place. The wait does not count towards
  `MW_HTTP_BODY_TIMEOUT_MS`. Event streams on `/jmap/eventsource` do not count here;
  they share a separate pool the size of `MW_HTTP_MAX_CONNECTIONS` and do not wait.
  `/healthz` and `/readyz` have their own pool of 8 and do not wait, so a full request
  pool does not delay them. A WebSocket counts as a request only until its `101`
  response.
- **Adaptive concurrency (off by default).** With
  `MW_HTTP_ADAPTIVE_CONCURRENCY=on`, the server measures how long each request
  takes until its response headers are ready. When the slowest tenth of the last 64
  requests take more than twice the usual time (and at least 2 ms more), it lowers
  the request limit by a tenth; while requests are fast and the limit is in use, it
  raises it by a twentieth. The limit stays between max(256, 16 × CPUs) and
  `MW_HTTP_MAX_REQUESTS`, never above it. Requests over the lowered limit wait and
  are refused as above. Downloads, uploads, exports, imports, attachment previews,
  the image proxy, DAV feeds, mail refresh, WebSocket upgrades, event streams and
  probes are not measured, because their time follows their size or a remote
  server. The current limit is shown under `admission.adaptive` in the HTTP
  admission check.
- **Headers too large or too many.** The request is answered `431` before it reaches
  the application.
- **Slow request body.** A body that does not arrive within
  `MW_HTTP_BODY_TIMEOUT_MS` is answered `408`. The deadline applies to what the client
  sends, not to responses, so an idle event stream or WebSocket is not closed by it.
  When an HTTP/1 request body is left unread, the server closes the connection rather
  than reading the rest.
- **Shutdown.** On a stop signal the server stops accepting new connections and gives
  requests in progress up to `MW_HTTP_SHUTDOWN_TIMEOUT_MS` to finish, then closes the
  rest, WebSockets included. When `MW_HTTP_SHUTDOWN_TIMEOUT_MS` is unset, the
  [`server.shutdownGraceSeconds`](../admin/settings.md#server.shutdownGraceSeconds)
  setting (1–120 seconds, default 30) decides instead.

HTTP/2 connections also have a fixed header-list size (`MW_HTTP_MAX_HEADER_BYTES`),
fixed flow-control windows, and a keep-alive ping every 15 seconds that closes the
connection after 10 seconds without an answer.

Endpoint-specific body limits still apply on top of these, for example 50 MB for an
attachment upload. The listener does not speak HTTP/3; a proxy in front can (see
[HTTP/3](reverse-proxy.md#http-3)).

::: warning
These limits do not protect against floods of malformed packets or distributed
denial of service. Keep the operating system's and the front proxy's own connection
limits in place.
:::

## In the admin console

**Settings → Maintenance** has the `server.http.*` keys: maximum connections,
maximum requests at once, HTTP/2 streams per connection, the wait for a free slot,
the requests that may wait, the adaptive request limit and its floor, and whether
to raise the open-file limit. Above them, **HTTP limits in force** shows what each
value resolved to on this server process, whether it is Auto, set in the console
or set by the environment, and the readings Auto used.

- **Precedence.** A set environment variable, then the console value, then Auto.
  A key whose variable is set cannot be saved in the console.
- **When a change applies.** Within about a second, without a restart: the request
  limit grows at once and shrinks as requests finish (none is cut off); the wait and
  queue values apply to the next request; the connection and HTTP/2 stream limits
  apply to new connections. Raising the open-file limit happens only at start.
- **Safety bounds.** A console value above what this machine allows is held to it:
  connections to (F − R) / 2 (at least 1024) when the open-file limit is known,
  requests to M / 4 MiB (256 to 16384) when memory is known. The readout says when
  that happened. An out-of-range value is refused with the field named.
- **The console stays reachable.** Requests under the admin path take a free request
  slot, else one of 8 slots kept for the console, before they wait. `/healthz` and
  `/readyz` have their own 8. So a save of the smallest values can be undone from
  the console.
- **Several processes.** Each server process applies the values on its own; with N
  processes behind a balancer the deployment admits up to N times the value.

## Related

- [Running behind a reverse proxy](reverse-proxy.md)
- [Realtime push behind a proxy](websocket.md)
- [TLS: ACME and external certificates](acme.md)
- [Deployment security assessment](../admin/deployment-security.md)
