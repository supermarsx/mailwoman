-- 0017 (26.16 t16): per-account user preferences — signature templates/rules and
-- notification rules + quiet hours. ADDITIVE over 0001..0016 — NEVER edit an earlier
-- migration. This is the SQLite variant; the behaviourally-identical Postgres variant
-- is `migrations_pg/0017_user_prefs.sql`.
--
-- NOTE ON saved_searches (W13): the plan grouped saved-searches persistence here, but
-- a `saved_searches` table ALREADY EXISTS FROZEN in 0003 (v2) with the exact W13 shape
-- (id, user, name, query_json, as_folder) and full CRUD in `v2.rs`. Re-declaring it
-- would collide (CREATE TABLE IF NOT EXISTS silently skips) or violate the frozen-
-- migration rule, so W13 reuses the 0003 table and 0017 adds only the two net-new
-- preference tables below.
--
-- INVARIANTS:
--   * These rows are NON-secret user preferences (no sealed columns): signature
--     bodies, JSON rule blobs, and quiet-hours windows are not credentials.
--   * Every boolean is the 0/1 column `is_default`/`enabled`, INTEGER here / BIGINT in
--     Postgres, i64-uniform (the V6 lesson — NEVER native BOOLEAN).
--   * `*_json` columns hold app-owned JSON text (rule/quiet-hours structure lives
--     app-side); the store treats them as opaque TEXT.

-- Zero-or-more signature templates per account; at most one is the default (enforced
-- app-side). `rule_json` carries optional auto-apply rules (per-identity/context).
CREATE TABLE IF NOT EXISTS signatures (
    account_id   TEXT NOT NULL,
    name         TEXT NOT NULL,
    body         TEXT NOT NULL DEFAULT '',
    is_default   INTEGER NOT NULL DEFAULT 0,    -- 0/1 (the account's default signature)
    rule_json    TEXT NOT NULL DEFAULT '',      -- opaque JSON: auto-apply rules
    updated_at   TEXT NOT NULL,
    PRIMARY KEY (account_id, name)
);

-- One notification configuration per account: the rule set + quiet-hours window +
-- an enabled switch. `rule_json` / `quiet_hours_json` are opaque app-owned JSON.
CREATE TABLE IF NOT EXISTS notification_rules (
    account_id       TEXT PRIMARY KEY NOT NULL,
    rule_json        TEXT NOT NULL DEFAULT '',  -- opaque JSON: per-account notification rules
    quiet_hours_json TEXT NOT NULL DEFAULT '',  -- opaque JSON: quiet-hours windows
    enabled          INTEGER NOT NULL DEFAULT 1,-- 0/1 (notifications active)
    updated_at       TEXT NOT NULL
);
