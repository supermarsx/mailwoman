# Bench rig: cpusets, cgroup CPU, commits, quiet host (t62-n12)

The rig runs the measurement plan of `.orchestration/plans/t62-100k.md` section 8: runs R0a (transport floor) and
R1 (per-shape cost of the goal mix). It does not change how the generator measures; it adds pinning, CPU accounting
from the kernel, a commit counter and a contamination verdict around the existing native fixture campaign.

Status: unit-tested only. No campaign has been run with it yet (see "Not proven" below).

## Files

| File | What it does |
| --- | --- |
| `scripts/perf/bench-rig.mjs` | One command: `plan`, `run`, `analyse`. Starts the containers of each leg, waits, copies evidence, removes containers and volumes, writes `summary.json` and `summary.md`. |
| `scripts/perf/rig-cpuset.mjs` | Disjoint cpusets for app / database / generator / upstream / rig; docker `--cpuset-cpus` and `--cpus`; the `MW_LOAD_HOST_*` declarations. |
| `scripts/perf/rig-sampler.mjs` | Sidecar. Reads cgroup v2 `cpu.stat` of the run's containers and of every other container, and `/proc/stat`, every 250 ms. |
| `scripts/perf/rig-cgroup.mjs` | Parsers; counter interpolation at the stage window edges; µs CPU per request per container; rps per logical CPU. |
| `scripts/perf/rig-quiet.mjs` | Quiet-host witness and the `contaminated` verdict. |
| `scripts/perf/rig-host-sampler.ps1` | Optional Windows host process sampler (`--host-sampler`). |
| `scripts/perf/rig-shapes.mjs` | R0a/R1 leg definitions, per-stage database/upstream counters, the summary table. |

Hooks in existing files (additive; no existing group, endpoint or default changed):

- `fixture-requests.mjs`: groups `goal-shape-<name>` (one goal-mix shape alone: the same validated template the mix
  uses, rotating over every user) and `transport-healthz` (GET `/healthz` from `x-forwarded-for: 127.0.0.1`, so the
  probe is admitted).
- `http-campaign.mjs`: those endpoint names, and the alias `goal-shapes` for all 17 shape groups in table order (the
  endpoint list is limited to 16 entries and the handoff environment to 256 characters per value).
- `http-multi-controller.mjs`, `http-multi-protocol.mjs` (`ENDPOINT_LABELS`), `t34-modes.mjs`
  (`GUARD_SNAPSHOT_LOOKUPS`: the transport stage and the two account polls alone never consult the guard snapshot).
- `request-inputs-experiment.mjs`: the shape lists of `goal-65-35`, every `goal-shape-*` group and
  `transport-healthz`, so a run that declares `MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT` (R1 does) no longer fails with
  "Invalid request-input workload shape proof" on those endpoints.
