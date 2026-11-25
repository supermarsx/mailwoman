---
title: PostgreSQL tuning
---
# PostgreSQL tuning

This page gives a PostgreSQL 16/17 configuration for a Mailwoman database, sized
by rule from the CPUs and memory the database server has. Each setting lists the
value or formula, why, the risk, and whether it changes durability. The rules are
implemented in `scripts/perf/postgres-profile.mjs` (`node
scripts/perf/postgres-profile.mjs --cpus 16 --memory 64GiB` prints them as
`-c name=value` arguments) and, for the `deploy.sh` full stack, in
`scripts/deploy-stack.sh`.

Measured effects on a 4-CPU container are on
[PostgreSQL flag-update measurements](../perf/postgresql-flag-update-bench.md);
what Mailwoman does to the server (pools, locks, statements) is on
[PostgreSQL usage audit](../perf/postgresql-audit.md).

## Before you start

- PostgreSQL 16 or 17. Every setting here exists in both; `wal_compression=lz4`
  needs a server built with lz4 (the Debian/Ubuntu PGDG packages and the
  official `postgres` image are).
- Know what the database server has: `C` = CPUs and `M` = memory available to
  PostgreSQL (the container or VM limit, not the host's total), and whether
  PostgreSQL shares the machine with Mailwoman.
- Settings marked *restart* need a server restart; the others take effect on
  `SELECT pg_reload_conf();`. Apply them with `ALTER SYSTEM SET name = 'value';`
  or in `postgresql.conf`, or as `-c name=value` arguments of `postgres`.

Three profiles:

| Profile | Use | Memory shares |
|---|---|---|
| dedicated | PostgreSQL alone on its machine or container limit | `shared_buffers` 25% of M, `effective_cache_size` 75% |
| colocated | PostgreSQL beside Mailwoman on one host (`deploy.sh` full stack) | from PostgreSQL's own share of the host and the connections the app opens (see [Small hosts](#small-hosts-and-the-colocated-profile)), not from the whole host |
| benchmark | dedicated, for measurement runs; every report must say it was used. Diagnosis (`pg_stat_statements`, `track_io_timing`) only with `--diagnostics` | as dedicated |

No profile turns `fsync`, `full_page_writes` or `synchronous_commit` off, and
none sets `commit_delay`.

**None of the profiles is a default.** A request-level run on 4 CPUs / 4 GiB
(t62-r2, same binary) measured the benchmark profile, then still with
`pg_stat_statements` and `track_io_timing`, slower than stock: identity 767
against 967 requests/s, the goal mix 143.9 against 154.4 requests/s with a
higher p99. My own measurements were database-level only (pgbench, flag
updates: throughput within the noise, WAL 5–42% lower), and no request-level
A/B of the dedicated or colocated profile without diagnostics exists yet. Until
one shows a profile is not slower than stock, `deploy.sh` starts PostgreSQL
with stock settings plus four settings that do not touch the query path (see
[What `deploy.sh` sets](#what-deploy-sh-sets)), and the profiles are opt-in:
`node scripts/perf/postgres-profile.mjs --cpus N --memory M --profile P --sql`
prints `ALTER SYSTEM` statements to apply, followed by a restart.

## Key values for a 16-CPU, 64 GiB dedicated server

`shared_buffers` 16 GB, `effective_cache_size` 48 GB, `max_connections` 160,
`work_mem` 64 MB, `maintenance_work_mem` 2 GB, `autovacuum_work_mem` 1 GB,
`wal_buffers` 64 MB, `max_wal_size` 32 GB, `min_wal_size` 4 GB,
`checkpoint_timeout` 15 min, `wal_compression` lz4, `synchronous_commit` on,
`random_page_cost` 4 (unchanged, see below), `jit` off, `autovacuum_max_workers` 4,
`autovacuum_naptime` 15 s, `autovacuum_vacuum_cost_limit` 2000.

## Memory

| Setting | Value | Why | Risk | Durability |
|---|---|---|---|---|
| `shared_buffers` (restart) | dedicated `clamp(0.25 × M, 128 MB, 32 GB)`; colocated: see [Small hosts](#small-hosts-and-the-colocated-profile) | PostgreSQL's own page cache; the stock 128 MB holds about 160,000 rows of `messages` (≈ 0.8 KB each) and is evicted constantly by list queries | too large starves the OS page cache (PostgreSQL then reads through two caches) and Mailwoman on a shared host; on Linux, sizes above about 8 GB want huge pages (below) | none |
| `effective_cache_size` | dedicated `0.75 × M`; colocated `0.75 ×` its share | a planner estimate of total cache, makes index scans look as cheap as they are | none (allocates nothing) | none |
| `work_mem` | dedicated `clamp((M − shared_buffers) / (4 × max_connections), 4 MB, 64 MB)`; colocated: see Small hosts | per sort/hash node of one query; Mailwoman's hot statements use indexes and rarely sort, so the default is enough for them; the formula leaves room for list and search queries over large mailboxes | each query node may use it, several per query and per parallel worker: the formula assumes at most a quarter of the connections use one such node at a time | none |
| `maintenance_work_mem` | `clamp(M / 16, 64 MB, 2 GB)` | `CREATE INDEX` in migrations, `VACUUM` | one per maintenance operation | none |
| `autovacuum_work_mem` | `clamp(M / 64, 64 MB, 1 GB)` | each autovacuum worker would otherwise take `maintenance_work_mem`; PostgreSQL 16 uses at most 1 GB of it per worker | none | none |
| `huge_pages` (restart) | `try` (default) in every profile; `on` only as described in [Huge pages](#huge-pages) | fewer TLB misses, smaller page tables per backend | see that section | none |

## Huge pages

- `huge_pages = try` (the default): at start PostgreSQL asks the kernel for its
  shared memory in 2 MB huge pages; if the kernel has none reserved, it uses
  normal 4 KB pages and says nothing. Nothing breaks, but with a large
  `shared_buffers` every backend then maps it in 4 KB pages: more page-table
  memory per backend (about 2 MB of page tables per GB of shared memory a
  backend touches) and more TLB misses.
- `huge_pages = on`: PostgreSQL refuses to start when the reserved huge pages
  do not cover its shared memory. Use it only together with a reservation, so a
  missing reservation is a visible start failure instead of a silent slowdown.
- When to set it: on a dedicated Linux server with `shared_buffers` of 8 GB or
  more. Reserve `vm.nr_hugepages` = the value `postgres -C
  shared_memory_size_in_huge_pages` prints (PostgreSQL 15+, run with the final
  settings) in `/etc/sysctl.d`, apply it with `sysctl -p` or a reboot, then set
  `on`. In a container the reservation is the host's, and Docker Desktop's VM
  usually has none: keep `try` there. Below 8 GB, and on the `deploy.sh` full
  stack, keep `try`.

## Small hosts and the colocated profile

On the `deploy.sh` full stack PostgreSQL, Mailwoman, Valkey and (with
`EXPOSURE=caddy`) Caddy share one host. Each must be sized from its own share;
if each takes its part of the whole host, together they overcommit it.
`stack_memory_shares` in `scripts/deploy-stack.sh` divides the host's memory
(as `docker info` reports it) like this, in MiB:

- a reserve for the OS and its page cache: 10% of the host, at least 256;
- Caddy: 64 when `deploy.sh` runs it;
- Valkey: `maxmemory` = 1/16 of the host, 32–256, plus 32 of overhead;
- PostgreSQL: 35% of what remains;
- the app: the rest. Its container limit is `MEMORY_LIMIT` (4g for new
  installations) or this share, whichever is smaller; `deploy.sh` logs a line
  when it lowers it and leaves `MEMORY_LIMIT` in `deploy.env` unchanged.
  Without the full stack the app gets everything but the reserve and Caddy.

With the colocated profile (opt-in, above), PostgreSQL's settings are computed
from its share `P` and from the
connections the app opens, `A` = pool (4 per app CPU, 4–64) + 8 dedicated
connections (authority mirror, maintenance pools):

| Setting | Colocated formula |
|---|---|
| `max_connections` | pool + 25, at least 30. The app subtracts 20 reserved connections from `max_connections` before sizing its pool, so pool + 20 gives it its full pool; the other 5 are for `pg_dump` backups and an administrator |
| `shared_buffers` | `P / 4`, 32 MB–32 GB |
| `work_mem` | `(P − shared_buffers − A × 8 MB) / (2 × A)`, 4–64 MB |
| `effective_cache_size` | `3P / 4`, at least 128 MB |
| `maintenance_work_mem` | `P / 8`, 32 MB–2 GB |
| `autovacuum_work_mem` | `P / 32`, 16 MB–1 GB |

The 8 MB per busy backend covers the process (about 5 MB), its catalog caches
and the prepared statements Mailwoman keeps: sqlx caches up to 100 statements
per connection and PostgreSQL keeps a plan for each on that backend (measured
on the [flag-update measurements](../perf/postgresql-flag-update-bench.md#prepared-statement-memory-per-backend)
page). The worst case the formula allows is `shared_buffers + A × (8 MB +
2 × work_mem)`: every connection the app opens busy with two sort or hash nodes
at once.

Worked values (full stack without Caddy; with Caddy the app gets 41 MiB less;
in brackets the app's share without the full stack):

| Host | Reserve | Valkey | PostgreSQL share | App share | `shared_buffers` | `work_mem` | `max_connections` | App connections | Worst case |
|---|---|---|---|---|---|---|---|---|---|
| 1 GiB, 1 CPU | 256 | 64 | 235 MiB | 437 MiB (768) | 58 MB | 4 MB | 30 | 12 | 202–250 MB |
| 2 GiB, 2 CPUs | 256 | 128 | 571 MiB | 1,061 MiB (1,792) | 142 MB | 9 MB | 33 | 16 | 414–558 MB |
| 4 GiB, 2 CPUs | 409 | 256 | 1,189 MiB | 2,210 MiB (3,687) | 297 MB | 23 MB | 33 | 16 | 0.8–1.1 GB |
| 16 GiB, 8 CPUs, app 4 | 1,638 | 256 | 5,060 MiB | 9,398 MiB, limit 4g | 1,265 MB | 64 MB | 41 | 24 | 3.0–4.4 GB |
| 64 GiB, 16 CPUs, app 4 | 6,553 | 256 | 20,543 MiB | 38,152 MiB, limit 4g | 5,135 MB | 64 MB | 41 | 24 | 6.7–8.2 GB |

"Worst case" is one, then two, `work_mem` nodes per app connection. On the
1 GiB host the 4 MB floor of `work_mem` lets the two-node case exceed the
235 MiB share by 15 MiB; the one-node case fits. There the app's share with the
full stack (437 MiB) is close to what a server for one user uses (about 380 MiB,
an estimate, not measured for one user); such a host is better served by the
single-container SQLite stack (768 MiB for the app). The previous colocated
profile gave this host `shared_buffers` 153 MB and `max_connections` 100, sized
from the whole host: up to 0.5–1 GiB for PostgreSQL alone. Stock PostgreSQL
(the `deploy.sh` default) uses `shared_buffers` 128 MB plus about 12 MB per
connection the app opens: about 270 MB on this host, near its 235 MiB share.

## Connections and pools

| Setting | Value | Why | Risk | Durability |
|---|---|---|---|---|
| `max_connections` (restart) | dedicated `clamp(10 × C, 100, 400)`; colocated app pool + 25, at least 30 | must cover Σ over Mailwoman processes of (pool maximum + about 8 dedicated connections) + `postgresReservedConnections` (20) + administration. Throughput is reached with active backends ≈ 2–4 × C; the rest are idle pool slack | each connection is a process (≈ 5–10 MB with its caches); hundreds of idle ones also cost snapshot time | none |
| Mailwoman pool | `cpus × 4` clamped 4..64 per process (application CPUs), then capped by the budget; set explicitly with `MW_DB_POOL='{"postgresMaxConnections":N}'` when the application and database have different sizes | pool size should follow the database's CPUs, not the application's, which Mailwoman cannot see. Measured on a 4-CPU database: flag-write throughput highest at 8 concurrent connections (2 × CPUs) and 30–45% lower at 32–64. Rule: 2–4 × database CPUs over all processes; for one process and a 16-CPU database, 32–64 | a pool far above 4 × C only adds queueing inside PostgreSQL | none |
| Minimum connections | `MW_DB_POOL` `minimumConnections` ≈ steady busy count for high load (default 1) | the pool retires idle connections after 60 s and reconnects under the next burst; each new connection is a fork plus SCRAM authentication (measured cost on the measurement page) | idle connections held | none |

**Connection poolers.** PgBouncer in *session* mode works (each client keeps a
server connection) but saves nothing for a bounded pool. *Transaction* mode is
not supported: Mailwoman uses named prepared statements on every connection
(sqlx default) and sets session parameters on its maintenance and mirror
connections; both break or leak when consecutive transactions run on different
server connections. Its advisory locks are transaction-scoped and would be
compatible. Details in the [audit](../perf/postgresql-audit.md#what-this-means-for-a-connection-pooler).

**TCP or Unix socket.** On the same host a Unix socket saves the TCP stack per
round trip (measured on the measurement page). Use it with
`postgres://user:pass@%2Fvar%2Frun%2Fpostgresql/mailwoman` (the host part is the
URL-encoded socket directory). A Mailwoman write makes many round trips (about
17 per changed message today), so this matters more than for a one-statement
workload.

**Password hashing.** `password_encryption = scram-sha-256` (default). SCRAM
costs its 4,096 PBKDF2 iterations on both sides per new connection; a pool that
keeps its connections pays it rarely. Do not lower `scram_iterations` to speed
this up; keep connections instead (minimum connections above).

## WAL, checkpoints and commits

| Setting | Value | Why | Risk | Durability |
|---|---|---|---|---|
| `wal_level` (restart) | `replica` (default) | needed for physical backups and standbys | — | none |
| `fsync` | `on` — never change | | `off`: a crash or power loss can corrupt the whole cluster | **yes** |
| `full_page_writes` | `on` — keep | protects against torn 8 KB pages | `off` is safe only on storage with atomic 8 KB writes (for example ZFS with matching record size); otherwise a crash can leave unrepairable pages | **yes** |
| `synchronous_commit` | `on` — keep for every user mutation | a commit returns after its WAL is flushed | see below | **yes** |
| `wal_compression` | `lz4` | compresses full-page images; after each checkpoint the first change of every page writes the whole 8 KB page | CPU per full-page image (small with lz4) | none |
| `wal_buffers` (restart) | `-1` (auto, 16 MB at most) below `shared_buffers` 4 GB; `64MB` above | a burst of writers fills 16 MB between flushes | memory only | none |
| `max_wal_size` | dedicated `clamp(M / 2, 2 GB, 64 GB)`; colocated 1 GB below 8 GB of host memory, else 4 GB. Better: peak WAL MB/s × `checkpoint_timeout` s × 1.5, bounded by free disk | checkpoints driven by WAL volume instead of time cause a flood of full-page images; the stock 1 GB is passed in seconds at the goal write rate (measured WAL per flag change: 2–4 KB) | disk space up to about that size for WAL; crash recovery replays up to that much WAL (longer restart after a crash) | none |
| `min_wal_size` | `max_wal_size / 8` (≥ 256 MB) | keeps recycled WAL segments instead of creating new ones | disk | none |
| `checkpoint_timeout` | 15 min | fewer checkpoints, fewer full-page images | longer crash recovery | none |
| `checkpoint_completion_target` | 0.9 (default since 14) | spreads checkpoint writes | — | none |
| `commit_delay` / `commit_siblings` | leave at 0 / 5 unless a measurement on your storage shows a gain | group commit: a committing backend waits `commit_delay` µs so others can share its WAL flush. PostgreSQL already groups concurrent flushes without it. Measured with 100 µs on a 4-CPU container: 10–29% fewer WAL flushes, throughput unchanged within the noise | adds latency to every commit; with Mailwoman's per-account advisory lock held until commit, it also lengthens the serialised section per account | none (commits stay durable) |
| `wal_writer_delay`, `wal_writer_flush_after` | defaults (200 ms, 1 MB) | only affect asynchronous commits | — | — |

**`synchronous_commit = off` is not recommended as a default.** With it, a
commit returns before its WAL reaches disk; the WAL writer flushes it within
about three times `wal_writer_delay` (≈ 600 ms by default). A crash of the
PostgreSQL server, the OS or the power within that window loses the most recent
transactions that Mailwoman already acknowledged: a flag change, a move, a sent
mail's bookkeeping, a sign-out or session revocation, a password change. The
database stays consistent (no corruption); the lost transactions are simply
not there after recovery, while the user, the upstream mail server (already
changed) and other clients may have seen them. Setting it per transaction for
data that is rebuilt anyway (for example request metrics) is an application
decision, not a server setting, and needs a review of what each such
transaction protects.

## Planner, I/O and parallelism

| Setting | Value | Why | Risk | Durability |
|---|---|---|---|---|
| `random_page_cost` | 4 (default) in every profile. 1.1 is the usual SSD advice, but it is a per-deployment step: check the plans of the list and count queries first | the default models seek times of spinning disks. Measured: with 1.1 alone, two of the store's PostgreSQL plan tests fail — an unread count in one folder scans the account-wide partial index `message_state_account_unread` with a filter instead of `message_state_unread`, and a deep-offset page without its index uses a bitmap scan instead of the sequential scan the test's negative control expects | plan changes on queries whose plans the store's tests pin | none |
| `effective_io_concurrency` | 200 on SSD | prefetch for bitmap heap scans | none on SSD | none |
| `maintenance_io_concurrency` | 100 | prefetch in VACUUM and index builds | — | none |
| `jit` | `off` | Mailwoman's statements are short OLTP queries; JIT compilation is triggered by cost estimates (`jit_above_cost` 100,000) and a misestimated list or search query can spend tens of milliseconds compiling | none for this workload | none |
| `plan_cache_mode` | `auto` (default) | prepared statements switch to a generic plan after five executions if it is not costlier; account sizes differ by orders of magnitude, so watch `pg_stat_statements` for statements whose mean time is far above their minimum, and set `force_custom_plan` for the database only if that appears | `force_custom_plan` plans every execution (more CPU) | none |
| `default_toast_compression` | `lz4` | cheaper compression of large `envelope_json` and other TOASTed values; applies to values written from then on | none | none |
| `max_worker_processes` (restart) | `max(8, C)` | room for parallel workers and autovacuum | — | none |
| `max_parallel_workers` | `max(2, C / 2)` | parallel scans of large tables (maintenance, search fallbacks) | parallel queries take CPUs from OLTP backends | none |
| `max_parallel_workers_per_gather` | 2 | | as above | none |
| `max_parallel_maintenance_workers` | `clamp(C / 4, 2, 4)` | index builds in migrations | | none |

## Autovacuum and per-table storage

| Setting | Value | Why | Risk | Durability |
|---|---|---|---|---|
| `autovacuum_max_workers` (restart) | `clamp(C / 4, 3, 8)` | many hot tables | each worker takes I/O | none |
| `autovacuum_naptime` | 15 s dedicated, 1 min colocated | hot small tables (per-account and per-folder rows rewritten on every message change) should be pruned often | more frequent wake-ups | none |
| `autovacuum_vacuum_cost_limit` | 2000 | the default 200 (with 2 ms delay) limits vacuum to a few MB/s, too slow for `messages` on SSD | more background I/O | none |
| `autovacuum_vacuum_scale_factor` | 0.05 | the default 0.2 waits for 20% dead rows: 2 million on a 10-million-row `messages` table | — | none |
| `autovacuum_vacuum_insert_scale_factor` | 0.05 | append-only change logs and journals | — | none |
| `autovacuum_analyze_scale_factor` | 0.02 | keeps statistics fresh as mailboxes grow | — | none |

Migration 0130 sets storage parameters on the tables every flag change updates
(PostgreSQL only; it takes a `SHARE UPDATE EXCLUSIVE` lock, no rewrite):

| Table | fillfactor | autovacuum | Why |
|---|---|---|---|
| `messages` | 85 | scale factors 0.02 | room on each page for the new row version, so a flag update stays HOT (no index entries) |
| `mail_data_rows` | 85 | vacuum scale factor 0.02 | one row per message, rewritten on each change |
| `mail_data_outbox`, `mail_data_accounts` | 50 | scale factor 0, threshold 1,000 | one row per account, rewritten on each message change |
| `mailboxes` | 70 | scale factor 0, threshold 1,000 | folder counters, rewritten on each unread change |

fillfactor applies to pages written after the change. To apply it to existing
data, rewrite the table in a maintenance window (`VACUUM FULL messages;` takes
an `ACCESS EXCLUSIVE` lock for the duration, or use `pg_repack`). A rewrite, or
any `ALTER TABLE … SET`, changes the schema stamp that prepared mail entries are
checked against; they are revalidated once.

`message_state` is not listed: its partial indexes test the `seen` and
`flagged` columns, so a flag change is never HOT there.

## Safety settings

| Setting | Value | Why | Risk | Durability |
|---|---|---|---|---|
| `idle_in_transaction_session_timeout` | 10 min | a session left inside a transaction holds back vacuum for every table | the server ends such a session and rolls its transaction back (nothing committed is lost). Mailwoman never leaves a transaction idle while waiting on the network | none |
| `statement_timeout` | 0 (default) at server level | migrations and start-up backfills can run longer than any request limit; Mailwoman sets its own limits on its maintenance and mirror connections | a server-wide value would abort migrations on large databases | none |
| `lock_timeout` | 0 (default) at server level | as above: migrations wait for their locks | — | none |
| `transaction_timeout` (17) | not set | same reason as `statement_timeout` | — | none |
| `log_lock_waits` | on | logs a lock wait longer than `deadlock_timeout` (1 s) | log volume under contention | none |
| `log_checkpoints` | on (default since 15) | shows whether checkpoints are time- or WAL-driven | — | none |
| `log_autovacuum_min_duration` | 10 s | shows long vacuums | — | none |
| `log_temp_files` | 16 MB | shows queries that need more `work_mem` | — | none |
| `log_min_duration_statement` | off by default; when enabled for diagnosis, also set `log_parameter_max_length = 0` | Mailwoman binds mail metadata (addresses, subjects, ids) as parameters; slow-statement logs would otherwise include them | the values end up in the server log | none |

## Diagnosis

- `shared_preload_libraries = 'pg_stat_statements'` (restart),
  `pg_stat_statements.max = 5000`, `pg_stat_statements.track = top`, then
  `CREATE EXTENSION pg_stat_statements;` in the Mailwoman database. It stores
  statements without their parameter values. Cost: a hash lookup per statement.
- `track_io_timing = on` and `track_wal_io_timing = on`: I/O and WAL flush
  times in `pg_stat_statements`, `pg_stat_io` and `pg_stat_wal`. Check the
  clock cost first with `pg_test_timing`; on virtual machines with a slow clock
  source each timing call is expensive.
- HOT share of a table: `SELECT n_tup_upd, n_tup_hot_upd FROM pg_stat_user_tables
  WHERE relname = 'messages';`.

## The start-up check

At store open on PostgreSQL, Mailwoman reads `fsync`, `full_page_writes`,
`synchronous_commit`, `shared_buffers` and `max_wal_size` from `pg_settings`
and writes a line beginning `[mw-store] PostgreSQL settings:` to standard error
for each of these cases. It never refuses to start.

- `fsync`, `full_page_writes` or `synchronous_commit` is `off` (always).
- `shared_buffers` is at most 128 MB or `max_wal_size` at most 1 GB, and the
  Mailwoman process sees at least 4 CPUs. Mailwoman cannot see the database
  server's memory; its own CPU count stands in for "a deployment with
  resources".

## What `deploy.sh` sets

The full stack (`STACK=full`) starts PostgreSQL with its stock settings plus
`idle_in_transaction_session_timeout` 10 min, `log_lock_waits` on,
`log_checkpoints` on and `log_autovacuum_min_duration` 10 s
(`stack_postgres_settings` in `scripts/deploy-stack.sh`, kept equal to
`DEPLOY_SAFE_SETTINGS` in `scripts/perf/postgres-profile.mjs` by a test). None
of these changes how a query is planned or executed; durability settings keep
their defaults. The sized profiles are opt-in (see the note under the profile
table) until a request-level A/B shows them not slower than stock.

What `deploy.sh` does size from the host is the app's memory limit and Valkey's
`maxmemory`, from the shares in [Small hosts](#small-hosts-and-the-colocated-profile):
on hosts below 4 GiB Valkey gets less than its former fixed 256 MB (64 MB on
1 GiB, 128 MB on 2 GiB), and the app's limit is lowered from 4g to its share
when that is smaller. Re-running a deploy after resizing the host recomputes
both.

To apply a profile to a `deploy.sh` full stack: run
`node scripts/perf/postgres-profile.mjs --cpus N --memory M --profile colocated --sql`
on the host, pipe the statements to
`docker compose exec -T postgres psql -U mailwoman -d mailwoman` in the
deployment directory, and restart the `postgres` service. The settings live in
the database volume (`postgresql.auto.conf`) and survive redeploys;
`ALTER SYSTEM RESET ALL` and a restart return to the default.

The Helm chart does not start PostgreSQL (its default database is SQLite);
configure the server you point `MW_DB_PATH` at with this page, or start it
with the output of `node scripts/perf/postgres-profile.mjs --cpus N --memory M`.

## Related

- [PostgreSQL backend](../deploy/postgres.md)
- [PostgreSQL usage audit](../perf/postgresql-audit.md)
- [PostgreSQL flag-update measurements](../perf/postgresql-flag-update-bench.md)
- [Database connection pools](../perf/database-pools.md)
