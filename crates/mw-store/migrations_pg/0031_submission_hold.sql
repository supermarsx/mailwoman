-- 0031 (26.20 t28-e12): a submission can be held until a person releases it —
-- POSTGRES variant. Behaviourally identical to the SQLite
-- `migrations/0031_submission_hold.sql`, whose header carries the full rationale;
-- no dialect difference (three nullable TEXT columns). ADDITIVE over 0001..0030 —
-- NEVER edit an earlier migration.
--
-- COLUMNS: `hold` (NULL = due by time; `manual` = never sent until released),
-- `origin` (JSON naming a non-owner creator, NULL = the owner's client),
-- `on_success` (JSON: the RFC 8621 §7.5 onSuccess patch, applied at send time).
-- Existing rows keep firing exactly as before.
ALTER TABLE submissions ADD COLUMN hold TEXT;
ALTER TABLE submissions ADD COLUMN origin TEXT;
ALTER TABLE submissions ADD COLUMN on_success TEXT;
