-- t50hp-e30: message metadata projection (design t50hp-e29 parts 4/5). Two tables
-- derived from `messages` and kept by the triggers below, so that a metadata query
-- (unread, flagged, keyword, folder, date, size) can be answered from SQL. Nothing
-- reads them yet; mw-store message_projection.rs holds the query builders and an
-- audit that recomputes both tables in Rust.
--
-- Derivation (must equal message_projection.rs, which the tests compare):
-- * seen = the flags value is a JSON array holding the JSON string "Seen". This is
--   exactly NOT flags_json_is_unread (cache.rs), the rule of the 0122 counters, so
--   COUNT(seen=0) in a folder equals mailboxes.unread.
-- * keyword rows = the JMAP keywords mw-engine shows for the value:
--   flags_to_keywords(flags_from_json(..)). flags_from_json is serde's Vec<Flag>
--   or nothing, so ONE element that is not a Flag ("seen", 5, an object with two
--   keys, {"Keyword":5}) leaves no keyword at all. Seen/Answered/Flagged/Draft map
--   to $seen/$answered/$flagged/$draft (also in serde's {"Seen":null} form),
--   Deleted/Recent map to nothing, {"Keyword":k} gives k verbatim (case kept).
-- * flagged = the keyword rows hold "$flagged".
-- A malformed or non-array value is an empty set and never an error, so a corrupt
-- value cannot fail the statement that writes it. (json_type raises on invalid
-- JSON, so every json_type sits behind json_valid in a nested CASE.)
--
-- Rowid tables, not WITHOUT ROWID: online SQLite maintenance (online_copy.rs) copies
-- and replays by rowid and refuses a table without one.
-- No foreign keys (rule of 0113/0116): an FK insert from a messages trigger would
-- take a parent lock after the message row lock. The rows go with their message
-- (delete trigger, which also fires for an FK cascade from accounts/mailboxes and,
-- with recursive_triggers ON, for REPLACE), and erasure deletes both tables by
-- account_id after the sources (data_map.rs). The trigger bodies use no conflict
-- clause and never insert a key that can already exist: an outer INSERT OR <x>
-- would otherwise impose its own conflict policy on them.
CREATE TABLE message_state (
 stable_id TEXT NOT NULL PRIMARY KEY,
 account_id TEXT NOT NULL,
 mailbox_id TEXT NOT NULL,
 internaldate TEXT,
 uid INTEGER NOT NULL,
 size INTEGER NOT NULL,
 seen INTEGER NOT NULL,
 flagged INTEGER NOT NULL
);
CREATE TABLE message_keyword (
 stable_id TEXT NOT NULL,
 account_id TEXT NOT NULL,
 keyword TEXT NOT NULL,
 PRIMARY KEY (stable_id, keyword)
);

-- backfill
INSERT INTO message_keyword(stable_id,account_id,keyword)
SELECT DISTINCT p.stable_id,p.account_id,p.k FROM
 (SELECT m.stable_id,m.account_id,CASE e.type WHEN 'text' THEN CASE e.value WHEN 'Seen' THEN '$seen' WHEN 'Answered' THEN '$answered' WHEN 'Flagged' THEN '$flagged' WHEN 'Draft' THEN '$draft' END ELSE (SELECT CASE o.key WHEN 'Keyword' THEN o.value WHEN 'Seen' THEN '$seen' WHEN 'Answered' THEN '$answered' WHEN 'Flagged' THEN '$flagged' WHEN 'Draft' THEN '$draft' END FROM json_each(e.value) o) END AS k
  FROM messages m, json_each(CASE WHEN CASE WHEN json_valid(m.flags_json) THEN CASE WHEN json_type(m.flags_json)='array' THEN NOT EXISTS(SELECT 1 FROM json_each(m.flags_json) v WHERE CASE v.type WHEN 'text' THEN v.value NOT IN ('Seen','Answered','Flagged','Deleted','Draft','Recent') WHEN 'object' THEN (SELECT COUNT(*)<>1 OR SUM((o.key='Keyword' AND o.type='text') OR (o.key IN ('Seen','Answered','Flagged','Deleted','Draft','Recent') AND o.type='null'))<>1 FROM json_each(v.value) o) ELSE 1 END) ELSE 0 END ELSE 0 END THEN m.flags_json ELSE '[]' END) e) p
 WHERE p.k IS NOT NULL;
INSERT INTO message_state(stable_id,account_id,mailbox_id,internaldate,uid,size,seen,flagged)
SELECT m.stable_id,m.account_id,m.mailbox_id,m.internaldate,m.uid,m.size,
 CASE WHEN json_valid(m.flags_json) THEN CASE WHEN json_type(m.flags_json)='array' THEN EXISTS(SELECT 1 FROM json_each(m.flags_json) WHERE type='text' AND value='Seen') ELSE 0 END ELSE 0 END,
 EXISTS(SELECT 1 FROM message_keyword k WHERE k.stable_id=m.stable_id AND k.keyword='$flagged')
FROM messages m;
-- end backfill

-- Partial indexes: a query uses one only with the literal seen=0 / flagged=1
-- (a bound parameter cannot prove the predicate); message_projection.rs emits them.
CREATE INDEX message_keyword_by_kw ON message_keyword(account_id,keyword,stable_id);
CREATE INDEX message_state_unread ON message_state(mailbox_id,internaldate DESC,uid DESC,stable_id) WHERE seen=0;
CREATE INDEX message_state_account_unread ON message_state(account_id,internaldate DESC,uid DESC,stable_id) WHERE seen=0;
CREATE INDEX message_state_flagged ON message_state(account_id,internaldate DESC,uid DESC,stable_id) WHERE flagged=1;
CREATE INDEX message_state_size ON message_state(mailbox_id,size DESC,uid DESC,stable_id);

-- Every statement below is keyed by the message's own stable_id: no shared row.
-- The delete of a leftover row before an insert keeps the bodies conflict-free.
CREATE TRIGGER message_projection_insert AFTER INSERT ON messages
BEGIN
 DELETE FROM message_keyword WHERE stable_id=NEW.stable_id;
 DELETE FROM message_state WHERE stable_id=NEW.stable_id;
 INSERT INTO message_keyword(stable_id,account_id,keyword) SELECT DISTINCT NEW.stable_id,NEW.account_id,p.k FROM (SELECT CASE e.type WHEN 'text' THEN CASE e.value WHEN 'Seen' THEN '$seen' WHEN 'Answered' THEN '$answered' WHEN 'Flagged' THEN '$flagged' WHEN 'Draft' THEN '$draft' END ELSE (SELECT CASE o.key WHEN 'Keyword' THEN o.value WHEN 'Seen' THEN '$seen' WHEN 'Answered' THEN '$answered' WHEN 'Flagged' THEN '$flagged' WHEN 'Draft' THEN '$draft' END FROM json_each(e.value) o) END AS k FROM json_each(CASE WHEN CASE WHEN json_valid(NEW.flags_json) THEN CASE WHEN json_type(NEW.flags_json)='array' THEN NOT EXISTS(SELECT 1 FROM json_each(NEW.flags_json) v WHERE CASE v.type WHEN 'text' THEN v.value NOT IN ('Seen','Answered','Flagged','Deleted','Draft','Recent') WHEN 'object' THEN (SELECT COUNT(*)<>1 OR SUM((o.key='Keyword' AND o.type='text') OR (o.key IN ('Seen','Answered','Flagged','Deleted','Draft','Recent') AND o.type='null'))<>1 FROM json_each(v.value) o) ELSE 1 END) ELSE 0 END ELSE 0 END THEN NEW.flags_json ELSE '[]' END) e) p WHERE p.k IS NOT NULL;
 INSERT INTO message_state(stable_id,account_id,mailbox_id,internaldate,uid,size,seen,flagged) VALUES(NEW.stable_id,NEW.account_id,NEW.mailbox_id,NEW.internaldate,NEW.uid,NEW.size,CASE WHEN json_valid(NEW.flags_json) THEN CASE WHEN json_type(NEW.flags_json)='array' THEN EXISTS(SELECT 1 FROM json_each(NEW.flags_json) WHERE type='text' AND value='Seen') ELSE 0 END ELSE 0 END,EXISTS(SELECT 1 FROM message_keyword WHERE stable_id=NEW.stable_id AND keyword='$flagged'));
END;

-- 0105's nested cache_revision UPDATE changes none of these columns, so it stops at
-- the WHEN. Keyword rows are recomputed only when flags, owner or id changed; the
-- diff (delete what is gone, add what is new) leaves unchanged rows alone. seen and
-- flagged are set only when flags changed and the location columns only when one
-- of them changed, so an index is rewritten only for a column the write touched.
CREATE TRIGGER message_projection_update AFTER UPDATE ON messages
WHEN NEW.flags_json IS NOT OLD.flags_json OR NEW.mailbox_id IS NOT OLD.mailbox_id OR NEW.account_id IS NOT OLD.account_id OR NEW.stable_id IS NOT OLD.stable_id OR NEW.internaldate IS NOT OLD.internaldate OR NEW.uid IS NOT OLD.uid OR NEW.size IS NOT OLD.size
BEGIN
 DELETE FROM message_keyword WHERE stable_id=OLD.stable_id AND NEW.stable_id IS NOT OLD.stable_id;
 DELETE FROM message_state WHERE stable_id=OLD.stable_id AND NEW.stable_id IS NOT OLD.stable_id;
 DELETE FROM message_keyword WHERE stable_id=NEW.stable_id AND (NEW.flags_json IS NOT OLD.flags_json OR NEW.account_id IS NOT OLD.account_id OR NEW.stable_id IS NOT OLD.stable_id) AND (account_id IS NOT NEW.account_id OR keyword NOT IN (SELECT p.k FROM (SELECT CASE e.type WHEN 'text' THEN CASE e.value WHEN 'Seen' THEN '$seen' WHEN 'Answered' THEN '$answered' WHEN 'Flagged' THEN '$flagged' WHEN 'Draft' THEN '$draft' END ELSE (SELECT CASE o.key WHEN 'Keyword' THEN o.value WHEN 'Seen' THEN '$seen' WHEN 'Answered' THEN '$answered' WHEN 'Flagged' THEN '$flagged' WHEN 'Draft' THEN '$draft' END FROM json_each(e.value) o) END AS k FROM json_each(CASE WHEN CASE WHEN json_valid(NEW.flags_json) THEN CASE WHEN json_type(NEW.flags_json)='array' THEN NOT EXISTS(SELECT 1 FROM json_each(NEW.flags_json) v WHERE CASE v.type WHEN 'text' THEN v.value NOT IN ('Seen','Answered','Flagged','Deleted','Draft','Recent') WHEN 'object' THEN (SELECT COUNT(*)<>1 OR SUM((o.key='Keyword' AND o.type='text') OR (o.key IN ('Seen','Answered','Flagged','Deleted','Draft','Recent') AND o.type='null'))<>1 FROM json_each(v.value) o) ELSE 1 END) ELSE 0 END ELSE 0 END THEN NEW.flags_json ELSE '[]' END) e) p WHERE p.k IS NOT NULL));
 INSERT INTO message_keyword(stable_id,account_id,keyword) SELECT DISTINCT NEW.stable_id,NEW.account_id,p.k FROM (SELECT CASE e.type WHEN 'text' THEN CASE e.value WHEN 'Seen' THEN '$seen' WHEN 'Answered' THEN '$answered' WHEN 'Flagged' THEN '$flagged' WHEN 'Draft' THEN '$draft' END ELSE (SELECT CASE o.key WHEN 'Keyword' THEN o.value WHEN 'Seen' THEN '$seen' WHEN 'Answered' THEN '$answered' WHEN 'Flagged' THEN '$flagged' WHEN 'Draft' THEN '$draft' END FROM json_each(e.value) o) END AS k FROM json_each(CASE WHEN CASE WHEN json_valid(NEW.flags_json) THEN CASE WHEN json_type(NEW.flags_json)='array' THEN NOT EXISTS(SELECT 1 FROM json_each(NEW.flags_json) v WHERE CASE v.type WHEN 'text' THEN v.value NOT IN ('Seen','Answered','Flagged','Deleted','Draft','Recent') WHEN 'object' THEN (SELECT COUNT(*)<>1 OR SUM((o.key='Keyword' AND o.type='text') OR (o.key IN ('Seen','Answered','Flagged','Deleted','Draft','Recent') AND o.type='null'))<>1 FROM json_each(v.value) o) ELSE 1 END) ELSE 0 END ELSE 0 END THEN NEW.flags_json ELSE '[]' END) e) p WHERE p.k IS NOT NULL AND (NEW.flags_json IS NOT OLD.flags_json OR NEW.account_id IS NOT OLD.account_id OR NEW.stable_id IS NOT OLD.stable_id) AND p.k NOT IN (SELECT keyword FROM message_keyword WHERE stable_id=NEW.stable_id);
 UPDATE message_state SET seen=CASE WHEN json_valid(NEW.flags_json) THEN CASE WHEN json_type(NEW.flags_json)='array' THEN EXISTS(SELECT 1 FROM json_each(NEW.flags_json) WHERE type='text' AND value='Seen') ELSE 0 END ELSE 0 END,flagged=EXISTS(SELECT 1 FROM message_keyword WHERE stable_id=NEW.stable_id AND keyword='$flagged') WHERE stable_id=NEW.stable_id AND NEW.flags_json IS NOT OLD.flags_json;
 UPDATE message_state SET account_id=NEW.account_id,mailbox_id=NEW.mailbox_id,internaldate=NEW.internaldate,uid=NEW.uid,size=NEW.size WHERE stable_id=NEW.stable_id AND (NEW.mailbox_id IS NOT OLD.mailbox_id OR NEW.account_id IS NOT OLD.account_id OR NEW.internaldate IS NOT OLD.internaldate OR NEW.uid IS NOT OLD.uid OR NEW.size IS NOT OLD.size);
 INSERT INTO message_state(stable_id,account_id,mailbox_id,internaldate,uid,size,seen,flagged) SELECT NEW.stable_id,NEW.account_id,NEW.mailbox_id,NEW.internaldate,NEW.uid,NEW.size,CASE WHEN json_valid(NEW.flags_json) THEN CASE WHEN json_type(NEW.flags_json)='array' THEN EXISTS(SELECT 1 FROM json_each(NEW.flags_json) WHERE type='text' AND value='Seen') ELSE 0 END ELSE 0 END,EXISTS(SELECT 1 FROM message_keyword WHERE stable_id=NEW.stable_id AND keyword='$flagged') WHERE NOT EXISTS(SELECT 1 FROM message_state WHERE stable_id=NEW.stable_id);
END;

CREATE TRIGGER message_projection_delete AFTER DELETE ON messages
BEGIN
 DELETE FROM message_keyword WHERE stable_id=OLD.stable_id;
 DELETE FROM message_state WHERE stable_id=OLD.stable_id;
END;
