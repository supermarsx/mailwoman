-- Separate genuine foreground interaction from authenticated request telemetry.
-- Existing sessions retain their prior idle baseline; deployment must not grant
-- a fresh lifetime to old credentials. Absolute age remains created_at.
ALTER TABLE sessions ADD COLUMN activity_at TEXT;
UPDATE sessions SET activity_at = last_seen;
