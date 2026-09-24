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
connect (`migrations/` for SQLite, `migrations_pg/` for Postgres — the same 0001–0029
schema in each dialect).

## TLS

The Postgres backend uses **pure-Rust rustls** (`sqlx` with the `tls-rustls`
feature) — there is **no OpenSSL** anywhere in the tree (enforced in `deny.toml`).
Request TLS to the database with the standard libpq DSN parameter:

```
postgres://mailwoman:secret@db.internal:5432/mailwoman?sslmode=require
```

## Migrating an existing SQLite store to Postgres

`mailwoman migrate-store` copies a populated SQLite database into a Postgres backend and
reports per-table counts. Within the tables it copies it is row-for-row: every id,
timestamp and sealed blob is preserved byte-for-byte. Sealed columns (credentials,
wrapped keys, webhook secrets) are copied as opaque bytes and are never opened or
re-encrypted on the way, so the source and destination **must share the same
`MW_SERVER_KEY`** — without it the copied bytes are unreadable on the destination.

**It does not copy the whole database.** As of migration 0029 it copies 40 of the 76
tables the schema defines. What is left behind is listed below; read it before cutting
over.

```sh
export MW_SERVER_KEY="…the key your SQLite deployment already uses…"

mailwoman migrate-store \
  --from "sqlite://var/lib/mailwoman/mailwoman.db" \
  --to   "postgres://mailwoman:secret@db.internal:5432/mailwoman"
# → migrated N rows across M tables from … → …
```

`--from`/`--to` also read `MW_MIGRATE_FROM` / `MW_MIGRATE_TO`. The command is a copy,
not a move: the SQLite file is left untouched, so you can verify the Postgres side and
cut over by changing `MW_DB_PATH`, then retire the old file.

### What is not copied

**Deliberately left behind — 16 tables.** The admin panel's own identity domain
(`admin_users`, `admin_sessions`), OAuth clients and tokens (`oauth_clients`,
`oauth_tokens`, `oauth_client_meta`, `oauth_dcr`), `api_keys`, `webhooks`, the managed
domain, directory, egress-proxy and SSO configuration (`domains`, `directory_config`,
`egress_proxy`, `sso_config`), and the plugin registries and grants (`plugins`,
`plugin_allowlist`, `ui_plugins`, `ui_plugin_grants`). These name the old deployment's
hosts, redirect URIs and network position, or are live session state, so the
destination starts empty and you re-configure them there. Plan for this: after cutting
over you will need to bootstrap an admin login, re-mint API keys, and re-approve OAuth
clients and plugins before those surfaces work again.

**Not yet decided — 20 tables.** These are *not* blessed as safe to drop; no decision
has been taken on them. They include per-account 2FA enrolments (`totp_secrets`,
`webauthn_credentials`, `recovery_codes`), user settings and content (`signatures`,
`notification_rules`, `passwd_config`, `remote_image_grants`, `masked_email`),
attachment upload metadata (`uploaded_blobs`), bridge and EWS account bindings, plugin
state, and the remaining append-only audit logs (`sso_login_audit`,
`password_change_audit`, `assist_audit`). **If your deployment relies on any of these,
`migrate-store` will lose them — check before you cut over.** The full list, with a
reason recorded per table, is `NOT_MIGRATED_UNCLASSIFIED` in
`crates/mw-store/tests/backend_parity.rs`.

Five tables moved from that list into the copied set in 26.20 because leaving them
behind lost data rather than deferring configuration: `zeroaccess_accounts` (the only
copy of each account's wrapped root key — without it the destination cannot decrypt
zero-access mail at all), `crypto_changes`, the `audit_log`, and `twofa_policy` and
`quotas`, whose absence silently relaxed a protection on the destination.

### What the tests check

Two gates in `crates/mw-store/tests/backend_parity.rs` assert against the live schema,
not against the migrator's own report, so a table or column the copier never mentions
still fails them:

- `migrate_store_accounts_for_every_schema_table` — every table in the schema must be
  copied or listed in one of the two lists above. A new migration fails it until
  someone classifies the table.
- `migrate_store_copies_every_column_of_every_copied_table` — every column of every
  copied table must appear in that table's copy spec.

Alongside them, the copy asserts row-count and content parity on a populated database
against a live `postgres:16` (`migrate-store-smoke`), and a table-driven backend-parity
suite runs every store repo method against both SQLite and Postgres and asserts
identical results (`store-dual-backend`).

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
