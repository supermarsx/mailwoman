-- Missing reply defaults inherit the existing new-message signature.
-- An empty value explicitly disables signatures for replies.
ALTER TABLE identities ADD COLUMN reply_signature_name TEXT;
ALTER TABLE personal_identity ADD COLUMN reply_signature_name TEXT;
