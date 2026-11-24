-- 0113's three row-level messages triggers updated the account's single
-- mail_data_accounts row and mail_data_outbox row once or twice per message.
-- Inside one statement every such update leaves a row version that cannot be
-- pruned while the transaction is open, and each later lookup of that key walks
-- all of them, so a statement over n messages cost O(n^2) (8,000 rows: 10 s).
-- These statement-level triggers read the transition tables and write each
-- account's two shared rows once per statement, and each message's
-- mail_data_rows tuple once. The resulting rows are those of 0113; revisions
-- are random either way, so "the same" means the same rows exist and the same
-- revisions change. mail_data_stamp_update is folded into the UPDATE trigger.
--
-- An UPDATE pairs its OLD and NEW rows by stable_id. A row whose stable_id
-- changes cannot be paired that way (A->B, B->C leaves B in both tables), so
-- the row-level mail_data_message_rekey trigger, which fires only for such rows,
-- applies 0113's per-row rules to it and lists its new stable_id in a
-- per-trigger-depth setting; the statement trigger skips the listed rows and
-- clears the list. The setting is session-level on purpose: a transaction-local
-- set_config inside a function that has its own SET clause is undone when the
-- function returns. An aborted (sub)transaction restores it.
DROP TRIGGER mail_data_message_insert ON messages;
DROP TRIGGER mail_data_message_update ON messages;
DROP TRIGGER mail_data_message_delete ON messages;
DROP TRIGGER mail_data_stamp_update ON messages;
DROP FUNCTION mail_data_message_insert_fn();
DROP FUNCTION mail_data_message_update_fn();
DROP FUNCTION mail_data_message_delete_fn();
DROP FUNCTION mail_data_stamp_update_fn();

CREATE FUNCTION mail_data_messages_inserted_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 WITH live AS MATERIALIZED (SELECT a.id AS account_id FROM accounts a WHERE a.id IN(SELECT n.account_id FROM mail_data_new n)),
 ensured AS (INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT l.account_id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM live l ORDER BY l.account_id ON CONFLICT(account_id) DO NOTHING RETURNING 1),
 focused AS (UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id IN(SELECT n.account_id FROM mail_data_new n) RETURNING 1),
 tuples AS (INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM mail_data_new n CROSS JOIN LATERAL (SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision FROM messages m WHERE m.stable_id=n.stable_id AND m.account_id=n.account_id LIMIT 1) m JOIN accounts parent ON parent.id=m.account_id ORDER BY m.stable_id ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision RETURNING 1)
 INSERT INTO mail_data_outbox(account_id,revision) SELECT l.account_id,md5(random()::text||clock_timestamp()::text) FROM live l ORDER BY l.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_messages_inserted AFTER INSERT ON messages REFERENCING NEW TABLE AS mail_data_new FOR EACH STATEMENT EXECUTE FUNCTION mail_data_messages_inserted_fn();

CREATE FUNCTION mail_data_message_rekey_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE setting TEXT := 'mw_mail_data.rekeyed_'||pg_trigger_depth();
BEGIN
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=OLD.account_id ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=OLD.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id=NEW.account_id ON CONFLICT(account_id) DO NOTHING;
 INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=NEW.stable_id AND m.account_id=NEW.account_id ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
 INSERT INTO mail_data_outbox(account_id,revision) SELECT d.account_id,md5(random()::text||clock_timestamp()::text) FROM mail_data_accounts d JOIN accounts parent ON parent.id=d.account_id WHERE d.account_id=NEW.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id IN(OLD.account_id,NEW.account_id) AND EXISTS(SELECT 1 FROM mailboxes box WHERE box.id IN(OLD.mailbox_id,NEW.mailbox_id) AND box.role='sent');
 IF NEW.cache_revision IS DISTINCT FROM OLD.cache_revision OR NEW.content_revision IS DISTINCT FROM OLD.content_revision THEN
  UPDATE mail_data_rows SET cache_revision=NEW.cache_revision,content_revision=NEW.content_revision WHERE stable_id=NEW.stable_id AND account_id=NEW.account_id AND (cache_revision IS DISTINCT FROM NEW.cache_revision OR content_revision IS DISTINCT FROM NEW.content_revision);
 END IF;
 PERFORM set_config(setting,(COALESCE(NULLIF(current_setting(setting,true),''),'[]')::jsonb||to_jsonb(NEW.stable_id))::text,false);
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_message_rekey AFTER UPDATE ON messages FOR EACH ROW WHEN (OLD.stable_id IS DISTINCT FROM NEW.stable_id) EXECUTE FUNCTION mail_data_message_rekey_fn();

