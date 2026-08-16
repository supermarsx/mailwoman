-- 0026 (26.20 t22-e12): operator-configured outbound egress proxy routes.
-- ADDITIVE over 0001..0023 + 0025 — NEVER edit an earlier migration. This is the
-- SQLite variant; the behaviourally-identical Postgres variant is
-- `migrations_pg/0026_egress_proxy.sql`.
--
-- WHY 0026 AND NOT 0024: there is no 0024 and there never was. `0025_message_paging_
-- nulls_last.sql` already exists in both dialects, so 0024 is a HOLE INSIDE an applied
-- sequence, not the end of one. sqlx 0.8.6 would not reject a file placed there — its
-- migrator applies any resolved-but-unapplied migration regardless of version order —
-- so a 0024 would run AFTER 0025 on every existing database and BEFORE it on every
-- fresh one, with no error on either path. 0024 is formally retired and must never be
-- filled, in either dialect, by any lane. `tests/t22_migration_tombstone.rs` enforces
-- that.
--
-- INVARIANTS (mirror 0011/0015/0018):
--   * The proxy password is a SECRET: stored SEALED (XChaCha20-Poly1305 under the
--     store ServerKey) in `sealed_password` (BLOB here / BYTEA in Postgres), never
--     plaintext — the same zero-access posture as `bridge_oauth_tokens.sealed_access_
--     token` and `ews_account_cred.sealed_cred`. `username` is NOT a secret and is
--     stored and logged in the clear.
--   * An empty sealed value (`x''` sealed, i.e. the sealing of an empty string) means
--     the route carries NO credentials, decoded back to `None` — the same empty-means-
--     absent convention as 0018's `sealed_refresh_token`.
--   * `host` is OPERATOR configuration and is deliberately NOT subject to the SSRF
--     address policy: an egress proxy on RFC1918 is the normal deployment. That
--     asymmetry is only safe while this column can never become request-derived.
--   * No native BOOLEAN anywhere (the V6 lesson): `allow_plaintext` is 0/1 INTEGER
--     here and BIGINT in Postgres.
CREATE TABLE IF NOT EXISTS egress_proxy (
    id               TEXT PRIMARY KEY NOT NULL,
    scheme           TEXT NOT NULL,                  -- 'http' | 'socks5'
    host             TEXT NOT NULL,                  -- operator config; name or IP literal
    port             INTEGER NOT NULL,
    username         TEXT NOT NULL DEFAULT '',       -- NOT a secret
    sealed_password  BLOB NOT NULL DEFAULT x'',      -- SEALED ('' sealed ⇒ no credentials)
    allow_plaintext  INTEGER NOT NULL DEFAULT 0,     -- 0/1, never BOOLEAN
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
);
