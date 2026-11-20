# Coordinated HTTP generator

The optional multi-process driver partitions an explicitly owned native fixture
between 1–16 Node workers. It shares the existing request factory, validators,
HTTP scheduler and raw histogram format with the default single-process driver.
It adds no Mailwoman authentication or policy bypass. Proxy fixtures use the
existing single-process driver.

The external sidecar selects multiple HTTP processes with either
`MW_LOAD_GENERATOR_WORKERS=4` in **the sidecar environment**, or `--workers=4`:

```sh
node scripts/perf/http-external-driver.mjs --root=/scratch --workers=4
```

This option does not change the server's Tokio worker count, Cargo jobs, or CPU
quota. All child processes share the sidecar's existing cgroup and isolated
network namespace. No Rust fixture rebuild is needed to select this script mode;
the container must include the new controller, worker, protocol and launch files.
The default remains one process and the original driver.

For an already available private native-fixture handoff:

```sh
MW_LOAD_FIXTURE_SECONDS=30 \
MW_LOAD_FIXTURE_CONCURRENCY=16,64,256 \
MW_LOAD_FIXTURE_MODE=capacity MW_LOAD_FIXTURE_LONG=1 \
node scripts/perf/http-multi-controller.mjs \
  --handoff=/scratch/handoff.private.json \
  --output=/scratch/coordinated.report.json --workers=4
```

Phase and warmup durations must use integer seconds (measured phases: 1–60).
Every measured concurrency/rate budget must provide at least one slot per worker.
The global outstanding limit is 4,096, divided between workers, and offered rates
are divided exactly rather than multiplied. The normal one-slot warmup is raised
to one slot per worker; reports record that actual warmup concurrency. Campaign
duration/deadline, overload and adaptive-stop controls remain the existing ones.

Each worker receives only its disjoint principal subset in an exclusive private
file, preflights its first assigned identity, and acknowledges phase readiness.
The controller then announces one future start and deadline to all workers.
Completion inside that common window determines successful RPS. A worker crash,
missing response, inconsistent partition, invalid histogram, or start lag over
50 ms invalidates the coordinated result. Temporary credential files are removed
only after child exit is confirmed; unknown shutdown retains them privately.
Worker command arguments and reports contain no credentials or account IDs. An unknown mutation outcome stops the entire campaign, preserves partial offered/aborted counters, and invalidates earlier clean stages as capacity evidence.

Global startup/index, seeded workload and loaded-policy evidence remain global;
they are not rescaled into independent proofs for each partition. Distinct users
and simulated clients reported as accepted are measured local coverage summed
across disjoint partitions. Configuring 10,000 users does not prove all were
reached. Simulated client addresses do not represent separate generator hosts.

Latency histograms are merged by bucket counts. Percentiles are never averaged,
and RPS uses the common interval rather than a sum of independent phase rates.
Generator CPU is summed across workers, while controller CPU is reported
separately. RSS is a sum of worker before/after observations, not synchronized
peak memory. Summed per-worker outstanding peaks are explicitly an upper bound,
not a simultaneous global observation.

The WebSocket sidecar can select the same HTTP driver with `--http-workers=4`
or the sidecar environment variable when `--with-http=true` is enabled. Its
existing verified WebSocket steady-window requirement still covers the entire
HTTP campaign, including worker preparation and warmup. Configure a long enough
hold; HTTP-only success cannot prove persistent WebSocket capacity.

The native controller also preserves the original anonymous `/.env` probe after measured phases and requires a real 403/429 response. It is not counted as successful workload traffic.

The 40k-RPS/10k-user HTTP target additionally requires an actual release native
fixture, verified startup and loaded-policy evidence, a common window of at least
30 seconds, successful coverage of 10,000 distinct users and simulated clients,
and no errors, throttles, timeouts or dropped arrivals. The overall capacity
target remains false without the separate simultaneous WebSocket verification.
Generator calibration and actual application measurements remain separate;
neither process count nor synthetic IPC tests are throughput evidence.
