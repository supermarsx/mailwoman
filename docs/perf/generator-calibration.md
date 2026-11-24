# Generator calibration

This synthetic constant-response server measures the Node generator and HTTP
transport. It does **not** measure Mailwoman capacity, authentication, policies,
email operations, storage, upstream protocols or security correctness. It uses no
Mailwoman accounts or data. Run it separately from application campaigns and
after every compiler, image build and other load generator has stopped.

The server provides a JSON response with 4,096 padding characters and a 65,536-byte
binary response. The client uses the same `measure()` scheduler, fetch pipeline,
bounded response buffering and content validators as application campaigns.
Each endpoint receives up to five offered-rate phases, with 100–250,000 offered
requests/second, 0.1–3,600 seconds per phase, and at most 50,000 outstanding requests.
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
| Outstanding requests, one stage | 4,096 | 50,000 (100,000 rps x 250 ms = 25,000; was 20,000 for the 40,000 rps goal) |
| Stage duration | 60 s (600 s single process) | 3,600 s (`MW_LOAD_FIXTURE_DEADLINE_SECONDS` up to 14,400) |
| Offered rate, one stage | 100,000 | 250,000 (the 100,000 goal plus overload stages above it) |
| Users | 10,000 | unchanged |

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
`generatorLimited: true`; such a stage does not satisfy the standing rps target
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
slots (581 scheduler drops, 7,685 at the cap) and reported p99 scheduling lag 100 ms; the
`dispatch` policy dropped none for lateness or capacity, left 4,552 slots unissued
at the window end, and reported scheduling lag p50 578 ms. Both were marked
generator-limited. This only demonstrates that the policies and fields behave as
described; it says nothing about the generator's ceiling on a rig.

## HTTP/2 mode (t53s)

The coordinated generator can speak HTTP/2 instead of HTTP/1.1. The default is
unchanged (HTTP/1.1). The request templates, response validators, byte limits and
503 classification are the same code for both protocols: HTTP/2 is only a
different transport behind `measure()`. A calibration or application result is
valid for the protocol it was run with; repeat the calibration per protocol and
do not carry numbers from one to the other. No HTTP/2 figures are claimed here.

Options (campaign environment, or the controller flags in parentheses):

| Option | Default | Meaning |
| --- | --- | --- |
| `MW_LOAD_FIXTURE_HTTP2` (`--http2=`) | unset = HTTP/1.1 | `h2c`: cleartext prior-knowledge HTTP/2 to the plain listener; `tls`: HTTP/2 over TLS negotiated by ALPN (targets are `https://` literal private addresses) |
| `MW_LOAD_FIXTURE_H2_PRESET` (`--h2-preset=`) | `proxy-like` | see below; `custom` also reads `MW_LOAD_FIXTURE_H2_PER_CLIENT=1` |
| `MW_LOAD_FIXTURE_H2_CONNECTIONS` (`--h2-connections=`) | preset (4 / 1) | connections per pool, 1-64 |
| `MW_LOAD_FIXTURE_H2_MAX_STREAMS` (`--h2-max-streams=`) | 100 | streams per connection used by the generator, 1-1000; the effective limit is the smaller of this and the server's announced `SETTINGS_MAX_CONCURRENT_STREAMS` (32 for the Mailwoman server) |
| `MW_LOAD_FIXTURE_H2_CA_FILE` (`--h2-ca-file=`) | none | rig CA (PEM) the server certificate is verified against; required for `tls` unless the next flag is used |
| `MW_LOAD_FIXTURE_H2_SERVERNAME` (`--h2-servername=`) | target address | TLS server name for verification |
| `MW_LOAD_RIG_ONLY_INSECURE_TLS_NO_VERIFY=1` | off | disables certificate verification; only for a throwaway rig, never together with a CA file |

Presets:

- `browser-like`: one pool per simulated client (target + client index), one
  connection per pool by default, up to the stream limit on each. A run with
  10,000 simulated clients therefore opens up to 10,000 connections (bounded at
  20,000); per-client connections are opened on first use, inside the measured
  window.
