-- 0013 (26.15 t15): persistent per-(plugin, account) plugin key/value storage —
-- POSTGRES variant. Behaviourally identical to the SQLite `migrations/0013_plugin_kv.sql`;
-- dialect differences: BLOB → BYTEA, INTEGER → BIGINT (i64-uniform, matching 0001..0012 —
-- NOT native BOOLEAN, the V6 lesson). TEXT stays TEXT. ADDITIVE — NEVER edit an earlier
-- migration.
--
-- INVARIANTS: identical to the SQLite variant — `sealed_value` holds the opaque value
-- bytes SEALED at rest, never plaintext; namespace = (plugin_id, account_id) derived
-- host-side; `size` feeds the app-side 5MiB / 1000-key quota; no TTL (purged only on the
-- plugin's own delete or on uninstall).
CREATE TABLE IF NOT EXISTS plugin_kv (
    plugin_id     TEXT NOT NULL,
    account_id    TEXT NOT NULL DEFAULT '',
    key           TEXT NOT NULL,
    sealed_value  BYTEA NOT NULL,
    size          BIGINT NOT NULL,
    updated_at    TEXT NOT NULL,
    PRIMARY KEY (plugin_id, account_id, key)
);
CREATE INDEX IF NOT EXISTS idx_plugin_kv_ns ON plugin_kv (plugin_id, account_id);
