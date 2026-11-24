---
title: PostgreSQL flag-update measurements (t62-n16)
---
# PostgreSQL flag-update measurements

Small-scale measurements behind [PostgreSQL tuning](../admin/postgresql-tuning.md),
taken on 2026-11-24 while a benchmark campaign ran on the same host. Read them as
"what limits a flag write on a 4-CPU container", not as capacity figures.

## Set-up

- Server: `postgres:16` (16.15), one container, `--cpus=4 --memory=8g`. The
  limit is a CFS quota, not a cpuset: PostgreSQL sees all 26 vCPUs of the WSL2
  VM and is throttled to 4 CPUs of time. Data directory on the container's
  volume on the VM disk; `pg_test_fsync` there: `fdatasync` 575 µs per 8 KB
  write (t34 measured 2.1 ms on the same kind of volume under other load).
- Client: `pgbench -M prepared` (named prepared statements, as sqlx uses) in a
  separate container limited to 2 CPUs, TCP over a Docker network. Each run 20 s.
- Schema: every file of `crates/mw-store/migrations_pg` at `c201b6d69`, all
  triggers enabled. Data: 64 accounts × 1 folder × 200 messages
  (`scripts/perf/pg-flag-bench/seed.sql`), 12,800 rows of about 0.8 KB; the
  whole database fits in the stock 128 MB `shared_buffers`, so these runs cannot
  show the effect of a larger cache.
- Scripts (`scripts/perf/pg-flag-bench/`):
  - `bare.sql`: one autocommit `UPDATE messages SET flags_json = …` toggling
    `Seen` on a random message of a random account — the single-row flag UPDATE
    with every trigger it fires.
  - `app.sql`: the statement sequence `Store::set_flags_batch` sends for one
    message (`BEGIN`, location SELECT, advisory lock, account `FOR KEY SHARE`,
    mailbox `FOR UPDATE`, message `FOR UPDATE`, flag UPDATE, counter UPDATE,
    `COMMIT`) followed by one change-log transaction (`BEGIN`, advisory lock,
    two `INSERT … MAX(state)+1`, `COMMIT`); each client works on its own
    account. The product today runs two change-log transactions (one per
    change type), so this script understates its round trips by four.
- Database CPU per transaction = cgroup `cpu.stat` `usage_usec` delta of the
  server container / transactions. Wait events: 100 samples of
  `pg_stat_activity` 50 ms apart in the middle of each run ("CPU" = active
  with no wait event). WAL bytes = `pg_current_wal_lsn()` delta.
- Configurations: *stock* = the image's defaults; *dedicated* = the profile of
  `scripts/perf/postgres-profile.mjs --cpus 4 --memory 8GiB` (`shared_buffers`
  2 GB, `max_wal_size` 4 GB, `wal_compression` lz4, `jit` off,
  `checkpoint_timeout` 15 min, autovacuum settings; as measured it also had
  `random_page_cost` 1.1, removed from the profile afterwards for the plan
  reason below — the measured statements are primary-key and unique-key
  lookups either way); *benchmark* = dedicated +
  `pg_stat_statements` + `track_io_timing`. `synchronous_commit` on in all.

## Noise

Identical configurations repeated minutes apart differed by up to 46% in
transactions/s (`app.sql`, 8 clients, stock: 864 and 1,264 tps). One pair of
runs that was meant to compare `commit_delay` 0 and 100 turned out to have run
both arms with 0 (the setting had not been applied) and still showed an
18–28% "gain" in four of six pairs. Throughput differences below about 30%
between single runs on this host mean nothing; only alternated repeats and
per-transaction quantities (CPU, WAL bytes, HOT share) are used below.

## Results

### Stock versus profile (alternated: stock, dedicated, stock, dedicated)

| Script, clients | stock tps | dedicated tps | stock DB CPU µs/tx | dedicated DB CPU µs/tx | stock WAL B/tx | dedicated WAL B/tx |
|---|---|---|---|---|---|---|
| bare, 8 | 3,060 / 3,537 | 2,449 / 3,094 | 1,117 / 1,058 | 1,133 / 1,158 | 2,434 / 2,359 | 2,303 / 2,203 |
| bare, 64 | 1,545 / 1,964 | 1,658 / 1,509 | 2,621 / 2,070 | 2,390 / 2,688 | 2,716 / 3,090 | 2,295 / 2,344 |
| app, 8 | 864 / 1,264 | 1,239 / 1,056 | 3,219 / 3,046 | 3,176 / 3,613 | 3,841 / 3,299 | 3,001 / 3,071 |
| app, 64 | 858 / 862 | 792 / 911 | 4,673 / 4,684 | 5,142 / 4,478 | 4,562 / 4,178 | 3,391 / 3,318 |

Series at 1 / 8 / 32 / 64 clients (one run each; stock, then benchmark):

