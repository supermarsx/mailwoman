-- V9 (26.10 deferred-tail: TypeScript UI-plugin tier · masked-email · OAuth DCR)
-- schema (t10 plan §2.4, SPEC §22.2/§28.4, V6 follow-up c). Additive over
-- 0001..0009; NEVER edit an earlier migration. This is the SQLite variant, run by
-- `sqlx::migrate!("./migrations")`. The Postgres variant lives in
-- `migrations_pg/0010_v9_tail.sql`; keep the two behaviourally identical.
--
-- INVARIANTS (mirror 0007..0009):
--   * BOOLEAN-as-0/1 columns use INTEGER here / BIGINT in Postgres so the i64
--     bind/read path is UNIFORM across dialects (V6 lesson — never native BOOLEAN).
--   * JSON (manifest, capabilities, extension_points, allowlists, scope) is stored
--     as TEXT (the store convention).
--   * `signature` is a BLOB (BYTEA in Postgres) holding a detached signature over
--     the plugin bundle — NOT a secret (it is a public signature), so it is stored
--     verbatim, not sealed. No column in this migration holds a secret; the DCR
--     registration-access-token is stored only as a HASH (never the raw token).
--   * DCR-issued OAuth clients are written to the 0007 `oauth_clients` table; the
--     tail-specific metadata lands in the side table `oauth_client_meta` here, so
--     0007 is never edited.

-- ── UI-plugin registry (SPEC §22.2): the sandboxed-iframe TypeScript plugin tier.
-- Deny-by-default: `enabled=0` and no grants until an admin approves. `manifest`
-- is the frozen §2.3 `ui-plugin.json`; `capabilities`/`extension_points` are the
-- declared allowlists (a grant can never exceed these).
CREATE TABLE IF NOT EXISTS ui_plugins (
    id                TEXT PRIMARY KEY NOT NULL,
    name              TEXT NOT NULL,
    version           TEXT NOT NULL,
    manifest          TEXT NOT NULL DEFAULT '{}',   -- JSON: the frozen ui-plugin.json
    signature         BLOB,                          -- detached signature over the bundle; NULL = unsigned
    approved_by       TEXT,                          -- admin operator id; NULL = not yet approved
    enabled           INTEGER NOT NULL DEFAULT 0,    -- 0/1 (deny-by-default)
    capabilities      TEXT NOT NULL DEFAULT '[]',    -- JSON: declared capability allowlist
    extension_points  TEXT NOT NULL DEFAULT '[]',    -- JSON: declared extension-point allowlist
    created_at        TEXT NOT NULL
);

-- ── Per-plugin capability grants (deny-by-default; each grant is intersected with
-- the manifest's declared `capabilities` at grant time). `params` carries the
-- capability's scoped config as JSON (e.g. the `net:host-allowlist` host set).
CREATE TABLE IF NOT EXISTS ui_plugin_grants (
    plugin_id    TEXT NOT NULL,
    capability   TEXT NOT NULL,                       -- e.g. 'ui:compose-action' | 'net:host-allowlist'
    params       TEXT NOT NULL DEFAULT '{}',          -- JSON: scoped config for the capability
    granted_by   TEXT NOT NULL,                        -- admin operator id
    granted_at   TEXT NOT NULL,
    PRIMARY KEY (plugin_id, capability)
);
CREATE INDEX IF NOT EXISTS idx_ui_plugin_grants_plugin ON ui_plugin_grants (plugin_id);

-- ── Masked-email aliases (SPEC §28.4). One row per alias; `state` is
-- 'enabled'|'disabled'|'deleted'. `target_desc` is a user-facing note (e.g. the
-- site the alias was created for) — NOT mail content.
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

-- ── OAuth dynamic client registration policy (RFC 7591; V6 follow-up c). A single
-- policy row (`id='default'`), DEFAULT DISABLED. When enabled, `register()` mints a
-- client under this gate: an optional initial-access-token, a redirect-host-suffix
-- allowlist, and a default scope. NO scope escalation beyond `default_scope`.
CREATE TABLE IF NOT EXISTS oauth_dcr (
    id                              TEXT PRIMARY KEY NOT NULL,   -- singleton: 'default'
    enabled                         INTEGER NOT NULL DEFAULT 0,  -- 0/1 (DEFAULT DISABLED)
    require_initial_access_token    INTEGER NOT NULL DEFAULT 0,  -- 0/1
    allowed_redirect_host_suffixes  TEXT NOT NULL DEFAULT '[]',  -- JSON: allowed redirect_uri host suffixes
    default_scope                   TEXT NOT NULL DEFAULT '{}',  -- JSON: mw-oauth Scope granted to DCR clients
    updated_at                      TEXT NOT NULL
);

-- ── Side table for DCR-issued client metadata (avoids editing 0007 `oauth_clients`).
-- The client itself lives in `oauth_clients`; this row records the RFC 7591 extras
-- and the registration-access-token HASH (never the raw token) used for the
-- client-configuration (read/update/delete) endpoint.
CREATE TABLE IF NOT EXISTS oauth_client_meta (
    client_id                     TEXT PRIMARY KEY NOT NULL,     -- FK-by-value to oauth_clients.id
    registration_access_token_hash TEXT,                         -- HASH of the reg-access-token; NULL = none
    software_id                   TEXT,
    software_version              TEXT,
    contacts                      TEXT NOT NULL DEFAULT '[]',    -- JSON: RFC 7591 contacts
    created_via                   TEXT NOT NULL DEFAULT 'dcr',
    created_at                    TEXT NOT NULL
);
