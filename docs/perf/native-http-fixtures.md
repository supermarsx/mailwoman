# Native mail HTTP load fixtures

For CPU diagnosis, the Dockerfile also provides an optional `fixture-profile`
target. It inherits the gated native release image, reuses its Cargo dependency
cache, and invokes `cargo rustc` with symbol stripping disabled only for a
separate `/out/request-load-fixture-profile` executable. The default command
continues to use `/out/request-load-fixture`; a SHA-256 check fails the build if
that normal release artifact changed during diagnostic linking. Both executable
hashes are recorded in `/out/fixture-executables.sha256`. This target accepts only
the normal native release profile. Check its actual compiler logs for artifact
reuse; Dockerfile syntax validation alone does not prove a successful build.
Use the symbol executable for diagnostic sampling, and the ordinary executable
for unprofiled throughput measurements. Profiling requires a separate bounded
fixture PID namespace, never host-wide access or production credentials.

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
Private handoffs have a 64 MiB ceiling across the publisher, HTTP controller and
WebSocket readers, sufficient for the supported 10,000-account / 500,000-ID
representative profile. Ordinary lifecycle controls retain their 64 KiB ceiling;
larger handoffs and linked/non-private files are refused before dispatch.
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

Representative requests use the actual default webmail property sets: all 36
preloaded list-header properties, including mentions, focus, authentication and
delivery/optional headers; the reader requests all 38 body/delivery/optional
properties with HTML/text body values, a 1,000,000-byte value limit and explicit
reader intent. The representative mode's baseline `Email/get` uses that same
list-header set. A source-contract test checks the shared fixture copies against
the UI arrays and default preload expansion without importing browser code.

The separate `baseline` workload keeps its five-field header request as a minimal
diagnostic. Reports record `workloadProfile.requestPropertySets`, naming both the
profile and its exact property/body-fetch choices. Earlier representative results
used five-field headers and a smaller reader projection; they measure that older
minimal request mix and must not be compared as full webmail workload results.

Body validation requires the exact configured ASCII payload and rejects shortened,
extra or truncated content. It accepts only the single trailing MIME boundary
line ending that the parser may preserve; that delimiter is excluded from the
configured payload length. Attachment validation checks every decoded byte.
It uses native buffer equality against a shared golden pattern, caching at most
four sample sizes (at most 1 MiB total), rather than allocating an attachment
buffer for every principal. Private per-principal lengths must agree with the
global workload profile.

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

List-field projections use exact-term Boolean queries for at most 32 distinct
message IDs, and retain the FST term-set query for larger chunks. A separate
release microprobe compared both forms with one and eight index segments:
32-ID projections improved in both fixtures, while 50–64-ID Boolean queries
regressed with eight segments. Account filtering, decoded ownership checks and
duplicate-document rejection remain the same. This bounded query result does
not establish an HTTP throughput gain; the representative campaign must measure
that separately.

Ingestion stores a versioned, bounded projection of the eleven supported
delivery/optional headers inside the sealed envelope. Current list requests use
that projection instead of loading the complete raw MIME BLOB. Legacy messages
fall back to raw headers and attempt a source-bound revision-checked repair;
oversized or invalid projections retain the fallback without truncating values.
The private projection is removed from API responses. Representative fixtures
use the same production encoder rather than constructing projection stamps.
Explicit requests containing only known list/preload metadata use a separate
sealed envelope capped at 128 KiB. This excludes decoded body values and raw
MIME, and has its own account/message/revision cache namespace. Ingestion and
envelope repairs update it atomically; source changes invalidate it, while
flags-only updates retain it and patch current flags from SQL. The additive
cache table starts empty on existing installations, with bounded lazy repair
instead of a full-mailbox startup sweep. Invalid or oversized projections retain
the full-envelope fallback without entering the lightweight cache namespace.
Reader intent, body-fetch flags, body/attachment properties, unknown extensions
and omitted property lists retain their full-envelope behavior. Account and
source checks still run before cache access. Neither optimization establishes a
new RPS measurement; verify the next frozen release against the same workload.
The SQL/bytecode checks prove that current metadata reads do not fetch full
envelope/body column values into the application. They do not prove zero SQLite
pager I/O: `blob_ref` and `cache_revision` occur after the full envelope in the
existing message-row layout, so SQLite may walk overflow pages to reach them.
Pointer maps can help on incremental-vacuum databases, but fragmentation or
older NONE-mode databases can require page reads and SQLCipher work. Measure
cold/warm pager behavior before attributing all remaining encryption costs or
claiming that every body-bearing database page has been avoided.

