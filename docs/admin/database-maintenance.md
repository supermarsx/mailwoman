# Database growth and on-demand maintenance

Open **Observability → Database** for file, occupied-page and reusable-page sizes,
WAL size, connection details and pool usage. Quick reports do not scan mail bodies.
The visible history graph has 15-minute, one-hour, six-hour and all-available
ranges, regional date/time labels and hover readouts. Samples use elapsed-time
positions; the file/occupied chart fits its vertical scale to show small changes.
WAL is plotted in its own chart so its smaller values remain readable. A table
alternative remains available for accessibility.
A background sampler runs once a minute even when the console is closed. It keeps
up to 720 samples in memory, separately reporting file allocation and WAL size.
Growth rates use occupied SQLite pages when available, so reuse of free pages does
not conceal increasing live data. History starts again after a restart.

The page automatically performs one read-only, time-bounded storage analysis after
loading its quick figures. **Inspect storage** repeats it. The report ranks measured
table consumers and names their largest text/binary columns. Logical payload bytes
are separate from physical sizes including indexes and page overhead. A physical
scan hitting its limit does not hide a measured payload total. Cached-body totals,
referenced bodies and unreferenced bodies are measured separately; unknown values
stay unavailable. A large file alone does not establish ongoing uncontrolled growth.
Old unreferenced bodies and reusable pages can remain after deduplication is deployed.
Repeat an inspection to see each table's size change; select the growth heading to
sort by that change. Unknown measurements stay unavailable rather than becoming zero.
PostgreSQL row counts are estimates. SQLite inspection is interrupted after 30 seconds
and partial results are labelled.

Recent write origins identify operation, table and store source file/line. These
are successful calls through the store SQL adapter, including transaction statements
and returning mutations; migration/encryption tools and external SQL clients are not
covered. Input bytes are parameter lengths, **not bytes committed to disk**. Affected
rows can include a transaction subsequently rolled back. No SQL text, parameter values,
credentials, email addresses or message contents are retained. The process-wide
15-minute window has bounded cardinality; it is not an exhaustive database audit.

`observability.database.growthMiBPerMinute` defaults to **64 MiB/minute**. Two
consecutive samples above the threshold trigger a visible warning and, when
`observability.database.growthAlerts` is enabled (default), an audit/server-log event.
Events include the leading write origins and are limited to one per 30 minutes.
Both settings are administrator-controlled global policies. No automatic deletion
or mail-ingestion throttling occurs when growth is detected.

## Known cached-body duplication

Repeated ingestion formerly allocated a new encrypted raw-body row and replaced
the message's reference, leaving the old body unreferenced. Raw MIME includes
attachments, so repeated synchronization could multiply their storage. New writes
now deduplicate identical content within an account using a private keyed reference.
Existing orphan bodies are not removed automatically. Measure their bytes before
attributing a particular production database's size to this issue.

## Skip the deployment backup copy

Backups remain the default. To explicitly omit a pre-deploy copy:

```sh
./deploy.sh deploy --skip-backup
```

This disables automatic rollback for that deployment and clears its rollback
pointer; older backup files are retained. SQLite-to-full-stack migration requires
its consistent source snapshot and refuses this flag. It is not a persistent
configuration change.

## Offline SQLite cleanup

In **Maintenance → Repair jobs**, the console provides copyable commands.
Execute them on the deployment host with the newly built application image.
The wrapper uses the deployed image; `--no-build` does not replace an old image.

Preview, without deleting or compacting anything:

```sh
./deploy.sh maintenance --orphan-bodies --compact
```

Apply: stop the app, take a consistent backup, clean selected data, then resume it:

```sh
./deploy.sh maintenance --apply --orphan-bodies --compact
```

If a verified backup already exists and another full copy is deliberately unwanted:

```sh
./deploy.sh maintenance --apply --skip-backup --orphan-bodies --compact
```

Additional independent targets can be combined:

| Option | Effect |
|---|---|
| `--orphan-bodies` | Delete encrypted cached bodies with no message reference. Retain referenced mail. |
| `--compact` | Rebuild SQLite with VACUUM and checkpoint its WAL to reclaim free pages. |
| `--logs-older-than-days 30` | Purge browser-error diagnostics and finished job/diagnostic records older than 30 days. `0` selects all eligible history. Running jobs remain. |
| `--purge-search-index` | Remove the default sibling search index. Rebuild it in Maintenance afterwards. Does not delete messages. Custom index paths use the existing online rebuild instead. |

Manual log ages are an explicit operator-selected override, not a replacement for
the regular retention policies. Historical statistics, audit, sign-in/security and compliance records are
excluded from this operational-log target list. Use the retention controls for them.

