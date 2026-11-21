-- Live source for typed canonical-subject ABSENCE proofs. No mail/source scans.
-- Each typed seed replaces this session's bounded current subject set. Foreign
-- keys tie derived PII to both its account and session, not an indefinite log.
CREATE TABLE authority_subject_anchor (
 account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
 session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
 account_incarnation TEXT NOT NULL CHECK(octet_length(account_incarnation)=32),
 session_incarnation TEXT NOT NULL CHECK(octet_length(session_incarnation)=32),
 scope_kind TEXT NOT NULL DEFAULT 'subject' CHECK(scope_kind='subject'),
 subject_id TEXT NOT NULL CHECK(octet_length(subject_id)<=4096),
 PRIMARY KEY(session_id,subject_id),
 FOREIGN KEY(scope_kind,subject_id) REFERENCES authority_epoch(scope_kind,scope_id) ON DELETE CASCADE
);
CREATE INDEX authority_subject_anchor_account ON authority_subject_anchor(account_id);
CREATE INDEX authority_subject_anchor_subject ON authority_subject_anchor(subject_id);
CREATE FUNCTION mw_authority_subject_anchor_gc() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 DELETE FROM authority_epoch WHERE scope_kind='subject' AND scope_id=OLD.subject_id AND NOT EXISTS(SELECT 1 FROM authority_subject_anchor a WHERE a.subject_id=OLD.subject_id)
 AND NOT EXISTS(SELECT 1 FROM accounts a WHERE a.id=OLD.subject_id)
 AND NOT EXISTS(SELECT 1 FROM setting_value v WHERE v.scope_kind IN ('user','admin_user') AND v.scope_id=OLD.subject_id)
 AND NOT EXISTS(SELECT 1 FROM personal_setting_value v WHERE v.owner_subject=OLD.subject_id)
 AND NOT EXISTS(SELECT 1 FROM user_group_member v WHERE v.account_id=OLD.subject_id)
 AND NOT EXISTS(SELECT 1 FROM feature_profile_assignment v WHERE v.scope_kind='user' AND v.scope_id=OLD.subject_id);
 RETURN NULL;
END; $$;
CREATE TRIGGER authority_subject_anchor_gc AFTER DELETE OR UPDATE ON authority_subject_anchor FOR EACH ROW EXECUTE FUNCTION mw_authority_subject_anchor_gc();
DO $bind_anchor_schema$
BEGIN
 EXECUTE format('ALTER FUNCTION %I.mw_authority_subject_anchor_gc() SET search_path TO pg_catalog,%I',current_schema(),current_schema());
END;
$bind_anchor_schema$;
