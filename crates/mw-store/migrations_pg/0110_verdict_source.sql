ALTER TABLE messages ADD COLUMN content_revision TEXT NOT NULL DEFAULT '';
ALTER TABLE security_verdicts ADD COLUMN source_revision TEXT;
CREATE FUNCTION mw_verdict_content_revision() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
  IF NEW.content_revision='' THEN NEW.content_revision:=gen_random_uuid()::text; END IF;
 ELSIF NEW.account_id IS DISTINCT FROM OLD.account_id OR NEW.stable_id IS DISTINCT FROM OLD.stable_id OR NEW.blob_ref IS DISTINCT FROM OLD.blob_ref THEN
  NEW.content_revision:=gen_random_uuid()::text;
 END IF;
 RETURN NEW;
END; $$;
CREATE TRIGGER verdict_content_revision BEFORE INSERT OR UPDATE ON messages FOR EACH ROW EXECUTE FUNCTION mw_verdict_content_revision();
CREATE FUNCTION mw_verdict_body_revision() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
  UPDATE messages SET content_revision=gen_random_uuid()::text WHERE account_id=NEW.account_id AND blob_ref=NEW.blob_ref;
 ELSIF TG_OP='DELETE' THEN
  UPDATE messages SET content_revision=gen_random_uuid()::text WHERE account_id=OLD.account_id AND blob_ref=OLD.blob_ref;
 ELSIF NEW.account_id IS DISTINCT FROM OLD.account_id OR NEW.blob_ref IS DISTINCT FROM OLD.blob_ref OR NEW.sealed_bytes IS DISTINCT FROM OLD.sealed_bytes THEN
  UPDATE messages SET content_revision=gen_random_uuid()::text WHERE (account_id=OLD.account_id AND blob_ref=OLD.blob_ref) OR (account_id=NEW.account_id AND blob_ref=NEW.blob_ref);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER verdict_body_revision AFTER INSERT OR UPDATE OR DELETE ON bodies FOR EACH ROW EXECUTE FUNCTION mw_verdict_body_revision();
