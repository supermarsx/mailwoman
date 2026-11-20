# Indexing-worker experiment

Three alternating-order release microbenchmarks compared the real Index update,
commit and reader-reload path with three workers and with one worker. Both used
the same 50-MB budget, default merge policy, 4-CPU/4-GiB container, prebuilt
document inputs and frozen `72d9f20e` source. Merger draining and writer teardown
were outside timed sections. All stored-document, foreign-document and commit
counter assertions passed.

| Phase | One worker median | Three workers median | Median paired ratio; range |
| --- | ---: | ---: | ---: |
| Bulk seed, 5,001 documents | 160.40 ms | 93.21 ms | 1.715; 1.686–2.651 |
| One changed document | 9.09 ms | 15.19 ms | 0.656; 0.358–2.851 |
| 25 changed documents | 9.77 ms | 15.35 ms | 0.667; 0.458–1.077 |
| 500 changed documents | 24.58 ms | 26.62 ms | 0.950; 0.793–1.448 |

Ratios are medians of paired per-run ratios; dividing the latency medians can
give a different value. Small-update medians favor one worker, while the bulk
result and variable tails prevent a blanket default-change claim. These timings
do not measure HTTP throughput or the full application startup/rebuild path.

Production defaults remain unchanged. `WriterOptions` provides an explicit,
bounded constructor configuration for diagnostic wiring; normal constructors
retain Tantivy's existing worker-selection algorithm and 50-MB budget. The
native load fixture can request one or three workers through
`MW_NATIVE_LOAD_INDEX_WORKERS`. Invalid worker/memory combinations fail before
seeding, and reports include the exact configuration passed to the writer.
Commit, reload, merge-policy and read-after-write behavior are unchanged.

The next test is a controlled whole-application comparison using one frozen
release image and the two explicit worker settings. A faster component benchmark
does not establish an application gain. The 40k-RPS target remains unverified.

Source, image and executable hashes, raw runs and aggregate results are retained
in `.orchestration/reports/perf-investigation/index-worker-probe-summary.json`.
