# Layered cache: Valkey / Redis (V6)

`mw-cache` is an **accelerator**, never authoritative. It layers an in-process cache
(`moka`) in front of an optional Redis/Valkey tier (`fred`), with the store as the
source of truth. Losing the cache loses performance, never data: every miss falls
through to the store, and a Redis outage degrades to memory + store transparently.

## Enabling it

Set `MW_REDIS_URL` to point at a Redis-protocol server:

```sh
MW_REDIS_URL="redis://cache.internal:6379" mailwoman serve
```

Unset (the default), Mailwoman runs memory + store only — fully functional, just
without the shared tier. TLS uses **pure-Rust rustls** (`fred`); no OpenSSL.

### Valkey vs Redis (a licensing note)

Mailwoman standardises on **Valkey** (`valkey/valkey:8`, BSD-3-Clause, the Linux
Foundation fork) for its CI and documented deployments. Redis 7.4+ is RSALv2/SSPL
(source-available, **not** permissive), which is why the license-clean default is
Valkey. `fred` speaks the same wire protocol to either, so an existing Redis is
equally supported at runtime — the choice is yours; only the *documented* default is
Valkey.

## Scope matrix (SPEC §15.6)

Each cache **class** has a policy: which layers it may occupy and a TTL. These are the
built-in defaults (admin-overridable per class):

| Class | Layers (default) | TTL |
|---|---|---|
| `Sessions` | memory + store | 1 h |
| `HeaderWindows` | memory | 5 min |
| `MessageBodies` | store | 24 h |
| `Blobs` | store | 7 d |
| `SearchHotSet` | memory | 2 min |
| `PushPresence` | memory | 1 min |
| `RateLimit` | memory | 1 min |
| `GalDirectory` | memory + store | 1 h |

The **Redis tier is opt-in per class**: an admin adds `redis` to a class's layers via
the admin panel / config override. `Blobs` is **not Redis-eligible** — a Redis tier
requested for it is dropped (and reported), because large opaque blobs belong in the
store, not a shared in-memory cache.

`mailwoman doctor` prints the effective posture — the resolved matrix, whether Redis is
configured/connected, and whether a store tier is attached — so an operator can see
exactly what is cached where.

## Zero-access exclusion (structural, not by diligence)

For a **zero-access** account (see [`../security/zero-access.md`](../security/zero-access.md)),
a value derived from decrypted plaintext must **never** enter a shared cache tier.
`mw-cache` enforces this in the type system, not by operator configuration: such a
value is wrapped in a `PlaintextDerived` marker, and when the account's posture is
`ZeroAccess` the cache **refuses** to place it in Redis, memory, or the store cache
tier — it is confined to per-request scope. This is verified in CI against a live
Valkey (`cache-valkey`), asserting a zero-access `PlaintextDerived` value is provably
never written to Redis or memory. The exclusion cannot be disabled by an operator; it
is a property of the code path.

## Redis-down degradation

If Redis is configured but unreachable, cache operations fall through to memory + the
store with **no data loss** — the store is always the authority. CI exercises this leg
explicitly (a Redis-down path alongside the live-Valkey round-trip). Treat Redis as a
speed-up you can lose at any moment; nothing in Mailwoman depends on it being up.

## Sizing

The in-process `moka` layer bounds itself; the Redis tier is sized on your Redis/Valkey
server. Because the cache is never authoritative, you can flush it, resize it, or take
it offline without touching correctness — only latency.
