---
title: PostgreSQL usage audit (t62-n16)
---
# PostgreSQL usage audit

What Mailwoman does to a PostgreSQL server today, read from the code at
`c201b6d69` and from a schema built by applying every file of
`crates/mw-store/migrations_pg` to PostgreSQL 16.15. It is the input to
[PostgreSQL tuning](../admin/postgresql-tuning.md) and to the measurements in
[PostgreSQL flag-update measurements](postgresql-flag-update-bench.md). File
references are `path:line` at that commit.

## Connections and pools

- **Bootstrap pool.** `Store::open_postgres` (`crates/mw-store/src/lib.rs:506`)
  connects with `PgPoolOptions::new().max_connections(5)`, runs the
  migrations and start-up backfills on it, then
  `configure_pool_at_startup` replaces it.
- **Main pool.** `PoolPolicy::postgres_plan`
  (`crates/mw-store/src/pool_policy.rs:143`): maximum =
  `postgresMaxConnections` if set, else `cpus × 4` clamped to 4..64, where
  `cpus` is `available_parallelism()` of the *application* process. That is
  then capped at `(min(postgresConnectionBudget, server max_connections) −
  max(postgresReservedConnections, superuser_reserved_connections +
  reserved_connections)) / replicaCount`. Defaults: budget 100, reserve 20,
  replica count 1, minimum 1, acquire timeout 30 s, idle timeout 60 s, maximum
  lifetime 1,800 s. With a stock server (`max_connections` 100) one process
  can get at most 80; the CPU rule gives 16 on 4 CPUs, 64 from 16 CPUs up.
  The server's limits are read once at start-up with one `current_setting`
  query.
- **Other connections per process.** The authority mirror opens one dedicated
  connection (`authority_publication/mirror.rs:715`); background maintenance,
  body-cache expiry and change-log retention each build their own
  one-connection pools with `after_connect` hooks
  (`background_maintenance.rs:228,249`, `body_cache_expiry.rs:209,225`,
  `change_log_retention.rs:142,154`); one maintenance path uses a 4-connection
  pool (`background_maintenance.rs:711`). So a process holds up to its main
  pool maximum + about 8 connections. `docs/deploy/postgres.md` said "up to 5
  connections (fixed in code)"; that described only the bootstrap pool and is
  corrected in this change.
- **Connection options.** sqlx defaults plus `statement_cache_capacity(100)`
  (`pool_policy.rs:27,341`). `test_before_acquire` stays on for PostgreSQL (one
  extra round trip, a ping, per acquire; asserted at `pool_policy.rs:486-498`).
  `application_name` is set only on the mirror connection. TLS is whatever the
  DSN says; `MW_DB_REQUIRE_TLS=1` refuses a DSN that does not force it.
- **Statement protocol.** Every `q()` statement (`backend.rs:227`) and every
  direct `sqlx::query` runs as a named prepared statement (sqlx default,
  `persistent = true`); nothing in `crates/mw-store/src` sets
  `persistent(false)`. The per-connection cache holds 100 statements, LRU;
  `database_pool_snapshot()` reports the cached count of each idle connection.
  A first use on a connection costs an extra Parse/Describe round trip; an
  eviction costs a Close plus that.

## Session state, isolation, locks

- **Isolation.** READ COMMITTED (server default) everywhere except the
  authority mirror poll: `BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY`, one
  SELECT, on its dedicated connection (`mirror.rs:932`).
- **Session `SET`s.** Only on dedicated connections: the mirror sets
  `statement_timeout` (2 × bound, at least 2 s), `lock_timeout = 100` (ms) and
  `idle_in_transaction_session_timeout = 10000` (`mirror.rs:725-733`); the
  maintenance, body-expiry and retention pools set `lock_timeout = '50ms'` and
  `statement_timeout = '500ms'` in `after_connect`. The main pool sets nothing.
- **Advisory locks.** All transaction-scoped (`pg_advisory_xact_lock` /
  `pg_try_advisory_xact_lock`), released at commit or rollback: per-account
  mail change log and body/ingest serialisation, key
  `hashtextextended(account, 719304)` (`background_maintenance.rs:151,160`,
  used by `v2.rs:1123-1131` and `cache.rs:1717`); PIM and crypto change logs, keys
  719305 / 719306 (`v2.rs:1134-1135`); one job kind lock
  (`jobs.rs:419`); the change-log pruner's try-lock (`mirror.rs:914`). No
  session-level advisory lock exists.
- **Row locks.** `FOR UPDATE` on mailbox and message rows in flag, move and
  delete paths (`cache.rs:1722,1737`, `publication_mutations.rs:18`),
  `FOR KEY SHARE` on the account row (`header_envelope.rs:297`), `FOR UPDATE
  SKIP LOCKED` for the authority outbox claim (`authority_publication.rs:338,396`).
- **LISTEN/NOTIFY.** None. Cross-process freshness is by polling
  (`authority_change_log`, xid horizon), not notifications.
