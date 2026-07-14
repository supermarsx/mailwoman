# Scoped API keys & OAuth 2.1 (V6)

Mailwoman exposes two ways for automation and third-party clients to act on an
account: **scoped API keys** (opaque bearer tokens) and an **OAuth 2.1 authorization
server** (authorization-code + PKCE + resource indicators). Both resolve to the same
typed **scope** model and the same per-request enforcement.

## The scope model

A scope is a typed capability set — there is no implicit escalation:

- **Verbs** — `read`, `send`, `delete` (grant only what the client needs; `no-send`
  and `no-delete` are simply the absence of those verbs).
- **Resource** — per-account and per-folder subsets, or `*`; and mail vs PIM.
- **Bounds** — an optional **IP allowlist**, an **expiry** (time-boxed keys), and a
  **per-key rate limit**.
- **MCP** — `mcp_tools: [...]` names the individual MCP tools a key may call (see
  [`mcp.md`](./mcp.md)).
- **`unattended_send`** — a distinct, dangerous capability, off by default (see MCP
  send-gating).

`Scope::allows(required)` is the grant/deny matrix: a request is permitted only if the
key's scope is a superset of what the operation requires.

## API keys

- **Wire format** — `mwk_<prefix>.<secret>`, a 256-bit random secret. The `<prefix>`
  indexes the row for O(1) lookup; the secret is **Argon2id-hashed at rest** and never
  stored in the clear.
- **Shown once** — the full key is returned exactly once at mint time. It cannot be
  recovered afterward; a lost key is revoked and re-minted.
- **Individually revocable** — `POST /api/keys/{prefix}/revoke` (user) or the admin
  oversight endpoint revoke each key independently.
- **Per-key audit** — every use writes an audit entry (key prefix, action, source IP).

User-facing endpoints: `GET/POST /api/keys`, `POST /api/keys/{prefix}/revoke`. Admin
oversight: `GET /admin/api-keys`, `POST /admin/api-keys/{id}/revoke`.

## OAuth 2.1 authorization server

- **Authorization code + mandatory PKCE (S256)** — `/oauth/consent` +
  `/oauth/decision` drive the browser consent flow; the code exchange is
  `/oauth/token`. PKCE is required, not optional.
- **Resource indicators (RFC 8707)** — a token is scoped to the resource it was issued
  for, so a token minted for one surface is not a bearer credential for another.
- **Admin-approved client registry** — clients must be registered/approved by an admin
  before they can complete an authorization; there is no open dynamic registration.
- **Opaque, hashed tokens** — access/refresh tokens are opaque and stored hashed (no
  JWT dependency by default). `/oauth/introspect` and `/oauth/revoke` complete the
  lifecycle.

## Enforcement

Presented credentials (an `mwk_` key or an OAuth access token) are resolved to a
`Scope` at the request boundary and checked against what the operation requires,
together with the key's **IP allowlist**, **expiry**, and **rate limit**. A denied
request is rejected and audited. The MCP surface (`/mcp`) enforces the caller's scope
per tool; see [`mcp.md`](./mcp.md).

### Honest status note (26.7)

The cookie-authenticated mailbox path (the SPA's own session) is unchanged. Scoped-key
enforcement for the REST convenience layer (`/api/v1`) is wired as a dedicated
`Send`-safe middleware in front of that surface as part of the V6 mount. Where a
capability is not yet enforced end-to-end, it is tracked in the orchestration state and
the live-E2E gate — this document describes the scope model and the intended
enforcement, and does not claim guarantees the code does not yet make.

## Operational guidance

- Grant the **narrowest** scope that works: read-only, single account/folder, no send,
  short expiry, and an IP allowlist where the caller has a stable address.
- Treat `mwk_` keys like passwords: they are shown once, hashed at rest, and should be
  stored in a secret manager, never in source.
- Revoke on suspicion — revocation is immediate and per-key, so one leaked key does not
  force a global rotation.
