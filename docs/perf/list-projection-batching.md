# Message-list index projections

`Email/get` first reads message rows through the account-scoped SQL getter. Only
those owned, still-present IDs can enter the list-projection index query. A fresh
posture check precedes derived index access; explicit zero-access posture skips
the projection and Focus correspondence index reads, including stale documents
left from before a posture change. No authorization decision is cached.

Mentions, calendar hints and the Focus list-header signal share a request-local
projection map. For a normal page of 30 distinct owned messages, the previous
mentions plus Focus path performed 60 exact-ID Tantivy searches. The new path
performs one account-filtered TermSet search and decodes each matched stored
document once. It still reads individual stored documents: one search is not one
physical document read. Larger internal batches use chunks of 5,000 IDs while
retaining one searcher generation. Requested duplicates are projected back in
their original positions without additional index work.

The index query filters by account and requested IDs, then verifies the decoded
account and stable-ID bindings again. The SQL ownership proof remains essential;
the index never establishes ownership. Separate Store and index reads do not
provide an atomic snapshot across both systems. A message absent before the SQL
read never reaches the index, and normal request-snapshot semantics remain for
concurrent changes after that read.

Only keywords, list-header signals and calendar hints are retained. Large body,
attachment-text and other unused strings are validated with non-retaining serde
visitors. Escaped JSON strings may still use the parser's scratch buffer. The
thin decoder preserves IndexDoc's required-field, type, optional-default and
malformed/missing rules; tests compare both decoders. Invalid documents remain
absent. Ambiguous duplicate live documents are treated as missing rather than
choosing an arbitrary version; excess hits fail the bounded batch and the engine
uses its existing absent-index defaults.

Focus contact lookups, Sent-window construction and recipient phrase matching
retain their existing semantics. Contact-query batching and persistent raw-header
projections are separate work. The authentication summary remains its existing
single account-scoped SQL batch.

Counters distinguish legacy single-document searches, list-projection searches,
and projection stored-document reads. They are structural work measurements,
not RPS or latency claims. Application throughput requires a separate controlled
run with the actual message-list properties and security policies.

## Measured follow-up

The 2026-11-20 follow-up used the same 100 users, 5,000 messages, UI-default
property sets, 16 outstanding requests and two generator workers as the
[previous diagnostic](ui-default-profile-baseline.md). Both ten-second phases
completed with zero errors, throttles or timeouts. Account isolation, locked
policy revocation/restoration and the enforced scanner probe passed; both
containers exited zero and the paired report artifacts were verified.

| Endpoint | Successful in-window RPS | p95 / p99 |
| --- | ---: | ---: |
| Identity | 1,108.3 | 23 / 28 ms |
| UI-default read/write mix | 110.4 | 330 / 368 ms |

In the mixed phase, legacy exact-document searches fell from about 10.9 to 0.396
per HTTP request. The new path added 0.400 batch searches and 5.300 stored-document
reads per request. SQL reads remained about 70.46 per request. App observations
were 3.18 logical CPUs and 307 MiB RSS. The lower index search count did **not**
produce a measured end-to-end throughput gain over the earlier 112.5 RPS. One
short run per version does not establish a regression or a capacity result;
optimized CPU profiling is needed to identify the remaining cost.

The runtime image was
`sha256:6aca3514a0120ecaca7a57963a1a754c29577f62f88180bf26a0d6a18893ab20`,
with normal executable SHA-256
`4cb3dec5767509ace10b4e366a2e21bda3ff10fc794d48979e0a28bd6bc59138`.
Its source snapshot ends at `72d9f20e`; subsequent admission batching and
change-history fixes were not included. Request profiling was enabled. The app
was limited to 4 CPUs/4 GiB and the generator to 2 CPUs/1 GiB. Dataset, TLS,
upstream, WebSocket and whole-host limitations from the previous diagnostic
still apply. The 40k-RPS goal remains unachieved.

Local evidence is under
`.orchestration/reports/perf-investigation/sqlite-projection-ui-profile-100-workers2/`.
