-- 0019 (26.17 t17): seal the Note sort/filter metadata at rest — title, tags,
-- colour and the pinned flag were plaintext columns in 0004's `notes`
-- (searchable/sortable in the clear); 26.17 seals them under the store ServerKey
-- like the rich-text body already is. ADDITIVE over 0001..0018 — NEVER edit an
-- earlier migration (0004 stays frozen; SQLite cannot cleanly drop its plaintext
-- columns, so `v3.rs` BLANKS them to neutral defaults once a row is sealed). This
-- is the SQLite variant (run by `sqlx::migrate!("./migrations")`); the
-- behaviourally-identical Postgres variant is
-- `migrations_pg/0019_notes_seal_metadata.sql`.
--
-- INVARIANTS (mirror 0011..0016):
--   * The four `*_sealed` columns hold SEALED bytes (XChaCha20-Poly1305 under the
--     store ServerKey), never plaintext — the same at-rest posture as
--     `notes.body_html_sealed`. They are NULLABLE: a row is not-yet-backfilled
--     while `title_sealed IS NULL`. The one-shot store-open backfill seals + blanks
--     such rows; `note_from_row` falls back to the frozen plaintext column only for
--     that window (belt-and-braces).
--   * `pinned` was a real 0/1 INTEGER; its sealed form `pinned_sealed` carries the
--     same 0/1 as sealed bytes. The ordering that once relied on the plaintext
--     column (`list_notes ... ORDER BY pinned DESC`) moves to a Rust stable sort
--     after decrypt, so no plaintext ordering signal survives at rest.
--   * `idx_notes_pinned` (0004) indexed the now-sealed plaintext `pinned` column —
--     dead once ordering is Rust-side — so it is dropped here.
ALTER TABLE notes ADD COLUMN title_sealed     BLOB;
ALTER TABLE notes ADD COLUMN tags_json_sealed BLOB;
ALTER TABLE notes ADD COLUMN color_sealed     BLOB;
ALTER TABLE notes ADD COLUMN pinned_sealed    BLOB;
DROP INDEX IF EXISTS idx_notes_pinned;
