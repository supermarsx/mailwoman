-- 0012 (26.15 t15): new-file upload blob metadata for POST /jmap/upload/{accountId}.
-- ADDITIVE over 0001..0011 — NEVER edit an earlier migration. SQLite variant (run by
-- `sqlx::migrate!("./migrations")`); the identical-shape Postgres variant is
-- `migrations_pg/0012_uploads.sql`.
--
-- INVARIANTS (mirror 0007..0011):
--   * NO uploaded bytes live in the DB. The sealed object bytes live on the pluggable
--     UploadBackend (filesystem by default, sealed at rest under the store ServerKey);
--     this table holds only METADATA + the server-minted `storage_key` locating the
--     object + `backend_kind` naming the backend that holds it.
--   * `blob_id` is the `U`+64-hex upload id namespace (collision-free vs the pure
--     64-hex stableIds), returned to the client and resolved by Engine::fetch_blob.
--   * `size` (the plaintext byte length) is INTEGER here / BIGINT in Postgres so the
--     i64 bind/read path is UNIFORM across dialects (the V6 lesson — no native BOOLEAN
--     anywhere; every integer column is i64-uniform).
--   * `created_at` drives the 24h unreferenced-upload TTL swept by
--     `mailwoman maintenance gc-uploads` (explicit one-shot, never automatic).
CREATE TABLE IF NOT EXISTS uploaded_blobs (
    blob_id       TEXT PRIMARY KEY NOT NULL,   -- 'U'+64-hex upload id (fetch_blob namespace)
    account_id    TEXT NOT NULL,               -- owning account (get is account-scoped)
    content_type  TEXT NOT NULL,               -- client-declared MIME type
    size          INTEGER NOT NULL,            -- plaintext byte length
    storage_key   TEXT NOT NULL,               -- server-minted hex object key on the backend
    backend_kind  TEXT NOT NULL DEFAULT 'fs',  -- which UploadBackend holds the object
    created_at    TEXT NOT NULL                -- RFC3339; drives the gc-uploads TTL sweep
);
CREATE INDEX IF NOT EXISTS idx_uploaded_blobs_created ON uploaded_blobs (created_at);
CREATE INDEX IF NOT EXISTS idx_uploaded_blobs_account ON uploaded_blobs (account_id);
