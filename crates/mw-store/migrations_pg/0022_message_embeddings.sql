-- 0022 (26.19 t19): per-message embedding vectors backing the OPT-IN semantic
-- search re-rank (SPEC §10.4/§14.3, gap A8) — POSTGRES variant. Behaviourally
-- identical to the SQLite `migrations/0022_message_embeddings.sql`; dialect
-- differences ONLY: INTEGER → BIGINT (the i64 `dim` count stays BIGINT so the
-- bind/read path is UNIFORM, matching 0011..0021 — NOT native BOOLEAN, and NOT a
-- bool: this is a real length) and BLOB → BYTEA (both decode to `Vec<u8>` via
-- sqlx, so the seal/open code is dialect-free). ADDITIVE over 0001..0021 — NEVER
-- edit an earlier migration.
--
-- INVARIANTS: identical to the SQLite variant —
--   * `vector_sealed` holds SEALED bytes (XChaCha20-Poly1305 under the store
--     ServerKey), never plaintext; opened it is `dim` little-endian `f32`s
--     (`dim * 4` bytes).
--   * `dim` and `model` are stored ALONGSIDE the vector so a reader can detect a
--     row written under a different embedding model and SKIP it (degrading that
--     hit to its lexical rank) rather than cosine-comparing mismatched vectors.
--   * `stable_id` is the PRIMARY KEY, so re-embedding a message REPLACES its
--     vector (upsert), never accumulates.
--   * Rows are written ONLY when a deployment has configured an Assist embedding
--     provider AND granted the `search-semantic` capability. A default deployment
--     never populates this table and never reads it.
CREATE TABLE IF NOT EXISTS message_embeddings (
    stable_id     TEXT PRIMARY KEY,
    account_id    TEXT NOT NULL,
    model         TEXT NOT NULL,
    dim           BIGINT NOT NULL,
    vector_sealed BYTEA NOT NULL,
    updated_at    TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_message_embeddings_account ON message_embeddings (account_id);
