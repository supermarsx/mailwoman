-- 0029 (26.20 t24-e7, B3): bounded retries for a submission that has not been sent.
-- ADDITIVE over 0001..0028 — NEVER edit an earlier migration. SQLite variant; the
-- behaviourally-identical Postgres variant is `migrations_pg/0029_submission_attempts.sql`.
-- (0024 remains retired and must never be filled — see `tests/t22_migration_tombstone.rs`.)
--
-- WHY
-- Before this migration a submission had no memory of failing. The dispatcher left a
-- failed row `pending` and tried it again on its next 500 ms scan, forever. Combined
-- with a Sent-copy APPEND failure after SMTP had already accepted the message, that
-- delivered the same message to its recipients twice a second (t23 E4-07 / B3). The
-- engine now marks a submission `final` the moment SMTP accepts it, before filing, so
-- these columns only ever describe attempts in which NOTHING was delivered.
--
-- COLUMNS
-- * `attempts` — dispatch attempts that ended without SMTP accepting the message.
--   A real i64 counter (INTEGER here, BIGINT in Postgres). DEFAULT 0: an existing
--   row has not failed yet.
-- * `last_error` — the most recent failure, for the Outbox. On a `final` row it
--   records a failure to file the Sent copy; the message itself was delivered.
--   NULL = no error recorded.
-- * `next_attempt_at` — RFC3339; the dispatcher does not retry before it. NULL = no
--   backoff in force, so an existing `pending` row fires exactly as it did before.
--
-- `undo_status` gains the terminal value `failed` (the column stays opaque TEXT to the
-- store; the engine owns the vocabulary). No constraint change is needed.
ALTER TABLE submissions ADD COLUMN attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE submissions ADD COLUMN last_error TEXT;
ALTER TABLE submissions ADD COLUMN next_attempt_at TEXT;
