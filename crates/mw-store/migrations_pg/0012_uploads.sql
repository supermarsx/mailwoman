-- 0012 (26.15 t15): new-file upload blob metadata — POSTGRES variant. Behaviourally
-- identical to the SQLite `migrations/0012_uploads.sql`; dialect difference: INTEGER →
-- BIGINT (every integer column stays i64-uniform, matching 0001..0011 — NOT native
-- BOOLEAN, which caused the V6 0007 bool-bind bug). TEXT stays TEXT. ADDITIVE — NEVER
-- edit an earlier migration.
--
-- INVARIANTS: identical to the SQLite variant — NO uploaded bytes live in the DB (the
-- sealed object bytes live on the pluggable UploadBackend, sealed at rest); this table
-- holds only metadata + the server-minted `storage_key` + `backend_kind`. `blob_id` is
-- the `U`+64-hex upload namespace; `created_at` drives the gc-uploads TTL sweep.
CREATE TABLE IF NOT EXISTS uploaded_blobs (
    blob_id       TEXT PRIMARY KEY NOT NULL,
    account_id    TEXT NOT NULL,
    content_type  TEXT NOT NULL,
    size          BIGINT NOT NULL,
    storage_key   TEXT NOT NULL,
    backend_kind  TEXT NOT NULL DEFAULT 'fs',
    created_at    TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_uploaded_blobs_created ON uploaded_blobs (created_at);
CREATE INDEX IF NOT EXISTS idx_uploaded_blobs_account ON uploaded_blobs (account_id);
