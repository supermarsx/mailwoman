-- 0018 (26.16 t16): cached bridge OAuth tokens (B1) — POSTGRES variant.
-- Behaviourally identical to the SQLite `migrations/0018_bridge_oauth_tokens.sql`;
-- dialect difference: BLOB → BYTEA (and the `x''` empty-blob default → `'\x'`, the
-- Postgres empty-bytea literal). TEXT stays TEXT. ADDITIVE — NEVER edit an earlier
-- migration.
--
-- INVARIANTS: identical to the SQLite variant — both tokens are stored SEALED
-- (XChaCha20-Poly1305), never plaintext; `expires_at` drives proactive refresh;
-- `scope` is non-secret; one cached pair per `bridge_account_id`; no BOOLEAN column.
CREATE TABLE IF NOT EXISTS bridge_oauth_tokens (
    bridge_account_id     TEXT PRIMARY KEY NOT NULL,
    sealed_access_token   BYTEA NOT NULL,
    sealed_refresh_token  BYTEA NOT NULL DEFAULT '\x',
    expires_at            TEXT NOT NULL,
    scope                 TEXT NOT NULL DEFAULT '',
    updated_at            TEXT NOT NULL
);
