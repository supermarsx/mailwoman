-- V7 (WASM Plugin Runtime · LDAP/GAL Directory · Password-Change · Assist(AI) ·
-- Graph/EWS/Gmail Bridges) schema (plan §2.7). Additive over 0001..0007; NEVER edit
-- an earlier migration. This is the SQLite variant, run by
-- `sqlx::migrate!("./migrations")`. The Postgres variant lives in
-- `migrations_pg/0008_v7.sql`; keep the two behaviourally identical.
--
-- Scaffolder note (e0): this is the SKELETON + the additive tables. The per-table
-- typed repo methods land later — e1/e14 (plugins/plugin_grants), e2/e9
-- (directory_config), e4/e9 (assist_config/assist_audit), e3/e9
-- (password_change_audit), e8/e10-e12 (bridge_accounts).
--
-- INVARIANTS (mirror 0007):
--   * BOOLEAN-as-0/1 columns use INTEGER here / BIGINT in Postgres so the i64
--     bind/read path is UNIFORM across dialects (V6 lesson — do NOT use native
--     BOOLEAN, which caused the 0007 bool-bind bug on Postgres).
--   * `signature` is stored as BLOB (BYTEA in Postgres); it is the DETACHED
--     signature over the component bytes (§22 signed registry).
--   * `*_audit` tables are APPEND-ONLY and CONTENT-FREE — the Assist audit carries
--     capability + scope summary + endpoint host, NEVER mail content (§14, R4).
--   * `account_id` columns are NOT foreign keys to `accounts` (proxy mode has no
--     local accounts rows) — same rationale as 0006/0007.
--   * JSON is stored as TEXT (the store convention), keeping the SQLite→Postgres
--     `migrate-store` copy a straight value move.

-- ── WASM plugin registry (§22, plan §2.1/§2.7). Signed-registry rows; `enabled`
-- and `approved_by` gate loading; `capabilities`/`net_allowlist`/`limits` mirror
-- the `plugin.toml` manifest.
CREATE TABLE IF NOT EXISTS plugins (
    id              TEXT PRIMARY KEY NOT NULL,
    name            TEXT NOT NULL,
    version         TEXT NOT NULL,
    signature       BLOB,                          -- detached sig; NULL = unsigned
    approved_by     TEXT,                          -- admin id; NULL = unapproved
    enabled         INTEGER NOT NULL DEFAULT 0,    -- 0/1
    capabilities    TEXT NOT NULL DEFAULT '[]',    -- JSON array of Capability
    net_allowlist   TEXT NOT NULL DEFAULT '[]',    -- JSON array of hosts
    limits          TEXT NOT NULL DEFAULT '{}',    -- JSON: { memory_mb, deadline_ms, fuel? }
    created_at      TEXT NOT NULL
);

-- ── Per-plugin capability grants, optionally scoped to an account (§22).
CREATE TABLE IF NOT EXISTS plugin_grants (
    plugin_id    TEXT NOT NULL,
    account_id   TEXT NOT NULL DEFAULT '',         -- '' = deployment-wide grant
    capability   TEXT NOT NULL,
    granted_by   TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    PRIMARY KEY (plugin_id, account_id, capability)
);
CREATE INDEX IF NOT EXISTS idx_plugin_grants_plugin ON plugin_grants (plugin_id);

-- ── LDAP/GAL directory endpoints, priority-ordered (§13, plan §2.2).
CREATE TABLE IF NOT EXISTS directory_config (
    id          TEXT PRIMARY KEY NOT NULL,
    url         TEXT NOT NULL,
    base_dn     TEXT NOT NULL,
    bind_dn     TEXT,
    tls         TEXT NOT NULL,                     -- 'none' | 'starttls' | 'ldaps'
    priority    INTEGER NOT NULL DEFAULT 0,        -- lower = queried first
    attr_map    TEXT NOT NULL DEFAULT '{}',        -- JSON attribute mapping
    enabled     INTEGER NOT NULL DEFAULT 1         -- 0/1
);

