-- V0 init schema — POSTGRES variant (t6-e1; plan §1.1, §2.1). Behaviourally
-- identical to the SQLite `migrations/0001_init.sql`. Dialect mapping: BLOB →
-- BYTEA. Timestamps stay TEXT (RFC 3339) to match the store convention and keep
-- `migrate-store` a straight value copy.

CREATE TABLE IF NOT EXISTS sessions (
    id            TEXT PRIMARY KEY NOT NULL,
    account_id    TEXT NOT NULL,
    username      TEXT NOT NULL,
    jmap_url      TEXT NOT NULL,
    api_url       TEXT NOT NULL,
    sealed_creds  BYTEA NOT NULL,
    created_at    TEXT NOT NULL,
    last_seen     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key    TEXT PRIMARY KEY NOT NULL,
    value  TEXT NOT NULL
);
