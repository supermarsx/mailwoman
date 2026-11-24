-- Settings revision scope (t34-e13 follow-up); see the SQLite migration.
-- (A) Session churn no longer moves the global settings revision. On
-- PostgreSQL these statement-level triggers also fired for zero-row
-- statements and took the settings_revision row lock inside every sign-in.
DROP TRIGGER IF EXISTS settings_revision_identity_principal ON identity_principal;
DROP TRIGGER IF EXISTS settings_revision_identity_binding ON identity_binding;
DROP TRIGGER IF EXISTS settings_revision_session_principal ON session_principal;
-- (B) The four guard keys of the key-value settings table bump the revision.
-- Filtered in the function body (as 0114 does), not by a trigger WHEN, and
-- with a pinned search_path, so the publication schema validator accepts it.
-- An update that keeps key and value bumps nothing. TRUNCATE has no rows.
CREATE FUNCTION mw_settings_revision_guard_keys() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='TRUNCATE' THEN
  UPDATE settings_revision SET n=n+1 WHERE id=1;
  RETURN NULL;
 END IF;
 IF TG_OP='UPDATE' AND NEW.key IS NOT DISTINCT FROM OLD.key AND NEW.value IS NOT DISTINCT FROM OLD.value THEN
  RETURN NULL;
 END IF;
 IF (TG_OP<>'INSERT' AND OLD.key IN ('v6:admin:bans','security.clientAccess.v1','maintenance.mode.v1','security.inbound-allowlist.v1'))
  OR (TG_OP<>'DELETE' AND NEW.key IN ('v6:admin:bans','security.clientAccess.v1','maintenance.mode.v1','security.inbound-allowlist.v1')) THEN
  UPDATE settings_revision SET n=n+1 WHERE id=1;
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER settings_revision_guard_keys AFTER INSERT OR UPDATE OR DELETE ON settings FOR EACH ROW EXECUTE FUNCTION mw_settings_revision_guard_keys();
CREATE TRIGGER settings_revision_guard_keys_truncate AFTER TRUNCATE ON settings FOR EACH STATEMENT EXECUTE FUNCTION mw_settings_revision_guard_keys();
DO $bind_settings_revision_guard_keys$
BEGIN
 EXECUTE format('ALTER FUNCTION %I.mw_settings_revision_guard_keys() SET search_path TO pg_catalog,%I',current_schema(),current_schema());
END;
$bind_settings_revision_guard_keys$;
