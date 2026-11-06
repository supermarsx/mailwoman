# Retention limits and log rotation

Mailwoman keeps personal and behavioural data in a number of stores: history of sign-ins, devices, sends, tokens and administrator actions. This page lists each store, what it holds, how long rows are kept by default, which setting controls it and what removes the rows. The limits are configurable controls. Choosing a limit does not by itself make a deployment compliant with any law or contract; that is for the operator to assess.

All `retention.*` and `logging.*` keys are global. They have no domain or user scope, because the sweep and the log files belong to the whole server. They are edited in **Administration > Retention** (inline) or **Settings**, and every change is an audit entry. A limit of 0 keeps rows until they are deleted by hand; the console shows a warning for it. The reference for each key is in [settings.md](settings.md).

## The sweeper

One job, `retention_sweep`, covers every store marked "sweeper" below.

- It runs about once a day. The next run starts 24 hours after the last one plus a random spread of up to `retention.sweeper.jitterMinutes`, so several servers on one database do not sweep together. It can also be scheduled by hand on the Jobs page.
- It deletes in batches: `retention.sweeper.batchRows` rows per statement and at most `retention.sweeper.maxBatches` statements per store per run. A store with more old rows than that continues in the next run. Each statement is one `DELETE ... WHERE key IN (SELECT key ... LIMIT n)`, the same on SQLite and Postgres, so a long sweep holds no long lock.
- It is idempotent. A second run finds nothing more to remove.
- It never removes rows that are still needed: a submission that is pending, being sent, failed, uncertain or has an unfiled Sent copy; a job run that is still running; rows of accounts covered by `dsr.legalHold.subjects`. A row of a store that has a "keep" marker (limit 0) is skipped.
- When a run removes at least `retention.sweeper.vacuumAfterRows` rows it compacts the database (`VACUUM`, never `FULL` on Postgres). The weekly `vacuum` job is separate.
- Each run records the rows removed per store, the time, errors and a running total, shown in the console and exported as the Prometheus counters `mw_retention_rows_deleted_total{store}` and `mw_retention_errors_total{store}` and the gauge `mw_retention_last_run_timestamp_seconds`. A run that removed rows writes a `retention-swept` audit entry.
- "Run now" in the console first offers a dry run that only counts the rows each store would lose.

### IP addresses

After `retention.ipAddresses.days` days the address in authentication history (`signin_event.ip`) and in audit entries (`audit_log.ip`) is reduced and the row is kept. `retention.ipAddresses.mode` chooses between truncation (IPv4 /24, IPv6 /48) and a short keyed hash (`h:<12 hex>`) that still groups equal addresses. The reduction is idempotent. Audit rows that carry a hash chain are left unchanged so the chain stays verifiable. Stores that hold a keyed hash of the address instead of the address (request protection, known-source table) are not affected.

## Stores

"Sweeper" stores are removed by `retention_sweep` with the listed key. "Existing" stores keep the code and setting they had before; the sweeper does not duplicate them and the console only reports them.

