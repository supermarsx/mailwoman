-- t50hp-e32 slice 1 (design t50hp-e29 part 9, "Mb"): keywords, pin, snooze and
-- follow-up are read from SQL (0125 projection, message_meta) by every query, so a
-- change of them no longer needs the search index and no longer journals.
-- messages: the update trigger keeps every column the index still reads, minus
-- flags_json. message_meta: its three journal triggers are dropped (pinned and
-- follow_up_at are no longer index fields the index is the reader of).
-- Content, membership and identity changes journal exactly as before.
DROP TRIGGER index_journal_message_update;
CREATE TRIGGER index_journal_message_update AFTER UPDATE ON messages
WHEN (NEW.stable_id IS NOT OLD.stable_id OR NEW.account_id IS NOT OLD.account_id OR NEW.mailbox_id IS NOT OLD.mailbox_id OR NEW.uid IS NOT OLD.uid OR NEW.uidvalidity IS NOT OLD.uidvalidity OR NEW.message_id IS NOT OLD.message_id OR NEW.thread_id IS NOT OLD.thread_id OR NEW.internaldate IS NOT OLD.internaldate OR NEW.size IS NOT OLD.size OR NEW.envelope_json IS NOT OLD.envelope_json OR NEW.blob_ref IS NOT OLD.blob_ref OR NEW.content_revision IS NOT OLD.content_revision)
 AND (SELECT enabled FROM index_publication_journal_control WHERE id=1)=1
BEGIN
 INSERT INTO index_publication_journal(account_id,stable_id,mode) SELECT OLD.account_id,OLD.stable_id,'delete' WHERE NEW.account_id IS NOT OLD.account_id OR NEW.stable_id IS NOT OLD.stable_id;
 INSERT INTO index_publication_journal(account_id,stable_id,mode) VALUES(NEW.account_id,NEW.stable_id,CASE WHEN NEW.envelope_json IS OLD.envelope_json AND NEW.blob_ref IS OLD.blob_ref AND NEW.content_revision IS OLD.content_revision AND NEW.account_id IS OLD.account_id AND NEW.stable_id IS OLD.stable_id THEN 'metadata' ELSE 'reconcile' END);
END;
DROP TRIGGER index_journal_meta_insert;
DROP TRIGGER index_journal_meta_update;
DROP TRIGGER index_journal_meta_delete;
