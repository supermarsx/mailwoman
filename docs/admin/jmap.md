# Connecting clients through Mailwoman

Mailwoman's native mobile and desktop shells use the shared web client's JMAP transport. The deployment provides the Mailwoman server address; the user signs in to their mailbox, and Mailwoman connects to the configured upstream. The client never receives upstream credentials or connects directly to that upstream.

## Discovery

| Endpoint | Response |
|---|---|
| `/.well-known/mailwoman` | Public connection metadata: the JMAP Session URL and the native sign-in/sign-out endpoints. No accounts or account capabilities. |
| `/.well-known/jmap` | Authenticated JMAP Session, with absolute API, upload, download and event-stream URLs pointing back through Mailwoman. |
| `/jmap/session` | Existing Session route, retained for the browser and older clients. |

These endpoints are also mounted under `MW_BASE_PATH`. The origin-root discovery endpoint advertises the deployment prefix. Set `MW_PUBLIC_URL` to the canonical public HTTPS address when reverse proxying; otherwise the usual trusted-proxy and Host handling supplies the origin. Session responses are `Cache-Control: no-store` and vary by credentials and linked-mailbox selector. An unauthenticated Session request returns 401 with a Bearer challenge.

An operator who wants domain-based discovery may publish an RFC 8620 `_jmap._tcp` SRV record pointing at the HTTPS host. This change does not create or modify DNS records. Mailwoman's own shells already receive their server address from deployment configuration.

## Native authentication and calls

1. Read the public connection metadata or request the JMAP Session URL directly.
2. Submit the existing `/api/login` JSON body with `username`, `password`, and `clientType: "native"`. Use the normal supported native origin. The built-in Tauri origins are accepted; `MW_NATIVE_ORIGINS` can add exact origins. Existing sign-in policy, anti-bot checks, two-factor challenges, and forced-password-change gates still apply.
3. Complete any challenge through the existing native sign-in flow. A completed sign-in returns the native bearer token, which the shell stores in its OS credential store.
4. Fetch the Session with `Authorization: Bearer <token>`. Choose the account and capabilities from that authenticated Session, then POST JMAP requests to its `apiUrl`. Use its upload and download templates for blobs.
5. Sign out through `/api/logout` and remove the local token.

The native shared client tries `/.well-known/jmap`, falls back to `/jmap/session` for older deployments lacking discovery, and consumes the advertised API address. Authentication refusals do not trigger that fallback. An advertised API outside the configured Mailwoman origin, deployment prefix, or JMAP namespace is refused before any bearer-bearing request is sent. Linked-mailbox Session discovery follows the same explicit account selection and authorization rules as the existing Session route.

## Compatibility boundaries

Discovery exposes the capabilities already supported by the selected Mailwoman account; it does not add unsupported JMAP methods or extensions. Mail, submission, storage quotas and the existing optional extensions remain capability-driven. The event-stream URL retains the existing server behavior; this does not add new event-stream query features or advertise a new WebSocket protocol extension.

Our shells use Mailwoman's native bearer flow. A third-party JMAP client must support that authentication flow or accept its resulting bearer token. Publishing discovery does not add HTTP Basic sign-in or a general-purpose OAuth mail grant. Existing scoped Sieve API keys remain Sieve-only and do not gain access to mail.

The loopback HTTP fixture verifies native login, authenticated discovery, and Mailbox/get plus Email/query through the advertised API. Shared-client tests verify discovery, fallback, bearer headers, proxy URL restrictions, linked routing and browser compatibility. These checks do not constitute an Android APK or physical-device test.

## Native JMAP Core behavior

The native engine answers `Core/echo` and returns method failures under the standard `error` response name. A failed method does not stop later calls. Result references validate the response name and support RFC 8620 wildcard array projection, flattening and JSON Pointer escapes. Conflicting normal and referenced arguments are refused. Echo preserves arbitrary client properties, including `type`.

Native HTTP admission rejects malformed JSON, malformed invocation envelopes, unknown capabilities and oversized requests before connecting an account or executing a method. Errors use `application/problem+json` and the standard JMAP problem URNs. Unknown request extension properties are ignored. In-process engine callers may omit `using` for compatibility; HTTP callers must provide it.

The native Session advertises these enforced limits:

