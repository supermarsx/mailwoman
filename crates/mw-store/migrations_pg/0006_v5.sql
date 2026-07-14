-- V5 (thin shells + push) schema — POSTGRES variant (t6-e1; plan §1.1, §2.1).
-- Behaviourally identical to the SQLite `migrations/0006_v5.sql`. Dialect
-- mapping: INTEGER → BIGINT (push_config singleton id), BLOB → BYTEA
-- (vapid_private_sealed). `push_subscriptions.account_id` /
-- `native_sessions.account_id` are intentionally NOT foreign keys (proxy mode has
-- no local `accounts` rows — same rationale as the SQLite variant). The VAPID
-- private key is SEALED at rest; native sessions store only a token hash.

CREATE TABLE IF NOT EXISTS push_subscriptions (
    id            TEXT PRIMARY KEY NOT NULL,
    account_id    TEXT NOT NULL,
    transport     TEXT NOT NULL,
    endpoint      TEXT NOT NULL,
    p256dh        TEXT,
    auth          TEXT,
    app_id        TEXT,
    expires_at    TEXT,
    created_at    TEXT NOT NULL,
    last_wake_at  TEXT
);
CREATE INDEX IF NOT EXISTS idx_push_subs_acct ON push_subscriptions (account_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_push_subs_endpoint ON push_subscriptions (endpoint);

CREATE TABLE IF NOT EXISTS push_config (
    id                    BIGINT PRIMARY KEY CHECK (id = 1),
    vapid_public          TEXT NOT NULL,
    vapid_private_sealed  BYTEA NOT NULL,
    created_at            TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS native_sessions (
    token_hash    TEXT PRIMARY KEY NOT NULL,
    account_id    TEXT NOT NULL,
    client_type   TEXT NOT NULL,
    created_at    TEXT NOT NULL,
    last_seen     TEXT NOT NULL,
    rotated_from  TEXT
);
CREATE INDEX IF NOT EXISTS idx_native_sessions_acct ON native_sessions (account_id);
