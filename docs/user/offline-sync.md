---
title: Sync and offline access
screen: true
---

# Sync and offline access

Mailwoman can keep an encrypted copy of your mail, calendars, tasks, notes and contacts
in the browser on this device. With that copy you can read what was downloaded when the
connection is down. Changes you make while offline are queued and sent when the
connection is back. Offline access is off until you turn it on, and it is meant for a
device whose browser profile you trust.

![The Keyboard and display settings page scrolled to the Offline cache card, with Offline access unticked.](/screenshots/user-features/offline-screen.webp){.mw-shot}

## Turn on offline access

Open **Settings**, then **Keyboard & display**, and find the **Offline cache** card.

1. Tick **Offline access**.
2. Choose what to keep with the other controls (below).

![The Offline cache card in Keyboard and display settings, with markers on Offline access, the header window, message bodies, attachments, the cache budget and the keep-for days.](/screenshots/user-features/offline-settings.webp){.mw-crop}

1. **Offline access** keeps encrypted copies of cached mail, calendars, tasks, notes and
   contacts on this device. Only cached content is available offline; queued changes
   wait for a verified connection.
2. **Offline header window** is how many days of message headers to download.
3. **Message bodies** is when to download message bodies.
4. **Attachments** is when to download attachments.
5. **Cache budget (MB)** limits the size of the cache.
6. **Keep for (days)** expires ordinary cached items after that many days.
{.mw-callouts}

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Offline access** ([`offline.enabled`](/admin/settings#offline.enabled)) | Keeps encrypted copies on this device | Off | You, unless an administrator locks it |
| **Offer persistent browser storage** ([`offline.requestPersistence`](/admin/settings#offline.requestpersistence)) | Offers the browser a request to protect the cache from automatic eviction | On | You, unless locked |
| **Offline header window** ([`sync.headersWindow`](/admin/settings#sync.headerswindow)) | **30 days**, **90 days**, **365 days** or **All available mail** | **90 days** | You, unless locked |
| **Message bodies** ([`sync.bodies`](/admin/settings#sync.bodies)) | **When opened**, or **Prefetch for offline use** | **When opened** | You, unless locked |
| **Attachments** ([`sync.attachments`](/admin/settings#sync.attachments)) | **When opened**, **Inline images**, or **All permitted attachments** | **When opened** | You, unless locked |
| **Cache budget (MB)** ([`offline.budgetMb`](/admin/settings#offline.budgetmb)) | The size limit for the offline cache | 250 | You, unless locked |
| **Keep for (days)** ([`offline.retentionDays`](/admin/settings#offline.retentiondays)) | Age after which ordinary cached items expire | 30 | You, unless locked |
| **When the budget is reached** ([`offline.eviction`](/admin/settings#offline.eviction)) | **Remove least recently used**, **Remove oldest first**, or **Only when I clear it** | **Remove least recently used** | You, unless locked |

The header window decides what is downloaded ahead of time, not what you can browse:
older mail stays available while you are online. Flagged and pinned messages are
downloaded even when they are older than the window. Attachments you prefetch are
limited to 10 MiB each. A file you attach while offline can be up to 50 MiB.

The card also shows how much browser storage the site uses, a **Request persistent
storage** button, and a line such as "Cached: 0 bytes in 0 items · Protected: 0 bytes".
The browser decides the quota and may grant persistence without showing a dialog.

::: warning
**Clear the offline cache now** removes the downloaded mail and other read caches from
this device. Drafts you have not sent and queued changes are kept. Clearing the
browser's site data removes everything, including those.
:::

## Working without a connection

When the connection drops, a banner says "You are offline — changes sync when you
reconnect", and "Working offline — N changes queued" tells you how many changes are
waiting. Mail, calendar, tasks, notes and contacts that were downloaded stay readable.

Composing a message, filing and flagging messages, and edits to calendar events, tasks,
notes and contacts are queued on the device and applied when the connection returns.
Queued changes wait for a verified connection. If your session expired, the banner says
"Your session expired — please sign in again". On a device where Mailwoman cannot sync in
the background, the notice "Keep Mailwoman open to sync queued changes on this device."
can appear. The desktop app can wake up in the background; see
[Desktop and mobile apps](/user/apps-desktop-mobile).

### What is stored for an offline start

The first time Mailwoman opens in a browser, it stores the files it needs to start
without a connection: the sign-in screen, the mailbox, reading and writing messages, the
background workers and the HTML sanitizer. With them you can start Mailwoman offline into
the mailbox, read the mail already downloaded to this device, write messages and have
queued ones sent when the connection is back, also after the tab was closed where the
browser supports background sync. A screen you have not opened online, such as Settings,
Calendar, Contacts or the PDF viewer, needs a connection the first time; Settings and the
calendar, tasks, notes and contacts modules are normally downloaded a few seconds after the
mailbox opens, unless the browser asks to save data. Encrypted (OpenPGP or S/MIME) mail
opens offline only on a device that has already opened encrypted mail online, which
downloads the decryption module. An administrator can have every file stored on the first
visit instead ([`web.offline.precacheMode`](/admin/settings#web.offline.precachemode)).

### Drafts

While you write, the message editor keeps a recovery copy in the browser. The status at
the bottom of the editor reads **Saved in this browser**. If the draft could not be
saved to your account, the editor says "Draft was not saved. Your local recovery copy
remains available." Closing a message that holds encrypted or decrypted text discards it
instead of saving it as a draft; see [Encryption keys](/user/encryption-keys).

## Unstable connections

When the live connection to the server is lost three times within 60 seconds, a notice
says "Connection unstable — frequent disconnections. Waiting for a stable connection."
A mailbox that keeps reconnecting is marked **Unstable connection** in its status. The
notice clears after 60 seconds without a loss, and signing out resets it. It appears
only while Mailwoman is open.

## Troubleshooting

- **Offline access cannot be turned on.** "Encrypted storage could not be accessed. Try
  again after checking browser storage permissions." Check that the browser allows this
  site to store data.
- **Offline changes are unavailable.** "Offline changes are unavailable until this
  session has verified storage authority. Reconnect and sign in again."

## Related

- [Settings: interface](/user/settings/interface)
- [Desktop and mobile apps](/user/apps-desktop-mobile)
- [Calendar](/user/calendar)
- [Unified mail](/user/unified-mail)
