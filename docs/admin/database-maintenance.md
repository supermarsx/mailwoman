---
title: Database growth and maintenance
screen: true
---
# Database growth and maintenance

This page covers how to see how large the database is and what fills it, and how to keep it in check: background clean-up, cached mail bodies, sync history, converting an SQLite database online, and the offline repair commands. The console screens are **Observability → Database**, **Observability → Resources** and the **Database**, **Cache** and **Repair jobs** tabs of **Maintenance**.

![The Database tab of Observability: file size, occupied pages, reusable space and write-ahead log size, with connection details and pool usage below.](/screenshots/admin-ops/opsa-database-storage.webp){.mw-shot}

## Before you start

- **Observability → Database** and **Resources** need the `observability.read` permission without a domain restriction. **Maintenance → Database** and **Cache** need `maintenance.read`, and **Repair jobs** needs `maintenance.run`.
- Changing a maintenance setting needs settings access. All `maintenance.database.*` settings are global.
- The offline commands run on the deployment host, with the application image that is deployed.

## See what uses the space

1. Open **Observability → Database**. **Refresh** reads the figures again, and **Updated** gives the time of the reading. The page is read-only and shows no credentials or message content.
2. **Storage** shows the **Database file**, **Occupied pages**, **Reusable space**, **Write-ahead log**, **Pages** and **Page size**. Reusable SQLite pages stay inside the file after deletions until the space is reclaimed, so a large file does not by itself mean the data grew.
3. **Connection** shows the **Driver**, **Server version**, **Database name**, **Server** (for SQLite, **Local file**), **Journal mode**, **Database response** and **Migrations applied**.
4. **Connection pool** shows the connections **Open**, **Idle** and **In use**, the **Minimum / maximum**, the **Connection wait limit** and counters for acquisitions, failures, waiting time, measured reads and writes. Connections grow with demand and shrink when idle, within the minimum and maximum. The limits are set in `database.pool` and change only after a full restart. For SQLite the card **SQLite page cache** also reports the cache targets and sampled hits and misses.
5. **Size history** plots the database file and the occupied pages, and the write-ahead log on a chart of its own, for **15 min**, **1 h 0 min**, **6 h 0 min** or **All available**. **View as table** gives the same numbers as a table. The server samples the database once a minute even when no console is open and keeps the samples in memory, so the history starts again after a restart.
6. **What uses the space** runs one read-only analysis when the page opens. **Inspect storage** repeats it. It ranks the tables that use the most space and names their largest text and binary columns, and it shows **Cached bodies**, **Cached body bytes**, **Referenced cached body bytes**, **Unreferenced bodies** and **Unreferenced body bytes**. Unreferenced bodies are cached message bodies that no message row points to; the inspection does not remove them. Payload sizes exclude indexes and page overhead, and a measurement that is not available is shown as unavailable, not as zero. On SQLite the inspection stops after 30 seconds and a partial result is labelled; PostgreSQL row counts are estimates. Press **Inspect storage** again later and compare table sizes to see what grows.
7. **Database growth** says **Collecting a baseline; at least two samples are needed.**, **No sustained rapid growth detected.** or **Sustained rapid growth detected. Inspect the largest tables and write origins.**, with the **Recent growth** and the **Alert threshold**. **Write origin** lists the operations, tables and source locations that wrote most in the last 15 minutes, with their **Statements** and **Input bytes**. Input bytes are the sizes of bound parameters, not bytes committed to disk. The list covers successful writes through the server's own database layer, in this process; it is not an audit of every write, and it contains no SQL text, parameter values, credentials, addresses or message content.

