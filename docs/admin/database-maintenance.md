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
