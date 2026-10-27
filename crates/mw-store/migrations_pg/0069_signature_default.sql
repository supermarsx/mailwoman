-- Repair only duplicate default flags; preserve every signature and both formats.
UPDATE personal_signature
SET is_default = 0
WHERE is_default <> 0 AND EXISTS (
    SELECT 1 FROM personal_signature newer
    WHERE newer.owner_subject = personal_signature.owner_subject
      AND newer.is_default <> 0
      AND (newer.updated_at > personal_signature.updated_at
        OR (newer.updated_at = personal_signature.updated_at
          AND newer.name COLLATE "C" < personal_signature.name COLLATE "C"))
);
CREATE UNIQUE INDEX personal_signature_one_default
    ON personal_signature(owner_subject) WHERE is_default <> 0;
