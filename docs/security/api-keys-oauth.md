# API keys and OAuth

Scripts and other applications can act on a mailbox in two ways: with an **API
key** the user creates, or with an **OAuth access token** an application obtains
after the user approves it. Both carry the same kind of scope, and the server
checks that scope on every request that presents them.

## The scope

A scope lists what a credential may do. Anything not listed is refused.

| Part | Values | Notes |
|---|---|---|
| Verbs | `read`, `send`, `delete` | Each must be granted explicitly. PIM changes (`calendar.propose`, `tasks.write`) and drafts count as `send`. |
| Surfaces | `mail`, `pim` (calendar, tasks, notes, contacts), `scripts` (Sieve) | `scripts` is never implied by `mail`. |
| Accounts | all, or a list | A key always acts as the account that created it. |
| Folders | all, or a list of folder ids | A folder-limited key covers a request only when the request names its folders and all of them are granted; listings are filtered to the granted folders. |
| IP allowlist | CIDRs or addresses; empty = any | Checked against the client address resolved through `MW_TRUSTED_PROXIES`, so a forwarded header from an untrusted peer cannot satisfy it. |
| Expiry | RFC 3339 time; empty = none | |
| Rate limit | requests per minute; empty = unlimited | Fixed one-minute window, counted per server process. With several processes, the effective limit is that many times higher. |
| MCP tools | list of tool names | See [MCP server](./mcp.md). |
| Unattended send | on / off | A request to send through MCP without the Outbox hold. It has no effect until an administrator countersigns the key. |

When a credential is checked, the server verifies it, then refuses an expired
credential, a source outside the allowlist, a request over the rate limit, a
token used against a resource it was not issued for, and finally any request the
scope does not cover. Each check, allowed or refused, writes an audit entry with
the key prefix or OAuth client id, the result, the reason and the source address.

## API keys

Users create keys in **Settings** → **API access** → **API keys**.

<!-- screenshot: id=security/apikeys-create; screen=Settings > API access > API keys; state=create form filled; element=Label, Capabilities, Surface, Accounts, Folders, Constraints, Create key -->

1. Enter a **Label** and choose **Capabilities**, **Surface**, **Accounts**,
   **Folders** and the **Constraints** (**Expiry**, **Rate limit**, **IP
   allowlist**).
2. Select **Create key**. The server asks you to confirm it is you (password
   plus your second factor, or a passkey).
3. Copy the key and select **I have saved it**. The key is shown once and cannot
   be retrieved again.

<!-- screenshot: id=security/apikeys-reveal; screen=Settings > API access > API keys; state=new key revealed; element=one-time secret with Copy and I have saved it -->

How keys are built and stored:

- **Format:** `mwk_<prefix>.<secret>`. The prefix is 8 random bytes (16 hex
  characters) stored in the clear to find the key; the secret is 32 random bytes.
- **Storage:** only an Argon2id hash of the secret is stored (the library's
  default parameters, not the [security policy's](./admin-panel.md#security-policy)
  cost settings). A database thief cannot use a key without guessing a 256-bit
  secret.
- **Use:** send the key as `Authorization: Bearer mwk_…` or `X-API-Key: mwk_…`.
  Keys authorise the REST API under `/api/v1` and the MCP endpoint `/mcp`. A key
  sent together with a browser session cookie narrows that session to the key's
  scope.
- **Revoking:** **Revoke** in **Existing keys** (`POST /api/keys/{prefix}/revoke`)
  revokes one key immediately. A user can revoke only their own keys; another
  user's prefix is answered like one that does not exist. Administrators see every
  key under **Integrations** → **API & MCP keys** and can revoke any of them.
- **Disabled accounts:** keys of an account an administrator has disabled are
  refused; keys of an account that must change its password are held.
- **Linked mailboxes:** a key created while a linked mailbox is selected is
  revoked when that mailbox is unlinked.
- **Logs:** strings that look like `mwk_` keys are cut out of log lines (see
  [Logging and monitoring](./observability.md)).

<!-- screenshot: id=security/admin-api-keys; screen=Admin console > Integrations; state=two keys listed; element=API & MCP keys list with revoke -->

## OAuth

Mailwoman includes an OAuth 2.1 authorization server for applications that act on
a user's behalf.

- **Authorization code with PKCE.** Only the authorization-code grant is offered.
  PKCE with `S256` is required; `plain` or a missing challenge is refused.
  Clients are public clients: there is no client secret, and PKCE is what binds the
  code to the application that started the flow.
- **Resource indicators (RFC 8707).** Every authorization must name the resource
  the token is for. The token request must repeat it, a refresh cannot change it,
  and a token bound to one resource is refused at another.
- **Registered clients only.** The `client_id` must exist and the `redirect_uri`
  must match one registered for it exactly. Administrators add clients (see
  [Identity provider](/admin/identity-provider)); applications can register
  themselves only if an administrator turns on **Let apps register themselves**
  under **OAuth clients**. Self-registration is off by default, refuses every
  redirect host not on **Allowed redirect hosts** (an empty list refuses all), can
  require a shared registration token (`MW_DCR_INITIAL_ACCESS_TOKEN`), and gives
  every self-registered app exactly the configured scope.
- **Consent.** The user approves the application on a consent screen in the mail
  app. The token's scope is what the user approved, limited by the client's
  ceiling. OAuth tokens never carry unattended send.
- **Tokens.** Authorization codes (10 minutes), access tokens (1 hour) and refresh
  tokens (30 days) are 32 random bytes each, stored only as SHA-256 hashes. A code
  is revoked when it is redeemed, and a refresh token when it is used to obtain a
  new pair.
- **Endpoints:** `/oauth/token`, `/oauth/introspect`, `/oauth/revoke`, and
  `/oauth/register` when self-registration is on.

<!-- screenshot: id=security/admin-oauth-clients; screen=Admin console > OAuth clients; state=self-registration off; element=Let apps register themselves, Allowed redirect hosts, Granted scope -->

### Limits

- `/oauth/introspect` and `/oauth/revoke` do not authenticate the caller. Anyone
  who holds a token can learn its scope, account id and expiry, or revoke it.
  Tokens are 256-bit random values, so this does not let anyone discover tokens
  they do not already have.
- Redeeming a code and rotating a refresh token check the token and then revoke
  it in two steps, not in one atomic operation. Two simultaneous requests with the
  same code and verifier could each receive a token pair.
- An OAuth access token is checked on `/mcp`. The REST API under `/api/v1` accepts
  API keys and browser sessions, not OAuth tokens.

## Who can do what

| Party | Can | Cannot |
|---|---|---|
| Holder of an API key | Everything the key's scope allows, from an allowed address, until it expires or is revoked. | Exceed the scope, act as another account, or send through MCP without the Outbox hold unless an administrator countersigned the key. |
| Holder of an OAuth access token | What the user approved for that client, at the resource it was issued for, for up to one hour (longer with the refresh token). | Use it at another resource or obtain unattended send. |
| Someone with a copy of the database | See key prefixes, scopes and hashes. | Recover a key secret or token from its hash. |
| Administrator | List and revoke any key; countersign unattended send; configure clients and self-registration. | Read a key secret after it was shown. |

## Related

- [MCP server](./mcp.md)
- [Identity provider](/admin/identity-provider)
- [OAuth clients](/admin/oauth-clients)
- [API access (user settings)](/user/settings/integrations#api-access)
