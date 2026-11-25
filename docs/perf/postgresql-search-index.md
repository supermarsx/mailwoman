---
title: PostgreSQL search index cost (M5, t62-n16)
---
# PostgreSQL search index cost (M5)

What the engine's search index costs on PostgreSQL, measured 2026-11-25 for the
memory audit (t63, finding F15). Today a PostgreSQL DSN without
`MW_SEARCH_DIR` keeps the index in RAM (`crates/mw-server/src/engine_mode.rs`,
`search_index_place`) and rebuilds it from the store at every start.

## Set-up

- PostgreSQL 16 in a container limited to 4 CPUs and 8 GiB
  (`shared_buffers` 1 GB). The measuring process is a release build of
  `crates/mw-engine/tests/m5_postgres_index_rebuild.rs`, running natively on
  the Windows host (not in the Linux image); the sealed on-disk index is on
  NTFS.
- Data: 100,000 messages in 50 accounts, one folder each, seeded through the
  store's own write path (`put_body`, `upsert_message`, all triggers on). Each
  message has about 2 KB of body text from a 6,000-word synthetic vocabulary
  with a skewed word choice, a six-word subject, and one of 997 senders.
- Run: `MW_TEST_PG=<dsn> cargo test --release -p mw-engine --test
  m5_postgres_index_rebuild -- --ignored --nocapture` (`M5_MESSAGES` changes
  the count; `M5_SKIP_RAM=1` skips the RAM build on a re-run).

## Results

| Measure | In RAM (PostgreSQL default today) | Sealed directory on disk |
|---|---|---|
| Full build from the store, 100,000 messages | 305 s (328 documents/s) | 641 s and 693 s in two runs (156 and 144 documents/s) |
| Index size (sum of segment bytes) | 212.0 MB = 2,120 B per message | 211.9 MB = 2,119 B per message |
| Process memory before → after the build | 31 → 286 MB (+255 MB) | 15 → 278 MB (Windows working set; includes mapped segment pages, which the OS can drop) |
| Start-up with an existing index | not possible: rebuilt every start (305 s here) | open 1.06 s + store/index count check 0.37 s, no rebuild |

Seed rate through the store: 100,000 messages in 152–180 s (556–657
messages/s with 50 concurrent writers).

A second data point: the t52 bulk run (PostgreSQL, 53,000 synthetic messages
with about 2.2 KB bodies, Linux container, 4 CPUs) built a sealed index at
290–372 documents/s and 50.2 MiB, about 1.0 KB per message
(`.orchestration/reports/perf-investigation/t52-writegate-bulk-20261124`,
`pg-10k-10k/report.md`).

## Extrapolation

Linear in the number of messages (postings, stored documents and fast fields
all grow per document; merges hold old and new segments at once for the
segments being merged, up to about 2× of those transiently):

| Messages | Index size | Full build at 145–370 documents/s |
|---|---|---|
| 100,000 (measured) | 0.1–0.2 GB | 4.5–11.5 min |
| 1 million | 1–2.1 GB | 45 min – 1.9 h |
| 10 million | 10–21 GB per process | 7.5–19 h |

Stated error: size about ±2× (real mail varies from a few hundred bytes to
tens of kilobytes of text per message, and a real vocabulary is larger than
6,000 words, which grows the term dictionary but not the postings); build rate
±30% for one process (PostgreSQL CPU, the store read path, the host). The rate
did not improve with more messages, so a 10-million-message rebuild is a matter
of hours, during which the index is incomplete.

## What this means for the default

- In RAM, every PostgreSQL process holds its own full copy (10–21 GB at
  10 million messages) and rebuilds it at every start.
- During that rebuild the index is partial. In the default publication mode
  (`commit`) the read fence does not wait for it (`Publisher::read_fence`
  returns at once unless the journal mode is on), so searches during those
  minutes or hours return results from a partial index without saying so.
- An existing on-disk index opens in about a second. What it lacks on
  PostgreSQL is a check that it is still current: the store is shared, other
  processes may have written while this one was down, and the start-up check
  compares document and message counts only (`Engine::reconcile_search_index`),
  which misses a flag change, a move or a replaced body.
