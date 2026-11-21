-- Integrate only after 0113 mail-data publication. No source/backfill scan.
-- These are rare shared configuration changes, not per-request observations.
CREATE TRIGGER authority_guard_config_insert AFTER INSERT ON settings
WHEN NEW.key IN ('security.inbound-allowlist.v1','security.clientAccess.v1','maintenance.mode.v1') BEGIN
 INSERT INTO authority_epoch(scope_kind,scope_id) VALUES('protection','')
 ON CONFLICT(scope_kind,scope_id) DO UPDATE SET version=authority_epoch.version+1;
END;
CREATE TRIGGER authority_guard_config_update AFTER UPDATE ON settings
WHEN (NEW.key IS NOT OLD.key OR NEW.value IS NOT OLD.value)
 AND (OLD.key IN ('security.inbound-allowlist.v1','security.clientAccess.v1','maintenance.mode.v1')
 OR NEW.key IN ('security.inbound-allowlist.v1','security.clientAccess.v1','maintenance.mode.v1')) BEGIN
 INSERT INTO authority_epoch(scope_kind,scope_id) VALUES('protection','')
 ON CONFLICT(scope_kind,scope_id) DO UPDATE SET version=authority_epoch.version+1;
END;
CREATE TRIGGER authority_guard_config_delete AFTER DELETE ON settings
WHEN OLD.key IN ('security.inbound-allowlist.v1','security.clientAccess.v1','maintenance.mode.v1') BEGIN
 INSERT INTO authority_epoch(scope_kind,scope_id) VALUES('protection','')
 ON CONFLICT(scope_kind,scope_id) DO UPDATE SET version=authority_epoch.version+1;
END;
