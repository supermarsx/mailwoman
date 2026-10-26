-- Bounded deployment-wide request budgets and adaptive observations.
-- Subject keys are purpose-separated HMACs, never raw account IDs or IPs.
CREATE TABLE request_protection_subject (
    subject_key TEXT PRIMARY KEY,
    subject_kind TEXT NOT NULL CHECK(subject_kind IN ('client','account','admin')),
    state_json TEXT NOT NULL,
    updated_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    revision TEXT NOT NULL
);
CREATE INDEX request_protection_expiry ON request_protection_subject(expires_at);
CREATE TABLE request_protection_event (
    id TEXT PRIMARY KEY,
    subject_key TEXT NOT NULL,
    subject_kind TEXT NOT NULL CHECK(subject_kind IN ('client','account','admin')),
    observed_at BIGINT NOT NULL,
    reason TEXT NOT NULL,
    action TEXT NOT NULL CHECK(action IN ('throttle','block','cleared')),
    enforced BIGINT NOT NULL CHECK(enforced IN (0,1)),
    retry_after_secs BIGINT NOT NULL,
    counts_json TEXT NOT NULL
);
CREATE INDEX request_protection_event_time ON request_protection_event(observed_at,id);
