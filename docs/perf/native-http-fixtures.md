# Native mail HTTP load fixtures

The opt-in `load-test-fixtures` Cargo feature adds an engine factory to the existing server builder. Ordinary builds do not expose that entry point. The factory supplies deterministic account runtimes; the HTTP router, authentication, verified principals, account/domain checks, settings and admission policies remain the production paths. There is no fixture login endpoint or HTTP authorization bypass.

`native_multi_principal_http_load_fixture` creates a fresh disposable database, 1, 100 or 10,000 distinct durable accounts, verified principals and sessions, and an Inbox with two messages and cached bodies per account. Seeding uses public Store methods, runs outside the measured interval and reports progress every 1,000 accounts. Seeding is intentionally bounded and serial; it does not represent login throughput. Passwords and session cookies are generated only for this isolated fixture.

```sh
MW_RUN_NATIVE_HTTP_LOAD_FIXTURE=1 MW_NATIVE_LOAD_PRINCIPALS=100 MW_LOAD_FIXTURE_WORKERS=4 cargo test --release -p mw-server --features load-test-fixtures --test native_load_fixture native_multi_principal_http_load_fixture -- --ignored --nocapture
```

The default is **100 independent users and simulated clients**, in debug and release builds. Use `MW_NATIVE_LOAD_PRINCIPALS=1`, `100` or `10000` to compare cardinality; the one-user profile is an explicit diagnostic. `MW_LOAD_REPORT_DIR` chooses the report directory. The Node load generator uses the same bounded campaign controls as the proxy fixture. `MW_LOAD_FIXTURE_DEADLINE_SECONDS` bounds the driver to 10–1,800 seconds; account seeding happens first. The listener binds only to loopback. The private handoff has owner-only permissions on Unix and is removed on normal completion or panic; never publish or commit it. Reports must omit credentials and per-user identifiers.

The debug runtime gives each Tokio worker an explicit 8 MiB stack budget for
large unoptimized native futures; release retains the 2 MiB worker default.
The report records `runtimeWorkerStackBytes`. This is a debug diagnostic setting,
not evidence of release memory requirements or an application startup change.

## Startup readiness before measurement

The fixture does not publish its HTTP/WebSocket handoff until the production
engine's startup search-index rebuild has caught up. It observes the real index
document count, then verifies `Engine::search_index_status`: no rebuild running,
the document count matching the indexable stored messages, and no documents for
zero-access accounts. For these seeded fixtures the normal expected count is
200 documents for 100 accounts or 20,000 for 10,000 accounts. Explicitly enabled
zero-access accounts are excluded from that expectation and reported separately.
An active indexing restriction blocks the warmed campaign rather than silently
disabling the restriction or claiming the missing documents are ready.

`MW_NATIVE_LOAD_READY_SECONDS` bounds this readiness phase to 10–1,800 seconds
(default 300), independently of seeding and the campaign's driver deadline.
Progress reports observed/expected documents and elapsed time every five seconds.
The gate polls the cheap engine document-count API; it performs the full,
potentially expensive account/store consistency check only once the counts match.
Read errors, stopped incomplete rebuilds and deadline expiry prevent a campaign
from starting. No fixed sleep substitutes for an observed ready state.

The handoff and HTTP/combined WebSocket reports contain `startupReadiness` with
verified state, startup/index-wait durations, seeded/expected/indexed counts,
zero-access exclusions and rebuild progress. Startup duration starts just before
the normal server builder, after seeding. These checks do not disable production
background workers and do not claim that every periodic job is idle. Older
reports lacking this proof remain startup-confounded; do not relabel them as
warmed results merely because their traffic generator had a warmup interval.

Each seeded session has a distinct, stable simulated client address from the
benchmark-only `198.18.0.0/15` range. The test starts an isolated copy of its test
binary with `MW_FORWARDED_MODE=xff` and `MW_TRUSTED_PROXIES=127.0.0.1/32` before
the runtime starts. Only that disposable fixture process trusts the loopback
generator to represent these clients; production proxy configuration is not
changed, and per-user/per-source admission policies remain active. The private
handoff explicitly declares this simulation and contains the client address
bound to each principal. The driver sends forwarded addresses only for this
declared fixture mode, and reports actual participating/accepted client counts.
Endpoint operations stay interleaved, with early arrivals spread across the user
pool. Every endpoint visits every principal over a complete cycle. The 100-user
and 10,000-user profiles reach their full population in the first 100 or 10,000
dispatched requests respectively, so a short overloaded phase does not reserve
all its traffic for the first few users. Dropped arrivals still do not count as
dispatched or accepted users.

Before measurement, real `Email/query` requests authenticate the first and last
principals from their assigned addresses. The production health admission guard
also rejects their external benchmark addresses while accepting the actual
unforwarded loopback peer, checking that the server honors the configured proxy
model rather than assigning every request to loopback. A unit test checks all
10,000 distinct addresses through the shared production resolver and verifies
that an untrusted peer cannot forge them. This simulates distinct clients behind
a trusted proxy; it does not create additional physical generator machines or
measure their network/TLS overhead. External generator placement and the number
of actual generator processes must still be recorded separately.

