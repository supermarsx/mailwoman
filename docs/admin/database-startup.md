---
title: SQLite start-up and recovery
---
# SQLite start-up and recovery

This page explains what the server does while it opens an SQLite database, which log lines show the progress, and what to do when start-up takes a long time or fails with `pool timed out while waiting for an open connection`.

## Before you start

You need access to the server's log (for a deployment made with `deploy.sh`, `bash deploy.sh logs`) and to the host that stores the database.

## What happens at start-up

Before the server creates its normal pool of connections, it opens the selected SQLite database with one dedicated connection. If the database was left by an interrupted write or an interrupted `VACUUM`, SQLite may have to recover the journal or the write-ahead log on this first open. The server waits for that to finish and for the connection to close before it goes on. This step does not run `VACUUM`, does not compact data and does not delete journal or write-ahead log files.

The server prints these lines to its standard error stream, in this order:

| Log line | Meaning |
|---|---|
| `[mw-store] SQLite startup: opening/recovering database before pool initialization; recovery budget <N>s` | The dedicated connection is being opened. `<N>` is the budget. |
| `[mw-store] SQLite startup: opening/recovering database; <N>s elapsed; waiting on SQLite recovery/locks/I/O, not serving requests yet` | A heartbeat every five seconds while it waits. |
| `[mw-store] SQLite startup: recovery/open check complete in <N>s; initializing normal connection pool` | Recovery finished. |
| `[mw-store] SQLite startup: normal pool ready; checking schema migrations` | The normal pool is open and migrations run. |
| `[mw-store] SQLite startup: schema ready; initializing metadata` | Migrations are applied. |
| `[mw-store] SQLite startup: metadata and connection pool ready` | Start-up of the store is complete. |

A new, empty database is created in incremental vacuum mode. An existing database keeps its mode; see [Database growth and maintenance](/admin/database-maintenance#convert-sqlite-online).

## Set the recovery budget

`MW_SQLITE_STARTUP_TIMEOUT_SECONDS` sets how long the recovery step may take. The default is 1,800 seconds and the allowed range is 1 to 86,400. Any other value stops the server with `MW_SQLITE_STARTUP_TIMEOUT_SECONDS must be an integer from 1 to 86400`. The budget is separate from the acquisition timeout of the normal connection pool (`acquireTimeoutMs` in `database.pool`, 30 seconds by default).

If the budget runs out while SQLite is still working, the server does not abandon the work. It logs `recovery budget exceeded; waiting for SQLite to finish and close safely before failing startup. Do not delete journal/WAL files`, waits until the worker has closed, and only then fails with `SQLite startup recovery exceeded its <N>s budget; worker has finished and closed. Keep database journal/WAL files intact; review disk activity and MW_SQLITE_STARTUP_TIMEOUT_SECONDS`. A slow filesystem can therefore make start-up take longer than the budget. Keep the database file, its journal and write-ahead log files and its generation files as they are.

`deploy.sh` has its own readiness deadline, `--timeout SECS` (10 to 3600, default 180). For a large database that is recovering, give the deployment enough time, for example:

```sh
bash deploy.sh deploy --timeout 3600
```

The deadline of `deploy.sh` and the recovery budget are independent: raise both when the log shows long recovery.

PostgreSQL installations do not use this step.

## Troubleshooting

- **`pool timed out while waiting for an open connection` at start-up.** It does not by itself mean that clients used up the pool. The acquisition deadline also covers the first connection setup, and opening a large database after an interrupted write can need recovery first. Another process that holds a lock on the file, or slow storage, delays it as well. Look at the heartbeat lines and at the host's disk activity before you change the pool.
- **The log stops after the first line and repeats the heartbeat.** SQLite is recovering. Wait; do not delete journal or write-ahead log files.
- **The log shows `recovery/open check complete`, but the pool still times out.** Recovery is not the cause. Look at the later lines (migrations, metadata), at locks held by other processes, and at the pool figures in **Observability → Database**.
- **`SQLite startup: opening/recovery failed after <N>s: …`** The open itself failed, for example because the key does not match an encrypted database or the file is not a database. The message after the colon is SQLite's.
- **`deploy.sh` reports that the app is not healthy while the log is still recovering.** The deadline of `deploy.sh` ran out first. Raise `--timeout` and run it again.

## Related

- [Database growth and maintenance](/admin/database-maintenance)
- [Diagnostics](/admin/diagnostics)
- [Backups](/deploy/backups)
