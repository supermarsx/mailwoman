-- Parity with SQLite 0126 (t50hp-e32 slice 1, "Mb"): a keyword, pin, snooze or
-- follow-up change no longer journals for the search index. The message trigger
-- gets a column list without flags_json (and cache_revision, set by 0105's BEFORE
-- trigger), so PostgreSQL does not call it for a flag change at all; its function
-- no longer compares flags_json. The message_meta journal trigger is dropped.
CREATE OR REPLACE FUNCTION index_journal_message_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='UPDATE' THEN
  IF NOT (NEW.stable_id IS DISTINCT FROM OLD.stable_id OR NEW.account_id IS DISTINCT FROM OLD.account_id OR NEW.mailbox_id IS DISTINCT FROM OLD.mailbox_id OR NEW.uid IS DISTINCT FROM OLD.uid OR NEW.uidvalidity IS DISTINCT FROM OLD.uidvalidity OR NEW.message_id IS DISTINCT FROM OLD.message_id OR NEW.thread_id IS DISTINCT FROM OLD.thread_id OR NEW.internaldate IS DISTINCT FROM OLD.internaldate OR NEW.size IS DISTINCT FROM OLD.size OR NEW.envelope_json IS DISTINCT FROM OLD.envelope_json OR NEW.blob_ref IS DISTINCT FROM OLD.blob_ref OR NEW.content_revision IS DISTINCT FROM OLD.content_revision) THEN
   RETURN NULL;
  END IF;
 END IF;
 IF NOT EXISTS (SELECT 1 FROM index_publication_journal_control WHERE id=1 AND enabled=1) THEN
  RETURN NULL;
 END IF;
 IF TG_OP='INSERT' THEN
  INSERT INTO index_publication_journal(account_id,stable_id,mode) VALUES(NEW.account_id,NEW.stable_id,'reconcile');
 ELSIF TG_OP='DELETE' THEN
  INSERT INTO index_publication_journal(account_id,stable_id,mode) VALUES(OLD.account_id,OLD.stable_id,'delete');
 ELSE
  IF NEW.account_id IS DISTINCT FROM OLD.account_id OR NEW.stable_id IS DISTINCT FROM OLD.stable_id THEN
   INSERT INTO index_publication_journal(account_id,stable_id,mode) VALUES(OLD.account_id,OLD.stable_id,'delete');
  END IF;
  INSERT INTO index_publication_journal(account_id,stable_id,mode) VALUES(NEW.account_id,NEW.stable_id,CASE WHEN NEW.envelope_json IS NOT DISTINCT FROM OLD.envelope_json AND NEW.blob_ref IS NOT DISTINCT FROM OLD.blob_ref AND NEW.content_revision IS NOT DISTINCT FROM OLD.content_revision AND NEW.account_id IS NOT DISTINCT FROM OLD.account_id AND NEW.stable_id IS NOT DISTINCT FROM OLD.stable_id THEN 'metadata' ELSE 'reconcile' END);
 END IF;
 RETURN NULL;
END $$;
DROP TRIGGER index_journal_message ON messages;
CREATE TRIGGER index_journal_message AFTER INSERT OR DELETE OR UPDATE OF stable_id,account_id,mailbox_id,uid,uidvalidity,message_id,thread_id,internaldate,size,envelope_json,blob_ref,content_revision ON messages FOR EACH ROW EXECUTE FUNCTION index_journal_message_fn();
DROP TRIGGER index_journal_meta ON message_meta;
DROP FUNCTION index_journal_meta_fn();
DO $$ BEGIN
 EXECUTE format('ALTER FUNCTION index_journal_message_fn() SET search_path = pg_catalog, %I',current_schema());
END $$;
