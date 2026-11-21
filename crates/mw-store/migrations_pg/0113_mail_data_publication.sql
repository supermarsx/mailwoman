-- Derived mail DATA versions, not caller authority or a per-event history.
-- Account epoch/outbox intentionally have no account FK: child source triggers
-- must not acquire a new parent-account FK lock after a Message row lock.
CREATE TABLE mail_data_accounts (
 account_id TEXT NOT NULL PRIMARY KEY,
 incarnation TEXT NOT NULL, focus_revision TEXT NOT NULL, policy_revision TEXT NOT NULL
);
CREATE TABLE mail_data_rows (
 stable_id TEXT NOT NULL PRIMARY KEY REFERENCES messages(stable_id) ON DELETE CASCADE ON UPDATE CASCADE,
 account_id TEXT NOT NULL,
 data_revision TEXT NOT NULL,
 cache_revision TEXT NOT NULL,
 content_revision TEXT NOT NULL
);
CREATE INDEX mail_data_rows_account ON mail_data_rows(account_id);
CREATE TABLE mail_data_outbox (
 account_id TEXT NOT NULL PRIMARY KEY,
 revision TEXT NOT NULL
);
--0110 body-content hooks advance messages.content_revision, reaching our
-- Message hook without duplicating a BLOB read or a body-reference scan.

CREATE FUNCTION mail_data_message_insert_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.stable_id AND m.account_id=NEW.account_id ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=NEW.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_message_insert AFTER INSERT ON messages FOR EACH ROW EXECUTE FUNCTION mail_data_message_insert_fn();

CREATE FUNCTION mail_data_message_update_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 IF NOT (NEW.content_revision IS DISTINCT FROM OLD.content_revision OR NEW.envelope_json IS DISTINCT FROM OLD.envelope_json OR NEW.account_id IS DISTINCT FROM OLD.account_id OR NEW.stable_id IS DISTINCT FROM OLD.stable_id OR NEW.blob_ref IS DISTINCT FROM OLD.blob_ref OR NEW.mailbox_id IS DISTINCT FROM OLD.mailbox_id OR NEW.uid IS DISTINCT FROM OLD.uid OR NEW.uidvalidity IS DISTINCT FROM OLD.uidvalidity OR NEW.message_id IS DISTINCT FROM OLD.message_id OR NEW.thread_id IS DISTINCT FROM OLD.thread_id OR NEW.internaldate IS DISTINCT FROM OLD.internaldate OR NEW.size IS DISTINCT FROM OLD.size OR NEW.flags_json IS DISTINCT FROM OLD.flags_json) THEN RETURN NULL; END IF;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.stable_id AND m.account_id=NEW.account_id ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id IN(OLD.account_id,NEW.account_id) AND (NEW.mailbox_id IS DISTINCT FROM OLD.mailbox_id OR NEW.thread_id IS DISTINCT FROM OLD.thread_id OR NEW.envelope_json IS DISTINCT FROM OLD.envelope_json OR NEW.blob_ref IS DISTINCT FROM OLD.blob_ref OR NEW.content_revision IS DISTINCT FROM OLD.content_revision OR NEW.internaldate IS DISTINCT FROM OLD.internaldate OR NEW.uid IS DISTINCT FROM OLD.uid OR NEW.stable_id IS DISTINCT FROM OLD.stable_id OR NEW.account_id IS DISTINCT FROM OLD.account_id) AND EXISTS(SELECT 1 FROM mailboxes box WHERE box.id IN(OLD.mailbox_id,NEW.mailbox_id) AND box.role='sent');
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_message_update AFTER UPDATE ON messages FOR EACH ROW EXECUTE FUNCTION mail_data_message_update_fn();

CREATE FUNCTION mail_data_message_delete_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=OLD.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN OLD;
END $$;
CREATE TRIGGER mail_data_message_delete BEFORE DELETE ON messages FOR EACH ROW EXECUTE FUNCTION mail_data_message_delete_fn();

CREATE FUNCTION mail_data_message_meta_insert_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.stable_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_message_meta_insert AFTER INSERT ON message_meta FOR EACH ROW EXECUTE FUNCTION mail_data_message_meta_insert_fn();

CREATE FUNCTION mail_data_message_meta_update_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 IF NOT (NEW.stable_id IS DISTINCT FROM OLD.stable_id OR NEW.pinned IS DISTINCT FROM OLD.pinned OR NEW.snoozed_until IS DISTINCT FROM OLD.snoozed_until OR NEW.follow_up_at IS DISTINCT FROM OLD.follow_up_at) THEN RETURN NULL; END IF;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=OLD.stable_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.stable_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_message_meta_update AFTER UPDATE ON message_meta FOR EACH ROW EXECUTE FUNCTION mail_data_message_meta_update_fn();

CREATE FUNCTION mail_data_message_meta_delete_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=OLD.stable_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN OLD;
END $$;
CREATE TRIGGER mail_data_message_meta_delete BEFORE DELETE ON message_meta FOR EACH ROW EXECUTE FUNCTION mail_data_message_meta_delete_fn();