Two consecutive samples above [`observability.database.growthMiBPerMinute`](/admin/settings#observability.database.growthmibperminute) show the warning. When [`observability.database.growthAlerts`](/admin/settings#observability.database.growthalerts) is on, the server also writes the log line `Sustained database growth: inspect Observability > Database` and a `database-rapid-growth` audit event with the leading write origins, at most one every 30 minutes. Nothing is deleted and mail ingestion is not slowed.

### Resources

**Observability → Resources** shows **CPU capabilities** (the architecture, the processor features the server found, and how often its ASCII lowercasing code used the vector paths), **Effective server capacity** (**Effective CPU cores** including fractional container quotas, **Memory limit** and **Available memory headroom**), the **Server process** (**Resident memory** and **CPU usage**), **Host memory** and the **Data filesystem** (**Filesystem capacity** and **Filesystem available**). It refreshes every 10 seconds while it is visible. **CPU usage** needs two readings, so the first is **Not available**, and 100% means one logical core. A measurement that cannot be read is never shown as zero. The filesystem figures describe the folder with the SQLite data, or the local working folder for another database; remote PostgreSQL storage is not included. Container detection is a hint, not proof that no limit exists.

## Background maintenance

![The Database tab of Maintenance: online SQLite conversion, retained databases, and background database maintenance with Configure maintenance and Run a batch.](/screenshots/admin-ops/opsa-maintenance-database.webp){.mw-shot}

The server runs small clean-up batches in the background: it collects cached bodies that no message points to, and it reclaims free SQLite pages. The first batch starts 30 seconds after startup, and then one batch runs every [`maintenance.database.intervalSeconds`](/admin/settings#maintenance.database.intervalseconds) seconds. A batch uses its own connection with a short lock wait and yields when other work owns the database.

1. Open **Maintenance → Database** and find **Background database maintenance**. It shows the time of the last batch (**Updated**), **Scanned**, **Removed** (with the bytes) and **Reclaimed pages**, and the **Next batch**.
2. Choose **Run a batch** to run one batch now with the same limits as the scheduler.
3. Choose **Configure maintenance** to open the settings.

If a batch could not do its work, the card says **Maintenance yielded to a busy database, its work budget, or WAL pressure. It will retry automatically.** The status and the body scan position are kept in memory and start again after a restart. The grace times of unreferenced bodies are stored, so they survive a restart.

- Collecting unreferenced bodies never deletes messages. It deletes only cached bodies that still have no message reference when the batch reaches them. The first time a body is seen without a reference starts its grace period ([`maintenance.database.graceHours`](/admin/settings#maintenance.database.gracehours), 24 hours by default). A body larger than the per-batch payload limit is left for the offline repair.
- Reclaiming pages needs an SQLite database in incremental vacuum mode. For an existing database that is not, the card says **This database needs conversion before incremental reclamation can run. Use online conversion, or preview the offline command without --apply.**
- Both are deferred when the write-ahead log is larger than [`maintenance.database.maxWalMiB`](/admin/settings#maintenance.database.maxwalmib) (32 MiB by default), so that the background job does not checkpoint an arbitrarily large log. Free pages can be reused without shrinking the file, and long-running readers can delay shrinking.
- Full `VACUUM` is not run in the background. It is the offline command `--compact`.

### Convert SQLite online

**Online SQLite conversion** builds a new SQLite database generation while mail stays online. Database requests may queue briefly while the server switches, extra disk space is needed, and the previous generation is retained.

1. Choose **Start conversion**. The card shows the **Stage** (**Preparing**, **Copying data**, **Verifying the copy**, **Applying recent changes**, **Waiting to switch**, **Switching database**, **Publishing the new generation**, then **Completed**), **Pending changes**, **Copied rows**, **Copied bytes**, **Started**, **Elapsed time** and **Average copy rate**. It checks the conversion every 2 seconds while the page is open.
2. **Cancel conversion** is possible until publication; later it says **Cancellation is no longer available at this stage.**
3. A conversion that stops with **Recovery review required** or **Failed** needs a look at the server log before you try again.

The same conversion can be started and followed from the host with `./deploy.sh maintenance --online`, `--online-status` and `--online-cancel`. The conversion needs no stop and no snapshot.

**Retained databases** lists the previous generation after a conversion. Only a verified previous generation can be deleted (**Delete database copy**, after a confirmation that names the file and its size). Keep an independent backup first. Deletion is refused while database maintenance is running (**Database maintenance is running. Try again after it finishes.**) and while SQLite recovery files are still present.

## Cached bodies and sync history

The **Cache** tab of **Maintenance** holds the settings for cached mail bodies, unreferenced bodies and sync history. Its other cards, **Prepared mail metadata cache** and **Public static asset RAM cache**, are memory caches described in [Caching](/deploy/cache).

- **Cached message lifetime**: with **Automatically expire cached message bodies** on, a cached body that was not opened for **Days without use before expiry** (60 by default) is detached. Only bodies that can be fetched again from an upstream mailbox are eligible, and the card says **No connected mailbox currently has a confirmed refetch path. Protected content is kept.** when none can. Opening an expired message fetches it again. Message headers stay. Opening and downloading a message renew its lifetime; scans and header refreshes do not. The first time a cached body is seen, its lifetime starts from then, not from the message's date. **Scanned**, **Detached cache references** and **Protected** show the last batch.
- **Unreferenced bodies and space reclamation**: the same background batch described above, with the batch limits under **Batch limits and timing**.
- **Sync history**: old change records expire after 60 days by default. A client that holds an older sync token receives a full-resync answer. Each batch removes at most [`maintenance.database.changeLogBatchRows`](/admin/settings#maintenance.database.changelogbatchrows) rows, so cleaning is gradual; 0 days turns pruning off. Deleting history frees space inside the file; shrinking the file needs page reclamation or the offline repair.
- **Inspect cache storage** opens **Observability → Database** to measure cached payload, referenced bodies and orphaned copies. **Run a batch** runs one expiry batch now.

Statistics are kept as compact counts, not request records. See [Statistics that survive a restart](/admin/metrics-persistence) and [Request latency and 404 paths](/admin/latency-metrics). Sign-in, audit, incident and compliance records have their own retention (see [Retention](/admin/retention)); a counter does not replace them.

## Offline repair of an SQLite database

![The Offline database repair card on the Repair jobs tab: the preview and apply commands with a Copy command button each, and the optional flags.](/screenshots/admin-ops/opsa-maintenance-repair-commands.webp){.mw-crop}

Run these commands on the deployment host. They are for SQLite installations.

1. Preview, which changes nothing. Choose **Copy command** under **Preview cleanup** or type:

   ```sh
   ./deploy.sh maintenance --orphan-bodies --compact
   ```

2. Apply. The wrapper stops the app, takes a backup, cleans the selected data and starts the app again, also after a failure:

   ```sh
   ./deploy.sh maintenance --apply --orphan-bodies --compact
   ```

3. If a verified backup already exists and you do not want another full copy, add `--skip-backup` to the apply command. This also disables automatic rollback for that run.

| Option | What it does |
|---|---|
| `--orphan-bodies` | Deletes encrypted cached bodies that no message points to. Referenced mail is kept. |
| `--compact` | Rebuilds the SQLite file with `VACUUM` and checkpoints its write-ahead log to reclaim free pages. |
| `--logs-older-than-days N` | Purges operational diagnostics older than N days. `0` selects all of them. |
| `--purge-search-index` | Removes the default search index next to the database. Rebuild it afterwards in **Maintenance → Repair jobs**. |
| `--enable-incremental-vacuum` | Turns on page reclamation for a database that is not in incremental vacuum mode. It rebuilds the database once and can need a lot of temporary disk space. |
| `--skip-backup` | Skips the copy that is normally made before applying. |

`VACUUM` can need up to twice the size of the database in free space. Compaction reclaims pages; it does not compress encrypted mail. Existing backups keep their size, and `./deploy.sh prune` applies the configured backup retention. Deleted rows cannot be restored without a backup.

The wrapper stops only this deployment's container. If other processes use the same SQLite file, stop them too. Without the wrapper, the same work is the `db` command of the program, which refuses to apply unless you confirm that every writer is stopped:

```sh
mailwoman db --db-path /data/mailwoman.db cleanup --apply --offline --orphan-bodies --compact
```

Whole-database encryption keys come from the same environment and key providers as for the server, and no decrypted copy is written. A deployment with PostgreSQL uses the database's own maintenance tools and the retention settings.

### Skip the backup before a deploy

Backups before a deploy are the default. To omit one deliberately:

```sh
./deploy.sh deploy --skip-backup
```

This disables automatic rollback for that deploy. It is not a saved setting, and older backups are kept.

## Log files

The application's own log files have size, age and count limits, and compression. **Cleanup** and **Rotate now** in **Maintenance → Repair jobs** apply them at once; see [Maintenance](/admin/maintenance#other-tabs). Docker's own logging of the containers is separate: the deployed services keep three files of 10 MB and compress the rotated ones, as the compose files configure it for newly created containers.

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Background database maintenance** ([`maintenance.database.enabled`](/admin/settings#maintenance.database.enabled)) | Run the background batches. | On | Administrators (global) |
| **Collect unreferenced cached bodies** ([`maintenance.database.orphanBodies`](/admin/settings#maintenance.database.orphanbodies)) | Collect cached bodies that no message points to, after the grace period. | On | Administrators (global) |
| **Expire inactive cached bodies** ([`maintenance.database.expireCachedBodies`](/admin/settings#maintenance.database.expirecachedbodies)) | Detach cached bodies that were not used for the lifetime below, when they can be fetched again. | On | Administrators (global) |
| **Cached body lifetime (days)** ([`maintenance.database.cachedBodyRetentionDays`](/admin/settings#maintenance.database.cachedbodyretentiondays)) | Days without use before expiry, 1 to 3650. | 60 | Administrators (global) |
| **Mail change-log retention (days)** ([`maintenance.database.changeLogRetentionDays`](/admin/settings#maintenance.database.changelogretentiondays)) | Days of sync history kept, 0 to 3650; 0 keeps it without limit. | 60 | Administrators (global) |
| **Change-log batch limit (rows)** ([`maintenance.database.changeLogBatchRows`](/admin/settings#maintenance.database.changelogbatchrows)) | Change rows removed per batch, 1 to 10000. | 1000 | Administrators (global) |
| **Incremental SQLite reclamation** ([`maintenance.database.incrementalVacuum`](/admin/settings#maintenance.database.incrementalvacuum)) | Reclaim free SQLite pages in the background. An existing database needs one conversion first. | On | Administrators (global) |
| **Maintenance interval (seconds)** ([`maintenance.database.intervalSeconds`](/admin/settings#maintenance.database.intervalseconds)) | Idle seconds between batches, 15 to 86400. | 60 | Administrators (global) |
| **Bodies per batch** ([`maintenance.database.batchRows`](/admin/settings#maintenance.database.batchrows)) | Bodies scanned and collected per batch, 1 to 512. | 64 | Administrators (global) |
| **Body deletion limit (MiB)** ([`maintenance.database.batchMiB`](/admin/settings#maintenance.database.batchmib)) | Sealed body payload deleted per batch, 1 to 256; larger bodies need the offline repair. | 16 | Administrators (global) |
| **Orphan body grace period (hours)** ([`maintenance.database.graceHours`](/admin/settings#maintenance.database.gracehours)) | Hours a body without a reference waits before deletion, 1 to 720. | 24 | Administrators (global) |
| **SQLite pages per batch** ([`maintenance.database.batchPages`](/admin/settings#maintenance.database.batchpages)) | Pages reclaimed per batch in steps of eight, 8 to 16384. | 1024 | Administrators (global) |
| **Batch work budget (milliseconds)** ([`maintenance.database.maxRunMs`](/admin/settings#maintenance.database.maxrunms)) | SQL time per batch, 25 to 2000. Lock waits and the bounded checkpoint can add time. | 250 | Administrators (global) |
| **Minimum reclaimable space (MiB)** ([`maintenance.database.minFreeMiB`](/admin/settings#maintenance.database.minfreemib)) | Free space SQLite needs before pages are reclaimed, 0 to 65536. | 16 | Administrators (global) |
| **Maximum background checkpoint input (MiB)** ([`maintenance.database.maxWalMiB`](/admin/settings#maintenance.database.maxwalmib)) | Background work is deferred when the write-ahead log is larger, 1 to 1024. | 32 | Administrators (global) |
| **Database growth alerts** ([`observability.database.growthAlerts`](/admin/settings#observability.database.growthalerts)) | Log and audit a sustained growth. | On | Administrators (global) |
| **Database growth warning threshold (MiB/minute)** ([`observability.database.growthMiBPerMinute`](/admin/settings#observability.database.growthmibperminute)) | Growth per minute of the database file that raises the warning. The write-ahead log is measured separately. | 64 | Administrators (global) |

The settings are read for the next batch without a restart. Orphan collection, expiry, reclamation and sync-history pruning each need **Background database maintenance** to be on.

## Troubleshooting

- **The file is large but "Reusable space" is also large.** Pages freed by deletions stay inside the file. Let background reclamation work, or run the offline `--compact`.
- **"Maintenance yielded to a busy database, its work budget, or WAL pressure."** A batch gave way. If it repeats, the write-ahead log may be above `maintenance.database.maxWalMiB`, or the batch limits are too small for the amount of work.
- **"This database needs conversion before incremental reclamation can run."** The SQLite file is not in incremental vacuum mode. Use **Start conversion**, or preview `--enable-incremental-vacuum`.
- **Large bodies are never removed by the background batch.** They are larger than the per-batch limit (**Large bodies deferred**). Use the offline `--orphan-bodies`.
- **Cache storage inspection says "Some measurements reached the scan limit or are unavailable on this database."** The analysis hit its time limit. The figures shown are still measured values.
- **Start-up is slow after a crash.** See [SQLite start-up and recovery](/admin/database-startup).
- **The growth warning appears during a large import.** The warning compares the growth rate of the file with the threshold. Check **Write origin**, and raise the threshold if the growth is expected.

## Related

- [SQLite start-up and recovery](/admin/database-startup)
- [Maintenance](/admin/maintenance)
- [Observability](/admin/observability)
- [Online SQLite maintenance](/deploy/online-sqlite-maintenance)
- [Backups](/deploy/backups)
- [Retention](/admin/retention)
