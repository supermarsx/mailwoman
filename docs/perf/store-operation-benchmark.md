# Owned SQLCipher Store diagnostic

Run only in the serial performance lane. This ignored case owns an encrypted
tempfile database; it never reads deployment DSNs, keys or account data.

```powershell
$env:MW_STORE_OPERATION_BENCH='1'
cargo test --offline --locked --release -p mw-store --lib operation_benchmark::real_sqlcipher_store_operation_breakdown -- --exact --ignored --test-threads=1 --nocapture
```

Require exactly one passed case, zero ignored and four STORE_OPERATION_BENCH JSON
records. The process flag is mandatory. Retain exact source revision, command,
Cargo profile, OS, hardware, storage volume and concurrent-job evidence with output.
The reported debug-assertion setting is not proof of a named Cargo profile or its
optimization flags; the retained command provides that provenance.

The fixed workload seeds 50 unread messages with sealed envelopes through real
Store APIs. Migrations, key derivation, seed and API warmup are outside timing.
Sequential warm point reads repeatedly fetch one message; five concurrent futures
perform the same 1,000 reads over 50 IDs. Futures share one measurement task; none
are spawned outside task-local counters. No dedicated-connection, all-pool-handles
or physical page-cache-hit claim follows. Production pool settings remain intact.

Scalar writes and twenty 50-row batch writes both perform 1,000 alternating
Seen/unread row changes through existing committed transaction APIs. Batch latency
is per 50-row operation; scalar latency is per single-row operation. Compare row
rates and elapsed times, not their operation percentiles directly. Stored flags
and mailbox unread counters are verified outside timing. SQLCipher version and
physical encryption are checked; header inspection happens after Store close.

qCounters covers q() and instrumented acquisitions. Query intervals include pool
acquisition and overlap acquisition intervals: do not add or subtract their sums
to invent SQL execution time. Commit and direct SQLx are not independently timed.
Cohort elapsed includes real commits, trigger work, Rust mapping and SQLCipher.
Crypto CPU, fsync time, SQL-only execution, raw engine capacity and HTTP/app-route
processing are unavailable. Warm reads do not measure cold page-decrypt throughput.

This short synthetic Store diagnostic is not sustained capacity, the representative
ten-shape HTTP workload, or evidence for the 40k goal. A scalar/batch difference
cannot be attributed solely to commits or encryption. Use separately retained route
profiles without subtracting unmatched populations or overlapping timers.
