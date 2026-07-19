-- 0016 (26.16 t16): per-account remote-image display grants (single / all /
-- per-sender / per-domain + revoke + blocked-count) — POSTGRES variant.
-- Behaviourally identical to the SQLite `migrations/0016_image_grants.sql`; dialect
-- differences: INTEGER → BIGINT (the boolean-as-0/1 `revoked` column stays BIGINT so
-- the i64 bind/read path is UNIFORM, matching 0001..0015 — NOT native BOOLEAN). TEXT
-- stays TEXT. ADDITIVE — NEVER edit an earlier migration.
--
-- INVARIANTS: identical to the SQLite variant — a grant is a non-secret deny-by-
-- default row; `scope_kind` ∈ {single, all, per-sender, per-domain}; `revoked` is a
-- soft flag kept for audit.
CREATE TABLE IF NOT EXISTS remote_image_grants (
    account_id   TEXT NOT NULL,
    scope_kind   TEXT NOT NULL,
    scope_value  TEXT NOT NULL DEFAULT '',
    granted_at   TEXT NOT NULL,
    revoked      BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, scope_kind, scope_value)
);
CREATE INDEX IF NOT EXISTS idx_remote_image_grants_account ON remote_image_grants (account_id);
