-- Authority publication is distinct from mail metadata/content and legacy
-- settings_revision. These small tables and DML triggers do not backfill data.
-- Clock/activity touches are observed fresh, not versioned. No-op ciphertext
-- rewrites may conservatively dirty a scoped epoch; ciphertext is not decoded here.
CREATE TABLE authority_epoch (
 scope_kind TEXT NOT NULL CHECK(scope_kind IN ('policy','routing','directory','protection','domain','account','principal','session','watch','subject')),
 scope_id TEXT NOT NULL CHECK(octet_length(scope_id)<=4096),
 incarnation TEXT NOT NULL DEFAULT(replace(gen_random_uuid()::text,'-','')) CHECK(octet_length(incarnation)=32 AND incarnation ~ '^[0-9a-f]{32}$'),
 version BIGINT NOT NULL DEFAULT 1 CHECK(version>0),
 PRIMARY KEY(scope_kind,scope_id)
);
CREATE TABLE authority_publication_outbox (
 scope_kind TEXT NOT NULL,
 scope_id TEXT NOT NULL,
 incarnation TEXT NOT NULL,
 target_version BIGINT NOT NULL CHECK(target_version>0),
 notified_version BIGINT NOT NULL DEFAULT 0 CHECK(notified_version>=0 AND notified_version<=target_version),
 claim_token TEXT,
 claim_until BIGINT,
 PRIMARY KEY(scope_kind,scope_id),
 FOREIGN KEY(scope_kind,scope_id) REFERENCES authority_epoch(scope_kind,scope_id) ON DELETE CASCADE
);
CREATE FUNCTION mw_authority_epoch_monotonic() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF NEW.scope_kind IS DISTINCT FROM OLD.scope_kind OR NEW.scope_id IS DISTINCT FROM OLD.scope_id OR NEW.incarnation IS DISTINCT FROM OLD.incarnation OR NEW.version<=OLD.version THEN
 RAISE EXCEPTION 'authority epoch cannot reset or change incarnation';
 END IF;
 RETURN NEW;
END; $$;
CREATE TRIGGER authority_epoch_monotonic BEFORE UPDATE ON authority_epoch FOR EACH ROW EXECUTE FUNCTION mw_authority_epoch_monotonic();
CREATE FUNCTION mw_authority_epoch_outbox() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 INSERT INTO authority_publication_outbox(scope_kind,scope_id,incarnation,target_version) VALUES(NEW.scope_kind,NEW.scope_id,NEW.incarnation,NEW.version)
 ON CONFLICT(scope_kind,scope_id) DO UPDATE SET incarnation=excluded.incarnation,target_version=excluded.target_version,
 notified_version=CASE WHEN authority_publication_outbox.incarnation=excluded.incarnation THEN authority_publication_outbox.notified_version ELSE 0 END,
 claim_token=NULL,claim_until=NULL;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_epoch_outbox AFTER INSERT OR UPDATE ON authority_epoch FOR EACH ROW EXECUTE FUNCTION mw_authority_epoch_outbox();
-- The first consumer supports opaque ASCII scope IDs. Oversized/international
-- source IDs remain legal business data but are never silently truncated into
-- another scope. Missing epochs force complete original authentication.
CREATE FUNCTION mw_authority_bump(k TEXT,s TEXT) RETURNS void LANGUAGE plpgsql AS $$
BEGIN
 IF s IS NULL OR octet_length(s)>4096 OR s ~ '[^ -~]' OR (s='' AND k NOT IN ('policy','routing','directory','protection')) THEN RETURN; END IF;
 INSERT INTO authority_epoch(scope_kind,scope_id) VALUES(k,s)
 ON CONFLICT(scope_kind,scope_id) DO UPDATE SET version=authority_epoch.version+1;
