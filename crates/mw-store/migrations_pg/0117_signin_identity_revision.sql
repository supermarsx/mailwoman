-- t34 session watch: see the SQLite migration. The account -> sign-in address
-- mapping is a settings-resolution input (the alert-preference subject), so a
-- change to it bumps the settings revision. Row-level, so an upsert that only
-- refreshes seen_at/event_id for the same address bumps nothing; TRUNCATE has
-- no row trigger and bumps at statement level. Parity: the bounded guard
-- snapshot is inert on PostgreSQL, but the 0078 memos compare this revision.
CREATE TRIGGER settings_revision_signin_identity_insert AFTER INSERT ON signin_identity FOR EACH ROW EXECUTE FUNCTION settings_revision_bump();
CREATE TRIGGER settings_revision_signin_identity_update AFTER UPDATE OF account_id,address ON signin_identity FOR EACH ROW WHEN (OLD.account_id IS DISTINCT FROM NEW.account_id OR OLD.address IS DISTINCT FROM NEW.address) EXECUTE FUNCTION settings_revision_bump();
CREATE TRIGGER settings_revision_signin_identity_delete AFTER DELETE ON signin_identity FOR EACH ROW EXECUTE FUNCTION settings_revision_bump();
CREATE TRIGGER settings_revision_signin_identity_truncate AFTER TRUNCATE ON signin_identity FOR EACH STATEMENT EXECUTE FUNCTION settings_revision_bump();
