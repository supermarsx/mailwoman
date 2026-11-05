-- Personal-data breach register: one row per incident, a per-incident trail and
-- the delivery log of data-subject notices. A row keeps the filterable columns
-- (status, severity, clock); the rest (description, categories, assessment,
-- timeline, evidence, authority notices, delay reasons, playbook checklist) is
-- in body_json, changed under the rev counter. aware_at is empty for a draft
-- whose clock has not started; deadline_at is aware_at plus the notice window.
CREATE TABLE breach_incident (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    domain TEXT NOT NULL,
    status TEXT NOT NULL,
    severity TEXT NOT NULL,
    source TEXT NOT NULL,
    detected_at BIGINT NOT NULL,
    aware_at BIGINT,
    deadline_at BIGINT,
    assignee TEXT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    rev BIGINT NOT NULL,
    body_json TEXT NOT NULL
);
CREATE INDEX breach_incident_status ON breach_incident(status, deadline_at);
CREATE INDEX breach_incident_domain ON breach_incident(domain, status);
CREATE INDEX breach_incident_detected ON breach_incident(detected_at);

CREATE TABLE breach_event (
    id TEXT PRIMARY KEY NOT NULL,
    incident_id TEXT NOT NULL,
    ts BIGINT NOT NULL,
    actor TEXT NOT NULL,
    kind TEXT NOT NULL,
    detail_json TEXT NOT NULL
);
CREATE INDEX breach_event_incident ON breach_event(incident_id, ts);

CREATE TABLE breach_delivery (
    id TEXT PRIMARY KEY NOT NULL,
    incident_id TEXT NOT NULL,
    notice_id TEXT NOT NULL,
    recipient TEXT NOT NULL,
    channel TEXT NOT NULL,
    status TEXT NOT NULL,
    detail TEXT NOT NULL,
    ts BIGINT NOT NULL
);
CREATE INDEX breach_delivery_incident ON breach_delivery(incident_id, ts);
