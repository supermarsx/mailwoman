# Connection and HTTP request limits

Mailwoman's built-in listener has fixed limits on TLS handshakes, PROXY protocol
headers, connections, requests and request headers. This page lists them, their
defaults and accepted ranges, and what happens when a limit is reached. The limits
apply only to Mailwoman's own listener; a reverse proxy in front has its own.

## Before you start

- Every variable on this page is read once at start-up. Restart the server after a
  change; they cannot be changed from the admin console.
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
| `MW_HTTP_MAX_CONNECTIONS` | Open connections, including upgraded WebSockets | 1024 | 16–8192 |
| `MW_HTTP_MAX_REQUESTS` | Requests in progress at once, each counted until its response body is sent | 256 | 1–4096 |
| `MW_HTTP_MAX_HEADER_BYTES` | Total size of a request's headers | 32768 | 8192–65536 |
| `MW_HTTP_MAX_HEADERS` | Number of request headers | 100 | 16–256 |
| `MW_HTTP_HEADER_TIMEOUT_MS` | Time to open the connection and send each HTTP/1 request's headers | 10000 | 1000–60000 |
| `MW_HTTP_BODY_TIMEOUT_MS` | Time to receive a request body | 60000 | 1000–300000 |
| `MW_HTTP2_MAX_STREAMS` | Concurrent HTTP/2 streams on one connection | 32 | 1–128 |
| `MW_HTTP_SHUTDOWN_TIMEOUT_MS` | Time requests in progress get to finish when the server stops | 30000 | 1000–120000 |

What happens at each limit:

- **Too many connections.** A connection beyond `MW_HTTP_MAX_CONNECTIONS` is closed
  before any request on it is read. A WebSocket counts until its connection closes.
- **Too many requests.** A request beyond `MW_HTTP_MAX_REQUESTS` is answered `503`
  with `Retry-After: 1`. Event streams on `/jmap/eventsource` do not count here; they
  share a separate pool the size of `MW_HTTP_MAX_CONNECTIONS`. Nothing is queued.
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

## Related

- [Running behind a reverse proxy](reverse-proxy.md)
- [Realtime push behind a proxy](websocket.md)
- [TLS: ACME and external certificates](acme.md)
- [Deployment security assessment](../admin/deployment-security.md)
