# Layered capacity measurements

The end-to-end target is 40,000 sustained successful authenticated HTTP requests
per second with 10,000 users and enforced production policies. A result for a
cheap endpoint, a database operation, or an individual JMAP method does not
establish that target. One HTTP JMAP batch is one HTTP request.

Measure the following separately before diagnosing a mixed-workload regression.

| Layer / operation | What the measurement includes | Essential companion measurements |
| --- | --- | --- |
| HTTP transport and router | `/healthz`, keep-alive, response delivery | CPU, connections, generator utilisation, bytes; **bypasses application guards**, so only a transport control |
| Guarded identity | Cookie-authenticated `/api/me`, unchanged loaded/enforced policy, identity and account checks | Guard self/inclusive times, SQL counts, pool wait, cache hits/misses and invalidations |
| Database | Real Store reads and transactions against an isolated encrypted database | Pool acquisition, elapsed operation time, failures, read/write mix, commit/durability, database/cache size |
| Header retrieval | `Email/get`, exact UI properties | Returned rows/bytes, SQL counts, serialization and source metadata work |
| Folder query and pagination | `Email/query` and the UI query/header batch | Count/order/filter work, returned rows, index searches, SQL and page-cache activity |
| Reader | Body and attachment metadata | Payload size, body-cache state, decryption/MIME work, serialization |
| Attachment delivery | Validated attachment download | Payload size, decoded byte throughput, pool/body lookup and decryption |
| Flag write | Mark read, verify, restore, verify | Actual DML/commits, index publication and sealed bytes, write queue/debt |
| Move write | Move, verify destination, restore, verify | Same write measurements; stable identity and reconciliation correctness |
| Drafts and settings | Actual authenticated load/save flows | Persistence/notification work, payload size and read-versus-write results |
| WebSocket | Same authenticated operations, persistent connections and push fanout | Connection count, messages/s, HTTP handshake versus action rate, delivery latency, reconnects and backpressure |
| Upstream ingestion and submission | Separate protocol fixtures or dedicated live accounts | Arrival-to-visible/push latency, reconciliation, submission success and actual upstream limits |

An encrypted Store operation benchmark is an operation rate, not an HTTP rate.
Run identity and application endpoints with the same security configuration;
never disable authentication, encryption, policies, or durability to make a
comparable application number larger. A separately labelled unencrypted control
can isolate encryption cost, but cannot qualify encrypted production capacity.

## Timing boundaries

Record separate distributions and sample counts for:

1. Scheduled arrival to generator execution (scheduling lag).
2. Waiting for that principal's preceding operation (client queue).
3. Actual HTTP dispatch through response consumption and validation.
4. Validation inside that interval.
5. Server request phases, SQL acquisition/operations and index publication.

The historical `latencyMs` begins before the principal queue. Retain it for
comparison and label it accurately. Dispatch-to-completion still includes
network, HTTP client connection scheduling, body consumption and validation;
it is not isolated server CPU time. Requests that time out before dispatch have
no HTTP timing sample. Report queue timeouts and actual dispatch counts.

Do not subtract percentiles, add overlapping phase times, or subtract a health
response time from an authenticated response to invent authentication cost.
Store operation elapsed time already includes acquisition; adding acquisition
again double-counts it. Process-wide counters include background work and
cannot be attributed to one route without scoped measurements.

## Run protocol

Use the existing opt-in native HTTP fixture and representative workload; do not
create a second synthetic application server. Record the source/image digest,
release profile, enforced-policy proof, encryption/index configuration, worker
count, pool limits, app and generator CPU/RAM limits, payloads, and user count.
Generators run outside the application resource group. Simulated client
addresses are not additional physical generator hosts.

For every isolated route, start with a low-concurrency latency run, then a
concurrency ramp, then offered rates around the observed knee. Repeat the
baseline/candidate in alternating order. Warmup, startup readiness, seeding,
diagnostic snapshots and drain are outside the scheduled measurement window.
Count only successfully validated responses completed in that window as RPS.
Keep errors, 429s, timeouts, dropped arrivals, outstanding work and debt visible.

Run reads separately from writes, then the same mixed workload. Use multiple
independent users while serializing conflicting operations for each user. A
mark/read/restore HTTP batch has four JMAP methods; document this work rather
than calling the batch four HTTP requests. A route's equal share of mixed RPS
is **not its standalone capacity**.

After individual routes, repeat the mixed workload with background maintenance,
indexing and telemetry enabled. Run a separate 10,000-user working-set and
WebSocket test. A short clean run is an observed throughput point, not proof of
sustained capacity; report the duration, latency budget and stability criteria.

## Interpreting a bottleneck

| Observation | Next controlled measurement |
| --- | --- |
| Transport control is slow and generator CPU is saturated | Separate generator limits from application transport/CPU costs |
| Identity is much slower than transport | Attribute guard/authentication state loads and serialization with scoped counters |
| DB operation or pool wait grows with concurrency | Inspect pool occupancy, active transactions, writer serialization and storage; do not assume more connections help |
| Reads are fast alone and stall beside mutations | Correlate commit/publication locks, pool wait and pending work in a mixed run |
| Flag/move writes publish large immutable documents | Measure publication bytes and commits; verify all current-metadata query paths before removing redundant publication |
| Reader/download scales with payload rather than request count | Report bytes/s, CPU, cache state and decryption/serialization time |
| New candidate is slower | Keep it available, split the changes into controlled arms and attribute the regression before deciding what to retain |

Current architectural correctness gates and SQL-count reductions do not replace
these capacity measurements. Unknown component capacities remain unknown until
their independent runs finish.

## Current fixture coverage and selectors

The representative native mail fixture now supports independent groups:

```text
MW_LOAD_FIXTURE_ENDPOINTS=identity,mailboxes,query,headers,refresh,paged-headers,reader,attachment,flag-write,move-write
```

Four client-hot groups (`jmap-session`, `list-batch`, `changes`, `account-polls`)
can be added to the same list.

Use `MW_NATIVE_LOAD_WORKLOAD=representative` for the last five groups. The
existing `representative` selector retains the complete ten-shape mixed run.
Both the single-process and partitioned generator use the same validated request
templates. These selectors change what is measured, not application policies.
See the [native fixture instructions](native-http-fixtures.md) for resource,
startup, encrypted database and policy controls. A complete ten-group ramp can
exceed the default campaign deadline; split it into bounded campaigns or select
an explicit sufficient deadline rather than treating a truncated run as complete.

| Application area | Existing seeded coverage | Status |
| --- | --- | --- |
| Mail and identity | Independent accounts, messages, folders, bodies and attachments; validated read/write/restore requests | Independent selectors available; capacity requires actual per-group release runs |
| Client-hot routes | `jmap-session`, `list-batch`, `changes` and `account-polls` groups (preferences, appearance customization, session activity); read-only, no extra seed | Selectable separately from the mail groups; shapes copied from the web client, see [client-hot route groups](native-http-fixtures.md#client-hot-route-groups); not yet run against a release image |
| Drafts | No representative drafts in the native load seed | Representative seed, save/reopen/send validators and measurements pending |
| Settings | Global fixture policies; no representative per-user preference writes | Defaults-only reads would be controls, not a representative settings workload |
| Notifications | No representative notification-centre records | Seed, dismiss/archive validators and measurements pending |
| Contacts, calendars, tasks and notes | No PIM records in the native load seed | Empty-list controls would not establish capacity for populated accounts |

Do not report the mail fixture as whole-application coverage. The source audit
above describes coverage, not successful execution of those pending cohorts.
