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

## Regression comparison (standing rule: do not regress)

`scripts/perf/rig-baseline.json` holds the tracked numbers per goal-mix shape and backend. The first rows come from
t34-bench1's R1 run on image 5717b070f (`.orchestration/reports/perf-investigation/t62-r0a-r1-20261124`; SQLite: 17
shapes and the mix; PostgreSQL: 14 shapes and the mix). That image did not count commits, so `databaseCommits` is
null there and not compared until a bless adds it.

```sh
# a run against the committed baseline (exit 1 on a regression)
node scripts/perf/bench-rig.mjs compare --candidate=<evidence>/<run-id> [--out=<path prefix>]
# two candidate repeats against two parent repeats
node scripts/perf/bench-rig.mjs compare --candidate=<b1>,<b2> --reference=<a1>,<a2>
# the default flow for a new image: parent vs candidate, order A B B A, then the comparison
node scripts/perf/bench-rig.mjs ab --run=r1 --run-id=<id> --parent-image=<A> --candidate-image=<B> \
  --parent-diagnostics-image=<A diag> --candidate-diagnostics-image=<B diag> \
  --report-root=<evidence dir> --private-root=<private dir> --confirm-bench-slot
# replace the baseline rows of the candidate's backend (never automatic)
node scripts/perf/bench-rig.mjs compare --candidate=<run> --bless --reason="<why these are the new numbers>"
```

Verdict per shape:

- Counters regress on any increase: statements per request (reads, writes, other = BEGIN and the rest), database
  commits, upstream commands per request, errors, 503, 429, timeouts. Per-request counts are compared at 0.1 when
  process-wide (they include background work; the second decimal is not reproducible) and at 0.01 when
  request-scoped. An increase in any candidate repeat counts.
- Noisy numbers (µs CPU per request, rps, rps per consumed CPU, p50, p99) regress only when worse by more than the
  threshold (default 8%, `--threshold`) in every repeat, with at least two repeats (A B B A pairs a1/b1 and a2/b2).
  With one repeat a worse value is `unconfirmed` and does not fail. Contaminated stages are not compared on them.
- A shape with no reference row is `no-reference`, not `same`.

`ab` writes `<run-id>-compare.md/.json` beside the four runs. A bless appends `{at, reason, from, backends}` to the
file's `blessed` list.

Basis difference to keep in mind: the 5717b070f rows divide CPU by completed requests and use ~1 s samples; the rig
divides by requests accepted inside the window and samples every 250 ms. For a closed-loop stage the two counts differ
by the requests in flight at the window end.

## Memory per stage (t63 memory audit, section 6)

Each stage row has `memory`:

| Field | Source |
| --- | --- |
| `containers.<label>`: `currentStartBytes`, `currentEndBytes`, `maxSampledBytes`, `peakBytes`, `events` (high, max, oom, oom_kill increase), `statEnd` (anon, file, kernel, sock, shmem), `pidsEnd` (tasks = threads) | cgroup v2 `memory.current`, `memory.peak`, `memory.events`, `memory.stat`, `pids.current` of each run container, sampled with the CPU counters |
| `server.process` (start, end): resident, anonymous, file-backed, shared, peak resident, threads | fixture diagnostics `processMemory` (`/proc/self/status`) |
| `server.pool`, `server.sqlitePageCache`, `preparedMailChargedBytes`, `writeBehindPending`, `sessionTouchBuffer` (held, pending), `searchWriterMemoryBudgetBytes` | fixture diagnostics |
| `derived.appBytesPerConnectedUser` | app cgroup memory at the stage end / users accepted in the window (one session per user in the fixture) |
| `postgres.processes`, `postgres.memoryCurrentEndBytes` | the database container's `pids.current` (backends plus auxiliary processes) and `memory.current` |

Not exposed by the fixture today, so not reported: the mail cache layer's weighted size and the public asset cache.
The table shows app memory at the stage end (largest sampled inside), bytes per user and OOM/max events.

`compare` adds `appMemoryEndBytes` (regression beyond 10% in every one of two repeats) and `oomEvents` (any increase).
The 5717b070f baseline has no memory numbers; they are compared after the first bless that includes them.

Every summary records the fixture image's C library and allocator (`imageRuntime`, from `ldd` inside the image).
Checked on 2026-11-25 for `t62-fixture:5717b070f`: Debian 12, glibc 2.36, dynamically linked `x86_64-unknown-linux-gnu`,
no `#[global_allocator]` in the workspace, so glibc malloc. Shipped installs use the musl static image (t63 audit), so
memory and allocator numbers from the fixture do not transfer to them as they are.

## As-shipped fixture and allocator A/B (t63 measurement M3)

`--target fixture-static` of `scripts/perf/Dockerfile` builds the native fixture the way the shipped image is linked
(root `Dockerfile` `build-musl`): `rust:1.98.1-alpine` (same digest), `x86_64-unknown-linux-musl`,
`RUSTFLAGS="-C target-feature=+crt-static"`, so musl libc and musl's malloc, statically linked. The image is the
ordinary fixture image with only `/out/request-load-fixture` replaced (Node and the scripts stay for the
generator); its gate checks the binary has no `libc.so` and passes the readiness tests. `MW_FIXTURE_DIAGNOSTICS`
applies to it too. Every report records libc and allocator from `ldd` (`imageRuntime`).

```sh
git archive --format=tar <commit> | docker build -f scripts/perf/Dockerfile --target fixture-static \
  --build-arg MW_BUILD_REVISION=<40-hex commit> -t rig-fixture-static:<short> -
node scripts/perf/bench-rig.mjs m3 --run=m3 --run-id=m3-1 --arm-image-glibc=rig-fixture:<short> \
  --arm-image-musl=rig-fixture-static:<short> [--arena-max=2] --report-root=<dir> --private-root=<dir> --confirm-bench-slot
```

Arms: `glibc` (default), `glibc-arena` (`MALLOC_ARENA_MAX`, `--arena-max`, default 2: a commonly used setting, not
derived from a measurement here), `musl` (as shipped), `jemalloc`, `mimalloc`. An arm runs only when its image is
given; the jemalloc and mimalloc arms need images built with allocator features that do not exist yet (they need
new crates from crates.io; waiting for approval). Order: `--repeat` (default 2, at most 6) runs the arms forward,
then reversed (A B C, C B A, ...). Stages per arm: idle 60 s after seeding with every session open
(`MW_LOAD_FIXTURE_IDLE_BEFORE_SECONDS`); identity, flag-and-restore and the goal mix, each at C32 and C64 for 120 s;
idle 300 s (`MW_LOAD_FIXTURE_IDLE_AFTER_SECONDS`; both 0–900, default 0) with readings at +60 s and at the end
(+300 s). The fixture writes the idle windows to `idle-windows.jsonl`. `<run-id>-m3.md` has one row per arm repeat
and stage. Memory cells are "anonymous / total": anonymous = the app container's cgroup `memory.stat` `anon`
(private memory such as the heap, no page cache); total = `memory.current` (page cache included). The fixture
process's own RSS split (`/proc/self/status`: resident, anonymous, file-backed) is in each stage row under
`memory.server.process`, at the stage edges only. Not built: the audit's "1,000 users open / close / reopen" stage
(the fixture seeds its sessions once; the generator has no close/reopen phase).
