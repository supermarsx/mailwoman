-- V8 (SSO: OIDC + SAML 2.0 login backends) schema — POSTGRES variant (t9 plan
-- §2-§4). Behaviourally identical to the SQLite `migrations/0009_v8_sso.sql`;
-- dialect differences: BLOB → BYTEA, INTEGER → BIGINT (boolean-as-0/1 columns stay
-- BIGINT so the i64 bind/read path is UNIFORM, matching 0001..0008 — NOT native
-- BOOLEAN, which caused the V6 0007 bool-bind bug). JSON stays TEXT.
--
-- INVARIANTS: identical to the SQLite variant — `secret_sealed` holds the
-- ServerKey-sealed secret (plaintext never touches the DB); `sso_login_audit` is
-- APPEND-ONLY + CONTENT-FREE with the subject stored only as a HASH.

-- ── SSO backend config, per scope ('deployment' | 'domain:<d>').
CREATE TABLE IF NOT EXISTS sso_config (
    id             TEXT PRIMARY KEY NOT NULL,
    kind           TEXT NOT NULL,
    display_name   TEXT NOT NULL,
    scope          TEXT NOT NULL DEFAULT 'deployment',
    enabled        BIGINT NOT NULL DEFAULT 0,
    config         TEXT NOT NULL DEFAULT '{}',
    secret_sealed  BYTEA,
    claim_map      TEXT NOT NULL DEFAULT '{}',
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_sso_config_scope ON sso_config (scope);

-- ── SSO login audit — APPEND-ONLY, CONTENT-FREE (§21.1). subject_hash only.
CREATE TABLE IF NOT EXISTS sso_login_audit (
    id            TEXT PRIMARY KEY NOT NULL,
    ts            TEXT NOT NULL,
    provider_id   TEXT NOT NULL,
    kind          TEXT NOT NULL,
    subject_hash  TEXT NOT NULL,
    outcome       TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_sso_audit_ts ON sso_login_audit (ts);
