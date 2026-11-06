-- Short-lived, server-side ceremony state that has to survive a process restart
-- and be visible to every server instance sharing the database (for example a
-- single sign-on redirect that is still in flight when the server restarts).
-- The payload is sealed with the server key; the lookup key is stored hashed, so
-- the table alone yields neither the state token nor its contents.
CREATE TABLE IF NOT EXISTS ephemeral_state (
    kind        TEXT NOT NULL,
    key_hash    TEXT NOT NULL,
    sealed      BLOB NOT NULL,
    expires_at  INTEGER NOT NULL,
    PRIMARY KEY (kind, key_hash)
);
CREATE INDEX IF NOT EXISTS idx_ephemeral_state_expiry ON ephemeral_state(expires_at);
