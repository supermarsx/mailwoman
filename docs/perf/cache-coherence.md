# Mail cache coherence across replicas

This describes the engine's header and raw-message read paths. It does not claim
that every generic cache class, upstream sync worker or WebSocket subscription is
coordinated between replicas.

## Authoritative validation

Migration `0105_message_cache_revision.sql` adds `messages.cache_revision` on both
SQLite and PostgreSQL. Database triggers change this opaque token on insert and
update, including envelope repair, flags, moves and ordinary direct SQL writes.
New tokens prevent a deleted/reinserted stable ID from selecting an old envelope.
The SQLite update trigger ignores its own token-changing update, including with
`recursive_triggers=ON`. PostgreSQL uses a before-write trigger.

The migration uses a constant empty default for existing rows and does not
backfill or rewrite every cached envelope/body. An existing row receives its first
random token when it changes. Account ID and stable ID are also part of its cache
key, so different existing rows cannot collide merely because their token is empty.

Every production cache read starts with a fresh authoritative `Message` row.
`Email/get` carries revisions in its existing batched `get_messages` projection;
validation adds no per-message revision query. Its live account cache posture is
read once per account and reused within the request. A failed posture lookup must
answer zero-access, which bypasses shared caches.

Envelope keys include account ID, stable ID and revision. Raw-body keys include
account ID and the immutable blob reference, rather than a mutable message ID.
Normal body writers insert immutable content-addressed blobs; replacing a message's
body changes its reference. Missing bodies are never cached. Expunged messages or
another account's IDs fail the authoritative ownership/existence lookup before
reaching a cached body. Orphan collection does not remove bodies still referenced
by messages.

A database failure during authoritative validation returns an error. It does not
serve the previous RAM/Redis entry as a stale fallback. On reconnection, the next
request validates a fresh row and selects its current token/reference. Correctness
therefore does not depend on delivery of an invalidation or pubsub event.

## Consistency boundary

The fresh row is a validation point, not a repeatable-read/MVCC snapshot spanning
all response assembly. A concurrent write after that read can cause another field
or a cache-miss loader to observe a newer committed value. An in-flight request is
not guaranteed to represent one atomic database instant. New requests obtain fresh
rows and cannot keep choosing an older message revision after its update commits.
This change does not add cross-replica optimistic concurrency to writes.

All replicas must use the same authoritative database and server key. A separate,
lagging read replica is not an interchangeable validation source without an
explicit consistency policy. This work also does not distribute upstream watcher
ownership or live push events.

## Placement, encryption and bounds

Standard-account mail follows its configured Memory/Redis layers. Zero-access
mail bypasses both. The mail-byte API also refuses to cache authentication sessions.
The Store remains the authoritative loader; versioned mail bytes are never copied
into persistent `settings` cache rows.

RAM uses a separate byte-weighted Moka cache with a 64 MiB capacity target, also
respecting the configured entry bound. A zero memory capacity disables RAM
admission. Values over 8 MiB bypass shared mail caches. The weight covers payload
and key bytes; allocator/cache metadata, in-flight requests and delayed maintenance
mean this is not a process-RSS ceiling.

Mail entries expire after a nonzero bounded lifetime: configured TTLs are capped
at one hour, and a configured zero uses five minutes. Old revisions can expire or
be evicted without affecting correctness.

Redis mail payloads are XChaCha20-Poly1305 envelopes bound to the complete cache
namespace: class, account and revision/content reference. They use the attached
Store's shared server key. Redis mail placement is bypassed when no Store/key is
attached. Corrupt payloads, a different key or a different namespace produce a
cache miss and fall through to the authoritative loader; they never produce
plaintext fallback writes. Standard-account RAM holds plaintext bytes.

## Legacy rows and compatibility

Existing generic cache APIs retain their previous behavior for other callers.
Production mail now uses the bounded, validated ephemeral API. Its advisory
invalidation hook evicts ephemeral legacy keys without creating persistent
`settings` tombstones.

Old `settings` entries under `mwcache:message-bodies:*` and
`mwcache:header-windows:*` are **retained** by this change. New mail reads do not use
those entries, and new mail reads/invalidation do not add replacement rows or
tombstones there. Migration 0105 deliberately does not perform a potentially
large blocking purge. Reclaiming those obsolete rows requires a separately bounded
cleanup; freeing their pages and shrinking the database file are separate steps.
No unrelated user settings should be removed.

`migrate-store` includes the new column. PostgreSQL's destination write trigger
intentionally assigns a fresh cache token while preserving user-authored and
sealed data.

## Regression coverage

Focused fixtures cover raw SQL revision changes, transaction rollback, envelope
repair, body replacement, delete/reinsert ABA, batch projection, account isolation,
two independent RAM caches sharing Store data, expunge, failed database validation,
reopening with retained RAM, live zero-access posture, missing-body bypass, byte
and entry bounds, disabled RAM, and absence of persistent mail-cache writes.

Encryption fixtures cover server-key/namespace binding and tamper rejection. The
live-Valkey test additionally checks that stored mail is encrypted and a second
cache instance can read it with the same Store key. Live PostgreSQL/Valkey legs
require their configured test services; source coverage alone is not a live
multi-node acceptance result.
