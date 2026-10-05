-- 0031 (26.20 t28-e12): a submission can be held until a person releases it.
-- ADDITIVE over 0001..0030 — NEVER edit an earlier migration. SQLite variant; the
-- behaviourally-identical Postgres variant is `migrations_pg/0031_submission_hold.sql`.
-- (0024 remains retired and must never be filled — see `tests/t22_migration_tombstone.rs`.)
--
-- WHY
-- Every `pending` submission was due at a time: its `send_at`, or `created_at +
-- hold_seconds`. There was no state for "do not send this until someone says so",
-- so an MCP `mail.send` from a key without unattended send, which is documented as
-- waiting for the mailbox owner, was transmitted at once (t26 audit OH-5).
--
-- COLUMNS (all TEXT, all NULL on existing rows, which therefore behave as before)
-- * `hold` — NULL = the row is due by time, as before. `manual` = the dispatcher
--   never sends it; `Store::release_submission` clears the column and the timers.
--   Opaque to the store; the engine treats any non-NULL value as held.
-- * `origin` — JSON naming what created the submission when it was not the mailbox
--   owner's own client (`{"kind":"apiKey","name":"<key prefix>"}`). NULL = the
--   owner's client. Shown in the Outbox beside a held row.
-- * `on_success` — JSON: the `onSuccessUpdateEmail` patch and/or the
--   `onSuccessDestroyEmail` flag of the `EmailSubmission/set` call (RFC 8621 §7.5),
--   kept so it can be applied when the message is actually sent, which for a held
--   or delayed submission is later than the call. NULL = none was given.
ALTER TABLE submissions ADD COLUMN hold TEXT;
ALTER TABLE submissions ADD COLUMN origin TEXT;
ALTER TABLE submissions ADD COLUMN on_success TEXT;