END; $$;
CREATE FUNCTION mw_authority_setting_policy() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
 PERFORM mw_authority_bump('policy','');
 ELSIF TG_OP='UPDATE' THEN
 PERFORM mw_authority_bump('policy','');
 ELSIF TG_OP='DELETE' THEN
 PERFORM mw_authority_bump('policy','');
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_setting_policy AFTER INSERT OR UPDATE OR DELETE ON setting_policy FOR EACH ROW EXECUTE FUNCTION mw_authority_setting_policy();
CREATE FUNCTION mw_authority_setting_value() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP<>'INSERT' THEN
 PERFORM mw_authority_bump(CASE WHEN OLD.scope_kind='domain' THEN 'domain' WHEN OLD.scope_kind IN ('user','admin_user') THEN 'subject' ELSE 'policy' END,CASE WHEN OLD.scope_kind IN ('domain','user','admin_user') THEN OLD.scope_id ELSE '' END);
 END IF;
 IF TG_OP<>'DELETE' THEN
 PERFORM mw_authority_bump(CASE WHEN NEW.scope_kind='domain' THEN 'domain' WHEN NEW.scope_kind IN ('user','admin_user') THEN 'subject' ELSE 'policy' END,CASE WHEN NEW.scope_kind IN ('domain','user','admin_user') THEN NEW.scope_id ELSE '' END);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_setting_value AFTER INSERT OR UPDATE OR DELETE ON setting_value FOR EACH ROW EXECUTE FUNCTION mw_authority_setting_value();
CREATE FUNCTION mw_authority_personal_setting_value() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP<>'INSERT' THEN
 PERFORM mw_authority_bump('subject',OLD.owner_subject);
 END IF;
 IF TG_OP<>'DELETE' THEN
 PERFORM mw_authority_bump('subject',NEW.owner_subject);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_personal_setting_value AFTER INSERT OR UPDATE OR DELETE ON personal_setting_value FOR EACH ROW EXECUTE FUNCTION mw_authority_personal_setting_value();
CREATE FUNCTION mw_authority_user_group() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
 PERFORM mw_authority_bump('policy','');
 ELSIF TG_OP='UPDATE' THEN
 PERFORM mw_authority_bump('policy','');
 ELSIF TG_OP='DELETE' THEN
 PERFORM mw_authority_bump('policy','');
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_user_group AFTER INSERT OR UPDATE OR DELETE ON user_group FOR EACH ROW EXECUTE FUNCTION mw_authority_user_group();
CREATE FUNCTION mw_authority_user_group_member() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP<>'INSERT' THEN
 PERFORM mw_authority_bump('subject',OLD.account_id);
 END IF;
 IF TG_OP<>'DELETE' THEN
 PERFORM mw_authority_bump('subject',NEW.account_id);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_user_group_member AFTER INSERT OR UPDATE OR DELETE ON user_group_member FOR EACH ROW EXECUTE FUNCTION mw_authority_user_group_member();
CREATE FUNCTION mw_authority_feature_profile() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
 PERFORM mw_authority_bump('policy','');
 ELSIF TG_OP='UPDATE' THEN
 PERFORM mw_authority_bump('policy','');
 ELSIF TG_OP='DELETE' THEN
 PERFORM mw_authority_bump('policy','');
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_feature_profile AFTER INSERT OR UPDATE OR DELETE ON feature_profile FOR EACH ROW EXECUTE FUNCTION mw_authority_feature_profile();
CREATE FUNCTION mw_authority_feature_profile_assignment() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP<>'INSERT' THEN
 PERFORM mw_authority_bump(CASE WHEN OLD.scope_kind='domain' THEN 'domain' WHEN OLD.scope_kind='user' THEN 'subject' ELSE 'policy' END,CASE WHEN OLD.scope_kind IN ('domain','user') THEN OLD.scope_id ELSE '' END);
 END IF;
 IF TG_OP<>'DELETE' THEN
 PERFORM mw_authority_bump(CASE WHEN NEW.scope_kind='domain' THEN 'domain' WHEN NEW.scope_kind='user' THEN 'subject' ELSE 'policy' END,CASE WHEN NEW.scope_kind IN ('domain','user') THEN NEW.scope_id ELSE '' END);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_feature_profile_assignment AFTER INSERT OR UPDATE OR DELETE ON feature_profile_assignment FOR EACH ROW EXECUTE FUNCTION mw_authority_feature_profile_assignment();
CREATE FUNCTION mw_authority_feature_user_subject() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP<>'INSERT' THEN
 PERFORM mw_authority_bump('account',OLD.account_id);
 END IF;
 IF TG_OP<>'DELETE' THEN
 PERFORM mw_authority_bump('account',NEW.account_id);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_feature_user_subject AFTER INSERT OR UPDATE OR DELETE ON feature_user_subject FOR EACH ROW EXECUTE FUNCTION mw_authority_feature_user_subject();
