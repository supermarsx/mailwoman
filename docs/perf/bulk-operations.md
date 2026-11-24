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

### Accounts

| Account | Contents | Used by |
| --- | --- | --- |
| `main` | INBOX with N messages | open-\*, flag-\*, move, move-back, mark-folder-read, import |
| `trash` | INBOX with N messages | trash-move, empty-trash |
| `destroy` | INBOX with N messages | destroy |
| `folder` | Folder `Bulk` with K messages | delete-folder |
| `ingest` | Nothing cached; N messages upstream | ingest |
| `reader` | INBOX with `MW_BULK_READER_MESSAGES` messages (default 1,000) | Concurrent reader |

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
| `--reader-rate` | Reads per second |
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

Each debug smoke took about ten minutes. That proves the suite runs and
verifies; it says nothing about speed.

The smoke also shows structural counts, which do not depend on the host:
- a flag change rewrites one full index document per message, body included
  (`documentsWrittenPerMessage` 1, body bytes about equal to the message text);
- ingest and import publish one index job, and one index commit, per message;
- a move writes one Email and two Mailbox change rows per message.

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

## Large-mailbox profile of the native load fixture

`MW_NATIVE_LOAD_PROFILE=large-mailbox`, with
`MW_NATIVE_LOAD_WORKLOAD=representative`, admits 1 or 4 principals
(`MW_NATIVE_LOAD_PRINCIPALS`) with 10,000 or 100,000 messages each
(`MW_NATIVE_LOAD_MESSAGES`), capped at 400,000 in total. The default profile
keeps its bounds: 2 to 100 messages per user and 500,000 in total.

The report records the profile in `workloadProfile.profile`. Seed time and
startup index wait are reported as before.

The representative HTTP generator still refuses principals with more than 100
seeded ids (`scripts/perf/fixture-requests.mjs`). Use this profile with
`MW_LOAD_FIXTURE_EXTERNAL_DRIVER=1` or with this suite until the generator
learns it.