- **Temporary tables.** Only in tests.
- **Long transactions.** No code path keeps a transaction open across an
  upstream (IMAP) command; flag and move transactions are short (comment at
  `cache.rs:1685-1690`). Migrations and start-up backfills run before the
  server serves, on the bootstrap pool, and can be long on a large database.

### What this means for a connection pooler

| Pooler mode | Works? | Why (from the code above) |
|---|---|---|
| PgBouncer **session** pooling | Yes | Each client connection keeps one server connection: prepared statements, the mirror's `SET`s and the `after_connect` `SET`s behave as with a direct connection. It adds a hop and saves nothing for a pool that is already bounded; useful only to multiplex many Mailwoman processes onto fewer server connections when they are mostly idle. |
| PgBouncer **transaction** pooling | Not supported | (1) Named prepared statements: sqlx prepares each statement once per connection and reuses the name; behind a transaction pooler the next transaction can land on a server connection that never saw that `Parse` (`prepared statement "sqlx_s_N" does not exist`). PgBouncer 1.21+ can track protocol-level prepared statements (`max_prepared_statements`), which removes this failure but is not tested with sqlx here. (2) The `SET lock_timeout`/`statement_timeout` of the maintenance pools and the mirror's three `SET`s are session state: they would apply to whichever server connection ran them and leak to other clients. The advisory locks are transaction-scoped and would be fine. |
| Statement pooling | No | Multi-statement transactions. |

The t53s finding stands: no pooler is needed at the connection counts the goal
needs (one process: ≤ 64 + about 8 connections).

## Round trips per mutation (from the code)

Per single-message flag change through `Store::set_flags_batch`
(`cache.rs:1691-1829`) on PostgreSQL: `BEGIN`; the location SELECT; the
advisory lock; `accounts … FOR KEY SHARE`; `mailboxes … FOR UPDATE` (one per
mailbox); `messages … FOR UPDATE`; the `UPDATE messages`; the
`UPDATE mailboxes` counter (when unread changes); `COMMIT` — 9 round trips, one
commit. The engine then records the Email and the Mailbox change with
`record_change` (`v2.rs:834`, called from `engine.rs:901-903`), each in its own
transaction (`append_changes`, `v2.rs:1164`): `BEGIN`, advisory lock,
`INSERT … RETURNING`, `COMMIT` — 4 round trips and one commit each. Total
**about 17 round trips and 3 commits per changed message**, before the request's
guard and session reads. The pool's acquire ping adds one round trip per
acquisition. Measured counts elsewhere: 27 statements per enforced identity
request on PostgreSQL with modes off (t53s part 10), 34–37 statements per moved
message on PostgreSQL in the bulk report (`t52-writegate-bulk-20261124`). The
plan's budget is 1 round trip per write request.

`MAX()+1` sequences: change states are `COALESCE(MAX(state),0)+1` per
`(account, type)` under the per-account advisory lock (`v2.rs:834-856`,
`cache.rs:1108,1933`), so every change of one account is serialised on that
lock until its transaction commits (a commit wait, about one fsync, at least).
Other `MAX()+1` users (`erasure.rs:296`, `scrub.rs:248,474`,
`index_publication/journal.rs:102`) are not on the request path.

## What one flag UPDATE writes (measured)

One `UPDATE messages SET flags_json = … WHERE stable_id = …` that changes
`$seen`, 1,000 times, on 64 accounts × 200 messages (`pg_stat_user_tables`
deltas, PostgreSQL 16, right after `VACUUM`):

| Table | Rows written per flag change | HOT share | Why |
|---|---|---|---|
| `messages` | 1 update | 60% right after VACUUM, 97–99% in steady runs | no index contains `flags_json`, `cache_revision` or `content_revision` (the columns the BEFORE triggers of 0105/0110 rewrite), so every update is HOT-eligible; it is HOT only when the page has room |
| `message_state` (0125) | 1 update | **0%** | partial indexes `WHERE seen = 0` and `WHERE flagged = 1` use the changed columns: every flag change writes new entries in all 5 indexes; fillfactor cannot change this |
| `message_keyword` (0125) | 1 insert or delete | — | keyword rows (`$seen`) with 2 indexes |
| `mail_data_rows` (0113) | 1 upsert | 87% | one row per message, only the PK and `account_id` indexed |
| `mail_data_outbox` (0113) | 2 upserts | 100% | one row per account, written for OLD and NEW account |
| `mailboxes` | 1 update (in the store path, not the bare UPDATE) | ~100% | counters are not indexed |

