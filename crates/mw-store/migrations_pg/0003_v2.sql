-- V2 schema — POSTGRES variant (t6-e1; plan §1.1, §2.1). Behaviourally identical
-- to the SQLite `migrations/0003_v2.sql`. Dialect mapping: INTEGER → BIGINT
-- (boolean-as-0/1 columns stay BIGINT so the i64 bind/read path is uniform),
-- foreign keys DEFERRABLE INITIALLY IMMEDIATE.

CREATE TABLE IF NOT EXISTS message_meta (
    stable_id     TEXT PRIMARY KEY NOT NULL REFERENCES messages(stable_id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    pinned        BIGINT NOT NULL DEFAULT 0,
    snoozed_until TEXT,
    follow_up_at  TEXT
);
CREATE INDEX IF NOT EXISTS idx_meta_snoozed ON message_meta (snoozed_until);
CREATE INDEX IF NOT EXISTS idx_meta_followup ON message_meta (follow_up_at);

CREATE TABLE IF NOT EXISTS tags (
    id    TEXT PRIMARY KEY NOT NULL,
    "user" TEXT NOT NULL,
    name  TEXT NOT NULL,
    color TEXT NOT NULL,
    icon  TEXT
);

CREATE TABLE IF NOT EXISTS saved_searches (
    id         TEXT PRIMARY KEY NOT NULL,
    "user"     TEXT NOT NULL,
    name       TEXT NOT NULL,
    query_json TEXT NOT NULL,
    as_folder  BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS submissions (
    id           TEXT PRIMARY KEY NOT NULL,
    account_id   TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    email_id     TEXT NOT NULL,
    identity_id  TEXT,
    send_at      TEXT,
    undo_status  TEXT NOT NULL DEFAULT 'pending',
    hold_seconds BIGINT NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_submissions_pending
    ON submissions (undo_status, send_at);

CREATE TABLE IF NOT EXISTS identities (
    id              TEXT PRIMARY KEY NOT NULL,
    account_id      TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    name            TEXT NOT NULL DEFAULT '',
    email           TEXT NOT NULL,
    reply_to        TEXT,
    signature_html  TEXT,
    signature_text  TEXT,
    sent_mailbox_id TEXT,
    source          TEXT NOT NULL DEFAULT 'configured'
);

CREATE TABLE IF NOT EXISTS changes (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    type       TEXT NOT NULL,
    state      BIGINT NOT NULL,
    stable_id  TEXT NOT NULL,
    op         TEXT NOT NULL,
    at         TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_changes_account_type_state
    ON changes (account_id, type, state);
