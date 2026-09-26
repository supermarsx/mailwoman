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

**It does not copy the whole database, but it now copies nearly all of it.** As of
migration 0029 it copies 59 of the 76 tables the schema defines. The other 17 are left
behind deliberately, listed below. There is no longer an undecided remainder.

### What is not copied — 17 tables, all deliberate

The admin panel's own identity domain (`admin_users`, `admin_sessions`); OAuth clients
and tokens (`oauth_clients`, `oauth_tokens`, `oauth_client_meta`, `oauth_dcr`);
`api_keys`; `webhooks`; the managed-domain, directory, egress-proxy and SSO
configuration (`domains`, `directory_config`, `egress_proxy`, `sso_config`); and the
plugin registries and their capability grants (`plugins`, `plugin_allowlist`,
`ui_plugins`, `plugin_grants`, `ui_plugin_grants`).

These either name the old deployment's hosts and redirect URIs, or are live session
state, or are deny-by-default surfaces an operator re-approves. **Plan for this:** after
cutting over you will need to bootstrap an admin login, re-mint API keys, re-register
webhooks, and re-approve OAuth clients and plugins before those surfaces work again.
Each table's reason is recorded at its entry in `NOT_MIGRATED_DELIBERATELY` in
`crates/mw-store/tests/backend_parity.rs`.

The two grant tables deserve a word: `plugin_grants` and `ui_plugin_grants` are **not**
copied on purpose, because the plugin registries they refer to are not copied either.
Copying a grant would silently re-arm a capability for a plugin the admin has not
re-approved on the new deployment. As it is, a re-installed plugin starts with no
capabilities until someone grants them again.

### What the copy carries that you may not expect

Everything else, including the things a store move most obviously must not lose: the
zero-access wrapped root keys, every 2FA enrolment and the policy that requires them,
all four append-only audit logs, sealed account and bridge credentials, per-account
settings and user-authored content, message embeddings, plugin state, and the Assist and
cache configuration. A spent recovery code stays spent, a revoked remote-image grant
stays revoked, and the TOTP replay counter travels with the secret, so migrating never
re-opens something the user or operator had closed.

### Two things that live outside the database

**Uploaded attachment objects.** `uploaded_blobs` rows carry the `storage_key` that
locates each sealed object, but the objects themselves live on the upload backend (a
filesystem directory, or S3), not in the database. `migrate-store` moves the database
only. If you also change host or upload directory, **copy the upload store across as
well**, or the metadata will point at objects that are not there.

**Passkeys and the deployment domain.** WebAuthn credentials are bound to the
deployment's Relying Party ID — its domain. `migrate-store` changes the database
backend, not the domain, so passkeys keep working across a normal cutover. If you also
move to a **different domain**, enrolled passkeys stop verifying there; that is true
whether or not the rows are copied, and users must re-enrol. TOTP secrets and recovery
codes are unaffected by a domain change.

**Masked-email aliases** are copied, but alias *delivery* also depends on the `domains`
routing configuration, which is not copied. Re-enter that routing on the new deployment
or the aliases will be listed but undeliverable.

### What the tests check

Two gates in `crates/mw-store/tests/backend_parity.rs` assert against the live schema,
not against the migrator's own report, so a table or column the copier never mentions
still fails them:

- `migrate_store_accounts_for_every_schema_table` — every table in the schema must be
  either copied or listed as deliberately not copied. A new migration fails it until
  someone classifies the table.
- `migrate_store_copies_every_column_of_every_copied_table` — every column of every
  copied table must appear in that table's copy spec, so a later `ALTER TABLE … ADD
  COLUMN` cannot drift away from the copier unnoticed.
- `migrate_store_carries_zero_access_and_policy_rows_sqlite`, and the same assertions on
  the Postgres destination, require the copied surfaces to be *usable* and not merely
  present: the wrapped root key, the TOTP secret, the EWS and bridge credentials and the
  plugin state must all still open under the destination's key, and a recovery code must
  still be accepted.

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
