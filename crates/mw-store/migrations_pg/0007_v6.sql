-- V6 schema — POSTGRES variant (plan §1.1, §2.1). Behaviourally identical to the
-- SQLite `migrations/0007_v6.sql`; the dialect differences are: BLOB → BYTEA,
-- INTEGER-boolean → BOOLEAN, and no PRAGMA. Timestamps stay TEXT (RFC 3339) to
-- match the existing store convention and keep the SQLite→Postgres `migrate-store`
-- copy a straight value move (plan §2.1).
--
-- Scaffolder note (e0): this is the 0007 SKELETON for the Postgres backend.
-- `migrations_pg/` currently holds ONLY 0007 — e1 authors the Postgres variants
-- of 0001..0006 to complete the directory before `sqlx::migrate!("./migrations_pg")`
-- is selected for a `postgres://` DSN. Nothing runs this file yet (e0 keeps
-- SQLite the default), so it is inert scaffolding for e1.
--
-- INVARIANTS: identical to the SQLite variant — hashes only for tokens,
-- SEALED webhook secrets / wrapped root keys, APPEND-ONLY audit_log, `account_id`
-- columns are NOT foreign keys, zero-access rows store ciphertext only.

-- ── Scoped API keys (§20.1).
CREATE TABLE IF NOT EXISTS api_keys (
    id               TEXT PRIMARY KEY NOT NULL,
    key_prefix       TEXT NOT NULL,
    key_hash         TEXT NOT NULL,
    account_id       TEXT NOT NULL,
    scopes           TEXT NOT NULL,
    ip_allowlist     TEXT,
    expires_at       TEXT,
    rate_limit       BIGINT,
    unattended_send  BOOLEAN NOT NULL DEFAULT FALSE,
    created_at       TEXT NOT NULL,
    last_used_at     TEXT,
    revoked_at       TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_api_keys_prefix ON api_keys (key_prefix);
CREATE INDEX IF NOT EXISTS idx_api_keys_acct ON api_keys (account_id);

-- ── OAuth 2.1 admin-approved client registry (§20.1).
CREATE TABLE IF NOT EXISTS oauth_clients (
    client_id      TEXT PRIMARY KEY NOT NULL,
    name           TEXT NOT NULL,
    redirect_uris  TEXT NOT NULL,
    approved_by    TEXT NOT NULL,
    created_at     TEXT NOT NULL
);

-- ── OAuth 2.1 tokens.
CREATE TABLE IF NOT EXISTS oauth_tokens (
    token_hash      TEXT PRIMARY KEY NOT NULL,
    client_id       TEXT NOT NULL,
    account_id      TEXT NOT NULL,
    scopes          TEXT NOT NULL,
    resource        TEXT,
    kind            TEXT NOT NULL,
    expires_at      TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    revoked_at      TEXT,
    pkce_challenge  TEXT
);
CREATE INDEX IF NOT EXISTS idx_oauth_tokens_client ON oauth_tokens (client_id);
CREATE INDEX IF NOT EXISTS idx_oauth_tokens_acct ON oauth_tokens (account_id);

-- ── Outbound webhooks (§20.2). Secret SEALED at rest.
CREATE TABLE IF NOT EXISTS webhooks (
    id             TEXT PRIMARY KEY NOT NULL,
    account_id     TEXT NOT NULL,
    url            TEXT NOT NULL,
    secret_sealed  BYTEA NOT NULL,
    event_filter   TEXT NOT NULL,
    created_at     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_webhooks_acct ON webhooks (account_id);

-- ── Append-only audit log (§21).
CREATE TABLE IF NOT EXISTS audit_log (
    id           TEXT PRIMARY KEY NOT NULL,
    ts           TEXT NOT NULL,
    actor        TEXT NOT NULL,
    actor_kind   TEXT NOT NULL,
    action       TEXT NOT NULL,
    target       TEXT,
    detail_json  TEXT NOT NULL,
    ip           TEXT
);
CREATE INDEX IF NOT EXISTS idx_audit_log_ts ON audit_log (ts);
CREATE INDEX IF NOT EXISTS idx_audit_log_actor ON audit_log (actor);

-- ── Admin panel: a SEPARATE session domain (§19, §2.5).
CREATE TABLE IF NOT EXISTS admin_users (
    id             TEXT PRIMARY KEY NOT NULL,
    username       TEXT NOT NULL,
    password_hash  TEXT,
    created_at     TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_admin_users_name ON admin_users (username);

CREATE TABLE IF NOT EXISTS admin_sessions (
    token_hash   TEXT PRIMARY KEY NOT NULL,
    admin_id     TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    last_seen    TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_admin_sessions_admin ON admin_sessions (admin_id);

-- ── Managed domains (§19).
CREATE TABLE IF NOT EXISTS domains (
    name           TEXT PRIMARY KEY NOT NULL,
    upstream_json  TEXT NOT NULL,
    allowlist      TEXT NOT NULL DEFAULT '[]',
    blocklist      TEXT NOT NULL DEFAULT '[]'
);

-- ── Per-account quotas (§19).
CREATE TABLE IF NOT EXISTS quotas (
    account_id   TEXT PRIMARY KEY NOT NULL,
    bytes_limit  BIGINT NOT NULL,
    msg_limit    BIGINT NOT NULL
);

-- ── Zero-access accounts (§9).
CREATE TABLE IF NOT EXISTS zeroaccess_accounts (
    account_id        TEXT PRIMARY KEY NOT NULL,
    enabled           BOOLEAN NOT NULL DEFAULT FALSE,
    wrapped_root_key  BYTEA NOT NULL,
    kdf_params        TEXT NOT NULL,
    recovery_wrapped  BYTEA,
    paired_devices    TEXT NOT NULL DEFAULT '[]'
);

-- ── Layered-cache scope matrix (§15.6).
CREATE TABLE IF NOT EXISTS cache_scope (
    class      TEXT PRIMARY KEY NOT NULL,
    layers     TEXT NOT NULL,
    ttl_secs   BIGINT NOT NULL
);