CREATE FUNCTION mw_authority_domain_alias() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
 PERFORM mw_authority_bump('routing','');
 PERFORM mw_authority_bump('policy','');
 ELSIF TG_OP='UPDATE' THEN
 PERFORM mw_authority_bump('routing','');
 PERFORM mw_authority_bump('policy','');
 ELSIF TG_OP='DELETE' THEN
 PERFORM mw_authority_bump('routing','');
 PERFORM mw_authority_bump('policy','');
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_domain_alias AFTER INSERT OR UPDATE OR DELETE ON domain_alias FOR EACH ROW EXECUTE FUNCTION mw_authority_domain_alias();
CREATE FUNCTION mw_authority_domains() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
 PERFORM mw_authority_bump('domain',NEW.name);
 ELSIF TG_OP='UPDATE' THEN
 IF NOT (NEW.name IS DISTINCT FROM OLD.name OR NEW.upstream_json IS DISTINCT FROM OLD.upstream_json OR NEW.allowlist IS DISTINCT FROM OLD.allowlist OR NEW.blocklist IS DISTINCT FROM OLD.blocklist OR NEW.settings_json IS DISTINCT FROM OLD.settings_json OR NEW.enabled IS DISTINCT FROM OLD.enabled OR NEW.preferred IS DISTINCT FROM OLD.preferred) THEN RETURN NULL; END IF;
 PERFORM mw_authority_bump('domain',OLD.name);
 PERFORM mw_authority_bump('domain',NEW.name);
 ELSIF TG_OP='DELETE' THEN
 PERFORM mw_authority_bump('domain',OLD.name);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_domains AFTER INSERT OR UPDATE OR DELETE ON domains FOR EACH ROW EXECUTE FUNCTION mw_authority_domains();
CREATE FUNCTION mw_authority_accounts() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
 PERFORM mw_authority_bump('account',NEW.id);
 ELSIF TG_OP='UPDATE' THEN
 IF NOT (NEW.id IS DISTINCT FROM OLD.id OR NEW.kind IS DISTINCT FROM OLD.kind OR NEW.backend_kind IS DISTINCT FROM OLD.backend_kind OR NEW.host IS DISTINCT FROM OLD.host OR NEW.port IS DISTINCT FROM OLD.port OR NEW.tls IS DISTINCT FROM OLD.tls OR NEW.username IS DISTINCT FROM OLD.username OR NEW.sealed_creds IS DISTINCT FROM OLD.sealed_creds OR NEW.sync_policy_json IS DISTINCT FROM OLD.sync_policy_json) THEN RETURN NULL; END IF;
 PERFORM mw_authority_bump('account',OLD.id);
 PERFORM mw_authority_bump('account',NEW.id);
 ELSIF TG_OP='DELETE' THEN
 PERFORM mw_authority_bump('account',OLD.id);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_accounts AFTER INSERT OR UPDATE OR DELETE ON accounts FOR EACH ROW EXECUTE FUNCTION mw_authority_accounts();
CREATE FUNCTION mw_authority_sessions() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
 PERFORM mw_authority_bump('session',NEW.id);
 PERFORM mw_authority_bump('account',NEW.account_id);
 ELSIF TG_OP='UPDATE' THEN
 IF NOT (NEW.id IS DISTINCT FROM OLD.id OR NEW.account_id IS DISTINCT FROM OLD.account_id OR NEW.username IS DISTINCT FROM OLD.username OR NEW.jmap_url IS DISTINCT FROM OLD.jmap_url OR NEW.api_url IS DISTINCT FROM OLD.api_url OR NEW.sealed_creds IS DISTINCT FROM OLD.sealed_creds OR NEW.linked_internal IS DISTINCT FROM OLD.linked_internal OR NEW.authentication_method IS DISTINCT FROM OLD.authentication_method OR NEW.user_agent IS DISTINCT FROM OLD.user_agent OR NEW.ip IS DISTINCT FROM OLD.ip OR NEW.client_type IS DISTINCT FROM OLD.client_type OR NEW.device_label IS DISTINCT FROM OLD.device_label OR NEW.client_version IS DISTINCT FROM OLD.client_version) THEN RETURN NULL; END IF;
 PERFORM mw_authority_bump('session',OLD.id);
 PERFORM mw_authority_bump('session',NEW.id);
 PERFORM mw_authority_bump('account',OLD.account_id);
 PERFORM mw_authority_bump('account',NEW.account_id);
 ELSIF TG_OP='DELETE' THEN
 PERFORM mw_authority_bump('session',OLD.id);
 PERFORM mw_authority_bump('account',OLD.account_id);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_sessions AFTER INSERT OR UPDATE OR DELETE ON sessions FOR EACH ROW EXECUTE FUNCTION mw_authority_sessions();
