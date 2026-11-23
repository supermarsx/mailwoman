---
screen: true
---
# Sessions

A session is what keeps a person signed in after they enter their password. This
page covers the **Sessions** tab of the **Security centre**, where you can see and
end mailbox sessions, how long sessions last, and what a server restart does to them.

![The Sessions tab of the Security centre with its filters and the session revocation controls.](/screenshots/admin-access/sessions-page.webp){.mw-shot}

The filters, the revocation controls and the session list:

![The Sessions tab with the filters and one mailbox session.](/screenshots/admin-access/sessions-list.webp){.mw-crop}

1. **Domain filter**, **Account ID filter** and **Client type filter** narrow the list.
2. **Preview session revocation** lists the sessions the filters match, before anything is ended.
3. The **×** button (**Revoke session**) ends one session.
{.mw-callouts}

## The Sessions tab

**Security centre** > **Sessions** (`#/security/sessions`) lists mailbox sessions
(web and app), most recently seen first, with the **Account**, **Device and client**,
**IP address**, **Last activity** and **Created**. At most 1,000 sessions are listed;
narrow the filters to see others. Console sessions are not listed here. (In the image above the two times are hidden, because the server stamps them with the real date.)

You need **sessions.read** to see the tab and **sessions.revoke** to end sessions.
A role limited to some domains sees and ends only sessions of accounts in those
domains.

### End one session

Select the **×** button (**Revoke session**) on its row and confirm. The account can sign in again.

### End several sessions

1. Set the filters, for example **Domain filter** `example.com`.
2. Select **Preview session revocation**. The list now shows only the sessions that
   match; nothing has been ended yet.
3. Select **Revoke these sessions** and confirm.

Only the sessions shown in the preview are ended; a session opened after the preview
is not included. You can also tick rows (or **Select all visible sessions**) and
select **Revoke selected sessions**; the result says how many were revoked, had
already ended, or could not be confirmed. Each ended session is recorded in the audit
log.

## How long a mailbox session lasts

By default a mailbox session ends after **seven days without foreground use**, and
has no fixed total lifetime.

- Opening the mailbox, returning to it, and clicking, typing, scrolling with the
  wheel or touching it in the foreground renew the seven days. The web app sends a
  renewal at most once every 30 seconds.
- Background requests (sync, notifications, a tab left open in the background) do
  not renew it.
- The session and CSRF cookies are kept for the remaining time, so closing and
  reopening the browser keeps the session.
- An expired or revoked session cannot be renewed.

You can shorten this, or add a fixed lifetime, in **Settings**. Both can be set for
the whole server, a domain or an account:

