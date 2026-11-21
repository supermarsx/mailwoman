# Guard input publication prerequisite (0114)

SOURCE ONLY. This concern provides coverage and immutable DATA preparation seams.
No request consumes PreparedGuardInputs yet, no guard is skipped, no allow/Session
or plaintext credential cache exists, and no SQL/RPS/zero-SQL claim follows.
Migration 0114 MUST integrate/deploy only after the separately owned mail-data 0113.
There is no source scan, automatic backfill, global mail-flag epoch or new worker
cache in this prerequisite.

## Exact authority coverage

The existing 111 protection scope covers raw source bans `v6:admin:bans`. New 114
triggers add only these three exact legacy raw settings keys:

- `security.inbound-allowlist.v1`
- `security.clientAccess.v1`
- `maintenance.mode.v1`

SQLite AFTER INSERT/UPDATE/DELETE and PostgreSQL schema-bound trigger function
bump the protection scope in the same source transaction. UPDATE bumps only on
null-safe changed key/value and old OR new exact-key membership, including rename
into/out of coverage. No-op updates, unrelated telemetry settings, and ordinary
session last_seen touches do not change these shared configuration epochs.
Ciphertext/random-nonce changes are conservatively source changes; no logical
JSON equivalence is guessed. Epoch/outbox update failure aborts the business
write, and rollback does not publish a changed version.

Required coverage validation reuses exact native 111/112 definitions and adds 114
triggers/functions. The cheap native schema stamp already observes DDL on the
settings table/its triggers and functions; missing/disabled/replaced definitions
cannot qualify this preparation. This is a supported-writer/schema-change
contract, not protection against a malicious DBA rewriting every source/proof.
The refactored base validator retains the already-integrated PostgreSQL CAST type
syntax correction (e2cab859); do not restore its old malformed CAST during merge.

## Store and preparation source contract

NativeGuardInputSnapshot is opaque Store/generation/current account/session/
principal/binding source plus four deferred exact raw sections. Cold extraction
uses a fresh before fence, cold required-definition checks, one current-owner
raw source SELECT (at most five rows including an overflow sentinel), and a fresh after fence. Candidate raw values are at most 1MiB
aggregate by SQL octet lengths; invalid/overbudget sections return NULL payloads
before driver transfer. This is a driver transfer/retention bound, NOT a hard DB
executor, sort, pager-I/O or allocator-workspace bound. Missing source/coverage
falls back; malformed sections remain deferred errors, never invented defaults.
A foreign Store or changed current-parent/source vector cannot certify the DATA.

PreparedGuardInputs reads global registry INPUTS using the existing bounded cold
settings_publication_inputs reader, resolves the existing pure registry loop
once, and retains immutable Effective DATA (8MiB bounded logical policy encoding)
and parsed original inbound/client/maintenance Config DATA. It checks the same
source again after all awaits. It does not use legacy settings_revision memo hits
as authority. Opaque parser wrappers call the original parsers/defaults/validation;
DNS answers/TTL, client PSK/proof verification, maintenance response decisions,
authoritative protection transactions and request-watch side effects stay in the
original guards. Source bans deliberately retain the old parser/admission path.
No successful preparation is an admission predicate.

## Consumer stage still required

The next independently reviewed concern must bind these DATA to the same original
guard positions and error precedence, with current source/IP/protocol/target,
request counters/quota, device/watch, DNS and process clocks recomputed. It must
keep a fresh late joint authority/data boundary after permits/connector/body
awaits and fall back to the complete old chain on unsupported coverage. Current
clock/last_seen values must come from that current request fence, not this cold
source frame. Independent Store/raw writers still require fresh SQL fences.
Asynchronous outbox delivery alone never makes zero-SQL authorization safe.

## Authored gates and boundaries

None of these new tests has run in this concern.

- Store authority_publication::tests filters containing `guard_`: 9 new cases,
  including 4 UUID-schema PostgreSQL cases using the existing private MW_TEST_PG
  wrapper. Actual PG is mandatory for qualified integration evidence.
- Server published_guard_inputs::tests: 3 cold-preparation cases.
- Client-access, inbound-allowlist and maintenance published_input_tests: 1 each.

Cases cover exact key insert/update/delete/renames, no-op/hot telemetry, coalesced
outbox target, rollback and injected outbox failure, owner/deferred malformed
sections, SQL NULL mask before oversized transfer, foreign Store binding, missing
or disabled schema coverage, deterministic mid-build raw change, actual separate
SQLite Store writer and restart, old-memo vs fresh global source, deferred guard
parse failure, original pure parser parity and a final preparation race.
Existing native schema/publication/independent-writer cases must also rerun after
integration. The isolated base predates a number of primary test-only corrections;
root preserves those during concern integration, and no missing 113 placeholder is
introduced here.
