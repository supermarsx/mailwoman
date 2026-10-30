-- Foreground activity is historical user metadata, not a session lifetime.
-- Keep it after logout/revocation; background requests never update this table.
CREATE TABLE user_activity (
    account_id TEXT PRIMARY KEY NOT NULL,
    last_seen_at TEXT NOT NULL
);
INSERT INTO user_activity(account_id,last_seen_at)
SELECT account_id,MAX(COALESCE(activity_at,created_at)) FROM sessions GROUP BY account_id;
