---
title: Troubleshooting
---
# Troubleshooting

This page lists problems an administrator can run into, each with the message or behaviour you see, what causes it and what to do. Every entry comes from a message, a log line or a behaviour in the server or the console. The page of the feature has the full detail; the links point there.

Where to look first: the **Application logs** and **Authentication logs** tabs ([Logs](/admin/logs)) and the checks in [Diagnostics](/admin/diagnostics).

## Starting the server

### The server stops with `pool timed out while waiting for an open connection`, or with `SQLite startup recovery exceeded its <N>s budget`

A large SQLite database that was interrupted mid-write needs time to recover when the server starts. The time allowed to get the first connection covers that recovery, and the budget ran out first.

Wait and watch for the `[mw-store] SQLite startup:` lines in the log. Raise `MW_SQLITE_STARTUP_TIMEOUT_SECONDS` (whole seconds, 1 to 86400) and, if you start through `deploy.sh`, `--timeout` (10 to 3600). Do not delete the database's journal or WAL files. See [Database start-up](/admin/database-startup).

### `MW_SQLITE_STARTUP_TIMEOUT_SECONDS must be an integer from 1 to 86400`

The variable holds something else than a whole number in that range. Correct it.

### A debug build overflows its stack on the first request (Windows)

A debug build of the server can overflow the default 2 MiB worker-thread stack on Windows. Start it with `RUST_MIN_STACK=67108864` in the environment.

### The log says `mw-render binary not found; /api/sanitize will sanitize in-process (degraded, no jail)`

The server cannot find the separate program that renders messages in isolation, so it renders in the server process instead. **Diagnostics** shows the same thing as a failed **Isolated message rendering** check. Point `MW_RENDER_BIN` at the `mw-render` program. See [Diagnostics](/admin/diagnostics).

### The log says `search index: <directory> could not be opened (...); keeping the index in RAM for this run`

The directory set in `MW_SEARCH_DIR` could not be opened, for example because it holds a damaged index. For this run the index is kept in memory and rebuilt from the stored mail. If the directory holds a damaged index, delete it and restart. The **Search index** card in **Maintenance** (the **Repair jobs** tab) tells you whether the index is on disk or in memory and how many stored messages are indexed; **Rebuild search index** rebuilds it, and only one rebuild runs at a time. See [Maintenance](/admin/maintenance).

### The log warns that the admin console is at a default address

The console is at its default path, which is the first path a scanner tries. Change it under **Security centre**, **Console address**.

## Reaching the console

### The console answers `429 requestProtectionLimited` with `retryAfterSecs` of about 300

Request protection treats a page load of a path under the console address that is not one of the console's own addresses as a scan of a foreign admin path, and blocks the source address (by default for the time in `security.protection.blockSecs`; the rule is `security.protection.probePathMode`, **enforce** by default). Open the console at its own address and go from there with the navigation; do not type other paths after it. Wait for the block to end or clear it as described in [Request protection](/admin/request-protection).

### `Request protection is temporarily unavailable` (503, `requestProtectionUnavailable`)

Request protection could not write its state and is set to refuse in that case (`storageFailure: closed`) while a rule is on **enforce**. Check that the database is available. See [Request protection](/admin/request-protection).

### Users see `Webmail is temporarily paused for maintenance. Please try again later.`

Maintenance mode is on, or its stored record cannot be read (it then fails closed). Open **Maintenance**, **Maintenance mode** and turn **Pause user access** off, or save a valid record. If saving answers `Reload maintenance settings before saving.`, someone else changed it in between; reload the tab. See [Maintenance](/admin/maintenance).

## Monitoring

### `/metrics` answers 401

The token is missing or wrong, or `MW_METRICS_TOKEN` is not set: the endpoint is never open without a token. Set a token in the server's environment and send it. `deploy.sh status` prints a **Metrics:** line. See [Observability](/admin/observability).

### `/healthz` answers 404 to an external monitor

The health endpoint answers only internal clients by default (`security.health.mode`). Set `MW_HEALTH_TOKEN` and send it in the `X-Health-Token` header, or change the mode. Behind a reverse proxy without `MW_FORWARDED_MODE` and `MW_TRUSTED_PROXIES`, the server sees the proxy's address for every request, which can admit more clients than you intend. See [Observability](/admin/observability).

### Statistics are not kept across restarts