| Script | config | tps at 1 / 8 / 32 / 64 | latency ms at 1 / 8 / 32 / 64 | DB CPU µs/tx at 1 / 8 / 32 / 64 |
|---|---|---|---|---|
| bare | stock | 489 / 2,831 / 2,084 / 1,983 | 2.05 / 2.83 / 15.4 / 32.3 | 1,085 / 1,256 / 1,906 / 1,993 |
| bare | benchmark | 525 / 3,038 / 2,049 / 1,669 | 1.90 / 2.63 / 15.6 / 38.3 | 1,162 / 1,112 / 1,956 / 2,433 |
| app | stock | 132 / 1,293 / 938 / 829 | 7.59 / 6.19 / 34.1 / 77.2 | 3,459 / 2,970 / 4,296 / 4,882 |
| app | benchmark | 163 / 626 / 916 / 885 | 6.13 / 12.8 / 34.9 / 72.3 | 3,565 / 3,727 / 3,926 / 4,602 |

Reading:
- The profile changes no throughput or CPU figure beyond the noise. It lowers
  WAL per transaction by 5–27% (lz4 full-page images, fewer checkpoints); that
  matters at write volume, not on this 11 MB data set.
- **Per flag change the database spends about 1.1 ms of CPU for the bare
  UPDATE and 3.0–3.6 ms for the store's statement sequence** at 1–8 clients,
  more under contention. The plan's budget is 100 µs. At the goal's 68,000
  mutations/s that is 75 CPUs (bare) or about 220 CPUs (store sequence) of
  database time; the plan gives PostgreSQL 16. No server setting closes a gap
  of 11–35×: the cost is in the triggers (t50hp-e30, t62-n15) and the number
  of statements and commits per mutation (audit).
- Throughput peaks at 8 clients (2 × the CPUs) and falls by 30–45% at 32–64
  clients, with database CPU per transaction rising 1.6–2×.

### Re-measured on top of 0129 and 0130

After t62-n15's 0129 (mail-data triggers per statement) and this change's
0130, on a fresh schema and seed (one run each, stock then dedicated):

| Script | config | tps at 1 / 8 / 32 / 64 | DB CPU µs/tx at 1 / 8 / 32 / 64 | WAL B/tx at 1 / 8 / 32 / 64 |
|---|---|---|---|---|
| bare | stock | 476 / 3,282 / 1,904 / 1,494 | 1,134 / 1,076 / 2,120 / 2,716 | 4,349 / 2,482 / 2,858 / 3,311 |
| bare | dedicated | 468 / 3,309 / 1,614 / 1,265 | 1,170 / 1,061 / 2,501 / 3,229 | 2,530 / 2,016 / 2,156 / 2,231 |
| app | stock | 129 / 1,165 / 826 / 716 | 3,627 / 3,044 / 4,887 / 5,643 | 3,737 / 3,126 / 4,110 / 4,698 |
| app | dedicated | 140 / 945 / 957 / 688 | 3,838 / 3,414 / 4,217 / 4,882 | 3,388 / 2,878 / 3,009 / 3,347 |

The database CPU per single-message flag change is unchanged by 0129 (about
1.1 ms bare, 3.0–3.8 ms for the store sequence at 1–8 clients): 0129 removes
the per-row cost that grows with the number of rows in one statement, and
these are one-row statements. The profile again lowers WAL per transaction
(by 19–42% here) and leaves throughput within the noise.

### What limits it

| Clients | Dominant samples (bare / app) | Meaning |
|---|---|---|
| 1 | IO:WALSync, CPU | each commit waits for its own WAL flush (≈ 0.6 ms) plus trigger CPU |
| 8 | CPU ≈ 50%, LWLock:WALWrite ≈ 40%, IO:WALSync | CPU-bound with a full commit queue; flushes are already shared (2.4 commits per `fdatasync` for bare at 8 clients) |
| 32–64 | LWLock:WALWrite, Lock:transactionid (bare), LWLock:LockManager, LWLock:WALInsert, LWLock:BufferContent (app) | commit queue behind one flusher; row-lock waits where random accounts collide on the per-account `mail_data_outbox` row; lock-manager and buffer contention on the one-page per-account tables. At 64 clients one flush covers about 10 commits and takes about 6 ms, which is far above the 0.6 ms idle `fdatasync`: the flushing backend is throttled by the CPU quota along with 63 others (a container effect, see below) |

### Group commit (`commit_delay` 100 µs, `commit_siblings` 5; dedicated profile; alternated 0, 100, 0, 100)

| Script, clients | delay 0 tps | delay 100 tps | delay 0 flushes | delay 100 flushes |
|---|---|---|---|---|
| bare, 8 | 3,025 / 2,551 | 2,484 / 3,160 | 25,299 / 21,529 | 18,041 / 20,487 |
| bare, 32 | 2,295 / 2,094 | 2,337 / 2,325 | 6,077 / 5,836 | 5,188 / 5,203 |
| bare, 64 | 1,992 / 1,683 | 1,863 / 1,717 | 3,823 / 3,435 | 2,977 / 2,549 |
| app, 32 | 941 / 967 | 974 / 988 | 6,669 / 7,286 | 5,090 / 5,611 |

