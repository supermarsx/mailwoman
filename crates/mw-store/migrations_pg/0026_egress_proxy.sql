-- 0026 (26.20 t22-e12): operator-configured outbound egress proxy routes —
-- POSTGRES variant. Behaviourally identical to the SQLite
-- `migrations/0026_egress_proxy.sql`; dialect differences: BLOB → BYTEA (and the
-- `x''` empty-blob default → `'\x'`, the Postgres empty-bytea literal), and the
-- boolean-as-0/1 `allow_plaintext` column is BIGINT so the dual-backend decode is
-- identical. TEXT stays TEXT. ADDITIVE — NEVER edit an earlier migration.
--
-- WHY 0026 AND NOT 0024: identical to the SQLite variant — 0024 is a hole inside an
-- applied sequence (0025 already exists here too), sqlx would silently apply a file
-- placed there out of order, and 0024 is formally retired. See the SQLite variant for
-- the full reasoning; `tests/t22_migration_tombstone.rs` enforces it for BOTH dialects.
--
-- INVARIANTS: identical to the SQLite variant — the password is stored SEALED
-- (XChaCha20-Poly1305), never plaintext; `username` is not a secret; an empty sealed
-- value means no credentials; `host` is operator configuration and deliberately not
-- subject to the SSRF address policy; no BOOLEAN column.
CREATE TABLE IF NOT EXISTS egress_proxy (
    id               TEXT PRIMARY KEY NOT NULL,
    scheme           TEXT NOT NULL,
    host             TEXT NOT NULL,
    port             BIGINT NOT NULL,
    username         TEXT NOT NULL DEFAULT '',
    sealed_password  BYTEA NOT NULL DEFAULT '\x',
    allow_plaintext  BIGINT NOT NULL DEFAULT 0,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
);
