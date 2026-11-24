# Bulk mailbox operation benchmark

This suite measures operations on large mailboxes: loading (initial sync) of
10,000 and 100,000 messages, opening such a mailbox, and deleting, moving,
flagging, emptying, folder-deleting, marking-read and importing 1,000 and
10,000 messages at a time. No other fixture covers this. The native load
fixture seeds 2 to 100 messages per user ([native-http-fixtures.md](native-http-fixtures.md)),
and its large-mailbox profile is described at the end of this page.

The suite builds benchmarks; it does not state results. Numbers come only from
records produced under controlled conditions (a dedicated host, a release build,
no concurrent jobs), following [layered-capacity.md](layered-capacity.md).

## What runs

The Rust test is `crates/mw-server/tests/t60_bulk_ops.rs` (feature
`load-test-fixtures`). It is ignored unless `MW_RUN_BULK_OPS_BENCH=1`. One run
follows these steps:

1. Creates a private temporary directory. The test re-runs itself as a worker
   process with:
   - forwarded-address trust for `127.0.0.1` only;
   - `MW_DB_ENCRYPTION=required` and a fresh random `MW_DB_KEY` (SQLite);
   - its own upload directory.
2. Opens the database. SQLite is SQLCipher, and the test checks the file header
   is not plaintext. PostgreSQL gets a new `mw_bulk_ops_*` schema in an
   acknowledged local `mw_fixture*` database, which is dropped at the end.
3. Installs the enforced security profile (`MW_BULK_SECURITY_PROFILE=enforced`,
   the default) and the loaded `access.protocols` policy.
4. Seeds the accounts that the selected scenarios need. Rows go in through
   `Store::upsert_message_bodies_batch`, the native fixture's path, with real
   MIME and envelopes.
5. Builds the production router with `build_app_with_fixture_engine`, a sealed
   on-disk search index and the selected index-publication options. It starts
   index publication and waits until startup indexing has covered every seeded
   message.
6. Runs the scenarios in a fixed order. Each emits one
   `BULK_OPS_RECORD {json}` line, also appended to `MW_BULK_REPORT` when that is
   set.

The fake upstream (`tests/common/bulk_ops_backend.rs`):
- keeps folders and UIDs;
- regenerates corpus messages from their seed instead of holding them in memory;
- answers the engine's real sync, fetch, STORE, MOVE (UIDPLUS), EXPUNGE, APPEND,
  folder delete and dated-import calls;
- counts every one of those calls.

It has no network latency. Upstream cost in these numbers is the engine's side
of each command only.

### Layers

| `MW_BULK_LAYER` | Requests go through | Includes | Excludes |
| --- | --- | --- | --- |
| `engine` | `Engine::handle_jmap`, plus the import route's two steps called directly | Engine, Store, change log, triggers, index publication | HTTP, cookies, CSRF, request protection, policy guards, JSON transport |
| `http` (default) | The production router over loopback HTTP, with a real session cookie, CSRF token and forwarded client address | Everything in `engine`, plus every HTTP layer and guard of the enforced profile | Network distance, TLS |

Use a separate run for each layer, because the scenarios change the data.

### Corpus

`tests/common/bulk_ops_corpus.rs` generates every message deterministically.
Every record carries this distribution in `inputs.sizeDistribution`.

| Share | Shape | Size |
| --- | --- | --- |
| 80% | Plain text | 600 to 2,599 body bytes |
| 15% | multipart/alternative | About 1.5 KiB of text and 4 KiB of HTML |
| 5% | multipart/mixed with a binary attachment | About 1 KiB of text and a 24 KiB attachment, base64-encoded |

