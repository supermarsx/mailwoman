-- 0019 (26.17 t17): seal the Note sort/filter metadata at rest (title / tags /
-- colour / pinned) — POSTGRES variant. Behaviourally identical to the SQLite
-- `migrations/0019_notes_seal_metadata.sql`; the only dialect difference is
-- BLOB → BYTEA (the four sealed columns). ADDITIVE over 0001..0018 — NEVER edit an
-- earlier migration (0004 stays frozen; its plaintext columns are BLANKED to
-- neutral defaults by `v3.rs` once a row is sealed).
--
-- INVARIANTS: identical to the SQLite variant — the four `*_sealed` columns hold
-- SEALED bytes (XChaCha20-Poly1305 under the store ServerKey), never plaintext,
-- and are NULLABLE (`title_sealed IS NULL` = not-yet-backfilled); `pinned_sealed`
-- carries the same 0/1 as sealed bytes with ordering moved to a Rust stable sort;
-- `idx_notes_pinned` (dead once `pinned` is sealed) is dropped.
ALTER TABLE notes ADD COLUMN title_sealed     BYTEA;
ALTER TABLE notes ADD COLUMN tags_json_sealed BYTEA;
ALTER TABLE notes ADD COLUMN color_sealed     BYTEA;
ALTER TABLE notes ADD COLUMN pinned_sealed    BYTEA;
DROP INDEX IF EXISTS idx_notes_pinned;
