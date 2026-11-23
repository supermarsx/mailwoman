# Online SQLite conversion

A SQLite database created with `auto_vacuum=NONE` must be rebuilt once before
background maintenance can reclaim free pages incrementally. Online conversion
builds that replacement while Mailwoman keeps serving requests, then switches to
it. The replacement keeps the same SQLCipher encryption settings and sealed data.
This applies to SQLite only; PostgreSQL installations do not use it.

![The Online SQLite conversion card with the Start conversion and Refresh buttons, and the Retained databases card with no retained database.](/screenshots/deploy/online-conversion-card.webp){.mw-crop}

## Before you start

- You need the `maintenance.read` permission to see the cards and
  `maintenance.run` to start, cancel or delete.
- Free disk space in the database directory must be at least the used size of
  the database plus 1 GiB, or the conversion stops with
  `Insufficient free space for a complete replacement plus the safety reserve`.
- Keep an independent, verified backup. The conversion keeps the previous
  database file, but that is not a backup.
- Stop any external program that writes to the SQLite file.

A database already in incremental mode completes at once without rebuilding. A
database in `FULL` auto-vacuum mode is refused and left unchanged.

## Start a conversion from the admin console

1. Open **Maintenance** > **Database**.
2. In **Online SQLite conversion**, select **Start conversion**.
3. Watch **Stage**. The card refreshes every 2 seconds while the page is visible
   and a conversion is running.
4. To stop before the switch, select **Cancel conversion**. Once switching has
   begun, the card shows `Cancellation is no longer available at this stage.`

The stages are **Preparing**, **Copying data**, **Verifying the copy**,
**Applying recent changes**, **Waiting to switch**, **Switching database** and
**Publishing the new generation**, ending in **Completed**, **Cancelled**,
**Failed** or **Recovery review required**.

The card also shows **Started**, **Finished**, **Elapsed time**,
**Average copy rate**, **Average replay rate**, and per-table progress:
**Copied rows**, **Copied bytes**, **Replayed keys** and **Pending changes**.

- The copy rate covers the copy phase; the replay rate covers catch-up,
  including the final drain. Neither includes verification or waiting to switch.
- Copied bytes are logical row data, not disk or network traffic.
- **Pending changes** counts journal events, so repeated updates to one row count
  more than once.
- The rates are averages, not an estimate of time to completion. A report that
  lacks a measurement shows **Not available**.

## Start a conversion from the command line

On a host managed by `deploy.sh`:

```sh
bash deploy.sh maintenance --online          # start
bash deploy.sh maintenance --online-status   # print the latest report
bash deploy.sh maintenance --online-cancel   # cancel before the switch
```

These run `mailwoman db online` inside the running app container. They do not
stop the app, create a helper container or take a snapshot. They cannot be
combined with `--apply`, `--skip-backup` or the offline cleanup options, and the
script refuses them on a full-stack (PostgreSQL) installation.

To tune the conversion, run the CLI in the container yourself, using your
deployment's Compose project and file options:

```sh
# Allow up to 10 seconds for the final switch on a quiet installation.
docker compose exec -T mailwoman mailwoman db online --max-cutover-ms 10000
```

The CLI leaves a short-lived request, sealed with the server key, for the
running server and waits up to 10
seconds for it to be accepted. It prints `The running server accepted this command.`
or the reason it was rejected.

| Option | What it does | Default | Range |
|---|---|---|---|
| `--status` | Print the latest report as JSON | | |
| `--cancel` | Request cancellation before the switch | | |
| `--batch-rows` | Rows per copy batch | 16 | 1-128 |
| `--batch-mib` | Data per copy batch, in MiB | 256 | 1-256 |
| `--journal-limit` | Maximum captured change events | 1,000,000 | 1,024-2,000,000 |
| `--pause-ms` | Pause between batches, in ms | 10 | 0-1,000 |
| `--max-wal-mib` | Largest source WAL allowed, in MiB | 512 | 32-4,096 |
| `--max-cutover-ms` | Time allowed to take exclusive access and finish the final checks at the switch | 2,000 | 100-10,000 |

