-- Settings revision scope (t34-e13 follow-up).
-- (A) Too much: 0078 bumped the global settings revision on every write to
-- session_principal, identity_principal and identity_binding, i.e. on every
-- sign-in, sign-out (session delete cascades, 0043), expiry and rotation, so
-- ordinary session churn ended every revision-keyed memo in every process.
-- The two consumers that depended on that identity half now validate this
-- session's identity themselves: the linked-accounts pass memo by the 0111
-- session/principal authority epochs, the primary policy witness by the
-- identity row values it reads.
DROP TRIGGER IF EXISTS settings_revision_identity_principal_insert;
DROP TRIGGER IF EXISTS settings_revision_identity_principal_update;
DROP TRIGGER IF EXISTS settings_revision_identity_principal_delete;
DROP TRIGGER IF EXISTS settings_revision_identity_binding_insert;
DROP TRIGGER IF EXISTS settings_revision_identity_binding_update;
DROP TRIGGER IF EXISTS settings_revision_identity_binding_delete;
DROP TRIGGER IF EXISTS settings_revision_session_principal_insert;
DROP TRIGGER IF EXISTS settings_revision_session_principal_update;
DROP TRIGGER IF EXISTS settings_revision_session_principal_delete;
-- (B) Too little: four administrator-written guard keys live in the key-value
-- settings table and bumped nothing, so they could not be memoised by
-- revision. Key-filtered: session watch writes sess-sec:v1:* rows on requests.
-- An update that keeps key and value bumps nothing.
CREATE TRIGGER settings_revision_guard_keys_insert AFTER INSERT ON settings
WHEN NEW.key IN ('v6:admin:bans','security.clientAccess.v1','maintenance.mode.v1','security.inbound-allowlist.v1') BEGIN
 UPDATE settings_revision SET n=n+1 WHERE id=1;
END;
CREATE TRIGGER settings_revision_guard_keys_update AFTER UPDATE ON settings
WHEN (NEW.key IS NOT OLD.key OR NEW.value IS NOT OLD.value)
 AND (OLD.key IN ('v6:admin:bans','security.clientAccess.v1','maintenance.mode.v1','security.inbound-allowlist.v1')
 OR NEW.key IN ('v6:admin:bans','security.clientAccess.v1','maintenance.mode.v1','security.inbound-allowlist.v1')) BEGIN
 UPDATE settings_revision SET n=n+1 WHERE id=1;
END;
CREATE TRIGGER settings_revision_guard_keys_delete AFTER DELETE ON settings
WHEN OLD.key IN ('v6:admin:bans','security.clientAccess.v1','maintenance.mode.v1','security.inbound-allowlist.v1') BEGIN
 UPDATE settings_revision SET n=n+1 WHERE id=1;
END;
