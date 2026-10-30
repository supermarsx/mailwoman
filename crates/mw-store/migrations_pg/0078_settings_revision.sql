-- t87/t99: a counter that changes in the same transaction as any write to a
-- table the settings resolution chain reads (policies, scoped values, personal
-- values, groups, profiles and their assignments, domain aliases). In-memory
-- memos of resolved settings compare it on every hit, so a change made by this
-- process, another process on the same database, or a raw SQL write is seen by
-- the next read, exactly at commit. One row; missing row = memos disabled.
CREATE TABLE settings_revision (
 id INTEGER PRIMARY KEY NOT NULL CHECK(id=1),
 n BIGINT NOT NULL
);
INSERT INTO settings_revision(id,n) VALUES(1,0);
CREATE FUNCTION settings_revision_bump() RETURNS trigger AS $$
BEGIN
 UPDATE settings_revision SET n=n+1 WHERE id=1;
 RETURN NULL;
END
$$ LANGUAGE plpgsql;
CREATE TRIGGER settings_revision_setting_policy AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON setting_policy FOR EACH STATEMENT EXECUTE FUNCTION settings_revision_bump();
CREATE TRIGGER settings_revision_setting_value AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON setting_value FOR EACH STATEMENT EXECUTE FUNCTION settings_revision_bump();
CREATE TRIGGER settings_revision_personal_setting_value AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON personal_setting_value FOR EACH STATEMENT EXECUTE FUNCTION settings_revision_bump();
CREATE TRIGGER settings_revision_user_group AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON user_group FOR EACH STATEMENT EXECUTE FUNCTION settings_revision_bump();
CREATE TRIGGER settings_revision_user_group_member AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON user_group_member FOR EACH STATEMENT EXECUTE FUNCTION settings_revision_bump();
CREATE TRIGGER settings_revision_feature_profile AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON feature_profile FOR EACH STATEMENT EXECUTE FUNCTION settings_revision_bump();
CREATE TRIGGER settings_revision_feature_profile_assignment AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON feature_profile_assignment FOR EACH STATEMENT EXECUTE FUNCTION settings_revision_bump();
CREATE TRIGGER settings_revision_domain_alias AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON domain_alias FOR EACH STATEMENT EXECUTE FUNCTION settings_revision_bump();

-- Linked-policy passes also depend on the verified session identity.
CREATE TRIGGER settings_revision_identity_principal AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON identity_principal FOR EACH STATEMENT EXECUTE FUNCTION settings_revision_bump();
CREATE TRIGGER settings_revision_identity_binding AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON identity_binding FOR EACH STATEMENT EXECUTE FUNCTION settings_revision_bump();
CREATE TRIGGER settings_revision_session_principal AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON session_principal FOR EACH STATEMENT EXECUTE FUNCTION settings_revision_bump();
