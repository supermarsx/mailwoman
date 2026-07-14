-- V8 (SSO: OIDC + SAML 2.0 login backends) schema (t9 plan §2-§4, SPEC §18.3).
-- Additive over 0001..0008; NEVER edit an earlier migration. This is the SQLite
-- variant, run by `sqlx::migrate!("./migrations")`. The Postgres variant lives in
-- `migrations_pg/0009_v8_sso.sql`; keep the two behaviourally identical.
--
-- INVARIANTS (mirror 0008):
--   * BOOLEAN-as-0/1 columns use INTEGER here / BIGINT in Postgres so the i64
--     bind/read path is UNIFORM across dialects (V6 lesson — do NOT use native
--     BOOLEAN, which caused the 0007 bool-bind bug on Postgres).
--   * `secret_sealed` is a BLOB (BYTEA in Postgres) holding the OIDC client_secret
--     / SAML SP private key sealed via the store's `ServerKey` (the V6 sealed-column
--     pattern) — the plaintext secret never touches the DB.
--   * JSON (`config`, `claim_map`) is stored as TEXT (the store convention).
--   * `sso_login_audit` is APPEND-ONLY and CONTENT-FREE (§21.1): the IdP subject is
--     stored only as `subject_hash` (a HASH of the sub/NameID, NOT the raw value),
--     and NO tokens, NO assertions, NO mail content are ever written.

-- ── SSO backend config, per scope ('deployment' | 'domain:<d>'), managed by the
-- admin panel (like directory_config). One row per configured IdP.
CREATE TABLE IF NOT EXISTS sso_config (
    id             TEXT PRIMARY KEY NOT NULL,
    kind           TEXT NOT NULL,                      -- 'oidc' | 'saml'
    display_name   TEXT NOT NULL,
    scope          TEXT NOT NULL DEFAULT 'deployment', -- 'deployment' | 'domain:<d>'
    enabled        INTEGER NOT NULL DEFAULT 0,         -- 0/1
    config         TEXT NOT NULL DEFAULT '{}',         -- JSON: kind-specific config (NO secrets)
    secret_sealed  BLOB,                               -- sealed client_secret / SP key; NULL = none
    claim_map      TEXT NOT NULL DEFAULT '{}',         -- JSON: IdP claim/attr -> account mapping
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_sso_config_scope ON sso_config (scope);

-- ── SSO login audit — APPEND-ONLY, CONTENT-FREE (§21.1). subject_hash is a HASH of
-- the IdP subject/NameID, never the raw value; NO tokens/assertions/mail content.
CREATE TABLE IF NOT EXISTS sso_login_audit (
    id            TEXT PRIMARY KEY NOT NULL,
    ts            TEXT NOT NULL,
    provider_id   TEXT NOT NULL,                       -- sso_config.id
    kind          TEXT NOT NULL,                       -- 'oidc' | 'saml'
    subject_hash  TEXT NOT NULL,                       -- HASH of the sub/NameID; NEVER the raw value
    outcome       TEXT NOT NULL                        -- 'ok' | 'error:<reason>'
);
CREATE INDEX IF NOT EXISTS idx_sso_audit_ts ON sso_login_audit (ts);