- `proxy-like`: four connections per target shared by all simulated clients, as
  an nginx/Caddy/Traefik upstream behaves, opened before the window. The
  simulated client address still travels in each request's forwarded header, so
  the 10,000 distinct addresses are unchanged.

When every connection is at its stream limit the request waits in the generator
(FIFO); the wait is added to the request's scheduling lag, so the 1 ms
generator-limited rule applies and nothing is dropped silently. A GOAWAY or
connection error retires the connection and opens a replacement (counted); a
refused stream is an error and is counted as such, not retried. The target
allow-list, private-address and ownership rules apply to HTTP/2 unchanged (for
`tls` the target scheme must be `https`); the ownership proof itself runs over the
chosen protocol.

Each stage reports `protocol` (`http/1.1`, `h2c`, `h2`) and, for HTTP/2, an
`http2` object: connections opened/closed, reconnects, GOAWAY count, RST_STREAM
counts by code, REFUSED_STREAM count, streams per connection (connections, min,
max, total), peak and queued streams, queue wait, the server's announced stream
limit. Flow-control stalls are not exposed by `node:http2` and are not reported.
Latency basis is unchanged (send, or the scheduled slot with `late=dispatch`).

`h2Burst()` in `http-h2-client.mjs` issues N distinct GETs at once on one
connection (the cold asset-burst pattern) and reports per-path status, the
reasons of any 503 and the connection counters. By default it stays within the
server's stream limit like a browser; `exceedServerLimit: true` opens all streams
at once to observe refusals.

Verification state: unit tests use an in-process `node:http2` server (stream
limit queueing, GOAWAY reconnect, refused streams, both presets, burst, protocol
parity, TLS with a throwaway CA). A short run against the Mailwoman debug build's
plain listener confirmed prior-knowledge h2c, the 32-stream limit and a 64-GET
burst on one connection; TLS against a real TLS-terminating proxy has not been
run.

## Judged target and byte accounting (t53s)

The standing goal is 100,000 successful requests/s on a balanced mix of about 65%
reads and 35% writes, with 10,000 simultaneous authenticated users, on this
machine as the base hardware. The user revised it on 2026-11-24, twice (it was
40,000 requests/s, then 30,000). Both numbers are parameters:

| Option | Default | Meaning |
| --- | --- | --- |
| `MW_LOAD_TARGET_RPS` | 100000 | requests/s a stage must offer and have accepted inside its window |
| `MW_LOAD_TARGET_USERS` | 10000 | distinct users and simulated clients a stage must cover (at most 10,000) |
| `MW_LOAD_NETWORK_GBIT` | unset | declared network budget; when set, every stage's body bytes (response plus request, in Gbit/s) must stay within it |
| `MW_LOAD_QUALIFICATION_RUN=1` | off | requires `MW_LOAD_NETWORK_GBIT`, and a run whose targets include loopback can never qualify |

Every report starts with `judgedAgainst` (target, users, and the network
claim), and a report judged against anything but 100,000 / 10,000 says
`NON-DEFAULT TARGET` there and in the first Markdown line. A loopback or
shared-network-namespace run is labelled as unable to qualify a network claim,
whatever its numbers.

Each stage carries `transfer`: accepted response bytes per second inside the
scheduled window (decoded body bytes the generator consumed), request body bytes
per second, the sum in Gbit/s, response size mean and a p99 upper bound (5% size
buckets), and compression: how many responses were encoded, which encodings, and
the compressed/uncompressed ratio over the responses whose encoded size is known.
HTTP/2 always knows it (the client advertises gzip/deflate like fetch and decodes
before validation); HTTP/1.1 through fetch only knows it from `Content-Length`,
otherwise the ratio is reported as not measured. Headers, TLS records and framing
are not counted, so the budget check understates the wire. This is the same
decoded-body counting as `http-response-bytes.md`, plus the distribution and
compression fields; `bytes` (all attempts) is unchanged.