CREATE FUNCTION mw_authority_session_principal() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
 PERFORM mw_authority_bump('session',NEW.session_id);
 ELSIF TG_OP='UPDATE' THEN
 PERFORM mw_authority_bump('session',OLD.session_id);
 PERFORM mw_authority_bump('session',NEW.session_id);
 ELSIF TG_OP='DELETE' THEN
 PERFORM mw_authority_bump('session',OLD.session_id);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_session_principal AFTER INSERT OR UPDATE OR DELETE ON session_principal FOR EACH ROW EXECUTE FUNCTION mw_authority_session_principal();
CREATE FUNCTION mw_authority_identity_principal() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
 PERFORM mw_authority_bump('principal',NEW.id);
 ELSIF TG_OP='UPDATE' THEN
 IF NOT (NEW.id IS DISTINCT FROM OLD.id OR NEW.state IS DISTINCT FROM OLD.state) THEN RETURN NULL; END IF;
 PERFORM mw_authority_bump('principal',OLD.id);
 PERFORM mw_authority_bump('principal',NEW.id);
 ELSIF TG_OP='DELETE' THEN
 PERFORM mw_authority_bump('principal',OLD.id);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_identity_principal AFTER INSERT OR UPDATE OR DELETE ON identity_principal FOR EACH ROW EXECUTE FUNCTION mw_authority_identity_principal();
CREATE FUNCTION mw_authority_identity_binding() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
 PERFORM mw_authority_bump('principal',NEW.principal_id);
 ELSIF TG_OP='UPDATE' THEN
 IF NOT (NEW.id IS DISTINCT FROM OLD.id OR NEW.principal_id IS DISTINCT FROM OLD.principal_id OR NEW.kind IS DISTINCT FROM OLD.kind OR NEW.authority IS DISTINCT FROM OLD.authority OR NEW.subject IS DISTINCT FROM OLD.subject OR NEW.retired_at IS DISTINCT FROM OLD.retired_at) THEN RETURN NULL; END IF;
 PERFORM mw_authority_bump('principal',OLD.principal_id);
 PERFORM mw_authority_bump('principal',NEW.principal_id);
 ELSIF TG_OP='DELETE' THEN
 PERFORM mw_authority_bump('principal',OLD.principal_id);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_identity_binding AFTER INSERT OR UPDATE OR DELETE ON identity_binding FOR EACH ROW EXECUTE FUNCTION mw_authority_identity_binding();
CREATE FUNCTION mw_authority_linked_account() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
 PERFORM mw_authority_bump('account',NEW.account_id);
 PERFORM mw_authority_bump('principal',NEW.owner_principal_id);
 PERFORM mw_authority_bump('principal',NEW.child_principal_id);
 ELSIF TG_OP='UPDATE' THEN
 PERFORM mw_authority_bump('account',OLD.account_id);
 PERFORM mw_authority_bump('account',NEW.account_id);
 PERFORM mw_authority_bump('principal',OLD.owner_principal_id);
 PERFORM mw_authority_bump('principal',NEW.owner_principal_id);
 PERFORM mw_authority_bump('principal',OLD.child_principal_id);
 PERFORM mw_authority_bump('principal',NEW.child_principal_id);
 ELSIF TG_OP='DELETE' THEN
 PERFORM mw_authority_bump('account',OLD.account_id);
 PERFORM mw_authority_bump('principal',OLD.owner_principal_id);
 PERFORM mw_authority_bump('principal',OLD.child_principal_id);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_linked_account AFTER INSERT OR UPDATE OR DELETE ON linked_account FOR EACH ROW EXECUTE FUNCTION mw_authority_linked_account();
