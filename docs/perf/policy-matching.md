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
fixtures against frozen pre-optimization classifiers. Both ignored release
benchmark tests passed. Each workload below also checks reference/optimized
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
No particular SIMD instruction execution was measured.

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
