# HTTP protocol guard

The server runs an early middleware, the protocol guard, that checks the method, version, request target and headers of every request before routing, sign-in or body parsing. It exists for the cases where two HTTP parsers (a front proxy and this server) could read the same bytes differently, and for requests that are malformed in ways no client of this application produces.

It is configured under `security.protocol.*` (see [settings](settings.md#security.protocol.framingMode)), takes effect within seconds of a saved change, and its counters are on the Adaptive protection page (`GET {console}/bans/protection`, field `protocolGuard`).

## Modes and what each class does

Every class has a mode: `off`, `detect` (log, count, send a typed signal to adaptive protection, allow the request) or `enforce` (reject).

| Class (`*Mode`) | Checks | Default | Reject |
| --- | --- | --- | --- |
| `framing` | `Content-Length` with `Transfer-Encoding`; `Transfer-Encoding` other than exactly `chunked`, or on HTTP/1.0 or HTTP/2; non-numeric, signed, overflowing or conflicting `Content-Length` | enforce | 400, `Connection: close` |
| `host` | `Host` missing on HTTP/1.1, empty, repeated, or holding userinfo, whitespace, path characters or a bad port (IPv6 literals, ports and a trailing dot are fine) | enforce | 400, `Connection: close` |
| `pathControl` | NUL, control characters, `%00`, `%0d`, `%0a` in the path (the query is not checked) | enforce | 400, `Connection: close` |
| `hostPinning` | `Host` not matching `MW_PUBLIC_URL` or `allowedHosts` (no effect while neither names a host; loopback names from a loopback peer are accepted) | detect | 400 |
| `target` | `%2e`, double encoding (`%252e`), overlong UTF-8, dot segments, `//`, backslash or `%5c`, raw non-ASCII bytes, empty path, absolute-form target naming another authority than `Host` | detect | 400 |
| `pathLength` | path over `maxPathBytes`, query over `maxQueryBytes` | detect | 414 |
| `method` | method outside `allowedMethods` (TRACE, TRACK, CONNECT, unknown and lower-case methods) | detect | 405 with `Allow` |
| `duplicateHeaders` | repeated `Content-Type`, `Authorization`, `Origin`, `Referer`, `User-Agent`, `Sec-Fetch-*`, conditional and range headers; repeated equal `Content-Length`; repeated `Cookie` on HTTP/1.x | detect | 400 |
| `forwarded` | `Forwarded`, `X-Forwarded-*`, `X-Real-IP` from a peer outside `MW_TRUSTED_PROXIES` | detect | 400 |
| `headerHygiene` | header count, single value or `Cookie` size over the caps; non-ASCII header bytes; `Upgrade` or `Connection: upgrade` outside `/jmap/ws`; `Connection` naming `Content-Length`, `Host` and similar | detect | 431 for size and count, 400 otherwise |
| `body` | GET or HEAD with a body; `Content-Length` over `maxBodyBytes`; `Expect` other than `100-continue`, or without a body | detect | 413 / 417 / 400 |

Why these defaults: the enforced classes are the ones where the request is either ambiguous about where it ends (request smuggling) or syntactically invalid, and no browser, mail client or API client in this project sends them. The detect classes are heuristics that a legitimate deployment can trip: a proxy that does not set `Host` the way the public URL says, an internal health check, an unusual but valid path, a CDN adding forwarding headers before you have configured `MW_TRUSTED_PROXIES`. Move a class to `enforce` once the counters have shown no false positives for your traffic.

What is checked on `/healthz` and `/readyz`: the same, but an enforced class is downgraded to detect, so a probe is never refused by this guard.

### What a rejection and a detection reveal

A rejection is a small JSON error in the same shape as the other error pages, `Cache-Control: no-store`, with no path, query or header value in it. `Connection: close` is added for the framing, Host and pathControl classes on HTTP/1.x. A detection writes one structured log line (target `protocol_guard`) with the class, a fixed code, the action, the source address (from the trusted-proxy authority, `scope_mw::proxy::client_ip`) and a coarse path class (`admin`, `jmap`, `dav`, `api`, `other`). It never contains a header value, cookie, token or query. The audit log gets at most one `protocol-anomaly` event per class and source per minute, so a scanner cannot use it to flood the log; the counters on the page are exact.

### Adaptive protection

Every finding emits one typed `protocol` signal for the source address into the same pipeline as the probe-path and malformed-request signals, so repeated offenders reach the existing throttle and block decisions. It counts in the rolling windows whatever the weak-signal selection says, but is never a one-request block. Nothing happens while adaptive protection is `off` (its default). The `forwarded` class sends no signal unless `MW_TRUSTED_PROXIES` is set: without it every request from a proxy would look like an untrusted peer and the proxy's own address would be charged.

### Exemptions

`security.protocol.exemptPrefixes` lists route prefixes (each starting with `/` and longer than one character, matched on whole path segments) where the heuristic classes do not run, for example a route a legitimate client uploads to. It cannot exempt `framing` or `pathControl`, and `/` is ignored. There is no global off switch beyond a class's own mode.

### Requests with no peer

A request that reaches the router without a peer address (only the in-process test transport does) is passed through untouched, the same convention as the source-ban guard. The production listener always supplies one.

## What the framework already does

Ground truth, measured by `crates/mw-server/tests/protocol_ground_truth.rs`: each vector is sent as raw bytes over TCP to the real listener (`inbound_http::serve`, hyper 1.x with the settings in `HttpLimits`), against a handler that records what it received. "Reaches" means the guard (and any middleware) sees it.

| Vector (HTTP/1.1 unless stated) | Framework result | Guard |
| --- | --- | --- |
| `Content-Length` + `Transfer-Encoding: chunked` | reaches; both headers are kept, hyper frames the body as chunked | `framing` |
| TE.CL with a pipelined second request inside the chunk | one request reaches; the embedded request is body bytes and never a second request | `framing` |
| duplicate `Content-Length`, differing values | 400 | n/a |
| duplicate `Content-Length`, identical values (two header lines) | reaches | `duplicateHeaders` |
| `Content-Length: 5, 6` and `Content-Length: 5, 5` | 400 | n/a |
| negative, `+5`, non-numeric, overflowing `Content-Length` | 400 | n/a |
| `Transfer-Encoding: xchunked` | 400 | n/a |
| `Transfer-Encoding: chunked, identity` | 400 | n/a |
| `identity, chunked`; `gzip, chunked`; `chunked` on two header lines | reaches | `framing` |
| tab or spaces around the `chunked` value | normalised to `chunked`; reaches as an ordinary chunked request | none needed |
| `Transfer-Encoding : chunked` (space before the colon) | 400 | n/a |
| obs-fold continuation line, in any header | 400 | n/a |
| header name with a space before the colon | 400 | n/a |
| `Transfer-Encoding` on HTTP/1.0 | 400 | n/a |
| bare LF line endings | reaches (hyper accepts LF-only) | not visible |
| bare CR in a header value | 400 | n/a |
| NUL, 0x01 or DEL in a header value | 400 | n/a |
| NUL, space or tab in the request target | 400 | n/a |
| raw non-ASCII (UTF-8) bytes in the request target | reaches unchanged | `target` |
| `#fragment` in the request target | hyper drops it; the application sees the path without it | not visible |
| UTF-8 or invalid UTF-8 bytes in a header value | reaches | `headerHygiene` (opaque bytes) |
| absolute-form target naming another authority than `Host` | reaches | `target` |
| HTTP/0.9 request line, `HTTP/1.1x`, `HTTP/2.0` or `HTTP/9.9` on an HTTP/1 connection | 400 | n/a |
| request line of 40 KiB, or heads over `MW_HTTP_MAX_HEADER_BYTES` | 431 | n/a |
| more than `MW_HTTP_MAX_HEADERS` headers | 431 | n/a |
| duplicate, empty, userinfo-carrying or missing `Host` | reaches | `host` |
| TRACE, CONNECT, unknown or lower-case method | reaches | `method` |
| `Expect: 100-continue` (hyper answers `100 Continue`) or an unknown `Expect` | reaches | `body` |
| GET with a `Content-Length` body | reaches | `body` |
| `%00`, `%2e%2e`, `//`, backslash, dot segments in the path | reaches unchanged | `pathControl`, `target` |
| pipelined, legitimate requests | each reaches | n/a |

HTTP/2 (plaintext prior-knowledge frames built by hand in the same test; the listener serves h2c and TLS ALPN `h2`):

| Vector | h2 stack result | Guard |
| --- | --- | --- |
| uppercase header name; CRLF or NUL inside a header value | connection error (GOAWAY) | n/a |
| `transfer-encoding: chunked`, `connection: keep-alive`, `te: gzip` | stream reset (RST_STREAM) | n/a |
| `te: trailers` | reaches | none needed |
| `content-length: 5` with no body | stream reset | n/a |
| pseudo-header after a regular header; repeated `:path`; NUL in `:path` | stream reset | n/a |
| `host` header differing from `:authority` | reaches; the request URI carries the `:authority`, the `Host` header carries the other name | `target` (authority-mismatch) |
| missing `:path` | reaches with an empty path | `target` (path-empty) |
| `:authority` without a `Host` header | reaches with no `Host` header; the authority is in the URI | `host` accepts it |

Everything else the h2 stack hands on is checked like HTTP/1.1. HTTP/2 has no request smuggling through message framing, but a front proxy that downgrades HTTP/2 to HTTP/1.1 can reintroduce it (CRLF inside a header value, a `content-length` that disagrees with the frames); that downgrade is the proxy's to refuse, and the guard only sees the downgraded request.

### What application code cannot enforce

* Anything hyper has already normalised or dropped is gone: the guard cannot report an obs-fold line or a NUL byte in a header that hyper refuses, only count that the connection ended in a protocol error (logged at debug level by the listener).
* Bare LF line endings are accepted by hyper and arrive as ordinary requests. Only a front proxy that rejects them can refuse them.
* The raw request line and the order and exact spelling of headers are not preserved, so duplicate `Host` and differing-case duplicates are seen only as header-map entries.
* A front proxy that rewrites the request (re-chunks, merges headers, normalises paths) hides the client's original bytes from every check here. The guard sees what the proxy chose to send. Request smuggling between proxy and server is prevented by the proxy's own parser, not by this guard; the guard is the second line.
* The guard cannot see TLS or HTTP/2 frame-level behaviour (rapid reset, continuation floods). Hyper's HTTP/2 settings (`max_concurrent_streams`, header list size) and the proxy cover those.
* Client addresses in forwarding headers are never trusted here, only inspected; address trust stays with `MW_TRUSTED_PROXIES`.

## Settings for the reverse proxy

Terminate and normalise at the edge, then let the proxy open a clean HTTP/1.1 connection to the server. Directive names below are the ones to look up; behaviours differ between releases, so confirm each with a raw request (the vectors in `protocol_ground_truth.rs` can be sent with `nc` or `openssl s_client`) after any upgrade.

* Reject duplicate or conflicting `Host` and `Content-Length`, and `Content-Length` with `Transfer-Encoding`; accept `Transfer-Encoding: chunked` only; use HTTP/1.1 to the backend with the proxy re-framing the body.
* Overwrite, never append, the forwarding headers a client could set (`X-Forwarded-For`, `X-Forwarded-Proto`, `X-Forwarded-Host`, `Forwarded`, `X-Real-IP`), and put the proxy's address in `MW_TRUSTED_PROXIES`.
* Pass the public `Host`, and set `MW_PUBLIC_URL`.
* Cap request line, header count and size, and body size to match or undercut the server's (`MW_HTTP_MAX_HEADER_BYTES` 32 KiB, `MW_HTTP_MAX_HEADERS` 100, `maxBodyBytes`).
* Merge slashes and refuse encoded slashes and NUL in the path.

nginx: keep `ignore_invalid_headers on` and `underscores_in_headers off`; `large_client_header_buffers 4 8k;`, `client_header_buffer_size 4k;`, `client_max_body_size`; `merge_slashes on`; `proxy_http_version 1.1;`; `proxy_set_header Host $host;`; `proxy_set_header X-Forwarded-For $remote_addr;` (replace, not `$proxy_add_x_forwarded_for`, at the edge); `proxy_set_header Forwarded "";`; a `default_server` that answers `444` so unknown Host values never reach the application.

Caddy: `servers { max_header_size 16KB; trusted_proxies static <cidr> }`; `request_body { max_size 100MB }`; `header_up -Forwarded`; a host matcher with `respond 421` for names other than the site; Caddy's reverse proxy sets `X-Forwarded-*` itself and drops client values from untrusted sources.

Traefik (v3): entry point `forwardedHeaders.trustedIPs` (leave empty at the edge so client values are overwritten); `http.sanitizePath: true` (default); `http.encodedCharacters` set to refuse encoded slash, backslash, null and percent; `transport.respondingTimeouts`; `http.maxHeaderBytes` where available; a `headers` middleware to blank `Forwarded` and `X-Real-IP`.

Apache httpd: `HttpProtocolOptions Strict` (default); `LimitRequestFields`, `LimitRequestFieldSize`, `LimitRequestLine`, `LimitRequestBody`; `AllowEncodedSlashes Off`; `MergeSlashes On`; `RequestHeader unset Forwarded` and `X-Forwarded-For` before `ProxyAddHeaders On`; `UseCanonicalName On` with `ServerName` set to the public name.

## Deferred: custom rules and virtual patching

Decision (from a comparison with a Coraza WAF): this change implements protocol-level anomaly detection only. Operator-written rules and virtual patching are deliberately not built yet.

Why deferred: a rule language is an attack surface and a support burden of its own (a rule can block sign-in for everyone, or hide an attack). It needs a safe-by-default design, a testing path and an audit trail before operators are given it, and the built-in classes above already cover the framing and syntax cases where a generic rule engine is least needed. Virtual patching (a rule that shields a specific route until a fix ships) depends on the same engine.

Rough design, for when it is picked up:

* An operator rule file (declarative, versioned, loaded from a configured path or stored as a setting) with rules that match on method, path prefix or exact path, a named header's presence or a size bound, query length, and declared body size. No regular expressions in the first version and no access to request bodies.
* Every rule starts in detect mode: log, count and emit the typed `protocol` signal, with the rule id as the code. Promotion to enforce is a separate, audited change per rule, with a dry-run view showing what the last N requests would have matched.
* Hot reload through the same cache and invalidation as `security.protocol.*`, with the previous rule set kept if a new one fails to parse.
* Rule edits and promotions write audit events with the actor, the rule id and the diff; hits use the same throttled audit and counter path as the built-in classes.
* A per-rule route-prefix scope and an expiry date, so a virtual patch cannot outlive the fix unnoticed.
* The same exemption limits as today: rules cannot exempt framing or path-control findings, and `/healthz` and `/readyz` stay reachable.
