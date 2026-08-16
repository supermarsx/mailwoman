-- 0027 (26.20 t22-e12): exactly one egress route may be ACTIVE — POSTGRES variant.
-- Behaviourally identical to the SQLite `migrations/0027_egress_proxy_active.sql`;
-- dialect differences: INTEGER → BIGINT for the 0/1 column, and the partial unique
-- index is written over a constant expression `((1))` in both dialects. ADDITIVE —
-- NEVER edit an earlier migration.
--
-- WHY AT MOST ONE: identical to the SQLite variant, and it is a SECURITY property,
-- not a simplification. `egress_proxy.host` is deliberately exempt from the SSRF
-- address policy because an egress proxy on RFC1918 is the normal deployment; that
-- carve-out is safe only while a route is deployment-wide operator configuration
-- that nothing request-derived can select. N rows implies a selection key, and a
-- request-derived key would turn the carve-out into a request-steerable SSRF
-- primitive. See the SQLite variant for the full reasoning.
ALTER TABLE egress_proxy ADD COLUMN active BIGINT NOT NULL DEFAULT 0;

CREATE UNIQUE INDEX IF NOT EXISTS idx_egress_proxy_single_active
    ON egress_proxy ((1)) WHERE active = 1;
