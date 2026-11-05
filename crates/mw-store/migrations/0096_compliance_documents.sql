-- Compliance documents: published privacy policy versions with the facts they
-- were generated from, per-section policy text overrides, operator-written
-- processing activities and sub-processors, and the DPIA register with its
-- version history. Text the administrator wrote; no end-user data.
CREATE TABLE compliance_policy_version (
    id TEXT PRIMARY KEY NOT NULL,
    domain TEXT NOT NULL,
    version BIGINT NOT NULL,
    kind TEXT NOT NULL,
    facts_json TEXT NOT NULL,
    changes_json TEXT NOT NULL,
    created_by TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    UNIQUE(domain, version)
);

CREATE TABLE compliance_policy_override (
    id TEXT PRIMARY KEY NOT NULL,
    domain TEXT NOT NULL,
    lang TEXT NOT NULL,
    section TEXT NOT NULL,
    body TEXT NOT NULL,
    updated_by TEXT NOT NULL,
    updated_at BIGINT NOT NULL,
    UNIQUE(domain, lang, section)
);

CREATE TABLE compliance_record (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    name TEXT NOT NULL,
    body_json TEXT NOT NULL,
    updated_by TEXT NOT NULL,
    updated_at BIGINT NOT NULL
);
CREATE INDEX compliance_record_kind ON compliance_record(kind, name);

CREATE TABLE dpia (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    template TEXT NOT NULL,
    body_json TEXT NOT NULL,
    version BIGINT NOT NULL,
    review_at BIGINT,
    created_by TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    updated_by TEXT NOT NULL,
    updated_at BIGINT NOT NULL
);

CREATE TABLE dpia_version (
    id TEXT PRIMARY KEY NOT NULL,
    dpia_id TEXT NOT NULL,
    version BIGINT NOT NULL,
    status TEXT NOT NULL,
    body_json TEXT NOT NULL,
    changed_by TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    UNIQUE(dpia_id, version)
);
