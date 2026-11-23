---
screen: true
---
# HTTP protocol guard

The protocol guard checks the method, version, request target and headers of every request before routing, sign-in or body parsing. It looks for requests that a front proxy and this server could read differently (request smuggling), and for requests that are malformed in ways no client of Mailwoman produces. Each class of finding can be off, only detected, or refused.

![The Protocol guard sub-tab of Aggregate and adaptive protection, with its explanation and the Overall mode card.](/screenshots/admin-access/protocol-guard-page.webp){.mw-shot}

## Before you start

- The policies are on **Security centre** > **Aggregate and adaptive protection** > **Protocol guard**, at `<console path>#/security/protection/protocol-guard`. Reading them needs the `settings.read` permission; changing them needs a role that can change settings.
- Set `MW_TRUSTED_PROXIES` and `MW_PUBLIC_URL` before you enforce the **Forwarding headers** or **Host pinning** classes (see [Reverse proxy](../deploy/reverse-proxy.md)).

## Changing the mode

1. Open **Protocol guard**. **Overall mode** shows **Off**, **Detect** and **Enforce**, and after **Now:** the mode all classes share, or **Mixed**.

   ![The Overall mode card of the Protocol guard sub-tab: the Off, Detect and Enforce buttons, the current state Mixed, and the count of requests that enforcing would refuse (masked).](/screenshots/admin-access/protocol-guard-overall.webp){.mw-crop}

2. To set every class at once, choose one of the three buttons. **Enforce** first asks for confirmation and says how many requests the classes that only detect have seen since the server started. Classes pinned by an environment variable keep their value.
3. To set one class, change its mode in its group below and choose **Save**. **Discard changes** drops an unsaved change.

![The Framing and smuggling group of the Protocol guard sub-tab, with the request framing and body framing classes and their modes.](/screenshots/admin-access/protocol-guard-framing.webp){.mw-crop}

Each row shows the setting key, whether the value is the default, the mode in effect and the counters since the server started, as "Detected 0, refused 0 since start." Changes apply to new requests without a restart. The groups are **Framing and smuggling**, **Host and request target**, **Headers**, **Limits**, and **Methods and exemptions**. The same counters appear on the **Overview** sub-tab.

## Modes

- **Off**: the class is not checked.
- **Detect**: the request is served, and the finding is counted and logged.
- **Enforce**: the request is refused.

`/healthz` and `/readyz` are checked too, but never refused: an enforced class acts as **Detect** for them.

## Classes

| Class (key) | What it reports | Default | Refusal |
|---|---|---|---|
| Request framing (`security.protocol.framingMode`) | `Content-Length` together with `Transfer-Encoding`; `Transfer-Encoding` other than exactly `chunked`, or on HTTP/1.0 or HTTP/2; non-numeric or conflicting `Content-Length` | enforce | `400`, connection closed |
| Host syntax (`hostMode`) | `Host` missing on HTTP/1.1, empty, repeated, or holding userinfo, whitespace or path characters. IPv6 literals, ports and a trailing dot are accepted | enforce | `400`, connection closed |
| Control characters in path (`pathControlMode`) | NUL, control characters, `%00`, `%0d` or `%0a` in the path | enforce | `400`, connection closed |
| Host pinning (`hostPinningMode`) | `Host` that is not the host of `MW_PUBLIC_URL` or one of `allowedHosts`. Does nothing while neither names a host | detect | `400` |
| Request-target tricks (`targetMode`) | Double or overlong encoding of dots and slashes, encoded dots, dot segments, `//`, backslashes, raw non-ASCII bytes, an absolute-form target naming another host than `Host` | detect | `400` |
| Path and query length (`pathLengthMode`) | Path over `maxPathBytes` (4096) or query over `maxQueryBytes` (8192) | detect | `414` |
| HTTP methods (`methodMode`) | A method outside `allowedMethods`, such as TRACE, CONNECT or a lower-case method | detect | `405` with `Allow` |
| Repeated headers (`duplicateHeadersMode`) | `Content-Type`, `Authorization`, `Origin`, `Referer`, `User-Agent`, `Sec-Fetch-*`, conditional and range headers sent more than once; a repeated equal `Content-Length`; repeated `Cookie` on HTTP/1.x | detect | `400` |
| Forwarding headers (`forwardedMode`) | `Forwarded`, `X-Forwarded-*` or `X-Real-IP` from a peer outside `MW_TRUSTED_PROXIES` | detect | `400` |
| Header hygiene (`headerHygieneMode`) | More than `maxHeaderCount` (64) headers, a value over `maxHeaderValueBytes` (8192), cookies over `maxCookieBytes` (16384); non-ASCII header bytes; an upgrade outside the WebSocket route; `Connection` naming headers such as `Content-Length` or `Host` | detect | `431` for size and count, `400` otherwise |
| Body framing (`bodyMode`) | GET or HEAD with a body; `Content-Length` over `maxBodyBytes` (128 MiB); `Expect` other than `100-continue`, or without a body | detect | `413`, `417` or `400` |

