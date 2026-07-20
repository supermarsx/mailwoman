-- 0020 (26.17 t17): persist Identity.signatureName (`identities.signature_name`) —
-- POSTGRES variant. Behaviourally identical to the SQLite
-- `migrations/0020_identity_signature_name.sql`; TEXT stays TEXT (no dialect
-- difference). ADDITIVE over 0001..0019 — NEVER edit an earlier migration.
--
-- INVARIANTS: identical to the SQLite variant — `signature_name` is a non-secret
-- display label, plaintext TEXT, NULLABLE.
ALTER TABLE identities ADD COLUMN signature_name TEXT;
