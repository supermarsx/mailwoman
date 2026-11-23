---
title: Maintenance
screen: true
---
# Maintenance

**Maintenance** (under **Operations** in the console navigation) has four tabs: **Maintenance mode**, **Database**, **Cache** and **Repair jobs**. Maintenance mode pauses user access while you work on the server. The other tabs run bounded clean-up and repair work on the database, the caches, the log files and the search index.

![The Maintenance mode tab: a status line saying webmail is accepting requests, the Pause user access switch, a message box, the retry delay and Save.](/screenshots/admin-ops/opsa-maintenance-mode.webp){.mw-shot}

## Before you start

- **Maintenance mode**, **Database** and **Cache** need the `maintenance.read` permission. Changing maintenance mode needs `maintenance.write` without a domain restriction; with only `maintenance.read` the form is read-only. **Repair jobs** needs `maintenance.run`.
- Saving maintenance mode asks for a recent confirmation that it is you (your password and second factor, or a passkey).

## Pause user access

Maintenance mode is off by default. When it is on, new webmail requests are refused with a message, but sessions, credentials, mail and accepted work are not touched.

1. Open **Maintenance**. The **Maintenance mode** tab says **Webmail is accepting requests** or **Maintenance mode is active**.
2. Tick **Pause user access**.
3. Under **Message for users**, type a plain-text message of up to 1,000 characters without control characters. Leave it empty to use the standard message.
4. Under **Suggested retry delay (seconds)**, enter a number from 1 to 86,400. It is sent in `Retry-After` responses. It does not end maintenance mode by itself.
5. Choose **Save** and confirm **Apply maintenance mode changes?**.

![The Maintenance mode form with markers 1 to 4 on the Pause user access switch, the message for users with a sample text, the retry delay and Save.](/screenshots/admin-ops/opsa-maintenance-mode-form.webp){.mw-crop}

1. **Pause user access** turns maintenance mode on or off.
2. **Message for users** is shown to people who are turned away.
3. **Suggested retry delay (seconds)** is the `Retry-After` value.
4. **Save** applies the change to every server that shares the database.
{.mw-callouts}

To end maintenance, clear **Pause user access** and save again. Sessions work again at once, subject to their normal expiry.

### What changes while it is on

- Requests to the mail routes of the web server get HTTP `503` with `Retry-After`, `Cache-Control: no-store` and a JSON body `{"code":"maintenance","message":…,"retryAfterSecs":…}`. This covers the webmail API and sign-in, JMAP, DAV, single sign-on, OpenID Connect, OAuth and MCP. With an empty message the body says **Mail service is temporarily unavailable for maintenance.**
- The administration console stays open with its normal sign-in and permissions, so you can recover.
- Still reachable: static files, the health checks, the public configuration, the sign-in challenge that the administrator login needs, branding assets, and sign-out and revocation routes.
- Only web requests are held. Background mail synchronization and submissions that were already accepted continue. Maintenance mode is not a way to shut the server down.
- The sign-in page shows your message, or **Webmail is temporarily paused for maintenance. Please try again later.** when the message is empty.
- The setting is for the whole deployment. It is not a per-user preference and it never turns itself off.

If the stored setting cannot be read, the server refuses user requests as if maintenance mode were on, and the tab says **The stored configuration could not be read. User access is paused. Review these replacement settings and save them to restore a valid configuration.**

## Other tabs

- **Database** and **Cache**: background maintenance of the database, online conversion, cached mail bodies and the public asset cache. See [Database growth and maintenance](/admin/database-maintenance).
- **Repair jobs**: three tools.
  - **Application log files**: **Cleanup** applies the configured age, file-count and size limits to the server's own log files at once, and **Rotate now** does the same and also rotates the current files. Compression may continue in the background.
  - **Offline database repair**: commands to copy and run on the deployment host. See [Database growth and maintenance](/admin/database-maintenance#offline-repair-of-an-sqlite-database).
  - **Search index**: shows how many stored messages are indexed (**… of … stored message(s) are indexed.**) and where the index lives. **Rebuild search index** reads the stored mail again and replaces what the index holds. The server also rebuilds the index at start when it does not match the stored mail. Mail of zero-access accounts is not indexed, so search finds nothing for those accounts. Only one rebuild runs at a time (**A rebuild is already running. Wait for it to finish.**). In proxy mode the server has no search index and the card says so.
  - **Re-thread mailbox**: choose an **Account** and **Re-thread mailbox**. It runs conversation threading again over the account's stored mail and gives the threads new keys. Threads may merge or split, and people may see conversations regrouped. It deletes no mail and can be repeated, and it never runs by itself. The result says how many messages were moved to a different thread.

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Pause user access** | Turns maintenance mode on. Stored as the deployment setting `maintenance.mode.v1`, not as a settings key. | Off | Administrators with `maintenance.write` |
| **Message for users** | Text shown to people who are turned away; at most 1,000 characters. | Empty (standard message) | Same |
| **Suggested retry delay (seconds)** | The `Retry-After` value, 1 to 86400. | 300 | Same |

There is no announcement before maintenance mode starts, and the **Message for users** above is the only text people see.

## Troubleshooting

- **Users get "Webmail is temporarily paused for maintenance."** Maintenance mode is on. Turn it off in the tab. If the tab says the stored configuration could not be read, save a valid configuration.
- **Save is refused as out of date.** Someone else changed the setting. The form needs the current revision (the server answers `412` for an old one and `428` when none was sent); reload the tab and apply your change again.
- **An administrator cannot save but can read.** The role has `maintenance.read` only.
- **A monitoring check fails during maintenance.** Health checks remain reachable; a failing check points at something other than maintenance mode.
- **The search index card says the index is kept in memory.** The server was started without a search directory, so the index is rebuilt at every start. Set `MW_SEARCH_DIR` to keep it on disk.

## Related

- [Database growth and maintenance](/admin/database-maintenance)
- [Observability](/admin/observability)
- [Diagnostics](/admin/diagnostics)
