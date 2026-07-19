-- 0013 (26.15 t15): persistent per-(plugin, account) plugin key/value storage backing
-- the store:kv-scoped capability (replaces the non-persistent HostKv stub). ADDITIVE
-- over 0001..0012 — NEVER edit an earlier migration. Postgres variant:
-- `migrations_pg/0013_plugin_kv.sql`.
--
-- INVARIANTS:
--   * `sealed_value` holds the guest's opaque value bytes SEALED at rest
--     (XChaCha20-Poly1305 under the store ServerKey), never plaintext — the same
--     zero-access posture as bodies/creds. `key` (<=256 bytes, enforced app-side) is a
--     non-secret lookup key.
--   * Namespace = (plugin_id, account_id), both derived HOST-side from the bound state
--     (never guest args); a deployment-wide plugin uses account_id='' (matching the
--     plugin_grants convention).
--   * `size` (plaintext value length) is INTEGER here / BIGINT in Postgres, i64-uniform
--     (the V6 lesson — no native BOOLEAN anywhere). It feeds the per-(plugin,account)
--     5MiB / 1000-key quota accounting (enforced app-side).
--   * No TTL: plugin KV is intentional state, purged only on the plugin's own delete or
--     on uninstall (whole-namespace purge over the (plugin_id, account_id) index).
CREATE TABLE IF NOT EXISTS plugin_kv (
    plugin_id     TEXT NOT NULL,
    account_id    TEXT NOT NULL DEFAULT '',    -- '' ⇒ deployment-wide (plugin_grants convention)
    key           TEXT NOT NULL,               -- <=256 bytes (enforced app-side)
    sealed_value  BLOB NOT NULL,               -- SEALED opaque value bytes (never plaintext)
    size          INTEGER NOT NULL,            -- plaintext value length (quota accounting)
    updated_at    TEXT NOT NULL,
    PRIMARY KEY (plugin_id, account_id, key)
);
CREATE INDEX IF NOT EXISTS idx_plugin_kv_ns ON plugin_kv (plugin_id, account_id);
