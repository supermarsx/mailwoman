# Index publication and load qualification

Native fixture diagnostics expose `searchIndex.publication`: bounded aggregate
queue, completion, failure and repair counters. Reports whitelist numeric and
boolean fields; account IDs, repair keys, paths and content are excluded.
Missing evidence from older fixtures remains unreported and cannot qualify a
new capacity result.

Both measurement boundaries must show a running, unpoisoned owner with no
pending jobs, bytes, queued jobs, repair keys, full-rebuild requirement or
active erasure. Cumulative counters must not move backwards, new failed jobs
are forbidden, and submitted/completed job deltas must agree. Both the
multi-user target and the single-process rate indicator require this proof.
The independent `productionIndexPublicationQualified` field records its result.

This verifies **observed submitted work**. It cannot establish that every SQL
mutation was submitted, close the SQL-to-publication process-crash gap, or
prove query correctness. Writer adoption audits, actual full-text mutation
tests and controlled release benchmarks remain necessary. Zero job deltas are
valid for read-only diagnostics and do not prove a mutation path was exercised.

`Email/set` retains its existing best-effort indexing behavior: successful
source mutations remain reported as updated if subsequent publication fails.
A successful owner receipt proves publication and reader reload; an HTTP
success alone does not prove that on a publication failure. Dirty/poisoned
publication prevents benchmark qualification rather than hiding deferred work.

Focused verification: `node --test scripts/perf/index-publication-health.test.mjs scripts/perf/fixture-clients.test.mjs scripts/perf/http-multi.test.mjs scripts/perf/fixture-properties.test.mjs`.
This tests report validation and qualification, including overlay dependencies;
it does not compile the Rust fixture or establish any throughput gain.