CREATE FUNCTION mw_authority_linked_context() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
 PERFORM mw_authority_bump('session',NEW.session_id);
 ELSIF TG_OP='UPDATE' THEN
 PERFORM mw_authority_bump('session',OLD.session_id);
 PERFORM mw_authority_bump('session',NEW.session_id);
 ELSIF TG_OP='DELETE' THEN
 PERFORM mw_authority_bump('session',OLD.session_id);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_linked_context AFTER INSERT OR UPDATE OR DELETE ON linked_context FOR EACH ROW EXECUTE FUNCTION mw_authority_linked_context();
CREATE FUNCTION mw_authority_mail_user() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
 PERFORM mw_authority_bump('directory','');
 ELSIF TG_OP='UPDATE' THEN
 IF NOT (NEW.address IS DISTINCT FROM OLD.address OR NEW.domain IS DISTINCT FROM OLD.domain OR NEW.status IS DISTINCT FROM OLD.status OR NEW.deleted_at IS DISTINCT FROM OLD.deleted_at) THEN RETURN NULL; END IF;
 PERFORM mw_authority_bump('directory','');
 ELSIF TG_OP='DELETE' THEN
 PERFORM mw_authority_bump('directory','');
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_mail_user AFTER INSERT OR UPDATE OR DELETE ON mail_user FOR EACH ROW EXECUTE FUNCTION mw_authority_mail_user();
CREATE FUNCTION mw_authority_mail_user_account() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP<>'INSERT' THEN
 PERFORM mw_authority_bump('account',OLD.account_id);
 END IF;
 IF TG_OP<>'DELETE' THEN
 PERFORM mw_authority_bump('account',NEW.account_id);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_mail_user_account AFTER INSERT OR UPDATE OR DELETE ON mail_user_account FOR EACH ROW EXECUTE FUNCTION mw_authority_mail_user_account();
CREATE FUNCTION mw_authority_zeroaccess_accounts() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
 PERFORM mw_authority_bump('account',NEW.account_id);
 ELSIF TG_OP='UPDATE' THEN
 PERFORM mw_authority_bump('account',OLD.account_id);
 PERFORM mw_authority_bump('account',NEW.account_id);
 ELSIF TG_OP='DELETE' THEN
 PERFORM mw_authority_bump('account',OLD.account_id);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_zeroaccess_accounts AFTER INSERT OR UPDATE OR DELETE ON zeroaccess_accounts FOR EACH ROW EXECUTE FUNCTION mw_authority_zeroaccess_accounts();
CREATE FUNCTION mw_authority_account_restriction() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
 PERFORM mw_authority_bump('account',NEW.account_id);
 ELSIF TG_OP='UPDATE' THEN
 PERFORM mw_authority_bump('account',OLD.account_id);
 PERFORM mw_authority_bump('account',NEW.account_id);
 ELSIF TG_OP='DELETE' THEN
 PERFORM mw_authority_bump('account',OLD.account_id);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_account_restriction AFTER INSERT OR UPDATE OR DELETE ON account_restriction FOR EACH ROW EXECUTE FUNCTION mw_authority_account_restriction();
