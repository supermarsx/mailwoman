# Connection and HTTP request limits

The built-in listener bounds TLS handshakes and PROXY protocol header negotiation. These are startup settings: changing a value requires restarting the process. The configuration catalogue and Diagnostics → Deployment security show the configured values. They do not measure limits at an external reverse proxy. Writing these startup settings from the administrator UI is not implemented.

| Environment variable | Default | Accepted range | Applies to |
| --- | ---: | ---: | --- |
| `MW_TLS_HANDSHAKE_TIMEOUT_MS` | 10000 | 1000–60000 | Complete external-certificate or ACME TLS handshake |
| `MW_TLS_MAX_HANDSHAKES` | 128 | 1–1024 | Concurrent handshakes inside the TLS listener |
| `MW_PROXY_HEADER_TIMEOUT_MS` | 5000 | 1000–30000 | Complete PROXY header classification and parsing |
| `MW_PROXY_MAX_HANDSHAKES` | 128 | 1–1024 | Concurrent PROXY header negotiations |

Unset variables use the defaults. Empty, malformed or out-of-range values refuse listener startup. These limits are fixed; no adaptive increase is enabled.

A silent TLS peer occupies one bounded handshake slot until its deadline. Other available slots continue to accept handshakes. When every slot is occupied, admission pauses until a slot completes or expires; the kernel TCP backlog remains subject to operating-system settings. The listener owns pending handshake futures, so dropping it closes their sockets. PROXY negotiation uses a bounded task set and a bounded completed-connection queue; shutdown aborts those tasks too. PROXY trust and required-header checks still apply before TLS or HTTP.

These bounds do not constitute protection against malformed TCP/IP packets, distributed denial of service, or an external proxy with different limits. HTTP request-header/body deadlines and HTTP/2 stream budgets below apply separately; the handshake limits do not replace them. Keep operating-system and front-proxy admission controls in place.


## HTTP admission and ingress

The built-in HTTP/1.1 and HTTP/2 server uses these fixed startup bounds. They are reported as configuration in the administrator catalogue and deployment assessment; writing them from the administrator UI remains pending.

| Environment variable | Default | Accepted range |
| --- | ---: | ---: |
| `MW_HTTP_MAX_CONNECTIONS` | 1024 | 16–8192 |
| `MW_HTTP_MAX_REQUESTS` | 256 | 1–4096 |
| `MW_HTTP_MAX_HEADER_BYTES` | 32768 | 8192–65536 |
| `MW_HTTP_MAX_HEADERS` | 100 | 16–256 |
| `MW_HTTP_HEADER_TIMEOUT_MS` | 10000 | 1000–60000 |
| `MW_HTTP_BODY_TIMEOUT_MS` | 60000 | 1000–300000 |
| `MW_HTTP2_MAX_STREAMS` | 32 | 1–128 |
| `MW_HTTP_SHUTDOWN_TIMEOUT_MS` | 30000 | 1000–120000 |

The connection limit includes upgraded WebSockets until their underlying connection closes. The request limit covers a response through body completion, so active server-sent event streams count toward it. Request-capacity refusal returns503 with `Retry-After: 1`; it is an overload response, not evidence of malicious behavior. Excess accepted connections are closed before entering application handlers. Queued work is not accumulated behind either limit.

The header deadline covers the initial protocol opening and each HTTP/1 request header. HTTP/2 has a bounded header list, a per-connection stream limit, fixed flow-control windows and keepalive checks. Oversized headers are rejected before an application handler. A request-body deadline applies to incoming data, not the outgoing response: an idle event stream or WebSocket is not closed by that deadline. An abandoned HTTP/1 request body closes keepalive rather than draining a slow body indefinitely. Existing endpoint-specific body byte limits still apply.

On shutdown, new admission stops and active HTTP requests may finish within the drain deadline. Remaining work and upgraded IO are then canceled. These bounds do not perform adaptive tuning, identify attackers, or replace deployment-level network controls. HTTP/3 termination is a separate reverse-proxy/deployment capability, not a protocol advertised by this listener.
