-- Extend the existing bounded protection tables with header-claim dimensions.
-- Preserve all existing subjects, active deadlines, revisions and audit events.
CREATE TABLE request_protection_subject_ua (
    subject_key TEXT NOT NULL PRIMARY KEY,
    subject_kind TEXT NOT NULL CHECK(subject_kind IN ('client','account','admin','ua-missing','ua-uncommon','ua-automation','ua-search','ua-ai-training','ua-ai-search','ua-ai-user','ua-custom')),
    state_json TEXT NOT NULL,
    updated_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    revision TEXT NOT NULL
);
INSERT INTO request_protection_subject_ua SELECT subject_key,subject_kind,state_json,updated_at,expires_at,revision FROM request_protection_subject;
DROP TABLE request_protection_subject;
ALTER TABLE request_protection_subject_ua RENAME TO request_protection_subject;
CREATE INDEX request_protection_expiry ON request_protection_subject(expires_at);
CREATE TABLE request_protection_event_ua (
    id TEXT NOT NULL PRIMARY KEY,
    subject_key TEXT NOT NULL,
    subject_kind TEXT NOT NULL CHECK(subject_kind IN ('client','account','admin','ua-missing','ua-uncommon','ua-automation','ua-search','ua-ai-training','ua-ai-search','ua-ai-user','ua-custom')),
    observed_at BIGINT NOT NULL,
    reason TEXT NOT NULL,
    action TEXT NOT NULL CHECK(action IN ('throttle','block','cleared')),
    enforced BIGINT NOT NULL CHECK(enforced IN (0,1)),
    retry_after_secs BIGINT NOT NULL,
    counts_json TEXT NOT NULL
);
INSERT INTO request_protection_event_ua SELECT id,subject_key,subject_kind,observed_at,reason,action,enforced,retry_after_secs,counts_json FROM request_protection_event;
DROP TABLE request_protection_event;
ALTER TABLE request_protection_event_ua RENAME TO request_protection_event;
CREATE INDEX request_protection_event_time ON request_protection_event(observed_at,id);

