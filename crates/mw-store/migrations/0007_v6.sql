-- V6 (Zero-Access · Admin Panel · API-Keys/OAuth 2.1 · MCP · Postgres · Cache)
-- schema (plan §2.1). Additive over 0001..0006; NEVER edit an earlier migration.
-- This is the SQLite variant, run by `sqlx::migrate!("./migrations")`. The
-- Postgres variant lives in `migrations_pg/0007_v6.sql` (selected per backend by
-- e1). Keep the two behaviourally identical.
--
-- Scaffolder note (e0): this is the skeleton + the additive tables. The per-table
-- typed repo methods land later — e3 (api_keys/oauth_*), e5 (admin_*/domains/
-- quotas/audit_log/cache_scope), e6+e10 (zeroaccess_accounts), e9 (webhooks). e1
-- authors the Postgres variants + the SQLite→Postgres copy (`migrate-store`).
--
-- INVARIANTS (plan §2.1):
--   * `api_keys.key_hash` / `oauth_tokens.token_hash` store ONLY a hash (Argon2id
--     for keys), never the plaintext token — shown once at mint (§20.1).
--   * `webhooks.secret_sealed` and `zeroaccess_accounts.wrapped_root_key` are
--     SEALED at rest via the existing `mw-store` seal (XChaCha20-Poly1305).
--   * `audit_log` is APPEND-ONLY — the repo exposes no UPDATE/DELETE path (§2.5).
--   * `account_id` columns are NOT foreign keys to `accounts` — API keys / OAuth /
--     zero-access exist in proxy mode too, which has no local `accounts` rows
--     (same rationale as `push_subscriptions`/`native_sessions` in 0006).
--   * Zero-access accounts' cache/message rows store CIPHERTEXT only (AAD per §9.3).

-- ── Scoped API keys (§20.1). `mwk_<key_prefix>.<secret>`; Argon2id hash at rest.
CREATE TABLE IF NOT EXISTS api_keys (
    id               TEXT PRIMARY KEY NOT NULL,
    key_prefix       TEXT NOT NULL,                 -- O(1) lookup index
    key_hash         TEXT NOT NULL,                 -- Argon2id(secret)
    account_id       TEXT NOT NULL,
    scopes           TEXT NOT NULL,                 -- JSON: the typed Scope set
    ip_allowlist     TEXT,                          -- JSON array, NULL = any
    expires_at       TEXT,                          -- RFC 3339, NULL = no expiry
    rate_limit       INTEGER,                       -- req/min, NULL = unlimited
    unattended_send  INTEGER NOT NULL DEFAULT 0,    -- 0/1; admin-countersign flag
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
    redirect_uris  TEXT NOT NULL,                   -- JSON array
    approved_by    TEXT NOT NULL,
    created_at     TEXT NOT NULL
);

-- ── OAuth 2.1 tokens (auth-code | access | refresh). Hash at rest; RFC 8707
-- `resource`; PKCE S256 challenge for auth-code grants.
CREATE TABLE IF NOT EXISTS oauth_tokens (
    token_hash      TEXT PRIMARY KEY NOT NULL,
    client_id       TEXT NOT NULL,
    account_id      TEXT NOT NULL,
    scopes          TEXT NOT NULL,                  -- JSON: the typed Scope set
    resource        TEXT,                           -- RFC 8707 resource indicator
    kind            TEXT NOT NULL,                  -- 'auth-code' | 'access' | 'refresh'
    expires_at      TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    revoked_at      TEXT,
    pkce_challenge  TEXT
);
CREATE INDEX IF NOT EXISTS idx_oauth_tokens_client ON oauth_tokens (client_id);
CREATE INDEX IF NOT EXISTS idx_oauth_tokens_acct ON oauth_tokens (account_id);

-- ── Outbound webhooks (§20.2). Secret SEALED at rest; HMAC-SHA256 signing (e9).
CREATE TABLE IF NOT EXISTS webhooks (
    id             TEXT PRIMARY KEY NOT NULL,
    account_id     TEXT NOT NULL,
    url            TEXT NOT NULL,
    secret_sealed  BLOB NOT NULL,                   -- sealed HMAC secret
    event_filter   TEXT NOT NULL,                   -- JSON: StateChange filter
    created_at     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_webhooks_acct ON webhooks (account_id);

-- ── Append-only audit log (§21). NO update/delete path exists (§2.5 invariant).
CREATE TABLE IF NOT EXISTS audit_log (
    id           TEXT PRIMARY KEY NOT NULL,
    ts           TEXT NOT NULL,
    actor        TEXT NOT NULL,
    actor_kind   TEXT NOT NULL,                     -- 'admin'|'user'|'api-key'|'system'
    action       TEXT NOT NULL,
    target       TEXT,
    detail_json  TEXT NOT NULL,
    ip           TEXT
);
CREATE INDEX IF NOT EXISTS idx_audit_log_ts ON audit_log (ts);
CREATE INDEX IF NOT EXISTS idx_audit_log_actor ON audit_log (actor);

-- ── Admin panel: a SEPARATE session domain (§19, §2.5). Passkey-capable; the
-- password hash is Argon2id at rest.
CREATE TABLE IF NOT EXISTS admin_users (
    id             TEXT PRIMARY KEY NOT NULL,
    username       TEXT NOT NULL,
    password_hash  TEXT,                            -- Argon2id, NULL = passkey-only
    created_at     TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_admin_users_name ON admin_users (username);

CREATE TABLE IF NOT EXISTS admin_sessions (
    token_hash   TEXT PRIMARY KEY NOT NULL,         -- hash only, mirrors sessions
    admin_id     TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    last_seen    TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_admin_sessions_admin ON admin_sessions (admin_id);

-- ── Managed domains (§19).
CREATE TABLE IF NOT EXISTS domains (
    name           TEXT PRIMARY KEY NOT NULL,
    upstream_json  TEXT NOT NULL,                   -- JSON upstream routing config
    allowlist      TEXT NOT NULL DEFAULT '[]',      -- JSON array
    blocklist      TEXT NOT NULL DEFAULT '[]'       -- JSON array
);

-- ── Per-account quotas (§19).
CREATE TABLE IF NOT EXISTS quotas (
    account_id   TEXT PRIMARY KEY NOT NULL,
    bytes_limit  INTEGER NOT NULL,
    msg_limit    INTEGER NOT NULL
);

-- ── Zero-access accounts (§9). The client-derived root key is stored WRAPPED
-- (the server never sees a plaintext key); recovery-wrapped copy optional; paired
-- devices as JSON (multi-device QR+SAS pairing relays ciphertext only).
CREATE TABLE IF NOT EXISTS zeroaccess_accounts (
    account_id        TEXT PRIMARY KEY NOT NULL,
    enabled           INTEGER NOT NULL DEFAULT 0,   -- 0/1
    wrapped_root_key  BLOB NOT NULL,                -- wrapped under KEK; opaque to server
    kdf_params        TEXT NOT NULL,                -- JSON: Argon2id / WebAuthn-PRF params
    recovery_wrapped  BLOB,                         -- recovery-phrase-wrapped copy
    paired_devices    TEXT NOT NULL DEFAULT '[]'    -- JSON array of device descriptors
);

-- ── Layered-cache scope matrix (§15.6). Admin-configurable per class; mirrors
-- `mw-cache`'s per-class layer+TTL policy.
CREATE TABLE IF NOT EXISTS cache_scope (
    class      TEXT PRIMARY KEY NOT NULL,           -- CacheClass id
    layers     TEXT NOT NULL,                       -- JSON array of 'memory'|'redis'|'store'
    ttl_secs   INTEGER NOT NULL
);
