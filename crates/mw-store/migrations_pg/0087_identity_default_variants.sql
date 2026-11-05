-- Sender identities: one default per owner, and variants of a base identity.
-- is_default: exactly one row per owner (enforced by the partial unique index).
-- variant_of: id of the base identity in the same owner; NULL for a base.
-- label: display name of a variant. created_at orders "oldest" for promotion.
ALTER TABLE personal_identity ADD COLUMN is_default BIGINT NOT NULL DEFAULT 0;
ALTER TABLE personal_identity ADD COLUMN variant_of TEXT;
ALTER TABLE personal_identity ADD COLUMN label TEXT;
ALTER TABLE personal_identity ADD COLUMN created_at BIGINT NOT NULL DEFAULT 0;
UPDATE personal_identity SET is_default = 1
 WHERE id = (SELECT p2.id FROM personal_identity p2
              WHERE p2.owner_subject = personal_identity.owner_subject
              ORDER BY p2.email, p2.id LIMIT 1);
CREATE UNIQUE INDEX idx_personal_identity_default ON personal_identity(owner_subject) WHERE is_default = 1;
