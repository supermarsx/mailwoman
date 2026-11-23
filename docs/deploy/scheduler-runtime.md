# Scheduler runtime

Every Mailwoman server process runs one scheduler loop. It starts runs of
background job kinds from the `scheduled_task` table and records each run in
`job_run`. This page describes the job kinds, the schedule format, how several
processes share one database safely, and how maintenance windows restrict jobs.

## Job kinds

No task is created or enabled by default.

| Kind | What it does | Default schedule offered |
|---|---|---|
| `vacuum` | Database vacuum | Weekly, Sunday 03:30 UTC |
| `gc_uploads` | Removes expired uploads | Daily, 04:00 UTC |
| `reindex` | Rebuilds the search index (engine mode only) | none |
| `rethread_all` | Recomputes threads for existing mail (engine mode only) | none |
| `audit_prune` | Removes audit-log entries past their retention | set by the audit retention feature |
| `retention_sweep` | Deletes rows older than the global `retention.*` limits | set by the feature |
| `user_retention` | Applies each account's own retention rules | set by the feature |
| `retention_archive` | Archive tier for long-retention records | set by the feature |

In proxy mode, `reindex` and `rethread_all` fail with
`reindex requires engine mode` or `rethread requires engine mode`.

The server has no admin console screen, HTTP API or CLI command for creating or
editing scheduled tasks. The audit-log retention setting creates its own
`audit_prune` task when an administrator first saves a retention in days, and
the retention sweep starts its own runs. The other kinds run only if a task row
exists in the database.

## Schedule format

A task's schedule is one of these JSON shapes (`scheduled_task.schedule_json`):

```json
{"everySecs": 3600}
{"dailyAt": "03:00", "tz": "Europe/Berlin"}
{"weekly": {"days": ["mon", "thu"], "at": "03:00", "tz": "UTC"}}
{"monthly": {"day": 1, "at": "03:00", "tz": "UTC"}}
```

- `everySecs` is 60 seconds to 31 days. `tz` defaults to `UTC`.
- Unknown fields and malformed schedules or parameters do not run.
- Times are wall-clock times in `tz`. A time inside a skipped daylight-saving
  hour runs at the next valid minute; a repeated hour runs once, the first time.
- A monthly `day` beyond the end of a shorter month runs on that month's last day.
- Whole skipped dates (for example Pacific/Apia on 30 December 2011) are handled
  without inventing a UTC time.

## Several processes on one database

- A due task first takes a database lease, then starts its run. With several
  processes on one database, a due task runs once.
- At most one run of each kind is running at a time. On PostgreSQL this check is
  serialised with a transaction advisory lock; SQLite serialises writes itself.
- Each process identifies itself by host name, process start time, process ID
  and a random UUID.
- A lease is between 2 seconds and 1 day (timestamps are stored in whole
  seconds). The run renews it every third of the lease, at most every 30
  seconds. A run whose heartbeat is older than three leases is marked
  `abandoned`, which frees the kind again. Abandoned rows keep that status.
- Editing a task revokes its lease. A stale worker cannot release a newer
  owner's lease or overwrite the edited next run time.
- A run is cancelled when it loses its lease, its heartbeat fails or times out,
  its task is disabled or deleted, or its maintenance window closes.

::: warning
Cancelling a run cannot undo work it has already committed, and cannot stop a
blocking database operation or network call that is already in progress. The
leases prevent two processes from owning the same task; they do not make
external side effects happen exactly once after a crash or long pause.
:::

## Maintenance windows

A task with `only_in_window` set runs only inside a **global** maintenance
window (rows in `maintenance_window` with global scope that are not cancelled).
A domain-only window never allows a job that affects the whole installation.

- The start time is included, the end time is not. A one-off window with no end
  stays open until it is ended or cancelled.
- Announcing a window in advance does not open it.
- Invalid time zones, invalid recurrence data and database errors all count as
  "outside the window".
- A due run outside a window does not start: the task records
  `skippedOutsideWindow` and moves on to its next scheduled time. A running task is cancelled when its window closes.

A recurring window uses the row's `tz` and its `starts_at` time as the anchor,
plus exactly one recurrence and a duration:

```json
{"weekly": {"days": ["sun"], "at": "02:30"}, "durationSecs": 3600}
{"monthly": {"day": 31, "at": "03:00"}, "durationSecs": 1800}
{"everyDays": 2, "durationSecs": 600}
```

- If `at` is left out of a weekly or monthly rule, the anchor's local hour and
  minute are used; `everyDays` always uses the anchor's time.
- `durationSecs` is 1 second to 31 days; `everyDays` is 1 to 3660.
- Unknown fields, repeated or invalid weekdays, more than one recurrence type,
  invalid dates or times, and zero intervals mean the window is never open.
- Ending an occurrence early closes only that occurrence; later ones still open.

## Related

- [Maintenance (admin)](../admin/maintenance.md)
- [Online SQLite conversion](online-sqlite-maintenance.md)
- [Scaling and multi-location boundaries](scaling.md)
