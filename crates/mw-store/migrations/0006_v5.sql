-- V5 (Thin desktop & mobile shells + push) schema (plan §2.4). Additive over
-- 0001_init + 0002_v1_cache + 0003_v2 + 0004_v3 + 0005_v4; NEVER edit an earlier
-- migration. This is the push-subscription store, the server VAPID keypair (private
-- key SEALED at rest), and the native bearer-token session table behind V5's
-- self-hostable push relay + additive native-client auth mode.
--
-- Scaffolder note (e0): this is the skeleton + the tables. e5 fills the typed repo
-- methods (`crates/mw-store/src/v5.rs`: push_subscriptions CRUD, VAPID get-or-init,
-- native_sessions CRUD) and the push dispatcher that reads them.
--
-- PRIVACY INVARIANTS (plan §2.3/§7/risk #8):
--   * NO message content is ever stored for push. A `push_subscriptions` row only
--     lets the server send an OPAQUE wake ("account X changed, wake + fetch").
--   * `push_config.vapid_private_sealed` is SEALED via the existing `mw-store`
--     seal (XChaCha20-Poly1305, the same mechanism V0 uses for upstream creds).
--     The VAPID private key NEVER leaves the server and is never stored in plaintext.
--   * `native_sessions` stores only a HASH of the bearer token (`token_hash`),
--     never the token itself — mirroring the opaque-cookie session handling.

-- Push subscriptions: one row per (account, endpoint). `transport` is an opaque
-- token the engine owns ('webpush' | 'unifiedpush' | 'apns'). `p256dh`/`auth` are
-- present for Web Push only; `app_id` for UnifiedPush/APNs. No content column.
-- NB: `account_id` is NOT a foreign key to `accounts`. Native auth + push are
-- first-class in BOTH engine mode (local `accounts` rows) AND proxy mode (V0 —
-- sessions carry an upstream account id with NO local `accounts` row, exactly like
-- the `sessions` table). A FK to `accounts(id)` would break the proxy path, so it
-- is intentionally omitted (app-layer cleanup handles account removal in engine mode).
CREATE TABLE IF NOT EXISTS push_subscriptions (
    id            TEXT PRIMARY KEY NOT NULL,
    account_id    TEXT NOT NULL,
    transport     TEXT NOT NULL,                 -- 'webpush' | 'unifiedpush' | 'apns'
    endpoint      TEXT NOT NULL,
    p256dh        TEXT,                           -- webpush only
    auth          TEXT,                           -- webpush only
    app_id        TEXT,                           -- unifiedpush / apns
    expires_at    TEXT,
    created_at    TEXT NOT NULL,
    last_wake_at  TEXT
);
CREATE INDEX IF NOT EXISTS idx_push_subs_acct ON push_subscriptions (account_id);
-- One subscription per endpoint (idempotent re-subscribe).
CREATE UNIQUE INDEX IF NOT EXISTS idx_push_subs_endpoint ON push_subscriptions (endpoint);

-- The server VAPID keypair (singleton, `id = 1`). Generated + persisted on first
-- boot; the private key is SEALED at rest. `GET /api/push/vapid` serves the PUBLIC
-- key only (clients need it to subscribe in-browser).
CREATE TABLE IF NOT EXISTS push_config (
    id                    INTEGER PRIMARY KEY CHECK (id = 1),
    vapid_public          TEXT NOT NULL,
    vapid_private_sealed  BLOB NOT NULL,
    created_at            TEXT NOT NULL
);

-- Native bearer-token sessions (plan §2.2). Additive to the cookie sessions; the
-- browser cookie path is UNCHANGED. Stores only the token HASH. `rotated_from`
-- links a rotated session to its predecessor (mirrors cookie-session rotation).
-- `account_id` is NOT a foreign key to `accounts` — same rationale as
-- `push_subscriptions` above: native bearer sessions exist in proxy mode too, which
-- has no local `accounts` rows.
CREATE TABLE IF NOT EXISTS native_sessions (
    token_hash    TEXT PRIMARY KEY NOT NULL,
    account_id    TEXT NOT NULL,
    client_type   TEXT NOT NULL,                 -- e.g. 'native'
    created_at    TEXT NOT NULL,
    last_seen     TEXT NOT NULL,
    rotated_from  TEXT
);
CREATE INDEX IF NOT EXISTS idx_native_sessions_acct ON native_sessions (account_id);
