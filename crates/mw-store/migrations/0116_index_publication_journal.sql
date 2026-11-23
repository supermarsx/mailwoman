-- Crash-replay journal for search-index publication (t34, MW_INDEX_PUBLICATION_ACK).
-- A row says: the index entry for (account_id, stable_id) may be stale, so reconcile
-- it to the current SQL source. It is a hint, not a delta. Rows are written by the
-- triggers below in the same transaction as the source change, and only while the
-- control row exists with enabled=1. An absent control row means disabled, so the
-- default (commit) mode stores nothing and pays one primary-key probe per firing.
-- mode is advisory: replay must treat every row as a reconcile of the current source.
-- No account foreign key: erasure deletes this table's rows explicitly, and a source
-- trigger must not take a parent lock after a Message row lock (same rule as 0113).
-- AUTOINCREMENT is required, not cosmetic: a reused seq after a full trim would let
-- a later acknowledgement delete a row it never observed.
CREATE TABLE index_publication_journal (
 seq INTEGER PRIMARY KEY AUTOINCREMENT,
 account_id TEXT NOT NULL,
 stable_id TEXT NOT NULL,
 mode TEXT NOT NULL CHECK(mode IN ('metadata','reconcile','delete'))
);
CREATE INDEX index_publication_journal_key ON index_publication_journal(account_id,stable_id,seq);
CREATE TABLE index_publication_journal_control (
 id INTEGER PRIMARY KEY CHECK(id=1),
 enabled INTEGER NOT NULL DEFAULT 0 CHECK(enabled IN (0,1))
);

-- messages: the UPDATE column list is every messages column the index source reader
-- (index_publication.rs) reads, plus content_revision, which 0110 advances for every
-- bodies insert/update/delete, so raw body changes reach this trigger. cache_revision
-- is excluded on purpose: 0105 advances it with a nested UPDATE after every messages
-- UPDATE, so listing it would journal each change twice. The journal_tests parity
-- test fails if a reader column or a new messages column is neither listed nor exempt.
CREATE TRIGGER index_journal_message_insert AFTER INSERT ON messages
WHEN (SELECT enabled FROM index_publication_journal_control WHERE id=1)=1
BEGIN
 INSERT INTO index_publication_journal(account_id,stable_id,mode) VALUES(NEW.account_id,NEW.stable_id,'reconcile');
END;
CREATE TRIGGER index_journal_message_update AFTER UPDATE ON messages
WHEN (NEW.stable_id IS NOT OLD.stable_id OR NEW.account_id IS NOT OLD.account_id OR NEW.mailbox_id IS NOT OLD.mailbox_id OR NEW.uid IS NOT OLD.uid OR NEW.uidvalidity IS NOT OLD.uidvalidity OR NEW.message_id IS NOT OLD.message_id OR NEW.thread_id IS NOT OLD.thread_id OR NEW.internaldate IS NOT OLD.internaldate OR NEW.size IS NOT OLD.size OR NEW.flags_json IS NOT OLD.flags_json OR NEW.envelope_json IS NOT OLD.envelope_json OR NEW.blob_ref IS NOT OLD.blob_ref OR NEW.content_revision IS NOT OLD.content_revision)
 AND (SELECT enabled FROM index_publication_journal_control WHERE id=1)=1
BEGIN
 INSERT INTO index_publication_journal(account_id,stable_id,mode) SELECT OLD.account_id,OLD.stable_id,'delete' WHERE NEW.account_id IS NOT OLD.account_id OR NEW.stable_id IS NOT OLD.stable_id;
 INSERT INTO index_publication_journal(account_id,stable_id,mode) VALUES(NEW.account_id,NEW.stable_id,CASE WHEN NEW.envelope_json IS OLD.envelope_json AND NEW.blob_ref IS OLD.blob_ref AND NEW.content_revision IS OLD.content_revision AND NEW.account_id IS OLD.account_id AND NEW.stable_id IS OLD.stable_id THEN 'metadata' ELSE 'reconcile' END);
