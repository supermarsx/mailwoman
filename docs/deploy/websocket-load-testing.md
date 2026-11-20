# Independent authenticated WebSocket load fixture

`mw-server/examples/websocket-load.rs` is a separate-process generator for the
opt-in native fixture. It tests connection/push acceptance independently of the
40,000 HTTP requests/second campaign. A WebSocket result is not HTTP throughput
and does not establish either target without actually completing its campaign.

## Build and private input

```sh
cargo build -p mw-server --release --features load-test-fixtures --example websocket-load
cargo test -p mw-server --features load-test-fixtures --example websocket-load
```

The native fixture (`tests/native_load_fixture.rs`) seeds 1, 100 or 10,000 distinct
verified durable principals and writes its private authentication handoff. Use
`MW_NATIVE_LOAD_PRINCIPALS=10000` and a sufficiently long
`MW_LOAD_FIXTURE_DEADLINE_SECONDS` (for example 600) when preparing a full run.
The input schema includes `base`, `engineMode: "native"`, `backend`,
`buildProfile`, and `principals` with `cookie`, `csrfToken`, `accountId` and an
optional matching `ownerAccountId`. Duplicate account IDs or cookie credentials
are rejected. These credentials must stay in the private handoff, not shell
arguments, stdout, artifacts or source control. Unix handoffs must have no group
or world access; the reader rejects symlinks and inputs over 32 MiB.

For distributed-client simulation, the private handoff also contains
`clientSimulation: {mode: "fixture-loopback-trusted-proxy", clientCount: N}` and
one unique canonical `clientAddress` in `198.18.0.0/15` per principal. Only this
explicit mode permits the generator to send `X-Forwarded-For`, and its origin
must be the fixture's literal HTTP loopback address. The fixture starts its own
worker with proxy trust limited to `127.0.0.1/32`; production policies are not
disabled. Missing, duplicate, malformed or external addresses are rejected.
The report distinguishes configured clients from clients that actually became
ready and remained live at the end. These addresses simulate client identities,
not separate physical hosts or network paths.

The fixture adds a random token-protected `pushTrigger` route after the production
router layers. It accepts only a bounded count and emission interval; account IDs
come from its own ordered seed list. It emits synthetic `StateChange` wakeups
through the actual `PushHandle`, without sending or mutating mail. The private
route is compiled only in the isolated fixture; there is no production API.

## Run

Keep the fixture alive through its external-driver control while executing:

```sh
websocket-load --manifest /shared/control/handoff.private.json \
  --report /shared/reports/websocket-10000.json \
  --protocol push --connections 10000 --ramp-per-second 100 \
  --max-pending 128 --warm-seconds 10 --hold-seconds 60 \
  --connect-timeout-seconds 30 --workers 2 --trigger-interval-ms 10
```

The paths above illustrate the private handoff and credential-free report. Use
the actual paths published by the external-driver controller. A separate Docker
generator can share the fixture's network namespace with `--network
container:<fixture-container>`, while retaining its own CPU/memory cgroup and
mounting the private control directory. Preserve same-user file access. Record
both generator and server resource limits, build profiles, descriptors, database
backend and policy configuration with the result. The generator never disables
certificate validation or follows push-trigger redirects.

### Controller for an entire WebSocket campaign

Use the WebSocket-aware sidecar **instead of** `http-external-driver.mjs` for a
WebSocket-only run. An HTTP-only controller would finish its own campaign and
allow the server fixture to shut down before a long WebSocket test completed.
The WebSocket controller claims the same private ready record and maintains its
heartbeat until its children exit; do not run both controllers against one run.

```sh
node /src/scripts/perf/websocket-external-driver.mjs \
  --root=/scratch/control --binary=/out/websocket-load \
  --connections=100 --ramp-per-second=100 --warm-seconds=5 --hold-seconds=10
```

First complete the 100-client smoke against the actual router. For a full run,
seed 10,000 principals, allow at least 600 seconds in the native fixture, then
use `--connections=10000 --hold-seconds=60`. The binary path must refer to the
matching compiled native example inside the generator container. Keep generator
CPU and memory limits separate from the server's. Failure reports remain under
the private run directory if the fixture refuses to export a failed campaign.

### Simultaneous HTTP and WebSockets

Add `--with-http=true` to the WebSocket controller to run HTTP while the sockets
remain live. The example publishes a private `--ready-file` receipt only after
ramp, warmup and push triggering with every socket authenticated and live. The
controller starts HTTP after reading that receipt, waits for both children and
requires the entire HTTP process to finish inside the WebSocket steady window.
Both results are retained separately in the combined credential-free report.
Independent passes outside this window do not satisfy the combined target.

Choose a hold long enough for HTTP warmup and every selected stage. For example,
one 30-second concurrency stage plus one 30-second offered-rate stage and a
10-second warmup needs more than 70 steady seconds; allow extra time for HTTP
preflights and generator overhead. The controller refuses expired receipts and
marks a non-overlapping HTTP run failed. Its overall lifetime is bounded by the
fixture's advertised deadline. The combined 10,000-user / 40,000-RPS result is
true only when every WebSocket/push check passes, HTTP independently meets its
actual successful-rate/multi-user criteria, and their measured windows overlap.
Time-window comparison uses the shared fixture/generator host clock; this is not
a cross-host clock-synchronization measurement.

## What passes

Push mode opens `/jmap/ws` using each principal's own cookie and the fixture
origin, requests the `jmap` subprotocol, and requires a matching Ping/Pong before
counting the socket ready. After ramp and warmup, it invokes the private trigger
and requires the synthetic Email-state marker under the correct account on every
socket. A `StateChange` naming another account is a failure. Initial metadata and
keepalive frames do not count as verified push recipients.

All requested sockets must remain live through the steady hold, with no errors
or premature closes, and every push recipient must verify. Full 10,000-user runs
require at least 60 steady seconds. Cleanup is bounded to 30 seconds, connections
are capped at 10,000, pending admissions at 512, and the campaign at one hour.
Frames/messages are capped at 64 KiB; transport buffers, tasks, TLS and OS socket
memory still need their own measured resource budget.

`--protocol actions` instead requires the real `mailwoman-actions-v1` ready frame
and a successful read-only `Mailbox/get` response for each authenticated account.
Its report explicitly states that action connections **do not prove push
delivery**. No send, submission, move or other mail mutation is issued.

Reports contain aggregate ready/live/closed/error counts, distinct authenticated
principal/client counts, steady-window timings, push verification counts, Ping/Pong observations and admission
timing totals/maxima. They contain no cookies, account IDs, CSRF tokens, fixture
secrets, raw frames or marker values. Invalid/incomplete campaigns exit nonzero.
The included local fake-WebSocket tests check readiness, own-account delivery,
foreign-account rejection and bounded cleanup; they do not replace a 10,000-user
run against the actual router.
