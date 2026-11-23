---
screen: true
---
# Retention

The **Retention** tab of **Compliance** shows how long each store keeps personal and behavioural data, what removes old rows, and how server log files rotate. This page lists every store with its setting and default, and says which code removes the data and when. The limits are settings you choose; choosing a limit does not by itself make a deployment meet any law or contract.

![The Retention tab of Compliance: the sweeper tiles, masked because they show the server's own clock, and Run the sweep.](/screenshots/admin-privacy/retention-overview.webp){.mw-shot}

## Before you start

- The tab needs `settings.read`, and `settings.write` to change a limit or run the sweep.
- Most limits are server-wide. The rules for people's own mail (`retention.user.*`) can also be set per domain.
- A limit of 0 keeps rows without limit for age limits; the console warns about it. For the "days after expiry" limits, 0 removes a row as soon as it expires.

::: warning Jobs that are not scheduled
The archive check (`retention_archive`), people's own retention rules and the organisation-wide mail rules (`user_retention`), and the removal of unattached uploads (`gc_uploads`) are jobs that run only when a scheduled task exists for them. The server creates no such task, and the console has no screen to create one. They run only when started by hand: **Check the archive now** on this tab, or **Run my rules now** by a person for their own mailbox. The sweeper and the other stores below run without a task.
:::

## The sweeper

The sweeper (`retention_sweep`) removes the rows of the stores marked "Sweeper" in the table below.

- It runs about once a day: 24 hours after the last run plus a random spread of up to `retention.sweeper.jitterMinutes`, so several servers on one database do not sweep together. With `retention.sweeper.enabled` off it does not run.
- It deletes in batches of `retention.sweeper.batchRows` rows (500), at most `retention.sweeper.maxBatches` batches (20) per store per run. A store with more old rows continues in the next run.
- It keeps rows that are still needed: a send record that is still pending or failed, a running job, and rows of accounts listed in **Legal holds** ([`dsr.legalHold.subjects`](/admin/settings#dsr.legalHold.subjects)). The legal hold protects only stores whose rows carry an account; it does not protect single sign-on records, assistant records, alerts, job history, sign-in challenges, sign-in provider codes and tokens, webhook nonces, source failures or image handles, nor the IP reduction below.
- When a run removes at least `retention.sweeper.vacuumAfterRows` rows (100,000), it compacts the database.
- Each run records the rows removed per store, exports the Prometheus counters `mw_retention_rows_deleted_total{store}` and `mw_retention_errors_total{store}`, and writes a `retention-swept` audit entry when it removed rows.
- The sweeper also rolls up stored metrics, prunes client error reports and moves rows into the archive (see below).

### Run the sweep by hand

1. On the **Retention** tab, under **Run the sweep**, select **Preview (dry run)**. It counts the rows each store would lose and changes nothing.

   ![The Run the sweep section after a preview, with markers on Preview (dry run) and Run now.](/screenshots/admin-privacy/retention-dry-run.webp){.mw-crop}

   1. **Preview (dry run)**.
   2. **Run now**, offered after a preview.
   {.mw-callouts}

2. Select **Run now** to run the sweep. **Refresh** updates the tiles when it has finished.

### IP addresses

After `retention.ipAddresses.days` (1095) days, the IP address in authentication history and in audit entries is reduced and the row is kept. `retention.ipAddresses.mode` chooses truncation (IPv4 /24, IPv6 /48, the default) or a short keyed hash (`h:` and 12 hexadecimal characters). The hash key is stored in the same database. Audit entries that are part of the sealed audit chain are not reduced, so their addresses stay in full.

## Stores

![The top of the Stores table on the Retention tab: each store, what it holds, its limit, its row count and what removes its rows.](/screenshots/admin-privacy/retention-stores.webp){.mw-crop}

"Sweeper" rows are removed by the sweeper. "Its own job" rows are removed by the feature that owns them, at the time given.

| Store (console name) | Id | Setting | Default | What removes the rows |
| --- | --- | --- | --- | --- |
| Password changes | `passwordChangeAudit` | `retention.passwordChangeAudit.days` | 365 days | Sweeper |
| Single sign-on logins | `ssoLoginAudit` | `retention.ssoLoginAudit.days` | 365 days | Sweeper |
| Data-loss-prevention matches | `dlpAudit` | `retention.dlpAudit.days` | 1825 days | Sweeper |
| Assistant access | `assistAudit` | `retention.assistAudit.days` | 730 days | Sweeper |
| Fired alerts | `alertEvents` | `retention.alertEvents.days` | 1095 days | Sweeper |
| Background job history | `jobRuns` | `retention.jobRuns.days` | 90 days | Sweeper; running jobs are kept |
| Send records | `outbox` | `retention.outbox.days` | 30 days | Sweeper; pending and failed sends are kept |
| Browser sessions | `sessions` | `retention.sessions.idleDays` | 1095 days unused | Sweeper |
| Native client sessions | `nativeSessions` | `retention.sessions.idleDays` | 1095 days unused | Sweeper |
| Remembered devices | `rememberedDevices` | `retention.rememberedDevices.graceDays` | 365 days after expiry | Sweeper |
| Sign-in challenges | `signinChallenges` | `retention.expiredCredentials.graceDays` | 1 day after expiry | Sweeper |
| Sign-in provider tokens | `oidcTokens` | `retention.expiredCredentials.graceDays` | 1 day after expiry | Sweeper |
| Sign-in provider codes | `oidcCodes` | `retention.expiredCredentials.graceDays` | 1 day after expiry | Sweeper |
| OAuth tokens | `oauthTokens` | `retention.expiredCredentials.graceDays` | 1 day after expiry | Sweeper |
| Webhook nonces | `webhookNonces` | `retention.expiredCredentials.graceDays` | 1 day after expiry | Sweeper |
| Failures per source address | `sourceFailures` | `retention.sourceFailures.graceDays` | 365 days after expiry | Sweeper |
| Image link handles | `imageHandles` | `retention.imageHandles.graceDays` | 365 days after expiry | Sweeper |
| IP in authentication history | `authHistoryIp` | `retention.ipAddresses.days` | reduced after 1095 days | Sweeper (reduces, keeps the row) |
| IP in audit entries | `auditLogIp` | `retention.ipAddresses.days` | reduced after 1095 days | Sweeper (reduces, keeps the row; sealed entries excepted) |
| Audit log | `auditLog` | the audit retention policy | kept until a policy is saved | The `audit_prune` job, created when a policy with a number of days is saved |
| Authentication history | `authHistory` | `logs.auth.retentionDays` | 1095 days, and at most 50,000 entries | On every write, and when the history is viewed |
| Request metrics | `requestMetrics` | `metrics.rollupDays` | 1095 days | When a new bucket is saved |
| Browser load times | `clientTiming` | `metrics.rollupDays` | 1095 days | When a new bucket is saved |
| Stored metrics | `metrics` | `retention.metrics.dayBucketsDays` | minutes 48 hours, hours 60 days, days 1095 days | Rolled up by the sweeper |
| Client error reports | `clientErrors` | `retention.clientErrors.eventDays` | 30 days (groups 180 days) | Sweeper |
| Request protection decisions | `protectionEvents` | none | 30 days | Request protection |
| Request protection counters | `protectionSubjects` | none | their own expiry | Request protection |
| Session watch | `sessionWatch` | `security.theft.retentionDays` | 365 days | Hourly, by the session watch |
| Operational incidents | `operationalIncidents` | `incidents.retention.days` | 2555 days after closing | The incidents worker; paused while any legal hold exists |
| Email notifications | `complianceMail` | `retention.complianceMail.days` | 730 days | The compliance email worker, every 30 seconds; paused while any legal hold exists |
| Data requests | `dsrRequests` | `dsr.retention.completedDays` | 1825 days after finishing | The data request worker |
| Open tracking | `openTracking` | `privacy.openTracking.retentionDays` | 365 days | Only when open tracking is used |
| Diagnostic runs | `diagnostics` | `diagnostics.retentionDays` | 180 days | After a diagnostic run, or when the history is viewed |
| Security notices | `securityNotices` | `security.alerts.noticeRetentionDays` | 730 days | Only when another notice is written to the same account |
| Usage counters | `usageCounters` | `usage.retentionDays` | 730 days | Nothing: no code reads this setting |
| Sign-in countries | `accountGeo` | `security.signin.accountGeoHistoryDays` | 365 days | The per-account country history of the sign-in country policy (country codes and dates only) |
| Unattached uploads | `uploads` | none | 1 day | The `gc_uploads` job, which is not scheduled (see above) |
| Server log files | `logFiles` | `logging.file.maxAgeDays` | 365 days | Log rotation (see below) |

Mail, calendars, contacts, notes, drafts and templates are people's own data and are not removed by age, except by the rules described in [People's own retention rules](#user-retention-rules). Account credentials and second factors are removed with the account. Database backups, write-ahead logs and copies of the archive directory keep rows this page removes until those copies are deleted.

## Retention classes

Every store belongs to a class with a ceiling in days.

| Class | Stores | Setting | Default |
| --- | --- | --- | --- |
| Short: operational data | `complianceMail`, `alertEvents`, `jobRuns`, `outbox`, `sessions`, `nativeSessions`, `rememberedDevices`, `signinChallenges`, `oidcTokens`, `oidcCodes`, `oauthTokens`, `webhookNonces`, `sourceFailures`, `imageHandles`, `requestMetrics`, `clientTiming`, `metrics`, `clientErrors`, `diagnostics`, `uploads`, `logFiles` | `retention.class.short.days` | 365 days |
| Medium: security and sign-in history | `authHistory`, `authHistoryIp`, `auditLogIp`, `openTracking`, `securityNotices`, `usageCounters`, `accountGeo`, `protectionEvents`, `protectionSubjects`, `sessionWatch` | `retention.class.medium.days` | 1095 days |
| Long: audit records and records of requests | `passwordChangeAudit`, `ssoLoginAudit`, `dlpAudit`, `assistAudit`, `auditLog`, `dsrRequests`, `operationalIncidents` | `retention.class.long.days` | 2555 days |

A class may not be shorter than the class below it; a value that is raises to the class below when used. Only the long class can be 0, which keeps its data without limit.

**Hold each store to its class ceiling** (`retention.class.enforce`, off by default). With it off, each store keeps its own limit and the ceilings are shown for comparison. With it on, the sweeper's stores, `complianceMail` and `operationalIncidents` are held to their ceiling. The other stores are not changed by it, although the console's class view and the retention people see in their settings show them with the ceiling applied. The "days after expiry" limits are never changed by it.

## Long-term archive

With **Archive aged rows of the long class** (`retention.archive.enabled`, off by default), rows of `passwordChangeAudit`, `ssoLoginAudit`, `dlpAudit` and `assistAudit` that pass their limit are moved by the sweeper into archive files instead of being deleted. A file holds gzip-compressed JSON lines encrypted with the server key, and each file is sealed with an HMAC chain so a changed, missing or reordered file is detected. Files are written to `retention.archive.directory` on the local disk.

**Check the archive now** checks every file and removes files whose newest row is older than the long class, unless **Legal hold on the archive** is on. The same check is the `retention_archive` job, which is not scheduled (see above), so files are only checked and removed when you start it. The archive keeps rows of people who were erased until their file is removed. You can look up archived rows by row id or account id, restore a file, or download it; each lookup is recorded in the audit log.

## User retention rules

People can set rules for their own mail under **Settings > Mail > Retention** (see [Mail settings](/user/settings/mail#retention)): keep, delete after a number of days (to Trash first), move to Trash, Archive or a folder, mark as read, and, when you allow it, remove large attachments. Flagged mail is never touched, and mail with a `keep` label is left alone by default. `retention.user.mandatory` adds organisation-wide rules per folder role (for example `trash:30`) that people cannot change.

The rules run through the mail engine, so they act on the mailbox on the upstream mail server, including accounts on an IMAP server. They run only when the `user_retention` job runs, which is not scheduled (see above), or when the person selects **Run my rules now**; organisation-wide rules therefore also run only for people who start a run. Each run changes at most `retention.user.maxPerRun` messages per account. Accounts listed in **Legal holds** are skipped, and so are restricted accounts unless **Run user rules for restricted accounts** is on.

Removing attachments rewrites the message: each removed part is replaced by a short text part, an `X-Mailwoman-Stripped` header is added, and the original is moved to Trash after the copy is stored. Signed or encrypted messages and messages over 128 MiB are never rewritten. At most 300 messages are rewritten per account per run.

| Setting | Default | Scope |
| --- | --- | --- |
| `retention.user.enabled` | on | domain |
| `retention.user.minDays` / `retention.user.maxDays` | 7 / 0 (no ceiling) | domain |
| `retention.user.maxRules` | 50 | domain |
| `retention.user.allowPermanentDelete` | off | domain |
| `retention.user.undoDays` | 7 | domain |
| `retention.user.mandatory` | none | domain |
| `retention.user.protectedFolders` | `drafts` | domain |
| `retention.user.allowStripAttachments` | off | domain |
| `retention.user.strip.minAgeDays` / `retention.user.strip.minAttachmentKb` | 90 / 100 | domain |
| `retention.user.calendar.enabled` / `retention.user.notes.enabled` | off / off | domain |
| `retention.user.runForRestricted` | off | server |
| `retention.user.intervalHours` | 24 | server |
| `retention.user.batchMessages` / `retention.user.maxPerRun` / `retention.user.pauseMs` | 200 (at most 1000) / 5000 / 100 | server |

## Log rotation

The server always writes its log to standard output. With `logging.file.enabled` on (off by default), it also writes `mailwoman.log` in the log directory and rotates it by size and age; rotated files older than **Keep rotated files (days)** (`logging.file.maxAgeDays`, 365) are deleted. Standard output is not rotated by the server: set the limit in your container platform (for example the Docker `json-file` options `max-size` and `max-file`).

::: warning Addresses in logs
**Email addresses in logs** (`logging.redaction.addresses`) and **IP addresses in logs** (`logging.redaction.ips`) are `full` by default, so log lines carry addresses and IP addresses unchanged. Other values are `truncated`, `hashed` and `none`. Redaction works on the text of a line and cannot recognise names or unusual spellings.
:::

## Options

The settings named on this page are in the [settings reference](/admin/settings), with their ranges. The defaults of the store limits are in the [Stores](#stores) table.

| Setting | Default |
| --- | --- |
| [`retention.sweeper.enabled`](/admin/settings#retention.sweeper.enabled) | on |
| [`retention.sweeper.batchRows`](/admin/settings#retention.sweeper.batchRows), [`retention.sweeper.maxBatches`](/admin/settings#retention.sweeper.maxBatches) | 500, 20 |
| [`retention.sweeper.vacuumAfterRows`](/admin/settings#retention.sweeper.vacuumAfterRows) | 100000 |
| [`retention.ipAddresses.mode`](/admin/settings#retention.ipAddresses.mode) | truncate |
| `retention.metrics.minuteBucketsHours`, `retention.metrics.hourBucketsDays` | 48, 60 |
| `retention.clientErrors.groupDays` | 180 |
| `retention.class.enforce` | off |
| `retention.archive.enabled`, `retention.archive.legalHold` | off, off |

## Related

- [Data requests](data-requests.md)
- [Compliance](compliance.md)
- [Logs](logs.md)
- [Metrics persistence](metrics-persistence.md)
