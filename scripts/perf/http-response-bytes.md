# Response byte measurements

`bytes` remains the sum of actual Fetch body-stream chunk lengths from all
attempts, including partial/failed responses and completions during drain.
It is decoded body size, not socket, HTTP header, TLS, or compressed wire size.

The generator adds these finite counters at aggregate and existing fixed-shape
`byRequest` boundaries:

- `acceptedDecodedBytes`: full validated successful2xx completions, including drain.
- `acceptedDuringScheduledWindow`: successful completions at or before the SAME
  monotonic scheduled deadline used by the existing accepted counter.
- `acceptedDuringScheduledWindowDecodedBytes`: only those in-window completions.

The deadline is inclusive. Partial reads, validation failures, non2xx responses,
and drain contribute nothing to the last counter. An empty successful response
can legitimately add a count with zero bytes. Raw worker snapshots require all
new counters; missing legacy proof is rejected rather than invented. Aggregation
requires top-level/window agreement and exact per-shape sums, bounded fixed
endpoint labels and safe integers; impossible or overflowing inputs fail.

`responseByteMeasurement` is `fetch-decoded-body`; `wireResponseBytes` is null.
Neither Content-Length nor configured body/sample sizes are measured wire proof.
No response contents, headers, private identities or credentials are reported.
In-window decoded bytes/second may be calculated from the new counter divided
by scheduledSeconds; use each shape's actual accepted window count for its mean.
Do not divide existing all-attempt bytes by the in-window accepted count.

Representative paged query+headers is one HTTP request containing four methods
and up to50 header rows. Mark/read+restore and move+restore are each one HTTP
with set/get/restore/get and two document mutations. Counters do not turn methods
or documents into HTTP operations. Mixed shape bandwidth needs measured shape
counts; a50-row payload must not be multiplied by all representative HTTP RPS.

Focused tests use a controlled monotonic clock for actual measure() streaming,
real loopback gzip Fetch decoding, and bounded synthetic worker receipts. They
are correctness gates, not a load run, production network or capacity proof.
