-- Lazily adopted renewable cache leases: no full body-table backfill at upgrade.
-- Missing leases are protected; adoption gives legacy cached messages a full TTL.
CREATE TABLE message_body_cache_lease (
 stable_id TEXT PRIMARY KEY REFERENCES messages(stable_id) ON DELETE CASCADE,
 account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
 blob_ref TEXT NOT NULL,
 uidvalidity BIGINT NOT NULL,
 uid BIGINT NOT NULL,
 last_used_at BIGINT NOT NULL
);
CREATE INDEX message_body_cache_lease_age ON message_body_cache_lease(last_used_at,stable_id);
CREATE INDEX submissions_cached_body_reference ON submissions(email_id);
CREATE INDEX account_draft_cached_body_reference ON account_draft(email_id);
CREATE INDEX account_draft_cached_body_cleanup ON account_draft(cleanup_email_id);
