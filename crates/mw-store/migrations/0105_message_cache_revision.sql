-- Opaque, transaction-coupled cache identities. Existing rows keep the cheap
-- empty default until their first mutation; no bulk body/envelope rewrite.
ALTER TABLE messages ADD COLUMN cache_revision TEXT NOT NULL DEFAULT '';
CREATE TRIGGER message_cache_revision_insert AFTER INSERT ON messages
WHEN NEW.cache_revision = ''
BEGIN
    UPDATE messages SET cache_revision = lower(hex(randomblob(16))) WHERE stable_id = NEW.stable_id;
END;
CREATE TRIGGER message_cache_revision_update AFTER UPDATE ON messages
WHEN NEW.cache_revision = OLD.cache_revision
BEGIN
    UPDATE messages SET cache_revision = lower(hex(randomblob(16))) WHERE stable_id = NEW.stable_id;
END;