END;
CREATE TRIGGER index_journal_message_delete AFTER DELETE ON messages
WHEN (SELECT enabled FROM index_publication_journal_control WHERE id=1)=1
BEGIN
 INSERT INTO index_publication_journal(account_id,stable_id,mode) VALUES(OLD.account_id,OLD.stable_id,'delete');
END;

-- message_meta: pinned and follow_up_at are indexed; snoozed_until is read by the
-- source reader. The owner comes from the Message row; a cascade from a Message
-- delete finds none and adds nothing (the Message delete row covers it).
CREATE TRIGGER index_journal_meta_insert AFTER INSERT ON message_meta
WHEN (SELECT enabled FROM index_publication_journal_control WHERE id=1)=1
BEGIN
 INSERT INTO index_publication_journal(account_id,stable_id,mode) SELECT m.account_id,m.stable_id,'metadata' FROM messages m WHERE m.stable_id=NEW.stable_id;
END;
CREATE TRIGGER index_journal_meta_update AFTER UPDATE ON message_meta
WHEN (NEW.stable_id IS NOT OLD.stable_id OR NEW.pinned IS NOT OLD.pinned OR NEW.snoozed_until IS NOT OLD.snoozed_until OR NEW.follow_up_at IS NOT OLD.follow_up_at)
 AND (SELECT enabled FROM index_publication_journal_control WHERE id=1)=1
BEGIN
 INSERT INTO index_publication_journal(account_id,stable_id,mode) SELECT m.account_id,m.stable_id,'metadata' FROM messages m WHERE m.stable_id=OLD.stable_id AND NEW.stable_id IS NOT OLD.stable_id;
 INSERT INTO index_publication_journal(account_id,stable_id,mode) SELECT m.account_id,m.stable_id,'metadata' FROM messages m WHERE m.stable_id=NEW.stable_id;
END;
CREATE TRIGGER index_journal_meta_delete AFTER DELETE ON message_meta
WHEN (SELECT enabled FROM index_publication_journal_control WHERE id=1)=1
BEGIN
 INSERT INTO index_publication_journal(account_id,stable_id,mode) SELECT m.account_id,m.stable_id,'metadata' FROM messages m WHERE m.stable_id=OLD.stable_id;
END;

-- message_body_cache_lease: the reader uses COALESCE(m.blob_ref,l.blob_ref) as the
-- content identity, so a lease matters only while the Message has no blob_ref.
-- last_used_at (renewed on body reads) is not read and is not journaled.
CREATE TRIGGER index_journal_lease_insert AFTER INSERT ON message_body_cache_lease
WHEN (SELECT enabled FROM index_publication_journal_control WHERE id=1)=1
BEGIN
 INSERT INTO index_publication_journal(account_id,stable_id,mode) SELECT m.account_id,m.stable_id,'reconcile' FROM messages m WHERE m.stable_id=NEW.stable_id AND m.blob_ref IS NULL;
END;
CREATE TRIGGER index_journal_lease_update AFTER UPDATE ON message_body_cache_lease
WHEN (NEW.stable_id IS NOT OLD.stable_id OR NEW.account_id IS NOT OLD.account_id OR NEW.blob_ref IS NOT OLD.blob_ref OR NEW.uidvalidity IS NOT OLD.uidvalidity OR NEW.uid IS NOT OLD.uid)
 AND (SELECT enabled FROM index_publication_journal_control WHERE id=1)=1
BEGIN
 INSERT INTO index_publication_journal(account_id,stable_id,mode) SELECT m.account_id,m.stable_id,'reconcile' FROM messages m WHERE m.stable_id IN (OLD.stable_id,NEW.stable_id) AND m.blob_ref IS NULL;
END;
CREATE TRIGGER index_journal_lease_delete AFTER DELETE ON message_body_cache_lease
WHEN (SELECT enabled FROM index_publication_journal_control WHERE id=1)=1
BEGIN
 INSERT INTO index_publication_journal(account_id,stable_id,mode) SELECT m.account_id,m.stable_id,'reconcile' FROM messages m WHERE m.stable_id=OLD.stable_id AND m.blob_ref IS NULL;
END;
