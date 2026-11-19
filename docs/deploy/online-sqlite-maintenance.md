# Online SQLite conversion

An older SQLite installation in `auto_vacuum=NONE` mode needs a rebuild once
before background incremental page reclamation can work. The online converter
builds a compact replacement while the app continues serving requests. It keeps
the same SQLCipher configuration and sealed data. PostgreSQL does not use this.

After rebuilding and deploying a server containing this feature, start it from
**Observability > Database > Online conversion**, or on the deployment host:

```sh
bash deploy.sh maintenance --online
bash deploy.sh maintenance --online-status
bash deploy.sh maintenance --online-cancel
```

These commands execute inside the running app container. They do not stop it,
create a helper container, or make an additional snapshot copy. Do not combine
them with `--apply`, `--skip-backup`, or offline cleanup flags. Online conversion
retains the original generation; keep independent verified backups as well.
An installation already using incremental vacuum completes without rebuilding.

The CLI submits a private authenticated local command, bound to the server key
and canonical database path, with a short lifetime and persisted replay guard.
Status reports copied rows/bytes, the current table, pending changes and phase.
The admin UI reads lightweight status every two seconds while visible and active.
No progress samples are appended to the database.

## Serving traffic during conversion

Short keyset batches copy the live schema and rows without a long-lived source
snapshot. Transactional triggers record changed row identifiers. Changes are
reconciled into the replacement before it is checked and published. Every Store
clone, streaming query, raw managed connection and transaction participates in
the same generation admission gate. Ordinary SQLite processes retain shared
lifecycle leases. Conversion upgrades to exclusive ownership and refuses if
another cooperating process has the database open; offline CLI writers use the
same exclusive lease. Other processes cannot open it during conversion.

Final cutover briefly queues new database work while existing work drains. The
default admission and SQL verification budgets are two seconds each. If the
database is too busy, verification exceeds its budget, the change journal fills,
source WAL exceeds its budget, or disk headroom is insufficient, conversion
fails safely and the original stays authoritative. Slow filesystem operations
and confirmed worker shutdown cannot be interrupted safely; the budget is not
a hard maximum request latency. This is online conversion, not a promise of
zero added latency. It has not been load-tested on the production 115 GB file.

Detailed tuning is available through the running container's CLI:

```sh
# Allow up to 10 seconds for admission and final SQL work on a quiet installation.
docker compose exec -T mailwoman mailwoman db online --max-cutover-ms 10000
```

Use the generated Compose configuration/project options for your deployment.
Other bounded options include `--batch-rows`, `--batch-mib`, `--journal-limit`,
`--pause-ms`, and `--max-wal-mib`. Large rows beyond the copy byte budget cause
a safe abort; they are never truncated. Final foreign-key and row-count checks
can exceed the budget on large occupied databases. Retry during a quieter
period or use the existing explicit offline conversion if necessary.

## Publication and recovery

Open database files are never renamed. A durable same-directory CURRENT manifest
selects an immutable generation filename, and the admission gate remains held
until the selected pool is usable. Publication uncertainty keeps access fenced
until durable selection is recovered. Missing/corrupt generation metadata fails
closed instead of reopening a stale original database.

Cancellation is accepted before switching starts. A restart before publication
keeps the selected source and removes that job's capture metadata and recorded
unpublished shadow; start a fresh copy. A restart after publication opens the
published replacement. Copy seeding does not resume across restarts. Normal
failure cleanup closes private workers before removing the exact unselected
shadow and its SQLite sidecars. A shadow with unconfirmed teardown is retained
for startup recovery. If shutdown cannot be confirmed, conversion ownership is
retained until a process restart; another conversion cannot overwrite that
job's recovery record.

The previous published database is deliberately retained. Conversion reduces
the active database footprint, but total disk usage and subsequent full-volume
snapshots still include this old copy. Never manually replace CURRENT or roll
back to that copy after new writes. Retiring an old generation requires a
separate reviewed cleanup after verifying the selected database and backup.
Back up the data directory, including generation metadata and the server key;
copying only the original `mailwoman.db` filename no longer captures current data.

Direct external SQLite writers, schema changes during copying, incremental BLOB
writes, virtual tables, generated columns and WITHOUT ROWID tables are outside
the supported live-copy contract and are rejected where detectable. Stop all
external writers. Mutating offline `mailwoman db` operations take the same
exclusive lifecycle lease and resolve CURRENT, so they refuse while the server
is running. Read-only status and cleanup previews retain a shared inspection
lease and remain available during ordinary service, but refuse during conversion.