-- ── Assist config, per scope ('deployment' | 'user:<id>') (§14, plan §2.4).
CREATE TABLE IF NOT EXISTS assist_config (
    scope             TEXT PRIMARY KEY NOT NULL,   -- 'deployment' | 'user:<account_id>'
    adapters          TEXT NOT NULL DEFAULT '[]',  -- JSON: endpoint adapter configs
    capability_grants TEXT NOT NULL DEFAULT '[]',  -- JSON: AssistCapability array
    data_ceilings     TEXT NOT NULL DEFAULT '{}',  -- JSON: DataScope ceiling
    enabled           INTEGER NOT NULL DEFAULT 0   -- 0/1; 0 ⇒ web hides all Assist UI
);

-- ── Assist audit — APPEND-ONLY, CONTENT-FREE (§14, R4). NO mail content ever.
CREATE TABLE IF NOT EXISTS assist_audit (
    id             TEXT PRIMARY KEY NOT NULL,
    ts             TEXT NOT NULL,
    actor          TEXT NOT NULL,
    capability     TEXT NOT NULL,
    scope_summary  TEXT NOT NULL,                  -- accounts/folders summary; NO content
    endpoint_host  TEXT NOT NULL                   -- host only; NO content
);
CREATE INDEX IF NOT EXISTS idx_assist_audit_ts ON assist_audit (ts);

-- ── Password-change audit — APPEND-ONLY (§18.3, plan §2.3).
CREATE TABLE IF NOT EXISTS password_change_audit (
    id           TEXT PRIMARY KEY NOT NULL,
    ts           TEXT NOT NULL,
    account_id   TEXT NOT NULL,
    backend      TEXT NOT NULL,                    -- 'local'|'ldap3062'|'dovecot'|'poppassd'|'webhook'
    outcome      TEXT NOT NULL                     -- 'ok' | 'error:<reason>'
);
CREATE INDEX IF NOT EXISTS idx_pwchange_audit_acct ON password_change_audit (account_id);

-- ── Per-account password-change config (§18.3, plan §2.3/§2.6). Holds the
-- displayed policy + the forced-change-on-next-login flag as the JSON serialization
-- of `mw_passwd::PasswdConfig` in `config`, with `force_change` mirrored as a
-- queryable 0/1 column (login can gate on it without parsing JSON). Added by e9
-- (the passwd_config table gap e0's 0008 left — password_change_audit shipped, this
-- table did not). `account_id` is NOT a foreign key (proxy mode has no accounts row).
CREATE TABLE IF NOT EXISTS passwd_config (
    account_id    TEXT PRIMARY KEY NOT NULL,
    config        TEXT NOT NULL DEFAULT '{}',     -- JSON of mw_passwd::PasswdConfig
    force_change  INTEGER NOT NULL DEFAULT 0,     -- 0/1 mirror of force_change_on_next_login
    updated_at    TEXT NOT NULL
);

-- ── Bridge accounts: which account is served by which bridge plugin (§6.5).
CREATE TABLE IF NOT EXISTS bridge_accounts (
    account_id   TEXT PRIMARY KEY NOT NULL,
    bridge_id    TEXT NOT NULL,                    -- plugins.id of the bridge
    oauth_ref    TEXT,                             -- opaque OAuth token ref; NULL = none
    extra        TEXT NOT NULL DEFAULT '{}'        -- JSON: bridge-specific settings
);
CREATE INDEX IF NOT EXISTS idx_bridge_accounts_bridge ON bridge_accounts (bridge_id);

-- ── Folded V6 follow-up (a): proxy-mode headless scoped-key REST reads resolve a
-- session BY ACCOUNT (no cookie). The `sessions` table (0001) already carries
-- `account_id`; this index makes `Store::sessions_by_account` an O(log n) lookup.
-- Additive index only — no schema change to 0001.
CREATE INDEX IF NOT EXISTS idx_sessions_account ON sessions (account_id);
