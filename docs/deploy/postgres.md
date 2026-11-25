# PostgreSQL backend

Mailwoman keeps its store in SQLite by default. You can run it on PostgreSQL
instead by pointing the server at a PostgreSQL connection URL. This page covers
choosing the backend, the server key PostgreSQL needs, TLS to the database, and
moving an existing SQLite store with `mailwoman migrate-store`.

## Choose the backend

The store location comes from `MW_DB_PATH` (or `--db-path`). Its prefix selects
the backend:

| `MW_DB_PATH` value | Backend |
|---|---|
| `mailwoman.db` (a plain path, the default) | SQLite file |
| `sqlite:…` URL, for example `sqlite::memory:` | SQLite |
| `postgres://user:pass@host:5432/db` | PostgreSQL |
| `postgresql://…` | PostgreSQL |

PostgreSQL also needs a persistent server key, set with `MW_SERVER_KEY` or
`MW_SERVER_KEY_FILE`:

```sh
MW_DB_PATH="postgres://mailwoman:secret@db.internal:5432/mailwoman" \
MW_SERVER_KEY="$(openssl rand -hex 32)" \
mailwoman serve
```

The schema migrations run automatically when the server connects. SQLite and
PostgreSQL each have their own migration set (`crates/mw-store/migrations` and
`crates/mw-store/migrations_pg`) describing the same schema.

## Server key

The server key (32 bytes, hex-encoded) seals stored secrets such as mailbox
credentials and second-factor secrets.

- With SQLite and no key configured, Mailwoman creates a key file next to the
  database (`<database>.server-key`) on first start and reuses it.
- With PostgreSQL there is no such fallback. The server refuses to start with
  `PostgreSQL requires MW_SERVER_KEY or MW_SERVER_KEY_FILE shared by all server instances`.
- `MW_SERVER_KEY` takes precedence over `MW_SERVER_KEY_FILE`. If the file named by
  `MW_SERVER_KEY_FILE` does not exist, Mailwoman creates it (mode `600` on Unix).
  On Unix, an existing key file with any group or other permission bits is
  refused (`server key file must be private (chmod 600)`).

Every server instance that shares the database must use the same key. Back the
key up with the database: without it, the sealed columns cannot be opened.

::: warning
Do not replace a damaged key file with a new key. Mailwoman never rotates the key
on its own; restore the original key instead.
:::

## TLS to the database

The PostgreSQL driver uses rustls; the build does not link OpenSSL. Ask for TLS
with the standard `sslmode` parameter in the URL:

```
postgres://mailwoman:secret@db.internal:5432/mailwoman?sslmode=require
```

To make the server refuse a URL that does not force TLS, set
`MW_DB_REQUIRE_TLS=1` (also accepts `true`, `yes`, `on`). With it set, the URL
must use `sslmode=require`, `verify-ca` or `verify-full`, or the store does not
open.

## Move an existing installation with deploy.sh