The statistics card says **Statistics are not being saved**: **Keep statistics across restarts** (`metrics.persist.enabled`) is off. See [Metrics persistence](/admin/metrics-persistence).

### The log shows `Sustained database growth: inspect Observability > Database`

The database grew unusually fast. Open **Observability**, **Database** to see which operations write the most. See [Observability](/admin/observability) and [Database maintenance](/admin/database-maintenance).

### Maintenance reports `Maintenance yielded to a busy database, its work budget, or WAL pressure.`

The write-ahead log is above `maintenance.database.maxWalMiB`, or the batch limits were reached. The work continues on the next run. See [Database maintenance](/admin/database-maintenance).

### `This database needs conversion before incremental reclamation can run.`

The SQLite file is not in incremental-vacuum mode. Convert it online from the **Database** tab, or run `deploy.sh maintenance --enable-incremental-vacuum`. See [Database maintenance](/admin/database-maintenance).

## Diagnostics

| What you see | Cause | What to do |
|---|---|---|
| **Database migrations** fails | The database is ahead of or behind the server's schema. | Restart so migrations apply. Never run an older server on a newer database. |
| **Server key** fails | `MW_SERVER_KEY` differs from the key the database was created with, so stored secrets cannot be opened. | Restore the original key. |
| **fail2ban client address** warns | Behind a proxy the server sees the proxy's address, so fail2ban would ban the proxy. | Set `MW_TRUSTED_PROXIES`. |
| **Public port** warns although users connect | The server cannot reach its own public address from inside (NAT hairpinning). | Check from outside the network. |
| `Results are shown, but history or audit storage failed. Check database availability before leaving this page.` | The report was produced but could not be saved. | Check that the database is available and run the checks again. |
| `429` when running checks | The run budget is 10 at once and 10 per minute for each administrator session (`rateLimit.diagnosticsBurst`, `rateLimit.diagnosticsPerMinute`). | Wait a minute. |

See [Diagnostics](/admin/diagnostics).

## Busy responses

### `Static asset loader is busy. Retry shortly.` (503)

The server is serving its own web files and every loader slot was in use. The response carries `Retry-After: 1` and `Cache-Control: no-store`; the browser's next request normally succeeds.

### `image proxy busy` (503)

No fetch slot for a remote image became free in time, or the server is at its ceiling for remote images. The response carries `Retry-After: 5`, `x-mw-remote-retryable: 1` and `no-store`, and the failure is not remembered, so loading the message again retries the image.

### `attachment preview workers are busy; retry shortly` (429)

All preview workers are in use. Open the preview again in a moment. See [Attachment preview](/deploy/attachment-preview).

### JMAP answers 429 with `Retry-After`

The account's concurrent requests were all in use and the wait in the queue (`ratelimit.queueMs`) ended. See [JMAP](/admin/jmap).

## Branding and fonts

| What you see | Cause | What to do |
|---|---|---|
| `The server refused a value. Check the fields and try again.` after uploading an image | The file is not PNG or JPEG, is over its size limit, or an installed-app icon is not exactly 192, 512 and 512 px. | Resize and upload again. See [Branding](/admin/branding). |
| `Someone changed it in the meantime. Reload and try again.` | Another administrator saved branding first. | Reload the screen and repeat your change. |
| The old logo still shows after **Restore default** | Branding images are cached by the browser as immutable. | Reload without the cache. |
| An installed app keeps the old name or icon | The browser or operating system has not refreshed the app. | The browser fetches the app manifest without caching, but the installed app is refreshed by the browser or operating system. Check again later. |
| `mailwoman fonts pull` fails with `fetching Google Fonts stylesheet` | The machine has no direct internet access; proxy variables are ignored. | Run it on a machine that has access. See [Fonts](/deploy/fonts). |

## Sending, drafts and receipts

