-- 0030 (26.20 t28-e8): admin sessions expire — POSTGRES variant. Behaviourally
-- identical to the SQLite `migrations/0030_admin_session_expiry.sql`, whose header
-- carries the full rationale; dialect difference: INTEGER -> BIGINT, so the i64
-- bind/read path stays uniform. ADDITIVE over 0001..0029 — NEVER edit an earlier
-- migration.
--
-- COLUMNS (unix seconds): `expires_at` (idle deadline, moved forward on each accepted
-- read), `absolute_expires_at` (hard cap, set at login). DEFAULT 0: a row written
-- before this migration reads as already expired.
ALTER TABLE admin_sessions ADD COLUMN expires_at BIGINT NOT NULL DEFAULT 0;
ALTER TABLE admin_sessions ADD COLUMN absolute_expires_at BIGINT NOT NULL DEFAULT 0;