### Local base hardware (`MW_LOAD_QUALIFICATION_HOST=local`)

The user designated this machine as the base test hardware, so a qualifying run
may use loopback or a shared network namespace. Setting
`MW_LOAD_QUALIFICATION_HOST=local` (the only defined value) stops the loopback
label from failing qualification of the throughput claim, and makes
`MW_LOAD_NETWORK_GBIT` optional for a qualification run. The label stays: the
report still says a loopback run cannot qualify a network claim, and
`judgedAgainst.network.qualifies` is false. `judgedAgainst` records host facts:
logical cores and memory from the operating system, and the Docker VM vCPUs and
the cpusets of the app, database and generator as declared in
`MW_LOAD_HOST_DOCKER_VCPUS`, `MW_LOAD_HOST_CPUSET_APP`, `MW_LOAD_HOST_CPUSET_DB`
and `MW_LOAD_HOST_CPUSET_GENERATOR` (`null` when not declared; Node cannot see
them). Without the option, a loopback qualification run still cannot qualify.

## The goal mix `goal-65-35` (t53s)

Selected as the campaign endpoint group `goal-65-35` (`MW_LOAD_FIXTURE_ENDPOINTS=goal-65-35`)
on a fixture seeded with `MW_NATIVE_LOAD_WORKLOAD=goal-65-35`, which adds a trash
and a drafts folder per user, planted search terms (`quarterly`, `zephyrine`) and
five or more messages, with `messageIds[2]` starting read and `messageIds[3]`
starting flagged. A fixture without that seeding simply has no such group. The
existing `representative` mix is unchanged.

One entry in the mix is one HTTP request. The mix is 100 slots in a fixed,
deterministic order (smooth weighted round robin), so any 100 consecutive requests
hold exactly 65 reads and 35 writes by request count.

| Shape | Kind | Weight | Mutations per request | Template |
| --- | --- | ---: | ---: | --- |
| `authenticated-identity` | read | 6 | 0 | `GET /api/me` |
| `jmap-session` | read | 5 | 0 | `GET /jmap/session` |
| `list-batch` | read | 15 | 0 | the web client's list batch: query + header get + unread count |
| `Email/get` | read | 9 | 0 | headers of two messages, default UI properties |
| `body-and-attachment-metadata` | read | 10 | 0 | reader body and attachment metadata |
| `changes` | read | 9 | 0 | `Email/query` + `Email/queryChanges` + `Email/changes` poll |
| `account-preferences` | read | 4 | 0 | `GET /api/account/preferences` |
| `appearance-customization` | read | 3 | 0 | `GET /api/account/appearance/customization` |
| `search-planted-term` | read | 4 | 0 | `Email/query` with `text: quarterly` in the inbox (total must equal the seeded count) + `Email/get` of the page |
| `mark-read-and-restore` | write | 7 | 2 | set `$seen`, read back, clear it, read back |
| `mark-unread-and-restore` | write | 4 | 2 | clear `$seen`, read back, set it, read back |
| `flag-and-restore` | write | 6 | 2 | set `$flagged`, read back, clear it, read back |
| `unflag-and-restore` | write | 3 | 2 | clear `$flagged`, read back, set it, read back |
| `move-and-restore` | write | 8 | 2 | move inbox to archive, read back, move back, read back |
| `delete-to-trash-and-restore` | write | 5 | 2 | move one message to trash, read back, move it back, read back |
| `draft-save` | write | 1 | 1 | one `Email/set` create of a draft in the drafts folder |
| `draft-discard` | write | 1 | 1 | `Email/get` of a saved draft, then `Email/set` destroy of it |

