# Published native admission inputs

This candidate publishes immutable authority and policy **data**, not an allow
decision, an authenticated Session, or plaintext credentials. It is source-only
until the listed gates execute. It makes no throughput or zero-SQL claim.

The first consumer is the existing single, bounded metadata `Email/get` path,
including the real 36-property, 50-ID list request. Other request shapes keep the
complete existing path. That is one of the ten representative HTTP request kinds;
multi-call queries and result references remain outside this concern.

## Freshness and fallback

The existing current primary hint remains. A ready candidate then compares a
fresh narrow SQL authority vector before taking the ordinary request permit.
After the unchanged helper/connector waits it compares another fresh vector,
including current source-derived owner/principal references, clock fields and a
schema stamp. These are three admission reads including the primary hint, not
two whole-HTTP reads. Earlier middleware and Engine data reads still run.

The request decodes the original sealed source and evaluates current process
clocks, protocol/features, directory/domain/account rules, credential rejection,
target and rate inputs. No legacy settings-revision memo is read by the builder.
Unsupported, cold, oversized, missing or changed input uses the original fresh
path. A late mismatch retains the existing one-shot, undispatched read restart;
there is no mutation replay or second restart. No authority await follows the
successful final evaluation before Engine dispatch.

The builder resolves pure effective policy inputs once per canonical versioned
component, then shares immutable policy rows, domain/account source parts and a
normalized held-directory set. Valid sibling names form an order-insensitive
rule input set; malformed/oversized sibling or held rows cannot qualify and keep
the ordinary ordered reader. Raw source rows remain sealed where appropriate.

## Publication and source coverage

One serial worker exists per shared Store instance, with weak process registry
ownership (at most 32 Stores). It uses a channel of 1,024 bounded hints, at most
256 pending keys, three source-race attempts per build and a bounded failure
cooldown. Only a current primary hint with the same selected owner can enqueue
request work. Invalid cookies do not start builders. No SQL connection or
transaction spans preparation, queue waits, or Engine execution.

The worker reads source/routing/policy dependencies, installs typed canonical
subject absence anchors in a small source-CAS transaction, then captures a full
before/source/after vector. Shared data is installed only after the complete
after vector matches. Outbox polling claims at most 32 rows; acknowledgment means
an invalidation hint was handled, not that another replica has installed RAM.
Outbox failure or lag never substitutes for the fresh request SQL vector.

Migration 0112 adds only a small anchor table and indexes/triggers, with no source
backfill. Each session replaces its current bounded subject set. Account/session
incarnations prevent reused IDs from retaining old proofs. Parent cascades and
final-order erasure remove canonical subject PII after its last source disappears;
another live account's shared subject remains. Cleanup uses bounded batches.

Warm schema stamps participate in every component identity. Cold validation
checks exact required 0111/0112 trigger definitions and the PostgreSQL trigger
functions under a schema-bound search path. Missing/replaced coverage rejects
reuse. This covers supported schema/DML writers; it is not protection against a
DBA deliberately rewriting both source and its epoch/catalog evidence. Strong
exclusive ownership and zero-SQL admission remain separate concerns.

## Resource and performance limits

The cache holds at most 16,384 entries and 32,768 shared components, with a 128 MiB
logical cache-ownership budget. This is **not** a strict process RSS or in-flight
heap ceiling: borrowed Arcs can outlive eviction, and source/parser/driver scratch
is separate. Output guards limit transferred source rows/bytes before driver
return, not SQL executor workspace, join/sort/scan work, or encrypted pager I/O.
In particular PostgreSQL may internally materialize wide CTE data before the
outer guarded projection. No SQL-index cost or wall-time bound follows.

A ready hit still hashes a bounded cookie/selector, uses a nonblocking memory
lookup, decodes source credentials, clones selected policy values, evaluates
dynamic checks and serializes the normal Engine response. It avoids a registry
resolution and whole held-directory scan per hit; it is not allocation-free.
The worker's awaited queries run outside request task-local query counters and
must be accounted separately in measurement.

## Required verification

Ten new Store contracts (five actual PostgreSQL wrappers) cover typed source,
routing/policy seed CAS, no-memo input parity, payload masking, component owner
and flag-key compatibility, required schema definitions, live anchors and erasure.
Seven new Server cases cover the real DEFAULT36 route/oracle/read counts, queued
revocation, live clock expiry, invalid-cookie/oversize fallback, missing-trigger
fallback, sibling telemetry ordering, and a deterministic independent writer
between worker input extraction and its final fence. Existing native/restart,
feature-boundary and account-gate oracles remain required. No authored case is
counted as passed until actual execution, including the private PostgreSQL gate.

Mail-data migration 0113 and prepared response bytes are a separate owner/concern.
Preparation must precede the last auth observation. Its optional typed data
descriptor must be validated in the same final SQL observation as authority;
finishing and emitting prepared data after that observation must be pure, with
no later source/posture await. Unsupported custom posture keeps the Value path.