| What you see | Cause | What to do |
|---|---|---|
| `Scheduled sending is turned off by your administrator.` or `That is too far ahead...` | `mail.send.maxDelayedSeconds` is 0 or shorter than the requested time. | Change the setting. See [Outbox and recall](/admin/outbox-recall). |
| `Recalling sent messages is turned off by your administrator...` | `mail.send.allowRecall` is off. | Turn it on. |
| Send is refused with `overQuota` | The account has too many queued or scheduled messages (`mail.send.maxQueued`). | Let some send or raise the limit. |
| The outbox shows **Not sent** after about an hour | The dispatcher stopped after 8 attempts. The **Sending attempts** setting (`outbox.maxAttempts`) does not change this; the server uses a fixed 8. | Resend from the outbox. |
| The outbox shows a delivery-unknown state | The server cannot tell whether the mail server accepted the message. It is never retried automatically. | Look in **Sent** before sending again. |
| `This account cannot currently confirm saves to its Drafts folder.` | Account drafts need a mail server with UIDPLUS or IMAP4rev2 and a mapped Drafts folder. | See [Account drafts](/deploy/account-drafts). |
| `This account already has 100 saved drafts.` | The per-account limit. | Delete drafts. |
| Outgoing mail fails with `outgoing header privacy conflicts with pre-signed DKIM/ARC headers` | The message was already signed over headers that the privacy setting would remove. | Turn outgoing header privacy off for that account. See [Outgoing header privacy](/deploy/outgoing-header-privacy). |
| `read receipts are disabled by administrator policy` or `delivery status notifications are turned off by the administrator` | The receipt policy is locked to never, or `receipts.dsnAllowed` is off. | Change the policy. See [Read receipts](/deploy/read-receipts). |
| An unsubscribe request fails with `bad-url`, `busy`, `refused`, `timeout` or `rejected` | The list's unsubscribe address or server rejected the request. | See [Mailing list actions](/deploy/mailing-list-actions). |
| `Package tracking is disabled or has not been configured by your administrator.` | The provider is off or has no key, `packages.allowed` is off, or the user has not opted in. | See [Package tracking](/admin/package-tracking). |

## Integrations

### Webhook URL refused

`The URL's host does not resolve to a public internet address, so deliveries to it would be refused. Private, loopback and link-local addresses are not allowed.` Use a public address. `Enter a full URL starting with https:// or http://.` means the URL has no scheme. See [Webhooks](/admin/webhooks).

### Inbound webhook answers 401 or 409

`invalid webhook signature` and `webhook timestamp outside the allowed window` (401): check both clocks and that the signature covers the timestamp, the nonce and the body. `webhook nonce already used` (409): send a fresh nonce for each delivery.

### Start-up refuses with `MW_WEBHOOK_INBOUND_SECRET is set but is no longer read`

Remove the variable and create inbound webhooks in the console instead.

### A plugin shows **Not loaded**

The line under it names the reason, for example that an administrator has not approved the plugin, that it is not enabled, or that it has no signature and unsigned plugins are not allowed. **Restart required to load this plugin.** means the plugin is approved and loads at the next start. See [Plugins](/admin/plugins).

### The AI assistant is not running

`The AI assistant is not running. The configured program ... is not listed in MW_ASSIST_LOCAL_PROGRAMS` means the local program is not on the allow list. A provider key that does not open with this server's key (`MW_SERVER_KEY`) needs to be entered again. When `MW_ASSIST_RATE_LIMIT_PER_MIN` is 0 every request is refused. See [AI assistant](/admin/ai-assistant).

### Import is refused

`Import requires a writable IMAP connection with UIDPLUS or IMAP4rev2 and a configured upload backend (MW_UPLOAD_DIR).` Check the mail server's capabilities and set `MW_UPLOAD_DIR`. See [Mail import and export](/deploy/mail-import-export).

### Reporting abuse answers 501

`abuse reporting is not configured (set MW_ABUSE_ADDRESS)`. See [Encryption and mail security server configuration](/deploy/crypto-security).

### Sign-in answers `clientServerNotAllowed`

The server chooses the mail server for each domain; a sign-in sends only the address and the password. See [IMAP and POP3 mail servers](/deploy/imap-pop3).

### Backups and settings transfer

Backup destinations refuse with `Use the configured rclone file; only RCLONE_CA_CERT is allowed`, `SFTP requires known_hosts_file with pinned host keys`, `NFS destination is not mounted as NFS; refusing local-disk fallback` or `appPasswordFile must be readable only by its owner (chmod 600)`. A settings import that stops with `Applied N policies, then stopped at <key>` met a concurrent change: reload and import again. See [Backups](/deploy/backups) and [Configuration transfer](/admin/configuration-transfer).

## Related

- [Logs](/admin/logs)
- [Diagnostics](/admin/diagnostics)
- [Observability](/admin/observability)
- [Maintenance](/admin/maintenance)
