# Native load generator (t62-n10)

`scripts/perf/native-generator/` is a Rust counterpart of the Node load
generator's measured stage (`measure()` in `scripts/perf/http-load.mjs`). The
Node generator stays the reference. With `MW_LOAD_GENERATOR=native`, the Node
driver (`http-real-fixture.mjs`) still runs discovery, preflights, isolation
checks, diagnostics, stop rules and report assembly. Only the measured stages
run in the native process (`mw-loadgen`), one process per stage. Each stage's
raw counters are rebuilt into the stage object by `native-measure.mjs`, using
the functions `http-load.mjs` already uses (`Measurements.merge/report`,
`mergeBytesSnapshots`, `bytesReport`, `mergeHttp503Reasons`,
`mergeLatencyBreakdowns`, `goalMixReport`). The report schema therefore stays
the same. Each native stage additionally has `generatorImplementation:
"native"`, `generator.kind: "native"`, `generator.cpuMicrosPerRequest` and a
`native` block. That block holds the microsecond histograms (latency from the
scheduled slot, scheduling lag, validation), the largest scheduling lag per
second, the rate-guard decision and processor facts. The report's top level
also carries `generatorImplementation: "native"`. `generatorLimited` uses the
same rule as Node: scheduling lag p99 above 1 ms, on Node's 1 ms buckets.

This is a fixture tool. It is NOT a Mailwoman component.

## Where it lives and why

- `scripts/perf/native-generator/Cargo.toml` declares its own `[workspace]` and
  has its own `Cargo.lock`. It is not a member of the server workspace. It adds
  nothing to the server `Cargo.lock`, and `cargo build/test --workspace` does not
  see it.
- The release image (`Dockerfile`) does not copy `scripts/`. The fixture image
  (`scripts/perf/Dockerfile`) copies `scripts/perf` as source text but never
  builds this crate. `.dockerignore` excludes `**/target`.
- Dependencies, all in its own lockfile: tokio, h2, http, httparse, bytes,
  serde, serde_json, flate2, plus libc (Unix) and windows-sys (Windows).

## Build

```sh
# Windows or Linux host
cargo build --release --manifest-path scripts/perf/native-generator/Cargo.toml
# Linux binary for the Docker VM, without a host toolchain (bookworm, like the fixture image)
docker run --rm -v "$PWD/scripts/perf/native-generator:/src" -v "<build dir>:/target" \
  -e CARGO_TARGET_DIR=/target -w /src rust:1.98.1-bookworm cargo build --release
```

The adapter looks for `scripts/perf/native-generator/target/release/mw-loadgen`.
Set `MW_NATIVE_GENERATOR_BIN` to use a binary somewhere else, for example one
mounted into a generator container.

## Running a goal-65-35 stage

The campaign variables stay the same. Add `MW_LOAD_GENERATOR=native`:

```sh
MW_LOAD_GENERATOR=native MW_NATIVE_GENERATOR_THREADS=12 \
MW_RUN_NATIVE_HTTP_LOAD_FIXTURE=1 MW_NATIVE_LOAD_WORKLOAD=goal-65-35 MW_NATIVE_LOAD_PRINCIPALS=10000 \
MW_NATIVE_LOAD_SECURITY_PROFILE=enforced MW_NATIVE_LOAD_POLICY=loaded \
MW_LOAD_FIXTURE_MODE=capacity MW_LOAD_FIXTURE_LONG=1 MW_LOAD_FIXTURE_ENDPOINTS=goal-65-35 \
MW_LOAD_FIXTURE_LATE=dispatch MW_LOAD_FIXTURE_RATES=25000,50000,100000 MW_LOAD_FIXTURE_CONCURRENCY=1,256,4096,20000 \
<fixture executable> native_multi_principal_http_load_fixture --exact --ignored --nocapture --test-threads=1
```

