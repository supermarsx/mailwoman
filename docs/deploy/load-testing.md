# HTTP load and adversarial acceptance

Run against a disposable, isolated Mailwoman installation and test mailbox. The
runner defaults to loopback and refuses other hosts unless explicitly enabled.
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
include failed requests and use fixed 1 ms histogram buckets rather than an
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

This is a **closed-loop concurrency test**, not an open-loop arrival-rate or
production capacity measurement. Record the exact server binary/image digest,
build profile, CPU/memory limits, database, mailbox population, proxy/TLS path and
concurrent host activity with results. Debug binaries on a shared build machine
are useful for functional stress acceptance, not capacity promises. Compare
release builds under the same conditions, including cold and warm caches,
SQLite and PostgreSQL, and enabled protection policies.

The runner does not transmit malformed TCP/IP packets, exhaust disk space, mutate
mail, or run a distributed denial-of-service campaign. Raw HTTP framing,
slow-header/body behavior, HTTP/2 stream pressure and QUIC-specific limits need
their own isolated transport tests. Existing parser fuzz targets are documented
in [fuzz/README.md](../../fuzz/README.md); having a harness is not evidence of a
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
