# HTTP load and adversarial acceptance

Run against a disposable, isolated Mailwoman installation and test mailbox. The
runner requires an explicit target origin and refuses other hosts unless explicitly enabled.
It performs reads only after one optional fixture login. It never retries a
refused login. Do not supply production account credentials.

```powershell
node --test scripts/perf/http-load.test.mjs
node scripts/perf/http-load.mjs --base=http://127.0.0.1:42214 --scenario=public --concurrency=1,8,32,64 --seconds=30 --output=.orchestration/reports/load-public.json

# Supply fixture credentials through the environment, not command-line arguments.
# MAILWOMAN_LOAD_USERNAME / MAILWOMAN_LOAD_PASSWORD, or MAILWOMAN_LOAD_BEARER.
node scripts/perf/http-load.mjs --base=http://127.0.0.1:42214 --scenario=mail --concurrency=1,8,32,64 --seconds=30 --max-p95=5000 --output=.orchestration/reports/load-mail.json
node scripts/perf/http-load.mjs --base=http://127.0.0.1:42214 --scenario=malformed --concurrency=1,8,32 --seconds=20 --output=.orchestration/reports/load-malformed.json
node scripts/perf/http-framing.mjs --base=http://127.0.0.1:42214
```

Public traffic exercises health, the application shell and public configuration. Mail traffic
also exercises authenticated identity/session reads, mailbox enumeration and
paged message queries. Malformed traffic mixes health probes with invalid JSON
and JMAP envelopes; rejection is expected, a crash or successful acceptance is
not. Application-level JMAP failures count as errors even when HTTP returns 200.

Each stage reports throughput, transferred bytes, p50/p95/p99/max latency,
status counts, timeouts and errors, both overall and per request type. Percentiles
include failed requests and use bounded histogram buckets rather than an
unbounded sample array. The stage deadline stops new work; in-flight requests
retain their bounded timeout and are included in elapsed time. The report also
checks health after the campaign. Unexpected responses fail the command; an
optional p95 budget can enforce a chosen deployment target.

HTTP 429 refusals are always counted separately as `throttled`. By default they
also fail a capacity run. Use `--allow-throttles=true` only when testing expected
admission control, optionally with `--max-throttle-rate=0.1`. This does not turn
them into successful mail operations: inspect the per-method status counts and
accepted traffic as well as total throughput. Never infer throughput from a
campaign dominated by refused work.

The default is a **closed-loop concurrency test**. Add `--arrival-rate=100`
for fixed arrivals independent of response speed; `--concurrency=32` then caps
outstanding requests rather than generating new requests after each response.
The report separates `droppedCapacity` (outstanding cap reached) from
`droppedScheduler` (the generator missed a scheduled arrival). Arrivals due within one event-loop turn are batched; arrivals over 100 ms
late are discarded instead of replaying an unbounded backlog. Both fail the run unless an explicit `--max-drop-rate`
budget allows them. Latency measures dispatched requests only, so always read
drop counts alongside percentiles. A saturated generator is not proof of server
capacity. Each stage's `offered` equals completed requests plus both drop counts.

Neither mode alone establishes production capacity. Record the exact server binary/image digest,
build profile, CPU/memory limits, database, mailbox population, proxy/TLS path and
concurrent host activity with results. Debug binaries on a shared build machine
are useful for functional stress acceptance, not capacity promises. Compare
release builds under the same conditions, including cold and warm caches,
SQLite and PostgreSQL, and enabled protection policies.

