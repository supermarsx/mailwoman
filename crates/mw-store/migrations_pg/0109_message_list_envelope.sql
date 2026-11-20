CREATE TABLE message_list_envelope (
 stable_id TEXT PRIMARY KEY NOT NULL REFERENCES messages(stable_id) ON DELETE CASCADE,
 account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
 sealed_envelope BYTEA NOT NULL
);
CREATE INDEX message_list_envelope_account ON message_list_envelope(account_id);
CREATE FUNCTION mw_invalidate_message_list_envelope() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF NEW.envelope_json IS DISTINCT FROM OLD.envelope_json OR NEW.blob_ref IS DISTINCT FROM OLD.blob_ref
  OR NEW.account_id IS DISTINCT FROM OLD.account_id OR NEW.stable_id IS DISTINCT FROM OLD.stable_id THEN
  DELETE FROM message_list_envelope WHERE stable_id=OLD.stable_id;
 END IF;
 RETURN NEW;
END;
$$;
CREATE TRIGGER invalidate_message_list_envelope BEFORE UPDATE OF envelope_json,blob_ref,account_id,stable_id ON messages
FOR EACH ROW EXECUTE FUNCTION mw_invalidate_message_list_envelope();
