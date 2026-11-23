---
title: Integrations settings
screen: true
---

# Integrations settings

The **Integrations** category connects Mailwoman to other services and lets other programs use your account. It has three sections.

![The Settings screen on Nextcloud accounts, with markers 1 to 3 on the three sections of the Integrations category in the section list.](/screenshots/user-settings/integrations-overview.webp){.mw-shot}

1. **Nextcloud accounts**: link your own account on a Nextcloud instance your administrator approved.
2. **Package tracking**: save parcels and check their status.
3. **API access**: keys for scripts and for AI agents.
{.mw-callouts}

## Nextcloud accounts

![The Nextcloud accounts section with an instance approved by the administrator and the form to link your account.](/screenshots/user-settings/integrations-nextcloud.webp){.mw-crop}

Link your own account to a Nextcloud instance that your administrator approved. Credentials are stored encrypted for this mailbox account only. When no instance is enabled, the section says **No enabled Nextcloud instances are available. Ask your administrator to configure one.** and there is nothing to do.

### Link an account

1. In Nextcloud, open your personal security settings and create an app password. Copy its username and password.
2. In **Settings**, open **Nextcloud accounts**. Each approved instance is listed with its address and its state, **Not linked** to start with.
3. Select **Link account**, enter the **Nextcloud app-password username** and **Nextcloud app password**, and select **Verify and link**. Verification makes one bounded request, and rejected credentials are not retried.
4. The instance now shows **Linked and ready**. When the administrator changes the instance's configuration it shows **Instance configuration changed. Verify the link again.** with **Verify link again**. **Unlink account** removes the link.

If verification fails, the password field is cleared and an existing working link is not replaced.

For the administrator's side, see [Nextcloud](/integrations/nextcloud).

## Package tracking

![The Package tracking section when the administrator has not configured a provider: the privacy note and the unavailable notice.](/screenshots/user-settings/integrations-packages.webp){.mw-crop}

Package tracking is off by default. It lets you save packages and check their status with the tracking provider your administrator configured. Only pressing **Track or refresh** sends the tracking number and carrier to the provider. Message contents, account email addresses and package labels are never sent, and Mailwoman does not scan your mailbox or poll in the background.

### Before you start

Package tracking has to be configured and allowed by your administrator. Until then the section shows **Package tracking is disabled or has not been configured by your administrator.** and its switch and **Add package** button are unavailable. When it is available, you also opt in yourself with **Enable package tracking for my account**.

### Add and check a package

1. Select **Add package**.
2. Enter an optional **Label** (a private name such as a shop or order, which stays in Mailwoman), the **Tracking number** (letters, digits or hyphens) and the **Carrier code**, which is the exact provider carrier code such as `ups`, `fedex` or `usps`.
3. Select **Save package**. Saving only stores the package in your account; no tracking request is sent.
4. In the list, use **Track or refresh** on a package to ask the provider for its status. The list shows **Status** and **Estimated delivery**. Opening a package shows **Latest tracking events** with carrier time, location and event.

Updates are requested explicitly, at most once per package within the interval the administrator set. Provider estimates are not delivery guarantees, and carrier event times are shown as supplied. Removing a package takes it out of Mailwoman only; records the provider already holds are unchanged. See [Package tracking](/user/packages).

## API access

Create keys so that scripts, integrations or an AI agent can use your account. The section has two cards.

### API keys

![The API keys card with the Create key form open: a label, capabilities, surface, accounts, folders and constraints.](/screenshots/user-settings/integrations-api-keys.webp){.mw-crop}

API keys are scoped. Each key is hashed at rest, shown once and individually revocable; grant the least scope that works. Select **Create key**, then fill in:

| Field | What it does |
|---|---|
| **Label** | A name that lets you recognise the key later. Required. |
| **Capabilities** | **Read**, **Send** and **Delete**. Read is ticked to start with. |
| **Surface** | **Mail**, **PIM (calendar / tasks / notes / contacts)** and **Manage Sieve scripts**. Mail is ticked to start with. |
| **Accounts** | **All accounts**, or only the account you are in. |
| **Folders** | **All folders**, or a comma-separated list of folder ids. |
| **Expiry (RFC 3339, empty = no expiry)** | When the key stops working, for example `2026-12-31T00:00:00Z`. |
| **Rate limit (requests/min, empty = unlimited)** | A request cap for the key. |
| **IP allowlist (CIDR/IP, comma-separated; empty = any)** | Only these addresses may use the key. |

When you create the key, its secret is shown once with a **Copy** button; select **I have saved it** when you have stored it. The secret cannot be retrieved again. **Existing keys** lists each key with when it was created, when it was last used (or never used) and whether it is **Active** or **Revoked**; **Revoke** ends a key.

### MCP keys

![The MCP keys card: a label, a checkbox for each tool, and the Create MCP key button.](/screenshots/user-settings/integrations-mcp-keys.webp){.mw-crop}

An MCP key lets an AI agent use the Model Context Protocol to work with your account with exactly the tools you choose. Mail bodies returned to the agent are labelled untrusted input, and each tool is granted individually. Enter a **Label**, tick at least one tool, and select **Create MCP key**. The secret is shown once.

| Tool | What it allows |
|---|---|
| **Search mail** | Search messages. Results are labelled untrusted input. |
| **Read mail** | Read a message body. Bodies are labelled untrusted input. |
| **List folders** | List mailbox folders. |
| **Create drafts** | Create a draft message, never sent automatically. |
| **Send mail** | Queue a message to the Outbox for in-app confirmation. |
| **Read calendar** | Read calendar events. |
| **Propose events** | Propose, not commit, calendar events. |
| **Read tasks** | Read tasks. |
| **Write tasks** | Create or update tasks. |
| **Read contacts** | Read contacts. |

API keys and MCP keys are stored as records and have no entry in the settings reference. See [Model Context Protocol access](/security/mcp) and [API keys and OAuth](/security/api-keys-oauth).

## Troubleshooting

| What you see | Why | What to do |
|---|---|---|
| **No enabled Nextcloud instances are available.** | The administrator has not enabled an instance. | Ask your administrator. |
| **Verification or saving failed.** | The instance, app-password username or password is wrong. | Check them and enter the app password again; the field is cleared after a failure. |
| **Package tracking is disabled or has not been configured by your administrator.** | No provider is set up or your account is not allowed. | Ask your administrator. |

## Related

- [Personal settings](./index.md)
- [Package tracking](/user/packages)
- [Nextcloud](/integrations/nextcloud)
- [API keys and OAuth](/security/api-keys-oauth)
