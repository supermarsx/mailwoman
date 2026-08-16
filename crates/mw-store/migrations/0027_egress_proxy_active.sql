-- 0027 (26.20 t22-e12): exactly one egress route may be ACTIVE.
-- ADDITIVE over 0026 — NEVER edit an earlier migration. SQLite variant; the
-- behaviourally-identical Postgres variant is `migrations_pg/0027_egress_proxy_active.sql`.
-- (0024 remains retired and must never be filled — see 0026's header and
-- `tests/t22_migration_tombstone.rs`.)
--
-- WHY AT MOST ONE, AND WHY IT IS A SECURITY PROPERTY RATHER THAN A SIMPLIFICATION
-- This will look like an arbitrary limitation to whoever next wants multi-route
-- support. It is not.
--
-- `egress_proxy.host` is deliberately NOT subject to the SSRF address policy: an
-- egress proxy on RFC1918 is the normal deployment ("Squid on localhost"), so
-- `mw_egress::proxy` permits its endpoint to resolve where `ip_allowed` would refuse.
-- That carve-out is safe for exactly one reason: a route is deployment-wide operator
-- configuration that NOTHING request-derived can select.
--
-- N configured rows implies a selection key. The moment that key is a destination
-- host, an account, or a header, **choosing a route becomes choosing a destination
-- whose host bypasses the address policy** — turning a bounded operator carve-out
-- into a request-steerable SSRF primitive. "A different proxy per destination
-- domain" is the reasonable-sounding feature that would do it.
--
-- So selection takes NO request-shaped input at all: `Store::active_egress_proxy`
-- takes only `&self`, and this index is what makes its answer well-defined. A
-- function with no steerable parameter cannot be steered, and that is checkable by
-- reading one signature rather than by auditing a body.
--
-- Adding rows is still allowed — staging a replacement route before switching to it
-- is a normal operator workflow. Only one may be live, and an operator who adds a
-- second is told which that is rather than discovering it from traffic.
--
-- `active` is 0/1 INTEGER, never BOOLEAN (the V6 lesson).
ALTER TABLE egress_proxy ADD COLUMN active INTEGER NOT NULL DEFAULT 0;

-- The constraint, enforced by the database rather than by remembering. A partial
-- unique index over a constant permits any number of inactive rows and at most one
-- active one.
CREATE UNIQUE INDEX IF NOT EXISTS idx_egress_proxy_single_active
    ON egress_proxy ((1)) WHERE active = 1;