CREATE FUNCTION mail_data_message_list_envelope_insert_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.stable_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_message_list_envelope_insert AFTER INSERT ON message_list_envelope FOR EACH ROW EXECUTE FUNCTION mail_data_message_list_envelope_insert_fn();

CREATE FUNCTION mail_data_message_list_envelope_update_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 IF NOT (NEW.stable_id IS DISTINCT FROM OLD.stable_id OR NEW.account_id IS DISTINCT FROM OLD.account_id OR NEW.sealed_envelope IS DISTINCT FROM OLD.sealed_envelope) THEN RETURN NULL; END IF;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=OLD.stable_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.stable_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_message_list_envelope_update AFTER UPDATE ON message_list_envelope FOR EACH ROW EXECUTE FUNCTION mail_data_message_list_envelope_update_fn();

CREATE FUNCTION mail_data_message_list_envelope_delete_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=OLD.stable_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.stable_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN OLD;
END $$;
CREATE TRIGGER mail_data_message_list_envelope_delete BEFORE DELETE ON message_list_envelope FOR EACH ROW EXECUTE FUNCTION mail_data_message_list_envelope_delete_fn();

CREATE FUNCTION mail_data_security_verdicts_insert_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.email_id LIMIT 1) ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.email_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.email_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.email_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_security_verdicts_insert AFTER INSERT ON security_verdicts FOR EACH ROW EXECUTE FUNCTION mail_data_security_verdicts_insert_fn();

CREATE FUNCTION mail_data_security_verdicts_update_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.email_id LIMIT 1) ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=OLD.email_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.email_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.email_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.email_id LIMIT 1) ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.email_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.email_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=NEW.email_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_security_verdicts_update AFTER UPDATE ON security_verdicts FOR EACH ROW EXECUTE FUNCTION mail_data_security_verdicts_update_fn();

CREATE FUNCTION mail_data_security_verdicts_delete_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.email_id LIMIT 1) ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=OLD.email_id AND m.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.email_id LIMIT 1) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT src.account_id FROM messages src WHERE src.stable_id=OLD.email_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN OLD;
END $$;
CREATE TRIGGER mail_data_security_verdicts_delete BEFORE DELETE ON security_verdicts FOR EACH ROW EXECUTE FUNCTION mail_data_security_verdicts_delete_fn();

CREATE FUNCTION mail_data_mailbox_insert_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=NEW.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_mailbox_insert AFTER INSERT ON mailboxes FOR EACH ROW EXECUTE FUNCTION mail_data_mailbox_insert_fn();

CREATE FUNCTION mail_data_contact_insert_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=(SELECT book.account_id FROM address_books book WHERE book.id=NEW.address_book_id LIMIT 1) ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=(SELECT book.account_id FROM address_books book WHERE book.id=NEW.address_book_id LIMIT 1);
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT book.account_id FROM address_books book WHERE book.id=NEW.address_book_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_contact_insert AFTER INSERT ON contacts FOR EACH ROW EXECUTE FUNCTION mail_data_contact_insert_fn();

CREATE FUNCTION mail_data_address_book_insert_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=NEW.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_address_book_insert AFTER INSERT ON address_books FOR EACH ROW EXECUTE FUNCTION mail_data_address_book_insert_fn();

CREATE FUNCTION mail_data_posture_insert_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET policy_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=NEW.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_posture_insert AFTER INSERT ON zeroaccess_accounts FOR EACH ROW EXECUTE FUNCTION mail_data_posture_insert_fn();

CREATE FUNCTION mail_data_mailbox_update_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 IF NOT (NEW.role IS DISTINCT FROM OLD.role OR NEW.account_id IS DISTINCT FROM OLD.account_id OR NEW.id IS DISTINCT FROM OLD.id) THEN RETURN NULL; END IF;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=OLD.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=NEW.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_mailbox_update AFTER UPDATE ON mailboxes FOR EACH ROW EXECUTE FUNCTION mail_data_mailbox_update_fn();

CREATE FUNCTION mail_data_contact_update_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=(SELECT book.account_id FROM address_books book WHERE book.id=OLD.address_book_id LIMIT 1) ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=(SELECT book.account_id FROM address_books book WHERE book.id=OLD.address_book_id LIMIT 1);
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT book.account_id FROM address_books book WHERE book.id=OLD.address_book_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=(SELECT book.account_id FROM address_books book WHERE book.id=NEW.address_book_id LIMIT 1) ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=(SELECT book.account_id FROM address_books book WHERE book.id=NEW.address_book_id LIMIT 1);
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT book.account_id FROM address_books book WHERE book.id=NEW.address_book_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_contact_update AFTER UPDATE ON contacts FOR EACH ROW EXECUTE FUNCTION mail_data_contact_update_fn();

CREATE FUNCTION mail_data_address_book_update_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=OLD.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=NEW.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_address_book_update AFTER UPDATE ON address_books FOR EACH ROW EXECUTE FUNCTION mail_data_address_book_update_fn();

CREATE FUNCTION mail_data_posture_update_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET policy_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=NEW.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_posture_update AFTER UPDATE ON zeroaccess_accounts FOR EACH ROW EXECUTE FUNCTION mail_data_posture_update_fn();