SQLite VACUUM can require up to twice the database's original size in free space;
see the [SQLite documentation](https://www.sqlite.org/lang_vacuum.html). Compaction
reclaims pages; it does not compress encrypted mail payloads. Existing snapshots
keep their original size. `./deploy.sh prune` applies configured backup retention.
Cleanup is batched, but once applied, row deletions require a backup to undo.

The wrapper resumes an originally running app after success or failure. It stops
this deployment's container only: operators sharing a SQLite file with other
processes must stop those writers too. Direct CLI users must acknowledge this:

```sh
mailwoman db --db-path /data/mailwoman.db cleanup --apply --offline --orphan-bodies --compact
```

Whole-database encryption keys continue to come from the existing environment/key
providers. No decrypted copy is written. Offline cleanup currently supports SQLite;
full-stack PostgreSQL installations keep their native maintenance tooling and the
existing policy-based retention/reindex jobs.

## Logs and counters have separate retention

Application-managed files support configured size/time rotation, gzip compression,
file-count/total-size limits, age retention, and address/IP/token redaction. The
console's **Cleanup** and **Rotate now** actions apply these existing policies
immediately. Compression may finish in the background. Docker stdout logs are
managed separately: deployed services retain three 10 MB files and compress rotated
files. These limits apply when containers are created with the new configuration;
see [Docker's logging options](https://docs.docker.com/engine/logging/drivers/json-file/).

For retained statistics, keep compact counts and histograms rather than individual
request records. The general statistics store already folds minutes into hours and
days using `retention.metrics.*`. Request history keeps minute buckets for up to
48 hours and hourly aggregates for `metrics.rollupDays` (normally three years).
`retention.requestMetrics.detailDays` now separately expires sampled normalized
404 paths after **30 days** by default, preserving request/404 counts and latency
histograms. Set it to 0 for counters-only history after a bucket closes. Periodic
minimization is bounded and runs at most hourly; scheduled/on-demand retention
also processes it using the configured batch limit.

Do not infer that a counter replaces security evidence: sign-in, audit, incident,
compliance and legal-hold retention keep their own policy and minimization rules.

## Background maintenance

The server starts bounded batches 30 seconds after startup. Defaults are one batch
per minute, 64 bodies, at most 16 MiB of deleted sealed payload, a 24-hour orphan
grace period, and 1,024 SQLite pages in eight-page steps. The SQL work budget is
250 ms; a short lock wait and a size-limited passive checkpoint can add time.
The worker uses a dedicated connection with a 50 ms lock timeout and yields when
foreground work owns the database. It never runs a full SQLite `VACUUM` online.

Observability → Database shows the last batch, scanned and removed bodies, freed
pages, deferral reasons, and the next batch. **Run a batch** uses the same limits
and policy as the scheduler. **Configure maintenance** opens the administrator
settings. All `maintenance.database.*` controls are global and hot-reloaded for
the next batch. Batch history and cursors are in memory; restarting starts a new
scan. The orphan grace timestamps are durable and survive a restart.

Orphan cleanup never deletes messages. It deletes only sealed cache bodies that
still have no message reference at the point of deletion. A first observation
starts the grace period, including for legacy rows. Ingestion stores a body and
its reference in one transaction. PostgreSQL additionally serializes that
lifecycle per account with transaction advisory locks. Standalone body writers
renew a lease, and attaching a missing body is refused instead of creating a
dangling reference. Oversized individual bodies are left for offline cleanup;
raise the batch byte limit or use the existing offline orphan command.

New SQLite databases use incremental auto-vacuum. An existing database in NONE
mode needs one offline conversion, which rebuilds the database once and can need
substantial temporary disk space. Preview first:

```sh
./deploy.sh maintenance --enable-incremental-vacuum
./deploy.sh maintenance --apply --enable-incremental-vacuum
```

The apply command stops the app, takes a backup by default, converts, and resumes
it. `--skip-backup` is an explicit operator opt-out. Whole-database encryption is
preserved. No startup or background job performs this conversion automatically.
Existing FULL auto-vacuum databases keep their mode. PostgreSQL uses its native
autovacuum for physical reclamation; body collection uses the same grace policy.

Offline cleanup reports each phase and a heartbeat every five seconds: phase
elapsed time, approximate SQLite VM steps, and database/WAL sizes. These are
activity measurements, not a completion percentage. SQLite cannot provide a
reliable percentage for a full `VACUUM`; encrypted large databases can require
substantial CPU, temporary disk space and disk I/O. A conversion-only request
does not scan orphan bodies; that potentially expensive scan runs only when
`--orphan-bodies` is selected. The final JSON report includes phase timings and
marks unrequested orphan statistics as unmeasured.

In versions with cooperative offline cancellation, Ctrl+C or SIGTERM requests a
SQLite VM interruption. The command waits for the active statement to finish or
roll back and for the database connection to close. Keep the app stopped until
the maintenance worker has exited; never delete journal or WAL files. Cancellation
does not undo cleanup batches that committed before the active statement. A
blocked filesystem operation or SQLite lock wait can delay interruption because
the progress callback runs at VM boundaries. Installing this version does not
add progress or cooperative cancellation to an already-running older worker.

The deployment wrapper launches a named detached maintenance worker and streams
its logs while the app is stopped. Ctrl+C or SIGTERM asks only that owned worker
to terminate cooperatively. The wrapper waits up to `--timeout` seconds for a
verified stopped state, checks that its process ID is zero and restart policy
is disabled, and removes the stopped container before resuming the app. Docker
control requests have a separate ten-second client deadline. It never uses a
forced worker removal as its normal cancellation path.

If the worker remains active, Docker is unavailable, ownership/state cannot be
verified, or removal fails, the wrapper leaves the app stopped and prints exact
inspection/recovery commands. Keep the worker's journal/WAL files intact. Confirm
that the named maintenance container has exited before manually restarting the
app. Replacing files on disk does not change an already-running older deployment
script or maintenance process.

Free pages may be reused without making the file smaller. Both body collection and incremental reclamation
are deferred when the WAL file exceeds `maintenance.database.maxWalMiB` (32 MiB by
default), avoiding a background job checkpointing an arbitrarily large WAL.
Long readers can delay physical shrinkage. No background checkpoint uses TRUNCATE
or waits for readers. The daily retention and weekly vacuum jobs delegate SQLite reclamation to
this worker; the explicit full-compaction maintenance command remains offline.


## Cache maintenance and inactivity expiry

Maintenance → Cache contains the administrator policy controls for cached MIME
bodies, orphan collection, sync-history retention and batch limits. Maintenance →
Database contains online SQLite conversion, background page reclamation and
verified retained-generation deletion. Observability → Database remains the
read-only storage breakdown and growth view.

Cached-body expiry is enabled by default with a 60-day inactivity lifetime,
controlled by `maintenance.database.expireCachedBodies` and
`maintenance.database.cachedBodyRetentionDays`. A first observation of a legacy
cache entry starts a full grace period; its email's received date is not treated
as evidence of when the cached copy was last used. Explicit reader opens and
downloads renew the lease, with database writes throttled to once per hour.
Header refreshes, scans and background prefetch do not renew it.

Expiry preserves message headers and detaches only cached content with a proven
refetch path. IMAP is the first supported source; local originals, POP3 messages,
drafts, queued submissions and other unproven sources remain protected. Opening
an expired message fetches and caches its content again. Ordinary background
ingestion cannot repeatedly refill the same expired copy. A shared body is
deleted only after its final message reference is gone.

Orphan cleanup has a separate 24-hour grace period by default, controlled by
`maintenance.database.graceHours`. Scanning an orphan does not renew that period.
Existing duplicate copies are not removed just by installing the deduplication
fix: they must be identified as unreferenced and pass this grace period. Batch
payload limits still apply; oversized entries are reported rather than forcing
an arbitrarily large live deletion. Use the Cache tab's batch controls or the
explicit offline cleanup procedure for these cases.

Mail sync history expires after 60 days by default, controlled by
`maintenance.database.changeLogRetentionDays` (`0` disables pruning). Maintenance
removes whole state batches within `maintenance.database.changeLogBatchRows`.
Durable checkpoints preserve monotonic state numbers; an expired sync token
returns `cannotCalculateChanges` so the client performs a full resync.

Already-checkpointed, oversized allocated WAL files may be recycled by SQLite
without checkpointing arbitrary pending data. A WAL that still needs recovery or
has blocking readers remains protected and reports that cleanup is deferred.
Checkpoint size limits are best-effort I/O safeguards, not hard wall-clock limits.

## Server resources

Observability → Resources reports effective CPU capacity (including fractional
container quotas), memory limits and available headroom, process RSS and CPU,
host memory, and the local data filesystem's total/available space. CPU usage is
interval-based: the first read is unavailable until a later sample, and 100%
means one logical core. It covers the webmail process, not separate renderer
children. Memory headroom is conservative and is not exact container memory usage.

The page refreshes every ten seconds while visible and offers manual refresh.
It requires an unscoped administrator with observability permission. Readings are
not cached by the browser or persisted as a new database row on every poll.
Unknown metrics stay unavailable. Filesystem readings describe the SQLite data
folder, or the local working filesystem for non-file databases; they do not
measure storage on a remote PostgreSQL server. Container detection is a hint,
not proof that resource limits are absent when no container is detected.
