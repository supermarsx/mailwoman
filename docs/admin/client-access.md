---
screen: true
---
# Client access

The **Client access** tab of the **Security centre** adds an optional step in front of the mail client: a private entry address, a pre-shared key, or both. People must pass this step before they see the mailbox sign-in page. It does not replace their password and does not change their accounts. Both controls are off by default.

![The Security centre on its Client access tab, below the notice that country features are on without a country database, with both entry controls off.](/screenshots/admin-access/client-access-tab.webp){.mw-shot}

Open it from **Security centre** > **Client access** (`{console}#/security/client-access`). The tab's form:

![The Client access tab with both controls off: Require a private client address with its address segment, Require a pre-shared key with a new key field, and the grant lifetime of 24 hours.](/screenshots/admin-access/client-access-form.webp){.mw-crop}

1. **Require a private client address**
2. **Client address segment**
3. **Require a pre-shared key**
4. **New pre-shared key**
5. **Client access grant lifetime (hours)**
{.mw-callouts}

**Current client entry:** shows the address people use to open the mail client.

## Before you start

- Viewing the tab needs a role with `settings.read`, saving needs `settings.write`; both for the whole deployment. Saving asks you to confirm it is you if you have not done so recently.
- The admin console is not affected by this step. Make sure you know the console address before you turn it on.
- Native apps and other programs that use the mail sign-in or JMAP routes are affected like browsers: they need the entry step too.

## Require a private address

1. Tick **Require a private client address**.
2. Enter a **Client address segment**: 4 to 64 lower-case letters, digits or hyphens, starting and ending with a letter or digit, for example `team-7f3k9q`. Names used by the application (such as `admin`, `api`, `jmap`, `login`, `mail`, `settings`) are refused, as is a segment that collides with the console address or the deployment's path prefix.
3. Click **Save client access**.

From then on, the mail client opens only through `https://<your server>/<segment>`. Anyone who opens the root address without having passed the entry step gets "Not found", and API requests are answered `403` with the code `clientAccessRequired`. The answer never reveals the segment.

Treat the segment as a shared secret that keeps casual visitors and scanners away, not as a sign-in: anyone who learns the address can use it.

## Require a pre-shared key

1. Tick **Require a pre-shared key**.
2. Enter the key in **New pre-shared key**: 16 to 256 bytes; spaces count. Leave the field empty to keep a key that is already stored.
3. Click **Save client access**.

People then see a page that asks for the key before the mail client loads. The key is stored only as a password hash and is never shown again; the tab says "A pre-shared key is stored. Its value is never returned." To delete a stored key, first untick **Require a pre-shared key**, then tick **Forget the stored pre-shared key** and save.

A wrong key counts as a failed sign-in from that address for the [temporary sign-in locks](./signin-locks.md), and the key page is subject to the same sign-in pacing as the mailbox sign-in.

With both controls on, people open the private address and then enter the key.

## The grant

After someone passes the step, the browser receives a cookie (HttpOnly, SameSite=Strict, Secure over HTTPS) that admits it for the **Client access grant lifetime (hours)**: 1 to 168, default 24. Returning to the private address with a valid grant goes straight to the mail client and does not extend the grant.

Saving any change on this tab ends all existing grants: everyone passes the step again. Existing mailbox sessions are not signed out.

## What is not covered

The step guards the mail client, the mailbox sign-in, JMAP and the other browser APIs. These keep their own authorization and do not need the step:

- the admin console;
- health, readiness and metrics addresses, static assets and the service worker;
- CalDAV/CardDAV (`/dav`), MCP (`/mcp`), the OAuth and identity provider routes (`/oauth`, `/oidc`) and `/.well-known`;
- the single sign-on callbacks and the inbound webhook address;
- links that carry their own signed token: remote images and files shown in messages, and read-tracking images.

It is not a firewall: it does not stop anyone from reaching these addresses, and it cannot take back data a browser has already downloaded.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| People get "Not found" on the usual address | **Require a private client address** is on. | Give them the private address. |
| Everyone has to enter the key again | A change was saved on this tab, or the grant lifetime passed. | Expected; ask them to enter it again. |
| The key page says "Please wait before trying again." | Too many attempts from the address, or the address is locked. | Wait, or see [Temporary sign-in locks](./signin-locks.md). |

## Related

- [Security centre](./security-center.md)
- [Temporary sign-in locks](./signin-locks.md)
- [Console path](../security/admin-panel.md#console-path)
