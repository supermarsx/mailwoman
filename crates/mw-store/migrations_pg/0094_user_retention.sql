-- User retention rules for folders and mail. A rule row keeps the columns the
-- executor orders and filters on; the match, the action and the exclusions are in
-- body_json. A run row keeps counts only, never a subject, sender or address.
CREATE TABLE user_retention_rule (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    enabled BIGINT NOT NULL,
    sort_order BIGINT NOT NULL,
    template TEXT,
    body_json TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);
CREATE INDEX user_retention_rule_account ON user_retention_rule(account_id, sort_order);

CREATE TABLE user_retention_account (
    account_id TEXT PRIMARY KEY NOT NULL,
    enabled BIGINT NOT NULL,
    trash_first BIGINT NOT NULL,
    undo_days BIGINT NOT NULL,
    paused_until BIGINT NOT NULL,
    folder_defaults_json TEXT NOT NULL,
    last_run_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);

CREATE TABLE user_retention_run (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL,
    started_at BIGINT NOT NULL,
    finished_at BIGINT NOT NULL,
    dry_run BIGINT NOT NULL,
    triggered_by TEXT NOT NULL,
    matched BIGINT NOT NULL,
    acted BIGINT NOT NULL,
    errors BIGINT NOT NULL,
    counts_json TEXT NOT NULL
);
CREATE INDEX user_retention_run_account ON user_retention_run(account_id, started_at);
CREATE INDEX user_retention_run_time ON user_retention_run(started_at);