The tuning options apply only when starting a conversion.

## How the app keeps serving

- Rows are copied in short batches. Triggers record which rows change while the
  copy runs, and those changes are applied to the replacement before it is
  checked and published.
- At the switch, new database work briefly waits while running work finishes.
  The `--max-cutover-ms` budget is not a limit on request latency: slow
  filesystem operations cannot be interrupted safely.
- While a conversion runs, no other process can open the database. Offline
  `mailwoman db` commands that change data refuse while the server is running;
  read-only status and cleanup previews work during normal service but refuse
  during a conversion.

The conversion stops safely, and the original database stays in use, when:

- the database is too busy to switch within the budget;
- the change journal reaches its limit or the source WAL exceeds its limit;
- disk space runs short;
- a row is larger than the batch size (rows are never truncated);
- changes are still pending 5 minutes after catch-up began. This does not by
  itself mean writes outpaced the copy; verification can build up a backlog.

A failed copy cannot be resumed; starting again begins a fresh copy. Final
foreign-key and row-count checks can exceed the budget on large, busy databases.
Retry at a quieter time, or use the offline conversion
(`mailwoman db cleanup --apply --offline --enable-incremental-vacuum` with the
server stopped).

Tables without a rowid, generated or hidden columns, schema changes during the
copy and external writers are not supported and are rejected where they can be
detected.

## Files, publication and recovery

Mailwoman never renames an open database file. A `CURRENT` file in the database
directory names the active database file (a "generation"). The switch writes a
new `CURRENT` durably and keeps database access paused until the new file is
usable. If the generation metadata is missing or damaged, Mailwoman refuses to
open the database rather than fall back to an older file.

- A restart before publication keeps the original and removes that job's
  unpublished copy; start a fresh conversion.
- A restart after publication opens the replacement.
- If a worker's shutdown cannot be confirmed, its copy is kept for recovery at
  the next start-up, and no new conversion can start until the process restarts.

::: warning
Never edit `CURRENT` by hand, and never go back to the previous file after new
writes. Back up the whole data directory, including the generation files and
the server key. Copying only `mailwoman.db` does not capture the current data.
:::

## Delete the previous database

The previous database file is kept after a conversion, so total disk use and
volume snapshots still include it.

1. Verify the active database and an independent backup.
2. Open **Maintenance** > **Database** > **Retained databases**. The table lists
   the file, its size and **Last modified**.
3. Select the delete button (**Delete** followed by the file name).
4. In **Delete retained database?**, type the exact file name and select
   **Delete database copy**.

Only the previous file recorded by the active `CURRENT` can be deleted; other
files in the directory are never offered. Deletion is refused while a conversion
or rebuild runs, while another process has the database open, for linked files,
and while SQLite WAL, SHM or journal files for that copy remain
(`SQLite recovery files are still present.`). A successful deletion is recorded
in the admin audit log.

## Mail change-log retention

Background maintenance also removes old mail change history (the records that
let clients fetch only what changed).

| Setting | What it does | Default |
|---|---|---|
| `maintenance.database.changeLogRetentionDays` | Days of change history to keep; `0` keeps it indefinitely | 60 |
| `maintenance.database.changeLogBatchRows` | Change rows removed per batch | 1,000 |
| `maintenance.database.enabled` | Switch for all background database maintenance, including this job | true |

- Cleanup is gradual: a large backlog or sustained writes can keep history
  longer than the retention window.
- Each run moves on to the next account and data type, so one busy account does
  not block the others.
- A client with a token older than the kept history gets `cannotCalculateChanges`
  for email, mailbox, submission, thread and email-query changes, and fetches the
  data again in full.
- Deleting history frees pages for reuse. Incremental vacuum reclaims them only
  on databases already in incremental mode.

## Related

- [Database maintenance](../admin/database-maintenance.md)
- [Deployment script](script.md)
- [Backups](backups.md)
- [PostgreSQL backend](postgres.md)
