-- Integrate only after 0113 mail-data publication. No source/backfill scan.
CREATE FUNCTION mw_authority_guard_config() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='UPDATE' AND NEW.key IS NOT DISTINCT FROM OLD.key AND NEW.value IS NOT DISTINCT FROM OLD.value THEN
  RETURN NULL;
 END IF;
 IF TG_OP<>'INSERT' AND OLD.key IN ('security.inbound-allowlist.v1','security.clientAccess.v1','maintenance.mode.v1') THEN
  PERFORM mw_authority_bump('protection','');
 ELSIF TG_OP<>'DELETE' AND NEW.key IN ('security.inbound-allowlist.v1','security.clientAccess.v1','maintenance.mode.v1') THEN
  PERFORM mw_authority_bump('protection','');
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_guard_config AFTER INSERT OR UPDATE OR DELETE ON settings FOR EACH ROW EXECUTE FUNCTION mw_authority_guard_config();
DO $bind_guard_schema$
BEGIN
 EXECUTE format('ALTER FUNCTION %I.mw_authority_guard_config() SET search_path TO pg_catalog,%I',current_schema(),current_schema());
END;
$bind_guard_schema$;
