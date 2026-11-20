-- Verdict proofs are bound to content, not mutable flags/mailbox placement.
ALTER TABLE messages ADD COLUMN content_revision TEXT NOT NULL DEFAULT '';
ALTER TABLE security_verdicts ADD COLUMN source_revision TEXT;
CREATE TRIGGER verdict_content_insert AFTER INSERT ON messages WHEN NEW.content_revision=''
BEGIN UPDATE messages SET content_revision=lower(hex(randomblob(16))) WHERE stable_id=NEW.stable_id; END;
CREATE TRIGGER verdict_content_update AFTER UPDATE ON messages
WHEN NEW.content_revision=OLD.content_revision AND (NEW.account_id IS NOT OLD.account_id OR NEW.stable_id IS NOT OLD.stable_id OR NEW.blob_ref IS NOT OLD.blob_ref)
BEGIN UPDATE messages SET content_revision=lower(hex(randomblob(16))) WHERE stable_id=NEW.stable_id; END;
CREATE TRIGGER verdict_body_insert AFTER INSERT ON bodies
BEGIN UPDATE messages SET content_revision=lower(hex(randomblob(16))) WHERE account_id=NEW.account_id AND blob_ref=NEW.blob_ref; END;
CREATE TRIGGER verdict_body_update AFTER UPDATE ON bodies
WHEN NEW.account_id IS NOT OLD.account_id OR NEW.blob_ref IS NOT OLD.blob_ref OR NEW.sealed_bytes IS NOT OLD.sealed_bytes
BEGIN UPDATE messages SET content_revision=lower(hex(randomblob(16))) WHERE (account_id=OLD.account_id AND blob_ref=OLD.blob_ref) OR (account_id=NEW.account_id AND blob_ref=NEW.blob_ref); END;
CREATE TRIGGER verdict_body_delete AFTER DELETE ON bodies
BEGIN UPDATE messages SET content_revision=lower(hex(randomblob(16))) WHERE account_id=OLD.account_id AND blob_ref=OLD.blob_ref; END;
