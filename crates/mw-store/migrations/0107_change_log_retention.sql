-- Lazy checkpoints preserve monotonic tokens when old change batches expire.
-- No scan or backfill of an existing, potentially very large, change log.
CREATE TABLE change_log_checkpoint (
 account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
 type TEXT NOT NULL,
 head_state BIGINT NOT NULL,
 pruned_through BIGINT NOT NULL,
 PRIMARY KEY(account_id,type)
);
