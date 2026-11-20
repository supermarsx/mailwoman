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

No request-per-second improvement is established by this source change. Focused
Store, real full-text publication, cancellation, race, erasure and integrated
Engine gates, followed by a frozen policy-loaded sealed-storage comparison, are
required before capacity claims.

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
