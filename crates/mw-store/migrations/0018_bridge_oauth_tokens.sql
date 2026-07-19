-- 0018 (26.16 t16): cached bridge OAuth tokens (B1). The bridge OAuth CLIENT
-- (device-code / auth-code / refresh flows) mints access+refresh tokens from the
-- `bridge_accounts.oauth_ref` binding stored at 0008; this table caches the minted
-- pair so a refresh isn't re-run on every call. ADDITIVE over 0001..0017 — NEVER edit
-- an earlier migration. This is the SQLite variant; the behaviourally-identical
-- Postgres variant is `migrations_pg/0018_bridge_oauth_tokens.sql`.
--
-- INVARIANTS (mirror 0011/0015):
--   * Both tokens are SECRETS: stored SEALED (XChaCha20-Poly1305 under the store
--     ServerKey) in `sealed_access_token` / `sealed_refresh_token` (BLOB here / BYTEA
--     in Postgres), never plaintext — the same zero-access posture as
--     `ews_account_cred.sealed_cred`. The host unseals only to answer the bridge's
--     gated OAuth-token import; the guest never holds a long-lived secret.
--   * `expires_at` (RFC 3339) drives proactive refresh; `scope` is the granted scope
--     string (non-secret). One cached pair per `bridge_account_id`.
--   * No native BOOLEAN anywhere (the V6 lesson); this table has no boolean column.
CREATE TABLE IF NOT EXISTS bridge_oauth_tokens (
    bridge_account_id     TEXT PRIMARY KEY NOT NULL,
    sealed_access_token   BLOB NOT NULL,             -- SEALED access token (never plaintext)
    sealed_refresh_token  BLOB NOT NULL DEFAULT x'', -- SEALED refresh token ('' sealed ⇒ none)
    expires_at            TEXT NOT NULL,             -- RFC 3339 access-token expiry
    scope                 TEXT NOT NULL DEFAULT '',  -- granted scope (non-secret)
    updated_at            TEXT NOT NULL
);
