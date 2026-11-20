-- Optional derived metadata cache. Existing messages remain NULL/missing until read.
CREATE TABLE message_list_envelope (
 stable_id TEXT PRIMARY KEY NOT NULL REFERENCES messages(stable_id) ON DELETE CASCADE,
 account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
 sealed_envelope BLOB NOT NULL
);
CREATE INDEX message_list_envelope_account ON message_list_envelope(account_id);
CREATE TRIGGER invalidate_message_list_envelope BEFORE UPDATE OF envelope_json,blob_ref,account_id,stable_id ON messages
WHEN NEW.envelope_json IS NOT OLD.envelope_json OR NEW.blob_ref IS NOT OLD.blob_ref
 OR NEW.account_id IS NOT OLD.account_id OR NEW.stable_id IS NOT OLD.stable_id
BEGIN
 DELETE FROM message_list_envelope WHERE stable_id=OLD.stable_id;
END;