Every flag change therefore rewrites two per-account rows
(`mail_data_outbox`, and `mail_data_accounts` on focus changes) and one
per-mailbox row. With 64 accounts these rows fill one heap page each, so
concurrent writers of different accounts contend on one buffer; see the
measurement page for the effect at 32–64 clients. Trigger CPU is the dominant
cost (t50hp-e30: 0.24 ms per row in 0125, 0.34–0.81 ms in 0113; t62-n15 is
replacing 0113's row triggers).

TOAST: `messages.envelope_json` (bytea) is stored inline up to about 2 KB and
TOASTed above; a flag update does not rewrite a TOASTed value (the TOAST
pointer is copied), so TOAST adds no write per flag change. Bodies are in
`bodies` and the blob store, not touched by flag changes.

## Storage parameters and autovacuum exposure

- No application table has `reloptions` (`pg_class.reloptions` is NULL for all
  of them): fillfactor 100 and the global autovacuum thresholds everywhere.
- Hot small tables (rows rewritten far more often than the table has rows):
  `mail_data_outbox` and `mail_data_accounts` (per account, every message
  change), `mailboxes` (per folder, every unread change), `authority_epoch`,
  `authority_publication_outbox`, `settings_revision`, `identity_serialization`,
  `change_log_checkpoint`, `sessions` (`last_seen`, now buffered by the touch
  buffer). With the default `autovacuum_vacuum_scale_factor` 0.2 and threshold
  50, a table of a few hundred rows qualifies after a few hundred dead rows,
  so the scale factor is not the limit; what limits cleanup is the 1-minute
  naptime, the cost limit (200), and any transaction that holds back the xmin
  horizon. HOT pruning on page access does most of the work for these tables
  as long as updates stay HOT.
- Append-mostly tables: `changes`, `pim_changes`, `crypto_changes`,
  `index_publication_journal`, `authority_change_log`, audit tables. They need
  `autovacuum_vacuum_insert_scale_factor` (PostgreSQL 13+) to keep their
  visibility maps current; pruning is done by the app's retention jobs.
- `messages` is the large table: at the default scale factor 0.2, a 10-million
  row table is vacuumed only after 2 million dead versions.

## Schema stamps and storage changes

`mail_data_schema.rs:15` and `native_publication_schema.rs:71` hash each stamped
table's `pg_class` `oid`, `xmin` and `relfilenode`. `ALTER TABLE … SET
(fillfactor …)` rewrites the table's `pg_class` row, so its `xmin` changes: the
stamp changes once and prepared entries are revalidated once (fail-safe, no
refusal). The same happens when an administrator applies a storage parameter by
hand.

## What the deployment artefacts start

| Artefact | PostgreSQL | Settings before this change |
|---|---|---|
| `deploy.sh` full stack (`scripts/deploy-stack.sh`) | `postgres:16` pinned by digest, `--data-checksums`, SCRAM, `shm_size: 128m`, no `command` | stock: `shared_buffers` 128 MB, `max_connections` 100, `max_wal_size` 1 GB, `checkpoint_timeout` 5 min, `jit` on, `random_page_cost` 4, `wal_compression` off, `effective_cache_size` 4 GB |
| Helm chart (`packaging/helm/mailwoman`) | does not start PostgreSQL; `MW_DB_PATH` defaults to SQLite | — |
| `docker-compose.ci.yml` | `postgres:16` for CI tests | stock (CI only, not a deployment) |
| Benchmark launchers (`.orchestration/reports/…/launch-pg-*.ps1`, not shipped) | `postgres:16`, 4 CPUs, 4 GiB, cpuset 4-7 | stock; `max_connections`, `shared_buffers`, `synchronous_commit`, `fsync`, `work_mem` recorded in the report |

So every PostgreSQL number measured so far, benchmark legs included, was taken
with stock settings.

## Larger items for the write-path executors (listed, not implemented here)

Estimates are arithmetic on the measurements of this audit and the bench page,
not measurements of the change itself.

| Item | What | Estimated effect |
|---|---|---|
| One transaction per Email/set | fold the two `record_change` transactions into the flag/move transaction (the lock is already the same advisory key) | 17 → about 11 round trips and 3 → 1 commits per changed message; at a 0.6–2.1 ms fsync that is the larger part of the latency of a single-message write |
| Set-based CTE per Email/set | lock plan, row locks, flag UPDATE, counter UPDATE and change-row INSERT as one statement with data-modifying CTEs | about 11 → 3 round trips (`BEGIN`, statement, `COMMIT`; or 1 with an implicit transaction) |
| Pipelining | sqlx 0.8 has no public pipeline API; the same effect needs the CTE or batching several requests' statements | — |
| Group commit in the app | several requests' mutations per transaction (the PostgreSQL counterpart of t50hp-e19) | commits/s from one per mutation to the plan's 500–1,000; removes the advisory-lock commit wait from the per-account critical path |
| `COPY` for bulk ingest | ingest of many messages (`COPY` into a staging table + one `INSERT … SELECT`) instead of per-row statements | ingest only; not on the goal mix's path |
| `message_state` partial indexes | make flag columns non-indexed (e.g. separate small table for unread/flagged sets, or BRIN/none and a counter) so flag changes are HOT | removes 5 index insertions per flag change; needs the list query owners (t50hp-e31) to agree |
| Per-account hot rows | `mail_data_outbox` / `mail_data_accounts` rewritten per message change: coalesce per statement (t62-n15) or per transaction | fewer row versions and less buffer contention between accounts on one page |