CREATE FUNCTION mail_data_mailbox_delete_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=OLD.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN OLD;
END $$;
CREATE TRIGGER mail_data_mailbox_delete BEFORE DELETE ON mailboxes FOR EACH ROW EXECUTE FUNCTION mail_data_mailbox_delete_fn();

CREATE FUNCTION mail_data_contact_delete_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=(SELECT book.account_id FROM address_books book WHERE book.id=OLD.address_book_id LIMIT 1) ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=(SELECT book.account_id FROM address_books book WHERE book.id=OLD.address_book_id LIMIT 1);
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=(SELECT book.account_id FROM address_books book WHERE book.id=OLD.address_book_id LIMIT 1) ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN OLD;
END $$;
CREATE TRIGGER mail_data_contact_delete BEFORE DELETE ON contacts FOR EACH ROW EXECUTE FUNCTION mail_data_contact_delete_fn();

CREATE FUNCTION mail_data_address_book_delete_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=OLD.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN OLD;
END $$;
CREATE TRIGGER mail_data_address_book_delete BEFORE DELETE ON address_books FOR EACH ROW EXECUTE FUNCTION mail_data_address_book_delete_fn();

CREATE FUNCTION mail_data_posture_delete_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO NOTHING;
 UPDATE mail_data_accounts SET policy_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=OLD.account_id;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN OLD;
END $$;
CREATE TRIGGER mail_data_posture_delete BEFORE DELETE ON zeroaccess_accounts FOR EACH ROW EXECUTE FUNCTION mail_data_posture_delete_fn();

CREATE FUNCTION mail_data_account_insert_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 DELETE FROM mail_data_rows WHERE account_id=NEW.id;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) VALUES(NEW.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text)) ON CONFLICT(account_id) DO UPDATE SET incarnation=excluded.incarnation,focus_revision=excluded.focus_revision,policy_revision=excluded.policy_revision;
 DELETE FROM mail_data_outbox WHERE account_id=NEW.id;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_account_insert AFTER INSERT ON accounts FOR EACH ROW EXECUTE FUNCTION mail_data_account_insert_fn();

CREATE FUNCTION mail_data_account_delete_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 DELETE FROM mail_data_outbox WHERE account_id=OLD.id;
 DELETE FROM mail_data_rows WHERE account_id=OLD.id;
 DELETE FROM mail_data_accounts WHERE account_id=OLD.id;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_account_delete AFTER DELETE ON accounts FOR EACH ROW EXECUTE FUNCTION mail_data_account_delete_fn();

CREATE FUNCTION mail_data_account_update_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 IF NEW.id IS NOT DISTINCT FROM OLD.id THEN RETURN NULL; END IF;
 DELETE FROM mail_data_outbox WHERE account_id=OLD.id;
 DELETE FROM mail_data_rows WHERE account_id=OLD.id;
 DELETE FROM mail_data_accounts WHERE account_id=OLD.id;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) VALUES(NEW.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text)) ON CONFLICT(account_id) DO UPDATE SET incarnation=excluded.incarnation,focus_revision=excluded.focus_revision,policy_revision=excluded.policy_revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_account_update AFTER UPDATE ON accounts FOR EACH ROW EXECUTE FUNCTION mail_data_account_update_fn();

CREATE FUNCTION mail_data_stamp_update_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 IF NEW.cache_revision IS DISTINCT FROM OLD.cache_revision OR NEW.content_revision IS DISTINCT FROM OLD.content_revision THEN
  UPDATE mail_data_rows SET cache_revision=NEW.cache_revision,content_revision=NEW.content_revision WHERE stable_id=NEW.stable_id AND account_id=NEW.account_id;
 END IF;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_stamp_update AFTER UPDATE OF cache_revision,content_revision ON messages FOR EACH ROW EXECUTE FUNCTION mail_data_stamp_update_fn();

DO $$ DECLARE function_name TEXT; BEGIN
 FOREACH function_name IN ARRAY ARRAY['mail_data_stamp_update_fn','mail_data_account_update_fn','mail_data_message_insert_fn','mail_data_message_update_fn','mail_data_message_delete_fn','mail_data_message_meta_insert_fn','mail_data_message_meta_update_fn','mail_data_message_meta_delete_fn','mail_data_message_list_envelope_insert_fn','mail_data_message_list_envelope_update_fn','mail_data_message_list_envelope_delete_fn','mail_data_security_verdicts_insert_fn','mail_data_security_verdicts_update_fn','mail_data_security_verdicts_delete_fn','mail_data_mailbox_insert_fn','mail_data_contact_insert_fn','mail_data_address_book_insert_fn','mail_data_posture_insert_fn','mail_data_mailbox_update_fn','mail_data_contact_update_fn','mail_data_address_book_update_fn','mail_data_posture_update_fn','mail_data_mailbox_delete_fn','mail_data_contact_delete_fn','mail_data_address_book_delete_fn','mail_data_posture_delete_fn','mail_data_account_insert_fn','mail_data_account_delete_fn'] LOOP
  EXECUTE format('ALTER FUNCTION %I() SET search_path = pg_catalog, %I',function_name,current_schema());
 END LOOP;
END $$;
