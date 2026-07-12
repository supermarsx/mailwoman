-- V1 message-cache schema (plan §2.3). Additive over 0001_init.sql; never edit
-- 0001. Bodies/envelopes are sealed with the existing mw-store seal (SPEC §7 —
-- encrypted at rest; NOT zero-access, which is V6).
--
-- Scaffolder note (e0): this is the skeleton. e5 refines column types/indexes
-- as it adds the typed repo methods (accounts/mailboxes/messages/bodies/
-- threads/pop3_uidl/sync_state CRUD + stable-id allocation + UID-map lookups).

-- Configured IMAP/POP3 accounts. `sealed_creds` reuses the 0001 seal.
CREATE TABLE IF NOT EXISTS accounts (
    id               TEXT PRIMARY KEY NOT NULL,
    kind             TEXT NOT NULL CHECK (kind IN ('imap', 'pop3')),
    host             TEXT NOT NULL,
    port             INTEGER NOT NULL,
    tls              TEXT NOT NULL,
    username         TEXT NOT NULL,
    sealed_creds     BLOB NOT NULL,
    sync_policy_json TEXT NOT NULL DEFAULT '{}'
);

-- Mailboxes/folders with special-use role + per-folder sync counters.
CREATE TABLE IF NOT EXISTS mailboxes (
    id             TEXT PRIMARY KEY NOT NULL,
    account_id     TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    name           TEXT NOT NULL,
    role           TEXT,
    uidvalidity    INTEGER NOT NULL DEFAULT 0,
    uidnext        INTEGER NOT NULL DEFAULT 0,
    highestmodseq  INTEGER NOT NULL DEFAULT 0,
    total          INTEGER NOT NULL DEFAULT 0,
    unread         INTEGER NOT NULL DEFAULT 0,
    parent_id      TEXT REFERENCES mailboxes(id) ON DELETE SET NULL,
    UNIQUE (account_id, name, uidvalidity)
);

-- Cached message headers/envelope keyed by (account, mailbox, UIDVALIDITY, UID).
-- `stable_id` is the store-assigned opaque JMAP Email.id (plan §1.6).
CREATE TABLE IF NOT EXISTS messages (
    stable_id      TEXT PRIMARY KEY NOT NULL,
    account_id     TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    mailbox_id     TEXT NOT NULL REFERENCES mailboxes(id) ON DELETE CASCADE,
    uid            INTEGER NOT NULL,
    uidvalidity    INTEGER NOT NULL,
    message_id     TEXT,
    thread_id      TEXT,
    internaldate   TEXT,
    size           INTEGER NOT NULL DEFAULT 0,
    flags_json     TEXT NOT NULL DEFAULT '[]',
    envelope_json  BLOB,
    blob_ref       TEXT,
    UNIQUE (account_id, mailbox_id, uidvalidity, uid)
);

-- List/sort by receivedAt (Email/query default sort is internaldate desc).
CREATE INDEX IF NOT EXISTS idx_messages_mailbox_date ON messages (mailbox_id, internaldate DESC);
CREATE INDEX IF NOT EXISTS idx_messages_thread ON messages (thread_id);
CREATE INDEX IF NOT EXISTS idx_messages_message_id ON messages (account_id, message_id);
-- UIDVALIDITY-change stable-id preservation: match a re-synced message to its
-- prior row by (message-id, internaldate, size) within the same mailbox (§1.6).
CREATE INDEX IF NOT EXISTS idx_messages_identity
    ON messages (account_id, mailbox_id, message_id, internaldate, size);

-- Sealed raw/parsed bodies, referenced by messages.blob_ref.
CREATE TABLE IF NOT EXISTS bodies (
    blob_ref     TEXT PRIMARY KEY NOT NULL,
    account_id   TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    sealed_bytes BLOB NOT NULL
);

-- Engine-side JWZ thread roots (plan §1.7). A thread is identified per account
-- by its root Message-ID, so lookups by (account, root) must be unique.
CREATE TABLE IF NOT EXISTS threads (
    thread_id       TEXT PRIMARY KEY NOT NULL,
    account_id      TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    root_message_id TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_threads_root
    ON threads (account_id, root_message_id);

-- POP3 UIDL -> stable-id map (POP3 has no UIDs; UIDL is the identity).
CREATE TABLE IF NOT EXISTS pop3_uidl (
    account_id  TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    uidl        TEXT NOT NULL,
    stable_id   TEXT NOT NULL,
    PRIMARY KEY (account_id, uidl)
);

-- Persisted per-mailbox sync cursor (SyncCursor as JSON) + last sync time.
CREATE TABLE IF NOT EXISTS sync_state (
    account_id   TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    mailbox_id   TEXT NOT NULL REFERENCES mailboxes(id) ON DELETE CASCADE,
    cursor_json  TEXT NOT NULL DEFAULT '{}',
    last_sync_at TEXT,
    PRIMARY KEY (account_id, mailbox_id)
);