SQLite is the default. PostgreSQL requires `MW_LOAD_FIXTURE_BACKEND=postgres`, `MW_LOAD_FIXTURE_PG_ACK_SCHEMA=1` and `MW_LOAD_FIXTURE_PG_DSN` pointing to an acknowledged local database whose name begins `mw_fixture`. Each run creates its own UUID schema; existing schemas/databases are never dropped. Schema-selection DSN overrides are rejected. Both the schema-creation pool and the seeding pool close before the measured server starts. Cleanup removes only that run's schema. Failed runs can leave disposable diagnostic data behind.

The fixture performs real `Email/query` requests for the first and last seeded account before measurement, so a successful `Core/echo` cannot conceal a disconnected engine runtime. The read-only backend supplies a stable mailbox list and no-change refresh observations. Sending, upstream protocol performance and real arrival/push behavior are outside this fixture. Default admission/security settings remain active and effective budgets are included in the handoff; rate limiting is a measured outcome, not a successful API response.

Ten thousand seeded users do not mean ten thousand concurrently open connections. A separate sustained WebSocket campaign is required to measure ten thousand live sockets, their memory, heartbeats and fanout. Likewise a measured request rate is evidence only for its recorded endpoint mix, concurrency, successful responses and resource limits. This fixture does not establish the target of 40,000 successful API requests per second.

## Loaded policy control

Set `MW_NATIVE_LOAD_POLICY=loaded` for production-policy throughput measurements.
The fixture stores a locked global `access.protocols=["imap"]` policy before
startup, retaining every admission and security budget. Before warmup it primes
the first and last real authenticated sessions, changes that policy to `[]` and
requires immediate HTTP 403 `feature_disabled` responses naming `access.protocols`
on both sessions. It then restores `["imap"]` and requires successful real mail
queries before handing control to the generator. Timed requests keep that loaded
policy. `policyEvidence` reports only aggregate control results. The default
`defaults` profile remains useful as a baseline but cannot satisfy the production
multi-user throughput target; debug and `fast` builds cannot satisfy it either.

Native drivers require verified startup-readiness metadata and scanner enforcement
at `/.env`; missing readiness proof or an unexpected scanner response fails the
campaign. This checks the configured production paths, without bypassing policy
evaluation or relaxing limits to improve a result.

## Separate representative read/write profile

Use `MW_NATIVE_LOAD_WORKLOAD=representative` together with
`MW_LOAD_FIXTURE_ENDPOINTS=representative`. `mixed` retains its five read-only
baseline request types. Representative mode defaults to 50 messages per account,
with `MW_NATIVE_LOAD_MESSAGES=2..100` and a total cap of 500,000 seeded messages.
It adds an Archive folder and a deterministic independent transport per account;
production HTTP authentication, policies, engine mutations, database changes and
search-index updates still run normally. Sending, SMTP delivery, upstream network
latency, new arrivals and durable transport failures remain outside this profile.

One message per account contains a default 4,096-byte text payload and a
65,536-byte decoded attachment. Set `MW_NATIVE_LOAD_BODY_BYTES` to 27..16,384 or
`MW_NATIVE_LOAD_ATTACHMENT_BYTES` to 4..262,144 to compare bounded payload sizes.
The other messages contain a 26-byte text payload including its line ending,
plus their headers. Reports record configured sample sizes, total seeded rows,
seeding duration and the actual observed startup/index-readiness durations.
At 10,000 accounts the default profile has 500,000 messages and 625 MiB of decoded
sample attachments alone. MIME encoding, cached raw copies, envelopes, transport
state and the search index add storage and memory; setup cost and resource limits
must be measured rather than inferred from the tiny baseline fixture.

The representative endpoint group mixes the baseline reads with two 25-message
query/header pages, an actual body/attachment-metadata read, a byte-validated
attachment download, mark-read/readback/restore/readback and
move/readback/restore/readback. The mutation sequences each use one real JMAP
batch: methods inside a batch do not inflate the reported HTTP request count.
Each result validates account binding, expected IDs, pagination, actual body and
download content, mutation acknowledgement and the restored state. Requests for
one principal are serialized in the generator so concurrent batches cannot
invalidate their own restoration checks. Waiting counts toward request latency
and the bounded outstanding limit; it cannot improve accepted throughput.

Body validation requires the exact configured ASCII payload and rejects shortened,
extra or truncated content. It accepts only the single trailing MIME boundary
line ending that the parser may preserve; that delimiter is excluded from the
configured payload length. Attachment validation checks every decoded byte.

If a dispatched stateful batch times out, disconnects or fails validation, its
server-side outcome is unknown even after the browser request has stopped.
The generator taints the campaign before releasing its principal queue, blocks
queued reuse, stops scheduling, drains outstanding client operations and retains
the failure report. It never retries the mutation or continues a later clean
stage against that state. The report records only aggregate tainted-principal,
unknown-outcome, blocked-queue and dispatch-attempt counts. Such a campaign exits
unsuccessfully and cannot satisfy any throughput target, including a clean stage
that preceded the uncertain mutation.

A production multi-user target also requires a mixed or representative endpoint
stage; an identity-only result cannot establish that target. These campaigns
measure the specified synthetic mix and never establish complete application
capacity or delivery correctness by themselves.
