-- Authentication history is bounded; policy identity is durable and separate.
ALTER TABLE signin_event ADD COLUMN surface TEXT NOT NULL DEFAULT 'webmail';
ALTER TABLE signin_event ADD COLUMN backend TEXT;
ALTER TABLE signin_event ADD COLUMN last_seen BIGINT NOT NULL DEFAULT 0;
ALTER TABLE signin_event ADD COLUMN repeat_count BIGINT NOT NULL DEFAULT 1;
ALTER TABLE signin_event ADD COLUMN aggregate_key TEXT;
UPDATE signin_event SET last_seen=ts;
CREATE UNIQUE INDEX idx_signin_event_aggregate ON signin_event(aggregate_key);
CREATE INDEX idx_signin_event_recent ON signin_event(last_seen DESC,id DESC);
CREATE INDEX idx_signin_event_surface_recent ON signin_event(surface,last_seen DESC,id DESC);
CREATE TABLE signin_identity (
    account_id TEXT PRIMARY KEY NOT NULL,
    address TEXT NOT NULL,
    seen_at BIGINT NOT NULL,
    event_id TEXT NOT NULL
);
CREATE INDEX idx_signin_identity_address ON signin_identity(lower(address),seen_at DESC,event_id DESC);
INSERT INTO signin_identity(account_id,address,seen_at,event_id)
SELECT account_id,username_attempted,ts,id FROM (
    SELECT account_id,username_attempted,ts,id,
           ROW_NUMBER() OVER(PARTITION BY account_id ORDER BY ts DESC,id DESC) AS position
    FROM signin_event WHERE result='success' AND account_id IS NOT NULL
) ranked WHERE position=1;