CREATE FUNCTION mw_authority_settings() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='UPDATE' THEN
  IF NEW.key IS NOT DISTINCT FROM OLD.key AND NEW.value IS NOT DISTINCT FROM OLD.value THEN RETURN NULL; END IF;
 END IF;
 IF TG_OP<>'INSERT' THEN
 IF OLD.key LIKE 'v6:admin:flags:%' THEN PERFORM mw_authority_bump('policy',''); END IF;
 IF OLD.key LIKE 'v6:account:names:%' THEN PERFORM mw_authority_bump('account',substr(OLD.key,18)); END IF;
 IF OLD.key LIKE 'sess-sec:v1:%' THEN PERFORM mw_authority_bump('watch',substr(OLD.key,13)); END IF;
 IF OLD.key LIKE 'sess-rot:v1:%' THEN PERFORM mw_authority_bump('watch',substr(OLD.key,13)); END IF;
 IF OLD.key='v6:admin:bans' THEN PERFORM mw_authority_bump('protection',''); END IF;
 END IF;
 IF TG_OP<>'DELETE' THEN
 IF NEW.key LIKE 'v6:admin:flags:%' THEN PERFORM mw_authority_bump('policy',''); END IF;
 IF NEW.key LIKE 'v6:account:names:%' THEN PERFORM mw_authority_bump('account',substr(NEW.key,18)); END IF;
 IF NEW.key LIKE 'sess-sec:v1:%' THEN PERFORM mw_authority_bump('watch',substr(NEW.key,13)); END IF;
 IF NEW.key LIKE 'sess-rot:v1:%' THEN PERFORM mw_authority_bump('watch',substr(NEW.key,13)); END IF;
 IF NEW.key='v6:admin:bans' THEN PERFORM mw_authority_bump('protection',''); END IF;
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_settings AFTER INSERT OR UPDATE OR DELETE ON settings FOR EACH ROW EXECUTE FUNCTION mw_authority_settings();
INSERT INTO authority_epoch(scope_kind,scope_id) VALUES('policy','');
INSERT INTO authority_epoch(scope_kind,scope_id) VALUES('routing','');
INSERT INTO authority_epoch(scope_kind,scope_id) VALUES('directory','');
INSERT INTO authority_epoch(scope_kind,scope_id) VALUES('protection','');
CREATE FUNCTION mw_authority_domains_topology() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='UPDATE' THEN
  IF NEW.name IS DISTINCT FROM OLD.name OR NEW.preferred IS DISTINCT FROM OLD.preferred THEN PERFORM mw_authority_bump('routing',''); END IF;
 ELSE
  PERFORM mw_authority_bump('routing','');
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_domains_topology AFTER INSERT OR UPDATE OR DELETE ON domains FOR EACH ROW EXECUTE FUNCTION mw_authority_domains_topology();

CREATE INDEX authority_outbox_pending ON authority_publication_outbox(scope_kind,scope_id) WHERE notified_version<target_version;

CREATE FUNCTION mw_authority_consent_record() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP='INSERT' THEN
 PERFORM mw_authority_bump('account',NEW.account_id);
 ELSIF TG_OP='UPDATE' THEN
 PERFORM mw_authority_bump('account',OLD.account_id);
 PERFORM mw_authority_bump('account',NEW.account_id);
 ELSIF TG_OP='DELETE' THEN
 PERFORM mw_authority_bump('account',OLD.account_id);
 END IF;
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_consent_record AFTER INSERT OR UPDATE OR DELETE ON consent_record FOR EACH ROW EXECUTE FUNCTION mw_authority_consent_record();
-- Bind all trigger function lookups to their actual creation schema. A raw
-- writer's different search_path must not redirect epoch/outbox updates.
DO $bind_authority_schema$
DECLARE owned_schema TEXT:=current_schema(); function_name TEXT;
BEGIN
 EXECUTE format('ALTER FUNCTION %I.mw_authority_bump(TEXT,TEXT) SET search_path TO pg_catalog,%I',owned_schema,owned_schema);
 FOREACH function_name IN ARRAY ARRAY['mw_authority_epoch_monotonic','mw_authority_epoch_outbox','mw_authority_setting_policy','mw_authority_setting_value','mw_authority_personal_setting_value','mw_authority_user_group','mw_authority_user_group_member','mw_authority_feature_profile','mw_authority_feature_profile_assignment','mw_authority_feature_user_subject','mw_authority_domain_alias','mw_authority_domains','mw_authority_accounts','mw_authority_sessions','mw_authority_session_principal','mw_authority_identity_principal','mw_authority_identity_binding','mw_authority_linked_account','mw_authority_linked_context','mw_authority_mail_user','mw_authority_mail_user_account','mw_authority_zeroaccess_accounts','mw_authority_account_restriction','mw_authority_consent_record','mw_authority_settings','mw_authority_domains_topology'] LOOP
  EXECUTE format('ALTER FUNCTION %I.%I() SET search_path TO pg_catalog,%I',owned_schema,function_name,owned_schema);
 END LOOP;
END;
$bind_authority_schema$;
