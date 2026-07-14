-- V9 (26.10 deferred-tail: TypeScript UI-plugin tier · masked-email · OAuth DCR)
-- schema — POSTGRES variant (t10 plan §2.4). Behaviourally identical to the SQLite
-- `migrations/0010_v9_tail.sql`; dialect differences: BLOB → BYTEA, INTEGER →
-- BIGINT (boolean-as-0/1 columns stay BIGINT so the i64 bind/read path is UNIFORM,
-- matching 0001..0009 — NOT native BOOLEAN, which caused the V6 0007 bool-bind bug).
-- JSON stays TEXT.
--
-- INVARIANTS: identical to the SQLite variant — `signature` is a public detached
-- signature stored verbatim (not sealed); no column holds a secret; the DCR
-- registration-access-token is stored only as a HASH; DCR-issued clients land in
-- 0007 `oauth_clients` with the tail metadata in `oauth_client_meta` (0007 never
-- edited).

-- ── UI-plugin registry (SPEC §22.2). Deny-by-default (enabled=0, no grants).
CREATE TABLE IF NOT EXISTS ui_plugins (
    id                TEXT PRIMARY KEY NOT NULL,
    name              TEXT NOT NULL,
    version           TEXT NOT NULL,
    manifest          TEXT NOT NULL DEFAULT '{}',
    signature         BYTEA,
    approved_by       TEXT,
    enabled           BIGINT NOT NULL DEFAULT 0,
    capabilities      TEXT NOT NULL DEFAULT '[]',
    extension_points  TEXT NOT NULL DEFAULT '[]',
    created_at        TEXT NOT NULL
);

-- ── Per-plugin capability grants (deny-by-default; intersected with the manifest).
CREATE TABLE IF NOT EXISTS ui_plugin_grants (
    plugin_id    TEXT NOT NULL,
    capability   TEXT NOT NULL,
    params       TEXT NOT NULL DEFAULT '{}',
    granted_by   TEXT NOT NULL,
    granted_at   TEXT NOT NULL,
    PRIMARY KEY (plugin_id, capability)
);
CREATE INDEX IF NOT EXISTS idx_ui_plugin_grants_plugin ON ui_plugin_grants (plugin_id);

-- ── Masked-email aliases (SPEC §28.4).
CREATE TABLE IF NOT EXISTS masked_email (
    id            TEXT PRIMARY KEY NOT NULL,
    account_id    TEXT NOT NULL,
    alias_addr    TEXT NOT NULL,
    target_desc   TEXT NOT NULL DEFAULT '',
    state         TEXT NOT NULL DEFAULT 'enabled',
    created_at    TEXT NOT NULL,
    last_used_at  TEXT
);
CREATE INDEX IF NOT EXISTS idx_masked_email_account ON masked_email (account_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_masked_email_alias ON masked_email (alias_addr);

-- ── OAuth dynamic client registration policy (RFC 7591). Singleton, DEFAULT DISABLED.
CREATE TABLE IF NOT EXISTS oauth_dcr (
    id                              TEXT PRIMARY KEY NOT NULL,
    enabled                         BIGINT NOT NULL DEFAULT 0,
    require_initial_access_token    BIGINT NOT NULL DEFAULT 0,
    allowed_redirect_host_suffixes  TEXT NOT NULL DEFAULT '[]',
    default_scope                   TEXT NOT NULL DEFAULT '{}',
    updated_at                      TEXT NOT NULL
);

-- ── Side table for DCR-issued client metadata (avoids editing 0007 `oauth_clients`).
CREATE TABLE IF NOT EXISTS oauth_client_meta (
    client_id                      TEXT PRIMARY KEY NOT NULL,
    registration_access_token_hash TEXT,
    software_id                    TEXT,
    software_version               TEXT,
    contacts                       TEXT NOT NULL DEFAULT '[]',
    created_via                    TEXT NOT NULL DEFAULT 'dcr',
    created_at                     TEXT NOT NULL
);
