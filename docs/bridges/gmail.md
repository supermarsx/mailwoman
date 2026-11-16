# Gmail API bridge (V7)

The Gmail bridge (`plugins/bridge-gmail`) connects Mailwoman to Gmail / Google
Workspace via the Gmail API. It is a first-party **WASM plugin** (`wasm32-wasip2`)
implementing the engine account-backend seam; a Gmail account looks like any other
backend to the engine.

The Gmail bridge is included as a first-party plugin. It is the first scope-cut
candidate, but remains in the 1.0 scope. (For most Workspace users, IMAP + XOAUTH2 also works;
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
  the `plugin-pim` world, but its Google calendar/contacts/task capability probes
  deliberately return false. The Gmail API bridge is mail-only.

## Guided authorization

An administrator must approve the plugin and grant `net` and `account-backend`.
An authenticated primary account can then open **Settings → Linked accounts →
Mail provider**, select Gmail API, enter its app ID and connect in the browser.
Register this exact callback in Google Cloud, including any deployment sub-path:

`https://mail.example.com/`

Google enrollment uses an authorization code with S256 PKCE and requests offline
consent. Device-code authorization is not offered for Gmail mail scopes. Configure
the confidential app ID and secret as `MW_GMAIL_OAUTH_CLIENT_ID` and
`MW_GMAIL_OAUTH_CLIENT_SECRET` on the server, or use
`MW_BRIDGE_OAUTH_CLIENT_SECRETS`, a JSON application-ID-to-secret map for distinct
users' registrations. The legacy `MW_BRIDGE_OAUTH_CLIENT_ID` /
`MW_BRIDGE_OAUTH_CLIENT_SECRET` pair is a fallback. Only the secret associated with
the exact selected app ID is sent; another user's BYO app never receives it.

The host verifies the returned Gmail profile address against the signed-in mailbox
before storing access and refresh tokens sealed at rest. A missing refresh token
requires renewed offline consent. Tokens and the PKCE verifier stay server-side.
The route enrolls an existing authenticated account; standalone Google sign-in is
not implemented here. Changing backend type on an account with cached messages is
refused until source migration is implemented; reauthorizing the same backend works.

## Complete mailbox synchronization

Baseline and history reads follow every page, reject repeated tokens, and store a
cursor only after the complete result. The baseline history head is captured before
enumeration so messages arriving during pagination remain discoverable. History is
read at account scope and projected into label membership, including moves into and
out of a label. Expired history IDs request an engine baseline reconciliation rather
than incorrectly reporting a missing mailbox. Trash and spam label views include
their messages. Moving into virtual All Mail archives without adding a nonexistent
Gmail label. The host drives a 30-second delta polling fallback.

See [Google's synchronization contract](https://developers.google.com/workspace/gmail/api/guides/sync).
