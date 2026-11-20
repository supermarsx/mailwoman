# Ordered index publication candidate

This candidate gives each Engine one bounded publication owner. Mutators reserve
job and input-byte admission before upstream or SQL work. Known successful SQL
changes synchronously transfer their current-source IDs to that owner before the
next suspension. Dropping a waiter does not cancel accepted work or release its
admission permits. A dropped armed ticket requests reconciliation of current
source, rather than applying an enqueue-time document.

The owner groups compatible requests within configurable job, document, byte and
collection-window bounds. It publishes one commit and reader reload per compatible
group and acknowledges only after publication and fresh source validation.
Rebuild and account-erasure commands are ordered barriers. Expensive parsing,
index writer locking, encrypted writes, commits, reloads, merges and garbage
collection run on the dedicated owner thread, rather than a Tokio worker.

SQL remains authoritative. Grouped observations include ownership, source
revision and content identity, metadata, account state, indexing restrictions,
and the SQLite generation. The current custom posture source is also checked.
These are optimistic observations, not a lease spanning SQL and Tantivy.
Revalidation after publication repairs changed source before success; bounded
instability or uncertain publication closes admission and retains bounded repair
keys. Explicit recovery reconciles those keys before reopening admission.
Overflow requires a full source rebuild. All-account rebuilding closes new
admission and captures a failure epoch before scanning accounts. Completion
reconciles the bounded retained repair keys, including deleted accounts absent
from the scan, and clears overflow only if the failure epoch remains unchanged
and no tracked repair keys remain. A new failure during the scan requires
another reconciliation; completion never silently clears it.

Account erasure closes an epoch-bound account fence before physical publication.
The source-purge caller holds that guard through authoritative deletion and
verification. An old guard cannot complete a newer erasure. Physical success
requires retired ciphertext segment files to be absent and garbage collection
to report no deletion failures. A caller retaining an external raw Tantivy
Searcher may retain already-decrypted plaintext in memory; this is not a global
reader-revocation mechanism.

Existing limitations remain explicit:

- An unknown SQL commit or process crash before ownership handoff is not covered
  by a durable dirty journal. Existing startup reconciliation remains necessary.
- Arbitrary raw SQL and external direct Search writers bypass the Engine's
  supported mutation/barrier protocol. A fresh snapshot is not cross-process
  linearizability or physical-erasure acknowledgement for those writers.
- Prepared output and materialized authoritative source have bounded group
  budgets. The Store source reader additionally caps one sealed source at
  64 MiB. Oversized work fails honestly rather than silently skipping mail.
- Reading an existing Tantivy stored document can allocate its first decoded
  representation before the encoded-JSON budget is checked. RFC822 size is an
  early guard, not a strict first-decode memory bound for legacy stored JSON.
- A legacy index document without a content witness needs current MIME for
  ingest, rebuild and strict content reconciliation. Explicit metadata-only
  intents preserve its existing owned content while updating current metadata;
  they do not create a content witness. Attachment projection repair updates
  only an existing document's attachment flag and avoids re-reading raw MIME.
  Cancellation and recovery retain the bounded intent. Any post-validation
  source change upgrades repair to strict content reconstruction. Missing
  required source fails into repair; no reader body lease is renewed.
- A failed commit may have reached durable storage. A failed reload is not a
  rollback. The owner remains closed until explicit recovery and source repair.

The frozen release comparison below establishes a diagnostic improvement for
this workload. It does not establish the 40,000-RPS or 10,000-user capacity target.

## Measured release comparison, 2026-11-20

Four alternating baseline/candidate/candidate/baseline runs used 100 simulated
authenticated users with 50 messages each, the representative UI property sets,
encrypted SQLite and sealed search storage. The application had four CPU cores,
4 GiB memory and no swap; two generator processes had separate CPU affinity and
a combined two-core/1 GiB limit. Every measured window lasted 60 seconds with
16 outstanding requests. No compiler or other owned load campaign ran alongside
the comparison.

| Diagnostic | Baseline | Publication owner |
| --- | --- | --- |
| Successful authenticated HTTP RPS, run 1 | 38.25 | 115.25 |
| Successful authenticated HTTP RPS, run 2 | 37.93 | 114.23 |
| Mean successful HTTP RPS | 38.09 | 114.74 |
| Mean of the two run p95 latencies | 1,020 ms | 627 ms |
| HTTP errors / throttled requests / timeouts | 0 / 0 / 0 | 0 / 0 / 0 |

The successful throughput ratio was **3.01x**. Throughput counts only validated
requests completed inside the scheduled window; requests drained afterward do
not inflate it. The latency row averages per-run quantiles and is not a pooled
percentile. All runs passed live policy revoke/restore, linked-login denial,
cross-account isolation, scanner enforcement and encrypted-storage checks.

The candidate also had to finish every required index publication without dirty,
failed, queued or poisoned work or telemetry debt. In its first run, 2,772 real
message changes produced 2,772 submitted/completed jobs and published changes,
grouped into 393 commits. A zero-work or partially skipped publication path does
not qualify as a throughput improvement.

Frozen sources and artifacts:

- Baseline revision: `426177bfcfaaf0ecfb7608cd3cead79379bd2b09`.
- Candidate revision: `4501ef46be8f97d214d9b326255d6e8f23ac25b0`.
- Baseline image: `sha256:544c8849784e251d3ae4b70e99d12f4f3efbdfe60c2b309c7b39b2bf56245c4f`.
- Candidate image: `sha256:75d72e0599e9d13f400a7d1935adb43475b1f668802da6b791111211088fe3f2`.
- Candidate normal executable: `e70d8c256bd3dc126e642af4920c18e64c054a5eb3460bdf395aeca700ab7651`.
- Local retained comparison: `.orchestration/reports/perf-investigation/index-publication-comparison-v2.json`.

The image's 13 Linux fixture checks passed and profiling did not change the
normal executable's checksum. A separate 20-second instrumented run initially
failed the harness's future-heartbeat timestamp check and produced no usable
report. Its retained failure was excluded; a fresh retry with unchanged guards
passed. Instrumented results are excluded from the throughput comparison.

The retry observed 140 commits totaling 16.06 seconds and 140 reader reloads
totaling 1.30 seconds. These process-wide elapsed counters include waits and
overlapping background work; they cannot be added or read as CPU percentages.
Commit/publication cost remains a priority for further measurement.

This is a local diagnostic with 100 synthetic users, not a 10,000-user capacity
test. The host differs from the target host despite enforced container limits.
Upstream mail servers, successful sending, external physical generators and
WebSocket capacity were not measured. The subsequent contact/header-cache and
DLP-worker changes are not included in these numbers. The 40,000-RPS target
remains unproven.

## Verification before the release benchmark

- Engine library: 324 passed, zero failed; one existing 20,000-row scale test
  remains ignored. This includes 17 behavioral publication cases and the
  exhaustive intent-union/coverage test.
- Search publication: nine passed, including real encrypted-file cleanup,
  rollback, poisoned-writer isolation and reader visibility.
- Store publication readers: nine passed, including actual PostgreSQL; source
  mutations: two contracts passed against SQLite and PostgreSQL.
- Benchmark validation: 25 Node tests passed, including missing/poisoned/deferred
  publication rejection and overlay dependency coverage.
- Native server fixture and main executable passed Cargo metadata checks with
  `load-test-fixtures`. These checks do not execute the release server.

These results establish component behavior and integration compatibility, not
40,000 RPS, 10,000-user capacity, live upstream verification, or deployment.