| Limit | Value |
|---|---:|
| Upload bytes | 50,000,000 |
| Request bytes | 2,097,152 |
| Calls per request | 64 |
| Objects per get/set | 5,000 |
| Concurrent requests | 4 |
| Concurrent uploads | 4 |

Concurrency uses conservative process-wide budgets, so all accounts share four request slots and four upload slots. Exhausted slots return a retryable HTTP 429 problem; permits release when processing completes or is cancelled. GET/SET object excess is a method-level `requestTooLarge` error. A GET without explicit IDs that would return too many objects is also refused.

`createdIds` is returned when the request supplies a map, including the supplied IDs and newly created objects. `Email/set` checks a string `ifInState` before mutation and returns `stateMismatch` when stale; absent or null means no precondition. This is a local JMAP state check, not an atomic compare-and-mutate guarantee from an IMAP server.

Upstream-proxy mode retains the upstream capability and method contract. These Core improvements are not a claim that every published JMAP RFC or every mail-property edge case is implemented. Native contact/calendar/task extensions remain distinct from standard RFC capabilities.

References: [RFC 8620](https://www.rfc-editor.org/rfc/rfc8620.html), [RFC 8621](https://www.rfc-editor.org/rfc/rfc8621.html).

### Standards inventory (source audit, 2026-10-26)

This inventory separates native protocol support from Mailwoman-specific features.
It is not a full conformance certificate. Proxy mode exposes the verified upstream
server's capabilities; a local UI feature does not add a standard capability to it.
The published capability inventory was checked against the
[IANA JMAP registry](https://www.iana.org/assignments/jmap/).

| Standard | Native source evidence | Remaining acceptance or implementation |
| --- | --- | --- |
| RFC8620 Core | `mw-engine/src/jmap_core.rs`, `mw-server/src/jmap_limits.rs`; request admission, references and limits | Full method/property/error and push-subscription conformance audit remains |
| RFC8621 Mail, Submission, VacationResponse | `mw-engine/src/jmap.rs`, `folders.rs`, `mail_ext/` | Native mail surfaces exist; backend capability and full mail-property compliance still need individual verification |
| RFC8887 WebSocket binding | `mw-server/src/push.rs` streams changes and negotiates the `jmap` subprotocol | Incoming request text is ignored; a complete bidirectional JMAP transport is not implemented |
| RFC9007 MDN | Standard MDN capability, native send/parse and account checks | Existing bounded acceptance is recorded in the session ledger; not a blanket mail-standard claim |
| RFC9219 S/MIME verification | Mailwoman crypto extensions exist separately | Standard `smimeverify` capability is not advertised; standard property semantics remain to implement |
| RFC9404 Blob management | Core HTTP upload/download and owned attachment paths exist | Standard `blob` capability and Blob method family are not advertised/implemented |
| RFC9425 Quota | `mw-engine/src/mail_ext/quota.rs` reports configured local cache limits | This is not upstream IMAP quota discovery; complete quota methods/properties remain to audit |
| RFC9610 Contacts | Mailwoman contact types use `urn:mailwoman:contacts` | Standard `contacts` capability and ContactCard/AddressBook contracts remain to implement |
| RFC9661 Sieve | `mw-engine/src/sieve_jmap.rs`, actual ManageSieve-backed tests in `t32_sieve_jmap.rs` | Capability-dependent; broader conformance and each live provider remain separate acceptance |
| RFC9670 Principals | Internal verified principals support authentication and ownership | Standard JMAP principals/owner capability and methods remain to implement |
| RFC9749 Web Push VAPID | Browser push and protected delivery code exist | Standard capability advertisement and JMAP PushSubscription lifecycle are not implemented |
| JMAP calendars / principal availability | Mailwoman calendar types use private capabilities | Standard calendar and availability contracts are not supplied by the private PIM API; check publication status before implementation |

The distinction for WebSocket transport follows
[RFC8887](https://www.rfc-editor.org/rfc/rfc8887.html); VAPID capability discovery
and subscription-key continuity follow
[RFC9749](https://www.rfc-editor.org/rfc/rfc9749.html).
Missing capabilities must not be advertised merely because similarly named
Mailwoman functionality exists. Each additional capability needs bounded input,
verified account ownership, policy enforcement, persistence, standard errors and
interoperability tests before publication.
