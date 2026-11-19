# mw-protection


## Matching and performance verification

The curated scanner catalog and known user-agent claims are compiled once into
Aho-Corasick automatons. Scanner matches still require whole path segments;
user-agent claims retain token boundaries and class precedence. Namespace
exclusions and path decoding limits remain part of the classifier. Operator
regex sets keep their existing bounded regex engine and compile once per loaded
policy. No identity is authenticated by a claimed user-agent string.

Delimiter scanning uses `memchr`'s portable, runtime-selected acceleration. No
build-wide CPU target or mandatory AVX instruction set is added. Small inputs
and unsupported architectures retain the library's scalar fallback.

Frozen pre-optimization classifiers compile only in tests. Differential tests
exercise catalog variants, encoding and namespace boundaries, browser assets,
claim precedence, and deterministic adversarial fixtures. Compare actual Rust
release-mode implementations without another build running:

```sh
cargo test -p mw-protection --release --lib policy_match_benchmark -- --ignored --nocapture --test-threads=1
```

The benchmark warms static policy compilation, checks old/new decisions, then
reports median nanoseconds per operation from seven alternating samples. It
covers normal, hostile, encoded and long inputs plus 1/32/256/512-rule catalogs.
These are classifier timings on the measured host, not whole-request latency or
application build-time claims.

See [measured policy matching results](../../docs/perf/policy-matching.md) for
the Windows release fixture results, reproduction commands and measurement limits.
