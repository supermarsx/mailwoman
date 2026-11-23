---
title: Connecting clients through Mailwoman
---

# Connecting clients through Mailwoman

Mailwoman's mobile and desktop apps talk to the server over JMAP, the same way the web app does. The deployment provides the server address; the user signs in to their mailbox, and the server connects to the configured upstream. The client never receives upstream credentials and never connects to the upstream itself. This page is a reference for people who run the server or write a client: it lists the discovery addresses, the native sign-in steps, and what the JMAP endpoint supports. It has no console screen.

## Discovery

| Address | Answer |
|---|---|
| `/.well-known/mailwoman` | Public connection metadata: the JMAP Session address and the native sign-in and sign-out addresses. It contains no account or capability information. |
| `/.well-known/jmap` | The authenticated JMAP Session. Its API, upload, download and event-stream addresses are absolute and point back through Mailwoman. |
| `/jmap/session` | The Session route used by the web app and older clients. |

When the server is hosted under a sub-path (`MW_BASE_PATH`), the discovery answers carry that prefix. Set `MW_PUBLIC_URL` to the public HTTPS address when a reverse proxy is in front of the server; otherwise the server derives its origin from the request and the trusted-proxy settings. If it cannot form a plain origin, discovery answers `400`. Session answers are marked `Cache-Control: no-store` and vary by credentials and by linked-mailbox selector. A Session request without credentials gets `401` with a `Bearer` challenge.

To let people find the server from their address, you can publish an RFC 8620 `_jmap._tcp` SRV record that points at the HTTPS host. Mailwoman does not create or change DNS records.

## Native sign-in and calls

1. Read the connection metadata, or request the Session address directly.
2. Send the same JSON body as the web sign-in to the sign-in address from the metadata, with `username`, `password` and `"clientType": "native"`. The request must come from a native origin: the built-in app origins `http://tauri.localhost`, `https://tauri.localhost` and `tauri://localhost` are accepted, and `MW_NATIVE_ORIGINS` adds more, comma-separated and exact. Sign-in policy, bot checks, second-factor challenges and forced password changes apply as for the web app. The metadata says `twoFactorChallenges: true`.
3. Complete any challenge. A finished sign-in returns a bearer token, which the app keeps in the operating system's credential store.
4. Fetch the Session with `Authorization: Bearer <token>`, pick the account and capabilities from it, and `POST` JMAP requests to its `apiUrl`. Use its upload and download templates for blobs.
5. Sign out through the sign-out address and delete the token.

The shared client tries `/.well-known/jmap` first and falls back to `/jmap/session` for servers without it; an authentication refusal does not trigger the fallback. It refuses an advertised API address outside the configured origin, prefix and JMAP namespace before sending any request with the token.

A third-party JMAP client has to support this bearer flow or accept the token it produces. HTTP Basic sign-in and a general OAuth grant for mail are not offered. Scoped Sieve API keys give access to Sieve only, not to mail.

## What the endpoint supports

The native engine advertises these in its Session:

| Capability | Meaning |
|---|---|
| `urn:ietf:params:jmap:core` | RFC 8620 core, with the limits below |
| `urn:ietf:params:jmap:mail` | Mailboxes, emails, threads, search snippets, and `Email/copy`, `Email/import`, `Email/parse` |
| `urn:ietf:params:jmap:submission` | Sending |
| `urn:ietf:params:jmap:mdn` | Read receipts |
| `urn:ietf:params:jmap:vacationresponse` | Vacation response |
| `urn:ietf:params:jmap:quota` | Quota, reported from the server's configured local limits rather than from an IMAP quota on the mail server |
| `urn:mailwoman:calendars`, `tasks`, `notes`, `contacts` | Mailwoman's own calendar, task, note and contact methods; they are not the standard JMAP calendar or contact capabilities |
| `urn:mailwoman:crypto`, `security`, `search` | Mailwoman's own key, security and search methods |

Sieve (`urn:ietf:params:jmap:sieve`) is advertised when the account's mail server supports it. The standard blob, S/MIME verification, principals, contacts and calendar capabilities are not advertised. In proxy mode the Session carries the upstream server's capabilities instead of this list.

The WebSocket stream the server offers sends state changes and negotiates the `jmap` subprotocol; it ignores request text sent by the client, so it is not a complete RFC 8887 transport.

### Limits

The Session advertises these limits, and the server enforces them before it connects an account:

| Limit | Value |
|---|---:|
| Upload size | 50,000,000 bytes |
| Request size | 2,097,152 bytes |
| Calls per request | 64 |
| Objects per get or set | 5,000 |
| Concurrent requests (advertised) | 4 |
| Concurrent uploads (advertised) | 4 |

A request that is not valid JSON, has a malformed method-call list, names an unknown capability, or is too large is refused with `application/problem+json` and the standard JMAP problem types. A get or set over the object limit gets the method-level error `requestTooLarge`. Unknown properties on the request object are ignored. HTTP callers must send `using`.

Concurrency is counted per signed-in account, across its tabs and devices, and the server as a whole has a higher ceiling. A request over the limit waits for a free slot and is refused with `429`, `Retry-After` and the `RateLimit-*` fields only if none frees up in time. These are set in [Settings](/admin/settings) by `ratelimit.jmap.perAccountConcurrent` (default 8, never below 4), `ratelimit.jmap.globalConcurrent` (default 256) and `ratelimit.queueMs` (default 3000 milliseconds; 0 refuses at once).

### Behaviour of the core methods

- `Core/echo` is answered and returns the client's properties unchanged. A failed method returns an `error` response and later calls still run.
- Result references check the response name and support wildcard array projection, flattening and JSON Pointer escapes. An argument given both directly and by reference is refused.
- `createdIds` is returned when the request supplies a map.
- `Email/set` checks a string `ifInState` before changing anything and returns `stateMismatch` when it is stale. This is a check against the server's own state; it is not an atomic compare-and-change on the upstream IMAP server.

References: [RFC 8620](https://www.rfc-editor.org/rfc/rfc8620.html), [RFC 8621](https://www.rfc-editor.org/rfc/rfc8621.html).

## Troubleshooting

- `400` from `/.well-known/mailwoman` or `/.well-known/jmap`: the server could not form its public origin. Set `MW_PUBLIC_URL` to the public HTTPS address, with no path, user, query or fragment.
- `401` with a `Bearer` challenge from the Session: the token is missing or expired; sign in again.
- A native sign-in is refused from an app that uses its own origin: add the origin to `MW_NATIVE_ORIGINS`.
- `429` on JMAP calls: the account is over its concurrent-request limit and no slot freed up within `ratelimit.queueMs`. Wait for `Retry-After`.

## Related

- [Settings reference](/admin/settings)
- [Android app](/deploy/mobile-android)
- [Desktop app](/deploy/desktop)
- [Mail server connection](/deploy/imap-pop3)