| Setting | Key | Default | Range |
|---|---|---|---|
| **Session idle timeout** | [`session.idleSecs`](/admin/settings#session.idlesecs) | 604800 (7 days) | 60 to 604800 |
| **Session lifetime** | [`session.absoluteSecs`](/admin/settings#session.absolutesecs) | 0 (no fixed lifetime) | 0, or 300 to 1296000 (15 days) |
| **Concurrent sessions** | [`session.maxConcurrent`](/admin/settings#session.maxconcurrent) | 10 | 1 to 100 |

The server's start-up options cap these: `MW_SESSION_IDLE_SECS` (default 604800)
is the longest idle time any setting can give, and `MW_SESSION_ABSOLUTE_SECS`
(default 0) adds a fixed lifetime for everyone. When both a setting and the start-up
option give a fixed lifetime, the shorter one applies, and never more than 15 days.

::: warning
With the defaults, a session that is used at least once a week never expires. If
that is too long for your users, set **Session lifetime**.
:::

Console sessions have their own limits: **Console idle timeout**
([`admin.session.idleSecs`](/admin/settings#admin.session.idlesecs), default 1800
seconds) and **Console session limit**
([`admin.session.absoluteSecs`](/admin/settings#admin.session.absolutesecs),
default 43200 seconds). The server applies them within 300 to 7200 seconds and 3600
to 86400 seconds; a larger saved value is used as the upper bound. See
[Admin console security](/security/admin-panel#sessions-cookies-and-csrf).

## Server restarts

Sessions are stored in the database, so a restart (a deploy, a crash, a container
re-created) does not sign anyone out. Any server that shares the database and the
server key can serve any session. The **Overview** tab of the Security centre shows
"Sessions survive restarts: yes" or "no", with the reason.

### What is stored

| State | Where | After a restart |
|---|---|---|
| Mailbox session | `sessions` table. The `mw_session` cookie carries the session id, and the table stores that same id. | Valid. Creation time and last activity are stored, so the deadlines do not move. |
| Mail server password and other upstream credentials | `sessions.sealed_creds`, sealed with the server key | Opened with the server key when next needed. |
| App (native) session | `native_sessions`, storing a hash of the bearer token, plus its `sessions` row | Valid. |
| Console session | `admin_sessions`, storing a hash of the session token, with both deadlines | Valid. The fixed deadline is set at sign-in and never extended. |
| CSRF token | Not stored: computed from the server key and the session id | Tokens issued before the restart still work if the server key is unchanged. |
| Remembered devices | `remembered_device`, storing a hash of the device token | Still recognised. |
| Sign-in history | `signin_event` | Unchanged. A restart does not record a sign-in or send a "new device" notice. |
| Single sign-on in progress | Sealed with the server key, single use, ten-minute expiry | A sign-in that was in progress still completes, also on another server. |

::: warning
Because the database holds mailbox session ids as they are and the sealed
credentials beside them, a copy of the database together with the server key gives
access to signed-in mailboxes. Protect backups of both. To end every session, set
**Keep sessions across restarts** off and restart, or revoke the sessions on this tab.
:::

### What is kept in memory only

| State | After a restart |
|---|---|
| "Confirm it is you" confirmations (five minutes) and their failed-attempt counter | The person confirms again. With several servers, a confirmation is known only to the server that received it. |
| A sign-in waiting for the second factor, passkey challenges, authenticator setup in progress | The person starts that sign-in or setup again. |
| Sign-in retry cooldowns and trust tiers | Counters start empty after a crash; trust tiers are written to the database on a clean stop. |
| Mail server connections, caches, the search index handle | Rebuilt when next needed. |
| Live update connections (WebSocket, EventSource) | The web app reconnects, waiting a little longer after each failure, and then reloads the lists once. |

While the server is away, the web app does not sign the person out: an answer of 502,
503 or 504, or no answer, shows "Reconnecting…" and retries for about a minute. Only
a `401` ends the session in the browser.

### The server key

Sessions are sealed with the server key: `MW_SERVER_KEY`, the file named by
`MW_SERVER_KEY_FILE`, or `<database>.server-key` next to a SQLite file. With
PostgreSQL, every server needs the same `MW_SERVER_KEY` or key file.

- If the key file is missing and the database exists, a new key is created and the
  stored sessions can no longer be opened: those people see the sign-in screen, and
  the Overview says why. Restore the original key file to bring the sessions back.
- An in-memory database, or a data directory that is wiped on restart, loses the
  sessions. Mount the data directory, or set `MW_SERVER_KEY`.
- A clock jump moves deadlines with it: forward past a deadline ends the session;
  backward lengthens live sessions.

## Settings

| Setting | Key | Default | Effect |
|---|---|---|---|
| **Keep sessions across restarts** | [`session.persistAcrossRestarts`](/admin/settings#session.persistacrossrestarts) | on | Off ends every stored mailbox, app and console session when the server starts. Read at start. |
| **Last-seen write interval** | [`session.lastActivityFlushSeconds`](/admin/settings#session.lastactivityflushseconds) | 15 | How often (0 to 300 seconds) a session's last-seen time is written. It does not extend the session. Read at start. |
| **Shutdown grace period** | [`server.shutdownGraceSeconds`](/admin/settings#server.shutdowngraceseconds) | 30 | How long a stopping server lets running requests finish (1 to 120 seconds). `MW_HTTP_SHUTDOWN_TIMEOUT_MS` overrides it. Read at start. |

The server stops on SIGTERM or Ctrl+C: it stops accepting connections, lets running
requests finish for up to the grace period, then closes the rest. Give your service
manager a longer stop timeout than the grace period.

## Related

- [Security centre](./security-center.md)
- [Admin console security](/security/admin-panel)
- [Backups](/deploy/backups)
