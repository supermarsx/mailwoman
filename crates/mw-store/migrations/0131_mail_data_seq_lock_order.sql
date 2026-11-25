-- Per-account change counter `seq` (plan t62-n3 section 2.4, M-2) and one lock
-- order. Every 0113 trigger's ensure of the account's mail_data_accounts row now
-- advances `seq` (DO UPDATE instead of DO NOTHING); new triggers advance it for
-- the three change logs and the checkpoint. SQLite has one writer, so the lock
-- order part (see the PostgreSQL file) changes nothing here.
ALTER TABLE mail_data_accounts ADD COLUMN seq INTEGER NOT NULL DEFAULT 0;

DROP TRIGGER mail_data_message_insert;
CREATE TRIGGER mail_data_message_insert AFTER INSERT ON messages
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,lower(hex(randomblob(16))) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.stable_id AND m.account_id=NEW.account_id ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET focus_revision=lower(hex(randomblob(16))) WHERE account_id=NEW.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_message_update;
CREATE TRIGGER mail_data_message_update AFTER UPDATE ON messages
WHEN NEW.content_revision IS NOT OLD.content_revision OR NEW.envelope_json IS NOT OLD.envelope_json OR NEW.account_id IS NOT OLD.account_id OR NEW.stable_id IS NOT OLD.stable_id OR NEW.blob_ref IS NOT OLD.blob_ref OR NEW.mailbox_id IS NOT OLD.mailbox_id OR NEW.uid IS NOT OLD.uid OR NEW.uidvalidity IS NOT OLD.uidvalidity OR NEW.message_id IS NOT OLD.message_id OR NEW.thread_id IS NOT OLD.thread_id OR NEW.internaldate IS NOT OLD.internaldate OR NEW.size IS NOT OLD.size OR NEW.flags_json IS NOT OLD.flags_json
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,lower(hex(randomblob(16))) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.stable_id AND m.account_id=NEW.account_id ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 UPDATE mail_data_accounts SET focus_revision=lower(hex(randomblob(16))) WHERE account_id IN(OLD.account_id,NEW.account_id) AND (NEW.mailbox_id IS NOT OLD.mailbox_id OR NEW.thread_id IS NOT OLD.thread_id OR NEW.envelope_json IS NOT OLD.envelope_json OR NEW.blob_ref IS NOT OLD.blob_ref OR NEW.content_revision IS NOT OLD.content_revision OR NEW.internaldate IS NOT OLD.internaldate OR NEW.uid IS NOT OLD.uid OR NEW.stable_id IS NOT OLD.stable_id OR NEW.account_id IS NOT OLD.account_id) AND EXISTS(SELECT 1 FROM mailboxes box WHERE box.id IN(OLD.mailbox_id,NEW.mailbox_id) AND box.role='sent');
END;

