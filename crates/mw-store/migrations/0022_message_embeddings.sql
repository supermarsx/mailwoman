-- 0022 (26.19 t19): per-message embedding vectors backing the OPT-IN semantic
-- search re-rank (SPEC §10.4/§14.3, gap A8). Until now the web sent a `semantic`
-- flag on `Email/query` that nothing server-side read; this table is the storage
-- half of the re-rank. ADDITIVE over 0001..0021 — NEVER edit an earlier migration.
-- This is the SQLite variant (run by `sqlx::migrate!("./migrations")`); the
-- behaviourally-identical Postgres variant is
-- `migrations_pg/0022_message_embeddings.sql`.
--
-- INVARIANTS (mirror 0011..0019):
--   * `vector_sealed` holds SEALED bytes (XChaCha20-Poly1305 under the store
--     ServerKey), never plaintext. An embedding is a lossy but content-DERIVED
--     projection of a message's subject/body, so it gets the same at-rest posture
--     as every other content-derived blob (`notes.body_html_sealed`,
--     `bridge_oauth_tokens.sealed_*`). The plaintext form is a `dim`-long array of
--     little-endian `f32`s, i.e. `dim * 4` bytes once opened.
--   * `dim` is a real i64 COUNT (the vector's length), NOT a boolean — INTEGER
--     here / BIGINT in Postgres, matching the uniform i64 bind/read path
--     (0011..0021, NEVER native BOOLEAN). It is stored ALONGSIDE the vector on
--     purpose: it is the guard that lets a reader detect a row written under a
--     different embedding model and SKIP it (degrading that hit to its lexical
--     rank) instead of cosine-comparing vectors of different shapes.
--   * `model` records the embedding model id the vector was produced by, for the
--     same reason — two models can share a dimension, and comparing across them
--     would silently corrupt the ordering.
--   * `stable_id` is the store's opaque message stable id and the PRIMARY KEY, so
--     re-embedding a message REPLACES its vector (upsert), never accumulates.
--   * Rows are written ONLY when a deployment has configured an Assist embedding
--     provider AND granted the `search-semantic` capability. A default deployment
--     never populates this table and never reads it.
CREATE TABLE IF NOT EXISTS message_embeddings (
    stable_id     TEXT PRIMARY KEY,
    account_id    TEXT NOT NULL,
    model         TEXT NOT NULL,
    dim           INTEGER NOT NULL,
    vector_sealed BLOB NOT NULL,
    updated_at    TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_message_embeddings_account ON message_embeddings (account_id);
