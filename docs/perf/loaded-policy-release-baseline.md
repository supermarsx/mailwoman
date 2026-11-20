# Loaded-policy release baseline: 100 users

This is a small diagnostic baseline, **not** evidence of the 40,000-RPS or
10,000-simultaneous-user target. It measures 100 distinct authenticated users,
two seeded messages each, and ten-second phases after five-second warmups.
The mixed profile is read-only. Sends, large mailboxes, persistent WebSocket
sessions, TLS, a reverse proxy and real upstream network traffic are not measured.

The Linux GNU fixture uses the production router, native engine and release
optimization profile. Its deterministic upstream transport does not bypass
authentication or server policy. Whole-database encryption is required.
The immutable image is
`sha256:facf1250be1b24d493694bb47d7c411a2fab16781cd0b8cfbf009e24000434d2`.

The application has 4 vCPUs, 4 GiB RAM and no swap, on CPUs 0–3. The separate
generator has 2 vCPUs, 1 GiB RAM and no swap, on CPUs 8–9. Both use an isolated
network namespace without published host ports. Simulated proxy client addresses
are distinct; these are not distinct physical generator hosts. The Docker VM has
more host memory than the application limit, so this does not establish a
16-GiB whole-host deployment result.

## Policy and startup controls

The run keeps a locked `access.protocols = ["imap"]` policy loaded throughout
measurement and leaves admission budgets at their administrator defaults.
Before measuring, the first and last sessions successfully authenticate, then
both immediately receive the expected HTTP 403 when the protocol list is emptied.
Restoring the policy restores successful real mail queries. The scanner probe is
also enforced. The startup gate verifies all 200 seeded documents are indexed and
no rebuild is running before handing control to the generator.

## Measurements

Successful throughput counts responses completed inside the scheduled window;
draining responses do not increase that rate.

| Requests | Outstanding | Successful RPS | p95 / p99 latency | Errors / throttles / timeouts |
| --- | ---: | ---: | ---: | ---: |
| Identity | 16 | 768.7 | 30 / 34 ms | 0 / 0 / 0 |
| Identity | 64 | 715.9 | 109 / 117 ms | 0 / 0 / 0 |
| Mixed mail reads | 16 | 284.0 | 85 / 95 ms | 0 / 0 / 0 |
| Mixed mail reads | 64 | 291.3 | 302 / 319 ms | 0 / 0 / 0 |

Every phase accepted requests from all 100 users. Application CPU observations
were 3.25–3.68 logical cores; observed process RSS was 275–288 MiB. Generator CPU
was 0.25–0.47 logical cores. These are observations, not peak-memory guarantees.
The demand-grown SQLite pool had eight open connections and no acquisition
failures. Increasing outstanding requests raised latency without a comparable
throughput gain.

## Follow-up profiling

A separate instrumented run reached approximately 764 identity RPS and
280 mixed RPS. Request-local query telemetry measured roughly 32 reads per
identity request and 63 per mixed request; mixed requests invoked full
authentication about 1.6 times each. Domain-upgrade and account-gate phases
dominate the measured authorization time. These timings include database waiting
and are not a CPU flame graph.

The next work is to batch those fresh reads without removing policy checks,
inspect CPU samples, validate generator headroom separately, and measure the
larger read/write profile. No 40k capacity claim follows from this baseline.

## CPU sample and middleware clone probe

A separate diagnostic fixture retained function symbols without changing the
original stripped executable. All 992 dependency artifacts were reused; only
the final diagnostic fixture was relinked. Its profiler ran in a separate,
bounded container attached only to that synthetic fixture's PID namespace.
The 15-second user-CPU capture contains 3,447 samples and reports zero lost
samples. It is not an unprofiled capacity measurement.

Flat self samples attributed 8.71% to `AppState::clone` and 5.54% to dropping
`AppState`. Those percentages describe sampled **user CPU**, excluding kernel
time. They do not mean database waiting consumes the same CPU share. The state
contains owned configuration and paths, so cloning it across middleware layers
also copies configuration rather than only sharing runtime handles.

An isolated probe using the installed Axum 0.8.9 and the production candidate
adapter measured the following clone counts per request:

| Middleware layers | Existing route chain | Shared route templates |
| ---: | ---: | ---: |
| 10 | 140 | 60 |
| 20 | 480 | 120 |
| 40 | 1,760 | 240 |

Behavioral tests preserve request/response ordering, early denial, underlying
readiness and errors, cancellation, and concurrent execution. This adapter is
restricted to Axum's existing always-ready Route boundaries. These structural
results motivate sharing application state and route templates; their RPS
effect still requires a new release run with the same security controls.

Local evidence is under
`.orchestration/reports/perf-investigation/sqlite-current-loaded-100/` and
`.orchestration/reports/perf-investigation/sqlite-current-loaded-profile-100/`.
Both directories contain the credential-free JSON/Markdown reports and resource
metadata. Private handoff files and generated encryption keys are separate and
must not be published.