`commit_delay` reduced WAL flushes by 10–29% (23% for the store sequence) and changed throughput by less
than the noise. PostgreSQL already shares flushes between concurrent commits;
with the per-account advisory lock held to commit, a delay also lengthens each
account's serialised section. Not recommended until a native run on the target
storage shows a gain.

### HOT updates and fillfactor (migration 0130)

12,800 bare flag updates + 1,600 store-sequence updates (8 clients) right after
`VACUUM FULL` of the four tables, alternated:

| fillfactor | `messages` pages | `messages` HOT | `mail_data_rows` HOT | `mail_data_outbox` HOT | `message_state` HOT | WAL for the run |
|---|---|---|---|---|---|---|
| 100 (stock) | 1,423 | 88.5% / 82.6% | 96.0% / 92.2% | 99.9% / 98.9% | 0% / 0% | 40.7 / 41.3 MB |
| 0130 (85 / 85 / 50) | 1,600 | 97.4% / 99.4% | 100% / 100% | 99.5% / 100% | 0% / 0% | 38.9 / 38.5 MB |

With 0130 the first update of nearly every row after a rewrite stays HOT (no
index entries), and WAL falls by 5–7% for this run, at the cost of 12% more
heap pages for `messages`. In long steady runs HOT on `messages` was already
97–99% at fillfactor 100, because page pruning frees room after the first
non-HOT update; the gain is concentrated after bulk loads, rewrites and
`VACUUM`. `message_state` stays at 0% HOT whatever the fillfactor (its partial
indexes test the flag columns).

### Round trips: TCP versus Unix socket; connection cost

Client inside the server container (same CPU quota for both arms), one
client, 15 s, alternated twice:

| Path | `SELECT 1` latency | store sequence latency | new connection + one statement |
|---|---|---|---|
| TCP 127.0.0.1 (trust) | 0.171 / 0.179 ms | 5.82 / 7.06 ms | 5.64 / 5.34 ms |
| Unix socket (trust) | 0.132 / 0.137 ms | 5.79 / 6.46 ms | 5.39 / 4.95 ms |
| TCP over the Docker network (SCRAM) | 0.156 / 0.158 ms | 6.03 / 5.79 ms | 16.0 / 15.1 ms |

- A Unix socket saves about 40 µs per round trip against loopback TCP; with
  about 17 round trips per changed message that is about 0.7 ms of latency per
  mutation, inside the noise of the store-sequence column here.
- A new connection costs about 5 ms (backend fork and start-up) and SCRAM adds
  about 10 ms more. A pool that keeps its connections pays this rarely; a pool
  that shrinks after 60 s idle and regrows under a burst pays it per new
  connection.

## Not measured here, and what a native run must repeat

- **Cache size.** The data set fits in 128 MB, so `shared_buffers`,
  `effective_cache_size` and `random_page_cost` had nothing to act on. A run
  with a database several times larger than `shared_buffers` (the 1,000-user ×
  50-message seed, or larger) is needed.
- **Checkpoints at volume.** 20-second runs at ≤ 9 MB/s of WAL never reached
  `max_wal_size`; at the goal's rate (68,000 mutations/s × 2–4 KB ≈ 140–270
  MB/s, arithmetic) the stock 1 GB forces a checkpoint every 4–7 s.
- **fsync.** The VM volume's flush time (0.6 ms idle here, 2.1 ms in t34) and
  the CPU-quota throttling of the flushing backend distort the commit-queue
  figures; a native Linux run on the target NVMe with a cpuset (not a quota)
  must repeat the 1/8/32/64-client series and the `commit_delay` A/B.
- **16 CPUs.** Contention figures (lock manager, buffer content on per-account
  pages, WAL insert locks) depend on the CPU count; the 16-CPU database of the
  final layout was not available.
- **Plans with `random_page_cost` 1.1.** The store's PostgreSQL test suite
  (`cargo test -p mw-store --lib` with `MW_TEST_PG`) passes on stock settings
  and on the profile, and fails two plan tests when `random_page_cost` is 1.1
  (reproduced twice with only that setting changed):
  `cache::tests::postgres_deep_offset_page_uses_the_covering_index` (its
  negative control expects a sequential scan; it gets a bitmap scan and sort)
  and `message_projection::tests::query_plans_use_the_projection_indexes_postgres`
  (the folder unread count scans `message_state_account_unread` with a mailbox
  filter instead of `message_state_unread`). The profiles therefore keep 4; the
  second plan is worse on accounts with many folders and needs the query
  owner's look before any deployment lowers the value.
- **JIT.** None of these statements reaches `jit_above_cost`, so `jit` on/off
  cannot differ here; the setting is protection against misestimated large
  queries, not measured.
- **Statement cache.** The store has about 1,700 distinct static statements
  (`q("…")` literals in `crates/mw-store/src`) against a per-connection cache
  of 100. Whether the goal mix's hot set per connection exceeds 100 needs the
  `statementCache` section of `database_pool_snapshot()` from a PostgreSQL
  goal-mix run (`idleConnectionsAtCapacity` > 0 means eviction and re-Parse
  round trips).
