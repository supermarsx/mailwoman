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
- **Calendar (including shared calendars and rooms), contacts + GAL, To-Do,
  reactions, voting, message-recall, and Focused-Inbox sync** — implemented, mapped,
  and fixture-tested. The bridge targets the `plugin-pim` WIT world, so these
  PIM/parity surfaces are exported across the plugin seam (not mail-only) and
  round-trip through the real jail in the recorded-fixture suite. Provider limits are
  stated honestly — e.g. message recall is best-effort and org-internal (the recall
  honesty matrix never reports a false success). **See the scope boundary below** for
  what is seam-proven vs. user-surface-complete.

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

- **Mail is delivered through the real plugin jail** and is drivable end-to-end today.
- **Calendar / tasks / contacts / reactions / voting / recall / Focused-sync are
  implemented and exported through the `plugin-pim` WIT seam** (the `calendar`,
  `tasks`, and `bridge-parity` interfaces) and round-trip through the real jail in the
  fixture suite. What is not yet fully proven is the **end-to-end engine→CalDAV/JMAP
  user-facing surface** and **live-tenant interop** (the nightly, secret-gated
  `live-interop` job). Treat these surfaces as seam-proven and fixture-proven, not yet
  user-surface-complete against a live tenant. Where a surface is not yet driven
  through the engine end-to-end, the UI's existing standards/header-convention
  fallbacks handle it.