The runner does not transmit malformed TCP/IP packets, exhaust disk space, mutate
mail, or run a distributed denial-of-service campaign. Raw HTTP framing,
slow-header/body behavior, HTTP/2 stream pressure and QUIC-specific limits need
their own isolated transport tests. Existing parser fuzz targets are documented
in [fuzz/README.md](https://github.com/supermarsx/mailwoman/blob/master/fuzz/README.md); having a harness is not evidence of a
completed long-running fuzz campaign. Preserve minimized failures as regression
tests and record actual fuzz runtime/corpus coverage separately.

`http-framing.mjs` sends a small real-socket corpus: invalid methods/header names,
conflicting or invalid lengths, and a 96 KiB header. It requires literal loopback,
checks healthy controls before/after, and fails if the proposed header budget is
not enforced. On the first baseline binary the four malformed cases returned
400, but the large header returned 200: this gate is intentionally pending the
explicit inbound header-budget implementation. Add
`--slow-header=true --timeout=12000` only when verifying the configured 10-second
incomplete-header deadline. The corpus does not replace parser fuzzing or
request-smuggling tests through the actual deployment proxy chain.

Adaptive request protection is a separate, disabled-by-default feature under
implementation. Load testing must distinguish expected configured throttles from
unexpected failures and verify ordinary traffic recovers after a temporary
penalty. A temporary test threshold must never be silently applied to production.

## Sustained, burst and peak campaigns

Profiles use open-loop offered rates of 20,000, 50,000 and 100,000 requests/sec by
default. These are requested rates, not throughput promises. Override them with
`--rates=100,500,1000`. Each profile stage lasts `--seconds`; burst uses the first
rate, then the final rate for at most five seconds, then the first rate again.
Peak climbs all rates and descends them; sustained holds each rate in order.

```sh
node scripts/perf/http-load.mjs --base=http://127.0.0.1:42214 --scenario=mail --profile=sustained --concurrency=256 --seconds=30 --output=.orchestration/reports/mail-sustained.json
node scripts/perf/http-load.mjs --base=http://127.0.0.1:42214 --scenario=mail --profile=burst --rates=100,500,1000 --concurrency=128 --seconds=10 --output=.orchestration/reports/mail-burst.json
node scripts/perf/http-load.mjs --base=http://127.0.0.1:42214 --scenario=mail --profile=peak --rates=100,500,1000 --concurrency=128 --seconds=10 --output=.orchestration/reports/mail-peak.json
```

Remote staging additionally requires `--allow-remote=true` and
`--acknowledge-staging=https://your-explicit-staging-origin` exactly matching
`--base`. Use only an installation and fixture accounts you control. No command
here sends mail, mutates messages, or automatically targets the public service.
Authenticate through environment variables described above. Headers, cookies,
credentials, response bodies, query arguments and account IDs are absent from
reports; the explicit origin and fixed scenario/request labels are retained.

Each JSON report also produces a sibling Markdown summary. Distinguish:

- `arrivalRate` and `offered`: the intended schedule, including missed arrivals.
- `attemptedRequestsPerSecond`: dispatched attempts divided by scheduled time.
- `requestsPerSecond` / `achievedRequestsPerSecond`: completions divided by total
  elapsed time, including the bounded drain after scheduling stops.
- `accepted` / `acceptedRequestsPerSecond`: expected successful 2xx application
  results and their elapsed-time rate; accepted 429 responses never count here.
- `acceptedDuringScheduledWindowRps`: successes actually completed before the
  scheduling deadline, divided by scheduled time; drain completions are excluded.
- `droppedCapacity`: arrivals not dispatched because outstanding work hit its cap.
- `droppedScheduler`: arrivals missed by the load generator, including overdue
  slots when the event loop could not keep up.
- `schedulingLagMs`: lateness of dispatched requests relative to their slots.
- `plannedArrivalLatencyMs`: latency from intended arrival through completion for
  dispatched requests. Dropped requests have no latency sample.

Always evaluate drops alongside both latency distributions. This prevents a
slow server or saturated generator from appearing healthy just because requests
were never offered to the server. Histogram resolution is 1 ms through one
second, 10 ms through ten seconds and 100 ms thereafter, with an overflow bucket.
The report includes timeout, error and throttle counts and per-endpoint results.

The generator keeps no pending-arrival queue and caps outstanding work at the
explicit concurrency (maximum 4096). Response streaming is capped at 8 MiB per
response; JSON validation buffers at most 64 KiB per outstanding response. Results
use bounded histograms and at most 100 request labels. Large concurrency still
costs connections, buffers and CPU; a single Node process may not reach 20k RPS.
Run a generator on separate hardware for application-capacity measurements and
record its CPU/network saturation. This fetch-based harness does not measure
HTTP/2 multiplexing, WebSocket push delivery or browser rendering.

## Isolated harness verification and application matrix

```sh
node --test scripts/perf/http-load.test.mjs
node scripts/perf/http-load-fixture.mjs .orchestration/reports/transport-smoke.json
```

The isolated fixture implements simulated authentication, read/refresh responses,
a one-item cache and deterministic protection throttles using Node HTTP. It
verifies actual transport, timeout, dispatch, histogram and overload reporting.
It does not implement Mailwoman, SQLite, upstream IMAP/JMAP sync, real protection
policies or real caching. Its "refresh" is a fixture response, not an upstream
reconciliation test. See the checked-in [local smoke artifact](../perf/http-fixture-smoke.md).

The opt-in Rust fixture starts the **actual Mailwoman HTTP router and middleware**
with a fresh SQLite store and the in-repo JMAP mock on ephemeral loopback ports:

```powershell
$env:MW_RUN_HTTP_LOAD_FIXTURE='1'
$env:MW_LOAD_FIXTURE_RATES='20000,50000,100000'
$env:MW_LOAD_FIXTURE_SECONDS='0.3'
cargo test -p mw-server --test request_load_fixture -- --ignored --nocapture
```

Omit rate/seconds overrides for three one-second 100/500/1000 RPS stages. The
test runs rate/adaptive/user-agent/scanner modes off, then rate/adaptive/user-agent
observation with scanner enforcement. Separate session rate limits retain their
deployment defaults, so load may produce expected visible 429 responses even in
the protection-off campaign.
Its Node driver exercises authenticated identity, mailbox enumeration, email
queries, seeded message-header reads and concurrent refresh-handler requests, followed by a scanner-path
probe. Every endpoint must pass an individual functional preflight before load;
overload errors are then reported rather than hidden by other successful methods.
Each process is bounded to 90 seconds; no listener accepts public traffic.
The fixture writes its session handoff privately (Unix 0600; inherited user temp
ACL on Windows), passes only the filename and removes it after the child exits.
Do not use a shared world-readable Windows temp directory. Session values never
appear in stdout or reports.

Reports go to `.orchestration/reports/real-http-load/`; override the directory
with `MW_LOAD_REPORT_DIR`. This test uses the Cargo profile selected for the
command. Add `--release` for performance comparisons and record that profile.
Fresh-after-sign-in and subsequently warmed stages are labelled explicitly;
sign-in/session discovery already touches caches, so the first stage is not an
untouched-cache claim. Proxy-mode refresh exercises the actual handler but does
not perform an IMAP reconciliation. Mock upstreams, local HTTP and a tiny seeded
mailbox remain different from a deployed TLS/reverse-proxy/upstream workload.

For an actual Mailwoman release candidate, run the authenticated `mail` and
`malformed` scenarios against the full server/proxy path, with protection enabled
and disabled in separate disposable deployments. Repeat after an explicit cold
cache reset and after warm-up. Record binary/image digest, release profile,
protection configuration, backend/database, mailbox/message population, cache
preparation, machine limits and independent generator resources. For concurrent
folder-refresh/upstream-sync acceptance use isolated backend fixtures and verify
new server state as well as latency; the read-only mail-query scenario does not
claim to exercise forced upstream refresh. Do not treat static assets, health
responses, simulated fixtures or rejected traffic as authenticated mail capacity.