| Store | Personal data | Default | Setting | How it is purged |
| --- | --- | --- | --- | --- |
| `passwordChangeAudit` | account id, change method, outcome, time | 365 days | `retention.passwordChangeAudit.days` | Sweeper, by age. |
| `ssoLoginAudit` | provider, hash of the subject, outcome, time | 90 days | `retention.ssoLoginAudit.days` | Sweeper, by age. |
| `dlpAudit` | account id, rule that matched, action, time | 365 days | `retention.dlpAudit.days` | Sweeper, by age. |
| `assistAudit` | actor, capability, scope summary, time | 365 days | `retention.assistAudit.days` | Sweeper, by age. |
| `alertEvents` | alert summary and delivery targets | 90 days | `retention.alertEvents.days` | Sweeper, by age. |
| `jobRuns` | who triggered a run, summary | 90 days | `retention.jobRuns.days` | Sweeper, by age; running jobs are kept. |
| `outbox` | send record: account, message id, identity, time, last error | 30 days | `retention.outbox.days` | Sweeper, by age, only delivered and cancelled sends. Pending, being-sent, failed, uncertain and unfiled-copy rows are never removed. |
| `sessions` | account, user agent, IP address, device label, last use | 90 days unused | `retention.sessions.idleDays` | Sweeper, by last use. Session timeouts apply first; this removes rows left behind. |
| `nativeSessions` | account, client type, last use | 90 days unused | `retention.sessions.idleDays` | Sweeper, by last use. |
| `rememberedDevices` | account, device label, token hash, expiry | 1 day after expiry | `retention.expiredCredentials.graceDays` | Sweeper, once expired for the grace period. |
| `signinChallenges` | peer key (hash), nonce, expiry | 1 day after expiry | `retention.expiredCredentials.graceDays` | Sweeper, once expired; consumed on use and trimmed on write. |
| `oidcTokens` | token hash, grant, expiry | 1 day after expiry | `retention.expiredCredentials.graceDays` | Sweeper, once expired; access tokens also trimmed on write. |
| `oidcCodes` | code hash, redirect URI, PKCE challenge | 1 day after expiry | `retention.expiredCredentials.graceDays` | Sweeper, once expired. |
| `oauthTokens` | token hash, client, account, scopes | 1 day after expiry | `retention.expiredCredentials.graceDays` | Sweeper, once expired. |
| `webhookNonces` | webhook id, nonce, time | 1 day after expiry | `retention.expiredCredentials.graceDays` | Sweeper, by time seen. |
| `sourceFailures` | source IP address, failure timestamps, lock expiry | 1 day after expiry | `retention.expiredCredentials.graceDays` | Sweeper, once expired. |
| `imageHandles` | image-proxy link handles (`rc.h.*` rows in settings): claims for one proxied image | 1 day after expiry | `retention.expiredCredentials.graceDays` | Sweeper, once expired; also trimmed when a handle is stored. |
| `authHistoryIp` | IP address on each authentication event | 90 days, then reduced | `retention.ipAddresses.days` | Sweeper, reduces the address and keeps the row. |
| `auditLogIp` | IP address on each audit entry | 90 days, then reduced | `retention.ipAddresses.days` | Sweeper, reduces the address and keeps the row (rows with a hash chain excepted). |
| `auditLog` | administrator and security actions, actors, IP address | keep until a policy is set | Audit retention page (per category, with legal hold) | Existing `audit_prune` job; writes a checkpoint when rows are removed. |
| `authHistory` | username attempted, result, IP address, user agent, country | 7 days (and `logs.auth.maxEntries` entries) | `logs.auth.retentionDays` | Existing; trimmed on write. |
| `requestMetrics` | route-class latency histograms (no person identifiers) | 30 days | `metrics.rollupDays` | Existing; trimmed when a bucket is saved. |
| `clientTiming` | anonymous browser load-time histograms; medians derive from them | 30 days | `metrics.rollupDays` | Existing; trimmed when a bucket is saved. |
| `metrics` | persisted performance statistics: counts and quantile sketches per minute, hour and day (image proxy latency, browser load times, rate-limit and request-protection decisions); no person identifiers | 48 hours of minutes, 60 days of hours, 400 days of days | `retention.metrics.minuteBucketsHours`, `retention.metrics.hourBucketsDays`, `retention.metrics.dayBucketsDays` | Rolled up by the sweeper: minute rows older than the first limit merge into hour rows, hour rows older than the second merge into day rows, day rows past the third are deleted. A day limit of 0 keeps day rows without limit (the console warns); the other two must be at least 1. See [metrics-persistence.md](metrics-persistence.md). |
| `clientErrors` | browser error reports: scrubbed error name, message, boundary, route template and stack frames, build, coarse browser, OS and viewport class; a keyed account hash only when `observability.client.includeAccountHash` is on | 30 days for occurrences, 180 days for groups | `retention.clientErrors.eventDays`, `retention.clientErrors.groupDays` | Pruned by the sweeper (occurrences first, then groups not seen since the group limit; at most 5000 groups are kept). See [observability.md](observability.md#client-errors). |
| `protectionEvents` | request-protection decisions keyed by a hash of the client | 30 days (fixed) | none | Existing; trimmed on write and on read. |
| `protectionSubjects` | counters per hashed client, account or admin | their own expiry | none | Existing; trimmed on write. |
| `sessionWatch` | session fingerprints for theft detection | 15 days | `security.theft.retentionDays` | Existing; periodic sweep. |
| `dsrRequests` | data-subject requests and their trail | 365 days after completion | `dsr.retention.completedDays` | Existing, owned by the data-subject request feature. |
| `openTracking` | open-tracking tokens and events | per setting | `privacy.openTracking.retentionDays` | Existing. |
| `diagnostics` | diagnostic run results | per setting | `diagnostics.retentionDays` | Existing; pruned after each run. |
| `securityNotices` | in-app security notices per account | per setting | `security.alerts.noticeRetentionDays` | Existing. |
| `usageCounters` | per-account usage counters | 400 days | `usage.retentionDays` | Existing. |
| `accountGeo` | countries an account signed in from | 365 days | `security.signin.accountGeoHistoryDays` | Existing. |
| `uploads` | uploaded attachment blobs not yet attached | 24 hours | none (the `gc_uploads` job parameter) | Existing `gc_uploads` job. |
| `logFiles` | server log files written by the server | 14 days | `logging.file.maxAgeDays` | Rotation, below. |

### Not swept, and why

- **Mail, calendars, contacts, notes, drafts and templates** are the user's own data, not logs. They stay until the user or an administrator deletes them, a retention rule the user set (or an organisation-wide rule, see below) acts on mail, or an erasure request removes them. Calendars, notes and contacts have no retention rules yet.
- **Account credentials and second factors** (password hashes, TOTP secrets, passkeys, recovery codes, push subscriptions, API keys) are live configuration of an account. They are removed with the account or by an erasure request, not by age.
- **Invitations and pre-provisioned sign-ins** expire by `signin.preprovision.ttlDays`; their records are removed with the account that used them.
- **Image proxy latency and statistics** (`remote_image_latency`, `remote_image_history`, tracker counts) are in process memory only. They hold no address, are kept 24 hours, and are lost on restart.
- **Search index** entries belong to the user's mail and are removed with the message or the account; there is no age limit.
- **Postgres or SQLite write-ahead logs and backups** follow the database's own settings. Database backups taken by `backup.*` keep rows that this page removes until the backup itself is deleted (`backup.retentionCount`).

## Retention classes

Every store above belongs to one of three classes. A class is a ceiling in days and a way to talk about a group of stores. The class of each store is in the console (**Administration > Retention**, classes) and in the table below.

| Class | What it holds | Ceiling setting | Default |
| --- | --- | --- | --- |
| Short | Operational data: sessions, tokens, remembered devices, sign-in challenges, source failures, image links, the outbox, alert events, job history, request and load-time metrics, diagnostics, uploads, log files. Stores: `alertEvents`, `jobRuns`, `outbox`, `sessions`, `nativeSessions`, `rememberedDevices`, `signinChallenges`, `oidcTokens`, `oidcCodes`, `oauthTokens`, `webhookNonces`, `sourceFailures`, `imageHandles`, `requestMetrics`, `clientTiming`, `metrics`, `diagnostics`, `uploads`, `logFiles`. | `retention.class.short.days` | 365 days |
| Medium | Security and sign-in history: `authHistory`, `authHistoryIp`, `auditLogIp`, `protectionEvents`, `protectionSubjects`, `sessionWatch`, `openTracking`, `securityNotices`, `usageCounters`, `accountGeo`. | `retention.class.medium.days` | 1095 days (3 years) |
| Long | Security audit and records of requests: `auditLog`, `passwordChangeAudit`, `ssoLoginAudit`, `dlpAudit`, `assistAudit`, `dsrRequests`. | `retention.class.long.days` | 2555 days (7 years) |

- **Cross-check.** The long class must not be below the medium class and the medium class must not be below the short class. A value that breaks this is raised to the class below it when it is used, and the console shows a warning.
- **Forever.** Only the long class can be set to 0, which keeps its data (including the archive) for ever. The console shows a warning for it. Choosing a period is a decision for the operator; this page does not say what period any law or contract requires.
- **`retention.class.enforce`** (off by default). With it off, each store keeps the limit set for it, and the class ceilings are shown for comparison; a store whose own limit is above its class (or 0) is marked in the console. With it on, each store's limit is held to its class ceiling and 0 counts as the ceiling. The expiry grace of tokens (`retention.expiredCredentials.graceDays`) is never changed by it.
- The age of the stored rows of each class is drawn in the console as bars (under 30 days, 30 to 90, 90 to 365, 1 to 3 years, over 3 years) for the stores that have an age column the sweeper reads.

## Long-term archive

With `retention.archive.enabled`, the stores that have an archive table (`passwordChangeAudit`, `ssoLoginAudit`, `dlpAudit`, `assistAudit`) do not delete rows older than their own limit. The sweeper moves them into sealed archive files instead. The hash-chained audit log keeps its own prune and checkpoint (see the audit retention page) and is not archived.

- **Files.** A file is gzip-compressed JSON lines: one header line, then one object per row. It is written under a temporary name and renamed, once. The directory is `retention.archive.directory` (a local directory, absolute or relative to the working directory; parent-directory segments are refused). There is no remote destination: the archive is a directory, and moving it to other storage is up to the operator.
- **Order.** A row leaves the hot table only after its file is on disk and its metadata and index rows are committed. A run that stops in between repeats only the delete on the next run; rows already in the index are not archived twice. `retention.archive.batchRows` rows go into one file and `retention.archive.maxSegments` files are written per store per run.
- **Seal and chain.** Each file has a SHA-256 hash and a seal, `HMAC-SHA256(key, previous seal | file hash | store | number | row count | first and last time | key generation)`. The key is derived from the server key and `retention.archive.keyGeneration`. The first file of a store links to the fixed value `genesis`. A changed file, a changed or removed metadata row, or a reordered file is reported as `contentChanged`, `sealMismatch`, `chainBroken` or `missing`.
- **Key rotation.** Raise `retention.archive.keyGeneration`. New files are sealed with the new generation; older files carry their own generation and still check. The seal key follows the server key, so losing the server key means the seals can no longer be checked.
- **Index.** Each archived row has an index entry (store, row key, time, account id), so a lookup by row id or by account opens one file. Row contents are only in the file.
- **Check job.** The `retention_archive` job (daily) verifies every file when `retention.archive.verifyDays` have passed since the last check, then removes files whose newest row is older than the long class. The outcome is shown in the console (files, rows, size, last check, files failing) and every failure or removal is an audit entry (counts only). **Check the archive now** runs the check at once.
- **Legal hold.** `retention.archive.legalHold` keeps every file whatever the long class says. Rows of accounts in `dsr.legalHold.subjects` are not archived or deleted by the sweeper.
- **Search, restore, export.** In the console an administrator can look up archived rows by row id or account id (each search is an audit entry), restore a file (its rows go back into the table; rows already there are skipped) and download a file as JSON lines.
- **Removing a file** deletes the file and its index rows and keeps its metadata row (state `purged`), so the chain of the files after it stays checkable.
- Database backups (`backup.*`) and copies of the archive directory keep data this page removes until those copies are deleted.

## User retention rules

Each person can set rules for their own mail in **Settings > Retention**, and start one from a folder's context menu (**Retention…**). Rules are kept in the database (`user_retention_rule`) and run by the `user_retention` job (daily, at most once per `retention.user.intervalHours` per account) and on request with **Run my rules now**. The job reaches the mailbox through the mail engine's JMAP methods (`Mailbox/get`, `Email/query`, `Email/get`, `Email/set`), the same way the web client does, so it works for accounts proxied to an IMAP server.

**What a rule can do.** Keep for ever; delete after N days (to Trash first, or permanently where allowed); move to Trash; move to Archive; move to a chosen folder; mark as read. Removing large attachments is available only when the administrator turns it on (see [Removing attachments](#removing-attachments) below). **What a rule can look at:** folders (with or without sub-folders, or a role such as Inbox, Junk, Trash), labels (keywords), sender addresses and domains, size, read state, attachment. **What a rule leaves alone:** flagged mail, unread mail (optional) and mail with a label such as `keep` (default). A rule that matches a message too young for it lets the next rule look; a `keep` rule or an exclusion ends the search with no change. Folder defaults apply to a folder and its sub-folders, deepest first, after the explicit rules; the Inbox has none unless the person adds one.

**Order of decisions** for a message: the Trash undo window, then the organisation-wide rules, then protected folders, then the person's rules and folder defaults.

- **Undo window.** A rule that deletes moves the message to Trash and adds the keyword `mw-ret-<day>`. It is purged `retention.user.undoDays` days after that day (a person may choose a shorter window). A message in Trash without that keyword is left to the organisation-wide Trash rule.
- **Dry run equals real run.** The preview and the run build the same plan from the same code, so "this would change 1,243 messages" is what the run does, up to `retention.user.maxPerRun`. The preview lists up to 20 examples (subject, sender, date, action); examples are shown to the person and never written to a log.
- **Batches and pacing.** `retention.user.batchMessages` messages per call (at most 200), `retention.user.pauseMs` between calls, `retention.user.maxPerRun` changes per account per run; the rest waits for the next run. A folder is read only when some rule could act on it, and only for messages older than the shortest age that could apply; one folder scan stops at 20,000 messages.
- **Idempotent.** A message that was moved, read or stamped no longer matches, so a second run finds nothing.
- **Skipped accounts.** An account covered by `dsr.legalHold.subjects` is skipped whole. An account under a restriction of processing is skipped unless `retention.user.runForRestricted` is on.
- **Records.** A run writes a row of counts (matched, changed, errors) and, when it changed something, an audit entry with the same counts. No subject, sender or address is stored.

| Setting | Scope | Default | What it controls |
| --- | --- | --- | --- |
| `retention.user.enabled` | global, domain | on | Whether rules can be saved and run. Off: existing rules do not run; the organisation-wide rules still do. |
| `retention.user.minDays` | global, domain | 7 | Fewest days before a deleting rule acts. Refused on save, raised when it runs. |
| `retention.user.maxDays` | global, domain | 0 | Most days a deleting rule may wait; 0 sets no ceiling. |
| `retention.user.maxRules` | global, domain | 50 | Rules per person. |
| `retention.user.allowPermanentDelete` | global, domain | off | Whether a rule may delete for good. Off: always Trash first. |
| `retention.user.undoDays` | global, domain | 7 | Days in Trash before a rule's purge. |
| `retention.user.mandatory` | global, domain | none | Organisation-wide rules, as `role:days` (for example `trash:30`, `junk:30`). They purge that folder, ignore a person's exclusions and pause switch, and cannot be changed by the person. Mail under a legal hold is skipped. |
| `retention.user.protectedFolders` | global, domain | `drafts` | Folders no rule touches, by role or name. A protected folder wins over an organisation-wide rule for the same folder. |
| `retention.user.templates` | global, domain | all six | Templates offered: `newsletters30`, `receipts365`, `notifications14`, `trash30`, `junk30`, `read-archive180`. They use the labels `newsletter`, `receipt` and `notification` and ask the person to check them. |
| `retention.user.runForRestricted` | global | off | Run for accounts under a restriction of processing. |
| `retention.user.intervalHours` | global | 24 | Hours between scheduled runs of one account. |
| `retention.user.batchMessages`, `retention.user.maxPerRun`, `retention.user.pauseMs` | global | 200, 5000, 100 | Batch size, changes per account per run, pause between calls. |
| `retention.user.allowStripAttachments` | global, domain | off | Whether people may create rules that remove attachments. See below. |
| `retention.user.strip.minAgeDays` | global, domain | 90 (at least 7) | Fewest days before attachments of a message may be removed. |
| `retention.user.strip.minAttachmentKb` | global, domain | 100 | Only attachments at least this large are removed. |
| `retention.user.strip.keepInlineImages` | global, domain | on | Keep images shown inside the message body. |
| `retention.user.strip.undoDays` | global, domain | -1 | Days the original stays in Trash; -1 follows `retention.user.undoDays`, 0 deletes it at once. |

### Removing attachments

The action **Remove large attachments** is **off by default**. While `retention.user.allowStripAttachments` is off: the action is not offered in the rule editor; saving a rule that uses it is refused with `stripDisabled` ("Removing attachments is disabled by your administrator"); rules saved while it was on are skipped when the job runs, and the run record counts them (`stripRulesSkipped`). Mail is not opened for them.

When it is on, a person can create a rule with the action. The rule's days must be at least `retention.user.strip.minAgeDays` (refused on save, raised when it runs). The pause switch of each rule works as for any rule. Per message:

1. The job asks the mail engine (`Email/mailwomanStrip`) for the message source and rebuilds it. Every header and every part that is kept stays byte for byte as it was, including `Message-ID`, `Date` and `References`. Each removed attachment part is replaced by a `text/plain` part reading "Attachment removed by retention rule on YYYY-MM-DD: name (type, size)", so the MIME structure stays valid. One `X-Mailwoman-Stripped` header per removed part (part path, type, size, name) is added; its presence is also the marker that stops a message being processed again.
2. Only attachments of at least `retention.user.strip.minAttachmentKb` (decoded size) are removed. Images shown inside the body (inline images) are kept when `retention.user.strip.keepInlineImages` is on. A `message/rfc822` attachment is removed whole. Calendar invitations and delivery reports are never removed. A message that is itself a single attachment is left alone.
3. Messages protected by a signature or encryption are never rewritten: S/MIME signed or enveloped mail, PGP/MIME, and mail with inline PGP text are skipped, as are messages over 128 MiB and messages whose MIME structure cannot be read. Such a message is marked `mw-strip-checked` so it is not fetched again.
4. The new message is imported into the same folder with the same flags and keywords, the same received date and the keyword `mw-stripped`. For IMAP accounts this is an APPEND with flags and internal date (UIDPLUS or IMAP4rev2 is required; otherwise the message is left alone). Only after the copy is acknowledged is the original moved to Trash with the dated keyword `mw-ret-<day>` (the undo window: `retention.user.strip.undoDays`, or `retention.user.undoDays` when -1) or, with a window of 0, deleted. A failure between the two steps leaves a duplicate, never a loss. The copy has the same `Message-ID` and `References`, so it threads with the conversation; until the original is purged the conversation can show both.
5. Not touched: mail in Trash, protected folders, flagged mail and other exclusions, accounts under a legal hold, accounts under a restriction of processing (unless `retention.user.runForRestricted`).

At most 300 messages are rewritten per account per run; the rest waits. The **preview** opens up to 100 of the matching messages (nothing is changed) and reports how many would lose attachments and how many bytes that frees; beyond those it counts by the messages' sizes and says the figure is an estimate. The run record and the audit entry carry counts only (messages rewritten, bytes freed, messages left alone); no subject, sender, file name or address is stored.

### Calendar events and notes

Two more opt-in kinds of retention run in the same job (`user_retention`), both **off by default**. A person sets them in **Settings > Retention**; the choice is stored in `user_pim_retention`.

| Setting | Scope | Default | What it controls |
| --- | --- | --- | --- |
| `retention.user.calendar.enabled` | global, domain | off | Whether people may delete ended events from calendars they choose. |
| `retention.user.calendar.minDays`, `retention.user.calendar.maxDays` | global, domain | 30, 3650 | Fewest and most days after an event ended. Refused on save, clamped when it runs. |
| `retention.user.notes.enabled` | global, domain | off | Whether people may remove notes they have not changed for a set time. |
| `retention.user.notes.minDays`, `retention.user.notes.maxDays` | global, domain | 30, 3650 | Fewest and most days since a note was last changed. |

- **Calendar.** The person picks calendars (only those are touched) and a number of days. Events in them that ended more than that many days ago are deleted through the mail engine (`CalendarEvent/set`), so a CalDAV calendar is told too. Recurring events have no end and are never touched. There is no archive calendar: events are deleted.
- **Notes.** Notes not changed for the chosen days are removed through the engine (`Note/set`). Pinned notes are never removed. The notes have no Trash, so before removal a copy of the whole note is sealed into `user_pim_retention` (kind `noteHold`) and kept for `retention.user.undoDays` days; the person restores it from **Removed notes you can restore** (`POST /api/retention/pim/notes/{id}/restore`) and the job purges it when the window ends. With an undo window of 0 the note is removed at once. A restored note is back in the database immediately; an open client shows it after its next refresh.
- **Off at the administrator.** While a switch is off the card shows only that it is turned off, saving an opt-in is refused (`pimDisabled`), and a choice saved earlier is not run; the run reports how many choices it skipped (`disabledSkipped`).
- **Safeguards.** Accounts under a legal hold, and restricted accounts (unless `retention.user.runForRestricted`), are skipped. At most 500 events and 500 notes are handled per account per run. **Preview** (`POST /api/retention/pim/preview`) reports the counts without changing anything. The run record and the audit entry (`user-pim-retention-run`) carry counts only; no title, name or address.
- **Contacts.** Mailwoman stores no interaction history or last-contacted log, so there is nothing to prune and no setting exists for it. Contacts themselves are never deleted by retention.

Because the keys have domain scope, the usual lock and default controls apply: an administrator can lock a value for the whole server or for a domain.

**What applies to a person.** `GET /api/retention/effective` (signed-in user) returns the retention that applies to the caller: the organisation-wide mail rules, the person's own rules with the days after the administrator's limits, and for each store that holds the person's data its class, days, whether it is kept for ever and whether the archive holds it afterwards. The same data comes from the function `user_retention::effective_for_account`, which other server code can call.

The console shows the number of people with rules, active rules, messages changed in the last 30 days and the last job.

## Log rotation

Log destinations the server controls are standard output, always, and optional files under `logging.file.directory`.

- **Files** (`logging.file.enabled`, off by default): the current file is `mailwoman.log`. It is closed and renamed to `mailwoman-<unix seconds>-<n>.log` when it reaches `logging.file.maxSizeMb`, or is `logging.file.rotateHours` old (0 rotates by size only). Rotated files are compressed with gzip when `logging.file.compress` is on. Files older than `logging.file.maxAgeDays`, files beyond `logging.file.maxFiles`, and the oldest files while the directory is over `logging.file.maxTotalMb` are deleted. The directory may be absolute or relative to the working directory; parent-directory segments are refused.
- **Standard output** is what a container platform collects. Mailwoman does not rotate it. Set the platform's limit instead, for example the Docker `json-file` driver options `max-size` and `max-file`, or the journald and Kubernetes log retention. The redaction and level controls below apply to standard output as well.
- **Levels**: `logging.level` is the default level and `logging.componentLevels` adds `component=level` entries (for example `mw_imap=debug`). Until the store is open, `MW_LOG` or `RUST_LOG` sets the levels; leaving both settings at their defaults keeps those start-up levels.
- **Redaction**, applied to every line before it is written anywhere: `logging.redaction.addresses` and `logging.redaction.ips` each take `full` (unchanged), `truncated` (first letter and domain; IPv4 /24 or IPv6 /48), `hashed` (a short hash with a key that exists only in process memory, stable while the process runs) or `none` (removed). `logging.redaction.tokens` cuts bearer tokens, API keys, passwords in URLs and the server's own secret values.

Settings take effect within about a minute of the change. The log policy is read after the store is open, so the first lines of a start-up use the defaults.

Redaction works on the text of a line. It cannot recognise an address spelled in an unusual way, or a person's name, so choosing `none` or `hashed` reduces what the logs contain without making them anonymous.
