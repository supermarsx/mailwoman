-- Durable logical drafts. Owners are verified realm subjects, never raw JMAP IDs.
-- Pending operations are never reclaimed automatically after a crash: APPEND
-- may have succeeded even when its acknowledgement or local commit was lost.
CREATE TABLE account_draft (
    owner TEXT NOT NULL,
    id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    revision TEXT,
    email_id TEXT,
    sealed_state BYTEA,
    sealed_summary BYTEA,
    updated_at BIGINT NOT NULL,
    operation_id TEXT NOT NULL,
    request_hash TEXT NOT NULL,
    operation_state TEXT NOT NULL CHECK(operation_state IN ('claimed','saved','uncertain','refused')),
    sealed_pending_state BYTEA,
    sealed_pending_summary BYTEA,
    cleanup_email_id TEXT,
    PRIMARY KEY(owner,id)
);
CREATE INDEX account_draft_owner_updated ON account_draft(owner,updated_at,id);
