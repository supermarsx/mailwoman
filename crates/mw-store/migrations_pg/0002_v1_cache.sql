-- V1 message-cache schema — POSTGRES variant (t6-e1; plan §1.1, §2.1).
-- Behaviourally identical to the SQLite `migrations/0002_v1_cache.sql`. Dialect
-- mapping: INTEGER → BIGINT (the store binds/reads these as i64, including the
-- boolean-as-0/1 columns), BLOB → BYTEA. Foreign keys are DEFERRABLE (INITIALLY
-- IMMEDIATE — identical runtime behaviour to SQLite's sqlx-default
-- `foreign_keys=ON`) so `migrate-store` can copy the whole graph in one
-- constraint-deferred transaction regardless of row order.

CREATE TABLE IF NOT EXISTS accounts (
    id               TEXT PRIMARY KEY NOT NULL,
    kind             TEXT NOT NULL CHECK (kind IN ('imap', 'pop3')),
    host             TEXT NOT NULL,
    port             BIGINT NOT NULL,
    tls              TEXT NOT NULL,
    username         TEXT NOT NULL,
    sealed_creds     BYTEA NOT NULL,
    sync_policy_json TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS mailboxes (
    id             TEXT PRIMARY KEY NOT NULL,
    account_id     TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    name           TEXT NOT NULL,
    role           TEXT,
    uidvalidity    BIGINT NOT NULL DEFAULT 0,
    uidnext        BIGINT NOT NULL DEFAULT 0,
    highestmodseq  BIGINT NOT NULL DEFAULT 0,
    total          BIGINT NOT NULL DEFAULT 0,
    unread         BIGINT NOT NULL DEFAULT 0,
    parent_id      TEXT REFERENCES mailboxes(id) ON DELETE SET NULL DEFERRABLE INITIALLY IMMEDIATE,
    UNIQUE (account_id, name, uidvalidity)
);

CREATE TABLE IF NOT EXISTS messages (
    stable_id      TEXT PRIMARY KEY NOT NULL,
    account_id     TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    mailbox_id     TEXT NOT NULL REFERENCES mailboxes(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    uid            BIGINT NOT NULL,
    uidvalidity    BIGINT NOT NULL,
    message_id     TEXT,
    thread_id      TEXT,
    internaldate   TEXT,
    size           BIGINT NOT NULL DEFAULT 0,
    flags_json     TEXT NOT NULL DEFAULT '[]',
    envelope_json  BYTEA,
    blob_ref       TEXT,
    UNIQUE (account_id, mailbox_id, uidvalidity, uid)
);

CREATE INDEX IF NOT EXISTS idx_messages_mailbox_date ON messages (mailbox_id, internaldate DESC);
CREATE INDEX IF NOT EXISTS idx_messages_thread ON messages (thread_id);
CREATE INDEX IF NOT EXISTS idx_messages_message_id ON messages (account_id, message_id);
CREATE INDEX IF NOT EXISTS idx_messages_identity
    ON messages (account_id, mailbox_id, message_id, internaldate, size);

CREATE TABLE IF NOT EXISTS bodies (
    blob_ref     TEXT PRIMARY KEY NOT NULL,
    account_id   TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    sealed_bytes BYTEA NOT NULL
);

CREATE TABLE IF NOT EXISTS threads (
    thread_id       TEXT PRIMARY KEY NOT NULL,
    account_id      TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    root_message_id TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_threads_root
    ON threads (account_id, root_message_id);

CREATE TABLE IF NOT EXISTS pop3_uidl (
    account_id  TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    uidl        TEXT NOT NULL,
    stable_id   TEXT NOT NULL,
    PRIMARY KEY (account_id, uidl)
);

CREATE TABLE IF NOT EXISTS sync_state (
    account_id   TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    mailbox_id   TEXT NOT NULL REFERENCES mailboxes(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    cursor_json  TEXT NOT NULL DEFAULT '{}',
    last_sync_at TEXT,
    PRIMARY KEY (account_id, mailbox_id)
);