In external-driver mode, `http-external-driver.mjs` passes its environment to
`http-real-fixture.mjs`, so `MW_LOAD_GENERATOR=native` works there too, with
one worker. The multi-process controller (`--workers>1`) is not wired to the
native generator: one native process with N threads replaces it.

| Variable | Default | Meaning |
| --- | --- | --- |
| `MW_LOAD_GENERATOR` | `node` | `native` runs the measured stages in mw-loadgen |
| `MW_NATIVE_GENERATOR_BIN` | `target/release/mw-loadgen` beside the crate | binary path |
| `MW_NATIVE_GENERATOR_THREADS` | min(available CPUs, 8) | runtime worker threads (1-64) |
| `MW_NATIVE_GENERATOR_RUNTIME` | `multi` | `multi` (one tokio multi-thread runtime) or `per-core` (one current-thread runtime per thread, principals affine) |
| `MW_NATIVE_GENERATOR_CLIENTS_PER_USER` | 1 | forwarded client addresses per user (plan N11/D1), rotated per request |
| `MW_NATIVE_GENERATOR_RATE_GUARD` | on | `off` disables the refusal below; the decision is in `native.rateGuard` |
| `MW_NATIVE_GENERATOR_TICK_US` | 400 | scheduler wake-up period (50-1000 µs) |

Behaviour that matches `measure()`:

- The templates are rebuilt from the same plan inputs. Group layout, shape
  names, serialise and stateful flags come from `prepareFixtureRequests`.
  Header properties and the native read contract come from the Node modules.
  Wire bytes are identical to what fetch sends, including fetch's default
  headers and header order.
- Rotation follows `fixturePrincipalIndex`. Each principal has one request in
  flight (FIFO). The timeout runs from send start, and the queue wait counts.
- Open loop drops late slots or dispatches them (`late=dispatch`). Latency is
  measured from the scheduled slot, and the outstanding cap applies. Closed
  loop runs `concurrency` senders.
- A stateful request that was dispatched but not accepted taints the stage.
  Queued requests then end as `campaign-aborted`, and unscheduled slots are
  counted.
- The draft pool is FIFO across principals. A discard slot destroys the oldest
  saved draft, and creates one when none exists. The adapter carries the pool
  from one stage to the next, as the Node process does.
- The bounds are the same: 8 MiB per response, 64 KiB for validated JSON,
  1 MiB for validated bytes, the 503 reason classes, and gzip/deflate decoding.
- The caps are the same: 50,000 outstanding requests, 3,600 s per stage,
  250,000 offered requests/s, 100 shapes and 10,000 users. The generator also
  accepts up to 64 threads.
- Targets follow the same rule: literal `127.0.0.1`, or a private address that
  is on the explicit allow-list.

Added in the native generator only:

- Several client addresses per user. Addresses can come from
  `principal.clientAddresses`. Otherwise they are derived in 198.19.0.0/16,
  which is disjoint from the fixture's 198.18.0.1 + index. Address 0 is the
  fixture's own address.
- A rate guard (plan N11). An open-loop stage is refused when the planned rate
  per client address exceeds 80% of the configured client limit, or the planned
  rate per account exceeds 80% of the account limit. The planned rate per
  client address is rate / users / addresses per user; per account it is
  rate / users. The limits come from the handoff's enforced policy, or from the
  defaults (600 per client and 1,800 per account per 60 s).
- Up to min(concurrency, 512) HTTP/1.1 connections are opened before the
  window. A Node stage reuses fetch's pool from earlier stages; a native stage
  is a fresh process. Without this warm-up, every native stage paid a first-second
  connect burst (measured: 181 ms lag at 40,000 rps).

## Equivalence (measured 2026-11-24, Windows host)

1. Request bytes: `equivalence/templates-compare.mjs` captures fetch's raw
   HTTP/1.1 requests on a loopback socket and compares them with
   `mw-loadgen templates`. It covers every shape of all 17 groups, plus the
   discard of a known draft. Result: 343 requests (7 synthetic users) and 245
   requests (5 users of a live fixture handoff), 0 differences. The serialise
   and stateful flags were equal too.