DROP TRIGGER mail_data_message_delete;
CREATE TRIGGER mail_data_message_delete BEFORE DELETE ON messages
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET focus_revision=lower(hex(randomblob(16))) WHERE account_id=OLD.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_message_meta_insert;
CREATE TRIGGER mail_data_message_meta_insert AFTER INSERT ON message_meta
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,lower(hex(randomblob(16))) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.stable_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_message_meta_update;
CREATE TRIGGER mail_data_message_meta_update AFTER UPDATE ON message_meta
WHEN NEW.stable_id IS NOT OLD.stable_id OR NEW.pinned IS NOT OLD.pinned OR NEW.snoozed_until IS NOT OLD.snoozed_until OR NEW.follow_up_at IS NOT OLD.follow_up_at
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,lower(hex(randomblob(16))) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=OLD.stable_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,lower(hex(randomblob(16))) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.stable_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_message_meta_delete;
CREATE TRIGGER mail_data_message_meta_delete BEFORE DELETE ON message_meta
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,lower(hex(randomblob(16))) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=OLD.stable_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_message_list_envelope_insert;
CREATE TRIGGER mail_data_message_list_envelope_insert AFTER INSERT ON message_list_envelope
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,lower(hex(randomblob(16))) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.stable_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_message_list_envelope_update;
CREATE TRIGGER mail_data_message_list_envelope_update AFTER UPDATE ON message_list_envelope
WHEN NEW.stable_id IS NOT OLD.stable_id OR NEW.account_id IS NOT OLD.account_id OR NEW.sealed_envelope IS NOT OLD.sealed_envelope
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,lower(hex(randomblob(16))) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=OLD.stable_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,lower(hex(randomblob(16))) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.stable_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_message_list_envelope_delete;
CREATE TRIGGER mail_data_message_list_envelope_delete BEFORE DELETE ON message_list_envelope
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,lower(hex(randomblob(16))) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=OLD.stable_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_security_verdicts_insert;
CREATE TRIGGER mail_data_security_verdicts_insert AFTER INSERT ON security_verdicts
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.email_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,lower(hex(randomblob(16))) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.email_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.email_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.email_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_security_verdicts_update;
CREATE TRIGGER mail_data_security_verdicts_update AFTER UPDATE ON security_verdicts
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.email_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,lower(hex(randomblob(16))) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=OLD.email_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.email_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.email_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.email_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,lower(hex(randomblob(16))) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.email_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.email_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.email_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_security_verdicts_delete;
CREATE TRIGGER mail_data_security_verdicts_delete BEFORE DELETE ON security_verdicts
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.email_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,lower(hex(randomblob(16))) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=OLD.email_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.email_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.email_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_mailbox_insert;
CREATE TRIGGER mail_data_mailbox_insert AFTER INSERT ON mailboxes
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET focus_revision=lower(hex(randomblob(16))) WHERE account_id=NEW.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_contact_insert;
CREATE TRIGGER mail_data_contact_insert AFTER INSERT ON contacts
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=(SELECT book.account_id FROM address_books book WHERE book.id=NEW.address_book_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET focus_revision=lower(hex(randomblob(16))) WHERE account_id=(SELECT book.account_id FROM address_books book WHERE book.id=NEW.address_book_id LIMIT 1);
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT book.account_id FROM address_books book WHERE book.id=NEW.address_book_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_address_book_insert;
CREATE TRIGGER mail_data_address_book_insert AFTER INSERT ON address_books
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET focus_revision=lower(hex(randomblob(16))) WHERE account_id=NEW.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_posture_insert;
CREATE TRIGGER mail_data_posture_insert AFTER INSERT ON zeroaccess_accounts
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET policy_revision=lower(hex(randomblob(16))) WHERE account_id=NEW.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_mailbox_update;
CREATE TRIGGER mail_data_mailbox_update AFTER UPDATE ON mailboxes
WHEN NEW.role IS NOT OLD.role OR NEW.account_id IS NOT OLD.account_id OR NEW.id IS NOT OLD.id
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET focus_revision=lower(hex(randomblob(16))) WHERE account_id=OLD.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET focus_revision=lower(hex(randomblob(16))) WHERE account_id=NEW.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_contact_update;
CREATE TRIGGER mail_data_contact_update AFTER UPDATE ON contacts
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=(SELECT book.account_id FROM address_books book WHERE book.id=OLD.address_book_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET focus_revision=lower(hex(randomblob(16))) WHERE account_id=(SELECT book.account_id FROM address_books book WHERE book.id=OLD.address_book_id LIMIT 1);
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT book.account_id FROM address_books book WHERE book.id=OLD.address_book_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=(SELECT book.account_id FROM address_books book WHERE book.id=NEW.address_book_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET focus_revision=lower(hex(randomblob(16))) WHERE account_id=(SELECT book.account_id FROM address_books book WHERE book.id=NEW.address_book_id LIMIT 1);
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT book.account_id FROM address_books book WHERE book.id=NEW.address_book_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_address_book_update;
CREATE TRIGGER mail_data_address_book_update AFTER UPDATE ON address_books
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET focus_revision=lower(hex(randomblob(16))) WHERE account_id=OLD.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET focus_revision=lower(hex(randomblob(16))) WHERE account_id=NEW.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_posture_update;
CREATE TRIGGER mail_data_posture_update AFTER UPDATE ON zeroaccess_accounts
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET policy_revision=lower(hex(randomblob(16))) WHERE account_id=NEW.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_mailbox_delete;
CREATE TRIGGER mail_data_mailbox_delete BEFORE DELETE ON mailboxes
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET focus_revision=lower(hex(randomblob(16))) WHERE account_id=OLD.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_contact_delete;
CREATE TRIGGER mail_data_contact_delete BEFORE DELETE ON contacts
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=(SELECT book.account_id FROM address_books book WHERE book.id=OLD.address_book_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET focus_revision=lower(hex(randomblob(16))) WHERE account_id=(SELECT book.account_id FROM address_books book WHERE book.id=OLD.address_book_id LIMIT 1);
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT book.account_id FROM address_books book WHERE book.id=OLD.address_book_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_address_book_delete;
CREATE TRIGGER mail_data_address_book_delete BEFORE DELETE ON address_books
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET focus_revision=lower(hex(randomblob(16))) WHERE account_id=OLD.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

DROP TRIGGER mail_data_posture_delete;
CREATE TRIGGER mail_data_posture_delete BEFORE DELETE ON zeroaccess_accounts
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
 UPDATE mail_data_accounts SET policy_revision=lower(hex(randomblob(16))) WHERE account_id=OLD.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,lower(hex(randomblob(16))) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
END;

CREATE TRIGGER mail_data_seq_changes_insert AFTER INSERT ON changes
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
END;

CREATE TRIGGER mail_data_seq_changes_update AFTER UPDATE ON changes
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
END;

CREATE TRIGGER mail_data_seq_changes_delete AFTER DELETE ON changes
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
END;

CREATE TRIGGER mail_data_seq_pim_changes_insert AFTER INSERT ON pim_changes
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
END;

CREATE TRIGGER mail_data_seq_pim_changes_update AFTER UPDATE ON pim_changes
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
END;

CREATE TRIGGER mail_data_seq_pim_changes_delete AFTER DELETE ON pim_changes
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
END;

CREATE TRIGGER mail_data_seq_crypto_changes_insert AFTER INSERT ON crypto_changes
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
END;

CREATE TRIGGER mail_data_seq_crypto_changes_update AFTER UPDATE ON crypto_changes
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
END;

CREATE TRIGGER mail_data_seq_crypto_changes_delete AFTER DELETE ON crypto_changes
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
END;

CREATE TRIGGER mail_data_seq_change_log_checkpoint_insert AFTER INSERT ON change_log_checkpoint
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
END;

CREATE TRIGGER mail_data_seq_change_log_checkpoint_update AFTER UPDATE ON change_log_checkpoint
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
END;

CREATE TRIGGER mail_data_seq_change_log_checkpoint_delete AFTER DELETE ON change_log_checkpoint
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,lower(hex(randomblob(16))),lower(hex(randomblob(16))),lower(hex(randomblob(16))) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1;
END;
