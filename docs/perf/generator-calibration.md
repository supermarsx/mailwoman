# Generator calibration

This synthetic constant-response server measures the Node generator and HTTP
transport. It does **not** measure Mailwoman capacity, authentication, policies,
email operations, storage, upstream protocols or security correctness. It uses no
Mailwoman accounts or data. Run it separately from application campaigns and
after every compiler, image build and other load generator has stopped.

The server provides a JSON response with 4,096 padding characters and a 65,536-byte
binary response. The client uses the same `measure()` scheduler, fetch pipeline,
bounded response buffering and content validators as application campaigns.
Each endpoint receives up to five offered-rate phases, with 100–80,000 offered
requests/second, 0.1–30 seconds per phase, and at most 1,024 outstanding requests.
Defaults are 10k/20k/40k/60k/80k, three seconds per phase, and 256 outstanding.
Longer phases should be used for evidence beyond an initial smoke check.

Reports contain successful requests completed inside the scheduled window,
offered arrivals, capacity/scheduler drops, errors, bytes, latency, generator CPU
and before/after RSS, and event-loop delay. The separate server reports CPU,
current RSS, event-loop delay, and served request/byte counts. RSS values are
observations rather than peak-memory guarantees. Server request counts include
requests it served even if the client later rejected or timed out their response.
CPU and latency intervals include drain time; compare `acceptedDuringScheduledWindowRps`
against the fixed scheduled duration when assessing offered-rate delivery.

No automatic application-capacity qualification follows from these results.
An overloaded server, generator CPU quota, scheduler drops, event-loop delay or
network transport can each limit the result. A client that cannot sustain the
desired offered rate leaves application throughput inconclusive. A calibration
that exceeds 40k successful requests/second only establishes headroom for this
synthetic response profile; it does not establish application capacity or equal
headroom for a more expensive representative workload.

## Disposable Docker recipe

Use a verified existing image containing Node; the example uses the existing
local fixture image. Inspect its image ID first and record it with the reports.
The two containers have separate CPU/memory cgroups but share the server's
isolated network namespace. They publish **no host ports** and use no external
network. The server binds only literal `127.0.0.1` on an ephemeral port. Substitute
the generator quota used for the real application run when comparing results.

Run these commands in Bash from the repository root, using a fresh unique name:

```sh
task_name="mw-calibration-$(date +%s)-$$"
task_scratch="$(pwd)/.orchestration/private/$task_name"
mkdir -p "$task_scratch"
chmod 700 "$task_scratch"
docker image inspect mailwoman:perf-native-release --format '{{.Id}}'
docker run -d --name "$task_name-server" --label mw.calibration.owner="$task_name" \
  --network none --cpus 2 --memory 512m --memory-swap 512m \
  --mount "type=bind,src=$(pwd)/scripts/perf,dst=/perf,readonly" \
  --mount "type=bind,src=$task_scratch,dst=/scratch" \
  --entrypoint node mailwoman:perf-native-release \
  /perf/http-calibration-server.mjs --handoff=/scratch/calibration.private.json --seconds=900
# Wait until the private handoff exists; do not print its contents.
for task_attempt in $(seq 1 100); do
  test -s "$task_scratch/calibration.private.json" && break
  sleep 0.1
done
test -s "$task_scratch/calibration.private.json"
docker run --name "$task_name-client" --label mw.calibration.owner="$task_name" \
  --network "container:$task_name-server" --cpus 2 --memory 1g --memory-swap 1g \
  --mount "type=bind,src=$(pwd)/scripts/perf,dst=/perf,readonly" \
  --mount "type=bind,src=$task_scratch,dst=/scratch" \
  --entrypoint node mailwoman:perf-native-release \
  /perf/http-calibration-client.mjs --handoff=/scratch/calibration.private.json \
  --output=/scratch/calibration.report.json --seconds=10 --concurrency=256 \
  --rates=10000,20000,40000,60000,80000
```

The private handoff contains only an ephemeral calibration-control token and
loopback address; the credential-free report contains neither. Keep the scratch
directory private. Both processes have bounded lifetimes, and server authority
expires after at most 15 minutes. For cleanup, stop and remove only the two exact
containers whose owner labels match this run. Retain the report and image ID;
remove the private handoff only after both containers have stopped. Do not use
broad container or volume pruning.

Multi-process generators are a separate follow-up. Independent single-process
percentiles cannot be averaged into a combined percentile, and independent
campaign RPS values cannot establish a common-window rate.

## Initial measured limit (2026-11-20)

A disposable run used separate two-CPU cgroups for the server and generator,
512 MiB and 1 GiB memory limits respectively, no swap, and no published ports.
Three-second phases used 256 outstanding requests. At 40,000 offered RPS,
the generator completed about 5,725 JSON requests/second and 3,298 binary
requests/second inside their scheduled windows. It dropped 102,591 and 109,852
offered arrivals respectively. Response validation reported no errors.

