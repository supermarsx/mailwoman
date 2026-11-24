-- t49pg: a PostgreSQL reader for authority_change_log (bounded guard snapshot).
-- SEQ values are taken at insert time, not at commit time, so a reader cannot
-- page by seq. Each row records the top-level transaction id of its writer
-- (pg_current_xact_id() returns the top-level id inside subtransactions too);
-- the reader pages by snapshot visibility of that id (see
-- crates/mw-store/src/authority_publication/mirror.rs).
-- The two log triggers append only. Unlike 0115 there is no per-insert prune
-- (0118, t34 S2): rows are deleted by the reader's background pruner, outside
-- every writer's transaction.
ALTER TABLE authority_change_log ADD COLUMN xid xid8 NOT NULL DEFAULT pg_current_xact_id();
CREATE INDEX authority_change_log_xid ON authority_change_log(xid);
CREATE FUNCTION mw_authority_change_log_epoch() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
  INSERT INTO authority_change_log(scope_kind,incarnation,at) VALUES(NEW.scope_kind,NEW.incarnation,floor(extract(epoch FROM clock_timestamp())*1000)::bigint);
 ELSE
  INSERT INTO authority_change_log(scope_kind,incarnation,at) VALUES(OLD.scope_kind,OLD.incarnation,floor(extract(epoch FROM clock_timestamp())*1000)::bigint);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_change_log_epoch AFTER INSERT OR UPDATE OR DELETE ON authority_epoch FOR EACH ROW EXECUTE FUNCTION mw_authority_change_log_epoch();
CREATE FUNCTION mw_authority_change_log_revision() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO authority_change_log(scope_kind,incarnation,at) VALUES('settings-revision','',floor(extract(epoch FROM clock_timestamp())*1000)::bigint);
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_change_log_revision AFTER INSERT OR UPDATE OR DELETE ON settings_revision FOR EACH ROW EXECUTE FUNCTION mw_authority_change_log_revision();
-- 0078's settings_revision_bump() had no pinned search_path. Pin it like every
-- 0111/0115 function, so the publication schema validator can require the
-- 0078 triggers on PostgreSQL as well (t49pg, review N1).
DO $bind_change_log_schema$
BEGIN
 EXECUTE format('ALTER FUNCTION %I.mw_authority_change_log_epoch() SET search_path TO pg_catalog,%I',current_schema(),current_schema());
 EXECUTE format('ALTER FUNCTION %I.mw_authority_change_log_revision() SET search_path TO pg_catalog,%I',current_schema(),current_schema());
 EXECUTE format('ALTER FUNCTION %I.settings_revision_bump() SET search_path TO pg_catalog,%I',current_schema(),current_schema());
END;
$bind_change_log_schema$;
