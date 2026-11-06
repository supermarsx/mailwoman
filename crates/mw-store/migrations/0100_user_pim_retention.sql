-- Retention for a person's calendar events and notes. One row per opt-in
-- (kind calendar or notes, config_json) and one per note held for the undo
-- window (kind noteHold, the whole note sealed in payload, due_at = purge time).
CREATE TABLE user_pim_retention (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    item_ref TEXT NOT NULL DEFAULT '',
    config_json TEXT NOT NULL DEFAULT '{}',
    payload BLOB NOT NULL,
    due_at BIGINT NOT NULL DEFAULT 0,
    updated_at BIGINT NOT NULL
);
CREATE INDEX user_pim_retention_account ON user_pim_retention(account_id, kind);
CREATE INDEX user_pim_retention_due ON user_pim_retention(kind, due_at);
