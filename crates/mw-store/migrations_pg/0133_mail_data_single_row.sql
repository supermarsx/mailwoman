-- Single-message writes no slower than before 0129 (t62-n15 round 3).
-- 0129/0131's statement triggers built one CTE statement per call; for the one-row
-- statements the goal mix sends (one id per Email/set) that was slower than 0113's
-- row triggers for DELETE and the flag UPDATE (measured). Each function now reads its
-- transition table once and, for one row (one account for DELETE), runs plain
-- statements with cached plans: the account row first (UPDATE seq/focus; the
-- ensure upsert only when the row does not exist yet), which keeps 0131's lock
-- order, then the message tuple and the outbox. More rows run 0131's statement.
-- Measured against 0113 on one session, statements interleaved (postgres:16, 300
-- each, outbox row present / absent): INSERT -8/-8%, flag -13/-9%, move -17/-17%,
-- DELETE -1/-3% (within noise) of statement time. Asserted (5% margin) by
-- postgres_one_row_writes_no_slower_than_before_0129.
--
-- 0131's change-log seq triggers are removed: each cost 100-125 us per change-log
-- INSERT (two per flag change), above the 30 us per row gate.
DROP TRIGGER mail_data_seq_changes_insert ON changes;
DROP TRIGGER mail_data_seq_changes_update ON changes;
DROP TRIGGER mail_data_seq_changes_delete ON changes;
DROP TRIGGER mail_data_seq_pim_changes_insert ON pim_changes;
DROP TRIGGER mail_data_seq_pim_changes_update ON pim_changes;
DROP TRIGGER mail_data_seq_pim_changes_delete ON pim_changes;
DROP TRIGGER mail_data_seq_crypto_changes_insert ON crypto_changes;
DROP TRIGGER mail_data_seq_crypto_changes_update ON crypto_changes;
DROP TRIGGER mail_data_seq_crypto_changes_delete ON crypto_changes;
DROP TRIGGER mail_data_seq_change_log_checkpoint_insert ON change_log_checkpoint;
DROP TRIGGER mail_data_seq_change_log_checkpoint_update ON change_log_checkpoint;
DROP TRIGGER mail_data_seq_change_log_checkpoint_delete ON change_log_checkpoint;
DROP FUNCTION mail_data_seq_inserted_fn();
DROP FUNCTION mail_data_seq_updated_fn();
DROP FUNCTION mail_data_seq_deleted_fn();

CREATE OR REPLACE FUNCTION mail_data_messages_inserted_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE cnt BIGINT; acc TEXT; sid TEXT;
BEGIN
 -- One row (a single-message write): plain statements with cached plans, no CTEs.
 SELECT count(*),min(n.account_id),min(n.stable_id) INTO cnt,acc,sid FROM mail_data_new n;
 IF cnt=0 THEN RETURN NULL; END IF;
 IF cnt=1 THEN
  UPDATE mail_data_accounts SET seq=seq+1,focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id IN(acc);
  IF NOT FOUND THEN
   INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id IN(acc) ORDER BY a.id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1,focus_revision=excluded.focus_revision;
  END IF;
  INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=sid AND m.account_id=acc ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
  INSERT INTO mail_data_outbox(account_id,revision) SELECT a.id,md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id IN(acc) ORDER BY a.id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
  RETURN NULL;
 END IF;
 -- More rows: 0131's statement below.
 WITH live AS MATERIALIZED (SELECT a.id AS account_id FROM accounts a WHERE a.id IN(SELECT n.account_id FROM mail_data_new n)),
 orphans AS (UPDATE mail_data_accounts d SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE d.account_id IN(SELECT n.account_id FROM mail_data_new n) AND NOT EXISTS(SELECT 1 FROM accounts a WHERE a.id=d.account_id) RETURNING 1),
 tuples AS (INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM mail_data_new n CROSS JOIN LATERAL (SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision FROM messages m WHERE m.stable_id=n.stable_id AND m.account_id=n.account_id LIMIT 1) m JOIN accounts parent ON parent.id=m.account_id ORDER BY m.stable_id ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision RETURNING 1),
 outboxed AS (INSERT INTO mail_data_outbox(account_id,revision) SELECT l.account_id,md5(random()::text||clock_timestamp()::text) FROM live l ORDER BY l.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision RETURNING 1)
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id IN(SELECT n.account_id FROM mail_data_new n) ORDER BY a.id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1,focus_revision=excluded.focus_revision;
 RETURN NULL;
