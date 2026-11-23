-- t34 schema parity only: no PostgreSQL reader exists and the bounded guard
-- snapshot stays off on PostgreSQL. Same rows and pruning as SQLite.
-- A future PostgreSQL reader must not page by seq: sequence values are taken
-- at insert time, not commit time, so a lower seq can become visible after a
-- higher one. It needs a commit-horizon design (for example an xid8 column and
-- pg_snapshot_xmin paging) added by its own migration.
CREATE TABLE authority_change_log (
 seq BIGSERIAL PRIMARY KEY,
 scope_kind TEXT NOT NULL,
 incarnation TEXT NOT NULL,
 at BIGINT NOT NULL
);
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
CREATE FUNCTION mw_authority_change_log_prune() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 DELETE FROM authority_change_log WHERE seq<=NEW.seq-65536;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_change_log_prune AFTER INSERT ON authority_change_log FOR EACH ROW EXECUTE FUNCTION mw_authority_change_log_prune();
DO $bind_change_log_schema$
BEGIN
 EXECUTE format('ALTER FUNCTION %I.mw_authority_change_log_epoch() SET search_path TO pg_catalog,%I',current_schema(),current_schema());
 EXECUTE format('ALTER FUNCTION %I.mw_authority_change_log_revision() SET search_path TO pg_catalog,%I',current_schema(),current_schema());
 EXECUTE format('ALTER FUNCTION %I.mw_authority_change_log_prune() SET search_path TO pg_catalog,%I',current_schema(),current_schema());
END;
$bind_change_log_schema$;
