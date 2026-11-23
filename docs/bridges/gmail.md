---
title: Gmail API bridge
---

# Gmail API bridge

The Gmail bridge connects a mailbox on Gmail or Google Workspace to Mailwoman through the Gmail API. It provides mail only: calendars, tasks and contacts are not read from Google through this bridge. For most Workspace users IMAP with OAuth also works; the API bridge adds Gmail's label model and sync by history ID.

The bridge plugin (`bridge-gmail`) can reach only `gmail.googleapis.com` and `oauth2.googleapis.com`. The server obtains and refreshes the OAuth token and gives the plugin a short-lived access token for each call.

## Before you start

- The server runs in engine mode and has a public HTTPS address (plain HTTP is accepted only for `localhost`, `127.0.0.1` and `[::1]`).
- An administrator has approved and enabled `bridge-gmail` and granted it `account-backend` and `net`. See [Plugins](../plugins/index.md#approve-and-enable).
- The administrator permission **Google bridge** is on (`cap.integration.google`, default on) together with **Plugins** (`cap.plugins`).
- You have a Google Cloud project with an OAuth client. Mailwoman does not register a client for you.

## Create the OAuth client in Google Cloud

1. In the Google Cloud console, create or pick a project and enable the **Gmail API**.
2. Configure the **OAuth consent screen**: **Internal** for a single Workspace organization, otherwise **External**.
3. Add the scope `https://www.googleapis.com/auth/gmail.modify`. This is the scope the server requests.
4. Create an **OAuth client ID** of type **Web application**. Add as an authorized redirect URI the address your users open Mailwoman at, ending in `/`, including any sub-path. For example `https://mail.example.com/`.
5. Copy the client ID and the client secret.
6. In a Workspace, an administrator may need to allow the app under **Security → API controls**.

## Give the server the client secret

Set these environment variables on the server:

| Variable | Meaning |
|---|---|
| `MW_GMAIL_OAUTH_CLIENT_ID` | The client ID. It must equal the ID a user enters. |
| `MW_GMAIL_OAUTH_CLIENT_SECRET` | The secret for that client. |
| `MW_BRIDGE_OAUTH_CLIENT_SECRETS` | A JSON object mapping client IDs to secrets, for users who bring their own project. It takes precedence. Up to 64 KiB. |
| `MW_BRIDGE_OAUTH_CLIENT_ID`, `MW_BRIDGE_OAUTH_CLIENT_SECRET` | A pair shared by both bridges, used when no provider-specific pair matches. |

A secret is sent only with the client ID it belongs to; another user's own project never receives it.

## Connect a mailbox

1. Sign in to Mailwoman as the mailbox owner, using your primary sign-in account. The section is not shown while a linked mailbox is active.
2. Open **Settings → Linked mailboxes**. The **Provider connection** section sits on that page.
3. Under **Provider**, choose **Gmail API**.
4. Enter the **Application client ID**.
5. Choose **Connect securely** and approve access at Google. The Gmail bridge offers only this browser flow; there is no device code option for it.
6. After approval, the page shows **Provider connected. Reload to use the new connection.** Reload the page.

<!-- screenshot: settings-provider-connection-gmail | Settings > Linked mailboxes, Provider connection with Gmail API selected (no tenant field, no device code button) | element: provider connection section | callouts: 1 provider select, 2 client ID, 3 Connect securely | stack M | note: needs approved bridge-gmail seed -->

The request uses an authorization code with an S256 PKCE challenge and asks for offline access with a consent prompt, so Google returns a refresh token. The server checks that the Gmail profile address equals the address of the mailbox you are signed in to, then stores the access and refresh tokens sealed. The PKCE verifier and tokens stay on the server. If Google returns no refresh token, the connection is refused and you must approve again.

## What works

| Area | Behaviour |
|---|---|
| Mail | Folders, sync, fetch, flags, move and send. |
| Labels | System labels map to mailbox roles (Inbox, Sent, Drafts, Trash, Junk). Starred and unread map to flags. A user label is shown both as a folder and as a keyword, so a message with several labels appears in each. A virtual **All Mail** folder spans every message. |
| Sync | The first sync reads the mailbox and remembers Gmail's history ID. Later syncs read the history from that ID. An expired history ID makes the server sync from scratch. |
| Calendar, tasks, contacts | Not supported by this bridge. |

## Limits

- Switching a mailbox that already has cached messages from one backend to another is refused with **This mailbox already contains mail from another backend. Provider migration is required before changing it. Your mail and connection were preserved.** Reconnecting Gmail keeps the cached mail.
- The connection enrolls an existing signed-in mailbox. It does not provide Google sign-in to Mailwoman.
- A domain that has an EWS backend selected cannot also be connected here.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Google reports a redirect URI mismatch. | The redirect URI in Google Cloud differs from the address you opened Mailwoman at. | Register that exact address, ending in `/`, including any sub-path. |
| The connection fails after consent. | The Gmail address differs from the mailbox address, or Google returned no refresh token. | Sign in with the matching Google account and approve offline access again. |
| The Provider connection section is missing. | A linked mailbox is active, or no bridge is approved. | Switch to the primary sign-in account; ask an administrator to approve the plugin. |

## Related

- [Mail provider bridges](./index.md)
- [Plugins](../plugins/index.md)
- [Linked mailboxes](../user/mailbox/linked-accounts.md)
- [Google's guide to synchronization](https://developers.google.com/workspace/gmail/api/guides/sync)