`MW_NATIVE_LOAD_INDEX_WORKERS` is an optional diagnostic fixture override. Leave
it unset for the ordinary Tantivy worker selection, or choose `1`, `2` or `3`
with the fixture's unchanged 50 MB writer budget. Other values fail validation
before database creation and message seeding. This setting is read only by the
ignored native load fixture, and does not change production configuration.
Reports include the requested/effective worker counts and exact memory budget;
before/after diagnostic snapshots also include the monotonic index commit-attempt
count. This counts calls to commit, including calls that subsequently fail.
Compare steady application throughput and latency with the same dataset and
resource limits: an isolated mutation microprobe does not settle the tradeoff
between bulk indexing, incremental changes and request handling.

The native fixture defaults to a sealed disk index, using the same derived Store
key and persistent Engine constructor path as SQLite deployments. Explicit
`MW_NATIVE_LOAD_INDEX_MODE=memory` selects a RAM-only diagnostic; `sealed` selects
the default. Invalid modes are rejected before database creation/seeding, and
index-open errors fail the fixture without a plaintext or RAM fallback.
Storage proof reports the constructor mode and an authenticated-ciphertext marker
on the actual index metadata after startup readiness. It exports no paths or
keys. This is a constructor/metadata proof, not an exhaustive runtime file audit.
Legacy reports without storage proof remain unreported; RAM and unknown storage
cannot qualify the new typical SQL capacity target. Earlier RAM comparisons
remain diagnostics. Private temporary storage may use a container overlay rather
than a deployment's named volume: matching the code path does not establish
production filesystem or whole-host capacity.

The latest completed RAM-index comparison used six alternating 60-second runs
with 100 authenticated users, 50 messages per user, the representative request
mix, 16 outstanding requests and two generator workers. The app was limited to
four CPUs and 4 GiB; the generator had separate two-CPU/1-GiB limits. On the same
release image (`9dd4c06c0d80a7ec506924688f0d16af6b3f1edd83c26c371d7ed00d02469781`),
three indexing workers reached a median 117.9 successful HTTP requests/second;
one worker reached 118.43. Both had median p95 latency of 260 ms. All runs had
zero request errors, throttles or timeouts and passed the loaded-policy,
cross-account isolation, linked-login revocation and scanner enforcement gates.
These differences do not demonstrate a meaningful throughput improvement, so
production worker defaults remain unchanged. The one-worker runs used less
resident memory (median approximately 275 MiB versus 354 MiB).

Those runs predate the sealed-index fixture default above and are RAM-index
diagnostics only. They also use one sender and no Sent messages or contacts.
They do not establish capacity for 10,000 users, 40,000 RPS, production storage,
or simultaneous WebSocket connections. A later encrypted-storage baseline
follows; neither series establishes full application capacity.

The subsequent encrypted-disk comparison used release source `7c0b4829`,
including the readable-header and lightweight-envelope changes. Six alternating
60-second runs used the same 100-user/50-message request mix, limits and
three/one/one/three/three/one worker order. Every run verified sealed index
storage, loaded policy enforcement, linked-login restrictions, cross-account
isolation and scanner blocking. All six had zero errors, throttles or timeouts.

| Index workers | Median successful HTTP RPS | Median p95 | Median p99 | Median RSS |
| --- | ---: | ---: | ---: | ---: |
| 3 | 35.6 | 1,060 ms | 1,180 ms | 330.8 MiB |
| 1 | 39.3 | 987 ms | 1,130 ms | 250.8 MiB |

The frozen image was
`caa4a604d65c26de173e1e7718cf91ac7ab45fe783727b3ddc3075ce87e07b4b`.
This establishes a small synthetic encrypted-storage baseline, not the
40,000-RPS/10,000-user target. The earlier RAM results are a different storage
configuration, so these numbers cannot attribute a before/after gain or
regression to the header optimization. Worker-count defaults remain unchanged;
the bulk-indexing tradeoff and broader workloads still need evaluation.
Source review identifies repeated whole-file reads/decryption when sealed index
readers reopen unchanged segment components during publication. Its contribution
has not yet been isolated by caller; neither file-cache shortcuts nor weaker
integrity/durability checks are justified by these measurements.

### Isolating reads from mutations

A subsequent diagnostic used that same immutable encrypted-storage image,
100 users, 50 messages each, three index workers, 16 outstanding requests,
and the same application/generator resource limits. Each phase lasted 20 seconds
with request-phase instrumentation enabled. All accounts participated; there
were no HTTP errors, throttles or timeouts, and the loaded-policy and isolation
checks passed.

