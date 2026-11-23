---
title: Microsoft Graph bridge
---

# Microsoft Graph bridge

The Graph bridge connects a mailbox on Microsoft 365, Exchange Online or Outlook.com to Mailwoman through the Microsoft Graph API. Mail comes from Graph, and free/busy lookups can use it. The mailbox owner signs in once with Microsoft, and the server keeps the refresh token sealed on the server.

The bridge plugin (`bridge-graph`) can reach only `graph.microsoft.com` and `login.microsoftonline.com`. It never sees an access token's secrets: the server obtains and refreshes the token and hands the plugin a short-lived access token for each call.

## Before you start

- The server runs in engine mode and has a public HTTPS address (plain HTTP is accepted only for `localhost`, `127.0.0.1` and `[::1]`).
- An administrator has approved and enabled `bridge-graph` and granted it `account-backend` and `net`. See [Plugins](../plugins/index.md#approve-and-enable).
- The administrator permission **Microsoft bridge** is on (`cap.integration.microsoft`, default on) together with **Plugins** (`cap.plugins`).
- You have an app registration in Microsoft Entra ID. Mailwoman does not register an app for you and does not use dynamic client registration.

## Register the app in Entra ID

1. In the Entra admin center, open **App registrations** and choose **New registration**.
2. Pick the **Supported account types** that match your users.
3. For browser authorization, add a **Web** redirect URI that is the address your users open Mailwoman at, ending in `/`, including any sub-path. For example `https://mail.example.com/`. For the device code flow, turn on **Allow public client flows**.
4. Under **API permissions**, add delegated Microsoft Graph permissions for the data the bridge reads and writes, and grant consent. The server requests the scope `https://graph.microsoft.com/.default` plus `offline_access`, so the token carries the permissions already consented on the registration. The plugin calls the mail folder and message endpoints, `sendMail`, `/me/events/delta`, `/me/calendar/getSchedule` and the To Do list and task endpoints, and the server reads `/me` to check the mailbox address. Consent at least:
   - `Mail.ReadWrite` and `Mail.Send`
   - `Calendars.Read`
   - `Tasks.ReadWrite`
   - `User.Read`
   - `offline_access`
5. Copy the **Application (client) ID** and, for a single-tenant registration, the **Directory (tenant) ID**.

The redirect URI cannot carry a query string. Mailwoman sends exactly `<public address><sub-path>/`.

## Confidential client (optional)

For a registration that has a client secret, set these environment variables on the server:

| Variable | Meaning |
|---|---|
| `MW_GRAPH_OAUTH_CLIENT_ID` | The application ID. It must equal the ID a user enters. |
| `MW_GRAPH_OAUTH_CLIENT_SECRET` | The secret for that application. |
| `MW_BRIDGE_OAUTH_CLIENT_SECRETS` | A JSON object mapping application IDs to secrets, for users who bring their own registrations. It takes precedence. Up to 64 KiB. |
| `MW_BRIDGE_OAUTH_CLIENT_ID`, `MW_BRIDGE_OAUTH_CLIENT_SECRET` | A pair shared by both bridges, used when no provider-specific pair matches. |

A secret is sent only with the application ID it belongs to. A public client needs none of these.

## Connect a mailbox

1. Sign in to Mailwoman as the mailbox owner, using your primary sign-in account. The section is not shown while a linked mailbox is active.
2. Open **Settings → Linked mailboxes**. The **Provider connection** section sits on that page.
3. Under **Provider**, choose **Microsoft Graph**.
4. Enter the **Application client ID**. If an administrator has already stored one for this mailbox, it appears as the placeholder.
5. Enter **Microsoft tenant (optional)**. Leave it empty to use `common`.
6. Choose **Connect securely** to sign in at Microsoft in the browser, or **Use a device code** to enter a code at the Microsoft page from another device. The device code page checks for approval on its own and offers **Cancel**.
7. After approval, the page shows **Provider connected. Reload to use the new connection.** Reload the page.

<!-- screenshot: settings-provider-connection | Settings > Linked mailboxes, Provider connection section with Provider set to Microsoft Graph, client ID and tenant filled | element: provider connection section | callouts: 1 provider select, 2 client ID, 3 tenant, 4 Connect securely, 5 Use a device code | stack M | note: needs a seed where bridge-graph is approved; otherwise mark "stack unavailable" -->

The server accepts the connection only when all of these hold. If one fails, nothing is stored and you see **The provider connection could not be completed. Check the app registration, bridge permissions and mailbox identity, then retry.**

- The Microsoft mailbox address equals the address of the mailbox you are signed in to.
- The sign-in returned a refresh token.
- The plugin is still approved and granted when the flow completes, and your session is still valid.
- The application ID has only letters, digits, `-`, `_` and `.`; the tenant has only letters, digits, `-` and `.`.

A browser flow expires after 10 minutes. A device code flow expires after the time Microsoft gives, at most 30 minutes.

## What works

| Area | Behaviour |
|---|---|
| Mail | Folders (nested folders included), delta sync, fetch, flags, move and send. |
| Free/busy | Looking up another person's free/busy asks Graph when no shared calendar answers. |
| Calendar and tasks | The plugin can read Graph calendars and To Do tasks, but the server does not copy them into the calendar and task screens. See [Calendars and tasks through a bridge](./pim.md). |

Mail changes are picked up by polling; the bridge reports no push connection.

## Limits

- Switching a mailbox that already has cached messages from one backend to another is refused with the message **This mailbox already contains mail from another backend. Provider migration is required before changing it. Your mail and connection were preserved.** Reconnecting the same provider keeps the cached mail.
- The connection enrolls an existing signed-in mailbox. It does not provide Microsoft sign-in to Mailwoman and does not create accounts.
- A domain that has an EWS backend selected cannot also be connected here; the server refuses the connection.
- The plugin contains code for message recall (a Graph beta action), voting and Focused Inbox, and for contacts and directory search. The server and web client do not call it, so none of these features are available. Graph has no Outlook reaction endpoint.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| The Provider connection section is missing. | A linked mailbox is active, or no bridge is approved. | Switch to the primary sign-in account; ask an administrator to approve the plugin. |
| **Connect securely** is disabled. | No application client ID is entered or stored. | Enter the **Application client ID**. |
| The connection fails after consent. | The Microsoft address differs from the mailbox address, or the registration returns no refresh token. | Sign in with the matching Microsoft account; keep `offline_access` consented. |
| The page says migration is required. | The mailbox already holds mail from IMAP or another backend. | Use a new mailbox for the provider connection. |

## Related

- [Mail provider bridges](./index.md)
- [Plugins](../plugins/index.md)
- [Linked mailboxes](../user/mailbox/linked-accounts.md)
- [Outbound connections](../security/egress.md)
