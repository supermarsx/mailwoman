# Load-time reads in one request: `GET /api/bootstrap`

The mail app reads about twenty small JSON resources when it starts. `GET
/api/bootstrap` returns the answers of those routes in one response, so a cold
load makes one request for them and a warm load one conditional request.

The endpoint is additive. Every route it reads stays available and unchanged;
a client that gets anything other than `200` or `304` from it uses the
individual routes for that page load.

## Response

```json
{"v":1,"sections":{
  "me":{"status":200,"etag":"\"…\"","body":{"username":"…","accountId":"…"}},
  "accounts":{"status":401,"etag":"\"…\"","body":{"error":"…","code":"…"}},
  "drafts":{"status":200,"etag":null,"body":null,"error":"bodyTooLarge"}
}}
```

- Every section below is always present, in this order.
- `status` and `body` are what the source route answers to the same request. A
  JSON body is embedded byte for byte; an empty body is `null`; a non-JSON body
  is a JSON string.
- `etag` is the route's own `ETag` when it sets one (the routes listed in
  `private_etag.rs`), otherwise a hash of the body that only marks a change.
- `error: "bodyTooLarge"` replaces a body larger than 4 MiB; the client reads
  that route directly.

| Section | Source route |
| --- | --- |
| `me` | `GET /api/me` |
| `jmapSession` | `GET /jmap/session` |
| `publicConfig` | `GET /api/public-config` |
| `cookiesConfig` | `GET /api/cookies/config` |
| `accounts` | `GET /api/accounts` |
| `settings` | `GET /api/settings` |
| `appearance` | `GET /api/account/appearance` |
| `preferences` | `GET /api/account/preferences` |
| `features` | `GET /api/features` |
| `notifications` | `GET /api/account/notifications` |
| `policy` | `GET /api/policy` |
| `restriction` | `GET /api/privacy/restriction` |
| `uiPlugins` | `GET /api/ui-plugins` |
| `assistConfig` | `GET /api/assist/config` |
| `version` | `GET /api/version` |
| `focusedSenders` | `GET /api/account/focused-senders` |
| `drafts` | `GET /api/account/drafts` |
| `notificationCenter` | `GET /api/account/notification-center` |
| `appearanceCustomization` | `GET /api/account/appearance/customization?accountId=<me.accountId>` |

`appearanceCustomization` takes the account id from the `me` section of the same
response. Without a `200` from `me` it is requested without the query and
carries the route's own refusal.

## Who sees what

Each section is an internal `GET` that carries the caller's headers and client
address and goes through the same router, with every layer, as a direct call.
It runs in a fresh task, so none of the outer request's per-request state is
visible to it. Each route's own guards and handler checks decide: a section the
caller may not read carries that route's status and error body, never its data.

- Anonymous callers get `publicConfig` and `cookiesConfig` with `200` and the
  other sections with their routes' refusals.
- An account held for a password change gets `me` (which reports the hold) and
  refusals elsewhere.
- Admin console sessions are not mailbox sessions: they get the public sections
  and refusals.
- A request carrying the linked-mailbox selector (`X-Mailwoman-Link`) is refused
  as a whole by the linked-scope guard (`403 linkedContextUnsupported`). The
  endpoint reads the primary mailbox only; reads for a linked mailbox are made
  after the switch, per route.
- Maintenance mode and the client-access gate apply to this path like any other
  governed path and may refuse the whole request.

The routes are a fixed list in `crates/mw-server/src/bootstrap.rs`. The query
string, headers and body of the outer request cannot add, remove or redirect a
section. Validators, ranges, body framing, hop-by-hop fields and
`Accept-Encoding` are not passed to the sections; every other header is.

## Caching

- `Cache-Control: private, no-cache` and `Vary: Cookie`: a browser may keep the
  response but asks the server each time; a shared cache must not keep it.
- `ETag` is a strong tag over each section's name, status and tag. It is absent
  when a section has `error`.
- `If-None-Match` with that tag answers `304`. The sections still run, so a
  `304` is never stale.
- `Set-Cookie`: only the CSRF cookie that `me` issues is passed on. A response
  that sets it is never answered `304`.

## Cost

Each section runs its route's whole guard chain, and request protection counts
each one like the direct call it replaces; the outer request is counted too. One
bootstrap therefore costs about what the nineteen direct calls cost, plus the
outer request's own chain. Sharing one authentication across the sections is a
separate change.

Measured by `bootstrap::tests::statements_per_bootstrap_are_recorded` (engine
mode, SQLite, signed-in mailbox, test defaults, 2026-11-24): one bootstrap ran
685 SQL statements (680 reads, 2 writes, 3 other); the nineteen direct calls ran
680 to 729. Every section passes the request-protection layer like a direct
call, and so does the outer request: one bootstrap is twenty requests to it.

The endpoint is HTTP only: it is listed as `cookie-bootstrap` in
`apps/web/src/api/websocket-action-contract.json`, so the WebSocket action
channel does not dispatch it.
