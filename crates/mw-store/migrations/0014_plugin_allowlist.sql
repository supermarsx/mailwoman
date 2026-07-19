-- 0014 (26.15 t15): admin-managed third-party component load allowlist
-- (admin-pins-digest-at-install). ADDITIVE over 0001..0013 — NEVER edit an earlier
-- migration. Postgres variant: `migrations_pg/0014_plugin_allowlist.sql`.
--
-- TRUST MODEL (TQ1/TQ2/TQ6): this table is a SEPARATE, admin-managed fallback that the
-- compiled-in first-party digest pin ALWAYS takes precedence over. resolve_component
-- checks the frozen first-party table FIRST and terminally; it consults this allowlist
-- ONLY for a non-first-party plugin_id, and admits bytes ONLY on a byte-exact SHA-256
-- match to a NON-REVOKED row here. A row whose plugin_id collides with a first-party id
-- is therefore unreachable — it can never override, shadow, or spoof a first-party
-- identity. NO component bytes are stored here — only the pinned 64-hex identity + the
-- admin approval provenance.
--
-- INVARIANTS:
--   * `digest_hex` is the admin-pinned lowercase 64-hex SHA-256 of the exact component
--     bytes the admin reviewed. `(plugin_id, digest_hex)` is the primary key.
--   * `revoked` is 0/1, INTEGER here / BIGINT in Postgres (i64-uniform — NEVER native
--     BOOLEAN; the V6 lesson). A revoked row never admits bytes.
--   * `approved_by` / `approved_at` record the explicit human approval (audit provenance);
--     `name` / `version` / `source` / `note` are optional descriptive metadata.
CREATE TABLE IF NOT EXISTS plugin_allowlist (
    plugin_id    TEXT NOT NULL,
    digest_hex   TEXT NOT NULL,                -- admin-pinned lowercase 64-hex SHA-256
    name         TEXT,
    version      TEXT,
    source       TEXT,
    note         TEXT,
    approved_by  TEXT NOT NULL,
    approved_at  TEXT NOT NULL,
    revoked      INTEGER NOT NULL DEFAULT 0,   -- 0/1 (BIGINT in Postgres; never BOOLEAN)
    PRIMARY KEY (plugin_id, digest_hex)
);
CREATE INDEX IF NOT EXISTS idx_plugin_allowlist_plugin ON plugin_allowlist (plugin_id);
