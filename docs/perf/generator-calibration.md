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
