-- 0017 (26.16 t16): per-account user preferences (signatures + notification rules /
-- quiet hours) — POSTGRES variant. Behaviourally identical to the SQLite
-- `migrations/0017_user_prefs.sql`; dialect difference: INTEGER → BIGINT (every
-- boolean-as-0/1 column stays BIGINT so the i64 bind/read path is UNIFORM, matching
-- 0001..0016 — NOT native BOOLEAN). TEXT stays TEXT. ADDITIVE — NEVER edit an earlier
-- migration.
--
-- NOTE ON saved_searches (W13): NOT declared here — it already exists FROZEN in 0003
-- (v2) with the exact W13 shape; W13 reuses that table (see the SQLite variant's note).
--
-- INVARIANTS: identical to the SQLite variant — non-secret preference rows (no sealed
-- columns); `is_default`/`enabled` are 0/1 flags; `*_json` are opaque app-owned TEXT.
CREATE TABLE IF NOT EXISTS signatures (
    account_id   TEXT NOT NULL,
    name         TEXT NOT NULL,
    body         TEXT NOT NULL DEFAULT '',
    is_default   BIGINT NOT NULL DEFAULT 0,
    rule_json    TEXT NOT NULL DEFAULT '',
    updated_at   TEXT NOT NULL,
    PRIMARY KEY (account_id, name)
);

CREATE TABLE IF NOT EXISTS notification_rules (
    account_id       TEXT PRIMARY KEY NOT NULL,
    rule_json        TEXT NOT NULL DEFAULT '',
    quiet_hours_json TEXT NOT NULL DEFAULT '',
    enabled          BIGINT NOT NULL DEFAULT 1,
    updated_at       TEXT NOT NULL
);
