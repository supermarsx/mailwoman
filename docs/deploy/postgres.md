# PostgreSQL backend (V6)

Mailwoman's store (`mw-store`) runs on **SQLite** (the default, zero-config) or
**PostgreSQL**. Both speak one logical schema; the backend is chosen at runtime by
the **DSN** you point the server at. SQLite stays the default — Postgres is strictly
opt-in, so single-user / self-contained deployments are unchanged.

## Choosing a backend (by DSN)

The store DSN comes from `MW_DB_PATH` (or `--db-path`). The scheme selects the
backend:

| `MW_DB_PATH` value | Backend |
|---|---|
| `mailwoman.db` (a bare path) | SQLite (file) |
| `sqlite://…` / `sqlite::memory:` | SQLite |
| `postgres://user:pass@host:5432/db` | PostgreSQL |
| `postgresql://…` | PostgreSQL |

Example (Postgres):

```sh
MW_DB_PATH="postgres://mailwoman:secret@db.internal:5432/mailwoman" \
MW_SERVER_KEY="$(openssl rand -hex 32)" \
mailwoman serve
```

Nothing else changes: the SPA, the JMAP surface, sessions, and every V1–V6 feature
behave identically on either backend. The migrations run automatically on first
connect (`migrations/` for SQLite, `migrations_pg/` for Postgres — the same 0001–0007
schema in each dialect).

## TLS

The Postgres backend uses **pure-Rust rustls** (`sqlx` with the `tls-rustls`
feature) — there is **no OpenSSL** anywhere in the tree (enforced in `deny.toml`).
Request TLS to the database with the standard libpq DSN parameter:

```
postgres://mailwoman:secret@db.internal:5432/mailwoman?sslmode=require
```

## Migrating an existing SQLite store to Postgres

`mailwoman migrate-store` copies a populated SQLite database into a Postgres backend,
row-for-row, and reports per-table counts. The source and destination **must share the
same `MW_SERVER_KEY`** — sealed columns (credentials, wrapped keys, webhook secrets)
are re-opened under that key during the copy, never decrypted to plaintext on the way.

```sh
export MW_SERVER_KEY="…the key your SQLite deployment already uses…"

mailwoman migrate-store \
  --from "sqlite://var/lib/mailwoman/mailwoman.db" \
  --to   "postgres://mailwoman:secret@db.internal:5432/mailwoman"
# → migrated N rows across M tables from … → …
```

`--from`/`--to` also read `MW_MIGRATE_FROM` / `MW_MIGRATE_TO`. The command is a copy,
not a move: the SQLite file is left untouched, so you can verify the Postgres side and
cut over by changing `MW_DB_PATH`, then retire the old file. The V6 admin/OAuth/webhook
tables (0007) are provisioned empty on the destination if the source predates them.

The copy asserts **row-count and content parity**; the same check runs in CI
(`migrate-store-smoke`), and a table-driven **backend-parity** suite runs every store
repo method against **both** SQLite and a live `postgres:16` and asserts identical
results (`store-dual-backend`).

## Connection pooling & sizing

`sqlx` manages the connection pool. Size the Postgres server's `max_connections` for
your expected concurrency; a single Mailwoman instance holds a modest pool. For
multiple instances against one database, no application coordination is required — the
schema and all writes are transactional.

## When to use Postgres

- **SQLite** — single-user, evaluation, self-contained desktop mode, small
  single-operator installs. Zero-config, one file, fully featured.
- **Postgres** — multi-instance / HA deployments, external backup/replication tooling,
  or operators who already run Postgres and want the store there.

The choice is purely operational; feature parity is a CI gate, not an aspiration.
