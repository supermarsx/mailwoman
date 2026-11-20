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
back to that copy after new writes. After verifying the selected database and
an independent backup, open **Maintenance → Database → Retained databases**.
The table shows the verified predecessor's path, size and last modification.
Deletion requires typing its exact filename in a confirmation dialog.

Only the predecessor recorded by the selected manifest is eligible. Arbitrary
neighbouring files and older unrecorded copies are not offered for deletion.
Retirement rechecks the active generation and the selected file's identity,
requires global maintenance permission and exclusive lifecycle ownership, and
refuses conversion/rebuild conflicts, other cooperating database processes,
linked files and any remaining SQLite WAL, SHM or rollback-journal sidecar.
It does not remove recovery sidecars to make a copy eligible. Close external
database tools and review recovery first; non-cooperating external writers are
outside the supported online-maintenance contract. Successful deletion removes
only the confirmed predecessor and records an admin audit event.
Back up the data directory, including generation metadata and the server key;
copying only the original `mailwoman.db` filename no longer captures current data.

Direct external SQLite writers, schema changes during copying, incremental BLOB
writes, virtual tables, generated columns and WITHOUT ROWID tables are outside
the supported live-copy contract and are rejected where detectable. Stop all
external writers. Mutating offline `mailwoman db` operations take the same
exclusive lifecycle lease and resolve CURRENT, so they refuse while the server
is running. Read-only status and cleanup previews retain a shared inspection
lease and remain available during ordinary service, but refuse during conversion.

### Mail change-log retention

Background maintenance retains 60 days of mail delta history by default.
`maintenance.database.changeLogRetentionDays` controls that window; zero keeps
history indefinitely. The maintenance master switch also controls this job.
`maintenance.database.changeLogBatchRows` limits each batch to 1,000 rows by
default. Larger states are removed in bounded chunks. Before any chunk is
removed, its whole state is fenced out of older-token deltas by the durable
checkpoint in the same transaction. Clients cannot read a partial retained state.
Recent or malformed timestamps visible in the bounded sample protect every
selected row of that state and the later tail. An invalid row first discovered
in a later chunk remains protected, but earlier valid old chunks stay expired;
retention does not first certify an arbitrarily large state's entire contents.
Each run rotates to the next account/type group, so a hot backlog cannot starve
other accounts. Cleanup is gradual rather than a strict expiration deadline:
large backlogs or sustained writes exceeding the configured cleanup capacity
can extend the retention window. The default global limit is 1,000 rows per
scheduled run, not 1,000 rows per account.

Retention preserves a durable head counter and
the oldest resumable token. Email, mailbox, submission and thread changes, plus
email query changes, refuse expired tokens with `cannotCalculateChanges` so
clients refetch instead of silently missing changes. Current tokens and new
appends remain monotonic after all historical rows have been removed. PIM and
crypto change logs retain their existing behavior. Deleting history frees pages
for reuse; bounded incremental vacuum can reclaim those pages on databases
already configured for incremental vacuum.
