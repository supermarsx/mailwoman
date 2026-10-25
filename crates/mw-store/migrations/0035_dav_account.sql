-- 0035 (26.20 f34 / t28-e19): CalDAV/CardDAV accounts, and the server's own href
-- for every synced calendar object, task and contact.
-- ADDITIVE over 0001..0034 — NEVER edit an earlier migration. SQLite variant; the
-- behaviourally-identical Postgres variant is `migrations_pg/0035_dav_account.sql`.
-- (0024 remains retired and must never be filled — see `tests/t22_migration_tombstone.rs`.)
--
-- `dav_account` — one row per mail account that syncs calendars and contacts with a
-- CalDAV/CardDAV server. The server's URLs are not stored here: they come from the
-- account's domain (its `dav` upstream), so an administrator who moves the DAV
-- server changes one domain, not every account.
-- * `use_mail_credentials` — 1: sign in to the DAV server with the account's mail
--   username and password. 0: `username` + `sealed_password` (sealed under the
--   server key) are used instead.
-- * `last_sync_at`, `last_error`, `failures`, `next_attempt_at` — the per-account
--   backoff the sync tick follows: a failing server is retried later and later
--   rather than every tick. Unix seconds.
--
-- `events.href`, `tasks.href`, `contacts.href` — the href the DAV server named the
-- resource with. Before 0035 the engine built `<collection>/<uid>.ics` itself, which
-- is wrong for every server that names resources another way, so a PUT made a
-- duplicate and a DELETE missed. NULL for local objects and rows written before 0035.
CREATE TABLE IF NOT EXISTS dav_account (
    account_id            TEXT PRIMARY KEY NOT NULL,
    enabled               INTEGER NOT NULL DEFAULT 1,
    use_mail_credentials  INTEGER NOT NULL DEFAULT 1,
    username              TEXT NOT NULL DEFAULT '',
    sealed_password       BLOB,
    last_sync_at          INTEGER,
    last_error            TEXT NOT NULL DEFAULT '',
    failures              INTEGER NOT NULL DEFAULT 0,
    next_attempt_at       INTEGER,
    created_at            INTEGER NOT NULL,
    updated_at            INTEGER NOT NULL
);

ALTER TABLE events ADD COLUMN href TEXT;
ALTER TABLE tasks ADD COLUMN href TEXT;
ALTER TABLE contacts ADD COLUMN href TEXT;