CREATE FUNCTION mail_data_messages_updated_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE setting TEXT := 'mw_mail_data.rekeyed_'||pg_trigger_depth(); rekeyed TEXT[] := '{}';
BEGIN
 IF COALESCE(current_setting(setting,true),'')<>'' THEN
  SELECT COALESCE(array_agg(x),'{}') INTO rekeyed FROM jsonb_array_elements_text(current_setting(setting)::jsonb) x;
  PERFORM set_config(setting,'',false);
 END IF;
 -- At most 8 rows: a plain join of the two transition tables. More rows: pair them with
 -- one sort (lag over stable_id), because a cached plan for the join may be a nested loop.
 IF NOT EXISTS(SELECT 1 FROM mail_data_new OFFSET 8) THEN
  WITH pair AS MATERIALIZED (SELECT n.stable_id,o.account_id AS old_account,n.account_id AS new_account,n.cache_revision,n.content_revision,(n.content_revision IS DISTINCT FROM o.content_revision OR n.envelope_json IS DISTINCT FROM o.envelope_json OR n.account_id IS DISTINCT FROM o.account_id OR n.blob_ref IS DISTINCT FROM o.blob_ref OR n.mailbox_id IS DISTINCT FROM o.mailbox_id OR n.uid IS DISTINCT FROM o.uid OR n.uidvalidity IS DISTINCT FROM o.uidvalidity OR n.message_id IS DISTINCT FROM o.message_id OR n.thread_id IS DISTINCT FROM o.thread_id OR n.internaldate IS DISTINCT FROM o.internaldate OR n.size IS DISTINCT FROM o.size OR n.flags_json IS DISTINCT FROM o.flags_json) AS changed,CASE WHEN (n.mailbox_id IS DISTINCT FROM o.mailbox_id OR n.thread_id IS DISTINCT FROM o.thread_id OR n.envelope_json IS DISTINCT FROM o.envelope_json OR n.blob_ref IS DISTINCT FROM o.blob_ref OR n.content_revision IS DISTINCT FROM o.content_revision OR n.internaldate IS DISTINCT FROM o.internaldate OR n.uid IS DISTINCT FROM o.uid OR n.account_id IS DISTINCT FROM o.account_id) THEN EXISTS(SELECT 1 FROM mailboxes box WHERE box.id IN(o.mailbox_id,n.mailbox_id) AND box.role='sent') ELSE false END AS focus,(n.cache_revision IS DISTINCT FROM o.cache_revision OR n.content_revision IS DISTINCT FROM o.content_revision) AS stamp FROM mail_data_new n JOIN mail_data_old o ON o.stable_id=n.stable_id WHERE NOT n.stable_id=ANY(rekeyed)),
  live AS MATERIALIZED (SELECT a.id AS account_id FROM accounts a WHERE a.id IN(SELECT x.account_id FROM pair p CROSS JOIN LATERAL (VALUES(p.old_account),(p.new_account)) x(account_id) WHERE p.changed)),
  ensured AS (INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT l.account_id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM live l ORDER BY l.account_id ON CONFLICT(account_id) DO NOTHING RETURNING 1),
  tuples AS (INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM pair p CROSS JOIN LATERAL (SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision FROM messages m WHERE m.stable_id=p.stable_id AND m.account_id=p.new_account LIMIT 1) m JOIN accounts parent ON parent.id=m.account_id WHERE p.changed ORDER BY m.stable_id ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision RETURNING 1),
  stamped AS (UPDATE mail_data_rows r SET cache_revision=p.cache_revision,content_revision=p.content_revision FROM pair p WHERE r.stable_id=p.stable_id AND r.account_id=p.new_account AND p.stamp AND NOT (p.changed AND EXISTS(SELECT 1 FROM accounts parent WHERE parent.id=p.new_account)) AND (r.cache_revision IS DISTINCT FROM p.cache_revision OR r.content_revision IS DISTINCT FROM p.content_revision) RETURNING 1),
  focused AS (UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id IN(SELECT x.account_id FROM pair p CROSS JOIN LATERAL (VALUES(p.old_account),(p.new_account)) x(account_id) WHERE p.focus) RETURNING 1)
  INSERT INTO mail_data_outbox(account_id,revision) SELECT l.account_id,md5(random()::text||clock_timestamp()::text) FROM live l ORDER BY l.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 ELSE
  WITH pair AS MATERIALIZED (SELECT q.stable_id,q.old_account,q.new_account,q.cache_revision,q.content_revision,q.changed,CASE WHEN q.focus_changed THEN EXISTS(SELECT 1 FROM mailboxes box WHERE box.id IN(q.old_mailbox,q.new_mailbox) AND box.role='sent') ELSE false END AS focus,q.stamp FROM (SELECT u.side,lag(u.side) OVER w AS old_side,u.stable_id,lag(u.account_id) OVER w AS old_account,u.account_id AS new_account,lag(u.mailbox_id) OVER w AS old_mailbox,u.mailbox_id AS new_mailbox,u.cache_revision,u.content_revision,(u.content_revision IS DISTINCT FROM lag(u.content_revision) OVER w OR u.envelope_json IS DISTINCT FROM lag(u.envelope_json) OVER w OR u.account_id IS DISTINCT FROM lag(u.account_id) OVER w OR u.blob_ref IS DISTINCT FROM lag(u.blob_ref) OVER w OR u.mailbox_id IS DISTINCT FROM lag(u.mailbox_id) OVER w OR u.uid IS DISTINCT FROM lag(u.uid) OVER w OR u.uidvalidity IS DISTINCT FROM lag(u.uidvalidity) OVER w OR u.message_id IS DISTINCT FROM lag(u.message_id) OVER w OR u.thread_id IS DISTINCT FROM lag(u.thread_id) OVER w OR u.internaldate IS DISTINCT FROM lag(u.internaldate) OVER w OR u.size IS DISTINCT FROM lag(u.size) OVER w OR u.flags_json IS DISTINCT FROM lag(u.flags_json) OVER w) AS changed,(u.mailbox_id IS DISTINCT FROM lag(u.mailbox_id) OVER w OR u.thread_id IS DISTINCT FROM lag(u.thread_id) OVER w OR u.envelope_json IS DISTINCT FROM lag(u.envelope_json) OVER w OR u.blob_ref IS DISTINCT FROM lag(u.blob_ref) OVER w OR u.content_revision IS DISTINCT FROM lag(u.content_revision) OVER w OR u.internaldate IS DISTINCT FROM lag(u.internaldate) OVER w OR u.uid IS DISTINCT FROM lag(u.uid) OVER w OR u.account_id IS DISTINCT FROM lag(u.account_id) OVER w) AS focus_changed,(u.cache_revision IS DISTINCT FROM lag(u.cache_revision) OVER w OR u.content_revision IS DISTINCT FROM lag(u.content_revision) OVER w) AS stamp FROM (SELECT 0 AS side,stable_id,account_id,mailbox_id,uid,uidvalidity,message_id,thread_id,internaldate,size,flags_json,envelope_json,blob_ref,cache_revision,content_revision FROM mail_data_old UNION ALL SELECT 1,stable_id,account_id,mailbox_id,uid,uidvalidity,message_id,thread_id,internaldate,size,flags_json,envelope_json,blob_ref,cache_revision,content_revision FROM mail_data_new) u WHERE NOT u.stable_id=ANY(rekeyed) WINDOW w AS (PARTITION BY u.stable_id ORDER BY u.side)) q WHERE q.side=1 AND q.old_side=0),
  live AS MATERIALIZED (SELECT a.id AS account_id FROM accounts a WHERE a.id IN(SELECT x.account_id FROM pair p CROSS JOIN LATERAL (VALUES(p.old_account),(p.new_account)) x(account_id) WHERE p.changed)),
  ensured AS (INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT l.account_id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM live l ORDER BY l.account_id ON CONFLICT(account_id) DO NOTHING RETURNING 1),
  tuples AS (INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM pair p CROSS JOIN LATERAL (SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision FROM messages m WHERE m.stable_id=p.stable_id AND m.account_id=p.new_account LIMIT 1) m JOIN accounts parent ON parent.id=m.account_id WHERE p.changed ORDER BY m.stable_id ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision RETURNING 1),
  stamped AS (UPDATE mail_data_rows r SET cache_revision=p.cache_revision,content_revision=p.content_revision FROM pair p WHERE r.stable_id=p.stable_id AND r.account_id=p.new_account AND p.stamp AND NOT (p.changed AND EXISTS(SELECT 1 FROM accounts parent WHERE parent.id=p.new_account)) AND (r.cache_revision IS DISTINCT FROM p.cache_revision OR r.content_revision IS DISTINCT FROM p.content_revision) RETURNING 1),
  focused AS (UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id IN(SELECT x.account_id FROM pair p CROSS JOIN LATERAL (VALUES(p.old_account),(p.new_account)) x(account_id) WHERE p.focus) RETURNING 1)
  INSERT INTO mail_data_outbox(account_id,revision) SELECT l.account_id,md5(random()::text||clock_timestamp()::text) FROM live l ORDER BY l.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 END IF;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_messages_updated AFTER UPDATE ON messages REFERENCING OLD TABLE AS mail_data_old NEW TABLE AS mail_data_new FOR EACH STATEMENT EXECUTE FUNCTION mail_data_messages_updated_fn();

