# Database connection pools

SQLx 0.8.6 opens connections when demand exceeds the idle set and retires idle
connections according to its policy. Mailwoman configures the minimum, hard
ceiling and time budgets. Physical occupancy adapts inside that ceiling; this is
**not a live controller that changes the ceiling**. SQLx exposes no public setter
for an existing pool's minimum/maximum connection counts.

SQLite skips its redundant pre-acquire worker ping when an idle connection was
returned less than one second ago. After a longer idle interval, the acquire hook
pings before reuse. SQLx still performs its normal release-time ping, including
flushing queued transaction rollbacks, and every actual SQL query still runs and
returns its errors. PostgreSQL acquisition checks and TLS behavior are unchanged.

A SQLite worker that dies during this short health window can make the next
query fail rather than be replaced during the pre-acquire check. Such failures
do not authorize a request or trigger automatic write replay; the release check
discards the broken connection. Tests cover deliberate worker closure and
replacement, fresh reads after an external change, failed SQL and cancellation
rollback. They do not inject an abrupt crash into an idle worker. This reduces
worker round trips; it does not reduce the number of SQL policy reads or establish
an HTTP throughput target. Measure the complete request pipeline before claiming
a performance improvement.

## Startup policy

The global administrator policy `database.pool` is a JSON object. All fields have
these defaults; omitted fields inherit them:

```json
{
  "minimumConnections": 1,
  "sqliteMaxConnections": null,
  "postgresMaxConnections": null,
  "acquireTimeoutMs": 30000,
  "idleTimeoutSeconds": 60,
  "maxLifetimeSeconds": 1800,
  "postgresConnectionBudget": 100,
  "postgresReservedConnections": 20,
  "replicaCount": 1
}
```

`MW_DB_POOL` accepts the same complete object as an environment pin, taking
precedence over the stored object. Unknown fields, invalid types and unsupported
bounds fail validation rather than silently falling back. Minimum connections may
be zero; maxima must be positive. A zero configured minimum does not prevent an
actual query from opening a connection.

A null maximum derives a conservative startup ceiling from Rust's effective
`available_parallelism`, which accounts for supported container CPU quotas:

| Backend | Automatic requested ceiling |
| --- | --- |
| SQLite | Twice the effective CPUs, clamped to 2–64 (2–8 before 2026-11-24) |
| PostgreSQL | Four times the effective CPUs, clamped to 4–64 |

An explicit SQLite ceiling supports 1–128 (1–32 before 2026-11-24); an explicit
PostgreSQL ceiling supports 1–1,024 and is still constrained by the shared
budget. More SQLite connections permit concurrent WAL readers; they do not create
additional concurrent writers. SQLite transactions still serialize at the write
lock, and on a managed database every writer first takes the one in-process write
permit (`MW_SQLITE_WRITER`).

Memory per SQLite connection, which is a separate SQLite handle:

- page cache: up to 2,000 KiB with the driver default (`cache_size=-2000`). It
  fills as pages are read; with SQLCipher it holds decrypted pages. With
  `sqlitePageCacheMiB` set, that total is divided by the pool maximum, so more
  connections mean a smaller share each, not more memory;
- up to 100 prepared statements (the statement cache);
- one SQLx worker thread and its stack.

So the derived ceiling of 64 can hold up to 64 × 2,000 KiB = 125 MiB of page
cache with the driver default; 8 connections held up to 15.6 MiB. Each open
reader also keeps its WAL snapshot until its statement ends, which can delay a
checkpoint. On a host with up to 4 effective CPUs the derived value is unchanged
(2 × CPUs ≤ 8).

PostgreSQL also reads its actual `max_connections`, superuser-reserved slots and
additional reserved slots when the server provides them. The per-replica ceiling
is bounded by:

```text
floor((min(configured budget, server max_connections)
       - max(configured reserve, server reserved slots)) / replicaCount)
```

The effective maximum is the smaller of this allocation and the requested
CPU-derived/explicit maximum. Zero allocation, or a minimum above the effective
maximum, fails startup. Save-time validation additionally rejects minima above an
explicit maximum or the configured per-replica allocation.

`replicaCount` must cover the actual maximum number of application instances
sharing this budget, including deployments that overlap during rollout. This is
an administrator-declared allocation, not a distributed lease allocator. Other
applications and administrative clients need reserve capacity. PostgreSQL's own
connection/role limits remain the final server-side protection; a client-side
setting cannot reserve connections against unrelated clients.

## Applying changes

All pool options are **process-restart settings**. Saving the policy does not
pretend that a running SQLx pool changed its hard ceiling. Startup reads the stored
policy after schema migration and configures the pool before handing it to the
application. A bootstrap pool is closed before its replacement is admitted.
Already-running SQLite managers reused in the same process keep their existing
pool; recreating a router is not a process restart.

SQLite startup replacement uses its shared generation manager and an owned task.
The new lazy pool is installed in the generation before awaiting its first
connection, so cancelled startup cannot leave its workers outside lifecycle
fencing. Online generation conversion and source recovery clone the current
SQLx pool options, preserving configured limits and timeouts.

## Observability and profiling

`Store::database_pool_snapshot()` returns occupancy and effective options without
running SQL or exposing a DSN. The database observability response includes it as
`pool`: open/idle/busy, minimum/maximum, acquisition timeout, idle timeout, maximum
lifetime and closed state. Occupancy fields are approximate concurrent snapshots.
The report explicitly marks ceiling changes as requiring restart.

Payload-free process counters and the optional request-local
`pool_telemetry::measure_queries` boundary expose calls, failures, total microseconds
and maximum microseconds for leading-verb read/write/other statements and pool
acquisition. They never retain SQL text, parameters, account/session identifiers
or credentials. A cancelled/failed operation counts as a failure. Statement
success does not imply its surrounding transaction later committed.

Query timings include acquisition wait: do not add both durations as independent
costs. `BEGIN` is in the other category; on SQLite it includes waiting for the
single write lock. Acquisition timings exclude waiting for the generation
cutover admission gate. The central q() paths and SQLite/PG acquisitions are
instrumented; raw SQLx statements elsewhere are explicitly outside statement
coverage. Task-local scopes are request/frame-local and do not automatically
propagate into arbitrary spawned child tasks.

A busy pool with high acquisition wait suggests pool pressure. Long SQLite BEGIN
or write duration with little acquisition wait suggests write contention instead.
Raising the connection ceiling cannot fix serialized writes, repeated SQL,
upstream latency or CPU saturation.

## Validation and capacity limits

Focused fixtures cover automatic CPU bounds, PostgreSQL server/reserve/replica
allocation, invalid policy rejection, atomic environment precedence, stored-policy
application on reopening, unchanged live ceilings, physical demand growth and idle
retirement, acquisition timeout telemetry and online-conversion option preservation.

The 10,000 simultaneous-user / 40,000 successful API-request-per-second objective
is an acceptance target, not a result established by these pool changes. It needs
a representative multi-principal campaign with correctness checks, realistic
query/upstream distributions and measured CPU/RAM/database/cache limits. A
4–8-shared-core, 16 GiB host must budget the application and any colocated database
or cache together. Increasing pool size alone is not capacity evidence.
