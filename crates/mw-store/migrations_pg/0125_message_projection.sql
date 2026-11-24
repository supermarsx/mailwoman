-- t50hp-e30: message metadata projection, PostgreSQL. Same tables, rules and
-- contract as the SQLite 0125 (read that header). Dialect notes:
-- * seen/flagged are BIGINT 0/1, not BOOLEAN: the house rule since 0004/0012 (the
--   V6 0007 bool-bind bug), and it lets one query text say seen=0 on both backends.
-- * The value is parsed as json, not jsonb: json keeps duplicate object keys, which
--   serde rejects for a Flag ({"Keyword":"a","Keyword":"b"}); jsonb would merge them.
-- * mw_message_flags catches every error of the cast and of the de-escaping (a value
--   that is not JSON, or "\u0000", which text cannot hold) and returns the empty set,
--   so a corrupt value never fails the writing statement. The block only reads, so
--   its subtransaction is never assigned an XID.
-- * No foreign keys (see SQLite). The update trigger lists its columns and has a
--   WHEN clause, so an UPDATE that changes none of them (0105's cache_revision, a
--   body or envelope write) never enters the function.
CREATE TABLE message_state (
 stable_id TEXT NOT NULL PRIMARY KEY,
 account_id TEXT NOT NULL,
 mailbox_id TEXT NOT NULL,
 internaldate TEXT,
 uid BIGINT NOT NULL,
 size BIGINT NOT NULL,
 seen BIGINT NOT NULL,
 flagged BIGINT NOT NULL
);
CREATE TABLE message_keyword (
 stable_id TEXT NOT NULL,
 account_id TEXT NOT NULL,
 keyword TEXT NOT NULL,
 PRIMARY KEY (stable_id, keyword)
);

CREATE FUNCTION mw_message_flags(flags TEXT, OUT seen BIGINT, OUT flagged BIGINT, OUT keywords TEXT[]) LANGUAGE plpgsql IMMUTABLE AS $$
DECLARE
 parsed json;
 complete BOOLEAN;
