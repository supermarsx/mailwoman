-- 0014 (26.15 t15): admin-managed third-party component load allowlist — POSTGRES
-- variant. Behaviourally identical to the SQLite `migrations/0014_plugin_allowlist.sql`;
-- dialect difference: INTEGER → BIGINT (the `revoked` 0/1 flag stays BIGINT so the i64
-- bind/read path is UNIFORM — NOT native BOOLEAN, the V6 lesson). TEXT stays TEXT.
-- ADDITIVE — NEVER edit an earlier migration.
--
-- TRUST MODEL: identical to the SQLite variant — a SEPARATE, admin-managed fallback the
-- compiled-in first-party pin ALWAYS takes precedence over; consulted ONLY for a
-- non-first-party plugin_id; admits bytes ONLY on a byte-exact SHA-256 match to a
-- non-revoked row. NO component bytes stored — only the pinned identity + admin provenance.
CREATE TABLE IF NOT EXISTS plugin_allowlist (
    plugin_id    TEXT NOT NULL,
    digest_hex   TEXT NOT NULL,
    name         TEXT,
    version      TEXT,
    source       TEXT,
    note         TEXT,
    approved_by  TEXT NOT NULL,
    approved_at  TEXT NOT NULL,
    revoked      BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (plugin_id, digest_hex)
);
CREATE INDEX IF NOT EXISTS idx_plugin_allowlist_plugin ON plugin_allowlist (plugin_id);
