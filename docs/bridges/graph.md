# Microsoft Graph bridge (V7)

The Graph bridge (`plugins/bridge-graph`) connects Mailwoman to Microsoft 365 /
Outlook.com / Exchange Online via the Microsoft Graph API. It is a first-party
**WASM plugin** (`wasm32-wasip2`) that implements the engine account-backend seam, so
once loaded the engine treats a Graph account the same as an IMAP account.

The guest never opens a socket and never holds OAuth secrets: all HTTP goes through
the host `http-fetch` import (restricted to the Graph hosts in the plugin's
`net_allowlist`), and tokens are injected by the host `oauth-token` import.

## What it delivers

- **Mail** — folders, message sync (delta), fetch (MIME), flag/move/submit — through
  the real jail, indistinguishable from IMAP.
- Implemented and fixture-tested: calendar (including shared calendars and rooms),
  contacts + GAL, To-Do, reactions, voting, message-recall (honesty matrix respected),
  and Focused-Inbox sync. **See the scope boundary below** — mail is drivable through
  the plugin seam today; the PIM/reactions surfaces are implemented and tested but not
  yet wired through the seam.

## Authentication (OAuth)

Graph uses OAuth 2.0. The bridge supports the device-code flow and the authorization-
code flow; the host acquires and refreshes the token. You register an application in
Microsoft Entra ID (Azure AD) and give Mailwoman its client ID.

### Admin app registration (Entra ID)

1. In the Entra admin center, go to **App registrations** → **New registration**.
2. Name it (e.g. "Mailwoman"). For **Supported account types**, choose the tenancy
   that matches your users (single-tenant, multitenant, or personal + work/school).
3. Add a **redirect URI** for the authorization-code flow (your Mailwoman callback
   URL), or enable **Allow public client flows** for the device-code flow.
4. Under **API permissions**, add the **Microsoft Graph delegated** permissions the
   bridge uses, then grant admin consent:
   - `Mail.ReadWrite`, `Mail.Send`
   - `Calendars.ReadWrite`, `Contacts.ReadWrite`, `Tasks.ReadWrite`
   - `User.Read`, `offline_access`
   - `People.Read` / directory read for GAL, as your policy allows.
5. Copy the **Application (client) ID** and the **Directory (tenant) ID**.

### Bring-your-own app ID

Provide the client ID (and tenant ID) to Mailwoman when adding the account. Mailwoman
acts as an OAuth **client** to your registration — it does not self-register. Using
your own registration keeps consent, conditional-access, and audit under your tenant's
control. There is no dynamic client registration in V7 (tracked for post-1.0).

## CI

The `bridge-fixtures` job replays recorded Graph request/response pairs through the
real engine (no live service). The nightly, secret-gated `live-interop` job hits a
real tenant only when tenant secrets are present.

## Scope boundary (honest)

- **Mail is delivered through the real plugin jail.** The WIT world currently exports
  the **account-backend (MAIL)** interface only.
- **Calendar / tasks / reactions / voting / recall / Focused-sync are implemented and
  fixture-tested but not yet drivable through the plugin seam** — advertised via the
  bridge's `capabilities()`, they are a post-1.0 WIT-export extension. Until then, the
  UI's existing standards/header-convention fallbacks handle those surfaces.