END $$;

CREATE OR REPLACE FUNCTION mail_data_messages_updated_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE cnt BIGINT; sid TEXT; old_acc TEXT; new_acc TEXT; changed BOOLEAN; focus BOOLEAN; stamp BOOLEAN; cache_rev TEXT; content_rev TEXT; locked BIGINT; setting TEXT := 'mw_mail_data.rekeyed_'||pg_trigger_depth(); rekeyed TEXT[] := '{}';
BEGIN
 IF COALESCE(current_setting(setting,true),'')<>'' THEN
  SELECT COALESCE(array_agg(x),'{}') INTO rekeyed FROM jsonb_array_elements_text(current_setting(setting)::jsonb) x;
  PERFORM set_config(setting,'',false);
 END IF;
 -- One row, not re-keyed (a single-message write): plain statements, no CTEs.
 SELECT c.n,p.stable_id,p.old_acc,p.new_acc,p.changed,p.focus,p.stamp,p.cache_revision,p.content_revision INTO cnt,sid,old_acc,new_acc,changed,focus,stamp,cache_rev,content_rev FROM (SELECT count(*) AS n FROM mail_data_new) c LEFT JOIN LATERAL (SELECT n.stable_id,o.account_id AS old_acc,n.account_id AS new_acc,(n.content_revision IS DISTINCT FROM o.content_revision OR n.envelope_json IS DISTINCT FROM o.envelope_json OR n.account_id IS DISTINCT FROM o.account_id OR n.blob_ref IS DISTINCT FROM o.blob_ref OR n.mailbox_id IS DISTINCT FROM o.mailbox_id OR n.uid IS DISTINCT FROM o.uid OR n.uidvalidity IS DISTINCT FROM o.uidvalidity OR n.message_id IS DISTINCT FROM o.message_id OR n.thread_id IS DISTINCT FROM o.thread_id OR n.internaldate IS DISTINCT FROM o.internaldate OR n.size IS DISTINCT FROM o.size OR n.flags_json IS DISTINCT FROM o.flags_json) AS changed,CASE WHEN (n.mailbox_id IS DISTINCT FROM o.mailbox_id OR n.thread_id IS DISTINCT FROM o.thread_id OR n.envelope_json IS DISTINCT FROM o.envelope_json OR n.blob_ref IS DISTINCT FROM o.blob_ref OR n.content_revision IS DISTINCT FROM o.content_revision OR n.internaldate IS DISTINCT FROM o.internaldate OR n.uid IS DISTINCT FROM o.uid OR n.account_id IS DISTINCT FROM o.account_id) THEN EXISTS(SELECT 1 FROM mailboxes box WHERE box.id IN(o.mailbox_id,n.mailbox_id) AND box.role='sent') ELSE false END AS focus,(n.cache_revision IS DISTINCT FROM o.cache_revision OR n.content_revision IS DISTINCT FROM o.content_revision) AS stamp,n.cache_revision,n.content_revision FROM mail_data_new n JOIN mail_data_old o ON o.stable_id=n.stable_id WHERE c.n=1 LIMIT 1) p ON true;
 IF cnt=0 THEN RETURN NULL; END IF;
 IF cnt=1 AND sid IS NOT NULL AND NOT sid=ANY(rekeyed) THEN
  IF changed THEN
   UPDATE mail_data_accounts SET seq=seq+1,focus_revision=CASE WHEN focus THEN md5(random()::text||clock_timestamp()::text) ELSE focus_revision END WHERE account_id IN(old_acc,new_acc);
   GET DIAGNOSTICS locked = ROW_COUNT;
   IF locked < (CASE WHEN old_acc=new_acc THEN 1 ELSE 2 END) THEN
    INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id IN(old_acc,new_acc) ORDER BY a.id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1,focus_revision=CASE WHEN focus THEN excluded.focus_revision ELSE mail_data_accounts.focus_revision END;
   END IF;
   INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM messages m JOIN accounts parent ON parent.id=m.account_id WHERE m.stable_id=sid AND m.account_id=new_acc ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision;
   IF NOT FOUND AND stamp THEN
    UPDATE mail_data_rows SET cache_revision=cache_rev,content_revision=content_rev WHERE stable_id=sid AND account_id=new_acc AND (cache_revision IS DISTINCT FROM cache_rev OR content_revision IS DISTINCT FROM content_rev);
   END IF;
   INSERT INTO mail_data_outbox(account_id,revision) SELECT a.id,md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id IN(old_acc,new_acc) ORDER BY a.id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
  ELSIF stamp THEN
   UPDATE mail_data_rows SET cache_revision=cache_rev,content_revision=content_rev WHERE stable_id=sid AND account_id=new_acc AND (cache_revision IS DISTINCT FROM cache_rev OR content_revision IS DISTINCT FROM content_rev);
  END IF;
  RETURN NULL;
 END IF;
 -- More rows: 0131's statement below.
 -- At most 8 rows: a plain join of the two transition tables. More rows: pair them with
 -- one sort (lag over stable_id), because a cached plan for the join may be a nested loop.
 IF NOT EXISTS(SELECT 1 FROM mail_data_new OFFSET 8) THEN
  WITH pair AS MATERIALIZED (SELECT n.stable_id,o.account_id AS old_account,n.account_id AS new_account,n.cache_revision,n.content_revision,(n.content_revision IS DISTINCT FROM o.content_revision OR n.envelope_json IS DISTINCT FROM o.envelope_json OR n.account_id IS DISTINCT FROM o.account_id OR n.blob_ref IS DISTINCT FROM o.blob_ref OR n.mailbox_id IS DISTINCT FROM o.mailbox_id OR n.uid IS DISTINCT FROM o.uid OR n.uidvalidity IS DISTINCT FROM o.uidvalidity OR n.message_id IS DISTINCT FROM o.message_id OR n.thread_id IS DISTINCT FROM o.thread_id OR n.internaldate IS DISTINCT FROM o.internaldate OR n.size IS DISTINCT FROM o.size OR n.flags_json IS DISTINCT FROM o.flags_json) AS changed,CASE WHEN (n.mailbox_id IS DISTINCT FROM o.mailbox_id OR n.thread_id IS DISTINCT FROM o.thread_id OR n.envelope_json IS DISTINCT FROM o.envelope_json OR n.blob_ref IS DISTINCT FROM o.blob_ref OR n.content_revision IS DISTINCT FROM o.content_revision OR n.internaldate IS DISTINCT FROM o.internaldate OR n.uid IS DISTINCT FROM o.uid OR n.account_id IS DISTINCT FROM o.account_id) THEN EXISTS(SELECT 1 FROM mailboxes box WHERE box.id IN(o.mailbox_id,n.mailbox_id) AND box.role='sent') ELSE false END AS focus,(n.cache_revision IS DISTINCT FROM o.cache_revision OR n.content_revision IS DISTINCT FROM o.content_revision) AS stamp FROM mail_data_new n JOIN mail_data_old o ON o.stable_id=n.stable_id WHERE NOT n.stable_id=ANY(rekeyed)),
  live AS MATERIALIZED (SELECT a.id AS account_id FROM accounts a WHERE a.id IN(SELECT x.account_id FROM pair p CROSS JOIN LATERAL (VALUES(p.old_account),(p.new_account)) x(account_id) WHERE p.changed)),
  orphans AS (UPDATE mail_data_accounts d SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE d.account_id IN(SELECT x.account_id FROM pair p CROSS JOIN LATERAL (VALUES(p.old_account),(p.new_account)) x(account_id) WHERE p.focus) AND NOT EXISTS(SELECT 1 FROM accounts a WHERE a.id=d.account_id) RETURNING 1),
  tuples AS (INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM pair p CROSS JOIN LATERAL (SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision FROM messages m WHERE m.stable_id=p.stable_id AND m.account_id=p.new_account LIMIT 1) m JOIN accounts parent ON parent.id=m.account_id WHERE p.changed ORDER BY m.stable_id ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision RETURNING 1),
  stamped AS (UPDATE mail_data_rows r SET cache_revision=p.cache_revision,content_revision=p.content_revision FROM pair p WHERE r.stable_id=p.stable_id AND r.account_id=p.new_account AND p.stamp AND NOT (p.changed AND EXISTS(SELECT 1 FROM accounts parent WHERE parent.id=p.new_account)) AND (r.cache_revision IS DISTINCT FROM p.cache_revision OR r.content_revision IS DISTINCT FROM p.content_revision) RETURNING 1),
  outboxed AS (INSERT INTO mail_data_outbox(account_id,revision) SELECT l.account_id,md5(random()::text||clock_timestamp()::text) FROM live l ORDER BY l.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision RETURNING 1)
  INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id IN(SELECT x.account_id FROM pair p CROSS JOIN LATERAL (VALUES(p.old_account),(p.new_account)) x(account_id) WHERE p.changed) ORDER BY a.id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1,focus_revision=CASE WHEN mail_data_accounts.account_id IN(SELECT x.account_id FROM pair p CROSS JOIN LATERAL (VALUES(p.old_account),(p.new_account)) x(account_id) WHERE p.focus) THEN excluded.focus_revision ELSE mail_data_accounts.focus_revision END;
 ELSE
  WITH pair AS MATERIALIZED (SELECT q.stable_id,q.old_account,q.new_account,q.cache_revision,q.content_revision,q.changed,CASE WHEN q.focus_changed THEN EXISTS(SELECT 1 FROM mailboxes box WHERE box.id IN(q.old_mailbox,q.new_mailbox) AND box.role='sent') ELSE false END AS focus,q.stamp FROM (SELECT u.side,lag(u.side) OVER w AS old_side,u.stable_id,lag(u.account_id) OVER w AS old_account,u.account_id AS new_account,lag(u.mailbox_id) OVER w AS old_mailbox,u.mailbox_id AS new_mailbox,u.cache_revision,u.content_revision,(u.content_revision IS DISTINCT FROM lag(u.content_revision) OVER w OR u.envelope_json IS DISTINCT FROM lag(u.envelope_json) OVER w OR u.account_id IS DISTINCT FROM lag(u.account_id) OVER w OR u.blob_ref IS DISTINCT FROM lag(u.blob_ref) OVER w OR u.mailbox_id IS DISTINCT FROM lag(u.mailbox_id) OVER w OR u.uid IS DISTINCT FROM lag(u.uid) OVER w OR u.uidvalidity IS DISTINCT FROM lag(u.uidvalidity) OVER w OR u.message_id IS DISTINCT FROM lag(u.message_id) OVER w OR u.thread_id IS DISTINCT FROM lag(u.thread_id) OVER w OR u.internaldate IS DISTINCT FROM lag(u.internaldate) OVER w OR u.size IS DISTINCT FROM lag(u.size) OVER w OR u.flags_json IS DISTINCT FROM lag(u.flags_json) OVER w) AS changed,(u.mailbox_id IS DISTINCT FROM lag(u.mailbox_id) OVER w OR u.thread_id IS DISTINCT FROM lag(u.thread_id) OVER w OR u.envelope_json IS DISTINCT FROM lag(u.envelope_json) OVER w OR u.blob_ref IS DISTINCT FROM lag(u.blob_ref) OVER w OR u.content_revision IS DISTINCT FROM lag(u.content_revision) OVER w OR u.internaldate IS DISTINCT FROM lag(u.internaldate) OVER w OR u.uid IS DISTINCT FROM lag(u.uid) OVER w OR u.account_id IS DISTINCT FROM lag(u.account_id) OVER w) AS focus_changed,(u.cache_revision IS DISTINCT FROM lag(u.cache_revision) OVER w OR u.content_revision IS DISTINCT FROM lag(u.content_revision) OVER w) AS stamp FROM (SELECT 0 AS side,stable_id,account_id,mailbox_id,uid,uidvalidity,message_id,thread_id,internaldate,size,flags_json,envelope_json,blob_ref,cache_revision,content_revision FROM mail_data_old UNION ALL SELECT 1,stable_id,account_id,mailbox_id,uid,uidvalidity,message_id,thread_id,internaldate,size,flags_json,envelope_json,blob_ref,cache_revision,content_revision FROM mail_data_new) u WHERE NOT u.stable_id=ANY(rekeyed) WINDOW w AS (PARTITION BY u.stable_id ORDER BY u.side)) q WHERE q.side=1 AND q.old_side=0),
  live AS MATERIALIZED (SELECT a.id AS account_id FROM accounts a WHERE a.id IN(SELECT x.account_id FROM pair p CROSS JOIN LATERAL (VALUES(p.old_account),(p.new_account)) x(account_id) WHERE p.changed)),
  orphans AS (UPDATE mail_data_accounts d SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE d.account_id IN(SELECT x.account_id FROM pair p CROSS JOIN LATERAL (VALUES(p.old_account),(p.new_account)) x(account_id) WHERE p.focus) AND NOT EXISTS(SELECT 1 FROM accounts a WHERE a.id=d.account_id) RETURNING 1),
  tuples AS (INSERT INTO mail_data_rows(stable_id,account_id,cache_revision,content_revision,data_revision) SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision,md5(random()::text||clock_timestamp()::text) FROM pair p CROSS JOIN LATERAL (SELECT m.stable_id,m.account_id,m.cache_revision,m.content_revision FROM messages m WHERE m.stable_id=p.stable_id AND m.account_id=p.new_account LIMIT 1) m JOIN accounts parent ON parent.id=m.account_id WHERE p.changed ORDER BY m.stable_id ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,cache_revision=excluded.cache_revision,content_revision=excluded.content_revision,data_revision=excluded.data_revision RETURNING 1),
  stamped AS (UPDATE mail_data_rows r SET cache_revision=p.cache_revision,content_revision=p.content_revision FROM pair p WHERE r.stable_id=p.stable_id AND r.account_id=p.new_account AND p.stamp AND NOT (p.changed AND EXISTS(SELECT 1 FROM accounts parent WHERE parent.id=p.new_account)) AND (r.cache_revision IS DISTINCT FROM p.cache_revision OR r.content_revision IS DISTINCT FROM p.content_revision) RETURNING 1),
  outboxed AS (INSERT INTO mail_data_outbox(account_id,revision) SELECT l.account_id,md5(random()::text||clock_timestamp()::text) FROM live l ORDER BY l.account_id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision RETURNING 1)
  INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id IN(SELECT x.account_id FROM pair p CROSS JOIN LATERAL (VALUES(p.old_account),(p.new_account)) x(account_id) WHERE p.changed) ORDER BY a.id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1,focus_revision=CASE WHEN mail_data_accounts.account_id IN(SELECT x.account_id FROM pair p CROSS JOIN LATERAL (VALUES(p.old_account),(p.new_account)) x(account_id) WHERE p.focus) THEN excluded.focus_revision ELSE mail_data_accounts.focus_revision END;
 END IF;
 RETURN NULL;