| HTTP workload | Successful in-window RPS | Mean latency | p95 | Scoped SQL reads per HTTP |
| --- | ---: | ---: | ---: | ---: |
| Identity | 1,176.2 | 13.6 ms | 21 ms | 23.0 |
| Mixed reads and refresh | 287.1 | 55.6 ms | 80 ms | 47.6 |
| Basic headers | 290.9 | 54.9 ms | 68 ms | 63.0 |
| Query | 374.6 | 42.6 ms | 56 ms | 58.0 |

No index commits occurred in these phases. Compared with the mutation-heavy
representative mix, this identifies substantial contention introduced by
mutations; it is not an optimization gain or a server throughput ceiling.
At fixed closed-loop concurrency, throughput is constrained by request latency.
The generator used about 0.1 CPU core and showed no scheduler/capacity drops in
the earlier representative runs, while the app used about 2.3 of four cores.
Memory exhaustion or generator CPU saturation is not supported by those runs.

In the representative workload, `mark-read-and-restore` and `move-and-restore`
are each one HTTP request containing two mutations and two readbacks. The
default-worker first run averaged 915 ms and 1,014 ms respectively, compared
with 517 ms for the two-page query/header batch. Its 856 commit attempts exactly
match two commits for each of the 214 flag and 214 move requests. These counts
do not establish duplicate requests. Batching across requests must preserve
the intervening readbacks, ordering and account-erasure guarantees.

A successful user-CPU profile of the same release recorded 2,885 samples with
zero lost samples, wholly within the completed representative phase. Leading
flat symbols included SHA-512 AVX2 (11.40%), FST registry entry (8.39%), FST
stream iteration (4.16%), registry-vector extension (3.26%), ChaCha AVX2
(2.88%), decompression (2.67%) and Poly1305 AVX2 (2.05%). These are executing
symbols, not caller attribution or time blocked on locks/I/O. They do not
establish which policy, query or mutation caused the work. Around the capture,
a 24.5-second I/O observation recorded about 2.34 GB of logical reads but only
106 KiB of physical reads, plus 243 MB of physical writes. Those counters also
cannot assign the activity to particular files or functions.

The next structural candidates are fewer fresh SQL round trips, bounded
background telemetry, and reducing synchronous index publication work. Verify
each on the same encrypted, policy-enforced workload. Moving a function into an
`async` block alone does not remove synchronous CPU work, file I/O or mutex
contention; offloading also needs bounded admission and explicit failure,
shutdown and recovery behavior. Security decisions remain authoritative.

### First structural changes: controlled result

Release source `426177bf` includes fresh final-admission query batching, a
bounded HTTP aggregation worker, a bounded application-log writer, and opt-in
index wall timings. Both server/native-fixture metadata checks passed, as did
22 HTTP-telemetry, 20 log/redaction, two index-timing and 21 generator tests.
The Linux release build executed all 13 fixture gates, including the strict
timing-switch parser. Its immutable image is
`544c8849784e251d3ae4b70e99d12f4f3efbdfe60c2b309c7b39b2bf56245c4f`.

Four uninstrumented, 60-second runs used baseline/candidate/candidate/baseline
order, the same 100-user/50-message encrypted corpus, default three index
workers, 16 outstanding requests and the previous CPU/RAM limits. Actual UI
property profiles matched. All HTTP, isolation, policy revoke/restore,
linked-login and scanner checks passed. Candidate telemetry had zero drops
and no pending aggregation debt at the final snapshots.

| Release | Successful in-window RPS, each run | Mean RPS | SQL reads per completed HTTP, each run |
| --- | --- | ---: | --- |
| Baseline `7c0b4829` | 37.08, 35.58 | 36.33 | 67.49, 67.58 |
| Candidate `426177bf` | 36.17, 37.18 | 36.68 | 62.04, 62.02 |

The roughly one-percent throughput difference is within observed variation;
there is **no demonstrated throughput gain**. Fewer SQL reads are verified,
but the principal bottleneck remains. The fixture does not install the CLI's
application-log writer, so this comparison cannot measure that writer's
production I/O improvement. It measures the combined HTTP/authentication
changes rather than isolating their individual effects.

Two separate 20-second candidate diagnostics enabled request/index timings.
At 16 outstanding requests, 834 responses completed in-window (41.7 RPS),
850 completed including drain, and p95 was 898 ms. At 64 outstanding, the
corresponding figures were 790 (39.5 RPS), 854 and 3,090 ms. Both passed the
same security/coverage/resource gates without HTTP errors or telemetry loss.
More concurrency did not demonstrate additional throughput. These instrumented,
shorter runs are excluded from the uninstrumented comparison above.

