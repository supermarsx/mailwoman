# Authority publication foundation

This first concern adds transactionally versioned DATA and a durable coalesced
outbox. It does not change routing or cache an allow decision, Session, password,
or plaintext credentials. No runtime tests or throughput measurements have run
for this concern yet. DEFAULT36 integration is a separate required stage.

## Transaction and source contract

Each scope has a random incarnation and positive monotonic counter. Source DML,
epoch increments and the last-per-scope outbox target share the source transaction
on SQLite and PostgreSQL, including independent writers. Counter overflow aborts
the mutation. Claims are only notification/build hints, never a RAM installation
proof. ACK requires the exact incarnation, target and random claim token. A stale
worker cannot ACK a recreated scope. Final-order person-erasure rules remove account/subject/principal epochs after
their sources and verify cascaded outbox removal; a recreated source is cold.
Bounded cleanup removes only acknowledged,
unclaimed absent-source epochs; recreation gets a different incarnation.

The migration creates two small empty operational tables, indexes/triggers and
four constant global roots. It does not scan/backfill existing accounts or mail.
Missing epochs require the original fresh SQL path. Destination database copies
start cold rather than inheriting another database's worker acknowledgement.

## Trigger inventory and granularity

- Shared policy definitions, groups and profiles dirty global policy. Alias
  mutations dirty routing/policy; domain insert/delete/name/preferred changes
  dirty routing, while domain configuration edits dirty their exact domain.
- User/admin-user setting values, personal values and membership changes dirty
  their exact subject (account ID or canonical address); domain values dirty the
  exact domain. Group/profile values and group assignments dirty shared policy.
  Ciphertext rewrites conservatively dirty the same scope even if plaintext is
  unchanged. These do not become global bumps for every personal preference.
- Account source fields dirty only that account. Session insert/delete and
  changes to ID/account/username/endpoints/sealed credentials/linked status,
  authentication method/UA/IP/client type/device label/client version dirty the
  old/new session and account. Principal/binding mutations dirty their scoped
  principal; primary-binding/context mutations dirty the scoped session.
- Linked-account changes dirty account/principal dependencies. Directory hold
  rows dirty global directory; zero-access, restrictions and consent dirty the
  account. Known legacy administrative flag/name/watch/ban settings have explicit
  mappings. Generic telemetry and arbitrary settings writes have no blanket
  global invalidation.
- Session created_at/activity_at/last_seen do NOT dirty authority epochs: the
  narrow current fence returns them fresh. This includes idle-expiry renewal;
  current clocks still require evaluation per request. No reusable sibling-name
  ordering is certified by this first fence. Remembered-device/user-activity
  telemetry is not published by this API; earlier device/clock guards remain.
- Mail flag/body/envelope/history/mailbox writes have no authority hook. Future
  mail.metadata, mail.content and mail.mailbox publication needs its own epochs,
  writer inventory and CAS/erasure proof, separate from authority globals.

## Fence and worker boundary

The typed narrow fence derives current account/principal/binding from the current
primary-session joins, never supplied owner IDs. Domain and subject dependency
lists must come from a verified cold builder with complete canonical coverage.
A missing/unsupported scope cannot prove absence: return fallback. This first API
cannot seed arbitrary address subjects without live source rows; it deliberately
falls back for that cold absence case. Scope identifiers are printable ASCII
with byte/count caps. Non-ASCII, NUL, oversized and missing dependencies fail
eligibility rather than being truncated. They remain legal source data for the
unchanged original path. No scope count/query shape is an authorization allow.

Versions are Store-scope and SQLite-generation bound. Generation is checked
before/after each observation; transactional ACK compares the already held
transaction generation without reacquiring a pin. No SQL transaction spans an
asynchronous builder or search fsync. All fields used by this first fence are
bounded before transfer. Current clock values are deliberately outside immutable
version equality. A future consumer must evaluate current clocks, device/source,
quota and process checks and preserve the late-after-body/permit boundary.

Legacy settings_revision memos are NOT a reusable authority cache: their source
coverage differs from these epochs. The future cold worker must use direct or
no-memo existing readers under a before/after vector, not certify old memos by a
new counter. Immutable published components should share global/domain Arcs with
bounded per-account budgets. Async outbox delivery alone never permits zero SQL.
This remains true with `MW_GUARD_SNAPSHOT=off` (the default); `bounded` replaces
the per-request fence read with a stated staleness bound, see
[guard snapshot](guard-snapshot.md).

## Authored verification (not executed)

`cargo test -p mw-store --lib authority_publication::tests --locked -- --test-threads=1`
selects eleven authored cases: five SQLite/PostgreSQL contract pairs plus one real
SQLCipher file restart/cross-Store/mail-independence case. Contracts cover source
rollback/outbox failure, concurrent updates/overflow, claim reclaim/ACK/ABA and
finite cleanup, current owner/clocks versus revocation, scoped personal/domain updates without global invalidation, and actual registry
erasure followed by exact source-ID recreation with a fresh incarnation. PostgreSQL cases create their own UUID
schema using MW_TEST_PG; an absent environment is explicitly a skipped contract,
not actual PostgreSQL evidence. The release gate must provide the private owned
fixture DSN to the child process and require zero skips.

## Unimplemented next stages

The opaque session/watch absent-source cleanup API is bounded but not an
automatically scheduled worker. Those operational tombstones can remain until
explicit maintenance calls it; they are not claimed to have automatic retention.
Current source-row updates coalesce into one scope row and clocks do not allocate
per-request new scopes.

A bounded cold worker and an actual DEFAULT36 handler consumer remain required;
no whole-HTTP SQL reduction or throughput claim follows from this foundation.
Optional strong SQLite zero-SQL mode also requires database-enforced ownership
against outside DML AND DDL, writer-settlement/read-lease proofs, lifecycle fencing
and separate warm/cold resource gates. An exclusive single handle can serialize
cold data reads; it is not a general scalability improvement. Default/legacy and
PostgreSQL retain fresh SQL fences and independent-writer revocation behavior.