- `http-goal.mjs` `hostFacts()`: `MW_LOAD_HOST_CPUSET_UPSTREAM` and `MW_LOAD_HOST_CPUSET_RIG`, recorded only when set.
- `scripts/perf/Dockerfile`: `--build-arg MW_FIXTURE_DIAGNOSTICS=1` builds the native fixture with the
  `sql-shape-diagnostics` feature (R1's diagnostics leg). Default 0 is the ordinary build.
- mw-store pool telemetry: the commit counter (below).

## Commands

Take the `bench` lock in `.orchestration/state.md` first, and the `quiet` lock for any stage above 8 CPUs. Build the
images from `git archive` of one commit that contains this rig (as the earlier campaigns did):

```sh
git archive --format=tar <commit> | docker build -f scripts/perf/Dockerfile --target fixture \
  --build-arg MW_BUILD_REVISION=<40-hex commit> -t rig-fixture:<short> -
git archive --format=tar <commit> | docker build -f scripts/perf/Dockerfile --target fixture \
  --build-arg MW_BUILD_REVISION=<40-hex commit> --build-arg MW_FIXTURE_DIAGNOSTICS=1 -t rig-fixture-diag:<short> -
```

Print a plan (no Docker):

```sh
node scripts/perf/bench-rig.mjs plan --run=r1 --vcpus=26
```

R0a (SQLite + SQLCipher, enforced profile, app 4 CPUs, generator 4, sampler 1; `/healthz` at C4 and C8, 30 s):

```sh
node scripts/perf/bench-rig.mjs run --run=r0a --run-id=r0a-1 --image=rig-fixture:<short> \
  --report-root=<evidence dir> --private-root=<private dir outside the repository> \
  --vcpus=26 --app=4 --generator=4 --rig=1 --host-sampler --confirm-bench-slot
```

R1 (1,000 users x 50 messages, `goal-65-35` seeding, guard snapshot bounded, write-behind accounting, SQLite write
gate; legs: each of the 17 shapes alone 30 s at C32, the mix 60 s at C64, then the diagnostics build with the request
profile on, 10 s per shape at C32):

```sh
node scripts/perf/bench-rig.mjs run --run=r1 --run-id=r1-1 --image=rig-fixture:<short> \
  --diagnostics-image=rig-fixture-diag:<short> --report-root=<evidence dir> --private-root=<private dir> \
  --vcpus=26 --app=4 --generator=4 --rig=1 --host-sampler --confirm-bench-slot
```

R1 on PostgreSQL 16 (own cpuset of 4; write-behind accounting is SQLite-only, so that leg uses synchronous
accounting): add `--backend=postgres --database=4`.

Generator: `--generator-impl=node` (default; one Node worker process per generator CPU) or
`--generator-impl=native --native-generator-bin=<host path of a Linux mw-loadgen>` (t62-n10,
`docs/perf/native-generator.md`): the binary's directory is mounted read-only at `/rig-bin`, the external driver runs
with one worker and `MW_LOAD_GENERATOR=native`, `MW_NATIVE_GENERATOR_THREADS` = generator CPUs. R1 only:
mw-loadgen has no template for the transport group, so R0a refuses it.

Default cpusets fit the 26-vCPU Docker VM (decision D3): app 4, database 4 (PostgreSQL), generator 4, sampler 1 from
CPU 1. The largest split for M0–M3 (app 12, database 4, generator 8, sampler 1) is CPUs 1–25.

`--legs=release-mix` (comma list) runs a subset; `--smoke` makes every stage 5 s; `--principals=<n>` changes the
user count; `analyse --dir=<evidence dir>/<run-id>` rebuilds the summary from the saved files.

Each leg seeds its own fixture (one fixture process per leg). The private directory holds the database key and the
PostgreSQL password while the leg runs and is deleted afterwards; nothing secret is written to the evidence folder.

## Report fields (summary.json, per stage row)

| Field | Meaning |
| --- | --- |
| `usecCpuPerRequest.server` | CPU time of the server containers (app, plus database and upstream when run) over the stage's scheduled window, from cgroup v2 `cpu.stat` `usage_usec`, divided by requests accepted inside the window. Not docker stats. |
| `usecCpuPerRequest.generator` | The same for the generator container. |
| `cpu.containers.<label>` | Per container: CPU µs, user/system µs, cores used, utilisation of its cpuset, `nr_periods`/`nr_throttled`/`throttled_usec` increase. |
| `rpsPerLogicalCpu.allottedServer` | Accepted rps / logical CPUs in the server cpusets (the plan's "rps per logical CPU"). `allottedApp` uses the app cpuset only; `consumedServer` divides by the CPU the server actually used. |
| `database.perRequest` | Statements per HTTP request by class (reads = SELECT, writes = INSERT/UPDATE/DELETE/REPLACE, other = BEGIN, PRAGMA and the rest), `transactionCommits`, `autocommitWrites`, `databaseCommits` (their sum), `upstreamCommands`. `database.source` says whether the counts are request-scoped (request profile on: the diagnostics leg) or process-wide (plain legs; background work and the drain after the window included). |
| `latencyMs.p50`, `p99` | From the generator, per stage. |
| `contaminated`, `quiet` | Quiet-host verdict (below) with its reasons and the numbers behind it. |
| `mix` (R1) | Weighted server µs per request from the 17 shapes (`GOAL_MIX_TABLE` weights, uncontaminated plain stages only) next to the measured mix. |

`summary.md` has one row per stage with these columns; it is the "today" column of plan sections 3.3 and 4.2.

Rows of the diagnostics leg are labelled instrumented: counts only. The transport row is labelled "transport control
only: GET /healthz bypasses the application guards".

## Commit counter

`mw_store::pool_telemetry::QuerySnapshot` now has `commits: {sqlite, postgres}` (calls, failures, total and maximum
µs of `Tx::commit`) and `autocommitWrites` (successful write statements run outside a transaction, each of which is
its own commit). `databaseCommits()` is their sum. The fields appear in the fixture diagnostics next to the statement
counts (`pool.operations`), and in the request profile (`requestProfile.queries`) when that is on. Commits made by
direct SQLx calls outside `q()`/`Tx` are not seen.

## Quiet host (plan section 8 rule 1)

A stage is `contaminated` (discarded, not averaged) when, over its scheduled window:

1. another container (any container that is not one of this leg's) used more than 0.05 CPU (5% of one CPU);
2. the VM vCPUs outside the run's cpusets were more than 10% busy (from `/proc/stat`);
3. with `--host-sampler`: Windows processes other than the VM's own used more than 10% of the host's logical CPUs, or
   build/test processes (cargo, rustc, linker, compiler, docker build, node, pnpm, vitest, playwright) used more than
   0.25 CPU ("builds paused" witness). Rule 3 is this rig's reading of rule 1, which names only the first two.

Without `--host-sampler`, work on the Windows host outside the Docker VM is invisible, and every report says so.
Reported but not a criterion: CPU on the run's own vCPUs that its containers did not account for (kernel work for the
run's own traffic is not always charged to its cgroup).

## R0a and the probe pool

`/healthz` and `/readyz` have their own admission pool of 8 permits with no wait (`http_admission.rs`, t57-e1).
Above 8 requests in flight the excess is refused, so R0a's default concurrency list is `4,8`; C32/C128 can be asked
for with `--transport-concurrency` and then measure refusals. `/healthz` reads the settings revision (it may be
cached) and runs the health-access check; it skips authentication, rate limits and request protection.

## Runtime and limits for large cpusets (N9)

| Knob | Default | Range | Before |
| --- | --- | --- | --- |
| `MW_RUNTIME_WORKER_THREADS` (server) | `auto`: Rust `available_parallelism`, i.e. the container's cpuset size, or its CPU quota if lower (rounded up) | 1–1024; `TOKIO_WORKER_THREADS` honoured when unset, same bounds | `#[tokio::main]`, which on tokio 1.54 reads the same `available_parallelism`: the default is unchanged, now bounded and logged at start-up |
| `MW_LOAD_FIXTURE_WORKERS` (fixture) | 4 | 1–128 | 1–32 |
| SQLite pool maximum, derived (`database.pool` / `MW_DB_POOL` `sqliteMaxConnections` null) | 2 per CPU, 2–64 | — | 2–8 |
| SQLite pool maximum, explicit (`sqliteMaxConnections`) | — | 1–128 | 1–32 |

An invalid value of any of them refuses start-up. The rig sets `MW_LOAD_FIXTURE_WORKERS` to the app cpuset size.

SQLite: more pooled connections are more concurrent WAL readers. There is still one writer at a time: every write
takes the in-process write permit (`MW_SQLITE_WRITER`) and SQLite's write lock. Memory per connection: up to 2,000 KiB
of page cache with the driver default (decrypted pages under SQLCipher; with `sqlitePageCacheMiB` set, the total is
divided across connections instead), up to 100 prepared statements, and one SQLx worker thread. 64 connections can
hold up to 125 MiB of page cache with the driver default. See `docs/perf/database-pools.md`.

### Admission values a large run must declare

These are capacity settings, not security features; a run report must state the values in force (the start-up line
and `httpAdmission` in diagnostics show them). Derived defaults (t57-e1): connections ≥ 1,024, requests
clamp(max(64 × CPUs, 16 × pool), 256, min(16,384, memory / 4 MiB)), HTTP/2 streams requests / 8 within 32–100.

| Setting | What to declare for a large run | Why |
| --- | --- | --- |
| `MW_HTTP_MAX_CONNECTIONS` (`server.http.maxConnections`) | at least 2 × the generator's client connections | each keep-alive client connection holds one connection permit |
| `MW_HTTP_MAX_REQUESTS` (`server.http.maxRequests`) | at least 2,048 (ESTIMATE in the plan: 200 in flight at 100,000 rps and 2 ms, 500 at 5 ms) | requests above it wait in the FIFO queue, then get 503 |
| `MW_HTTP_REQUEST_QUEUE_MS`, `MW_HTTP_REQUEST_QUEUE_MAX` | state them; defaults 200 ms and the request limit | a refused request is a 503 in the report |
| `MW_HTTP2_MAX_STREAMS` | state it when HTTP/2 is used (1–128) | each stream holds a request permit |
| `ratelimit.jmap.globalConcurrent` | Auto (= derived request limit, at least 256) or an explicit value ≥ the run's JMAP requests in flight | the global JMAP slot gate |
| `ratelimit.jmap.perAccountConcurrent` | unchanged (8) | per-account fairness; the goal mix has one request in flight per user |
| `security.protection.clientRequests`, `accountRequests` | unchanged unless decision D1 (b) is taken; then the first line of the report says "non-default rate limits" | default limits cap 1,000 users at 10,000 rps (plan F2) |

## Not proven

- No live run: the bench slot was held by another benchmark while this was built. The Docker orchestration is
  tested with a recorded fake Docker only.
- The cgroup path layout inside the Docker Desktop VM (`docker/<id>` or `system.slice/docker-<id>.scope`) is
  discovered at run time and not yet observed on this host; a leg whose containers' cgroups are not found reports
  "cgroup not found" per sample and its CPU fields are unavailable rather than zero.
- That the fixture image builds and passes its gated tests with `MW_FIXTURE_DIAGNOSTICS=1`.
- The native generator option with the single-shape groups (the launcher wiring is tested; a live stage is not).
- The VM-to-host clock offset (for the Windows host sampler) is measured once per leg with `docker exec`.
