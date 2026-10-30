-- Additive identity defaults; existing identities keep null values.
ALTER TABLE identities ADD COLUMN organization TEXT;
ALTER TABLE personal_identity ADD COLUMN organization TEXT;
ALTER TABLE personal_identity ADD COLUMN auto_bcc TEXT;
ALTER TABLE personal_identity ADD COLUMN pgp_key_id TEXT;
