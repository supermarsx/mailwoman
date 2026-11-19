# Policy matching performance

The scanner-path catalog and known user-agent claims use statically compiled
Aho-Corasick matchers. Whole-segment path checks, user-agent token boundaries,
category precedence, namespace exclusions and bounded path normalization remain
part of the decision. Operator regex policies compile once when loaded.

Delimiter scans use `memchr`. Both dependencies retain portable fallbacks and
select supported acceleration at runtime; the build does not require a global
CPU target or a particular SIMD instruction set.

## Measured results

Measured on the Windows development host on 19 November 2026, with the Rust
release profile. The normal test run passed 31 tests, including differential
fixtures against frozen pre-optimization classifiers. Both classifier release
benchmarks passed. Each workload below also checks reference/optimized
decision equality before timing.

| Workload | Reference time / optimized time |
| --- | ---: |
| Scanner paths: ordinary | 3.542x |
| Scanner paths: hostile | 3.772x |
| Scanner paths: long negative inputs | 7.567x |
| Scanner paths: encoded inputs | 2.813x |
| Catalog: 1 rule | 1.023x |
| Catalog: 32 rules | 2.191x |
| Catalog: 256 rules | 10.135x |
| Catalog: 512 rules | 21.378x |
| User agents: ordinary | 15.940x |
| User agents: bots | 17.023x |
| User agents: long inputs | 9.697x |

A ratio above 1 means the optimized classifier took less time for that fixture.
The single-rule result is close to parity. These figures describe this host and
these in-process fixtures; they do not establish end-to-end request throughput,
server latency, Rust build speed, or performance on another architecture.
Those classifier measurements did not identify particular SIMD instructions.
The subsequent [CPU dispatch work](./cpu-dispatch.md) separately measures and
inspects the emitted AVX-512 kernel.

## Request validation and policy caches

The request guard normalizes each path once and reuses the normalized console
prefix, keyed by its live configured value. Source addresses are canonicalized
once rather than converted to text and parsed back for country lookup. Verified
session evidence stays local to that request; handlers still authenticate
independently, and no authentication-result TTL cache is introduced.

Health, protocol and request-protection middleware share one fresh, Store-bound
settings revision within each HTTP request. Policy and subject memos are keyed
by that revision, so a committed policy change is visible on the next request
without waiting for a local TTL or invalidation event. Parsed trust and rate
policy caches bound their entries and coalesce concurrent refreshes. Failed
refreshes do not promote fallback data to a newer revision, and older refreshes
cannot overwrite a newer cached revision. Missing revision rows disable reuse.
The revision is a validation boundary, not an MVCC snapshot of all response data.

Each WebSocket action is also a new request boundary. Its dispatcher removes
the upgrade request's revision proof before invoking the HTTP router. A socket
therefore cannot retain its handshake revision across later policy changes.
The HTTP regression keeps one authenticated socket open, changes the stored
method policy without invalidation, and checks enforcement on the next frame.

The single-normalization benchmark compares the relevant helpers, including
decision parity. It does not include the complete middleware, authentication,
database or network pipeline:

```sh
cargo test -p mw-protection --release --lib request_probe_single_normalization_benchmark -- --ignored --nocapture --test-threads=1
```

The final seven-sample release run on the same development host measured
1,335.24 ns/operation for the repeated-normalization reference and 630.67 ns for
the single-normalization helper: **2.117x** reference/optimized. Decision parity
passed before timing. The raw local log is
`.orchestration/reports/optimization-validation/request-normalization-release-final.log`.

## Reproduce

Run without a competing build or other CPU-heavy task:

```sh
cargo test -p mw-protection --lib
cargo test -p mw-protection --release --lib policy_match_benchmark -- --ignored --nocapture --test-threads=1
```

The benchmark warms static matcher initialization, calibrates up to 100,000
fixture rounds against a roughly 50 ms reference sample, and takes seven samples
with alternating reference/optimized order. It prints median nanoseconds per
operation and their ratio. `black_box` prevents unused-result elimination.
Static matcher construction is excluded from timing. Catalog scaling fixtures
include the common segment preparation in both measured paths.

Fixtures cover ordinary traffic, known scanner paths, encoded and long negative
paths, synthetic catalogs, browser/mail user agents, bot claims and overlapping
tokens. Differential tests additionally exercise decoding, resource exclusions,
whole-segment boundaries, precedence and deterministic adversarial combinations.
They are bounded test corpora, not a proof for every possible input or a live
deployment load test.