CREATE FUNCTION mail_data_messages_deleted_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 WITH live AS MATERIALIZED (SELECT a.id AS account_id FROM accounts a WHERE a.id IN(SELECT o.account_id FROM mail_data_old o)),
 ensured AS (INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT l.account_id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM live l ORDER BY l.account_id ON CONFLICT(account_id) DO NOTHING RETURNING 1),
 focused AS (UPDATE mail_data_accounts SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id IN(SELECT o.account_id FROM mail_data_old o) RETURNING 1)
 INSERT INTO mail_data_outbox(account_id,revision) SELECT l.account_id,md5(random()::text||clock_timestamp()::text) FROM live l ORDER BY l.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
 RETURN NULL;
END $$;
CREATE TRIGGER mail_data_messages_deleted AFTER DELETE ON messages REFERENCING OLD TABLE AS mail_data_old FOR EACH STATEMENT EXECUTE FUNCTION mail_data_messages_deleted_fn();

DO $$ DECLARE function_name TEXT; BEGIN
 FOREACH function_name IN ARRAY ARRAY['mail_data_messages_inserted_fn','mail_data_message_rekey_fn','mail_data_messages_updated_fn','mail_data_messages_deleted_fn'] LOOP
  EXECUTE format('ALTER FUNCTION %I() SET search_path = pg_catalog, %I',function_name,current_schema());
 END LOOP;
END $$;
