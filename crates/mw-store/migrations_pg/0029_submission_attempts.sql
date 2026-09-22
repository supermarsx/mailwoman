-- 0029 (26.20 t24-e7, B3): bounded retries for a submission that has not been sent —
-- POSTGRES variant. Behaviourally identical to the SQLite
-- `migrations/0029_submission_attempts.sql`, whose header carries the full rationale;
-- dialect difference: INTEGER -> BIGINT for the counter, so the i64 bind/read path
-- stays uniform (never native BOOLEAN or INT4). ADDITIVE over 0001..0028 — NEVER edit
-- an earlier migration.
--
-- COLUMNS: `attempts` (dispatch attempts in which SMTP accepted nothing, DEFAULT 0),
-- `last_error` (NULL = none; on a `final` row, a Sent-copy filing failure),
-- `next_attempt_at` (RFC3339, NULL = no backoff in force). Existing rows keep firing
-- exactly as before.
ALTER TABLE submissions ADD COLUMN attempts BIGINT NOT NULL DEFAULT 0;
ALTER TABLE submissions ADD COLUMN last_error TEXT;
ALTER TABLE submissions ADD COLUMN next_attempt_at TEXT;
