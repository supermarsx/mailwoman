# Scheduler runtime

Each application starts a scheduler loop backed by `scheduled_task` and `job_run`. No tasks are created or enabled by default. The current built-in jobs perform database vacuum, expired-upload cleanup, search reindexing, and historical rethreading. Reindex/rethread require engine mode and report failures rather than pretending to succeed in proxy mode. The admin jobs API, maintenance-mode HTTP gate, readiness integration, CLI, and screens are separate pending work.

A due task must acquire a database lease before creating its run. Run creation verifies the lease and serializes the one-running-job-per-kind check (SQLite write serialization; PostgreSQL transaction advisory lock). Instance identities include hostname, process start, process ID, and a random UUID. A heartbeat transaction renews both the running row and its still-live, enabled task lease; failure rolls back both updates. Expired ownership cannot be resurrected. Editing a task revokes its lease, and the worker refreshes the task snapshot after acquisition. A stale worker cannot release a replacement owner's lease or replace the edited next-run time.

Workers check ownership before execution, periodically, and before recording completion. Lost ownership, database heartbeat failure/timeout, task disable/delete, or loss of a required maintenance window cancels the async job. Abandoned rows retain their terminal status. Effective runtime leases are clamped to 2 seconds through 1 day because persisted timestamps have whole-second precision. Heartbeats occur every third of the lease, capped at 30 seconds; unresponsive heartbeat operations time out. Runs with heartbeats older than three leases are abandoned. Cancellation cannot undo committed effects or forcibly stop an already-running blocking database operation or network I/O outside the cooperative async task. These leases prevent competing scheduler ownership; they do not promise exactly-once external side effects after a process crash or prolonged pause.

Schedules accept `everySecs`, `dailyAt` with `tz`, `weekly:{days,at,tz}`, or `monthly:{day,at,tz}`. Unsupported fields and malformed schedules/parameter objects do not execute. Local wall-clock gaps move to the next valid minute; repeated times use the first occurrence only. Monthly dates clamp to the last day of shorter months. Whole skipped dates, such as Pacific/Apia in 2011, are handled without inventing a UTC time.

Tasks marked `only_in_window` use persisted, uncancelled **global** maintenance windows. A domain-only window cannot authorize a job affecting the whole installation. The scheduler reads current rows directly, including `occurrence_ended_at`; advance announcements do not open a window. Start is inclusive, end exclusive. A one-off window with no end remains active until ended or cancelled. Invalid timezone/recurrence data and database errors deny execution. Outside-window due occurrences are recorded as skipped and advanced to their next schedule; an already running window-bound task is cancelled when the window closes.

The bounded scheduler recurrence contract uses the row's `tz` and `starts_at` anchor, plus exactly one recurrence and an elapsed duration:

```json
{"weekly":{"days":["sun"],"at":"02:30"},"durationSecs":3600}
{"monthly":{"day":31,"at":"03:00"},"durationSecs":1800}
{"everyDays":2,"durationSecs":600}
```

Omitted weekly/monthly `at` uses the anchor's local hour/minute; every-N-days always uses it. Duration is 1 second through 31 days; everyDays is 1 through 3660. Unknown fields, duplicate/invalid weekdays, mixed recurrence types, invalid dates/times, and zero intervals fail closed. Ending an occurrence suppresses that occurrence, while later recurrences remain eligible. The future maintenance API/gate must reuse this contract rather than introduce a second evaluator.
