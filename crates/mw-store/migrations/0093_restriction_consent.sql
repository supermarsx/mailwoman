-- Art.18 restriction of processing and the purpose consent ledger.
--
-- account_restriction: one row per restriction an account was placed under. A
-- row is active while lifted_at is NULL and expires_at is NULL or in the future.
-- scopes_json lists the processing categories that are paused. A lifted row is
-- kept as the history.
--
-- consent_record: an append-only ledger of purpose decisions (granted, withdrawn,
-- declined). The current state of a purpose is the newest row for the account.
-- A row holds the purpose, the text version shown, the legal basis shown and the
-- source; no IP address or user agent.
CREATE TABLE account_restriction (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL,
    address TEXT NOT NULL,
    domain TEXT NOT NULL,
    scopes_json TEXT NOT NULL,
    mail_delivery TEXT NOT NULL,
    reason TEXT NOT NULL,
    source TEXT NOT NULL,
    request_id TEXT,
    created_by TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    expires_at BIGINT,
    lifted_at BIGINT,
    lifted_by TEXT,
    lift_reason TEXT
);
CREATE INDEX account_restriction_account ON account_restriction(account_id, lifted_at);
CREATE INDEX account_restriction_domain ON account_restriction(domain, lifted_at);

CREATE TABLE consent_record (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL,
    domain TEXT NOT NULL,
    purpose TEXT NOT NULL,
    decision TEXT NOT NULL,
    version BIGINT NOT NULL,
    basis TEXT NOT NULL,
    source TEXT NOT NULL,
    created_at BIGINT NOT NULL
);
CREATE INDEX consent_record_account ON consent_record(account_id, purpose, created_at);
CREATE INDEX consent_record_time ON consent_record(created_at);