2. Validators: `equivalence/validators-compare.mjs` runs the same corpus
   through the Node validators and through `mw-loadgen validate`. The corpus
   holds:
   - well-formed responses for every shape;
   - every JSON node deleted, set to null, `{}`, `[]`, `""`, `"zz"`, 0, 1, -1,
     1.5, true or false;
   - array elements dropped, duplicated or added;
   - JSON.parse edge cases: duplicate keys, escapes, lone surrogates, BOM,
     grammar errors, control characters, deep nesting;
   - invalid UTF-8, truncated and empty bodies, and non-200 statuses;
   - 200 recorded live responses (125 from the first fixture run, 75 from the
     second) and 229,744 corruptions of them.

   Result: 48,410, 103,085 and 172,383 entries in three runs, 0 differences.
3. Live campaign: one debug fixture with 100 users, `goal-65-35` seeding, the
   enforced profile and the loaded policy. Node and then the native generator
   ran the same smoke campaign on it: identity, mixed, reader, attachment,
   flag-write, move-write, jmap-session, list-batch, changes, account-polls and
   goal-65-35, at 4 and 10 rps for 5 s each, with a 30 s timeout. Both
   campaigns completed untainted. 65 of 66 (stage, shape) rows had identical
   requests, accepted, errors and status counts. In the remaining row
   (flag-write at 10 rps), all requests were accepted by both generators, but
   Node dispatched 33 and native 40, because the outstanding cap is reached at
   different moments. Debug writes took about 4.5 s, so this row depends on
   timing.

To reproduce: start the fixture with `MW_LOAD_FIXTURE_EXTERNAL_DRIVER=1
MW_LOAD_FIXTURE_EXTERNAL_ROOT=<dir>`, then run
`node scripts/perf/native-generator/equivalence/live-session.mjs --root=<dir>
--out=<private dir>`. It records the corpus, runs both campaigns and writes
`comparison.json`. All outputs contain fixture cookies: keep them private.

## Calibration (measured 2026-11-24)

Setup: `calibrate.mjs` runs both generators against the same `mw-loadgen serve`
calibration server (2 threads) in the same session. For (b), the server is a
replay server. It returns the responses recorded from the live fixture, renamed
for 200 cloned users, to exactly the requests the goal mix sends. Every
response therefore passes full validation on real response sizes: list-batch
31 KB, jmap-session 3.5 KB, reader 6 KB. Closed loop, 64 outstanding, 10 s per
phase, generator CPU from the generator process only. Linux numbers come from a
`node:24.20.0-bookworm` container on cpuset 22-25 of the Docker VM, with two
repeats each. A benchmark campaign was running on the host. Host CPU was 13%
when the Windows series started.

| Linux container, 1 generator thread | Node µs CPU/request | native µs CPU/request | Node requests per CPU-second | native requests per CPU-second |
| --- | ---: | ---: | ---: | ---: |
| (a) trivial GET, HTTP/1.1 | 258 / 262 | 11.0 / 11.9 | 3,875 / 3,810 | 91,033 / 84,270 |
| (a) trivial GET, h2c (4 connections) | 171 / 172 | 17.0 / 16.3 | 5,841 / 5,811 | 58,990 / 61,407 |
| (b) goal-65-35, full validation, HTTP/1.1 | 749 / 726 | 59.8 / 64.1 | 1,335 / 1,377 | 16,719 / 15,609 |
| (b) goal-65-35, full validation, h2c | 414 / 420 | 76.9 / 73.5 | 2,414 / 2,381 | 13,002 / 13,612 |

| Windows host, same scripts | Node | native |
| --- | ---: | ---: |
| (a) trivial GET, HTTP/1.1 | 362 | 44.9 |
| (a) trivial GET, h2c | 158 | 42.2 |
| (b) goal-65-35, HTTP/1.1 | 811 | 83.2 |
| (b) goal-65-35, h2c | 399 | 90.9 |

