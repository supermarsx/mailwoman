-- 0035 (26.20 f34 / t28-e19): CalDAV/CardDAV accounts, and the server's own href
-- for every synced calendar object, task and contact — POSTGRES variant.
-- Behaviourally identical to the SQLite `migrations/0035_dav_account.sql`, whose
-- header carries the full rationale. Dialect differences: INTEGER → BIGINT (the
-- uniform i64 bind/read path) and BLOB → BYTEA. ADDITIVE over 0001..0034 — NEVER
-- edit an earlier migration.
--
-- `dav_account`: per-account DAV sign-in (mail credentials, or a sealed separate
-- password) and the sync backoff; the server URLs come from the account's domain.
-- `events.href`, `tasks.href`, `contacts.href`: the href the DAV server named the
-- resource with (NULL for local objects).
CREATE TABLE IF NOT EXISTS dav_account (
    account_id            TEXT PRIMARY KEY NOT NULL,
    enabled               BIGINT NOT NULL DEFAULT 1,
    use_mail_credentials  BIGINT NOT NULL DEFAULT 1,
    username              TEXT NOT NULL DEFAULT '',
    sealed_password       BYTEA,
    last_sync_at          BIGINT,
    last_error            TEXT NOT NULL DEFAULT '',
    failures              BIGINT NOT NULL DEFAULT 0,
    next_attempt_at       BIGINT,
    created_at            BIGINT NOT NULL,
    updated_at            BIGINT NOT NULL
);

ALTER TABLE events ADD COLUMN href TEXT;
ALTER TABLE tasks ADD COLUMN href TEXT;
ALTER TABLE contacts ADD COLUMN href TEXT;