If `deploy.sh` manages your installation, `bash deploy.sh deploy --full-stack`
moves it from SQLite to PostgreSQL in one step: it takes a backup, upgrades an
offline copy of the SQLite store, runs `migrate-store --complete`, and checks
health before switching over. See
[Full stack and one-step SQLite migration](script.md#full-stack-and-one-step-sqlite-migration).

## Copy a SQLite store with migrate-store

`mailwoman migrate-store` copies a SQLite store into a PostgreSQL database inside
one destination transaction, preserving ids, timestamps and sealed values byte for
byte. Sealed values are copied as opaque bytes and are not re-encrypted.

### Before you start

- Stop the server that writes to the SQLite file, or work on a copy of it.
- Create an empty PostgreSQL database for the destination.
- Set `MW_SERVER_KEY` to the hex key the SQLite installation uses. The command
  reads only `MW_SERVER_KEY` (not `MW_SERVER_KEY_FILE`) and stops with
  `MW_SERVER_KEY must be set (src + dst share the key)` without it. If the SQLite
  installation used the generated `<database>.server-key` file, use the key from
  that file.
- If the SQLite file is encrypted, set the same database key the server uses
  (`MW_DB_KEY`, `MW_DB_KEY_FILE` or `MW_DB_KEY_COMMAND`).

### Run the copy

```sh
export MW_SERVER_KEY=<the installation's key>
export MW_MIGRATE_TO="postgres://mailwoman:secret@db.internal:5432/mailwoman"
mailwoman migrate-store --from /var/lib/mailwoman/mailwoman.db --complete
```

On success it prints `migrated <rows> rows across <tables> tables; destination committed`.
Any error rolls the destination transaction back.

| Flag | Environment variable | What it does |
|---|---|---|
| `--from <path or sqlite: URL>` | `MW_MIGRATE_FROM` | Source SQLite store. Required. |
| `--to <postgres URL>` | `MW_MIGRATE_TO`, then `MW_DB_PATH` | Destination. Must start with `postgres://` or `postgresql://`. Passing it through an environment variable keeps the password out of the process list. |
| `--complete` | | Copy every table, for moving the same installation (see below). |
| `--upgrade-source` | | Bring the source's schema up to date before copying. Requires `--complete`. It writes to the source file, so use it only on an offline copy. |

### Complete mode (same installation)

With `--complete`, every table except the migration ledger is copied, including
admin accounts, OAuth registrations, API keys, webhooks, domains, SSO and
plugin grants. The command:

- refuses a destination that already holds data (`complete migration requires an empty destination`);
- refuses when the source and destination table sets differ
  (`source and destination schemas differ; upgrade the offline source copy first`);
  add `--upgrade-source` on an offline copy to fix this;
- compares the copied rows with the source before it commits.

Use this mode when you move your own installation to PostgreSQL.

### Default mode (into another installation)

Without `--complete`, the command copies a fixed list of tables and leaves out
those tied to the old deployment's identity or network position. It does not
check that the destination is empty, so point it at a fresh database.

Left out:

- admin console accounts and sessions;
- OAuth clients and tokens, and the configuration, signing keys, consents and
  tokens of the built-in OpenID provider;
- API keys, outgoing webhooks, incoming webhooks and their replay cache;
- managed domains and their aliases and hosts, directory, egress proxy, SSO, SSH
  tunnel and integration configuration;
- plugin registries and their capability grants (a reinstalled plugin starts with
  no capabilities);
- derived state the destination rebuilds itself, such as search-index journals
  and change-publication records.

After switching over in this mode, plan to create an admin login, issue new API
keys, register webhooks again, approve OAuth clients and plugins again, and
re-enter domain routing. Masked-email aliases are copied, but they are not
delivered until the domain routing exists again.

Copied, among others: accounts and their settings and content, zero-access
wrapped keys, TOTP secrets, passkeys and recovery codes, the 2FA policy, quotas,
the audit logs, sealed account and bridge credentials, upload metadata, message
embeddings and remote-image grants. The full list is `TABLES` in
`crates/mw-store/src/migrate.rs`; each table left out has its reason in
`NOT_MIGRATED_DELIBERATELY` in `crates/mw-store/tests/backend_parity.rs`.

### What lives outside the database

- **Uploaded attachments.** Upload metadata is in the database, but the sealed
  files are in the directory set by `MW_UPLOAD_DIR`. `migrate-store` does not
  touch them. If the new server uses a different directory or host, copy that
  directory across too.
- **Passkeys and the domain.** Passkeys are bound to the site's domain. They keep
  working when only the database changes. If you also change the domain, users
  must register their passkeys again; TOTP and recovery codes are not affected.

## Connections

Each Mailwoman process opens a bootstrap pool of 5 connections for migrations,
then replaces it with its main pool: `cpus × 4` connections clamped to 4–64
(application CPUs), capped by the `database.pool` budget (default 100) and the
server's `max_connections` minus 20 reserved, divided by `replicaCount`. Add
about 8 dedicated connections per process (authority mirror, maintenance jobs).
When several Mailwoman instances share one database, size the PostgreSQL
server's `max_connections` for all of them; [PostgreSQL tuning](../admin/postgresql-tuning.md)
gives the rule, the other server settings, and why a transaction-mode pooler
is not supported. A shared database does not
make the other parts of Mailwoman multi-instance aware: read
[scaling and multi-location boundaries](scaling.md) before adding replicas.

## Change-state collisions from earlier releases

Releases from 26.10 until this fix could give two concurrent changes for one
account the same change state on PostgreSQL (mail, PIM and crypto/rule change
logs). A client that synced at that state could miss the second change; SQLite
installations were not affected. Rows written in one batch share one timestamp,
so a state whose rows carry more than one timestamp is a collision. This
read-only query lists them (it cannot find collisions written within the same
timestamp):

```sql
SELECT 'changes' AS log, account_id, type, state, COUNT(DISTINCT at) AS stamps
  FROM changes GROUP BY account_id, type, state HAVING COUNT(DISTINCT at) > 1
UNION ALL
SELECT 'pim_changes', account_id, type, state, COUNT(DISTINCT at)
  FROM pim_changes GROUP BY account_id, type, state HAVING COUNT(DISTINCT at) > 1
UNION ALL
SELECT 'crypto_changes', account_id, type, state, COUNT(DISTINCT at)
  FROM crypto_changes GROUP BY account_id, type, state HAVING COUNT(DISTINCT at) > 1;
```

No repair is needed in the database. An affected client receives the missed
item again the next time that item changes, or on its next full resync, when it
discards its cached state and fetches the account again.

## When to use PostgreSQL

- **SQLite**: single-user and small installations, evaluation, the desktop app.
  No separate database server.
- **PostgreSQL**: you already run PostgreSQL, or you want its backup and
  replication tools.

## Related

- [PostgreSQL tuning](../admin/postgresql-tuning.md)
- [Deployment script](script.md)
- [Backups](backups.md)
- [Scaling and multi-location boundaries](scaling.md)
- [Shared cache (Redis/Valkey)](cache.md)

## Advisory locks used by the server

Advisory lock keys are `hashtextextended(<text>, <class>)`. Reserved classes:

| Class | Lock | Text |
|---|---|---|
| 719304 | mail change log (transaction) | account id |
| 719305 | PIM change log (transaction) | account id |
| 719306 | crypto change log (transaction) | account id |
| 719307 | request-protection witness, one session per server process (shared; momentarily exclusive during an ownership check) | `current_database():current_schema():<id derived from the server key>` |
| 719308 | request-protection witness admission (transaction) | as 719307 |

The witness text contains the database, the schema and a value derived from the installation's server key, so two installations in one cluster, or a test and a production schema in one database, never share a key. Each server process keeps one extra connection (`application_name` `mailwoman-protection-witness`) for the witness, outside its pool. Write-behind accounting does not own through a transaction-mode connection pooler; connect the server directly or through a session-mode pooler.

