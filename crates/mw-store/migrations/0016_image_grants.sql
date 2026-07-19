-- 0016 (26.16 t16): per-account remote-image display grants backing the
-- anonymizing image proxy's 4-grant model (single / all / per-sender / per-domain)
-- plus revocation and blocked-count reporting. ADDITIVE over 0001..0015 — NEVER edit
-- an earlier migration. This is the SQLite variant; the behaviourally-identical
-- Postgres variant is `migrations_pg/0016_image_grants.sql`.
--
-- INVARIANTS:
--   * A grant is a NON-secret row (no sealed column): it records that an account
--     chose to load remote images for a `scope_kind` at `scope_value`. Deny-by-
--     default — absence of a matching, non-revoked grant means images stay blocked.
--   * `scope_kind` ∈ {single, all, per-sender, per-domain}; `scope_value` carries the
--     message id (single), sender address (per-sender), or domain (per-domain), and
--     is '' for the account-wide `all` grant.
--   * `revoked` is the 0/1 column, INTEGER here / BIGINT in Postgres, i64-uniform
--     (the V6 lesson — NEVER native BOOLEAN). Revocation is soft (flip `revoked`) so
--     the audit trail of what was ever granted survives.
CREATE TABLE IF NOT EXISTS remote_image_grants (
    account_id   TEXT NOT NULL,
    scope_kind   TEXT NOT NULL,                 -- 'single' | 'all' | 'per-sender' | 'per-domain'
    scope_value  TEXT NOT NULL DEFAULT '',      -- message id / sender / domain; '' for 'all'
    granted_at   TEXT NOT NULL,
    revoked      INTEGER NOT NULL DEFAULT 0,    -- 0/1 (soft-revoked; kept for audit)
    PRIMARY KEY (account_id, scope_kind, scope_value)
);
CREATE INDEX IF NOT EXISTS idx_remote_image_grants_account ON remote_image_grants (account_id);
