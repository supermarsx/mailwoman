-- t34 session watch: the alert preferences that session binding and theft
-- detection read on a request are resolved for the account's sign-in address
-- (`signin_identity.address`: its domain and address are the settings subject).
-- That address is therefore a settings-resolution input, like the 0078 tables:
-- bump the settings revision when the account -> address mapping changes, so a
-- value cached under an unchanged revision (bounded guard snapshot, 0115 change
-- log) can never outlive a changed address. A sign-in that only refreshes
-- seen_at/event_id for the same address changes nothing and bumps nothing.
CREATE TRIGGER settings_revision_signin_identity_insert AFTER INSERT ON signin_identity BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_signin_identity_update AFTER UPDATE OF account_id,address ON signin_identity WHEN OLD.account_id IS NOT NEW.account_id OR OLD.address IS NOT NEW.address BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_signin_identity_delete AFTER DELETE ON signin_identity BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
