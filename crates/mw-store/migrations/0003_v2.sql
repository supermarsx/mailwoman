-- V2 schema (plan §2.7). Additive over 0001_init.sql + 0002_v1_cache.sql; never
-- edit an earlier migration. Everything here is engine-local metadata IMAP
-- cannot hold (pins/snooze/follow-up, tag colors, saved searches), the real
-- submission queue (undo-send / send-later), sending identities, and the
-- per-account change log that backs real JMAP state tokens + `*/changes`.
--
-- Scaffolder note (e0): this is the skeleton. e9 fills the typed repo methods
-- (message_meta / tags / saved_searches / submissions / identities / changes
-- CRUD) and the stable-id-preserving `relocate_message` (plan §1.4).

-- Engine-local per-message metadata, keyed by the store's stable_id (§2.1).
-- pinned/snoozed/follow-up are surfaced as extra Email properties.
CREATE TABLE IF NOT EXISTS message_meta (
    stable_id     TEXT PRIMARY KEY NOT NULL REFERENCES messages(stable_id) ON DELETE CASCADE,
    pinned        INTEGER NOT NULL DEFAULT 0,
    snoozed_until TEXT,
    follow_up_at  TEXT
);
CREATE INDEX IF NOT EXISTS idx_meta_snoozed ON message_meta (snoozed_until);
CREATE INDEX IF NOT EXISTS idx_meta_followup ON message_meta (follow_up_at);

-- Per-user tag color/icon registry (plan §1.5). The label round-trips to IMAP
-- as a JMAP keyword; only the presentation metadata lives here.
CREATE TABLE IF NOT EXISTS tags (
    id    TEXT PRIMARY KEY NOT NULL,
    user  TEXT NOT NULL,
    name  TEXT NOT NULL,
    color TEXT NOT NULL,
    icon  TEXT
);

-- Saved searches surfaced as virtual search folders (§2.1).
CREATE TABLE IF NOT EXISTS saved_searches (
    id         TEXT PRIMARY KEY NOT NULL,
    user       TEXT NOT NULL,
    name       TEXT NOT NULL,
    query_json TEXT NOT NULL,
    as_folder  INTEGER NOT NULL DEFAULT 0
);

-- The persisted submission queue (plan §1.3): create=enqueue with a hold/
-- send-at, update=cancel; a single dispatcher fires SMTP when the window
-- elapses. `EmailSubmission/query` = the Outbox.
CREATE TABLE IF NOT EXISTS submissions (
    id           TEXT PRIMARY KEY NOT NULL,
    account_id   TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    email_id     TEXT NOT NULL,
    identity_id  TEXT,
    send_at      TEXT,
    undo_status  TEXT NOT NULL DEFAULT 'pending',
    hold_seconds INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL
);
-- The dispatcher scans pending rows due to fire.
CREATE INDEX IF NOT EXISTS idx_submissions_pending
    ON submissions (undo_status, send_at);

-- Sending identities (plan §0.7): configured + server-pulled allowed-froms.
-- `source` distinguishes 'configured' from server-'pulled'.
CREATE TABLE IF NOT EXISTS identities (
    id              TEXT PRIMARY KEY NOT NULL,
    account_id      TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    name            TEXT NOT NULL DEFAULT '',
    email           TEXT NOT NULL,
    reply_to        TEXT,
    signature_html  TEXT,
    signature_text  TEXT,
    sent_mailbox_id TEXT,
    source          TEXT NOT NULL DEFAULT 'configured'
);

-- Per-account monotonic change log (plan §1.2): the raw material for real
-- state tokens + Email/changes / Mailbox/changes / Email/queryChanges.
CREATE TABLE IF NOT EXISTS changes (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    type       TEXT NOT NULL,
    state      INTEGER NOT NULL,
    stable_id  TEXT NOT NULL,
    op         TEXT NOT NULL,
    at         TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_changes_account_type_state
    ON changes (account_id, type, state);