All keys start with `security.protocol.`; see the [settings reference](/admin/settings#security.protocol.framingMode). The enforced defaults are the classes where a request is ambiguous about where it ends, or is invalid; the detect defaults can be tripped by a legitimate setup, such as a proxy or health check that uses an internal host name, or a CDN that adds forwarding headers before `MW_TRUSTED_PROXIES` is set. Move a class to **Enforce** once its counters show no false positives for your traffic.

**`allowedMethods`** lists GET, HEAD, POST, PUT, PATCH, DELETE, OPTIONS and the WebDAV, CalDAV and CardDAV methods by default. **`exemptPrefixes`** lists route prefixes (each starting with `/`, longer than one character, matched on whole path segments) for which the detect-type classes do not run, for example a large upload route. Request framing and control characters in the path are never exempted, and `/` is ignored.

## What a finding produces

- **A refusal** is a short JSON error in the shape of the other error pages, with no path, query or header value in it. Request framing, Host syntax and control-character refusals also close the connection on HTTP/1.x.
- **A log line** at `INFO` level (target `protocol_guard`) per finding, with the class, a fixed code, the action, the client address and a coarse path class (`admin`, `jmap`, `dav`, `api` or `other`). It contains no header value, cookie, token or query.
- **An audit event** `protocol-anomaly`, at most one per class and client address per minute. The counters on the tab are exact; they are kept in memory and reset when the server restarts.
- **A signal for adaptive protection**: each finding counts as an unusual request for the client address, whatever **What counts as unusual** selects. Nothing happens while **Unusual request protection** is off, its default. Forwarding-header findings send no signal until `MW_TRUSTED_PROXIES` is set, so a proxy's own address is not charged. See [Aggregate and adaptive protection](./request-protection.md).

## What the guard cannot see

The HTTP parser in front of the guard refuses some malformed requests itself, so they never reach the guard and are not counted. The test `crates/mw-server/tests/protocol_ground_truth.rs` sends raw requests to the real listener and records the result:

| HTTP/1.1 request | Result |
|---|---|
| Two `Content-Length` headers with different values; `Content-Length: 5, 6` or `5, 5`; negative, `+5`, non-numeric or overflowing `Content-Length` | `400` from the parser |
| `Transfer-Encoding: xchunked`, `chunked, identity`, a space before the colon, an obs-fold continuation line, `Transfer-Encoding` on HTTP/1.0 | `400` from the parser |
| Bare CR, NUL, 0x01 or DEL in a header value; NUL, space or tab in the request target | `400` from the parser |
| HTTP/0.9 request line, `HTTP/1.1x`, `HTTP/2.0` or `HTTP/9.9` on an HTTP/1 connection | `400` from the parser |
| A 40 KiB request line | `431` from the parser |
| `Content-Length` with `Transfer-Encoding: chunked`; `identity, chunked`; `gzip, chunked`; `chunked` twice | reaches the guard (Request framing) |
| Two identical `Content-Length` headers | reaches the guard (Repeated headers) |
| Duplicate, empty, userinfo or missing `Host` | reaches the guard (Host syntax) |
| TRACE, CONNECT, an unknown or lower-case method | reaches the guard (HTTP methods) |
| `Expect: 100-continue` or an unknown `Expect`; GET with a body | reaches the guard (Body framing) |
| Dot segments, `%2e%2e`, `//`, backslash or `%00` in the path; raw UTF-8 in the path; absolute-form target for another host | reaches the guard (Request-target tricks, Control characters in path) |
| Bare LF line endings; `#fragment` in the target | accepted by the parser; the guard cannot tell |

A front proxy that rewrites requests (re-chunks, merges headers, normalises paths) hides the client's original bytes from every check here: the guard sees what the proxy sends. Preventing smuggling between proxy and server is the proxy parser's job; the guard is a second check. The guard does not see TLS or HTTP/2 frame-level behaviour.

## What to set on the reverse proxy

- Refuse duplicate or conflicting `Host` and `Content-Length`, and `Content-Length` with `Transfer-Encoding`; talk HTTP/1.1 to Mailwoman with the proxy framing the body.
- Overwrite, not append to, `X-Forwarded-For`, `X-Forwarded-Proto`, `X-Forwarded-Host`, `Forwarded` and `X-Real-IP`, and put the proxy's address in `MW_TRUSTED_PROXIES`.
- Pass the public `Host` and set `MW_PUBLIC_URL`.
- Keep the proxy's header and body limits at or below the server's (`MW_HTTP_MAX_HEADER_BYTES`, `MW_HTTP_MAX_HEADERS`, `maxBodyBytes`).

[Reverse proxy](../deploy/reverse-proxy.md) has tested example configurations.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Uploads or imports answer `413` after enforcing Body framing | The request is larger than `maxBodyBytes` | Raise `maxBodyBytes`, or add the route to `exemptPrefixes`. |
| Requests answer `400` after enforcing Host pinning | A proxy or health check uses another host name | Add the name to `allowedHosts`. |
| Forwarding-header findings count every request | `MW_TRUSTED_PROXIES` is not set | Set it to the proxy's address. |

## Related

- [Aggregate and adaptive protection](./request-protection.md)
- [Reverse proxy](../deploy/reverse-proxy.md)
- [Settings reference](./settings.md)
