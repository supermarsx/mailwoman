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

Local evidence is under
`.orchestration/reports/perf-investigation/sqlite-current-loaded-100/` and
`.orchestration/reports/perf-investigation/sqlite-current-loaded-profile-100/`.
Both directories contain the credential-free JSON/Markdown reports and resource
metadata. Private handoff files and generated encryption keys are separate and
must not be published.
