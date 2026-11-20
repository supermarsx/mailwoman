# SQLite startup and recovery

`pool timed out while waiting for an open connection` does not by itself prove
that clients exhausted the pool. SQLx applies its acquisition deadline to the
initial connection setup as well. Opening a large SQLite database after an
interrupted write or VACUUM may need journal/WAL recovery before any requests
can be served. A different process holding locks or stalled storage can also
delay opening; check the startup phase and host I/O before assigning a cause.

New server builds first open and verify the selected SQLite generation with a
dedicated recovery connection, before initializing the normal request pool.
Logs identify opening/recovery, normal pool initialization, migrations and
metadata initialization. Opening/recovery emits a heartbeat every five seconds.
This step does not run VACUUM, compact data or delete journal/WAL files.

`MW_SQLITE_STARTUP_TIMEOUT_SECONDS` sets the startup recovery budget, default
1,800 seconds, permitted range 1–86,400 seconds. It is separate from the normal
request pool acquisition timeout. The connection must finish and close before
startup ownership is released. If the budget expires during an operation,
the server reports that it is waiting for SQLite to finish safely; it fails
startup only after the worker closes. Slow filesystem operations therefore can
exceed this budget. Preserve the database, its journals/WAL and generation files.

Deployment has a separate readiness deadline. For a large recovering database,
allow enough time for startup, for example:

```sh
bash deploy.sh deploy --timeout 3600
```

Use existing deployment options for backup/build choices. Increasing the deploy
deadline alone does not change the bootstrap timeout in older server images.
The new recovery step requires rebuilding/deploying a server containing it.
If the normal pool still times out after the recovery completion log, inspect
the later startup stage, locks and pool occupancy rather than assuming recovery
or increasing the pool ceiling. PostgreSQL startup is unchanged by this step.
