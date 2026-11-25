-- Two row triggers fired on every single-message flag change and did nothing
-- (t62-n15 per-statement measurement, postgres:16, median of 300):
-- mail_data_mailbox_update on the mailbox counter UPDATE (~25 us; its function
-- returns unless role, account_id or id changed) and verdict_content_revision on
-- the flag UPDATE (~10 us; acts only when account_id, stable_id or blob_ref
-- changed). With these column lists PostgreSQL does not call them unless the
-- UPDATE names one of those columns; their functions still compare OLD and NEW.
-- No BEFORE trigger on either table changes those columns, so nothing is missed.
DROP TRIGGER mail_data_mailbox_update ON mailboxes;
CREATE TRIGGER mail_data_mailbox_update AFTER UPDATE OF role,account_id,id ON mailboxes FOR EACH ROW EXECUTE FUNCTION mail_data_mailbox_update_fn();
DROP TRIGGER verdict_content_revision ON messages;
CREATE TRIGGER verdict_content_revision BEFORE INSERT OR UPDATE OF account_id,stable_id,blob_ref ON messages FOR EACH ROW EXECUTE FUNCTION mw_verdict_content_revision();