Message properties:
- 97 distinct senders.
- Received times are one minute apart, so the newest message has the highest seed.
- Every fifth message contains the word `quarterly`, which is the many-hit text search.
- Planted search terms, each found nowhere else in the corpus:

  | Term | Where | Frequency |
  | --- | --- | --- |
  | `quarterly` | body | seed % 5 == 0 (20%) |
  | `zephyrine` (rare) | body | seed % 2000 == 7 |
  | `zephyrus` (shares the `zephyr` prefix) | body | seed % 1000 == 501 |
  | "amber lantern" as a phrase | body | seed % 50 == 3 |
  | `amber` alone | body | seed % 10 == 6 |
  | `lantern` alone | body | seed % 7 == 2 |
  | `urgent` | subject | seed % 40 == 11 |
  | Cc `audit@corpus.invalid` | header | seed % 25 == 0 |

- The `search` account also carries a flag pattern: 30% unread (seed % 10 < 3)
  and 2% flagged (seed % 50 == 0), in Store and upstream.

### Accounts

| Account | Contents | Used by |
| --- | --- | --- |
| `main` | INBOX with N messages | open-\*, flag-\*, move, move-back, mark-folder-read, import |
| `trash` | INBOX with N messages | trash-move, empty-trash |
| `destroy` | INBOX with N messages | destroy |
| `folder` | Folder `Bulk` with K messages | delete-folder |
| `ingest` | Nothing cached; N messages upstream | ingest |
| `search` | INBOX with N messages and Archive with N/10, with the flag pattern | search-\* |
| `reader` … `reader8` | INBOX with `MW_BULK_READER_MESSAGES` messages (default 1,000); `MW_BULK_READER_ACCOUNTS` of them (default 2) | Concurrent readers |

N is `MW_BULK_MAILBOX_MESSAGES` and K is `MW_BULK_OPERATION_MESSAGES`. An
account exists only when a selected scenario needs it.

## Scenarios and their records

Within a run the scenarios execute in this order. `MW_BULK_SCENARIOS` takes a
comma-separated list of the group names in the first column.

