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
