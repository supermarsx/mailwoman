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
CREATE TRIGGER settings_revision_setting_policy_insert AFTER INSERT ON setting_policy BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_setting_policy_update AFTER UPDATE ON setting_policy BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_setting_policy_delete AFTER DELETE ON setting_policy BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_setting_value_insert AFTER INSERT ON setting_value BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_setting_value_update AFTER UPDATE ON setting_value BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_setting_value_delete AFTER DELETE ON setting_value BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_personal_setting_value_insert AFTER INSERT ON personal_setting_value BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_personal_setting_value_update AFTER UPDATE ON personal_setting_value BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_personal_setting_value_delete AFTER DELETE ON personal_setting_value BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_user_group_insert AFTER INSERT ON user_group BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_user_group_update AFTER UPDATE ON user_group BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_user_group_delete AFTER DELETE ON user_group BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_user_group_member_insert AFTER INSERT ON user_group_member BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_user_group_member_update AFTER UPDATE ON user_group_member BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_user_group_member_delete AFTER DELETE ON user_group_member BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_feature_profile_insert AFTER INSERT ON feature_profile BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_feature_profile_update AFTER UPDATE ON feature_profile BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_feature_profile_delete AFTER DELETE ON feature_profile BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_feature_profile_assignment_insert AFTER INSERT ON feature_profile_assignment BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_feature_profile_assignment_update AFTER UPDATE ON feature_profile_assignment BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_feature_profile_assignment_delete AFTER DELETE ON feature_profile_assignment BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_domain_alias_insert AFTER INSERT ON domain_alias BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_domain_alias_update AFTER UPDATE ON domain_alias BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_domain_alias_delete AFTER DELETE ON domain_alias BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;

-- Linked-policy passes also depend on the verified session identity.
CREATE TRIGGER settings_revision_identity_principal_insert AFTER INSERT ON identity_principal BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_identity_principal_update AFTER UPDATE ON identity_principal BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_identity_principal_delete AFTER DELETE ON identity_principal BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_identity_binding_insert AFTER INSERT ON identity_binding BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_identity_binding_update AFTER UPDATE ON identity_binding BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_identity_binding_delete AFTER DELETE ON identity_binding BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_session_principal_insert AFTER INSERT ON session_principal BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_session_principal_update AFTER UPDATE ON session_principal BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
CREATE TRIGGER settings_revision_session_principal_delete AFTER DELETE ON session_principal BEGIN UPDATE settings_revision SET n=n+1 WHERE id=1; END;
