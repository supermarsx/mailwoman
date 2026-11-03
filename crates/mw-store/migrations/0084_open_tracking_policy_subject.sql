-- Verified policy identity for anonymous tracking redemption. NULL keeps legacy
-- tokens distinguishable; JSON is sealed under the existing server key.
ALTER TABLE open_tracking ADD COLUMN policy_subject_sealed BLOB;
