# Gmail API bridge (V7)

The Gmail bridge (`plugins/bridge-gmail`) connects Mailwoman to Gmail / Google
Workspace via the Gmail API. It is a first-party **WASM plugin** (`wasm32-wasip2`)
implementing the engine account-backend seam; a Gmail account looks like any other
backend to the engine.

The Gmail bridge **shipped fully in V7** — it was the sanctioned first scope-cut
candidate, but it was not cut. (For most Workspace users, IMAP + XOAUTH2 also works;
the API bridge additionally gives true Gmail label semantics and history-ID delta
sync.)

The guest never opens a socket and never holds OAuth secrets: HTTP goes through the
host `http-fetch` import (restricted to the Gmail hosts in `net_allowlist`); tokens are
injected by the host `oauth-token` import.

## What it delivers

- **Mail** through the real jail, indistinguishable from IMAP, with:
  - **true Gmail label semantics** (labels mapped to roles/folders rather than IMAP's
    folder approximation), and
  - **history-ID delta sync** (efficient incremental sync via the Gmail `historyId`,
    carried through the engine's opaque plugin sync cursor).

## Authentication (OAuth, per-user client)

Gmail uses OAuth 2.0 with a per-user Google client. You create OAuth credentials in a
Google Cloud project and give Mailwoman the client ID.

### Admin app registration (Google Cloud)

1. In the Google Cloud console, create (or pick) a **project**.
2. Enable the **Gmail API** for the project.
3. Configure the **OAuth consent screen** (Internal for a single Workspace org, or
   External otherwise) and add the scopes the bridge uses:
   - `https://www.googleapis.com/auth/gmail.modify` (read/label/modify)
   - `https://www.googleapis.com/auth/gmail.send`
   - plus profile/email scopes as needed.
4. Create an **OAuth client ID** (Web application or the appropriate type) and add your
   Mailwoman redirect URI.
5. Copy the **client ID** (and client secret if your flow uses one).
6. For Workspace, an admin may need to **allow** the app / scopes in the Admin console
   (API controls → app access control).

### Bring-your-own app ID

Provide the client ID to Mailwoman when adding the account. Mailwoman acts as an OAuth
**client** to your project; it does not self-register (no dynamic client registration
in V7). Using your own project keeps consent and audit in your Google org.

## CI

The `bridge-fixtures` job replays recorded Gmail request/response pairs (label mapping
+ history-ID delta) through the real engine. The nightly, secret-gated `live-interop`
job hits a real account only when secrets are present.

## Scope boundary (honest)

- **Mail is delivered through the real plugin jail.** The WIT world currently exports
  the **account-backend (MAIL)** interface only; any Google calendar/contacts surfaces
  a bridge advertises follow the same post-1.0 seam extension as the other bridges.
