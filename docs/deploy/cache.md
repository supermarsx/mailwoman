# Shared cache (Redis/Valkey)

Mailwoman's cache (`mw-cache`) speeds up repeated reads. It is never the source
of truth: on a miss, Mailwoman reads from the store or the mail server. The cache
has an in-process memory layer, an optional Redis or Valkey layer, and the store
itself. This page explains what `MW_REDIS_URL` does, which data goes where, and
what happens when Redis is unreachable.

::: warning What the Redis layer does today
With the built-in defaults no cache class uses the Redis layer, and the server
has no setting, console screen or command to change those defaults. Setting
`MW_REDIS_URL` makes the server connect at start-up and `mailwoman doctor`
report the connection, but nothing is read from or written to Redis.
:::

## Connect a Redis or Valkey server

Set `MW_REDIS_URL` before starting the server:

```sh
MW_REDIS_URL="redis://cache.internal:6379" mailwoman serve
```

- Use `rediss://` for TLS. The client (`fred`) uses rustls.
- The cache is attached only in engine mode (`--mode engine` or
  `MW_MODE=engine`). In proxy mode, the default, the setting has no effect.
- Leave `MW_REDIS_URL` unset (the default) to run with memory and store only.

`bash deploy.sh deploy --full-stack` starts a Valkey 8 container and sets
`MW_REDIS_URL` itself; remove any external `MW_REDIS_URL` from `app.env` first,
or the script stops with `MW_REDIS_URL is managed by --full-stack`. See
[the deployment script](script.md#full-stack-and-one-step-sqlite-migration).

Valkey is BSD-licensed. Redis 7.4 and later use the RSALv2/SSPL licences.
Both speak the same protocol, and the client works with either.

## Cache classes

Each cache class has a list of layers it may use and a lifetime (TTL). These are
the defaults the server runs with:

| Class | Layers | TTL |
|---|---|---|
| `sessions` | memory + store | 1 h |
| `header-windows` | memory | 5 min |
| `message-bodies` | store | 24 h |
| `blobs` | store | 7 days |
| `search-hot-set` | memory | 2 min |
| `push-presence` | memory | 1 min |
| `rate-limit` | memory | 1 min |
| `gal-directory` | memory + store | 1 h |

`blobs` can never use Redis: a Redis layer requested for it is dropped.

The memory layer holds up to 10,000 entries, plus a separate mail-content
memory area limited to 64 MiB, with single entries up to 8 MiB.

## Zero-access accounts

For an account with [zero-access encryption](../security/zero-access.md), values
derived from decrypted mail are never written to any cache layer (memory, Redis
or store). They are loaded for the request and discarded. This is part of the
code path, not a setting, and cannot be turned off.

## Check the cache

```sh
mailwoman doctor
```

`doctor` builds the cache the same way the server does and prints:

- `redis/valkey:` `not configured`, `connected`, or
  `configured, UNREACHABLE (degraded)`;
- `store fall-through:` `attached`, or `none` if the store could not be opened;
- one line per class with its layers and TTL.

## When Redis is unreachable

- **At start-up:** the server logs `Redis/Valkey unreachable — degrading to memory + store (no data loss)`
  and continues without the Redis layer. It does not retry later; restart the
  server once Redis is back.
- **While running:** a failed Redis call is logged once as
  `Redis/Valkey error — degrading to memory + store` and the request continues
  from memory or the store.

You can flush, resize or stop the Redis server without losing data.

## Options

| Variable | What it does | Default |
|---|---|---|
| `MW_REDIS_URL` | Redis or Valkey URL (`redis://` or `rediss://`) | unset: memory and store only |

## Related

- [PostgreSQL backend](postgres.md)
- [Deployment script](script.md)
- [Zero-access encryption](../security/zero-access.md)
- [Scaling and multi-location boundaries](scaling.md)
