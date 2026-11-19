# Real HTTP fixture smoke

The opt-in real Mailwoman HTTP fixture passed on 19 November 2026. It used
Windows, Rust 1.98.1, the unoptimized test/debug profile, and a host with two
Intel Xeon Gold 6138 CPUs (40 physical cores / 80 logical processors). The test
runtime used four Tokio worker threads, with a separate Node 24.14.1 generator
and a maximum of 32 outstanding requests.

This is a short correctness/overload smoke, **not production capacity acceptance**.
Each stage scheduled arrivals for only 0.3 seconds, followed by a bounded drain.
The workload used fresh temporary SQLite stores and a tiny in-repo JMAP mock over
local HTTP, without TLS, a deployment reverse proxy, real upstream latency or a
production mailbox population. Sign-in, discovery and individual endpoint
preflights precede the first stage; subsequent stages are warmed. These are not
untouched-cache measurements or a controlled before/after performance comparison.

## Observed requests

The actual router/middleware served authenticated identity reads, Mailbox/get,
Email/query, seeded Email/get header reads and concurrent refresh-handler calls.
Email/get validation required every sampled ID in the response. Proxy-mode refresh
was explicitly validated as `upstream:false`: this exercises the authenticated
route, not upstream reconciliation. No messages were sent or changed.

| Policy configuration | Offered RPS | Completed requests | Completed RPS | Accepted RPS | Capacity / scheduler drops | 429 during load | p50 / p95 / p99 ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Off | 20,000 | 71 | 194.2 | 194.2 | 5,916 / 13 | 0 | 142 / 205 / 219 |
| Off | 50,000 | 77 | 211.3 | 211.3 | 14,821 / 102 | 0 | 141 / 185 / 196 |
| Off | 100,000 | 75 | 210.9 | 210.9 | 29,530 / 395 | 0 | 140 / 185 / 204 |
| Observe + scanner enforce | 20,000 | 48 | 98.5 | 98.5 | 5,916 / 36 | 0 | 257 / 360 / 361 |
| Observe + scanner enforce | 50,000 | 48 | 114.9 | 114.9 | 14,562 / 390 | 0 | 246 / 326 / 327 |
| Observe + scanner enforce | 100,000 | 52 | 102.4 | 102.4 | 28,967 / 981 | 0 | 248 / 366 / 391 |

All dispatched load requests returned validated successful responses: no application
errors or timeouts were observed. Most scheduled arrivals were dropped by the
bounded generator; they never reached the server and have no latency samples.
Completed/accepted rates divide by elapsed time including drain. The raw reports
also provide actual successes completed within the scheduling window, scheduling
lag, planned-arrival latency and per-endpoint counts. Small sample sizes make the
reported tail percentiles descriptive rather than stable capacity estimates.

The off campaign disables rate/adaptive/user-agent/scanner modes in that policy
family. The on campaign observes rate/adaptive/user-agent activity and enforces
scanner paths; separate session rate limits retain their deployment defaults.
A post-campaign `/.env` probe returned **404 with scanner protection off** and
**429 with enforcement on**. This separate scanner probe is not load throughput.

## Local artifacts and reproduction

Raw ignored reports, relative to the repository root:

- `.orchestration/reports/optimization-validation/real-http-load/protection-off.json`
- `.orchestration/reports/optimization-validation/real-http-load/protection-on.json`
- `.orchestration/reports/optimization-validation/server-real-http-load.log`

Each JSON has a sibling Markdown summary. The fixture test passed in 29.18 seconds,
including setup, preflights, both campaigns and cleanup. Follow
[load-testing instructions](../deploy/load-testing.md) to reproduce; use a release
build and a representative isolated deployment for any capacity investigation.