| Index wall counter | 16 outstanding | 64 outstanding |
| --- | ---: | ---: |
| Commit calls | 336 | 340 |
| Summed commit time | 20.316 s | 21.783 s |
| Mean commit time | 60.46 ms | 64.07 ms |
| Summed writer-lock wait | 47.245 s | 50.272 s |
| Summed underlying sealed-file writes | 15.578 s | 16.314 s |
| Summed reader reload | 3.090 s | 3.710 s |
| Summed sealed decode | 2.500 s | 2.848 s |

The completed phase intervals including drain were 20.623 and 22.238 seconds.
The serialized writer spent almost those entire intervals committing. App CPU
was approximately 2.17/2.21 cores and post-phase RSS 346/348 MiB. These findings
support index publication as the immediate throughput constraint. The write
counter includes underlying durability work; it does not isolate `fsync`.
Nested and concurrent times overlap and must never be added or interpreted as
CPU percentages.

The next diagnostic measures existing bulk updates before selecting publication
batch sizes. Any new publisher must keep live source/posture checks at execution,
preserve acknowledgements after commit/reload, and order mutations with rebuild
and physical account erasure. Additional crash-recovery or early-acknowledgement
guarantees require durable replay design. None of these index architecture
changes is implemented by this measurement. The 40,000-RPS/10,000-user,
simultaneous-WebSocket and whole-host capacity targets remain unproven.

### Bulk publication amortization

A further diagnostic kept the same frozen candidate server, sealed storage,
policies, 100 accounts, 50 messages per account and resource limits. Additive
generator helpers used the existing JMAP API; server/default workload modules
were unchanged. Each successful HTTP cycle set `$seen` on the first N messages,
read all N back, restored their flags, and verified that restoration. Thus each
cycle contained four JMAP methods and two N-document publications. The
helpers explicitly exported a bulk-diagnostic profile rather than claiming the
ordinary representative request mix. Timing instrumentation was enabled.

Each fresh, 20-second run independently verified all 5,000 message IDs, flags
and original mailboxes before and after measurement. Unknown mutations,
errors, telemetry loss, excessive aggregation debt or failed state restoration
prevented a successful result. All four runs passed. Timed population coverage
is reported separately from the complete before/after state validation.

| Messages per update | Successful HTTP cycles/s | Validated document applications/s | Mean commit | Timed users |
| --- | ---: | ---: | ---: | ---: |
| 1 | 8.05 | 16.1 | 59.24 ms | 100/100 |
| 8 | 5.70 | 91.2 | 84.24 ms | 100/100 |
| 32 | 5.05 | 323.2 | 91.81 ms | 98/100 |
| 50 | 5.10 | 510.0 | 88.92 ms | 100/100 |

The 50-message case processed about 31.7 times as many validated document
applications per second as the single-message case while retaining durable
publication. This measures the amortization of the existing multi-ID update
path; it is neither a 31.7-fold HTTP throughput gain nor verification of a new
cross-request publisher. The measurements support implementing bounded,
ordered cross-request coalescing and then remeasuring the original workload.

Native fixtures can opt into additional index wall-time diagnostics with
`MW_NATIVE_LOAD_INDEX_TIMING=1` (`0` or absence keeps them disabled). This is
validated before seeding; ordinary production constructors do not enable it.
`diagnostics.searchIndex.timings` contains fixed process-wide cumulative calls,
failures, total/max microseconds and bytes for writer-lock wait, commit, reader
reload, sealed file read/decode/encode/write, and directory sync. Compare stage
before/after snapshots; counters are never reset. No paths, identities or mail
content are retained. Old fixture reports leave this information unreported.
Fields are sampled independently while background work may continue; each
snapshot is approximate, and overlapping field reads are not a transaction.

These are overlapping wall timings: commit and reload include work also counted
in the sealed-directory phases. Do not add them together or call them CPU
attribution. Sealed read bytes count successfully returned ciphertext; decode
and encode bytes count input buffers, including failed attempts; sealed write
bytes count attempted ciphertext. Atomic-write timing includes the underlying
durability work and does not separately identify fsync time. Enabled diagnostics
can affect performance, so label measured runs as instrumented.

Before/after `httpTelemetry` snapshots expose only worker queue counters and
health. New capacity qualification requires the worker running and not closing
at both snapshots, with no dropped-observation increase. Missing legacy health
remains unreported and cannot establish this qualification. Queue depth includes
pending work; it is visible separately and is not itself an HTTP error.