The generator consumed about 1.50 and 1.62 logical CPUs; the constant-response
server consumed about 0.41 and 0.40. These short observations demonstrate that
this single-process setup did not deliver the requested load. They establish
neither a final transport ceiling nor Mailwoman throughput. Coordinated workers
must be calibrated before using them to assess the application target.

## Scale-out of the coordinated generator (t53s)

Everything in this section applies to the coordinated multi-process generator
(`http-multi-controller.mjs`). Defaults are unchanged: without the options below
the behaviour, caps that were already in use and report shapes match the earlier
campaigns. None of this has been run at rate; the loopback checks mentioned below
are logic checks, not capacity evidence.

### Limits

| Limit | Before | Now |
| --- | --- | --- |
| Workers per host | 16 | 64 (256 across all hosts) |
| Outstanding requests, one stage | 4,096 | 20,000 (40,000 rps x 250 ms = 10,000) |
| Stage duration | 60 s (600 s single process) | 3,600 s (`MW_LOAD_FIXTURE_DEADLINE_SECONDS` up to 14,400) |
| Users, offered rate | 10,000, 100,000 | unchanged |

Values past a limit are rejected before any worker starts. The calibration client
(`http-calibration-*.mjs`) keeps its own, smaller bounds.

### Late-arrival policy

`MW_LOAD_FIXTURE_LATE=drop` (default, as before) discards an arrival slot that is
more than 100 ms late. `MW_LOAD_FIXTURE_LATE=dispatch` never discards a late slot:
it is dispatched late, latency is measured from the scheduled slot, and a slot due
at the outstanding cap waits instead of being dropped. The queue that the drop
policy hides appears as scheduling lag. Slots still unissued when the window
closes are counted in `unissuedAtWindowEnd` (and in `droppedScheduler`);
`droppedLateSlots` counts only slots discarded for lateness (always 0 for
`dispatch`). Stage fields: `latePolicy`, `latencyBasis` (`send` or
`scheduled-slot`), `lateSlotsDispatched`.

Scheduling lag (slot to actual send) is reported per worker as p50/p99/max in
`perWorker[]`. A worker or stage whose lag p99 is above 1 ms is marked
`generatorLimited: true`; such a stage does not satisfy the 40,000 rps target
rule and is named in the Markdown report. Percentiles use 1 ms histogram buckets,
so p99 above 1 means a lag above 1 ms.

### Per-second accepted series

Each stage records `acceptedPerSecond[]` (accepted completions inside the
scheduled window per second, summed over workers, one entry per scheduled second)
with `acceptedPerSecondMin/Max`. Simulated client addresses are unchanged: one
distinct address per user, up to 10,000.

### Targets

The generator still refuses arbitrary hosts. By default the only target is the
literal `127.0.0.1` fixture. A handoff may instead list `targets` (origins) which
workers rotate across, one per request in order. Each non-loopback target must be:

1. a literal private address with an explicit non-default port (RFC 1918 IPv4 or
   IPv6 unique-local `fc00::/7`); public, link-local (including metadata
   addresses), CGNAT, multicast, hostnames and IPv4-mapped IPv6 are rejected;
2. named in the operator's explicit allow-list (`--allow-targets=10.0.0.5:8080,...`
   on the controller; `--allow-targets` on `http-external-driver.mjs`); and
3. proven to belong to the run: the run's control path and token must be accepted
   by that target and a request without the token must be refused.

The controller and every worker check these rules, so a handoff alone cannot
widen the target set. Diagnostics snapshots are still read from the first target
only; per-instance snapshots belong to the rig topology task.

### Several generator hosts

On each additional generator host run
`node http-multi-host.mjs --listen=<private-address>:<port> --token-file=<private file>`
(a loopback or private bind is required). The controller takes
`--hosts-file=<private JSON>` with `[{"origin":"http://10.0.1.7:7700","token":"...","workers":32}]`
and `--workers=` for its own workers (0 allowed when hosts are given). Principals
are partitioned across all workers; the agent receives its workers' private
handoffs over HTTP, so use this on the rig's private network only.

The controller measures each host's clock offset (eight round trips, lowest
round trip kept, offset in whole milliseconds, uncertainty = half that round trip)
before every stage, sends the shared start time on each host's clock, and shifts
returned windows back before validation. Offsets, round trips and the tolerance
used (`clockToleranceMs`) are written to `coordination.hosts`, and
`generatorHosts` lists the hosts. Raw histograms from all workers are merged
bucket by bucket; per-host or per-worker percentiles are never averaged.
`distinctPhysicalGeneratorHostsMeasured` is true only when hosts were used.

### Self-test (2026-11-24, this development machine, not a rig)

One process and an in-process constant-response server, 512 outstanding, 5,000
offered requests/s for three seconds: the `drop` policy dropped 8,266 of 15,000
slots (581 late, the rest at the cap) and reported p99 scheduling lag 100 ms; the
`dispatch` policy dropped none for lateness or capacity, left 4,552 slots unissued
at the window end, and reported scheduling lag p50 578 ms. Both were marked
generator-limited. This only demonstrates that the policies and fields behave as
described; it says nothing about the generator's ceiling on a rig.
