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

- **Mail, calendars, contacts, notes, drafts and templates** are the user's own data, not logs. They stay until the user or an administrator deletes them, or an erasure request removes them.
- **Account credentials and second factors** (password hashes, TOTP secrets, passkeys, recovery codes, push subscriptions, API keys) are live configuration of an account. They are removed with the account or by an erasure request, not by age.
- **Invitations and pre-provisioned sign-ins** expire by `signin.preprovision.ttlDays`; their records are removed with the account that used them.
- **Image proxy latency and statistics** (`remote_image_latency`, `remote_image_history`, tracker counts) are in process memory only. They hold no address, are kept 24 hours, and are lost on restart.
- **Search index** entries belong to the user's mail and are removed with the message or the account; there is no age limit.
- **Postgres or SQLite write-ahead logs and backups** follow the database's own settings. Database backups taken by `backup.*` keep rows that this page removes until the backup itself is deleted (`backup.retentionCount`).

## Log rotation

Log destinations the server controls are standard output, always, and optional files under `logging.file.directory`.

- **Files** (`logging.file.enabled`, off by default): the current file is `mailwoman.log`. It is closed and renamed to `mailwoman-<unix seconds>-<n>.log` when it reaches `logging.file.maxSizeMb`, or is `logging.file.rotateHours` old (0 rotates by size only). Rotated files are compressed with gzip when `logging.file.compress` is on. Files older than `logging.file.maxAgeDays`, files beyond `logging.file.maxFiles`, and the oldest files while the directory is over `logging.file.maxTotalMb` are deleted. The directory may be absolute or relative to the working directory; parent-directory segments are refused.
- **Standard output** is what a container platform collects. Mailwoman does not rotate it. Set the platform's limit instead, for example the Docker `json-file` driver options `max-size` and `max-file`, or the journald and Kubernetes log retention. The redaction and level controls below apply to standard output as well.
- **Levels**: `logging.level` is the default level and `logging.componentLevels` adds `component=level` entries (for example `mw_imap=debug`). Until the store is open, `MW_LOG` or `RUST_LOG` sets the levels; leaving both settings at their defaults keeps those start-up levels.
- **Redaction**, applied to every line before it is written anywhere: `logging.redaction.addresses` and `logging.redaction.ips` each take `full` (unchanged), `truncated` (first letter and domain; IPv4 /24 or IPv6 /48), `hashed` (a short hash with a key that exists only in process memory, stable while the process runs) or `none` (removed). `logging.redaction.tokens` cuts bearer tokens, API keys, passwords in URLs and the server's own secret values.

Settings take effect within about a minute of the change. The log policy is read after the store is open, so the first lines of a start-up use the defaults.

Redaction works on the text of a line. It cannot recognise an address spelled in an unusual way, or a person's name, so choosing `none` or `hashed` reduces what the logs contain without making them anonymous.
