-- V7 schema — POSTGRES variant (plan §2.7). Behaviourally identical to the SQLite
-- `migrations/0008_v7.sql`; dialect differences: BLOB → BYTEA, INTEGER → BIGINT
-- (boolean-as-0/1 columns stay BIGINT so the i64 bind/read path is UNIFORM, matching
-- 0001..0007 — NOT native BOOLEAN, which caused the V6 0007 bool-bind bug). JSON stays
-- TEXT (the store convention) so the SQLite→Postgres `migrate-store` copy is a
-- straight value move.
--
-- Scaffolder note (e0): this is the 0008 SKELETON for the Postgres backend. Nothing
-- runs it yet (SQLite stays the default); it is inert scaffolding for the store owners.
--
-- INVARIANTS: identical to the SQLite variant — signed-registry signatures, CONTENT-
-- FREE append-only audit tables, `account_id` columns are NOT foreign keys.

-- ── WASM plugin registry (§22).
CREATE TABLE IF NOT EXISTS plugins (
    id              TEXT PRIMARY KEY NOT NULL,
    name            TEXT NOT NULL,
    version         TEXT NOT NULL,
    signature       BYTEA,
    approved_by     TEXT,
    enabled         BIGINT NOT NULL DEFAULT 0,
    capabilities    TEXT NOT NULL DEFAULT '[]',
    net_allowlist   TEXT NOT NULL DEFAULT '[]',
    limits          TEXT NOT NULL DEFAULT '{}',
    created_at      TEXT NOT NULL
);

-- ── Per-plugin capability grants (§22).
CREATE TABLE IF NOT EXISTS plugin_grants (
    plugin_id    TEXT NOT NULL,
    account_id   TEXT NOT NULL DEFAULT '',
    capability   TEXT NOT NULL,
    granted_by   TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    PRIMARY KEY (plugin_id, account_id, capability)
);
CREATE INDEX IF NOT EXISTS idx_plugin_grants_plugin ON plugin_grants (plugin_id);

-- ── LDAP/GAL directory endpoints (§13).
CREATE TABLE IF NOT EXISTS directory_config (
    id          TEXT PRIMARY KEY NOT NULL,
    url         TEXT NOT NULL,
    base_dn     TEXT NOT NULL,
    bind_dn     TEXT,
    tls         TEXT NOT NULL,
    priority    BIGINT NOT NULL DEFAULT 0,
    attr_map    TEXT NOT NULL DEFAULT '{}',
    enabled     BIGINT NOT NULL DEFAULT 1
);

-- ── Assist config (§14).
CREATE TABLE IF NOT EXISTS assist_config (
    scope             TEXT PRIMARY KEY NOT NULL,
    adapters          TEXT NOT NULL DEFAULT '[]',
    capability_grants TEXT NOT NULL DEFAULT '[]',
    data_ceilings     TEXT NOT NULL DEFAULT '{}',
    enabled           BIGINT NOT NULL DEFAULT 0
);

-- ── Assist audit — APPEND-ONLY, CONTENT-FREE (§14, R4).
CREATE TABLE IF NOT EXISTS assist_audit (
    id             TEXT PRIMARY KEY NOT NULL,
    ts             TEXT NOT NULL,
    actor          TEXT NOT NULL,
    capability     TEXT NOT NULL,
    scope_summary  TEXT NOT NULL,
    endpoint_host  TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_assist_audit_ts ON assist_audit (ts);

-- ── Password-change audit — APPEND-ONLY (§18.3).
CREATE TABLE IF NOT EXISTS password_change_audit (
    id           TEXT PRIMARY KEY NOT NULL,
    ts           TEXT NOT NULL,
    account_id   TEXT NOT NULL,
    backend      TEXT NOT NULL,
    outcome      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_pwchange_audit_acct ON password_change_audit (account_id);

-- ── Per-account password-change config (§18.3, plan §2.3/§2.6). `force_change` is
-- BIGINT (0/1) — the uniform integer-boolean convention (matching 0001-0007), NOT
-- native BOOLEAN (the V6 bool-bind bug). `config` holds the JSON of
-- `mw_passwd::PasswdConfig`. Added by e9 (the passwd_config table gap in 0008).
CREATE TABLE IF NOT EXISTS passwd_config (
    account_id    TEXT PRIMARY KEY NOT NULL,
    config        TEXT NOT NULL DEFAULT '{}',
    force_change  BIGINT NOT NULL DEFAULT 0,
    updated_at    TEXT NOT NULL
);

-- ── Bridge accounts (§6.5).
CREATE TABLE IF NOT EXISTS bridge_accounts (
    account_id   TEXT PRIMARY KEY NOT NULL,
    bridge_id    TEXT NOT NULL,
    oauth_ref    TEXT,
    extra        TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX IF NOT EXISTS idx_bridge_accounts_bridge ON bridge_accounts (bridge_id);

-- ── Folded V6 follow-up (a): index sessions by account for headless REST reads.
CREATE INDEX IF NOT EXISTS idx_sessions_account ON sessions (account_id);
