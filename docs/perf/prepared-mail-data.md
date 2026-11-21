# Prepared default36 mail DATA (experimental)

This source candidate is disabled by default. It has not yet passed its Rust,
router or controlled performance gates. It does not establish zero-SQL reads,
10,000-user fit, or a capacity improvement. All existing Index writes, query
semantics, physical-erasure barriers and ordinary read fallback remain.

`Engine::prepare_metadata_candidate(account,args)` must run **before** final
authorization. It accepts exactly the representative36 metadata properties and
at most50 literal IDs. It retains encoded row fragments, never Session/ALLOW,
credentials, call IDs or a decoded copy of every row. Strict preparation reads
only current owner-bound projected envelopes; legacy/corrupt/oversized data
falls back to the ordinary reader rather than fetching MIME in this cache.

The native single-call metadata endpoint extends its **last joint authorization statement** with the
optional Store DATA descriptor. Compute `joined_mail_data_sql` only on the
actual authenticated source row, require descriptor account equality, and mint
the fence with that statement's actual serving-generation context. No async
authority/data/posture read may follow the last joint proof. After all existing
auth checks, `finish_prepared_metadata(candidate,joint,call_id)` synchronously
checks source, posture and clean Index publication state, emits escaped current
state/call-ID envelope plus retained row bytes, then checks the pure proof again.
Its optional byte result is complete JMAP JSON. The compatibility Value helper
retains parsing/final serialization costs. Multi-call/result-reference requests,
outer `createdIds`, and unsupported property sets keep the ordinary dispatcher.
The source-wired HTTP consumer still requires its actual router/runtime gates.
Ready candidate selection now precedes standalone DATA SQL; cold preparation
retains those source/schema reads. Final emission still requires the joint query;
missing/mismatched final DATA discards only the same retained candidate Arc, never
a concurrent newer installation. Early guards and the initial policy fence remain. A one-query
Store helper is not a claim of one query or zero queries for the whole request.

Migration0113 follows0112. Compact row tuples contain data/cache/content
revisions maintained by source triggers; final proof still checks actual early
message owner/liveness columns. Missing tuples mean needs-seed, never notFound
or permission. Short parent-first seeding occurs only for missing derived
source, before assembly/Index work. Contacts, Sent role/order/threads/recipients,
verdicts, body changes and posture invalidate dependent results. Outbox hints
are revision-CAS acknowledgements, never authority or a RAM-convergence lease.
Copy regenerates derived state; account/DSR erasure removes it after source
deletion. Exact schema/function/trigger checks reject unsupported definitions.

Logical Index proof is selected-message/account scoped, with conservative Focus
dependencies and rare global rebuild/recovery/unknown-writer invalidation.
Pending, poison, dirty, erasure and stopped-owner states refuse a prepared hit.
Other accounts' ordinary owned commits do not invalidate these selected rows.
Raw Index escape disables eligibility rather than inventing write coverage.

`configure_prepared_mail_at_revision` applies already-authorized configuration
before preparation and the endpoint compares it with the final policy proof.
Older differing settings revisions and later unfenced setters cannot overwrite
a newer configuration; older identical DATA does not lower the revision floor.
Configuration
changes invalidate retained candidates. Defaults are disabled,64MiB,256 entries,
128 accounts and30s idle. Finite bounds permit16..16384MiB,1..100000 entries,
1..10000 accounts and1..3600s. These are measured-profile ceilings, not a claim
that defaults fit10,000 principals. The global `performance.preparedMail` registry
entry, bounded admin editor and aggregate Cache maintenance card are wired in
source; policy publication explicitly collects that key and a registered but
uncollected value refuses native eligibility. No per-hit config query is added.

Fixed bounds are16 admitted preparations,2 active,50ms preparer wait,9MiB build
reservation and8MiB encoded output. KiB permits round upward and survive held
or evicted result Arcs. Shrinking/disable does not manufacture permits or free
live bytes; new admission stops above the configured budget. Parser/SQL scratch
and RSS are not a hard memory or CPU sandbox. Numeric status reports current
charged bytes, optional lock-consistent entry/config counts, hit/miss/build/
eviction/seed/hint totals and separate worker polling. No user identities appear.

Authored tests are unexecuted: actual50-message property/byte parity, duplicate
and missing IDs, selected-source and pending-writer refusal, retained charges,
unselected metadata recertification, direct Index invalidation, config disable,
and actual newest500 Sent cutoff classification; SQLite/PostgreSQL DML, tuple,
seed, corruption, outbox, cascade/recreation and real person-erasure contracts.
Actual backend/runtime/router, cancellation, copy and concurrency verification
must precede performance claims.
