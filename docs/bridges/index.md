---
title: Mail provider bridges
---

# Mail provider bridges

A bridge connects a mailbox to a provider that does not offer plain IMAP and SMTP access: Microsoft 365 and Exchange Online (Microsoft Graph), Gmail and Google Workspace (Gmail API), and on-premises Exchange (Exchange Web Services, EWS). Each bridge is a first-party plugin that runs in the server's WebAssembly sandbox. Once it is loaded, the mailbox behaves like any other account in the mailbox, search and settings.

Bridges need the server to run in **engine mode**. A server in proxy mode does not process mail itself and loads no bridge.

![The Plugins page of the administration console listing the Exchange, Gmail and Microsoft Graph bridges and the Rspamd classifier, all first-party, none loaded and none approved.](/screenshots/integrations/admin-plugins-bridges.webp){.mw-shot}

## What each bridge supports

| | [Microsoft Graph](./graph.md) | [Gmail API](./gmail.md) | [Exchange (EWS)](./ews.md) |
|---|---|---|---|
| Plugin id | `bridge-graph` | `bridge-gmail` | `bridge-ews` |
| Sign-in | OAuth 2.0: browser authorization (PKCE) or device code | OAuth 2.0: browser authorization (PKCE) | Password: HTTPS Basic or NTLMv2 |
| Where it is connected | **Settings → Linked mailboxes → Provider connection**, per mailbox | the same section | **Admin → Domains**, as a domain backend |
| Mail: folders, sync, flags, move, send | Yes | Yes (Gmail labels map to folders and keywords) | Yes |
| Free/busy lookup for other people | Yes | No | Yes |
| Calendar events and tasks on the calendar and task screens | No | No | No |

The Graph and EWS plugins contain more code than the server uses. Copying provider calendars and tasks into the calendar and task screens, voting, message recall, Focused Inbox sync, contact and directory search, room lists and Out-of-Office are written in the plugins, but nothing in the running server starts them, so they are not available to users. [Calendars and tasks through a bridge](./pim.md) gives the detail.

## How a bridge is allowed to run

A bridge loads for a mailbox only when all of these hold:

1. The plugin is approved, enabled and granted the `account-backend` and `net` capabilities in **Admin → Plugins**. See [Plugins](../plugins/index.md#approve-and-enable).
2. The permission switches for the plugin are on: **Plugins** and, for the Microsoft and Google bridges, **Microsoft bridge** and **Google bridge** under **Integration permissions** in the admin settings (`cap.plugins`, `cap.integration.microsoft`, `cap.integration.google`).
3. The mailbox is bound to the bridge, by a completed provider connection (Graph, Gmail) or by a domain configured with an EWS backend.

A change to the plugin registry takes effect for running bridges at once when it removes permission. Starting a bridge that was not running needs a server restart, except the EWS activation that happens when a user signs in on a configured domain.

## Related

- [Microsoft Graph](./graph.md), [Gmail API](./gmail.md), [Exchange (EWS)](./ews.md)
- [Plugins](../plugins/index.md): approval, grants, limits
- [Outbound connections](../security/egress.md): the **Approved plugin and provider requests** traffic class that bridge requests use