BEGIN
 seen := 0;
 flagged := 0;
 keywords := '{}';
 BEGIN
  parsed := flags::json;
  IF json_typeof(parsed) <> 'array' THEN
   RETURN;
  END IF;
  SELECT CASE WHEN COALESCE(bool_or(x.s = 'Seen'), false) THEN 1 ELSE 0 END,
         COALESCE(bool_and(x.ok), true),
         COALESCE(array_agg(DISTINCT x.k) FILTER (WHERE x.k IS NOT NULL), '{}')
    INTO seen, complete, keywords
    FROM (SELECT CASE WHEN json_typeof(e.value) = 'string' THEN e.value #>> '{}' END AS s,
                 CASE json_typeof(e.value)
                  WHEN 'string' THEN (e.value #>> '{}') IN ('Seen','Answered','Flagged','Deleted','Draft','Recent')
                  WHEN 'object' THEN o.n = 1 AND ((o.name = 'Keyword' AND o.kind = 'string') OR (o.name IN ('Seen','Answered','Flagged','Deleted','Draft','Recent') AND o.kind = 'null'))
                  ELSE false END AS ok,
                 CASE CASE json_typeof(e.value) WHEN 'string' THEN e.value #>> '{}' WHEN 'object' THEN o.name END
                  WHEN 'Seen' THEN '$seen' WHEN 'Answered' THEN '$answered' WHEN 'Flagged' THEN '$flagged' WHEN 'Draft' THEN '$draft'
                  WHEN 'Keyword' THEN CASE WHEN json_typeof(e.value) = 'object' THEN o.word END
                 END AS k
            FROM json_array_elements(parsed) e
            LEFT JOIN LATERAL (SELECT count(*) AS n, min(p.key) AS name, min(json_typeof(p.value)) AS kind, min(p.value #>> '{}') AS word
                                 FROM json_each(CASE WHEN json_typeof(e.value) = 'object' THEN e.value ELSE '{}'::json END) p) o ON true) x;
 EXCEPTION WHEN others THEN
  seen := 0;
  keywords := '{}';
  RETURN;
 END;
 IF NOT complete THEN
  keywords := '{}';
 END IF;
 flagged := CASE WHEN '$flagged' = ANY(keywords) THEN 1 ELSE 0 END;
END $$;

-- backfill
WITH p AS (
 SELECT m.stable_id, m.account_id, m.mailbox_id, m.internaldate, m.uid, m.size, f.seen, f.flagged, f.keywords
 FROM messages m CROSS JOIN LATERAL mw_message_flags(m.flags_json) f
), s AS (
 INSERT INTO message_state(stable_id,account_id,mailbox_id,internaldate,uid,size,seen,flagged)
 SELECT stable_id,account_id,mailbox_id,internaldate,uid,size,seen,flagged FROM p
)
INSERT INTO message_keyword(stable_id,account_id,keyword)
SELECT p.stable_id, p.account_id, k FROM p CROSS JOIN LATERAL unnest(p.keywords) k;
-- end backfill

-- Partial indexes need the literal seen=0 / flagged=1 in the query (see SQLite).
CREATE INDEX message_keyword_by_kw ON message_keyword(account_id,keyword,stable_id);
CREATE INDEX message_state_unread ON message_state(mailbox_id,internaldate DESC NULLS LAST,uid DESC,stable_id) WHERE seen=0;
CREATE INDEX message_state_account_unread ON message_state(account_id,internaldate DESC NULLS LAST,uid DESC,stable_id) WHERE seen=0;
CREATE INDEX message_state_flagged ON message_state(account_id,internaldate DESC NULLS LAST,uid DESC,stable_id) WHERE flagged=1;
CREATE INDEX message_state_size ON message_state(mailbox_id,size DESC,uid DESC,stable_id);

-- Every statement is keyed by the message's own stable_id: no shared row, so the
-- per-row cost does not grow with the statement (unlike 0113's account rows).
CREATE FUNCTION message_projection_insert_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE
 p record;
BEGIN
 SELECT * INTO p FROM mw_message_flags(NEW.flags_json);
 DELETE FROM message_keyword WHERE stable_id = NEW.stable_id;
 INSERT INTO message_keyword(stable_id,account_id,keyword) SELECT NEW.stable_id, NEW.account_id, k FROM unnest(p.keywords) k;
 INSERT INTO message_state(stable_id,account_id,mailbox_id,internaldate,uid,size,seen,flagged)
  VALUES(NEW.stable_id, NEW.account_id, NEW.mailbox_id, NEW.internaldate, NEW.uid, NEW.size, p.seen, p.flagged)
  ON CONFLICT (stable_id) DO UPDATE SET account_id = EXCLUDED.account_id, mailbox_id = EXCLUDED.mailbox_id, internaldate = EXCLUDED.internaldate, uid = EXCLUDED.uid, size = EXCLUDED.size, seen = EXCLUDED.seen, flagged = EXCLUDED.flagged;
 RETURN NULL;
END $$;
CREATE TRIGGER message_projection_insert AFTER INSERT ON messages FOR EACH ROW EXECUTE FUNCTION message_projection_insert_fn();

CREATE FUNCTION message_projection_update_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE
 p record;
BEGIN
 IF NEW.stable_id IS DISTINCT FROM OLD.stable_id THEN
  DELETE FROM message_keyword WHERE stable_id IN (OLD.stable_id, NEW.stable_id);
  DELETE FROM message_state WHERE stable_id = OLD.stable_id;
 END IF;
 IF NEW.flags_json IS DISTINCT FROM OLD.flags_json OR NEW.account_id IS DISTINCT FROM OLD.account_id OR NEW.stable_id IS DISTINCT FROM OLD.stable_id THEN
  SELECT * INTO p FROM mw_message_flags(NEW.flags_json);
  DELETE FROM message_keyword WHERE stable_id = NEW.stable_id AND (account_id IS DISTINCT FROM NEW.account_id OR keyword <> ALL(p.keywords));
  INSERT INTO message_keyword(stable_id,account_id,keyword) SELECT NEW.stable_id, NEW.account_id, k FROM unnest(p.keywords) k ON CONFLICT DO NOTHING;
  INSERT INTO message_state(stable_id,account_id,mailbox_id,internaldate,uid,size,seen,flagged)
   VALUES(NEW.stable_id, NEW.account_id, NEW.mailbox_id, NEW.internaldate, NEW.uid, NEW.size, p.seen, p.flagged)
   ON CONFLICT (stable_id) DO UPDATE SET account_id = EXCLUDED.account_id, mailbox_id = EXCLUDED.mailbox_id, internaldate = EXCLUDED.internaldate, uid = EXCLUDED.uid, size = EXCLUDED.size, seen = EXCLUDED.seen, flagged = EXCLUDED.flagged
   WHERE (message_state.account_id, message_state.mailbox_id, message_state.internaldate, message_state.uid, message_state.size, message_state.seen, message_state.flagged)
    IS DISTINCT FROM (EXCLUDED.account_id, EXCLUDED.mailbox_id, EXCLUDED.internaldate, EXCLUDED.uid, EXCLUDED.size, EXCLUDED.seen, EXCLUDED.flagged);
 ELSE
  UPDATE message_state SET account_id = NEW.account_id, mailbox_id = NEW.mailbox_id, internaldate = NEW.internaldate, uid = NEW.uid, size = NEW.size WHERE stable_id = NEW.stable_id;
  IF NOT FOUND THEN
   SELECT * INTO p FROM mw_message_flags(NEW.flags_json);
   INSERT INTO message_state(stable_id,account_id,mailbox_id,internaldate,uid,size,seen,flagged)
    VALUES(NEW.stable_id, NEW.account_id, NEW.mailbox_id, NEW.internaldate, NEW.uid, NEW.size, p.seen, p.flagged)
    ON CONFLICT (stable_id) DO NOTHING;
  END IF;
 END IF;
 RETURN NULL;
END $$;
CREATE TRIGGER message_projection_update AFTER UPDATE OF flags_json, mailbox_id, account_id, stable_id, internaldate, uid, size ON messages FOR EACH ROW
 WHEN (OLD.flags_json IS DISTINCT FROM NEW.flags_json OR OLD.mailbox_id IS DISTINCT FROM NEW.mailbox_id OR OLD.account_id IS DISTINCT FROM NEW.account_id OR OLD.stable_id IS DISTINCT FROM NEW.stable_id OR OLD.internaldate IS DISTINCT FROM NEW.internaldate OR OLD.uid IS DISTINCT FROM NEW.uid OR OLD.size IS DISTINCT FROM NEW.size)
 EXECUTE FUNCTION message_projection_update_fn();

CREATE FUNCTION message_projection_delete_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
 DELETE FROM message_keyword WHERE stable_id = OLD.stable_id;
 DELETE FROM message_state WHERE stable_id = OLD.stable_id;
 RETURN NULL;
END $$;
CREATE TRIGGER message_projection_delete AFTER DELETE ON messages FOR EACH ROW EXECUTE FUNCTION message_projection_delete_fn();

-- House rule (0113, 0120): every function a trigger reaches runs with a pinned
-- search_path, so a writer's own search_path cannot redirect these statements.
DO $$ BEGIN
 EXECUTE format('ALTER FUNCTION %I.mw_message_flags(TEXT) SET search_path TO pg_catalog,%I',current_schema(),current_schema());
 EXECUTE format('ALTER FUNCTION %I.message_projection_insert_fn() SET search_path TO pg_catalog,%I',current_schema(),current_schema());
 EXECUTE format('ALTER FUNCTION %I.message_projection_update_fn() SET search_path TO pg_catalog,%I',current_schema(),current_schema());
 EXECUTE format('ALTER FUNCTION %I.message_projection_delete_fn() SET search_path TO pg_catalog,%I',current_schema(),current_schema());
END $$;