What one write request is: toggle, move and trash writes are one `POST /jmap/api`
with four method calls, `Email/set`, `Email/get`, `Email/set`, `Email/get`: two
message mutations and two verifying reads. A draft is two requests, each with one
mutation: the save creates it, the discard reads it back and destroys it; together
they hold exactly two write slots. Over a full cycle the 35 write slots carry
(7+4+6+3+8+5) x 2 + 2 = 68 mutations, so 0.68 mutations per request overall.
`mutationsPerSecond` counts verified mutations only (accepted write requests in the
window times that row's count).

A discard slot destroys the oldest draft this generator process saved, for the user
that saved it. Before any draft exists (the first cycle of a run) the slot creates
a draft instead and is counted under the discard label. No creation back-reference
is used, because the engine does not resolve it. Each save currently leaves a copy
in the upstream Drafts folder (a known engine defect), which is why the draft
weight is low.

Every write shape verifies its effect by read-back, and an unverified or lost
mutation is an error that taints the stage. Requests for one user are serialised
(as in the representative mix), including reads, because the change poll assumes a
stable state. The shapes use distinct messages (`messageIds[0..4]`).

Each stage of this group reports `mix`:
- attempted and accepted read/write counts and shares, read and write requests per second;
- `mutationsPerSecond`;
- `upstream`: from the fixture diagnostics `upstreamCommands` counters, the delta
  between the stage's before and after snapshots: commands per second, per kind
  (store, move, expunge, append, fetch, list), and write commands (store + move +
  append + expunge) per write request attempted. The counters are process-wide and
  cumulative and the delta includes drain after the window. `upstreamCommandsPerSecond`
  is `null` when the snapshots lack them or a counter reset;
- `requestInputs.consumedFramesPerContext`: consumed frames per ordinary context
  from the same diagnostics, when the request-inputs counters are present;
- `perUser`: accepted requests per second per user and the serialised ceiling per
  user, 1000 / mean latency ms, because one user has one request in flight.

A `goal-65-35` stage qualifies only when its accepted write share is 30-40%;
`capacityTarget.goalMixMet` is judged on that group alone. `t34-modes.mjs` counts
`goal-65-35` as a mutating group. The request-inputs evidence in
`request-inputs-experiment.mjs` does not know this group's shape list; do not
combine `MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT` with it until that is extended.

## Can this host offer 100,000 requests/s? (t53s, 2026-11-24)

Limits of the generator: the validation caps above accept a 100,000 requests/s
stage with 25,000 outstanding requests, 64 workers (about 1,563 requests/s each)
and campaign rates up to 250,000. The HTTP/2 proxy-like preset shares 4
connections per worker; with the server's 32-stream limit that is 128 concurrent
streams per worker, enough while the mean latency stays below about 80 ms at
1,563 requests/s per worker (beyond that requests queue in the generator and show
as scheduling lag, which marks the stage generator-limited). The default capacity
ramp rates (`1000,...,42000`) are unchanged for comparability: set
`MW_LOAD_FIXTURE_RATES` explicitly for a 100,000 requests/s campaign.

What it costs in CPU, from the only measurement that exists: one Node process
completed about 5,725 constant-response JSON requests/s using about 1.5 logical
CPUs (calibration, 2026-11-20), roughly 3,800 requests/s per CPU. Offering
100,000 requests/s is then about 26 logical CPUs at saturation, and about 52 to
keep the generator at or below 50% CPU. That figure is for a constant response with
trivial validation; the real templates (JSON parse, validators, 10,000 simulated
clients) cost more, so treat it as a lower bound. The multi-process path has never
been run at rate, so the per-CPU figure is unconfirmed for it.

This host reports 40 logical cores. At saturation the generator alone would take
about 26 of them, leaving about 14 for the server, the database and everything
else; at the 50% CPU rule it would need more cores than exist. So, on these
numbers, one host cannot both offer 100,000 requests/s with generator headroom
and serve it: a claim at that rate needs either the generator on other hosts
(multi-host mode, private network, which makes the run a network run rather than
a loopback one) or a measured per-CPU generator cost that is better than the
single-process figure. A stage at that rate on this host must report
`generatorLimited` honestly; it is not evidence about the server when it is set.