END $$;

CREATE OR REPLACE FUNCTION mail_data_messages_deleted_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE acc TEXT;
BEGIN
 -- One account (every single-message delete) whose account row exists: the row's
 -- UPDATE finds the account itself, then the outbox. Anything else: 0131's statement.
 UPDATE mail_data_accounts SET seq=seq+1,focus_revision=md5(random()::text||clock_timestamp()::text) WHERE account_id=(SELECT min(o.account_id) FROM mail_data_old o) AND account_id=(SELECT max(o.account_id) FROM mail_data_old o) RETURNING account_id INTO acc;
 IF acc IS NOT NULL THEN
  INSERT INTO mail_data_outbox(account_id,revision) SELECT a.id,md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id IN(acc) ORDER BY a.id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision;
  RETURN NULL;
 END IF;
 IF NOT EXISTS(SELECT 1 FROM mail_data_old) THEN RETURN NULL; END IF;
 -- More rows: 0131's statement below.
 WITH orphans AS (UPDATE mail_data_accounts d SET focus_revision=md5(random()::text||clock_timestamp()::text) WHERE d.account_id IN(SELECT o.account_id FROM mail_data_old o) AND NOT EXISTS(SELECT 1 FROM accounts a WHERE a.id=d.account_id) RETURNING 1),
 outboxed AS (INSERT INTO mail_data_outbox(account_id,revision) SELECT a.id,md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id IN(SELECT o.account_id FROM mail_data_old o) ORDER BY a.id ON CONFLICT(account_id) DO UPDATE SET revision=excluded.revision RETURNING 1)
 INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT a.id,md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text),md5(random()::text||clock_timestamp()::text) FROM accounts a WHERE a.id IN(SELECT o.account_id FROM mail_data_old o) ORDER BY a.id ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1,focus_revision=excluded.focus_revision;
 RETURN NULL;
END $$;

DO $$ DECLARE function_name TEXT; BEGIN
 FOREACH function_name IN ARRAY ARRAY['mail_data_messages_inserted_fn','mail_data_messages_updated_fn','mail_data_messages_deleted_fn'] LOOP
  EXECUTE format('ALTER FUNCTION %I() SET search_path = pg_catalog, %I',function_name,current_schema());
 END LOOP;
END $$;