| Group | Records | What is sent | Verified end state |
| --- | --- | --- | --- |
| (always) | `seed` | Store batch seed, then startup index build | Index documents equal stored messages |
| `ingest` | `ingest` | `cache_mailbox_list` and `start_initial_sync`, which is what engine-mode sign-in does. The store, the index and the first page are polled every 100 ms. | N rows; N index documents; unread counters N in Store, `Mailbox/get` and upstream; one fetch per message; the first page is the newest message |
| `open` | `open-first-page`, `open-page-2`, `open-deep-page` (offset N/2), `open-unread`, `open-sort-from`, `open-text-search`, `open-changes` | The web client's list batch (`Email/query` plus `Email/get` with the client's list properties), cold then 5 warm. `open-changes` delivers one upstream arrival, refreshes the folder and calls `Email/changes`. | Totals, page length, order, first row, folder membership, warm equals cold; exactly one created id, which is the arrival |
| `search` | `search-*`; see [Search scenarios](#search-scenarios) | The client's search request | Against a brute-force oracle over the corpus |
| `flag` | `flag-seen`, `flag-unseen`, `flag-flagged`, `flag-unflagged` | `Email/set` update `keywords/$seen` or `keywords/$flagged` (true or null) plus `Mailbox/get`, the client's shape | Unread and flagged counts in Store, `Mailbox/get` and upstream; keywords of every message; states strictly increase; index equals Store |
| `move` | `move`, `move-back` | `Email/set` update `mailboxIds: {<dest>: true}` | Counts in both folders; every row in the destination; each destination UID holds the same Message-ID upstream; UIDs unique and contiguous; the destination lists every moved id; index equals Store |
| `mark-folder-read` | `mark-folder-read` | The client's `folderBulk.ts`: `Email/query notKeyword $seen` 100 at a time, then `Email/set` 100 per request | Every unread id listed once; unread counter 0 everywhere |
| `import` | `import` | `POST /api/import/mbox?mailboxId=` (engine layer: `store_upload` plus `Email/import`), split under `import.maxMessages` and 60 MiB | K new rows in `Import`; K upstream APPENDs; index equals Store |
| `trash` | `trash-move`, `empty-trash` | Move to Trash, as the client's Delete key does; then `Mailbox/empty` with `outcome: destroy`, 100 per call, after a `limit: 0` preview | Trash empty; K upstream expunges; every removed id is `notFound`; no removed id in Store or index |
| `destroy` | `destroy` | `Email/set destroy` with `mailwomanPermanent: true` (the client's permanent delete) | Same as empty-trash, for N−K remaining |
| `delete-folder` | `delete-folder` | `Mailbox/set destroy` with `onDestroyRemoveEmails: true` | Folder gone in Store and upstream; messages gone; index documents dropped by K |

The web client sends one unchunked `Email/set` for a selection
(apps/web/src/state/slices/mail.ts). The server advertises and enforces
`maxObjectsInSet` = 5,000 (crates/mw-engine/src/jmap_core.rs). So the default
batch (`MW_BULK_BATCH=max`) is the advertised `maxObjectsInSet`, and a
10,000-message operation is two requests. `MW_BULK_BATCH=100` reproduces the
client's folder-operation chunking. Every record reports the request count.
More than 5,000 ids in one request fails with `requestTooLarge`. That is
recorded as a refusal, not worked around.

## Search scenarios

The web client sends search as you type, 250 ms after the last keystroke
(apps/web/src/components/search/SearchBox.tsx:21). It puts the whole typed
string, operators included, into `filter.text` (state/slices/mail.ts). With the
default scope it adds `inMailbox` for the current folder and `notKeyword: $junk`.
With the "all folders" scope it uses `inMailboxOtherThan` Trash and Junk instead.
It asks for `limit: 50` with `calculateTotal: true`, plus the list `Email/get`.

The suite sends exactly these requests. Every expected result comes from a
brute-force oracle (`corpus::Fields`). The oracle:
- re-tokenises each generated message the way the index's default analyzer
  does (split on non-alphanumeric characters, lowercase, drop tokens over 40
  bytes), over from, to, cc, subject, body and filename;
- applies date bounds as the index does (`after:` inclusive, `before:`
  exclusive) and size bounds as `>`/`<`.

Every record checks:
- the total against the oracle;
- the page length;
- that every row matches;
- for date order, the exact page of ids.

A mismatch is FAILED.

| Record | Query (`filter.text` unless noted) | Measures |
| --- | --- | --- |
| `search-rare-term`, `-common-term`, `-phrase`, `-and-terms`, `-prefix` | `zephyrine`, `quarterly`, `"amber lantern"`, `amber lantern`, `zephyr*` | Selectivity. The phrase and the AND differ by construction. |
| `search-from`, `-cc`, `-subject`, `-has-attachment`, `-filename`, `-date-range`, `-larger`, `-smaller`, `-is-unread`, `-is-flagged` | The client's operators (`from:sender07`, `cc:audit`, `subject:urgent`, `has:attachment`, `filename:report`, `after:… before:…`, `larger:20k`, `smaller:1500`, `is:unread`, `is:flagged`) | Field filters alone |
| `search-unread-and-text`, `-from-and-attachment`, `-text-and-date`, `-subject-and-flagged` | Combinations | Field filters combined with text |
| `search-all-folders`, `-saved-search-folder` | `quarterly` without a folder (Trash and Junk excluded); a saved search created through `PUT /api/account/saved-searches`, queried as `inMailbox=<id>` | Scope |
| `search-page-2`, `-page-5`, `-deep-page`, `-page-2-no-total`, `-sort-from`, `-sort-size`, `-size-sort-folder` | `quarterly` at positions 50, 200 and half the hits; `calculateTotal: false`; sort by sender or size | Sorting and paging. There is no relevance sort: the engine supports receivedAt, sentAt, importance, size, from and subject. |
| `search-unread-count`, `-flagged-page` | The client's unread count (`notKeyword $seen`, limit 1, workspaceMail.ts); `is:flagged` | The planner's endpoint groups |
| `search-as-you-type` | `q`, `qu`, … `quarterly`, one request per keystroke | The worst case: the user pauses at least 250 ms after every key. Per-keystroke latency, and how many responses took longer than the debounce. |
| `search-snippets` | `SearchSnippet/get` for the 50 ids of the common-term page | The engine implements it; the web client never calls it |
| `search-after-reader-reload` | First common-term query after one committed flag write | Cold reader after reload, against warm |
| `search-fresh-after-arrival`, `-mark-read`, `-move-1000` | After one arrival with the rare term, one mark-read, or a move of up to 1,000 matches to Archive | What the first query returns (`firstQueryWasCurrent`), when it becomes correct (`correctAfterChangeMs`), read-fence waits. A concurrent reader runs during the change. |
| `search-index-rebuild` | `rebuild_search_index_all` | Rebuild time, documents per second, index size before and after, then a verified query |

The `seed` record adds `searchIndexOpenMs`: Engine construction with the sealed
index at start-up.

Every query record carries the following. Each counter is read around one
request with no other request running, but the counters are process-wide.

| Field | Meaning |
| --- | --- |
| `hitsMatched` | Oracle count |
| `cold.hitsMaterialisedPerRequest` | Stored hits the ordered search read before paging |
| `storedDocumentsDecodedPerRequest` | Hits plus document fetches plus list-projection reads |
| `sealedIndexBytesReadPerRequest` | Bytes decrypted from sealed index files (mw-search timing counters, enabled for this suite). It is normally 0 per query, because files are read when the reader opens. |
| `processCpuMsPerRequest` | Process CPU, all threads |
| `executionContext` | The search runs inline in the async request task (no `spawn_blocking`), from reading crates/mw-engine/src/jmap.rs |

Not covered:
- Unified search across linked accounts. It needs linked-account setup, and the
  client sends one query per account (apps/web/src/api/workspaceMail.ts:61-78).
- Highlighting in the client. It does not request snippets.

## What each number means

| Field | Meaning | Does not mean |
| --- | --- | --- |
| `elapsedMs`, `messagesPerSecond` | From the first bulk request to the last response, divided into the K messages. In journal mode the index may still be catching up. | Capacity under concurrency or a sustained rate |
| `requests.latencyMs` | Nearest-rank p50/p95/p99/max per HTTP (or engine) request. Over HTTP this includes loopback transport. | Server CPU time |
| `listCorrectAfterLastResponseMs` | Time after the last response until the probe queries return the expected totals. The probes use the search path, as a list view does (for example `notKeyword $seen` for unread). Polled every 50 ms, limit 120 s. | Push latency |
| `indexPublicationIdleAfterLastResponseMs` | Time until index publication has no pending or queued jobs | Per-account visibility; it is process-wide |
| `counters.statements` | Store statement counts across the whole process during the operation, including background work. `perMessage` divides by K. | Statements of this request alone, or SQL execution time |
| `counters.commits.index` | Tantivy commits during the operation | Store commits. The Store does not count those separately, so the field is `null`. |
| `counters.searchIndex.documentsWritten`, `bodyBytesTokenised` | Index documents fully rewritten, and the body and attachment text bytes in them. A document write tokenises every field again. | Bytes written to disk |
| `counters.searchIndex.hitsMaterialised` | Stored hits read by ordered searches before paging (at most 100,000 per search) | Rows the SQL list path scanned. That is not observable here. |
| `counters.publication` | Index publication jobs submitted, completed or failed, batches, admission refusals and read-fence waits | Per-account work |
| `counters.upstream` | Upstream commands: STORE, MOVE, EXPUNGE, APPEND, FETCH, folder DELETE, and the messages they named | Commands a real IMAP server would need; the fake has no round trips |
| `changeRows` | `Email` and `Mailbox` change-log rows written since the operation started | Rows pruned later |
| `concurrentReader` | A second account reads its inbox first page (plus `/api/me` over HTTP) at `MW_BULK_READER_RATE` per second. Windows: 3 s before, during the bulk requests, and 3 s after. | The effect of many concurrent users |
| `memory.sampledPeakResidentBytes` | Highest resident size sampled every 200 ms during the scenario, starting from the current size. On Linux, `processLifetimePeakResidentBytes` is VmHWM. | Memory owned by this scenario alone |
| `disk` | Database, WAL, shm and search-index sizes after the scenario. PostgreSQL reports the schema's table bytes, and its WAL is `null` because it is cluster-wide. | Space a VACUUM or checkpoint would reclaim |
| `coldMs` (open-\*) | First request of that shape in the process after startup. The OS page cache is not dropped. | A cold-disk read |
| Ingest `timeTo*` | From `cache_mailbox_list` to: first rows; first usable page (a full first page answered); all rows; initial sync ready; index documents equal N; and the text search answering the full count through the read fence | Behaviour with a real network upstream |

Statuses:
- **OK**: every end-state check passed.
- **FAILED**: a check failed. The record carries no numbers, only the checks.
- **REFUSED**: a server-side limit stopped the operation. The record has the
  stage, for example "after 5,000 of 10,000", and the response (HTTP status,
  JMAP error, or partial acceptance), with `partialResults`.
- **INCOMPLETE**: `MW_BULK_SCENARIO_DEADLINE_SECONDS` expired.

REFUSED and INCOMPLETE are results. The limit that stopped the run is in the
record. Limits are never raised to make a scenario pass.

Every record declares its inputs:
- source revision, and whether the tree was dirty;
- build profile, backend, and the encryption proof;
- the configuration (sizes, layer, batch, reader);
- the corpus distribution;
- limits in force: the JMAP core capability as served, `Mailbox/empty` batch,
  `import.maxMessages`, the import body limit, index publication options and
  request-protection budgets;
- the t34 mode witness, plus `MW_SQLITE_WRITER` when set;
- the enforced-policy preflight;
- hardware fields supplied by the runner.

## Running

Use a host with no other jobs and a release build:

```sh
# SQLite, HTTP layer, 10,000-message mailboxes, 1,000-message operations
node scripts/perf/bulk-ops/run.mjs --backend sqlite --layer http --mailbox 10000 --operation 1000 --release

# The full matrix (both backends, both layers, 10k/100k x 1k/10k); --plan prints it without running
node scripts/perf/bulk-ops/run.mjs --matrix --release --plan

# PostgreSQL: the DSN stays in the environment
MW_LOAD_FIXTURE_PG_DSN=postgres://user:pass@127.0.0.1:5432/mw_fixture node scripts/perf/bulk-ops/run.mjs --backend postgres --release

# Markdown tables from any record files or captured logs
node scripts/perf/bulk-ops/report.mjs .orchestration/reports/bulk-ops/<run>/*.jsonl
```

Runner options:

| Option | Values |
| --- | --- |
| `--scenarios` | Comma-separated group names, or `all` |
| `--batch` | `max` or 1 to 5,000 |
| `--on-limit` | `stop`, or `retry-after` (honours Retry-After and counts it) |
| `--reader` | `on` or `off` |
| `--reader-rate` | Reads per second, per reader account |
| `--reader-accounts` | Concurrent reader accounts, 1 to 8 (default 2) |
| `--deadline` | Seconds per scenario |
| `--ready` | Seconds allowed for the startup index |
| `--ack` | `commit` or `journal`: index publication acknowledgement |
| `--accounting` | `synchronous` or `write-behind`: request-protection accounting |
| `--guard` | `off` or `bounded`: guard snapshot |
| `--sqlite-writer` | Passed as `MW_SQLITE_WRITER` when that gate exists |
| `--hardware` | A JSON file merged into the hardware fields |
| `--keep-dir` | Keep the temporary database |
| `--out` | Output directory |

The runner records:
- the commit (`git rev-parse HEAD`) and whether the tree is dirty;
- CPU model and count, memory and OS, plus anything in `--hardware`.

It writes `<label>.log`, `<label>.jsonl` and `report.md`.

The underlying command (what the runner executes):

```sh
MW_RUN_BULK_OPS_BENCH=1 MW_BULK_MAILBOX_MESSAGES=10000 MW_BULK_OPERATION_MESSAGES=1000 \
  cargo test --locked --release -p mw-server --features load-test-fixtures --test t60_bulk_ops \
  bulk_mailbox_operations_benchmark -- --exact --ignored --nocapture --test-threads=1
```

Windows debug builds need `RUST_MIN_STACK=67108864`; the runner sets it.

### Duration

These durations have not been measured on a controlled host. Nothing here
predicts them. A run is bounded as follows:
- seed time, which is reported in the `seed` record;
- the startup index build, at most `MW_BULK_READY_SECONDS` (default 7,200 s);
- for each scenario, at most `MW_BULK_SCENARIO_DEADLINE_SECONDS` (default 3,600 s),
  plus up to 120 s of visibility polling and 6 s of reader windows.

An earlier native-fixture run waited 960 s for a 20,000-document startup index
(10,000 users with 2 messages each). The startup index build is therefore its
own result in the `seed` record, not setup noise.

The suite was checked with a debug smoke at 200 messages per mailbox and 100
per operation, on a shared development host:
- SQLite, engine layer: 21 of 21 records OK.
- SQLite, HTTP layer: 21 of 21 OK, after one fix to the Message-ID comparison
  of the move check.
- PostgreSQL 16, HTTP layer: 21 of 21 OK.

Each debug smoke took about ten minutes.

After the search group was added:
- Search group only, 2,000 messages per mailbox, 100 per operation:
  - SQLite, HTTP layer: 37 of 38 OK. The saved-search PUT lacked the storage
    authority header the client sends; it now sends it.
  - PostgreSQL 16, HTTP layer: 38 of 38 OK.
- Whole suite, 300 messages per mailbox and 100 per operation (the corpus now
  has planted terms and two reader accounts), SQLite, engine layer: 58 of 58 OK.

These smokes prove the suite runs and verifies; they say nothing about speed.

The smoke also shows structural counts, which do not depend on the host:
- a flag change rewrites one full index document per message, body included
  (`documentsWrittenPerMessage` 1, body bytes about equal to the message text);
- ingest and import publish one index job, and one index commit, per message;
- a move writes one Email and two Mailbox change rows per message;
- the client's unread count (`limit: 1`) reads one stored hit per unread
  message (`search-unread-count`: hits materialised equal the unread count);
- every filtered or custom-sorted page reads every matching stored hit before
  paging (hits materialised equal `hitsMatched`, not the 50 returned).

## Known limits found by reading the code

These are what the benchmark is expected to expose. They are listed so a
REFUSED or slow record can be traced; none of them has been measured here.

- Moves send one upstream MOVE per message (`move_email_inner`), and flag
  changes send one STORE per message (`update_email`). Each move records three
  change rows.
- `Email/set destroy` reads and deletes one message at a time
  (`destroy_emails`, `remove_emails_locally`). The index publication is one
  ticket for the request.
- Filtered and custom-sorted pages (Unread, sort by sender, text search) use the
  search path. It reads every stored hit, up to 100,000 (`MAX_HITS`), before
  paging (mw-search `search_hits_ordered`). The `hitsMaterialised` counter shows
  this per request.
- Initial sync ingests one message at a time, 64 per upstream fetch
  (`sync_one`). The engine gives up an initial sync attempt after 1,800 s and
  retries twice. Retries appear in `initialSyncRetries`.
- Index publication admission waits at most 30 s. Its queues hold 128 jobs,
  32 per account in journal mode, and 16 MiB. A refusal surfaces as a
  `serverFail` or partial acceptance and is recorded as REFUSED.
- Under the enforced profile a signed-in account has a budget of 1,800 requests
  per 60 s. The client's mark-folder-read on 100,000 unread messages needs about
  2,000 requests.

## Reading the "before" numbers

These records are the baseline for planned fixes. Each concern is one field:

| Concern | Record and field |
| --- | --- |
| Upstream commands per message | `counters.upstream` (`store`, `move`, `expunge`) divided by `messages`, on flag-\*, move, destroy |
| Change rows per moved message | `changeRows.email` and `changeRows.mailbox` divided by `messages`, on move |
| Store statements per destroyed message | `counters.statements.perMessage` and `writePerMessage`, on destroy and empty-trash. Store commits are not counted separately (`null`). |
| Stored documents loaded per page | `cold.storedDocumentsDecodedPerRequest` and `hitsMaterialisedPerRequest`, on search-\* and open-unread or open-sort-from (`coldCounters.searchIndex.hitsMaterialised`) |
| Messages per ingest write | `counters.commits.indexPerMessage` and `counters.publication.submittedJobs` divided by `messages`, on ingest and import |
| Unread count enumerating every unread message | `search-unread-count`: `hitsMaterialisedPerRequest` against `hitsMatched`; the mailbox row already holds the count |
| Flag change or move re-tokenising the body | `counters.searchIndex.documentsWrittenPerMessage` and `bodyBytesTokenised`, on flag-\* and move |

## Large-mailbox profile of the native load fixture

`MW_NATIVE_LOAD_PROFILE=large-mailbox`, with `MW_NATIVE_LOAD_WORKLOAD=representative`
and `MW_NATIVE_LOAD_PRINCIPALS=1000`, uses the hot-path planner's shape:

| Users | Messages each |
| --- | --- |
| 2 | 100,000 |
| 8 | 20,000 |
| 990 | 50 |

That is 409,500 messages, inside the 500,000 cap. Every user has:
- 30% unread (UID % 10 < 3) and 2% flagged (UID % 50 == 0), in the Store and in
  the fake upstream;
- small bodies with planted terms: `quarterly` on every fifth message, and
  `zephyrine` where UID % 2000 == 7.

Seeding goes through the batch API. `MW_NATIVE_LOAD_MESSAGES` must stay unset.

`MW_NATIVE_LOAD_LARGE_MAILBOX_SCALE=50` gives a smoke-only shape for checking
the driver: 2 x 2,000, 8 x 400 and 90 x 50 messages, 100 users (set
`MW_NATIVE_LOAD_PRINCIPALS=100`). The handoff records `smokeScale: true`. Never
report it as the profile.
The default profile keeps its bounds: 2 to 100 messages per user and 500,000 in
total.

Each principal in the handoff carries:
- `cohort` (`big-100000`, `big-20000` or `ordinary`);
- `messageCount`, `unreadCount`, `flaggedCount` and `plantedQuarterly`.

`workloadProfile` lists the cohorts.

The representative generator refuses more than 100 seeded ids per principal
(`scripts/perf/fixture-requests.mjs`). This profile therefore has its own
driver, selected with `MW_NATIVE_LOAD_DRIVER=large-mailbox`:
`scripts/perf/bulk-ops/large-mailbox-driver.mjs`.

```sh
MW_RUN_NATIVE_HTTP_LOAD_FIXTURE=1 MW_NATIVE_LOAD_PROFILE=large-mailbox MW_NATIVE_LOAD_WORKLOAD=representative \
MW_NATIVE_LOAD_PRINCIPALS=1000 MW_NATIVE_LOAD_POLICY=loaded MW_NATIVE_LOAD_SECURITY_PROFILE=enforced \
MW_NATIVE_LOAD_DRIVER=large-mailbox MW_NATIVE_LOAD_READY_SECONDS=1800 MW_LOAD_FIXTURE_DEADLINE_SECONDS=1800 \
  cargo test --locked --release -p mw-server --features load-test-fixtures --test native_load_fixture \
  native_multi_principal_http_load_fixture -- --exact --ignored --nocapture --test-threads=1
```

The driver runs four phases. Durations and rates come from `MW_LARGE_DRIVER_*`:
phase seconds (default 60), ordinary rate (default 50 requests per second),
isolated requests (default 5), heavy loops per account (default 1).

1. **isolated**: each heavy group alone against one 100,000-message and one
   20,000-message account, with fixture diagnostics before and after. It
   reports per request:
   - latency;
   - `hitsEnumeratedPerRequest`;
   - `storedDocumentsLoadedPerRequest`;
   - `cpuMsPerRequest`.

   The groups are unread page, flagged page, size-sort page, text search page 1
   and page 5, the client's unread count, and the list batch.
2. **baseline**: ordinary users only. Identity (`/api/me`) and the list batch
   alternate, open loop, at the fixed rate. Latency is measured from the
   scheduled send time.
3. **heavy**: the same ordinary traffic while every big account runs the heavy
   groups in a closed loop. A 429 from request protection counts as throttled
   (reported under `<group>:throttled`), not as a wrong answer.
4. **recovery**: ordinary users only.

`headline.ordinaryIdentityP99Ms` and `headline.ordinaryListP99Ms` compare the
three ordinary windows. Every response is validated against the seeded counts.
Any mismatch makes the report FAILED and the fixture fails.

## Product finding: selections over 5,000 messages

Nothing in the web client keeps a selection under the server's
`maxObjectsInSet` (5,000):
- Infinite scroll keeps adding pages (apps/web/src/state/slices/mail.ts:2006),
  and select-all takes every loaded row (mail.ts:1106).
- Move, archive, spam and delete (move to Trash) put every id in one `Email/set`
  (`relocateWithUndo`, mail.ts:1502-1516). The client never reads
  `maxObjectsInSet`.

What happens then:
- **Move and delete**: the server answers with the method error
  `requestTooLarge` (crates/mw-engine/src/jmap.rs:227). The client marks every
  id refused and puts the rows back (mail.ts:1050-1056, 1531-1536). The toast
  reads "The server refused the move for {count} messages: requestTooLarge"
  (`mail-toast-update-refused-reason`, locales/en/mail.ftl:1387). It gives no
  hint to select fewer.
- **Read/flag changes**: these are undone with "The server refused the change
  for N messages", which names no reason (mail.ts:1186, mail.ftl:426), or
  silently when the change is quiet.
- **Permanent delete**: also sends one unchunked request (apps/web/src/api/jmap.ts:459).

This suite does not change any of that. With `MW_BULK_BATCH=max` it sends
5,000 per request, as the advertised limit allows.

## goal-65-35 seed of the native load fixture

`MW_NATIVE_LOAD_WORKLOAD=goal-65-35` seeds exactly as `representative` does,
plus the folders and terms the goal mix's write and search shapes need:
- Two extra empty folders: `Trash` (role trash) and `Drafts` (role drafts).
  Each principal in the handoff carries `trashId` and `draftsId` beside
  `inboxId` and `archiveId`.
- Planted terms in the bodies:
  - `quarterly` where UID % 5 == 0;
  - `zephyrine` where UID % 2000 == 7.

  UID 1, the sample message, never has a planted term. Per principal the
  handoff carries `plantedQuarterly` (10 at 50 messages) and `plantedZephyrine`
  (1 at 50 messages).
- Flags: `messageIds[2]` (UID 3) starts read and `messageIds[3]` (UID 4) starts
  flagged. The mark-unread and unflag shapes therefore change state on their
  first pass; every other message is unread and unflagged.

The fake upstream accepts the extra operations only where they belong. It
appends only to Drafts and expunges only from Drafts or Trash. Everything else
keeps the representative profile's behaviour.

The fixture diagnostics carry `upstreamCommands`: process-wide counts of
fetch, store, move, expunge, append and list commands, with message counts.

`MW_NATIVE_LOAD_DRIVER=goal-mix-smoke` runs
`scripts/perf/bulk-ops/goal-mix-smoke.mjs`. It makes two validated passes of
every goal-mix write shape and of candidate shapes for delete-to-trash, draft
save and search on one principal. It is a smoke, not a measurement.