- Validation (parse plus checks) is 31-34 µs of the native goal-mix figure.
  Most of it is the 31 KB list-batch response, which takes about 100 µs to parse.
- Open loop costs more per request than a saturated closed loop. Measured on
  Linux, goal mix, 2 threads, 2,000 users × 2 client addresses:
  - 10,000 rps: 123-132 µs CPU per request, scheduling lag p99 ≤ 1 ms after
    the warm-up was added;
  - 20,000 rps: 79-89 µs, not generator-limited;
  - 30,000 rps: over 2 threads, generator-limited, lag up to 1.9 s.

  Cost falls as utilisation rises, because a mostly idle worker pays a wake-up
  per request.
- `multi` against `per-core`: 1 thread, goal mix HTTP/1.1, run in the order
  multi, per-core, per-core, multi. `multi` measured 57.6 and 60.5 µs;
  `per-core` measured 57.1 and 60.9 µs. No measurable difference, so `multi` is
  the default.
- Estimate for 100,000 rps of the goal mix: 100,000 × 60-85 µs = 6-8.5
  logical CPUs of generator, against the 16 the plan budgets. This is
  arithmetic from the figures above, not a measured 100,000 rps stage.

Reproduce (paths private):
`node scripts/perf/native-generator/calibrate.mjs --mode=trivial|goal
[--handoff=… --recorded=<live corpus>] [--protocol=h2c] [--seconds=10]
[--concurrency=64] [--rate=N --threads=N --clients-per-user=K]`.
Phases are limited to 120 s.

## Windows processor groups

On this host (2 × 20 cores, 80 logical CPUs), `std::thread::available_parallelism`
and Node's `os.availableParallelism()` both report 40. That is the primary
processor group. `mw-loadgen sysinfo` reports 2 active groups, 80 logical CPUs,
and a process group affinity of groups 0 and 1: on Windows 11, a process's
threads may run in both groups. The default thread count is therefore
min(40, 8). `MW_NATIVE_GENERATOR_THREADS` up to 64 is accepted. I did not
measure whether threads beyond 40 are actually scheduled in group 1. Inside the
Linux Docker VM, the processor groups do not apply. The VM's vCPU count, or the
container's cpuset, is what the generator sees.

On native Windows, the scheduler's sub-millisecond sleeps are coarser. In the
4 and 10 rps live stages, native lag p99 was 1-2 ms (Node's was 14-16 ms), so
`generatorLimited` may read true at very low rates on Windows. In the Linux
container at 10,000-40,000 rps, lag p99 was ≤ 1 ms.

## Not equivalent yet

- HTTP/2 over TLS (`mode: 'tls'`) is not implemented. Only h2c is.
- The multi-host or multi-process controller does not drive native workers.
  `includeRawMeasurements` is passed through, so this can be added.
- Native scheduling lag is measured when the request task starts, so it
  includes runtime queueing. Node measures at dispatch in its scheduler loop.
  The native measure is stricter.
- After a body failure (a bound exceeded, a reset), the decoded-byte count can
  differ from Node's, which depends on chunk boundaries. The verdict and status
  are the same.
- Two JavaScript semantics are approximated:
  - string indexing is per Unicode scalar, not per UTF-16 unit;
  - a lone surrogate escape becomes U+FFFD.

  No validator compares such values with anything they could equal. JSON
  nesting deeper than 4,000 levels is rejected, whereas Node accepts more.
- Not verified against a live fixture:
  - several client addresses per user, tested only against the calibration
    server;
  - the rate guard, tested only by refusing 2,000 rps for 200 users.

  Whether the server accepts one session from several forwarded addresses is
  part of N11.
- The large-mailbox driver, the goal-mix smoke driver, WebSocket load and the
  calibration protocol of `http-calibration-*.mjs` are not ported.
